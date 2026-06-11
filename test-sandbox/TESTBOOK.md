# CLAN TESTBOOK

**The single catalog of every test CLAN runs to prove its claims and track its shortcomings.**
Audience: any AI/LLM runner (or human). Pick the tests for your cadence, execute exactly as written, record results into `TestResult.clan`. Version 1 · 2026-06-11 · covers CLI + agentic flows (viewer/app tests are out of scope — they need a human at the GUI).

---

## 0. Runner contract — read first

1. **Environment.** Repo root = the `clan` repository. Build the CLI first: `cargo build --release` → binary at `target/release/clan(.exe)`. All commands below assume the binary is on PATH or aliased as `clan`.
2. **Honesty rules.** Never fabricate a measurement. Numbers come from artifacts (files, exit codes, snapshot sizes), not from your own estimates. If a step fails, record the failure and your recovery — failures are findings, not embarrassments.
3. **Receipts.** Every agentic test requires a per-agent JSON receipt (schema in §2.4). Deterministic tests emit their own JSON.
4. **Scripts vs manual.** Where a script exists, use it — it keeps numbers comparable run-over-run. The manual fallback exists for runners without the script or when the script breaks; if you use the fallback, say so in your record.
5. **Record every run** — pass or fail — into `TestResult.clan` per §2. One recording command per run.
6. **Cost discipline.** Each test lists its token cost. Do not start a test whose budget you cannot finish.

## 1. Test index

| ID | Name | Kind | Cost | Cadence | Claims (claims.yaml) |
|---|---|---|---|---|---|
| D-CARGO | Workspace unit tests | deterministic | free | every change | C-CORE |
| D-CONF | CLI conformance & F-regressions | deterministic | free | every change | C-CORE, C-PROVENANCE, C-FORK-MERGE, C-AGENT-ERGONOMICS, C-HINTS-CORRECT |
| D-TOON | TOON encoding savings | deterministic | free | on toon.rs/inject.rs changes | C-TOON |
| D-INJECT | Injection anatomy: scaffolding bound + guide byte-stability | deterministic | free | every release | C-SCAFFOLD, C-GUIDE-STABLE |
| D-CHAINZIP | Decision-chain two-tier compression | deterministic | free | on compress.rs changes | C-CHAIN-COMPRESS |
| B-LAYERS | Five-layer handoff audit (spec §22) on every final artifact | deterministic over agentic outputs | free | after every agentic test | C-LAYERS |
| L-H1 | Revision loop (lite) | agentic | ~150k tok | each release | C-REVISION-TOKENS, C-PROV-E2E, C-FIDELITY, C-RELIABILITY |
| L-H2 | Long chain (lite) | agentic | ~150k tok | each release | C-CROSSOVER, C-SYNTH-WIN |
| B-META | Metamorphosis (flow-11) | agentic | ~150k tok | on pack-html/schema changes | C-METAMORPHOSIS |
| B-UNGUIDED | Teachable interface | agentic | ~100k tok | each minor version | C-TEACHABLE |
| B-FORKMERGE | Engineered contested keys (H5-L) | agentic | ~120k tok | on fork/merge changes | C-FORK-MERGE (e2e), conflict recall |
| B-HITL | Human attribution | agentic+human | ~80k tok | on viewer releases | C-HITL |
| L-H3 | Cold resume (lite) | agentic | ~80k tok | once, then on injection changes | C-RESUME |
| L-H4 | Cross-vendor relay (lite) | agentic | ~150k tok | once per foreign model | C-XVENDOR |
| P-DELTA | Delta-per-hop injection savings | blocked (v1.2 feature) | ~100k tok | when `read agent --since` ships | C-DELTA |
| P-CACHE | Prompt-cache hits on the stable guide | manual, needs provider API | ~$ small | once per provider; re-run when guide changes | C-CACHE |
| H-H1…H-H7 | Heavy suite | agentic | 0.1–2M each | per milestone | publishable numbers for all above |

Recommended order for a full session: D-CARGO → D-CONF → the agentic tests you have budget for → record.

---

## 2. TestResult.clan — the accumulating results ledger

