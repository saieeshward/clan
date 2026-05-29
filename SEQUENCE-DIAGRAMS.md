# LACE Sequence Diagrams

All diagrams use Mermaid syntax. Render with any Mermaid-compatible viewer (GitHub, Obsidian, Mermaid Live Editor).

---

## 1. First LACE Creation (Human-Initiated)

A human opens the app, clicks "New LACE", enters a title and brief. The SDK creates the container and a first-pass agent produces the initial document.

```mermaid
sequenceDiagram
    actor Human
    participant App as LACE App (Tauri)
    participant SDK as LACE SDK (Rust)
    participant FS as File System
    participant Agent as First Agent (LLM)

    Human->>App: Click "New LACE"
    App->>Human: Show creation dialog (title + brief)
    Human->>App: "Invoice Review" + "Analyze Q2 invoice from Acme Corp"

    App->>SDK: lace.create(title, brief, type="invoice")

    SDK->>SDK: Select template (invoice)
    SDK->>SDK: Generate UUID for document
    SDK->>SDK: Write manifest.yaml (lace_version, id, title, created_at)
    SDK->>SDK: Copy spec/lace.md and spec/agent-guide.md from template
    SDK->>SDK: Write shared/data.yaml (empty, schema declared)
    SDK->>SDK: Write agent/context.md from brief
    SDK->>SDK: Write agent/output-schema.json (full-html for first pass)
    SDK->>SDK: Write agent/state.yaml (stage: 0, status: pending)
    SDK->>SDK: Write agent/decision-chain.yaml (empty)
    SDK->>SDK: Write human/index.html (pending state template)
    SDK->>SDK: Package into invoice-review.lace (ZIP)

    SDK->>FS: Write invoice-review.lace
    SDK-->>App: LACE file path

    App->>App: Open LACE — show pending state in document WebView
    App->>Agent: SDK assembles context (guide + context.md + schema)

    Agent->>Agent: Read agent_guide (understand LACE format)
    Agent->>Agent: Read context.md (understand task)
    Agent->>Agent: Reason and produce full-html output

    Agent-->>SDK: JSON output { mode: "full-html", structured: {...}, human: {...} }

    SDK->>SDK: Validate output against output-schema.json
    SDK->>SDK: Sanitise HTML (ammonia rules)
    SDK->>SDK: Update shared/data.yaml with structured fields
    SDK->>SDK: Write human/index.html (agent-generated)
    SDK->>SDK: Write human/styles.css
    SDK->>SDK: Write human/assets/* (SVG charts)
    SDK->>SDK: Append to decision-chain.yaml
    SDK->>SDK: Update state.yaml
    SDK->>SDK: Repackage into invoice-review.lace

    SDK-->>App: Updated LACE ready
    App->>App: Re-render document WebView with agent HTML
    App->>Human: Fully formatted invoice document displayed
```

---

## 2. Agent Ingestion Flow (SDK Path)

How the SDK prepares context for an agent receiving an existing LACE file.

```mermaid
sequenceDiagram
    participant Pipeline as Orchestrator
    participant SDK as LACE SDK (Rust)
    participant LACE as .lace file (ZIP)
    participant Agent as Agent (LLM)
    participant Cache as Session Cache

    Pipeline->>SDK: sdk.open("invoice-review.lace", agent_role="validator")

    SDK->>LACE: Open ZIP archive
    SDK->>LACE: Read manifest.yaml (by_name — first entry)
    SDK->>LACE: Read spec/agent-guide.md
    SDK->>LACE: Read agent/context.md
    SDK->>LACE: Read agent/output-schema.json
    SDK->>LACE: Read shared/data.yaml
    SDK->>LACE: Read agent/decision-chain.yaml
    SDK->>LACE: Read human/patches.yaml (optional)
    SDK->>LACE: Close ZIP (no full extraction)

    SDK->>Cache: Check: is spec guide cached for this session?

    alt Guide not cached
        SDK->>SDK: Prepend spec/agent-guide.md to context
        SDK->>Cache: Cache guide for session
    end

    SDK->>SDK: Assemble context object:
    note right of SDK: 1. spec/agent-guide.md (~800 tokens)<br/>2. agent/context.md (~400 tokens)<br/>3. agent/output-schema.json (~300 tokens)<br/>4. shared/data.yaml (variable)<br/>5. decision-chain.yaml (compressed)

    SDK-->>Agent: Assembled context object
    Agent->>Agent: Read and understand format (from guide)
    Agent->>Agent: Read task (from context.md)
    Agent->>Agent: Reason and produce output
    Agent-->>SDK: JSON { mode, structured, ... }
```

