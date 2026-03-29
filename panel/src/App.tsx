import { type ReactNode, useEffect, useState } from 'react'

import { AdditionalFeatures } from '@/components/AdditionalFeatures'
import { DaemonControl } from '@/components/DaemonControl'
import { Diagnostics } from '@/components/Diagnostics'
import { GuardMatrix } from '@/components/GuardMatrix'
import { RiskGate } from '@/components/RiskGate'
import { Settings } from '@/components/Settings'
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
    updateAvailable,
    updateStatus,
  } = useAppStore()
  const [showDiagnostics, setShowDiagnostics] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [theme, setTheme] = useState<'light' | 'dark'>(() => (localStorage.getItem(THEME_KEY) === 'dark' ? 'dark' : 'light'))

  const preflightOk = preflight?.passed === true

  useEffect(() => {
    bootstrap()
  }, [bootstrap])

  useEffect(() => {
    const root = document.documentElement
    root.classList.add('nu-disable-theme-transition')
    root.setAttribute('data-theme', theme)
    localStorage.setItem(THEME_KEY, theme)

    let raf2 = 0
    const raf1 = window.requestAnimationFrame(() => {
      raf2 = window.requestAnimationFrame(() => {
        root.classList.remove('nu-disable-theme-transition')
      })
    })

    return () => {
      window.cancelAnimationFrame(raf1)
      if (raf2) {
        window.cancelAnimationFrame(raf2)
      }
      root.classList.remove('nu-disable-theme-transition')
    }
  }, [theme])

  useEffect(() => {
    if (!riskAccepted || !updaterCheckEnabled) {
      return
    }
    void checkForUpdates({ fromAuto: true })
  }, [riskAccepted, updaterCheckEnabled, checkForUpdates])

  useEffect(() => {
    if (!showSettings) {
      return
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setShowSettings(false)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [showSettings])

  if (!riskAccepted) {
    return <RiskGate onAccept={acceptRisk} />
  }

  let shell: ReactNode
  if (showSettings) {
    shell = (
      <Settings
        onBack={() => {
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
            ? () => {
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
              onClick={() => {
                setShowDiagnostics(true)
              }}
            >
              前置检查
            </button>
            <span className="nu-cmd-toolbar-divider" aria-hidden />
            <button
              className="nu-icon-btn nu-cmd-toolbar-icon"
              type="button"
              onClick={() => {
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
              onClick={() => {
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
              onAction={(pointId, action) => {
                executePoint(pointId, action)
              }}
            />
          </section>

          <aside className="nu-cmd-aside">
            <DaemonControl
              busy={busy || loading}
              snapshot={daemonSnapshot}
              onRegister={registerService}
              onToggleRunning={(running) => {
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
      {!showSettings ? (
        <UpdateToast
          onOpenSettings={() => {
            setShowSettings(true)
            if (!updaterCheckEnabled) {
              return
            }
            if (updateStatus === 'checking' || updateStatus === 'downloading' || updateAvailable) {
              return
            }
            void checkForUpdates()
          }}
        />
      ) : null}
    </>
  )
}
