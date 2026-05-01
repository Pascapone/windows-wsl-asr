import type { AppSnapshot } from '../lib/types'

type Props = {
  snapshot: AppSnapshot
  onStart: () => void
  onStop: () => void
  onRestart: () => void
  onOpenLogs: () => void
}

export function BackendStatusCard({ snapshot, onStart, onStop, onRestart, onOpenLogs }: Props) {
  return (
    <article className="panel backend-panel">
      <div className="backend-header">
        <div>
          <p className="eyebrow">Backend</p>
          <h2>{snapshot.backendStatus}</h2>
        </div>
        <span className={`status-pill status-${snapshot.backendStatus}`}>{snapshot.backendStatus}</span>
      </div>

      <p className="muted">
        Model loaded: {snapshot.backendModelLoaded ? 'yes' : 'no'}
        {' | '}
        Owned process: {snapshot.backendOwned ? 'yes' : 'no'}
      </p>
      {snapshot.errorMessage ? <p className="error-line">{snapshot.errorMessage}</p> : null}

      <div className="button-row">
        <button onClick={onStart} disabled={snapshot.backendStatus === 'ready' || snapshot.backendStatus === 'starting'}>
          Start
        </button>
        <button className="secondary" onClick={onStop} disabled={snapshot.backendStatus === 'stopped'}>
          Stop
        </button>
        <button className="secondary" onClick={onRestart}>
          Restart
        </button>
        <button className="secondary" onClick={onOpenLogs}>
          Logs
        </button>
      </div>
    </article>
  )
}
