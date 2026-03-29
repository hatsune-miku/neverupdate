import { useAppStore } from '@/store'

import './index.scss'

interface SettingsProps {
  onBack: () => void
}

export function Settings({ onBack }: SettingsProps) {
  const updaterCheckEnabled = useAppStore(function (s) {
    return s.updaterCheckEnabled
  })
  const setUpdaterCheckEnabled = useAppStore(function (s) {
    return s.setUpdaterCheckEnabled
  })
  const updateStatus = useAppStore(function (s) {
    return s.updateStatus
  })
  const updateMessage = useAppStore(function (s) {
    return s.updateMessage
  })
  const updateAvailable = useAppStore(function (s) {
    return s.updateAvailable
  })
  const updateDownloaded = useAppStore(function (s) {
    return s.updateDownloaded
  })
  const updateTotal = useAppStore(function (s) {
    return s.updateTotal
  })
  const checkForUpdates = useAppStore(function (s) {
    return s.checkForUpdates
  })
  const installUpdate = useAppStore(function (s) {
    return s.installUpdate
  })

  return (
    <div className="settings-screen">
      <div className="settings-card">
        <h1>NeverUpdate 设置</h1>

        <div className="settings-section">
          <div className="settings-row">
            <div className="settings-row-text">
              <span className="settings-row-title">应用更新检测</span>
              <span className="settings-row-desc">控制 NeverUpdate 自身的更新检测。关闭后不再自动检查新版本，本页下方的检查与安装入口也会隐藏；已显示的右下角提醒会关闭。</span>
            </div>
            <button
              className={`settings-switch ${updaterCheckEnabled ? 'on' : ''}`}
              type="button"
              role="switch"
              aria-checked={updaterCheckEnabled}
              aria-label="NeverUpdate 应用更新检测"
              onClick={function () {
                setUpdaterCheckEnabled(!updaterCheckEnabled)
              }}
            >
              <span className="settings-switch-knob" />
            </button>
          </div>

          {updaterCheckEnabled ? (
            <div className="settings-updater">
              <div className="settings-updater-actions">
                <button
                  className="nu-btn nu-btn-ghost"
                  disabled={updateStatus === 'checking' || updateStatus === 'downloading'}
                  type="button"
                  onClick={function () {
                    void checkForUpdates()
                  }}
                >
                  {updateStatus === 'checking' ? '检查中' : '检查更新'}
                </button>
                {updateAvailable ? (
                  <button
                    className="nu-btn nu-btn-primary"
                    disabled={updateStatus === 'downloading'}
                    type="button"
                    onClick={function () {
                      void installUpdate()
                    }}
                  >
                    {updateStatus === 'downloading' ? '更新中' : `更新到 ${updateAvailable.version}`}
                  </button>
                ) : null}
              </div>
              {updateMessage ? (
                <div className={`settings-updater-status ${updateStatus === 'error' ? 'error' : ''}`}>
                  <span className="settings-updater-status-text">{updateMessage}</span>
                  {updateStatus === 'downloading' && updateTotal ? <span className="nu-mono settings-updater-progress">{Math.round((updateDownloaded / updateTotal) * 100)}%</span> : null}
                </div>
              ) : null}
            </div>
          ) : null}
        </div>

        <div className="settings-actions">
          <button className="nu-btn nu-btn-primary" type="button" onClick={onBack}>
            返回面板
          </button>
        </div>
      </div>
    </div>
  )
}
