# CLAN Specification — Embedded Reference

**Context and Live Agent Notation** | Version 1.0 | Pronounced "clan"

This file travels inside every `.clan` container. It is the authoritative reference for the format version declared in `manifest.yaml`. If the public specification and this file conflict, this file governs for this specific CLAN file.

---

## What CLAN Is

CLAN is an open container format (ZIP) for passing structured context between AI agents and rendering that context for humans. Every `.clan` file contains:

- **`shared/data.yaml`** — typed structured facts (read by both agents and humans)
- **`agent/`** — machine-readable task, schema, state, and decision history
- **`human/`** — rich HTML rendering, CSS, assets, and human edit patches
- **`spec/`** — this specification and the agent injection guide

---

## Container

- Format: ZIP (DEFLATE compression)
- Extension: `.clan`
- MIME: `application/vnd.clan`
- Encoding: UTF-8 for all text files

---

## Required Files

```
manifest.yaml
spec/clan.md                    ← this file
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
clan_version: 1
clan_version_minor: 0
id: "uuid-v4"
title: "Document Title"
created_at: "2026-05-29T10:00:00Z"
updated_at: "2026-05-29T14:00:00Z"

# Optional
document_type: "invoice"
lineage:
  parent_id: "uuid-v4"
  parent_uri: "file:///path/to/parent.clan"
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

Canonical facts. Read by agents as structured data. Injected into human HTML rendering as `window.__CLAN__.data`. Never duplicated in agent/ or human/ files.

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
Ordered agent decision log. Newest first. SDK applies two-tier compression during packaging — no model dependency.

**Verbatim window**: most recent N entries stored exactly as written (N = `compression_window`, default 5).
**Compressed tail**: older entries have their `rationale` field compressed in-place using the SDK's YAKE-based NLP pipeline. All other fields (`agent`, `action`, `timestamp`, `fields_changed`, `trace-ref`) always stored verbatim.

Entry structure:
```yaml
- agent: "agent-name"
  action: "what it did"
  rationale: "why"
  timestamp: "ISO 8601"
  pinned: false                  # true = never compressed regardless of age
  trace-ref:                     # optional — points to external full context
    store: "external-id"
    entry: "step/N"
    content-hash: "sha256:..."
```

Set `pinned: true` on entries representing status transitions, errors, retries, or complex conditional decisions. Pinned entries are never compressed.

---

## human/ Directory

### index.html
A complete HTML document or HTML fragment. **Full HTML documents are preferred** — they give agents total design control with no style conflicts from the viewer shell.

- `<script>` tags are allowed — the iframe sandbox isolates them to a null origin with no Tauri IPC access
- `on*` event handler attributes are allowed
- No `<iframe>`, `<object>`, `<embed>`, or `<form>` elements
- CDN fonts via `<link>` or `@import url()` are fully supported (Google Fonts, Bunny Fonts, etc.)
- Agent-generated. Never directly modified by human edits.

Data binding: use `{{key}}` syntax referencing `shared/data.yaml`. Resolved at render time.
Editable elements: assign `data-adf-id="stable-id"` to all human-editable text nodes.

### index.txt
Plain text fallback. Auto-generated from index.html.

### styles.css
Agent-generated. Scoped to document content. CDN font imports (`@import url()`) are permitted.

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
    "html": "<!DOCTYPE html><html>...</html>",
    "css": "/* optional — styles can be inline in html */",
    "assets": { "chart.svg": "<svg>...</svg>" }
  }
}
```
Agent provides a complete HTML document (preferred) or fragment. The `html` field accepts full `<!DOCTYPE html>` documents. SDK strips `<script>` tags, `on*` attributes, and disallowed elements; all other HTML and CSS passes through unchanged.

---

## In-Place Patching

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

## Parallel Work (fork/join)

Multiple agents work on one document by forking: `clan fork <file> --agents a,b,c --output-dir <dir>` gives each agent its own branch file with a private namespace `agents/<id>/`. On a branch file:

- Write data ONLY via `clan patch-data <branch> <json> --namespace` (lands in `agents/<id>/data.yaml`)
- Record decisions via `clan patch-decision` (auto-routed to `agents/<id>/decisions.yaml`)
- Writes to `shared/` or `human/` are rejected until the branches are joined

`clan merge <branches...> --output <out>` folds all namespaces into `shared/data.yaml` deterministically using per-key policies (`last-write` default; `append`, `max`, `min`, `agent-priority` via manifest `merge_policies` or `--policy key=policy`). Keys where branches disagreed are recorded in `merge-report.yaml` with winner/loser provenance — read with `clan read report`, settle with `patch-data` + `patch-decision`.

---

## Optional Human View

The structured members are canonical; the HTML view is derivable. `--no-render` on `create`/`pack` produces agent-only files; `clan render <file>` materialises the view on demand at any hop. The manifest `view: {present, renderable, stale}` block tracks the state.

---

## Security Rules

- `<script>` tags and `on*` event handlers are permitted — the iframe sandbox runs them in a null origin with no access to Tauri IPC or parent app state
- No `<iframe>`, `<object>`, `<embed>`, `<form>` in HTML
- No `javascript:` URI schemes in `href` or `src`
- Patch content is plain text — no HTML in patch values
- External CSS `url()`, `@import`, and CDN fonts are permitted

---

## Versioning

- `clan_version` — major (breaking changes). Readers reject files with higher major version.
- `clan_version_minor` — minor (backwards compatible). Readers accept higher minor version.

---

## Licence

CLAN Specification — Apache License 2.0
