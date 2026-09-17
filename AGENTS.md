# AGENTS.md — working with CLAN

CLAN (Context and Live Agent Notation) is a file format for multi-agent pipelines. A `.clan` file is a standard ZIP carrying the task spec, structured data, attributed decision history, output schema, and a rendered human view. There is no runtime — the CLI below is a reference implementation.

This file teaches you to **create and mutate `.clan` files**. If you were *handed* a `.clan` file, the protocol guide is embedded inside it at `spec/agent-guide.md` (also in this repo) — read that instead.

## Install the CLI

```bash
curl -fsSL https://raw.githubusercontent.com/saieeshward/clan/main/install.sh | bash
# or from a clone:
cargo install --path crates/clan-cli
```

Then run `clan agent-help` — a compact (<200 token) agent-oriented reference. Every command also emits a `next:` hint telling you the correct next step. Prefer `clan agent-help` over `clan --help`.

## Create a file and work with it (happy path)

```bash
clan create --title "Q3 Market Analysis" \
  --brief "Evaluate CRM options for a 40-person agency" --output doc.clan

clan read agent doc.clan          # full accumulated context as one optimized prompt
clan patch-data doc.clan --set "verdict=HubSpot" \
  --agent analyst --action "set verdict" --rationale "best fit for budget"
clan read chain doc.clan          # attributed decision history
clan validate doc.clan            # check the output contract
```

## Hard rules

- **Mutate only via the CLI.** Never unzip, edit members, and rezip — you will break checksums and lineage.
- **Attribution is mandatory.** Any mutation without `--agent` and `--action` is rejected, the same way a JSON parser rejects malformed syntax. Add `--rationale` for anything non-obvious.
- **`patch-data` is a JSON Merge Patch (RFC 7396).** Keys you omit are kept — only restate what you change. Arrays replace by default; use `--append <key>` to add an item.
- **Do not re-transcribe carried-forward data.** Prior hops' data survives automatically.
- **The structured payload must conform to `agent/output-schema.json`.** To restructure the document's purpose, run `clan patch-schema` first (or pass `--schema` to `clan pack`).

## Pick the right write command

| Changing | Command |
|---|---|
| Structured data only | `clan patch-data` (surgical, lowest token cost) |
| One element of the HTML view | `clan patch-html --selector '#id'` |
| Whole HTML view | `clan pack-html` (expensive — use sparingly) |
| Data + view | `clan pack-html` with `structured:` YAML frontmatter |
| Decision/rationale only | `clan patch-decision` |
| Private scratchpad | `clan patch-state` |

`pack-html` errors if you pass `structured:` data without changing the view — use `patch-data` instead.

## Parallel work

```bash
clan fork doc.clan --agents researcher,analyst --output-dir branches
# each agent writes ONLY inside its own namespace, with --namespace
clan patch-data branches/researcher.clan --namespace \
  --set "finding=market is growing" --agent researcher --action research
clan merge branches/*.clan --output merged.clan
# deterministic, zero LLM tokens; conflicts land in merge-report.yaml, not failures
```

## Working on this repo itself

- Rust workspace; CLI in `crates/clan-cli`, SDK in `crates/`. Desktop viewer (Tauri + React) in `app/`.
- Run `cargo test --workspace` before proposing changes; the conformance harness is `conformance.json`.
- `spec/agent-guide.md` is **byte-stable within a build** (prompt-cache friendly) — do not edit it casually, and never duplicate its content elsewhere.
- Never list an AI as contributor, maintainer, or author.

## References

- [spec/CLAN-SPEC.md](spec/CLAN-SPEC.md) — full format specification
- [spec/agent-guide.md](spec/agent-guide.md) — protocol for agents *receiving* a `.clan` file
- [spec/SEQUENCE-DIAGRAMS.md](spec/SEQUENCE-DIAGRAMS.md) — key interaction flows
