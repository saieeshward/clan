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
#[derive(Debug)]
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

#[derive(Debug)]
pub struct HumanPayload {
    pub html: String,
    pub css: Option<String>,
    pub assets: HashMap<String, Vec<u8>>,
    pub patch_selector: Option<String>,
    pub patch_action: Option<String>,
}

#[derive(Debug)]
pub struct DecisionEntry {
    pub agent_name: String,
    pub action: String,
    pub rationale: String,
    pub pinned: bool,
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
        let output = serde_json::json!({
            "mode": self.mode,
            "structured": self.structured,
        });
        let compiled = jsonschema::validator_for(&schema)
            .map_err(|e| Error::Schema(format!("could not compile schema: {e}")))?;
        let errors: Vec<String> = compiled
            .iter_errors(&output)
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
}

/// Pack a new `.clan` archive from a parent file and agent output.
pub fn pack(
    parent: &ClanFile,
    output: AgentOutput,
    opts: PackOptions,
    compressor: Option<&Compressor>,
) -> Result<Vec<u8>> {
    let now = Utc::now().to_rfc3339();
    let parent_manifest = parent.manifest();

    // --- Merge structured data into shared/data.yaml ---
    let existing_data_bytes = parent.read_entry("shared/data.yaml")?;
    let mut data: Value = serde_yaml::from_slice(&existing_data_bytes)?;
    if let (Some(obj), Some(updates)) = (data.as_object_mut(), output.structured.as_object()) {
        for (k, v) in updates {
            obj.insert(k.clone(), v.clone());
        }
    }
    let new_data_yaml = serde_yaml::to_string(&data)?.into_bytes();

    // --- Update decision chain ---
    let chain_bytes = parent.read_entry("agent/decision-chain.yaml")?;
    let mut chain = DecisionChain::from_yaml(&chain_bytes)?;

    let decision = match output.decision {
        Some(d) => Decision {
            agent: d.agent_name,
            version: None,
            action: d.action,
            rationale: d.rationale,
            timestamp: now.clone(),
            fields_changed: output.structured
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default(),
            pinned: d.pinned,
            trace_ref: None,
        },
        None => Decision::new("unknown-agent", "processed document", "", &now),
    };
    chain.prepend(decision);
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
    });

    // Rebuild the files registry, keeping everything from the parent.
    // New/updated entries will have their sha256 recomputed by ClanBuilder.
    // Remove stale human entries if the agent is replacing them.
    if output.mode == "full-html" {
        new_manifest
            .files
            .retain(|f| !f.role.starts_with("human-"));
    }

    // --- Assemble builder ---
    let mut builder = ClanBuilder::new(new_manifest);

    // Carry over all parent entries unchanged, except ones we're replacing.
    for path in parent.entry_paths()? {
        if path == "manifest.yaml" { continue; }
        if path == "shared/data.yaml" { continue; }
        if path == "agent/decision-chain.yaml" { continue; }
        if output.mode == "full-html" && path.starts_with("human/") { continue; }
        // For patch-html, we keep existing human assets and index.html, we'll rewrite index.html below
        if let Ok(bytes) = parent.read_entry(&path) {
            builder.add_entry(path, bytes);
        }
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
                    builder.add_entry(format!("human/assets/{name}"), content);
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
                let new_html = apply_html_patch(&existing, &h);
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
pub fn pack_html(
    parent: &ClanFile,
    raw_html: &str,
    assets_dir_files: Option<HashMap<String, Vec<u8>>>,
    delta: Option<String>,
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
        decision: decision_entry,
    };

    let mut opts = PackOptions { delta, ..Default::default() };
    let mut bytes = pack(parent, output, opts, compressor)?;
    
    // Handoff via context.md
    if let Some(handoff) = context_handoff {
        let mut builder = ClanBuilder::new(ClanFile::from_bytes(bytes.clone())?.manifest().clone());
        let parent_clan = ClanFile::from_bytes(bytes)?;
        for path in parent_clan.entry_paths()? {
            if path == "manifest.yaml" || path == "agent/context.md" { continue; }
            if let Ok(b) = parent_clan.read_entry(&path) {
                builder.add_entry(path, b);
            }
        }
        let existing_ctx = parent.read_entry_string("agent/context.md").unwrap_or_default();
        let new_ctx = if existing_ctx.is_empty() { handoff } else { format!("{}\n\n---\n{}", existing_ctx, handoff) };
        builder.add_entry("agent/context.md", new_ctx.into_bytes());
        bytes = builder.build()?;
    }
    
    Ok(bytes)
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
        })
    });

    (mode, structured, decision_entry, patch_selector, patch_action, context_handoff, html_body)
}

fn apply_html_patch(existing: &str, payload: &HumanPayload) -> String {
    use lol_html::{rewrite_str, element, RewriteStrSettings};
    let selector = payload.patch_selector.as_deref().unwrap_or("body");
    let action = payload.patch_action.as_deref().unwrap_or("append");
    let html = &payload.html;
    
    let result = rewrite_str(
        existing,
        RewriteStrSettings {
            element_content_handlers: vec![
                element!(selector, |el| {
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
    );
    result.unwrap_or_else(|_| existing.to_string())
}

/// Strip `<script>...</script>` blocks and `on*` event handler attributes.
/// Everything else — including full `<html>/<head>/<body>`, external CSS
/// `@import`, Google Fonts, CDN stylesheets — is allowed through. The viewer
/// iframe sandbox controls execution; the SDK's job is only to prevent XSS.
pub fn strip_scripts(html: &str) -> String {
    // Remove <script ...>...</script> blocks (case-insensitive, multiline).
    let mut out = html.to_string();
    loop {
        let lower = out.to_lowercase();
        let Some(start) = lower.find("<script") else { break };
        let Some(end_rel) = lower[start..].find("</script>") else {
            // Unclosed script tag — strip to end.
            out = out[..start].to_string();
            break;
        };
        let end = start + end_rel + "</script>".len();
        out = format!("{}{}", &out[..start], &out[end..]);
    }
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
