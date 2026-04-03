<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="modelValue" class="modal-overlay" @click="close"></div>
    </Transition>
    <Transition name="scale">
      <div v-if="modelValue" class="modal-container" @click.self="close">
        <div class="source-modal">
          <div class="modal-header">
            <div>
              <h2>书源管理 ({{ sources.length }})</h2>
              <p class="header-subtitle">支持本地导入、远程订阅、导出和直接编辑 JSON</p>
            </div>
            <div class="header-actions">
              <button class="icon-btn" @click="loadSources" title="刷新" :class="{ spinning: loading }">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                  <path d="M3 3v5h5" />
                  <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                  <path d="M16 16h5v5" />
                </svg>
              </button>
              <button class="icon-btn" @click="close" title="关闭">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6 6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>

          <div class="toolbar">
            <button class="action-btn action-btn-solid" @click="triggerFileImport">本地导入</button>
            <button class="action-btn action-btn-solid" @click="exportSources">导出书源</button>
            <button class="action-btn primary" @click="createSource">新增书源</button>
            <input ref="fileInputRef" type="file" accept=".json,.txt" class="hidden-input" style="display: none" @change="handleFileImport" />
          </div>

          <div class="remote-panel">
            <div class="remote-panel-header">
              <div>
                <h3>远程书源订阅</h3>
                <p>保存书源合集链接，后续可以一键同步更新</p>
              </div>
              <button class="mini-btn" @click="showRemotePanel = !showRemotePanel">
                {{ showRemotePanel ? '收起' : '展开' }}
              </button>
            </div>
            <div v-if="showRemotePanel" class="remote-panel-body">
            <div class="remote-form">
              <input v-model.trim="remoteUrl" type="text" placeholder="输入远程书源合集链接" />
              <button class="action-btn primary" @click="importRemoteSource">立即同步</button>
              <button class="action-btn" @click="saveSubscription">保存订阅</button>
            </div>
            <div v-if="subscriptions.length" class="subscription-list">
              <div v-for="item in subscriptions" :key="item.url" class="subscription-item">
                <div class="subscription-main">
                  <span class="subscription-url">{{ item.url }}</span>
                  <span class="subscription-time" v-if="item.lastSyncedAt">上次同步 {{ formatTime(item.lastSyncedAt) }}</span>
                </div>
                <div class="subscription-actions">
                  <button class="mini-btn" @click="syncSubscription(item.url)">同步</button>
                  <button class="mini-btn danger" @click="removeSubscription(item.url)">删除</button>
                </div>
              </div>
            </div>
            </div>
          </div>

          <div class="filter-bar">
            <div class="search-field">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.3-4.3" />
              </svg>
              <input v-model="filterText" type="text" placeholder="搜索书源..." />
            </div>
            <select v-model="filterGroup" class="group-select">
              <option value="">全部分组</option>
              <option v-for="g in groupList" :key="g" :value="g">{{ g }}</option>
            </select>
          </div>

          <div class="content-grid">
            <div class="source-list-wrapper">
              <div class="source-list" v-if="!loading">
                <div
                  v-for="source in filteredSources"
                  :key="source.bookSourceUrl"
                  class="source-item"
                  :class="{ active: editingSource?.bookSourceUrl === source.bookSourceUrl }"
                >
                  <div class="source-info" @click="editSource(source)">
                    <span class="source-name">{{ source.bookSourceName }}</span>
                    <span class="source-url">{{ source.bookSourceUrl }}</span>
                    <span v-if="source.bookSourceGroup" class="source-group">{{ source.bookSourceGroup }}</span>
                  </div>
                  <div class="source-actions">
                    <label class="toggle">
                      <input type="checkbox" :checked="source.enabled !== false" @change="toggleSource(source)" />
                      <span class="toggle-slider"></span>
                    </label>
                    <button class="icon-btn small" @click="editSource(source)" title="编辑">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 20h9" />
                        <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
                      </svg>
                    </button>
                    <button class="icon-btn small danger" @click="removeSource(source)" title="删除">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
              <div v-else class="loading-state">
                <div class="loading-spinner"></div>
                <p>加载书源中...</p>
              </div>
            </div>

            <div class="editor-panel">
              <div class="editor-head">
                <h3>{{ editingSource ? '编辑书源' : '新增书源' }}</h3>
                <div class="editor-actions">
                  <button
                    v-if="canLoginSource"
                    class="mini-btn"
                    :disabled="sourceLoginLoading"
                    @click="handleSourceLogin"
                  >
                    {{ sourceLoginLoading ? '登录中...' : '书源登录' }}
                  </button>
                  <button class="mini-btn" @click="formatEditor">格式化</button>
                  <button class="mini-btn primary" @click="saveEditor">保存</button>
                </div>
              </div>
              <textarea v-model="editorText" class="editor-textarea" spellcheck="false"></textarea>
            </div>
          </div>

          <div class="modal-footer">
            <span class="count-info">显示 {{ filteredSources.length }} / {{ sources.length }}</span>
          </div>
        </div>
      </div>
    </Transition>
    <Transition name="scale">
      <div v-if="loginPreviewVisible" class="login-preview-container" @click.self="loginPreviewVisible = false">
        <div class="login-preview-modal">
          <div class="login-preview-header">
            <div>
              <h3>书源登录页</h3>
              <p>{{ loginPreviewUrl }}</p>
            </div>
            <button class="icon-btn" @click="loginPreviewVisible = false" title="关闭">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6 6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
          <iframe class="login-preview-frame" :src="loginPreviewFrameUrl"></iframe>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  getBookSources,
  deleteBookSource,
  loginBookSource,
  saveBookSource,
  saveBookSources,
  readRemoteSourceFile,
  readSourceFile,
} from '../api/source'
import { useAppStore } from '../stores/app'
import type { BookSource } from '../types'

