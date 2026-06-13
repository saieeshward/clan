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

> Not a framework. Not a runtime. A file format — where the hard parts are solved by design.

---

## The problem with agent handoffs isn't the agents

When one agent finishes and passes work to the next, five things need to survive that transition:

1. **What's already been decided** — not the original brief, but every choice made since, compressed so the next agent isn't drowning before it starts
2. **What the next agent must produce** — a real schema, validated at handoff time, not "hopefully a JSON that looks like the last one"
3. **Who decided what, and why** — in the artifact itself, not in a log somewhere you'll never find
4. **A way for a human to see and act on the work** — and when they do, that edit needs to land back in the artifact, on the record
5. **Safety when agents work in parallel** — because when two agents write the same field, one of them loses. Silently.

Most pipelines handle one or two of these — patched over with careful prompting, bespoke glue code, and hope. It works until it doesn't.

The reason it's hard isn't the agents or the models or the frameworks. It's that there's no standard for what the artifact between agents should look like. Every team invents their own. The next tool in the chain can't read it. State evaporates at every boundary.

**CLAN is that artifact.** A file format — open, model-agnostic, framework-agnostic, environment-agnostic — where all five of those things are solved by design, not by runtime enforcement.

---

## Design, not runtime

This is the part worth understanding.

CLAN doesn't sit between your agents. It doesn't hook into your framework. It doesn't route calls or manage state at runtime. It has no opinions about which model you use, which orchestrator you run, or which environment you're in.

It's a file format. Like JSON. Like PDF. The properties aren't delivered by a CLAN process — they're structural.

**Provenance exists because the format requires attribution on every write.** Not because CLAN is watching — because a mutation without `--agent` and `--action` is rejected at the CLI level, the same way a JSON parser rejects malformed JSON. The decision chain is part of the file's structure, not a side log.

**Parallel safety exists because the namespace design makes collisions impossible by construction.** Forked agents write into `agents/<id>/` — a different path by definition. They cannot touch each other's keys. The merge is deterministic and purely mechanical: zero LLM tokens, zero runtime coordination. Contested keys surface in `merge-report.yaml` with both sides documented.

**Human readability exists because the format carries the HTML.** The human view isn't generated on request by a CLAN service — it's inside the ZIP. Open the file with anything. The data and the view are the same artifact; they can't drift.

**The format is self-describing because every `.clan` file contains its own spec.** An agent that has never heard of CLAN can open the file, read the embedded guide, and know exactly what to do. No training required. No integration required.

None of this depends on CLAN being "in the loop." The file does the work.

---

## What everything else leaves out

The tools people use today each solve a real problem — just not this one.

**Orchestration frameworks** give you a runtime for coordinating agents. State lives in memory, or in a database your framework manages. That works well inside one pipeline, on one team, in one environment. The moment you hand the work to a different framework, a different team, or a different model provider, the state doesn't travel. There's no artifact. You're back to re-briefing from scratch or writing glue code the next tool can't read. And if something went wrong three hops ago, there's no record you can inspect — because the record was in memory.

**Agent communication protocols** solve the transport layer. They define how agents send messages to each other at runtime. They don't define what the artifact looks like after the conversation ends. There's no provenance baked into the message. There's no human view. There's no output contract. Message persistence isn't even guaranteed. When the session ends, the work is a summary in someone's context window.

**Token optimization tools** make your inputs cheaper — by compressing what reaches the model. That's a real and useful problem. But it's a different layer entirely. A cheaper input is not a richer artifact. It doesn't make the handoff safer, the provenance traceable, or the human edits attributable. It just costs less to feed the same incomplete context.

None of these tools are wrong. They just don't produce an artifact that survives the boundary crossing. CLAN is what you put at the boundary.

### How CLAN sits relative to the landscape

| | Orchestration frameworks | Agent protocols | Token optimization | **CLAN** |
|---|:---:|:---:|:---:|:---:|
| Coordinates agents at runtime | ✅ | ✅ | ❌ | ❌ |
| State survives framework boundaries | ❌ | ❌ | ❌ | ✅ |
| Provenance enforced by design | ❌ | ❌ | ❌ | ✅ |
| Human-readable artifact (in the file) | ❌ | ❌ | ❌ | ✅ |
| Human edits attributable on the record | ❌ | ❌ | ❌ | ✅ |
| Output contract enforced at write time | ❌ | ❌ | ❌ | ✅ |
| Deterministic parallel merge, zero LLM | ❌ | ❌ | ❌ | ✅ |
| Agent picks up cold from artifact alone | ❌ | ❌ | ❌ | ✅ |
| No runtime dependency | ❌ | ❌ | ✅ | ✅ |
| Model agnostic | 〰️ | ✅ | ✅ | ✅ |
| Open spec, any language can implement | 〰️ | ✅ | ✅ | ✅ |

〰️ = varies by tool.

---

## What it looks like

