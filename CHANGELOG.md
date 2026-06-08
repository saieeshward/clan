# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-06-08

First public release of CLAN — Context and Live Agent Notation.

### Added

- **CLAN specification** (`spec/CLAN-SPEC.md`) defining the `.clan` container format.
- **`clan-sdk`** — reference Rust SDK to read, write, validate, and pipeline `.clan` files,
  including decision chains, TOON serialization, and token-optimized agent prompt assembly.
- **`clan-cli`** — the `clan` command-line tool to create, validate, read, pack, and export `.clan` files.
- **CLAN Viewer** — Tauri desktop app for rendering the human view of a `.clan` file.

[Unreleased]: https://github.com/saieeshward/clan/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/saieeshward/clan/releases/tag/v1.0.0
