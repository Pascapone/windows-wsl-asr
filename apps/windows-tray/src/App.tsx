import { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { copyTextToClipboard, getSnapshot } from './lib/api'
import type { AppSnapshot } from './lib/types'
import { SettingsPage } from './pages/SettingsPage'

type WindowMode = 'main' | 'overlay'

function OverlayView({ snapshot }: { snapshot: AppSnapshot | null }) {
  const textRef = useRef<HTMLDivElement | null>(null)
  const [copyState, setCopyState] = useState<'idle' | 'done' | 'error'>('idle')

  useEffect(() => {
    const node = textRef.current
    if (!node) {
      return
    }
    node.scrollTop = node.scrollHeight
  }, [snapshot?.partialText, snapshot?.dictationStatus, snapshot?.errorMessage])

  useEffect(() => {
    if (copyState === 'idle') {
      return
    }

    const timeout = window.setTimeout(() => {
      setCopyState('idle')
    }, 1400)

    return () => window.clearTimeout(timeout)
  }, [copyState])

  const displayText =
    snapshot?.partialText?.trim() ||
    snapshot?.lastTranscript?.trim() ||
    (snapshot?.dictationStatus === 'finalizing'
      ? 'Finalisiere lokales Transkript...'
      : snapshot?.dictationStatus === 'backend_starting'
        ? 'Backend startet in WSL...'
        : snapshot?.dictationStatus === 'error'
          ? 'Das Backend hat angehalten. Der letzte sichtbare Text bleibt hier zum Sichern erhalten.'
          : 'Sprich nach dem Druecken des Hotkeys. Partials erscheinen hier.')

  const canCopy = Boolean(snapshot?.partialText?.trim() || snapshot?.lastTranscript?.trim())

  const handleCopy = async () => {
    if (!canCopy) {
      return
    }

    try {
      await copyTextToClipboard(displayText)
      setCopyState('done')
    } catch (error) {
      console.error('copy failed', error)
      setCopyState('error')
    }
  }

  return (
    <main className="overlay-shell">
      <div className={`overlay-card status-${snapshot?.dictationStatus ?? 'idle'}`}>
        <div className="overlay-status-row">
          <div className="overlay-status-meta">
            <span className="status-chip">{snapshot?.dictationStatus ?? 'idle'}</span>
            <span className="overlay-backend">{snapshot?.backendStatus ?? 'stopped'}</span>
          </div>
          <button
            type="button"
            className={`overlay-copy-button copy-${copyState}`}
            onClick={() => void handleCopy()}
            disabled={!canCopy}
            title={
              copyState === 'done'
                ? 'In Zwischenablage kopiert'
                : copyState === 'error'
                  ? 'Kopieren fehlgeschlagen'
                  : 'Text in Zwischenablage kopieren'
            }
            aria-label="Text in Zwischenablage kopieren"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M9 9h9v11H9z" />
              <path d="M6 4h9v3H9v9H6z" />
            </svg>
          </button>
        </div>
        {snapshot?.errorMessage ? <div className="overlay-error">{snapshot.errorMessage}</div> : null}
        <div ref={textRef} className="overlay-text">
          {displayText}
        </div>
      </div>
    </main>
  )
}

export default function App() {
  const [mode, setMode] = useState<WindowMode>('main')
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null)

  useEffect(() => {
    if (getCurrentWindow().label === 'overlay') {
      setMode('overlay')
    }

    let disposed = false
    const bootstrap = async () => {
      const nextSnapshot = await getSnapshot()
      if (!disposed) {
        setSnapshot(nextSnapshot)
      }
    }

    void bootstrap()

    const unlistenPromise = listen<AppSnapshot>('state://changed', (event) => {
      setSnapshot(event.payload)
    })

    return () => {
      disposed = true
      void unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  if (mode === 'overlay') {
    return <OverlayView snapshot={snapshot} />
  }

  return <SettingsPage snapshot={snapshot} />
}
