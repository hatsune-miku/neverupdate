import { useMemo } from 'react'

import type { DaemonSnapshot, GuardAction, GuardPointStatus, PreflightReport } from '@/types'

import './index.scss'

interface StatusHeroProps {
  loading: boolean
  busy: boolean
  preflight: PreflightReport | null
  statuses: GuardPointStatus[]
  daemonSnapshot: DaemonSnapshot | null
  onExecuteAll: (action: GuardAction) => void
  onRefresh: () => void
}

export function StatusHero({ loading, busy, preflight, statuses, daemonSnapshot, onExecuteAll, onRefresh }: StatusHeroProps) {
  const guardedCount = useMemo(function () {
    return statuses.filter(function (s) {
      return s.guarded
    }).length
  }, [statuses])

  const breachedCount = useMemo(function () {
    return statuses.filter(function (s) {
      return s.breached
    }).length
  }, [statuses])

  const preflightOk = preflight?.passed === true
  const daemonRunning = daemonSnapshot?.runtime.running || false

  const tone = useMemo(function () {
    if (!preflight) {
      return 'loading'
    }
    if (!preflightOk) {
      return 'critical'
    }
    if (breachedCount > 0) {
      return 'breach'
    }
    return 'secure'
  }, [preflight, preflightOk, breachedCount])

  const statusText = useMemo(function () {
    if (loading) {
      return '正在读取系统状态…'
    }
    if (!preflightOk) {
      return '前置检查未通过'
    }
    if (breachedCount > 0) {
      return `${breachedCount} 个防护点已失守`
    }
    return '所有防线运行正常'
  }, [loading, preflightOk, breachedCount])

  const disabled = busy || loading

  return (
    <section className={`status-hero ${tone}`}>
      <div className="status-hero-seal">
        <div className="status-hero-seal-inner" />
      </div>

      <div className="status-hero-info">
        <h2 className="status-hero-title">{statusText}</h2>
        <div className="status-hero-metrics">
          <span className="status-hero-metric-guarded">
            <svg className="status-hero-check" viewBox="0 0 16 16" aria-hidden>
              <path
                fill="currentColor"
                d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 1 1 1.06-1.06l2.72 2.72 6.72-6.72a.75.75 0 0 1 1.06 0Z"
              />
            </svg>
            {guardedCount} 阻断中
          </span>
          <span className="status-hero-sep" />
          <span>{breachedCount} 失守</span>
          <span className="status-hero-sep" />
          <span>{daemonRunning ? '守护运行中' : '守护未运行'}</span>
        </div>
      </div>

      <div className="status-hero-actions">
        <button className="nu-btn nu-btn-primary" disabled={disabled} type="button" onClick={function () { onExecuteAll('guard') }}>
          全部阻断
        </button>
        <button className="nu-btn nu-btn-ghost" disabled={disabled} type="button" onClick={function () { onExecuteAll('release') }}>
          全部放开
        </button>
        <button className="nu-btn nu-btn-ghost" disabled={disabled} type="button" onClick={onRefresh}>
          刷新数据
        </button>
      </div>
    </section>
  )
}
