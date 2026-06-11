// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Fork/join concurrency (spec §24): namespace-per-agent branch files and a
//! deterministic per-key fold at merge time.
//!
//! The model mirrors what production frameworks ship as their only supported
//! concurrency mechanism (isolated per-branch writes + per-key reducers):
//! `fork` hands every agent the same parent and a private namespace
//! (`agents/<agent_id>/`), so conflicts are impossible by construction during
//! the parallel interval; `merge` folds the namespaces into
//! `shared/data.yaml` using manifest-declared per-key policies and records
//! every contested key in `merge-report.yaml` — conflicts are data, not
//! failures.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::container::{ClanBuilder, ClanFile};
use crate::decision::{Decision, DecisionChain};
use crate::error::{Error, Result};
use crate::manifest::{FileEntry, ForkInfo, Lineage, MergePolicies, ParentRef};

/// Archive path of the merge report member (spec §24.4).
pub const MERGE_REPORT_PATH: &str = "merge-report.yaml";

/// The machine-readable conflict record written by [`merge`] (spec §24.4).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MergeReport {
    pub generated_by: String,
    #[serde(default)]
    pub conflicts: Vec<MergeConflict>,
    /// Count of conflicts not yet adjudicated. Decremented by `patch-data`
    /// writes that settle a contested key (spec §25).
    #[serde(default)]
    pub unresolved: usize,
}

/// One contested key: two or more branches wrote different values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    pub key: String,
    pub winner: ConflictValue,
    #[serde(default)]
    pub losers: Vec<ConflictValue>,
    /// Optional hint when the conflict shape suggests a better policy — e.g.
    /// prose-valued keys every branch wrote independently are usually
    /// complementary, not contradictory, and `append` keeps them all (F6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// A value one branch wrote for a contested key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictValue {
    pub value: Value,
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
}

impl MergeReport {
    pub fn from_yaml(bytes: &[u8]) -> Result<Self> {
        Ok(serde_yaml::from_slice(bytes)?)
    }

    pub fn to_yaml(&self) -> Result<Vec<u8>> {
        Ok(serde_yaml::to_string(self)?.into_bytes())
    }
}

/// Options for [`merge`].
#[derive(Debug, Default)]
pub struct MergeOptions {
    /// Override the per-key policies declared in the first branch's manifest.
    pub policies: Option<MergePolicies>,
    /// Drop the `agents/<id>/` namespaces from the merged file instead of
    /// retaining them as provenance.
    pub prune_namespaces: bool,
    /// Human-readable delta for the lineage block.
    pub delta: Option<String>,
}

/// The result of a [`merge`].
#[derive(Debug)]
pub struct MergeOutcome {
    pub bytes: Vec<u8>,
    pub report: MergeReport,
}

/// Fork a parent into one branch file per agent (spec §24.1). Each branch
/// carries the full parent content (pass-through rule, spec §22) plus an
/// empty private namespace the named agent writes into.
pub fn fork(parent: &ClanFile, agent_ids: &[String]) -> Result<Vec<(String, Vec<u8>)>> {
    fork_with_contexts(parent, agent_ids, None)
}

