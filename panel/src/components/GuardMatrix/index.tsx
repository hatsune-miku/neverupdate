import { useMemo, useState } from 'react'

import { getGuardPrinciple } from '@/content/principles'
import type { GuardAction, GuardPointDefinition, GuardPointStatus } from '@/types'
import ReactMarkdown from 'react-markdown'

import './index.scss'

interface GuardMatrixProps {
  points: GuardPointDefinition[]
  statuses: GuardPointStatus[]
  busy: boolean
  onAction: (pointId: string, action: GuardAction) => void
}

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

function resolveAction(status: GuardPointStatus | null): ActionDescriptor {
  if (status?.breached) {
    return { action: 'guard', label: '阻断', tone: 'primary' }
  }
  if (status?.guarded) {
    return { action: 'release', label: '放开', tone: 'ghost' }
  }
  return { action: 'guard', label: '阻断', tone: 'primary' }
}

export function GuardMatrix({ points, statuses, busy, onAction }: GuardMatrixProps) {
  const [principlePointId, setPrinciplePointId] = useState<string | null>(null)

  const sorted = useMemo(
    function () {
      return points.filter(function (p) {
        return p.id !== 'extreme_mode'
      })
    },
    [points],
  )

  function handleAction(pointId: string, action: GuardAction) {
    onAction(pointId, action)
  }

  const principle = principlePointId ? getGuardPrinciple(principlePointId) : null

  return (
    <div className="guard-matrix">
      <div className="guard-matrix-list">
        {sorted.map(function (point) {
          const status = findStatus(statuses, point.id)
          const guarded = status?.guarded || false
          const breached = status?.breached || false
          const desc = resolveAction(status)

          const rowClass = ['guard-row', guarded ? 'guarded' : 'released', breached ? 'breached' : ''].filter(Boolean).join(' ')

          const tagLabel = breached ? '失守了！' : guarded ? '阻断中' : '已放开'
          const tagTone = breached ? 'kurenai' : guarded ? 'pink' : 'dim'
          const showGuardOkTag = guarded

          return (
            <article key={point.id} className={rowClass}>
              <div className="guard-row-body">
                <div className="guard-row-top">
                  <h4>{point.title}</h4>
                  <button
                    className="guard-row-help"
                    type="button"
                    aria-label={`${point.title} 技术原理`}
                    onClick={function () {
                      setPrinciplePointId(point.id)
                    }}
                  >
                    ?
                  </button>
                  <span className={`guard-row-tag ${tagTone}`}>
                    {showGuardOkTag ? (
                      <>
                        <svg className="guard-row-tag-icon" viewBox="0 0 16 16" aria-hidden>
                          <path
                            fill="currentColor"
                            d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 1 1 1.06-1.06l2.72 2.72 6.72-6.72a.75.75 0 0 1 1.06 0Z"
                          />
                        </svg>
                        阻断中
                      </>
                    ) : (
                      tagLabel
                    )}
                  </span>
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

      {principle ? (
        <div
          className="guard-principle-backdrop"
          role="presentation"
          onClick={function () {
            setPrinciplePointId(null)
          }}
        >
          <section
            className="guard-principle-modal"
            onClick={function (event) {
              event.stopPropagation()
            }}
          >
            <header className="guard-principle-header">
              <h3>{principle.title} · 技术原理</h3>
              <button className="nu-icon-btn" type="button" aria-label="关闭" onClick={function () { setPrinciplePointId(null) }}>
                <svg viewBox="0 0 16 16" aria-hidden>
                  <path
                    fill="currentColor"
                    d="M3.22 3.22a.75.75 0 0 1 1.06 0L8 6.94l3.72-3.72a.75.75 0 1 1 1.06 1.06L9.06 8l3.72 3.72a.75.75 0 0 1-1.06 1.06L8 9.06l-3.72 3.72a.75.75 0 1 1-1.06-1.06L6.94 8 3.22 4.28a.75.75 0 0 1 0-1.06Z"
                  />
                </svg>
              </button>
            </header>
            <div className="guard-principle-content">
              <ReactMarkdown>{principle.markdown}</ReactMarkdown>
            </div>
          </section>
        </div>
      ) : null}
    </div>
  )
}