One CLAN file is the progress tracker. The decision chain accumulates every run (who ran what, when, verdict); `shared/data.yaml` accumulates run records and keeps a combined overview. The latest run is always the newest chain entry + `latest_run` in data. The viewer renders the overview.

### 2.1 Bootstrap (only if `test-sandbox/TestResult.clan` does not exist)

```bash
cat > /tmp/tr-schema.json << 'EOF'
{ "type": "object", "properties": {
  "mode": { "type": "string" },
  "structured": { "type": "object", "properties": {
    "overview": { "type": "object", "description": "test-id -> {status,value,last_run} — combined latest state of every test" },
    "latest_run": { "type": "object" },
    "runs": { "type": "array", "description": "append-only full run records" } } } } }
EOF
clan create --title "CLAN Test Results" \
  --brief "Accumulating ledger of all TESTBOOK runs. Append runs via patch-data --append runs; merge overview + latest_run in the same patch. Never edit history." \
  --schema /tmp/tr-schema.json --output test-sandbox/TestResult.clan
```

### 2.2 Recording a run (one command, all three keys, attributed)

Author `run.json`:

```json
{ "runs": [ { "run_id": "2026-06-11-A", "date": "2026-06-11", "runner": "claude-fable-5",
    "clan_version": "1.1.0", "tests_run": ["D-CONF","L-H1"],
    "results": [
      { "test": "D-CONF", "verdict": "pass", "value": "21/22, T20 known-red", "notes": "" },
      { "test": "L-H1", "verdict": "pass", "value": "ratio 0.49; chain 8/8", "notes": "post-F14/F15" } ] } ],
  "latest_run": { "run_id": "2026-06-11-A", "tests": ["D-CONF","L-H1"], "verdicts": "pass,pass" },
  "overview": {
      "D-CONF": { "status": "pass", "value": "21/22", "last_run": "2026-06-11-A" },
      "L-H1":   { "status": "pass", "value": "0.49",  "last_run": "2026-06-11-A" } } }
```

```bash
clan patch-data test-sandbox/TestResult.clan run.json --append runs \
  --agent "<runner-id>" --action "ran D-CONF,L-H1: pass,pass" \
  --rationale "<one line of key numbers, e.g. 'H1 ratio 0.49 (was 0.652); provenance 8/8 (was 1/8)'>"
clan render test-sandbox/TestResult.clan     # refresh the human view
clan validate test-sandbox/TestResult.clan   # must print OK
```

Rules: `runs` is append-only (`--append runs` — never resend the array). `overview` and `latest_run` are merge-patched (send only changed test-ids). Always attribute (`--agent/--action`); never `--no-decision`. Verdicts: `pass | fail | known-red | blocked`.

### 2.3 Reading progress

`clan read chain test-sandbox/TestResult.clan` = run log, newest first. `clan read data` = overview + full history. `clan info` + viewer = rendered overview.

### 2.4 Agentic receipt schema (unchanged from research/14)

```json
{ "role","hop","arm","flow","commands_run":[],"files_read":[{"file","approx_chars"}],
  "files_written":[{"file","chars"}],"output_chars":0,
  "errors":[{"what","recovered_by"}],"problems_and_friction":"","context_understood":"" }
```

---

## 3. Deterministic tests

### D-CARGO — workspace unit tests
**Scope:** SDK + CLI + viewer-backend Rust units (container, pack, merge, TOON, patch guards, no-op save).
**Steps:** `cargo test --workspace`. **Expected:** exit 0, 0 failures. **Record:** `value` = "Np/Nf".

### D-CONF — CLI conformance & finding regressions
**Scope:** black-box CLI behavior: core lifecycle + a regression assertion for every fixed finding F1–F15, plus F2b (expect-red until implemented).
**Script:** `node test-sandbox/pipeline/conformance.mjs --clan <binary>` (or full: `node test-sandbox/pipeline/pipeline.mjs --clan <binary>` which also runs D-CARGO and emits a scorecard).
**Manual fallback** (assert each; temp dir; `CLAN_NO_HINTS=1` except T20):

