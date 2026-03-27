import type { DaemonSnapshot } from '@/types'

import './index.scss'

interface DaemonControlProps {
  busy: boolean
  snapshot: DaemonSnapshot | null
  onRegisterOrReregister: () => void
  onToggleRunning: (running: boolean) => void
  onUnregister: () => void
}

export function DaemonControl({ busy, snapshot, onRegisterOrReregister, onToggleRunning, onUnregister }: DaemonControlProps) {
  const runtime = snapshot?.runtime
  const running = runtime?.running || false
  const heartbeat = snapshot?.timestamp ? new Date(snapshot.timestamp).toLocaleTimeString() : '--:--'

  return (
    <section className="daemon-ctrl">
      <header className="daemon-ctrl-header">
        <h3>守护进程</h3>
        <span className={`daemon-ctrl-status ${running ? 'active' : 'idle'}`}>
          <span className="daemon-ctrl-dot" />
          {running ? '运行中' : '已停止'}
        </span>
      </header>

      <dl className="daemon-ctrl-info">
        <div>
          <dt>服务名</dt>
          <dd className="nu-mono">{runtime?.service_name || 'NeverUpdateDaemon'}</dd>
        </div>
        <div>
          <dt>注册状态</dt>
          <dd>{runtime?.service_registered ? '已注册' : '未注册'}</dd>
        </div>
        <div>
          <dt>最近心跳</dt>
          <dd className="nu-mono">{heartbeat}</dd>
        </div>
      </dl>

      <div className="daemon-ctrl-actions">
        <button className="nu-btn nu-btn-primary" disabled={busy} type="button" onClick={function () { onToggleRunning(running) }}>
          {running ? '停止服务' : '启动服务'}
        </button>
        <button className="nu-btn nu-btn-ghost" disabled={busy} type="button" onClick={onRegisterOrReregister}>
          注册
        </button>
        <button className="nu-btn nu-btn-danger" disabled={busy} type="button" onClick={onUnregister}>
          卸载
        </button>
      </div>
    </section>
  )
}
