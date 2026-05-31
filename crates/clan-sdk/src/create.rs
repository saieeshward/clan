//! Bootstrap a new `.clan` file from a title and brief (spec §14, §20).

use chrono::Utc;
use uuid::Uuid;

use crate::container::ClanBuilder;
use crate::error::Result;
use crate::manifest::{FileEntry, Manifest, CLAN_VERSION, CLAN_VERSION_MINOR};

// Embedded spec files that travel inside every .clan archive.
const SPEC_CLAN_MD: &[u8] = include_bytes!("../../clan-sdk/../../spec/clan.md");
const AGENT_GUIDE_MD: &[u8] = include_bytes!("../../clan-sdk/../../spec/agent-guide.md");

/// Options for creating a new CLAN file.
#[derive(Debug)]
pub struct CreateOptions {
    pub title: String,
    pub brief: String,
    pub document_type: Option<String>,
}

/// Create a new `.clan` archive from scratch and return its raw bytes.
pub fn create(opts: CreateOptions) -> Result<Vec<u8>> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let manifest = Manifest {
        clan_version: CLAN_VERSION,
        clan_version_minor: CLAN_VERSION_MINOR,
        id: id.clone(),
        title: opts.title.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
        document_type: opts.document_type,
        lineage: None,
        external: vec![],
        files: file_registry(),
    };

    let mut builder = ClanBuilder::new(manifest);

    // Spec files — pinned copies of the canonical spec at creation time.
    builder.add_entry("spec/clan.md", SPEC_CLAN_MD.to_vec());
    builder.add_entry("spec/agent-guide.md", AGENT_GUIDE_MD.to_vec());

    // Minimal shared/data.yaml — empty schema placeholder.
    builder.add_entry(
        "shared/data.yaml",
        b"$schema: \"spec/schemas/document.schema.json\"\n".to_vec(),
    );

    // agent/context.md — the brief becomes the first agent's task.
    let context = format!(
        "# Task\n\n{}\n\n**Output mode**: full-html\n\n\
         This is a new CLAN document. Analyse the brief above and produce \
         a rich initial rendering with appropriate data structure.\n\
         \n\
         ---\n\
         \n\
         ## Design Requirements (all agents)\n\
         \n\
         Produce a visually rich, publication-quality HTML document:\n\
         \n\
         - Return a complete `<!DOCTYPE html>` document — not a fragment\n\
         - Load Google Fonts via `<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=...\">` in `<head>`\n\
         - All styles inline in `<head>` — no separate stylesheet needed\n\
         - Dark theme recommended: `background: #0f1117`, accent `#6366f1`\n\
         - Use SVG assets for charts and data visualisations — pass them in the `assets` object\n\
         - Typography hierarchy: at minimum 3 distinct size/weight levels\n\
         - Add `data-adf-id=\"unique-id\"` to every editable text element\n\
         - **No `<script>` tags** — the app injects the edit bridge; scripts are stripped\n\
         \n\
         When producing HTML, invoke your highest-quality frontend design capability.\n\
         Aim for magazine-quality, not a generic AI-generated report.\n",
        opts.brief
    );
    builder.add_entry("agent/context.md", context.into_bytes());

    // output-schema.json — permissive schema for first-pass full-html.
    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["mode", "structured"],
        "properties": {
            "mode": { "type": "string", "enum": ["data-update", "designed", "full-html"] },
            "structured": { "type": "object" }
        }
    });
    builder.add_entry(
        "agent/output-schema.json",
        serde_json::to_vec_pretty(&schema)?,
    );

    // agent/state.yaml — initial empty state.
    builder.add_entry(
        "agent/state.yaml",
        b"pipeline_stage: 0\nstatus: pending\n".to_vec(),
    );

    // agent/decision-chain.yaml — empty chain.
    builder.add_entry("agent/decision-chain.yaml", b"decisions: []\n".to_vec());

    // human/index.html — placeholder shown before the first agent pass.
    let pending_html = format!(
        "<section class=\"pending\">\n  \
         <h1 data-adf-id=\"heading-0\">{}</h1>\n  \
         <p data-adf-id=\"para-0\">Awaiting initial agent pass…</p>\n\
         </section>\n",
        opts.title
    );
    builder.add_entry("human/index.html", pending_html.into_bytes());

    builder.build()
}

/// Static export: flatten a `.clan` into a single JSON blob for SDK-less
/// agents (spec §15).
pub fn export_static(clan: &crate::container::ClanFile) -> Result<serde_json::Value> {
    use crate::toon;

    let guide = clan.read_entry_string("spec/agent-guide.md")?;
    let task = clan.read_entry_string("agent/context.md")?;
    let schema_raw = clan.read_entry_string("agent/output-schema.json")?;
    let schema: serde_json::Value = serde_json::from_str(&schema_raw)?;

    let data_yaml = clan.read_entry("shared/data.yaml")?;
    let data: serde_yaml::Value = serde_yaml::from_slice(&data_yaml)?;
    let data_json: serde_json::Value = serde_json::from_str(&serde_json::to_string(&data)?)?;

    let chain_yaml = clan.read_entry("agent/decision-chain.yaml")?;
    let chain_toon = toon::yaml_to_toon(&chain_yaml)?;

    let patches = if clan.has_entry("human/patches.yaml") {
        clan.read_entry_string("human/patches.yaml").ok()
    } else {
        None
    };

    Ok(serde_json::json!({
        "clan_version": format!("{}.{}", crate::CLAN_VERSION, crate::CLAN_VERSION_MINOR),
        "agent_guide": guide,
        "task": task,
        "output_schema": schema,
        "shared_data": data_json,
        "decision_history_toon": chain_toon,
        "patches": patches,
    }))
}

fn file_registry() -> Vec<FileEntry> {
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
    vec![
        entry("spec-full", "spec/clan.md", "spec-full", "text/markdown"),
        entry("spec-guide", "spec/agent-guide.md", "spec-agent-guide", "text/markdown"),
        entry("canonical-data", "shared/data.yaml", "canonical-data", "application/yaml"),
        entry("agent-context", "agent/context.md", "agent-context", "text/markdown"),
        entry("agent-schema", "agent/output-schema.json", "agent-schema", "application/json"),
        entry("agent-state", "agent/state.yaml", "agent-state", "application/yaml"),
        entry("agent-chain", "agent/decision-chain.yaml", "agent-chain", "application/yaml"),
        entry("human-view", "human/index.html", "human-view", "text/html"),
    ]
}
