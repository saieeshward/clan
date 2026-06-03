import { useEffect, useRef, useCallback, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
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
  var clanScheme = window.navigator.userAgent.includes('Mac') || window.navigator.userAgent.includes('iP') ? 'clan://localhost' : 'http://clan.localhost';
  if (window.__clan_bridge_listening) return;
  window.__clan_bridge_listening = true;

  window.__clan_edit_mode = false;

  function activateEditing() {
    document.querySelectorAll('[data-adf-id]').forEach(function(el) {
      if (el.dataset.clanEditSetup) return;
      el.dataset.clanEditSetup = 'true';
      var originalOutline = el.style.outline;
      var originalCursor = el.style.cursor;
      el.dataset.origOutline = originalOutline || '';
      el.dataset.origCursor = originalCursor || '';
      el.style.outline = '2px solid rgba(59, 130, 246, 0.5)';
      el.style.cursor = 'text';

      el.addEventListener('click', function(e) {
        if (!window.__clan_edit_mode) return;
        e.preventDefault();
        e.stopPropagation();
        el.setAttribute('contenteditable', 'true');
        el.focus();
      });

      el.addEventListener('blur', function() {
        if (!window.__clan_edit_mode) return;
        el.removeAttribute('contenteditable');
        var id = el.getAttribute('data-adf-id');
        var content = el.innerHTML;
        // Bypass postMessage entirely using our custom HTTP protocol
        fetch(clanScheme + '/patch', {
          method: 'POST',
          body: JSON.stringify({ id: id, content: content })
        }).catch(function(err) { console.error('Patch failed:', err); });
      });

      el.addEventListener('keydown', function(e) {
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          el.blur();
        }
      });
    });
  }

  function deactivateEditing() {
    document.querySelectorAll('[data-adf-id]').forEach(function(el) {
      el.style.outline = el.dataset.origOutline || '';
      el.style.cursor = el.dataset.origCursor || '';
      el.removeAttribute('contenteditable');
    });
  }

  // Poll for edit mode state to bypass cross-origin postMessage restrictions
  setInterval(function() {
    fetch(clanScheme + '/edit-mode')
      .then(function(res) { return res.text(); })
      .then(function(text) {
        var active = text === 'true';
        if (active !== window.__clan_edit_mode) {
          window.__clan_edit_mode = active;
          if (active) activateEditing();
          else deactivateEditing();
        }
      })
      .catch(function() {});
  }, 300);
})();
`

export default function DocumentView({ htmlContent, hasHumanView, manifest, editMode, onPatch }: Props) {
  const iframeRef = useRef<HTMLIFrameElement>(null)
  const editModeRef = useRef(editMode)
  
  useEffect(() => {
    editModeRef.current = editMode
  }, [editMode])

  // Send edit mode state to the Rust backend
  const sendEditMode = useCallback((active: boolean) => {
    invoke('set_edit_mode', { active }).catch(console.error)
  }, [])

  // When editMode changes, push the new state in.
  useEffect(() => {
    sendEditMode(editMode)
  }, [editMode, sendEditMode])

  // Listen for patch messages from the Rust backend.
  useEffect(() => {
    const unlistenPromise = listen('clan-patch-saved', (event) => {
      const payload = event.payload as any
      if (payload && typeof payload.id === 'string' && typeof payload.content === 'string') {
        onPatch(payload.id, payload.content)
      }
    })
    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [onPatch])

  const [iframeSrc, setIframeSrc] = useState<string>('')

  useEffect(() => {
    if (!hasHumanView) return;

    // Detect whether the agent produced a full HTML document or a fragment.
    const isFullDoc = /^\s*<!doctype\s+html/i.test(htmlContent) || /^\s*<html/i.test(htmlContent)
    
    const bridge = `<script>${EDIT_BRIDGE}</script>`
    let fullHtml = ''

    if (isFullDoc) {
      if (/<\/body>/i.test(htmlContent)) {
        fullHtml = htmlContent.replace(/<\/body>/i, `${bridge}</body>`)
      } else {
        fullHtml = htmlContent + bridge
      }
    } else {
      fullHtml = `<!DOCTYPE html>
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
  ${htmlContent}
  ${bridge}
</body>
</html>`
    }

    const clanScheme = window.navigator.userAgent.includes('Mac') || window.navigator.userAgent.includes('iP') ? 'clan://localhost' : 'http://clan.localhost';
    invoke('update_preview_html', { html: fullHtml }).then(() => {
      setIframeSrc(clanScheme + '/document?t=' + Date.now())
    }).catch(console.error)
  }, [htmlContent, hasHumanView])

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

  return (
    <iframe
      ref={iframeRef}
      src={iframeSrc}
      style={{ width: '100%', flex: 1, border: 'none', background: '#0f1117' }}
      sandbox="allow-scripts allow-popups"
      title={manifest.title}
    />
  )
}
