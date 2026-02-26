import { create } from 'zustand'

import { bridge } from '@/bridge'
import type { DaemonSnapshot, GuardAction, GuardPointDefinition, GuardPointStatus, GuardSummary, HistoryEntry, PreflightReport } from '@/types'

interface AppState {
  loading: boolean
  busy: boolean
  lastError: string | null

  riskAccepted: boolean
  preflight: PreflightReport | null
  points: GuardPointDefinition[]
  statuses: GuardPointStatus[]
  history: HistoryEntry[]
  daemonSnapshot: DaemonSnapshot | null

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
}

const RISK_KEY = 'neverupdate-risk-accepted'

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
  daemonSnapshot: null,

  bootstrap: async function () {
    set({ loading: true, lastError: null })

    await Promise.all([
      bridge.runPreflightChecks().then((preflight) => set({ preflight })),
      bridge.listGuardPoints().then((points) => set({ points })),
      bridge.queryGuardStates().then((statuses) => set({ statuses })),
      bridge.readHistory(200).then((history) => set({ history })),
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
      const [preflight, statuses, history, daemonSnapshot] = await Promise.all([bridge.runPreflightChecks(), bridge.queryGuardStates(), bridge.readHistory(200), bridge.daemonSnapshot()])

      set({ preflight, statuses, history, daemonSnapshot })
    })
  },

  executePoint: async function (pointId: string, action: GuardAction) {
    await withBusy(set, async function () {
      const updated = await bridge.executeGuardAction(pointId, action)
      const old = get().statuses
      const statuses = old.map((item) => (item.id === updated.id ? updated : item))
      set({ statuses })

      const history = await bridge.readHistory(200)
      set({ history })
    })
  },

  executeAll: async function (action: GuardAction) {
    await withBusy(set, async function () {
      const summary: GuardSummary = await bridge.executeAll(action)

      const statuses = await bridge.queryGuardStates()
      set({ statuses })

      const history = await bridge.readHistory(200)
      set({ history })

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
      const history = await bridge.readHistory(200)
      set({ statuses, history })
    })
  },
}))
