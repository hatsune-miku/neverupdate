import { useAppStore } from '@/store'

import './index.scss'

interface UpdateToastProps {
  onOpenSettings: () => void
}

export function UpdateToast({ onOpenSettings }: UpdateToastProps) {
  const updaterCheckEnabled = useAppStore((s) => s.updaterCheckEnabled)
  const updateToastVisible = useAppStore((s) => s.updateToastVisible)
  const updateAvailable = useAppStore((s) => s.updateAvailable)
  const updateStatus = useAppStore((s) => s.updateStatus)
  const updateMessage = useAppStore((s) => s.updateMessage)
  const updateDownloaded = useAppStore((s) => s.updateDownloaded)
  const updateTotal = useAppStore((s) => s.updateTotal)
  const dismissUpdateToast = useAppStore((s) => s.dismissUpdateToast)

  const showReady = updateStatus === 'ready' && updateAvailable
  const showDownloading = updateStatus === 'downloading'
  const showError = updateStatus === 'error' && updateMessage
  const hasLiveToast = updaterCheckEnabled && updateToastVisible && (showReady || showDownloading || showError)
  const version = updateAvailable?.version

  if (!hasLiveToast) {
    return null
  }

  return (
    <div className="nu-update-toast" role="status">
      {showReady && version ? (
        <>
          <div className="nu-update-toast-body">
            <span className="nu-update-toast-title">发现新版本</span>
            <span className="nu-update-toast-version">{version}</span>
          </div>
          <div className="nu-update-toast-actions">
            <button className="nu-btn nu-btn-ghost nu-update-toast-btn" type="button" onClick={dismissUpdateToast}>
              下次再说
            </button>
            <button
              className="nu-btn nu-btn-ghost nu-update-toast-btn"
              type="button"
              onClick={() => {
                dismissUpdateToast()
                onOpenSettings()
              }}
            >
              更改更新设置
            </button>
            <button
              className="nu-btn nu-btn-primary nu-update-toast-btn"
              type="button"
              onClick={() => {
                dismissUpdateToast()
                onOpenSettings()
              }}
            >
              前往更新
            </button>
          </div>
        </>
      ) : null}
      {showDownloading ? (
        <div className="nu-update-toast-body">
          <span className="nu-update-toast-title">正在下载更新</span>
          {updateTotal ? (
            <span className="nu-mono nu-update-toast-progress">{Math.round((updateDownloaded / updateTotal) * 100)}%</span>
          ) : (
            <span className="nu-update-toast-progress-muted">准备中…</span>
          )}
        </div>
      ) : null}
      {showError ? (
        <>
          <div className="nu-update-toast-body nu-update-toast-body-error">
            <span className="nu-update-toast-title">更新失败</span>
            <span className="nu-update-toast-err">{updateMessage}</span>
          </div>
          <div className="nu-update-toast-actions">
            <button className="nu-btn nu-btn-primary nu-update-toast-btn" type="button" onClick={dismissUpdateToast}>
              关闭
            </button>
          </div>
        </>
      ) : null}
    </div>
  )
}
