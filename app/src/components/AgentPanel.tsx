// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface Props { onClose: () => void }

type Tab = 'chain' | 'state' | 'context'

const s: Record<string, React.CSSProperties> = {
  panel: {
    width: 360, background: 'var(--surface)', borderLeft: '1px solid var(--border)',
    display: 'flex', flexDirection: 'column', overflow: 'hidden', flexShrink: 0,
  },
  header: {
    padding: '10px 14px', borderBottom: '1px solid var(--border)',
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  },
  title: { fontSize: 12, fontWeight: 700, letterSpacing: '0.05em', color: 'var(--muted)', textTransform: 'uppercase' },
  tabs: { display: 'flex', borderBottom: '1px solid var(--border)' },
  tab: { flex: 1, padding: '8px 0', fontSize: 12, textAlign: 'center' as const, cursor: 'pointer', color: 'var(--muted)', background: 'transparent', border: 'none' },
  tabActive: { color: 'var(--accent)', borderBottom: '2px solid var(--accent)' },
  content: { flex: 1, overflow: 'auto', padding: 14 },
  pre: { fontSize: 11, fontFamily: 'monospace', color: '#94a3b8', whiteSpace: 'pre-wrap', wordBreak: 'break-word', lineHeight: 1.6 },
  close: { background: 'transparent', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 16, padding: 4 },
}

export default function AgentPanel({ onClose }: Props) {
  const [tab, setTab] = useState<Tab>('chain')
  const [content, setContent] = useState('')
  const [loading, setLoading] = useState(false)

  async function load(t: Tab) {
    setLoading(true)
    try {
      const cmd = t === 'chain' ? 'get_chain' : t === 'state' ? 'get_agent_state' : 'get_context'
      const data = await invoke<string>(cmd)
      setContent(data)
    } catch (e) {
      setContent(`Error: ${e}`)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    load(tab)
  }, [tab])

  const tabs: { id: Tab; label: string }[] = [
    { id: 'chain', label: 'Decisions' },
    { id: 'state', label: 'State' },
    { id: 'context', label: 'Context' },
  ]

  return (
    <div style={s.panel}>
      <div style={s.header}>
        <span style={s.title}>🤖 Agent Panel</span>
        <button style={s.close} onClick={onClose}>✕</button>
      </div>
      <div style={s.tabs}>
        {tabs.map(t => (
          <button
            key={t.id}
            style={{ ...s.tab, ...(tab === t.id ? s.tabActive : {}) }}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>
      <div style={s.content}>
        {loading ? (
          <span style={{ color: 'var(--muted)', fontSize: 12 }}>Loading…</span>
        ) : (
          <pre style={s.pre}>{content || '(empty)'}</pre>
        )}
      </div>
    </div>
  )
}
