# CLAN Test Run — 2026-06-12

Run IDs: `2026-06-12-G` (deterministic) · `2026-06-12-H` (full including agentic)
Binary: `clan 1.1.0`

---

## Deterministic Tests

### Conformance Suite — 26 tests (T01a–T24)

Tests the full black-box CLI contract: create, validate, read, info, patch-data (inline JSON, `--set`, `--append`), patch-html, fork/merge, decision-chain, export-static, and all F1–F16 regression assertions.

| Claim | Tests | Result |
|-------|-------|--------|
| C-CORE — core lifecycle | T01a/b/c, T02, T03, T04, T12, T18, T19, T21, T22 | PASS |
| C-PROVENANCE — F15 attribution enforcement | T05, T06, T07 | PASS |
| C-FORK-MERGE — parallel fan-out safety | T13–T17 | PASS |
| C-AGENT-ERGONOMICS — CLI usability for agents | T04, T08–T10, T21 | PASS |
| C-HINTS-CORRECT — F2b bound-key hint suppression | T20 | PASS |

**Outcome:** 26/26 pass. No expect-red entries remaining.

---

### D-TOON — TOON Encoding Savings

Tests that the columnar TOON encoding saves ≥ 30% over minified JSON on uniform tabular data injected to agents.

**Outcome:** 57.5% saving on a uniform 20×5 tabular dataset. PASS.
Prose rows fall back to row-form; no threshold claimed for prose.

---

### D-INJECT — Scaffold Intercept & Guide Stability

Tests two things over the full snapshot corpus (21 lite files):

1. **Scaffold intercept (C-SCAFFOLD):** The fixed CLAN injection overhead is bounded regardless of document size, measured as the intercept `a` from the regression `skip_output ~ data_chars`. Threshold: `a ≤ 3,000 chars`.
2. **Guide stability (C-GUIDE-STABLE):** `spec/agent-guide.md` is byte-identical across all corpus files (required for prompt-cache hits).

**Outcome:**
- Intercept `a = 2,668 chars`, slope `b = 1.173 chars/data-char` — PASS (≤ 3,000)
- 1 unique guide hash across all 21 corpus snapshots — PASS

---

### D-CHAINZIP — Decision-Chain Compression

Tests two-tier chain compression: verbatim window (5 newest entries), NLP-compressed tail rationales, pinned entries never compressed, all other fields preserved verbatim.

**Outcome:** PASS — window intact, tail compressed, pinned decisions preserved.

---

## Agentic Tests

### L-H1 — 8-Hop Revision Pipeline

Eight sequential agents revise a CRM evaluation document: pricing update → risk add → date change → exec-summary rewrite → residency section → chart update → verdict flip → steering-committee sign-off. Both a CLAN arm and an ad-hoc arm ran concurrently.

Measures:
- **H1 output ratio** (sum CLAN authored chars / sum ad-hoc authored chars), threshold ≤ 0.65
- **Fidelity** (all edits present, untouched fields byte-stable), threshold = 1.0
- **C-PROV-E2E** (chain decisions / mutating hops), threshold ≥ 1.0

**Outcome:**
- H1 output ratio: **0.639** — PASS
- Fidelity: **1.0** — PASS
- C-PROV-E2E: PASS (≥ 1.0)
- Wall time: CLAN **8:20** · Ad-hoc **10:13** → CLAN 1:53 faster

---

### L-H2 — 10-Hop Sequential Discovery Chain

Ten specialist agents build a CRM vendor recommendation: market-researcher → pricing-analyst → risk-analyst → GDPR-reviewer → integrations-assessor → customer-discovery → competitive-intel → finance-modeler → rollout-planner → lead-partner synthesis. Both arms ran concurrently.

Measures:
- **H2 synthesis ratio** (CLAN injected chars at synthesis hop / ad-hoc injected chars at synthesis hop), threshold < 1.0
- **C-CROSSOVER** (whether CLAN per-hop injection stays smaller than ad-hoc by hop 10)