type SourceSubscription = {
  url: string
  lastSyncedAt?: number
}

const SUBSCRIPTION_KEY = 'reader-source-subscriptions'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const appStore = useAppStore()

const sources = ref<BookSource[]>([])
const loading = ref(false)
const filterText = ref('')
const filterGroup = ref('')
const fileInputRef = ref<HTMLInputElement | null>(null)

const showRemotePanel = ref(true)
const remoteUrl = ref('')
const subscriptions = ref<SourceSubscription[]>(loadSubscriptions())

const editingSource = ref<BookSource | null>(null)
const editorText = ref(JSON.stringify(createEmptySource(), null, 2))
const sourceLoginLoading = ref(false)
const loginPreviewVisible = ref(false)
const loginPreviewUrl = ref('')
const loginPreviewFrameUrl = ref('')

const groupList = computed(() => {
  const groups = new Set<string>()
  sources.value.forEach((s) => {
    if (s.bookSourceGroup) {
      s.bookSourceGroup.split(/[,;；、]/).forEach((g) => {
        const trimmed = g.trim()
        if (trimmed) groups.add(trimmed)
      })
    }
  })
  return Array.from(groups).sort()
})

const filteredSources = computed(() => {
  let list = sources.value
  if (filterText.value) {
    const kw = filterText.value.toLowerCase()
    list = list.filter(
      (s) =>
        s.bookSourceName.toLowerCase().includes(kw) ||
        s.bookSourceUrl.toLowerCase().includes(kw)
    )
  }
  if (filterGroup.value) {
    list = list.filter((s) => s.bookSourceGroup?.includes(filterGroup.value))
  }
  return list
})

const canLoginSource = computed(() => {
  try {
    const parsed = JSON.parse(editorText.value) as BookSource
    return Boolean(parsed.bookSourceUrl?.trim() && parsed.loginUrl?.trim())
  } catch {
    return false
  }
})

function createEmptySource(): BookSource {
  return {
    bookSourceName: '新增书源',
    bookSourceUrl: '',
    enabled: true,
  }
}

