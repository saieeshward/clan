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

    // 1b. Situational blocks (spec §26) — selected by file state, appended
    // after the byte-stable guide so an agent never reads about a capability
    // its current file does not exhibit. A sequential agent on an unforked,
    // conflict-free file gets exactly the v1.0 injection.
    if let Some(fork) = &clan.manifest().fork {
        parts.push(section(
            "# Branch Mode (this file is forked)",
            &format!(
                "You are branch agent `{agent}`. Write ONLY inside `{ns}`:\n\
                 - data: `clan patch-data <file> <json> --namespace` (routes to {ns}data.yaml)\n\
                 - decisions: `clan patch-decision` (auto-routed to {ns}decisions.yaml)\n\
                 Writes to shared/ are rejected until the branches are joined with `clan merge`.\n\
                 Your namespace folds into shared/data.yaml via the manifest merge policies.\n\
                 Naming: pick distinct, specific keys. If you write a prose/narrative field a \
                 sibling is also likely to write (e.g. `assumptions`, `summary`, `notes`), the \
                 default last-write-wins fold will keep only one — prefix it with your agent id \
                 (e.g. `assumptions_{agent}`) or expect the merge to flag it as contested.",
                agent = fork.agent_id,
                ns = fork.namespace,
            ),
        ));
    }
    if clan.has_entry(crate::merge::MERGE_REPORT_PATH) {
        if let Ok(report) =
            crate::merge::MergeReport::from_yaml(&clan.read_entry(crate::merge::MERGE_REPORT_PATH)?)
        {
            if report.unresolved > 0 {
                let report_toon =
                    toon::yaml_to_toon(&clan.read_entry(crate::merge::MERGE_REPORT_PATH)?)?;
                parts.push(section(
                    "# Contested Keys (merge report — adjudication pending)",
                    &format!(
                        "{report_toon}\n\
                         These keys were contested when parallel branches merged; the listed winner \
                         was picked by policy and currently sits in the data. To adjudicate a key: \
                         `clan patch-data` with your chosen value, then `clan patch-decision` \
                         recording why."
                    ),
                ));
            }
        }
    }
    if let Some(view) = &clan.manifest().view {
        if !view.present && view.renderable {
            parts.push(section(
                "# Human View",
                "No human view exists in this file (agent-only chain). Produce one with \
                 `clan render <file>` or `clan pack-html` only if your task requires it.",
            ));
        }
    }
    if clan.has_entry("agent/requirements.yaml") {
        let req_toon = toon::yaml_to_toon(&clan.read_entry("agent/requirements.yaml")?)?;
        parts.push(section(
            "# Capability Requirements (agent/requirements.yaml)",
            &req_toon,
        ));
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
        clan_with_guide(GUIDE, schema, chain_yaml)
    }

    fn clan_with_guide(guide: &str, schema: &str, chain_yaml: &str) -> ClanFile {
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
            view: None,
            fork: None,
            merge_policies: None,
            external: vec![],
            files: vec![
                entry(
                    "spec-guide",
                    "spec/agent-guide.md",
                    "spec-agent-guide",
                    "text/markdown",
                ),
                entry(
                    "agent-context",
                    "agent/context.md",
                    "agent-context",
                    "text/markdown",
                ),
                entry(
                    "agent-schema",
                    "agent/output-schema.json",
                    "agent-schema",
                    "application/json",
                ),
                entry(
                    "canonical-data",
                    "shared/data.yaml",
                    "canonical-data",
                    "application/yaml",
                ),
                entry(
                    "agent-chain",
                    "agent/decision-chain.yaml",
                    "agent-chain",
                    "application/yaml",
                ),
            ],
        };
        let mut builder = ClanBuilder::new(manifest);
        builder.add_entry("spec/agent-guide.md", guide.as_bytes().to_vec());
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
            ctx.text
                .contains("# Output Schema (TOON-encoded JSON Schema"),
            "{}",
            ctx.text
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
            ctx.text
                .contains("# Output Schema (return JSON matching this exactly)"),
            "{}",
            ctx.text
        );
        assert!(
            ctx.text.contains(schema),
            "raw schema must be injected verbatim"
        );
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
        let opts = InjectOptions {
            skip_guide: true,
            ..Default::default()
        };
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
        let opts = InjectOptions {
            skip_guide: true,
            ..Default::default()
        };
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

    // --- #25 adversarial: window boundaries ---
    // Sentinel field names ("zq-sent-N") are deliberately unique so the
    // contains/!contains assertions cannot collide with other context text
    // (agent names, headings, schema keys, ...).

    /// Chain of `n` decisions, newest first, each changing one unique
    /// sentinel field.
    fn sentinel_chain(n: usize) -> String {
        let mut chain = String::from("decisions:\n");
        for i in 0..n {
            let field = format!("zq-sent-{i}");
            chain.push_str(&decision_yaml(i, &[&field]));
        }
        chain
    }

    #[test]
    fn chains_shorter_than_window_keep_all_fields_changed() {
        for n in 1..=4 {
            let clan = clan_with(SIMPLE_SCHEMA, &sentinel_chain(n));
            let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
            for i in 0..n {
                assert!(
                    ctx.text.contains(&format!("zq-sent-{i}")),
                    "chain of {n}: entry {i} must keep fields_changed:\n{}",
                    ctx.text
                );
            }
        }
    }

    #[test]
    fn chain_of_exactly_window_size_keeps_everything() {
        let clan = clan_with(SIMPLE_SCHEMA, &sentinel_chain(FIELDS_CHANGED_WINDOW));
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        for i in 0..FIELDS_CHANGED_WINDOW {
            assert!(
                ctx.text.contains(&format!("zq-sent-{i}")),
                "entry {i} of an exactly-window-sized chain must be kept:\n{}",
                ctx.text
            );
        }
    }

    #[test]
    fn chain_of_window_plus_one_prunes_only_the_oldest() {
        let clan = clan_with(SIMPLE_SCHEMA, &sentinel_chain(FIELDS_CHANGED_WINDOW + 1));
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        for i in 0..FIELDS_CHANGED_WINDOW {
            assert!(
                ctx.text.contains(&format!("zq-sent-{i}")),
                "entry {i} is inside the window and must be kept:\n{}",
                ctx.text
            );
        }
        assert!(
            !ctx.text
                .contains(&format!("zq-sent-{FIELDS_CHANGED_WINDOW}")),
            "entry {FIELDS_CHANGED_WINDOW} (first outside the window) must be pruned:\n{}",
            ctx.text
        );
    }

    // --- #25 adversarial: missing key, pinned entries, name collisions ---

    #[test]
    fn missing_fields_changed_key_is_neither_panic_nor_invented() {
        // Decision serialisation skips empty fields_changed entirely
        // (#[serde(skip_serializing_if = "Vec::is_empty")]), so real chains
        // routinely lack the key. Both recent and old entries here omit it.
        let mut chain = String::from("decisions:\n");
        for i in 0..7 {
            chain.push_str(&format!(
                "- agent: agent{i}\n  action: act{i}\n  rationale: r{i}\n  timestamp: \"2026-06-01T00:00:00Z\"\n"
            ));
        }
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            !ctx.text.contains("fields_changed"),
            "the key must not be invented for entries that never had it:\n{}",
            ctx.text
        );
        // All seven entries are still present. (Space-free uniform entries
        // encode columnar — `agentN` is a row token rather than `agent: agentN`
        // — so assert on the value, which appears either way.)
        for i in 0..7 {
            assert!(ctx.text.contains(&format!("agent{i}")), "{}", ctx.text);
        }
    }

    #[test]
    fn pinned_old_entries_currently_lose_fields_changed_too() {
        // CURRENT BEHAVIOR, asserted on purpose: pruning is purely positional
        // (index >= FIELDS_CHANGED_WINDOW); `pinned: true` does NOT exempt an
        // old entry's fields_changed. If pinning is ever meant to preserve
        // them, this test must be consciously rewritten.
        let mut chain = sentinel_chain(5);
        chain.push_str(
            "- agent: agent-old\n  action: act-old\n  rationale: r-old\n  timestamp: \"2026-05-01T00:00:00Z\"\n  pinned: true\n  fields_changed: [zq-pinned-sentinel]\n",
        );
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            !ctx.text.contains("zq-pinned-sentinel"),
            "pinned-but-old fields_changed is pruned under the current design:\n{}",
            ctx.text
        );
        // The pinned entry itself (and its flag) survive — only the list goes.
        assert!(ctx.text.contains("agent: agent-old"), "{}", ctx.text);
        assert!(ctx.text.contains("pinned: true"), "{}", ctx.text);
    }

    #[test]
    fn pruning_with_field_names_colliding_with_context_text() {
        // A field literally named "agent" (also a decision key and a word in
        // the guide/context) must not confuse pruning. The recent entry keeps
        // it; the old entry's unique sentinel proves old lists still go.
        let mut chain = String::from("decisions:\n");
        chain.push_str(&decision_yaml(0, &["agent", "zq-recent-sentinel"]));
        for i in 1..5 {
            chain.push_str(&decision_yaml(i, &[]));
        }
        chain.push_str(&decision_yaml(5, &["agent", "zq-old-sentinel"]));
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            ctx.text.contains("fields_changed [2]"),
            "recent two-element list must survive intact:\n{}",
            ctx.text
        );
        assert!(ctx.text.contains("zq-recent-sentinel"), "{}", ctx.text);
        assert!(
            !ctx.text.contains("zq-old-sentinel"),
            "old list must be pruned even when its sibling values collide with other text:\n{}",
            ctx.text
        );
    }

    // --- #25 adversarial: malformed-ish chains must not panic ---

    #[test]
    fn scalar_items_in_decisions_list_do_not_panic() {
        // Valid YAML, wrong shape: scalars instead of mappings.
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: [one, 2, true, null]\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(ctx.text.contains("decisions [4]"), "{}", ctx.text);
    }

    #[test]
    fn mixed_scalars_and_mappings_in_decisions_do_not_panic() {
        let chain = format!(
            "decisions:\n- just-a-string\n{}- 42\n",
            decision_yaml(1, &["zq-mixed-sentinel"])
        );
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        // The mapping entry (index 1, inside the window) keeps its list.
        assert!(ctx.text.contains("zq-mixed-sentinel"), "{}", ctx.text);
    }

    #[test]
    fn empty_decisions_list_does_not_panic() {
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(ctx.text.contains("decisions [0]"), "{}", ctx.text);
        assert!(!ctx.text.contains("fields_changed"), "{}", ctx.text);
    }

    #[test]
    fn empty_mapping_chain_does_not_panic() {
        let clan = clan_with(SIMPLE_SCHEMA, "{}\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            ctx.text.contains("# Decision History (TOON)"),
            "{}",
            ctx.text
        );
    }

    #[test]
    fn non_sequence_decisions_value_does_not_panic() {
        for chain in [
            "decisions: nope\n",
            "decisions: {a: 1}\n",
            "decisions: null\n",
        ] {
            let clan = clan_with(SIMPLE_SCHEMA, chain);
            assemble(&clan, &InjectOptions::default())
                .unwrap_or_else(|e| panic!("chain {chain:?} must assemble: {e}"));
        }
    }

    #[test]
    fn non_list_fields_changed_value_does_not_panic() {
        // fields_changed holding a scalar instead of a list: not "empty", so
        // it is kept inside the window and dropped outside it — never a panic.
        let mut chain = String::from(
            "decisions:\n- agent: a0\n  action: x\n  rationale: r\n  timestamp: t\n  fields_changed: zq-scalar-recent\n",
        );
        for i in 1..5 {
            chain.push_str(&decision_yaml(i, &[]));
        }
        chain.push_str(
            "- agent: a5\n  action: x\n  rationale: r\n  timestamp: t\n  fields_changed: zq-scalar-old\n",
        );
        let clan = clan_with(SIMPLE_SCHEMA, &chain);
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(ctx.text.contains("zq-scalar-recent"), "{}", ctx.text);
        assert!(!ctx.text.contains("zq-scalar-old"), "{}", ctx.text);
    }

    #[test]
    fn truly_invalid_yaml_chain_is_an_error_not_a_panic() {
        let clan = clan_with(SIMPLE_SCHEMA, "decisions: [unclosed\n");
        assert!(assemble(&clan, &InjectOptions::default()).is_err());
    }

    // --- #24 adversarial: digest tracks guide content ---

    #[test]
    fn skip_guide_digest_tracks_guide_content() {
        let opts = InjectOptions {
            skip_guide: true,
            ..Default::default()
        };

        let guide_a = "guide version A";
        let guide_b = "guide version B — content changed";
        let ctx_a = assemble(
            &clan_with_guide(guide_a, SIMPLE_SCHEMA, "decisions: []\n"),
            &opts,
        )
        .unwrap();
        let ctx_b = assemble(
            &clan_with_guide(guide_b, SIMPLE_SCHEMA, "decisions: []\n"),
            &opts,
        )
        .unwrap();

        let digest_a = crate::hash::sha256_prefixed(guide_a.as_bytes());
        let digest_b = crate::hash::sha256_prefixed(guide_b.as_bytes());
        assert_ne!(digest_a, digest_b);
        assert!(ctx_a.text.contains(&digest_a), "{}", ctx_a.text);
        assert!(ctx_b.text.contains(&digest_b), "{}", ctx_b.text);
        assert!(
            !ctx_a.text.contains(&digest_b) && !ctx_b.text.contains(&digest_a),
            "each note must carry exactly its own guide's digest"
        );
        // Neither body leaks into the skipped view.
        assert!(!ctx_a.text.contains(guide_a) && !ctx_b.text.contains(guide_b));
    }

    // --- #23 adversarial: schema injection edge cases ---

    #[test]
    fn invalid_json_schema_falls_back_to_raw_injection() {
        // assemble() never validates the schema file; a broken one must be
        // injected raw, not turn into an error or panic.
        let schema = "{not json";
        let clan = clan_with(schema, "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            ctx.text
                .contains("# Output Schema (return JSON matching this exactly)"),
            "{}",
            ctx.text
        );
        assert!(ctx.text.contains(schema), "{}", ctx.text);
        assert_eq!(ctx.output_schema_json, schema);
    }

    #[test]
    fn empty_object_schema_falls_back_to_raw() {
        // "{}" TOON-encodes to an empty string, which the verifier rejects;
        // the raw two bytes must be injected instead of an empty section.
        let clan = clan_with("{}", "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        assert!(
            ctx.text
                .contains("# Output Schema (return JSON matching this exactly)\n\n{}"),
            "{}",
            ctx.text
        );
        assert_eq!(ctx.output_schema_json, "{}");
    }

    #[test]
    fn unicode_schema_keys_are_handled() {
        let schema = r#"{
  "type": "object",
  "properties": {
    "tîtré": {"type": "string"},
    "金額": {"type": "number"}
  }
}"#;
        let clan = clan_with(schema, "decisions: []\n");
        let ctx = assemble(&clan, &InjectOptions::default()).unwrap();
        // Whichever injection path wins, the unicode keys must appear and the
        // validation contract must stay byte-identical.
        assert!(ctx.text.contains("tîtré"), "{}", ctx.text);
        assert!(ctx.text.contains("金額"), "{}", ctx.text);
        assert_eq!(ctx.output_schema_json, schema);
    }

    // --- #23/#24 interaction: the validation contract is inviolable ---

    #[test]
    fn output_schema_json_is_always_the_raw_bytes() {
        // Every combination of guide handling and schema path (TOON-able,
        // ambiguous, invalid JSON) must surface the raw file content.
        let ambiguous = r#"{"type": "object", "properties": {"code": {"enum": ["123"]}}}"#;
        for schema in [SIMPLE_SCHEMA, ambiguous, "{not json", "{}"] {
            for skip_guide in [false, true] {
                for include_patches in [false, true] {
                    let clan = clan_with(schema, "decisions: []\n");
                    let opts = InjectOptions {
                        include_patches,
                        skip_guide,
                    };
                    let ctx = assemble(&clan, &opts).unwrap();
                    assert_eq!(
                        ctx.output_schema_json, schema,
                        "raw schema bytes lost (skip_guide={skip_guide}, include_patches={include_patches})"
                    );
                }
            }
        }
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
            let Some(map) = decision.as_mapping_mut() else {
                continue;
            };
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
