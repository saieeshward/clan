# CLAN progress pipeline

One repeatable flow that tests every claim we make and every shortcoming we're tracking. Run it after each fix batch; compare runs via `history.jsonl`.

## The four stages

| Stage | What | Cost | When |
|---|---|---|---|
| 0 | `cargo test --workspace` | minutes, free | every run |
| 1 | `conformance.mjs` — 22 black-box CLI assertions covering core lifecycle + every fixed finding F1–F15 (+ F2b as expect-red) | seconds, free | every run |
| 2 | Agentic lite benchmark — H1 (revision loop) + H2 (long chain), real subagents via `../happy/lite/run-h1-h2.workflow.js` → `metrics-lite.json` | ~300k tokens | each release / after CLI-behavior changes |
| 3 | Scorecard — `claims.yaml` × stage outputs → `results/<run>/scorecard.{json,md}`, appends `history.jsonl`, flags regressions vs previous run | free | every run |

## Usage

```powershell
cargo build --release
# cheap loop (after any code change):
node test-sandbox/pipeline/pipeline.mjs --clan target/release/clan.exe

# full loop (after running the agentic flows):
#   1. run the H1/H2 workflow with subagents (see ../happy/lite/run-h1-h2.workflow.js)
#   2. regenerate metrics: pwsh ../happy/lite/measure-lite.ps1
node test-sandbox/pipeline/pipeline.mjs --clan target/release/clan.exe --metrics test-sandbox/happy/lite/metrics-lite.json
```

Exit code is non-zero on any cargo failure, conformance hard-failure, or claim regression (pass → fail vs the previous history entry) — CI-friendly.

## Reading the scorecard

- **PASS / FAIL** — claim tested this run.
- **KNOWN-RED** — tracked shortcoming, `expect: red` in `claims.yaml` (currently: C-HINTS-CORRECT / F2b; C-CROSSOVER pending heavy run). Going red→green prints a reminder to update `claims.yaml`; green→red counts as a regression.
- **NO-DATA** — agentic claim with no (or stale) `metrics-lite.json`. Decide consciously whether to accept the stale number or re-run stage 2.
- **MANUAL** — claims proven once in research/14 (teachability, HITL, metamorphosis) with a re-verify cadence noted in `claims.yaml`; not automated.
- **PLANNED** — H3 (cold resume) / H4 (cross-vendor) from research/15, not yet implemented as flows.

## Rules of the road

1. **`claims.yaml` is the contract.** A new claim in a README/spec/pitch must land here with a test or an honest `manual`/`planned` tag — same discipline in reverse for new findings: every F-fix gets a conformance test before it's marked fixed in TODO.md.
2. **Never publish a number that didn't come from a scorecard run** (lite numbers are directional; heavy runs per research/15 produce publishable ones).
3. **`history.jsonl` is append-only.** It is the progress record — don't regenerate or edit it.
4. Conformance tests must stay **black-box** (CLI in, files/exit codes out) so they keep working across SDK refactors.

## Provenance of the test set

Collated from: research/02 (CLI test matrix), research/11 (patch commands), research/14 findings F1–F12, the lite-pass findings F13–F15 + F2b (TODO.md), and the claim set of research/14 §7 + research/15. The agentic stage reuses the lite benchmark flows verbatim so numbers stay comparable run-over-run.
