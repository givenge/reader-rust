<template>
  <div class="rss-manage-view">
    <header class="rss-manage-hero">
      <div>
        <h1>RSS 源管理</h1>
        <p>在这里导入、导出、创建和编辑 RSS 源配置。</p>
      </div>
      <div class="rss-manage-actions">
        <button class="ghost-btn" @click="goBack">返回 RSS</button>
        <button class="ghost-btn" @click="refreshAll">刷新</button>
        <button class="ghost-btn" @click="exportSources">导出 JSON</button>
      </div>
    </header>

    <section class="rss-manage-tools">
      <button class="tool-btn" @click="triggerFileImport">本地导入 JSON</button>
      <button class="tool-btn" @click="importRemoteJson">远程导入 JSON</button>
      <button class="tool-btn primary" @click="createSource">新增 RSS 源</button>
      <input ref="fileInputRef" type="file" accept=".json,.txt" class="hidden-input" @change="handleFileImport" />
      <input v-model.trim="remoteJsonUrl" class="remote-input" placeholder="输入 Legado RSS JSON 链接" />
    </section>

    <section class="rss-manage-layout">
      <aside class="rss-sources">
        <div class="panel-title">RSS 源 ({{ filteredSources.length }})</div>
        <div class="filter-row">
          <input v-model.trim="filterText" placeholder="搜索源名称或链接" />
          <select v-model="filterGroup">
            <option value="">全部分组</option>
            <option v-for="group in groupList" :key="group" :value="group">{{ group }}</option>
          </select>
        </div>

        <div v-if="!store.sources.length" class="empty-box">暂无 RSS 源</div>
        <button
          v-for="source in filteredSources"
          :key="source.sourceUrl"
          class="source-item"
          :class="{ active: editingSource?.sourceUrl === source.sourceUrl }"
          @click="editSource(source)"
        >
          <div class="source-main">
            <div class="source-name-row">
              <span class="source-name">{{ source.sourceName }}</span>
              <span v-if="source.sourceGroup" class="source-group">{{ source.sourceGroup }}</span>
            </div>
            <div class="source-url">{{ source.sourceUrl }}</div>
          </div>
          <div class="source-actions">
            <label class="toggle">
              <input type="checkbox" :checked="source.enabled !== false" @change.stop="toggleSource(source)" />
              <span class="toggle-slider"></span>
            </label>
            <button class="mini-btn danger" @click.stop="handleDelete(source)">删除</button>
          </div>
        </button>
      </aside>

      <section class="rss-editor">
        <div class="panel-title-row">
          <div class="panel-title">{{ editingSource ? '编辑 RSS 源' : '新增 RSS 源' }}</div>
          <div class="panel-actions">
            <button class="ghost-btn" @click="formatEditor">格式化</button>
            <button class="ghost-btn primary" @click="saveEditor">保存</button>
          </div>
        </div>

        <div class="visual-form">
          <label>
            <span>源名称</span>
            <input v-model.trim="formSourceName" @input="patchEditorField('sourceName', formSourceName)" placeholder="请输入 RSS 名称" />
          </label>
          <label>
            <span>源链接</span>
            <input v-model.trim="formSourceUrl" @input="patchEditorField('sourceUrl', formSourceUrl)" placeholder="请输入 RSS 链接" />
          </label>
          <label class="group-field">
            <span>分组</span>
            <input v-model.trim="formSourceGroup" @input="patchEditorField('sourceGroup', formSourceGroup)" placeholder="多个分组可用逗号分隔" />
          </label>
          <div class="group-chip-row" v-if="groupList.length">
            <button
              v-for="group in groupList"
              :key="group"
              class="group-chip-btn"
              :class="{ active: currentEditorGroups.includes(group) }"
              @click="toggleEditorGroup(group)"
            >
              {{ group }}
            </button>
          </div>
        </div>

        <textarea v-model="editorText" class="editor-textarea" spellcheck="false"></textarea>
      </section>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { readRemoteRssSourceFile, readRssSourceFile, saveRssSource } from '../api/rss'
import { useAppStore } from '../stores/app'
import { useRssStore } from '../stores/rss'
import type { RssSource } from '../types'

const router = useRouter()
const appStore = useAppStore()
const store = useRssStore()

const fileInputRef = ref<HTMLInputElement | null>(null)
const remoteJsonUrl = ref('')
const filterText = ref('')
const filterGroup = ref('')
const editingSource = ref<RssSource | null>(null)
const editorText = ref(JSON.stringify(createEmptySource(), null, 2))
const formSourceName = ref('')
const formSourceUrl = ref('')
const formSourceGroup = ref('')

