import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import Toolbar from './components/Toolbar'
import Sidebar from './components/Sidebar'
import DocumentView from './components/DocumentView'
import Welcome from './components/Welcome'
import AgentPanel from './components/AgentPanel'
import './index.css'

export interface ManifestInfo {
  title: string
  id: string
  version: string
  created_at: string
  updated_at: string
  document_type?: string
  sha256: string
  file_count: number
  lineage?: {
    parent_id: string
    parent_uri: string
    parent_sha256?: string
    delta: string
  }
}

export interface OpenResult {
  path: string
  manifest: ManifestInfo
  validation: string
  has_human_view: boolean
}

export default function App() {
  const [openResult, setOpenResult] = useState<OpenResult | null>(null)
  const [htmlContent, setHtmlContent] = useState<string>('')
  const [agentPanelOpen, setAgentPanelOpen] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleOpenFile(filePath?: string) {
    setLoading(true)
    setError(null)
    try {
      let path = filePath
      if (!path) {
        const selected = await openDialog({
          multiple: false,
          filters: [{ name: 'CLAN Files', extensions: ['clan'] }],
        })
        if (!selected) { setLoading(false); return }
        path = selected as string
      }

      const result = await invoke<OpenResult>('open_clan', { path })
      setOpenResult(result)

      if (result.has_human_view) {
        const html = await invoke<string>('get_human_html')
        setHtmlContent(html)
      } else {
        setHtmlContent('')
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <Toolbar
        title={openResult?.manifest.title}
        onOpen={() => handleOpenFile()}
        onToggleAgent={() => setAgentPanelOpen(o => !o)}
        agentPanelOpen={agentPanelOpen}
        loading={loading}
        validation={openResult?.validation}
      />
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <Sidebar manifest={openResult?.manifest ?? null} path={openResult?.path ?? null} />
        <main style={{ flex: 1, overflow: 'auto', background: 'var(--bg)' }}>
          {error && (
            <div style={{ padding: 24, color: 'var(--danger)', fontFamily: 'monospace' }}>
              <strong>Error:</strong> {error}
            </div>
          )}
          {!openResult && !error && <Welcome onOpen={() => handleOpenFile()} />}
          {openResult && (
            <DocumentView
              htmlContent={htmlContent}
              hasHumanView={openResult.has_human_view}
              manifest={openResult.manifest}
            />
          )}
        </main>
        {agentPanelOpen && openResult && (
          <AgentPanel onClose={() => setAgentPanelOpen(false)} />
        )}
      </div>
    </div>
  )
}
