import { useEffect } from 'react'

import { DaemonControl } from '@/components/DaemonControl'
import { AdditionalFeatures } from '@/components/AdditionalFeatures'
import { Diagnostics } from '@/components/Diagnostics'
import { GuardMatrix } from '@/components/GuardMatrix'
import { RiskGate } from '@/components/RiskGate'
import { StatusHero } from '@/components/StatusHero'
import { useAppStore } from '@/store'

import './App.scss'

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
    interceptions,
    daemonSnapshot,
    bootstrap,
    acceptRisk,
    refresh,
    executePoint,
    executeAll,
    registerService,
    reregisterService,
    startService,
    stopService,
    unregisterService,
    runExtremeMode,
    clearHistory,
  } = useAppStore()

  const preflightOk = preflight?.passed === true

  useEffect(
    function () {
      bootstrap()
    },
    [bootstrap],
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
              executePoint(pointId, action)
            }}
          />
        </section>

        <aside className="nu-cmd-aside">
          <DaemonControl
            busy={busy || loading}
            snapshot={daemonSnapshot}
            onRegisterOrReregister={
              daemonSnapshot?.runtime.service_registered ? reregisterService : registerService
            }
            onToggleRunning={function (running) {
              if (running) {
                stopService()
                return
              }
              startService()
            }}
            onUnregister={unregisterService}
          />
          <AdditionalFeatures
            busy={busy || loading}
            history={history}
            interceptions={interceptions}
            onClearHistory={clearHistory}
            onRunExtremeMode={runExtremeMode}
          />
        </aside>
      </div>
    </main>
  )
}
