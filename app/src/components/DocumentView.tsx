import type { ManifestInfo } from '../App'

interface Props {
  htmlContent: string
  hasHumanView: boolean
  manifest: ManifestInfo
}

// Base styles injected into every CLAN document render.
// Agents produce CSS-only fragments (no external URLs per spec §8),
// so the viewer provides rich typography and layout foundations.
const baseStyles = `
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    :root {
      --bg: #0f1117;
      --bg-card: #141922;
      --bg-card2: #1a2032;
      --border: #1e2d45;
      --border2: #2a3350;
      --text: #e2e8f0;
      --text-muted: #94a3b8;
      --text-dim: #64748b;
      --accent: #6366f1;
      --accent-light: #818cf8;
      --green: #4ade80;
      --yellow: #fbbf24;
      --red: #f87171;
      --radius: 10px;
      --radius-sm: 6px;
      --shadow: 0 4px 24px rgba(0,0,0,0.4);
    }

    @font-face {
      font-family: 'SystemUI';
      src: local('-apple-system'), local('BlinkMacSystemFont'), local('Segoe UI'), local('Inter');
    }

    html { scroll-behavior: smooth; }

    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', system-ui, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.65;
      font-size: 15px;
      -webkit-font-smoothing: antialiased;
      padding: 0;
      margin: 0;
    }

    /* ── Typography ── */
    h1 {
      font-size: clamp(1.8rem, 4vw, 3rem);
      font-weight: 800;
      letter-spacing: -0.03em;
      color: #fff;
      line-height: 1.1;
      margin: 0 0 12px;
    }
    h2 {
      font-size: 1.4rem;
      font-weight: 700;
      color: #f1f5f9;
      letter-spacing: -0.02em;
      margin: 32px 0 10px;
    }
    h2::after {
      content: '';
      display: block;
      width: 32px;
      height: 3px;
      background: var(--accent);
      border-radius: 2px;
      margin-top: 8px;
    }
    h3 { font-size: 1.05rem; font-weight: 600; color: #cbd5e1; margin: 20px 0 8px; }
    h4 { font-size: 0.85rem; font-weight: 600; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.07em; margin: 16px 0 6px; }
    p { margin: 0 0 14px; color: var(--text-muted); font-size: 0.95rem; line-height: 1.7; }
    strong { color: #f1f5f9; font-weight: 600; }
    em { color: var(--text-muted); }
    a { color: var(--accent-light); text-decoration: none; }
    a:hover { text-decoration: underline; }

    /* ── Lists ── */
    ul, ol { padding-left: 22px; color: var(--text-muted); margin: 0 0 14px; }
    li { margin: 5px 0; font-size: 0.92rem; line-height: 1.6; }

    /* ── Code ── */
    code {
      background: #1a2032;
      border: 1px solid var(--border);
      color: var(--accent-light);
      padding: 2px 7px;
      border-radius: 4px;
      font-size: 0.82em;
      font-family: 'SF Mono', 'Fira Code', Consolas, monospace;
    }
    pre {
      background: #0d1117;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 20px;
      overflow-x: auto;
      margin: 0 0 20px;
    }
    pre code { background: none; border: none; padding: 0; font-size: 0.85rem; color: #94a3b8; }

    /* ── Tables ── */
    table { width: 100%; border-collapse: collapse; margin: 16px 0 24px; border-radius: var(--radius); overflow: hidden; border: 1px solid var(--border); }
    thead tr { background: rgba(99,102,241,0.12); }
    th {
      color: var(--accent-light);
      text-align: left;
      padding: 11px 14px;
      font-size: 0.75rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.07em;
      border-bottom: 1px solid var(--border);
      white-space: nowrap;
    }
    td { padding: 11px 14px; border-bottom: 1px solid rgba(30,45,69,0.7); color: var(--text-muted); font-size: 0.88rem; }
    tr:last-child td { border-bottom: none; }
    tbody tr:hover { background: rgba(99,102,241,0.04); }
    td:first-child { color: #f1f5f9; font-weight: 500; }

    /* ── Cards ── */
    .card, .summary-card, .ig-profile-card, [class*="-card"] {
      background: var(--bg-card);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 24px;
      margin: 0;
      box-shadow: 0 2px 12px rgba(0,0,0,0.3);
      transition: border-color 0.2s, box-shadow 0.2s;
    }
    .card:hover, .summary-card:hover { border-color: var(--accent); box-shadow: var(--shadow); }

    /* ── Grids ── */
    .grid, .card-grid, [class*="-grid"] {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 20px;
      margin: 16px 0 24px;
    }

    /* ── Badges & Pills ── */
    .badge, .status, [class*="-badge"], [class*="-pill"] {
      display: inline-block;
      padding: 3px 12px;
      border-radius: 999px;
      font-size: 0.75rem;
      font-weight: 600;
      letter-spacing: 0.04em;
    }

    /* ── Utility colours ── */
    label { display: block; font-size: 0.72rem; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 4px; }
    .highlight, [class*="callout"] { border-left: 3px solid var(--accent); padding-left: 14px; }
    section { margin: 0 0 48px; }
    footer { border-top: 1px solid var(--border); margin-top: 48px; padding-top: 32px; }

    /* ── Scrollbar ── */
    ::-webkit-scrollbar { width: 6px; height: 6px; }
    ::-webkit-scrollbar-track { background: transparent; }
    ::-webkit-scrollbar-thumb { background: var(--border2); border-radius: 3px; }

    /* ── Pending state ── */
    .pending { text-align: center; padding: 80px 40px; }
    .pending h1 { color: var(--text-dim); font-size: 1.5rem; }
    .pending p { color: var(--text-dim); }
  </style>
`

export default function DocumentView({ htmlContent, hasHumanView, manifest }: Props) {
  if (!hasHumanView) {
    return (
      <div style={{ padding: 40, color: 'var(--muted)', maxWidth: 600, margin: '0 auto' }}>
        <h2 style={{ color: 'var(--text)', marginBottom: 12 }}>{manifest.title}</h2>
        <p>This .clan file has no human view yet (awaiting first agent pass).</p>
        <p style={{ marginTop: 8, fontSize: 12 }}>Run an agent to generate the document:</p>
        <pre style={{ marginTop: 8, background: 'var(--surface)', padding: 12, borderRadius: 6, fontSize: 12, border: '1px solid var(--border)' }}>
          {`clan read agent <file.clan> | claude\nclan pack --output next.clan parent.clan output.json`}
        </pre>
      </div>
    )
  }

  // Wrap agent HTML in a full document with base styles.
  const fullHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  ${baseStyles}
</head>
<body>
${htmlContent}
</body>
</html>`

  return (
    <iframe
      srcDoc={fullHtml}
      style={{
        width: '100%',
        height: '100%',
        border: 'none',
        background: '#0f1117',
      }}
      sandbox="allow-same-origin"
      title="Document view"
    />
  )
}
