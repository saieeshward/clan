# CLAN System Overview

## What Is CLAN?

CLAN (Context and Live Agent Notation) is a file format and toolchain for passing structured context between AI agents while co-producing a human-readable output document. It solves a specific problem: in a multi-agent pipeline where each agent adds to a shared document, how do you pass just the right context to each agent, track every decision, and let humans override results — without writing custom orchestration infrastructure for each pipeline?

A `.clan` file is a ZIP archive containing both machine-readable data and a rendered human view, versioned with a full decision chain. Every `clan pack` operation creates a new `.clan` with lineage back to its parent, a delta description, and a SHA256 hash.

## File Format

A `.clan` file is a standard ZIP archive. Contents:

```
manifest.yaml              # Document metadata, lineage, SHA256
shared/data.yaml           # Accumulated structured data (canonical source of truth)
agent/context.md           # Task description for the next agent
agent/decision-chain.yaml  # Full history of agent decisions (TOON-compressed)
agent/output-schema.json   # JSON Schema the next agent must match
human/index.html           # Latest rendered human view
human/patches.yaml         # Human text overrides (keyed by data-adf-id)
human/styles.css           # Optional injected stylesheet
```

The ZIP format means any zip tool can inspect a `.clan` file. No custom parser is required to read the raw contents.

## TOON — Token-Oriented Object Notation

TOON is CLAN's internal serialisation format for structured data passed to agents. It is a compact representation of YAML/JSON that reduces token consumption by ~40% by:

- Omitting quotation marks around values
- Collapsing array lengths into bracket notation (`items [3]` instead of `- item` × 3)
- Removing JSON/YAML boilerplate (colons-then-space, explicit nulls)
- Sorting keys alphabetically (consistent, predictable for caching)

TOON is not a separate format — it is a rendering of the same underlying YAML data. `clan read data` returns raw insertion-order YAML; `clan read agent` returns the same data TOON-encoded.

**Example comparison:**

Raw YAML (68 chars):
```yaml
pricing_tiers:
  - name: Starter
    seats: 3
    monthly_eur: 400
```

TOON (49 chars):
```
pricing_tiers [1]
  monthly_eur: 400
  name: Starter
  seats: 3
```

## Decision Chain

Every `clan pack` operation appends an entry to `agent/decision-chain.yaml`:

```yaml
- agent: market-researcher
  action: "Completed Irish AdTech OS market research"
  rationale: "Identified €46M TAM, clear competitive whitespace..."
  timestamp: 2026-06-01T15:47:57.734101+00:00
  fields_changed:
    - analysis_title
    - analyst
    - competitive_landscape
    - market_overview
    - pain_points
```

Entries beyond a verbatim window are compressed by the SDK. Entries marked `pinned: true` are never compressed — useful for status transitions, escalations, or complex conditionals that must be preserved exactly.

## Data Accumulation Model

Structured data accumulates across pipeline stages. When a stage-2 agent adds fields `go_to_market` and `overall_risk_rating`, they are merged into `shared/data.yaml` alongside all fields written by stage-1 agents. Every subsequent agent receives the full accumulated dataset.

This is the key architectural property: **a downstream agent always sees everything every upstream agent has written**, without the pipeline author needing to write any merge logic.

Key distinction from append-only logs: fields can be **overwritten**. If stage-2 updates a field set by stage-1, the new value becomes canonical and the change is recorded in `fields_changed` in the decision chain entry.

## The `patch-html` Human Override System

Humans can apply text edits to a rendered document without breaking the agent pipeline:

```bash
clan patch-html document.clan - << 'EOF'
---
mode: patch-html
patch_selector: "[data-adf-id='exec-summary']"
patch_action: replace
---
<p data-adf-id="exec-summary">Partner override: milestone-linked tranche structure recommended.</p>
EOF
```

Patches are stored in `human/patches.yaml` keyed by `data-adf-id`. They are applied at render time (not baked into the HTML), which means:
1. The original agent-generated HTML is preserved
2. Patches are visible to subsequent agents via `export-static`
3. Patches can be inspected, removed, or replaced independently

The desktop app (Tauri) integrates this system — clicking any `data-adf-id` element in edit mode writes directly to `human/patches.yaml` via the Rust backend.

## The `export-static` SDK-less Path

For agents that cannot use the CLAN SDK directly, `clan export-static` produces a flat JSON blob:

```json
{
  "clan_version": "1.0",
  "task": "...(context.md contents)...",
  "agent_guide": "...(full markdown guide)...",
  "shared_data": { ...parsed JSON object... },
  "decision_history_toon": "...(TOON string)...",
  "output_schema": { ...JSON Schema... },
  "patches": "...(raw YAML string)..."
}
```

`shared_data` is a fully parsed JSON object — SDK-less agents get clean structured access with no YAML/TOON parsing required.

## CLI Subcommands (v1.0.0)

| Command | Purpose |
|---------|---------|
| `clan create --title ... --brief ... <output>` | Initialise a new root document |
| `clan pack --output ... --delta ... <parent> <output.json>` | Pack a JSON agent output into a new stage |
| `clan pack-html --output ... --delta ... <parent> <output.html>` | Pack an HTML file (optionally with YAML frontmatter) into a new stage |
| `clan patch-html <file> -` | Apply an in-place HTML DOM patch from stdin |
| `clan read agent <file>` | Print full agent context (guide + task + TOON data + decision chain) |
| `clan read human <file>` | Print current rendered HTML |
| `clan read data <file>` | Print raw YAML structured data |
| `clan read chain <file>` | Print raw YAML decision chain |
| `clan validate <file>` | Structural integrity check (exits non-zero on failure) |
| `clan info <file>` | Print manifest metadata (title, id, version, lineage, SHA256, file count) |
| `clan export-static <file>` | Export flat JSON for SDK-less agents |
| `clan edit <file>` | Open structured data in `$EDITOR` for interactive editing |
| `clan agent-help` | Print compact agent-oriented reference (<200 tokens) |

## Desktop Application

The CLAN Viewer is a Tauri (Rust backend) + React (TypeScript frontend) desktop application. Key features:

- Opens `.clan` files via a native file dialog or CLI argument
- Renders `human/index.html` in a sandboxed iframe via the `clan://document` custom URI scheme
- Edit Mode: clicking any element with `data-adf-id` makes it contenteditable
- Edits are saved back to the `.clan` file via Rust, triggering automatic repacking
- Agent Panel: side panel showing decision chain and structured data
- CLI arg loading: `cargo tauri dev -- -- /path/to/file.clan` auto-loads the file on mount

**Tech stack**: Tauri v2, React 18, TypeScript, Vite 8, Rust (clan-sdk crate)