| T | Assert |
|---|---|
| T01 | `create --title --brief --output` → file exists; `validate` OK; `info` shows title |
| T02 | `create --schema s.json` seeds real schema (export-static contains your property) — F9 |
| T03/T04 | `patch-data` accepts `--set k=v` and inline `{"k":1}` (attributed) — F13 |
| T05 | mutating patch with NO `--agent/--action/--no-decision` → non-zero exit, error names the flags — F15 |
| T06 | `--no-decision` → chain unchanged, never `unknown-agent` — F1 |
| T07 | attributed patch → chain entry has agent + `fields_changed` with the patched key — F15 |
| T08 | `--append risks` on existing 1-array + 1-array patch → length 2 — F14 |
| T09 | BOM-prefixed JSON file accepted — F3 |
| T10 | `read agent --skip-guide` ≥1k chars smaller than full |
| T11 | `patch-html` with non-matching selector → non-zero exit |
| T12 | `patch-html` body-append applies (fragment visible in `read human`) |
| T13 | `fork --agents a,b --output-dir d` → `d/a.clan`, `d/b.clan` |
| T14 | direct `patch-data` on a branch (no `--namespace`) → rejected, mentions namespace |
| T15 | `patch-data --namespace` on branches succeeds |
| T16 | `merge` of branches that both wrote `verdict` → `read report` lists `verdict` with branch provenance |
| T17 | `merge --policy notes=append` → both values present |
| T18 | `patch-decision --pinned` → entry with `pinned: true` |
| T19 | `create --no-render` then `render` → human view materialises |
| T20 | patch a `{{bound}}` key with hints ON → output contains NO stale-view hint — F2b, **expect-red** |
| T21 | `read decisions` works (alias of `read chain`) — F12 |
| T22 | `export-static` JSON has task/shared_data/decision_history_toon/output_schema keys |
| T23 | `patch-requirements req.yaml` then `read agent` → requirements surfaced in injected context; unmet requirement warns (not fails) — F8/L5 |

**Expected:** 0 hard failures; T20 red until F2b lands (when green: update claims.yaml + record it). **Record:** `value` = "passed/total + expect-red list".

### D-TOON — TOON encoding savings (the ~40% claim, honestly bounded)
**Scope:** research/12 claims ~40% token savings for TOON over JSON/YAML, flagged dataset-dependent. Pin it per data shape.
**Steps:** build two fixture files: (a) `uniform.clan` — data = one 20-row × 5-col table of short scalars; (b) `prose.clan` — data = 10 nested objects with sentence-length string values. For each: `chars_toon` = the data section of `clan read agent --skip-guide` output; `chars_json` = the same data from `export-static` (minified JSON).
**Expected:** uniform: TOON ≥ 30% smaller. Prose: record the actual number with NO threshold — it is the honest bound we publish ("40% on tabular, X% on prose"). Regression = uniform savings dropping below 30%.
**Record:** `value` = "uniform −N% / prose −M%".

### D-INJECT — injection anatomy: scaffolding bound + guide byte-stability
**Scope:** flow-14's loss was fixed scaffolding; v1.1's cache story needs a byte-stable guide. Both are measurable from the snapshot corpus you already have.
**Steps:** over ≥10 `.clan` snapshots from past runs (`happy/lite/*/snapshots/`, `benchmark/*/snapshots/`): (1) extract `spec/agent-guide.md` from each → hash; (2) for each, measure `read agent` full vs `--skip-guide`; (3) regress `injected_skipguide = a + b·data_chars` across the corpus.
**Expected:** all guide hashes IDENTICAL (one differing hash = C-GUIDE-STABLE fail = prompt-cache story dead); scaffolding intercept `a` ≤ 3,000 chars; guide-skip delta stable (~7.4k chars) across all files. Record `a`, `b`, and guide hash.

### D-CHAINZIP — decision-chain two-tier compression
**Scope:** spec §"decision-chain" promises verbatim window (default 5) + YAKE-compressed tail, all other fields untouched.
**Steps:** create a file; append 12 decisions with ~200-char rationales (`patch-decision` ×12, distinct agents); read chain.
**Expected:** newest 5 rationales byte-identical to authored; entries 6–12 rationales shorter than authored (compressed); `agent/action/timestamp/pinned` verbatim on ALL entries; one `pinned: true` entry older than the window stays uncompressed; file validates. **Record:** `value` = "tail saved N%".

