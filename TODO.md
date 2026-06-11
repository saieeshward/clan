# CLAN — To-Do

Priority order: fix blockers first, then correctness, then packaging, then optimisation.

---

## Blockers (fix before any public release)

- [X] `Cargo.toml:8` — change `license = "Apache-2.0"` to `"MPL-2.0"` (wrong license on every crate)
- [x] `Cargo.toml:9` — fix `repository` URL (`xon` → `clan`)
- [ ] Add `.github/workflows/ci.yml` — cargo check + test + tsc on every PR
- [ ] Add `.github/workflows/release.yml` — tag push → CLI binaries + DMG/MSI/AppImage via `tauri-action`
- [x] Add `.nvmrc` containing `20` in `app/` (Node 16 users get cryptic failures)
- [x] Add `"engines": { "node": ">=20" }` to `app/package.json`

---

## Code Bugs

### Critical

- [x] **Double-save on every edit** — `app/src-tauri/src/main.rs:441–448` [TEST]
  The `clan://patch` URI handler calls `do_save_patch` AND emits `clan-patch-saved`. React listener calls `invoke('save_patch')` on that event. Every edit writes the file twice. Fix: URI handler should only emit, or only save — not both.

- [x] **Double file read in `open_clan`** — `app/src-tauri/src/main.rs:72–96` [TEST]
  `ClanFile::open()` reads the file; then `std::fs::read(&p)` reads it again. Use `clan.raw_bytes()` instead of the second read.

- [x] **`has_entry` fully decompresses the entry just to check existence** — `crates/clan-sdk/src/container.rs:76–78` [TEST]
  `read_named` allocates and decompresses into `Vec<u8>`. Should check the ZIP central directory only, without decompressing.

- [x] **O(n) ZIP opens in `pack()`** — `crates/clan-sdk/src/pack.rs:261–273` [TEST]
  `entry_paths()` + per-entry `read_entry()` in a loop = N+1 ZipArchive instantiations per pack. Add `read_all_entries() -> Result<Vec<(String, Vec<u8>)>>` to `ClanFile` for a single-pass read.

- [x] **`strip_scripts` calls `to_lowercase()` on every loop iteration** — `crates/clan-sdk/src/pack.rs:483–492` [OPT]
  Full O(n) string allocation on each pass through the loop. Compute `lower` once before the loop, track offsets into the original.

- [x] **`patch_decision` runs full schema validation just to append a decision** — `crates/clan-sdk/src/pack.rs:594–615` [TEST]
  Routes through `pack()` which validates all structured data. Should use `repack_with_entry` to swap only `agent/decision-chain.yaml`.

- [x] **`pack_html` with `context_handoff` clones archive bytes 3× ** — `crates/clan-sdk/src/pack.rs:381–394` [TEST] [IMP]
  `bytes.clone()` → ClanFile1 → manifest clone → ClanFile2 → full entry iteration. Carry the manifest and entries through without rebuilding.

- [x] **`strip_on_handlers` mishandles `>` inside non-`on*` attribute values** — `crates/clan-sdk/src/pack.rs:499–546` [TEST][OPT]
  `<div class="a>b" onclick="evil()">` exits tag state at the `>` inside the class value. The `>` guard must only apply outside quoted strings.

### Non-critical

- [x] **`cmd_agent_help` hardcodes version `v0.13`** — `crates/clan-cli/src/main.rs:497` [GOOD]
  Binary is v1.0.0. Replace with `env!("CARGO_PKG_VERSION")`.

- [x] **`ammonia` is a dead dependency** — `crates/clan-sdk/Cargo.toml` [GOOD]
  Listed but never imported in the SDK. Remove it.

- [x] **`resolve_bindings` allocates `Vec<char>` over the full HTML** — `app/src-tauri/src/main.rs:193`
  `html.chars().collect()` allocates ~4× the HTML size. Replace with byte-indexed `str::find("{{")`. [IMP]

- [x] **`auto_inject_adf_ids` makes 10 full passes over the HTML** — `app/src-tauri/src/main.rs:333–340` [GOOD]
  One pass per tag type (`h1…h6, p, li, td, th`). Rewrite as a single-pass state machine.

- [x] **`patch-html` silent failure on non-matching selector** — `crates/clan-sdk/src/pack.rs:449–475` [GOOD][DEV]
  `apply_html_patch` returns the original HTML unchanged when the selector matches 0 elements, exits 0, and prints "Patched in-place". Should exit non-zero and warn to stderr.

- [x] **`clan create` uses a positional arg while `pack`/`pack-html` use `--output`** — `crates/clan-cli/src/main.rs:248` [TEST][OPT]
  Inconsistent CLI interface. Standardise on `--output` across all write commands.

