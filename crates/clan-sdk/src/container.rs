// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ZIP container layer (spec §3). A `.clan` file is a ZIP (DEFLATE) archive
//! with `manifest.yaml` stored as the first entry.
//!
//! [`ClanFile`] is the lazy reader: it keeps the raw archive in memory and
//! decompresses individual entries on demand, honouring the lazy-loading
//! contract (spec §14) — callers only pay for the entries they request.
//!
//! [`ClanBuilder`] is the writer: it collects entries, computes their
//! SHA-256 digests into the manifest, and serialises a new archive.

use std::io::{Cursor, Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::{Error, Result};
use crate::hash;
use crate::manifest::Manifest;

/// The well-known path of the manifest within every archive.
pub const MANIFEST_PATH: &str = "manifest.yaml";

/// Lazy reader over an in-memory `.clan` archive.
pub struct ClanFile {
    raw: Vec<u8>,
    manifest: Manifest,
}

impl ClanFile {
    /// Open a `.clan` file from disk.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let raw = std::fs::read(path)?;
        Self::from_bytes(raw)
    }

    /// Open a `.clan` archive from raw bytes already in memory.
    pub fn from_bytes(raw: Vec<u8>) -> Result<Self> {
        // The manifest is the only guaranteed upfront read (spec §14).
        let manifest_bytes = read_named(&raw, MANIFEST_PATH)?;
        let manifest = Manifest::from_yaml(&manifest_bytes)?;
        Ok(Self { raw, manifest })
    }

    /// The parsed manifest.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Read and decompress a single entry by archive path. Lazy: only the
    /// requested entry is decompressed.
    pub fn read_entry(&self, path: &str) -> Result<Vec<u8>> {
        read_named(&self.raw, path)
    }

    /// Read a single entry as a UTF-8 string.
    pub fn read_entry_string(&self, path: &str) -> Result<String> {
        let bytes = self.read_entry(path)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Read an entry identified by its manifest `id`.
    pub fn read_by_id(&self, id: &str) -> Result<Vec<u8>> {
        let entry = self
            .manifest
            .file_by_id(id)
            .ok_or_else(|| Error::EntryNotFound(format!("id={id}")))?;
        self.read_entry(&entry.path)
    }

    /// True if the archive contains an entry at `path`. Consults the ZIP
    /// central directory only — the entry is never decompressed.
    pub fn has_entry(&self, path: &str) -> bool {
        let cursor = Cursor::new(&self.raw);
        match ZipArchive::new(cursor) {
            Ok(archive) => archive.index_for_name(path).is_some(),
            Err(_) => false,
        }
    }

    /// The raw archive bytes (used to compute this file's own digest for the
    /// next generation's `parent_sha256`).
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// `sha256:<hex>` digest of the whole archive.
    pub fn sha256(&self) -> String {
        hash::sha256_prefixed(&self.raw)
    }

    /// Read and decompress every entry in a single archive pass
    /// (central-directory order). Use this instead of looping over
    /// [`ClanFile::entry_paths`] + [`ClanFile::read_entry`], which would
    /// instantiate one `ZipArchive` per entry.
    pub fn read_all_entries(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let cursor = Cursor::new(&self.raw);
        let mut archive = ZipArchive::new(cursor)?;
        let mut entries = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)?;
            entries.push((name, buf));
        }
        Ok(entries)
    }

    /// List all entry paths in the archive (central-directory order).
    pub fn entry_paths(&self) -> Result<Vec<String>> {
        let cursor = Cursor::new(&self.raw);
        let mut archive = ZipArchive::new(cursor)?;
        let mut names = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            names.push(file.name().to_string());
        }
        Ok(names)
    }
}

