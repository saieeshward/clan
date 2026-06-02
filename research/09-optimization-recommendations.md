# Optimisation Recommendations

Priority-ranked. Includes effort estimates and the rationale for each.

---

## Priority 1 — Critical Correctness (Fix Before Any Real Use)

### OPT-1: Fix Double-Save Architecture
**Effort**: 30 minutes  
**Impact**: Critical — every edit writes the ZIP twice

In `main.rs`, the `clan://patch` URI handler currently calls `do_save_patch()` AND emits `clan-patch-saved`. React receives the event and calls `invoke('save_patch')` again.

**Change**: URI handler should only emit. React does the actual save.

```rust
// main.rs — clan://patch handler (current)
let _ = do_save_patch(id.to_string(), content.to_string(), &*state);  // remove this
let _ = app.app_handle().emit("clan-patch-saved", ...);               // keep this

// Result: React handlePatch is the single save path
// patchInFlight mutex now actually works
```

---

### OPT-2: `patch-html` Must Exit Non-Zero on Zero Matches
**Effort**: 20 minutes  
**Impact**: Critical — silent data loss in pipelines

After applying the patch, check if the selector matched anything. If not, exit 2 and write to stderr.

```rust
// After applying patch, count replacements made
if replacement_count == 0 {
    eprintln!("patch-html: selector '{}' matched 0 elements — no patch applied", selector);
    std::process::exit(2);
}
```

Additionally: don't rewrite the ZIP at all if no changes were made (saves unnecessary I/O).

---

### OPT-3: Warn When `pack-html` Frontmatter Has No `structured:` Key
**Effort**: 20 minutes  
**Impact**: High — silent complete structured data loss, triggered in 75% of test agents

Check frontmatter for the `structured:` key after parsing. If frontmatter has top-level keys other than `decision:` but no `structured:` block, emit a warning.

```
WARNING: pack-html: frontmatter detected but no 'structured:' key found.
         Did you intend to wrap your data fields under 'structured:'?
         Example:
           structured:
             my_field: value
         Current top-level keys (ignored): stage, analyst, revenue
```

---

## Priority 2 — High Impact, Low Effort

### OPT-4: Add `.nvmrc` with Node 20
**Effort**: 2 minutes  
**Impact**: High — eliminates onboarding friction for every new developer

```bash
echo "20" > app/.nvmrc
```

Also add to `app/package.json`:
```json
"engines": { "node": ">=20" }
```

---

### OPT-5: Fix `clan create` Argument Inconsistency
**Effort**: 15 minutes  
**Impact**: Medium — UX friction on the first command every user runs

Add `--output` as an alias for the positional `<OUTPUT>` argument in `clan create`. Both forms should work:

```bash
clan create --title "..." --brief "..." output.clan        # keep (positional)
clan create --title "..." --brief "..." --output output.clan  # add (flag)
```

---

### OPT-6: Add `create` to `clan agent-help`
**Effort**: 5 minutes  
**Impact**: Medium — agents currently have no documented path to start a new document

Add one line to agent-help output:
```
NEW DOCUMENT:
  clan create --title "..." --brief "..." output.clan
```

Cost: ~12 tokens. An agent that needs to bootstrap a pipeline from scratch currently has no guidance.

---

## Priority 3 — Medium Impact, Medium Effort

### OPT-7: Add `--dry-run` to `patch-html`
**Effort**: 1 hour  
**Impact**: High — enables pipeline validation without side effects

```bash
clan patch-html --dry-run file.clan << 'EOF'
---
mode: patch-html
patch_selector: "[data-adf-id='title']"
patch_action: replace
---
<h1>New Title</h1>
EOF
# Output: dry-run: selector '[data-adf-id="title"]' matched 1 element (h1 at offset 4523)
# No file written.
```

Pairs naturally with OPT-2 (non-zero exit on zero matches) as a pre-flight check.

---

### OPT-8: Replace `clan://` Edit Bridge Polling with postMessage
**Effort**: 2–3 hours  
**Impact**: Medium — eliminates 300ms edit-mode lag, enables browser-based development

Current architecture:
```
React shell → invoke('set_edit_mode') → Rust state → iframe polls clan://edit-mode every 300ms
```

Proposed:
```
React shell → iframe.contentWindow.postMessage({type: 'set-edit-mode', active: true})
```

Benefits:
- Zero polling, immediate edit-mode activation
- Works in browser dev server (postMessage is standard)
- Removes the `clan://edit-mode` URI handler entirely
- For patching: iframe postMessages `{id, content}` to React shell → React invokes save once

This also fixes BUG-5 (browser dev experience) as a side effect.

---

### OPT-9: Queue Last Patch Instead of Dropping
**Effort**: 45 minutes  
**Impact**: Medium — prevents silent edit loss under rapid editing

