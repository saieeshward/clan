// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface Props {
  title?: string
  onOpen: () => void
  onToggleAgent: () => void
  onToggleEdit: () => void
  agentPanelOpen: boolean
  editMode: boolean
  loading: boolean
  validation?: string
  hasFile: boolean
}

const ClanMark = () => (
  <svg width="22" height="22" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg" style={{ flexShrink: 0 }}>
    <line x1="70" y1="24" x2="37" y2="21" stroke="rgba(120,160,230,0.42)" strokeWidth="2.8" strokeLinecap="round"/>
    <line x1="37" y1="21" x2="18" y2="50" stroke="rgba(120,160,230,0.42)" strokeWidth="2.8" strokeLinecap="round"/>
    <line x1="18" y1="50" x2="37" y2="79" stroke="rgba(120,160,230,0.42)" strokeWidth="2.8" strokeLinecap="round"/>
    <line x1="37" y1="79" x2="70" y2="76" stroke="rgba(120,160,230,0.42)" strokeWidth="2.8" strokeLinecap="round"/>
    <circle cx="70" cy="24" r="5.6" fill="#6366f1"/>
    <circle cx="37" cy="21" r="5.6" fill="#5682e9"/>
    <circle cx="18" cy="50" r="7" fill="#489de0"/>
    <circle cx="37" cy="79" r="5.6" fill="#3bb9d8"/>
    <circle cx="70" cy="76" r="5.6" fill="#2dd4cf"/>
  </svg>
)

const s: Record<string, React.CSSProperties> = {
  bar: {
    height: 48, background: '#0a0d14', borderBottom: '1px solid var(--border)',
    display: 'flex', alignItems: 'center', padding: '0 16px', gap: 12, flexShrink: 0,
    userSelect: 'none',
  },
  logo: { display: 'flex', alignItems: 'center', gap: 7, marginRight: 4 },
  logoText: { fontSize: 13, fontWeight: 700, letterSpacing: '0.08em', color: 'var(--accent)' },
  title: { flex: 1, fontSize: 13, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  badge: { fontSize: 11, padding: '2px 8px', borderRadius: 999, background: '#1e3a2f', color: '#4ade80', border: '1px solid #166534', letterSpacing: '0.05em' },
  badgeWarn: { background: '#3a2a1e', color: 'var(--warn)', border: '1px solid #92400e' },
  btn: {
    height: 30, padding: '0 12px', borderRadius: 6, border: '1px solid var(--border)',
    background: 'var(--surface)', color: 'var(--text)', cursor: 'pointer', fontSize: 12,
    display: 'flex', alignItems: 'center', gap: 6,
  },
  btnActive: { background: 'var(--accent)', borderColor: 'var(--accent)', color: '#fff' },
  btnEdit: { background: '#1e3a2f', borderColor: '#166534', color: '#4ade80' },
  editBadge: {
    fontSize: 10, padding: '2px 8px', borderRadius: 999,
    background: 'rgba(74,222,128,0.15)', color: '#4ade80',
    border: '1px solid rgba(74,222,128,0.4)', letterSpacing: '0.05em',
    animation: 'pulse 2s infinite',
  },
}

export default function Toolbar({
  title, onOpen, onToggleAgent, onToggleEdit,
  agentPanelOpen, editMode, loading, validation, hasFile
}: Props) {
  const valid = validation === 'OK'
  return (
    <div style={s.bar}>
      <span style={s.logo}><ClanMark /><span style={s.logoText}>CLAN</span></span>
      <span style={s.title}>{loading ? 'Loading…' : (title ?? 'No file open')}</span>
      {validation && (
        <span
          style={{ ...s.badge, ...(valid ? {} : s.badgeWarn), cursor: valid ? 'default' : 'help' }}
          title={valid ? 'No validation issues' : validation}
        >
          {valid ? '✓ valid' : '⚠ issues'}
        </span>
      )}
      {editMode && <span style={s.editBadge}>● EDITING</span>}
      <button style={s.btn} onClick={onOpen}>📂 Open</button>
      {hasFile && (
        <button
          style={{ ...s.btn, ...(editMode ? s.btnEdit : {}) }}
          onClick={onToggleEdit}
          title={editMode ? 'Exit edit mode' : 'Edit document text'}
        >
          {editMode ? '✏️ Done' : '✏️ Edit'}
        </button>
      )}
      <button
        style={{ ...s.btn, ...(agentPanelOpen ? s.btnActive : {}) }}
        onClick={onToggleAgent}
      >
        🤖 Agent
      </button>
    </div>
  )
}
