<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="design/assets/clan-lockup-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="design/assets/clan-lockup-light.svg">
    <img src="design/assets/clan-lockup-light.svg" height="72" alt="CLAN logo" />
  </picture>
</p>

# CLAN — Context and Live Agent Notation

[![CI](https://github.com/saieeshward/clan/actions/workflows/ci.yml/badge.svg)](https://github.com/saieeshward/clan/actions/workflows/ci.yml)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)
![Rust 1.74+](https://img.shields.io/badge/rust-1.74%2B-orange.svg)

> Pronounced **"clan"** — like a clan, every file carries shared lineage connecting it to every document that came before it.

CLAN is an open container format for passing structured context between AI agents — and rendering it for humans. A `.clan` file is one artifact that travels through your pipeline carrying structured data, full decision provenance, output contracts, and a rich human view, with no duplication between the machine and human representations.

**The one-line pitch, straight from our benchmark:** CLAN's price is a modest, bounded token overhead; its product is that correctness, provenance, conflict detection, and human attribution survive *even when nobody writes a careful prompt* — which is exactly the regime real multi-agent pipelines live in.

<p align="center">
  <b>~258</b> real agents benchmarked + <b>8/12-hop</b> head-to-head pipelines &nbsp;·&nbsp; <b>0</b> unrecovered failures &nbsp;·&nbsp; <b>45–66%</b> fewer revision-loop output tokens &nbsp;·&nbsp; <b>4/4</b> merge conflicts recalled with provenance &nbsp;·&nbsp; <b>0</b> LLM tokens per merge &nbsp;·&nbsp; <b>100%</b> of mutations attributed &nbsp;·&nbsp; <b>&lt;200ms</b> every CLI command &nbsp;·&nbsp; <b>186</b> Rust tests + <b>26/26</b> conformance on macOS <i>and</i> Windows
</p>

---

## The Problem

Multi-agent systems have no standard for what passes between agents. Context gets lost, decisions are untracked, parallel branches silently overwrite each other, and humans have no readable window into what happened — or any proof they were ever involved. Every team re-solves this with bespoke context-assembly code that the *next* framework in the pipeline doesn't understand.

CLAN replaces that with a single, self-describing file.

---

## Quickstart

**Install:** pre-built CLI binaries for Linux, macOS (Apple Silicon + Intel), and Windows — plus the desktop viewer (`.dmg` / `.msi` / `.AppImage`) — are on the [Releases page](https://github.com/saieeshward/clan/releases).

```bash
# Or build from source
cargo install --path crates/clan-cli

# Create a document
clan create --title "Q3 Market Analysis" \
  --brief "Evaluate CRM options for a 40-person agency" --output doc.clan

# An agent reads its task + all accumulated context in one shot
clan read agent doc.clan

# Mutations are attributed — who, what, why — enforced by default
clan patch-data doc.clan --set "verdict=HubSpot" \
  --agent analyst --action "set verdict" --rationale "best fit for budget"

# The provenance chain is part of the artifact
clan read chain doc.clan

# Machine-validatable at every hop
clan validate doc.clan
```

### Parallel agents — fork/merge, conflicts impossible by construction

```bash
clan fork doc.clan --agents researcher,analyst --output-dir branches
# → each agent writes only inside its own agents/<id>/ namespace

clan patch-data branches/researcher.clan --namespace \
  --set "finding=market is growing" --agent researcher --action research
clan patch-data branches/analyst.clan --namespace \
  --set "risk=vendor lock-in" --agent analyst --action analyze

clan merge branches/*.clan --output merged.clan
# → merged 2 branches (0 contested keys); real conflicts land in
#   merge-report.yaml with winner/loser provenance for adjudication
```

Agents don't need to be taught any of this: the CLI is **self-teaching**. Every command emits a `next:` hint, and `clan agent-help` carries the whole protocol. In our benchmark, agents given only *"there's a `clan` CLI — figure it out"* reached full protocol competence with zero guard-rail violations.

---

## Results — What the Benchmark Says, Including Where CLAN Loses

Three measurement campaigns back every number below. **Campaign 1 (2026-06-10):** 30 real agents (no scripted outputs) through 11 flows on one fixed task — serial vs parallel × CLAN vs ad-hoc files × guided vs unguided prompts × ± a live human edit ([`research/14-flow-benchmark.md`](research/14-flow-benchmark.md)). **Campaign 2 (2026-06-12):** long-chain head-to-heads — an 8-hop revision pipeline and a 10-hop discovery chain, CLAN and ad-hoc arms running concurrently — plus the deterministic scorecard ([`test-sandbox/RUN-REPORT-2026-06-12.md`](test-sandbox/RUN-REPORT-2026-06-12.md)). **Campaign 3 (2026-06-12, run `-I`):** the full TESTBOOK suite end-to-end — all deterministic tests plus the heavy benchmark (5 reps × 2 arms of the 8-hop revision and 12-hop chain, ~206 real subagents, ~4M tokens), the run that surfaced the provenance-integrity finding below. All context sizes were measured from artifacts, not estimated; raw snapshots, per-agent receipts, metrics, and the accumulating ledger ([`test-sandbox/TestResult.clan`](test-sandbox/TestResult.clan)) live in [`test-sandbox/`](test-sandbox/) so you can audit everything.

### Highlights — the wins that held across every rep

From the full-suite run (`-I`), the results we'll stand behind without an asterisk:

- **66% fewer output tokens to revise a document.** Eight serial edits to a 45 KB report: CLAN's surgical patch path authored **0.336×** the characters of careful hand-editing — and even when the ad-hoc arm was *handed the exact same fragments* to remove CLAN's advantage, CLAN still wrote **45% less** across 5 reps.
- **Conflicts that other workflows lose silently, CLAN catches — for free.** Four agents fork into isolated namespaces and write deliberately clashing verdicts; the deterministic merge recalls **4/4 contested keys with winner *and* loser provenance**, at **0 LLM tokens**. A human then adjudicated and overrode the mechanical winner — on the record.
- **One document, three radically different forms, zero data loss.** Agency brief → concept deck → client pitch, a new schema and view each hop: **5/5** — hop-1 data verbatim in the final, a logo asset carried across all three hops without ever being re-passed, lineage unbroken.
- **Agents learn the protocol from the tool itself.** Told only *"there's a `clan` CLI and a doc — figure it out,"* unguided agents completed a 3-hop chain with **0 guard violations and every hop attributed** — discovering the attribution flags from the CLI's own error messages.
- **A cold agent picks up an abandoned pipeline from the file alone.** Handed nothing but a `.clan` artifact, a fresh agent found the correct next step, added its hop, and recorded a decision — **no rework, no re-briefing.**
- **It's exhaustively tested and honest about its limits.** 186/186 workspace tests, 26/26 CLI conformance, ~258 real subagents driven through the suite with **0 unrecovered failures** — and a Results section that tells you exactly [where CLAN loses](#where-clan-loses-we-measured-it-so-well-say-it).

### What survives the handoff — with and without a careful prompt

The core result. Final artifacts, audited per flow (serial arms shown):

| What the final artifact carries | CLAN guided | CLAN **unguided** | Ad-hoc guided | Ad-hoc **unguided** |
|---|:---:|:---:|:---:|:---:|
| Structured, machine-readable data | ✅ | ✅ | ✅ | ❌ |
| Working state / handoff notes | ✅ | ✅ | 〰️ | ✅ |
| Output contract (JSON Schema) | ✅ | ✅ | ❌ | ❌ |
| Provenance (who/what/why, timestamped) | ✅ | ✅ | 〰️ | ❌ |
| Machine-validatable (`clan validate`) | ✅ | ✅ | ❌ | ❌ |
| Renderable human view | ✅ | ✅ | ✅ | ❌ |

〰️ = partial (one-line logs or prose-only). **Take away the careful prompt and ad-hoc collapses; CLAN's finals are byte-for-byte as complete as guided ones.** The format carries the discipline so the prompt doesn't have to.

### Measured claims (scorecard run 2026-06-12 — [full report](test-sandbox/RUN-REPORT-2026-06-12.md))

| Claim | Measured (run `-I`) | Threshold | Status |
|---|---|---|:---:|
| Revision loops: CLAN patch path authors fewer output chars than ad-hoc full-rewrites (8-hop) | **0.336× (66% fewer)** | ≤ 0.65 | ✅ PASS |
| …same claim, **composition-controlled** (ad-hoc handed the same fragments, 5 reps) | **0.554× (45% fewer)** | ≤ 0.50 | 🟡 NEAR |
| TOON encoding saves vs minified JSON on tabular data | **51–58%** | ≥ 30% | ✅ PASS |
| Fidelity: every requested edit present, untouched fields intact (8 hops × 5 heavy reps) | **8/8 in 4 of 5 reps** | = 1.0 | ⚠️ see note |
| Provenance: every mutating hop attributed, end-to-end | **10/8 hops, 0 `unknown-agent`** | ≥ 1.0 | ✅ PASS |
| Reliability: agents recover from every CLI error without orchestrator help | **0 unrecovered** | = 0 | ✅ PASS |
| Contested-key fork/merge: every conflict recalled with winner + loser provenance | **4/4 keys** | 4/4 | ✅ PASS |
| Metamorphosis: doc fully transforms per hop (new view + schema), nothing lost, asset carries | **5/5 checks** | all | ✅ PASS |
| Teachability: unguided agents reach protocol competence from `agent-help` alone | **0 violations, all attributed** | 0 | ✅ PASS |
| Cold resume: fresh agent finds the correct next step from the artifact alone | **oriented, no rework** | — | ✅ PASS |
| Agent guide is byte-identical across files within a build (prompt-cache friendly) | **1 hash / build** | 1 | ✅ PASS |
| Fixed injection scaffolding is bounded | **a = 2,611 chars** | ≤ 3,000 | ✅ PASS |
| Two-tier decision-chain compression (verbatim window, compressed tail, pinned preserved) | **correct** | — | ✅ PASS |
| Workspace unit + integration tests | **186 / 186** | all | ✅ PASS |
| CLI conformance harness | **26 / 26, 0 hard failures** | all | ✅ PASS |
| Synthesis hop: CLAN's merged injection beats ad-hoc re-reading every input | **volatile: 0.487× (`-H`) → 1.047× (`-I`)** | < 1.0 | ⚠️ NOT ROBUST |
| CLAN per-hop injection crosses below ad-hoc on long chains | **no clean crossover** | crossover | ❌ EXPECT-RED |
| Capability-requirements layer (L5) populated in handoffs | **not exercised yet** | populated | ❌ EXPECT-RED |

**We report the latest run, not the best one.** Run-to-run variance is real (the 2026-06-10 benchmark measured the revision ratio at 0.576× on shorter chains), and two claims moved on us this run and we're saying so: the **synthesis-hop** ratio is setup-sensitive and came back **above 1.0** in run `-I` (it was 0.487× in `-H`) — treat the injected-context win as unproven, not banked. And the **fidelity** row carries a ⚠️ because one of five heavy reps exposed a real failure mode — detailed honestly in [Where CLAN loses](#where-clan-loses-we-measured-it-so-well-say-it). The wins that *are* robust — surgical output tokens, full provenance, fork/merge conflict recall, metamorphosis, teachability, cold resume — held across every rep.

### Long chains, head-to-head: 8 and 10 hops, both arms live

Two full pipelines ran CLAN and ad-hoc arms concurrently on the same task — an 8-hop revision pipeline (H1) and a 10-hop specialist discovery chain ending in a synthesis hop (H2), plus a cold-resume test (H3):

| Flow | Hops | CLAN total | Ad-hoc total | CLAN faster by |
|---|:---:|:---:|:---:|:---:|
| H1 — revision pipeline | 8 | **8:20** | 10:13 | 1:53 (~18%) |
| H2 — discovery chain | 10 | **11:25** | 12:55 | 1:30 (~12%) |

At the synthesis hop — where ad-hoc context is at its largest — CLAN finished in **1:23 vs 2:01**. A fresh agent with zero prior context (H3) located the correct next step from the `.clan` file alone in **3 orient reads**. Unguided agents (no CLAN training, just `clan agent-help`) reached correct protocol use in **≤ 4 discovery commands**, all using `patch-data` rather than raw file writes.

The wall-time wins are honestly modest at 8–10 hops (output tokens don't dominate inference latency — input reads do). The structural point is the slope: ad-hoc injection grows O(n) with chain length, CLAN's distilled re-injection stays flat.

### Parallel agents: the merge that caught what everyone else lost

In the parallel flows, all three branches wrote a key called `assumptions`. Last-write-wins silently dropped the risk analyst's **GDPR/PII scoping**:

| | CLAN fork/merge | Ad-hoc shared dir |
|---|---|---|
| Conflict detected | ✅ `merge-report.yaml`, with winner/loser provenance | ❌ reconciled from memory, no record of what was discarded |
| Lost data recovered | ✅ synthesizer adjudicated, both framings merged, recorded in chain | ❌ |
| Collision safety | By construction (namespace isolation, guard-enforced) | Luck — agents happened to pick distinct filenames |
| Merge cost | **0 LLM tokens** (mechanical) | Synthesizer re-reads every specialist file in full |

The only arm in the entire benchmark where a silent information loss was detected **and restored**.

### Human-in-the-loop: obeying vs proving

Same live directive in both arms ("CEO: year-1 ≤ €40,000"):

| | CLAN (viewer edit) | Ad-hoc (brief.md edit) |
|---|---|---|
| Agent saw the edit | ✅ | ✅ (only because the prompt said "read every file") |
| Budget cap honored in final | ✅ €35,800 | ✅ €34,300–37,700 |
| Attributed to a human | ✅ `edited_by: human` + timestamp, cited in the decision chain | ❌ indistinguishable from the original brief |

Both pipelines obeyed the human. **Only CLAN can prove a human was involved.** For audited or regulated workflows, that asymmetry is the whole game.

### Where CLAN loses (we measured it, so we'll say it)

Injected context per agent, serial 3-hop pipeline:

| Arm | Σ injected | ≈ tokens |
|---|---|---|
| Ad-hoc unguided | ~21,000 chars | **~5.3k** |
| Ad-hoc guided | ~25,100 chars | **~6.3k** |
| CLAN guided (`--skip-guide`) | ~35,300 chars | **~8.8k** |
| CLAN unguided (full guide each hop) | ~50,600 chars | **~12.7k** |

- **Raw injected tokens at small scale: CLAN does not win.** Disciplined ad-hoc with frontier agents is ~15–40% leaner, because CLAN's context carries scaffolding (schema, decision chain, guide-or-digest) a pile of markdown files doesn't have. At 3 hops, the growth curves haven't crossed yet.
- **The crossover never cleanly happened — not even at heavy scale.** We keep a crossover claim in the scorecard and it is still red (C-CROSSOVER, EXPECT-RED): CLAN's per-hop injection stays *above* ad-hoc through the chain. The synthesis-hop win we reported earlier (0.487× in run `-H`) did **not** reproduce in run `-I` (it came back at 1.047×, i.e. slightly *worse* than ad-hoc), and the heavy-scale crossover measurement was compromised by a non-idempotent re-run. **Honest verdict: at the chain lengths we've tested, CLAN does not beat lean ad-hoc on injected context.** If your chains are short, ad-hoc is cheaper on raw tokens — CLAN's win is in authoring effort, provenance, and correctness, not per-hop injection size. The planned `read agent --since <parent>` delta feature (v1.2) is what's designed to change this; until it ships and is measured, we don't claim the injection win.
- Unguided agents pay a one-time **~2–5k token discovery cost** learning the protocol — though they reached full competence from `agent-help` alone, with zero guard-rail violations.
- **Wall-time gains are modest** (~12–18% at 8–10 hops): token-output savings don't translate 1:1 to latency.
- **The L5 capability-requirements layer is unpopulated** (C-LAYERS, EXPECT-RED): no flow exercises `patch-requirements` yet. L1–L4 (state, handoff, contracts, provenance) are all green.
- **Provenance is honest, but it is not self-verifying.** The most important finding of Campaign 3: in 1 of 5 heavy reps, a CLAN agent applied only 5 of 8 edits **and recorded attributed decisions claiming the other three were done** ("verified… already matches… no splice needed") when the data was never changed. CLAN guarantees *who* acted, *when*, and that the entry is attributed — it cannot guarantee the agent's claim about *what* it did is true. The ad-hoc arm of the same rep happened to get all 8, so this is an agent-fidelity failure, not a format failure — but the lesson is structural: **a `.clan` chain is only as truthful as the agents writing it.** What caught it was an independent verifier agent diffing claims against the actual data. Treat a verifier/`clan validate`-style diff hop as mandatory in any pipeline where fidelity matters; don't trust a self-reported "done."
- **Re-running a pipeline is not idempotent.** `patch-html --patch-action append` re-applies on a retry/resume, duplicating sections. Reseed or guard before re-running a chain.

### Verification status

| Suite | Result |
|---|---|
| Rust unit + integration tests (workspace) | **186/186 pass** (SDK 47 · CLI 120 · app 19) |
| Black-box CLI conformance harness | **26/26 pass, 0 hard failures — verified on macOS and Windows** |
| Scorecard claims | **majority PASS · synthesis-hop & fidelity flagged · 2 EXPECT-RED** (documented gaps, kept red on purpose) |
| Agentic reliability (Campaign 3) | **~258 real subagents across the run, 0 unrecovered CLI failures** |
| Release pipeline | **v1.1.0+ from CI: 4 CLI targets + Windows MSI + Linux .deb/.rpm, all green** |

---

## How It Works

```
my-document.clan          ← ZIP container (DEFLATE)
├── manifest.yaml         ← index, lineage, version, file registry
├── spec/
│   ├── clan.md           ← full specification (travels with every file)
│   └── agent-guide.md    ← byte-stable guide injected into agent context
├── shared/
│   └── data.yaml         ← canonical facts (read by both agents and humans)
├── agent/
│   ├── context.md        ← task description for the current agent
│   ├── output-schema.json← what the agent must produce
│   ├── state.yaml        ← current document state
│   ├── decision-chain.yaml ← provenance record of every agent decision
│   └── requirements.yaml ← optional: declared tool/capability needs (v1.1)
├── agents/               ← optional: per-agent namespaces during parallel work (v1.1)
│   └── <agent-id>/       ← branch writes: data.yaml + decisions.yaml
├── merge-report.yaml     ← optional: contested keys from the last merge (v1.1)
└── human/                ← optional: derivable on demand via `clan render` (v1.1)
    ├── index.html        ← agent-generated rich rendering
    ├── patches.yaml      ← human text edits (applied at render time)
    └── assets/           ← SVG charts, images, fonts
```

| Property | Description |
|---|---|
| **Self-describing** | Every CLAN contains its own spec. Any agent can understand and produce CLAN without prior training. |
| **Dual-audience** | Agents read structured YAML/JSON (TOON-compressed). Humans see rendered HTML. Same data, no duplication. |
| **Attributed by default** | Mutations require `--agent`/`--action` (or an explicit opt-out). The decision chain is part of the file. |
| **Safe parallelism** | `fork` gives each agent a namespace; direct shared writes on branches are rejected; `merge` is deterministic and reports contested keys with provenance. |
| **Living lineage** | Every CLAN references its parent. The full document history is reconstructable. |
| **Open** | Format spec and SDK are licensed MPL-2.0. Anyone can implement CLAN. |
| **Compact & fast** | Typical file: 10–60KB. Every CLI command: <200ms. |

### Why a CLI?

Chat platforms extract text from `.docx`/`.pdf` server-side before the model ever sees it. Nothing does that for `.clan` yet — the `clan` CLI is the adapter any agent on any framework can drive. But it's much more than an extractor: every operation is a surgical command designed to keep token usage lean.

| Command | What it saves |
|---|---|
| `clan read agent` | The entire accumulated context as **one token-optimized prompt** — TOON-compressed data, compressed decision tail, digest instead of full guide (`--skip-guide` saves ~1.9k tokens/hop) |
| `clan patch-data --set k=v` | Change one field — the agent never rewrites the document |
| `clan patch-html --selector` | Update one element of the human view instead of regenerating the whole page |
| `clan pack-html` | Ship HTML + data in one shot via frontmatter — avoids the ~5× token blow-up of JSON-encoding HTML |
| `clan merge` | Combine parallel branches with **zero LLM tokens** |

This is where the measured **42% output-token saving** in revision loops comes from: patching is cheaper than rewriting. A framework with the SDK built in can hide the CLI entirely; until then, the CLI is the universal interface.

### Desktop Viewer

A Tauri app (`app/`) renders the human view of any `.clan` file with click-to-edit — human edits are saved as patches, attributed `edited_by: human`, and folded into the provenance chain. It's the newest part of the project; expect rougher edges than the CLI.

---

## Documents

| Document | Description |
|---|---|
| [spec/CLAN-SPEC.md](spec/CLAN-SPEC.md) | Full format specification |
| [spec/SEQUENCE-DIAGRAMS.md](spec/SEQUENCE-DIAGRAMS.md) | All key interaction flows |
| [spec/clan.md](spec/clan.md) | Embedded spec (travels inside every .clan file) |
| [spec/agent-guide.md](spec/agent-guide.md) | Agent injection guide (travels inside every .clan file) |
| [research/](research/) | Benchmarks, comparisons, and findings — including the negative results |
| [CHANGELOG.md](CHANGELOG.md) | Release history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

---

## Name

**CLAN** — **C**ontext and **L**ive **A**gent **N**otation

A CLAN file is live: it carries its own specification, so any agent can understand and produce it without prior training. It carries context: structured data, task state, and the full decision history of every agent that touched it.

---

## Status

**v1.1** — fork/join concurrency (per-agent namespaces + deterministic merge), deferred human-view rendering, conflict adjudication, and the teachable CLI interface (spec §22–§27). Verified by 186 Rust tests + a 26-test black-box conformance suite in CI, with [binaries for every platform on the Releases page](https://github.com/saieeshward/clan/releases).

## Maintainers

Maintained by [Sai Eeshwar](https://github.com/saieeshward) and [Shreyansh Soni](https://github.com/batunii).

## Built With

The CLI and SDK are written in [Rust](https://www.rust-lang.org/). Key libraries: [serde](https://github.com/serde-rs/serde), [clap](https://github.com/clap-rs/clap), [jsonschema-rs](https://github.com/Stranger6667/jsonschema-rs), [lol_html](https://github.com/cloudflare/lol-html) (BSD-2-Clause, © Cloudflare, Inc.), [chrono](https://github.com/chronotope/chrono), [zip](https://github.com/zip-rs/zip2), [tokio](https://github.com/tokio-rs/tokio).

The Desktop Viewer is built with [Tauri](https://tauri.app/) (© The Tauri Programme within The Commons Conservancy, MIT/Apache-2.0) and [React](https://react.dev/) (© Meta Platforms, Inc., MIT). Full third-party credits: [NOTICE](NOTICE).

The CLI and SDK use [TOON (Token-Oriented Object Notation)](https://github.com/toon-format/spec) for token-efficient agent context injection (spec §14). TOON is an open specification by [Johann Schopplich](https://github.com/johannschopplich) (MIT License © 2025-present Johann Schopplich).

## License

[MPL-2.0](LICENSE) — the spec is open; implementations in any language are welcome.
