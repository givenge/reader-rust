import type { useAppStore } from '../stores/app'

type AppStore = ReturnType<typeof useAppStore>

export function registerPwa(appStore: AppStore) {
  if (typeof window === 'undefined') return

  appStore.setOnlineStatus(navigator.onLine)

  window.addEventListener('online', () => {
    appStore.setOnlineStatus(true)
    appStore.showToast('网络已恢复', 'success')
  })

  window.addEventListener('offline', () => {
    appStore.setOnlineStatus(false)
    appStore.showToast('已进入离线模式', 'warning')
  })

  window.addEventListener('beforeinstallprompt', (event) => {
    event.preventDefault()
    appStore.setDeferredInstallPrompt(event)
  })

  window.addEventListener('appinstalled', () => {
    appStore.setDeferredInstallPrompt(null)
    appStore.showToast('已安装到主屏幕', 'success')
  })

  if (!('serviceWorker' in navigator)) return

  function bindWaitingWorker(worker: ServiceWorker | null | undefined) {
    if (!worker) return
    appStore.setWaitingServiceWorker(worker)
    appStore.setPwaUpdateAvailable(true)
  }

  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').then((registration) => {
      appStore.setPwaReady(true)

      if (registration.waiting) {
        bindWaitingWorker(registration.waiting)
      }

      registration.addEventListener('updatefound', () => {
        const installing = registration.installing
        if (!installing) return
        installing.addEventListener('statechange', () => {
          if (installing.state === 'installed' && navigator.serviceWorker.controller) {
            bindWaitingWorker(registration.waiting || installing)
          }
        })
      })

      let refreshing = false
      navigator.serviceWorker.addEventListener('controllerchange', () => {
        if (refreshing) return
        refreshing = true
        appStore.setPwaUpdateAvailable(false)
        appStore.setWaitingServiceWorker(null)
        window.location.reload()
      })
    }).catch(() => {
      appStore.setPwaReady(false)
    })
  })
}
