# Desktop App Test Results

## Environment

- **App stack**: Tauri v2 + React 18 + TypeScript + Vite 8
- **Rust binary**: `clan-app v1.0.0`
- **Test file**: `/tmp/adtech-ie-deep/branch-financial.clan` (18KB, full financial model HTML)
- **Launch command**: `cargo tauri dev -- -- /tmp/adtech-ie-deep/branch-financial.clan`

---

## Pre-Launch: Node.js Version Issue

### Discovery
```bash
node --version         # → v16.19.1 (system default)
npx vite --version     # → Error: Vite requires Node.js version 20.19+ or 22.12+
```

`package.json` pins `"vite": "^8.0.12"`. Vite 8 dropped support for Node < 20. The project has no `.nvmrc`, no `.node-version`, and no `engines` field in `package.json`.

**Impact**: Any developer with Node 16 or 18 cannot run the app at all. The error message from Vite is informative but only appears after `npm run dev` is invoked — there's no pre-flight check.

### Resolution
Switch to Node 20 via nvm:
```bash
nvm use 20  # → Now using node v20.20.2
npm install # → No issues
```

With Node 20, Vite 8 starts in 143ms. The issue is entirely in environment setup, not the code.

---

## Launch Sequence

```bash
export NVM_DIR="$HOME/.nvm" && source "$NVM_DIR/nvm.sh" && nvm use 20
export PATH="$HOME/.cargo/bin:$PATH"
cargo tauri dev -- -- /tmp/adtech-ie-deep/branch-financial.clan
```

**Build output**:
```
Running BeforeDevCommand (`npm run dev`)
  VITE v8.0.14  ready in 142 ms
  ➜  Local:   http://localhost:1420/
Running DevCommand (`cargo run --no-default-features --color always -- /tmp/adtech-ie-deep/branch-financial.clan`)
Compiling clan-app v1.0.0
  Building [=======================> ] 453/454: clan-app(bin)
  Finished `dev` profile in 2.98s
  Running `/Users/saieeshwar/.../clan-app /tmp/adtech-ie-deep/branch-financial.clan`
```

First run: ~3s compile (incremental). Cold start (no cached artifacts): ~45–90s.

---

## JavaScript Errors on Load

Two errors appeared in the Vite console immediately after launch:

### Error 1: Empty iframe src
```
An empty string ("") was passed to the src attribute. This may cause the browser
to download the whole page again over the network.
```

**Cause**: In `DocumentView.tsx`, `iframeSrc` state is initialised to `''`. The iframe renders immediately with `src=""` before the `useEffect` that sets the real `clan://document?t=...` URL runs. React renders the empty string, browser complains.

**Fix**: Initialise `iframeSrc` to `null` and conditionally render the iframe: `{iframeSrc && <iframe src={iframeSrc} .../>}`.

### Error 2: Tauri event unlisten race
```
TypeError: undefined is not an object (evaluating 'listeners[eventId].handlerId')
  at unregisterListener
  at _unlisten @tauri-apps/api/event.js
```

**Cause**: The `listen('clan-patch-saved', ...)` in `DocumentView.tsx` returns an unlistener promise. The cleanup function in `useEffect` calls `unlistenPromise.then(unlisten => unlisten())`. In React Strict Mode (development), effects are torn down and re-run immediately. The unlisten is called before the listener is fully registered, causing the `handlers[eventId]` lookup to fail.

**Fix**: Guard the unlisten with a mounted flag, or use `await` at the start of the effect to ensure the listener is registered before the cleanup can run.

---

## File Auto-Load Verification

From `clan-debug.log` (`/tmp/clan-debug.log`), the most recent entries at the time of launch:

```
[1780329972257] get_human_html: called
[1780329972267] get_human_html: called
```

These timestamps correspond to our launch time (2026-06-01 ~17:06:12 UTC). `get_human_html` was called twice within 10ms — consistent with React Strict Mode double-invoking effects. No `apply_patches` entries followed because `branch-financial.clan` has no existing patches.

**Conclusion**: The CLI argument (`-- -- /path/to/file.clan`) is correctly parsed by `main.rs:422-423`, stored as `default_path`, and `App.tsx:45-53` correctly calls `get_default_path` → `handleOpenFile` on mount. **Auto-load works.**

---

## Edit Mode & Patch Pipeline — Evidence from Debug Log

