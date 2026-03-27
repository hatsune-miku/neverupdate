import type { HistoryEntry } from '@/types'

import './index.scss'

interface TimelineProps {
  history: HistoryEntry[]
}

function formatTime(value: string): string {
  return new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function formatDate(value: string): string {
  return new Date(value).toLocaleDateString([], { month: '2-digit', day: '2-digit' })
}

const ActionLabels: Record<string, string> = {
  guard: '阻断',
  release: '放开',
  repair: '修复',
}

export function Timeline({ history }: TimelineProps) {
  return (
    <section className="tl">
      <header className="tl-header">
        <h3>操作记录</h3>
        <span className="tl-count">{history.length}</span>
      </header>

      <div className="tl-scroll">
        {history.length === 0 ? <p className="tl-empty">暂无记录</p> : null}

        {history.map(function (entry, i) {
          const ok = entry.success
          return (
            <div key={`${entry.point_id}-${entry.timestamp}-${i}`} className="tl-entry">
              <div className="tl-rail">
                <span className={`tl-dot ${ok ? 'ok' : 'fail'}`} />
                {i < history.length - 1 ? <span className="tl-line" /> : null}
              </div>
              <div className="tl-content">
                <div className="tl-content-top">
                  <strong>{entry.point_id}</strong>
                  <span className={`tl-action-tag ${ok ? 'ok' : 'fail'}`}>{ActionLabels[entry.action] || entry.action}</span>
                </div>
                <span className="tl-msg">{entry.message || (ok ? '操作成功' : '操作失败')}</span>
                <span className="tl-time nu-mono">{formatDate(entry.timestamp)} {formatTime(entry.timestamp)}</span>
              </div>
            </div>
          )
        })}
      </div>
    </section>
  )
}
