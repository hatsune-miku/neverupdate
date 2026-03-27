import { useEffect, useMemo } from 'react'

import { DaemonControl } from '@/components/DaemonControl'
import { Diagnostics } from '@/components/Diagnostics'
import { GuardMatrix } from '@/components/GuardMatrix'
import { RiskGate } from '@/components/RiskGate'
import { StatusHero } from '@/components/StatusHero'
import { Timeline } from '@/components/Timeline'
import { useAppStore } from '@/store'

import './App.scss'

const EXTREME_POINT_ID = 'extreme_mode'

export default function App() {
  const {
    loading,
    busy,
    lastError,
    riskAccepted,
    preflight,
    points,
    statuses,
    history,
    daemonSnapshot,
    bootstrap,
    acceptRisk,
    refresh,
    executePoint,
    executeAll,
    reregisterService,
    startService,
    stopService,
    unregisterService,
    runExtremeMode,
  } = useAppStore()

  const preflightOk = preflight?.passed === true

  useEffect(
    function () {
      bootstrap()
    },
    [bootstrap],
  )

  const breachedCount = useMemo(
    function () {
      return statuses.filter(function (s) {
        return s.breached
      }).length
    },
    [statuses],
  )

  if (!riskAccepted) {
    return <RiskGate onAccept={acceptRisk} />
  }

  if (!preflightOk) {
    return <Diagnostics preflight={preflight} busy={busy} loading={loading} onRefresh={refresh} />
  }

  return (
    <main className="nu-cmd">
      <header className="nu-cmd-header">
        <h1 className="nu-cmd-brand">
          NeverUpdate&nbsp;<span className="nu-cmd-brand-emoticon">(˶˃ ᵕ ˂˶)</span>
        </h1>
        <div className="nu-cmd-toolbar">
          <span className="nu-cmd-badge-passive">前置已通过</span>
          <span className="nu-cmd-badge-passive">{daemonSnapshot?.runtime.running ? '守护运行中' : '守护未运行'}</span>
          <button className="nu-btn nu-btn-ghost" disabled={busy || loading} type="button" onClick={refresh}>
            刷新数据
          </button>
        </div>
      </header>

      {lastError ? <div className="nu-cmd-error">{lastError}</div> : null}

      <StatusHero loading={loading} busy={busy} preflight={preflight} statuses={statuses} daemonSnapshot={daemonSnapshot} onExecuteAll={executeAll} />

      <div className="nu-cmd-body">
        <section className="nu-cmd-main">
          <GuardMatrix
            busy={busy || loading}
            points={points}
            statuses={statuses}
            onAction={function (pointId, action) {
              if (pointId === EXTREME_POINT_ID) {
                runExtremeMode()
                return
              }
              executePoint(pointId, action)
            }}
          />
        </section>

        <aside className="nu-cmd-aside">
          <DaemonControl
            busy={busy || loading}
            snapshot={daemonSnapshot}
            onRegisterOrReregister={reregisterService}
            onToggleRunning={function (running) {
              if (running) {
                stopService()
                return
              }
              startService()
            }}
            onUnregister={unregisterService}
          />
          <Timeline history={history} />
        </aside>
      </div>
    </main>
  )
}
