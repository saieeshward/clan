# 15 — Happy-Path Benchmark Plan: Where CLAN Should Win

**Date:** 2026-06-11 · **CLAN version:** 1.1.0 · **Status:** PLAN (not yet run) · **Predecessor:** research/14

## Why this benchmark exists

Flow-14's token result ("ad-hoc is 15–40% leaner on injected context") is real but misleading as a headline, for three measured reasons:

1. **CLAN's biggest output-token lever was never exercised.** Every measured arm — CLAN and ad-hoc alike — had agents author full magazine-quality HTML per hop. `patch-data`/`patch-html` never ran in a measured arm. Writing tokens (the dominant cost: ~31.7k avg/agent spend vs ≤2.5k injected-context delta) were identical in both arms by construction, leaving only CLAN's fixed scaffolding visible.
2. **Fixed overhead never amortized.** CLAN's per-hop overhead (~2–5k chars: schema, chain, digest, namespace blocks) is roughly constant; the task was 3 hops of modest content. Research/14's own words: "at 3 hops the curves haven't crossed yet."
3. **The baseline was best-case ad-hoc.** Frontier agents spontaneously invented clean conventions. That discipline is emergent and unenforced — flow-14 §3 shows it collapses in the unguided arms — but it made the comparison arm as lean as ad-hoc can ever be.

This plan defines **seven happy-path flows (H1–H7) engineered around the places current agentic infrastructure actually suffers**: repeated small revisions, long chains, interruption/resume, cross-vendor handoff, concurrent write conflicts, context-window pressure, and post-hoc auditability. No failure injection, no adversarial inputs — best case both arms, and we measure who wins.

---

## Global protocol (applies to every flow)

**Sandbox layout** (mirrors flow-14): `test-sandbox/happy/<flow>-rep<N>/` containing `work/`, `snapshots/`, `receipts/`. Extend `measure.ps1` to walk `happy/` as well.

**Runner conventions**
- Real subagents, no scripted outputs. Same model for every agent in a head-to-head pair (exception: H4, where the model difference *is* the variable).
- Knowledge-only (no web) unless a flow says otherwise, so content quality is comparable across arms.
- Every agent files a JSON receipt (same schema as flow-14: `commands_run`, `files_read` with `approx_chars`, `files_written`, `errors`, `problems_and_friction`, `context_understood`) and snapshots its hop artifact before exiting.
- `$env:CLAN_NO_HINTS` unset (hints on) — happy path means the product as shipped.
- All metrics computed from artifacts/snapshots by the measure pass, never from agent self-report.

**Fairness rules** (so the results survive scrutiny)
- Both arms get equally good *guided* prompts. Ad-hoc arms get explicit, well-designed file conventions — we are benchmarking infrastructure, not prompt sabotage.
- Identical task briefs, role lists, and edit/change instructions across arms, byte-for-byte where possible.
- CLAN arms must not receive content hints absent from ad-hoc arms (and vice versa).
- Any arm may fail; failures are reported, not rerun (except infra incidents, which are written off and logged as in flow-14).

**Metrics (upgraded from flow-14 — this fixes the measurement gap)**

| Metric | How measured | New vs 14? |
|---|---|---|
| Injected context / hop | `clan read agent` chars (CLAN) vs sum of files a hop must read (ad-hoc), from snapshots | same |
| **Output chars / hop** | sum of chars of files + command payloads the agent wrote, from receipts + snapshot diffs | **NEW — primary for H1** |
| **Total agent spend** | workflow-reported tokens per subagent | promoted to first-class |
| Artifact bytes / hop | snapshot file sizes | same |
| Integrity / fidelity | flow-specific checklist scored against ground truth (see each flow) | **NEW** |
| Fixed-vs-marginal split | regress per-hop injected chars = a + b·(content chars); report a (scaffolding) and b separately | **NEW** |

**Two tiers.** Every flow ships in two variants sharing the same design, protocol, and metrics — only scale parameters differ:

