interface Props { onOpen: () => void }

export default function Welcome({ onOpen }: Props) {
  return (
    <div style={{
      display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
      height: '100%', gap: 20, padding: 40, textAlign: 'center',
    }}>
      <div style={{ fontSize: 48, marginBottom: 8 }}>📄</div>
      <h1 style={{ fontSize: '1.8rem', fontWeight: 700, letterSpacing: '-0.03em', color: 'var(--text)', marginBottom: 4 }}>
        CLAN Viewer
      </h1>
      <p style={{ color: 'var(--muted)', maxWidth: 440, lineHeight: 1.6 }}>
        Context and Live Agent Notation — open a <code style={{ background: 'var(--surface)', padding: '2px 6px', borderRadius: 4, fontSize: 13 }}>.clan</code> file to view agent-generated documents with full lineage and provenance.
      </p>
      <button
        onClick={onOpen}
        style={{
          marginTop: 8, padding: '10px 24px', borderRadius: 8,
          background: 'var(--accent)', border: 'none', color: '#fff',
          fontSize: 14, fontWeight: 600, cursor: 'pointer',
        }}
      >
        Open .clan file
      </button>
      <p style={{ fontSize: 12, color: 'var(--muted)', marginTop: 8 }}>
        Or use <code style={{ background: 'var(--surface)', padding: '1px 5px', borderRadius: 3, fontSize: 11 }}>clan create</code> to make one from the CLI.
      </p>
    </div>
  )
}
