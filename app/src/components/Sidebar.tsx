import type { ManifestInfo } from '../App'

interface Props {
  manifest: ManifestInfo | null
  path: string | null
}

const s: Record<string, React.CSSProperties> = {
  sidebar: {
    width: 260, background: 'var(--surface)', borderRight: '1px solid var(--border)',
    display: 'flex', flexDirection: 'column', overflow: 'hidden', flexShrink: 0,
  },
  section: { padding: '12px 14px', borderBottom: '1px solid var(--border)' },
  label: { fontSize: 10, fontWeight: 700, letterSpacing: '0.1em', color: 'var(--muted)', textTransform: 'uppercase', marginBottom: 8 },
  row: { display: 'flex', flexDirection: 'column', gap: 2, marginBottom: 8 },
  key: { fontSize: 11, color: 'var(--muted)' },
  val: { fontSize: 12, color: 'var(--text)', wordBreak: 'break-all' },
  mono: { fontSize: 10, color: 'var(--muted)', fontFamily: 'monospace', wordBreak: 'break-all' },
  pill: { display: 'inline-block', fontSize: 10, padding: '1px 7px', borderRadius: 999, background: '#1e2d45', color: 'var(--accent)', marginTop: 2 },
  empty: { padding: 20, color: 'var(--muted)', fontSize: 12 },
}

function Row({ k, v, mono }: { k: string; v: string; mono?: boolean }) {
  return (
    <div style={s.row}>
      <span style={s.key}>{k}</span>
      <span style={mono ? s.mono : s.val}>{v}</span>
    </div>
  )
}

export default function Sidebar({ manifest, path }: Props) {
  if (!manifest) return <div style={s.sidebar}><p style={s.empty}>Open a .clan file to begin.</p></div>

  return (
    <div style={{ ...s.sidebar, overflowY: 'auto' }}>
      <div style={s.section}>
        <div style={s.label}>Document</div>
        <Row k="Title" v={manifest.title} />
        {manifest.document_type && <div style={{ ...s.pill, marginBottom: 8 }}>{manifest.document_type}</div>}
        <Row k="Version" v={manifest.version} />
        <Row k="Created" v={new Date(manifest.created_at).toLocaleString()} />
        <Row k="Updated" v={new Date(manifest.updated_at).toLocaleString()} />
        <Row k="Files" v={String(manifest.file_count)} />
      </div>

      <div style={s.section}>
        <div style={s.label}>Identity</div>
        <Row k="ID" v={manifest.id} mono />
        <Row k="SHA-256" v={manifest.sha256.slice(0, 20) + '…'} mono />
      </div>

      {manifest.lineage ? (
        <div style={s.section}>
          <div style={s.label}>Lineage</div>
          <Row k="Parent ID" v={manifest.lineage.parent_id} mono />
          <Row k="Delta" v={manifest.lineage.delta} />
          {manifest.lineage.parent_sha256 && (
            <Row k="Parent SHA-256" v={manifest.lineage.parent_sha256.slice(0, 20) + '…'} mono />
          )}
        </div>
      ) : (
        <div style={s.section}>
          <div style={s.label}>Lineage</div>
          <span style={{ fontSize: 12, color: 'var(--muted)' }}>Root document — no parent.</span>
        </div>
      )}

      {path && (
        <div style={s.section}>
          <div style={s.label}>File</div>
          <span style={s.mono}>{path}</span>
        </div>
      )}
    </div>
  )
}
