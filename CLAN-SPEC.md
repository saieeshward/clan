# CLAN Specification v1.0

**Context and Live Agent Notation**
Pronounced "clan"

---

## Table of Contents

1. [Overview](#1-overview)
2. [Design Principles](#2-design-principles)
3. [Container Format](#3-container-format)
4. [Directory Structure](#4-directory-structure)
5. [manifest.yaml](#5-manifestyaml)
6. [shared/data.yaml](#6-shareddatayaml)
7. [agent/ Directory](#7-agent-directory)
8. [human/ Directory](#8-human-directory)
9. [spec/ Directory](#9-spec-directory)
10. [Output Modes](#10-output-modes)
11. [Patch System](#11-patch-system)
12. [Lineage Model](#12-lineage-model)
13. [External Store References](#13-external-store-references)
14. [Agent Injection Protocol](#14-agent-injection-protocol)
15. [Static Export](#15-static-export)
16. [Security Model](#16-security-model)
17. [Validation Rules](#17-validation-rules)
18. [Versioning](#18-versioning)
19. [MCP Compatibility](#19-mcp-compatibility)
20. [Implementation Checklist](#20-implementation-checklist)

---

## 1. Overview

CLAN (Context and Live Agent Notation) is an open container format for passing structured context between AI agents and rendering that context for humans. A `.clan` file is simultaneously:

- A **machine-readable data container** for AI agents — structured YAML/JSON with explicit schemas
- A **human-renderable document** — rich HTML with CSS, charts, and typography
- A **provenance record** — full history of every agent decision that produced it
- A **self-describing artifact** — carries its own specification so any agent can understand and produce it without prior training

Like a clan, every CLAN file carries its lineage — a chain of parent references connecting every version back to the origin document. Agents are the members. CLAN files are the shared history that binds them.

---

## 2. Design Principles

### Economy
Agents receive only what they need. The SDK extracts and assembles relevant sections rather than presenting the entire container. Older decision history is semantically compressed.

### Provenance
Every agent decision is recorded with rationale and timestamp. Full reasoning traces are referenced via content-addressed pointers to external stores, not embedded inline.

### Isolation
Agent content (`agent/`) and human content (`human/`) are strictly separated. Agents skip human sections entirely. The app collapses agent sections by default.

### Sufficiency
Every CLAN file carries its own specification (`spec/`). An agent that has never encountered CLAN before can understand the format and produce valid output from a single file.

### Openness
The format specification and reference SDK are open (Apache 2.0). No CLAN-compatible implementation requires a licence. The format does not depend on any proprietary system.

---

## 3. Container Format

| Property | Value |
|---|---|
| Container | ZIP (ISO 21320-1) |
| Compression | DEFLATE (level 6 default) |
| File extension | `.clan` |
| MIME type | `application/vnd.clan` |
| Encoding | UTF-8 for all text files |
| Binary assets | Stored as-is within ZIP |

### ZIP Ordering Convention

The `manifest.yaml` file SHOULD be stored as the first entry in the ZIP Central Directory. This allows consumers to read container metadata with minimal I/O — open archive, read first entry, parse manifest, then selectively access other files by name.

Do NOT use the "first entry uncompressed" trick (as EPUB does with `mimetype`). This is fragile — ZIP tools are not required to preserve entry order.

### File Naming

All file paths within the archive use forward slashes (`/`) regardless of host OS. No path may begin with `/` or `..`. All names are case-sensitive.

---

## 4. Directory Structure

### Required Files

Every valid CLAN archive MUST contain these files:

```
manifest.yaml
spec/clan.md
spec/agent-guide.md
shared/data.yaml
agent/context.md
agent/output-schema.json
agent/state.yaml
agent/decision-chain.yaml
```

### Optional Files

```
human/index.html          ← rich human rendering (HTML fragment)
human/index.txt           ← plain text fallback
human/styles.css          ← document styles
human/patches.yaml        ← human text edits
human/assets/*            ← SVG, images, fonts
agent/next-agent-brief.md ← instructions for the agent after this one
spec/schemas/*            ← JSON Schema files for validation
```

### Reserved Prefixes

- `spec/` — format specification files. Do not use for application data.
- `agent/` — machine-readable content. Do not embed rendered content here.
- `human/` — human-renderable content. Do not embed structured data here.
- `shared/` — canonical data shared by both audiences.

---

## 5. manifest.yaml

The manifest is the root index of every CLAN file. It MUST be valid YAML and MUST be the first entry in the ZIP.

### Schema

```yaml
# Required fields
clan_version: 1                          # integer — major version
clan_version_minor: 0                    # integer — minor version
id: "550e8400-e29b-41d4-a716-446655440000"  # UUID v4
title: "Invoice Review — Acme Corp Q2"  # string
created_at: "2026-05-29T10:00:00Z"      # ISO 8601
updated_at: "2026-05-29T14:32:00Z"      # ISO 8601

# Optional: document classification
document_type: "invoice"                # string, freeform

# Optional: lineage (absent on first CLAN in a chain)
lineage:
  parent_id: "3f9a2b1c-..."            # UUID of parent CLAN
  parent_uri: "file:///docs/inv-001.clan"  # URI to parent file
  parent_sha256: "sha256:a3f4b2c1..."  # SHA-256 of the parent .clan file (ZIP bytes)
  delta: "Corrected vendor name, re-extracted line items"  # semantic description

# Optional: external persistent store references
external:
  - id: "context-store"                # stable ID for internal reference
    uri: "mcp://context.example.ai/sessions/abc123"
    type: "mcp-resource"               # mcp-resource | vector-store | s3 | custom
    description: "Full reasoning traces for this document pipeline"
    access: "read"                     # read | write | read-write

# Required: file registry
files:
  - id: "canonical-data"              # stable ID (not path-dependent)
    path: "shared/data.yaml"
    role: "canonical-data"
    type: "application/yaml"
    sha256: "sha256:..."              # SHA-256 of the uncompressed file bytes
  - id: "agent-state"
    path: "agent/state.yaml"
    role: "agent-state"
    type: "application/yaml"
    sha256: "sha256:..."
  - id: "agent-context"
    path: "agent/context.md"
    role: "agent-context"
    type: "text/markdown"
    sha256: "sha256:..."
  - id: "agent-schema"
    path: "agent/output-schema.json"
    role: "agent-schema"
    type: "application/json"
    sha256: "sha256:..."
  - id: "agent-chain"
    path: "agent/decision-chain.yaml"
    role: "agent-chain"
    type: "application/yaml"
    sha256: "sha256:..."
  - id: "human-view"
    path: "human/index.html"
    role: "human-view"
    type: "text/html"
    priority: 1                        # preferred representation
    sha256: "sha256:..."
  - id: "human-text"
    path: "human/index.txt"
    role: "human-view"
    type: "text/plain"
    priority: 2                        # fallback representation
    sha256: "sha256:..."
  - id: "human-style"
    path: "human/styles.css"
    role: "human-style"
    type: "text/css"
    sha256: "sha256:..."
  - id: "human-patches"
    path: "human/patches.yaml"
    role: "human-patch"
    type: "application/yaml"
    sha256: "sha256:..."
  - id: "spec-full"
    path: "spec/clan.md"
    role: "spec-full"
    type: "text/markdown"
    sha256: "sha256:..."
  - id: "spec-guide"
    path: "spec/agent-guide.md"
    role: "spec-agent-guide"
    type: "text/markdown"
    sha256: "sha256:..."
```

### Role Enumeration

| Role | Description |
|---|---|
| `canonical-data` | Typed structured data, read by both agents and human rendering |
| `agent-state` | Current document state for agent consumption |
| `agent-context` | Task description for the current agent |
| `agent-schema` | JSON Schema defining valid agent output |
| `agent-chain` | Ordered provenance record of agent decisions |
| `agent-brief` | Instructions for the next agent in the pipeline |
| `human-view` | Human-renderable content (HTML, multiple allowed with priority) |
| `human-style` | Stylesheet for human content |
| `human-patch` | Human edit patches applied at render time |
| `human-asset` | Image, SVG, font, or other asset referenced by human content |
| `spec-full` | Complete format specification |
| `spec-agent-guide` | Compressed agent injection guide |

---

## 6. shared/data.yaml

The canonical data layer. Contains typed, structured facts that are simultaneously:
- Read by agents as structured data
- Injected into the human rendering as `window.__CLAN__.data` by the app

### Schema Declaration

Every `shared/data.yaml` MUST begin with a `$schema` declaration that references a JSON Schema file within the archive or a public URI.

```yaml
$schema: "spec/schemas/invoice.schema.json"

# Document data follows the schema
vendor: "Acme Corporation"
vendor_id: "V-20291"
invoice_number: "INV-2026-0042"
issue_date: "2026-05-29"
due_date: "2026-06-28"
currency: "EUR"
subtotal: 12500.00
tax_rate: 0.23
tax_amount: 2875.00
total: 15375.00
status: "pending-approval"
line_items:
  - description: "Software Development Services"
    quantity: 100
    unit_price: 125.00
    amount: 12500.00
```

### Rules

- All keys MUST be valid YAML scalar types (string, integer, float, boolean, null)
- Nested objects and arrays are permitted
- No computed or derived values — only canonical facts
- Values are not duplicated in `agent/state.yaml` or `human/index.html`
- Agents update this file via the SDK; they do not write it directly

---

## 7. agent/ Directory

### context.md

Plain Markdown. Describes what the current agent should do. Written by the SDK or a previous agent. Human-readable for debugging.

```markdown
# Invoice Extraction — Stage 1 of 3

This is a new CLAN document.

**Task**: Extract all fields from the attached invoice PDF and populate
shared/data.yaml. Achieve >0.90 confidence on all numeric fields.

**Output mode**: data-update

**Notes**:
- Vendor name may appear in two formats — use the legal entity name
- Line items should preserve original descriptions verbatim
- If confidence < 0.90 on any field, set that field to null and note it
  in your rationale

**Next stage**: Validation agent will verify extracted fields against
the supplier database.
```

### output-schema.json

A valid JSON Schema (Draft 7 or later) defining the exact structure the current agent must return. The SDK validates agent output against this schema before packaging.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["mode", "structured"],
  "properties": {
    "mode": {
      "type": "string",
      "enum": ["data-update", "designed", "full-html"]
    },
    "structured": {
      "type": "object",
      "required": ["vendor", "total", "status", "confidence"],
      "properties": {
        "vendor": { "type": ["string", "null"] },
        "total": { "type": ["number", "null"] },
        "status": { "type": "string", "enum": ["extracted", "partial", "failed"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "rationale": { "type": "string" }
      }
    }
  }
}
```

### state.yaml

The current canonical agent-readable state of the document. Updated by the SDK after each agent pass. Contains processed, agent-derived state — not raw input data (that lives in `shared/data.yaml`).

```yaml
pipeline_stage: 1
pipeline_total: 3
current_agent: "extractor-v2"
status: "extraction-complete"
confidence_overall: 0.94
fields_extracted: 12
fields_null: 0
ready_for: "validation"
```

### decision-chain.yaml

An ordered list of every agent decision. Newer entries are at the top. Older entries are semantically compressed by the SDK.

```yaml
# Compression tiers:
# - Entries 1-3:  full fidelity
# - Entries 4-15: key facts + rationale, no raw reasoning
# - Entry 16+:    2-sentence summary per entry

decisions:
  - agent: "extractor-v2"
    version: "2.1.4"
    action: "extracted invoice fields"
    rationale: "Used document structure analysis. Confidence 0.94 across
      all 12 fields. Vendor name matched legal entity format from footer.
      Line item descriptions preserved verbatim."
    timestamp: "2026-05-29T10:23:00Z"
    fields_changed: ["vendor", "total", "line_items", "status"]
    trace-ref:
      store: "context-store"
      entry: "step/1"
      content-hash: "sha256:a3f4b2c1d9e8f7a6b5c4d3e2f1a0b9c8"
```

**Compression rule**: The SDK applies semantic compression when a new decision is added:
- Entries 1–3: stored verbatim
- Entries 4–15: SDK passes through an LLM to compress — key facts + rationale preserved, raw reasoning removed
- Entry 16+: SDK compresses to a 2-sentence summary per entry

This caps `decision-chain.yaml` at approximately 15–20KB regardless of pipeline length.

### next-agent-brief.md (optional)

Instructions for the agent that will process the next CLAN in the lineage chain. Written by the current agent or a human orchestrator.

```markdown
# For the Validation Agent

You will receive a CLAN document with extracted invoice fields.

**Your task**: Verify vendor details against the supplier database.
Cross-reference invoice totals against the purchase order system.

**Expected output**:
- validated: boolean
- discrepancies: array of field names with issues
- recommendation: "approve" | "reject" | "escalate"
```

---

## 8. human/ Directory

### index.html

An HTML **fragment** — not a full HTML document. The app wraps it in a shell with its own `<head>`, base stylesheet, and script injection.

**Rules**:
- MUST NOT contain `<html>`, `<head>`, or `<body>` tags
- MUST NOT contain `<script>` tags
- MUST NOT reference external URLs in CSS `url()` calls
- SHOULD use the app's CSS design tokens (`var(--clan-accent)`, etc.)
- SHOULD assign `data-adf-id` attributes to all human-editable text elements
- MAY reference assets via relative paths (`./assets/chart.svg`)

```html
<section class="invoice-hero">
  <header class="doc-header">
    <h1 data-adf-id="heading-0" class="doc-title">Invoice Review</h1>
    <span class="doc-status status-pending">Pending Approval</span>
  </header>

  <div class="summary-grid">
    <div class="summary-card">
      <label>Vendor</label>
      <span class="vendor-name">{{vendor}}</span>
    </div>
    <div class="summary-card highlight">
      <label>Total</label>
      <span class="total-amount">{{currency}} {{total}}</span>
    </div>
  </div>

  <figure class="chart-container">
    <img src="./assets/line-items.svg" alt="Line items breakdown">
  </figure>

  <p data-adf-id="para-summary" class="summary-text">
    Invoice from {{vendor}} for services rendered in Q2 2026.
  </p>
</section>
```

### Data Binding

The app resolves `{{token}}` expressions by reading `shared/data.yaml` and injecting `window.__CLAN__ = { data: { ... } }` before rendering. The HTML references tokens using the double-brace syntax. Complex expressions are not supported — only direct key references and dot-notation for nested keys (`{{line_items.0.amount}}`).

### index.txt

Plain text fallback. Auto-generated by the SDK from the HTML fragment (strip tags, preserve structure). Used by agents requesting a human-readable summary and by accessibility tools.

### styles.css

Agent-generated stylesheet. Scoped to the document content — the app enforces CSS containment so agent styles cannot override app chrome.

### patches.yaml

Human edit patches applied at render time. Out-of-band from the agent-generated HTML — the HTML is never modified by human edits.

```yaml
patches:
  - id: "heading-0"                    # matches data-adf-id in HTML
    content: "Invoice Review — Amended"
    edited_at: "2026-05-29T14:15:00Z"
    edited_by: "human"
  - id: "para-summary"
    content: "Invoice from Acme Corporation. Line items verified. Awaiting finance sign-off."
    edited_at: "2026-05-29T14:17:00Z"
    edited_by: "human"
```

**Patch application order**:
1. Parse `human/index.html`
2. Resolve data bindings from `shared/data.yaml`
3. Assign `data-adf-id` to editable elements (Rust, at serve time)
4. Apply patches from `patches.yaml` (match by `data-adf-id`)
5. Serve via `clan://` custom protocol to document WebView

### assets/

Binary and vector assets referenced by `human/index.html`. All references MUST use relative paths (`./assets/filename`). The app's custom protocol resolves these correctly from within the ZIP.

Allowed asset types: SVG, PNG, JPEG, WebP, WOFF2, WOFF.

---

## 9. spec/ Directory

### clan.md

The complete CLAN specification. This file is the canonical spec that travels inside every `.clan` file. Its contents are identical to the public specification. Version pinned at creation time.

Purpose: enables any consumer — agent, human, tooling — to understand the CLAN format from a single file, without external documentation.

### agent-guide.md

A compressed, token-efficient guide written specifically for injection into LLM agent context. Target size: 600–1,000 tokens. See [Section 14](#14-agent-injection-protocol) for the canonical content of this file.

### schemas/

JSON Schema files for structured validation. Referenced by `shared/data.yaml` via `$schema`. The SDK validates `shared/data.yaml` against these schemas before packaging.

---

## 10. Output Modes

Agent output is a single JSON object. The `mode` field determines how the SDK processes it.

### Mode 1: data-update

Agent updates structured data only. The SDK re-renders `human/index.html` by substituting new values into the existing template via data bindings. HTML design is preserved.

Use for: extraction agents, validation agents, decision agents. Any agent that does not change the visual presentation.

```json
{
  "mode": "data-update",
  "structured": {
    "status": "validated",
    "confidence": 0.97,
    "discrepancies": [],
    "rationale": "All fields verified against supplier database. Totals match PO-2026-0018."
  }
}
```

### Mode 2: designed

Agent provides structured data and symbolic visual directives. The SDK renders HTML from the directives using the template library.

Use for: agents with design preferences that don't want to write raw HTML.

```json
{
  "mode": "designed",
  "structured": { ... },
  "design": {
    "theme": "dark-minimal",
    "accent_color": "#6366f1",
    "layout": "card-grid",
    "highlight_fields": ["total", "status"],
    "typography": "serif-display",
    "custom_css": ".invoice-total { font-size: 2rem; font-weight: 700; }"
  }
}
```

Supported themes: `light-clean`, `dark-minimal`, `warm-document`, `high-contrast`
Supported layouts: `card-grid`, `single-column`, `two-column`, `table-primary`, `timeline`

### Mode 3: full-html

Agent provides complete HTML fragment, CSS, and SVG assets. The SDK validates, sanitises, and packages them.

Use for: document-generating agents, report writers, first-time CLAN creation. Any agent with full design responsibility.

```json
{
  "mode": "full-html",
  "structured": { ... },
  "human": {
    "html": "<section class='report-hero'>...</section>",
    "css": ":root { --accent: #f59e0b; } .report-hero { ... }",
    "assets": {
      "revenue-chart.svg": "<svg viewBox='0 0 400 200'>...</svg>",
      "brand-mark.svg": "<svg>...</svg>"
    }
  }
}
```

**HTML rules for full-html mode**:
- Fragment only — no `<html>`, `<head>`, `<body>`
- No `<script>` tags
- No external URL references in CSS
- `data-adf-id` on all editable text elements
- Asset filenames match keys in the `assets` object

---

## 11. Patch System

Human text edits are stored out-of-band in `human/patches.yaml`. The agent-generated HTML is never directly modified by human input.

### Edit Flow

1. User opens CLAN in the app — human view rendered in sandboxed WebView
2. User activates edit mode (toolbar button or keyboard shortcut)
3. App injects edit bridge into document WebView via `clan://current/edit-bridge.js`
4. Edit bridge enables `contenteditable` on elements with `data-adf-id`
5. On text blur, edit bridge sends `postMessage({ type: "clan-edit", id, content })`
6. Shell WebView receives postMessage, calls Tauri `invoke("save_patch", { id, content })`
7. Rust backend appends to `human/patches.yaml` in the extracted CLAN
8. App re-renders to confirm change

### Patch Format

```yaml
patches:
  - id: string              # matches data-adf-id attribute in HTML
    content: string         # new text content (text/plain, no HTML)
    edited_at: string       # ISO 8601
    edited_by: string       # "human" | agent identifier
```

### Agent Access to Patches

Agents receive `human/patches.yaml` as part of their context (if the SDK is configured to include it). This allows agents to:
- Understand what humans changed since the last agent pass
- Incorporate human corrections into the next version's `shared/data.yaml`
- Acknowledge human feedback in the next decision-chain entry

---

## 12. Lineage Model

Every CLAN is a node in a directed acyclic graph (DAG) of document versions. The `lineage` block in `manifest.yaml` records the parent.

### Lineage Block

```yaml
lineage:
  parent_id: "3f9a2b1c-e29b-41d4-a716-446655440001"
  parent_uri: "file:///invoices/inv-001.clan"
  parent_sha256: "sha256:a3f4b2c1d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4"
  delta: "Validation complete. Vendor name corrected from 'ACME' to 'Acme Corporation'."
```

- `parent_id` — UUID of the parent CLAN (from its `manifest.yaml`)
- `parent_uri` — URI pointing to the parent file (file://, https://, mcp://, etc.)
- `parent_sha256` — SHA-256 of the parent `.clan` file's raw ZIP bytes. Makes the lineage chain cryptographically verifiable — any modification to a parent file is detectable when walking the chain.
- `delta` — human and agent-readable semantic description of what changed

### Rules

- First CLAN in a chain has no `lineage` block (or `lineage: null`)
- A CLAN MUST NOT reference itself as its own parent
- The lineage chain is reconstructed by following `parent_uri` references
- Parent CLAN files are never embedded — only referenced
- The app reconstructs and renders the lineage chain visually on demand
- Consumers verifying the chain SHOULD check `parent_sha256` against the resolved parent file and warn if the hash does not match

---

## 13. External Store References

Large context data (full reasoning traces, embedding indexes, raw conversation logs) lives outside the CLAN file and is referenced by URI with a content hash for integrity.

### manifest.yaml External Block

```yaml
external:
  - id: "context-store"
    uri: "mcp://context.example.ai/sessions/abc123"
    type: "mcp-resource"
    description: "Full reasoning traces for all agents in this pipeline"
    access: "read"
  - id: "vector-index"
    uri: "pinecone://clan-contexts/invoice-cluster"
    type: "vector-store"
    description: "Semantic search index for related CLAN files"
    access: "read"
```

### decision-chain.yaml trace-ref

Individual decision entries can reference their full context in the external store:

```yaml
- agent: "validator-v1"
  action: "verified vendor details"
  rationale: "Vendor confirmed in supplier database. PO match found."
  timestamp: "2026-05-29T11:15:00Z"
  trace-ref:
    store: "context-store"        # references external[].id in manifest.yaml
    entry: "step/2"
    content-hash: "sha256:b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2"
```

The `content-hash` enables any consumer to verify the external content hasn't changed since the CLAN was written. Consumers MAY skip verification but SHOULD warn if verification fails.

---

## 14. Agent Injection Protocol

When the SDK loads a CLAN file for an agent, it assembles a context object in this order:

| Order | Source | Approx. Tokens | Purpose |
|---|---|---|---|
| 1 | `spec/agent-guide.md` | ~800 | Format knowledge — always first |
| 2 | `agent/context.md` | ~400 | Task knowledge |
| 3 | `agent/output-schema.json` | ~300 | Output contract |
| 4 | `shared/data.yaml` | variable | Current facts |
| 5 | `agent/decision-chain.yaml` | variable | History |
| 6 | `human/patches.yaml` | ~200 | Human edits (if configured) |

The spec guide is injected once per agent session and cached. Subsequent calls in the same session do not re-inject it.

### Lazy Loading Contract

The SDK MUST NOT read entries from the ZIP that are not required for the current operation. Specifically:

- When assembling agent context: `human/` directory entries MUST NOT be read
- When rendering for humans: `agent/` directory entries MUST NOT be read (except `state.yaml` for the agent panel)
- `manifest.yaml` is always read first and is the only guaranteed upfront I/O
- All other entries are loaded on demand by role, never by scanning the full archive

This ensures the SDK's I/O footprint matches the operation being performed, not the total file size.

### TOON Serialisation for Injected Context

The SDK serialises `shared/data.yaml` and `agent/decision-chain.yaml` as **TOON (Token-Oriented Object Notation)** before injection. TOON encodes the same data model as JSON using indentation and explicit length declarations instead of brackets, achieving approximately 40% fewer tokens on structured data. This reduction applies on every agent call across the entire pipeline.

```
# shared/data.yaml injected as TOON — not raw YAML
vendor: Acme Corporation
total: 15375.00
status: pending-approval
line_items [2]
  description: Software Development Services
  amount: 12500.00
  description: Consulting
  amount: 2875.00
```

`spec/agent-guide.md`, `agent/context.md`, and `agent/output-schema.json` are injected as-is (Markdown and JSON respectively) — TOON conversion applies only to structured data files.

Agent output remains **JSON**. The agent returns a JSON object matching `output-schema.json`. The SDK validates the JSON output against the schema before packaging. TOON is an input-side optimisation only — the output contract is JSON for reliability and schema compatibility.

Implementations MAY offer a configuration flag to disable TOON injection and use raw YAML for debugging or compatibility.

### Canonical agent-guide.md Content

```markdown
# CLAN Agent Guide — v1.0

CLAN (Context and Live Agent Notation) is a file format for passing context
between AI agents and rendering it for humans. You received a .clan file.
The SDK has extracted what you need. Read this guide, then your task.

## What you received
- Your task:   agent/context.md        — read this first
- Data:        shared/data.yaml        — canonical facts
- History:     agent/decision-chain.yaml — what previous agents did
- Output spec: agent/output-schema.json — what you must return

## What you must return
A single JSON object matching agent/output-schema.json exactly.
The SDK packages your output into a valid .clan file automatically.
Return only the JSON object — no markdown wrapper, no explanation.

## Three output modes (set in output-schema.json)

"data-update"  → return updated data fields only.
               SDK re-renders the human view automatically.

"designed"     → return data + visual directives.
               { theme, accent_color, layout, highlight_fields }
               SDK generates HTML from your directives.

"full-html"    → return data + HTML fragment + CSS + SVG assets.
               You have full design control.

## What the SDK handles — do NOT attempt these
- Creating ZIP files or file paths
- Writing manifest.yaml
- Applying patches or tracking lineage
- Validating schemas
- HTML sanitisation

## Rules
- Return valid JSON only
- Match output-schema.json exactly
- HTML must be a fragment (no html/head/body tags)
- No <script> tags in HTML
- Use data-adf-id on editable text elements

## If something is unclear
Read agent/context.md — it has task-specific rules that override these.
```

---

## 15. Static Export

For agents without SDK access (direct LLM API calls, third-party agents), the SDK produces a single JSON export:

```python
xon_sdk.export_static("document.clan", output="document-static.json")
```

```json
{
  "clan_version": "1.0",
  "agent_guide": "...full contents of spec/agent-guide.md...",
  "task": "...contents of agent/context.md...",
  "output_schema": { "...": "JSON Schema object" },
  "shared_data": { "vendor": "Acme Corp", "total": 15375.00 },
  "decision_history": [ { "...": "compressed entries" } ],
  "patches": [ { "id": "heading-0", "content": "Amended title" } ]
}
```

The agent reads this, follows the embedded `agent_guide`, produces JSON matching `output_schema`, and returns it. The SDK receives the response and packages it into a valid `.clan` file. **The agent never knew it was working with CLAN.**

---

## 16. Security Model

### HTML Sanitisation (SDK layer)

Applied to all agent-provided HTML before packaging into the archive:

- Strip all `<script>` elements and content
- Strip all `on*` event handler attributes (`onclick`, `onerror`, `onload`, etc.)
- Strip `javascript:` URI schemes in `href` and `src`
- Strip CSS `expression()`, `behavior:`, and external `url()` imports
- Disallow `<iframe>`, `<object>`, `<embed>`, `<form>` elements
- Allow all other standard HTML5 elements and attributes

Implementation: use `ammonia` (Rust) with an explicit allowlist.

### Rendering Sandbox (App layer)

The Tauri app uses a multi-webview architecture:

- **Shell WebView** — trusted. Full Tauri IPC access. Renders app chrome (sidebar, toolbar, panels).
- **Document WebView** — sandboxed. No Tauri IPC. Renders agent-generated HTML via `clan://` custom protocol.

The document WebView is subject to CSP:
```
Content-Security-Policy:
  default-src 'none';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data:;
  font-src 'self';
  script-src 'none'
```

The `clan://` custom protocol is served by Rust from the in-memory extracted ZIP. No filesystem access from within the document WebView.

### Edit Bridge Security

The postMessage bridge between document WebView and shell WebView MUST:
- Validate message `type` against an allowlist (`clan-edit` only)
- Validate `id` against `data-adf-id` values from the rendered document
- Reject messages with unexpected origins
- Sanitise `content` (plain text only — no HTML in patch content)

---

## 17. Validation Rules

### Structural Validation

A CLAN file is **valid** if and only if:

- [ ] It is a readable ZIP archive
- [ ] `manifest.yaml` exists and parses as valid YAML
- [ ] `manifest.yaml` contains all required fields (`clan_version`, `clan_version_minor`, `id`, `title`, `created_at`, `updated_at`)
- [ ] `manifest.yaml` `id` is a valid UUID v4
- [ ] All files listed in `manifest.yaml` `files[]` exist in the archive
- [ ] `spec/clan.md` exists and is non-empty
- [ ] `spec/agent-guide.md` exists and is non-empty
- [ ] `shared/data.yaml` exists and parses as valid YAML
- [ ] `agent/context.md` exists and is non-empty
- [ ] `agent/output-schema.json` exists and is valid JSON Schema
- [ ] `agent/state.yaml` exists and parses as valid YAML
- [ ] `agent/decision-chain.yaml` exists and parses as valid YAML

### Content Validation

A CLAN file is **content-valid** if additionally:

- [ ] `shared/data.yaml` validates against its declared `$schema`
- [ ] `human/index.html` (if present) contains no `<script>` tags
- [ ] `human/index.html` (if present) contains no `<html>`, `<head>`, or `<body>` tags
- [ ] `human/patches.yaml` (if present) — all `id` values are non-empty strings
- [ ] `agent/decision-chain.yaml` — all entries have `agent`, `action`, `rationale`, `timestamp`
- [ ] `manifest.yaml` `lineage.parent_id` (if present) is a valid UUID v4
- [ ] All `sha256` values in `manifest.yaml` `files[]` match the SHA-256 of the corresponding entry's uncompressed bytes
- [ ] `manifest.yaml` `lineage.parent_sha256` (if present) matches `sha256:<hex>` format

### Version Validation

- Readers MUST reject files where `clan_version` > their supported major version
- Readers MUST accept files where `clan_version_minor` > their supported minor version (forward compatible)
- Readers SHOULD log a warning for minor version mismatches

---

## 18. Versioning

CLAN uses a two-integer version scheme declared in `manifest.yaml`:

```yaml
clan_version: 1        # major — breaking changes increment this
clan_version_minor: 0  # minor — backwards-compatible additions increment this
```

### Breaking Changes (major version increment)

- Renaming or removing required fields in `manifest.yaml`
- Changing the directory structure of required files
- Changing the output mode protocol
- Removing support for an existing output mode
- Changing the patch system key format

### Non-Breaking Changes (minor version increment)

- Adding new optional fields to `manifest.yaml`
- Adding new optional files to the directory structure
- Adding new `design` themes or layouts (Mode 2)
- Adding new `external` store types
- Adding new file roles

---

## 19. MCP Compatibility

CLAN is designed to be compatible with the Model Context Protocol (MCP). The agent context object assembled by the SDK (Section 14) follows MCP resource conventions.

An CLAN SDK MAY expose CLAN files as MCP resources:

```json
{
  "uri": "clan://document/shared/data",
  "name": "Canonical document data",
  "mimeType": "application/yaml",
  "text": "...contents of shared/data.yaml..."
}
```

An CLAN file may reference MCP resources in its `external` block (type `mcp-resource`). The SDK resolves these references via the MCP client when assembling agent context.

---

## 20. Implementation Checklist

For SDK implementors, in priority order:

**Core (required for basic compliance)**
- [ ] ZIP read with selective entry access (`by_name`)
- [ ] ZIP write with DEFLATE compression
- [ ] `manifest.yaml` parse, validate, write
- [ ] SHA-256 computation for all file entries written to manifest
- [ ] SHA-256 verification of file entries on open (warn on mismatch, do not silently pass)
- [ ] `spec/agent-guide.md` injection into agent context
- [ ] TOON serialisation of `shared/data.yaml` and `decision-chain.yaml` at injection time
- [ ] `agent/output-schema.json` validation of agent output (JSON)
- [ ] `data-update` output mode handling
- [ ] `shared/data.yaml` schema validation
- [ ] `human/patches.yaml` application at render time
- [ ] Lazy loading enforced: never read entries outside the required role set for the current operation

> **Implementation note**: Ship single-threaded and correct before adding any concurrency. Premature parallelism in file I/O and compression has caused severe regressions in comparable systems (up to 96% throughput degradation). Measure a real bottleneck before introducing worker pools or async I/O.

**Full (required for full compliance)**
- [ ] `designed` output mode (theme/layout rendering)
- [ ] `full-html` output mode (HTML sanitisation, asset packaging)
- [ ] Lineage tracking (parent reference, delta generation)
- [ ] Decision-chain compression (three-tier semantic compression)
- [ ] Static export generation
- [ ] External store reference resolution
- [ ] `data-adf-id` assignment at render time
- [ ] Content-hash verification for `trace-ref` entries

**App (required for CLAN viewer implementation)**
- [ ] File association registration (`.clan` → app)
- [ ] Cold-open file path buffering (wait for WebView ready signal)
- [ ] Multi-webview architecture (shell + sandboxed document)
- [ ] `clan://` custom protocol handler serving from in-memory ZIP
- [ ] Data binding resolution (`{{token}}` → `shared/data.yaml` values)
- [ ] Edit bridge injection and postMessage handling
- [ ] Lineage timeline rendering
- [ ] Agent section panel (structured YAML display, collapsed by default)
