# LACE Specification — Embedded Reference

**Living Agent Context Envelope** | Version 1.0 | Pronounced "lace"

This file travels inside every `.lace` container. It is the authoritative reference for the format version declared in `manifest.yaml`. If the public specification and this file conflict, this file governs for this specific LACE file.

---

## What LACE Is

LACE is an open container format (ZIP) for passing structured context between AI agents and rendering that context for humans. Every `.lace` file contains:

- **`shared/data.yaml`** — typed structured facts (read by both agents and humans)
- **`agent/`** — machine-readable task, schema, state, and decision history
- **`human/`** — rich HTML rendering, CSS, assets, and human edit patches
- **`spec/`** — this specification and the agent injection guide

---

## Container

- Format: ZIP (DEFLATE compression)
- Extension: `.lace`
- MIME: `application/vnd.lace`
- Encoding: UTF-8 for all text files

---

## Required Files

```
manifest.yaml
spec/lace.md                    ← this file
spec/agent-guide.md
shared/data.yaml
agent/context.md
agent/output-schema.json
agent/state.yaml
agent/decision-chain.yaml
```

---

## manifest.yaml

Root index of the container. First entry in the ZIP. Required fields:

```yaml
lace_version: 1
lace_version_minor: 0
id: "uuid-v4"
title: "Document Title"
created_at: "2026-05-29T10:00:00Z"
updated_at: "2026-05-29T14:00:00Z"

# Optional
document_type: "invoice"
lineage:
  parent_id: "uuid-v4"
  parent_uri: "file:///path/to/parent.lace"
  delta: "What changed from parent"
external:
  - id: "store-id"
    uri: "mcp://..."
    type: "mcp-resource"       # mcp-resource | vector-store | s3 | custom
    description: "..."
    access: "read"             # read | write | read-write
files:
  - id: "stable-id"
    path: "shared/data.yaml"
    role: "canonical-data"
    type: "application/yaml"
```

---

## shared/data.yaml

Canonical facts. Read by agents as structured data. Injected into human HTML rendering as `window.__LACE__.data`. Never duplicated in agent/ or human/ files.

Must begin with `$schema` declaration:
```yaml
$schema: "spec/schemas/document-type.schema.json"
# data fields follow
```

---

## Agent Injection Serialisation

When the SDK assembles context for an agent, `shared/data.yaml` and `agent/decision-chain.yaml` are serialised as **TOON (Token-Oriented Object Notation)** — approximately 40% fewer tokens than equivalent JSON or YAML. Agent output is always returned as JSON and validated against `agent/output-schema.json` before packaging.

---

## agent/ Directory

### context.md
What the current agent should do. Plain Markdown. Task-specific rules here override agent-guide.md defaults.

### output-schema.json
JSON Schema (Draft 7+) defining exactly what the agent must return. The SDK validates output before packaging. Required field: `mode` (one of `data-update`, `designed`, `full-html`).

### state.yaml
Current agent-readable document state. Pipeline stage, status, confidence, readiness flags.

### decision-chain.yaml
Ordered agent decision log. Newest first. SDK applies tiered compression:
- Entries 1–3: full fidelity
- Entries 4–15: key facts + rationale
- Entry 16+: 2-sentence summary

Entry structure:
```yaml
- agent: "agent-name"
  action: "what it did"
  rationale: "why"
  timestamp: "ISO 8601"
  trace-ref:                     # optional — points to external full context
    store: "external-id"
    entry: "step/N"
    content-hash: "sha256:..."
```

---

## human/ Directory

### index.html
HTML **fragment** only — no `<html>`, `<head>`, `<body>` tags. No `<script>` tags. No external URL references. Agent-generated. Never directly modified by human edits.

Data binding: use `{{key}}` syntax referencing `shared/data.yaml`. Resolved at render time.
Editable elements: assign `data-adf-id="stable-id"` to all human-editable text nodes.

### index.txt
Plain text fallback. Auto-generated from index.html.

### styles.css
Agent-generated. Scoped to document content. No external imports.

### patches.yaml
Human text edits stored out-of-band. Applied over the HTML at render time.
```yaml
patches:
  - id: "data-adf-id-value"
    content: "New text content"
    edited_at: "ISO 8601"
    edited_by: "human"
```

### assets/
SVG, PNG, JPEG, WebP, WOFF2. Referenced via relative paths in index.html.

---

## Output Modes

Agents return a JSON object matching output-schema.json. The `mode` field controls SDK behaviour.

### data-update
```json
{ "mode": "data-update", "structured": { "...": "..." } }
```
SDK updates shared/data.yaml. Human view re-renders via data binding. HTML design preserved.

### designed
```json
{
  "mode": "designed",
  "structured": { "...": "..." },
  "design": {
    "theme": "dark-minimal",
    "accent_color": "#6366f1",
    "layout": "card-grid",
    "highlight_fields": ["field1", "field2"],
    "custom_css": "..."
  }
}
```
SDK generates HTML from directives. Themes: `light-clean`, `dark-minimal`, `warm-document`, `high-contrast`.

### full-html
```json
{
  "mode": "full-html",
  "structured": { "...": "..." },
  "human": {
    "html": "<section>...</section>",
    "css": "...",
    "assets": { "chart.svg": "<svg>...</svg>" }
  }
}
```
Agent provides complete HTML. SDK sanitises (no scripts, no events) and packages.

---

## Security Rules

- No `<script>` tags in human/index.html
- No `on*` event handler attributes in HTML
- No external URL references in CSS `url()`
- No `<iframe>`, `<object>`, `<embed>`, `<form>` in HTML
- Patch content is plain text — no HTML in patch values

---

## Versioning

- `lace_version` — major (breaking changes). Readers reject files with higher major version.
- `lace_version_minor` — minor (backwards compatible). Readers accept higher minor version.

---

## Licence

LACE Specification — Apache License 2.0