- **LITE — ≤200k tokens per test.** n=1, smaller documents, data-only where the view isn't the thing being measured. Purpose: directional signal, protocol debugging, cheap repeatability. No variance claims may be published from LITE alone.
- **HEAVY — ≤2M tokens per test.** Full reps (n=3–5), full document sizes. Purpose: publishable numbers with variance bands.

Run LITE first as the smoke/directional pass; promote to HEAVY only flows where LITE shows signal worth paying for (H1/H2 are expected promotions regardless — they carry the headline claims). Per-flow parameters in the run-plan tables below.

**Prerequisites before running** — confirm shipped in 1.1.0: F1 fix (`patch-data` without decision adds no `unknown-agent` entry — spec/clan.md now says so), F11 fix (merge-patch "omit fields to keep them" documented in agent-guide/agent-help), F12 (`read decisions` alias or clean error). F2 (stale-view hint) matters for H1 — verify the hint no longer suggests `render` over a hand-authored view, or document it as a known cost.

---

## H1 — Revision loop: patch vs rewrite (the headline flow)

**Hypothesis:** when a pipeline *revises* a large document instead of regenerating it, CLAN's surgical patch commands cut output tokens by ≥50% vs ad-hoc file editing, at equal or better fidelity. This is the flow that directly answers flow-14's token result.

**Setup (uncounted):** one designer agent produces a seed document — the Brightline CRM recommendation as a ~45KB full-html `.clan` with ≥25 data-bound `{{key}}` fields, one SVG chart asset, and a populated schema. Export the same content for the ad-hoc arm as `report.html` + `data.json` + `decisions.log` (give ad-hoc its best possible starting structure).

**Pipeline:** 8 serial revision agents, each given exactly one small change (identical instructions both arms):

| Hop | Change | Touches |
|---|---|---|
| 1 | Update Zoho per-seat price (new figure supplied) | data field |
| 2 | Add one row to the risk table | data + view fragment |
| 3 | Change rollout phase-2 date | data field |
| 4 | Rewrite exec-summary paragraph (text supplied) | view prose |
| 5 | Add a "Data residency" subsection (~150 words supplied) | view fragment |
| 6 | Update the cost-comparison chart's Zoho bar | asset |
| 7 | Flip verdict from HubSpot to Zoho + one-line rationale | data + view |
| 8 | Record final sign-off decision | decision/chain |

**CLAN arm protocol (guided):** `clan read agent doc.clan --skip-guide` → make the change via the narrowest applicable command (`patch-data`, `patch-html`, `patch-asset`, `patch-decision`) → `clan validate`. **`pack-html` is forbidden** — the whole point is the patch path.
**Ad-hoc arm protocol (guided):** read whatever files needed → edit in place however the agent judges best (full rewrite or splice) → append a line to `decisions.log`.

**Measure:** output chars/hop (primary), injected chars/hop, total spend, artifact bytes growth, wall time.
**Fidelity check (scored by measure pass + one verifier agent):** all 8 changes present in final; the other ~24 data fields and untouched sections byte-identical to seed (snapshot diff); chain/log records all 8 actors.

**Win criteria:** CLAN total output chars ≤ 50% of ad-hoc; fidelity ≥ ad-hoc; zero unintended diffs. **Risk:** if ad-hoc agents splice surgically with their editor tools (frontier models can), the output gap narrows — then the result honestly becomes "CLAN guarantees what ad-hoc achieves on a good day," measured by the unintended-diff count.

---

## H2 — Long chain: find the crossover hop

**Hypothesis:** CLAN's TOON-distilled re-injection grows sub-linearly while ad-hoc's read-the-accumulating-narrative grows ~linearly per hop; the cumulative curves cross by hop ~6–10.

