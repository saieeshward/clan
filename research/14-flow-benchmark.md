# 14 — Multi-Agent Flow Benchmark: CLAN vs Ad-hoc, Serial vs Parallel, Guided vs Unguided, ± Human

**Date:** 2026-06-10 · **CLAN version:** 1.1.0 (fork/merge/render/teachable-interface) · **Runner:** 30 real subagents (no scripted outputs), one fixed task for every flow.

**Task (identical brief everywhere):** evaluate Salesforce vs HubSpot vs Zoho CRM for Brightline, a 40-person Dublin ad agency; produce a costed recommendation + rollout plan. Roles: market-researcher, risk-analyst, (customer-discovery in parallel flows), lead-partner. Knowledge-only (no web), so content quality is comparable across arms.

**The matrix (10 flows):**

| Flow | Topology | Medium | Guidance | Human |
|---|---|---|---|---|
| 01 | serial | CLAN | guided (exact protocol, `--skip-guide`) | – |
| 02 | serial | CLAN | unguided ("there's a `clan` CLI — figure it out") | – |
| 03 | serial | ad-hoc files | guided (named files, merge-by-hand) | – |
| 04 | serial | ad-hoc files | unguided | – |
| 05 | parallel 3+1 | CLAN fork/merge | guided | – |
| 06 | parallel 3+1 | CLAN fork/merge | unguided | – |
| 07 | parallel 3+1 | ad-hoc shared dir | guided | – |
| 08 | parallel 3+1 | ad-hoc shared dir | unguided | – |
| 09 | parallel 3+1 | CLAN | guided | live human edit in viewer |
| 10 | parallel 3+1 | ad-hoc | guided | live human edit of brief.md |

"Guided/unguided" = how much *protocol* the orchestrator put in the agent prompt; orchestration mechanics (sequencing, `fork`) were constant. Every agent filed a JSON receipt (commands, files read, errors + recoveries, friction) and snapshotted its hop, so context sizes below are **measured from artifacts**, not estimated by agents. Tokens ≈ chars/4.

---

## 1. Token usage (injected context per hop — what each agent had to read)

Serial flows (hop1 → hop2 → hop3 inputs, chars):

| Flow | hop1 | hop2 | hop3 | Σ chars | ≈ tokens |
|---|---|---|---|---|---|
| 01 clan-guided (`--skip-guide`) | ~2,400 | 9,812 | 23,122 | ~35,300 | **~8.8k** |
| 02 clan-unguided (full guide each hop) | ~9,700 | 14,797 | 26,101 | ~50,600 | **~12.7k** |
| 03 adhoc-guided | 341 | 6,774 | 18,031 | ~25,100 | **~6.3k** |
| 04 adhoc-unguided | 341 | 6,499 | 14,171 | ~21,000 | **~5.3k** |

Parallel flows (3 branch inputs + synthesizer input, chars):

| Flow | branch input ×3 | synth input | Σ chars | ≈ tokens |
|---|---|---|---|---|
| 05 clan-guided | ~2,515 each | 21,637 (merged, skip-guide) | ~29,200 | **~7.3k** |
| 06 clan-unguided | ~9,937 each (full guide) | ~36,300 | ~66,100 | **~16.5k** |
| 07 adhoc-guided | 341 each | 24,280 (7 files) | ~25,300 | **~6.3k** |
| 08 adhoc-unguided | 341 each | 17,507 | ~18,500 | **~4.6k** |

Actual model spend (workflow-measured): flows 1–8 = **886,158 subagent tokens / 28 agents** (~31.7k avg, dominated by reasoning + writing, not context); HITL synthesizers = 81,525 / 2.

**Honest headline: at this scale CLAN does NOT win on raw injected tokens.** Disciplined ad-hoc with frontier agents is ~15–40% leaner, because CLAN's context carries scaffolding (schema, decision chain, guide-or-digest, namespace blocks) the ad-hoc baseline simply doesn't have. The earlier 65–75% saving (research/05) was against a *raw-JSON accumulating* baseline; against smart agents inventing clean markdown conventions, the token race is roughly a tie. CLAN's case is what the tokens buy (sections 3–5) — and that the ad-hoc baseline's discipline is *emergent and unenforced*: nothing stops hop 5 or framework #2 from breaking it.

