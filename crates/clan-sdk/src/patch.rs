//! human/patches.yaml — read and write human edit patches (spec §11).

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::container::{ClanBuilder, ClanFile};
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

/// Apply a single human patch to an open ClanFile, repack, and return the
/// updated archive bytes. The caller is responsible for writing them to disk.
pub fn apply_patch_and_repack(
    clan: &ClanFile,
    id: String,
    content: String,
) -> Result<Vec<u8>> {
    // Load existing patches (or start fresh).
    let mut patches = if clan.has_entry("human/patches.yaml") {
        let bytes = clan.read_entry("human/patches.yaml")?;
        Patches::from_yaml(&bytes)?
    } else {
        Patches::default()
    };

    patches.upsert(id, content, "human".to_string());
    let patches_yaml = patches.to_yaml()?;

    // Rebuild the archive with updated patches.yaml.
    let mut builder = ClanBuilder::new(clan.manifest().clone());

    for path in clan.entry_paths()? {
        if path == "manifest.yaml" || path == "human/patches.yaml" {
            continue;
        }
        if let Ok(bytes) = clan.read_entry(&path) {
            builder.add_entry(path, bytes);
        }
    }
    builder.add_entry("human/patches.yaml", patches_yaml);

    builder.build()
}
