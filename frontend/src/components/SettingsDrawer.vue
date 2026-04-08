<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="modelValue" class="drawer-overlay" @click="close"></div>
    </Transition>
    <Transition name="slide-right">
      <aside v-if="modelValue" class="settings-drawer">
        <div class="drawer-header">
          <h2>&#35774;&#32622;</h2>
          <button class="close-btn" @click="close">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="drawer-body">
          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
              &#29992;&#25143;
            </h3>
            <div v-if="appStore.isLoggedIn" class="user-info-card">
              <div class="user-avatar-lg">
                {{ appStore.userInfo?.username?.charAt(0)?.toUpperCase() || 'U' }}
              </div>
              <div class="user-panel">
                <div class="user-card-header">
                  <div class="user-detail">
                    <span class="user-name">{{ appStore.userInfo?.username }}</span>
                    <span class="user-role">{{ appStore.userInfo?.isAdmin ? '\u7ba1\u7406\u5458' : '\u666e\u901a\u7528\u6237' }}</span>
                  </div>
                  <button class="action-btn danger" @click="handleLogout">&#27880;&#38144;</button>
                </div>
                <button class="action-btn inline-link" @click="togglePasswordPanel">
                  {{ showPasswordPanel ? '\u6536\u8d77\u4fee\u6539\u5bc6\u7801' : '\u4fee\u6539\u5bc6\u7801' }}
                </button>
                <div v-if="showPasswordPanel" class="password-panel embedded">
                  <label class="password-field">
                    <span>&#24403;&#21069;&#23494;&#30721;</span>
                    <input v-model="passwordForm.oldPassword" type="password" autocomplete="current-password" />
                  </label>
                  <label class="password-field">
                    <span>&#26032;&#23494;&#30721;</span>
                    <input v-model="passwordForm.newPassword" type="password" autocomplete="new-password" />
                  </label>
                  <label class="password-field">
                    <span>&#30830;&#35748;&#26032;&#23494;&#30721;</span>
                    <input v-model="passwordForm.confirmPassword" type="password" autocomplete="new-password" />
                  </label>
                  <div class="password-actions">
                    <button class="action-btn primary" :disabled="changingPassword" @click="handleChangePassword">
                      {{ changingPassword ? '\u63d0\u4ea4\u4e2d...' : '\u4fdd\u5b58\u65b0\u5bc6\u7801' }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
            <button v-else class="action-btn primary full" @click="handleLogin">
              &#30331;&#24405; / &#27880;&#20876;
            </button>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" />
              </svg>
              &#20070;&#28304;&#31649;&#29702;
            </h3>
            <div class="btn-group">
              <button class="action-btn" @click="openSourceManager">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                  <path d="M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" />
                </svg>
                &#20070;&#28304;&#31649;&#29702;
              </button>
            </div>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                <circle cx="9" cy="7" r="4" />
                <path d="M19 8v6" />
                <path d="M22 11h-6" />
              </svg>
              &#29992;&#25143;&#31649;&#29702;
            </h3>
            <div v-if="appStore.isSecureMode" class="status-card">
              <span>{{ userManagerTitle }}</span>
              <small>{{ userManagerMessage }}</small>
            </div>
            <div v-else class="status-card">
              <span>&#24403;&#21069;&#26410;&#24320;&#21551;&#23433;&#20840;&#27169;&#24335;</span>
              <small>&#29992;&#25143;&#31649;&#29702;&#20165;&#22312;&#22810;&#29992;&#25143;&#23433;&#20840;&#27169;&#24335;&#19979;&#21487;&#29992;&#12290;</small>
            </div>
            <div class="btn-group">
              <button class="action-btn" :disabled="!canManageUsers" @click="openUserManager">
                &#29992;&#25143;&#31649;&#29702;
              </button>
            </div>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <path d="M7 10l5 5 5-5" />
                <path d="M12 15V3" />
              </svg>
              &#26381;&#21153;&#22120;&#22791;&#20221;
            </h3>
            <div class="status-card">
              <span>{{ webdavStatusTitle }}</span>
              <small>{{ webdavStatusMessage }}</small>
            </div>
            <div class="btn-group">
              <button class="action-btn" :disabled="!canOpenWebdav" @click="openWebdavManager">
                &#22791;&#20221;&#19982;&#24674;&#22797;
              </button>
            </div>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <path d="M12 16V4" />
                <path d="m7 9 5-5 5 5" />
                <path d="M20 16.5a2.5 2.5 0 0 1-2.5 2.5h-11A2.5 2.5 0 0 1 4 16.5" />
              </svg>
              &#24212;&#29992;
            </h3>
            <div class="status-card">
              <span>{{ appStore.isOnline ? '\u5728\u7ebf' : '\u79bb\u7ebf' }}</span>
              <small>{{ appStore.pwaReady ? '\u5df2\u542f\u7528\u79bb\u7ebf\u5916\u58f3\u7f13\u5b58' : '\u79bb\u7ebf\u5916\u58f3\u672a\u542f\u7528' }}</small>
            </div>
            <div v-if="appStore.pwaUpdateAvailable" class="status-card accent">
              <span>&#21457;&#29616;&#26032;&#29256;&#26412;</span>
              <small>&#21047;&#26032;&#21518;&#21487;&#20351;&#29992;&#26368;&#26032;&#31163;&#32447;&#36164;&#28304;</small>
            </div>
            <div class="btn-group">
              <button class="action-btn" :disabled="!appStore.deferredInstallPrompt" @click="handleInstallPwa">
                &#23433;&#35013;&#21040;&#20027;&#23631;&#24149;
              </button>
              <button class="action-btn primary" :disabled="!appStore.pwaUpdateAvailable" @click="handleApplyUpdate">
                &#26356;&#26032;&#24212;&#29992;
              </button>
            </div>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <rect width="7" height="7" x="3" y="3" rx="1" />
                <rect width="7" height="7" x="14" y="3" rx="1" />
                <rect width="7" height="7" x="3" y="14" rx="1" />
                <rect width="7" height="7" x="14" y="14" rx="1" />
              </svg>
              &#20070;&#26550;&#35774;&#32622;
            </h3>
            <div class="btn-group">
              <button class="action-btn" @click="refreshCache">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                  <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                  <path d="M3 3v5h5" />
                  <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                  <path d="M16 16h5v5" />
                </svg>
                &#21047;&#26032;&#32531;&#23384;
              </button>
            </div>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <path d="M12 8v4l3 3" />
                <circle cx="12" cy="12" r="9" />
              </svg>
              &#38405;&#35835;&#32479;&#35745;
            </h3>
            <div class="stats-grid">
              <div class="status-card">
                <span>{{ appStore.readingStatsSummary.totalTimeText }}</span>
                <small>&#32047;&#35745;&#38405;&#35835;&#26102;&#38271;</small>
              </div>
              <div class="status-card">
                <span>{{ appStore.readingStatsSummary.openedBooks }}</span>
                <small>&#25171;&#24320;&#36807;&#30340;&#20070;&#31821;</small>
              </div>
              <div class="status-card">
                <span>{{ appStore.readingStatsSummary.readChapters }}</span>
                <small>&#38405;&#35835;&#31456;&#33410;&#25968;</small>
              </div>
              <div class="status-card">
                <span>{{ appStore.readingStatsSummary.completedBooks }}</span>
                <small>&#35835;&#23436;&#20070;&#31821;&#25968;</small>
              </div>
            </div>
          </section>

          <section class="drawer-section">
            <h3 class="section-title">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
                <circle cx="12" cy="12" r="4" />
                <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
              </svg>
              &#22806;&#35266;
            </h3>
            <div class="theme-toggle">
              <button
                class="theme-option"
                :class="{ active: appStore.theme === 'light' }"
                @click="setTheme('light')"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20">
                  <circle cx="12" cy="12" r="4" />
                  <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
                </svg>
                &#20142;&#33394;
              </button>
              <button
                class="theme-option"
                :class="{ active: appStore.theme === 'dark' }"
                @click="setTheme('dark')"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                </svg>
                &#26263;&#33394;
              </button>
            </div>
          </section>
        </div>
      </aside>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useAppStore } from '../stores/app'