/// Read a single named entry from raw ZIP bytes.
fn read_named(raw: &[u8], path: &str) -> Result<Vec<u8>> {
    let cursor = Cursor::new(raw);
    let mut archive = ZipArchive::new(cursor)?;
    let mut file = archive
        .by_name(path)
        .map_err(|_| Error::EntryNotFound(path.to_string()))?;
    let mut buf = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Writer for a new `.clan` archive.
pub struct ClanBuilder {
    manifest: Manifest,
    entries: Vec<(String, Vec<u8>)>,
}

impl ClanBuilder {
    /// Start a builder around a manifest. The manifest's `files[]` registry is
    /// the source of truth for what must be written; entries are added with
    /// [`ClanBuilder::add_entry`].
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            entries: Vec::new(),
        }
    }

    /// Add (or replace) an entry by archive path.
    pub fn add_entry(&mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        let path = path.into();
        let bytes = bytes.into();
        if let Some(existing) = self.entries.iter_mut().find(|(p, _)| *p == path) {
            existing.1 = bytes;
        } else {
            self.entries.push((path, bytes));
        }
    }

    /// Mutable access to the manifest (e.g. to set timestamps before build).
    pub fn manifest_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
    }

    /// Finalise: recompute per-entry SHA-256 digests into the manifest,
    /// serialise the manifest, and write the ZIP with `manifest.yaml` first.
    pub fn build(mut self) -> Result<Vec<u8>> {
        // Compute sha256 for every registered file entry whose bytes we hold.
        for entry in &mut self.manifest.files {
            if let Some((_, bytes)) = self.entries.iter().find(|(p, _)| *p == entry.path) {
                entry.sha256 = Some(hash::sha256_prefixed(bytes));
            }
        }

        let manifest_bytes = self.manifest.to_yaml()?;

        let mut out = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut out);
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

            // manifest.yaml MUST be the first entry (spec §3).
            zip.start_file(MANIFEST_PATH, options)?;
            zip.write_all(&manifest_bytes)?;

            for (path, bytes) in &self.entries {
                if path == MANIFEST_PATH {
                    continue; // never write the manifest twice
                }
                zip.start_file(path.as_str(), options)?;
                zip.write_all(bytes)?;
            }
            zip.finish()?;
        }
        Ok(out.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{FileEntry, Manifest};

    fn sample_manifest() -> Manifest {
        Manifest {
            clan_version: 1,
            clan_version_minor: 0,
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            title: "Test".into(),
            created_at: "2026-05-31T10:00:00Z".into(),
            updated_at: "2026-05-31T10:00:00Z".into(),
            document_type: None,
            lineage: None,
            external: vec![],
            files: vec![FileEntry {
                id: "canonical-data".into(),
                path: "shared/data.yaml".into(),
                role: "canonical-data".into(),
                content_type: "application/yaml".into(),
                priority: None,
                sha256: None,
            }],
        }
    }

    #[test]
    fn build_and_read_roundtrip() {
        let mut builder = ClanBuilder::new(sample_manifest());
        builder.add_entry("shared/data.yaml", b"vendor: Acme\n".to_vec());
        let bytes = builder.build().unwrap();

        let clan = ClanFile::from_bytes(bytes).unwrap();
        assert_eq!(clan.manifest().title, "Test");
        let data = clan.read_entry_string("shared/data.yaml").unwrap();
        assert_eq!(data, "vendor: Acme\n");

        // sha256 was populated on build.
        let entry = clan.manifest().file_by_id("canonical-data").unwrap();
        assert!(entry.sha256.as_deref().unwrap().starts_with("sha256:"));

        // manifest is the first central-directory entry.
        let paths = clan.entry_paths().unwrap();
        assert_eq!(paths[0], MANIFEST_PATH);
    }

    #[test]
    fn missing_entry_errors() {
        let builder = ClanBuilder::new(sample_manifest());
        let bytes = builder.build().unwrap();
        let clan = ClanFile::from_bytes(bytes).unwrap();
        assert!(clan.read_entry("nope.txt").is_err());
        assert!(!clan.has_entry("nope.txt"));
    }

    // Regression for #11: has_entry must answer from the central directory
    // without decompressing the entry. We corrupt the entry's data bytes so
    // any read fails its CRC check — has_entry must still return true.
    #[test]
    fn has_entry_does_not_decompress() {
        let manifest_yaml = sample_manifest().to_yaml().unwrap();
        let payload = b"CORRUPTME-PAYLOAD-CORRUPTME";

        let mut out = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut out);
            let deflated =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            let stored =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            zip.start_file(MANIFEST_PATH, deflated).unwrap();
            zip.write_all(&manifest_yaml).unwrap();
            // Stored, so the payload appears verbatim in the archive bytes.
            zip.start_file("shared/data.yaml", stored).unwrap();
            zip.write_all(payload).unwrap();
            zip.finish().unwrap();
        }
        let mut raw = out.into_inner();

        let offset = raw
            .windows(payload.len())
            .position(|w| w == payload)
            .expect("stored payload not found in archive");
        raw[offset] ^= 0xFF; // corrupt the entry data → CRC mismatch on read

        let clan = ClanFile::from_bytes(raw).unwrap();
        assert!(
            clan.has_entry("shared/data.yaml"),
            "has_entry must not depend on decompressing the entry"
        );
        assert!(
            clan.read_entry("shared/data.yaml").is_err(),
            "corrupted entry should fail an actual read (test setup sanity check)"
        );
    }

    // Regression for #12: read_all_entries must return the same set of
    // entries as the per-entry read path, in central-directory order.
    #[test]
    fn read_all_entries_matches_per_entry_reads() {
        let mut manifest = sample_manifest();
        manifest.files.push(FileEntry {
            id: "context".into(),
            path: "agent/context.md".into(),
            role: "agent-context".into(),
            content_type: "text/markdown".into(),
            priority: None,
            sha256: None,
        });
        let mut builder = ClanBuilder::new(manifest);
        builder.add_entry("shared/data.yaml", b"vendor: Acme\n".to_vec());
        builder.add_entry("agent/context.md", b"do the thing\n".to_vec());
        let clan = ClanFile::from_bytes(builder.build().unwrap()).unwrap();

        let all = clan.read_all_entries().unwrap();
        let paths = clan.entry_paths().unwrap();
        assert_eq!(
            all.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(),
            paths
        );
        for (path, bytes) in &all {
            assert_eq!(bytes, &clan.read_entry(path).unwrap(), "mismatch at {path}");
        }
    }
}