---

## Token Optimisations

- [x] **Schema injection is raw JSON** — `crates/clan-sdk/src/inject.rs:59` [TEST HEAVY]
  `agent/output-schema.json` is injected verbatim. JSON schema boilerplate (`"type": "object"`, `"properties": {`) is ideal for TOON-style compression — estimated ~40% saving on schema tokens.

- [x] **Agent guide injected in full on every `clan read agent`** — `crates/clan-sdk/src/inject.rs:51` [OPT][HEAVY TEST]
  The guide is ~800 tokens and identical across all clan files. Add a `--skip-guide` flag (or version-hash check) so agents operating in a sequence can skip re-reading it.

- [x] **`fields_changed` on old decision-chain entries is noise** [HEAVY TEST]
  TOON output for decision-chain entries deep in the tail includes `fields_changed` lists that add tokens without value. Strip or omit empty lists in TOON encoding.

---

## Open Source Packaging

### Crate publishing (`clan-sdk`)

- [x] Add `keywords`, `categories`, `description` to `[workspace.package]` in `Cargo.toml`
  ```toml
  keywords = ["clan", "agent", "ai", "document", "pipeline"]
  categories = ["encoding", "file-formats", "parser-implementations"]
  ```
- [x] Create `crates/clan-sdk/README.md` — crates.io renders this; without it the page is blank
- [x] Promote `lol_html = "2.1.0"` from `clan-sdk/Cargo.toml` into `[workspace.dependencies]`
- [x] Add `[[bin]]` entry to `crates/clan-cli/Cargo.toml` declaring binary name explicitly
- [ ] Add correct logo for the app.
- [x] ASCII Art on first installation [OPT]

### App packaging

- [x] Rename `app/package.json` `"name"` from `"app"` to `"clan-viewer"`
- [x] Set `app/package.json` `"version"` to match workspace version (`1.0.0`)
- [ ] Add correct logo for the app.
- [ ] File Tree in App [OPT]

### Repository hygiene

- [x] Add `CHANGELOG.md` with v1.0.0 entry
- [x] Add `.github/ISSUE_TEMPLATE/bug_report.md`
- [x] Add `.github/ISSUE_TEMPLATE/feature_request.md`
- [x] Add `.github/PULL_REQUEST_TEMPLATE.md`

### Distribution

- [ ] CLI: publish `clan-cli` to crates.io on tag + attach pre-built binaries (Linux x86_64/ARM, macOS ARM/Intel, Windows x86_64) via `cross`
- [ ] SDK: publish `clan-sdk` to crates.io on tag
- [ ] Viewer: attach `.dmg`, `.msi`, `.AppImage` to GitHub Release via `tauri-action`
- [ ] Spec: host `spec/CLAN-SPEC.md` at a versioned URL and reference from crates.io docs
- [ ] Packages on github for download : Discuss

---

## v1.1 — Multi-Agent Flow (design: `spec/V1.1-DRAFT.md`, evidence: `research/13`)

### Phase 1 — Format foundations (SDK)

- [x] Manifest: `view: {present, renderable, stale}` field; `pack`/`patch-data` set `stale: true` when data changes without view update (§23)
- [x] Manifest: `fork: {agent_id, namespace, forked_from}` block (§24.1)
- [x] Manifest: `merge_policies: {default, keys: {...}}` — policies: `last-write`, `append`, `max`, `min`, `agent-priority` (§24.3)
- [x] Lineage: `parents[]` multi-parent form; keep single-parent form valid (§24.2)
- [x] New optional member `agent/requirements.yaml` — validated + injected when present, warn-not-fail (§22)
- [x] Pass-through preservation by construction in all pack/patch paths (§22)
- [x] Enforcement: on forked files, writes outside `agents/<agent_id>/` are rejected at write time with teaching errors; namespace members validated (§18 amendment)

### Phase 2 — Fork / merge / render (CLI + SDK)

- [x] `clan fork <parent> --agents a,b,c --output-dir <dir>` — one child per agent, namespace stamped (§24.1)
- [x] `clan merge <branches...> --output <file>` — deterministic per-key fold, no LLM call; branch decisions folded in timestamp order; `--prune-namespaces` (§24.3)
- [x] `merge-report.yaml` — every contested key with winner/losers/provenance; exit 0 with conflicts (conflicts are data); `unresolved` counter (§24.4)
- [x] `clan read report <file>` — TOON-encoded merge report
- [x] Adjudication flow: `patch-data` on contested key removes it from report + decrements `unresolved`; hint pairs it with `patch-decision` (§25)
- [x] `--no-render` on `create`; `clan render <file>` materialises human view on demand (§23) — `pack` inherits view state automatically
- [x] `--namespace` routing on `patch-data`; `patch-decision` auto-routes on forked files (§24.1)