---

## 4. Agentic tests — lite (the release loop)

> Shared conventions for §4–§5: sandbox under `test-sandbox/happy/lite/<test>/` with `work/`, `snapshots/` (copy artifact after every hop), `receipts/`; each agent = a REAL subagent (fresh context, no scripted outputs); knowledge-only (no web); identical briefs across arms byte-for-byte; metrics computed from snapshots/receipts, never self-reported.

### L-H1 — revision loop, patch vs rewrite
**Scope:** CLAN's surgical patch path vs ad-hoc file editing on 8 small serial revisions to a seeded ~45KB CRM report. THE output-token claim.
**Script path:** `test-sandbox/happy/lite/run-h1-h2.workflow.js` (H1 phases) + `measure-lite.ps1` → `metrics-lite.json`; then `node test-sandbox/pipeline/pipeline.mjs --metrics …` evaluates thresholds.
**Manual:** seed both arms from `_inputs/seed.clan` (CLAN) and its exported `report.html`+`data.json`+`decisions.log` (ad-hoc). Run the 8 changes (pricing-update, risk-add, date-update, summary-rewrite, residency-section, chart-update, verdict-flip, signoff — exact payloads in `_inputs/h1-supplied/` and the workflow file). CLAN arm: narrowest patch command only, `pack-html` FORBIDDEN, every patch attributed (`--agent <role> --action <change>`); use `--append risks` for hop 2. Ad-hoc arm: edit files however best, append one log line. Save authored payloads to `receipts/hopNN-output/`.
**Measure:** Σ authored chars per arm; final chain entries; fidelity (all 8 edits present; untouched fields byte-stable vs seed; `validate` OK).
**Expected:** ratio ≤ 0.65 (post-F14/F15 target ≤ 0.50); chain entries = 8/8 (C-PROV-E2E — was 1/8 pre-F15, must now pass); fidelity 1.0; 0 unrecovered failures.

### L-H2 — long chain, injection growth
**Scope:** injected-context growth over a 10-hop serial analyst chain, CLAN (`read agent --skip-guide`, patch-first) vs ad-hoc (findings files + handoff notes). Hops 1–9 data-only; hop 10 synthesizes (CLAN: designed-mode render; ad-hoc: report.html).
**Script path:** same workflow + measure as L-H1. Manual: roles and per-hop findings in `_inputs/h2-supplied/`.
**Measure:** injected chars per hop both arms (CLAN = `clan read agent --skip-guide` output length at each snapshot; ad-hoc = Σ files a hop must read); synthesis-hop ratio; crossover hop (excluding synthesis hop).
**Expected:** synth ratio < 1.0 (lite baseline 0.776). Crossover ≤ 10 is **expect-red at lite scale** (scaffolding dominates tiny hops) — record the curve; H-H2 settles the claim.

### L-H3 — cold resume *(first run pending)*
**Scope:** fresh agent resumes an interrupted pipeline from the artifact alone.
**Steps:** run L-H2 hops 1–2 both arms, stop. Preserve only `doc.clan` (CLAN) / the working dir (ad-hoc). New subagent, minimal prompt: *"You are taking over an in-progress analysis. Everything known is in <artifact>. Work out where it stands and complete the next step only."*
**Measure:** chars read to orient; probe commands before first productive write; correctness (right next step, no redo, schema respected, decision recorded).
**Expected:** CLAN orients in ≤3 reads with correct next step; ad-hoc needs more probing or errs. Record orientation cost with/without one-off `agent-help` discovery (~2–5k tok).

### L-H4 — cross-vendor relay *(first run pending)*
**Scope:** hop 2 of a 3-hop chain runs on a different vendor/model, **unguided** ("Continue the work in <artifact>; advance it one step"). The artifact is the only carrier.
**Measure:** five-layer scorecard (spec §22) at final; hop-2 protocol competence (write path, namespace, decision); discovery cost; errors.
**Expected:** CLAN final retains L1–L4 with the foreign hop attributed in-chain; ad-hoc arm shows convention drift. Name the foreign model in the record.

