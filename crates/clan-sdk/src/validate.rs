// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Structural and content validation (spec §17).

use crate::container::ClanFile;
use crate::error::{Error, Result};
use crate::hash;

/// A collected set of validation problems. Empty == valid.
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub structural: Vec<String>,
    pub content: Vec<String>,
    pub integrity: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.structural.is_empty() && self.integrity.is_empty()
    }

    pub fn is_content_valid(&self) -> bool {
        self.is_valid() && self.content.is_empty()
    }

    /// Return Err with a summary if structurally invalid.
    pub fn require_valid(&self) -> Result<()> {
        if self.is_valid() {
            Ok(())
        } else {
            let mut msgs = self.structural.clone();
            msgs.extend_from_slice(&self.integrity);
            Err(Error::Validation(msgs.join("; ")))
        }
    }

    pub fn display(&self) -> String {
        let mut lines = Vec::new();
        for m in &self.structural {
            lines.push(format!("[structural] {m}"));
        }
        for m in &self.integrity {
            lines.push(format!("[integrity]  {m}"));
        }
        for m in &self.content {
            lines.push(format!("[content]    {m}"));
        }
        if lines.is_empty() {
            "OK".to_string()
        } else {
            lines.join("\n")
        }
    }
}

/// Run the full validation suite against an open [`ClanFile`].
pub fn validate(clan: &ClanFile) -> ValidationReport {
    let mut report = ValidationReport::default();
    let m = clan.manifest();

    // --- Version check ---
    if m.clan_version > crate::CLAN_VERSION {
        report.structural.push(format!(
            "clan_version {} is newer than this SDK supports ({})",
            m.clan_version,
            crate::CLAN_VERSION
        ));
    }

    // --- Manifest structural problems ---
    report.structural.extend(m.structural_problems());

    // --- Required files ---
    const REQUIRED: &[&str] = &[
        "spec/clan.md",
        "spec/agent-guide.md",
        "shared/data.yaml",
        "agent/context.md",
        "agent/output-schema.json",
        "agent/state.yaml",
        "agent/decision-chain.yaml",
    ];
    for path in REQUIRED {
        if !clan.has_entry(path) {
            report
                .structural
                .push(format!("required entry missing: {path}"));
        }
    }

    // --- All manifest-listed files must exist ---
    for entry in &m.files {
        if !clan.has_entry(&entry.path) {
            report.structural.push(format!(
                "manifest file registry references missing entry: {}",
                entry.path
            ));
        }
    }

    // --- Integrity: SHA-256 verification ---
    for entry in &m.files {
        if let Some(expected) = &entry.sha256 {
            match clan.read_entry(&entry.path) {
                Ok(bytes) => {
                    if !hash::verify_prefixed(&bytes, expected) {
                        let actual = hash::sha256_prefixed(&bytes);
                        report.integrity.push(format!(
                            "sha256 mismatch for {}: expected {expected}, got {actual}",
                            entry.path
                        ));
                    }
                }
                Err(_) => {} // already reported above as missing
            }
        }
    }

    // --- Content checks (non-fatal for basic validity) ---
    content_checks(clan, &mut report);

    report
}

fn content_checks(clan: &ClanFile, report: &mut ValidationReport) {
    let m = clan.manifest();

    // View consistency (spec §23): a declared-present view must exist; an
    // undeclared one is only worth a note.
    if let Some(view) = &m.view {
        if view.present && !clan.has_entry("human/index.html") {
            report
                .structural
                .push("manifest.view.present is true but human/index.html is missing".into());
        }
        if !view.present && clan.has_entry("human/index.html") {
            report.content.push(
                "human/index.html exists but manifest.view.present is false — flag is out of date"
                    .into(),
            );
        }
    }

    // Forked files (spec §24.1): the namespace members must exist.
    if let Some(fork) = &m.fork {
        for member in ["data.yaml", "decisions.yaml"] {
            let path = format!("{}{member}", fork.namespace);
            if !clan.has_entry(&path) {
                report.structural.push(format!(
                    "forked file is missing its namespace member: {path}"
                ));
            }
        }
    }

    // Merge report (spec §24.4): must parse; every conflict key must exist in
    // the (post-merge) shared data; `unresolved` must match the record.
    if clan.has_entry(crate::merge::MERGE_REPORT_PATH) {
        match clan
            .read_entry(crate::merge::MERGE_REPORT_PATH)
            .map_err(|e| e.to_string())
            .and_then(|b| crate::merge::MergeReport::from_yaml(&b).map_err(|e| e.to_string()))
        {
            Ok(mr) => {
                if mr.unresolved != mr.conflicts.len() {
                    report.content.push(format!(
                        "merge-report.yaml unresolved count ({}) does not match its conflict list ({})",
                        mr.unresolved,
                        mr.conflicts.len()
                    ));
                }
                if let Ok(bytes) = clan.read_entry("shared/data.yaml") {
                    if let Ok(data) = serde_yaml::from_slice::<serde_yaml::Value>(&bytes) {
                        for conflict in &mr.conflicts {
                            if data.get(conflict.key.as_str()).is_none() {
                                report.content.push(format!(
                                    "merge-report.yaml references key {:?} that is absent from shared/data.yaml",
                                    conflict.key
                                ));
                            }
                        }
                    }
                }
            }
            Err(e) => report
                .content
                .push(format!("merge-report.yaml does not parse: {e}")),
        }
    }

    // agent/requirements.yaml (spec §22): optional, but must parse if present.
    if clan.has_entry("agent/requirements.yaml") {
        if let Ok(bytes) = clan.read_entry("agent/requirements.yaml") {
            if serde_yaml::from_slice::<serde_yaml::Value>(&bytes).is_err() {
                report
                    .content
                    .push("agent/requirements.yaml does not parse as valid YAML".into());
            }
        }
    }

    // shared/data.yaml must parse as valid YAML.
    if let Ok(bytes) = clan.read_entry("shared/data.yaml") {
        if serde_yaml::from_slice::<serde_yaml::Value>(&bytes).is_err() {
            report
                .content
                .push("shared/data.yaml does not parse as valid YAML".into());
        }
    }

    // agent/output-schema.json must parse as valid JSON.
    if let Ok(bytes) = clan.read_entry("agent/output-schema.json") {
        if serde_json::from_slice::<serde_json::Value>(&bytes).is_err() {
            report
                .content
                .push("agent/output-schema.json does not parse as valid JSON".into());
        }
    }

    // human/index.html: no content checks on HTML structure.
    // Full HTML documents, <script> tags, and on* event handlers are all permitted.
    // The iframe sandbox (allow-scripts, no allow-same-origin) isolates agent JS
    // to a null origin — it cannot reach Tauri IPC or parent app state.

    // decision-chain.yaml entries must have required fields.
    if let Ok(bytes) = clan.read_entry("agent/decision-chain.yaml") {
        if let Ok(chain) = serde_yaml::from_slice::<serde_yaml::Value>(&bytes) {
            if let Some(decisions) = chain.get("decisions").and_then(|v| v.as_sequence()) {
                for (i, entry) in decisions.iter().enumerate() {
                    for field in &["agent", "action", "rationale", "timestamp"] {
                        if entry.get(*field).is_none() {
                            report.content.push(format!(
                                "decision-chain.yaml entry {i} missing required field: {field}"
                            ));
                        }
                    }
                }
            }
        }
    }
}
