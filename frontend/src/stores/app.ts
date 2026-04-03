import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { getUserInfo } from '../api/user'
import type { UserInfo } from '../types'
import { applySystemTheme } from '../utils/systemUi'

export const useAppStore = defineStore('app', () => {
  // ─── Theme ───
  const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null
  const legacyReaderNight = localStorage.getItem('reader-isNight') === 'true'
  const theme = ref<'light' | 'dark'>(
    savedTheme || (legacyReaderNight ? 'dark' : 'light')
  )

  function setTheme(value: 'light' | 'dark') {
    theme.value = value
    localStorage.setItem('theme', value)
    applySystemTheme(value)
  }

  function toggleTheme() {
    setTheme(theme.value === 'light' ? 'dark' : 'light')
  }

  watch(theme, (val) => {
    localStorage.setItem('theme', val)
    applySystemTheme(val)
  }, { immediate: true })

  // ─── User ───
  const userInfo = ref<UserInfo | null>(null)
  const isSecureMode = ref(false)
  const needSecureKey = ref(false)
  const isLoggedIn = ref(false)

  async function fetchUserInfo() {
    try {
      const data = await getUserInfo()
      userInfo.value = data.userInfo
      isSecureMode.value = data.secure
      needSecureKey.value = data.secureKey
      isLoggedIn.value = !!data.userInfo?.username
    } catch {
      isLoggedIn.value = false
    }
  }

  function setUser(user: UserInfo) {
    userInfo.value = user
    isLoggedIn.value = true
    localStorage.setItem('accessToken', user.accessToken)
  }

  function clearUser() {
    userInfo.value = null
    isLoggedIn.value = false
    localStorage.removeItem('accessToken')
  }

  // ─── UI State ───
  const showLoginModal = ref(false)
  const showSettingsDrawer = ref(false)
  const showSourceManager = ref(false)
  const isOnline = ref(typeof navigator !== 'undefined' ? navigator.onLine : true)
  const pwaReady = ref(false)
  const pwaUpdateAvailable = ref(false)
  const deferredInstallPrompt = ref<any>(null)
  const waitingServiceWorker = ref<ServiceWorker | null>(null)

  // ─── Toast ───
  const toasts = ref<Array<{ id: number; message: string; type: string }>>([])
  let toastId = 0

  function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id)
    }, 3000)
  }

  function setOnlineStatus(value: boolean) {
    isOnline.value = value
  }

  function setPwaReady(value: boolean) {
    pwaReady.value = value
  }

  function setPwaUpdateAvailable(value: boolean) {
    pwaUpdateAvailable.value = value
  }

  function setWaitingServiceWorker(value: ServiceWorker | null) {
    waitingServiceWorker.value = value
  }

  function setDeferredInstallPrompt(value: any) {
    deferredInstallPrompt.value = value
  }

  async function installPwa() {
    if (!deferredInstallPrompt.value) return false
    deferredInstallPrompt.value.prompt()
    const result = await deferredInstallPrompt.value.userChoice.catch(() => null)
    deferredInstallPrompt.value = null
    return result?.outcome === 'accepted'
  }

  function applyPwaUpdate() {
    if (!waitingServiceWorker.value) return false
    waitingServiceWorker.value.postMessage({ type: 'SKIP_WAITING' })
    return true
  }

  return {
    theme, setTheme, toggleTheme,
    userInfo, isSecureMode, needSecureKey, isLoggedIn,
    fetchUserInfo, setUser, clearUser,
    showLoginModal, showSettingsDrawer, showSourceManager,
    isOnline, pwaReady, pwaUpdateAvailable, deferredInstallPrompt, waitingServiceWorker,
    setOnlineStatus, setPwaReady, setPwaUpdateAvailable, setDeferredInstallPrompt, setWaitingServiceWorker, installPwa, applyPwaUpdate,
    toasts, showToast,
  }
})
