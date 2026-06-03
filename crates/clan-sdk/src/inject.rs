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
}

impl Default for InjectOptions {
    fn default() -> Self {
        Self {
            include_patches: true,
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

    // 1. Format guide (always injected first).
    let guide = clan.read_entry_string("spec/agent-guide.md")?;
    parts.push(section("# CLAN Agent Guide", &guide));

    // 2. Task context.
    let task = clan.read_entry_string("agent/context.md")?;
    parts.push(section("# Your Task", &task));

    // 3. Output contract.
    let schema_raw = clan.read_entry_string("agent/output-schema.json")?;
    parts.push(section("# Output Schema (return JSON matching this exactly)", &schema_raw));

    // 4. Canonical data → TOON.
    let data_yaml = clan.read_entry("shared/data.yaml")?;
    let data_toon = toon::yaml_to_toon(&data_yaml)?;
    parts.push(section("# Document Data (TOON)", &data_toon));

    // 5. Decision chain → TOON.
    let chain_yaml = clan.read_entry("agent/decision-chain.yaml")?;
    let chain_toon = toon::yaml_to_toon(&chain_yaml)?;
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
