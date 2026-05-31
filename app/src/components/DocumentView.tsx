import type { ManifestInfo } from '../App'

interface Props {
  htmlContent: string
  hasHumanView: boolean
  manifest: ManifestInfo
}

const baseStyles = `
  <style>
    body, .clan-doc {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
      background: #0f1117; color: #e2e8f0; line-height: 1.6;
      max-width: 860px; margin: 0 auto; padding: 40px 32px;
    }
    h1 { font-size: 2rem; font-weight: 700; letter-spacing: -0.02em; color: #f1f5f9; margin: 0 0 8px; }
    h2 { font-size: 1.25rem; font-weight: 600; color: #cbd5e1; margin: 24px 0 8px; }
    h3 { font-size: 1rem; font-weight: 600; color: #94a3b8; margin: 16px 0 6px; }
    p { margin: 0 0 12px; color: #cbd5e1; }
    table { width: 100%; border-collapse: collapse; margin: 16px 0; }
    th { background: #1e2d45; color: #94a3b8; text-align: left; padding: 8px 12px; font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; }
    td { padding: 10px 12px; border-bottom: 1px solid #1e2533; color: #e2e8f0; font-size: 14px; }
    tr:last-child td { border-bottom: none; }
    .badge, .status { display: inline-block; padding: 2px 10px; border-radius: 999px; font-size: 12px; font-weight: 600; }
    .card, .summary-card { background: #141922; border: 1px solid #1e2d45; border-radius: 10px; padding: 20px; margin: 12px 0; }
    .grid, .card-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin: 16px 0; }
    label { display: block; font-size: 11px; color: #64748b; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }
    strong { color: #f1f5f9; }
    ul, ol { padding-left: 20px; color: #cbd5e1; margin: 0 0 12px; }
    li { margin: 4px 0; }
    code { background: #1e2533; padding: 2px 6px; border-radius: 4px; font-size: 13px; font-family: monospace; }
    .highlight { border-left: 3px solid #6366f1; padding-left: 12px; }
    .pending h1 { color: #64748b; }
    .pending p { color: #475569; }
    section { margin: 0 0 32px; }
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
