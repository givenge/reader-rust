<template>
  <div class="ai-book-view">
    <div v-if="loading" class="ai-loading">
      <div class="loading-spinner"></div>
      <span>加载中...</span>
    </div>

    <div v-else-if="book && memory" class="ai-shell">
      <header class="ai-header">
        <div>
          <button class="back-btn" @click="goBack">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="m15 18-6-6 6-6" />
            </svg>
            返回
          </button>
          <h1>{{ book.name }}</h1>
          <p>{{ book.author || '未知作者' }} · {{ progressText }}</p>
        </div>

        <div class="header-actions">
          <label class="enable-switch">
            <input type="checkbox" :checked="memory.enabled" @change="toggleEnabled" />
            <span></span>
            自动更新
          </label>
          <button class="primary-btn" :disabled="aiStore.isBusy" @click="updateToCurrent">
            {{ aiStore.phase === 'text' ? '更新中...' : '更新到当前进度' }}
          </button>
        </div>
      </header>

      <div v-if="aiStore.statusText || memory.lastError" class="status-strip" :class="{ error: aiStore.phase === 'error' || memory.lastError }">
        {{ aiStore.statusText || memory.lastError }}
      </div>

      <nav class="tabs">
        <button v-for="tab in tabs" :key="tab.key" :class="{ active: activeTab === tab.key }" @click="activeTab = tab.key">
          {{ tab.label }}
        </button>
      </nav>

      <main class="ai-content">
        <section v-if="activeTab === 'overview'" class="overview-grid">
          <div class="overview-main">
            <h2>世界观</h2>
            <p class="summary">{{ memory.summary || '暂无资料' }}</p>
            <div class="note-list">
              <article v-for="note in memory.worldview" :key="note.title" class="note-item">
                <div class="item-title">
                  <h3>{{ note.title }}</h3>
                  <span v-if="note.confidence">{{ note.confidence }}</span>
                </div>
                <p>{{ note.content }}</p>
              </article>
            </div>
          </div>

          <aside class="overview-side">
            <div class="metric">
              <span>角色</span>
              <strong>{{ memory.characters.length }}</strong>
            </div>
            <div class="metric">
              <span>关系</span>
              <strong>{{ memory.relationships.length }}</strong>
            </div>
            <div class="metric">
              <span>地点</span>
              <strong>{{ memory.locations.length }}</strong>
            </div>
            <div class="metric">
              <span>最近章节</span>
              <strong>{{ memory.processedChapterIndex != null ? memory.processedChapterIndex + 1 : '-' }}</strong>
            </div>
          </aside>
        </section>

        <section v-else-if="activeTab === 'characters'" class="list-panel">
          <article v-for="character in memory.characters" :key="character.name" class="list-item">
            <div class="item-title">
              <h3>{{ character.name }}</h3>
              <span v-if="character.faction">{{ character.faction }}</span>
            </div>
            <p>{{ character.status || character.description || '暂无状态' }}</p>
            <div class="meta-line">
              <span v-if="character.location">位置：{{ character.location }}</span>
              <span v-if="character.lastSeenChapter">最近：{{ character.lastSeenChapter }}</span>
              <span v-if="character.aliases?.length">别名：{{ character.aliases.join('、') }}</span>
            </div>
          </article>
          <EmptyState v-if="!memory.characters.length" text="暂无角色资料" />
        </section>

        <section v-else-if="activeTab === 'relationships'" class="relation-grid">
          <article v-for="relationship in memory.relationships" :key="`${relationship.source}-${relationship.target}-${relationship.relation}`" class="relation-item">
            <div class="relation-head">
              <strong>{{ relationship.source }}</strong>
              <span>{{ relationship.relation }}</span>
              <strong>{{ relationship.target }}</strong>
            </div>
            <p>{{ relationship.description || relationship.status || '暂无说明' }}</p>
          </article>
          <EmptyState v-if="!memory.relationships.length" text="暂无人物关系" />
        </section>

        <section v-else-if="activeTab === 'map'" class="map-panel">
          <div class="map-toolbar">
            <div>
              <h2>世界地图</h2>
              <p>{{ memory.map?.updatedAt ? formatTime(memory.map.updatedAt) : '未生成' }}</p>
            </div>
            <button class="secondary-btn" :disabled="aiStore.isBusy" @click="redrawMap">
              {{ aiStore.phase === 'map' ? '绘制中...' : '重绘地图' }}
            </button>
          </div>

          <div class="map-frame">
            <img v-if="memory.map?.imageUrl" :src="memory.map.imageUrl" alt="世界地图" />
            <div v-else class="map-empty">暂无地图</div>
          </div>

          <div class="location-list">
            <article v-for="location in memory.locations" :key="location.name" class="location-item">
              <div class="item-title">
                <h3>{{ location.name }}</h3>
                <span v-if="location.kind">{{ location.kind }}</span>
              </div>
              <p>{{ location.description }}</p>
              <div class="meta-line">
                <span v-if="location.status">状态：{{ location.status }}</span>
                <span v-if="location.relatedCharacters?.length">相关：{{ location.relatedCharacters.join('、') }}</span>
              </div>
            </article>
          </div>
        </section>

        <section v-else class="settings-panel">
          <div class="settings-grid">
            <label>
              <span>Base URL</span>
              <input v-model="configDraft.baseUrl" placeholder="http://localhost:8825" />
            </label>
            <label>
              <span>API Key</span>
              <input v-model="configDraft.apiKey" type="password" autocomplete="off" />
            </label>
            <label>
              <span>文本模型</span>
              <input v-model="configDraft.textModel" />
            </label>
            <label>
              <span>图片模型</span>
              <input v-model="configDraft.imageModel" />
            </label>
            <label>
              <span>图片尺寸</span>
              <select v-model="configDraft.imageSize">
                <option value="1024x1024">1024x1024</option>
                <option value="1792x1024">1792x1024</option>
                <option value="1024x1792">1024x1792</option>
              </select>
            </label>
          </div>
          <div class="settings-actions">
            <button class="primary-btn" @click="saveConfig">保存配置</button>
            <button class="danger-btn" @click="resetMemory">重置 AI资料</button>
          </div>
        </section>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getBookContent, getChapterList, getShelfBook } from '../api/bookshelf'
