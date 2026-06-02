# Confirmed Bugs and Issues

All bugs confirmed through direct testing. Reproduction steps provided for each.

---

## BUG-1 [HIGH] — `patch-html` Silent Success on Non-Matching Selector

**Component**: CLI — `clan patch-html` subcommand  
**Severity**: High — silent data loss in automated pipelines  
**Confirmed**: Yes (twice: CLI test + live Tauri debug log)

### Reproduction
```bash
printf -- '---\nmode: patch-html\npatch_selector: ".nonexistent"\npatch_action: replace\n---\n<p>Patch content</p>\n' \
  | clan patch-html file.clan -
echo "Exit: $?"           # → 0
clan read human file.clan | grep "Patch content"  # → (empty — no match)
```

File mtime is updated (ZIP was rewritten), "Patched in-place" is printed, exit code is 0. The patch content does not appear anywhere in the output.

### Also confirmed in live app log
```
[1780328474034] apply_patches: id=".vc-left" NOT FOUND in HTML
```
The runtime logs "NOT FOUND" internally but the calling code reports success.

### Expected behaviour
- Exit code 2
- Stderr message: `patch-html: selector '.nonexistent' matched 0 elements — no patch applied`
- File should NOT be rewritten (no ZIP write if no change)

### Impact
Any automated pipeline using `patch-html` with a typo'd selector gets a false success signal. The pipeline continues under the assumption the patch was applied. In a multi-stage pipeline, subsequent agents will not see the intended override.

### Code location
The selector match count is not checked before or after applying; the "Patched in-place" message is unconditional.

---

## BUG-2 [HIGH] — Double-Save on Every Edit (Desktop App)

**Component**: `main.rs` URI handler + `App.tsx` event listener  
**Severity**: High — every edit writes the `.clan` file twice, bypasses concurrency guard  
**Confirmed**: Yes (debug log timestamps 518 and 539, 21ms apart, identical content)

### The flow
1. User edits element → `blur` in iframe
2. Iframe: `fetch('clan://patch', {id, content})` → Rust URI handler
3. Rust URI handler (`main.rs:446-460`): calls `do_save_patch()` → writes to disk → emits `clan-patch-saved` event
4. React `DocumentView`: `listen('clan-patch-saved')` fires → calls `onPatch(id, content)`
5. `App.tsx:handlePatch`: calls `invoke('save_patch', {id, content})` → calls `do_save_patch()` **again**

### Debug log evidence
```
[1780328494518] save_patch: id="memo-title" content="Paylane Technologies"…
[1780328494537] save_patch: done, file repacked. id="memo-title"
[1780328494539] save_patch: id="memo-title" content="Paylane Technologies"…  ← DUPLICATE
[1780328494553] save_patch: done, file repacked. id="memo-title"             ← DUPLICATE
```

### Consequences
- ZIP repacked twice per edit (unnecessary I/O)
- `patchInFlight` mutex in `App.tsx` is bypassed — the first save happens at Rust URI handler level before React sees the event
- For `replace` patches: idempotent but wasteful
- For `append` patches: would double the appended content
- Race condition window: if user triggers a second edit in the 21ms between first and second save, the second save overwrites with stale state

### Code locations
- `main.rs:446-460`: URI handler for `clan://patch`
- `App.tsx:85-103`: `handlePatch` function
- `DocumentView.tsx:89-100`: `listen('clan-patch-saved')` → `onPatch`

### Fix
The URI handler should do ONE of:
- **Option A (preferred)**: Save only, do NOT emit. Re-fetch HTML in the URI handler response or via polling.
- **Option B**: Emit only, do NOT save. Let React handle the save via `invoke`.

---

## BUG-3 [HIGH] — `pack-html` Silently Discards Structured Data When `structured:` Key Absent

**Component**: CLI — `clan pack-html` frontmatter parser  
**Severity**: High — complete structured data loss with no warning  
**Confirmed**: Yes — 3 of 4 agents in the 6-agent fan-out wrote flat frontmatter; `clan read data` returned only `$schema`

### Reproduction
```html
<!-- bad-frontmatter.html -->
---
stage: "My Stage"
analyst: "Agent Name"
revenue: 500000
---
<!DOCTYPE html>
<html>...</html>
```

```bash
clan pack-html --delta "test" --output test.clan root.clan bad-frontmatter.html
clan read data test.clan
# Output: $schema: spec/schemas/document.schema.json
# (stage, analyst, revenue are all gone)
```

### Why it happens
`clan pack-html` reads frontmatter looking for a top-level `structured:` key. If absent, it treats the frontmatter as having no structured data. It does parse `decision:` at the root level (that works). Only the structured data block requires the `structured:` wrapper.

### Why it's hard to avoid
YAML authors naturally write flat top-level keys. The `structured:` wrapper is a CLAN-specific convention that is not obvious from the HTML file context. An agent writing an HTML file with embedded data will naturally write:
```yaml
---
my_field: value
decision:
  agent: "my-agent"
---
```
Not:
```yaml
---
structured:
  my_field: value
decision:
  agent: "my-agent"
---
```

The CLAN agent guide documents this correctly, but agents working in HTML context often do not re-read the full guide before writing their frontmatter.

### Impact
An agent whose output is silently stripped of structured data produces a `.clan` file that appears valid (`clan validate` passes) but has no data for downstream agents. A subsequent synthesis agent calling `clan read agent` on that file gets an empty data block.

