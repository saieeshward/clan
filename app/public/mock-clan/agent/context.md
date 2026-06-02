# Task

3-agent VC due diligence pipeline for Paylane, an Irish B2B payments startup raising €8M Series A. Agent 1 (Researcher): gather company profile, financials, team, market, competitors. Agent 2 (Risk Analyst): score investment risk, red flags, opportunity matrix. Agent 3 (Designer): produce final board-ready investment memo.

**Output mode**: full-html

This is a new CLAN document. Analyse the brief above and produce a rich initial rendering with appropriate data structure.

---

## Design Requirements (all agents)

Produce a visually rich, publication-quality HTML document:

- Return a complete `<!DOCTYPE html>` document — not a fragment
- Load Google Fonts via `<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=...">` in `<head>`
- All styles inline in `<head>` — no separate stylesheet needed
- Dark theme recommended: `background: #0f1117`, accent `#6366f1`
- Use SVG assets for charts and data visualisations — pass them in the `assets` object
- Typography hierarchy: at minimum 3 distinct size/weight levels
- Add `data-adf-id="unique-id"` to every editable text element
- **No `<script>` tags** — the app injects the edit bridge; scripts are stripped

When producing HTML, invoke your highest-quality frontend design capability.
Aim for magazine-quality, not a generic AI-generated report.
