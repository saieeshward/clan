// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `agent/decision-chain.yaml` model (spec §7).

use serde::{Deserialize, Serialize};

/// The ordered decision log. Newest entries first.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionChain {
    #[serde(default)]
    pub decisions: Vec<Decision>,
}

/// A single agent decision entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub action: String,
    pub rationale: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields_changed: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
    #[serde(rename = "trace-ref", default, skip_serializing_if = "Option::is_none")]
    pub trace_ref: Option<TraceRef>,
}

/// Reference to full context held in an external store (spec §13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRef {
    pub store: String,
    pub entry: String,
    #[serde(rename = "content-hash")]
    pub content_hash: String,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl DecisionChain {
    pub fn from_yaml(bytes: &[u8]) -> crate::Result<Self> {
        Ok(serde_yaml::from_slice(bytes)?)
    }

    pub fn to_yaml(&self) -> crate::Result<Vec<u8>> {
        Ok(serde_yaml::to_string(self)?.into_bytes())
    }

    /// Prepend a new decision (newest-first ordering).
    pub fn prepend(&mut self, decision: Decision) {
        self.decisions.insert(0, decision);
    }
}

impl Decision {
    /// A minimal decision with required fields populated.
    pub fn new(
        agent: impl Into<String>,
        action: impl Into<String>,
        rationale: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> Self {
        Self {
            agent: agent.into(),
            version: None,
            action: action.into(),
            rationale: rationale.into(),
            timestamp: timestamp.into(),
            fields_changed: Vec::new(),
            pinned: false,
            trace_ref: None,
        }
    }
}