---

## 3. Agent Output and LACE Packaging

How the SDK validates and packages agent output into a new LACE file.

```mermaid
sequenceDiagram
    participant Agent as Agent (LLM)
    participant SDK as LACE SDK (Rust)
    participant LACE as New .lace file
    participant LLM as Compression LLM

    Agent-->>SDK: JSON output object

    SDK->>SDK: Parse JSON output
    SDK->>SDK: Validate against output-schema.json
    alt Validation fails
        SDK-->>Agent: Error: schema mismatch details
    end

    SDK->>SDK: Check output mode

    alt mode = "data-update"
        SDK->>SDK: Update shared/data.yaml with structured fields
        SDK->>SDK: Re-render human/index.html (data binding substitution)
        SDK->>SDK: Preserve existing human/styles.css

    else mode = "designed"
        SDK->>SDK: Update shared/data.yaml
        SDK->>SDK: Apply design directives to template
        SDK->>SDK: Generate human/index.html from directed template
        SDK->>SDK: Generate human/styles.css from theme

    else mode = "full-html"
        SDK->>SDK: Update shared/data.yaml
        SDK->>SDK: Sanitise HTML (ammonia — strip scripts, events)
        SDK->>SDK: Write human/index.html (sanitised fragment)
        SDK->>SDK: Scope and write human/styles.css
        SDK->>SDK: Write human/assets/* (SVG, images)
    end

    SDK->>SDK: Generate human/index.txt from HTML (strip tags)

    SDK->>SDK: Prepare new decision-chain entry
    SDK->>SDK: Count existing entries

    alt entries >= 16
        SDK->>LLM: Compress oldest entries to 2-sentence summaries
        LLM-->>SDK: Compressed summaries
    else entries >= 4
        SDK->>LLM: Compress entries 4-N to key-facts format
        LLM-->>SDK: Compressed entries
    end

    SDK->>SDK: Prepend new decision entry (full fidelity)
    SDK->>SDK: Write agent/decision-chain.yaml

    SDK->>SDK: Update agent/state.yaml
    SDK->>SDK: Update manifest.yaml (updated_at, lineage delta)
    SDK->>SDK: Generate new UUID for this LACE version

    SDK->>LACE: Package all files into new .lace (ZIP + DEFLATE)

    SDK-->>Pipeline: New .lace file path
```

---

## 4. Human Edit Flow

How a user edits text in the rendered document WebView and how edits are persisted.

```mermaid
sequenceDiagram
    actor Human
    participant Shell as Shell WebView (Svelte)
    participant Rust as Tauri Rust Backend
    participant Doc as Document WebView (sandboxed)
    participant FS as LACE File

    Human->>Shell: Click "Edit" button in toolbar
    Shell->>Rust: invoke("enter_edit_mode")
    Rust->>Rust: Read current data-adf-id assignments from rendered HTML
    Rust->>Doc: Serve updated lace://current/edit-bridge.js (edit mode active)
    Doc->>Doc: edit-bridge.js runs:
    note right of Doc: querySelectorAll("[data-adf-id]")<br/>set contentEditable = true<br/>add blur listeners

    Human->>Doc: Click on headline — cursor appears
    Human->>Doc: Type "Updated Invoice Review — Amended"
    Human->>Doc: Click elsewhere (blur event fires)

    Doc->>Shell: postMessage({ type: "lace-edit", id: "heading-0", content: "Updated Invoice Review — Amended" })

    Shell->>Shell: Validate message type ("lace-edit" allowed)
    Shell->>Shell: Validate id (exists in current data-adf-id map)
    Shell->>Rust: invoke("save_patch", { id: "heading-0", content: "..." })

    Rust->>FS: Open .lace ZIP
    Rust->>FS: Read human/patches.yaml
    Rust->>Rust: Append patch entry { id, content, edited_at, edited_by: "human" }
    Rust->>FS: Write updated human/patches.yaml into ZIP
    Rust->>FS: Close ZIP

    Rust-->>Shell: Patch saved confirmation
    Shell-->>Human: Visual confirmation (brief flash or checkmark)

    note over Doc: On next render cycle:<br/>Rust reads patches.yaml<br/>Applies to HTML before serving<br/>User sees their edit reflected
```