```
my-document.clan          ← standard ZIP — open it with anything
├── manifest.yaml         ← identity, lineage, file registry with checksums
├── spec/
│   ├── clan.md           ← the full spec, embedded in every file
│   └── agent-guide.md    ← byte-stable protocol guide; agents read this to learn CLAN
├── shared/
│   └── data.yaml         ← canonical facts; agents and humans read the same data
├── agent/
│   ├── context.md        ← the current agent's task
│   ├── output-schema.json← what this agent must produce — validated at pack time
│   ├── state.yaml        ← current document state
│   └── decision-chain.yaml ← every decision, attributed, compressed beyond the window
├── agents/               ← per-agent namespaces; writes outside your namespace are rejected
│   └── <agent-id>/
├── merge-report.yaml     ← contested keys from the last merge, both sides, with provenance
└── human/
    ├── index.html        ← the human-readable view, inside the artifact
    ├── patches.yaml      ← human edits, attributed edited_by: human
    └── assets/
```

Plain text. Standard ZIP. No proprietary encoding. Any language can read and write `.clan` — the Rust SDK is a reference implementation, not a gate.

---

## Quickstart

**Install:** pre-built binaries for Linux, macOS (Apple Silicon + Intel), and Windows — plus a desktop viewer (`.dmg` / `.msi` / `.AppImage`) — on the [Releases page](https://github.com/saieeshward/clan/releases).

```bash
cargo install --path crates/clan-cli

clan create --title "Q3 Market Analysis" \
  --brief "Evaluate CRM options for a 40-person agency" --output doc.clan

# The file tells any agent everything it needs
clan read agent doc.clan

# Attribution is enforced by the format, not by convention
clan patch-data doc.clan --set "verdict=HubSpot" \
  --agent analyst --action "set verdict" --rationale "best fit for budget"

# The decision chain lives in the file
clan read chain doc.clan

# The output contract is validatable at any point
clan validate doc.clan
```

### Parallel work

```bash
clan fork doc.clan --agents researcher,analyst --output-dir branches
# Writes outside agents/<id>/ are rejected — not by convention, by the CLI

clan patch-data branches/researcher.clan --namespace \
  --set "finding=market is growing" --agent researcher --action research
clan patch-data branches/analyst.clan --namespace \
  --set "risk=vendor lock-in" --agent analyst --action analyze

clan merge branches/*.clan --output merged.clan
# Deterministic. Zero LLM tokens. Contested keys in merge-report.yaml with both sides.
```

The CLI teaches itself — every command emits a `next:` hint. In our benchmark, agents given only *"there's a `clan` CLI — figure it out"* reached correct usage in under 4 discovery commands, with zero violations. The embedded guide is the only training material needed.

---

## What the benchmarks say

258 real agents. No scripted outputs. CLAN and ad-hoc arms running concurrently on identical tasks. Three campaigns. Every artifact is in [`test-sandbox/`](test-sandbox/) so you can check our work.

### What held up

**Revision loops: 66% fewer output characters.** Eight serial edits to a 45 KB report — CLAN's patch path wrote 0.336× what careful hand-editing produced. When we handed the ad-hoc arm the exact same text fragments to remove the structural advantage, CLAN still came out 45% leaner across 5 reps. Patching a field is cheaper than rewriting a document. That's a property of the format.

**The merge caught what every other arm missed.** Four agents ran in parallel and all wrote to a key called `assumptions`. In the ad-hoc arm, last-write-wins silently dropped the risk analyst's GDPR/PII scoping. Nobody noticed. In the CLAN arm, the merge surfaced both versions with full provenance at zero LLM tokens. The synthesizer restored the dropped finding, on the record. That's a property of the namespace design.

**When a human edited the document mid-pipeline, the file proved it.** Same live directive in both arms. Both agents obeyed it. Only the CLAN artifact can demonstrate a human was involved: `edited_by: human`, timestamped, cited in the decision chain. In a regulated context, that's not a nice-to-have. That's a property of the format.

**A fresh agent resumed an abandoned pipeline from the file alone.** No briefing, no summary, no context from the previous agent — just the `.clan` file. Three reads to orient, then it continued correctly. That's a property of the embedded spec.

**The format survived bad prompts.** Strip away the careful system prompt: ad-hoc pipelines produce unstructured outputs, miss fields, lose provenance. CLAN files came out byte-for-byte as complete with zero CLAN-specific prompting as they did with careful guidance. The structure carries the discipline.

| What the final artifact carries | CLAN guided | CLAN **unguided** | Ad-hoc guided | Ad-hoc **unguided** |
|---|:---:|:---:|:---:|:---:|
| Structured, machine-readable data | ✅ | ✅ | ✅ | ❌ |
| Working state / handoff notes | ✅ | ✅ | 〰️ | ✅ |
| Output contract (JSON Schema) | ✅ | ✅ | ❌ | ❌ |
| Provenance (who/what/why, timestamped) | ✅ | ✅ | 〰️ | ❌ |
| Machine-validatable | ✅ | ✅ | ❌ | ❌ |
| Renderable human view | ✅ | ✅ | ✅ | ❌ |

〰️ = partial.

### The full scorecard — including the losses

| Claim | Measured (run 2026-06-12-I) | Threshold | Status |
|---|---|---|:---:|
| Revision loops: CLAN patch path vs ad-hoc full-rewrites (8-hop) | **0.336× (66% fewer chars)** | ≤ 0.65 | ✅ PASS |
| …composition-controlled (ad-hoc handed same fragments, 5 reps) | **0.554× (45% fewer)** | ≤ 0.50 | 🟡 NEAR |
| TOON encoding saves vs minified JSON on tabular data | **51–58%** | ≥ 30% | ✅ PASS |
| Fidelity: every requested edit present, untouched fields intact | **8/8 in 4 of 5 heavy reps** | = 1.0 | ⚠️ see note |
| Provenance: every mutating hop attributed end-to-end | **0 `unknown-agent` entries** | ≥ 1.0 | ✅ PASS |
| Reliability: agents recover from CLI errors without orchestrator help | **0 unrecovered** | = 0 | ✅ PASS |
| Contested-key fork/merge: all conflicts recalled with winner + loser provenance | **4/4 keys** | 4/4 | ✅ PASS |
| Metamorphosis: doc transforms fully per hop, nothing lost | **5/5 checks** | all | ✅ PASS |
| Teachability: unguided agents reach correct protocol from `agent-help` alone | **0 violations, all attributed** | 0 | ✅ PASS |
| Cold resume: fresh agent finds correct next step from artifact alone | **oriented, no rework** | — | ✅ PASS |
| Agent guide byte-identical within a build (prompt-cache friendly) | **1 hash / build** | 1 | ✅ PASS |
| Workspace unit + integration tests | **186 / 186** | all | ✅ PASS |
| CLI conformance harness (macOS + Windows) | **26 / 26, 0 hard failures** | all | ✅ PASS |
| Synthesis hop: CLAN injection beats ad-hoc re-reading all inputs | **volatile: 0.487× (run -H) → 1.047× (run -I)** | < 1.0 | ⚠️ NOT ROBUST |
| CLAN per-hop injection crosses below ad-hoc on long chains | **no clean crossover** | crossover | ❌ EXPECT-RED |

**We report the latest run, not the best one.**

### Where CLAN loses

**Short chains: lean ad-hoc costs fewer input tokens.** The format carries scaffolding — a schema, a decision chain, a guide digest — that a pile of markdown files doesn't. At 3 hops, that's overhead with no payoff yet. If your pipeline is short and your prompts are disciplined, ad-hoc will be leaner on raw injection size.

**The injection crossover didn't hold.** The synthesis-hop win measured 0.487× in one run and 1.047× in the next. We kept the claim in the scorecard and marked it red.

**Provenance is only as truthful as the agents.** In one of five heavy reps, an agent wrote attributed decisions claiming it had applied edits that were never made. CLAN guarantees who acted and when — it cannot verify the agent's account of what they did. Run a verifier hop in any pipeline where fidelity matters.

**Wall-time savings are modest.** ~12–18% on 8–10 hop chains. Output token savings don't dominate latency.

---

## CLI reference

The CLI is the universal read/write interface for the format — the equivalent of `jq` for JSON.

| Command | What it does |
|---|---|
| `clan read agent` | The full accumulated context as one optimized prompt — compressed data, decision tail digest, guide |
| `clan patch-data --set k=v` | Write one field with attribution — rejected without `--agent` and `--action` |
| `clan patch-html --selector` | Update one element of the human view |
| `clan merge` | Merge parallel branches deterministically, zero LLM tokens |
| `clan validate` | Check the output contract against the schema at any point |

A Tauri desktop app renders the human view with click-to-edit. Edits go into the file as `edited_by: human` patches, part of the provenance chain.

---

## Documents

| Document | Description |
|---|---|
| [spec/CLAN-SPEC.md](spec/CLAN-SPEC.md) | Full format specification |
| [spec/SEQUENCE-DIAGRAMS.md](spec/SEQUENCE-DIAGRAMS.md) | Key interaction flows |
| [research/](research/) | Benchmarks, comparisons, and the negative results |
| [CHANGELOG.md](CHANGELOG.md) | Release history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

---

## Name

**CLAN** — **C**ontext and **L**ive **A**gent **N**otation.

Like a clan, every file carries shared lineage — connecting it to every document that came before it.

---

## Status

**v1.1** — fork/join concurrency, deferred human-view rendering, conflict adjudication, self-teaching CLI. Verified by 186 Rust tests + 26-test black-box conformance in CI, with [binaries for every platform on the Releases page](https://github.com/saieeshward/clan/releases).

## Maintainers

Maintained by [Sai Eeshwar](https://github.com/saieeshward) and [Shreyansh Soni](https://github.com/batunii).

## License

[MPL-2.0](LICENSE) — the spec is open; implementations in any language are welcome.
