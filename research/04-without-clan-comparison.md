# Without CLAN vs With CLAN — Full Comparison

This document compares the same multi-agent pipeline (4 specialist analysts + synthesis + final decision) executed with and without CLAN. The goal is to isolate exactly what CLAN contributes and where it falls short.

---

## The Pipeline Being Compared

**Scenario**: AgencyOS Ireland — Investment Analysis  
**Agents**: Financial Modeler, Competitive Intel, Customer Discovery, Regulatory/Product, Synthesis Lead, Investment Committee  
**Task**: Each agent reads prior context, adds specialist analysis, passes to next

---

## Step-by-Step Comparison

### Step 1 — Giving Each Agent Its Task

**Without CLAN**:
- Write a system prompt per agent (custom for each role)
- Every agent call must include: the original brief, all prior agent outputs (full JSON), and a schema/format spec
- Token cost scales linearly: stage 4 agent must receive stage 1 + stage 2 + stage 3 outputs in full
- No standard format — you invent your own schema or use ad-hoc JSON

**With CLAN**:
- `clan read agent file.clan` — one command, always current
- Output is ~1,340–1,740 tokens regardless of pipeline depth (TOON compression + agent guide reuse)
- Format is standardised: guide + task + TOON data + decision chain + output schema
- Zero custom context assembly code needed

**Advantage**: CLAN — ~65–75% fewer tokens at the synthesis stage

---

### Step 2 — Agent Produces Output

**Without CLAN**:
- Agent returns whatever JSON or HTML it produces
- You write validation code (JSON Schema, runtime checks) or get silent shape errors
- HTML artifacts stored separately from data; they can drift out of sync
- No lineage — no record of what field each agent wrote

**With CLAN**:
- `clan pack` / `clan pack-html` validates output at pack time
- `pack` rejects missing required fields with a clear error message
- HTML and data co-located in the same ZIP — cannot drift
- `fields_changed[]` recorded automatically in the decision chain

**Advantage**: CLAN — format contract enforced at pack time, lineage automatic

---

### Step 3 — Passing Prior Context to the Next Agent

**Without CLAN**:
- Write an orchestrator function that merges `financial.json + competitive.json + ...`
- Custom code per pipeline — no standard merge semantics
- Field name collisions between agents are silently resolved (usually last-writer-wins)
- The merged context object grows proportionally with every stage
- By stage 5: sending all prior outputs = 5,000–10,000 tokens of raw JSON just for context

**With CLAN**:
- The `.clan` file IS the accumulated context — no merge code needed
- TOON encoding compresses accumulated data ~40% vs raw YAML
- Decision chain is auto-compressed beyond the verbatim window
- Agent context size stays stable (~1,340–1,740 tokens) regardless of pipeline depth

**Advantage**: CLAN — no merge code, stable token cost, automatic compression

---

### Step 4 — HTML Co-Located with Data

**Without CLAN**:
- Two artifacts: `report.html` + `data.json` stored separately
- If an agent updates `data.json` but not `report.html` (or vice versa), they drift silently
- No render-time data binding — values in HTML must be hardcoded by the agent
- Large HTML strings in JSON context tokens (~5× expansion from escaping)

**With CLAN**:
- Single `.clan` ZIP contains both artifacts — they always move together
- Template binding `{{key}}` resolved at render time — HTML references data by name
- `clan pack-html` path: agent writes HTML directly, no JSON encoding needed
- HTML compression via ZIP: 180KB of raw HTML across 4 branches → 77KB in `.clan` files (57% ratio)

**Advantage**: CLAN — co-location prevents drift, template binding, lower token cost for HTML agents

---

### Step 5 — Tracking What Each Agent Decided and Why

**Without CLAN**:
- Either: add a `metadata` field to your JSON schema and remember to fill it in every agent prompt
- Or: don't, and decision history is lost entirely
- No standard for `fields_changed` — you either build it or don't have it
- No compression — metadata grows unbounded or you truncate it manually

**With CLAN**:
- `agent/decision-chain.yaml` updated automatically on every `pack`
- Every field mutation recorded with: agent name, action, rationale, timestamp, `fields_changed[]`
- `pinned: true` preserves critical decisions from compression
- Full chain readable with `clan read chain`

