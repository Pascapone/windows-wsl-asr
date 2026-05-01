import { useEffect, useMemo, useRef, useState } from 'react'
import {
  cancelRecording,
  getSnapshot,
  listInputDevices,
  openConfig,
  openLogs,
  reloadConfig,
  refreshAudioDevices,
  restartBackend,
  saveConfig,
  startBackend,
  startRecording,
  stopBackend,
  stopRecording,
} from '../lib/api'
import type { AppConfig, AppSnapshot, AudioDeviceInfo } from '../lib/types'
import { BackendStatusCard } from '../components/BackendStatusCard'
import { DeviceSelect } from '../components/DeviceSelect'
import { DictionaryEditor } from '../components/DictionaryEditor'
import { HotkeyField } from '../components/HotkeyField'
import { ToggleField } from '../components/ToggleField'

function cloneConfig(config: AppConfig): AppConfig {
  return JSON.parse(JSON.stringify(config)) as AppConfig
}

export function SettingsPage({ snapshot }: { snapshot: AppSnapshot | null }) {
  const [draft, setDraft] = useState<AppConfig | null>(null)
  const [dictionaryText, setDictionaryText] = useState('')
  const [baseConfigJson, setBaseConfigJson] = useState<string | null>(null)
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([])
  const [saving, setSaving] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const draftRef = useRef<AppConfig | null>(null)
  const baseConfigJsonRef = useRef<string | null>(null)

  useEffect(() => {
    draftRef.current = draft
  }, [draft])

  useEffect(() => {
    baseConfigJsonRef.current = baseConfigJson
  }, [baseConfigJson])

  useEffect(() => {
    if (snapshot?.config) {
      const incomingJson = JSON.stringify(snapshot.config)
      const currentDraft = draftRef.current
      const hasLocalEdits = currentDraft && baseConfigJsonRef.current !== JSON.stringify(currentDraft)
      if (!currentDraft || !hasLocalEdits) {
        setDraft(cloneConfig(snapshot.config))
        setDictionaryText(snapshot.config.dictionary.terms.join('\n'))
        setBaseConfigJson(incomingJson)
      }
    }
  }, [snapshot])

  useEffect(() => {
    const loadDevices = async () => {
      setDevices(await listInputDevices())
    }

    void loadDevices()
  }, [])

  const parseDictionaryTerms = (value: string) =>
    value
      .split('\n')
      .map((entry) => entry.trim())
      .filter(Boolean)

  const dictionaryCount = draft?.dictionary.terms.length ?? 0
  const hasUnsavedChanges = useMemo(() => {
    if (!baseConfigJson || !draft) {
      return false
    }
    return baseConfigJson !== JSON.stringify(draft)
  }, [baseConfigJson, draft])
  const startupMessage = useMemo(() => {
    if (!snapshot) {
      return 'Lokalen App-State laden...'
    }
    if (snapshot.backendStatus === 'ready') {
      return 'Modell geladen. Du kannst jetzt aufnehmen.'
    }
    if (snapshot.backendStatus === 'starting' || snapshot.dictationStatus === 'backend_starting') {
      return 'Modell wird gerade in WSL geladen. Aufnahme ist blockiert, bis der Status auf ready steht.'
    }
    if (snapshot.errorMessage) {
      return snapshot.errorMessage
    }
    return 'Backend ist noch nicht bereit.'
  }, [snapshot])

  if (!snapshot || !draft) {
    return <main className="shell loading">Lade lokalen App-State...</main>
  }

  const patchConfig = (recipe: (config: AppConfig) => void) => {
    setDraft((current) => {
      if (!current) return current
      const next = cloneConfig(current)
      recipe(next)
      return next
    })
  }

  const persist = async () => {
    setSaving(true)
    setMessage(null)
    try {
      const normalizedDraft = cloneConfig(draft)
      normalizedDraft.dictionary.terms = parseDictionaryTerms(dictionaryText)
      const nextSnapshot = await saveConfig(normalizedDraft)
      setDraft(cloneConfig(nextSnapshot.config))
      setDictionaryText(nextSnapshot.config.dictionary.terms.join('\n'))
      setBaseConfigJson(JSON.stringify(nextSnapshot.config))
      setMessage('Konfiguration gespeichert.')
    } catch (error) {
      setMessage(error instanceof Error ? error.message : 'Konfiguration konnte nicht gespeichert werden.')
    } finally {
      setSaving(false)
    }
  }

  const persistDictionaryAndRestart = async () => {
    setSaving(true)
    setMessage(null)
    try {
      const normalizedDraft = cloneConfig(draft)
      normalizedDraft.dictionary.terms = parseDictionaryTerms(dictionaryText)
      await saveConfig(normalizedDraft)
      await restartBackend()
      const nextSnapshot = await getSnapshot()
      setDraft(cloneConfig(nextSnapshot.config))
      setDictionaryText(nextSnapshot.config.dictionary.terms.join('\n'))
      setBaseConfigJson(JSON.stringify(nextSnapshot.config))
      setMessage('Dictionary gespeichert und Backend neu gestartet.')
    } catch (error) {
      setMessage(error instanceof Error ? error.message : 'Dictionary konnte nicht gespeichert werden.')
    } finally {
      setSaving(false)
    }
  }

  const reloadDevices = async () => {
    await refreshAudioDevices()
    setDevices(await listInputDevices())
  }

  const reloadConfigFromDisk = async () => {
    setSaving(true)
    setMessage(null)
    try {
      const nextSnapshot = await reloadConfig()
      setDraft(cloneConfig(nextSnapshot.config))
      setDictionaryText(nextSnapshot.config.dictionary.terms.join('\n'))
      setBaseConfigJson(JSON.stringify(nextSnapshot.config))
      setMessage('Konfiguration aus JSON neu geladen.')
    } catch (error) {
      setMessage(error instanceof Error ? error.message : 'Konfiguration konnte nicht neu geladen werden.')
    } finally {
      setSaving(false)
    }
  }

  const updateDictionaryText = (value: string) => {
    setDictionaryText(value)
    patchConfig((config) => {
      config.dictionary.terms = parseDictionaryTerms(value)
    })
  }

  return (
    <main className="shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">Pibo Local ASR Tray</p>
          <h1>Lokales Diktat</h1>
          <p className="lede">{startupMessage}</p>
        </div>
        <div className="header-actions">
          <button onClick={() => void startRecording()} disabled={snapshot.backendStatus !== 'ready'}>
            Start Recording
          </button>
          <button onClick={() => void stopRecording()} disabled={snapshot.dictationStatus !== 'recording'}>
            Stop Recording
          </button>
          <button className="secondary" onClick={() => void cancelRecording()} disabled={snapshot.dictationStatus === 'idle'}>
            Cancel
          </button>
        </div>
      </header>

      <section className="grid">
        <BackendStatusCard
          snapshot={snapshot}
          onStart={() => void startBackend()}
          onStop={() => void stopBackend()}
          onRestart={() => void restartBackend()}
          onOpenLogs={() => void openLogs()}
        />

        <article className="panel status-panel">
          <div className="status-line">
            <span>Hotkey</span>
            <strong>{draft.capture.hotkey}</strong>
          </div>
          <div className="status-line">
            <span>Dictionary</span>
            <strong>{dictionaryCount} Eintraege</strong>
          </div>
          <div className="status-line">
            <span>Dictation</span>
            <strong>{snapshot.dictationStatus}</strong>
          </div>
          {snapshot.errorMessage ? <p className="error-line">{snapshot.errorMessage}</p> : null}
        </article>

        <article className="panel compact-panel">
          <h2>General</h2>
          <ToggleField
            label="Launch backend automatically"
            checked={draft.backend.autoStartBackend}
            onChange={(value) => patchConfig((config) => { config.backend.autoStartBackend = value })}
          />
          <ToggleField
            label="Start app on login"
            checked={draft.general.startOnLogin}
            onChange={(value) => patchConfig((config) => { config.general.startOnLogin = value })}
          />
        </article>

        <article className="panel compact-panel">
          <h2>Input</h2>
          <DeviceSelect
            devices={devices}
            selectedId={draft.capture.inputDeviceId}
            onRefresh={reloadDevices}
            onSelect={(value) => patchConfig((config) => { config.capture.inputDeviceId = value })}
          />
        </article>

        <article className="panel compact-panel">
          <h2>Hotkey</h2>
          <HotkeyField
            value={draft.capture.hotkey}
            onChange={(value) => patchConfig((config) => { config.capture.hotkey = value })}
          />
        </article>

        <article className="panel">
          <h2>Backend</h2>
          <div className="form-grid">
            <label>
              <span>WSL distro</span>
              <input
                value={draft.backend.wslDistro}
                onChange={(event) => patchConfig((config) => { config.backend.wslDistro = event.target.value })}
              />
            </label>
            <label>
              <span>Port</span>
              <input
                type="number"
                value={draft.backend.port}
                onChange={(event) => patchConfig((config) => { config.backend.port = Number(event.target.value) || 8765 })}
              />
            </label>
            <label>
              <span>Model</span>
              <input
                value={draft.backend.modelName}
                onChange={(event) => patchConfig((config) => { config.backend.modelName = event.target.value })}
              />
            </label>
            <label>
              <span>GPU memory utilization</span>
              <input
                type="number"
                min="0.1"
                max="0.99"
                step="0.01"
                value={draft.backend.gpuMemoryUtilization}
                onChange={(event) => patchConfig((config) => { config.backend.gpuMemoryUtilization = Number(event.target.value) || 0.85 })}
              />
            </label>
            <label>
              <span>Chunk size sec</span>
              <input
                type="number"
                min="0.1"
                step="0.1"
                value={draft.backend.chunkSizeSec}
                onChange={(event) => patchConfig((config) => { config.backend.chunkSizeSec = Number(event.target.value) || 0.5 })}
              />
            </label>
          </div>
        </article>

        <article className="panel">
          <h2>Dictation</h2>
          <ToggleField
            label="Auto paste final transcript"
            checked={draft.dictation.autoPaste}
            onChange={(value) => patchConfig((config) => { config.dictation.autoPaste = value })}
          />
          <ToggleField
            label="Restore clipboard after paste"
            checked={draft.dictation.restoreClipboard}
            onChange={(value) => patchConfig((config) => { config.dictation.restoreClipboard = value })}
          />
          <ToggleField
            label="Show overlay while recording"
            checked={draft.overlay.enabled}
            onChange={(value) => patchConfig((config) => { config.overlay.enabled = value })}
          />
          <ToggleField
            label="Overlay follows mouse anchor at recording start"
            checked={draft.overlay.anchor === 'mouse'}
            onChange={(value) => patchConfig((config) => { config.overlay.anchor = value ? 'mouse' : 'center' })}
          />
          <label>
            <span>Language hint</span>
            <input
              value={draft.dictation.languageHint ?? ''}
              onChange={(event) => patchConfig((config) => { config.dictation.languageHint = event.target.value || null })}
            />
          </label>
        </article>

        <article className="panel span-2 dictionary-panel">
          <div className="section-heading">
            <div>
              <h2>Dictionary</h2>
              <p className="muted">Ein Begriff pro Zeile. Gespeicherte Aenderungen gelten fuer neue Aufnahmen.</p>
            </div>
            <span className="hint">{hasUnsavedChanges ? 'ungespeichert' : 'gespeichert'}</span>
          </div>
          <DictionaryEditor
            value={dictionaryText}
            onChange={updateDictionaryText}
          />
          <div className="dictionary-toolbar">
            <span className="hint">
              {dictionaryCount} Eintraege{hasUnsavedChanges ? ' | ungespeicherte Aenderungen' : ''}
            </span>
            <div className="button-row">
              <button onClick={() => void persist()} disabled={saving}>
                Speichern
              </button>
              <button onClick={() => void persistDictionaryAndRestart()} disabled={saving}>
                Speichern + Backend neu starten
              </button>
            </div>
          </div>
        </article>
      </section>

      <footer className="footer">
        <div>
          <strong>Backend:</strong> {snapshot.backendStatus}
          {' | '}
          <strong>Dictation:</strong> {snapshot.dictationStatus}
          {snapshot.errorMessage ? ` | ${snapshot.errorMessage}` : ''}
        </div>
        <div className="footer-actions">
          <button className="secondary" onClick={() => void openConfig()}>
            JSON oeffnen
          </button>
          <button className="secondary" onClick={() => void reloadConfigFromDisk()} disabled={saving}>
            JSON neu laden
          </button>
          <button className="secondary" onClick={() => void openLogs()}>
            Logs
          </button>
          <button onClick={() => void persist()} disabled={saving}>
            {saving ? 'Speichern...' : 'Einstellungen speichern'}
          </button>
        </div>
      </footer>

      {message ? <div className="toast">{message}</div> : null}
    </main>
  )
}
