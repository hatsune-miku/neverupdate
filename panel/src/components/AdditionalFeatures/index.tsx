import { useMemo, useState } from 'react'
import type { CSSProperties, ReactNode } from 'react'
import ReactMarkdown from 'react-markdown'

import { getGuardPrinciple } from '@/content/principles'
import type { HistoryEntry, InterceptionEntry } from '@/types'

import './index.scss'

interface AdditionalFeaturesProps {
  busy: boolean
  history: HistoryEntry[]
  interceptions: InterceptionEntry[]
  onClearHistory: () => void
  onClearInterceptions: () => void
  onRunExtremeMode: () => void
}

type ModalKind = 'history' | 'interception' | 'extreme' | null

const MAX_RECORD_ITEMS = 500

const ACTION_LABELS: Record<string, string> = {
  guard: '阻断',
  release: '放开',
  repair: '阻断',
}

function formatTime(value: string): string {
  return new Date(value).toLocaleString([], {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function HistoryRow({ item, index, style }: { item: HistoryEntry; index: number; style: CSSProperties }) {
  return (
    <div className="af-vrow af-vrow-history" style={style}>
      <span className="nu-mono af-col-time">{formatTime(item.timestamp)}</span>
      <span className="af-col-action">{ACTION_LABELS[item.action] || item.action}</span>
      <span className={`af-col-result ${item.success ? 'ok' : 'fail'}`}>{item.success ? '成功' : '失败'}</span>
      <span className="af-col-message">{item.message || '-'}</span>
      <span className="af-col-index">#{historyIndexLabel(index)}</span>
    </div>
  )
}

function historyIndexLabel(index: number): string {
  return String(index + 1).padStart(3, '0')
}

function InterceptionRow({ item, style }: { item: InterceptionEntry; style: CSSProperties }) {
  return (
    <div className="af-vrow af-vrow-interception" style={style}>
      <span className="nu-mono af-col-time">{formatTime(item.timestamp)}</span>
      <span className="af-col-behavior">{item.behavior}</span>
      <span className={`af-col-result ${item.blocked ? 'ok' : 'fail'}`}>{item.blocked ? '已阻断' : '阻断失败！'}</span>
    </div>
  )
}

function VirtualList<T>({ items, rowHeight, height, renderRow }: { items: T[]; rowHeight: number; height: number; renderRow: (item: T, index: number, style: CSSProperties) => ReactNode }) {
  const [scrollTop, setScrollTop] = useState(0)
  const totalHeight = items.length * rowHeight
  const start = Math.max(0, Math.floor(scrollTop / rowHeight) - 4)
  const visibleCount = Math.ceil(height / rowHeight) + 8
  const end = Math.min(items.length, start + visibleCount)
  const visible = items.slice(start, end)

  return (
    <div
      className="af-vlist"
      style={{ height }}
      onScroll={function (event) {
        setScrollTop(event.currentTarget.scrollTop)
      }}
    >
      <div className="af-vlist-spacer" style={{ height: totalHeight }}>
        {visible.map(function (item, offset) {
          const index = start + offset
          const style: CSSProperties = {
            position: 'absolute',
            top: index * rowHeight,
            left: 0,
            right: 0,
            height: rowHeight,
          }
          return <div key={index}>{renderRow(item, index, style)}</div>
        })}
      </div>
    </div>
  )
}

function ModalShell({ title, onClose, children }: { title: string; onClose: () => void; children: ReactNode }) {
  return (
    <div className="af-modal-backdrop" onClick={onClose} role="presentation">
      <section
        className="af-modal"
        onClick={function (event) {
          event.stopPropagation()
        }}
      >
        <header className="af-modal-header">
          <h3>{title}</h3>
          <button className="nu-icon-btn" type="button" aria-label="关闭" onClick={onClose}>
            <svg viewBox="0 0 16 16" aria-hidden>
              <path
                fill="currentColor"
                d="M3.22 3.22a.75.75 0 0 1 1.06 0L8 6.94l3.72-3.72a.75.75 0 1 1 1.06 1.06L9.06 8l3.72 3.72a.75.75 0 0 1-1.06 1.06L8 9.06l-3.72 3.72a.75.75 0 1 1-1.06-1.06L6.94 8 3.22 4.28a.75.75 0 0 1 0-1.06Z"
              />
            </svg>
          </button>
        </header>
        {children}
      </section>
    </div>
  )
}

export function AdditionalFeatures({ busy, history, interceptions, onClearHistory, onClearInterceptions, onRunExtremeMode }: AdditionalFeaturesProps) {
  const [modal, setModal] = useState<ModalKind>(null)
  const [extremeStep2, setExtremeStep2] = useState(false)
  const [storeAck, setStoreAck] = useState(false)
  const visibleHistory = useMemo(
    function () {
      return history.slice(0, MAX_RECORD_ITEMS)
    },
    [history],
  )
  const visibleInterceptions = useMemo(
    function () {
      return interceptions.slice(0, MAX_RECORD_ITEMS)
    },
    [interceptions],
  )

  const interceptionCount = useMemo(
    function () {
      return visibleInterceptions.length
    },
    [visibleInterceptions],
  )
  const extremePrinciple = getGuardPrinciple('extreme_mode')

  function closeModal() {
    setModal(null)
    setExtremeStep2(false)
    setStoreAck(false)
  }

  function openExtremeModal() {
    setModal('extreme')
    setExtremeStep2(false)
    setStoreAck(false)
  }

  async function confirmExtreme() {
    if (!extremeStep2) {
      setExtremeStep2(true)
      return
    }
    if (!storeAck || busy) {
      return
    }
    await onRunExtremeMode()
    closeModal()
  }

  return (
    <section className="af">
      <header className="af-header">
        <h3>附加功能</h3>
      </header>

      <div className="af-grid">
        <button
          className="af-card"
          disabled={busy}
          type="button"
          onClick={function () {
            setModal('history')
          }}
        >
          <strong>操作记录</strong>
          <span>查看最近 500 条动作记录</span>
        </button>

        <button
          className="af-card"
          disabled={busy}
          type="button"
          onClick={function () {
            setModal('interception')
          }}
        >
          <strong>拦截记录</strong>
          <span>已拦截 {interceptionCount} 次更新行为</span>
        </button>

        <button className="af-card af-card-danger" disabled={busy} type="button" onClick={openExtremeModal}>
          <strong>应用极端手段</strong>
          <span>二次确认后执行</span>
        </button>
      </div>

      {modal === 'history' ? (
        <ModalShell title="操作记录（最新 500 条）" onClose={closeModal}>
          <div className="af-modal-toolbar">
            <span>共 {visibleHistory.length} 条</span>
            <button className="nu-btn nu-btn-ghost" disabled={busy || visibleHistory.length === 0} type="button" onClick={onClearHistory}>
              清空记录
            </button>
          </div>
          {visibleHistory.length === 0 ? (
            <p className="af-empty">暂无记录</p>
          ) : (
            <>
              <div className="af-head af-head-history">
                <span>时间</span>
                <span>行为</span>
                <span>结果</span>
                <span>详情</span>
                <span>#</span>
              </div>
              <VirtualList
                items={visibleHistory}
                rowHeight={44}
                height={420}
                renderRow={function (item, index, style) {
                  return <HistoryRow item={item} index={index} style={style} />
                }}
              />
            </>
          )}
        </ModalShell>
      ) : null}

      {modal === 'interception' ? (
        <ModalShell title="拦截记录（最新 500 条）" onClose={closeModal}>
          <div className="af-modal-toolbar">
            <span>已拦截 {interceptionCount} 次更新行为</span>
            <button className="nu-btn nu-btn-ghost" disabled={busy || visibleInterceptions.length === 0} type="button" onClick={onClearInterceptions}>
              清空记录
            </button>
          </div>
          {visibleInterceptions.length === 0 ? (
            <p className="af-empty">暂无拦截记录</p>
          ) : (
            <>
              <div className="af-head af-head-interception">
                <span>时间</span>
                <span>行为</span>
                <span>结果</span>
              </div>
              <VirtualList
                items={visibleInterceptions}
                rowHeight={44}
                height={420}
                renderRow={function (item, _index, style) {
                  return <InterceptionRow item={item} style={style} />
                }}
              />
            </>
          )}
        </ModalShell>
      ) : null}

      {modal === 'extreme' ? (
        <ModalShell title="应用极端手段" onClose={closeModal}>
          <p className="af-extreme-note">该操作会破坏系统更新链路，执行后将很难恢复。</p>
          <div className="af-principle">
            <h4>{extremePrinciple.title} · 技术原理</h4>
            <ReactMarkdown>{extremePrinciple.markdown}</ReactMarkdown>
          </div>
          <label className="af-extreme-check">
            <input
              type="checkbox"
              checked={storeAck}
              onChange={function (event) {
                setStoreAck(event.target.checked)
              }}
            />
            <span>我不需要微软商店，继续执行</span>
          </label>
          <div className="af-modal-actions">
            <button className="nu-btn nu-btn-ghost" type="button" onClick={closeModal}>
              取消
            </button>
            <button className="nu-btn nu-btn-danger" disabled={!storeAck || busy} type="button" onClick={confirmExtreme}>
              {extremeStep2 ? '确认执行' : '继续'}
            </button>
          </div>
        </ModalShell>
      ) : null}
    </section>
  )
}