### Fix
When frontmatter is present but has no `structured:` key, print to stderr:
```
WARNING: pack-html: frontmatter found but no 'structured:' key detected.
All top-level keys were ignored. Did you mean to wrap your fields under 'structured:'?
  structured:
    your_field: value
```
Still pack (don't break the pipeline), but warn so the operator can investigate.

---

## BUG-4 [MEDIUM] — `clan create` Uses Positional Arg; `clan pack`/`pack-html` Use `--output` Flag

**Component**: CLI — argument style inconsistency  
**Severity**: Medium — UX friction, caused confusion during testing  
**Confirmed**: Yes — first test attempt with `--output` on `create` failed

### Details
```
clan create --title "..." --brief "..." output.clan        # ← positional (works)
clan create --title "..." --brief "..." --output out.clan  # ← fails: "unexpected argument '--output'"
clan pack --output output.clan parent.clan input.json      # ← flag (works)
clan pack output.clan parent.clan input.json               # ← would fail
```

### Fix
Add `--output` as an alias for the positional argument in `clan create`. Or document the asymmetry prominently in `clan create --help`.

---

## BUG-5 [MEDIUM] — `clan://` Protocol Fails Silently in Browser Dev Server

**Component**: `DocumentView.tsx` — edit bridge  
**Severity**: Medium — app entirely non-functional in browser, no developer fallback  
**Confirmed**: Verified by code analysis; not directly tested in browser during this session

### Details
All Tauri IPC calls (`invoke`, `listen`) throw immediately when `window.__TAURI__` is absent (browser context). No guard exists anywhere in the codebase.

The `clan://edit-mode` poll silently fails (`.catch(() => {})`). The `clan://patch` fetch logs `console.error` but the edit is silently discarded. The iframe stays blank because `invoke('update_preview_html', ...)` throws before setting `iframeSrc`.

### Impact
- Cannot run `npm run dev` and see the UI without a full Tauri compile (45–90s cold, 3s hot)
- Developers testing UI changes are forced into a slow iteration loop
- Demos using a browser are impossible without the desktop app installed

### Fix
Add `window.__TAURI__` check in `DocumentView.tsx`. In browser mode, fall back to postMessage-based IPC between React shell and iframe. This also resolves the `clan://` polling issue entirely.

---

## BUG-6 [MEDIUM] — Concurrent Patch Edit Silently Dropped

**Component**: `App.tsx` — `handlePatch` function  
**Severity**: Medium — user edit is lost without feedback  
**Confirmed**: Code analysis

### Details
```typescript
// App.tsx:88-91
if (patchInFlight.current) {
  console.warn('clan: patch dropped (previous save still in flight)', { id })
  return  // ← edit is gone
}
```

If a user edits element A, then immediately edits element B before A's save completes, B's edit is silently dropped with only a `console.warn`. The user sees no UI indication that a save is in progress.

### Fix
- Store the last-received patch in a `pendingPatch` ref
- After the in-flight save completes, flush pending patch
- Add a visible in-flight indicator (pulsing save icon, subtle top border animation)

---

## BUG-7 [LOW] — Auto-Inject IDs Are Positionally Unstable

**Component**: `main.rs` — `auto_inject_adf_ids` / `inject_ids_for_tag`  
**Severity**: Low — affects patches on auto-ID elements when HTML structure changes

### Details
Auto-injected IDs are sequential per tag type: `auto-p-0`, `auto-p-1`, `auto-p-2`...

If an agent adds a new `<p>` element before position 3 in a subsequent stage, all `auto-p-3+` IDs shift by one. A human patch stored against `auto-p-3` now applies to what was previously the 4th paragraph — a different element than intended.

### Impact
Only affects patches on elements that did not have explicit `data-adf-id` attributes from the agent. Agents following the guide correctly annotate all editable elements, so this only affects HTML authored without annotations.

### Fix
Hash-based IDs: `auto-p-{hash of text content + sibling index}`. Stable across re-orderings as long as content doesn't change.

---

## BUG-8 [LOW] — `export-static` Patches Field Is Raw YAML String

**Component**: CLI — `clan export-static`  
**Severity**: Low — API inconsistency

### Details
```json
{
  "shared_data": { ...parsed JSON object... },    ← clean
  "patches": "patches:\n- id: exec-summary\n  content: ...", ← raw YAML string
}
```

`shared_data` is a fully parsed JSON object. `patches` is a raw YAML string requiring a YAML parser.

### Fix
Parse patches to `[{id: string, content: string}]` JSON array in the export.

---

## ISSUE-9 — React Strict Mode Double-Invocation

**Component**: `App.tsx` / `DocumentView.tsx` — React Strict Mode effects  
**Severity**: Cosmetic/performance (dev only)

React Strict Mode double-invokes effects in development. Observed in debug log:
```
[1780329972257] get_human_html: called
[1780329972267] get_human_html: called  ← 10ms later, same operation
```

Two full HTML reads + patch applications per mount. Harmless in production (Strict Mode is dev-only) but adds latency to the dev iteration cycle.

### Fix
Guard the `useEffect` that calls `get_human_html` with a `mounted` ref flag.

---

## ISSUE-10 — No Node Version Pinning

**Component**: `app/` directory  
**Severity**: High (onboarding friction)

`package.json` has `"vite": "^8.0.12"` (requires Node 20+) but no `.nvmrc`, `.node-version`, or `"engines"` field. System default on the test machine was Node 16.19.1, causing immediate failure.

### Fix (2 minutes)
```bash
echo "20" > app/.nvmrc
```
And add to `package.json`:
```json
"engines": { "node": ">=20" }
```
