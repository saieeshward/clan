// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Pack a new `.clan` from agent output — spec §10, §11, §12.
//!
//! Accepts a [`PackInput`] (the validated agent JSON + the parent [`ClanFile`])
//! and produces the raw bytes of the next-generation `.clan` archive.

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::compress::{compress_chain, CompressionConfig, Compressor};
use crate::container::{ClanBuilder, ClanFile};
use crate::decision::{Decision, DecisionChain};
use crate::error::{Error, Result};
use crate::manifest::{FileEntry, Lineage, CLAN_VERSION, CLAN_VERSION_MINOR};

/// Agent output decoded from JSON.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub mode: String,
    /// Structured data fields to merge into `shared/data.yaml`.
    pub structured: Value,
    /// For `designed` mode: visual directives.
    pub design: Option<Value>,
    /// For `full-html` or `patch-html` mode.
    pub human: Option<HumanPayload>,
    /// Decision entry written by this agent (optional; auto-generated if absent).
    pub decision: Option<DecisionEntry>,
}

#[derive(Debug, Clone)]
pub struct HumanPayload {
    pub html: String,
    pub css: Option<String>,
    pub assets: HashMap<String, Vec<u8>>,
    pub patch_selector: Option<String>,
    pub patch_action: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DecisionEntry {
    pub agent_name: String,
    pub action: String,
    pub rationale: String,
    pub pinned: bool,
    /// Exact keys this decision changed. `Some` records them verbatim (F15:
    /// the merge-patch keys); `None` lets `pack` derive them from the
    /// structured payload (used by `pack_html`, whose payload IS the delta).
    pub fields_changed: Option<Vec<String>>,
}

impl AgentOutput {
    /// Parse and basic-validate agent JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(json)
            .map_err(|e| Error::OutputRejected(format!("invalid JSON: {e}")))?;

        let mode = v["mode"]
            .as_str()
            .ok_or_else(|| Error::OutputRejected("missing field: mode".into()))?
            .to_string();

        match mode.as_str() {
            "data-update" | "designed" | "full-html" | "patch-html" => {}
            other => return Err(Error::UnsupportedMode(other.to_string())),
        }

        let structured = v["structured"].clone();
        if structured.is_null() {
            return Err(Error::OutputRejected("missing field: structured".into()));
        }

        let design = if mode == "designed" {
            Some(v["design"].clone())
        } else {
            None
        };

        let human = if mode == "full-html" || mode == "patch-html" {
            let html = v["human"]["html"]
                .as_str()
                .ok_or_else(|| Error::OutputRejected(format!("{} missing human.html", mode)))?
                .to_string();
            let css = v["human"]["css"].as_str().map(|s| s.to_string());
            let assets = if let Some(obj) = v["human"]["assets"].as_object() {
                obj.iter()
                    .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.as_bytes().to_vec())))
                    .collect()
            } else {
                HashMap::new()
            };
            let patch_selector = v["human"]["patch_selector"].as_str().map(|s| s.to_string());
            let patch_action = v["human"]["patch_action"].as_str().map(|s| s.to_string());
            Some(HumanPayload { html, css, assets, patch_selector, patch_action })
        } else {
            None
        };

        let decision = if !v["decision"].is_null() {
            Some(DecisionEntry {
                agent_name: v["decision"]["agent"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                action: v["decision"]["action"].as_str().unwrap_or("").to_string(),
                rationale: v["decision"]["rationale"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                pinned: v["decision"]["pinned"].as_bool().unwrap_or(false),
                fields_changed: None,
            })
        } else {
            None
        };

        Ok(AgentOutput {
            mode,
            structured,
            design,
            human,
            decision,
        })
    }

    /// Validate the output against the declared JSON Schema.
    pub fn validate_schema(&self, schema_json: &str) -> Result<()> {
        let schema: Value = serde_json::from_str(schema_json)
            .map_err(|e| Error::Schema(format!("invalid schema: {e}")))?;
        let compiled = jsonschema::validator_for(&schema)
            .map_err(|e| Error::Schema(format!("could not compile schema: {e}")))?;
            
        let mut payload_to_validate = self.structured.clone();
        if let Some(obj) = payload_to_validate.as_object_mut() {
            obj.remove("$schema");
        }
            
        let errors: Vec<String> = compiled
            .iter_errors(&payload_to_validate)
            .map(|e| e.to_string())
            .collect();
        if !errors.is_empty() {
            return Err(Error::OutputRejected(errors.join("; ")));
        }
        Ok(())
    }
}

/// Options for the pack operation.
#[derive(Debug, Default)]
pub struct PackOptions {
    pub compression: CompressionConfig,
    /// Optional delta description for the lineage block.
    pub delta: Option<String>,
    /// Optional output path hint (informational only).
    pub output_path: Option<String>,
    /// Optional schema override.
    pub schema_override: Option<String>,
    /// Extra entries to write into the new archive, taking precedence over
    /// entries carried over from the parent.
    pub extra_entries: Vec<(String, Vec<u8>)>,
}

/// Teaching guard (spec §27.2): on a forked branch file only the branch
/// namespace may be written, so shared/human-mutating operations are
/// rejected with the correct alternative named in the message.
fn reject_if_forked(parent: &ClanFile, attempted: &str) -> Result<()> {
    if let Some(fork) = &parent.manifest().fork {
        return Err(Error::NamespaceViolation(format!(
            "this file is forked for agent '{agent}' — {attempted} is not allowed during a parallel interval.\n\
             next: write your data with `clan patch-data --namespace` (routes to {ns}data.yaml)\n\
             next: record decisions with `clan patch-decision` (auto-routed to {ns}decisions.yaml)\n\
             next: when all branches are done, join them with `clan merge`",
            agent = fork.agent_id,
            ns = fork.namespace,
        )));
    }
    Ok(())
}

