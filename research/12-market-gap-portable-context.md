# Market & Research Gap — Portable Agent-Context Artifact

**Question investigated:** Is there a genuine, unoccupied gap for a *portable agent-context file* — a framework-agnostic, file-based, token-efficient, provenance-carrying artifact for passing structured context between AI agents (including agent-to-agent handoff with no human in the loop)?

**Method:** Deep multi-source pass — 24 sources fetched, 116 falsifiable claims extracted, 25 adversarially verified (3-vote, need 2/3 to kill). 24 confirmed, 1 refuted. Prioritised primary sources (framework docs, protocol specs, official blogs), 2024–2026.

**Verdict:** The gap is **real but narrower and more contested** than a purely structural argument suggests. No unified portable + token-efficient + provenance-carrying inter-agent context capsule exists as a product or standard today; the building blocks exist *separately*. The conclusion rests substantially on **absence of evidence** (an unindexed/proprietary format could exist).

---

## Confirmed findings (high confidence unless noted)

### 1. Frameworks keep inter-agent context as runtime/storage state, not portable files
LangGraph persists state via checkpointers backed by databases (InMemory, SQLite, Postgres, Cosmos DB) conforming to `BaseCheckpointSaver`, inspectable only through its runtime API (`get_state`/`get_state_history`). Checkpoints are serialised internally but there is **no documented framework-agnostic standalone-file export**. ("not portable" = no documented portable export, not "never serialised".)
- Sources: [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence), [checkpoints reference](https://reference.langchain.com/python/langgraph/checkpoints) — *primary*

### 2. Anthropic's subagent handoff is an in-context summary, not a persisted format
Each subagent "returns only a condensed, distilled summary of its work (often 1,000–2,000 tokens)." Anthropic discusses architectural patterns but **does not define a portable inter-agent context format.**
- Source: [Effective context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) — *primary*

### 3. MCP is agent↔tools, not an agent↔agent context artifact
MCP is "a JSON-RPC client-server interface for secure context ingestion and structured tool invocation" — "a USB-C port for AI applications." Not a persisted bidirectional handoff artifact.
- Sources: [MCP intro](https://modelcontextprotocol.io/introduction), [Anthropic MCP](https://www.anthropic.com/news/model-context-protocol), [arXiv 2505.02279](https://arxiv.org/pdf/2505.02279) — *primary*

### 4. A2A / ACP / ANP / AGNTCY are runtime plumbing, not portable context files
A2A is an open standard for **runtime** agent-to-agent messaging (Messages, stateful Tasks, Artifacts over JSON-RPC/gRPC/HTTP+SSE) with capability-advertising Agent Cards — message persistence in Task history is **not even guaranteed**. MCP and A2A are positioned as complementary, non-overlapping. AGNTCY (Linux Foundation, donated July 2025; Cisco/Dell/Google Cloud/Oracle/Red Hat) is discovery/identity/messaging/observability infrastructure. **None defines a persisted, framework-agnostic context-handoff file.** (ACP merged into A2A under the LF in late 2025.)
- Sources: [A2A spec](https://a2a-protocol.org/latest/specification/), [A2A×MCP](https://a2a-protocol.org/latest/topics/a2a-and-mcp/), [AGNTCY docs](https://docs.agntcy.org/), [arXiv 2505.02279](https://arxiv.org/pdf/2505.02279) — *primary*

### 5. A token-efficient serialization standard already exists publicly — TOON
TOON (Token-Oriented Object Notation) is a compact, lossless representation of the JSON data model for LLM input: "76.4% accuracy (vs JSON's 75.0%) while using 39.9% fewer tokens." Real spec-driven artifact: `.toon` extension, provisional `text/toon` media type, Working Draft v3.3, academic benchmark. Gains are **dataset-dependent** (best on uniform arrays; plain JSON can win on deeply nested/flat/short data; ~5% header overhead).
- Sources: [toon-format/toon](https://github.com/toon-format/toon), [spec](https://github.com/toon-format/spec), [toonformat.dev](https://toonformat.dev), [arXiv 2603.03306](https://arxiv.org/abs/2603.03306) — *primary*
- **⚠ Action for CLAN:** CLAN's spec §14 "TOON" is the same concept as this public standard. Reconcile: confirm spec-compatibility (→ free interop) or resolve the naming collision.

### 6. Linearly-growing token cost is a named, recognised problem with shipping mitigations
Anthropic names "attention budget" / context rot ("as tokens increase, the model's ability to accurately recall... decreases"). Microsoft ships a dedicated (experimental) compaction framework citing token limits / cost / latency, with composable strategies: Truncation, SlidingWindow, ToolResultCompaction, Summarization.
- Sources: [Anthropic](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents), [Microsoft compaction](https://learn.microsoft.com/en-us/agent-framework/agents/conversations/compaction) — *primary*

### 7. File-based "agentic memory / note-taking" is officially endorsed — but format-unspecified
"Structured note-taking... where the agent regularly writes notes persisted to memory outside of the context window," pulled back later (Claude Code to-do lists, `NOTES.md`; memory tool shipped with Sonnet 4.5). **No portable standard format is defined** — this is the open slot.
- Source: [Anthropic](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) — *primary*

### 8. Provenance/lineage as a portable artifact is UNDER-DETERMINED (not a proven gap)
The claim "multi-agent decision-chain lineage is an unsolved problem" was **refuted (0-3)**. No product/standard baking provenance into a portable artifact surfaced either. Neither clearly unmet nor clearly solved.
- Confidence: *medium* (rests on absence-of-evidence + one refuted claim)

---

## Implications for CLAN

- **The wedge is boundary-crossing, not framework-replacement.** Value is highest where framework runtime state is weakest: cross-framework handoff (LangGraph → CrewAI → AutoGen), durable/versionable artifacts, token-optimized re-ingestion. Redundant inside a single framework's runtime.
- **Sharpest positioning:** *the concrete, portable file format for the agent-memory/handoff pattern Anthropic endorses but left unspecified — that travels across frameworks.*
- **Token efficiency (TOON) is table stakes, not a moat** — it's now a public standard. Align with it.
- **Do not headline provenance** — it's under-determined.
- **The concept/name is already being circled** — `contextcapsule.ai` and a "Capsules" blog appeared (low-quality, unverified). Demand signal + the name may be taken + the window is narrowing.

---

## Caveats
- "Gap exists" rests substantially on **absence of evidence**; an unindexed startup or proprietary internal format could exist unsurfaced.
- **Coverage gap:** CrewAI (task outputs/memory) and OpenAI Agents SDK / Swarm (handoffs) were named in the question but not covered by surviving confirmed claims — pending a focused follow-up.
- Fast-moving field (standards merging in 2025–2026); one arXiv ID looked future-stamped (treat its date cautiously).

## Open questions (for follow-up)
1. How do **CrewAI** and **OpenAI Agents SDK / Swarm** actually pass context — portable files or runtime state?
2. What is **contextcapsule.ai** / the "Capsules" blog — a real competing product, and what does it do?
3. Is there any existing standard that bakes provenance/lineage into a portable, inspectable artifact (e.g., W3C PROV-style signed audit capsules)?
4. Could public **TOON** serve directly as CLAN's serialization layer (spec-compatibility check)?

---

*Generated from a deep-research pass (run `wf_95679b9d-2cd`), 2026-06-09. 24 sources, 116 claims, 25 verified.*
