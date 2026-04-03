import { defineStore } from 'pinia'
import { ref, computed, reactive, watch } from 'vue'
import { useAppStore } from './app'
import { useBookshelfStore } from './bookshelf'
import {
  getChapterList,
  getBookContent,
  saveBookProgress,
  setBookSource as apiSetBookSource,
} from '../api/bookshelf'
import {
  getBookmarks,
  saveBookmark,
  deleteBookmark as apiDeleteBookmark,
  deleteBookmarks as apiDeleteBookmarks,
} from '../api/bookmark'
import { getReplaceRules } from '../api/replaceRule'
import type { Book, BookChapter, Bookmark, ReplaceRule } from '../types'
import { getBrowserCachedChapter, setBrowserCachedChapter } from '../utils/browserCache'

const READER_SESSION_KEY = 'reader-last-session'

interface PersistedReaderSession {
  book: Book
  chapters: BookChapter[]
  currentIndex: number
  chapterScrollProgress: number
  updatedAt: number
}

/* ─── Reading config type ─── */
export interface ReadConfig {
  fontSize: number
  fontWeight: number
  fontFamily: string
  lineHeight: number
  paragraphSpacing: number
  fontColor: string
  pageWidth: number
  pageMode: 'auto' | 'mobile'
  readMethod: '上下滑动' | '左右翻页' | '上下滚动' | '上下滚动2'
  animateDuration: number
  autoPageMode: 'pixel' | 'paragraph'
  scrollPixel: number
  pageSpeed: number
  clickAction: 'next' | 'auto' | 'none'
  selectAction: 'popup' | 'ignore'
  chineseMode: 'simplified' | 'traditional'
  specialMode: 'normal' | 'simple'
}

const defaultConfig: ReadConfig = {
  fontSize: 18,
  fontWeight: 400,
  fontFamily: 'system',
  lineHeight: 1.8,
  paragraphSpacing: 0.2,
  fontColor: '',
  pageWidth: 800,
  pageMode: 'auto',
  readMethod: '上下滑动',
  animateDuration: 300,
  autoPageMode: 'pixel',
  scrollPixel: 1,
  pageSpeed: 1000,
  clickAction: 'auto',
  selectAction: 'ignore',
  chineseMode: 'simplified',
  specialMode: 'normal',
}

function loadConfig(): ReadConfig {
  try {
    const saved = localStorage.getItem('readConfig')
    if (saved) return { ...defaultConfig, ...JSON.parse(saved) }
  } catch { /* ignore */ }
  return { ...defaultConfig }
}

/* ─── Theme presets ─── */
export interface ThemePreset {
  name: string
  body: string
  content: string
  fontColor: string
  popup: string
}

export const themePresets: ThemePreset[] = [
  { name: '默认', body: '#f5ede4', content: '#fff9f0', fontColor: '#333', popup: '#fff' },
  { name: '纯白', body: '#ffffff', content: '#ffffff', fontColor: '#333', popup: '#fff' },
  { name: '琥珀', body: '#f5e6ce', content: '#faf0e4', fontColor: '#5b4636', popup: '#faf0e4' },
  { name: '薄荷', body: '#e0f0e8', content: '#eaf5ef', fontColor: '#2d4a3e', popup: '#eaf5ef' },
  { name: '天蓝', body: '#dce8f0', content: '#e8f0f6', fontColor: '#2c3e50', popup: '#e8f0f6' },
  { name: '粉白', body: '#f5e4e8', content: '#faf0f3', fontColor: '#4a2d36', popup: '#faf0f3' },
  { name: '浅灰', body: '#eaeaea', content: '#f5f5f5', fontColor: '#333', popup: '#f5f5f5' },
  { name: '暗灰', body: '#808080', content: '#999', fontColor: '#eee', popup: '#888' },
  { name: '暗夜', body: '#141414', content: '#16213e', fontColor: '#c8c8c8', popup: '#141414' },
]

/* ─── Font presets ─── */
export const fontPresets = [
  { label: '系统', value: 'system', family: '' },
  { label: '黑体', value: 'heiti', family: '"SimHei", "STHeiti", "Heiti SC", sans-serif' },
  { label: '楷体', value: 'kaiti', family: '"KaiTi", "STKaiti", "BiauKai", serif' },
  { label: '宋体', value: 'songti', family: '"SimSun", "STSong", "Songti SC", serif' },
  { label: '仿宋', value: 'fangsong', family: '"FangSong", "STFangsong", serif' },
]