---

## 5. Agentic tests — claim-specific

### B-META — metamorphosis (research/14 flow-11)
**Scope:** one `doc.clan`, three hops, each a radically different document (agency brief → concept deck → client pitch): new schema per hop (`pack-html --schema`), full view replacement, nothing lost.
**Steps:** 3 real agents; hop briefs as research/14 §6b; carry one SVG asset from hop 1.
**Expected:** 3 visually distinct views; every generation validates; hop-1 fields (`single_minded_proposition`, `budget_eur`, persona) verbatim in final data; hop-2 concept names verbatim; unbroken lineage; **asset carried without re-passing `--assets`** (F10 regression); no full-data re-transcription needed (F11: omit-to-keep documented).

### B-UNGUIDED — teachable interface (research/14 flows 02/06 pattern)
**Scope:** protocol competence from the file itself. 3-hop serial chain where agents are told only: *"There's a `clan` CLI and a doc.clan. Figure it out."*
**Expected:** all hops complete; 0 namespace-guard violations; discovery ≤8 extra commands (~2–5k tok)/agent; correct use of patch-vs-pack; **with F15: discovery includes attribution flags** (the error message must teach them — if an agent gets stuck on the attribution error, that's a FAIL and a UX finding).

### B-FORKMERGE — engineered contested keys (H5-L)
**Scope:** conflict detection with guaranteed disagreement. Fork into 4 branches whose per-branch contexts (`fork --context-dir`) REQUIRE writing `recommendation`, `budget_eur`, `top_risks`, `assumptions` with seeded conflicting values; data-only branches; merge; synthesizer adjudicates from `read report`.
**Expected:** merge-report recall = 4/4 contested keys with winner/loser provenance; adjudication recorded via attributed patch + decision; ad-hoc comparison arm (shared `summary.json`) shows ≥1 silent overwrite. Branch writes only via `patch-data --namespace`.

### B-LAYERS — five-layer handoff audit (spec §22, systematized from research/14 §3)
**Scope:** research/14 scored finals against the five handoff layers by hand, once. This makes it a standard rubric applied to **every agentic test's final artifact** — run it as the closing step of L-H1/L-H2/B-META/B-FORKMERGE/H-* and include the score in that test's record.
**Rubric (artifact-only checks; ✓ / ~ / ✗ per layer):**

| Layer | ✓ requires | Check |
|---|---|---|
| L1 distilled state | structured data present, parseable, beyond `$schema` | `export-static` → `shared_data` parses with ≥5 fields |
| L2 transcript/handoff | current stage + handoff context readable by the next agent | `read context` / `state.yaml` reflects the LAST hop (not the seed brief); ~ if only chain rationales carry it |
| L3 contracts | real output schema | `output-schema.json` has named properties (non-stub — F9) |
| L4 provenance | every mutating hop attributed | chain entries ≥ mutating hops; all have agent+action+timestamp; `fields_changed` on data mutations; zero `unknown-agent`; human edits as `edited_by: human`/`agent: human` |
| L5 capability requirements | declared + surfaced | `agent/requirements.yaml` present with ≥1 declared need AND visible in `read agent` output |

**Expected:** CLAN finals score L1–L4 ✓ by construction (any ✗ = regression). **L5 is expect-red** until flows actually declare requirements — to flip it: add one `patch-requirements` step to L-H2's hop 1 (declare "needs: web-search none; filesystem read-write") and assert hop 2's injected context surfaces it (conformance T23 covers the mechanics; B-LAYERS covers real-flow adoption). Ad-hoc comparison arms: record their score too (research/14 baseline: 1–2.5 of 5) — the gap IS the claim.
**Record:** `value` = e.g. "L1✓ L2✓ L3✓ L4✓ L5✗ vs adhoc 2/5".

### B-HITL — human attribution (research/14 flows 09/10 pattern; needs a human)
**Scope:** a live human edit mid-pipeline ("CEO: year-1 ≤ €40,000" in the viewer) must be obeyed AND provable.
**Expected:** final honors the cap; edit carries `edited_by: human` + timestamp; agent decisions cite the human edit; on full-html repack the superseded patch folds into the chain as `agent: human` (F5). Compare: ad-hoc arm obeys but cannot prove a human was involved.

---

## 6. Heavy suite (H-H1 … H-H7) — publishable numbers

Full designs, arms, reps, and win criteria live in **research/15** (run plan B). Summary for the runner:

| Test | Design | Reps×arms | Win criterion |
|---|---|---|---|
| H-H1 | L-H1 at full size + verifier agent | 5×2 | ratio ≤ 0.50, fidelity 1.0, zero unintended diffs |
| H-H2 | 12-hop chain, full content | 5×2 | crossover hop ≤ 10, widening after |
| H-H3 | cold resume, full | 2×2 | ≤3 reads, correct resume both reps |
| H-H4 | cross-vendor, real foreign vendor | 2×2 | L1–L4 retained; competence from agent-help |
| H-H5 | contested keys, HTML branches | 3×2 | 4/4 recall + provenance; adjudication in chain |
| H-H6 | window pressure, 25KB×6 batches | 3×2 | hop-6 injection ≤50% of replay OR strictly better probe recall |
| H-H7 | audit replay, 10 questions, artifact-only | 4 artifacts | CLAN ≥9/10, ad-hoc ≤5/10 |

Never publish a number from a lite run; heavy runs only, with variance bands (n per research/15).

## 6b. Planned token-efficiency tests (blocked — do not run yet)

### P-DELTA — delta-per-hop injection savings *(blocked on v1.2 `read agent --since <parent-sha>`)*
**Scope:** research/13 OQ2 — the highest-leverage token feature: inject only namespace writes + chain entries since the parent snapshot instead of the whole document.
**Steps (when shipped):** re-run L-H2 with hop N+1 reading `--since` instead of full; same measure pass.
**Expected:** per-hop injected chars roughly flat (delta-sized) instead of growing; combined with TOON, this is what finally beats lean ad-hoc on EVERY hop, not just synthesis. Threshold set after first measurement.

### P-CACHE — prompt-cache hits on the byte-stable guide *(needs provider API access)*
**Scope:** research/13 OQ3 — the byte-stable ~800-token guide is cache-friendly *in theory*; nobody has verified an actual cache hit across separate agent invocations.
**Steps:** precondition: D-INJECT guide hashes identical. Make two consecutive API calls (Anthropic, then repeat for OpenAI) whose prompts start with the identical injected guide prefix; read the provider's usage fields (`cache_read_input_tokens` / `cached_tokens`).
**Expected:** call 2 reports the guide prefix as cache-read (≈ guide size). If providers' cache scoping prevents hits across invocations, record that honestly and drop the caching claim from the pitch (per research/13: "do not claim savings without verification").
**Record:** per provider: "hit/miss, N cached tokens".

---

## 7. Session checklist

1. `cargo build --release` → D-CARGO → D-CONF (+ D-TOON/D-INJECT/D-CHAINZIP if the relevant code changed). Any hard failure: stop, record, fix first.
2. Run the agentic tests for your cadence (§1 table).
3. Close every agentic test with the B-LAYERS audit on its final artifact; include the layer score in the record.
4. Compute metrics from artifacts (scripts where available).
5. Record ONE run entry into `TestResult.clan` (§2.2) covering everything you ran, with per-test verdicts. Validate the file.
6. If an expect-red went green or a pass regressed: say so in the decision rationale, and flag `claims.yaml` for update.
7. New friction discovered = candidate finding: describe it in the run record `notes`; maintainer triages into TODO.md F-numbering.

*Provenance: collated from research/02 (CLI matrix), research/11 (patch commands), research/12–13 (token threads), research/14 (flows 01–11 + F1–F12 + §3 five-layer scoring), the lite pass (F13–F15, F2b), research/15 (H1–H7), TODO.md, and pipeline/claims.yaml. When this book and claims.yaml disagree, claims.yaml wins for thresholds; this book wins for procedure.*