import { useAiBookStore } from '../stores/aiBook'
import { useAppStore } from '../stores/app'
import { useReaderStore } from '../stores/reader'
import type { AiBookConfig, Book, BookChapter } from '../types'

type AiTab = 'overview' | 'characters' | 'relationships' | 'map' | 'settings'

const EmptyState = defineComponent({
  props: { text: { type: String, required: true } },
  setup(props) {
    return () => h('div', { class: 'empty-state' }, props.text)
  },
})

const route = useRoute()
const router = useRouter()
const aiStore = useAiBookStore()
const appStore = useAppStore()
const readerStore = useReaderStore()

const loading = ref(true)
const activeTab = ref<AiTab>('overview')
const book = ref<Book | null>(null)
const chapters = ref<BookChapter[]>([])
const configDraft = reactive<AiBookConfig>({ ...aiStore.config })

const tabs: Array<{ key: AiTab; label: string }> = [
  { key: 'overview', label: '总览' },
  { key: 'characters', label: '角色' },
  { key: 'relationships', label: '关系' },
  { key: 'map', label: '地图' },
  { key: 'settings', label: '设置' },
]

const memory = computed(() => aiStore.memory)
const progressText = computed(() => {
  const index = memory.value?.processedChapterIndex
  if (index == null) return '尚未生成'
  return `已更新至第 ${index + 1} 章`
})

watch(
  () => aiStore.config,
  (next) => Object.assign(configDraft, next),
  { deep: true },
)