interface TTSOptions {
  onStart?: () => void
  onEnd?: () => void
  onError?: (event?: SpeechSynthesisErrorEvent) => void
}

interface SpeechConfig {
  voiceName: string
  speechRate: number
  speechPitch: number
  stopAfterMinutes: number
}

const defaultSpeechConfig: SpeechConfig = {
  voiceName: '',
  speechRate: 1,
  speechPitch: 1,
  stopAfterMinutes: 0,
}

function loadSpeechConfig(): SpeechConfig {
  try {
    const saved = localStorage.getItem('reader-speechConfig')
    if (saved) return { ...defaultSpeechConfig, ...JSON.parse(saved) }
  } catch { /* ignore */ }
  return { ...defaultSpeechConfig }
}

export const useReaderStore = defineStore('reader', () => {
  const appStore = useAppStore()
  const shelfStore = useBookshelfStore()
  const book = ref<Book | null>(null)
  const chapters = ref<BookChapter[]>([])
  const currentIndex = ref(0)
  const content = ref('')
  const loading = ref(false)
  const chaptersLoading = ref(false)
  const bookmarks = ref<Bookmark[]>([])
  const replaceRules = ref<ReplaceRule[]>([])
  const preloadedContent = ref<Map<number, string>>(new Map()) // index -> content
  const isAutoScrolling = ref(false)
  const chapterScrollProgress = ref(0)

  const currentChapter = computed(() => chapters.value[currentIndex.value] || null)
  const hasNext = computed(() => currentIndex.value < chapters.value.length - 1)
  const hasPrev = computed(() => currentIndex.value > 0)

  const readingProgress = computed(() => {
    if (chapters.value.length === 0) return '0%'
    const progress = ((currentIndex.value + chapterScrollProgress.value) / chapters.value.length) * 100
    const normalized = Math.max(0, Math.min(100, progress))
    return `${normalized < 10 ? normalized.toFixed(1) : Math.round(normalized)}%`
  })

  /* ─── Reading config ─── */
  const config = reactive<ReadConfig>(loadConfig())

  function saveConfig() {
    localStorage.setItem('readConfig', JSON.stringify(config))
  }

  function updateConfig<K extends keyof ReadConfig>(key: K, value: ReadConfig[K]) {
    config[key] = value
    saveConfig()
  }

  function resetConfig() {
    Object.assign(config, defaultConfig)
    saveConfig()
  }

  const chineseConverter = ref<((text: string) => string) | null>(null)
  let chineseLoading: Promise<void> | null = null

  async function ensureChineseConverterLoaded() {
    if (chineseConverter.value || chineseLoading) return chineseLoading || Promise.resolve()
    chineseLoading = import('../../../web/src/plugins/chinese.js')
      .then((module) => {
        chineseConverter.value = module.traditionalized
      })
      .catch(() => {
        chineseConverter.value = null
      })
      .finally(() => {
        chineseLoading = null
      })
    return chineseLoading
  }

  /* ─── Theme ─── */
  const themeIndex = ref(parseInt(localStorage.getItem('reader-themeIndex') || '0'))
  const isNight = computed({
    get: () => appStore.theme === 'dark',
    set: (value: boolean) => {
      appStore.setTheme(value ? 'dark' : 'light')
      localStorage.setItem('reader-isNight', String(value))
    },
  })

  const currentTheme = computed(() => {
    if (isNight.value) return themePresets[themePresets.length - 1]
    return themePresets[themeIndex.value] || themePresets[0]
  })

  function setThemeIndex(idx: number) {
    themeIndex.value = idx
    isNight.value = false
    localStorage.setItem('reader-themeIndex', String(idx))
    localStorage.setItem('reader-isNight', 'false')
  }

  function toggleNight() {
    isNight.value = !isNight.value
  }

  /* ─── Chinese Conversion (OpenCC) ─── */
  /* ─── Content Filtering (Replace Rules) ─── */
  function applyReplaceRules(text: string) {
    if (!text) return ''
    let result = text
    const currentBook = book.value

    function matchRuleScope(rule: ReplaceRule) {
      const scope = (rule.scope || '').trim()
      if (!scope || scope === '*') return true
      if (!currentBook) return false

      if (scope.startsWith('source:')) {
        return scope.slice('source:'.length) === currentBook.origin
      }

      if (scope.startsWith('book:')) {
        return scope.slice('book:'.length) === currentBook.bookUrl
      }

      const scopeParts = scope.split(';')
      if (scopeParts[0] !== '*' && scopeParts[0] !== currentBook.name) {
        return false
      }
      return scopeParts.length === 1 || scopeParts[1] === currentBook.bookUrl
    }

    // Sort by order and apply enabled rules
    const enabledRules = [...replaceRules.value]
      .filter(r => r.isEnabled && matchRuleScope(r))
      .sort((a, b) => a.order - b.order)

    for (const rule of enabledRules) {
      try {
        if (rule.isRegex) {
          const re = new RegExp(rule.pattern, 'gm')
          result = result.replace(re, rule.replacement)
        } else {
          result = result.replaceAll(rule.pattern, rule.replacement)
        }
      } catch (e) {
        console.error(`Failed to apply rule: ${rule.name}`, e)
      }
    }
    return result
  }

  function convertContent(text: string) {
    if (!text || !chineseConverter.value) return text
    return chineseConverter.value(text)
  }

  const displayContent = computed(() => {
    let text = applyReplaceRules(content.value)
    return convertContent(text)
  })

  watch(
    () => config.chineseMode,
    (mode) => {
      if (mode === 'traditional') {
        void ensureChineseConverterLoaded()
      }
    },
    { immediate: true },
  )

  function saveReaderSession() {
    if (!book.value || !chapters.value.length) return
    const payload: PersistedReaderSession = {
      book: book.value,
      chapters: chapters.value,
      currentIndex: currentIndex.value,
      chapterScrollProgress: chapterScrollProgress.value,
      updatedAt: Date.now(),
    }
    localStorage.setItem(READER_SESSION_KEY, JSON.stringify(payload))
  }

  function getPersistedReaderSession(): PersistedReaderSession | null {
    try {
      const raw = localStorage.getItem(READER_SESSION_KEY)
      if (!raw) return null
      return JSON.parse(raw) as PersistedReaderSession
    } catch {
      return null
    }
  }

  async function restorePersistedSession() {
    const session = getPersistedReaderSession()
    if (!session?.book || !session.chapters?.length) return false

    book.value = session.book
    chapters.value = session.chapters

    const nextIndex = Math.max(0, Math.min(session.currentIndex || 0, session.chapters.length - 1))
    try {
      const chapterContent = await fetchChapterContent(nextIndex)
      if (chapterContent == null) return false
      setActiveChapterState(nextIndex, chapterContent, session.chapterScrollProgress || 0)
      return true
    } catch {
      return false
    }
  }

  /* ─── Auto reading ─── */
  const autoReading = ref(false)
  const autoReadingTimer = ref<number | null>(null)

  function toggleAutoReading() {
    isAutoScrolling.value = !isAutoScrolling.value
    autoReading.value = isAutoScrolling.value
  }

  function stopAutoReading() {
    isAutoScrolling.value = false
    autoReading.value = false
    if (autoReadingTimer.value) {
      clearInterval(autoReadingTimer.value)
      autoReadingTimer.value = null
    }
  }

  /* ─── TTS (Text To Speech) ─── */
  const isSpeaking = ref(false)
  const isPaused = ref(false)
  const voiceList = ref<SpeechSynthesisVoice[]>([])
  const speechConfig = reactive<SpeechConfig>(loadSpeechConfig())
  const speechStopAt = ref(0)
  let speechStopTimer: number | null = null
  let synth: SpeechSynthesis | null = typeof window !== 'undefined' ? window.speechSynthesis : null
  let currentUtterance: SpeechSynthesisUtterance | null = null
  let currentTTSOptions: TTSOptions | null = null
  let skipAutoNext = false

  function saveSpeechConfig() {
    localStorage.setItem('reader-speechConfig', JSON.stringify(speechConfig))
  }

  function fetchVoices() {
    if (!synth) return
    voiceList.value = synth.getVoices().slice().sort((a, b) => {
      const aZh = a.lang.startsWith('zh-')
      const bZh = b.lang.startsWith('zh-')
      if (aZh && !bZh) return -1
      if (!aZh && bZh) return 1
      return a.lang.localeCompare(b.lang)
    })
    if (!speechConfig.voiceName && voiceList.value.length > 0) {
      const zhVoice = voiceList.value.find((v) => v.lang.startsWith('zh-'))
      speechConfig.voiceName = (zhVoice || voiceList.value[0]).name
      saveSpeechConfig()
    }
  }

  function setVoiceName(name: string) {
    speechConfig.voiceName = name
    saveSpeechConfig()
  }

  function setSpeechRate(rate: number) {
    speechConfig.speechRate = rate
    saveSpeechConfig()
  }

  function setSpeechPitch(pitch: number) {
    speechConfig.speechPitch = pitch
    saveSpeechConfig()
  }

  function clearSpeechStopTimer(resetConfig = true) {
    if (speechStopTimer) {
      clearTimeout(speechStopTimer)
      speechStopTimer = null
    }
    speechStopAt.value = 0
    if (resetConfig) {
      speechConfig.stopAfterMinutes = 0
      saveSpeechConfig()
    }
  }

  function setSpeechStopTimer(minutes: number) {
    clearSpeechStopTimer(false)
    const normalized = Math.max(0, Math.min(180, Math.round(minutes)))
    speechConfig.stopAfterMinutes = normalized
    saveSpeechConfig()
    if (!normalized) {
      speechStopAt.value = 0
      return
    }
    speechStopAt.value = Date.now() + normalized * 60 * 1000
    speechStopTimer = window.setTimeout(() => {
      stopTTS()
      clearSpeechStopTimer(false)
      speechConfig.stopAfterMinutes = 0
      saveSpeechConfig()
      appStore.showToast('朗读已按定时设置停止', 'success')
    }, normalized * 60 * 1000)
  }

  function startTTS(text?: string, options: TTSOptions = {}) {
    if (!synth) return
    stopTTS(false)

    const rawText = text || content.value.replace(/<[^>]+>/g, '') // Strip HTML
    if (!rawText) return

    if (!voiceList.value.length) {
      fetchVoices()
    }

    currentUtterance = new SpeechSynthesisUtterance(rawText)
    currentTTSOptions = options
    skipAutoNext = false

    const selectedVoice = voiceList.value.find((voice) => voice.name === speechConfig.voiceName)
    currentUtterance.lang = selectedVoice?.lang || 'zh-CN'
    currentUtterance.voice = selectedVoice || null
    currentUtterance.rate = speechConfig.speechRate
    currentUtterance.pitch = speechConfig.speechPitch

    currentUtterance.onstart = () => {
      isSpeaking.value = true
      isPaused.value = false
      currentTTSOptions?.onStart?.()
    }
    currentUtterance.onend = () => {
      isSpeaking.value = false
      isPaused.value = false
      const shouldContinue = !skipAutoNext
      currentUtterance = null
      if (shouldContinue) {
        currentTTSOptions?.onEnd?.()
      }
    }
    currentUtterance.onerror = (event) => {
      isSpeaking.value = false
      isPaused.value = false
      const interrupted = event.error === 'interrupted' || event.error === 'canceled'
      currentUtterance = null
      if (!interrupted) {
        currentTTSOptions?.onError?.(event)
      }
    }

    synth.speak(currentUtterance)
  }

  function pauseTTS() {
    if (!synth) return
    if (synth.speaking && !synth.paused) {
      synth.pause()
      isPaused.value = true
    } else if (synth.paused) {
      synth.resume()
      isPaused.value = false
    }
  }

  function stopTTS(resetCallbacks = true) {
    if (synth) {
      skipAutoNext = true
      synth.cancel()
      isSpeaking.value = false
      isPaused.value = false
      currentUtterance = null
      if (resetCallbacks) {
        currentTTSOptions = null
        clearSpeechStopTimer()
      }
    }
  }

  /* ─── Book / chapter ops ─── */
  async function loadBook(b: Book) {
    book.value = b
    appStore.markBookOpened(b.bookUrl)
    currentIndex.value = b.durChapterIndex || 0
    preloadedContent.value.clear()
    chaptersLoading.value = true
    try {
      chapters.value = await getChapterList({
        bookUrl: b.bookUrl,
        bookSourceUrl: b.origin,
      })
      saveReaderSession()
    } finally {
      chaptersLoading.value = false
    }
  }

  function setActiveChapterState(index: number, chapterContent: string, progress = 0) {
    currentIndex.value = index
    content.value = chapterContent
    chapterScrollProgress.value = Math.max(0, Math.min(1, progress))
    localStorage.setItem('reader-currentIndex', String(index))
    saveReaderSession()
  }

  async function fetchChapterContent(index: number, forceRefresh = false) {
    if (!book.value || !chapters.value[index]) return null

    if (!forceRefresh && preloadedContent.value.has(index)) {
      return preloadedContent.value.get(index) || null
    }

    const chapter = chapters.value[index]

    const browserCached = await getBrowserCachedChapter(book.value.bookUrl, chapter.url).catch(() => null)

    if (!forceRefresh && browserCached) {
      return browserCached
    }

    if (!appStore.isOnline) {
      if (browserCached) {
        return browserCached
      }
      throw new Error('当前处于离线状态，且该章节未缓存到浏览器')
    }

    let chapterContent = ''
    try {
      chapterContent = await getBookContent({
        chapterUrl: chapter.url,
        bookSourceUrl: book.value.origin,
        refresh: forceRefresh ? 1 : 0,
      })
    } catch (error) {
      if (browserCached) {
        appStore.showToast('网络请求失败，已切换到本地缓存章节', 'warning')
        return browserCached
      }
      throw error
    }

    await setBrowserCachedChapter({
      bookUrl: book.value.bookUrl,
      chapterUrl: chapter.url,
      chapterTitle: chapter.title,
      content: chapterContent,
    }).catch(() => undefined)

    return chapterContent
  }

  async function loadChapter(index: number, forceRefresh = false) {
    if (!book.value || !chapters.value[index]) return

    loading.value = true
    try {
      const chapterContent = await fetchChapterContent(index, forceRefresh)
      if (chapterContent == null) return

      setActiveChapterState(index, chapterContent, 0)
      appStore.markChapterRead(book.value.bookUrl, index, chapters.value.length)

      await saveBookProgress({
        bookUrl: book.value.bookUrl,
        index,
      }).catch(() => { })

      setTimeout(() => preloadAroundChapter(index), forceRefresh ? 1500 : 1000)
    } finally {
      loading.value = false
    }
  }

  async function preloadAroundChapter(index: number) {
    if (!book.value) return
    const targets = [index + 1, index + 2, index - 1]
      .filter((target, pos, list) => target >= 0 && target < chapters.value.length && list.indexOf(target) === pos)
    for (const target of targets) {
      await preloadNextChapter(target)
    }
  }

  async function preloadNextChapter(index: number) {
    if (!book.value || index >= chapters.value.length || preloadedContent.value.has(index)) return
    
    // Keep max 3 preloaded chapters
    if (preloadedContent.value.size > 3) {
      const firstKey = preloadedContent.value.keys().next().value
      if (firstKey !== undefined) preloadedContent.value.delete(firstKey)
    }

    try {
      const res = await fetchChapterContent(index)
      if (!res) return
      preloadedContent.value.set(index, res)
      console.log(`[Reader] Preloaded chapter ${index}`)
    } catch { /* ignore */ }
  }

  function normalizeChapterTitle(title?: string) {
    return (title || '')
      .replace(/\s+/g, '')
      .replace(/[：:,.，。！？!?\-—_()（）【】\[\]<>《》'"“”‘’]/g, '')
      .toLowerCase()
  }

  function resolveChapterIndexByTitle(list: BookChapter[], targetTitle?: string, fallbackIndex = 0) {
    if (!list.length) return 0
    const normalizedTarget = normalizeChapterTitle(targetTitle)
    if (!normalizedTarget) {
      return Math.max(0, Math.min(list.length - 1, fallbackIndex))
    }

    const exactIndex = list.findIndex((chapter) => normalizeChapterTitle(chapter.title) === normalizedTarget)
    if (exactIndex >= 0) return exactIndex

    const partialIndex = list.findIndex((chapter) => {
      const title = normalizeChapterTitle(chapter.title)
      return title.includes(normalizedTarget) || normalizedTarget.includes(title)
    })
    if (partialIndex >= 0) return partialIndex

    return Math.max(0, Math.min(list.length - 1, fallbackIndex))
  }

  /* ─── Switch Source ─── */
  async function switchSource(newUrl: string, sourceUrl: string) {
    if (!book.value) return
    const previousChapterTitle = currentChapter.value?.title || book.value.durChapterTitle
    const previousIndex = currentIndex.value
    const previousProgress = chapterScrollProgress.value
    loading.value = true
    try {
      const updatedBook = await apiSetBookSource({
        bookUrl: book.value.bookUrl,
        newUrl,
        bookSourceUrl: sourceUrl,
      })
      if (!updatedBook) return null

      await loadBook(updatedBook)
      const targetIndex = resolveChapterIndexByTitle(
        chapters.value,
        previousChapterTitle,
        typeof updatedBook.durChapterIndex === 'number' ? updatedBook.durChapterIndex : previousIndex,
      )
      await loadChapter(targetIndex)
      setChapterScrollProgress(previousProgress)
      await shelfStore.fetchBooks().catch(() => undefined)
      return updatedBook
    } finally {
      loading.value = false
    }
  }

  async function refreshContent() {
    if (!book.value || !chapters.value[currentIndex.value]) return
    loading.value = true
    try {
      const chapterContent = await fetchChapterContent(currentIndex.value, true)
      if (chapterContent == null) return
      setActiveChapterState(currentIndex.value, chapterContent, chapterScrollProgress.value)
      void preloadAroundChapter(currentIndex.value)
    } finally {
      loading.value = false
    }
  }

  async function refreshChapters() {
    if (!book.value) return
    chaptersLoading.value = true
    try {
      preloadedContent.value.clear()
      chapters.value = await getChapterList({
        bookUrl: book.value.bookUrl,
        bookSourceUrl: book.value.origin,
        refresh: 1,
      })
      const targetIndex = Math.max(0, Math.min(chapters.value.length - 1, currentIndex.value))
      if (chapters.value[targetIndex]) {
        await loadChapter(targetIndex, true)
      }
    } finally {
      chaptersLoading.value = false
    }
  }

  function setChapterScrollProgress(value: number) {
    chapterScrollProgress.value = Math.max(0, Math.min(1, value))
    saveReaderSession()
  }

  async function nextChapter() {
    if (hasNext.value) {
      await loadChapter(currentIndex.value + 1)
    }
  }

  async function prevChapter() {
    if (hasPrev.value) {
      await loadChapter(currentIndex.value - 1)
    }
  }

  /* ─── Replace Rules ─── */
  async function fetchReplaceRules() {
    try {
      replaceRules.value = await getReplaceRules()
    } catch { /* ignore */ }
  }

  /* ─── Bookmarks ─── */
  async function fetchBookmarks() {
    try {
      const all = await getBookmarks()
      // Filter for current book
      if (book.value) {
        bookmarks.value = all.filter(b => b.bookName === book.value?.name && b.bookAuthor === book.value?.author)
      } else {
        bookmarks.value = all
      }
    } catch { /* ignore */ }
  }

  async function addBookmark(pos: number = 0, snippet: string = '') {
    if (!book.value || !currentChapter.value) return
    const b: Bookmark = {
      bookName: book.value.name,
      bookAuthor: book.value.author,
      chapterIndex: currentIndex.value,
      chapterName: currentChapter.value.title,
      chapterPos: pos,
      bookText: snippet || content.value.slice(0, 50).replace(/<[^>]+>/g, ''),
      time: Date.now(),
      content: '',
    }
    await saveBookmark(b)
    await fetchBookmarks()
  }

  async function removeBookmark(b: Bookmark) {
    await apiDeleteBookmark(b)
    await fetchBookmarks()
  }

  async function removeBookmarks(items: Bookmark[]) {
    if (!items.length) return
    await apiDeleteBookmarks(items)
    await fetchBookmarks()
  }

  function clear() {
    book.value = null
    chapters.value = []
    content.value = ''
    currentIndex.value = 0
    chapterScrollProgress.value = 0
    stopAutoReading()
  }

  /* ─── Panel visibility ─── */
  const activePanel = ref<'catalog' | 'settings' | 'bookshelf' | 'source' | 'bookmark' | 'rule' | 'cache' | null>(null)

  function togglePanel(panel: typeof activePanel.value) {
    activePanel.value = activePanel.value === panel ? null : panel
  }

  function closePanel() {
    activePanel.value = null
  }

  return {
    book, chapters, currentIndex, content, loading, chaptersLoading,
    currentChapter, hasNext, hasPrev, readingProgress,
    loadBook, loadChapter, fetchChapterContent, setActiveChapterState, refreshContent, nextChapter, prevChapter, clear,
    chapterScrollProgress, setChapterScrollProgress,
    getPersistedReaderSession, restorePersistedSession,
    config, updateConfig, resetConfig, saveConfig,
    themeIndex, isNight, currentTheme, setThemeIndex, toggleNight,
    autoReading, autoReadingTimer, toggleAutoReading, stopAutoReading,
    activePanel, togglePanel, closePanel,
    bookmarks, fetchBookmarks, addBookmark, removeBookmark, removeBookmarks,
    replaceRules, fetchReplaceRules,
    switchSource, preloadNextChapter, preloadAroundChapter,
    refreshChapters,
    isSpeaking, isPaused, startTTS, pauseTTS, stopTTS,
    voiceList, speechConfig, speechStopAt, fetchVoices, setVoiceName, setSpeechRate, setSpeechPitch, setSpeechStopTimer, clearSpeechStopTimer,
    displayContent,
    isAutoScrolling,
  }
})
