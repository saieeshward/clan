# Viewer Extensibility — Core + Shell Architecture

**Status:** DESIGN (not implemented) · **Date:** 2026-06-11 · **Decision:** build-time core + shell, plus a declarative zero-code runtime plugin layer (§3.2); no runtime *code* loading · **Priority extension points:** custom toolbar/panel controls, pipeline/CLI hooks. Branding/theme included as table stakes for "rebrand it."

## Goal

The current viewer becomes a **base** any team can pick up, rebrand, and extend with their own controls — while the core (open/validate/render/edit/patch) remains CLAN-maintained and non-forked. A downstream team should ship a branded, domain-specific viewer by writing **only** a config object and their custom components, never touching core source.

**Non-goals:** runtime plugin loading (no sandboxing/versioning problem we don't need yet — revisit if a marketplace emerges); web-only build; replacing core security paths.

---

## 1. Current state inventory

| Layer | Today | Classification |
|---|---|---|
| `src-tauri/main.rs` (~1,100 lines) | 9 commands: `open_clan`, `get_human_html`, `get_data`, `get_chain`, `get_agent_state`, `get_context`, `save_patch`, `set_edit_mode`, `update_preview_html` + `clan://` URI scheme + bindings/adf-id/strip logic | **core** |
| `DocumentView.tsx` (iframe + edit bridge) | sandboxed render, edit instrumentation, patch postMessage | **core** |
| `App.tsx` | layout, state wiring, open dialog | **shell** (currently monolithic) |
| `Toolbar.tsx` | hardcoded `ClanMark` SVG, "CLAN" wordmark, fixed button set | **shell + brand** (currently hardcoded) |
| `AgentPanel.tsx` | chain/state/context tabs | **core feature, should become the first extension-API consumer** |
| `Sidebar.tsx`, `Welcome.tsx` | manifest display, empty state | shell, slot-replaceable |
| `index.css` | `--bg --surface --border --text --muted --accent --warn --danger` CSS vars | **theme tokens** (already var-based — good) |

The codebase is small (~680 lines TS, ~1,100 Rust) — restructuring now is cheap; after more features land it won't be.

---

## 2. Target architecture

```
clan/
├── crates/
│   ├── clan-sdk/                  (unchanged)
│   ├── clan-cli/                  (unchanged)
│   └── clan-viewer-tauri/         NEW — Rust core as a Tauri plugin
├── viewer/
│   ├── core/                      NEW — @clan/viewer-core (npm package)
│   └── shell/                     the current app/, rebuilt on core (reference shell)
└── examples/
    └── branded-viewer/            NEW — minimal white-label example (~60 lines)
```

### 2.1 Rust side: `clan-viewer-tauri` (Tauri plugin crate)

Move all commands, the `clan://` URI scheme handler, `AppState`, bindings resolution, adf-id injection, and strip logic out of the app binary into a library crate exposing the standard Tauri plugin pattern:

```rust
// downstream src-tauri/main.rs — the entire backend integration:
tauri::Builder::default()
    .plugin(clan_viewer_tauri::init(clan_viewer_tauri::Config {
        capabilities: Capabilities { write_ops: true, cli_ops: true },
    }))
    .run(tauri::generate_context!())
```

Existing unit tests (double-save guard, SAVE_COUNT, strip tests) move with the code. The reference shell's `main.rs` shrinks to the snippet above — proof the plugin is sufficient.

### 2.2 TS side: `@clan/viewer-core`

Exports:

- **`<ClanViewer>`** — the assembled application (toolbar + sidebar + document view + panels), fully driven by one `ViewerConfig` prop. This is the 90% path: rebrand + add controls without composing anything.
- **Primitives** — `<DocumentView>`, `<AgentPanel>`, `<Sidebar>`, `<Toolbar>`, `useClanDocument()`, `useClanCommands()` for the 10% who want their own layout.
- **Types** — `ViewerConfig`, `ToolbarAction`, `PanelExtension`, `ViewerCtx`, `ManifestInfo`, `OpenResult`.

---

## 3. The extension API (`ViewerConfig`)

```ts
interface ViewerConfig {
  brand?: BrandConfig
  toolbar?: ToolbarAction[]        // appended controls
  panels?: PanelExtension[]        // additional right-side panels
  docks?: DockExtension[]          // persistent regions: bottom bar, left rail, status strip
  slots?: SlotOverrides            // replace welcome/sidebar if desired
  features?: FeatureFlags          // hide built-ins: { edit?: false, agentPanel?: false, toolbar?: false, sidebar?: false, fileTree?: true }
                                   // fileTree (default off): built-in left-rail dock listing .clan files in the open
                                   // document's folder (via fs.listDir, manifest title/stage peek), click-to-open.
                                   // Deliberately NOT a container-internals tree: users see documents, never ZIP
                                   // entries — same illusion .docx maintains. Dogfooded via DockExtension.
  hooks?: LifecycleHooks
}

interface DockExtension {
  id: string
  region: 'bottom' | 'left' | 'statusbar'
  render: (ctx: ViewerCtx) => React.ReactNode   // always mounted, not toggled like panels
}

interface BrandConfig {
  appName: string                          // window title + wordmark text
  logo?: React.ComponentType               // replaces ClanMark
  theme?: Partial<Record<ThemeToken, string>> // the 8 CSS vars; merged over defaults
  attribution?: 'badge' | 'about' | 'none' // "Powered by CLAN" placement (default 'about')
}

interface ToolbarAction {
  id: string
  label: string
  icon?: React.ReactNode
  placement?: 'left' | 'right'             // default 'right', after built-ins
  when?: (ctx: ViewerCtx) => boolean       // e.g. only when a file is open / data has key X
  onClick: (ctx: ViewerCtx) => void | Promise<void>
}

interface PanelExtension {
  id: string
  title: string
  icon?: React.ReactNode
  render: (ctx: ViewerCtx) => React.ReactNode
}

interface LifecycleHooks {
  onDocumentOpened?: (ctx: ViewerCtx) => void
  onPatchSaved?: (patch: { id: string; content: string }, ctx: ViewerCtx) => void
  onValidation?: (report: string, ctx: ViewerCtx) => void
}
```

**Dogfood rule:** the built-in Agent panel and Edit button are reimplemented *through* `PanelExtension`/`ToolbarAction` respectively. If the built-ins can't be expressed in the public API, the API is too weak — this is the acceptance test for the refactor.

### 3.2 Declarative plugin layer (runtime, no code execution)

For "add buttons / remove UI / rebrand without recompiling": a JSON plugin format loaded at runtime from a `plugins/` folder (or File → Load Plugin). Plugins carry **no executable code** — they deserialize directly into `ViewerConfig`, so this layer costs almost nothing once §3 exists:

```json
{
  "name": "Ledgerline",
  "clan_viewer_api": 1,
  "brand": { "appName": "Ledgerline Reviews", "theme": { "accent": "#0e7a5f" } },
  "remove": ["edit", "agentPanel"],
  "buttons": [{
    "id": "approve", "label": "Approve", "icon": "check",
    "when": "doc.validation == 'OK'",
    "action": { "op": "patchDecision",
                "args": { "agent": "reviewer-ui", "action": "Approved batch",
                          "rationale": "Manual review passed" } },
    "then": { "toast": "Approved and recorded" }
  }]
}
```

Constraints that keep it safe: `action.op` may only name a typed `ClanOps` verb (still capability-gated Rust-side); `when` is a minimal expression grammar over `doc.*` and data keys (no eval); `icon` names a built-in icon set; `remove` maps to `FeatureFlags`. Anything needing real logic or custom rendering graduates to the build-time API. Sequenced actions (`action` as an array, e.g. patchData → patchDecision → toast) cover the common approve/sign-off pipelines without code.

**Layering summary:** JSON plugins (runtime, zero-code, shareable file) → `ViewerConfig` (build-time, typed, full control) → primitives (own layout). Each layer compiles down to the one below it.

### 3.3 Building full products on the chassis (the brief-maker pattern)

The target isn't only "CLAN viewer + extra buttons" — it's whole domain apps (brief maker, invoice reviewer, campaign planner) where CLAN is invisible plumbing. Reference shape: document canvas in the center, a **phase bar** in the bottom dock ("Push to Concept", "Push to Client Pitch"), and a **live agent chat** in the right panel that edits the document. Three core capabilities make this possible:

**1. Reactive document store (replaces App.tsx's manual state).** One store in core holds `{doc, data, stage, html}`; every mutation path — `ClanOps` call, agent write, external file change (fs watch on the open path), human edit-bridge patch — funnels through it and re-renders all subscribed components. Extensions read it via `useClanDocument()`. Without this, a side-panel agent that patches the file leaves a stale canvas; with it, "agent edits → canvas updates" is automatic. This is the single biggest addition to phase 2 and the heart of the chassis.

**2. Phases are just document state + ops.** A phase bar is a `DockExtension` reading `stage` from `agent/state.yaml` and advancing via sequenced ops — no workflow engine in core:

```tsx
const phases = ['brief', 'concept', 'client-pitch']
docks: [{ id: 'phase-bar', region: 'bottom', render: ctx => (
  <PhaseBar phases={phases} current={ctx.doc?.stage}
    onAdvance={async next => {
      await ctx.cli.patchState({ stage: next })
      await ctx.cli.patchDecision({ agent: 'brief-maker-ui',
        action: `Advanced to ${next}`, rationale: 'Phase gate passed' })
      await ctx.agent?.run(`Transform this document into the ${next} stage`)  // optional
    }} />
)}]
```

This is research/14's flow-11 metamorphosis (Brief → Concept Deck → Client Pitch) with buttons on it — the pipeline is already proven; the viewer becomes its cockpit. Schema migration per phase uses the same `--schema` path flow-11 exercised.

**3. Agent adapter — core is model-agnostic, hosts bring the brain.** Core does NOT ship an LLM integration. It ships the contract and does the CLAN-side work:

```ts
interface AgentAdapter {                       // host implements (Claude API, Agent SDK, local model…)
  run(req: { prompt: string; docContext: string }, onEvent: (e: AgentEvent) => void): Promise<AgentResult>
}
// core provides to the adapter:
//  - docContext: the same distilled injection `clan read agent` produces (guide-skipped, TOON data + chain)
//  - op application: AgentResult.ops (typed ClanOps calls) are applied + attributed + chain-recorded by core
//  - the store: every applied op re-renders the canvas live
ctx.agent?: { run(prompt: string): Promise<void>; history: AgentTurn[] }   // present when host registered an adapter
```

The agent panel UI itself is just a `PanelExtension` the host writes (or a stock `<AgentChatPanel adapter={...}>` we ship as an optional component). Security note: agent-proposed ops flow through the same capability gates and attribution rules as buttons — an agent write is a chain entry like any other (`agent: "brief-maker-agent"`), which is precisely CLAN's provenance pitch applied to the host's own product.

### 3.1 `ViewerCtx` — what extensions can touch

```ts
interface ViewerCtx {
  doc: { path: string; manifest: ManifestInfo; validation: string } | null
  read: {                                   // read-only, always available
    data(): Promise<string>                 // shared/data.yaml
    dataParsed<T = unknown>(): Promise<T>   // YAML-parsed convenience
    chain(): Promise<string>
    state(): Promise<string>
    context(): Promise<string>
    html(): Promise<string>
    entries(): Promise<{ path: string; size: number; role?: string }[]>  // container listing — API for host tooling /
    entry(path: string): Promise<string>    // dev inspector ONLY; never surfaced in default end-user UI (see fileTree note)
  }
  fs?: {                                    // present only with capabilities.fs_browse (read-only)
    listDir(dir?: string): Promise<{ path: string; manifest?: ManifestInfo }[]>  // .clan files only, shallow manifest peek
  }
  cli: ClanOps                              // §4 — gated by capabilities
  ui: {
    openFile(path?: string): Promise<void>
    reload(): Promise<void>                 // re-open current file from disk
    toast(msg: string, kind?: 'info' | 'warn' | 'error'): void
    setPanelOpen(id: string, open: boolean): void
  }
}
```

---

## 4. Pipeline/CLI hooks (`ClanOps`)

The hosts' main want: trigger CLAN operations from their own UI ("Approve & sign off" → `patch-decision` + `patch-data`; "Send to pipeline" → `fork`; "Finalize" → `merge` + `export-static`).

**Mechanism — link `clan-sdk`, don't shell out.** `clan-viewer-tauri` already builds in the workspace next to `clan-sdk`; the CLI's commands are thin wrappers over SDK calls. Typed plugin commands beat sidecar process management (no binary discovery, no PATH issues, no output parsing, identical behavior across OSes). A sidecar escape hatch (`cli.raw(args)`) is **out of scope for v1** — if a host needs an op we haven't typed, we type it.

```ts
interface ClanOps {
  validate(path?: string): Promise<string>
  exportStatic(path?: string): Promise<string>            // returns JSON
  render(path?: string): Promise<void>
  patchData(json: object, opts?: { namespace?: boolean }): Promise<void>
  patchDecision(d: { agent: string; action: string; rationale: string; pinned?: boolean }): Promise<void>
  patchHtml(patch: { selector: string; action: 'replace' | 'append'; html: string }): Promise<void>
  patchState(json: object): Promise<void>
  fork(opts: { agents: string[]; outputDir: string; contextDir?: string }): Promise<string[]>
  merge(branches: string[], opts: { output: string; policies?: Record<string, MergePolicy> }): Promise<string>
  readReport(path?: string): Promise<string>               // merge-report.yaml
}
```

**Capability gating.** Write/CLI ops exist only if the host enabled them in the Rust plugin config (`capabilities.write_ops`, `capabilities.cli_ops`). Disabled → commands aren't registered with Tauri at all (not merely hidden), so a compromised webview can't invoke them. Read ops are always on.

**Decision-chain hygiene.** Every `ClanOps` write op requires or auto-attaches attribution: `patchData` without an accompanying decision logs `agent: "viewer:<appName>"` — UI-driven mutations must be as auditable as agent-driven ones (same principle as F1).

---

## 5. What stays non-overridable (core guarantees)

Extensions and rebrands **cannot** replace: container open/validate, the sandboxed-iframe render path (null origin, no IPC), script/handler stripping rules, the edit bridge and patch persistence semantics, lineage/manifest truth in the sidebar data (display is replaceable; the values are not synthesizable). A `.clan` file must behave identically in every downstream viewer — that's the format promise, and it's what makes "viewable in any CLAN viewer" a credible claim.

---

## 6. Migration plan

| Phase | Work | Est. |
|---|---|---|
| 1 | Extract `crates/clan-viewer-tauri` plugin from `app/src-tauri/main.rs`; reference shell consumes it; all existing tests pass unchanged | 1–2 days |
| 2 | Extract `viewer/core` package: move components, introduce `ViewerConfig` + `ViewerCtx`; **reactive document store + fs watch (§3.3.1)**; re-express Agent panel + Edit button via the public API (dogfood test) | 3–4 days |
| 2b | `DockExtension` regions + `AgentAdapter` contract with op application/attribution (§3.3.3); optional stock `<AgentChatPanel>` | 1–2 days |
| 3 | `ClanOps` typed commands + capability gating in the plugin | 1–2 days |
| 4 | `examples/branded-viewer` — fictional brand ("Ledgerline Reviews"), custom theme, one custom panel (renders `shared/data.yaml` domain view), two toolbar actions (one calling `patchDecision`) | 1 day |
| 4b | `examples/brief-maker` — the §3.3 reference app: phase bar dock, agent chat panel with a stub adapter, flow-11-style stage transitions | 1–2 days |
| 5 | `viewer/core/README.md` extension guide + this doc updated to IMPLEMENTED | 0.5 day |
| 6 | Declarative JSON plugin loader (§3.2): folder scan, schema validation, `ViewerConfig` mapping, expression grammar for `when` | 1–2 days |

No format/spec changes. No CLI changes. `app/` path can remain as an alias or move to `viewer/shell` (housekeeping decision at phase 2).

## 7. Example downstream shell (the whole thing)

```tsx
import { ClanViewer } from '@clan/viewer-core'
import { ApproveIcon, LedgerLogo } from './brand'

export default () => (
  <ClanViewer config={{
    brand: { appName: 'Ledgerline Reviews', logo: LedgerLogo,
             theme: { accent: '#0e7a5f', bg: '#0b0f0d' } },
    toolbar: [{
      id: 'approve', label: 'Approve', icon: <ApproveIcon />,
      when: ctx => !!ctx.doc && ctx.doc.validation === 'OK',
      onClick: async ctx => {
        await ctx.cli.patchDecision({ agent: 'reviewer-ui', action: 'Approved invoice batch',
                                      rationale: 'Manual review passed' })
        await ctx.cli.patchData({ review_status: 'approved' })
        ctx.ui.toast('Approved and recorded in the chain')
      },
    }],
    panels: [{ id: 'totals', title: 'Batch Totals',
               render: ctx => <TotalsTable load={() => ctx.read.dataParsed()} /> }],
  }} />
)
```

## 8. Open questions

1. **npm distribution** — publish `@clan/viewer-core` (+ `clan-viewer-tauri` to crates.io) at v1.1, or keep workspace-only until two real downstream consumers exist? (Leaning: workspace-only first; publishing is a one-way door on API stability.)
2. **Attribution default** — is `'about'`-only CLAN attribution acceptable for MPL-2.0 + brand goals, or should the validation badge keep a small CLAN mark?
3. **Panel layout** — multiple open panels: tabs in one rail (proposed) vs side-by-side stacking?
4. **`features.edit = false`** — should disabling edit also drop `save_patch` registration Rust-side (capability symmetry)? Proposed: yes.
5. **Multi-document workspace** — the chassis currently assumes one open document. Queue/inbox apps (invoice approval stacks, pipeline monitors, fork/merge cockpits reviewing N branches) need list-many/open-many + tabs. Deferred: ship single-doc core first; design a `workspace` capability (multi-doc store keyed by path — the store in §3.3.1 is built per-document precisely so this composes later) when a real consumer needs it.
