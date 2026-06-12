// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # CLAN SDK
//!
//! Reference implementation of **CLAN — Context and Live Agent Notation**.
//! A `.clan` file is a ZIP container that is simultaneously machine-readable
//! for AI agents and human-renderable, carrying its own specification and a
//! verifiable lineage chain.
//!
//! See `CLAN-SPEC.md` for the full format specification.

pub mod compress;
pub mod container;
pub mod create;
pub mod decision;
pub mod error;
pub mod hash;
pub mod inject;
pub mod manifest;
pub mod merge;
pub mod pack;
pub mod patch;
pub mod render;
pub mod toon;
pub mod validate;

pub use compress::{compress_chain, nlp_compress, CompressionConfig, Compressor};
pub use container::{ClanBuilder, ClanFile, MANIFEST_PATH};
pub use create::{create, export_static, CreateOptions};
pub use decision::{Decision, DecisionChain, TraceRef};
pub use error::{Error, Result};
pub use inject::{assemble, AgentContext, InjectOptions};
pub use manifest::{
    ExternalRef, FileEntry, ForkInfo, Lineage, Manifest, MergePolicies, ParentRef, ViewState,
    CLAN_VERSION, CLAN_VERSION_MINOR,
};
pub use merge::{
    fork, fork_with_contexts, merge, ConflictValue, MergeConflict, MergeOptions, MergeOutcome,
    MergeReport, MERGE_REPORT_PATH,
};
pub use pack::{
    pack, pack_html, pack_html_targeted, patch_asset, patch_context, patch_data,
    patch_data_namespaced, patch_decision, patch_requirements, patch_state, AgentOutput,
    DecisionEntry, HumanPayload, PackOptions, PatchTargeting,
};
pub use patch::{apply_patch_and_repack, Patch, Patches};
pub use render::render;
pub use toon::{to_toon, yaml_to_toon};
pub use validate::{validate, ValidationReport};
