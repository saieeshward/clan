<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="design/assets/clan-lockup-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="design/assets/clan-lockup-light.svg">
    <img src="design/assets/clan-lockup-light.svg" height="72" alt="CLAN logo" />
  </picture>
</p>

# CLAN — Context and Live Agent Notation

> Pronounced **"clan"** — like a clan, every file carries shared lineage connecting it to every document that came before it.

CLAN is an open container format for passing structured context between AI agents and rendering it for humans. Every `.clan` file is simultaneously machine-readable for agents and visually renderable for humans — with no duplication of data between those two representations.

---

## What Problem It Solves

Multi-agent AI systems have no standard for what passes between agents. Context gets lost, decisions are untracked, outputs are unstructured, and humans have no readable window into what happened. CLAN is the artifact that travels through the pipeline — carrying structured data, provenance, and a rich human view in a single file.

---

## Why a CLI?

You actually touched on the exact reason why the `clan` CLI exists! To understand why you can't just upload a `.clan` file directly to a standard LLM chat interface right now, it helps to look at how platforms handle files like `.docx` or `.pdf`.

### How LLMs handle `.docx` and `.pdf` files
When you upload a `.docx` file to ChatGPT, Claude, or Gemini, the AI itself isn't actually reading the Word document file format. A `.docx` file is technically a ZIP archive full of complex XML files and media. 

Instead, the platform's backend (the web app) intercepts your upload, runs a script to extract the raw text out of those XML files, formats it nicely, and secretly pastes that plain text into your chat prompt behind the scenes. The LLM only ever sees the extracted plain text, never the actual `.docx` file.

### Why `.clan` files are different (for now)
A `.clan` file is also a packaged archive containing multiple files (Markdown, YAML data, JSON schemas, HTML UIs). 

The problem is that standard LLM web interfaces don't have a built-in "extractor" for `.clan` files yet. If you drag and drop a `.clan` file into a standard chat window, the platform doesn't know how to unzip it, figure out which file is the context vs. the decision history, and format it for the AI. It will likely just throw an "unsupported file format" error or see binary garbage.

### The Role of the CLI
Because the chat interfaces don't know how to parse a `.clan` file natively, the **`clan` CLI acts as the translator**. 

When we run `clan read agent file.clan`, the CLI is doing exactly what the ChatGPT backend does for a Word document: it opens the archive, extracts the `context.md`, the `decision-chain.yaml`, and the `data.yaml`, compresses them to save tokens, and turns them into a single, highly-optimized text prompt that the LLM can easily read.

### The Future of CLAN Uploads
In the future, if a platform natively supports the CLAN format (or if you are using an agentic framework that has the CLAN SDK built-in), **you absolutely will be able to just upload it directly**. The framework would intercept the file, unpack it, and feed the context to the AI automatically, completely hiding the CLI from both you and the AI!

---

## Key Properties

| Property | Description |
|---|---|
| **Self-describing** | Every CLAN contains its own spec. Any agent can understand and produce CLAN without prior training. |
| **Dual-audience** | Agents read structured YAML/JSON. Humans see rendered HTML. Same data, no duplication. |
| **Living lineage** | Every CLAN references its parent. The full document history is reconstructable. |
| **Open** | Format spec and SDK are licensed under Mozilla Public License 2.0. Anyone can implement CLAN. |
| **Compact** | Typical CLAN file: 10–60KB. Rich document with charts: 100–500KB. |

---

## Maintainers

Maintained and owned by Sai Eeshwar (https://github.com/saieeshward) and Shreyansh Soni (https://github.com/batunii).

---

## File Structure

```
my-document.clan          ← ZIP container (DEFLATE)
├── manifest.yaml         ← index, lineage, version, file registry
├── spec/
│   ├── clan.md            ← full specification (travels with every file)
│   └── agent-guide.md    ← compressed guide injected into agent context
├── shared/
│   └── data.yaml         ← canonical facts (read by both agents and humans)
├── agent/
│   ├── context.md        ← task description for current agent
│   ├── output-schema.json← what the agent must produce
│   ├── state.yaml        ← current document state
│   ├── decision-chain.yaml ← provenance record of all agent decisions
│   └── requirements.yaml ← optional: declared tool/capability needs (v1.1)
├── agents/               ← optional: per-agent namespaces during parallel work (v1.1)
│   └── <agent-id>/       ← branch writes: data.yaml + decisions.yaml
├── merge-report.yaml     ← optional: contested keys from the last merge (v1.1)
└── human/                ← optional: derivable on demand via `clan render` (v1.1)
    ├── index.html        ← agent-generated HTML fragment (rich rendering)
    ├── index.txt         ← plain text fallback
    ├── styles.css        ← agent-generated styles
    ├── patches.yaml      ← human text edits (applied at render time)
    └── assets/           ← SVG charts, images, fonts
```

---

## Documents

| Document | Description |
|---|---|
| [spec/CLAN-SPEC.md](spec/CLAN-SPEC.md) | Full format specification |
| [spec/SEQUENCE-DIAGRAMS.md](spec/SEQUENCE-DIAGRAMS.md) | All key interaction flows |
| [spec/clan.md](spec/clan.md) | Embedded spec (travels inside every .clan file) |
| [spec/agent-guide.md](spec/agent-guide.md) | Agent injection guide (travels inside every .clan file) |
| [design/](design/) | Logo assets and design handoff files |

---

## Name

**CLAN** — **C**ontext and **L**ive **A**gent **N**otation

A CLAN file is live: it carries its own specification, so any agent can understand and produce it without prior training. It carries context: structured data, task state, and the full decision history of every agent that touched it.

---

## Status

Pre-release specification. Version 1.1 draft — adds fork/join concurrency (per-agent namespaces + deterministic merge), deferred human-view rendering, conflict adjudication, and a teachable CLI interface (spec §22–§27).
