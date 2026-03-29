import { create } from 'zustand'

import { bridge } from '@/bridge'
import type { DaemonSnapshot, GuardAction, GuardPointDefinition, GuardPointStatus, GuardSummary, HistoryEntry, InterceptionEntry, PreflightReport } from '@/types'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'

interface AppState {
  loading: boolean
  busy: boolean
  lastError: string | null

  riskAccepted: boolean
  preflight: PreflightReport | null
  points: GuardPointDefinition[]
  statuses: GuardPointStatus[]
  history: HistoryEntry[]
  interceptions: InterceptionEntry[]
  daemonSnapshot: DaemonSnapshot | null
  updateAvailable: Awaited<ReturnType<typeof check>>
  updateStatus: 'idle' | 'checking' | 'ready' | 'latest' | 'downloading' | 'error'
  updateMessage: string
  updateDownloaded: number
  updateTotal: number | null
  updaterCheckEnabled: boolean
  updateToastVisible: boolean

  bootstrap: () => Promise<void>
  acceptRisk: () => void
  refresh: () => Promise<void>
  executePoint: (pointId: string, action: GuardAction) => Promise<void>
  executeAll: (action: GuardAction) => Promise<void>

  registerService: () => Promise<void>
  reregisterService: () => Promise<void>
  startService: () => Promise<void>
  stopService: () => Promise<void>
  unregisterService: () => Promise<void>

  runExtremeMode: () => Promise<void>
  clearHistory: () => Promise<void>
  clearInterceptions: () => Promise<void>
  checkForUpdates: (opts?: { fromAuto?: boolean }) => Promise<void>
  installUpdate: () => Promise<void>
  setUpdaterCheckEnabled: (enabled: boolean) => void
  dismissUpdateToast: () => void
}

const RISK_KEY = 'neverupdate-risk-accepted'
const UPDATER_CHECK_KEY = 'neverupdate-updater-check'

function readUpdaterCheckEnabled() {
  if (typeof localStorage === 'undefined') {
    return true
  }
  return localStorage.getItem(UPDATER_CHECK_KEY) !== '0'
}

function withBusy<T>(set: any, fn: () => Promise<T>) {
  set({ busy: true, lastError: null })
  return fn()
    .catch((error) => {
      set({ lastError: String(error) })
      return null
    })
    .finally(() => {
      set({ busy: false })
    })
}

