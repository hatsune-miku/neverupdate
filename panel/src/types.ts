export interface GuardPointDefinition {
  id: string
  title: string
  description: string
}

export interface GuardPointStatus {
  id: string
  title: string
  guarded: boolean
  breached: boolean
  message?: string
  checked_at: string
}

export interface GuardSummary {
  statuses: GuardPointStatus[]
  errors: string[]
}

export interface PreflightCheck {
  id: string
  title: string
  passed: boolean
  detail: string
}

export interface PreflightReport {
  passed: boolean
  checks: PreflightCheck[]
}

export interface HistoryEntry {
  point_id: string
  action: GuardAction
  success: boolean
  timestamp: string
  message?: string
}

export interface DaemonRuntimeStatus {
  running: boolean
  service_registered: boolean
  service_name: string
}

export interface DaemonSnapshot {
  timestamp: string
  statuses: GuardPointStatus[]
  message?: string
  runtime: DaemonRuntimeStatus
}

export const GuardActions = ['guard', 'release', 'repair'] as const
export type GuardAction = (typeof GuardActions)[number]