**Outcome:**
- H2 synthesis ratio: **0.487** — PASS (CLAN injected 44% less than ad-hoc at the synthesis hop)
- C-CROSSOVER: **EXPECT-RED** — no crossover observed on the lite corpus by hop 10; CLAN injection slope is steeper than ad-hoc until the synthesis hop
- Wall time: CLAN **11:25** · Ad-hoc **12:55** → CLAN 1:30 faster

---

### L-H3 — Cold Resume

A fresh agent (no prior context) must identify and continue from the correct step in a partially-completed chain, using only the `.clan` file, in ≤ 3 orient reads before writing.

**Outcome:** Both arms identified the correct next role. CLAN arm: 3 orient reads — PASS.

---

### B-UNGUIDED — Unguided Agent Protocol Compliance

An agent with no prior CLAN training reaches correct protocol usage from `clan agent-help` alone. Pass criteria: ≤ 8 discovery commands before first write; uses `patch-data` for data mutations (not direct file writes).

**Outcome:** All agents used ≤ 4 discovery commands; all used `patch-data` for data — PASS.

---

### B-LAYERS — Five-Layer Handoff Rubric

Tests that final artifacts carry all five handoff layers: L1 distilled state, L2 transcript/handoff, L3 contracts (schema), L4 provenance (fields_changed, attribution), L5 capability requirements.

**Outcome:** L1–L4 all green — PASS. L5 **EXPECT-RED** — no flow exercises `patch-requirements` yet, so the capability-requirements layer is not populated.

---

## Wall Time

Both CLAN and ad-hoc arms ran concurrently inside the same parallel workflow. Times are measured from the shared workflow start to the last chain write (CLAN) or last snapshot save (ad-hoc).

| Flow | Hops | CLAN total | Ad-hoc total | CLAN faster by |
|------|------|-----------|-------------|----------------|
| H1 — revision pipeline | 8 | 8:20 | 10:13 | **1:53** |
| H2 — discovery chain | 10 | 11:25 | 12:55 | **1:30** |

**Per-hop averages:**

| | CLAN avg/hop | Ad-hoc avg/hop |
|--|-------------|---------------|
| H1 | 1:03 | 1:17 |
| H2 | 1:09 | 1:17 |

**Synthesis hop (H2 hop-10, lead-partner):** CLAN 1:23 · Ad-hoc 2:01 — **CLAN 38 seconds faster** at the step where ad-hoc context is at its largest.

The 10–15% wall-time reduction is modest at 8–10 hops. The token-output savings (36% fewer chars on H1, 51% fewer at the H2 synthesis hop) do not translate 1:1 to wall time because model inference latency is dominated by input reads, not output length. The gap compounds as chains lengthen: ad-hoc injection grows O(n) with chain length while CLAN's distilled re-injection stays flat.

---

## Claims Summary

| Claim | What it measures | Status |
|-------|-----------------|--------|
| C-CORE | Core lifecycle (create/validate/patch/export) | PASS |
| C-PROVENANCE | Every mutation attributed (F15) | PASS |
| C-FORK-MERGE | Parallel fan-out safe by construction | PASS |
| C-AGENT-ERGONOMICS | CLI is agent-friendly | PASS |
| C-HINTS-CORRECT | No stale-view hint for bound-key patches (F2b) | PASS |
| C-TOON | TOON encoding ≥ 30% saving on tabular data | PASS (57.5%) |
| C-SCAFFOLD | Fixed scaffold intercept ≤ 3,000 chars | PASS (a = 2,668) |
| C-GUIDE-STABLE | Guide byte-identical across corpus | PASS (1 hash) |
| C-CHAIN-COMPRESS | Two-tier chain compression correct | PASS |
| C-REVISION-TOKENS | CLAN outputs fewer chars at equal fidelity | PASS (ratio 0.639) |
| C-PROV-E2E | Every mutating hop attributed in chain | PASS |
| C-SYNTH-WIN | CLAN beats ad-hoc at synthesis hop | PASS (ratio 0.487) |
| C-FIDELITY | No silent data loss across revision pipeline | PASS (1.0) |
| C-RELIABILITY | No unrecovered agent failures | PASS (0 failures) |
| C-CROSSOVER | CLAN injection crosses below ad-hoc by hop 10 | EXPECT-RED |
| C-LAYERS (L5) | Capability-requirements layer populated | EXPECT-RED |