### Tokens wasted / saved (measured)
- **Guide skip:** full vs `--skip-guide` context differs by ~7,420 chars (~1.86k tok) per hop. Unguided serial CLAN paid it 3× (≈5.6k tok); guided paid ~0. The byte-stable-guide + digest mechanism works.
- **Discovery cost (unguided CLAN):** +3 to +8 extra commands per agent (`--help`, `agent-help`, probing) ≈ ~2–5k tokens/agent. **Zero failures followed discovery** — `agent-help` alone was sufficient in every case, including one synthesizer that discovered `clan merge` unprompted, used it correctly, and verified data survival afterwards.
- **Branch isolation:** parallel CLAN branch agents read ~2.5k chars each and **never read a sibling's transcript**; the merge was mechanical (no LLM tokens at all). Ad-hoc synthesizers re-read every specialist file in full.
- **Noise found:** `patch-data` without a decision auto-appends `unknown-agent / processed document` chain entries — one agent explicitly flagged the waste (finding F1).

## 2. Reliability — errors, recoveries, problems faced

30 agents, **30 completions, 0 unrecovered failures**. Errors encountered (all logged in receipts):

| # | What | Arm | Recovery |
|---|---|---|---|
| 1 | Harness Write-tool guard blocked creating report-draft.md | adhoc-guided | switched to `Add-Content` |
| 2 | Hop-1 leftover scratch files confused hop-2 | clan-guided | read-before-overwrite |
| 3 | **`clan patch-data` rejected UTF-8 BOM** (PowerShell 5.1 `Out-File` default!) with unhelpful "expected value at line 1 column 1" | clan-guided synth | rewrote file BOM-free |

- The namespace **guard error was never triggered** in unguided CLAN flows — agents discovered `--namespace` from `agent-help` before violating. Teaching worked *before* the error path was even needed.
- **Hints:** praised in 4 receipts ("the `next:` hints made the protocol easy to follow"); **misleading in 1 scenario** — after `patch-data` on a doc with a hand-authored full-html view, the "view is stale — `clan render` to refresh" hint would have *clobbered* the hand-built HTML. Two agents independently spotted this and used `patch-html` instead (one called the hint "misleading"). Real design bug → F2.
- Parallel ad-hoc shared-dir collisions: **none occurred** — but only because every agent independently chose role-prefixed filenames; one agent noted it had to *inspect a sibling's files mid-flight* to mirror an emergent pick-file convention. The safety was luck + convention, not construction.

## 3. The five handoff layers (spec §22) at the final artifact

| Layer | 01 | 02 | 05 | 06 | 09 | 03 | 04 | 07 | 08 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| L1 distilled state (structured, machine-readable) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ~ | ✓ |
| L2 transcript/scratchpad (handoff notes, state) | ✓ | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | ~ | ~ | ~ |
| L3 contracts (output schema) | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| L4 provenance (who/what/why, timestamped) | ✓* | ✓ | ✓ | ✓ | ✓ | ~ | ✗ | ~ | ✗ | ~ |
| L5 capability requirements | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Machine-validatable (`clan validate`) | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Human view in final | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ |

\* = with `unknown-agent` noise entries (F1). ~ = partial (one-line log entries, or prose-only). L5 was exercised by **no flow** — `requirements.yaml` needs first-class commands before agents will use it.

**CLAN final artifacts score 4/5 layers by construction; ad-hoc finals scored 1–2.5/5, and the two *unguided* ad-hoc flows lost the most** (flow-04: no structured data, no provenance, no renderable view; flow-08: pick-files only). Unguided CLAN lost *nothing* relative to guided CLAN — the format carried the discipline so the prompt didn't have to. That is the cleanest single result in the benchmark.

## 4. Concurrency: the merge experiment

