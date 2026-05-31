import { useEffect, useRef } from 'react'
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
  if (window.__clan_edit_bridge_active) return;
  window.__clan_edit_bridge_active = true;

  function activateEditing() {
    document.querySelectorAll('[data-adf-id]').forEach(function(el) {
      el.setAttribute('contenteditable', 'true');
      el.style.outline = '2px solid rgba(99,102,241,0.6)';
      el.style.borderRadius = '3px';
      el.style.minHeight = '1em';
      el.style.cursor = 'text';
      el.addEventListener('blur', function() {
        window.parent.postMessage({
          type: 'clan-edit',
          id: el.getAttribute('data-adf-id'),
          content: el.innerText
        }, '*');
      });
    });
  }

  function deactivateEditing() {
    document.querySelectorAll('[data-adf-id]').forEach(function(el) {
      el.removeAttribute('contenteditable');
      el.style.outline = '';
      el.style.cursor = '';
    });
  }

  window.addEventListener('message', function(e) {
    if (e.data && e.data.type === 'clan-edit-mode') {
      if (e.data.active) activateEditing();
      else deactivateEditing();
    }
  });
})();
`

export default function DocumentView({ htmlContent, hasHumanView, manifest, editMode, onPatch }: Props) {
  const iframeRef = useRef<HTMLIFrameElement>(null)

  // Forward edit mode changes into the iframe via postMessage.
  useEffect(() => {
    iframeRef.current?.contentWindow?.postMessage(
      { type: 'clan-edit-mode', active: editMode },
      '*'
    )
  }, [editMode])

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
      style={{ width: '100%', flex: 1, border: 'none', background: '#0f1117' }}
      // allow-scripts: needed for the edit bridge we inject (trusted code).
      // allow-same-origin: needed so postMessage works cross-frame.
      // Scripts from the agent are already stripped by the SDK before storage.
      sandbox="allow-scripts allow-same-origin allow-popups"
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

/** Wrap an HTML fragment in a full document with base styles + edit bridge. */
function wrapFragment(fragment: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    :root {
      --bg:#0f1117; --bg-card:#141922; --bg-card2:#1a2032;
      --border:#1e2d45; --border2:#2a3350;
      --text:#e2e8f0; --text-muted:#94a3b8; --text-dim:#64748b;
      --accent:#6366f1; --accent-light:#818cf8;
      --green:#4ade80; --yellow:#fbbf24; --red:#f87171;
      --radius:10px; --radius-sm:6px;
      --shadow:0 4px 24px rgba(0,0,0,0.4);
    }
    html { scroll-behavior: smooth; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', system-ui, sans-serif;
      background: var(--bg); color: var(--text);
      line-height: 1.65; font-size: 15px;
      -webkit-font-smoothing: antialiased;
    }
    h1 { font-size: clamp(1.8rem,4vw,3rem); font-weight: 800; letter-spacing: -0.03em; color: #fff; line-height: 1.1; margin: 0 0 12px; }
    h2 { font-size: 1.4rem; font-weight: 700; color: #f1f5f9; letter-spacing: -0.02em; margin: 32px 0 10px; }
    h2::after { content:''; display:block; width:32px; height:3px; background:var(--accent); border-radius:2px; margin-top:8px; }
    h3 { font-size: 1.05rem; font-weight: 600; color: #cbd5e1; margin: 20px 0 8px; }
    h4 { font-size: 0.85rem; font-weight: 600; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.07em; margin: 16px 0 6px; }
    p { margin: 0 0 14px; color: var(--text-muted); font-size: 0.95rem; line-height: 1.7; }
    strong { color: #f1f5f9; font-weight: 600; }
    ul, ol { padding-left: 22px; color: var(--text-muted); margin: 0 0 14px; }
    li { margin: 5px 0; font-size: 0.92rem; line-height: 1.6; }
    code { background:#1a2032; border:1px solid var(--border); color:var(--accent-light); padding:2px 7px; border-radius:4px; font-size:0.82em; font-family:'SF Mono','Fira Code',Consolas,monospace; }
    table { width:100%; border-collapse:collapse; margin:16px 0 24px; border-radius:var(--radius); overflow:hidden; border:1px solid var(--border); }
    thead tr { background:rgba(99,102,241,0.12); }
    th { color:var(--accent-light); text-align:left; padding:11px 14px; font-size:0.75rem; font-weight:700; text-transform:uppercase; letter-spacing:0.07em; border-bottom:1px solid var(--border); }
    td { padding:11px 14px; border-bottom:1px solid rgba(30,45,69,0.7); color:var(--text-muted); font-size:0.88rem; }
    tr:last-child td { border-bottom:none; }
    tbody tr:hover { background:rgba(99,102,241,0.04); }
    td:first-child { color:#f1f5f9; font-weight:500; }
    .card,.summary-card,[class*="-card"] { background:var(--bg-card); border:1px solid var(--border); border-radius:var(--radius); padding:24px; box-shadow:0 2px 12px rgba(0,0,0,0.3); }
    .grid,.card-grid,[class*="-grid"] { display:grid; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); gap:20px; margin:16px 0 24px; }
    .badge,.status,[class*="-badge"] { display:inline-block; padding:3px 12px; border-radius:999px; font-size:0.75rem; font-weight:600; }
    label { display:block; font-size:0.72rem; color:var(--text-dim); text-transform:uppercase; letter-spacing:0.06em; margin-bottom:4px; }
    section { margin:0 0 48px; }
    footer { border-top:1px solid var(--border); margin-top:48px; padding-top:32px; }
    ::-webkit-scrollbar { width:6px; } ::-webkit-scrollbar-thumb { background:var(--border2); border-radius:3px; }
  </style>
</head>
<body>
${fragment}
<script>${EDIT_BRIDGE}</script>
</body>
</html>`
}