---

## 5. Multi-Agent Pipeline

A complete pipeline showing how LACE files flow through multiple agents, accumulating decisions and lineage.

```mermaid
sequenceDiagram
    actor Human
    participant App as LACE App
    participant SDK as LACE SDK
    participant A1 as Extractor Agent
    participant A2 as Validator Agent
    participant A3 as Approval Agent

    Human->>App: New LACE — "Process Acme Q2 Invoice"
    App->>SDK: create(brief, type="invoice")
    SDK-->>App: invoice-v0.lace (pending state, no data)

    App->>SDK: open(invoice-v0.lace, agent=extractor)
    SDK-->>A1: Context (guide + context.md + schema + empty data)
    A1-->>SDK: { mode: "full-html", structured: {vendor, total, ...}, human: {html, css} }
    SDK->>SDK: Package → invoice-v1.lace (lineage: parent=v0)
    note right of SDK: decision-chain: [extractor: extracted 12 fields]

    SDK-->>App: invoice-v1.lace ready
    App->>App: Render — human sees formatted invoice

    App->>SDK: open(invoice-v1.lace, agent=validator)
    SDK-->>A2: Context (guide + context.md + schema + extracted data + chain)
    A2-->>SDK: { mode: "data-update", structured: {status: "validated", discrepancies: []} }
    SDK->>SDK: Package → invoice-v2.lace (lineage: parent=v1)
    note right of SDK: decision-chain: [extractor, validator]

    SDK-->>App: invoice-v2.lace ready
    App->>App: Re-render — status badge updates to "Validated"

    Human->>App: Edit heading text via WebView
    App->>SDK: save_patch({ id: "heading-0", content: "Invoice — Amended" })
    note right of SDK: patches.yaml updated — HTML unchanged

    App->>SDK: open(invoice-v2.lace, agent=approver)
    SDK-->>A3: Context (includes patches.yaml — sees human amendment)
    A3-->>SDK: { mode: "data-update", structured: {status: "approved", approval_reason: "..."} }
    SDK->>SDK: Package → invoice-v3.lace (lineage: parent=v2)
    note right of SDK: decision-chain: [extractor, validator, approver]

    SDK-->>App: invoice-v3.lace ready
    App->>Human: Final approved invoice — full lineage timeline visible in sidebar
```

---

## 6. Static Export (No-SDK Agent Path)

How an agent with no SDK access can still produce valid LACE output.

```mermaid
sequenceDiagram
    participant Orch as Orchestrator
    participant SDK as LACE SDK
    participant LACE as .lace file
    participant API as LLM API (no SDK)
    participant Agent as External Agent

    Orch->>SDK: sdk.export_static("invoice.lace")
    SDK->>LACE: Read all sections
    SDK->>SDK: Assemble static export:
    note right of SDK: {<br/>  lace_version: "1.0",<br/>  agent_guide: "...spec/agent-guide.md...",<br/>  task: "...agent/context.md...",<br/>  output_schema: {...},<br/>  shared_data: {...},<br/>  decision_history: [...]<br/>}
    SDK-->>Orch: invoice-static.json

    Orch->>API: POST /chat/completions
    note right of Orch: System: "You are processing a LACE document.<br/>Read agent_guide first."<br/>User: [invoice-static.json contents]

    API->>Agent: Agent receives static JSON
    Agent->>Agent: Read embedded agent_guide
    Agent->>Agent: Read task and output_schema
    Agent->>Agent: Reason and produce structured output
    Agent-->>API: JSON matching output_schema

    API-->>Orch: Agent response JSON

    Orch->>SDK: sdk.package_output(response_json, source_xon="invoice.lace")
    SDK->>SDK: Validate against output-schema.json
    SDK->>SDK: Package into new invoice-v2.lace
    SDK-->>Orch: invoice-v2.lace (valid LACE, full lineage preserved)

    note over Agent: Agent never knew it was working with LACE.
    note over Orch: Output is a fully valid LACE file.
```

