import { useMemo, useState } from 'react'
import type { UIEvent } from 'react'

import './index.scss'

interface RiskGateProps {
  onAccept: () => void
}

export function RiskGate({ onAccept }: RiskGateProps) {
  const [scrollReached, setScrollReached] = useState(false)

  const clauses = useMemo(function () {
    return [
      'NeverUpdate 将以管理员权限持续修改系统更新相关的注册表、服务、防火墙规则和组策略设置。',
      '这意味着你的设备将无法接收 Windows 安全补丁与功能更新，可能增大安全风险。',
      '你应当理解这些操作的全部后果——包括但不限于系统稳定性降低、安全漏洞无法修补。',
      '如果你没有完全理解上述影响，请立即关闭本程序。',
      '本工具不会替你承担任何数据丢失、系统损坏或安全事故的责任。',
      '继续使用即代表你明确知晓：你正在主动阻断 Windows Update 默认行为，并对系统更新链路施加强干预。',
    ]
  }, [])

  function handleScroll(event: UIEvent<HTMLDivElement>) {
    const el = event.currentTarget
    if (el.scrollTop + el.clientHeight + 4 >= el.scrollHeight) {
      setScrollReached(true)
    }
  }

  return (
    <div className="risk-gate">
      <div className="risk-gate-card">
        <div className="risk-gate-icon">!</div>
        <h1>使用须知</h1>
        <p className="risk-gate-subtitle">在继续使用之前，请仔细阅读以下内容</p>

        <div className="risk-gate-scroll" onScroll={handleScroll}>
          <div className="risk-gate-clauses">
            {clauses.map(function (text, i) {
              return (
                <p key={i} className="risk-gate-clause">
                  <span className="risk-gate-clause-dot" />
                  {text}
                </p>
              )
            })}
          </div>
          <p className="risk-gate-scroll-hint">请滚动至底部后继续</p>
        </div>

        <button className="risk-gate-accept" type="button" disabled={!scrollReached} onClick={onAccept}>
          我已知悉，继续使用
        </button>
      </div>
    </div>
  )
}
