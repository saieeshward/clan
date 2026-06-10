// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Agent context assembly — spec §14.
//!
//! Assembles context in the canonical injection order:
//!   1. spec/agent-guide.md   (~800 tokens, format knowledge)
//!   2. agent/context.md      (~400 tokens, task)
//!   3. agent/output-schema.json (~300 tokens, contract)
//!   4. shared/data.yaml      → TOON
//!   5. agent/decision-chain.yaml → TOON
//!   6. human/patches.yaml    (optional, if include_patches = true)
//!
//! Lazy loading: only the entries needed for agent context are read.
//! human/ entries are never read here (spec §14 lazy-loading contract).

use crate::container::ClanFile;
use crate::error::Result;
use crate::toon;

/// Options for context assembly.
#[derive(Debug, Clone)]
pub struct InjectOptions {
    /// Include `human/patches.yaml` in the assembled context.
    pub include_patches: bool,
    /// Skip the ~800-token agent guide body, replacing it with a one-line
    /// note carrying the guide's digest. For agents operating in a sequence
    /// that have already read the guide; the digest lets callers detect a
    /// guide-version change and re-read in full.
    pub skip_guide: bool,
}

impl Default for InjectOptions {
    fn default() -> Self {
        Self {
            include_patches: true,
            skip_guide: false,
        }
    }
}

/// The assembled agent context, ready to prepend to an LLM prompt.
#[derive(Debug)]
pub struct AgentContext {
    /// Full assembled context string in injection order.
    pub text: String,
    /// The raw output-schema JSON, for validation after the agent responds.
    pub output_schema_json: String,
}

/// Assemble the agent context from an open [`ClanFile`].
pub fn assemble(clan: &ClanFile, opts: &InjectOptions) -> Result<AgentContext> {
    let mut parts: Vec<String> = Vec::new();

    // 1. Format guide (injected first; skippable for sequenced agents).
    let guide_bytes = clan.read_entry("spec/agent-guide.md")?;
    if opts.skip_guide {
        let digest = crate::hash::sha256_prefixed(&guide_bytes);
        parts.push(section(
            "# CLAN Agent Guide",
            &format!("(guide body skipped — unchanged since your previous read; digest {digest})"),
        ));
    } else {
        let guide = String::from_utf8_lossy(&guide_bytes);
        parts.push(section("# CLAN Agent Guide", &guide));
    }

    // 2. Task context.
    let task = clan.read_entry_string("agent/context.md")?;
    parts.push(section("# Your Task", &task));

    // 3. Output contract. TOON-encoded when (and only when) the encoding is
    // verified lossless; otherwise the raw JSON is injected unchanged.
    let schema_raw = clan.read_entry_string("agent/output-schema.json")?;
    match toon_schema(&schema_raw) {
        Some(schema_toon) => parts.push(section(
            "# Output Schema (TOON-encoded JSON Schema — return JSON matching it exactly)",
            &schema_toon,
        )),
        None => parts.push(section(
            "# Output Schema (return JSON matching this exactly)",
            &schema_raw,
        )),
    }

    // 4. Canonical data → TOON.
    let data_yaml = clan.read_entry("shared/data.yaml")?;
    let data_toon = toon::yaml_to_toon(&data_yaml)?;
    parts.push(section("# Document Data (TOON)", &data_toon));

    // 5. Decision chain → TOON, with fields_changed noise pruned from the
    // injected view only (the archive's YAML is untouched).
    let chain_yaml = clan.read_entry("agent/decision-chain.yaml")?;
    let chain_value: serde_yaml::Value = serde_yaml::from_slice(&chain_yaml)?;
    let chain_toon = toon::to_toon(&prune_chain_for_injection(chain_value));
    parts.push(section("# Decision History (TOON)", &chain_toon));

    // 6. Human patches (optional).
    if opts.include_patches && clan.has_entry("human/patches.yaml") {
        let patches = clan.read_entry_string("human/patches.yaml")?;
        parts.push(section("# Human Edits (patches.yaml)", &patches));
    }

    let text = parts.join("\n\n---\n\n");

    Ok(AgentContext {
        text,
        output_schema_json: schema_raw,
    })
}