/// Pack a new `.clan` archive from a parent file and agent output.
pub fn pack(
    parent: &ClanFile,
    output: AgentOutput,
    opts: PackOptions,
    compressor: Option<&Compressor>,
) -> Result<Vec<u8>> {
    reject_if_forked(parent, "packing a new generation (writes shared/data.yaml)")?;
    let now = Utc::now().to_rfc3339();
    let parent_manifest = parent.manifest();

    // --- Merge structured data into shared/data.yaml ---
    let existing_data_bytes = parent.read_entry("shared/data.yaml")?;
    let mut data: Value = serde_yaml::from_slice(&existing_data_bytes)?;
    
    if output.mode == "data-update" {
        // patch_data and patch_schema pass the fully merged state
        data = output.structured.clone();
    } else {
        // pack_html passes a partial update delta, use RFC 7396 merge
        merge_json(&mut data, &output.structured);
    }
    
    // --- ENFORCE SCHEMA VALIDATION ---
    // We validate the FULLY MERGED data against the schema, not just the partial update.
    let schema_json = match &opts.schema_override {
        Some(s) => Ok(s.clone()),
        None => parent.read_entry_string("agent/output-schema.json"),
    };
    
    if let Ok(schema_str) = schema_json {
        let mut validation_output = output.clone();
        validation_output.structured = data.clone();
        validation_output.validate_schema(&schema_str)?;
    }
    
    let new_data_yaml = serde_yaml::to_string(&data)?.into_bytes();

    // --- Update decision chain ---
    let chain_bytes = parent.read_entry("agent/decision-chain.yaml")?;
    let mut chain = DecisionChain::from_yaml(&chain_bytes)?;

    // Only record a decision when the agent actually supplied one. A bare
    // data/HTML update with no decision must NOT spawn an "unknown-agent /
    // processed document" placeholder — that was pure provenance noise (F1).
    // `clan patch-decision` is the path for recording a decision.
    if let Some(d) = output.decision {
        // F15: honour an explicit fields_changed (the exact merge-patch keys);
        // otherwise derive from the structured payload, which for pack_html is
        // itself the delta.
        let fields_changed = d.fields_changed.unwrap_or_else(|| {
            output.structured
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default()
        });
        chain.prepend(Decision {
            agent: d.agent_name,
            version: None,
            action: d.action,
            rationale: d.rationale,
            timestamp: now.clone(),
            fields_changed,
            pinned: d.pinned,
            trace_ref: None,
        });
    }

    // Fold any superseded human edits into the chain before they are dropped
    // (F5). full-html replacement discards human/ (its data-adf-id targets no
    // longer exist), but the human's intent is provenance and must survive
    // per the §22 preservation rule. Recorded as one attributed entry.
    if output.mode == "full-html" {
        if let Ok(patch_bytes) = parent.read_entry("human/patches.yaml") {
            if let Ok(patches) = crate::patch::Patches::from_yaml(&patch_bytes) {
                if !patches.patches.is_empty() {
                    let summary = patches
                        .patches
                        .iter()
                        .map(|p| format!("{}: {}", p.id, p.content.trim()))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    chain.prepend(Decision {
                        agent: "human".into(),
                        version: None,
                        action: format!(
                            "{} human edit(s) superseded by full-html replacement",
                            patches.patches.len()
                        ),
                        rationale: summary,
                        timestamp: now.clone(),
                        fields_changed: Vec::new(),
                        pinned: true,
                        trace_ref: None,
                    });
                }
            }
        }
    }

    compress_chain(&mut chain, &opts.compression, compressor);
    let new_chain_yaml = chain.to_yaml()?;

    // --- Build new manifest ---
    let delta = opts.delta.unwrap_or_else(|| {
        let fields: Vec<String> = output
            .structured
            .as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        if fields.is_empty() {
            "agent processing pass".to_string()
        } else {
            format!("updated fields: {}", fields.join(", "))
        }
    });

    let mut new_manifest = parent_manifest.clone();
    new_manifest.id = Uuid::new_v4().to_string();
    new_manifest.updated_at = now.clone();
    new_manifest.clan_version = CLAN_VERSION;
    new_manifest.clan_version_minor = CLAN_VERSION_MINOR;
    new_manifest.lineage = Some(Lineage {
        parent_id: parent_manifest.id.clone(),
        parent_uri: opts
            .output_path
            .clone()
            .unwrap_or_else(|| format!("file:///unknown/{}.clan", parent_manifest.id)),
        parent_sha256: Some(parent.sha256()),
        delta,
        parents: Vec::new(),
        merge: false,
    });

    // View bookkeeping (spec §23): view-producing modes refresh the view and
    // mark it agent-authored (NOT safe to clobber with `clan render`); a
    // data-only update leaves an existing view stale (F2).
    match output.mode.as_str() {
        "full-html" | "patch-html" | "designed" => {
            if let Some(view) = &mut new_manifest.view {
                view.present = true;
                view.stale = false;
                view.source = Some("agent".into());
            }
        }
        _ => {
            if let Some(view) = &mut new_manifest.view {
                if view.present {
                    view.stale = true;
                }
            }
        }
    }

    // Rebuild the files registry, keeping everything from the parent.
    // full-html replaces the *view* files (index.html/.txt/styles.css) but
    // KEEPS human/assets/ — assets the new HTML references must survive the
    // replacement (F10); new payload assets overwrite by path below.
    if output.mode == "full-html" {
        new_manifest
            .files
            .retain(|f| !(f.role.starts_with("human-") && f.role != "human-asset"));
    }

    // --- Assemble builder ---
    let mut builder = ClanBuilder::new(new_manifest);

    // Carry over all parent entries unchanged, except ones we're replacing.
    // Single-pass read: one ZipArchive instantiation for the whole archive.
    for (path, bytes) in parent.read_all_entries()? {
        if path == "manifest.yaml" { continue; }
        if path == "shared/data.yaml" { continue; }
        if path == "agent/decision-chain.yaml" { continue; }
        if opts.schema_override.is_some() && path == "agent/output-schema.json" { continue; }
        // full-html drops the prior view + patches, but carries human/assets/
        // forward (F5 folded the patches into the chain above; F10 keeps assets).
        if output.mode == "full-html"
            && path.starts_with("human/")
            && !path.starts_with("human/assets/")
        {
            continue;
        }
        // For patch-html, we keep existing human assets and index.html, we'll rewrite index.html below
        builder.add_entry(path, bytes);
    }

    if let Some(new_schema) = &opts.schema_override {
        builder.add_entry("agent/output-schema.json", new_schema.clone().into_bytes());
    }

    // Updated entries.
    builder.add_entry("shared/data.yaml", new_data_yaml);
    builder.add_entry("agent/decision-chain.yaml", new_chain_yaml);

    // Handle output modes.
    match output.mode.as_str() {
        "full-html" => {
            if let Some(h) = output.human {
                builder.add_entry("human/index.html", h.html.into_bytes());
                if let Some(css) = h.css {
                    builder.add_entry("human/styles.css", css.into_bytes());
                }
                for (name, content) in h.assets {
                    let path = format!("human/assets/{name}");
                    builder.add_entry(path.clone(), content);
                    // Register new assets so they carry sha256 + survive the
                    // next full-html replacement (F10).
                    let m = builder.manifest_mut();
                    if !m.files.iter().any(|f| f.path == path) {
                        m.files.push(FileEntry {
                            id: format!("human-asset-{}", name.replace(['/', '.'], "-")),
                            path,
                            role: "human-asset".into(),
                            content_type: "application/octet-stream".into(),
                            priority: None,
                            sha256: None,
                        });
                    }
                }
                let m = builder.manifest_mut();
                if !m.files.iter().any(|f| f.role == "human-view") {
                    m.files.push(FileEntry {
                        id: "human-view".into(),
                        path: "human/index.html".into(),
                        role: "human-view".into(),
                        content_type: "text/html".into(),
                        priority: Some(1),
                        sha256: None,
                    });
                }
            }
        }
        "patch-html" => {
            if let Some(h) = output.human {
                let existing = parent.read_entry_string("human/index.html").unwrap_or_else(|_| "".to_string());
                let new_html = apply_html_patch(&existing, &h)?;
                builder.add_entry("human/index.html", new_html.into_bytes());
            }
        }
        "designed" => {
            // For v1 the SDK re-renders the existing template with updated
            // data bindings. Designed-mode template rendering is deferred to
            // Phase 3. The data update is already applied above.
        }
        "data-update" => {
            // Data already merged above; HTML re-renders via data bindings at
            // serve time. Nothing extra to do here.
        }
        _ => {}
    }

    for (path, bytes) in opts.extra_entries {
        builder.add_entry(path, bytes);
    }

    builder.build()
}

