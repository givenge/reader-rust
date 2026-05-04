import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { deleteAiBookMemory, getAiBookMemory, saveAiBookMemory } from '../api/aiBook'
import type { AiBookMemory, Book, BookChapter } from '../types'
import { useAppStore } from './app'
import { getAiBookConfig, saveAiBookConfig } from '../utils/aiBookConfig'
import type { AiBookConfig } from '../types'
import {
  applyMapToMemory,
  createEmptyAiBookMemory,
  requestAiBookMapImage,
  requestAiBookMemoryUpdate,
  shouldRunAiBookAutoUpdate,
  uploadGeneratedMap,
} from '../utils/aiBookGeneration'

type GenerationPhase = 'idle' | 'loading' | 'text' | 'map' | 'saving' | 'error'

export const useAiBookStore = defineStore('aiBook', () => {
  const appStore = useAppStore()
  const memory = ref<AiBookMemory | null>(null)
  const loading = ref(false)
  const phase = ref<GenerationPhase>('idle')
  const statusText = ref('')
  const updatingChapterKeys = new Set<string>()

  const username = computed(() => appStore.userInfo?.username || 'default')
  const config = ref<AiBookConfig>(getAiBookConfig(username.value))
  const isBusy = computed(() => loading.value || phase.value !== 'idle')

  function refreshConfig() {
    config.value = getAiBookConfig(username.value)
    return config.value
  }

  function persistConfig(next: AiBookConfig) {
    config.value = saveAiBookConfig(username.value, next)
    return config.value
  }

  async function load(book: Book) {
    loading.value = true
    try {
      const saved = await getAiBookMemory(book.bookUrl)
      memory.value = saved || createEmptyAiBookMemory(book)
      return memory.value
    } finally {
      loading.value = false
    }
  }

  async function save(next: AiBookMemory) {
    phase.value = 'saving'
    statusText.value = '保存 AI 资料...'
    try {
      memory.value = await saveAiBookMemory(next)
      return memory.value
    } finally {
      phase.value = 'idle'
      statusText.value = ''
    }
  }

  async function setEnabled(book: Book, enabled: boolean) {
    const current = memory.value?.bookUrl === book.bookUrl ? memory.value : await load(book)
    return save({
      ...current,
      bookUrl: book.bookUrl,
      bookName: book.name,
      author: book.author,
      enabled,
      updatedAt: Date.now(),
    })
  }

  async function reset(book: Book) {
    await deleteAiBookMemory(book.bookUrl)
    memory.value = createEmptyAiBookMemory(book)
    phase.value = 'idle'
    statusText.value = ''
    return memory.value
  }

  async function autoUpdateCompletedChapter(params: {
    book: Book
    chapter: BookChapter
    chapterContent: string
  }) {
    const current = memory.value?.bookUrl === params.book.bookUrl
      ? memory.value
      : await getAiBookMemory(params.book.bookUrl).catch(() => null)
    if (!current?.enabled) return null

    const currentConfig = refreshConfig()
    if (!shouldRunAiBookAutoUpdate(current, params.chapter.index, currentConfig)) {
      return current
    }
    return runChapterUpdate({ ...params, current, allowSkip: true })
  }

  async function runChapterUpdate(params: {
    book: Book
    chapter: BookChapter
    chapterContent: string
    current?: AiBookMemory
    allowSkip?: boolean
  }) {
    const currentConfig = refreshConfig()
    const current = params.current || (memory.value?.bookUrl === params.book.bookUrl ? memory.value : await load(params.book))
    if (params.allowSkip && !shouldRunAiBookAutoUpdate(current, params.chapter.index, currentConfig)) {
      return current
    }

    const key = `${params.book.bookUrl}::${params.chapter.index}`
    if (updatingChapterKeys.has(key)) return current
    updatingChapterKeys.add(key)
    phase.value = 'text'
    statusText.value = `更新 ${params.chapter.title} 的 AI 资料...`

    try {
      const update = await requestAiBookMemoryUpdate({
        config: currentConfig,
        book: params.book,
        chapter: params.chapter,
        chapterContent: params.chapterContent,
        memory: current,
      })
      let next = await saveAiBookMemory(update.memory)
      memory.value = next

      if (update.shouldRegenerateMap && update.mapPrompt) {
        next = await redrawMap(params.book, update.mapPrompt, params.chapter.index, next)
      }

      phase.value = 'idle'
      statusText.value = ''
      return next
    } catch (error) {
      const message = (error as Error).message || 'AI 资料更新失败'
      phase.value = 'error'
      statusText.value = message
      const failed: AiBookMemory = {
        ...current,
        bookUrl: params.book.bookUrl,
        bookName: params.book.name,
        author: params.book.author,
        lastError: message,
        updatedAt: Date.now(),
      }
      memory.value = await saveAiBookMemory(failed).catch(() => failed)
      return memory.value
    } finally {
      updatingChapterKeys.delete(key)
      if (phase.value === 'error') {
        window.setTimeout(() => {
          if (phase.value === 'error') {
            phase.value = 'idle'
            statusText.value = ''
          }
        }, 3000)
      }
    }
  }

  async function redrawMap(book: Book, prompt?: string, sourceChapterIndex?: number, currentMemory?: AiBookMemory) {
    const currentConfig = refreshConfig()
    const current = currentMemory || memory.value || await load(book)
    const resolvedPrompt = prompt || current.map?.prompt || buildFallbackMapPrompt(current, book)
    phase.value = 'map'
    statusText.value = '生成世界地图...'
    try {
      const image = await requestAiBookMapImage({
        config: currentConfig,
        prompt: resolvedPrompt,
      })
      const imageUrl = await uploadGeneratedMap({
        b64Json: image.b64Json,
        imageUrl: image.imageUrl,
        filename: `${Date.now()}-${slugify(book.name || 'map')}.png`,
      })
      const next = applyMapToMemory(current, {
        imageUrl,
        prompt: resolvedPrompt,
        updatedAt: Date.now(),
        sourceChapterIndex,
      })
      memory.value = await saveAiBookMemory(next)
      phase.value = 'idle'
      statusText.value = ''
      return memory.value
    } catch (error) {
      const message = (error as Error).message || '地图生成失败'
      const failed: AiBookMemory = {
        ...current,
        mapDirty: true,
        lastError: message,
        updatedAt: Date.now(),
      }
      memory.value = await saveAiBookMemory(failed).catch(() => failed)
      phase.value = 'error'
      statusText.value = message
      return memory.value
    }
  }

  return {
    memory,
    loading,
    phase,
    statusText,
    isBusy,
    config,
    refreshConfig,
    persistConfig,
    load,
    save,
    setEnabled,
    reset,
    autoUpdateCompletedChapter,
    runChapterUpdate,
    redrawMap,
  }
})

function buildFallbackMapPrompt(memory: AiBookMemory, book: Book) {
  const locations = memory.locations.map((item) => `${item.name}: ${item.description}`).join('\n')
  return [
    `为小说《${book.name}》绘制一张不剧透的世界地图。`,
    '只包含已读进度中出现的地点和势力范围。',
    locations || memory.summary || '保留未知区域，以卷轴地图风格呈现。',
  ].join('\n')
}

function slugify(value: string) {
  const slug = value
    .toLowerCase()
    .replace(/[^a-z0-9\u4e00-\u9fa5]+/gi, '-')
    .replace(/^-+|-+$/g, '')
  return slug || 'map'
}