- Flow-05/09 produced a **real contested key**: all three branches wrote `assumptions`; last-write silently preferred customer-discovery's version, dropping the risk-analyst's GDPR/PII scoping — and `merge-report.yaml` caught exactly that, with winner/loser provenance. The synthesizer (and in flow-09, with the human's input) adjudicated by *merging both framings* and recorded the adjudication in the chain. **This is the only arm in the whole benchmark where a silent information loss was detected and restored.** In ad-hoc flows, equivalent disagreements (cost ranges, seat counts) were reconciled from memory with no record of what was discarded.
- Flow-06 merged with 0 conflicts (branch keys disjoint). Convention note → F6: prose-y common keys like `assumptions` should default to `append` policy or carry role prefixes.

## 5. Human-in-the-loop (live, same directive in both: "CEO: year-1 ≤ €40,000")

| Property | 09 CLAN (viewer edit) | 10 ad-hoc (brief.md edit) |
|---|---|---|
| Agent saw the edit | ✓ — `# Human Edits` section in injected context | ✓ — but only because protocol said "read every file" |
| Cap honored in final | ✓ €35,800 (down-scoped seats) | ✓ €34,300–37,700 |
| Attributed to a human | ✓ `edited_by: human` + timestamp; adjudication rationale cites "the human (CEO) edit"; data field labeled "(human edit, binding)" | ✗ — indistinguishable from original brief text; no trace a human ever intervened |
| Edit record survives to final | **Partial** — semantically absorbed + cited in pinned decision, but raw `patches.yaml` dropped by full-html repack (→ F5) | ✗ nothing to survive |
| Viewer UX findings | Edit bridge persisted its own instrumentation into patch content; click-without-change saved no-op patches (→ F4) | n/a |

## 6. Findings (product backlog input)

1. **F1** `patch_data`/`pack` auto-append `unknown-agent / processed document` decisions — skip the auto-entry or label it `patch-data`, no empty rationale.
2. **F2** Stale-view hint suggests `clan render` even when the view is hand-authored full-html (render would clobber it). Track view origin (`rendered-by: clan render` vs agent) or hint `patch-html | render`.
3. **F3** `patch-data`/`pack` should tolerate or explicitly diagnose UTF-8 BOM — it is PowerShell 5.1's *default* `Out-File` encoding; the current error is unhelpful.
4. **F4** Viewer (v1.0 build): edit bridge saves `data-clan-edit-setup`/inline-style instrumentation into patch content, and saves no-op patches on blur-without-change. Strip instrumentation + diff-before-save.
5. **F5** Full-html repack drops `human/patches.yaml` (correct for stale DOM ids, but loses human-edit provenance). Auto-fold superseded human patches into the decision chain as `agent: human` entries.
6. **F6** Common prose keys (`assumptions`, `summary`) collide across branches by convention; ship default merge-policy guidance (`append` for known-prose keys) or role-prefix conventions in the Branch Mode injection block.
7. **F7** Fork copies the parent `context.md` verbatim, so branch agents see a task block (output mode full-html, design requirements) that conflicts with branch-mode rules — 4 receipts flagged the ambiguity. `fork` should support per-branch context rewriting (e.g. `--brief-per-agent`).
8. **F8** `requirements.yaml` (L5) is dead weight until `create`/`fork` can declare it and `read agent` surfaces unmet requirements loudly.

## 7. Verdict

- **Tokens:** roughly a tie at this scale; CLAN +15–40% injected scaffolding vs disciplined ad-hoc, offset partly by `--skip-guide`, branch isolation, and a zero-LLM merge. Sequential CLAN's TOON-distilled re-injection grows slower than ad-hoc's re-read-the-narrative pattern as pipelines lengthen — at 3 hops the curves haven't crossed yet.
- **Structure survival:** decisive CLAN win, and the *unguided* comparison is the proof: take away the prompt discipline and ad-hoc loses structured data, provenance, and the human view, while CLAN's finals are byte-for-byte as complete as guided ones. The format, not the prompt, carries the protocol.
- **Concurrency:** CLAN's fork/merge surfaced and recovered a real silent-information-loss; ad-hoc parallelism survived on emergent convention and luck.
- **Teachable interface:** unguided agents reached full protocol competence from `agent-help` + hints alone, at ~2–5k tokens discovery cost, with zero guard-rail violations. Hints helped 4×, misled 1× (F2).
- **HITL:** both pipelines *obeyed* the human; only CLAN can *prove* a human was involved. For any audited/regulated workflow, that attribution asymmetry is the whole game.

**One-line conclusion:** CLAN's price is a modest, bounded token overhead; its product is that correctness, provenance, conflict detection, and human attribution survive *even when nobody writes a careful prompt* — which is exactly the regime real multi-framework pipelines live in.

---
*Method notes: all context sizes measured from hop snapshots (`measure.ps1` → `metrics.json` in the benchmark sandbox); receipts written by the agents themselves; flows 1–8 ran concurrently (17.6 min wall-clock); model per agent identical (session default). One infra incident excluded from agent stats: background (non-workflow) agents cannot acquire shell permissions; the two HITL synthesizers were rerun under a workflow (34k tokens written off).*
