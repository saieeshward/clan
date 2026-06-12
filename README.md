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
  <b>30</b> real agents benchmarked &nbsp;·&nbsp; <b>0</b> unrecovered failures &nbsp;·&nbsp; <b>42%</b> fewer revision-loop output tokens &nbsp;·&nbsp; <b>44%</b> smaller synthesis injection &nbsp;·&nbsp; <b>0</b> LLM tokens per merge &nbsp;·&nbsp; <b>100%</b> of mutations attributed &nbsp;·&nbsp; <b>&lt;200ms</b> every CLI command &nbsp;·&nbsp; <b>165</b> tests + <b>26</b>-test conformance suite green
</p>

---

## The Problem

Multi-agent systems have no standard for what passes between agents. Context gets lost, decisions are untracked, parallel branches silently overwrite each other, and humans have no readable window into what happened — or any proof they were ever involved. Every team re-solves this with bespoke context-assembly code that the *next* framework in the pipeline doesn't understand.

CLAN replaces that with a single, self-describing file.

---

## Quickstart

```bash
# Install (from source; binaries on the Releases page)
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

We ran **30 real agents** (no scripted outputs) through **11 flows** on one fixed task: serial vs parallel × CLAN vs ad-hoc files × guided vs unguided prompts × ± a live human edit. All context sizes were measured from artifacts, not estimated. Full write-up: [`research/14-flow-benchmark.md`](research/14-flow-benchmark.md).

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

### Measured claims (scorecard, latest run)

| Claim | Measured | Threshold | Status |
|---|---|---|:---:|
| Revision loops: CLAN patch path authors fewer output chars than ad-hoc full-rewrites | **0.576× (42% fewer)** | ≤ 0.65 | ✅ PASS |
| Synthesis hop: CLAN's merged injection beats ad-hoc re-reading every input | **0.557× (44% less)** | < 1.0 | ✅ PASS |
| TOON encoding saves vs minified JSON on tabular data | **≥ 30%** | ≥ 30% | ✅ PASS |
| Fidelity: every requested edit present, untouched fields intact | **1.0** | = 1.0 | ✅ PASS |
| Provenance: every mutating hop attributed, end-to-end | **1.125** | ≥ 1.0 | ✅ PASS |
| Reliability: agents recover from every CLI error without orchestrator help | **0 unrecovered** | = 0 | ✅ PASS |
| Agent guide is byte-identical across all files and hops (prompt-cache friendly) | **1 unique hash** | 1 | ✅ PASS |
| Fixed injection scaffolding is bounded | **≤ 3,000 chars** | ≤ 3,000 | ✅ PASS |

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
- Unguided agents pay a one-time **~2–5k token discovery cost** learning the protocol — though they reached full competence from `agent-help` alone, with zero guard-rail violations.

### Verification status

| Suite | Result |
|---|---|
| Rust unit + integration tests | **165/165 pass** |
| Black-box CLI conformance harness | **26/26 pass, 0 hard failures** |
| Scorecard claims | **14 PASS · 0 FAIL** |
| Benchmark reliability | **30/30 agent completions, 0 unrecovered failures** |

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

**v1.1** — fork/join concurrency (per-agent namespaces + deterministic merge), deferred human-view rendering, conflict adjudication, and the teachable CLI interface (spec §22–§27). Verified by 165 Rust tests + a 26-test black-box conformance suite in CI.

## Maintainers

Maintained by [Sai Eeshwar](https://github.com/saieeshward) and [Shreyansh Soni](https://github.com/batunii).

## License

[MPL-2.0](LICENSE) — the spec is open; implementations in any language are welcome.
