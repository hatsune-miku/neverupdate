import { Button, Card, Descriptions, Space, Tag, Typography } from 'antd'

import type { DaemonSnapshot } from '@/types'

import './index.scss'

interface DaemonPanelProps {
  busy: boolean
  snapshot: DaemonSnapshot | null
  onRegisterOrReregister: () => void
  onToggleRunning: (running: boolean) => void
  onUnregister: () => void
}

export function DaemonPanel({ busy, snapshot, onRegisterOrReregister, onToggleRunning, onUnregister }: DaemonPanelProps) {
  const runtime = snapshot?.runtime
  const heartbeat = snapshot?.timestamp ? new Date(snapshot.timestamp).toLocaleString() : '无'
  const running = runtime?.running || false

  return (
    <Card className="daemon-panel" title="守护进程">
      <Space className="daemon-panel-header" size={8}>
        <Tag color={running ? 'success' : 'default'}>{running ? '运行中' : '未检测到心跳'}</Tag>
        <Typography.Text type="secondary">状态随最近快照刷新</Typography.Text>
      </Space>

      <Descriptions className="daemon-panel-metrics" column={3} size="small">
        <Descriptions.Item label="服务名">{runtime?.service_name || 'NeverUpdateDaemon'}</Descriptions.Item>
        <Descriptions.Item label="注册状态">{runtime?.service_registered ? '已注册' : '未注册'}</Descriptions.Item>
        <Descriptions.Item label="最近心跳">{heartbeat}</Descriptions.Item>
      </Descriptions>

      <Space className="daemon-panel-actions" size={8} wrap>
        <Button disabled={busy} onClick={onRegisterOrReregister}>
          (重新)注册服务
        </Button>
        <Button
          disabled={busy}
          type="primary"
          onClick={function () {
            onToggleRunning(running)
          }}
        >
          {running ? '停止服务' : '启动服务'}
        </Button>
        <Button danger disabled={busy} onClick={onUnregister}>
          卸载服务
        </Button>
      </Space>
    </Card>
  )
}
