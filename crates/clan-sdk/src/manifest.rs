// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `manifest.yaml` model — the root index of every CLAN file (spec §5).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Major format version this SDK produces and supports.
pub const CLAN_VERSION: u32 = 1;
/// Minor format version this SDK produces.
pub const CLAN_VERSION_MINOR: u32 = 0;

/// The root `manifest.yaml` structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub clan_version: u32,
    pub clan_version_minor: u32,
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<Lineage>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external: Vec<ExternalRef>,

    pub files: Vec<FileEntry>,
}

/// Lineage block — records the parent CLAN file (spec §12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    pub parent_id: String,
    pub parent_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_sha256: Option<String>,
    pub delta: String,
}

/// A single entry in the file registry (spec §5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub path: String,
    pub role: String,
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// External persistent store reference (spec §13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRef {
    pub id: String,
    pub uri: String,
    #[serde(rename = "type")]
    pub store_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub access: String,
}

impl Manifest {
    /// Parse a manifest from YAML bytes.
    pub fn from_yaml(bytes: &[u8]) -> Result<Self> {
        let manifest: Manifest = serde_yaml::from_slice(bytes)?;
        Ok(manifest)
    }

    /// Serialise the manifest to YAML bytes.
    pub fn to_yaml(&self) -> Result<Vec<u8>> {
        Ok(serde_yaml::to_string(self)?.into_bytes())
    }

    /// Look up a file entry by its stable `id`.
    pub fn file_by_id(&self, id: &str) -> Option<&FileEntry> {
        self.files.iter().find(|f| f.id == id)
    }

    /// Look up a file entry by its archive `path`.
    pub fn file_by_path(&self, path: &str) -> Option<&FileEntry> {
        self.files.iter().find(|f| f.path == path)
    }

    /// All file entries declared with a given `role`.
    pub fn files_with_role<'a>(&'a self, role: &'a str) -> impl Iterator<Item = &'a FileEntry> {
        self.files.iter().filter(move |f| f.role == role)
    }

    /// Check required fields are present and well-formed. Returns a list of
    /// human-readable problems (empty == valid).
    pub fn structural_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();

        if self.id.trim().is_empty() {
            problems.push("manifest.id is empty".into());
        } else if !is_uuid_v4(&self.id) {
            problems.push(format!("manifest.id is not a valid UUID v4: {}", self.id));
        }
        if self.title.trim().is_empty() {
            problems.push("manifest.title is empty".into());
        }
        if self.created_at.trim().is_empty() {
            problems.push("manifest.created_at is empty".into());
        }
        if self.updated_at.trim().is_empty() {
            problems.push("manifest.updated_at is empty".into());
        }
        if self.files.is_empty() {
            problems.push("manifest.files registry is empty".into());
        }
        if let Some(lineage) = &self.lineage {
            if !is_uuid_v4(&lineage.parent_id) {
                problems.push(format!(
                    "lineage.parent_id is not a valid UUID v4: {}",
                    lineage.parent_id
                ));
            }
            if lineage.parent_id == self.id {
                problems.push("a CLAN must not reference itself as its own parent".into());
            }
            if let Some(ps) = &lineage.parent_sha256 {
                if !crate::hash::is_valid_prefixed(ps) {
                    problems.push(format!(
                        "lineage.parent_sha256 is malformed (expected sha256:<64 hex>): {ps}"
                    ));
                }
            }
        }
        problems
    }

    /// Validate or error.
    pub fn validate(&self) -> Result<()> {
        let problems = self.structural_problems();
        if problems.is_empty() {
            Ok(())
        } else {
            Err(Error::InvalidManifest(problems.join("; ")))
        }
    }
}

/// Loose UUID v4 check: 8-4-4-4-12 hex with version nibble `4` and a valid
/// variant nibble. Good enough for validation without pulling parsing in.
pub fn is_uuid_v4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let lens = [8, 4, 4, 4, 12];
    for (part, want) in parts.iter().zip(lens.iter()) {
        if part.len() != *want || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    // version nibble (first char of 3rd group) must be '4'
    let version_ok = parts[2].chars().next() == Some('4');
    // variant nibble (first char of 4th group) must be 8, 9, a, or b
    let variant_ok = matches!(
        parts[3].chars().next().map(|c| c.to_ascii_lowercase()),
        Some('8') | Some('9') | Some('a') | Some('b')
    );
    version_ok && variant_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_v4_checks() {
        assert!(is_uuid_v4("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_uuid_v4("550e8400-e29b-11d4-a716-446655440000")); // v1
        assert!(!is_uuid_v4("not-a-uuid"));
    }

    #[test]
    fn roundtrip_yaml() {
        let m = Manifest {
            clan_version: 1,
            clan_version_minor: 0,
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            title: "Test".into(),
            created_at: "2026-05-31T10:00:00Z".into(),
            updated_at: "2026-05-31T10:00:00Z".into(),
            document_type: Some("invoice".into()),
            lineage: None,
            external: vec![],
            files: vec![FileEntry {
                id: "canonical-data".into(),
                path: "shared/data.yaml".into(),
                role: "canonical-data".into(),
                content_type: "application/yaml".into(),
                priority: None,
                sha256: Some("sha256:abc".into()),
            }],
        };
        let bytes = m.to_yaml().unwrap();
        let back = Manifest::from_yaml(&bytes).unwrap();
        assert_eq!(back.id, m.id);
        assert_eq!(back.files[0].content_type, "application/yaml");
        assert!(m.validate().is_ok());
    }
}
