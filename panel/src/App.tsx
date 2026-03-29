import { useEffect, useState, type ReactNode } from 'react'

import { DaemonControl } from '@/components/DaemonControl'
import { AdditionalFeatures } from '@/components/AdditionalFeatures'
import { Diagnostics } from '@/components/Diagnostics'
import { Settings } from '@/components/Settings'
import { GuardMatrix } from '@/components/GuardMatrix'
import { RiskGate } from '@/components/RiskGate'
import { StatusHero } from '@/components/StatusHero'
import { UpdateToast } from '@/components/UpdateToast'
import { useAppStore } from '@/store'

import './App.scss'

const THEME_KEY = 'neverupdate-theme'

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
    startService,
    stopService,
    unregisterService,
    runExtremeMode,
    clearHistory,
    clearInterceptions,
    checkForUpdates,
    updaterCheckEnabled,
  } = useAppStore()
  const [showDiagnostics, setShowDiagnostics] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [theme, setTheme] = useState<'light' | 'dark'>(function () {
    return localStorage.getItem(THEME_KEY) === 'dark' ? 'dark' : 'light'
  })

  const preflightOk = preflight?.passed === true

  useEffect(
    function () {
      bootstrap()
    },
    [bootstrap],
  )

  useEffect(
    function () {
      document.documentElement.setAttribute('data-theme', theme)
      localStorage.setItem(THEME_KEY, theme)
    },
    [theme],
  )

  useEffect(
    function () {
      if (!riskAccepted || !updaterCheckEnabled) {
        return
      }
      void checkForUpdates({ fromAuto: true })
    },
    [riskAccepted, updaterCheckEnabled, checkForUpdates],
  )

  if (!riskAccepted) {
    return <RiskGate onAccept={acceptRisk} />
  }

  let shell: ReactNode
  if (showSettings) {
    shell = (
      <Settings
        onBack={function () {
          setShowSettings(false)
        }}
      />
    )
  } else if (!preflightOk || showDiagnostics) {
    shell = (
      <Diagnostics
        preflight={preflight}
        busy={busy}
        loading={loading}
        onRefresh={refresh}
        onBack={
          preflightOk
            ? function () {
                setShowDiagnostics(false)
              }
            : undefined
        }
      />
    )
  } else {
    shell = (
      <main className="nu-cmd">
        <header className="nu-cmd-header">
          <h1 className="nu-cmd-brand">
            NeverUpdate&nbsp;<span className="nu-cmd-brand-emoticon">(˶˃ ᵕ ˂˶)</span>
          </h1>
          <div className="nu-cmd-toolbar">
            <button
              className="nu-cmd-badge"
              type="button"
              onClick={function () {
                setShowDiagnostics(true)
              }}
            >
              前置检查
            </button>
            <span className="nu-cmd-toolbar-divider" aria-hidden />
            <button
              className="nu-icon-btn nu-cmd-toolbar-icon"
              type="button"
              onClick={function () {
                setShowSettings(true)
              }}
              aria-label="设置"
              title="设置"
            >
              <i className="fa-solid fa-gear" aria-hidden />
            </button>
            <button
              className="nu-icon-btn nu-cmd-toolbar-icon"
              type="button"
              onClick={function () {
                setTheme(theme === 'dark' ? 'light' : 'dark')
              }}
              aria-label="切换暗色模式"
              title={theme === 'dark' ? '切换到亮色模式' : '切换到暗色模式'}
            >
              <i className="fa-solid fa-circle-half-stroke" aria-hidden />
            </button>
          </div>
        </header>

        {lastError ? <div className="nu-cmd-error">{lastError}</div> : null}

        <StatusHero loading={loading} busy={busy} preflight={preflight} statuses={statuses} daemonSnapshot={daemonSnapshot} onExecuteAll={executeAll} onRefresh={refresh} />

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
              onRegister={registerService}
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
              onClearInterceptions={clearInterceptions}
              onRunExtremeMode={runExtremeMode}
            />
          </aside>
        </div>
      </main>
    )
  }

  return (
    <>
      {shell}
      {!showSettings ? <UpdateToast /> : null}
    </>
  )
}
