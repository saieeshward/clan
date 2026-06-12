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

## What the Benchmark Says — Including Where CLAN Loses

We ran 30 real agents (no scripted outputs) through 11 flows on one fixed task: serial vs parallel × CLAN vs ad-hoc files × guided vs unguided prompts × ± a live human edit. All context sizes measured from artifacts. Full write-up: [`research/14-flow-benchmark.md`](research/14-flow-benchmark.md).

**Where CLAN wins:**

| Result | Measured |
|---|---|
| Handoff completeness (structured data, contracts, provenance, human view) | CLAN finals: **4/5 layers by construction**; ad-hoc finals: **1–2.5/5** |
| Take away the careful prompt | Unguided CLAN lost **nothing** vs guided; unguided ad-hoc lost structured data, provenance, and the human view |
| Parallel conflict safety | CLAN's merge **detected and recovered a real silent data loss** (a risk analyst's GDPR scoping dropped by last-write) — the only arm in the benchmark to do so |
| Merge cost | **Zero LLM tokens** — the merge is mechanical |
| Revision loops | CLAN patch path authored **0.576×** the output chars of ad-hoc full-rewrites |
| Synthesis hop | CLAN's merged injection was **0.557×** ad-hoc's re-read-everything |
| Human-in-the-loop | Both pipelines obeyed the human; **only CLAN can prove it** (`edited_by: human`, timestamped, cited in the chain) |
| Reliability | 30/30 agent completions, 0 unrecovered failures |

**Where CLAN loses (we measured it, so we'll say it):**

- **Raw injected tokens at small scale: CLAN does not win.** Disciplined ad-hoc with frontier agents is ~15–40% leaner, because CLAN's context carries scaffolding (schema, decision chain, guide-or-digest) the baseline simply doesn't have. At 3 hops, the growth curves haven't crossed yet.
- Unguided agents pay a one-time **~2–5k token discovery cost** learning the protocol.
- Several claims are still unmeasured (prompt-cache hit rates, the hop count where the token curves cross, cross-vendor survival) — tracked in the [scorecard](test-sandbox/pipeline/results/).

Every finding the benchmark surfaced (16 of them — silent failures, BOM handling, misleading hints, …) has been fixed and locked in by a **26/26 black-box conformance suite** that runs in CI alongside 165 Rust tests.

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

Chat platforms extract text from `.docx`/`.pdf` server-side before the model ever sees it. Nothing does that for `.clan` yet — so the `clan` CLI is the extractor: `clan read agent file.clan` unpacks the container, compresses the context, and emits one token-optimized prompt. A framework with the SDK built in can hide the CLI entirely; until then, the CLI is the universal adapter any agent can drive.

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