const groupList = computed(() => {
  const groups = new Set<string>()
  store.sources.forEach((source) => {
    if (source.sourceGroup) {
      source.sourceGroup.split(/[,;，、]/).forEach((item) => {
        const trimmed = item.trim()
        if (trimmed) groups.add(trimmed)
      })
    }
  })
  return Array.from(groups).sort()
})

const currentEditorGroups = computed(() => formSourceGroup.value.split(/[,;，、]/).map((item) => item.trim()).filter(Boolean))

const filteredSources = computed(() => {
  let list = store.sources
  if (filterText.value) {
    const keyword = filterText.value.toLowerCase()
    list = list.filter((source) =>
      source.sourceName.toLowerCase().includes(keyword) ||
      source.sourceUrl.toLowerCase().includes(keyword),
    )
  }
  if (filterGroup.value) {
    list = list.filter((source) => source.sourceGroup?.includes(filterGroup.value))
  }
  return list
})

onMounted(async () => {
  await store.fetchSources()
  if (store.sources.length) {
    editSource(store.sources[0])
  } else {
    syncFormFromEditor()
  }
})

function createEmptySource(): RssSource {
  return {
    sourceName: '新增 RSS 源',
    sourceUrl: '',
    enabled: true,
    enabledCookieJar: true,
    enableJs: true,
    loadWithBaseUrl: true,
    singleUrl: true,
    sourceGroup: '',
    sourceIcon: '',
    customOrder: 0,
    articleStyle: 0,
    lastUpdateTime: 0,
  }
}

function parseEditor() {
  return JSON.parse(editorText.value) as RssSource
}

function syncFormFromEditor() {
  try {
    const parsed = parseEditor()
    formSourceName.value = parsed.sourceName || ''
    formSourceUrl.value = parsed.sourceUrl || ''
    formSourceGroup.value = parsed.sourceGroup || ''
  } catch {
    formSourceName.value = ''
    formSourceUrl.value = ''
    formSourceGroup.value = ''
  }
}

function patchEditorField(field: keyof RssSource, value: string) {
  try {
    const parsed = parseEditor()
    parsed[field] = value as never
    editorText.value = JSON.stringify(parsed, null, 2)
  } catch {
    // ignore until valid JSON
  }
}

function toggleEditorGroup(group: string) {
  const groups = new Set(currentEditorGroups.value)
  if (groups.has(group)) {
    groups.delete(group)
  } else {
    groups.add(group)
  }
  formSourceGroup.value = Array.from(groups).join(', ')
  patchEditorField('sourceGroup', formSourceGroup.value)
}

async function refreshAll() {
  await store.fetchSources()
}

function goBack() {
  router.push('/rss')
}

function editSource(source: RssSource) {
  editingSource.value = source
  editorText.value = JSON.stringify(source, null, 2)
  syncFormFromEditor()
}

function createSource() {
  editingSource.value = null
  editorText.value = JSON.stringify(createEmptySource(), null, 2)
  syncFormFromEditor()
}

function formatEditor() {
  try {
    const parsed = parseEditor()
    editorText.value = JSON.stringify(parsed, null, 2)
    syncFormFromEditor()
  } catch {
    appStore.showToast('JSON 格式错误，无法格式化', 'error')
  }
}

async function saveEditor() {
  try {
    const parsed = parseEditor()
    if (!parsed.sourceName?.trim()) throw new Error('RSS 名称不能为空')
    if (!parsed.sourceUrl?.trim()) throw new Error('RSS 链接不能为空')
    await saveRssSource(parsed)
    await store.fetchSources()
    const latest = store.sources.find((item) => item.sourceUrl === parsed.sourceUrl)
    if (latest) {
      editSource(latest)
    }
    appStore.showToast('RSS 源已保存', 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '保存失败', 'error')
  }
}

async function toggleSource(source: RssSource) {
  const next = { ...source, enabled: source.enabled === false ? true : false }
  await saveRssSource(next)
  Object.assign(source, next)
  if (editingSource.value?.sourceUrl === source.sourceUrl) {
    editSource(next)
  }
  appStore.showToast('RSS 源状态已更新', 'success')
}

async function handleDelete(source: RssSource) {
  if (!confirm(`确定删除 RSS 源 "${source.sourceName}"？`)) return
  await store.removeSource(source)
  if (editingSource.value?.sourceUrl === source.sourceUrl) {
    createSource()
  }
  appStore.showToast('RSS 源已删除', 'success')
}

function triggerFileImport() {
  fileInputRef.value?.click()
}

async function handleFileImport(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  try {
    const imported = await readRssSourceFile(file)
    if (!imported.length) throw new Error('文件中没有可导入的 RSS 源')
    await store.addSources(imported)
    appStore.showToast(`成功导入 ${imported.length} 个 RSS 源`, 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '导入失败', 'error')
  } finally {
    input.value = ''
  }
}