The debug log contains evidence from a prior active session with a different `.clan` file (an Irish investment guide). This provides real-world evidence of the edit pipeline working — and its bugs.

### Normal edit flow (working correctly)
```
[1780239346425] get_human_html: called
[1780239346432] apply_patches: id="hero-title" tag="h1" content_start=23798 close_pos=23861
                old="How to Invest<br/>€100 in Dublin"…
                new="How to Invest €100 in Ireland"…
```
The patch is found, the element is located by `data-adf-id`, and the content is replaced correctly.

### Double-Save Bug — Live Confirmation
```
[1780328494518] save_patch: id="memo-title" content="Paylane Technologies"…
[1780328494537] save_patch: done, file repacked. id="memo-title"
[1780328494539] save_patch: id="memo-title" content="Paylane Technologies"…    ← DUPLICATE
[1780328494553] save_patch: done, file repacked. id="memo-title"               ← DUPLICATE
[1780328494554] get_human_html: called
```

**"memo-title" is saved TWICE at timestamps 518 and 539 — 21ms apart, identical content.**

This confirms the double-save architecture bug: the `clan://patch` URI handler saves to disk AND emits `clan-patch-saved`. React receives the event and calls `invoke('save_patch')` again — a second save with identical content.

**Note**: For this specific case (`replace` semantics), the double-save is idempotent — the same content is applied twice to `human/patches.yaml`, overwriting the same key. However:
1. The file is repacked twice unnecessarily (ZIP re-written twice per edit)
2. The `patchInFlight` mutex in React is bypassed (the first save happens before React even sees the event)
3. For `append` semantics this would double the appended content

### Silent Patch Failure — Live Confirmation
```
[1780328474034] apply_patches: id=".vc-left" NOT FOUND in HTML
```

A patch with `id=".vc-left"` (a CSS class selector used as a data-adf-id value) was applied against an HTML document that doesn't contain `data-adf-id=".vc-left"`. The runtime log shows "NOT FOUND" but the calling code (from `clan patch-html`) exited 0 with "Patched in-place" message.

**This confirms the silent failure bug is present in both the CLI and the runtime patch path.**

---

## CSP and Security Observations

`tauri.conf.json` CSP:
```
script-src 'self' 'unsafe-inline' clan:;
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com ...;
```

`'unsafe-inline'` is intentional — the edit bridge is injected as an inline `<script>` tag into the iframe's HTML. This cannot be removed without changing the bridge injection architecture.

**XSS surface**: If an agent produces HTML with inline scripts containing malicious content, `'unsafe-inline'` permits execution. The iframe `sandbox="allow-scripts allow-popups"` limits but does not eliminate this surface — `allow-scripts` re-enables inline execution within the sandbox.

**Note**: The iframe sandbox does prevent Tauri IPC access from within the iframe (no `__TAURI__` in the sandboxed iframe), so the agent-supplied HTML cannot call Rust commands. The risk is UI-level XSS within the iframe only.

---

## `clan://` Protocol in Browser Dev Server

When running `vite dev` (browser, not Tauri):

- `fetch('clan://edit-mode')` — silently fails (caught by `.catch(() => {})`)
- `fetch('clan://patch', ...)` — logs `console.error` but edit is discarded
- `invoke('update_preview_html', ...)` — throws immediately (no `window.__TAURI__` guard)
- Result: iframe stays blank, no error surface for the developer

**Impact**: The app is completely non-functional in a browser. No degraded mode, no feature detection, no `window.__TAURI__` check anywhere in the codebase. Developers must do a full Tauri compile to see any UI change — on a cold Rust build that's 45–90 seconds per iteration.

---

## Summary of App Issues

| # | Issue | Severity | Source |
|---|-------|----------|--------|
| 1 | Node 16 / Vite 8 incompatibility | High | `package.json`, no `.nvmrc` |
| 2 | Double-save on every edit | High | `main.rs:446-460`, `App.tsx:85-103` |
| 3 | `clan://` fails in browser, no fallback | Medium | `DocumentView.tsx` |
| 4 | Concurrent patch silently dropped | Medium | `App.tsx:88-91` |
| 5 | Empty iframe src on mount | Low | `DocumentView.tsx` (init state) |
| 6 | Unlisten race in Strict Mode | Low | `DocumentView.tsx` useEffect |
| 7 | `unsafe-inline` CSP (intentional but notable) | Info | `tauri.conf.json` |
