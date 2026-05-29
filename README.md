# XON — eXchange Object Notation

> Pronounced **"axon"** — like the nerve fibre that transmits signals between neurons.

XON is an open container format for passing structured context between AI agents and rendering it for humans. Every `.xon` file is simultaneously machine-readable for agents and visually renderable for humans — with no duplication of data between those two representations.

---

## What Problem It Solves

Multi-agent AI systems have no standard for what passes between agents. Context gets lost, decisions are untracked, outputs are unstructured, and humans have no readable window into what happened. XON is the artifact that travels through the pipeline — carrying structured data, provenance, and a rich human view in a single file.

---

## Key Properties

| Property | Description |
|---|---|
| **Self-describing** | Every XON contains its own spec. Any agent can understand and produce XON without prior training. |
| **Dual-audience** | Agents read structured YAML/JSON. Humans see rendered HTML. Same data, no duplication. |
| **Living lineage** | Every XON references its parent. The full document history is reconstructable. |
| **Open** | Format spec and SDK are Apache 2.0. Anyone can implement XON. |
| **Compact** | Typical XON file: 10–60KB. Rich document with charts: 100–500KB. |

---

## File Structure

```
my-document.xon          ← ZIP container (DEFLATE)
├── manifest.yaml         ← index, lineage, version, file registry
├── spec/
│   ├── xon.md            ← full specification (travels with every file)
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
| [XON-SPEC.md](XON-SPEC.md) | Full format specification |
| [ARCHITECTURE.html](ARCHITECTURE.html) | System architecture diagrams |
| [SEQUENCE-DIAGRAMS.md](SEQUENCE-DIAGRAMS.md) | All key interaction flows |
| [LICENSE-GUIDANCE.md](LICENSE-GUIDANCE.md) | Licensing strategy |
| [PATENT-GUIDANCE.md](PATENT-GUIDANCE.md) | Defensive publication and patent strategy |
| [spec/xon.md](spec/xon.md) | Embedded spec (travels inside every .xon file) |
| [spec/agent-guide.md](spec/agent-guide.md) | Agent injection guide (travels inside every .xon file) |

---

## Name

**XON** — **e**X**c**hange **O**bject **N**otation

Like JSON (JavaScript Object Notation) is a notation for JavaScript objects, XON is a notation for objects that are exchanged — between agents, between systems, and between machines and humans.

The pronunciation "axon" is intentional. An axon is the nerve fibre that carries signals between neurons. Agents are the neurons. XON files are the axons.

---

## Status

Pre-release specification. Version 1.0 draft.
