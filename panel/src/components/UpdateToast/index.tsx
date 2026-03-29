import * as Toast from '@radix-ui/react-toast'

import { useAppStore } from '@/store'

import './index.scss'

export function UpdateToast() {
  const updaterCheckEnabled = useAppStore(function (s) {
    return s.updaterCheckEnabled
  })
  const updateToastVisible = useAppStore(function (s) {
    return s.updateToastVisible
  })
  const updateAvailable = useAppStore(function (s) {
    return s.updateAvailable
  })
  const updateStatus = useAppStore(function (s) {
    return s.updateStatus
  })
  const updateMessage = useAppStore(function (s) {
    return s.updateMessage
  })
  const updateDownloaded = useAppStore(function (s) {
    return s.updateDownloaded
  })
  const updateTotal = useAppStore(function (s) {
    return s.updateTotal
  })
  const installUpdate = useAppStore(function (s) {
    return s.installUpdate
  })
  const dismissUpdateToast = useAppStore(function (s) {
    return s.dismissUpdateToast
  })

  if (!updaterCheckEnabled || !updateToastVisible) {
    return null
  }

  const showReady = updateStatus === 'ready' && updateAvailable
  const showDownloading = updateStatus === 'downloading'
  const showError = updateStatus === 'error' && updateMessage

  if (!showReady && !showDownloading && !showError) {
    return null
  }

  const version = updateAvailable?.version

  return (
    <Toast.Provider swipeDirection="right">
      <Toast.Root
        className="nu-update-toast"
        open={updateToastVisible}
        duration={showDownloading ? 60_000 : 10_000}
        onOpenChange={function (open) {
          if (!open) {
            dismissUpdateToast()
          }
        }}
      >
        {showReady && version ? (
          <>
            <div className="nu-update-toast-body">
              <Toast.Title className="nu-update-toast-title">发现新版本</Toast.Title>
              <Toast.Description className="nu-update-toast-version">{version}</Toast.Description>
            </div>
            <div className="nu-update-toast-actions">
              <Toast.Action className="nu-update-toast-btn nu-btn nu-btn-ghost" altText="稍后再更新" asChild>
                <button type="button" onClick={dismissUpdateToast}>
                  稍后
                </button>
              </Toast.Action>
              <Toast.Action className="nu-update-toast-btn nu-btn nu-btn-primary" altText="立即安装更新" asChild>
                <button
                  type="button"
                  onClick={function () {
                    void installUpdate()
                  }}
                >
                  立即更新
                </button>
              </Toast.Action>
            </div>
          </>
        ) : null}
        {showDownloading ? (
          <div className="nu-update-toast-body">
            <Toast.Title className="nu-update-toast-title">正在下载更新</Toast.Title>
            {updateTotal ? (
              <Toast.Description className="nu-mono nu-update-toast-progress">
                {Math.round((updateDownloaded / updateTotal) * 100)}%
              </Toast.Description>
            ) : (
              <Toast.Description className="nu-update-toast-progress-muted">准备中…</Toast.Description>
            )}
          </div>
        ) : null}
        {showError ? (
          <>
            <div className="nu-update-toast-body nu-update-toast-body-error">
              <Toast.Title className="nu-update-toast-title">更新失败</Toast.Title>
              <Toast.Description className="nu-update-toast-err">{updateMessage}</Toast.Description>
            </div>
            <div className="nu-update-toast-actions">
              <Toast.Close className="nu-update-toast-btn nu-btn nu-btn-primary" asChild>
                <button type="button">关闭</button>
              </Toast.Close>
            </div>
          </>
        ) : null}
      </Toast.Root>
      <Toast.Viewport className="nu-update-toast-viewport" />
    </Toast.Provider>
  )
}
