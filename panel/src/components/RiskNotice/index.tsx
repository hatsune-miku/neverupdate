import { useMemo, useState } from 'react'
import type { UIEvent } from 'react'

import './index.scss'

interface RiskNoticeProps {
  onAccept: () => void
}

export function RiskNotice({ onAccept }: RiskNoticeProps) {
  const [scrollReached, setScrollReached] = useState(false)

  const notes = useMemo(function () {
    return [
      'NeverUpdate 会以管理员权限持续修改系统更新相关设置。',
      '你应当理解这可能影响系统安全补丁获取与设备稳定性。',
      '如果你没有完全理解影响，请不要继续。',
      '本工具不会替你承担任何数据与系统风险。',
    ]
  }, [])

  function handleScroll(event: UIEvent<HTMLDivElement>) {
    const element = event.currentTarget
    const reached = element.scrollTop + element.clientHeight + 4 >= element.scrollHeight
    if (reached) {
      setScrollReached(true)
    }
  }

  return (
    <div className="risk-notice-screen">
      <div className="risk-notice-panel">
        <h1>NeverUpdate 风险告知</h1>
        <div className="risk-notice-content" onScroll={handleScroll}>
          {notes.map(function (item) {
            return <p key={item}>{item}</p>
          })}
          <p>继续使用即代表你明确知道：你在主动改变 Windows Update 默认行为，且会对系统更新链路做强干预。</p>
          <p>请滚动到底部后再点击继续。</p>
        </div>
        <button type="button" disabled={!scrollReached} onClick={onAccept}>
          我已知悉并继续
        </button>
      </div>
    </div>
  )
}