onMounted(async () => {
  await appStore.fetchUserInfo()
  aiStore.refreshConfig()
  Object.assign(configDraft, aiStore.config)
  const bookUrl = String(route.query.bookUrl || '')
  if (!bookUrl) {
    router.replace('/')
    return
  }
  try {
    book.value = await getShelfBook(bookUrl)
    await aiStore.load(book.value)
    chapters.value = await getChapterList({
      bookUrl: book.value.bookUrl,
      bookSourceUrl: book.value.origin,
    }).catch(() => [])
  } catch (error) {
    appStore.showToast((error as Error).message || 'AI资料加载失败', 'error')
    router.replace('/')
  } finally {
    loading.value = false
  }
})

function goBack() {
  router.back()
}

async function toggleEnabled(event: Event) {
  if (!book.value) return
  const enabled = (event.target as HTMLInputElement).checked
  try {
    await aiStore.setEnabled(book.value, enabled)
    appStore.showToast(enabled ? '已开启自动更新' : '已关闭自动更新', 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || '设置失败', 'error')
  }
}

async function updateToCurrent() {
  if (!book.value || !memory.value) return
  const targetIndex = resolveCurrentIndex()
  if (!chapters.value.length) {
    appStore.showToast('目录未加载，无法更新', 'warning')
    return
  }
  const startIndex = Math.max(0, (memory.value.processedChapterIndex ?? -1) + 1)
  if (startIndex > targetIndex) {
    appStore.showToast('当前进度已更新', 'success')
    return
  }

  try {
    let currentMemory = memory.value
    for (let index = startIndex; index <= targetIndex; index += 1) {
      const chapter = chapters.value[index]
      if (!chapter) continue
      const chapterContent = await resolveChapterContent(index, chapter)
      currentMemory = await aiStore.runChapterUpdate({
        book: book.value,
        chapter,
        chapterContent,
        current: currentMemory,
      })
    }
    appStore.showToast('AI资料已更新', 'success')
  } catch (error) {
    appStore.showToast((error as Error).message || 'AI资料更新失败', 'error')
  }
}

async function redrawMap() {
  if (!book.value) return
  await aiStore.redrawMap(book.value)
  if (aiStore.phase !== 'error') {
    appStore.showToast('地图已更新', 'success')
  }
}

function saveConfig() {
  aiStore.persistConfig({ ...configDraft })
  appStore.showToast('AI配置已保存', 'success')
}

async function resetMemory() {
  if (!book.value) return
  if (!confirm('确定重置当前书的 AI资料？')) return
  await aiStore.reset(book.value)
  appStore.showToast('AI资料已重置', 'success')
}

function resolveCurrentIndex() {
  if (readerStore.book?.bookUrl === book.value?.bookUrl) {
    return Math.max(0, readerStore.currentIndex)
  }
  return Math.max(0, Math.min(chapters.value.length - 1, book.value?.durChapterIndex || 0))
}

async function resolveChapterContent(index: number, chapter: BookChapter) {
  if (readerStore.book?.bookUrl === book.value?.bookUrl) {
    const content = await readerStore.fetchChapterContent(index)
    if (content) return content
  }
  return getBookContent({
    chapterUrl: chapter.url,
    bookSourceUrl: book.value?.origin,
  })
}

function formatTime(value: number) {
  return new Date(value).toLocaleString()
}
</script>

<style scoped>
.ai-book-view {
  height: 100%;
  overflow: hidden;
  background: var(--color-bg);
  color: var(--color-text);
}

.ai-loading {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-secondary);
}

.ai-shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  max-width: 1180px;
  margin: 0 auto;
  padding: 24px 28px;
  box-sizing: border-box;
  min-height: 0;
}

.ai-header {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  align-items: flex-end;
  border-bottom: 1px solid var(--color-border-light);
  padding-bottom: 18px;
}

.ai-header h1 {
  margin: 8px 0 4px;
  font-size: 28px;
  line-height: 1.15;
}