import { useBookshelfStore } from '../stores/bookshelf'
import { changePassword, logout as apiLogout } from '../api/user'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const appStore = useAppStore()
const shelfStore = useBookshelfStore()
const showPasswordPanel = ref(false)
const changingPassword = ref(false)
const passwordForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: '',
})

const canManageUsers = computed(() => appStore.isSecureMode && appStore.isLoggedIn && !!appStore.userInfo?.isAdmin && !appStore.needSecureKey)
const userManagerTitle = computed(() => {
  if (!appStore.isLoggedIn) return '\u767b\u5f55\u540e\u53ef\u67e5\u770b\u72b6\u6001'
  if (appStore.needSecureKey) return '\u5f53\u524d\u9700\u8981\u7ba1\u7406\u5bc6\u7801'
  return appStore.userInfo?.isAdmin ? '\u5f53\u524d\u8d26\u53f7\u62e5\u6709\u7ba1\u7406\u5458\u6743\u9650' : '\u5f53\u524d\u8d26\u53f7\u4e0d\u662f\u7ba1\u7406\u5458'
})
const userManagerMessage = computed(() => {
  if (!appStore.isLoggedIn) return '\u8bf7\u5148\u767b\u5f55\u7ba1\u7406\u5458\u8d26\u53f7\u540e\u7ba1\u7406\u5176\u4ed6\u7528\u6237\u3002'
  if (appStore.needSecureKey) return '\u670d\u52a1\u7aef\u5df2\u5f00\u542f\u7ba1\u7406\u5bc6\u7801\u6821\u9a8c\uff0c\u672a\u9a8c\u8bc1\u524d\u65e0\u6cd5\u8bfb\u53d6\u7528\u6237\u5217\u8868\u3002'
  return appStore.userInfo?.isAdmin
    ? '\u652f\u6301\u65b0\u589e\u7528\u6237\u3001\u91cd\u7f6e\u5bc6\u7801\u3001\u5220\u9664\u7528\u6237\u548c\u8c03\u6574\u6743\u9650\u3002'
    : '\u8bf7\u4f7f\u7528\u7ba1\u7406\u5458\u8d26\u53f7\u767b\u5f55\u540e\u518d\u8fdb\u884c\u7528\u6237\u7ba1\u7406\u3002'
})
const canOpenWebdav = computed(() => appStore.isSecureMode && appStore.isLoggedIn && !!appStore.userInfo?.enableWebdav)
const webdavStatusTitle = computed(() => {
  if (!appStore.isSecureMode) return '\u4ec5\u5b89\u5168\u6a21\u5f0f\u652f\u6301\u670d\u52a1\u5668\u5907\u4efd'
  if (!appStore.isLoggedIn) return '\u767b\u5f55\u540e\u53ef\u7528'
  return appStore.userInfo?.enableWebdav ? '\u5f53\u524d\u8d26\u53f7\u5df2\u5f00\u542f\u670d\u52a1\u5668\u5907\u4efd' : '\u5f53\u524d\u8d26\u53f7\u672a\u5f00\u542f\u670d\u52a1\u5668\u5907\u4efd'
})
const webdavStatusMessage = computed(() => {
  if (!appStore.isSecureMode) return '\u4e3a\u907f\u514d\u5171\u4eab\u5907\u4efd\u7a7a\u95f4\uff0c\u8bf7\u5148\u5f00\u542f\u591a\u7528\u6237\u5b89\u5168\u6a21\u5f0f\u3002'
  if (!appStore.isLoggedIn) return '\u767b\u5f55\u5e76\u5177\u5907\u5907\u4efd\u6743\u9650\u540e\uff0c\u53ef\u7ba1\u7406\u670d\u52a1\u5668\u4e2d\u7684\u5907\u4efd\u6587\u4ef6\u3002'
  return appStore.userInfo?.enableWebdav
    ? '\u652f\u6301\u5c06\u6570\u636e\u5907\u4efd\u5230\u670d\u52a1\u5668\u3001\u4e0b\u8f7d\u5907\u4efd\u6587\u4ef6\u3001\u4e0a\u4f20\u5907\u4efd\u6587\u4ef6\u5e76\u6267\u884c\u6062\u590d\u3002'
    : '\u8bf7\u5728\u7528\u6237\u7ba1\u7406\u4e2d\u4e3a\u5f53\u524d\u8d26\u53f7\u5f00\u542f\u670d\u52a1\u5668\u5907\u4efd\u6743\u9650\u3002'
})