**Advantage**: CLAN — automatic, no agent effort required, searchable

---

### Step 6 — Human Override Mid-Pipeline

**Without CLAN**:
- Edit `competitive.json` or `report.html` directly
- No record of who changed what, when, or why
- Next agent's context is now inconsistent with the audit trail (it might re-use stale data)
- No mechanism to distinguish "human edited" from "agent wrote"

**With CLAN**:
- `clan patch-html` stores override in `human/patches.yaml` keyed by `data-adf-id`
- Override visible to all subsequent agents via `export-static` (the `patches` field)
- Original agent HTML preserved — patch is applied at render time, not baked in
- Override is reversible (delete the patch entry)

**Advantage**: CLAN — auditable, reversible, pipeline-safe

---

### Step 7 — Parallel Branch Fan-Out (This Simulation)

**Without CLAN**:
- Run 4 agents concurrently, collect outputs into a folder
- Write a merge function: handle key collisions, type mismatches, missing fields across 4 different schemas
- No standard for resolving conflicts — last-writer-wins is the typical silent default
- Synthesis agent prompt: manually assemble `{financial: {...}, competitive: {...}, ...}` — 8,000–15,000 tokens of raw JSON

**With CLAN**:
- Each branch produces its own `.clan` from the same root — no merge infrastructure needed
- Synthesis agent: call `clan export-static` on each branch, get `shared_data` as clean JSON
- Import the 4 clean JSON objects into the synthesis prompt — each ~10–15KB
- Chain synthesis output from one branch as parent using `clan pack-html`

**Mixed**: CLAN handles each branch cleanly, but merging branches into one chain requires custom export-static reads and a choice of which branch to chain from. CLAN has no native multi-parent merge. This is a limitation.

---

### Step 8 — Synthesis Agent Context Cost

**Without CLAN (this pipeline)**:
- Brief: 184 words
- Financial output raw JSON: ~800 words
- Competitive output raw JSON: ~600 words
- Customer output raw JSON: ~700 words
- Regulatory output raw JSON: ~750 words
- Custom merge prompt instructions: ~200 words
- **Total: ~3,234 words → ~4,200 tokens**

**With CLAN (this pipeline)**:
- `clan read agent synthesis-parent.clan`: ~1,337 words → ~1,740 tokens (all data TOON-compressed)
- **Total: ~1,740 tokens**

**Saving: ~58% fewer tokens on the synthesis call.**

---

## What CLAN Does NOT Fix

| Limitation | Detail |
|---|---|
| Native multi-parent merge | CLAN is linear. Parallel branches require custom synthesis using `export-static`. No built-in merge. |
| Streaming output | `clan pack` is batch-only. No way to stream agent output tokens into a growing `.clan` file. |
| Arbitrary agent topology | DAG, loops, retries — CLAN assumes linear or fan-out-then-synthesize. Complex topologies require orchestration outside CLAN. |
| Non-HTML human view | The human layer is HTML only. If you want PDF, Markdown, or other formats as the canonical output, you need to convert. |
| Debugging / introspection | No `--verbose` flag. When something goes wrong in a pipeline, you inspect the ZIP manually or check `clan read data`. |
| Frontmatter format discoverability | The `structured:` wrapper key is not obvious to agents writing HTML. 3 of 4 agents in this simulation got it wrong on first try. |

---

## CLAN's Core Value Proposition, Precisely Stated

**CLAN converts pipeline infrastructure into a file format.**

Without CLAN, every multi-agent pipeline requires custom code for: context assembly, schema validation, lineage tracking, artifact co-location, human override management, and context compression. These are solved problems that every team re-solves differently.

CLAN solves them once, in a format that any agent, any tool, and any language can consume (the ZIP is standard; the TOON is readable text; the export-static is plain JSON). The cost is accepting CLAN's conventions: the `structured:` key, the `data-adf-id` attribute, the linear chaining model.

For pipelines that fit the model — and most document-production pipelines do — the fit is very high. For pipelines that need DAG topologies, streaming, or non-HTML output, CLAN is either insufficient or requires custom extensions.
