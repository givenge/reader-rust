<template>
  <div class="rss-view">
    <div class="rss-header">
      <div>
        <h2>RSS 订阅</h2>
        <p>对齐 Legado 风格的 RSS 源管理、JSON 导入与文章阅读</p>
      </div>
      <div class="rss-header-actions">
        <button class="ghost-btn" @click="refreshAll">刷新</button>
        <button class="ghost-btn" @click="exportSources">导出 JSON</button>
      </div>
    </div>

    <div class="rss-tools">
      <button class="tool-btn" @click="triggerFileImport">本地导入 JSON</button>
      <button class="tool-btn" @click="importRemoteJson">远程导入 JSON</button>
      <button class="tool-btn primary" @click="createSource">新增 RSS 源</button>
      <input ref="fileInputRef" type="file" accept=".json,.txt" class="hidden-input" @change="handleFileImport" />
      <input v-model.trim="remoteJsonUrl" class="remote-input" placeholder="输入 Legado RSS JSON 链接" />
    </div>

    <div class="rss-layout">
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
        <textarea v-model="editorText" class="editor-textarea" spellcheck="false"></textarea>
      </section>

      <section class="rss-reader">
        <div class="reader-top">
          <div class="reader-source-select">
            <select v-model="store.activeSourceUrl" @change="handleReaderSourceChange" v-if="store.sources.length">
              <option v-for="source in store.sources" :key="source.sourceUrl" :value="source.sourceUrl">
                {{ source.sourceName }}
              </option>
            </select>
          </div>
          <button class="ghost-btn" :disabled="!store.activeSourceUrl" @click="store.fetchArticles(true)">刷新文章</button>
        </div>

        <div class="reader-body">
          <div class="article-list">
            <div class="sub-title">文章列表</div>
            <div v-if="!store.activeSourceUrl" class="empty-box">先选择一个 RSS 源</div>
            <template v-else>
              <button
                v-for="article in store.articles"
                :key="article.link"
                class="article-item"
                :class="{ active: store.activeArticle?.link === article.link }"
                @click="store.openArticle(article)"
              >
                <div class="article-title">{{ article.title || '无标题' }}</div>
                <div class="article-meta">{{ article.pubDate || article.origin }}</div>
                <div v-if="article.description" class="article-desc">{{ article.description }}</div>
              </button>
              <button v-if="store.hasMore && !store.loading" class="load-more-btn" @click="store.fetchArticles()">加载更多</button>
              <div v-if="store.loading" class="empty-box">文章加载中...</div>
            </template>
          </div>

          <div class="article-content">
            <div class="sub-title">{{ store.activeArticle?.title || '正文' }}</div>
            <div v-if="store.contentLoading" class="empty-box">正文加载中...</div>
            <div v-else-if="store.activeContent" class="content-html" v-html="store.activeContent"></div>
            <div v-else class="empty-box">请选择文章阅读</div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAppStore } from '../stores/app'
import { useRssStore } from '../stores/rss'
import { readRemoteRssSourceFile, readRssSourceFile, saveRssSource } from '../api/rss'
import type { RssSource } from '../types'

const appStore = useAppStore()
const store = useRssStore()

const fileInputRef = ref<HTMLInputElement | null>(null)
const remoteJsonUrl = ref('')
const filterText = ref('')
const filterGroup = ref('')
const editingSource = ref<RssSource | null>(null)
const editorText = ref(JSON.stringify(createEmptySource(), null, 2))

const groupList = computed(() => {
  const groups = new Set<string>()
  store.sources.forEach((source) => {
    if (source.sourceGroup) {
      source.sourceGroup.split(/[,;；、]/).forEach((item) => {
        const trimmed = item.trim()
        if (trimmed) groups.add(trimmed)
      })
    }
  })
  return Array.from(groups).sort()
})

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
    if (store.activeSourceUrl) {
      await store.fetchArticles(true)
    }
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

async function refreshAll() {
  await store.fetchSources()
  if (store.activeSourceUrl) {
    await store.fetchArticles(true)
  }
}

function editSource(source: RssSource) {
  editingSource.value = source
  editorText.value = JSON.stringify(source, null, 2)
}

function createSource() {
  editingSource.value = null
  editorText.value = JSON.stringify(createEmptySource(), null, 2)
}