function close() {
  emit('update:modelValue', false)
}

function handleLogin() {
  close()
  appStore.showLoginModal = true
}

async function handleLogout() {
  await apiLogout()
  appStore.clearUser()
  close()
  shelfStore.fetchBooks()
}

function resetPasswordForm() {
  passwordForm.oldPassword = ''
  passwordForm.newPassword = ''
  passwordForm.confirmPassword = ''
}

function togglePasswordPanel() {
  showPasswordPanel.value = !showPasswordPanel.value
  if (!showPasswordPanel.value) {
    resetPasswordForm()
  }
}

async function handleChangePassword() {
  if (!passwordForm.oldPassword || !passwordForm.newPassword || !passwordForm.confirmPassword) {
    appStore.showToast('\u8bf7\u586b\u5199\u5b8c\u6574\u7684\u5bc6\u7801\u4fe1\u606f', 'warning')
    return
  }
  if (passwordForm.newPassword !== passwordForm.confirmPassword) {
    appStore.showToast('\u4e24\u6b21\u8f93\u5165\u7684\u65b0\u5bc6\u7801\u4e0d\u4e00\u81f4', 'warning')
    return
  }
  changingPassword.value = true
  try {
    await changePassword(passwordForm.oldPassword, passwordForm.newPassword)
    appStore.showToast('\u5bc6\u7801\u4fee\u6539\u6210\u529f', 'success')
    showPasswordPanel.value = false
    resetPasswordForm()
  } catch (error) {
    appStore.showToast((error as Error).message || '\u5bc6\u7801\u4fee\u6539\u5931\u8d25', 'error')
  } finally {
    changingPassword.value = false
  }
}

