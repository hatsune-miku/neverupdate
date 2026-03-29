import { useEffect, useState } from 'react'

import type { DaemonSnapshot } from '@/types'

import './index.scss'

interface DaemonControlProps {
  busy: boolean
  snapshot: DaemonSnapshot | null
  onRegister: () => void
  onToggleRunning: (running: boolean) => void
  onUnregister: () => void
}

export function DaemonControl({ busy, snapshot, onRegister, onToggleRunning, onUnregister }: DaemonControlProps) {
  const [showHelp, setShowHelp] = useState(false)
  const runtime = snapshot?.runtime
  const running = runtime?.running || false
  const installed = runtime?.service_registered || false
  const heartbeat = snapshot?.timestamp ? new Date(snapshot.timestamp).toLocaleTimeString() : '--:--'

  useEffect(() => {
    if (!showHelp) {
      return
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setShowHelp(false)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [showHelp])

  return (
    <section className="daemon-ctrl">
      <header className="daemon-ctrl-header">
        <div className="daemon-ctrl-title">
          <h3>守护进程</h3>
          <button className="daemon-ctrl-help" type="button" aria-label="守护进程说明" onClick={() => setShowHelp(true)}>
            ?
          </button>
        </div>
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
          <dt>安装状态</dt>
          <dd>{runtime?.service_registered ? '已安装' : '未安装'}</dd>
        </div>
        <div>
          <dt>最近心跳</dt>
          <dd className="nu-mono">{heartbeat}</dd>
        </div>
      </dl>

      <div className="daemon-ctrl-actions">
        {installed && (
          <button className="nu-btn nu-btn-primary" disabled={busy} type="button" onClick={() => onToggleRunning(running)}>
            {running ? '停止服务' : '启动服务'}
          </button>
        )}
        {!installed ? <button className="nu-btn nu-btn-ghost" disabled={busy} type="button" onClick={onRegister}>安装服务</button> : null}
        {installed && (
          <button className="nu-btn nu-btn-ghost" disabled={busy} type="button" onClick={onUnregister}>
            卸载服务
          </button>
        )}
      </div>

      {showHelp ? (
        <div
          className="daemon-ctrl-help-backdrop"
          role="presentation"
          onClick={() => {
            setShowHelp(false)
          }}
        >
          <section
            className="daemon-ctrl-help-modal"
            onClick={(event) => {
              event.stopPropagation()
            }}
          >
            <header className="daemon-ctrl-help-header">
              <h4>守护进程说明</h4>
              <button className="nu-icon-btn" type="button" aria-label="关闭" onClick={() => setShowHelp(false)}>
                <svg viewBox="0 0 16 16" aria-hidden>
                  <path
                    fill="currentColor"
                    d="M3.22 3.22a.75.75 0 0 1 1.06 0L8 6.94l3.72-3.72a.75.75 0 1 1 1.06 1.06L9.06 8l3.72 3.72a.75.75 0 0 1-1.06 1.06L8 9.06l-3.72 3.72a.75.75 0 1 1-1.06-1.06L6.94 8 3.22 4.28a.75.75 0 0 1 0-1.06Z"
                  />
                </svg>
              </button>
            </header>
            <div className="daemon-ctrl-help-content">
              <p>守护进程用于常驻检查系统状态，并在检测到更新链路被恢复时触发拦截记录。</p>
              <ul>
                <li>安装服务：写入并注册 Windows 服务。</li>
                <li>启动/停止服务：控制守护进程的运行状态。</li>
                <li>卸载服务：移除服务注册，停止守护监控。</li>
              </ul>
            </div>
          </section>
        </div>
      ) : null}
    </section>
  )
}