function formatEditor() {
  try {
    const parsed = JSON.parse(editorText.value)
    editorText.value = JSON.stringify(parsed, null, 2)
  } catch {
    appStore.showToast('JSON 格式错误，无法格式化', 'error')
  }
}

async function saveEditor() {
  try {
    const parsed = JSON.parse(editorText.value) as RssSource
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

async function handleReaderSourceChange() {
  if (store.activeSourceUrl) {
    await store.setSource(store.activeSourceUrl)
  }
}
</script>

<style scoped>
.rss-view {
  min-height: calc(100dvh - var(--header-height) - var(--safe-area-top));
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.rss-header,
.rss-tools {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  padding: 18px 20px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-light);
  border-radius: 20px;
}

.rss-header h2 {
  margin: 0;
}

.rss-header p {
  margin: 6px 0 0;
  color: var(--color-text-tertiary);
}

.rss-header-actions,
.panel-actions,
.source-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.rss-tools {
  justify-content: flex-start;
}

.remote-input,
.filter-row input,
.filter-row select {
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  border-radius: 12px;
  padding: 10px 12px;
}

.remote-input {
  min-width: min(420px, 100%);
  flex: 1;
}

.rss-layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 320px 380px minmax(0, 1fr);
  gap: 16px;
}

.rss-sources,
.rss-editor,
.rss-reader {
  min-height: 0;
  overflow: auto;
  padding: 16px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-light);
  border-radius: 20px;
}

.panel-title,
.sub-title {
  font-size: 14px;
  font-weight: 700;
  margin-bottom: 12px;
}

.panel-title-row,
.reader-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.filter-row {
  display: grid;
  grid-template-columns: 1fr 120px;
  gap: 8px;
  margin-bottom: 12px;
}

.source-item,
.article-item {
  width: 100%;
  text-align: left;
  border: 1px solid var(--color-border-light);
  background: var(--color-bg);
  border-radius: 14px;
  padding: 12px;
  margin-bottom: 10px;
  cursor: pointer;
}

.source-item.active,
.article-item.active {
  border-color: var(--color-primary);
  background: rgba(201, 127, 58, 0.08);
}

.source-item {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.source-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.source-name,
.article-title {
  font-weight: 700;
}

.source-group {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 999px;
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-primary);
}

.source-url,
.article-meta {
  font-size: 12px;
  color: var(--color-text-tertiary);
  word-break: break-all;
}

.article-desc {
  margin-top: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.editor-textarea {
  width: 100%;
  min-height: 580px;
  resize: vertical;
  border: 1px solid var(--color-border-light);
  border-radius: 16px;
  padding: 14px;
  background: var(--color-bg);
  font: 12px/1.6 ui-monospace, SFMono-Regular, Menlo, monospace;
}

.reader-body {
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  gap: 12px;
}

.article-list,
.article-content {
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--color-border-light);
  border-radius: 16px;
  padding: 12px;
  background: var(--color-bg);
}

.content-html {
  line-height: 1.8;
}

.content-html :deep(img) {
  max-width: 100%;
  height: auto;
}

.ghost-btn,
.tool-btn,
.mini-btn,
.load-more-btn,
.toggle-slider {
  border: 1px solid var(--color-border);
  background: transparent;
  border-radius: 12px;
}

.ghost-btn,
.tool-btn,
.mini-btn,
.load-more-btn {
  padding: 10px 14px;
  cursor: pointer;
}

.tool-btn.primary,
.ghost-btn.primary {
  background: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}

.mini-btn.danger {
  color: var(--color-danger);
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
  position: relative;
  width: 42px;
  height: 24px;
  background: rgba(0,0,0,0.08);
  border-radius: 999px;
}

.toggle-slider::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 4px rgba(0,0,0,0.18);
  transition: transform 0.2s ease;
}

.toggle input:checked + .toggle-slider {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.toggle input:checked + .toggle-slider::after {
  transform: translateX(18px);
}

.empty-box {
  padding: 28px 12px;
  text-align: center;
  color: var(--color-text-tertiary);
}

.hidden-input {
  display: none;
}

@media (max-width: 1280px) {
  .rss-layout {
    grid-template-columns: 1fr;
  }

  .reader-body {
    grid-template-columns: 1fr;
  }
}
</style>
