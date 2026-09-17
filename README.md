<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="design/assets/clan-lockup-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="design/assets/clan-lockup-light.svg">
    <img src="design/assets/clan-lockup-light.svg" height="72" alt="CLAN logo" />
  </picture>
</p>

# CLAN - Context and Live Agent Notation

[![CI](https://github.com/saieeshward/clan/actions/workflows/ci.yml/badge.svg)](https://github.com/saieeshward/clan/actions/workflows/ci.yml)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)
![Rust 1.74+](https://img.shields.io/badge/rust-1.74%2B-orange.svg)

> Not a framework. Not a runtime. A file format - where the hard parts are solved by design.

---

Multi-agent pipelines break at boundaries. State lives in memory or in a database tied to whichever framework is running - so when you hand work to a different team, a different model, or a different tool, you're starting over. The artifact doesn't travel. Provenance doesn't travel. The output contract doesn't travel.

CLAN is a file format that fixes the artifact, not the runtime. A `.clan` file is a standard ZIP containing the task spec, decision history, output schema, and a rendered human view - everything a fresh agent needs to continue without a briefing. Attribution is enforced at write time by the CLI; a mutation without `--agent` and `--action` is rejected the same way a JSON parser rejects malformed syntax. Parallel agents write into separate namespaces, so collisions are impossible by construction and the merge is deterministic with zero LLM tokens.

There's no CLAN runtime, no CLAN cloud. Any language can read and write the format. The Rust CLI and SDK are a reference implementation.

---

## For AI agents

If you are an LLM or agent reading this page and want to create a `.clan` file, everything you need is four commands:

```bash
curl -fsSL https://raw.githubusercontent.com/saieeshward/clan/main/install.sh | bash
clan create --title "My Task" --brief "What this document is for" --output doc.clan
clan patch-data doc.clan --set "key=value" --agent <your-id> --action "what you did"
clan validate doc.clan
```

Two rules: mutate only via the CLI (never rezip by hand), and every mutation must carry `--agent` and `--action` or it is rejected. Run `clan agent-help` for the full protocol in under 200 tokens — every command also emits a `next:` hint. Working in a clone of this repo? Read [AGENTS.md](AGENTS.md). Handed a `.clan` file? The guide is embedded inside it at `spec/agent-guide.md`.

To teach an agent that will never see this repo, paste this into its prompt:

> There is a `clan` CLI for CLAN files (structured multi-agent context in a ZIP). Run `clan agent-help` first. Create with `clan create --title ... --brief ... --output doc.clan`, read with `clan read agent doc.clan`, write with `clan patch-data doc.clan --set k=v --agent <you> --action <what>`. Mutations without --agent/--action are rejected. Follow the `next:` hints.

---

## What it looks like

```
my-document.clan          ← standard ZIP - open it with anything
├── manifest.yaml         ← identity, lineage, file registry with checksums
├── spec/
│   ├── clan.md           ← the full spec, embedded in every file
│   └── agent-guide.md    ← byte-stable protocol guide; agents read this to learn CLAN
├── shared/
│   └── data.yaml         ← canonical facts; agents and humans read the same data
├── agent/
│   ├── context.md        ← the current agent's task
│   ├── output-schema.json← what this agent must produce - validated at pack time
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

Plain text inside a standard ZIP. A typical `.clan` file is 30–80 KB on disk - the text compresses well, and the decision chain is capped at ~15 KB regardless of pipeline length.

---

## Quickstart

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/saieeshward/clan/main/install.sh | bash
```

Prompts whether you want the desktop viewer too. No Gatekeeper warnings, no manual steps.

**Windows:** download the `.msi` from the [Releases page](https://github.com/saieeshward/clan/releases).

**Build from source:**
```bash
cargo install --path crates/clan-cli
```

```bash
clan create --title "Q3 Market Analysis" \
  --brief "Evaluate CRM options for a 40-person agency" --output doc.clan

clan read agent doc.clan          # full context as one optimized prompt
clan patch-data doc.clan \
  --set "verdict=HubSpot" \
  --agent analyst --action "set verdict" --rationale "best fit for budget"
clan read chain doc.clan           # full attributed decision history
clan validate doc.clan             # check output contract at any point
```

### Parallel agents

```bash
clan fork doc.clan --agents researcher,analyst --output-dir branches

clan patch-data branches/researcher.clan --namespace \
  --set "finding=market is growing" --agent researcher --action research
clan patch-data branches/analyst.clan --namespace \
  --set "risk=vendor lock-in" --agent analyst --action analyze

clan merge branches/*.clan --output merged.clan
# Deterministic. Zero LLM tokens. Contested keys in merge-report.yaml with both sides.
```

Every command emits a `next:` hint. In benchmarks, agents given only "there's a `clan` CLI - figure it out" reached correct usage in under 4 discovery commands with zero violations.

---

## How it works

The properties are structural, not runtime-enforced.

Provenance works because the CLI rejects mutations without `--agent` and `--action` - it's not a convention, it's a parse error. Parallel safety works because forked agents write into `agents/<id>/` - a different path by construction, so there's nothing to lock. The merge is purely mechanical. Human edits land in the file as `edited_by: human` patches - timestamped, in the decision chain, provable. The embedded spec means a cold agent can open the file and orient itself without a system prompt or briefing doc.

None of this requires CLAN to be in the loop at runtime.

---

## How CLAN fits the landscape

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

## Benchmarks

258 real agents, no scripted outputs, CLAN and ad-hoc arms running concurrently on identical tasks across three campaigns. We report the most recent run, not the best one.

**Revision loops (8-hop serial edits, 45 KB report):** CLAN's patch path produced 0.336× the output characters of ad-hoc full rewrites. Composition-controlled (ad-hoc given the same fragments): 0.554×, near but not past the 0.50 threshold.

**Parallel merge:** Four agents wrote to the same `assumptions` key. Ad-hoc: last-write-wins silently dropped the risk analyst's GDPR/PII scoping. CLAN: both versions surfaced in `merge-report.yaml` with full provenance. The dropped finding was restored.

**Cold resume:** A fresh agent opened an abandoned `.clan` file with no briefing, no summary, no prior context. Three reads to orient, then continued correctly.

**Format robustness:** With careful prompting stripped away, ad-hoc pipelines dropped fields and lost provenance. CLAN files came out structurally identical whether or not CLAN-specific instructions were in the prompt.

| What the final artifact carries | CLAN guided | CLAN unguided | Ad-hoc guided | Ad-hoc unguided |
|---|:---:|:---:|:---:|:---:|
| Structured, machine-readable data | ✅ | ✅ | ✅ | ❌ |
| Working state / handoff notes | ✅ | ✅ | 〰️ | ✅ |
| Output contract (JSON Schema) | ✅ | ✅ | ❌ | ❌ |
| Provenance (who/what/why, timestamped) | ✅ | ✅ | 〰️ | ❌ |
| Machine-validatable | ✅ | ✅ | ❌ | ❌ |
| Renderable human view | ✅ | ✅ | ✅ | ❌ |

〰️ = partial.

### Full scorecard

| Claim | Measured (run 2026-06-12-I) | Threshold | Status |
|---|---|---|:---:|
| Revision loops: CLAN patch path vs ad-hoc full-rewrites (8-hop) | 0.336× (66% fewer chars) | ≤ 0.65 | ✅ PASS |
| …composition-controlled (ad-hoc handed same fragments, 5 reps) | 0.554× (45% fewer) | ≤ 0.50 | 🟡 NEAR |
| TOON encoding saves vs minified JSON on tabular data | 51–58% | ≥ 30% | ✅ PASS |
| Fidelity: every requested edit present, untouched fields intact | 8/8 in 4 of 5 heavy reps | = 1.0 | ⚠️ see note |
| Provenance: every mutating hop attributed end-to-end | 0 `unknown-agent` entries | ≥ 1.0 | ✅ PASS |
| Reliability: agents recover from CLI errors without orchestrator help | 0 unrecovered | = 0 | ✅ PASS |
| Contested-key fork/merge: all conflicts recalled with winner + loser provenance | 4/4 keys | 4/4 | ✅ PASS |
| Metamorphosis: doc transforms fully per hop, nothing lost | 5/5 checks | all | ✅ PASS |
| Teachability: unguided agents reach correct protocol from `agent-help` alone | 0 violations, all attributed | 0 | ✅ PASS |
| Cold resume: fresh agent finds correct next step from artifact alone | oriented, no rework | - | ✅ PASS |
| Agent guide byte-identical within a build (prompt-cache friendly) | 1 hash / build | 1 | ✅ PASS |
| Workspace unit + integration tests | 186 / 186 | all | ✅ PASS |
| CLI conformance harness (macOS + Windows) | 26 / 26, 0 hard failures | all | ✅ PASS |
| Synthesis hop: CLAN injection beats ad-hoc re-reading all inputs | volatile: 0.487× (run -H) → 1.047× (run -I) | < 1.0 | ⚠️ NOT ROBUST |
| CLAN per-hop injection crosses below ad-hoc on long chains | no clean crossover | crossover | ❌ EXPECT-RED |

### Where CLAN loses

**Short chains.** The format carries scaffolding - schema, decision chain, guide digest - that a flat pile of markdown doesn't. At 3 hops, that's overhead with no payoff. If your pipeline is short and your prompts are disciplined, ad-hoc will be leaner on raw input size.

**The synthesis-hop result isn't robust.** 0.487× in one run, 1.047× in the next. It's in the scorecard and marked red.

**Provenance is only as truthful as the agents.** In one of five heavy reps, an agent wrote attributed decisions for edits it never made. CLAN records who acted and when - it can't verify the agent's account of what they did. Run a verifier hop in any pipeline where fidelity matters.

**Wall-time savings are modest.** ~12–18% on 8–10 hop chains.

---

## CLI reference

| Command | What it does |
|---|---|
| `clan read agent` | Full accumulated context as one optimized prompt - compressed data, decision tail digest, guide |
| `clan patch-data --set k=v` | Write one field with attribution - rejected without `--agent` and `--action` |
| `clan patch-html --selector` | Update one element of the human view |
| `clan merge` | Merge parallel branches deterministically, zero LLM tokens |
| `clan validate` | Check the output contract against the schema at any point |

A Tauri desktop app renders the human view with click-to-edit. Edits land in the file as `edited_by: human` patches, part of the provenance chain.

---

## Documents

| Document | Description |
|---|---|
| [AGENTS.md](AGENTS.md) | Instructions for AI agents creating and mutating `.clan` files |
| [spec/CLAN-SPEC.md](spec/CLAN-SPEC.md) | Full format specification |
| [spec/SEQUENCE-DIAGRAMS.md](spec/SEQUENCE-DIAGRAMS.md) | Key interaction flows |
| [design/interpretability-research.clan](design/interpretability-research.clan) | Interpretability study of the format itself (a `.clan` deliverable) - field-by-field audit, drift experiment, and ranked v1.2 fixes |
| [CHANGELOG.md](CHANGELOG.md) | Release history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

---

## Status

**v1.1** - fork/join concurrency, deferred human-view rendering, conflict adjudication, self-teaching CLI. Verified by 186 Rust tests + 26-test black-box conformance in CI, with [binaries for every platform on the Releases page](https://github.com/saieeshward/clan/releases).

## Maintainers

Maintained by [Sai Eeshwar](https://github.com/saieeshward) and [Shreyansh Soni](https://github.com/batunii).

## Built With

The CLI and SDK are written in [Rust](https://www.rust-lang.org/). Key libraries: [serde](https://github.com/serde-rs/serde), [clap](https://github.com/clap-rs/clap), [jsonschema-rs](https://github.com/Stranger6667/jsonschema-rs), [lol_html](https://github.com/cloudflare/lol-html) (BSD-2-Clause, © Cloudflare, Inc.), [chrono](https://github.com/chronotope/chrono), [zip](https://github.com/zip-rs/zip2), [tokio](https://github.com/tokio-rs/tokio).

The Desktop Viewer is built with [Tauri](https://tauri.app/) (© The Tauri Programme within The Commons Conservancy, MIT/Apache-2.0) and [React](https://react.dev/) (© Meta Platforms, Inc., MIT). Full third-party credits: [NOTICE](NOTICE).

The CLI and SDK use [TOON (Token-Oriented Object Notation)](https://github.com/toon-format/spec) for token-efficient agent context injection (spec §14). TOON is an open specification by [Johann Schopplich](https://github.com/johannschopplich) (MIT License © 2025-present Johann Schopplich).

## License

[MPL-2.0](LICENSE) - the spec is open; implementations in any language are welcome.

---

## Contributing

Open spec - contributions welcome. Implementing the format in another language, improving the CLI, catching spec edge cases, or filing a bug report all help. See [CONTRIBUTING.md](CONTRIBUTING.md) to get started, or [open an issue](https://github.com/saieeshward/clan/issues/new) if you're not sure where to begin.