fn section(heading: &str, body: &str) -> String {
    format!("{heading}\n\n{body}")
}

/// TOON-encode a JSON Schema for injection, only if verified lossless and
/// actually smaller than the raw JSON.
fn toon_schema(schema_raw: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(schema_raw).ok()?;
    let toon = toon::json_to_toon_verified(&value)?;
    (toon.len() < schema_raw.len()).then_some(toon)
}

/// Number of newest decision-chain entries whose `fields_changed` lists are
/// kept in the injected TOON view. Older entries lose them as noise.
const FIELDS_CHANGED_WINDOW: usize = 5;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::ClanBuilder;
    use crate::manifest::{FileEntry, Manifest, CLAN_VERSION, CLAN_VERSION_MINOR};

    const GUIDE: &str = "GUIDE BODY MARKER — full format knowledge lives here.";
    const SIMPLE_SCHEMA: &str = r#"{
  "type": "object",
  "required": ["title"],
  "properties": {
    "title": {"type": "string"}
  }
}"#;

    fn clan_with(schema: &str, chain_yaml: &str) -> ClanFile {
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
            id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
            title: "Inject Test".into(),
            created_at: "2026-06-01T10:00:00Z".into(),
            updated_at: "2026-06-01T10:00:00Z".into(),
            document_type: None,
            lineage: None,
            external: vec![],
            files: vec![
                entry("spec-guide", "spec/agent-guide.md", "spec-agent-guide", "text/markdown"),
                entry("agent-context", "agent/context.md", "agent-context", "text/markdown"),
                entry("agent-schema", "agent/output-schema.json", "agent-schema", "application/json"),
                entry("canonical-data", "shared/data.yaml", "canonical-data", "application/yaml"),
                entry("agent-chain", "agent/decision-chain.yaml", "agent-chain", "application/yaml"),
            ],
        };
        let mut builder = ClanBuilder::new(manifest);
        builder.add_entry("spec/agent-guide.md", GUIDE.as_bytes().to_vec());
        builder.add_entry("agent/context.md", b"the task".to_vec());
        builder.add_entry("agent/output-schema.json", schema.as_bytes().to_vec());
        builder.add_entry("shared/data.yaml", b"vendor: Acme\n".to_vec());
        builder.add_entry("agent/decision-chain.yaml", chain_yaml.as_bytes().to_vec());
        ClanFile::from_bytes(builder.build().unwrap()).unwrap()
    }

    fn decision_yaml(i: usize, fields: &[&str]) -> String {
        let fields = if fields.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", fields.join(", "))
        };
        format!(
            "- agent: agent{i}\n  action: act{i}\n  rationale: r{i}\n  timestamp: \"2026-06-0{}T00:00:00Z\"\n  fields_changed: {fields}\n",
            (i % 9) + 1
        )
    }

    // --- #23: schema TOON injection ---

    #[test]
    fn schema_injected_as_toon_when_lossless() {
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            ctx.text.contains("# Output Schema (TOON-encoded JSON Schema"),
            "{}", ctx.text
        );
        assert!(ctx.text.contains("type: object"), "{}", ctx.text);
        assert!(
            !ctx.text.contains("\"properties\""),
            "raw JSON boilerplate should be gone:\n{}",
            ctx.text
        );
        // The validation contract is untouched: raw JSON, byte for byte.
        assert_eq!(ctx.output_schema_json, SIMPLE_SCHEMA);
    }

    #[test]
    fn ambiguous_schema_falls_back_to_raw_json() {
        // "123" as an enum value cannot be TOON-encoded unambiguously.
        let schema = r#"{"type": "object", "properties": {"code": {"enum": ["123"]}}}"#;
        let clan = clan_with(schema, "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            ctx.text.contains("# Output Schema (return JSON matching this exactly)"),
            "{}", ctx.text
        );
        assert!(ctx.text.contains(schema), "raw schema must be injected verbatim");
        assert_eq!(ctx.output_schema_json, schema);
    }

    // --- #24: skip-guide ---

    #[test]
    fn guide_included_in_full_by_default() {
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(ctx.text.contains(GUIDE));
    }

    #[test]
    fn skip_guide_replaces_body_with_digest_note() {
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: []\n");
        let opts = InjectOptions { skip_guide: true, ..Default::default() };
        let ctx = assemble(&clan, &opts).unwrap();

        assert!(!ctx.text.contains(GUIDE), "guide body must be skipped");
        assert!(ctx.text.contains("guide body skipped"), "{}", ctx.text);
        let digest = crate::hash::sha256_prefixed(GUIDE.as_bytes());
        assert!(
            ctx.text.contains(&digest),
            "note must carry the guide digest so version changes are detectable"
        );
        // Everything after the guide section is identical to a full read.
        let full = assemble(&clan, &InjectOptions::default()).unwrap();
        let tail = |s: &str| s.split("# Your Task").nth(1).unwrap().to_string();
        assert_eq!(tail(&ctx.text), tail(&full.text));
    }

    #[test]
    fn skip_guide_is_deterministic() {
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: []\n");
        let opts = InjectOptions { skip_guide: true, ..Default::default() };
        assert_eq!(
            assemble(&clan, &opts).unwrap().text,
            assemble(&clan, &opts).unwrap().text
        );
    }

    // --- #25: fields_changed pruning in the injected TOON view ---

    #[test]
    fn old_and_empty_fields_changed_are_pruned_from_toon_only() {
        // 7 decisions, newest first. Entry 2 has an empty list; the rest name
        // a unique field each.
        let mut chain = String::from("decisions:\n");
        for i in 0..7 {
            if i == 2 {
                chain.push_str(&decision_yaml(i, &[]));
            } else {
                let field = format!("field-{i}");
                chain.push_str(&decision_yaml(i, &[&field]));
            }
        }
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();

        // Recent window (indices 0..5) keeps non-empty lists.
        for i in [0usize, 1, 3, 4] {
            assert!(
                ctx.text.contains(&format!("field-{i}")),
                "recent entry {i} must keep fields_changed:\n{}",
                ctx.text
            );
        }
        // Old entries lose the list; empty lists vanish everywhere.
        for i in [5usize, 6] {
            assert!(
                !ctx.text.contains(&format!("field-{i}")),
                "old entry {i} must be pruned:\n{}",
                ctx.text
            );
        }
        assert!(
            !ctx.text.contains("fields_changed [0]"),
            "empty fields_changed must not appear:\n{}",
            ctx.text
        );

        // Lossless for consumers: the archive's YAML still has every field.
        assert_eq!(
            clan.read_entry_string("agent/decision-chain.yaml").unwrap(),
            chain,
            "pruning must never touch the stored decision chain"
        );
    }

    #[test]
    fn recent_fields_changed_survive_in_toon() {
        let chain = format!("decisions:\n{}", decision_yaml(0, &["alpha", "beta"]));
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(ctx.text.contains("fields_changed [2]"), "{}", ctx.text);
        assert!(ctx.text.contains("alpha"), "{}", ctx.text);
        assert!(ctx.text.contains("beta"), "{}", ctx.text);
    }
}

/// Prune `fields_changed` noise from the decision chain before TOON encoding:
/// empty lists are always dropped, and entries older than the recent window
/// drop the list entirely. Affects the injected view only — the chain YAML
/// inside the archive keeps every field.
fn prune_chain_for_injection(mut chain: serde_yaml::Value) -> serde_yaml::Value {
    let key = serde_yaml::Value::String("fields_changed".into());
    if let Some(decisions) = chain
        .as_mapping_mut()
        .and_then(|m| m.get_mut(serde_yaml::Value::String("decisions".into())))
        .and_then(|d| d.as_sequence_mut())
    {
        // Newest-first ordering: index >= window means "old".
        for (i, decision) in decisions.iter_mut().enumerate() {
            let Some(map) = decision.as_mapping_mut() else { continue };
            let empty = map
                .get(&key)
                .and_then(|v| v.as_sequence())
                .map_or(false, |s| s.is_empty());
            if empty || i >= FIELDS_CHANGED_WINDOW {
                map.remove(&key);
            }
        }
    }
    chain
}
