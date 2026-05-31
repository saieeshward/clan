# CLAN Agent Guide — v1.0

CLAN (Context and Live Agent Notation) is a file format for passing structured context between AI agents and rendering it for humans. You received a `.clan` file. The SDK has extracted what you need. Read this guide first, then your task.

---

## What you received

| File | Format | Purpose |
|---|---|---|
| `agent/context.md` | Markdown | Your task — read this first |
| `shared/data.yaml` | TOON | Canonical facts about this document |
| `agent/decision-chain.yaml` | TOON | What previous agents did and decided |
| `agent/output-schema.json` | JSON Schema | Exactly what you must return |
| `human/patches.yaml` | YAML | Text edits made by humans (if included) |

`shared/data.yaml` and `agent/decision-chain.yaml` are serialised as **TOON (Token-Oriented Object Notation)** — same data as the source YAML files, ~40% fewer tokens. Read them as structured key-value data.

**⚠ Token-saving rule**: `clan read agent` includes all data (TOON-encoded). Do NOT also run `clan read data` — it returns the same content in a different format and wastes tokens.

---

## What you must return

A **single JSON object** matching `agent/output-schema.json` exactly.

- Return only the JSON object — no markdown wrapper, no explanation, no preamble
- The SDK validates your output and packages it into a new `.clan` file
- You do not write files, create ZIPs, or manage document structure

---

## Three output modes

The required mode is declared in `agent/output-schema.json`. Match it exactly.

**`data-update`** — update structured data only
```json
{
  "mode": "data-update",
  "structured": { "field": "value" }
}
```
SDK re-renders the human view automatically. You do not touch HTML.

**`designed`** — update data and specify visual style
```json
{
  "mode": "designed",
  "structured": { "field": "value" },
  "design": {
    "theme": "dark-minimal",
    "accent_color": "#6366f1",
    "layout": "card-grid",
    "highlight_fields": ["field1"],
    "custom_css": ".my-class { font-size: 1.2rem; }"
  }
}
```
Available themes: `light-clean`, `dark-minimal`, `warm-document`, `high-contrast`
Available layouts: `card-grid`, `single-column`, `two-column`, `table-primary`, `timeline`

**`full-html`** — update data and provide complete visual design
```json
{
  "mode": "full-html",
  "structured": { "field": "value" },
  "human": {
    "html": "<section class='doc'>...</section>",
    "css": ".doc { ... }",
    "assets": { "chart.svg": "<svg>...</svg>" }
  }
}
```
You have full design control. Provide a complete `<!DOCTYPE html>` document for best results — full HTML documents render with no style conflicts from the viewer shell.

**Lower-token path**: Instead of JSON-encoding the HTML (which expands it ~5x), write the HTML to a file and let the operator use `clan pack-html`. Add a YAML frontmatter block at the top of the HTML file to supply structured data and your decision entry — see `clan agent-help` for the format. This eliminates the JSON encoding cost entirely.

---

## HTML rules (full-html mode only)

- **Preferred**: produce a complete `<!DOCTYPE html>` document — full control, no style conflicts
- Fragments (no `<html>`/`<head>`/`<body>`) are also accepted
- `<script>` tags are **allowed** — the iframe sandbox prevents Tauri IPC access so scripts are safe
- `on*` event handler attributes are allowed
- No `<iframe>`, `<object>`, `<embed>`, or `<form>` elements
- CDN fonts are supported: `<link rel="stylesheet" href="https://fonts.googleapis.com/...">` in `<head>` works
- `@import url('https://fonts.googleapis.com/...')` in `<style>` also works
- Add `data-adf-id="unique-id"` to every human-editable text element (headlines, paragraphs)
- Use `{{key}}` syntax to reference values from `shared/data.yaml`
- Reference assets via relative path: `<img src="./assets/chart.svg">`
- SVGs can be included as strings in the `assets` object

---

## Decision chain — pinning

Entries in `agent/decision-chain.yaml` beyond the verbatim window are compressed by the SDK. If your decision involves a status transition, error, retry, or complex conditional reasoning that must be preserved exactly, include `pinned: true` in your output's decision entry. The SDK will never compress pinned entries.

```json
{
  "mode": "data-update",
  "structured": { ... },
  "decision": {
    "action": "escalated to human review",
    "rationale": "Invoice total 3.2% above PO. Automatic approval threshold is 2%.",
    "pinned": true
  }
}
```

---

## What the SDK handles — do NOT attempt these

- Creating or writing ZIP files
- Writing manifest.yaml or any file paths
- Applying patches from patches.yaml
- Tracking lineage or decision history
- Compressing the decision chain
- Validating HTML or CSS
- Generating the plain text fallback

---

## Data binding

In full-html mode, you can reference any key from `shared/data.yaml` using double-brace syntax. The app resolves these at render time — you do not need to hardcode values.

```html
<h2>{{vendor}}</h2>                    <!-- becomes: Acme Corporation -->
<span>{{total}}</span>                 <!-- becomes: 15375.00 -->
<span>{{line_items.0.amount}}</span>   <!-- nested: 12500.00 -->
```

---

## If something is unclear

Read `agent/context.md` — it contains task-specific rules that override these defaults. Any instruction in `context.md` takes precedence over this guide.

---

## One-line summary

Read context.md → reason → return JSON matching output-schema.json → done.