### Phase 3 — Teachable interface (CLI)

- [x] Hint engine: deterministic `f(command, result, file state)` → ≤3 `next:` lines on stderr; precondition-gated (never mention capabilities the file state doesn't exhibit) (§27.1)
- [x] `--quiet` / `CLAN_NO_HINTS=1` suppression (§27.1)
- [x] Teaching errors: every guardrail rejection names the correct alternative command (§27.2) — kills the `structured:`-trap bug class
- [x] Initial hint map per §27.3 (create, fork, pack, pack-html, merge, patch-data-on-contested-key, patch-decision-on-branch, validate, stale-view, render)
- [x] `agent-help`: fork/merge/conflict/render section added

### Phase 4 — Injection + viewer

- [x] Conditional injection blocks after byte-stable base guide: fork block, unresolved merge report (TOON), renderable-no-view note, requirements (§26); sequential unforked files get exactly the v1.0 injection (test-asserted)
- [x] `spec/agent-guide.md` untouched — byte-stable (prompt-cache friendly); shrink target ≤400 tokens by v1.2 as hints absorb protocol mechanics (§27.4)
- [ ] Viewer: badge bindings whose key is in `merge-report.yaml`; show competing values + agent provenance; pick → `patch-data` + `patch-decision` via existing patch bridge (§25)
- [x] Folded §22–§27 into `CLAN-SPEC.md` (v1.1); embedded `spec/clan.md` gained Parallel Work + Optional Human View sections

### Benchmark findings F1–F12 (research/14; fixed 2026-06-11)

- [x] **F1** `patch-data`/`pack` no longer append an `unknown-agent / processed document` decision when none is supplied
- [x] **F2** stale-view hint is view-source-aware (`view.source`); never suggests destructive `render` over a hand-authored view
- [x] **F3** CLI strips a leading UTF-8 BOM on all text inputs (PowerShell 5.1 `Out-File` default no longer breaks JSON parsing)
- [x] **F4** viewer edit bridge strips instrumentation before saving + skips no-op patches; SDK-side no-op guard in `do_save_patch`. **VERIFIED 2026-06-11** (VS Build Tools reinstalled): Rust no-op guard has a regression test; runtime check in the production build confirmed saved patch content carries no instrumentation and an unchanged element produces no patch
- [x] **F5** full-html replacement folds superseded `human/patches.yaml` into the decision chain as an `agent: human` entry before dropping them
- [x] **F6** prose-key conflicts carry an `append` suggestion in the merge report; branch-mode injection + agent-help warn about prose-key collisions
- [x] **F7** `clan fork --context-dir` overrides per-branch `context.md`; every branch gets a branch-mode banner so inherited full-html/design instructions don't mislead
- [x] **F8** `clan patch-requirements` + `clan create --requirements` make `agent/requirements.yaml` (layer 5) writable
- [x] **F9** `clan create --schema <file>` seeds a real output schema instead of the `{type:object}` stub
- [x] **F10** full-html replacement carries parent `human/assets/` forward (new payload assets overwrite by path)
- [x] **F11** documented `pack-html` frontmatter merge-patch semantics (omit fields to keep them) in agent-guide.md, clan.md, agent-help
- [x] **F12** `clan read decisions` is an alias of `clan read chain`

### Deferred to v1.2 (do not start)

- [ ] Event-log / append-only delta state (state = fold of deltas)
- [ ] Delta-per-hop injection (`read agent --since <parent-sha>`) — measure savings first (research/13 OQ2)
- [ ] LLMLingua-style compression / key interning / schema-default omission — no verified numbers yet

### Strategic backlog (from 2026-06-10 deep-dive, separate from v1.1)

- [ ] Python bindings (PyO3/maturin) + JS/TS package — #1 adoption blocker
- [ ] `clan-mcp` MCP server exposing read/pack/patch as tools
- [ ] TOON naming: confirm spec-compatibility with public `toon-format/toon` or rename §14 (research/12 action)
- [ ] Conformance kit: versioned spec URL + valid/invalid test-vector `.clan` files
- [ ] `pack-html` warning when frontmatter lacks `structured:` key (audit: 75% first-try failure)
- [ ] README: rewrite conversational "Why a CLI?" section
- [ ] Move simulation artifacts (`research/strategy.md`, `06-`, `regulatory.md`) out of `research/` before going public
- [ ] One cross-framework demo: LangGraph → OpenAI Agents → human review in viewer
