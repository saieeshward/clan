# CLAN scorecard — 2026-06-12T13-58-13

CLI: clan 1.1.0 · cargo: skipped · conformance: 26/26

| Claim | Status | Value | Threshold |
|---|---|---|---|
| **C-CORE** Core lifecycle works: create, validate, read, info, patch, export | PASS | 9 tests green | all pass |
| **C-PROVENANCE** Every mutation is attributable: patch commands require agent/action (or explicit | PASS | 3 tests green | all pass |
| **C-FORK-MERGE** Parallel fan-out is safe by construction: namespace isolation enforced, conflict | PASS | 5 tests green | all pass |
| **C-AGENT-ERGONOMICS** Agents don't trip on the CLI: inline JSON, --set, --append, BOM tolerance, skip- | PASS | 5 tests green | all pass |
| **C-HINTS-CORRECT** Teaching hints are never wrong: no stale-view hint when patched keys are data-bo | PASS | 1 tests green | all pass |
| **C-TOON** TOON encoding saves >=30% vs minified JSON on uniform/tabular data (prose number | PASS | 1 tests green | uniform >= 30% saving |
| **C-SCAFFOLD** CLAN's fixed injection scaffolding is bounded: skip-guide intercept <= 3,000 cha | PASS | 1 tests green | intercept a <= 3000 chars |
| **C-GUIDE-STABLE** spec/agent-guide.md is byte-identical across all files and hops (prompt-cache pr | PASS | 1 tests green | 1 unique hash across corpus |
| **C-CHAIN-COMPRESS** Decision-chain two-tier compression: verbatim window intact, tail rationales com | PASS | 1 tests green | all pass |
| **C-LAYERS** Final artifacts carry the five handoff layers (spec §22): L1-L4 by construction; | NO-DATA |  | >= 4 of 5, L1-L4 all green |
| **C-DELTA** Delta-per-hop injection keeps per-hop injected size ~flat on sequential chains | PLANNED |  |  |
| **C-CACHE** The byte-stable guide produces real provider prompt-cache hits across agent invo | PLANNED |  |  |
| **C-REVISION-TOKENS** Revision-loop pipelines: CLAN patch path authors materially fewer output chars t | PASS | 0.336 | <= 0.65 |
| **C-PROV-E2E** A revision pipeline's final chain attributes every mutating hop | PASS | 1.25 | >= 1.0 |
| **C-CROSSOVER** Sequential chains: CLAN distilled re-injection beats ad-hoc accumulating re-read | NO-DATA |  | <= 10 |
| **C-SYNTH-WIN** At the synthesis hop, CLAN's merged injection beats ad-hoc re-reading all inputs | FAIL | 1.047 | < 1.0 |
| **C-FIDELITY** Nothing is silently lost: all requested edits present in final, untouched fields | PASS | 1 | == 1.0 |
| **C-RELIABILITY** Agents recover from every CLI error without orchestrator help | PASS | 0 | == 0 |
| **C-TEACHABLE** Unguided agents reach protocol competence from agent-help alone | MANUAL |  |  |
| **C-HITL** Human involvement is provable in the artifact (edited_by, patch fold-in) | MANUAL |  |  |
| **C-METAMORPHOSIS** One document can transform radically per hop (view + schema) with zero context l | MANUAL |  |  |
| **C-RESUME** Cold resume from a single file beats resuming from a directory | PLANNED |  |  |
| **C-XVENDOR** The artifact survives a vendor/model boundary; conventions don't | PLANNED |  |  |

**REGRESSIONS vs previous run: C-SYNTH-WIN**