function loadSubscriptions(): SourceSubscription[] {
  try {
    const raw = localStorage.getItem(SUBSCRIPTION_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

function persistSubscriptions() {
  localStorage.setItem(SUBSCRIPTION_KEY, JSON.stringify(subscriptions.value))
}

async function loadSources() {
  loading.value = true
  try {
    sources.value = await getBookSources()
  } catch (e: unknown) {
    appStore.showToast((e as Error).message, 'error')
  } finally {
    loading.value = false
  }
}

async function toggleSource(source: BookSource) {
  const next = { ...source, enabled: source.enabled === false ? true : false }
  try {
    await saveBookSource(next)
    Object.assign(source, next)
    appStore.showToast('书源状态已更新', 'success')
  } catch (e: unknown) {
    appStore.showToast((e as Error).message, 'error')
  }
}

async function removeSource(source: BookSource) {
  if (!confirm(`确定删除书源 "${source.bookSourceName}"？`)) return
  try {
    await deleteBookSource(source.bookSourceUrl)
    sources.value = sources.value.filter((s) => s.bookSourceUrl !== source.bookSourceUrl)
    if (editingSource.value?.bookSourceUrl === source.bookSourceUrl) {
      createSource()
    }
    appStore.showToast('已删除', 'success')
  } catch (e: unknown) {
    appStore.showToast((e as Error).message, 'error')
  }
}

function createSource() {
  editingSource.value = null
  editorText.value = JSON.stringify(createEmptySource(), null, 2)
}

function editSource(source: BookSource) {
  editingSource.value = source
  editorText.value = JSON.stringify(source, null, 2)
}

function formatEditor() {
  try {
    const parsed = JSON.parse(editorText.value)
    editorText.value = JSON.stringify(parsed, null, 2)
  } catch (e) {
    appStore.showToast('JSON 格式错误，无法格式化', 'error')
  }
}

async function saveEditor() {
  try {
    const parsed = JSON.parse(editorText.value) as BookSource
    if (!parsed.bookSourceName?.trim()) {
      throw new Error('书源名称不能为空')
    }
    if (!parsed.bookSourceUrl?.trim()) {
      throw new Error('书源链接不能为空')
    }
    await saveBookSource(parsed)
    appStore.showToast('保存书源成功', 'success')
    await loadSources()
    const latest = sources.value.find((item) => item.bookSourceUrl === parsed.bookSourceUrl)
    if (latest) {
      editSource(latest)
    }
  } catch (e: unknown) {
    appStore.showToast((e as Error).message || '保存失败', 'error')
  }
}

async function handleSourceLogin() {
  try {
    const parsed = JSON.parse(editorText.value) as BookSource
    if (!parsed.bookSourceUrl?.trim()) {
      throw new Error('书源链接不能为空')
    }
    if (!parsed.loginUrl?.trim()) {
      throw new Error('当前书源未配置 loginUrl')
    }

    sourceLoginLoading.value = true
    if (!editingSource.value || editingSource.value.bookSourceUrl !== parsed.bookSourceUrl) {
      await saveBookSource(parsed)
      await loadSources()
      const latest = sources.value.find((item) => item.bookSourceUrl === parsed.bookSourceUrl)
      if (latest) {
        editSource(latest)
      }
    }

    const result = await loginBookSource(parsed.bookSourceUrl)
    const check = typeof result.checkResult === 'string' && result.checkResult.trim()
      ? `，校验结果：${result.checkResult}`
      : ''
    if (result.url?.trim()) {
      loginPreviewUrl.value = result.url
      loginPreviewFrameUrl.value = buildLoginProxyUrl(parsed.bookSourceUrl, result.url)
      loginPreviewVisible.value = true
    }
    appStore.showToast(`书源登录请求已完成，状态 ${result.status}${check}`, 'success')
  } catch (e: unknown) {
    appStore.showToast((e as Error).message || '书源登录失败', 'error')
  } finally {
    sourceLoginLoading.value = false
  }
}

function buildLoginProxyUrl(bookSourceUrl: string, targetUrl: string) {
  const params = new URLSearchParams()
  const accessToken = localStorage.getItem('accessToken')
  if (accessToken) {
    params.set('accessToken', accessToken)
  }
  params.set('bookSourceUrl', bookSourceUrl)
  params.set('url', targetUrl)
  return `/reader3/bookSourceProxy?${params.toString()}`
}

function triggerFileImport() {
  fileInputRef.value?.click()
}

async function handleFileImport(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  try {
    const imported = await readSourceFile(file)
    if (!imported.length) {
      throw new Error('文件中没有可导入的书源')
    }
    await saveBookSources(imported)
    appStore.showToast(`成功导入 ${imported.length} 个书源`, 'success')
    await loadSources()
  } catch (e: unknown) {
    appStore.showToast((e as Error).message || '导入失败', 'error')
  } finally {
    input.value = ''
  }
}

async function importRemoteSource() {
  if (!remoteUrl.value) {
    appStore.showToast('请输入远程书源链接', 'warning')
    return
  }
  try {
    const raw = await readRemoteSourceFile(remoteUrl.value)
    const parsed = raw.flatMap((item) => {
      try {
        const value = JSON.parse(item)
        return Array.isArray(value) ? value : [value]
      } catch {
        return []
      }
    }) as BookSource[]
    if (!parsed.length) {
      throw new Error('远程书源文件为空或格式错误')
    }
    await saveBookSources(parsed)
    touchSubscription(remoteUrl.value)
    appStore.showToast(`成功同步 ${parsed.length} 个书源`, 'success')
    await loadSources()
  } catch (e: unknown) {
    appStore.showToast((e as Error).message || '远程同步失败', 'error')
  }
}

function saveSubscription() {
  if (!remoteUrl.value) {
    appStore.showToast('请输入远程书源链接', 'warning')
    return
  }
  if (!subscriptions.value.find((item) => item.url === remoteUrl.value)) {
    subscriptions.value.unshift({ url: remoteUrl.value })
    persistSubscriptions()
    appStore.showToast('订阅已保存', 'success')
  }
}

async function syncSubscription(url: string) {
  remoteUrl.value = url
  await importRemoteSource()
}

function removeSubscription(url: string) {
  subscriptions.value = subscriptions.value.filter((item) => item.url !== url)
  persistSubscriptions()
}

function touchSubscription(url: string) {
  const existing = subscriptions.value.find((item) => item.url === url)
  if (existing) {
    existing.lastSyncedAt = Date.now()
  } else {
    subscriptions.value.unshift({ url, lastSyncedAt: Date.now() })
  }
  persistSubscriptions()
}

function exportSources() {
  const blob = new Blob([JSON.stringify(sources.value, null, 2)], { type: 'application/json;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = `reader-book-sources-${formatDateForFile()}.json`
  anchor.click()
  URL.revokeObjectURL(url)
}

function formatDateForFile() {
  const date = new Date()
  const pad = (v: number) => `${v}`.padStart(2, '0')
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}-${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`
}

function formatTime(ts: number) {
  return new Date(ts).toLocaleString()
}

function close() {
  emit('update:modelValue', false)
}

watch(() => props.modelValue, (v) => {
  if (v) {
    if (sources.value.length === 0) {
      loadSources()
    }
    if (!editingSource.value && !editorText.value.trim()) {
      createSource()
    }
  }
}, { immediate: true })
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

.login-preview-container {
  position: fixed;
  inset: 0;
  z-index: calc(var(--z-modal) + 1);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(0, 0, 0, 0.35);
}

.login-preview-modal {
  width: min(980px, 100%);
  height: min(86vh, 900px);
  background: var(--color-bg-elevated);
  border-radius: var(--radius-xl);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-xl);
}

.login-preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-light);
}

.login-preview-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
}

.login-preview-header p {
  margin-top: 6px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  word-break: break-all;
}

.login-preview-frame {
  flex: 1;
  width: 100%;
  border: none;
  background: #fff;
}

.source-modal {
  width: min(1120px, 100%);
  max-height: min(88vh, calc(100dvh - var(--safe-area-top) - var(--safe-area-bottom) - 32px));
  background: var(--color-bg-elevated);
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-xl);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: calc(var(--space-5) + var(--safe-area-top)) var(--space-6) var(--space-5);
  border-bottom: 1px solid var(--color-border-light);
  flex-shrink: 0;
}

.modal-header h2 {
  font-size: var(--text-lg);
  font-weight: 700;
}

.header-subtitle {
  margin-top: 6px;
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.header-actions,
.toolbar,
.source-actions,
.editor-actions,
.subscription-actions {
  display: flex;
  gap: var(--space-2);
}

.toolbar {
  padding: 16px 24px 0;
  flex-wrap: wrap;
  align-items: center;
}

.action-btn,
.mini-btn,
.icon-btn {
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background: transparent;
  cursor: pointer;
  transition: all var(--duration-fast);
}

.action-btn {
  padding: 10px 14px;
  font-size: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 40px;
  white-space: nowrap;
}

.action-btn.primary,
.mini-btn.primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

.action-btn-solid {
  background: var(--color-bg-hover);
}

.mini-btn {
  padding: 6px 10px;
  font-size: 12px;
}

.mini-btn.danger {
  color: var(--color-danger);
  border-color: rgba(245, 34, 45, 0.2);
}

.icon-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}

.icon-btn.small {
  width: 28px;
  height: 28px;
}

.icon-btn:hover,
.action-btn:hover,
.mini-btn:hover {
  background: var(--color-bg-hover);
}

.icon-btn svg {
  width: 18px;
  height: 18px;
}

.icon-btn.small svg {
  width: 14px;
  height: 14px;
}

.icon-btn.danger:hover {
  background: rgba(245, 34, 45, 0.08);
  color: var(--color-danger);
}

.icon-btn.spinning svg {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.hidden-input {
  display: none;
}

.remote-panel,
.filter-bar,
.modal-footer {
  padding: 16px 24px 0;
}

.remote-panel {
  margin-top: 8px;
}

.remote-panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 16px;
  border: 1px solid var(--color-border-light);
  border-radius: 16px;
  background: rgba(201, 127, 58, 0.05);
}

.remote-panel-header h3 {
  margin: 0;
  font-size: 15px;
}

.remote-panel-header p {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.remote-panel-body {
  padding-top: 12px;
}

.remote-form {
  display: flex;
  gap: 12px;
}

.remote-form input,
.search-field input,
.group-select,
.editor-textarea {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg);
  color: inherit;
}

.remote-form input {
  flex: 1;
  padding: 10px 12px;
}

.subscription-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.subscription-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--color-border-light);
  border-radius: 12px;
}

.subscription-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.subscription-url {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.subscription-time {
  margin-top: 4px;
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.filter-bar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.search-field {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg);
}

.search-field input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
}

.group-select {
  min-width: 160px;
  padding: 10px 12px;
}

.content-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(360px, 420px);
  gap: 16px;
  min-height: 0;
  flex: 1;
  padding: 16px 24px 0;
  overflow: hidden;
}

.source-list-wrapper,
.editor-panel {
  min-height: 0;
  border: 1px solid var(--color-border-light);
  border-radius: 18px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.source-list {
  flex: 1;
  overflow: auto;
}

.source-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border-light);
}

.source-item.active {
  background: rgba(201, 127, 58, 0.06);
}

.source-info {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  cursor: pointer;
}

.source-name {
  font-weight: 600;
}

.source-url {
  font-size: 12px;
  color: var(--color-text-tertiary);
  word-break: break-all;
}

.source-group {
  width: fit-content;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--color-bg-hover);
  font-size: 11px;
}

.editor-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border-light);
}

.editor-head h3 {
  margin: 0;
  font-size: 15px;
}

.editor-textarea {
  flex: 1;
  width: 100%;
  resize: none;
  border: none;
  outline: none;
  padding: 16px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.5;
}

.loading-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.loading-spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.count-info {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding-bottom: 16px;
  display: inline-block;
}

@media (max-width: 900px) {
  .modal-container {
    align-items: stretch;
    padding: 8px;
  }

  .source-modal {
    width: 100%;
    max-height: calc(100dvh - 16px);
    border-radius: 24px;
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
  }

  .content-grid {
    grid-template-columns: 1fr;
    overflow: visible;
    flex: none;
  }

  .source-list-wrapper {
    min-height: 38vh;
    overflow: visible;
  }

  .editor-panel {
    min-height: 220px;
    max-height: 32vh;
    overflow: hidden;
  }

  .source-list {
    overflow: visible;
  }

  .remote-form,
  .filter-bar,
  .subscription-item {
    flex-direction: column;
    align-items: stretch;
  }
}

@media (max-width: 420px) {
  .modal-container {
    padding:
      calc(8px + var(--safe-area-top))
      calc(8px + var(--safe-area-right))
      calc(8px + var(--safe-area-bottom))
      calc(8px + var(--safe-area-left));
  }

  .source-modal {
    border-radius: 20px;
  }

  .modal-header {
    padding: calc(var(--space-4) + var(--safe-area-top)) var(--space-4) var(--space-4);
  }

  .toolbar,
  .remote-panel,
  .filter-bar,
  .content-grid,
  .modal-footer {
    padding-left: 16px;
    padding-right: 16px;
  }

  .action-btn {
    min-height: 36px;
    padding: 8px 12px;
    font-size: 13px;
  }
}
</style>
