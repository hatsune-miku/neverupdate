import { useEffect, useMemo, useState } from 'react'

import { Alert, Button, Card, Space, Tabs, Tag } from 'antd'

import { DaemonPanel } from '@/components/DaemonPanel'
import { GuardPointList } from '@/components/GuardPointList'
import { HistoryList } from '@/components/HistoryList'
import { PreflightList } from '@/components/PreflightList'
import { RiskNotice } from '@/components/RiskNotice'
import { useAppStore } from '@/store'

import './App.scss'

const EXTREME_POINT_ID = 'extreme_mode'

const TabsValues = ['guards', 'daemon'] as const
type Tab = (typeof TabsValues)[number]

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

  const [activeTab, setActiveTab] = useState<Tab>('guards')
  const [enteredConsole, setEnteredConsole] = useState(false)
  const preflightPassed = preflight?.passed === true

  useEffect(
    function () {
      bootstrap()
    },
    [bootstrap],
  )

  useEffect(
    function () {
      if (enteredConsole && !preflightPassed) {
        setEnteredConsole(false)
      }
    },
    [enteredConsole, preflightPassed],
  )

  const tabItems = useMemo(function () {
    return [
      {
        key: 'guards',
        label: '阻断点',
      },
      {
        key: 'daemon',
        label: 'Daemon 进程',
      },
    ]
  }, [])

  if (!riskAccepted) {
    return <RiskNotice onAccept={acceptRisk} />
  }

  if (!enteredConsole) {
    return (
      <main className="nu-preflight-route">
        <Card className="nu-preflight-card" bordered={false}>
          <PreflightList preflight={preflight} />

          <Space className="nu-preflight-actions" size={8}>
            <Button loading={busy || loading} onClick={refresh}>
              刷新检查
            </Button>
            <Button
              type="primary"
              disabled={loading || !preflightPassed}
              onClick={function () {
                if (preflightPassed) {
                  setEnteredConsole(true)
                }
              }}
            >
              进入控制台
            </Button>
          </Space>
        </Card>
      </main>
    )
  }

  return (
    <main className="nu-shell">
      <header className="nu-toolbar">
        <Tabs
          activeKey={activeTab}
          className="nu-tabs"
          items={tabItems}
          size="small"
          onChange={function (nextKey) {
            if (nextKey === 'guards' || nextKey === 'daemon') {
              setActiveTab(nextKey)
            }
          }}
        />

        <Space className="nu-toolbar-meta" size={6}>
          <Tag
            className="nu-clickable-tag"
            color={preflight?.passed ? 'success' : 'error'}
            onClick={function () {
              setEnteredConsole(false)
            }}
          >
            {preflight?.passed ? '前置检查通过' : '前置检查异常'}
          </Tag>
          <Tag color={daemonSnapshot?.runtime.running ? 'success' : 'default'}>{daemonSnapshot?.runtime.running ? '守护运行中' : '守护未运行'}</Tag>
        </Space>
      </header>

      {lastError ? <Alert className="nu-error-bar" message={lastError} showIcon type="error" /> : null}

      {activeTab === 'guards' ? (
        <section className="nu-content-guards">
          <Space className="nu-guard-actions" size={8} wrap>
            <Button loading={busy || loading} onClick={refresh}>
              刷新
            </Button>
            <Button
              disabled={busy || loading}
              onClick={function () {
                executeAll('guard')
              }}
            >
              一键阻断
            </Button>
            <Button
              disabled={busy || loading}
              onClick={function () {
                executeAll('release')
              }}
            >
              一键放开
            </Button>
            <Button
              disabled={busy || loading}
              onClick={function () {
                executeAll('repair')
              }}
            >
              一键修复
            </Button>
          </Space>

          <GuardPointList
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
      ) : (
        <section className="nu-content-daemon">
          <Space className="nu-daemon-actions-row" size={8} wrap>
            <Button loading={busy || loading} onClick={refresh}>
              刷新
            </Button>
          </Space>

          <DaemonPanel
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

          <HistoryList history={history} />
        </section>
      )}
    </main>
  )
}
