<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="modelValue" class="modal-overlay" @click="close"></div>
    </Transition>
    <Transition name="scale">
      <div v-if="modelValue" class="modal-container">
        <div class="login-modal">
          <button class="modal-close" @click="close">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>

          <div class="modal-header">
            <div class="modal-logo">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="32" height="32">
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
              </svg>
            </div>
            <h2>{{ isLogin ? '登录' : '注册' }}</h2>
            <p class="modal-desc">{{ isLogin ? '登录以同步你的阅读数据' : '创建新账号开始阅读' }}</p>
          </div>

          <form class="login-form" @submit.prevent="handleSubmit">
            <div class="form-field">
              <label for="username">用户名</label>
              <input
                id="username"
                v-model="form.username"
                type="text"
                placeholder="请输入用户名"
                required
                autocomplete="username"
              />
            </div>
            <div class="form-field">
              <label for="password">密码</label>
              <input
                id="password"
                v-model="form.password"
                type="password"
                placeholder="请输入密码"
                required
                autocomplete="current-password"
              />
            </div>

            <button type="submit" class="submit-btn" :disabled="submitting">
              <span v-if="submitting" class="btn-spinner"></span>
              {{ isLogin ? '登 录' : '注 册' }}
            </button>
          </form>

          <p class="switch-mode">
            {{ isLogin ? '没有账号？' : '已有账号？' }}
            <a href="#" @click.prevent="isLogin = !isLogin">
              {{ isLogin ? '注册' : '登录' }}
            </a>
          </p>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { login, register } from '../api/user'
import { useAppStore } from '../stores/app'
import { useBookshelfStore } from '../stores/bookshelf'

defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const appStore = useAppStore()
const shelfStore = useBookshelfStore()

const isLogin = ref(true)
const submitting = ref(false)
const form = reactive({
  username: '',
  password: '',
})

function close() {
  emit('update:modelValue', false)
}

async function handleSubmit() {
  if (!form.username || !form.password) return
  submitting.value = true
  try {
    const user = isLogin.value
      ? await login(form.username, form.password)
      : await register(form.username, form.password)
    appStore.setUser(user)
    appStore.showToast(isLogin.value ? '登录成功' : '注册成功', 'success')
    close()
    shelfStore.fetchBooks()
    shelfStore.fetchGroups()
  } catch (e: unknown) {
    appStore.showToast((e as Error).message || '操作失败', 'error')
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: var(--z-overlay);
  backdrop-filter: blur(4px);
}

.modal-container {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: flex;
  align-items: center;
  justify-content: center;
  padding:
    calc(var(--space-6) + var(--safe-area-top))
    calc(var(--space-6) + var(--safe-area-right))
    calc(var(--space-6) + var(--safe-area-bottom))
    calc(var(--space-6) + var(--safe-area-left));
}

.login-modal {
  width: 100%;
  max-width: 400px;
  background: var(--color-bg-elevated);
  border-radius: var(--radius-xl);
  padding: var(--space-8);
  position: relative;
  box-shadow: var(--shadow-xl);
  max-height: min(720px, calc(100dvh - var(--safe-area-top) - var(--safe-area-bottom) - 32px));
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}

.modal-close {
  position: absolute;
  top: max(var(--space-4), calc(var(--safe-area-top) * 0.35));
  right: var(--space-4);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  color: var(--color-text-tertiary);
  transition: all var(--duration-fast);
}

.modal-close:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.modal-close svg {
  width: 18px;
  height: 18px;
}

.modal-header {
  text-align: center;
  margin-bottom: var(--space-8);
}

.modal-logo {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: var(--radius-lg);
  background: var(--color-primary-bg);
  color: var(--color-primary);
  margin-bottom: var(--space-4);
}

.modal-header h2 {
  font-size: var(--text-2xl);
  font-weight: 700;
  margin-bottom: var(--space-2);
}

.modal-desc {
  color: var(--color-text-tertiary);
  font-size: var(--text-sm);
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.form-field label {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
}

.form-field input {
  padding: var(--space-3) var(--space-4);
  border: 1.5px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg);
  outline: none;
  transition: all var(--duration-fast);
  font-size: var(--text-base);
}

.form-field input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-bg);
}

.submit-btn {
  padding: var(--space-3) var(--space-6);
  background: var(--color-primary);
  color: white;
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  font-weight: 600;
  transition: all var(--duration-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  margin-top: var(--space-2);
}

.submit-btn:hover:not(:disabled) {
  background: var(--color-primary-dark);
  transform: translateY(-1px);
}

.submit-btn:active:not(:disabled) {
  transform: translateY(0);
}

.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .login-modal {
    padding: var(--space-6);
    border-radius: 20px;
  }
}

.btn-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.switch-mode {
  text-align: center;
  margin-top: var(--space-6);
  font-size: var(--text-sm);
  color: var(--color-text-tertiary);
}

.switch-mode a {
  color: var(--color-primary);
  font-weight: 500;
}
</style>