```typescript
// App.tsx
const pendingPatch = useRef<{id: string, content: string} | null>(null)

async function handlePatch(id: string, content: string) {
  if (patchInFlight.current) {
    pendingPatch.current = {id, content}  // store instead of drop
    return
  }
  patchInFlight.current = true
  try {
    await invoke('save_patch', {id, content})
    const html = await invoke<string>('get_human_html')
    setHtmlContent(html)
    // Flush pending patch
    const pending = pendingPatch.current
    if (pending) {
      pendingPatch.current = null
      await handlePatch(pending.id, pending.content)
    }
  } finally {
    patchInFlight.current = false
  }
}
```

---

### OPT-10: Fix Empty iframe `src` on Mount
**Effort**: 10 minutes  
**Impact**: Low — eliminates a browser console error

```typescript
// DocumentView.tsx
const [iframeSrc, setIframeSrc] = useState<string | null>(null)  // null instead of ''

// In JSX:
{iframeSrc && (
  <iframe
    src={iframeSrc}
    sandbox="allow-scripts allow-popups"
    ...
  />
)}
```

---

## Priority 4 — Low Impact, Worth Doing

### OPT-11: TOON Should Preserve Stage-Level Grouping
**Effort**: 2–3 hours  
**Impact**: Low — improves human readability, no agent impact

Current: all fields sorted alphabetically across all stages.  
Proposed: sort alphabetically within each "stage block" — fields added by each agent grouped together with a comment separator.

```
# stage 1 — market-researcher
analysis_title: Irish AdTech OS
analyst: Market Researcher
competitive_landscape [5]
  ...

# stage 2 — risk-analyst  
go_to_market
  phase_1: Dublin-first...
overall_risk_rating: MEDIUM
```

This preserves the full audit value of TOON while making human debugging practical.

---

### OPT-12: Parse `patches` in `export-static` to JSON Array
**Effort**: 30 minutes  
**Impact**: Low — API consistency

```json
// Current
"patches": "patches:\n- id: exec-summary\n  content: ..."

// Proposed
"patches": [
  {"id": "exec-summary", "content": "<p>...partner override...</p>"},
  {"id": "stat-anpost", "content": "EXEMPT"}
]
```

---

### OPT-13: Hash-Based Auto-Inject IDs
**Effort**: 1 hour  
**Impact**: Low — prevents patch misapplication after HTML restructuring

```rust
// Instead of sequential: auto-p-0, auto-p-1, auto-p-2
// Use: auto-p-{short_hash_of_content_context}
fn stable_id(tag: &str, content: &str, parent_context: &str) -> String {
    let hash_input = format!("{}{}{}", tag, content, parent_context);
    let h = fnv_hash(hash_input.as_bytes()) & 0xFFFF;
    format!("auto-{}-{:04x}", tag, h)
}
```

---

### OPT-14: Add `--verbose` Flag to CLI
**Effort**: 2 hours  
**Impact**: Low — debugging aid

`--verbose` / `-v` on `pack`, `pack-html`, `patch-html` prints to stderr:
- Which files are being written to the ZIP
- What the parsed frontmatter structured data looks like
- What selector matched (or didn't) during `patch-html`
- What TOON output was generated

---

### OPT-15: Guard `get_human_html` Against React Strict Mode Double-Invocation
**Effort**: 15 minutes  
**Impact**: Cosmetic

```typescript
// App.tsx
const hasLoaded = useRef(false)

useEffect(() => {
  if (hasLoaded.current) return
  hasLoaded.current = true
  invoke<string | null>('get_default_path').then(...)
}, [])
```

---

## Summary Table

| # | Recommendation | Effort | Impact | Category |
|---|---|---|---|---|
| OPT-1 | Fix double-save architecture | 30 min | Critical | Bug fix |
| OPT-2 | `patch-html` exit non-zero on 0 matches | 20 min | Critical | Bug fix |
| OPT-3 | Warn on missing `structured:` key | 20 min | High | UX |
| OPT-4 | Add `.nvmrc` with Node 20 | 2 min | High | Config |
| OPT-5 | `clan create` `--output` flag alias | 15 min | Medium | UX |
| OPT-6 | Add `create` to `agent-help` | 5 min | Medium | Docs |
| OPT-7 | `--dry-run` for `patch-html` | 1 hr | High | Feature |
| OPT-8 | Replace clan:// polling with postMessage | 2–3 hrs | Medium | Architecture |
| OPT-9 | Queue last patch instead of drop | 45 min | Medium | UX |
| OPT-10 | Fix empty iframe src on mount | 10 min | Low | Bug fix |
| OPT-11 | TOON stage-level grouping | 2–3 hrs | Low | Format |
| OPT-12 | `export-static` patches as JSON array | 30 min | Low | API |
| OPT-13 | Hash-based auto-inject IDs | 1 hr | Low | Stability |
| OPT-14 | `--verbose` flag on CLI | 2 hrs | Low | DX |
| OPT-15 | Guard against Strict Mode double-invoke | 15 min | Cosmetic | Bug fix |