export const useAppStore = create<AppState>((set, get) => ({
  loading: true,
  busy: false,
  lastError: null,

  riskAccepted: localStorage.getItem(RISK_KEY) === '1',
  preflight: null,
  points: [],
  statuses: [],
  history: [],
  interceptions: [],
  daemonSnapshot: null,
  updateAvailable: null,
  updateStatus: 'idle',
  updateMessage: '',
  updateDownloaded: 0,
  updateTotal: null,
  updaterCheckEnabled: readUpdaterCheckEnabled(),
  updateToastVisible: false,

  bootstrap: async function () {
    set({ loading: true, lastError: null })

    await Promise.all([
      bridge.runPreflightChecks().then((preflight) => set({ preflight })),
      bridge.listGuardPoints().then((points) => set({ points })),
      bridge.queryGuardStates().then((statuses) => set({ statuses })),
      bridge.readHistory(500).then((history) => set({ history })),
      bridge.readInterceptions(500).then((interceptions) => set({ interceptions })),
      bridge.daemonSnapshot().then((daemonSnapshot) => set({ daemonSnapshot })),
    ]).catch((error) => {
      set({ lastError: String(error) })
    })

    set({ loading: false })
  },

  acceptRisk: function () {
    localStorage.setItem(RISK_KEY, '1')
    set({ riskAccepted: true })
  },

  refresh: async function () {
    await withBusy(set, async function () {
      const [preflight, statuses, history, interceptions, daemonSnapshot] = await Promise.all([
        bridge.runPreflightChecks(),
        bridge.queryGuardStates(),
        bridge.readHistory(500),
        bridge.readInterceptions(500),
        bridge.daemonSnapshot(),
      ])

      set({ preflight, statuses, history, interceptions, daemonSnapshot })
    })
  },

  executePoint: async function (pointId: string, action: GuardAction) {
    await withBusy(set, async function () {
      const updated = await bridge.executeGuardAction(pointId, action)
      const old = get().statuses
      const statuses = old.map((item) => (item.id === updated.id ? updated : item))
      set({ statuses })

      const [history, interceptions] = await Promise.all([bridge.readHistory(500), bridge.readInterceptions(500)])
      set({ history, interceptions })
    })
  },

  executeAll: async function (action: GuardAction) {
    await withBusy(set, async function () {
      const summary: GuardSummary = await bridge.executeAll(action)

      const statuses = await bridge.queryGuardStates()
      set({ statuses })

      const [history, interceptions] = await Promise.all([bridge.readHistory(500), bridge.readInterceptions(500)])
      set({ history, interceptions })

      if (summary.errors.length > 0) {
        set({ lastError: summary.errors.join(' | ') })
      }
    })
  },

  registerService: async function () {
    await withBusy(set, async function () {
      await bridge.daemonServiceRegister()
      const daemonSnapshot = await bridge.daemonSnapshot()
      set({ daemonSnapshot })
    })
  },

  reregisterService: async function () {
    await withBusy(set, async function () {
      await bridge.daemonServiceReregister()
      const daemonSnapshot = await bridge.daemonSnapshot()
      set({ daemonSnapshot })
    })
  },

  startService: async function () {
    await withBusy(set, async function () {
      await bridge.daemonServiceStart()
      const daemonSnapshot = await bridge.daemonSnapshot()
      set({ daemonSnapshot })
    })
  },

  stopService: async function () {
    await withBusy(set, async function () {
      await bridge.daemonServiceStop()
      const daemonSnapshot = await bridge.daemonSnapshot()
      set({ daemonSnapshot })
    })
  },

  unregisterService: async function () {
    await withBusy(set, async function () {
      await bridge.daemonServiceUnregister()
      const daemonSnapshot = await bridge.daemonSnapshot()
      set({ daemonSnapshot })
    })
  },

  runExtremeMode: async function () {
    await withBusy(set, async function () {
      await bridge.runExtremeMode()
      const statuses = await bridge.queryGuardStates()
      const [history, interceptions] = await Promise.all([bridge.readHistory(500), bridge.readInterceptions(500)])
      set({ statuses, history, interceptions })
    })
  },

  clearHistory: async function () {
    await withBusy(set, async function () {
      await bridge.clearHistory()
      set({ history: [] })
    })
  },

  clearInterceptions: async function () {
    await withBusy(set, async function () {
      await bridge.clearInterceptions()
      set({ interceptions: [] })
    })
  },

  setUpdaterCheckEnabled: function (enabled: boolean) {
    localStorage.setItem(UPDATER_CHECK_KEY, enabled ? '1' : '0')
    if (!enabled) {
      set({
        updaterCheckEnabled: false,
        updateAvailable: null,
        updateStatus: 'idle',
        updateMessage: '',
        updateDownloaded: 0,
        updateTotal: null,
        updateToastVisible: false,
      })
      return
    }
    set({ updaterCheckEnabled: true })
  },

  dismissUpdateToast: function () {
    set({ updateToastVisible: false })
  },

  checkForUpdates: async function (opts?: { fromAuto?: boolean }) {
    if (!get().updaterCheckEnabled) {
      return
    }
    const fromAuto = opts?.fromAuto === true
    if (!fromAuto) {
      set({ updateToastVisible: false })
    }
    set({ updateStatus: 'checking', updateMessage: '' })
    try {
      const update = await check()
      if (update) {
        set({
          updateAvailable: update,
          updateStatus: 'ready',
          updateMessage: `发现新版本 ${update.version}`,
          ...(fromAuto ? { updateToastVisible: true } : { updateToastVisible: false }),
        })
        return
      }
      set({
        updateAvailable: null,
        updateStatus: 'latest',
        updateMessage: '已经是最新版本',
        updateToastVisible: false,
      })
    } catch (error) {
      set({
        updateStatus: 'error',
        updateMessage: error instanceof Error ? error.message : '检查更新失败',
        updateToastVisible: false,
      })
    }
  },

  installUpdate: async function () {
    if (!get().updaterCheckEnabled) {
      return
    }
    const update = get().updateAvailable
    if (!update) {
      return
    }
    set({
      updateStatus: 'downloading',
      updateMessage: '',
      updateDownloaded: 0,
      updateTotal: null,
    })
    try {
      await update.downloadAndInstall(function (event) {
        if (event.event === 'Started') {
          set({ updateTotal: event.data.contentLength })
          return
        }
        if (event.event === 'Progress') {
          set((state) => ({ updateDownloaded: state.updateDownloaded + event.data.chunkLength }))
        }
      })
      set({ updateMessage: '更新已就绪，正在重启...' })
      await relaunch()
    } catch (error) {
      set({
        updateStatus: 'error',
        updateMessage: error instanceof Error ? error.message : '安装更新失败',
      })
    }
  },
}))
