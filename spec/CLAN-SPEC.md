# CLAN Specification v1.1

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
11. [In-Place Patching](#11-in-place-patching)
12. [Patch System](#12-patch-system)
13. [Lineage Model](#13-lineage-model)
14. [External Store References](#14-external-store-references)
15. [Agent Injection Protocol](#15-agent-injection-protocol)
16. [Static Export](#16-static-export)
17. [Security Model](#17-security-model)
18. [Validation Rules](#18-validation-rules)
19. [Versioning](#19-versioning)
20. [MCP Compatibility](#20-mcp-compatibility)
21. [Implementation Checklist](#21-implementation-checklist)
22. [Agent-Only Flow — The Preservation Rule](#22-agent-only-flow--the-preservation-rule)
23. [Deferred Rendering](#23-deferred-rendering)
24. [Fork/Join Concurrency](#24-forkjoin-concurrency)
25. [Conflict Adjudication](#25-conflict-adjudication)
26. [Conditional Context Injection](#26-conditional-context-injection)
27. [Teachable Interface](#27-teachable-interface)

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
The format specification and reference SDK are open (MPL-2.0). No CLAN-compatible implementation requires a licence. The format does not depend on any proprietary system.

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
human/index.html          ← rich human rendering (full HTML document or fragment)
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

An ordered list of every agent decision. Newer entries are at the top. Older entries are compressed by the SDK using a deterministic NLP pipeline — no model dependency.

```yaml
decisions:
  - agent: "extractor-v2"
    version: "2.1.4"
    action: "extracted invoice fields"
    rationale: "Used document structure analysis. Confidence 0.94 across
      all 12 fields. Vendor name matched legal entity format from footer.
      Line item descriptions preserved verbatim."
    timestamp: "2026-05-29T10:23:00Z"
    fields_changed: ["vendor", "total", "line_items", "status"]
    pinned: false                       # if true, never compressed regardless of age
    trace-ref:
      store: "context-store"
      entry: "step/1"
      content-hash: "sha256:a3f4b2c1d9e8f7a6b5c4d3e2f1a0b9c8"
```

**Compression model**: Two tiers, applied when a new decision is added during packaging.

- **Verbatim window** (most recent `compression_window` entries, default N=5): stored exactly as written. Current agents always receive full-fidelity recent history.
- **Compressed tail** (all entries beyond N): rationale field compressed in-place using the SDK's NLP pipeline (see below). All other fields — `agent`, `action`, `timestamp`, `fields_changed`, `trace-ref` — are always stored verbatim.

**Short-circuit rule**: if an entry's rationale is already at or under `compression_char_budget` (default 280 characters), it is stored verbatim regardless of position. No compression is applied to entries that don't need it.

**Pinned entries**: setting `pinned: true` on any entry opts it out of compression permanently, regardless of age or position. Agents SHOULD pin entries that represent status transitions, errors, retries, or decisions with complex conditional reasoning.

**`compression_window`** is configurable per SDK instance (default 5). Set higher for pipelines with many agents; set lower for pipelines where token economy is critical.

This caps `decision-chain.yaml` at approximately 10–15KB regardless of pipeline length.

### SDK Compression Pipeline

The SDK compresses older rationale fields using a deterministic pipeline. No model weights. No external API. Runs in ~0.5ms per entry and does not block the agent path — it runs during packaging after the agent has returned its output.

**Pipeline steps** (in order):

1. **YAKE keyword extraction** (`yake-rust` crate) — extract top-10 keyphrases from `action + rationale` combined. YAKE is single-document and unsupervised; no corpus required. Handles capitalised identifiers (agent names, field names, system IDs) as meaningful tokens.

2. **Sentence scoring** — score each sentence in `rationale` by YAKE keyword overlap, normalised by sentence length.

3. **Position weights** — first sentence +0.10 (restates task), last sentence +0.20 (states outcome). Outcomes appear last in agent rationale; standard extractive methods systematically miss them without this correction.

4. **Numeric lock** — any sentence containing a digit, percentage, or currency value is automatically included regardless of score. Numeric values (`confidence: 0.94`, `3.2% above PO`) are the most information-dense parts of agent rationale.

5. **Identifier preservation** — any sentence containing a hyphenated or versioned identifier (`extractor-v2`, `PO-2026-0018`) receives a +0.15 score bonus.

6. **Pick until budget** — select highest-scoring sentences until `compression_char_budget` is reached.

**Overriding the pipeline**: The SDK exposes a `CompressorFn` callback for applications that want higher-quality compression:

```rust
sdk.set_compressor(|action: &str, rationale: &str, budget: usize| -> String {
    // caller provides any model or strategy
    my_model.summarise(action, rationale, budget)
});
```

If no callback is set, the deterministic pipeline above is used. The format is identical either way — the callback is a quality upgrade, not a protocol change.

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

A complete HTML document or an HTML fragment. Full HTML documents are preferred — they give agents total design control with no style conflicts. The app detects which form it receives and handles both.

**Rules**:
- MUST NOT contain `<script>` tags (the app injects the edit bridge)
- MUST NOT contain `on*` event handler attributes
- MUST NOT contain `<iframe>`, `<object>`, `<embed>`, or `<form>` elements
- SHOULD assign `data-adf-id` attributes to all human-editable text elements
- MAY load fonts from CDNs (Google Fonts, Bunny Fonts, etc.) via `<link>` or `@import`
- MAY reference assets via relative paths (`./assets/chart.svg`)
- MAY use any CSS features including animations, custom properties, and external font imports

**Recommended structure** (full HTML document):
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=...">
  <style>/* all document styles here */</style>
</head>
<body>
  <!-- document content -->
</body>
</html>
```

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

Agent provides a complete HTML document (or fragment), CSS, and SVG assets. The SDK validates and packages them.

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
- Full HTML documents (`<!DOCTYPE html>...`) are preferred — use them for rich visual output
- Fragments (bare HTML without `<html>`/`<head>`/`<body>`) are also accepted
- `<script>` tags are fully allowed — use D3, Chart.js, count-up animations, anything
- `on*` event handler attributes are allowed
- No `<iframe>`, `<object>`, `<embed>`, or `<form>` elements
- `data-adf-id` on all editable text elements
- CDN fonts (`@import url()`, `<link>`) are fully supported
- Asset filenames match keys in the `assets` object

---

## 11. In-Place Patching

For minimal token cost, agents can bypass JSON/HTML output entirely and mutate the document in-place using precise patch formats (via `clan patch-*` CLI commands).

### `patch-html`
Target: `human/index.html`
Format: YAML frontmatter + HTML body.
```html
---
mode: patch-html
patch_selector: "div.content"
patch_action: "append"
---
<p>New content</p>
```

### `patch-data` & `patch-state`
Target: `shared/data.yaml` or `agent/state.yaml`
Format: JSON payload. Applied using RFC 7396 JSON Merge Patch.

### `patch-decision`
Target: `agent/decision-chain.yaml`
Format: CLI flags `--agent`, `--action`, `--rationale` to append cleanly.

### `patch-context`
Target: `agent/context.md`
Format: Markdown payload (overwrites or appends via `--append`).

### `patch-asset`
Target: `human/assets/`
Format: Binary/text injection natively into the ZIP without touching any other files.

---

## 12. Patch System

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

## 13. Lineage Model

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

## 14. External Store References

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

## 15. Agent Injection Protocol

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
- Full HTML documents are preferred; fragments are also accepted
- No `<script>` tags in agent-provided HTML
- No `on*` event handlers, no `<iframe>/<object>/<embed>/<form>`
- Use `data-adf-id` on editable text elements
- Use `{{key}}` in HTML for native data binding
- Use `window.__CLAN__.data` in scripts for native JS access
- CDN fonts (`@import`, `<link>`) are supported

## If something is unclear
Read agent/context.md — it has task-specific rules that override these.
```

---

## 16. Static Export

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

## 17. Security Model

### HTML Sanitisation (SDK layer)

Applied to all agent-provided HTML before packaging into the archive:

- `<script>` tags and `on*` event handlers are **not stripped** — the iframe sandbox (see below) confines them to a null origin with no Tauri IPC access, making them safe
- Strip `javascript:` URI schemes in `href` and `src`
- Strip CSS `expression()` and `behavior:` (legacy IE exploit vectors only)
- Disallow `<iframe>`, `<object>`, `<embed>`, `<form>` elements
- Allow all other standard HTML5 elements, attributes, and scripts
- Allow external CSS `url()` and `@import` for fonts and design assets

Implementation: SDK validates for the short disallowed list only; does not rewrite scripts or CSS URLs.

### Rendering Sandbox (App layer)

The Tauri app uses a multi-webview architecture:

- **Shell WebView** — trusted. Full Tauri IPC access. Renders app chrome (sidebar, toolbar, panels).
- **Document WebView** — sandboxed iframe (`sandbox="allow-scripts allow-popups"`, no `allow-same-origin`). Renders agent-generated HTML.

The iframe has `allow-scripts` (agent JS runs freely) but **not** `allow-same-origin`. Without `allow-same-origin`, `srcDoc` iframes get a null/opaque origin — agent scripts cannot access the parent's Tauri IPC, localStorage, or app state. `postMessage` (used by the edit bridge) works regardless of origin and is unaffected by this restriction.

The document WebView is subject to CSP (configured in `tauri.conf.json`):
```
Content-Security-Policy:
  default-src 'self' asset: https://asset.localhost https:;
  style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://fonts.gstatic.com https:;
  font-src 'self' https://fonts.gstatic.com https: data:;
  img-src 'self' asset: data: blob: https:;
  script-src 'self' 'unsafe-inline';
  connect-src 'self' https://ipc.localhost
```

CDN fonts (Google Fonts, etc.) are explicitly permitted. The `clan://` custom protocol is served by Rust from the in-memory extracted ZIP. No filesystem access from within the document WebView.

### Edit Bridge Security

The postMessage bridge between document WebView and shell WebView MUST:
- Validate message `type` against an allowlist (`clan-edit` only)
- Validate `id` against `data-adf-id` values from the rendered document
- Reject messages with unexpected origins
- Sanitise `content` (plain text only — no HTML in patch content)

---

## 18. Validation Rules

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
- [ ] **Strict Contract:** `shared/data.yaml` strictly conforms to the JSON Schema defined in `agent/output-schema.json`. (CLI mutation commands enforce this and will reject invalid data patches. The schema describes the data payload ONLY, not the CLI wrappers).
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

## 19. Versioning

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

## 20. MCP Compatibility

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

## 21. Implementation Checklist

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
- [ ] Decision-chain compression (verbatim window + YAKE NLP pipeline for tail entries)
- [ ] `pinned: true` flag respected — pinned entries never compressed
- [ ] `CompressorFn` callback API — optional override for AI-based compression
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
- [ ] Conflict badges on bindings whose key appears in `merge-report.yaml` (Section 25)

---

## 22. Agent-Only Flow — The Preservation Rule

A CLAN chain is **lossless** iff every hop preserves all five context layers. Each layer is optional to *read* but mandatory to *preserve*:

| # | Layer | Member(s) |
|---|---|---|
| 1 | Distilled state | `shared/data.yaml` |
| 2 | Working transcript / scratchpad | `agent/context.md`, `agent/state.yaml`, optional `agent/transcript/` |
| 3 | Contracts | `agent/output-schema.json` |
| 4 | Provenance | `agent/decision-chain.yaml` |
| 5 | Capability requirements | `agent/requirements.yaml` (optional) |

**The pass-through rule (normative):** an agent or SDK that consumes only a subset of layers MUST carry all other layers forward unmodified into the child file. `pack`, `pack-html`, and all `patch-*` operations satisfy this by construction; any third-party implementation MUST too. This single rule is what makes a chain pluggable into a different framework at any hop: the receiving framework finds every layer intact regardless of what intermediate hops read.

### agent/requirements.yaml (optional)

Tool *implementations* are framework-resident and cannot portably travel. CLAN carries the **declaration** instead, so a receiving framework can check capability fit before running the next hop:

```yaml
requires:
  tools:
    - name: web_search
      why: "verify vendor registration numbers"
    - name: code_execution
      why: "recompute totals"
  model_hints:
    min_context_tokens: 64000
```

Receiving orchestrators SHOULD warn (not fail) on unmet requirements. When present, the member is injected into agent context (Section 26) and must parse as YAML (Section 18). Written via `clan patch-requirements <file> <yaml>` or seeded at creation with `clan create --requirements <file>`.

---

## 23. Deferred Rendering

The structured members (Section 22, layers 1–5) are canonical. The human view (`human/`) is a **derivable artifact**: any conforming implementation can materialise it from the structured members at any point in the chain. Pure agent-to-agent chains skip rendering at every hop; the last hop — or a human's tooling — renders once.

### manifest.yaml: `view` field

```yaml
view:
  present: false        # human/index.html exists in this file
  renderable: true      # a view CAN be produced from current structured members
  stale: false          # present but produced from an older data.yaml generation
  source: agent         # how it was produced: "render" (default theme, safe to
                        # re-run) or "agent" (hand-authored, NOT safe to clobber)
```

Rules:
- `present: false, renderable: true` — agent-only file; render on demand.
- `stale: true` is set automatically by any data-changing operation that does not update the view; view-producing output modes (`full-html`, `patch-html`, `designed`) clear it and set `source: agent`. `clan render` sets `source: render`.
- The stale-view hint (§27) MUST NOT suggest a destructive `clan render` over a `source: agent` view — re-packing with `pack-html` preserves the hand-authored design; `render` replaces it with the default theme.
- A file with `present: true` MUST render without error from its own members (the Section 25 invariant guarantees `shared/data.yaml` is always single-valued, hence always bindable).
- The field is absent on v1.0 files: readers treat absence as "present iff `human/index.html` exists".

### CLI

- `clan create --no-render` / `clan pack --no-render` — produce agent-only files (`present: false`).
- `clan render <file>` — materialise `human/` from structured members (default deterministic theme renderer; an agent may instead produce a richer view via `pack-html`); sets `present: true, stale: false`. Scalar fields are emitted as `{{key}}` bindings so the viewer resolves live data.

Render-per-hop workflows are unchanged: omit `--no-render` and behaviour is identical to v1.0.

---

## 24. Fork/Join Concurrency

Concurrency model: **isolate writes, then deterministically fold.** No locks, no CRDTs, no merge servers. This mirrors the per-branch isolation + per-key reducer model production agent frameworks ship as their only supported concurrency mechanism — expressed as files.

### 24.1 `clan fork`

```
clan fork parent.clan --agents researcher,analyst,critic --output-dir branches/
```

Produces one child per agent. Each child:
- carries the full parent content (pass-through rule, Section 22);
- gains a manifest `fork` block naming its writer:

```yaml
fork:
  agent_id: researcher
  namespace: agents/researcher/
  forked_from: sha256:<parent digest>
```

- gains an empty private namespace: `agents/<agent_id>/data.yaml` and `agents/<agent_id>/decisions.yaml`, registered in the file registry with roles `branch-data` / `branch-decisions`.

**Namespace rule (normative):** on a forked file, the named agent MUST write only inside `agents/<agent_id>/`. Conforming implementations MUST reject writes to `shared/`, `agent/` contracts, and `human/` with a teaching error (Section 27.2). `patch-data --namespace` routes to the branch data; `patch-decision` auto-routes to the branch decision log; `patch-state` (the agent-private scratchpad) remains allowed. Conflicts are thereby impossible by construction during the parallel interval. Forking a branch file is rejected: merge first.

### 24.2 Multi-parent lineage

A merged file lists every parent. The single-parent fields remain the v1.0 form (`parent_id` holds the first parent, so v1.0 readers keep working):

```yaml
lineage:
  parent_id: <first-branch uuid>
  parent_uri: file:///...
  parent_sha256: sha256:<first-branch digest>
  delta: "merged branches: researcher, analyst"
  merge: true
  parents:
    - id: <branch uuid>
      sha256: sha256:<digest>
      agent_id: researcher
    - id: <branch uuid>
      sha256: sha256:<digest>
      agent_id: analyst
```

`merge: true` with fewer than 2 `parents[]` entries is a structural error.

### 24.3 `clan merge`

```
clan merge branches/*.clan --output merged.clan [--policy findings=append] [--prune-namespaces]
```

Folds every branch's `agents/<id>/data.yaml` into `shared/data.yaml`, deterministically, using per-key policies — explicit `--policy` overrides, else the manifest's `merge_policies` block, else `last-write`:

```yaml
merge_policies:
  default: last-write
  keys:
    findings: append           # concatenate values (arrays flatten)
    risk_score: max            # numeric max (min likewise)
    status: last-write         # the latest-listed writer wins
    summary: agent-priority    # the earliest-listed writer wins
```

Branches are folded in argument order; `last-write` means the latest-listed writer of a key wins. All branches MUST share one fork point (`forked_from`) and carry distinct agent ids. Branch `decisions.yaml` entries fold into the merged `agent/decision-chain.yaml` in timestamp order beneath a pinned-style `clan-merge` summary entry. Branch namespaces are retained in the merged file as provenance (cheap — ZIP-deflated) unless `--prune-namespaces`.

The merge is **mechanical — no LLM call required.** Token cost of the whole fan-out: each branch agent reads the shared base once plus its own namespace; no agent ever reads a sibling's transcript.

### 24.4 merge-report.yaml

Every key where two or more parents wrote **different values** under a winner-picking policy is recorded (`append` keeps everything and is therefore never a conflict):

```yaml
generated_by: clan merge v1.1.0
conflicts:
  - key: status
    winner: { value: "needs-review", agent: analyst, policy: last-write }
    losers:
      - { value: "approved", agent: researcher }
    # Optional: present when the shape suggests a better policy. Prose-valued
    # keys every branch wrote independently are usually complementary, so the
    # merge suggests `append` rather than silently dropping the losers.
    suggestion: "all 2 branches wrote prose for 'status'; re-run with `--policy status=append` to keep every branch's text"
unresolved: 1   # decremented as adjudications land (Section 25)
```

Conflicts are **data, not failures**: `clan merge` exits 0 with conflicts present (non-zero only on structural errors), reports the contested-key count, and hints adjudication (Section 27). `clan read report` prints the report TOON-encoded.

---

## 25. Conflict Adjudication

**Invariant (normative): `shared/data.yaml` is never ambiguous.** Merge always produces exactly one value per key. No inline conflict markers, no multi-values — the UI bindings (`{{key}}`) and downstream injection both depend on this. Losing values survive in branch namespaces and `merge-report.yaml`.

Adjudication protocol — identical for humans and agents:
1. Read `merge-report.yaml` (injected automatically while `unresolved > 0`, Section 26).
2. Choose a value → `clan patch-data` with the chosen value.
3. Record it → `clan patch-decision --agent <id> --action "adjudicated <key>" --rationale "..."`.
4. The write removes the key from `merge-report.yaml` and decrements `unresolved`.

Viewer behaviour: elements whose binding key appears in `merge-report.yaml` SHOULD be badged (contested-value indicator) surfacing the competing values with agent provenance; the human's pick runs the same protocol via the existing patch bridge. Conflicts are rendered, not erased.

---

## 26. Conditional Context Injection

The base agent guide (`spec/agent-guide.md`) is **byte-stable** across all files and hops — never edited per file (prompt-cache-friendly). Situational knowledge is appended *after* it as small blocks, selected by **file state**, so an agent never reads about a capability its current file doesn't exhibit:

| Condition (from manifest / members) | Injected block |
|---|---|
| `fork` block present | Branch mode: agent id, namespace, the two allowed write commands |
| `merge-report.yaml` with `unresolved > 0` | The report (TOON) + the 4-step adjudication protocol (Section 25) |
| `view: {present: false, renderable: true}` | One-line note: no view exists; `clan render` materialises one if the task requires it |
| `agent/requirements.yaml` present | The requirements member (TOON) |
| none of the above | nothing |

A sequential agent on an unforked, conflict-free file receives exactly the v1.0 injection — zero added cost for the common case.

---

## 27. Teachable Interface

**Principle: every command teaches the next step; nothing teaches ahead of need.** An agent learns the CLAN protocol through the outputs of the commands it actually runs. An agent that never forks never hears about merging.

### 27.1 Hint lines

Every CLI command MAY append `next:` hint lines to its **diagnostic stream (stderr)** after its primary output — stdout stays pure data, so piped consumers are never polluted:

```
next: 2 contested key(s) in merge-report.yaml — `clan read report merged.clan`
next: adjudicate each: clan patch-data <file> <json>, then clan patch-decision ...
```

Normative properties:
- **Deterministic.** `hint = f(command, result, file state)` — same inputs, same hints. Testable.
- **Bounded.** At most 3 lines per command. Hints are headlines, not documentation; each names the command that gives the detail.
- **Stable prefix.** Every hint line starts `next:` (machine-strippable). `--quiet` or `CLAN_NO_HINTS=1` suppresses all hints.
- **Precondition-gated.** A hint MUST NOT mention a capability whose preconditions are absent from the current file state: no merge hints on unforked files, no adjudication hints when `unresolved == 0`, no render hints when `present: true, stale: false`.

### 27.2 Errors teach

Every guardrail rejection MUST name the correct alternative in the same message:

```
error: this file is forked for agent 'researcher' — a direct write to shared/data.yaml
       is not allowed during a parallel interval.
next:  write your data with `clan patch-data --namespace` (routes to agents/researcher/data.yaml)
next:  record decisions with `clan patch-decision` (auto-routed)
next:  when all branches are done, join them with `clan merge`
```

For agents, the error message *is* the documentation: conventions that rely on agents remembering prose fail; commands that correct on contact succeed.

### 27.3 Consequence for the base guide

As the hint map covers protocol mechanics at point of use, the byte-stable base guide SHOULD shrink over minor versions toward its irreducible core: what you received, what you must return, the schema contract. Everything procedural moves to the channel where it is actionable — the command output.
