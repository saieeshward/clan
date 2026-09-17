// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! human/patches.yaml — read and write human edit patches (spec §11).

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::container::{ClanBuilder, ClanFile};
use crate::decision::{Decision, DecisionChain};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Patches {
    #[serde(default)]
    pub patches: Vec<Patch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub id: String,
    pub content: String,
    pub edited_at: String,
    pub edited_by: String,
}

impl Patches {
    pub fn from_yaml(bytes: &[u8]) -> Result<Self> {
        Ok(serde_yaml::from_slice(bytes).unwrap_or_default())
    }

    pub fn to_yaml(&self) -> Result<Vec<u8>> {
        Ok(serde_yaml::to_string(self)?.into_bytes())
    }

    /// Upsert a patch by id. If an entry with the same id exists, update it;
    /// otherwise append a new one.
    pub fn upsert(&mut self, id: String, content: String, edited_by: String) {
        let now = Utc::now().to_rfc3339();
        if let Some(existing) = self.patches.iter_mut().find(|p| p.id == id) {
            existing.content = content;
            existing.edited_at = now;
            existing.edited_by = edited_by;
        } else {
            self.patches.push(Patch {
                id,
                content,
                edited_at: now,
                edited_by,
            });
        }
    }
}

/// Path of the shared decision chain a human edit is logged to.
const CHAIN_PATH: &str = "agent/decision-chain.yaml";

/// Longest edit excerpt kept in the chain rationale. Patches hold the full
/// text; the chain entry is provenance, so it stays short.
const RATIONALE_EXCERPT: usize = 200;

/// Build the decision-chain entry that records one human edit, so agents
/// (and the viewer's Decisions tab) see human changes alongside agent ones
/// (spec §12 "Agent Access to Patches").
fn human_edit_decision(id: &str, content: &str, timestamp: String) -> Decision {
    let trimmed = content.trim();
    let excerpt: String = trimmed.chars().take(RATIONALE_EXCERPT).collect();
    let rationale = if excerpt.chars().count() < trimmed.chars().count() {
        format!("{id}: {excerpt}…")
    } else {
        format!("{id}: {excerpt}")
    };
    Decision {
        agent: "human".into(),
        version: None,
        action: format!("edited {id} in the viewer"),
        rationale,
        timestamp,
        fields_changed: Vec::new(),
        pinned: false,
        trace_ref: None,
    }
}

/// Apply a single human patch to an open ClanFile, repack, and return the
/// updated archive bytes. The caller is responsible for writing them to disk.
///
/// Besides upserting `human/patches.yaml`, the edit is logged as a `human`
/// entry at the top of `agent/decision-chain.yaml` so the change is visible
/// to the next agent and in the viewer's agent panel. The manifest is left
/// untouched: a human edit is out-of-band and does not start a new
/// generation.
pub fn apply_patch_and_repack(clan: &ClanFile, id: String, content: String) -> Result<Vec<u8>> {
    // Load existing patches (or start fresh).
    let mut patches = if clan.has_entry("human/patches.yaml") {
        let bytes = clan.read_entry("human/patches.yaml")?;
        Patches::from_yaml(&bytes)?
    } else {
        Patches::default()
    };

    let now = Utc::now().to_rfc3339();
    patches.upsert(id.clone(), content.clone(), "human".to_string());
    let patches_yaml = patches.to_yaml()?;

    // Log the edit in the shared chain. A missing or unparseable chain is
    // left alone rather than failing the save — the patch itself is the
    // primary record and must never be lost over provenance.
    let chain_yaml = match clan.read_entry(CHAIN_PATH) {
        Ok(bytes) => match DecisionChain::from_yaml(&bytes) {
            Ok(mut chain) => {
                chain.prepend(human_edit_decision(&id, &content, now));
                Some(chain.to_yaml()?)
            }
            Err(_) => None,
        },
        Err(_) => None,
    };

    // Rebuild the archive with updated patches.yaml (and chain).
    let mut builder = ClanBuilder::new(clan.manifest().clone());

    for (path, bytes) in clan.read_all_entries()? {
        if path == "manifest.yaml" || path == "human/patches.yaml" {
            continue;
        }
        if chain_yaml.is_some() && path == CHAIN_PATH {
            continue;
        }
        builder.add_entry(path, bytes);
    }
    builder.add_entry("human/patches.yaml", patches_yaml);
    if let Some(yaml) = chain_yaml {
        builder.add_entry(CHAIN_PATH, yaml);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_clan() -> ClanFile {
        let bytes = crate::create(crate::CreateOptions {
            title: "Patch Test".into(),
            brief: "brief".into(),
            document_type: None,
            no_render: false,
            schema: None,
        })
        .unwrap();
        ClanFile::from_bytes(bytes).unwrap()
    }

    fn chain_of(clan: &ClanFile) -> DecisionChain {
        DecisionChain::from_yaml(&clan.read_entry(CHAIN_PATH).unwrap()).unwrap()
    }

    #[test]
    fn human_edit_is_logged_in_decision_chain() {
        let parent = test_clan();
        let before = chain_of(&parent).decisions.len();

        let next = ClanFile::from_bytes(
            apply_patch_and_repack(&parent, "heading-0".into(), "Amended Title".into()).unwrap(),
        )
        .unwrap();

        let chain = chain_of(&next);
        assert_eq!(
            chain.decisions.len(),
            before + 1,
            "one edit adds one chain entry"
        );
        let top = &chain.decisions[0];
        assert_eq!(top.agent, "human");
        assert!(top.action.contains("heading-0"));
        assert!(top.rationale.contains("Amended Title"));
        assert!(!top.timestamp.is_empty());

        // The patch itself is still recorded.
        let patches = Patches::from_yaml(&next.read_entry("human/patches.yaml").unwrap()).unwrap();
        assert_eq!(patches.patches.len(), 1);
        assert_eq!(patches.patches[0].content, "Amended Title");
    }

    #[test]
    fn human_edit_does_not_start_a_new_generation() {
        let parent = test_clan();
        let next = ClanFile::from_bytes(
            apply_patch_and_repack(&parent, "heading-0".into(), "x".into()).unwrap(),
        )
        .unwrap();
        assert_eq!(next.manifest().id, parent.manifest().id);
    }

    #[test]
    fn long_edit_is_excerpted_in_rationale_without_panicking() {
        let parent = test_clan();
        let content = "é".repeat(RATIONALE_EXCERPT + 50);
        let next = ClanFile::from_bytes(
            apply_patch_and_repack(&parent, "para-0".into(), content).unwrap(),
        )
        .unwrap();
        let top = chain_of(&next).decisions[0].clone();
        assert!(top.rationale.ends_with('…'));
        assert!(top.rationale.chars().count() <= RATIONALE_EXCERPT + "para-0: …".len());
    }
}
