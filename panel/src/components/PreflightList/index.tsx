import { Card, Col, Row, Tag, Typography } from 'antd'

import type { PreflightReport } from '@/types'

import './index.scss'

interface PreflightListProps {
  preflight: PreflightReport | null
}

export function PreflightList({ preflight }: PreflightListProps) {
  if (!preflight) {
    return <Card className="preflight-list">正在读取前置检查...</Card>
  }

  const failedCount = preflight.checks.filter(function (item) {
    return !item.passed
  }).length

  return (
    <Card className="preflight-list" extra={<Tag color={preflight.passed ? 'success' : 'error'}>{preflight.passed ? '全部通过' : `${failedCount} 项未通过`}</Tag>} title="前置系统检查">
      <Row gutter={[8, 8]}>
        {preflight.checks.map(function (item) {
          return (
            <Col key={item.id} lg={8} md={12} sm={24} xs={24}>
              <Card className={`preflight-item ${item.passed ? 'ok' : 'bad'}`} size="small">
                <Typography.Text strong>{item.title}</Typography.Text>
                <Typography.Paragraph>{item.detail}</Typography.Paragraph>
              </Card>
            </Col>
          )
        })}
      </Row>
    </Card>
  )
}
