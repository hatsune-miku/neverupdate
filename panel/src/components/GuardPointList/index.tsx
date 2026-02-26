import { useMemo, useState } from 'react'

import { Button, Card, Col, Row, Tag, Typography } from 'antd'

import type { GuardAction, GuardPointDefinition, GuardPointStatus } from '@/types'

import './index.scss'

interface GuardPointListProps {
  points: GuardPointDefinition[]
  statuses: GuardPointStatus[]
  busy: boolean
  onAction: (pointId: string, action: GuardAction) => void
}

const EXTREME_POINT_ID = 'extreme_mode'

interface PointActionDescriptor {
  action: GuardAction
  label: string
}

function findStatus(statuses: GuardPointStatus[], pointId: string): GuardPointStatus | null {
  const item = statuses.find(function (status) {
    return status.id === pointId
  })

  return item || null
}

function resolvePrimaryAction(pointId: string, status: GuardPointStatus | null, extremeArmed: boolean): PointActionDescriptor {
  if (pointId === EXTREME_POINT_ID) {
    if (extremeArmed) {
      return { action: 'guard', label: '再次确认执行' }
    }

    return { action: 'guard', label: '执行（需二次确认）' }
  }

  const breached = status?.breached || false
  const guarded = status?.guarded || false

  if (breached) {
    return { action: 'repair', label: '修复' }
  }

  if (guarded) {
    return { action: 'release', label: '放开' }
  }

  return { action: 'guard', label: '阻断' }
}

export function GuardPointList({ points, statuses, busy, onAction }: GuardPointListProps) {
  const [extremeArmed, setExtremeArmed] = useState(false)

  const sorted = useMemo(
    function () {
      const normal = points.filter(function (point) {
        return point.id !== EXTREME_POINT_ID
      })
      const extreme = points.filter(function (point) {
        return point.id === EXTREME_POINT_ID
      })

      return [...normal, ...extreme]
    },
    [points],
  )

  function handlePointAction(pointId: string, action: GuardAction) {
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
    <Row className="guard-point-list" gutter={[10, 10]}>
      {sorted.map(function (point) {
        const status = findStatus(statuses, point.id)
        const guarded = status?.guarded || false
        const breached = status?.breached || false
        const primaryAction = resolvePrimaryAction(point.id, status, extremeArmed)
        const actionDanger = primaryAction.action === 'repair' || point.id === EXTREME_POINT_ID

        return (
          <Col key={point.id} lg={8} md={12} sm={24} xs={24}>
            <Card hoverable className={`guard-point-card ${guarded ? 'guarded' : 'released'} ${breached ? 'breached' : ''} ${point.id === EXTREME_POINT_ID ? 'extreme' : ''}`}>
              <div className="guard-point-heading">
                <Typography.Title level={5}>{point.title}</Typography.Title>
                <Tag color={point.id === EXTREME_POINT_ID ? 'error' : guarded ? 'success' : 'default'}>{point.id === EXTREME_POINT_ID ? '高风险' : guarded ? '阻断中' : '已放开'}</Tag>
              </div>

              <Typography.Paragraph className="guard-point-description">{point.description}</Typography.Paragraph>

              <Typography.Paragraph className="guard-point-message">{status?.message || '暂无额外信息'}</Typography.Paragraph>

              <div className="guard-point-actions">
                <Button
                  block
                  danger={actionDanger}
                  disabled={busy}
                  type={primaryAction.action === 'release' ? 'default' : 'primary'}
                  onClick={function () {
                    handlePointAction(point.id, primaryAction.action)
                  }}
                >
                  {primaryAction.label}
                </Button>
              </div>
            </Card>
          </Col>
        )
      })}
    </Row>
  )
}
