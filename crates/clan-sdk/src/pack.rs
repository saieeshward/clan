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
    /// For `full-html` mode: html + css + assets.
    pub human: Option<HumanPayload>,
    /// Decision entry written by this agent (optional; auto-generated if absent).
    pub decision: Option<DecisionEntry>,
}

#[derive(Debug)]
pub struct HumanPayload {
    pub html: String,
    pub css: Option<String>,
    pub assets: HashMap<String, String>,
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
            "data-update" | "designed" | "full-html" => {}
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

        let human = if mode == "full-html" {
            let html = v["human"]["html"]
                .as_str()
                .ok_or_else(|| Error::OutputRejected("full-html missing human.html".into()))?
                .to_string();
            let css = v["human"]["css"].as_str().map(|s| s.to_string());
            let assets = if let Some(obj) = v["human"]["assets"].as_object() {
                obj.iter()
                    .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            } else {
                HashMap::new()
            };
            Some(HumanPayload { html, css, assets })
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
                // Strip only <script> tags — allow full HTML, external fonts/styles.
                // Agents have creative freedom; the viewer sandbox controls execution.
                let clean_html = strip_scripts(&h.html);
                builder.add_entry("human/index.html", clean_html.into_bytes());
                if let Some(css) = h.css {
                    builder.add_entry("human/styles.css", css.into_bytes());
                }
                for (name, content) in h.assets {
                    builder.add_entry(format!("human/assets/{name}"), content.into_bytes());
                }
                // Register new human entries in the manifest.
                let m = builder.manifest_mut();
                m.files.push(FileEntry {
                    id: "human-view".into(),
                    path: "human/index.html".into(),
                    role: "human-view".into(),
                    content_type: "text/html".into(),
                    priority: Some(1),
                    sha256: None, // populated by build()
                });
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