/// Pack a new `.clan` directly from an HTML file (or stdin) — no JSON encoding needed.
///
/// This is the token-efficient path for `full-html` agents: instead of JSON-encoding
/// a 12 KB HTML string into ~62 KB of escaped JSON, the agent writes the HTML to a
/// file and the operator calls `clan pack-html`. Token cost: zero for the HTML encoding step.
///
/// Accepts an optional YAML-frontmatter block at the very start of the file:
/// ```text
/// ---
/// structured:
///   title: "My Report"
///   key_finding: "example"
/// decision:
///   agent: "agent3"
///   action: "produced final design"
///   rationale: "combined prior agent work into polished output"
///   pinned: true
/// ---
/// <!DOCTYPE html>
/// ...
/// ```
/// If no frontmatter is present the HTML is packed as-is with empty structured data.
/// True when the HTML's YAML frontmatter already carries a `decision:` block.
/// Lets the CLI enforce attribution (F15) without re-implementing the parser.
pub fn frontmatter_has_decision(raw_html: &str) -> bool {
    parse_html_frontmatter(raw_html).2.is_some()
}

pub fn pack_html(
    parent: &ClanFile,
    raw_html: &str,
    assets_dir_files: Option<HashMap<String, Vec<u8>>>,
    schema_override: Option<String>,
    delta: Option<String>,
    compressor: Option<&Compressor>,
) -> Result<Vec<u8>> {
    pack_html_with(parent, raw_html, assets_dir_files, schema_override, delta, None, compressor)
}

/// Like [`pack_html`] but a `decision_override`, when present, supplies the
/// decision instead of (or in the absence of) one in the frontmatter (F15:
/// attribution recorded from CLI flags).
#[allow(clippy::too_many_arguments)]
pub fn pack_html_with(
    parent: &ClanFile,
    raw_html: &str,
    assets_dir_files: Option<HashMap<String, Vec<u8>>>,
    schema_override: Option<String>,
    delta: Option<String>,
    decision_override: Option<DecisionEntry>,
    compressor: Option<&Compressor>,
) -> Result<Vec<u8>> {
    // Parse optional YAML frontmatter.
    let (parsed_mode, structured, decision_entry, patch_selector, patch_action, context_handoff, html_body) = parse_html_frontmatter(raw_html);
    let mode = parsed_mode.unwrap_or_else(|| "full-html".to_string());

    let output = AgentOutput {
        mode,
        structured,
        design: None,
        human: Some(HumanPayload {
            html: html_body,
            css: None,
            assets: assets_dir_files.unwrap_or_default(),
            patch_selector,
            patch_action,
        }),
        decision: decision_override.or(decision_entry),
    };

    let mut opts = PackOptions { delta, schema_override, ..Default::default() };

    // Handoff via context.md: passed straight into pack() as an extra entry,
    // so the archive is built exactly once (no rebuild, no byte clones).
    if let Some(handoff) = context_handoff {
        let existing_ctx = parent.read_entry_string("agent/context.md").unwrap_or_default();
        let new_ctx = if existing_ctx.is_empty() { handoff } else { format!("{}\n\n---\n{}", existing_ctx, handoff) };
        opts.extra_entries.push(("agent/context.md".to_string(), new_ctx.into_bytes()));
    }

    pack(parent, output, opts, compressor)
}

/// Parse optional YAML frontmatter from the top of an HTML string.
/// Returns (mode, structured_data, decision_entry, patch_selector, patch_action, context_handoff, html_without_frontmatter).
fn parse_html_frontmatter(input: &str) -> (Option<String>, Value, Option<DecisionEntry>, Option<String>, Option<String>, Option<String>, String) {
    let empty = (None, Value::Object(Default::default()), None, None, None, None, input.to_string());

    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return empty;
    }

    let after_open = &trimmed[3..];
    let Some(close) = after_open.find("\n---") else { return empty };

    let yaml_src = &after_open[..close];
    let html_body = after_open[close + 4..].trim_start().to_string();

    let val = match serde_yaml::from_str::<serde_yaml::Value>(yaml_src) {
        Ok(v) => v,
        Err(_) => {
            // Auto-correction fallback for common LLM indentation/quote errors
            let fixed = yaml_src.replace("\t", "  ");
            match serde_yaml::from_str::<serde_yaml::Value>(&fixed) {
                Ok(v) => v,
                Err(_) => return empty,
            }
        }
    };

    let mode = val.get("mode").and_then(|v| v.as_str().map(|s| s.to_string()));
    let patch_selector = val.get("patch_selector").and_then(|v| v.as_str().map(|s| s.to_string()));
    let patch_action = val.get("patch_action").and_then(|v| v.as_str().map(|s| s.to_string()));
    let context_handoff = val.get("next_task").or_else(|| val.get("context")).and_then(|v| v.as_str().map(|s| s.to_string()));

    let structured: Value = val
        .get("structured")
        .and_then(|v| serde_json::to_value(v).ok())
        .unwrap_or_else(|| Value::Object(Default::default()));

    let decision_entry = val.get("decision").and_then(|d| {
        Some(DecisionEntry {
            agent_name: d["agent"].as_str().unwrap_or("unknown").to_string(),
            action: d["action"].as_str().unwrap_or("").to_string(),
            rationale: d["rationale"].as_str().unwrap_or("").to_string(),
            pinned: d["pinned"].as_bool().unwrap_or(false),
            fields_changed: None,
        })
    });

    (mode, structured, decision_entry, patch_selector, patch_action, context_handoff, html_body)
}

fn apply_html_patch(existing: &str, payload: &HumanPayload) -> Result<String> {
    use lol_html::{rewrite_str, element, RewriteStrSettings};
    use std::cell::Cell;
    let selector = payload.patch_selector.as_deref().unwrap_or("body");
    let action = payload.patch_action.as_deref().unwrap_or("append");
    let html = &payload.html;
    let matched = Cell::new(0usize);

    let result = rewrite_str(
        existing,
        RewriteStrSettings {
            element_content_handlers: vec![
                element!(selector, |el| {
                    matched.set(matched.get() + 1);
                    match action {
                        "append" => el.append(&html, lol_html::html_content::ContentType::Html),
                        "prepend" => el.prepend(&html, lol_html::html_content::ContentType::Html),
                        "replace" => el.replace(&html, lol_html::html_content::ContentType::Html),
                        "before" => el.before(&html, lol_html::html_content::ContentType::Html),
                        "after" => el.after(&html, lol_html::html_content::ContentType::Html),
                        _ => el.append(&html, lol_html::html_content::ContentType::Html),
                    }
                    Ok(())
                })
            ],
            ..RewriteStrSettings::default()
        }
    )
    .map_err(|e| Error::OutputRejected(format!("patch-html: could not apply selector {selector:?}: {e}")))?;

    if matched.get() == 0 {
        return Err(Error::OutputRejected(format!(
            "patch-html: selector {selector:?} matched no elements — nothing was patched"
        )));
    }
    Ok(result)
}

/// Strip `<script>...</script>` blocks and `on*` event handler attributes.
/// Everything else — including full `<html>/<head>/<body>`, external CSS
/// `@import`, Google Fonts, CDN stylesheets — is allowed through. The viewer
/// iframe sandbox controls execution; the SDK's job is only to prevent XSS.
pub fn strip_scripts(html: &str) -> String {
    // Remove <script ...>...</script> blocks (case-insensitive, multiline).
    // Lowercase once and search offsets into the original string instead of
    // re-allocating a lowercased copy on every iteration.
    let lower = html.to_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut pos = 0;
    while let Some(rel) = lower[pos..].find("<script") {
        let start = pos + rel;
        out.push_str(&html[pos..start]);
        match lower[start..].find("</script>") {
            Some(end_rel) => {
                pos = start + end_rel + "</script>".len();
            }
            None => {
                // Unclosed script tag — strip to end.
                pos = html.len();
                break;
            }
        }
    }
    out.push_str(&html[pos..]);
    // Strip on* event handlers: onclick=, onerror=, etc.
    strip_on_handlers(&out)
}

