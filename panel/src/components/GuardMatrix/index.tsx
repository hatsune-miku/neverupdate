import { useMemo, useState } from 'react'

import type { GuardAction, GuardPointDefinition, GuardPointStatus } from '@/types'

import './index.scss'

interface GuardMatrixProps {
  points: GuardPointDefinition[]
  statuses: GuardPointStatus[]
  busy: boolean
  onAction: (pointId: string, action: GuardAction) => void
}

const EXTREME_POINT_ID = 'extreme_mode'

interface ActionDescriptor {
  action: GuardAction
  label: string
  tone: string
}

function findStatus(statuses: GuardPointStatus[], pointId: string): GuardPointStatus | null {
  return (
    statuses.find(function (s) {
      return s.id === pointId
    }) || null
  )
}

function resolveAction(pointId: string, status: GuardPointStatus | null, extremeArmed: boolean): ActionDescriptor {
  if (pointId === EXTREME_POINT_ID) {
    if (extremeArmed) {
      return { action: 'guard', label: '确认执行', tone: 'danger' }
    }
    return { action: 'guard', label: '执行', tone: 'danger' }
  }

  if (status?.breached) {
    return { action: 'repair', label: '修复', tone: 'danger' }
  }
  if (status?.guarded) {
    return { action: 'release', label: '放开', tone: 'ghost' }
  }
  return { action: 'guard', label: '阻断', tone: 'primary' }
}

export function GuardMatrix({ points, statuses, busy, onAction }: GuardMatrixProps) {
  const [extremeArmed, setExtremeArmed] = useState(false)

  const sorted = useMemo(
    function () {
      const normal = points.filter(function (p) {
        return p.id !== EXTREME_POINT_ID
      })
      const extreme = points.filter(function (p) {
        return p.id === EXTREME_POINT_ID
      })
      return [...normal, ...extreme]
    },
    [points],
  )

  function handleAction(pointId: string, action: GuardAction) {
    if (pointId !== EXTREME_POINT_ID) {
      onAction(pointId, action)
      return
    }
    if (!extremeArmed) {
      setExtremeArmed(true)
      window.setTimeout(function () {
        setExtremeArmed(false)
      }, 8000)
      return
    }
    onAction(pointId, action)
    setExtremeArmed(false)
  }

  return (
    <div className="guard-matrix">
      <header className="guard-matrix-header">
        <h3>检查点</h3>
        <span className="guard-matrix-count">{statuses.length} 个检查点</span>
      </header>

      <div className="guard-matrix-list">
        {sorted.map(function (point) {
          const status = findStatus(statuses, point.id)
          const guarded = status?.guarded || false
          const breached = status?.breached || false
          const extreme = point.id === EXTREME_POINT_ID
          const desc = resolveAction(point.id, status, extremeArmed)

          const rowClass = ['guard-row', guarded ? 'guarded' : 'released', breached ? 'breached' : '', extreme ? 'extreme' : ''].filter(Boolean).join(' ')

          const tagLabel = extreme ? '高危' : breached ? '失守了！' : guarded ? '阻断中' : '已放开'
          const tagTone = extreme ? 'kurenai' : breached ? 'kurenai' : guarded ? 'pink' : 'dim'

          return (
            <article key={point.id} className={rowClass}>
              <div className="guard-row-body">
                <div className="guard-row-top">
                  <h4>{point.title}</h4>
                  <span className={`guard-row-tag ${tagTone}`}>{tagLabel}</span>
                </div>
                <p className="guard-row-desc">{point.description}</p>
                <div className="guard-row-meta nu-mono">
                  {status?.message ? <span>{status.message}</span> : null}
                  {status?.checked_at ? <span>{new Date(status.checked_at).toLocaleTimeString()}</span> : null}
                </div>
              </div>
              <div className="guard-row-action">
                <button
                  className={`nu-btn nu-btn-${desc.tone}`}
                  disabled={busy}
                  type="button"
                  onClick={function () {
                    handleAction(point.id, desc.action)
                  }}
                >
                  {desc.label}
                </button>
              </div>
            </article>
          )
        })}
      </div>
    </div>
  )
}