/// Like [`fork`], but each branch's `agent/context.md` can be replaced with a
/// per-agent task (keyed by agent id). This stops branch agents inheriting the
/// parent's full-html/design instructions that don't apply during a parallel
/// data-only interval (F7). When no override is given for an agent, its branch
/// keeps the parent context with a short branch-mode banner prepended.
pub fn fork_with_contexts(
    parent: &ClanFile,
    agent_ids: &[String],
    contexts: Option<&std::collections::BTreeMap<String, String>>,
) -> Result<Vec<(String, Vec<u8>)>> {
    if parent.manifest().fork.is_some() {
        return Err(Error::NamespaceViolation(
            "this file is already a fork branch — forking a branch is not supported.\n\
             next: finish the branch work and `clan merge` the branches first, then fork the merged file"
                .into(),
        ));
    }
    if agent_ids.len() < 2 {
        return Err(Error::Merge(format!(
            "fork needs at least 2 agents, got {} — a single sequential agent needs no fork",
            agent_ids.len()
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for id in agent_ids {
        let trimmed = id.trim();
        if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') {
            return Err(Error::Merge(format!(
                "invalid agent id {id:?}: must be non-empty and contain no path separators"
            )));
        }
        if !seen.insert(trimmed.to_string()) {
            return Err(Error::Merge(format!("duplicate agent id {trimmed:?}")));
        }
    }

    let now = Utc::now().to_rfc3339();
    let parent_manifest = parent.manifest();
    let parent_entries = parent.read_all_entries()?;
    let parent_sha = parent.sha256();
    let parent_context = parent.read_entry_string("agent/context.md").unwrap_or_default();

    let mut branches = Vec::with_capacity(agent_ids.len());
    for agent_id in agent_ids {
        let agent_id = agent_id.trim().to_string();
        let namespace = format!("agents/{agent_id}/");

        // Per-branch task: an explicit override, else the parent context with
        // a branch-mode banner so inherited full-html/design instructions
        // don't mislead a data-only branch agent (F7).
        let base_task = contexts
            .and_then(|c| c.get(&agent_id))
            .cloned()
            .unwrap_or_else(|| parent_context.clone());
        let branch_context = format!(
            "# Branch Mode — you are agent `{agent_id}`\n\n\
             You are one of several agents working this document in parallel. Write ONLY \
             inside `{namespace}`: data via `clan patch-data <file> <json> --namespace`, \
             decisions via `clan patch-decision` (auto-routed). Writes to shared/ or human/ \
             are rejected until the branches are joined with `clan merge`. Any output-mode or \
             design instructions below apply AFTER the merge, not to your branch — contribute \
             structured data now; rendering happens later.\n\n---\n\n{base_task}"
        );

        let mut manifest = parent_manifest.clone();
        manifest.id = Uuid::new_v4().to_string();
        manifest.updated_at = now.clone();
        manifest.lineage = Some(Lineage {
            parent_id: parent_manifest.id.clone(),
            parent_uri: format!("file:///unknown/{}.clan", parent_manifest.id),
            parent_sha256: Some(parent_sha.clone()),
            delta: format!("forked branch for agent {agent_id}"),
            parents: Vec::new(),
            merge: false,
        });
        manifest.fork = Some(ForkInfo {
            agent_id: agent_id.clone(),
            namespace: namespace.clone(),
            forked_from: Some(parent_sha.clone()),
        });
        manifest.files.push(FileEntry {
            id: format!("branch-data-{agent_id}"),
            path: format!("{namespace}data.yaml"),
            role: "branch-data".into(),
            content_type: "application/yaml".into(),
            priority: None,
            sha256: None,
        });
        manifest.files.push(FileEntry {
            id: format!("branch-decisions-{agent_id}"),
            path: format!("{namespace}decisions.yaml"),
            role: "branch-decisions".into(),
            content_type: "application/yaml".into(),
            priority: None,
            sha256: None,
        });

        let mut builder = ClanBuilder::new(manifest);
        for (path, bytes) in &parent_entries {
            if path == crate::container::MANIFEST_PATH || path == "agent/context.md" {
                continue;
            }
            builder.add_entry(path.clone(), bytes.clone());
        }
        builder.add_entry("agent/context.md", branch_context.into_bytes());
        builder.add_entry(format!("{namespace}data.yaml"), b"{}\n".to_vec());
        builder.add_entry(format!("{namespace}decisions.yaml"), b"decisions: []\n".to_vec());
        branches.push((agent_id, builder.build()?));
    }
    Ok(branches)
}

/// Join fork branches into one merged file (spec §24.3): a deterministic,
/// mechanical per-key fold of every branch namespace into `shared/data.yaml`
/// — no LLM call. Branches are folded in argument order; `last-write` means
/// the latest-listed writer of a key wins.
pub fn merge(branches: &[ClanFile], opts: MergeOptions) -> Result<MergeOutcome> {
    if branches.len() < 2 {
        return Err(Error::Merge(format!(
            "merge needs at least 2 branch files, got {}",
            branches.len()
        )));
    }
    let mut forks = Vec::with_capacity(branches.len());
    for (i, branch) in branches.iter().enumerate() {
        let fork = branch.manifest().fork.as_ref().ok_or_else(|| {
            Error::Merge(format!(
                "branch {i} ({}) has no fork block — only files produced by `clan fork` can be merged",
                branch.manifest().id
            ))
        })?;
        forks.push(fork.clone());
    }
    let origin = &forks[0].forked_from;
    for (i, fork) in forks.iter().enumerate().skip(1) {
        if &fork.forked_from != origin {
            return Err(Error::Merge(format!(
                "branch {i} (agent '{}') was forked from a different parent — all branches must share one fork point",
                fork.agent_id
            )));
        }
        if forks[..i].iter().any(|f| f.agent_id == fork.agent_id) {
            return Err(Error::Merge(format!(
                "two branches carry the same agent id '{}' — every branch must come from one fork call",
                fork.agent_id
            )));
        }
    }

    let now = Utc::now().to_rfc3339();
    let base = &branches[0];
    let base_manifest = base.manifest();

    // Per-key policy table: explicit override > base manifest > defaults.
    let policies = opts
        .policies
        .or_else(|| base_manifest.merge_policies.clone())
        .unwrap_or_default();
    let default_policy = policies.default.clone().unwrap_or_else(|| "last-write".into());
    let policy_for = |key: &str| -> String {
        policies.keys.get(key).cloned().unwrap_or_else(|| default_policy.clone())
    };

    // Collect every branch's namespace writes: key -> [(agent, value)] in
    // branch (argument) order.
    let mut writes: std::collections::BTreeMap<String, Vec<(String, Value)>> = Default::default();
    for (branch, fork) in branches.iter().zip(&forks) {
        let data_path = format!("{}data.yaml", fork.namespace);
        if !branch.has_entry(&data_path) {
            continue;
        }
        let text = branch.read_entry_string(&data_path)?;
        if text.trim().is_empty() {
            continue;
        }
        let value: Value = serde_yaml::from_str::<serde_yaml::Value>(&text)
            .ok()
            .and_then(|y| serde_json::to_value(y).ok())
            .unwrap_or(Value::Null);
        if let Value::Object(map) = value {
            for (key, val) in map {
                writes.entry(key).or_default().push((fork.agent_id.clone(), val));
            }
        }
    }

    // Fold into the shared data with per-key policies.
    let base_data_text = base.read_entry_string("shared/data.yaml").unwrap_or_default();
    let mut data: Value = serde_yaml::from_str::<serde_yaml::Value>(&base_data_text)
        .ok()
        .and_then(|y| serde_json::to_value(y).ok())
        .unwrap_or_else(|| Value::Object(Default::default()));
    if !data.is_object() {
        data = Value::Object(Default::default());
    }

    let mut conflicts = Vec::new();
    for (key, writers) in &writes {
        let policy = policy_for(key);
        let distinct: Vec<&(String, Value)> = {
            let mut seen: Vec<&(String, Value)> = Vec::new();
            for w in writers {
                if !seen.iter().any(|s| s.1 == w.1) {
                    seen.push(w);
                }
            }
            seen
        };
        let (merged, winner_idx): (Value, Option<usize>) = match policy.as_str() {
            "append" => {
                let mut acc = Vec::new();
                for (_, v) in writers {
                    match v {
                        Value::Array(items) => acc.extend(items.clone()),
                        other => acc.push(other.clone()),
                    }
                }
                (Value::Array(acc), None)
            }
            "agent-priority" => (writers[0].1.clone(), Some(0)),
            "max" | "min" => {
                let numeric: Vec<(usize, f64)> = writers
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (_, v))| v.as_f64().map(|n| (i, n)))
                    .collect();
                match numeric.iter().copied().reduce(|a, b| {
                    let pick_b = if policy == "max" { b.1 >= a.1 } else { b.1 <= a.1 };
                    if pick_b { b } else { a }
                }) {
                    Some((idx, _)) => (writers[idx].1.clone(), Some(idx)),
                    // No numeric writer — fall back to last-write.
                    None => (writers[writers.len() - 1].1.clone(), Some(writers.len() - 1)),
                }
            }
            // "last-write" and anything unknown (already flagged by
            // validation) fold deterministically as last-write.
            _ => (writers[writers.len() - 1].1.clone(), Some(writers.len() - 1)),
        };

        // A conflict is a discarded value: >1 distinct value under a policy
        // that picks one winner. `append` keeps everything — not a conflict.
        if let Some(widx) = winner_idx {
            if distinct.len() > 1 {
                let winner = &writers[widx];
                // Prose-valued keys (every writer wrote a non-empty string)
                // are usually complementary notes, not contradictions — a
                // winner-takes-all policy silently drops the others, so
                // suggest `append` (F6).
                let all_prose = writers
                    .iter()
                    .all(|(_, v)| v.as_str().map_or(false, |s| !s.trim().is_empty()));
                let suggestion = (all_prose && policy != "append").then(|| {
                    format!(
                        "all {} branches wrote prose for '{key}'; they may be complementary — re-run with `--policy {key}=append` to keep every branch's text",
                        writers.len()
                    )
                });
                conflicts.push(MergeConflict {
                    key: key.clone(),
                    winner: ConflictValue {
                        value: winner.1.clone(),
                        agent: winner.0.clone(),
                        policy: Some(policy.clone()),
                    },
                    losers: writers
                        .iter()
                        .enumerate()
                        .filter(|(i, (_, v))| *i != widx && *v != winner.1)
                        .map(|(_, (agent, value))| ConflictValue {
                            value: value.clone(),
                            agent: agent.clone(),
                            policy: None,
                        })
                        .collect(),
                    suggestion,
                });
            }
        }

        data.as_object_mut()
            .expect("data forced to object above")
            .insert(key.clone(), merged);
    }

    let report = MergeReport {
        generated_by: format!("clan merge v{}", env!("CARGO_PKG_VERSION")),
        unresolved: conflicts.len(),
        conflicts,
    };

    // Fold branch decision logs into the shared chain: branch entries land
    // newest-first above the (identical) base chain, ordered by timestamp.
    let mut branch_decisions: Vec<Decision> = Vec::new();
    for (branch, fork) in branches.iter().zip(&forks) {
        let path = format!("{}decisions.yaml", fork.namespace);
        if branch.has_entry(&path) {
            if let Ok(chain) = DecisionChain::from_yaml(&branch.read_entry(&path)?) {
                branch_decisions.extend(chain.decisions);
            }
        }
    }
    branch_decisions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let mut chain = DecisionChain::from_yaml(&base.read_entry("agent/decision-chain.yaml")?)?;
    for decision in branch_decisions.into_iter().rev() {
        chain.prepend(decision);
    }
    let agents: Vec<String> = forks.iter().map(|f| f.agent_id.clone()).collect();
    chain.prepend(Decision {
        agent: "clan-merge".into(),
        version: None,
        action: format!("merged {} branches: {}", branches.len(), agents.join(", ")),
        rationale: format!(
            "deterministic per-key fold; {} contested key(s) recorded in {MERGE_REPORT_PATH}",
            report.unresolved
        ),
        timestamp: now.clone(),
        fields_changed: writes.keys().cloned().collect(),
        pinned: false,
        trace_ref: None,
    });

    // --- Merged manifest ---
    let mut manifest = base_manifest.clone();
    manifest.id = Uuid::new_v4().to_string();
    manifest.updated_at = now.clone();
    manifest.fork = None;
    manifest.lineage = Some(Lineage {
        parent_id: base_manifest.id.clone(),
        parent_uri: format!("file:///unknown/{}.clan", base_manifest.id),
        parent_sha256: Some(base.sha256()),
        delta: opts
            .delta
            .unwrap_or_else(|| format!("merged branches: {}", agents.join(", "))),
        parents: branches
            .iter()
            .zip(&forks)
            .map(|(b, f)| ParentRef {
                id: b.manifest().id.clone(),
                sha256: Some(b.sha256()),
                delta: None,
                agent_id: Some(f.agent_id.clone()),
            })
            .collect(),
        merge: true,
    });
    // The carried-over view (if any) predates the fold.
    if let Some(view) = &mut manifest.view {
        if view.present {
            view.stale = true;
        }
    }

    if opts.prune_namespaces {
        manifest.files.retain(|f| !f.path.starts_with("agents/"));
    } else {
        // Register every branch's namespace members (base's are already there).
        for (branch, _) in branches.iter().zip(&forks).skip(1) {
            for entry in &branch.manifest().files {
                if entry.path.starts_with("agents/")
                    && !manifest.files.iter().any(|f| f.path == entry.path)
                {
                    manifest.files.push(entry.clone());
                }
            }
        }
    }
    if !manifest.files.iter().any(|f| f.path == MERGE_REPORT_PATH) {
        manifest.files.push(FileEntry {
            id: "merge-report".into(),
            path: MERGE_REPORT_PATH.into(),
            role: "merge-report".into(),
            content_type: "application/yaml".into(),
            priority: None,
            sha256: None,
        });
    }

    // --- Assemble ---
    let mut builder = ClanBuilder::new(manifest);
    for (path, bytes) in base.read_all_entries()? {
        if path == crate::container::MANIFEST_PATH
            || path == "shared/data.yaml"
            || path == "agent/decision-chain.yaml"
            || path == MERGE_REPORT_PATH
        {
            continue;
        }
        if opts.prune_namespaces && path.starts_with("agents/") {
            continue;
        }
        builder.add_entry(path, bytes);
    }
    if !opts.prune_namespaces {
        for (branch, fork) in branches.iter().zip(&forks).skip(1) {
            for path in [
                format!("{}data.yaml", fork.namespace),
                format!("{}decisions.yaml", fork.namespace),
            ] {
                if branch.has_entry(&path) {
                    builder.add_entry(path.clone(), branch.read_entry(&path)?);
                }
            }
        }
    }

    let data_yaml: serde_yaml::Value = serde_json::from_str(&serde_json::to_string(&data)?)?;
    builder.add_entry("shared/data.yaml", serde_yaml::to_string(&data_yaml)?.into_bytes());
    builder.add_entry("agent/decision-chain.yaml", chain.to_yaml()?);
    builder.add_entry(MERGE_REPORT_PATH, report.to_yaml()?);

    Ok(MergeOutcome {
        bytes: builder.build()?,
        report,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create::{create, CreateOptions};

    fn root() -> ClanFile {
        let bytes = create(CreateOptions {
            title: "Fork Test".into(),
            brief: "test brief".into(),
            document_type: None,
            no_render: false,
            schema: None,
        })
        .unwrap();
        ClanFile::from_bytes(bytes).unwrap()
    }

    fn forked_pair() -> (ClanFile, ClanFile) {
        let parent = root();
        let branches = fork(&parent, &["alpha".into(), "beta".into()]).unwrap();
        let mut iter = branches.into_iter();
        let a = ClanFile::from_bytes(iter.next().unwrap().1).unwrap();
        let b = ClanFile::from_bytes(iter.next().unwrap().1).unwrap();
        (a, b)
    }

    fn with_namespace_data(branch: &ClanFile, patch: serde_json::Value) -> ClanFile {
        let bytes = crate::pack::patch_data_namespaced(branch, &patch).unwrap();
        ClanFile::from_bytes(bytes).unwrap()
    }

    #[test]
    fn fork_stamps_namespace_and_preserves_parent_content() {
        let parent = root();
        let branches = fork(&parent, &["alpha".into(), "beta".into()]).unwrap();
        assert_eq!(branches.len(), 2);
        for (agent, bytes) in branches {
            let branch = ClanFile::from_bytes(bytes).unwrap();
            let fork_info = branch.manifest().fork.as_ref().unwrap();
            assert_eq!(fork_info.agent_id, agent);
            assert_eq!(fork_info.namespace, format!("agents/{agent}/"));
            assert_eq!(fork_info.forked_from.as_deref(), Some(parent.sha256().as_str()));
            // Pass-through: parent members intact, namespace created empty.
            assert!(branch.has_entry("shared/data.yaml"));
            assert!(branch.has_entry("agent/decision-chain.yaml"));
            assert!(branch.has_entry(&format!("agents/{agent}/data.yaml")));
            assert!(branch.has_entry(&format!("agents/{agent}/decisions.yaml")));
            assert_eq!(
                branch.manifest().lineage.as_ref().unwrap().parent_id,
                parent.manifest().id
            );
        }
    }

    #[test]
    fn fork_rejects_single_agent_and_bad_ids() {
        let parent = root();
        assert!(fork(&parent, &["solo".into()]).is_err());
        assert!(fork(&parent, &["a/b".into(), "c".into()]).is_err());
        assert!(fork(&parent, &["dup".into(), "dup".into()]).is_err());
    }

    #[test]
    fn fork_of_fork_is_a_teaching_error() {
        let (a, _) = forked_pair();
        let err = fork(&a, &["x".into(), "y".into()]).unwrap_err();
        assert!(err.to_string().contains("clan merge"), "{err}");
    }

    #[test]
    fn branch_agents_cannot_write_shared_members() {
        let (a, _) = forked_pair();
        let patch = serde_json::json!({"status": "done"});
        let err = crate::pack::patch_data(&a, &patch, None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("alpha"), "{msg}");
        assert!(msg.contains("--namespace"), "teaching error must name the fix: {msg}");
        assert!(crate::pack::patch_context(&a, "x", false).is_err());
        assert!(crate::pack::patch_schema(&a, "{}", None).is_err());
        assert!(crate::pack::patch_asset(&a, "x.svg", vec![1]).is_err());
    }

    #[test]
    fn namespaced_writes_land_in_the_namespace_only() {
        let (a, _) = forked_pair();
        let next = with_namespace_data(&a, serde_json::json!({"findings": ["f1"]}));
        let ns_data = next.read_entry_string("agents/alpha/data.yaml").unwrap();
        assert!(ns_data.contains("f1"));
        // Shared data untouched.
        assert_eq!(
            next.read_entry_string("shared/data.yaml").unwrap(),
            a.read_entry_string("shared/data.yaml").unwrap()
        );
    }

    #[test]
    fn branch_decisions_auto_route_to_namespace() {
        let (a, _) = forked_pair();
        let bytes = crate::pack::patch_decision(
            &a,
            crate::pack::DecisionEntry {
                agent_name: "alpha".into(),
                action: "researched".into(),
                rationale: "found things".into(),
                pinned: false,
            },
            None,
        )
        .unwrap();
        let next = ClanFile::from_bytes(bytes).unwrap();
        let ns_chain =
            DecisionChain::from_yaml(&next.read_entry("agents/alpha/decisions.yaml").unwrap())
                .unwrap();
        assert_eq!(ns_chain.decisions.len(), 1);
        // The shared chain is untouched.
        assert_eq!(
            next.read_entry_string("agent/decision-chain.yaml").unwrap(),
            a.read_entry_string("agent/decision-chain.yaml").unwrap()
        );
    }

    #[test]
    fn merge_folds_disjoint_keys_without_conflicts() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"alpha_finding": "A"}));
        let b = with_namespace_data(&b, serde_json::json!({"beta_finding": "B"}));

        let outcome = merge(&[a, b], MergeOptions::default()).unwrap();
        assert_eq!(outcome.report.unresolved, 0);
        assert!(outcome.report.conflicts.is_empty());

        let merged = ClanFile::from_bytes(outcome.bytes).unwrap();
        let data = merged.read_entry_string("shared/data.yaml").unwrap();
        assert!(data.contains("alpha_finding"), "{data}");
        assert!(data.contains("beta_finding"), "{data}");
        let lineage = merged.manifest().lineage.as_ref().unwrap();
        assert!(lineage.merge);
        assert_eq!(lineage.parents.len(), 2);
        assert!(merged.manifest().fork.is_none());
        assert!(merged.has_entry(MERGE_REPORT_PATH));
    }

    #[test]
    fn merge_last_write_records_conflict_with_provenance() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"status": "approved"}));
        let b = with_namespace_data(&b, serde_json::json!({"status": "needs-review"}));

        let outcome = merge(&[a, b], MergeOptions::default()).unwrap();
        assert_eq!(outcome.report.unresolved, 1);
        let conflict = &outcome.report.conflicts[0];
        assert_eq!(conflict.key, "status");
        // last-write: the latest-listed branch (beta) wins.
        assert_eq!(conflict.winner.agent, "beta");
        assert_eq!(conflict.winner.value, serde_json::json!("needs-review"));
        assert_eq!(conflict.winner.policy.as_deref(), Some("last-write"));
        assert_eq!(conflict.losers.len(), 1);
        assert_eq!(conflict.losers[0].agent, "alpha");

        let merged = ClanFile::from_bytes(outcome.bytes).unwrap();
        let data = merged.read_entry_string("shared/data.yaml").unwrap();
        assert!(data.contains("needs-review"), "{data}");
    }

    #[test]
    fn merge_append_policy_keeps_everything_and_is_not_a_conflict() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"findings": ["f1", "f2"]}));
        let b = with_namespace_data(&b, serde_json::json!({"findings": ["f3"]}));

        let opts = MergeOptions {
            policies: Some(MergePolicies {
                default: None,
                keys: [("findings".to_string(), "append".to_string())].into(),
            }),
            ..Default::default()
        };
        let outcome = merge(&[a, b], opts).unwrap();
        assert_eq!(outcome.report.unresolved, 0);

        let merged = ClanFile::from_bytes(outcome.bytes).unwrap();
        let data: serde_yaml::Value =
            serde_yaml::from_str(&merged.read_entry_string("shared/data.yaml").unwrap()).unwrap();
        let findings = data.get("findings").unwrap().as_sequence().unwrap();
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn merge_max_policy_picks_numeric_winner() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"risk_score": 7}));
        let b = with_namespace_data(&b, serde_json::json!({"risk_score": 4}));

        let opts = MergeOptions {
            policies: Some(MergePolicies {
                default: None,
                keys: [("risk_score".to_string(), "max".to_string())].into(),
            }),
            ..Default::default()
        };
        let outcome = merge(&[a, b], opts).unwrap();
        assert_eq!(outcome.report.conflicts[0].winner.value, serde_json::json!(7));
        assert_eq!(outcome.report.conflicts[0].winner.agent, "alpha");
    }

    #[test]
    fn merge_folds_branch_decisions_into_shared_chain() {
        let (a, b) = forked_pair();
        let entry = |agent: &str, action: &str| crate::pack::DecisionEntry {
            agent_name: agent.into(),
            action: action.into(),
            rationale: "r".into(),
            pinned: false,
        };
        let a = ClanFile::from_bytes(
            crate::pack::patch_decision(&a, entry("alpha", "researched"), None).unwrap(),
        )
        .unwrap();
        let b = ClanFile::from_bytes(
            crate::pack::patch_decision(&b, entry("beta", "analysed"), None).unwrap(),
        )
        .unwrap();

        let outcome = merge(&[a, b], MergeOptions::default()).unwrap();
        let merged = ClanFile::from_bytes(outcome.bytes).unwrap();
        let chain =
            DecisionChain::from_yaml(&merged.read_entry("agent/decision-chain.yaml").unwrap())
                .unwrap();
        // Merge entry on top, both branch decisions folded in.
        assert_eq!(chain.decisions[0].agent, "clan-merge");
        let agents: Vec<&str> = chain.decisions.iter().map(|d| d.agent.as_str()).collect();
        assert!(agents.contains(&"alpha"), "{agents:?}");
        assert!(agents.contains(&"beta"), "{agents:?}");
    }

    #[test]
    fn merge_rejects_mismatched_fork_points_and_unforked_files() {
        let (a, _) = forked_pair();
        let other_parent = root();
        let other =
            ClanFile::from_bytes(fork(&other_parent, &["beta".into(), "gamma".into()]).unwrap()[0].1.clone())
                .unwrap();
        assert!(merge(&[a, other], MergeOptions::default()).is_err());

        let (a, _) = forked_pair();
        let unforked = root();
        assert!(merge(&[a, unforked], MergeOptions::default()).is_err());
    }

    #[test]
    fn merged_file_is_writable_again() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"x": 1}));
        let outcome = merge(&[a, b], MergeOptions::default()).unwrap();
        let merged = ClanFile::from_bytes(outcome.bytes).unwrap();
        // The parallel interval is over: direct shared writes work again.
        let next = crate::pack::patch_data(&merged, &serde_json::json!({"y": 2}), None).unwrap();
        assert!(ClanFile::from_bytes(next).is_ok());
    }

    #[test]
    fn prune_namespaces_drops_branch_members() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"x": 1}));
        let opts = MergeOptions { prune_namespaces: true, ..Default::default() };
        let outcome = merge(&[a, b], opts).unwrap();
        let merged = ClanFile::from_bytes(outcome.bytes).unwrap();
        assert!(!merged.has_entry("agents/alpha/data.yaml"));
        assert!(!merged.has_entry("agents/beta/data.yaml"));
        assert!(!merged.manifest().files.iter().any(|f| f.path.starts_with("agents/")));
    }

    // F7: branches get a branch-mode banner, and --context-dir overrides the task.
    #[test]
    fn fork_branches_carry_branch_banner_and_context_override() {
        let parent = root();
        let mut ctx = std::collections::BTreeMap::new();
        ctx.insert("alpha".to_string(), "ALPHA SPECIFIC TASK: do the alpha thing.".to_string());
        let branches = fork_with_contexts(&parent, &["alpha".into(), "beta".into()], Some(&ctx)).unwrap();
        for (agent, bytes) in branches {
            let b = ClanFile::from_bytes(bytes).unwrap();
            let context = b.read_entry_string("agent/context.md").unwrap();
            assert!(context.contains("# Branch Mode"), "every branch gets the banner: {agent}");
            assert!(context.contains(&format!("agent `{agent}`")));
            if agent == "alpha" {
                assert!(context.contains("ALPHA SPECIFIC TASK"), "override applied for alpha");
            } else {
                // beta had no override → keeps the parent task below the banner.
                assert!(context.contains("test brief"), "beta keeps parent task: {context}");
            }
        }
    }

    // F6: a prose key all branches wrote gets an append suggestion.
    #[test]
    fn prose_conflict_suggests_append() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"assumptions": "alpha's view of the world"}));
        let b = with_namespace_data(&b, serde_json::json!({"assumptions": "beta's different framing"}));
        let outcome = merge(&[a, b], MergeOptions::default()).unwrap();
        let conflict = outcome.report.conflicts.iter().find(|c| c.key == "assumptions").unwrap();
        let suggestion = conflict.suggestion.as_ref().expect("prose conflict must carry a suggestion (F6)");
        assert!(suggestion.contains("--policy assumptions=append"), "{suggestion}");
    }

    #[test]
    fn append_policy_conflict_has_no_suggestion() {
        let (a, b) = forked_pair();
        let a = with_namespace_data(&a, serde_json::json!({"notes": "x"}));
        let b = with_namespace_data(&b, serde_json::json!({"notes": "y"}));
        let opts = MergeOptions {
            policies: Some(MergePolicies { default: None, keys: [("notes".to_string(), "append".to_string())].into() }),
            ..Default::default()
        };
        let outcome = merge(&[a, b], opts).unwrap();
        // append keeps both → not a conflict at all.
        assert!(outcome.report.conflicts.iter().all(|c| c.key != "notes"));
    }
}