function openSourceManager() {
  close()
  appStore.showSourceManager = true
}

function openUserManager() {
  close()
  appStore.showUserManager = true
}

function openWebdavManager() {
  close()
  appStore.showWebdavManager = true
}

function refreshCache() {
  shelfStore.fetchBooks()
  appStore.showToast('\u4e66\u67b6\u5df2\u5237\u65b0', 'success')
  close()
}

function setTheme(t: 'light' | 'dark') {
  appStore.setTheme(t)
}

async function handleInstallPwa() {
  const accepted = await appStore.installPwa()
  if (!accepted) {
    appStore.showToast('\u5f53\u524d\u73af\u5883\u6682\u4e0d\u652f\u6301\u5b89\u88c5\uff0c\u6216\u7528\u6237\u5df2\u53d6\u6d88', 'warning')
    return
  }
  appStore.showToast('\u5b89\u88c5\u8bf7\u6c42\u5df2\u63d0\u4ea4', 'success')
}

function handleApplyUpdate() {
  const ok = appStore.applyPwaUpdate()
  if (!ok) {
    appStore.showToast('\u5f53\u524d\u6ca1\u6709\u53ef\u5e94\u7528\u7684\u65b0\u7248\u672c', 'warning')
  }
}
</script>

<style scoped>
.drawer-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: var(--z-overlay);
  backdrop-filter: blur(4px);
}

