# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/saieeshward/clan/compare/v1.1.1...HEAD
[1.1.1]: https://github.com/saieeshward/clan/releases/tag/v1.1.1
[1.1.0]: https://github.com/saieeshward/clan/releases/tag/v1.1.0
[1.0.0]: https://github.com/saieeshward/clan/releases/tag/v1.0.0
