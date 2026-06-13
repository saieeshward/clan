# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.5] - 2026-06-13

### Added

- **Combined Windows installer** — the `.msi` now installs both the CLAN Viewer
  and the `clan` CLI, with the CLI added to the system `PATH` so it works from
  any terminal after install.
- **CLI in Linux packages** — the `.deb` and `.rpm` install the `clan` CLI to
  `/usr/bin` alongside the viewer. (The `.AppImage` stays viewer-only; standalone
  CLI tarballs remain for macOS and AppImage users.)

### Changed

- **One Windows artifact** — Windows ships a single combined `.msi` instead of a
  separate viewer installer plus CLI `.zip`.
- **Tag-driven versioning** — the release version is taken from the pushed git
  tag and stamped into the `clan` binary, the CLI archive names, and every
  desktop bundle, so all artifacts in a release share one version.

### Fixed

- **Reproducible release builds** — `Cargo.lock` is now committed, pinning
  dependencies so a floating upstream release can no longer break the release
  pipeline.
- **Installer version mismatch** — desktop bundles no longer lag the release tag
  (a `v1.1.4` tag previously produced `1.1.2`-stamped installers).

## [1.1.2] - 2026-06-12

### Added

- **`.clan` file associations** — double-clicking a `.clan` file (or "Open with")
  now launches the CLAN Viewer with the file loaded, on macOS, Windows, and Linux.

### Fixed

- **Viewer file opening** — files passed via OS launch events open correctly.

## [1.1.1] - 2026-06-12

### Added

- **CI/CD release pipeline** — tagged releases build CLI binaries for Linux, macOS
  (Apple Silicon + Intel), and Windows, plus viewer bundles (universal `.dmg`,
  `.AppImage`, `.deb`, `.rpm`, `.msi`, NSIS `.exe`) automatically.

### Changed

- **App icon** — viewer now ships the designed CLAN constellation mark
  (was a placeholder solid square).
- **Toolbar branding** — viewer toolbar renders the canonical `ClanMark`
  component, animated while a file loads.
- **README** — results updated to the 2026-06-12 scorecard run, including
  long-chain wall times and documented EXPECT-RED gaps.

## [1.1.0] - 2026-06-12

### Added

- **Fork/join concurrency** — per-agent namespaces, deterministic `merge`,
  contested-key reporting with provenance (spec §22–§27).
- **Deferred human-view rendering** (`clan render`) and conflict adjudication.
- **Teachable CLI interface** — `agent-help`, `next:` hints, F-series guard rails.

## [1.0.0] - 2026-06-08

First public release of CLAN — Context and Live Agent Notation.

### Added

- **CLAN specification** (`spec/CLAN-SPEC.md`) defining the `.clan` container format.
- **`clan-sdk`** — reference Rust SDK to read, write, validate, and pipeline `.clan` files,
  including decision chains, TOON serialization, and token-optimized agent prompt assembly.
- **`clan-cli`** — the `clan` command-line tool to create, validate, read, pack, and export `.clan` files.
- **CLAN Viewer** — Tauri desktop app for rendering the human view of a `.clan` file.

[Unreleased]: https://github.com/saieeshward/clan/compare/v1.1.5...HEAD
[1.1.5]: https://github.com/saieeshward/clan/releases/tag/v1.1.5
[1.1.2]: https://github.com/saieeshward/clan/releases/tag/v1.1.2
[1.1.1]: https://github.com/saieeshward/clan/releases/tag/v1.1.1
[1.1.0]: https://github.com/saieeshward/clan/releases/tag/v1.1.0
[1.0.0]: https://github.com/saieeshward/clan/releases/tag/v1.0.0
