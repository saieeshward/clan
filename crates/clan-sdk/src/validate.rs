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

    // human/index.html (if present): no <script>, no <html>/<head>/<body> tags.
    if let Ok(html) = clan.read_entry_string("human/index.html") {
        let lower = html.to_lowercase();
        // Use precise patterns to avoid false positives (e.g. <header matching <head).
        for (label, pattern) in &[
            ("<script", "<script"),
            ("<html", "<html"),
            ("<head>", "<head>"),    // exact close, not <header>
            ("<head ", "<head "),    // <head with attribute
            ("<body", "<body"),
        ] {
            if lower.contains(*pattern) {
                report.content.push(format!(
                    "human/index.html contains forbidden tag: {label}"
                ));
            }
        }
    }

    // decision-chain.yaml entries must have required fields.
    if let Ok(bytes) = clan.read_entry("agent/decision-chain.yaml") {
        if let Ok(chain) =
            serde_yaml::from_slice::<serde_yaml::Value>(&bytes)
        {
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