**Pipeline:** the CRM task stretched to **12 serial hops**, each role adding bounded structured findings (≤15 fields) + a ≤10-line handoff note: market-researcher → pricing-analyst → risk-analyst → gdpr-reviewer → integrations-assessor → customer-discovery → competitive-intel → finance-modeler → rollout-planner → procurement-reviewer → legal-reviewer → lead-partner (final report, full-html, last hop only).

**Arms:** CLAN guided (`--skip-guide`, patch-first protocol per H1) vs ad-hoc guided (the flow-14 convention set: named files, role prefixes, append to `findings.md` + `handoff.md`).
**Protocol constraint (both arms):** hops 1–11 are data/notes only — no HTML until hop 12. This isolates context growth from view authoring.

**Measure:** injected chars at every hop (both arms, from snapshots via measure pass); fit both cumulative curves; report crossover hop h\* with n=5 variance band; fixed-vs-marginal regression.
**Win criteria:** h\* ≤ 10 with the gap widening monotonically after it. **Risk:** if ad-hoc agents spontaneously distill (summarize-then-discard), curves may not cross — in that case measure what distillation *lost* (run the H7 audit probe against both finals) and report the trade.

---

## H3 — Cold resume: one file vs a directory

**Hypothesis:** a fresh agent with zero memory resumes a CLAN pipeline from a single file faster, cheaper, and more correctly than from an ad-hoc working directory.

**Setup:** run hops 1–2 of the H2 pipeline in both arms, then stop. Preserve only the artifact: CLAN = `doc.clan` alone; ad-hoc = the working directory exactly as hop 2 left it. No handover prompt, no transcript.
**Resume prompt (identical, deliberately minimal):** *"You are taking over an in-progress analysis. Everything known is in <artifact>. Work out where it stands and complete the next step only."*

