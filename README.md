# LACE — Living Agent Context Envelope

> Pronounced **"lace"** — the thread that weaves agent context and human view into one artifact.

LACE is an open container format for passing structured context between AI agents and rendering it for humans. Every `.lace` file is simultaneously machine-readable for agents and visually renderable for humans — with no duplication of data between those two representations.

---

## What Problem It Solves

Multi-agent AI systems have no standard for what passes between agents. Context gets lost, decisions are untracked, outputs are unstructured, and humans have no readable window into what happened. LACE is the artifact that travels through the pipeline — carrying structured data, provenance, and a rich human view in a single file.

---

## Key Properties

| Property | Description |
|---|---|
| **Self-describing** | Every LACE contains its own spec. Any agent can understand and produce LACE without prior training. |
| **Dual-audience** | Agents read structured YAML/JSON. Humans see rendered HTML. Same data, no duplication. |
| **Living lineage** | Every LACE references its parent. The full document history is reconstructable. |
| **Open** | Format spec and SDK are Apache 2.0. Anyone can implement LACE. |
| **Compact** | Typical LACE file: 10–60KB. Rich document with charts: 100–500KB. |

---

## File Structure

```
my-document.lace          ← ZIP container (DEFLATE)
├── manifest.yaml         ← index, lineage, version, file registry
├── spec/
│   ├── lace.md            ← full specification (travels with every file)
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
| [LACE-SPEC.md](LACE-SPEC.md) | Full format specification |
| [ARCHITECTURE.html](ARCHITECTURE.html) | System architecture diagrams |
| [SEQUENCE-DIAGRAMS.md](SEQUENCE-DIAGRAMS.md) | All key interaction flows |
| [LICENSE-GUIDANCE.md](LICENSE-GUIDANCE.md) | Licensing strategy |
| [PATENT-GUIDANCE.md](PATENT-GUIDANCE.md) | Defensive publication and patent strategy |
| [spec/lace.md](spec/lace.md) | Embedded spec (travels inside every .lace file) |
| [spec/agent-guide.md](spec/agent-guide.md) | Agent injection guide (travels inside every .lace file) |

---

## Name

**LACE** — **L**iving **A**gent **C**ontext **E**nvelope

A LACE file is alive: it carries its own specification, so any agent can understand it without prior training. It is an envelope: a container passed along a pipeline, with a return address (lineage) and a clear recipient (agent or human). Like lacing threads together, it weaves structured agent data and rich human rendering into a single artifact.

---

## Status

Pre-release specification. Version 1.0 draft.
