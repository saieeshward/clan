# CLAN — Context and Live Agent Notation

> Pronounced **"clan"** — like a clan, every file carries shared lineage connecting it to every document that came before it.

CLAN is an open container format for passing structured context between AI agents and rendering it for humans. Every `.clan` file is simultaneously machine-readable for agents and visually renderable for humans — with no duplication of data between those two representations.

---

## What Problem It Solves

Multi-agent AI systems have no standard for what passes between agents. Context gets lost, decisions are untracked, outputs are unstructured, and humans have no readable window into what happened. CLAN is the artifact that travels through the pipeline — carrying structured data, provenance, and a rich human view in a single file.

---

## Key Properties

| Property | Description |
|---|---|
| **Self-describing** | Every CLAN contains its own spec. Any agent can understand and produce CLAN without prior training. |
| **Dual-audience** | Agents read structured YAML/JSON. Humans see rendered HTML. Same data, no duplication. |
| **Living lineage** | Every CLAN references its parent. The full document history is reconstructable. |
| **Open** | Format spec and SDK are Apache 2.0. Anyone can implement CLAN. |
| **Compact** | Typical CLAN file: 10–60KB. Rich document with charts: 100–500KB. |

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
│   └── decision-chain.yaml ← provenance record of all agent decisions
└── human/
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
| [CLAN-SPEC.md](CLAN-SPEC.md) | Full format specification |
| [ARCHITECTURE.html](ARCHITECTURE.html) | System architecture diagrams |
| [SEQUENCE-DIAGRAMS.md](SEQUENCE-DIAGRAMS.md) | All key interaction flows |
| [LICENSE-GUIDANCE.md](LICENSE-GUIDANCE.md) | Licensing strategy |
| [PATENT-GUIDANCE.md](PATENT-GUIDANCE.md) | Defensive publication and patent strategy |
| [spec/clan.md](spec/clan.md) | Embedded spec (travels inside every .clan file) |
| [spec/agent-guide.md](spec/agent-guide.md) | Agent injection guide (travels inside every .clan file) |

---

## Name

**CLAN** — **C**ontext and **L**ive **A**gent **N**otation

A CLAN file is live: it carries its own specification, so any agent can understand and produce it without prior training. It carries context: structured data, task state, and the full decision history of every agent that touched it.

---

## Status

Pre-release specification. Version 1.0 draft.
