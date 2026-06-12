// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
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
  const [editMode, setEditMode] = useState(false)
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

  useEffect(() => {
    // On launch the OS may have handed us a .clan file (double-click /
    // "Open with"). Pull it from the backend and open it.
    invoke<string | null>('take_launch_file')
      .then(path => { if (path) handleOpenFile(path) })
      .catch(() => {})

    // If the viewer is already running and the user opens another .clan file,
    // the single-instance plugin re-routes the path here as an event.
    const unlisten = listen<string>('open-file', e => {
      if (e.payload) handleOpenFile(e.payload)
    })
    return () => { unlisten.then(f => f()) }
  }, [])

  function handlePatch(_id: string, _content: string) {
    // The protocol handler already saved the patch. The user's edit is already
    // visible in the DOM — don't reload the iframe or it will revert to the
    // unpatched template (especially for JS-rendered content).
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <Toolbar
        title={openResult?.manifest.title}
        onOpen={() => handleOpenFile()}
        onToggleAgent={() => setAgentPanelOpen(o => !o)}
        agentPanelOpen={agentPanelOpen}
        editMode={editMode}
        onToggleEdit={() => setEditMode(o => !o)}
        loading={loading}
        validation={openResult?.validation}
        hasFile={!!openResult}
      />
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <Sidebar manifest={openResult?.manifest ?? null} path={openResult?.path ?? null} />
        <main style={{ flex: 1, overflow: 'hidden', background: 'var(--bg)', display: 'flex', flexDirection: 'column' }}>
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
              editMode={editMode}
              onPatch={handlePatch}
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