async function importRemoteJson() {
  if (!remoteJsonUrl.value) {
    appStore.showToast('请输入远程 RSS JSON 链接', 'warning')
    return
  }
  try {
    const raw = await readRemoteRssSourceFile(remoteJsonUrl.value)
    const parsed = raw.flatMap((item) => {
      try {
        const value = JSON.parse(item)
        return Array.isArray(value) ? value : [value]
      } catch {
        return []
      }
    }) as RssSource[]
    if (!parsed.length) throw new Error('远程 RSS JSON 为空或格式错误')
    await store.addSources(parsed)
    appStore.showToast(`成功导入 ${parsed.length} 个 RSS 源`, 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '远程导入失败', 'error')
  }
}

function exportSources() {
  const blob = new Blob([JSON.stringify(store.sources, null, 2)], { type: 'application/json;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = `reader-rss-sources-${Date.now()}.json`
  anchor.click()
  URL.revokeObjectURL(url)
}
</script>

<style scoped>
.rss-manage-view {
  min-height: calc(100dvh - var(--header-height) - var(--safe-area-top));
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.rss-manage-hero,
.rss-manage-tools,
.rss-sources,
.rss-editor {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-light);
  border-radius: 24px;
}

.rss-manage-hero,
.rss-manage-tools {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  padding: 18px 20px;
}

.rss-manage-hero h1 {
  margin: 0;
}

.rss-manage-hero p {
  margin: 6px 0 0;
  color: var(--color-text-tertiary);
}

.rss-manage-actions,
.panel-actions,
.source-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.rss-manage-tools {
  justify-content: flex-start;
}

.remote-input,
.filter-row input,
.filter-row select,
.visual-form input {
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  border-radius: 12px;
  padding: 10px 12px;
}

.remote-input {
  min-width: min(420px, 100%);
  flex: 1;
}

.rss-manage-layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  gap: 16px;
}

.rss-sources,
.rss-editor {
  min-height: 0;
  overflow: auto;
  padding: 16px;
}

.panel-title {
  font-size: 14px;
  font-weight: 700;
  margin-bottom: 12px;
}

.panel-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.filter-row,
.visual-form {
  display: grid;
  gap: 10px;
  margin-bottom: 14px;
}

.visual-form label {
  display: grid;
  gap: 6px;
}

.visual-form span {
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.group-chip-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.group-chip-btn,
.tool-btn,
.ghost-btn,
.mini-btn {
  padding: 10px 16px;
  border-radius: 999px;
  border: 1px solid var(--color-border-light);
  background: var(--color-bg);
  transition: all var(--duration-fast) var(--ease-out);
}

.group-chip-btn.active,
.tool-btn.primary,
.ghost-btn.primary {
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-primary);
  border-color: rgba(201, 127, 58, 0.2);
}

.source-item {
  width: 100%;
  text-align: left;
  border: 1px solid transparent;
  background: var(--color-bg);
  border-radius: 16px;
  padding: 14px;
  margin-bottom: 10px;
  transition: all var(--duration-fast) var(--ease-out);
}

.source-item.active {
  border-color: rgba(201, 127, 58, 0.26);
  background: rgba(201, 127, 58, 0.08);
}

.source-main {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.source-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.source-name {
  font-weight: 700;
}

.source-group {
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-primary);
  font-size: 12px;
}

.source-url {
  color: var(--color-text-tertiary);
  font-size: 12px;
  line-height: 1.5;
  word-break: break-all;
}

.source-actions {
  margin-top: 12px;
  justify-content: space-between;
  align-items: center;
}

.editor-textarea {
  width: 100%;
  min-height: 460px;
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  border-radius: 18px;
  padding: 14px 16px;
  resize: vertical;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  line-height: 1.6;
}

.mini-btn.danger {
  color: var(--color-danger);
}

.empty-box {
  min-height: 140px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--color-text-tertiary);
}

.hidden-input {
  display: none;
}

.toggle {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.toggle input {
  display: none;
}

.toggle-slider {
  width: 42px;
  height: 24px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.14);
  position: relative;
}

.toggle-slider::after {
  content: '';
  position: absolute;
  top: 3px;
  left: 3px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: white;
  transition: transform var(--duration-fast) var(--ease-out);
}

.toggle input:checked + .toggle-slider {
  background: rgba(201, 127, 58, 0.38);
}

.toggle input:checked + .toggle-slider::after {
  transform: translateX(18px);
}

@media (max-width: 960px) {
  .rss-manage-layout {
    grid-template-columns: 1fr;
  }
}
</style>
