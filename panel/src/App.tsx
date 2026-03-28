import { useEffect, useState } from 'react'

import { DaemonControl } from '@/components/DaemonControl'
import { AdditionalFeatures } from '@/components/AdditionalFeatures'
import { Diagnostics } from '@/components/Diagnostics'
import { GuardMatrix } from '@/components/GuardMatrix'
import { RiskGate } from '@/components/RiskGate'
import { StatusHero } from '@/components/StatusHero'
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
    updateAvailable,
    updateStatus,
    updateMessage,
    updateDownloaded,
    updateTotal,
    checkForUpdates,
    installUpdate,
  } = useAppStore()
  const [showDiagnostics, setShowDiagnostics] = useState(false)
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
      if (!riskAccepted) {
        return
      }
      void checkForUpdates()
    },
    [riskAccepted, checkForUpdates],
  )

  if (!riskAccepted) {
    return <RiskGate onAccept={acceptRisk} />
  }

  if (!preflightOk || showDiagnostics) {
    return (
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
  }

  return (
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
          <button
            className="nu-btn nu-btn-ghost"
            disabled={updateStatus === 'checking' || updateStatus === 'downloading'}
            type="button"
            onClick={function () {
              void checkForUpdates()
            }}
          >
            {updateStatus === 'checking' ? '检查中' : '检查更新'}
          </button>
          {updateAvailable ? (
            <button
              className="nu-btn nu-btn-primary"
              disabled={updateStatus === 'downloading'}
              type="button"
              onClick={function () {
                void installUpdate()
              }}
            >
              {updateStatus === 'downloading' ? '更新中' : `更新到 ${updateAvailable.version}`}
            </button>
          ) : null}
          <span className="nu-cmd-toolbar-divider" aria-hidden />
          <button
            className="nu-icon-btn nu-cmd-toolbar-icon"
            type="button"
            onClick={function () {
              setTheme(theme === 'dark' ? 'light' : 'dark')
            }}
            aria-label="切换暗色模式"
            title={theme === 'dark' ? '切换到亮色模式' : '切换到暗色模式'}
          >
            <i className={theme === 'dark' ? 'fa-regular fa-sun' : 'fa-regular fa-moon'} aria-hidden />
          </button>
          <button
            className="nu-icon-btn nu-cmd-toolbar-icon"
            disabled={busy || loading}
            type="button"
            onClick={refresh}
            aria-label="刷新数据"
            title="刷新数据"
          >
            <i className="fa-solid fa-arrows-rotate" aria-hidden />
          </button>
        </div>
      </header>

      {lastError ? <div className="nu-cmd-error">{lastError}</div> : null}
      {updateMessage ? (
        <div className={`nu-cmd-update ${updateStatus === 'error' ? 'error' : ''}`}>
          <span>{updateMessage}</span>
          {updateStatus === 'downloading' && updateTotal ? (
            <span className="nu-mono">
              {Math.round((updateDownloaded / updateTotal) * 100)}%
            </span>
          ) : null}
        </div>
      ) : null}

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
            onRunExtremeMode={runExtremeMode}
          />
        </aside>
      </div>
    </main>
  )
}
