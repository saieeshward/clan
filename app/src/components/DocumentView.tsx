import { useEffect, useRef, useCallback } from 'react'
import type { ManifestInfo } from '../App'

interface Props {
  htmlContent: string
  hasHumanView: boolean
  manifest: ManifestInfo
  editMode: boolean
  onPatch: (id: string, content: string) => void
}

// Edit bridge script injected by the viewer (trusted — not from the agent).
// Makes elements with data-adf-id contenteditable and sends postMessage on blur.
const EDIT_BRIDGE = `
(function() {
  // Idempotent — safe to call multiple times (e.g. React hot-reload).
  if (window.__clan_bridge_listening) return;
  window.__clan_bridge_listening = true;

  function activateEditing() {
    document.querySelectorAll('[data-adf-id]').forEach(function(el) {
      if (el.dataset.clanEditing) return; // already wired
      el.dataset.clanEditing = '1';
      el.setAttribute('contenteditable', 'true');
      el.style.outline = '2px solid rgba(99,102,241,0.7)';
      el.style.outlineOffset = '2px';
      el.style.borderRadius = '3px';
      el.style.cursor = 'text';
      el.style.transition = 'outline 0.15s';
      el.addEventListener('blur', function onBlur() {
        window.parent.postMessage({
          type: 'clan-edit',
          id: el.getAttribute('data-adf-id'),
          content: el.innerText.trim()
        }, '*');
      });
    });
    // Toast hint
    showHint('Click any highlighted element to edit it');
  }

  function deactivateEditing() {
    document.querySelectorAll('[data-adf-id]').forEach(function(el) {
      el.removeAttribute('contenteditable');
      el.removeAttribute('data-clan-editing');
      el.style.outline = '';
      el.style.outlineOffset = '';
      el.style.cursor = '';
    });
    removeHint();
  }

  function showHint(msg) {
    removeHint();
    var hint = document.createElement('div');
    hint.id = '__clan_hint';
    hint.textContent = msg;
    hint.style.cssText = 'position:fixed;bottom:20px;left:50%;transform:translateX(-50%);' +
      'background:rgba(99,102,241,0.9);color:#fff;padding:8px 18px;border-radius:20px;' +
      'font-size:13px;font-family:system-ui;z-index:99999;pointer-events:none;' +
      'box-shadow:0 4px 16px rgba(0,0,0,0.4);';
    document.body.appendChild(hint);
    setTimeout(removeHint, 3000);
  }

  function removeHint() {
    var h = document.getElementById('__clan_hint');
    if (h) h.remove();
  }

  window.addEventListener('message', function(e) {
    if (!e.data || e.data.type !== 'clan-edit-mode') return;
    if (e.data.active) activateEditing();
    else deactivateEditing();
  });
})();
`

export default function DocumentView({ htmlContent, hasHumanView, manifest, editMode, onPatch }: Props) {
  const iframeRef = useRef<HTMLIFrameElement>(null)
  const editModeRef = useRef(editMode)
  editModeRef.current = editMode

  // Send edit mode state to the iframe. Safe to call at any time.
  const sendEditMode = useCallback((active: boolean) => {
    iframeRef.current?.contentWindow?.postMessage(
      { type: 'clan-edit-mode', active },
      '*'
    )
  }, [])

  // When editMode changes, push the new state in.
  useEffect(() => {
    sendEditMode(editMode)
  }, [editMode, sendEditMode])

  // When the iframe finishes loading, push the current editMode so it's
  // never missed by a race condition between render and script execution.
  const handleIframeLoad = useCallback(() => {
    sendEditMode(editModeRef.current)
  }, [sendEditMode])

  // Listen for patch messages from the edit bridge inside the iframe.
  useEffect(() => {
    function handleMessage(e: MessageEvent) {
      if (
        e.data?.type === 'clan-edit' &&
        typeof e.data.id === 'string' &&
        typeof e.data.content === 'string'
      ) {
        onPatch(e.data.id, e.data.content)
      }
    }
    window.addEventListener('message', handleMessage)
    return () => window.removeEventListener('message', handleMessage)
  }, [onPatch])

  if (!hasHumanView) {
    return (
      <div style={{ padding: 40, color: 'var(--muted)', maxWidth: 600, margin: '0 auto' }}>
        <h2 style={{ color: 'var(--text)', marginBottom: 12 }}>{manifest.title}</h2>
        <p>This .clan file has no human view yet — awaiting first agent pass.</p>
        <pre style={{ marginTop: 12, background: 'var(--surface)', padding: 14, borderRadius: 6, fontSize: 12, border: '1px solid var(--border)' }}>
          {`clan read agent <file.clan>\nclan pack --output next.clan parent.clan output.json`}
        </pre>
      </div>
    )
  }

  // Detect whether the agent produced a full HTML document or a fragment.
  const isFullDoc = /^\s*<!doctype\s+html/i.test(htmlContent) || /^\s*<html/i.test(htmlContent)

  const srcDoc = isFullDoc
    ? injectIntoFullDoc(htmlContent)
    : wrapFragment(htmlContent)

  return (
    <iframe
      ref={iframeRef}
      srcDoc={srcDoc}
      onLoad={handleIframeLoad}
      style={{ width: '100%', flex: 1, border: 'none', background: '#0f1117' }}
      sandbox="allow-scripts allow-popups"
      title={manifest.title}
    />
  )
}

/** Inject the edit bridge into an agent-provided full HTML document. */
function injectIntoFullDoc(html: string): string {
  const bridge = `<script>${EDIT_BRIDGE}</script>`
  // Inject just before </body> if present, otherwise append.
  if (/<\/body>/i.test(html)) {
    return html.replace(/<\/body>/i, `${bridge}</body>`)
  }
  return html + bridge
}

/**
 * Wrap an HTML fragment in a minimal full document + edit bridge.
 * Intentionally bare — agents that provide their own styles (inline <style> or
 * via styles.css injected by main.rs) should not be clobbered by viewer defaults.
 */
function wrapFragment(fragment: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    html { scroll-behavior: smooth; }
    body {
      background: #0f1117;
      color: #e2e8f0;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
      font-size: 15px;
      line-height: 1.65;
      -webkit-font-smoothing: antialiased;
    }
    ::-webkit-scrollbar { width: 6px; }
    ::-webkit-scrollbar-thumb { background: #1e2d45; border-radius: 3px; }
  </style>
</head>
<body>
${fragment}
<script>${EDIT_BRIDGE}</script>
</body>
</html>`
}
