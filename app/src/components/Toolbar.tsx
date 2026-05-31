interface Props {
  title?: string
  onOpen: () => void
  onToggleAgent: () => void
  agentPanelOpen: boolean
  loading: boolean
  validation?: string
}

const s: Record<string, React.CSSProperties> = {
  bar: {
    height: 48, background: '#0a0d14', borderBottom: '1px solid var(--border)',
    display: 'flex', alignItems: 'center', padding: '0 16px', gap: 12, flexShrink: 0,
    userSelect: 'none',
  },
  logo: { fontSize: 13, fontWeight: 700, letterSpacing: '0.08em', color: 'var(--accent)', marginRight: 8 },
  title: { flex: 1, fontSize: 13, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  badge: { fontSize: 11, padding: '2px 8px', borderRadius: 999, background: '#1e3a2f', color: '#4ade80', border: '1px solid #166534', letterSpacing: '0.05em' },
  badgeWarn: { background: '#3a2a1e', color: 'var(--warn)', border: '1px solid #92400e' },
  btn: {
    height: 30, padding: '0 12px', borderRadius: 6, border: '1px solid var(--border)',
    background: 'var(--surface)', color: 'var(--text)', cursor: 'pointer', fontSize: 12,
    display: 'flex', alignItems: 'center', gap: 6,
  },
  btnActive: { background: 'var(--accent)', borderColor: 'var(--accent)', color: '#fff' },
}

export default function Toolbar({ title, onOpen, onToggleAgent, agentPanelOpen, loading, validation }: Props) {
  const valid = validation === 'OK'
  return (
    <div style={s.bar}>
      <span style={s.logo}>CLAN</span>
      <span style={s.title}>{loading ? 'Loading…' : (title ?? 'No file open')}</span>
      {validation && (
        <span style={{ ...s.badge, ...(valid ? {} : s.badgeWarn) }}>
          {valid ? '✓ valid' : '⚠ issues'}
        </span>
      )}
      <button style={s.btn} onClick={onOpen}>📂 Open</button>
      <button
        style={{ ...s.btn, ...(agentPanelOpen ? s.btnActive : {}) }}
        onClick={onToggleAgent}
      >
        🤖 Agent
      </button>
    </div>
  )
}
