// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Error types for the CLAN SDK.

use thiserror::Error;

/// Result alias used throughout the SDK.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("entry not found in container: {0}")]
    EntryNotFound(String),

    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("schema error: {0}")]
    Schema(String),

    #[error("agent output rejected: {0}")]
    OutputRejected(String),

    #[error("unsupported output mode: {0}")]
    UnsupportedMode(String),

    #[error("forked-file namespace violation: {0}")]
    NamespaceViolation(String),

    #[error("merge error: {0}")]
    Merge(String),
}