fn strip_on_handlers(html: &str) -> String {
    // Simple state-machine: inside a tag, remove attributes starting with "on".
    let bytes = html.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_tag = false;
    while i < bytes.len() {
        if !in_tag && bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1] != b'/' {
            in_tag = true;
            out.push(bytes[i]);
            i += 1;
        } else if in_tag && bytes[i] == b'>' {
            in_tag = false;
            out.push(bytes[i]);
            i += 1;
        } else if in_tag && (bytes[i] == b'"' || bytes[i] == b'\'') {
            // Copy a quoted attribute value verbatim so a '>' inside it does
            // not prematurely end the tag (e.g. `class="a>b" onclick="..."`).
            let q = bytes[i];
            out.push(bytes[i]);
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                out.push(bytes[i]);
                i += 1;
            }
            if i < bytes.len() {
                out.push(bytes[i]); // closing quote
                i += 1;
            }
        } else if in_tag {
            // Check for on* attribute.
            let rest = &bytes[i..];
            if rest.starts_with(b"on")
                && rest.len() > 2
                && rest[2..].iter().next().map_or(false, |c| c.is_ascii_alphabetic())
            {
                // Skip to end of attribute value.
                while i < bytes.len() && bytes[i] != b'>' {
                    if bytes[i] == b'"' || bytes[i] == b'\'' {
                        let q = bytes[i];
                        i += 1;
                        while i < bytes.len() && bytes[i] != q { i += 1; }
                    }
                    if i < bytes.len() {
                        // Stop at next attribute or tag end.
                        if bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'>' {
                            break;
                        }
                        i += 1;
                    }
                }
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Perform an RFC 7396 JSON Merge Patch on `a` using `b`.
fn merge_json(a: &mut Value, b: &Value) {
    if let Value::Object(b_map) = b {
        if let Value::Object(a_map) = a {
            for (k, v) in b_map {
                if v.is_null() {
                    a_map.remove(k);
                } else {
                    merge_json(a_map.entry(k.clone()).or_insert(Value::Null), v);
                }
            }
            return;
        }
    }
    *a = b.clone();
}

/// Patch `shared/data.yaml` inside the archive with a JSON Merge Patch (RFC 7396).
/// This merges the patch over the existing data, preserving all other files,
/// and increments the generation.
/// Options for [`patch_data_with`].
#[derive(Debug, Default)]
pub struct PatchDataOptions {
    /// Keys whose array value should be appended to the existing array rather
    /// than replaced (F14). RFC 7396 replaces arrays wholesale; for these keys
    /// the patch's array is concatenated onto the existing one (mirrors the
    /// `append` merge policy, spec §24.3).
    pub append_keys: Vec<String>,
    /// Decision to record for this change (F15). When present, its
    /// `fields_changed` is set to the exact merge-patch keys by the caller.
    pub decision: Option<DecisionEntry>,
}

/// Append `new_val` onto the array at `slot`, coercing a non-array slot into a
/// single-element array first (so a first-time append still works).
fn append_into(slot: &mut Value, new_val: &Value) {
    if !slot.is_array() {
        let cur = std::mem::replace(slot, Value::Null);
        *slot = Value::Array(if cur.is_null() { Vec::new() } else { vec![cur] });
    }
    let arr = slot.as_array_mut().expect("coerced to array above");
    match new_val {
        Value::Array(items) => arr.extend(items.iter().cloned()),
        other => arr.push(other.clone()),
    }
}

/// Apply `patch` to `data` with RFC 7396 merge semantics, except keys named in
/// `append_keys` are array-appended rather than replaced (F14).
fn apply_patch_with_append(data: &mut Value, patch: &Value, append_keys: &[String]) {
    let append_set: std::collections::BTreeSet<&str> =
        append_keys.iter().map(String::as_str).collect();
    let patch_obj = match patch.as_object() {
        Some(o) if !append_set.is_empty() => o,
        _ => {
            merge_json(data, patch);
            return;
        }
    };
    // 1) merge every non-append key with normal merge-patch semantics.
    let merge_part: serde_json::Map<String, Value> = patch_obj
        .iter()
        .filter(|(k, _)| !append_set.contains(k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    if !merge_part.is_empty() {
        merge_json(data, &Value::Object(merge_part));
    }
    // 2) concatenate the append keys onto whatever is already there.
    if !data.is_object() {
        *data = Value::Object(Default::default());
    }
    let data_obj = data.as_object_mut().expect("forced to object above");
    for key in append_keys {
        if let Some(new_val) = patch_obj.get(key) {
            let slot = data_obj.entry(key.clone()).or_insert_with(|| Value::Array(Vec::new()));
            append_into(slot, new_val);
        }
    }
}

pub fn patch_data(parent: &ClanFile, patch: &Value, compressor: Option<&Compressor>) -> Result<Vec<u8>> {
    patch_data_with(parent, patch, PatchDataOptions::default(), compressor)
}

/// Like [`patch_data`] but with array-append keys (F14) and an optional
/// attributed decision (F15).
pub fn patch_data_with(
    parent: &ClanFile,
    patch: &Value,
    pd_opts: PatchDataOptions,
    compressor: Option<&Compressor>,
) -> Result<Vec<u8>> {
    reject_if_forked(parent, "a direct write to shared/data.yaml")?;
    let existing_str = parent.read_entry_string("shared/data.yaml").unwrap_or_default();
    let mut data: Value = if existing_str.is_empty() {
        Value::Object(Default::default())
    } else {
        serde_yaml::from_str(&existing_str).map_err(|e| Error::OutputRejected(format!("existing data.yaml is invalid: {}", e)))?
    };

    apply_patch_with_append(&mut data, patch, &pd_opts.append_keys);

    // To preserve lineage, we formulate an AgentOutput just like pack_html does,
    // where we only set structured data, and let `pack` handle generating the new ZIP.
    let output = AgentOutput {
        mode: "data-update".to_string(),
        structured: data,
        design: None,
        human: None,
        decision: pd_opts.decision,
    };

    let mut opts = PackOptions { delta: None, ..Default::default() };

    // Adjudication (spec §25): a data write that settles a contested key
    // removes it from the merge report and decrements `unresolved`.
    if parent.has_entry(crate::merge::MERGE_REPORT_PATH) {
        if let Ok(mut report) =
            crate::merge::MergeReport::from_yaml(&parent.read_entry(crate::merge::MERGE_REPORT_PATH)?)
        {
            let patched: std::collections::BTreeSet<&str> = patch
                .as_object()
                .map(|o| o.keys().map(String::as_str).collect())
                .unwrap_or_default();
            let before = report.conflicts.len();
            report.conflicts.retain(|c| !patched.contains(c.key.as_str()));
            if report.conflicts.len() != before {
                report.unresolved = report.conflicts.len();
                opts.extra_entries
                    .push((crate::merge::MERGE_REPORT_PATH.to_string(), report.to_yaml()?));
            }
        }
    }

    pack(parent, output, opts, compressor)
}

/// Append a new Decision to `agent/decision-chain.yaml` inside the archive.
/// Preserves all other files and increments the generation. Swaps only the
/// chain entry — no schema validation runs, since the data is untouched.
pub fn patch_decision(parent: &ClanFile, entry: DecisionEntry, compressor: Option<&Compressor>) -> Result<Vec<u8>> {
    let now = Utc::now().to_rfc3339();

    // On a forked branch file decisions are auto-routed into the branch
    // namespace, so they fold cleanly at `clan merge` (spec §24.1).
    let chain_path = match parent.manifest().fork_namespace() {
        Some(ns) => format!("{ns}decisions.yaml"),
        None => "agent/decision-chain.yaml".to_string(),
    };
    let mut chain = if parent.has_entry(&chain_path) {
        DecisionChain::from_yaml(&parent.read_entry(&chain_path)?)?
    } else if chain_path == "agent/decision-chain.yaml" {
        // The shared chain is a required member; surface the real error.
        DecisionChain::from_yaml(&parent.read_entry(&chain_path)?)?
    } else {
        DecisionChain::default()
    };

    let delta = format!("appended decision by {}", entry.agent_name);
    chain.prepend(Decision {
        agent: entry.agent_name,
        version: None,
        action: entry.action,
        rationale: entry.rationale,
        timestamp: now,
        fields_changed: Vec::new(),
        pinned: entry.pinned,
        trace_ref: None,
    });
    compress_chain(&mut chain, &CompressionConfig::default(), compressor);
    let new_chain_yaml = chain.to_yaml()?;

    repack_with_entry(parent, &chain_path, new_chain_yaml, Some(delta))
}

/// Merge-patch the branch namespace `agents/<agent_id>/data.yaml` of a forked
/// file (RFC 7396). The shared members are untouched — this is the only data
/// write a branch agent is allowed (spec §24.1).
pub fn patch_data_namespaced(parent: &ClanFile, patch: &Value) -> Result<Vec<u8>> {
    let ns = parent.manifest().fork_namespace().ok_or_else(|| {
        Error::NamespaceViolation(
            "this file is not forked — there is no branch namespace.\n\
             next: write shared data directly with `clan patch-data` (no --namespace)"
                .into(),
        )
    })?;
    let data_path = format!("{ns}data.yaml");
    let mut data: Value = if parent.has_entry(&data_path) {
        let existing = parent.read_entry_string(&data_path)?;
        if existing.trim().is_empty() {
            Value::Object(Default::default())
        } else {
            serde_yaml::from_str(&existing)
                .map_err(|e| Error::OutputRejected(format!("existing {data_path} is invalid: {e}")))?
        }
    } else {
        Value::Object(Default::default())
    };
    merge_json(&mut data, patch);
    let new_yaml = serde_yaml::to_string(&data)?.into_bytes();
    repack_with_entry(parent, &data_path, new_yaml, Some(format!("patched {data_path}")))
}

/// Replace `agent/output-schema.json` inside the archive.
/// This allows an agent to atomically migrate a file's structure.
pub fn patch_schema(parent: &ClanFile, schema_json: &str, compressor: Option<&Compressor>) -> Result<Vec<u8>> {
    reject_if_forked(parent, "replacing the shared output schema")?;
    let existing_data_str = parent.read_entry_string("shared/data.yaml").unwrap_or_default();
    let structured: Value = if existing_data_str.is_empty() {
        Value::Object(Default::default())
    } else {
        serde_yaml::from_str(&existing_data_str).unwrap_or(Value::Object(Default::default()))
    };

    let output = AgentOutput {
        mode: "data-update".to_string(), // use data-update to avoid touching HTML
        structured,
        design: None,
        human: None,
        decision: None,
    };

    let opts = PackOptions { 
        delta: Some("schema migrated".to_string()), 
        schema_override: Some(schema_json.to_string()),
        ..Default::default()
    };
    
    pack(parent, output, opts, compressor)
}

/// Helper to repack a `.clan` file with a single updated or added entry.
fn repack_with_entry(parent: &ClanFile, target_path: &str, new_bytes: Vec<u8>, delta: Option<String>) -> Result<Vec<u8>> {
    repack_with_entry_decision(parent, target_path, new_bytes, delta, None)
}

/// Like [`repack_with_entry`] but, when `decision` is present, also prepends it
/// to `agent/decision-chain.yaml` in the same generation (F15: attributed asset
/// writes). `fields_changed` defaults to empty for non-data members.
fn repack_with_entry_decision(
    parent: &ClanFile,
    target_path: &str,
    new_bytes: Vec<u8>,
    delta: Option<String>,
    decision: Option<DecisionEntry>,
) -> Result<Vec<u8>> {
    let now = chrono::Utc::now().to_rfc3339();
    let parent_manifest = parent.manifest();
    let mut new_manifest = parent_manifest.clone();
    new_manifest.id = uuid::Uuid::new_v4().to_string();
    new_manifest.updated_at = now.clone();
    new_manifest.lineage = Some(Lineage {
        parent_id: parent_manifest.id.clone(),
        parent_uri: format!("file:///unknown/{}.clan", parent_manifest.id),
        parent_sha256: Some(parent.sha256()),
        delta: delta.unwrap_or_default(),
        parents: Vec::new(),
        merge: false,
    });

    // Ensure the file is tracked in manifest files array if it's an asset or state
    let role = if target_path == "agent/state.yaml" {
        "agent-state"
    } else if target_path == "agent/context.md" {
        "agent-context"
    } else if target_path.starts_with("human/assets/") {
        "human-asset"
    } else if target_path == "agent/requirements.yaml" {
        "agent-requirements"
    } else if target_path.starts_with("agents/") && target_path.ends_with("/data.yaml") {
        "branch-data"
    } else if target_path.starts_with("agents/") && target_path.ends_with("/decisions.yaml") {
        "branch-decisions"
    } else {
        "unknown"
    };

    if !new_manifest.files.iter().any(|f| f.path == target_path) {
        new_manifest.files.push(crate::manifest::FileEntry {
            id: target_path.replace("/", "-"),
            path: target_path.to_string(),
            role: role.to_string(),
            content_type: "application/octet-stream".to_string(), // Builder will override this if it can infer
            priority: None,
            sha256: None,
        });
    }

    // Optional attributed decision (F15): prepend to the shared chain.
    const CHAIN_PATH: &str = "agent/decision-chain.yaml";
    let chain_override: Option<Vec<u8>> = match decision {
        Some(d) => {
            let mut chain = DecisionChain::from_yaml(&parent.read_entry(CHAIN_PATH)?)?;
            chain.prepend(Decision {
                agent: d.agent_name,
                version: None,
                action: d.action,
                rationale: d.rationale,
                timestamp: now.clone(),
                fields_changed: d.fields_changed.unwrap_or_default(),
                pinned: d.pinned,
                trace_ref: None,
            });
            Some(chain.to_yaml()?)
        }
        None => None,
    };

    let mut builder = ClanBuilder::new(new_manifest);

    for (path, bytes) in parent.read_all_entries()? {
        if path == "manifest.yaml" || path == target_path { continue; }
        if chain_override.is_some() && path == CHAIN_PATH { continue; }
        builder.add_entry(path, bytes);
    }

    builder.add_entry(target_path, new_bytes);
    if let Some(chain_bytes) = chain_override {
        builder.add_entry(CHAIN_PATH, chain_bytes);
    }
    builder.build()
}

/// Patch `agent/state.yaml` inside the archive with a JSON Merge Patch (RFC 7396).
pub fn patch_state(parent: &ClanFile, patch: &Value) -> Result<Vec<u8>> {
    let existing_str = parent.read_entry_string("agent/state.yaml").unwrap_or_default();
    let mut state: Value = if existing_str.is_empty() {
        Value::Object(Default::default())
    } else {
        serde_yaml::from_str(&existing_str).map_err(|e| Error::OutputRejected(format!("invalid state: {}", e)))?
    };
    merge_json(&mut state, patch);
    let new_state = serde_yaml::to_string(&state)?.into_bytes();
    repack_with_entry(parent, "agent/state.yaml", new_state, Some("patched agent/state.yaml".into()))
}

/// Patch or append to `agent/context.md`.
pub fn patch_context(parent: &ClanFile, text: &str, append: bool) -> Result<Vec<u8>> {
    reject_if_forked(parent, "rewriting the shared agent/context.md")?;
    let mut existing = parent.read_entry_string("agent/context.md").unwrap_or_default();
    if append {
        if !existing.is_empty() && !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(text);
    } else {
        existing = text.to_string();
    }
    repack_with_entry(parent, "agent/context.md", existing.into_bytes(), Some("patched agent/context.md".into()))
}

/// Write or replace `agent/requirements.yaml` — declared tool/capability needs
/// (spec §22 layer 5). Validated as YAML; surfaced in agent context (§26) and
/// the static export. This makes layer 5 first-class instead of dead weight (F8).
pub fn patch_requirements(parent: &ClanFile, yaml_text: &str) -> Result<Vec<u8>> {
    reject_if_forked(parent, "writing the shared agent/requirements.yaml")?;
    serde_yaml::from_str::<serde_yaml::Value>(yaml_text)
        .map_err(|e| Error::OutputRejected(format!("requirements.yaml is not valid YAML: {e}")))?;
    repack_with_entry(
        parent,
        "agent/requirements.yaml",
        yaml_text.as_bytes().to_vec(),
        Some("declared capability requirements".into()),
    )
}

/// Inject or replace an asset in `human/assets/`.
pub fn patch_asset(parent: &ClanFile, internal_path: &str, bytes: Vec<u8>) -> Result<Vec<u8>> {
    patch_asset_with(parent, internal_path, bytes, None)
}

/// Like [`patch_asset`] but records an attributed decision (F15) when present.
pub fn patch_asset_with(
    parent: &ClanFile,
    internal_path: &str,
    bytes: Vec<u8>,
    decision: Option<DecisionEntry>,
) -> Result<Vec<u8>> {
    reject_if_forked(parent, "writing human/ assets")?;
    let full_path = if internal_path.starts_with("human/assets/") {
        internal_path.to_string()
    } else {
        format!("human/assets/{}", internal_path)
    };
    repack_with_entry_decision(
        parent,
        &full_path,
        bytes,
        Some(format!("added asset {}", internal_path)),
        decision,
    )
}

#[cfg(test)]
mod tests {
    use super::strip_scripts;
    use super::*;
    use crate::manifest::Manifest;

    /// Build a minimal in-memory clan with the given data and schema.
    fn test_clan(data_yaml: &str, schema_json: &str) -> ClanFile {
        fn entry(id: &str, path: &str, role: &str, ct: &str) -> FileEntry {
            FileEntry {
                id: id.into(),
                path: path.into(),
                role: role.into(),
                content_type: ct.into(),
                priority: None,
                sha256: None,
            }
        }
        let manifest = Manifest {
            clan_version: CLAN_VERSION,
            clan_version_minor: CLAN_VERSION_MINOR,
            id: "11111111-2222-3333-4444-555555555555".into(),
            title: "Test".into(),
            created_at: "2026-06-01T10:00:00Z".into(),
            updated_at: "2026-06-01T10:00:00Z".into(),
            document_type: None,
            lineage: None,
            view: None,
            fork: None,
            merge_policies: None,
            external: vec![],
            files: vec![
                entry("canonical-data", "shared/data.yaml", "canonical-data", "application/yaml"),
                entry("agent-context", "agent/context.md", "agent-context", "text/markdown"),
                entry("agent-schema", "agent/output-schema.json", "agent-schema", "application/json"),
                entry("agent-chain", "agent/decision-chain.yaml", "agent-chain", "application/yaml"),
                entry("human-view", "human/index.html", "human-view", "text/html"),
            ],
        };
        let mut builder = ClanBuilder::new(manifest);
        builder.add_entry("shared/data.yaml", data_yaml.as_bytes().to_vec());
        builder.add_entry("agent/context.md", b"original task".to_vec());
        builder.add_entry("agent/output-schema.json", schema_json.as_bytes().to_vec());
        builder.add_entry("agent/decision-chain.yaml", b"decisions: []\n".to_vec());
        builder.add_entry(
            "human/index.html",
            b"<html><body><div id=\"app\">x</div></body></html>".to_vec(),
        );
        ClanFile::from_bytes(builder.build().unwrap()).unwrap()
    }

    const PERMISSIVE_SCHEMA: &str = r#"{"type": "object"}"#;
    const STRICT_SCHEMA: &str =
        r#"{"type": "object", "required": ["title"], "properties": {"title": {"type": "string"}}}"#;

    // Regression for #14: appending a decision must not run schema validation,
    // because the canonical data is untouched. The old implementation routed
    // through pack(), which rejected the append whenever the existing data
    // happened to violate the schema.
    #[test]
    fn patch_decision_does_not_validate_schema() {
        // Data violates the strict schema (no `title`).
        let parent = test_clan("vendor: Acme\n", STRICT_SCHEMA);

        // Sanity: the full pack() route (old behaviour) rejects this archive.
        let output = AgentOutput {
            mode: "data-update".into(),
            structured: serde_json::json!({"vendor": "Acme"}),
            design: None,
            human: None,
            decision: None,
        };
        assert!(
            pack(&parent, output, PackOptions::default(), None).is_err(),
            "test setup: pack() should reject schema-violating data"
        );

        let bytes = patch_decision(
            &parent,
            DecisionEntry {
                agent_name: "agent7".into(),
                action: "reviewed".into(),
                rationale: "looks good".into(),
                pinned: true,
                fields_changed: None,
            },
            None,
        )
        .expect("patch_decision must succeed without validating data");

        let next = ClanFile::from_bytes(bytes).unwrap();
        let chain =
            DecisionChain::from_yaml(&next.read_entry("agent/decision-chain.yaml").unwrap())
                .unwrap();
        assert_eq!(chain.decisions.len(), 1);
        assert_eq!(chain.decisions[0].agent, "agent7");
        assert!(chain.decisions[0].pinned);
        assert!(chain.decisions[0].fields_changed.is_empty());
        // Data and lineage preserved.
        assert_eq!(
            next.read_entry_string("shared/data.yaml").unwrap(),
            "vendor: Acme\n"
        );
        assert_eq!(
            next.manifest().lineage.as_ref().unwrap().parent_id,
            parent.manifest().id
        );
    }

    // Regression for #15: a context_handoff must be applied inside the single
    // pack() build — one new generation, parent entries intact, contexts merged.
    #[test]
    fn pack_html_context_handoff_builds_once() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let html = "---\n\
                    structured:\n  title: \"T\"\n\
                    next_task: \"agent2: refine the layout\"\n\
                    ---\n\
                    <!DOCTYPE html><html><body><p>hi</p></body></html>";

        let bytes = pack_html(&parent, html, None, None, None, None).unwrap();
        let next = ClanFile::from_bytes(bytes).unwrap();

        // Contexts merged: original + separator + handoff.
        assert_eq!(
            next.read_entry_string("agent/context.md").unwrap(),
            "original task\n\n---\nagent2: refine the layout"
        );
        // Exactly one generation: lineage points straight at the parent.
        assert_eq!(
            next.manifest().lineage.as_ref().unwrap().parent_id,
            parent.manifest().id
        );
        // New HTML landed; other parent entries carried over.
        assert!(next
            .read_entry_string("human/index.html")
            .unwrap()
            .contains("<p>hi</p>"));
        assert!(next.has_entry("agent/output-schema.json"));
        assert!(next.has_entry("agent/decision-chain.yaml"));
    }

    #[test]
    fn pack_html_handoff_without_existing_context() {
        let parent = {
            // Same as test_clan but with an empty context.
            let clan = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
            let mut builder = ClanBuilder::new(clan.manifest().clone());
            for (path, bytes) in clan.read_all_entries().unwrap() {
                if path == "manifest.yaml" { continue; }
                builder.add_entry(path, bytes);
            }
            builder.add_entry("agent/context.md", Vec::new());
            ClanFile::from_bytes(builder.build().unwrap()).unwrap()
        };
        let html = "---\nnext_task: \"start here\"\n---\n<p>doc</p>";
        let next = ClanFile::from_bytes(
            pack_html(&parent, html, None, None, None, None).unwrap(),
        )
        .unwrap();
        assert_eq!(
            next.read_entry_string("agent/context.md").unwrap(),
            "start here"
        );
    }

    // Regression for #21: a patch whose selector matches nothing must fail
    // loudly instead of returning the document unchanged and "succeeding".
    #[test]
    fn patch_html_errors_when_selector_matches_nothing() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let payload = |selector: &str| AgentOutput {
            mode: "patch-html".into(),
            structured: serde_json::json!({}),
            design: None,
            human: Some(HumanPayload {
                html: "<span>new</span>".into(),
                css: None,
                assets: HashMap::new(),
                patch_selector: Some(selector.into()),
                patch_action: Some("append".into()),
            }),
            decision: None,
        };

        let err = pack(&parent, payload("#does-not-exist"), PackOptions::default(), None)
            .expect_err("zero-match selector must be an error");
        assert!(
            err.to_string().contains("matched no elements"),
            "unexpected error message: {err}"
        );

        // A matching selector still works.
        let bytes = pack(&parent, payload("#app"), PackOptions::default(), None).unwrap();
        let next = ClanFile::from_bytes(bytes).unwrap();
        assert!(next
            .read_entry_string("human/index.html")
            .unwrap()
            .contains("<span>new</span>"));
    }

    #[test]
    fn strips_script_blocks() {
        let html = "<p>hi</p><script>evil()</script><b>bye</b>";
        assert_eq!(strip_scripts(html), "<p>hi</p><b>bye</b>");
    }

    #[test]
    fn strips_script_case_insensitively() {
        let html = "a<SCRIPT>x</SCRIPT>b<ScRiPt>y</sCrIpT>c";
        assert_eq!(strip_scripts(html), "abc");
    }

    #[test]
    fn strips_unclosed_script_to_end() {
        let html = "keep<script>dangling";
        assert_eq!(strip_scripts(html), "keep");
    }

    #[test]
    fn strips_on_handler_attribute() {
        let out = strip_scripts(r#"<div onclick="evil()">x</div>"#);
        assert!(!out.contains("onclick"), "onclick survived: {out}");
        assert!(out.contains(">x</div>"));
    }

    // Regression for #16: a '>' inside a quoted attribute value must not end
    // the tag early, or an on* handler after it would survive sanitisation.
    #[test]
    fn gt_inside_quoted_attr_does_not_bypass_on_handler_strip() {
        let out = strip_scripts(r#"<div class="a>b" onclick="evil()">x</div>"#);
        assert!(
            !out.contains("onclick"),
            "sanitizer bypassed — onclick survived: {out}"
        );
        // The quoted value (including the inner '>') is preserved verbatim.
        assert!(out.contains(r#"class="a>b""#), "quoted attr mangled: {out}");
    }

    #[test]
    fn single_quoted_gt_also_handled() {
        let out = strip_scripts(r#"<img alt='x>y' onerror='boom()'>"#);
        assert!(!out.contains("onerror"), "onerror survived: {out}");
        assert!(out.contains("alt='x>y'"), "quoted attr mangled: {out}");
    }

    fn chain_of(clan: &ClanFile) -> DecisionChain {
        DecisionChain::from_yaml(&clan.read_entry("agent/decision-chain.yaml").unwrap()).unwrap()
    }

    // F1: a data update with no decision must NOT spawn an unknown-agent entry.
    #[test]
    fn pack_without_decision_adds_no_chain_entry() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        assert_eq!(chain_of(&parent).decisions.len(), 0);

        let next = ClanFile::from_bytes(
            patch_data(&parent, &serde_json::json!({"total": 10}), None).unwrap(),
        )
        .unwrap();
        let chain = chain_of(&next);
        assert_eq!(chain.decisions.len(), 0, "no decision supplied → no chain entry (F1)");
        assert!(
            !next.read_entry_string("agent/decision-chain.yaml").unwrap().contains("unknown-agent"),
            "the unknown-agent placeholder must be gone"
        );
    }

    #[test]
    fn pack_with_decision_still_records_it() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let out = AgentOutput {
            mode: "data-update".into(),
            structured: serde_json::json!({"vendor": "Acme", "total": 5}),
            design: None,
            human: None,
            decision: Some(DecisionEntry {
                agent_name: "agent1".into(),
                action: "set total".into(),
                rationale: "from invoice".into(),
                pinned: false,
                fields_changed: None,
            }),
        };
        let next = ClanFile::from_bytes(pack(&parent, out, PackOptions::default(), None).unwrap()).unwrap();
        let chain = chain_of(&next);
        assert_eq!(chain.decisions.len(), 1);
        assert_eq!(chain.decisions[0].agent, "agent1");
    }

    // F14: --append concatenates an array key instead of replacing it.
    #[test]
    fn patch_data_append_concatenates_arrays() {
        let parent = test_clan("tags:\n  - a\n  - b\n", PERMISSIVE_SCHEMA);
        let opts = PatchDataOptions { append_keys: vec!["tags".into()], decision: None };
        let next = ClanFile::from_bytes(
            patch_data_with(&parent, &serde_json::json!({"tags": ["c"]}), opts, None).unwrap(),
        )
        .unwrap();
        let data: Value = serde_yaml::from_str(&next.read_entry_string("shared/data.yaml").unwrap()).unwrap();
        let tags = data["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 3, "append must keep all three: {tags:?}");

        // A non-append key on the same patch still replaces (RFC 7396 default).
        let opts = PatchDataOptions { append_keys: vec!["tags".into()], decision: None };
        let next2 = ClanFile::from_bytes(
            patch_data_with(&next, &serde_json::json!({"tags": ["d"], "vendor": "X"}), opts, None).unwrap(),
        )
        .unwrap();
        let data2: Value = serde_yaml::from_str(&next2.read_entry_string("shared/data.yaml").unwrap()).unwrap();
        assert_eq!(data2["tags"].as_array().unwrap().len(), 4, "tags appended");
        assert_eq!(data2["vendor"], serde_json::json!("X"), "non-append key set normally");
    }

    // F14: appending to a missing key creates the array.
    #[test]
    fn patch_data_append_creates_missing_array() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let opts = PatchDataOptions { append_keys: vec!["notes".into()], decision: None };
        let next = ClanFile::from_bytes(
            patch_data_with(&parent, &serde_json::json!({"notes": ["first"]}), opts, None).unwrap(),
        )
        .unwrap();
        let data: Value = serde_yaml::from_str(&next.read_entry_string("shared/data.yaml").unwrap()).unwrap();
        assert_eq!(data["notes"].as_array().unwrap().len(), 1);
    }

    // F15: an attributed patch_data records a decision whose fields_changed is
    // EXACTLY the patch keys, not the whole merged document.
    #[test]
    fn patch_data_decision_fields_changed_are_patch_keys_only() {
        let parent = test_clan("vendor: Acme\nexisting: keep\n", PERMISSIVE_SCHEMA);
        let decision = Some(DecisionEntry {
            agent_name: "pricing".into(),
            action: "set total".into(),
            rationale: String::new(),
            pinned: false,
            fields_changed: Some(vec!["total".into()]),
        });
        let opts = PatchDataOptions { append_keys: vec![], decision };
        let next = ClanFile::from_bytes(
            patch_data_with(&parent, &serde_json::json!({"total": 5}), opts, None).unwrap(),
        )
        .unwrap();
        let chain = chain_of(&next);
        assert_eq!(chain.decisions.len(), 1);
        assert_eq!(chain.decisions[0].fields_changed, vec!["total".to_string()]);
        // The untouched key survives (pass-through) but is NOT in fields_changed.
        let data: Value = serde_yaml::from_str(&next.read_entry_string("shared/data.yaml").unwrap()).unwrap();
        assert_eq!(data["existing"], serde_json::json!("keep"));
    }

    // F15: the default patch_data (no options) still records no decision, so a
    // bare data update adds no chain entry (F1 preserved).
    #[test]
    fn patch_data_without_options_records_no_decision() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let before = chain_of(&parent).decisions.len();
        let next = ClanFile::from_bytes(
            patch_data(&parent, &serde_json::json!({"total": 5}), None).unwrap(),
        )
        .unwrap();
        assert_eq!(chain_of(&next).decisions.len(), before, "bare patch_data adds no entry");
    }

    // F2: full-html marks the view agent-authored; data-update leaves no source.
    #[test]
    fn full_html_marks_view_source_agent() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let out = AgentOutput {
            mode: "full-html".into(),
            structured: serde_json::json!({}),
            design: None,
            human: Some(HumanPayload {
                html: "<!DOCTYPE html><html><body><p>hi</p></body></html>".into(),
                css: None,
                assets: HashMap::new(),
                patch_selector: None,
                patch_action: None,
            }),
            decision: None,
        };
        let next = ClanFile::from_bytes(pack(&parent, out, PackOptions::default(), None).unwrap()).unwrap();
        // test_clan's manifest has no view block, so nothing to assert there;
        // build one that does.
        let mut m = parent.manifest().clone();
        m.view = Some(crate::manifest::ViewState { present: true, renderable: true, stale: false, source: None });
        // Rebuild parent carrying the view, then full-html pack it.
        let mut b = ClanBuilder::new(m);
        for (p, by) in parent.read_all_entries().unwrap() { if p != "manifest.yaml" { b.add_entry(p, by); } }
        let parent2 = ClanFile::from_bytes(b.build().unwrap()).unwrap();
        let out2 = AgentOutput {
            mode: "full-html".into(),
            structured: serde_json::json!({}),
            design: None,
            human: Some(HumanPayload { html: "<p>x</p>".into(), css: None, assets: HashMap::new(), patch_selector: None, patch_action: None }),
            decision: None,
        };
        let n2 = ClanFile::from_bytes(pack(&parent2, out2, PackOptions::default(), None).unwrap()).unwrap();
        assert_eq!(n2.manifest().view.as_ref().unwrap().source.as_deref(), Some("agent"));
        let _ = next;
    }

    // F5: full-html replacement folds superseded human patches into the chain.
    #[test]
    fn full_html_folds_human_patches_into_chain() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        // Add a human patch via the patch model.
        let parent = ClanFile::from_bytes(
            crate::patch::apply_patch_and_repack(&parent, "heading-0".into(), "Amended Title".into()).unwrap(),
        )
        .unwrap();
        assert!(parent.has_entry("human/patches.yaml"));

        let out = AgentOutput {
            mode: "full-html".into(),
            structured: serde_json::json!({}),
            design: None,
            human: Some(HumanPayload {
                html: "<!DOCTYPE html><html><body><p>brand new doc</p></body></html>".into(),
                css: None, assets: HashMap::new(), patch_selector: None, patch_action: None,
            }),
            decision: None,
        };
        let next = ClanFile::from_bytes(pack(&parent, out, PackOptions::default(), None).unwrap()).unwrap();
        let chain = chain_of(&next);
        let human = chain.decisions.iter().find(|d| d.agent == "human");
        assert!(human.is_some(), "human edit must be folded into the chain (F5)");
        assert!(human.unwrap().rationale.contains("Amended Title"), "the edit content survives as provenance");
        // The stale DOM patches were dropped from the new view.
        assert!(!next.has_entry("human/patches.yaml"));
    }

    // F8: requirements.yaml is writable and validated.
    #[test]
    fn patch_requirements_writes_and_validates() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        let next = ClanFile::from_bytes(
            patch_requirements(&parent, "requires:\n  tools:\n    - name: web_search\n").unwrap(),
        )
        .unwrap();
        assert!(next.has_entry("agent/requirements.yaml"));
        assert!(next.read_entry_string("agent/requirements.yaml").unwrap().contains("web_search"));
        // Invalid YAML is rejected.
        assert!(patch_requirements(&parent, "requires: [unclosed\n").is_err());
    }

    // F10: full-html replacement keeps parent human/assets that the new view references.
    #[test]
    fn full_html_carries_parent_assets() {
        let parent = test_clan("vendor: Acme\n", PERMISSIVE_SCHEMA);
        // Seed an asset on the parent.
        let parent = ClanFile::from_bytes(
            patch_asset(&parent, "logo.svg", b"<svg/>".to_vec()).unwrap(),
        )
        .unwrap();
        assert!(parent.has_entry("human/assets/logo.svg"));

        let out = AgentOutput {
            mode: "full-html".into(),
            structured: serde_json::json!({}),
            design: None,
            human: Some(HumanPayload {
                html: "<!DOCTYPE html><html><body><img src=\"./assets/logo.svg\"></body></html>".into(),
                css: None, assets: HashMap::new(), patch_selector: None, patch_action: None,
            }),
            decision: None,
        };
        let next = ClanFile::from_bytes(pack(&parent, out, PackOptions::default(), None).unwrap()).unwrap();
        assert!(
            next.has_entry("human/assets/logo.svg"),
            "parent asset must survive full-html replacement (F10)"
        );
        // The new view replaced the old index.html.
        assert!(next.read_entry_string("human/index.html").unwrap().contains("logo.svg"));
    }
}