---

## 7. App Rendering Pipeline

How the Tauri app renders a LACE file in the document WebView.

```mermaid
sequenceDiagram
    participant FS as File System
    participant Rust as Tauri Rust Backend
    participant Shell as Shell WebView (Svelte)
    participant Protocol as lace:// Protocol Handler
    participant Doc as Document WebView (sandboxed)

    FS->>Rust: File open event (double-click or invoke)
    Rust->>Rust: Buffer file path (WebView may not be ready)

    Shell->>Rust: invoke("webview_ready") [from JS onMount]
    Rust->>Rust: Dequeue buffered file path

    Rust->>FS: Open .lace ZIP
    Rust->>FS: Extract manifest.yaml → parse
    Rust->>FS: Extract shared/data.yaml → parse
    Rust->>FS: Extract human/patches.yaml → parse
    Rust->>FS: Extract human/index.html
    Rust->>FS: Keep ZIP open in memory (assets served on demand)

    Rust->>Rust: Resolve data bindings ({{token}} → shared/data.yaml values)
    Rust->>Rust: Assign data-adf-id to editable text elements
    Rust->>Rust: Apply patches.yaml (replace content for matching IDs)
    Rust->>Rust: Generate bindings.js (window.__LACE__ = { data: {...} })

    Shell->>Rust: invoke("get_agent_panels")
    Rust-->>Shell: { state: {...}, decision_chain: [...], lineage: {...} }
    Shell->>Shell: Render sidebar panels (Svelte)

    Doc->>Protocol: GET lace://current/index.html
    Protocol->>Rust: Route to in-memory handler
    Rust-->>Protocol: Processed HTML fragment + CSP headers
    Protocol-->>Doc: HTML fragment rendered

    Doc->>Protocol: GET lace://current/styles.css
    Protocol->>Rust: Serve human/styles.css from ZIP
    Rust-->>Doc: CSS (scoped)

    Doc->>Protocol: GET lace://current/bindings.js
    Protocol->>Rust: Serve generated bindings.js
    Rust-->>Doc: window.__LACE__ = { data: {...} }

    Doc->>Protocol: GET lace://current/assets/chart.svg
    Protocol->>Rust: Extract from ZIP on demand
    Rust-->>Doc: SVG content

    Doc->>Doc: Render complete document
    Shell->>Shell: Render lineage timeline and metadata
```

---

## 8. LACE File Creation — CLI Path

How a developer creates a LACE from the command line.

```mermaid
sequenceDiagram
    actor Dev as Developer
    participant CLI as lace CLI
    participant SDK as LACE SDK (Rust)
    participant FS as File System

    Dev->>CLI: lace create --type invoice --title "Q2 Invoice"

    CLI->>SDK: sdk.create(type="invoice", title="Q2 Invoice")
    SDK->>SDK: Load invoice template from embedded library
    SDK->>SDK: Generate UUID
    SDK->>SDK: Write manifest.yaml
    SDK->>SDK: Copy spec/lace.md (pinned to current SDK version)
    SDK->>SDK: Copy spec/agent-guide.md
    SDK->>SDK: Write shared/data.yaml (empty + invoice schema ref)
    SDK->>SDK: Write agent/context.md (placeholder — prompt user to fill)
    SDK->>SDK: Write agent/output-schema.json (invoice schema)
    SDK->>SDK: Write agent/state.yaml (stage: 0)
    SDK->>SDK: Write agent/decision-chain.yaml (empty list)
    SDK->>SDK: Write human/index.html (pending state)
    SDK->>SDK: Package into q2-invoice.lace

    SDK->>FS: Write q2-invoice.lace
    SDK-->>CLI: Created: q2-invoice.lace

    CLI-->>Dev: ✓ q2-invoice.lace created
    Dev->>CLI: lace read q2-invoice.lace agent
    CLI->>SDK: sdk.read("q2-invoice.lace", section="agent")
    SDK-->>CLI: YAML output of all agent/ files
    CLI-->>Dev: Displays agent context (for debugging)

    Dev->>CLI: lace validate q2-invoice.lace
    CLI->>SDK: sdk.validate("q2-invoice.lace")
    SDK->>SDK: Run all validation checks (Section 17)
    SDK-->>CLI: Valid ✓ (or list of errors)
```