.ai-header p,
.map-toolbar p {
  margin: 0;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.back-btn,
.secondary-btn,
.primary-btn,
.danger-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 36px;
  padding: 0 14px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  font-weight: 600;
  cursor: pointer;
}

.back-btn svg {
  width: 16px;
  height: 16px;
}

.primary-btn {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

.danger-btn {
  color: var(--color-danger, #d14b4b);
}

.primary-btn:disabled,
.secondary-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.header-actions,
.settings-actions,
.map-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.enable-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.enable-switch input {
  display: none;
}

.enable-switch span {
  width: 36px;
  height: 20px;
  border-radius: 999px;
  background: var(--color-border);
  position: relative;
  transition: background var(--duration-fast);
}

.enable-switch span::after {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  border-radius: 50%;
  background: #fff;
  transition: transform var(--duration-fast);
}

.enable-switch input:checked + span {
  background: var(--color-primary);
}

.enable-switch input:checked + span::after {
  transform: translateX(16px);
}

.status-strip {
  margin-top: 14px;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-text-secondary);
  font-size: 13px;
}

.status-strip.error {
  background: rgba(209, 75, 75, 0.12);
}

.tabs {
  display: flex;
  gap: 4px;
  margin-top: 18px;
  border-bottom: 1px solid var(--color-border-light);
}

.tabs button {
  padding: 12px 16px;
  color: var(--color-text-tertiary);
  font-weight: 600;
  border-bottom: 2px solid transparent;
}

.tabs button.active {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.ai-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 22px 0 32px;
  scrollbar-width: none;
}

.overview-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 260px;
  gap: 24px;
}

.overview-main h2,
.map-toolbar h2 {
  margin: 0 0 14px;
  font-size: 18px;
}

.summary {
  margin: 0 0 18px;
  line-height: 1.8;
  color: var(--color-text-secondary);
}

.note-list,
.list-panel,
.relation-grid,
.location-list {
  display: grid;
  gap: 12px;
}

.note-item,
.list-item,
.relation-item,
.location-item,
.metric {
  border: 1px solid var(--color-border-light);
  border-radius: 8px;
  padding: 14px;
  background: var(--color-bg-elevated);
}

.item-title,
.relation-head {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-content: space-between;
}

.item-title h3 {
  margin: 0;
  font-size: 15px;
}

.item-title span,
.relation-head span {
  font-size: 12px;
  color: var(--color-primary);
}

.note-item p,
.list-item p,
.relation-item p,
.location-item p {
  margin: 8px 0 0;
  line-height: 1.7;
  color: var(--color-text-secondary);
}

.overview-side {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-content: start;
  gap: 12px;
}

.metric {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metric span,
.meta-line {
  color: var(--color-text-tertiary);
  font-size: 12px;
}

.metric strong {
  font-size: 28px;
}

.meta-line {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 10px;
}

.relation-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.map-panel {
  display: grid;
  gap: 18px;
}

.map-toolbar {
  justify-content: space-between;
}

.map-frame {
  min-height: 360px;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--color-border-light);
  background: #1f2522;
  display: flex;
  align-items: center;
  justify-content: center;
}

.map-frame img {
  width: 100%;
  height: 100%;
  max-height: 620px;
  object-fit: contain;
  display: block;
}

.map-empty,
.empty-state {
  color: var(--color-text-tertiary);
  padding: 48px;
  text-align: center;
}

.settings-panel {
  display: grid;
  gap: 18px;
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.settings-grid label {
  display: grid;
  gap: 8px;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 600;
}

.settings-grid input,
.settings-grid select {
  min-height: 38px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text);
  padding: 0 12px;
  outline: none;
}

@media (max-width: 768px) {
  .ai-shell {
    padding: 16px;
  }

  .ai-header,
  .header-actions,
  .map-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .overview-grid,
  .relation-grid,
  .settings-grid {
    grid-template-columns: 1fr;
  }

  .tabs {
    overflow-x: auto;
  }

  .tabs button {
    flex: 0 0 auto;
  }
}
</style>
