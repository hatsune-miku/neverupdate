import { Card, List, Tag, Typography } from 'antd'

import type { HistoryEntry } from '@/types'

import './index.scss'

interface HistoryListProps {
  history: HistoryEntry[]
}

function formatDate(value: string): string {
  const date = new Date(value)
  return date.toLocaleString()
}

export function HistoryList({ history }: HistoryListProps) {
  return (
    <Card className="history-list-panel" title="历史记录" extra={<Tag>{history.length}</Tag>}>
      <List
        className="history-list-items"
        dataSource={history}
        itemLayout="vertical"
        renderItem={function (entry, index) {
          return (
            <List.Item key={`${entry.point_id}-${entry.timestamp}-${index}`}>
              <div className="history-item-heading">
                <Typography.Text strong>{entry.point_id}</Typography.Text>
                <Tag color={entry.success ? 'success' : 'error'}>{entry.action}</Tag>
              </div>
              <Typography.Text className="history-item-time" type="secondary">
                {formatDate(entry.timestamp)}
              </Typography.Text>
              <Typography.Text className="history-item-message">{entry.message || (entry.success ? '操作成功' : '操作失败')}</Typography.Text>
            </List.Item>
          )
        }}
      />
    </Card>
  )
}