**Measure:** chars read to orient; number of probe commands/file-opens before first productive write; correctness checklist — resumed the *right* next step (didn't redo hops 1–2), respected schema, recorded its decision, didn't disturb prior work.
**Win criteria:** CLAN orients from `clan read agent` (+`read chain`) alone, ≤3 reads, correct next step both reps; ad-hoc needs more probing or errs on state. **Risk:** flow-14 receipts showed `agent-help` discovery costs ~2–5k tokens for CLAN-naive agents — report orientation cost with and without that one-off.

---

## H4 — Cross-vendor relay: the artifact is the only carrier

**Hypothesis:** CLAN's self-describing container (embedded spec + `agent-help` teachable interface) survives a vendor/model boundary; ad-hoc conventions don't transfer to an agent that never saw the convention prompt.

**Pipeline:** 3 hops of the CRM task. Hop 2 runs on a **different vendor's model** (preferred: GPT/Gemini via their CLI; fallback if unavailable: the weakest available model class in this harness — still a real generalization test, label it honestly). Hops 1 and 3 on the session default.
**Key constraint:** hop 2 is **unguided** in both arms — it receives only *"Continue the work in <artifact>; advance it one step"*. The CLAN arm may discover the protocol from `clan agent-help`; the ad-hoc arm gets whatever the files themselves communicate.

**Measure:** five-layer scorecard (spec §22) at hop 3's final; hop-2 protocol competence (correct write path, namespace respect, decision recorded); discovery cost; errors/recoveries.
**Win criteria:** CLAN final retains L1–L4 with the foreign hop attributed in the chain; ad-hoc shows convention drift (wrong filenames, lost provenance, clobbered structure) at hop 2. **Risk:** a frontier foreign model may read ad-hoc files just fine — the differentiator to report is *attribution and structure*, not completion.

---

## H5 — Contested-key fan-out: engineered conflicts

**Hypothesis:** when parallel branches *must* write the same keys, CLAN detects 100% of conflicts with winner/loser provenance (`merge-report.yaml`); ad-hoc loses at least one branch's value silently. (Flow-05 found exactly one accidental contested key — the only silent-loss detection in research/14. Here we guarantee them.)

**Setup:** 4 specialist branches (financial, competitive, customer, regulatory), each **required by its context to write** the same four keys: `recommendation`, `budget_eur`, `top_risks`, `assumptions` — values guaranteed to disagree (each role's brief seeds a different budget ceiling and verdict lean). CLAN arm: `clan fork doc.clan --agents fin,comp,cust,reg --context-dir contexts/ --output-dir branches/` (per-branch contexts, exercising the F7 fix), branch writes via `patch-data --namespace`, then `clan merge branches/*.clan --output merged.clan`, synthesizer reads `clan read report` and adjudicates via `patch-data` + `patch-decision`. Ad-hoc arm: 4 agents in a shared dir with the same required keys in a shared `summary.json` (guided: "merge your findings into it"), then a synthesizer.

**Measure:** conflict recall — of the 4 engineered contested keys × 4 branches, how many disagreements are (a) detected and recorded, (b) silently resolved with no record, (c) lost outright; synthesizer input size (CLAN merged+report vs ad-hoc re-reading 4 specialists); adjudication provenance in the final.
**Win criteria:** CLAN records 4/4 contested keys with full provenance and the synthesizer's adjudication is in the chain; ad-hoc has ≥1 silent overwrite (last-writer wins in `summary.json`) or unrecorded reconciliation. **Risk:** none structural — this is CLAN's home turf; the honest number to also publish is merge+report token cost vs ad-hoc's.

---

## H6 — Context-window pressure: bounded injection vs full replay

**Hypothesis:** as accumulated state grows large, CLAN's distilled injection stays bounded and lossless-on-probe, while full-replay ad-hoc either blows the read budget or summarizes lossily.

**Pipeline:** 6 serial hops; each hop ingests a supplied ~25KB findings batch (synthetic vendor-evaluation tables — uniform, TOON-friendly) and must integrate it into shared state. By hop 6, total ingested content ≈150KB. Final hop answers a fixed 10-question probe about hop-1 and hop-2 facts *from its injected context / artifact only*.
**Arms:** CLAN (`patch-data` per hop; injection = TOON-distilled data + chain) vs ad-hoc guided (per-hop findings files + a running `state.md` the agents maintain — again, best-case ad-hoc: we *let* them distill).

**Measure:** injected chars at each hop; probe recall (10 questions, ground-truthed); where ad-hoc chose to drop/summarize, what was lost.
**Win criteria:** CLAN hop-6 injection ≤ 50% of ad-hoc full-replay (or, if ad-hoc distills, CLAN probe recall strictly higher at comparable injection size). **Risk:** TOON's ~40% advantage is dataset-dependent (research/12) — uniform tables are its best case; say so in the writeup and label this flow "favourable-content regime."

---

## H7 — Audit replay: answer from the artifact alone

**Hypothesis:** only the CLAN final can answer provenance questions after the fact. Zero extra pipeline cost — reuses H1 and H5 finals from both arms.

**Protocol:** a fresh auditor agent receives *only* the final artifact (no receipts, no transcripts) and 10 fixed questions, e.g.: Who changed the verdict, when, and on what rationale? What was the budget figure before adjudication, and whose number lost? Was a human involved anywhere? Which fields did hop 3 change? What did the regulatory branch contribute? Answers scored against ground-truth receipts by the measure pass.

**Measure:** correct answers /10 per arm per source artifact; auditor tokens spent.
**Win criteria:** CLAN ≥9/10 (chain + merge-report + lineage); ad-hoc ≤5/10. **Risk:** minimal; this is the regulated-workflow pitch (flow-14 §5: "only CLAN can *prove* a human was involved") generalized to all provenance.

---

## Run plan A — LITE suite (≤200k tokens per test)

Sandbox: `test-sandbox/happy/lite/<flow>/`. All flows n=1, both arms.

| Order | Flow | LITE deltas vs full design | ~Agents | Est. tokens* |
|---|---|---|---|---|
| 1 | H1-L revision loop | same 8 edits; seed doc generated once and reused by both arms (uncounted) | 18 | ~150k |
| 2 | H7-L audit replay | unchanged — audits H1-L + H5-L finals (4 artifacts) | 4 | ~100k |
| 3 | H3-L cold resume | unchanged (design is already cheap) | 6 | ~80k |
| 4 | H5-L contested fan-out | branches **data-only** (no HTML reports); same 4 engineered contested keys | 10 | ~120k |
| 5 | H6-L window pressure | ingest batches 10KB (≈60KB total by hop 6); same 10-question probe | 12 | ~130k |
| 6 | H2-L long chain | **10 hops**; hop 12→10 final is **data + designed-mode view** (no hand-authored full-html) | 20 | ~150k |
| 7 | H4-L cross-vendor | unchanged; single foreign-model rep | 6 | ~150k |

LITE total ≈ **0.9M**. Caveats to print in any LITE writeup: n=1 (directional only); H2-L's crossover hop h\* is provisional — if h\* lands at hop 8–10, the 10-hop window is too short to confirm the post-crossover trend and H2-H is mandatory; H5-L/H6-L's data-only regime removes view-authoring noise but also hides view-carry costs.

## Run plan B — HEAVY suite (≤2M tokens per test)

Sandbox: `test-sandbox/happy/heavy/<flow>/`. Full designs as specified in the flow sections.

| Order | Flow | Reps×arms | ~Agents | Est. tokens* | Cap check |
|---|---|---|---|---|---|
| 1 | H1-H revision loop | 5×2 (+1 verifier/rep) | 90 | ~0.8M | ✓ |
| 2 | H2-H long chain (12 hops) | 5×2 | 120 | ~2.0M | at cap |
| 3 | H5-H contested fan-out | 3×2 | 30 | ~0.9M | ✓ |
| 4 | H6-H window pressure | 3×2 | 36 | ~0.9M | ✓ |
| 5 | H3-H cold resume | 2×2 | 12 | ~0.3M | ✓ |
| 6 | H4-H cross-vendor | 2×2 | 12 | ~0.4M | ✓ |
| 7 | H7-H audit replay | 4 artifacts | 4 | ~0.1M | ✓ |

\* flow-14 actuals (~31.7k/agent full-report regime, much less for edit-only hops) scaled by hop type. HEAVY total ≈5.4M; no single test exceeds 2M (H2-H sits at the cap — if a rep overruns, drop to n=4 rather than cutting hops, the hop count is the experiment). **Indispensable HEAVY core: H1-H and H2-H at n=3 (~1.7M)** — H1 answers the patch-vs-rewrite gripe, H2 settles the crossover claim.

**LITE → HEAVY promotion rules:** promote a flow when (a) LITE shows a CLAN win or loss bigger than ±20% on its primary metric (worth confirming either way), or (b) LITE results are ambiguous *and* the flow carries a headline claim. Never publish a number from LITE; publish direction only.

## Deliverables

- `research/16-happy-path-lite-results.md` — LITE pass: directional results + promotion decisions per flow
- `research/17-happy-path-results.md` — HEAVY pass in flow-14's format (measured tables, variance bands, receipts-backed findings F13+, honest losses included)
- `test-sandbox/happy/lite/` and `test-sandbox/happy/heavy/` — all artifacts, snapshots, receipts; `measure.ps1` extended and re-run deterministic
- One chart: cumulative injected tokens vs hop (H2), with crossover marked — the single image that answers research/14's token headline
- A `.clan` summary of the results (dogfood, as `benchmark-results.clan` did for 14)

## What we are explicitly NOT testing here

Failure injection, malformed inputs, adversarial HTML, concurrent same-file writes, weak-model frontmatter traps, n-vendor interop matrices. Those are the robustness suite (next pass). This suite exists to measure the *ceiling*: when everything goes right, what does CLAN buy that disciplined ad-hoc cannot guarantee — and does the patch path finally show up in the token column where flow-14 never let it.
