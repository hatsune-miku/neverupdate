import type { PreflightReport } from '@/types'

import './index.scss'

interface DiagnosticsProps {
  preflight: PreflightReport | null
  busy: boolean
  loading: boolean
  onRefresh: () => void
  onBack?: () => void
}

export function Diagnostics({ preflight, busy, loading, onRefresh, onBack }: DiagnosticsProps) {
  const failedCount = preflight?.checks.filter(function (c) {
    return !c.passed
  }).length || 0
  const allPassed = preflight ? failedCount === 0 : false

  return (
    <div className="diag-screen">
      <div className="diag-card">
        <h1>前置检查</h1>
        <p className="diag-subtitle">以下条件需要全部满足才能继续</p>

        {!preflight ? (
          <div className="diag-loading-area">
            <div className="diag-spinner" />
            <p>正在检查系统环境…</p>
          </div>
        ) : (
          <>
            <div className="diag-badge-row">
              <span className={`diag-badge ${allPassed ? 'ok' : 'fail'}`}>
                {allPassed ? '全部通过' : `${failedCount} 项未通过`}
              </span>
            </div>

            <div className="diag-grid">
              {preflight.checks.map(function (check) {
                return (
                  <div key={check.id} className={`diag-item ${check.passed ? 'ok' : 'fail'}`}>
                    <span className="diag-icon">{check.passed ? '\u25CB' : '\u00D7'}</span>
                    <div className="diag-item-body">
                      <span className="diag-item-title">{check.title}</span>
                      <span className="diag-item-detail">{check.detail}</span>
                    </div>
                  </div>
                )
              })}
            </div>

            <p className="diag-warning">{allPassed ? '全部检查已通过' : '请先解决未通过的检查项，然后点击刷新重试'}</p>
          </>
        )}

        <div className="diag-actions">
          {onBack ? (
            <button className="nu-btn nu-btn-ghost" disabled={busy || loading} type="button" onClick={onBack}>
              返回面板
            </button>
          ) : null}
          <button className="nu-btn nu-btn-primary" disabled={busy || loading} type="button" onClick={onRefresh}>
            刷新检查
          </button>
        </div>
      </div>
    </div>
  )
}
