import type { DaemonSnapshot, GuardAction, GuardPointDefinition, GuardPointStatus, GuardSummary, HistoryEntry, PreflightReport } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export const bridge = {
  runPreflightChecks: function (): Promise<PreflightReport> {
    return invoke('run_preflight_checks_cmd')
  },

  listGuardPoints: function (): Promise<GuardPointDefinition[]> {
    return invoke('list_guard_points_cmd')
  },

  queryGuardStates: function (): Promise<GuardPointStatus[]> {
    return invoke('query_guard_states_cmd')
  },

  executeGuardAction: function (pointId: string, action: GuardAction): Promise<GuardPointStatus> {
    return invoke('execute_guard_action_cmd', { pointId, action })
  },

  executeAll: function (action: GuardAction): Promise<GuardSummary> {
    return invoke('execute_all_cmd', { action })
  },

  readHistory: function (limit: number): Promise<HistoryEntry[]> {
    return invoke('read_history_cmd', { limit })
  },

  daemonSnapshot: function (): Promise<DaemonSnapshot | null> {
    return invoke('daemon_snapshot_cmd')
  },

  daemonServiceRegister: function (): Promise<boolean> {
    return invoke('daemon_service_register')
  },

  daemonServiceReregister: function (): Promise<boolean> {
    return invoke('daemon_service_reregister')
  },

  daemonServiceStart: function (): Promise<boolean> {
    return invoke('daemon_service_start')
  },

  daemonServiceStop: function (): Promise<boolean> {
    return invoke('daemon_service_stop')
  },

  daemonServiceUnregister: function (): Promise<boolean> {
    return invoke('daemon_service_unregister')
  },

  runExtremeMode: function (): Promise<boolean> {
    return invoke('run_extreme_mode_cmd')
  },
}