.settings-drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: min(380px, 90vw);
  background: var(--color-bg-elevated);
  z-index: var(--z-modal);
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-xl);
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: calc(var(--space-5) + var(--safe-area-top)) calc(var(--space-6) + var(--safe-area-right)) var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--color-border-light);
  flex-shrink: 0;
}

.drawer-header h2 {
  font-size: var(--text-xl);
  font-weight: 700;
  letter-spacing: -0.01em;
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  transition: all var(--duration-fast);
}

.close-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.close-btn svg {
  width: 20px;
  height: 20px;
}

.drawer-body {
  flex: 1;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  overscroll-behavior: contain;
  padding: var(--space-4) calc(var(--space-6) + var(--safe-area-right)) calc(var(--space-4) + var(--safe-area-bottom)) var(--space-6);
}

@media (max-width: 768px) {
  .settings-drawer {
    width: min(420px, 92vw);
  }
}

.drawer-section {
  padding: var(--space-4) 0;
  border-bottom: 1px solid var(--color-divider);
}

.drawer-section:last-child {
  border-bottom: none;
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-3);
}

.user-info-card {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-3);
  background: var(--color-bg-sunken);
  border-radius: var(--radius-md);
}

.user-panel {
  flex: 1;
  display: grid;
  gap: var(--space-3);
}

.user-card-header {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
}

.user-avatar-lg {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-full);
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: var(--text-lg);
  flex-shrink: 0;
}

.user-detail {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.user-name {
  font-weight: 600;
  font-size: var(--text-sm);
}

.user-role {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.password-panel {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
  background: var(--color-bg);
}

.password-panel.embedded {
  background: var(--color-bg-elevated);
}

.password-field {
  display: grid;
  gap: var(--space-2);
}

.password-field span {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.password-field input {
  min-height: 40px;
  padding: 0 var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  color: inherit;
}

.password-actions {
  display: flex;
  justify-content: flex-start;
}

.btn-group {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  font-weight: 500;
  background: var(--color-bg-sunken);
  color: var(--color-text);
  border: 1px solid var(--color-border-light);
  transition: all var(--duration-fast) var(--ease-out);
}

.action-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-border);
}

.action-btn:active {
  transform: scale(0.97);
}

.action-btn.primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.action-btn.primary:hover {
  background: var(--color-primary-dark);
}

.action-btn.danger {
  color: var(--color-danger);
  border-color: transparent;
  background: transparent;
  padding: var(--space-1) var(--space-2);
}

.action-btn.danger:hover {
  background: rgba(245, 34, 45, 0.08);
}

.action-btn.full {
  width: 100%;
  justify-content: center;
}

.inline-link {
  padding: 0;
  background: transparent;
  border: none;
  color: var(--color-primary);
  font-weight: 500;
  justify-content: flex-start;
}

.inline-link:hover {
  background: transparent;
  border: none;
  color: var(--color-primary-dark);
}

.theme-toggle {
  display: flex;
  gap: var(--space-2);
}

.status-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: var(--space-3);
  background: var(--color-bg-sunken);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-3);
}

.status-card span {
  font-size: var(--text-sm);
  font-weight: 600;
}

.status-card small {
  color: var(--color-text-tertiary);
}

.status-card.accent {
  background: rgba(201, 127, 58, 0.12);
  border: 1px solid rgba(201, 127, 58, 0.18);
}

.action-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-2);
}

.theme-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  border: 2px solid var(--color-border-light);
  background: var(--color-bg);
  font-size: var(--text-sm);
  font-weight: 500;
  transition: all var(--duration-fast);
  color: var(--color-text-secondary);
}

.theme-option.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-bg);
}

.theme-option:hover:not(.active) {
  border-color: var(--color-border);
}
</style>
