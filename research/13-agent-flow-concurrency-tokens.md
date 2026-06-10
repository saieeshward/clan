# 13 — Agent-Only Handoff Flow, Concurrent Multi-Agent Writes, and Token Efficiency

**Date:** 2026-06-10
**Method:** Multi-source research pass with 3-vote adversarial verification. 21 claims survived (20 unanimous 3-0, 1 split 2-1); 4 claims were refuted and excluded. All surviving claims rest on primary (first-party vendor/standards) documentation fetched 2026-06-10.
**Companion:** Builds on `research/12-market-gap-portable-context.md` (June 2026), which established: LangGraph persists state in DB checkpointers with no portable export; Anthropic endorses file-based memory but specifies no format; MCP is agent-to-tool; A2A/AGNTCY are runtime plumbing; TOON is a public standard with ~40% dataset-dependent token savings.

---

## Executive summary

No major framework treats an agent handoff as a portable artifact: in LangGraph, the OpenAI Agents SDK, and AutoGen, handoff is a runtime construct (a `Command` state mutation, a `transfer_to_<agent>` tool call, a delegate-tool topic publish) inside one process, and what crosses the boundary ranges from a full raw transcript (OpenAI SDK default, AutoGen Core pattern) to a deliberately minimal "handoff pair" (LangGraph's recommendation) to a bare task-output string (CrewAI) — so "no work lost" is nowhere a guarantee today, and agent configuration (system prompts, tool definitions) is *never* part of the payload. For concurrency, the production-proven answer is LangGraph's model: built-in parallel fan-out with deterministic **per-key reducer merge** of concurrent writes (default last-write-wins unless a reducer is declared) — which maps directly onto CLAN's easiest-and-correct v1.1 design: namespace-per-agent member files inside the ZIP, multi-parent lineage, and a deterministic manifest-driven fold at merge time; CRDTs are unwarranted for a file-passing artifact (judgment call, flagged as such). On token efficiency, the verified evidence in this pass is thin: the two confirmed signals are that minimal-payload handoff exists precisely because full transcripts inflate token cost (LangGraph), and that OpenAI's `previous_response_id` server-side continuation demonstrates the value of delta-style "send only what's new" — at the price of vendor lock-in that a portable file format like CLAN avoids while still being able to adopt the delta pattern (per-hop delta records + fold). The optional human HTML view has strong architectural precedent: nbviewer renders notebook JSON to HTML on demand as a service rather than embedding it, and the W3C TAG finding holds that content should live in semantically rich markup with presentation delivered optionally.

---

## Thread 1 — Agent-only handoff flow: what crosses framework boundaries today

### 1.1 Handoffs are runtime control-flow, never portable artifacts (HIGH confidence)

Every surveyed framework implements agent-to-agent handoff as an in-process mechanism, not a serialized object that could cross a framework boundary:

- **LangGraph**: handoff tools return `Command` objects that mutate graph state and route execution ("Handoff tools navigate between agent nodes using `Command.PARENT` to specify which node to execute next"). It is an in-process state mutation inside one graph runtime. Corroborated by langgraph-swarm requiring both graphs to share a state schema, and checkpointers serializing only into framework-internal, thread-organized DB checkpoints. [LangChain handoffs docs](https://docs.langchain.com/oss/python/langchain/multi-agent/handoffs)
- **OpenAI Agents SDK**: a handoff is literally an LLM tool call with a generated name — "if there's a handoff to an agent named `Refund Agent`, the tool would be called `transfer_to_refund_agent`." No separate persistence or export mechanism exists for handoffs. [OpenAI Agents SDK handoffs](https://openai.github.io/openai-agents-python/handoffs/)
- **AutoGen Core** (documented handoffs pattern): handoffs are special **delegate tools**, distinct from regular tools — when the model invokes one, the tool returns a target topic-type string and the agent publishes a `UserTask` message to that topic instead of executing locally (`if call.name in self._delegate_tools: ...`). [AutoGen Core handoffs pattern](https://microsoft.github.io/autogen/stable//user-guide/core-user-guide/design-patterns/handoffs.html)

**Implication for CLAN:** the portable-artifact niche identified in research/12 is confirmed at the handoff layer specifically. CLAN does not compete with these mechanisms; it is the missing serialization of what they pass in memory.

### 1.2 Frameworks disagree on the default payload — "no work lost" is not anyone's guarantee (HIGH confidence)

What actually crosses the boundary varies by an order of magnitude across frameworks:

| Framework | Default handoff payload | Lossless? |
|---|---|---|
| OpenAI Agents SDK | **Full transcript** — "the new agent takes over the conversation, and gets to see the entire previous conversation history" | Yes by default, but mutable: `input_filter` receives a `HandoffInputData` and returns a new one; the pre-built `handoff_filters.remove_all_tools` strips all tool calls from history. Loss is opt-in. |
| AutoGen Core (handoffs pattern) | **Full accumulated message list** — delegate context = complete prior messages + the function call + its result, published to the target topic | Yes for conversation; no for config (below) |
| LangGraph | **Minimal "handoff pair"** — only the AIMessage with the triggering tool call plus a matching ToolMessage; docs explicitly discourage full history ("the receiving agent may become confused by irrelevant internal reasoning, and token costs increase unnecessarily"; "By passing only the handoff pair, you keep the parent graph's context focused on high-level coordination"). langgraph-supervisor defaults to `last_message`, not `full_history`. | No — deliberately lossy; full history is opt-in |
| CrewAI (sequential) | **Task output only** — "the output of one task is automatically relayed into the next one"; by default `TaskOutput` includes only the `raw` unstructured text (structured `json_dict`/`pydantic` only if `output_json`/`output_pydantic` is configured). Intermediate reasoning, tool calls, and scratchpad state are not in the payload. | No — distilled by design |

Sources: [LangChain handoffs](https://docs.langchain.com/oss/python/langchain/multi-agent/handoffs), [OpenAI handoffs](https://openai.github.io/openai-agents-python/handoffs/), [AutoGen Core handoffs](https://microsoft.github.io/autogen/stable//user-guide/core-user-guide/design-patterns/handoffs.html), [CrewAI tasks](https://docs.crewai.com/core-concepts/Tasks/) (current path: /en/concepts/tasks).

**Implication:** a formal "no work lost" definition for CLAN must cover **both** poles, because real workflows mix them — full message history (OpenAI/AutoGen style) *and* distilled task state (LangGraph/CrewAI style) are each "the work" depending on the consuming framework.

### 1.3 What is systematically lost at every boundary today (HIGH confidence)

- **Agent configuration never travels.** In AutoGen's handoff pattern each agent retains its own system prompt and tool set; only the conversation context moves. Tool definitions and instructions are framework-resident, constructor-bound. The same design holds in OpenAI Swarm/Agents SDK. [AutoGen Core handoffs](https://microsoft.github.io/autogen/stable//user-guide/core-user-guide/design-patterns/handoffs.html)
- **Configuration export exists but is state-free.** AutoGen AgentChat can serialize agents, teams, and termination conditions to declarative JSON via `dump_component()`/`load_component()` — but this captures provider/component_type/version/config only, **not** runtime state, message history, or memory. (AutoGen's separate `save_state()` does export message history as a JSON dict — the config/state split is two disjoint mechanisms, neither carrying the other.) Note: AutoGen is in maintenance mode, superseded by Microsoft Agent Framework (Oct 2025). [AutoGen serialize-components](https://microsoft.github.io/autogen/stable//user-guide/agentchat-user-guide/serialize-components.html)
- **History is mutable at the boundary.** OpenAI's `input_filter` mechanism makes the transcript deliberately rewritable per hop (e.g., stripping all tool calls) — useful, but it means downstream agents cannot assume completeness.
- **Intermediate work is dropped by output-relay designs.** CrewAI's default `TaskOutput` (raw text only) discards reasoning, tool calls, and scratchpads entirely.

**"No work lost" — proposed formal definition for CLAN v1.1.** A handoff is lossless iff the artifact carries all five layers, each independently optional to *read* but mandatory to *preserve*:
1. **Distilled state** (`shared/data.yaml`) — the CrewAI/LangGraph-style task output; what the next hop needs.
2. **Full working transcript / scratchpad** (`agent/context.md` + a new `agent/transcript/` member) — the OpenAI/AutoGen-style raw history, preserved even when the next hop only reads the distilled layer.
3. **Contracts** (`output-schema.json`) — expected output shape; today this exists nowhere in handoff payloads.
4. **Provenance** (`decision-chain.yaml`) — who did what, why, in what order; today reconstructable only from transcripts, if at all.
5. **Capability requirements** (recommendation: new `agent/requirements.yaml`) — declared tool/skill needs, since actual tool definitions are framework-resident and *cannot* portably travel; CLAN should carry the requirement declaration, not the implementation.

The pass-through rule matters most: **an agent that consumes only layer 1 must still copy layers 2–5 forward unmodified.** That single rule is what no framework provides and what makes the chain pluggable into another framework later.

### 1.4 Continuation mechanisms confirm the artifact gap (HIGH confidence; sessions claim MEDIUM, 2-1 vote)

The OpenAI Agents SDK offers three continuation modes, none of which is a portable file:
- **Manual replay**: applications owning local history reuse `history` (TS) / `to_input_list()` (Python) — a raw message list the *application* must carry; lossless continuation is the app's responsibility. [Results guide](https://developers.openai.com/api/docs/guides/agents/results)
- **Sessions** (MEDIUM — 2-1 vote): "keep passing the same session and let the SDK load and persist history for you" — state lives in SDK-managed storage (SQLite/server-side), extractable programmatically but with no defined exportable format.
- **Server-managed continuation**: pass only the new user input plus the stored response ID (`previous_response_id`) "instead of replaying the full transcript" — a genuine **delta pattern** with real token savings, but it pins conversation state to OpenAI's servers (~30-day retention).

**Implication:** the delta-continuation idea is proven and valuable; CLAN can replicate it *portably* (see Thread 2's event-log design and Thread 3).

### 1.5 Deferred human view: strong precedents (HIGH confidence)

- **nbviewer**: Jupyter notebooks are machine-readable JSON; nbviewer is "nbconvert as a web service: Render Jupyter Notebooks as static web pages" — the human-readable HTML is materialized on demand, never embedded in the .ipynb. [github.com/jupyter/nbviewer](https://github.com/jupyter/nbviewer/)
- **W3C TAG finding** (draft, later affirmed in AWWW §4.3 as a W3C Recommendation principle): "important information SHOULD be stored and (optionally) delivered with markup that is as semantically rich as achievable" — content/presentation separation is architecturally sound. [W3C contentPresentation-26](https://www.w3.org/2001/tag/doc/contentPresentation-26.html)

**Recommendation (v1.1): `--no-render` / lazy-view mode.** The structured members are canonical; the HTML view becomes an optional, derivable member. Add a manifest flag (`view: {present: false, renderable: true, renderer_spec: <embedded spec section>}`) so any hop — or a standalone `clan render` command — can materialize the view later from the structured members alone, nbviewer-style. Agents in a pure A2A chain skip rendering at every hop; the last hop (or the human) renders once. This also saves tokens and wall-clock at each hop for free.

---

## Thread 2 — Concurrent multi-agent writes to one artifact

### 2.1 What production frameworks actually do (HIGH confidence)

- **LangGraph — parallel fan-out is the built-in execution model**: "If a node has multiple outgoing edges, all of those destination nodes will be executed in parallel as a part of the next superstep" (Pregel-style supersteps; `Send` is only needed for *dynamic* fan-out). [Graph API docs](https://docs.langchain.com/oss/python/langgraph/graph-api)
- **LangGraph — concurrent writes resolve via deterministic per-key reducers**: each state key declares its own reducer with `Annotated` (e.g., `operator.add` appends list updates). Without a reducer, updates are **last-write-wins overrides** (node returning `{"foo": 2}` against `{"foo": 1, "bar": ["hi"]}` yields `{"foo": 2, "bar": ["hi"]}`), and concurrent unreduced writes to one key raise `INVALID_CONCURRENT_GRAPH_UPDATE`. Merge is opt-in per key; overwrite is the default. [Graph API docs](https://docs.langchain.com/oss/python/langgraph/graph-api), [INVALID_CONCURRENT_GRAPH_UPDATE](https://docs.langchain.com/oss/python/langgraph/errors/INVALID_CONCURRENT_GRAPH_UPDATE)
- **CrewAI — fan-out via `async_execution=True`** ("the crew will not wait for it to be completed to continue with the next task"); the **join** is a later task listing the async tasks in its `context` attribute. The merged payload is, again, task outputs only. [CrewAI tasks](https://docs.crewai.com/core-concepts/Tasks/)

The industry's revealed answer to "multiple agents, one artifact" is therefore: **isolate writes (per-branch state / per-task output), then deterministically fold at a join point** — not CRDTs, not OT, not locking.

### 2.2 Recommended design for CLAN v1.1

**EASIEST way (and, for this problem, also correct): namespace-per-agent member files + manifest + deterministic fold.**
- On fan-out, the orchestrator (or `clan fork`) issues each agent the same parent artifact; each agent writes **only** to its own namespace inside the ZIP (`agents/<agent-id>/data.yaml`, `agents/<agent-id>/context.md`, `agents/<agent-id>/decisions.yaml`). Zero write conflicts by construction — this is LangGraph's per-branch isolation expressed as files.
- On join, `clan merge` produces a child with **multi-parent lineage** (extend `decision-chain.yaml` from single `parent:` to `parents: [...]` — the one spec change v1.1 needs) and folds namespaces into `shared/data.yaml` using **per-key merge policies declared in the manifest**, directly mirroring LangGraph reducers: `merge: {findings: append, status: last-write, scores: max, summary: agent-priority}`. Default = last-write-wins **with a machine-readable conflict report member** (`merge-report.yaml`) listing every key where parents disagreed, so a downstream agent (or human) can adjudicate — conflicts become data, not failures.
- Token cost: each parallel agent receives the shared base **once** plus only its own namespace; the merge step is mechanical (no LLM call) or, at worst, one LLM call over the conflict report rather than over N full documents. No agent ever re-reads sibling transcripts.

**CORRECT-but-heavier alternatives, and why to defer them:**
- **Event-log / append-only deltas** (each agent appends a delta record; state = fold of deltas): the most principled design, naturally delta-token-efficient (echoes OpenAI's `previous_response_id` pattern portably), and a good v1.2 target — but it changes every reader from "read data.yaml" to "fold the log," a larger spec break.
- **Git-style three-way merge on YAML/JSON**: correct but requires a common-ancestor diff engine and YAML-semantic (not line-based) merging; heavy for v1.1. *(Assessment — not adversarially verified.)*
- **CRDTs (Automerge/Yjs/Loro)**: built for real-time character-level co-editing with arbitrary concurrent edits to the *same* values; a file-passing format with explicit fork/join points doesn't have that problem, so the runtime dependency and binary state formats are overkill. *(Judgment call — flagged as not adversarially verified in this pass.)*
- **OT**: requires a central transformation server; categorically wrong for a portable file. **Lock files/leases**: serialize the work, defeating fan-out; only useful as an optional advisory `lock` member for shared-filesystem deployments.

The strongest evidence-backed argument for the recommended design: it is structurally identical to what LangGraph ships as its only supported concurrency model (per-key reducers over isolated branch writes), and to CrewAI's async/context join — CLAN merely makes the fold explicit, portable, and auditable.

---

## Thread 3 — Token efficiency beyond TOON

This thread had the weakest verified evidence in this pass: most candidate claims (LLMLingua compression ratios, KV-cache prefix economics, CBOR/MessagePack token behavior, Anthropic's condensed-summary guidance, Microsoft Agent Framework compaction) did **not** survive to the confirmed set and are deferred to a follow-up pass. Two confirmed signals and their design consequences:

1. **Minimal-payload handoff is motivated by token cost in primary docs (HIGH).** LangGraph explicitly recommends the two-message handoff pair over full history because "token costs increase unnecessarily" otherwise, and langgraph-supervisor defaults to `last_message`. CrewAI relays only task outputs. → CLAN's injection layer should make **distilled-only injection the default** (inject `shared/data.yaml` + the latest decision entry, TOON-encoded), with full-transcript members carried in the ZIP but injected only on demand. The artifact is lossless; the *injection* is lean. This combines both frameworks' philosophies without their loss.
2. **Delta continuation is proven but vendor-locked at OpenAI (HIGH).** `previous_response_id` continuation sends "only the new user input ... instead of replaying the full transcript," with state held on OpenAI's servers. → CLAN can offer the same economics portably: when agent N+1 runs in the same chain, inject only the **per-hop delta records** (new namespace writes + new decision entries since the parent snapshot) rather than the whole document. This falls out naturally from the Thread-2 namespace/fold design and is the highest-leverage token feature v1.1 can add on top of TOON.

**Priority order for v1.1 (evidence-weighted):**
1. Distilled-default injection with lazy full-history dereference (backed by confirmed framework behavior).
2. Delta-per-hop injection from namespace/decision-chain structure (backed by confirmed OpenAI pattern; portable variant is design inference).
3. Keep the ~800-token agent guide byte-stable across hops to remain KV-cache/prompt-cache friendly (*speculative in this pass — caching benefit not adversarially verified; verify against provider prompt-caching docs before claiming savings*).
4. TOON for tabular/uniform members (carried over from research/12: ~40%, dataset-dependent).
5. Defer: LLMLingua-style compression, key interning/abbreviation schemes, schema-default omission — promising but unverified here; do not ship or claim numbers without a dedicated research pass.

---

## Refuted claims (transparency)

Four claims failed adversarial verification and are excluded from findings:
- "LangGraph context propagation is opt-in/developer-curated, so unforwarded context is lost by default" (0-3) — overstated; shared-state topologies propagate automatically.
- "OpenAI `RunConfig.nest_handoff_history` is a built-in distilled-summary handoff mode" (0-3) — exists only as a non-default beta; the claim as worded overstated it.
- "AutoGen selector functions silently dropped and tools cannot be serialized at all" (1-2) — partially outdated (FunctionTool support added ~v0.4.4).
- "OpenAI documents only `lastAgent` reuse for handoff continuation with unspecified context preservation" (0-3) — contradicted by the documented full-history default.

## Caveats

- **Time-sensitivity:** all framework claims reflect documentation as of 2026-06-10. AutoGen is in maintenance mode (superseded by Microsoft Agent Framework, Oct 2025); its findings describe a framework whose successor was not directly verified in this pass. OpenAI's beta `nest_handoff_history` suggests distilled-handoff modes are actively evolving.
- **Thread 3 is under-evidenced:** quantitative claims about LLMLingua, KV-cache savings, binary serialization token behavior, and Anthropic's condensed-handoff guidance did not survive verification and the recommendations there beyond items 1-2 are design inference, marked speculative.
- **CRDT/OT/three-way-merge dismissals are engineering judgment**, consistent with but not proven by the verified evidence (no surviving claim directly evaluated CRDT fitness for file-based handoff).
- One finding (OpenAI sessions as SDK-managed storage) carried a 2-1 split vote → MEDIUM; the dissent concerned whether a file-backed SQLite session counts as "exportable," not the substance.
- The CrewAI citation path `core-concepts/Tasks/` redirects; current canonical path is `/en/concepts/tasks`.

## Open questions

1. Does Microsoft Agent Framework (AutoGen's successor) introduce portable state export or compaction strategies that change the Thread-1 gap analysis or Thread-3 recommendations?
2. What are the *measured* token savings of distilled-default + delta-per-hop injection on real CLAN chains versus full-document re-injection — and how do they compose with TOON's ~40%?
3. Do provider prompt-caching mechanisms (Anthropic/OpenAI) actually yield cache hits on CLAN's stable ~800-token agent guide across separate agent invocations, given cache scoping rules?
4. For multi-parent merge: what fraction of real fan-out workloads produce per-key conflicts that last-write-wins + conflict-report handles poorly, and would that justify pulling the event-log design forward from v1.2?
