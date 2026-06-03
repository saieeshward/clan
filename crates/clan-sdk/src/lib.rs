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
pub mod pack;
pub mod patch;
pub mod toon;
pub mod validate;

pub use compress::{compress_chain, nlp_compress, CompressionConfig, Compressor};
pub use container::{ClanBuilder, ClanFile, MANIFEST_PATH};
pub use create::{create, export_static, CreateOptions};
pub use decision::{Decision, DecisionChain, TraceRef};
pub use error::{Error, Result};
pub use inject::{assemble, AgentContext, InjectOptions};
pub use manifest::{
    ExternalRef, FileEntry, Lineage, Manifest, CLAN_VERSION, CLAN_VERSION_MINOR,
};
pub use pack::{pack, pack_html, patch_data, patch_decision, patch_state, patch_context, patch_asset, AgentOutput, DecisionEntry, HumanPayload, PackOptions};
pub use patch::{apply_patch_and_repack, Patch, Patches};
pub use toon::{to_toon, yaml_to_toon};
pub use validate::{validate, ValidationReport};
