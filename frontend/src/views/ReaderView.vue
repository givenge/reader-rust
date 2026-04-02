<template>
  <div
    class="reader-view"
    :class="{ 'disable-system-callout': disableSystemCallout }"
    :style="{
      background: theme.body,
      color: theme.fontColor,
      fontFamily: currentFontFamily,
      '--color-primary': '#c97f3a'
    }"
    @click="handleBackgroundClick"
    @contextmenu.prevent="handleContextMenu"
  >
    <!-- Left Drawer Panels -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="store.activePanel" class="reader-overlay" @click="store.closePanel()"></div>
      </Transition>
      <Transition name="slide-left">
        <div v-if="store.activePanel" class="reader-drawer" :style="{ background: theme.popup }">
          <ReaderCatalog v-if="store.activePanel === 'catalog'" />
          <ReadSettings v-else-if="store.activePanel === 'settings'" />
          <ReaderBookshelf v-else-if="store.activePanel === 'bookshelf'" />
          <ReaderSource v-else-if="store.activePanel === 'source'" />
          <ReplaceRuleManager v-else-if="store.activePanel === 'rule'" />
          <CacheManager v-else-if="store.activePanel === 'cache'" />
        </div>
      </Transition>
    </Teleport>

    <!-- PC Desktop Toolbars (Always shown) -->
    <ReaderSidebar
      v-if="!isMobile"
      @goHome="goHome"
      @scrollTop="scrollToTop"
      @scrollBottom="scrollToBottom"
    />
    <ReaderToolbar
      v-if="!isMobile"
      :is-speaking="store.isSpeaking"
      :is-paused="store.isPaused"
      @bookmark="toggleBookmark"
      @search="toggleSearch"
      @info="openInfo"
      @tts="handleTTS"
      @prev="prevChapter"
      @next="nextChapter"
      @progress="openCachePanel"
    />

    <!-- Mobile Controls (Click to toggle) -->
    <ReaderMobileControls
      v-if="isMobile"
      :show="showControls || !!store.activePanel"
      @goHome="goHome"
      @scrollTop="scrollToTop"
      @scrollBottom="scrollToBottom"
      @prev="prevChapter"
      @next="nextChapter"
      @bookmark="toggleBookmark"
      @search="openSearch"
      @info="openInfo"
      @tts="handleTTS"
      @progress="openCachePanel"
    />

    <!-- TTS Control Overlay -->
    <Transition name="slide-up">
      <div v-if="store.isSpeaking" class="tts-controls" :style="{ background: theme.popup }">
        <div class="tts-info">正在朗读: {{ store.currentChapter?.title }}</div>
        <div class="tts-btns">
          <button @click="speechPrev">上一段</button>
          <button @click="store.pauseTTS()">{{ store.isPaused ? '恢复' : '暂停' }}</button>
          <button @click="store.stopTTS()">停止</button>
          <button @click="speechNext">下一段</button>
        </div>
        <select class="tts-voice-select" :value="store.speechConfig.voiceName" @change="handleVoiceChange">
          <option value="">系统默认</option>
          <option v-for="voice in store.voiceList" :key="voice.name" :value="voice.name">
            {{ voice.name }} ({{ voice.lang }})
          </option>
        </select>
        <div class="tts-tuning">
          <div class="tts-stepper">
            <span class="tts-label">语速</span>
            <button @click="adjustSpeechRate(-0.1)">-</button>
            <span>{{ store.speechConfig.speechRate.toFixed(1) }}</span>
            <button @click="adjustSpeechRate(0.1)">+</button>
          </div>
          <div class="tts-stepper">
            <span class="tts-label">语调</span>
            <button @click="adjustSpeechPitch(-0.1)">-</button>
            <span>{{ store.speechConfig.speechPitch.toFixed(1) }}</span>
            <button @click="adjustSpeechPitch(0.1)">+</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Main Content Area -->
    <div
      class="reader-scroll-container"
      :class="{ 'horizontal-page-mode': isHorizontalPageMode }"
      ref="scrollContainerRef"
      @scroll="handleScroll"
      @mousedown="stopAutoScroll"
      @touchstart="handleTouchStart"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
      @click="handleGlobalClick"
    >
      <div v-if="store.loading" class="content-loading">
        <div class="loading-spinner"></div>
      </div>

      <article
        v-else-if="!isContinuousMode"
        class="chapter-content"
        :class="{ 'horizontal-page-article': isHorizontalPageMode }"
        :style="{
          maxWidth: isHorizontalPageMode ? 'none' : (config.pageWidth + 'px'),
          fontSize: config.fontSize + 'px',
          fontWeight: config.fontWeight,
          lineHeight: config.lineHeight,
          '--reader-page-width': config.pageWidth + 'px',
          '--reader-side-padding': '24px',
          '--reader-page-step': horizontalPageStepStyle,
        }"
      >
        <div v-if="isHorizontalPageMode" class="horizontal-page-layout">
          <section class="horizontal-content-page">
            <div
              ref="chapterTextRef"
              class="horizontal-pages"
            >
              <section v-for="(page, idx) in horizontalPages" :key="`h-page-${idx}`" class="horizontal-page">
                <div
                  class="chapter-text horizontal-page-content"
                  :style="{
                    '--p-spacing': config.paragraphSpacing + 'em',
                  }"
                  v-html="page"
                ></div>
              </section>
            </div>
          </section>
        </div>

        <div v-else>
          <div class="chapter-title">{{ store.currentChapter?.title || '加载中...' }}</div>

          <div
            ref="chapterTextRef"
            class="chapter-text"
            :style="{
              '--p-spacing': config.paragraphSpacing + 'em',
            }"
            v-html="formattedContent"
          ></div>

          <div class="chapter-footer">
            <button class="next-btn" :disabled="!store.hasNext" @click="nextChapter">
              {{ store.hasNext ? '下一章' : '没有更多了' }}
            </button>
          </div>
        </div>
      </article>

      <Transition name="fade">
        <div v-if="!store.loading && isHorizontalPageMode && isHorizontalAtEnd" class="horizontal-next-floating">
          <button class="next-btn" :disabled="!store.hasNext" @click="nextChapter">
            {{ store.hasNext ? '下一章' : '没有更多了' }}
          </button>
        </div>
      </Transition>

      <div
        v-if="!store.loading && isContinuousMode"
        class="continuous-reading"
        :style="{
          maxWidth: config.pageWidth + 'px',
          fontSize: config.fontSize + 'px',
          fontWeight: config.fontWeight,
          lineHeight: config.lineHeight,
        }"
      >
        <div v-if="continuousLoadingPrev" class="continuous-loading-inline">正在加载上一章...</div>

        <section
          v-for="chapter in continuousChapters"
          :key="chapter.index"
          class="chapter-content continuous-chapter"
          :data-chapter-index="chapter.index"
        >
          <div class="chapter-title">{{ chapter.title }}</div>

          <div
            class="chapter-text"
            data-role="continuous"
            :data-chapter-index="chapter.index"
            :style="{
              '--p-spacing': config.paragraphSpacing + 'em',
            }"
            v-html="chapter.html"
          ></div>

          <div v-if="chapter.index === continuousChapters[continuousChapters.length - 1]?.index" class="chapter-footer">
            <button class="next-btn" :disabled="!store.hasNext" @click="nextChapter">
              {{ store.hasNext ? '继续加载下一章' : '已经到底了' }}
            </button>
          </div>
        </section>

        <div v-if="continuousLoadingNext" class="continuous-loading-inline">正在加载下一章...</div>
      </div>
    </div>



    <!-- Chapter Search Overlay -->
    <Transition name="fade">
      <div v-if="showSearch" class="reader-search-bar" :style="{ background: theme.popup }">
        <input 
          v-model="searchQuery" 
          placeholder="搜索章节内容..." 
          ref="searchInputRef"
          @keyup.enter="nextSearchResult"
        />
        <div class="search-actions">
          <span v-if="searchCount">{{ searchIndex + 1 }} / {{ searchCount }}</span>
          <button @click="prevSearchResult" :disabled="!searchCount">↑</button>
          <button @click="nextSearchResult" :disabled="!searchCount">↓</button>
          <button @click="showSearch = false" class="close-search">×</button>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div
        v-if="selectionMenu.visible"
        class="selection-menu"
        @click.stop
        :style="{
          top: selectionMenu.top + 'px',
          left: selectionMenu.left + 'px',
          background: theme.popup,
          color: theme.fontColor,
        }"
      >
        <div class="selection-menu-text">{{ selectionMenu.text }}</div>
        <div class="selection-menu-actions">
          <button @click="addSelectionBookmark">加入书签</button>
          <button @click="addSelectionReplaceRule('book')">按本书替换</button>
          <button @click="addSelectionReplaceRule('source')">按书源替换</button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useReaderStore, fontPresets } from '../stores/reader'
import { useAppStore } from '../stores/app'
import { saveReplaceRule } from '../api/replaceRule'

import ReaderSidebar from '../components/reader/ReaderSidebar.vue'
import ReaderToolbar from '../components/reader/ReaderToolbar.vue'
import ReaderCatalog from '../components/reader/ReaderCatalog.vue'
import ReadSettings from '../components/reader/ReadSettings.vue'
import ReaderBookshelf from '../components/reader/ReaderBookshelf.vue'
import ReaderSource from '../components/reader/ReaderSource.vue'
import ReplaceRuleManager from '../components/reader/ReplaceRuleManager.vue'
import CacheManager from '../components/reader/CacheManager.vue'
import ReaderMobileControls from '../components/reader/ReaderMobileControls.vue'

const router = useRouter()
const store = useReaderStore()
const appStore = useAppStore()

const config = computed(() => store.config)
const theme = computed(() => store.currentTheme)

const scrollContainerRef = ref<HTMLElement>()
const chapterTextRef = ref<HTMLElement>()
const showControls = ref(false)
const isMobile = ref(false)
let autoScrollId: number | null = null
let autoParagraphTimer: number | null = null
let autoReadingParagraphIndex = -1
let autoReadingProcessing = false
let speechRestartTimer: number | null = null
let isSpeechTransitioning = false
let continuousStateSyncTimer: number | null = null
let suppressContinuousSync = false
let suppressNextTapUntil = 0
let suppressSelectionCloseUntil = 0
let selectionMenuUpdateTimer: number | null = null

interface ContinuousChapterItem {
  index: number
  title: string
  content: string
  html: string
}

const continuousChapters = ref<ContinuousChapterItem[]>([])
const continuousLoadingNext = ref(false)
const continuousLoadingPrev = ref(false)
const isContinuousMode = computed(() =>
  config.value.readMethod === '上下滚动' || config.value.readMethod === '上下滚动2',
)
const isHorizontalPageMode = computed(() => config.value.readMethod === '左右翻页')
const disableSystemCallout = computed(() => isMobile.value && config.value.selectAction === 'popup')
const isHorizontalAtEnd = ref(false)
const selectionMenu = ref({
  visible: false,
  text: '',
  top: 0,
  left: 0,
})
const activeSelectionText = ref('')
const touchState = ref({
  startX: 0,
  startY: 0,
  startAt: 0,
  moving: false,
  horizontalLocked: false,
})
const horizontalPageIndex = ref(0)
const horizontalPageStep = ref(1)
const horizontalPageStepStyle = computed(() => `${Math.max(1, horizontalPageStep.value)}px`)
const horizontalPages = ref<string[]>([])

function checkMedia() {
  isMobile.value = window.innerWidth <= 768
  window.setTimeout(() => {
    updateHorizontalMetrics()
    if (isHorizontalPageMode.value) {
      rebuildHorizontalPages()
    }
  }, 0)
}

const currentFontFamily = computed(() => {
  const preset = fontPresets.find(p => p.value === config.value.fontFamily)
  return preset ? preset.family : ''
})

function formatChapterHtml(rawText: string) {
  if (!rawText) return ''
  let text = rawText

  if (showSearch.value && searchQuery.value) {
    try {
      const regex = new RegExp(`(${searchQuery.value})`, 'gi')
      text = text.replace(regex, '<mark class="search-highlight">$1</mark>')
    } catch { /* invalid regex */ }
  }

  return text
    .split(/\n/)
    .filter((line: string) => line.trim())
    .map((line: string) => `<p style="margin-top: 0; margin-bottom: ${config.value.paragraphSpacing}em; text-indent: 2em;">${line.trim()}</p>`)
    .join('')
}

const formattedContent = computed(() => formatChapterHtml(store.displayContent || ''))
function escapeHtml(input: string) {
  return input
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
}

function splitTextByPunctuation(text: string) {
  const result: string[] = []
  let current = ''
  const breakers = new Set(['。', '！', '？', '；', '，', '、', '.', '!', '?', ';', ','])
  for (const ch of text) {
    current += ch
    if (breakers.has(ch) && current.trim().length >= 10) {
      result.push(current.trim())
      current = ''
    }
  }
  if (current.trim()) {
    result.push(current.trim())
  }
  return result.length ? result : [text]
}

function parseInlineStyle(styleText: string) {
  const entries = styleText
    .split(';')
    .map((item) => item.trim())
    .filter(Boolean)
    .map((item) => {
      const idx = item.indexOf(':')
      if (idx <= 0) return null
      const key = item.slice(0, idx).trim().toLowerCase()
      const value = item.slice(idx + 1).trim()
      if (!key || !value) return null
      return [key, value] as const
    })
    .filter(Boolean) as Array<readonly [string, string]>
  return Object.fromEntries(entries) as Record<string, string>
}

function buildInlineStyle(styleObj: Record<string, string>) {
  return Object.entries(styleObj)
    .map(([key, value]) => `${key}: ${value}`)
    .join('; ')
}

function splitParagraphHtml(paragraphHtml: string) {
  const wrapper = document.createElement('div')
  wrapper.innerHTML = paragraphHtml
  const paragraph = wrapper.querySelector('p')
  if (!paragraph) return [paragraphHtml]
  const text = (paragraph.textContent || '').trim()
  if (!text) return [paragraphHtml]
  const style = paragraph.getAttribute('style') || ''
  const baseStyle = parseInlineStyle(style)
  const segments = splitTextByPunctuation(text)
  return segments.map((segment, idx) => {
    const styleObj = { ...baseStyle }
    if (idx > 0) {
      delete styleObj['text-indent']
    }
    if (idx < segments.length - 1) {
      delete styleObj['margin-bottom']
    }
    const styleText = buildInlineStyle(styleObj)
    const stylePart = styleText ? ` style="${styleText}"` : ''
    return `<p${stylePart}>${escapeHtml(segment)}</p>`
  })
}

function parseParagraphHtml(paragraphHtml: string) {
  const wrapper = document.createElement('div')
  wrapper.innerHTML = paragraphHtml
  const paragraph = wrapper.querySelector('p')
  if (!paragraph) return null
  return {
    style: paragraph.getAttribute('style') || '',
    text: (paragraph.textContent || '').trim(),
  }
}

function buildParagraphHtml(style: string, text: string) {
  const stylePart = style ? ` style="${style}"` : ''
  return `<p${stylePart}>${escapeHtml(text)}</p>`
}

function buildHorizontalParagraphs() {
  const root = document.createElement('div')
  root.innerHTML = formattedContent.value
  const paragraphs = Array.from(root.querySelectorAll('p')).map((node) => node.outerHTML)
  return paragraphs
}

async function rebuildHorizontalPages() {
  if (!isHorizontalPageMode.value) {
    horizontalPages.value = []
    return
  }
  await nextTick()
  const container = scrollContainerRef.value
  if (!container) return

  updateHorizontalMetrics()

  const sidePadding = 24
  const innerWidth = Math.max(120, horizontalPageStep.value - sidePadding * 2)
  const pageHeight = Math.max(200, window.innerHeight - 48)
  const title = (store.currentChapter?.title || '加载中...').trim()
  const titleHtml = `<h1 class="horizontal-flow-title">${escapeHtml(title)}</h1>`
  const paragraphs = buildHorizontalParagraphs()

  const measurer = document.createElement('div')
  measurer.style.position = 'fixed'
  measurer.style.left = '-99999px'
  measurer.style.top = '0'
  measurer.style.width = `${innerWidth}px`
  measurer.style.height = `${pageHeight}px`
  measurer.style.overflow = 'hidden'
  measurer.style.visibility = 'hidden'
  measurer.style.pointerEvents = 'none'
  measurer.style.boxSizing = 'border-box'
  measurer.style.fontSize = `${config.value.fontSize}px`
  measurer.style.fontWeight = String(config.value.fontWeight)
  measurer.style.lineHeight = String(config.value.lineHeight)
  measurer.style.fontFamily = currentFontFamily.value || ''
  document.body.appendChild(measurer)

  const pages: string[] = []
  let currentParts: string[] = [titleHtml]

  const overflows = (parts: string[]) => {
    measurer.innerHTML = parts.join('')
    return measurer.scrollHeight > measurer.clientHeight
  }

  const flushPage = () => {
    if (!currentParts.length) return
    pages.push(currentParts.join(''))
    currentParts = []
  }

  const appendBlock = (blockHtml: string) => {
    const withBlock = [...currentParts, blockHtml]
    if (!overflows(withBlock)) {
      currentParts = withBlock
      return
    }

    const pieces = splitParagraphHtml(blockHtml)
    for (const piece of pieces) {
      let pending = piece
      while (pending) {
        const nextPiece = [...currentParts, pending]
        if (!overflows(nextPiece)) {
          currentParts = nextPiece
          pending = ''
          continue
        }

        const parsed = parseParagraphHtml(pending)
        const canSplit = parsed && parsed.text.length > 1
        if (canSplit) {
          const { style, text } = parsed
          const styleObj = parseInlineStyle(style)
          const splitPieceStyle = { ...styleObj }
          delete splitPieceStyle['margin-bottom']
          const splitPieceStyleText = buildInlineStyle(splitPieceStyle)
          let left = 1
          let right = text.length
          let fitCount = 0
          while (left <= right) {
            const mid = Math.floor((left + right) / 2)
            const tryStyle = mid < text.length ? splitPieceStyleText : style
            const tryHtml = buildParagraphHtml(tryStyle, text.slice(0, mid))
            const tryParts = [...currentParts, tryHtml]
            if (!overflows(tryParts)) {
              fitCount = mid
              left = mid + 1
            } else {
              right = mid - 1
            }
          }

          if (fitCount > 0) {
            const fitStyle = fitCount < text.length ? splitPieceStyleText : style
            const fitHtml = buildParagraphHtml(fitStyle, text.slice(0, fitCount))
            currentParts = [...currentParts, fitHtml]
            flushPage()
            const remain = text.slice(fitCount)
            pending = remain ? buildParagraphHtml(style, remain) : ''
            continue
          }
        }

        if (currentParts.length) {
          flushPage()
          continue
        }

        pages.push(pending)
        pending = ''
      }
    }
  }

  for (const paragraph of paragraphs) {
    appendBlock(paragraph)
  }

  if (currentParts.length) {
    flushPage()
  }

  if (!pages.length) {
    pages.push(titleHtml)
  } else if (pages.length === 1 && !pages[0].includes('horizontal-flow-title')) {
    // Keep title on first page when content existed but the first page was flushed early.
    pages[0] = `${titleHtml}${pages[0]}`
  }

  document.body.removeChild(measurer)
  horizontalPages.value = pages
  horizontalPageIndex.value = Math.min(horizontalPageIndex.value, pages.length - 1)

  const targetLeft = horizontalPageIndex.value * horizontalPageStep.value
  container.scrollTo({ left: targetLeft, behavior: 'auto' })
  updateHorizontalEndState()
}

async function buildContinuousChapter(index: number, forceRefresh = false) {
  const chapter = store.chapters[index]
  if (!chapter) return null
  const chapterContent = await store.fetchChapterContent(index, forceRefresh)
  if (chapterContent == null) return null
  return {
    index,
    title: chapter.title,
    content: chapterContent,
    html: formatChapterHtml(chapterContent),
  } satisfies ContinuousChapterItem
}

function syncContinuousChapterHtml() {
  continuousChapters.value = continuousChapters.value.map((chapter) => ({
    ...chapter,
    html: formatChapterHtml(chapter.content),
  }))
}

function getContinuousChapter(index: number) {
  return continuousChapters.value.find((chapter) => chapter.index === index) || null
}

function setContinuousActiveChapter(index: number, chapterContent: string, progress: number) {
  suppressContinuousSync = true
  store.setActiveChapterState(index, chapterContent, progress)
  if (continuousStateSyncTimer) {
    clearTimeout(continuousStateSyncTimer)
  }
  continuousStateSyncTimer = window.setTimeout(() => {
    suppressContinuousSync = false
  }, 0)
}

async function initializeContinuousChapters(targetIndex = store.currentIndex, smooth = false) {
  if (!isContinuousMode.value || !store.chapters[targetIndex]) return

  const current = await buildContinuousChapter(targetIndex)
  if (!current) return

  const list: ContinuousChapterItem[] = [current]
  const next = await buildContinuousChapter(targetIndex + 1)
  if (next) {
    list.push(next)
  }
  continuousChapters.value = list
  setContinuousActiveChapter(targetIndex, current.content, 0)

  await nextTick()
  scrollToContinuousChapter(targetIndex, smooth)
}

async function syncContinuousToStoreState() {
  if (!isContinuousMode.value || suppressContinuousSync || store.loading || !store.chapters[store.currentIndex]) return

  const current = getContinuousChapter(store.currentIndex)
  if (current) {
    if (current.content !== store.content) {
      current.content = store.content
      current.html = formatChapterHtml(store.content)
    }
    await nextTick()
    scrollToContinuousChapter(store.currentIndex, false)
    return
  }

  await initializeContinuousChapters(store.currentIndex, false)
}

async function loadContinuousNext() {
  if (continuousLoadingNext.value || !continuousChapters.value.length) return
  const last = continuousChapters.value[continuousChapters.value.length - 1]
  const nextIndex = last.index + 1
  if (nextIndex >= store.chapters.length) return

  continuousLoadingNext.value = true
  try {
    const next = await buildContinuousChapter(nextIndex)
    if (next && !getContinuousChapter(next.index)) {
      continuousChapters.value = [...continuousChapters.value, next]
    }
  } finally {
    continuousLoadingNext.value = false
  }
}

async function loadContinuousPrev() {
  if (continuousLoadingPrev.value || !continuousChapters.value.length) return
  const first = continuousChapters.value[0]
  const prevIndex = first.index - 1
  if (prevIndex < 0) return

  const container = scrollContainerRef.value
  const previousHeight = container?.scrollHeight || 0
  const previousTop = container?.scrollTop || 0

  continuousLoadingPrev.value = true
  try {
    const prev = await buildContinuousChapter(prevIndex)
    if (prev && !getContinuousChapter(prev.index)) {
      continuousChapters.value = [prev, ...continuousChapters.value]
      await nextTick()
      if (container) {
        const heightDiff = container.scrollHeight - previousHeight
        container.scrollTop = previousTop + heightDiff
      }
    }
  } finally {
    continuousLoadingPrev.value = false
  }
}

async function ensureContinuousChapterLoaded(index: number) {
  if (getContinuousChapter(index)) return
  if (!continuousChapters.value.length) {
    await initializeContinuousChapters(index, false)
    return
  }

  while (continuousChapters.value[0] && index < continuousChapters.value[0].index) {
    await loadContinuousPrev()
  }

  while (continuousChapters.value[continuousChapters.value.length - 1] &&
    index > continuousChapters.value[continuousChapters.value.length - 1].index) {
    await loadContinuousNext()
  }
}

function getContinuousSections() {
  const container = scrollContainerRef.value
  if (!container) return [] as HTMLElement[]
  return Array.from(container.querySelectorAll('.continuous-chapter')) as HTMLElement[]
}

function scrollToContinuousChapter(index: number, smooth = true) {
  const container = scrollContainerRef.value
  if (!container) return
  const section = container.querySelector(`.continuous-chapter[data-chapter-index="${index}"]`) as HTMLElement | null
  if (!section) return
  container.scrollTo({
    top: Math.max(0, section.offsetTop),
    behavior: smooth ? 'smooth' : 'auto',
  })
}

function pageForward() {
  const container = scrollContainerRef.value
  if (!container) return
  if (isHorizontalPageMode.value) {
    const pageWidth = Math.max(1, horizontalPageStep.value || container.clientWidth)
    const maxPage = Math.max(0, horizontalPages.value.length - 1)
    if (horizontalPageIndex.value >= maxPage) {
      nextChapter()
      return
    }
    horizontalPageIndex.value = Math.min(maxPage, horizontalPageIndex.value + 1)
    const targetLeft = horizontalPageIndex.value * pageWidth
    container.scrollTo({ left: targetLeft, behavior: 'auto' })
    updateHorizontalEndState()
    return
  }
  const step = container.clientHeight * 0.88
  if (container.scrollTop + container.clientHeight >= container.scrollHeight - 10) {
    nextChapter()
    return
  }
  container.scrollBy({ top: step, behavior: 'smooth' })
}

function pageBackward() {
  const container = scrollContainerRef.value
  if (!container) return
  if (isHorizontalPageMode.value) {
    const pageWidth = Math.max(1, horizontalPageStep.value || container.clientWidth)
    if (horizontalPageIndex.value <= 0) {
      prevChapter()
      return
    }
    horizontalPageIndex.value = Math.max(0, horizontalPageIndex.value - 1)
    const targetLeft = horizontalPageIndex.value * pageWidth
    container.scrollTo({ left: targetLeft, behavior: 'auto' })
    updateHorizontalEndState()
    return
  }
  const step = container.clientHeight * 0.88
  if (container.scrollTop <= 10) {
    prevChapter()
    return
  }
  container.scrollBy({ top: -step, behavior: 'smooth' })
}

function updateHorizontalEndState() {
  const container = scrollContainerRef.value
  if (!container || !isHorizontalPageMode.value) {
    isHorizontalAtEnd.value = false
    return
  }
  updateHorizontalMetrics()
  const maxPage = Math.max(0, horizontalPages.value.length - 1)
  isHorizontalAtEnd.value = horizontalPageIndex.value >= maxPage
}

function alignHorizontalToNearestPage() {
  const container = scrollContainerRef.value
  if (!container || !isHorizontalPageMode.value || touchState.value.moving) return
  updateHorizontalMetrics()
  const pageWidth = Math.max(1, horizontalPageStep.value || container.clientWidth)
  const maxPage = Math.max(0, horizontalPages.value.length - 1)
  const nearestPage = Math.round(container.scrollLeft / pageWidth)
  horizontalPageIndex.value = Math.max(0, Math.min(maxPage, nearestPage))
  const targetLeft = horizontalPageIndex.value * pageWidth
  if (Math.abs(container.scrollLeft - targetLeft) > 2) {
    container.scrollTo({ left: targetLeft, behavior: 'auto' })
  }
}

function updateHorizontalMetrics() {
  const container = scrollContainerRef.value
  if (!container || !isHorizontalPageMode.value) return
  horizontalPageStep.value = Math.max(1, container.clientWidth)
}

// Navigation
function goHome() {
  router.push('/')
}

async function prevChapter() {
  const targetIndex = store.currentIndex - 1
  if (targetIndex < 0) return

  if (!isContinuousMode.value) {
    await store.prevChapter()
    scrollToTop()
    return
  }

  await ensureContinuousChapterLoaded(targetIndex)
  const chapter = getContinuousChapter(targetIndex)
  if (!chapter) return
  setContinuousActiveChapter(targetIndex, chapter.content, 0)
  await nextTick()
  scrollToContinuousChapter(targetIndex)
}

async function nextChapter() {
  const targetIndex = store.currentIndex + 1
  if (targetIndex >= store.chapters.length) return

  if (!isContinuousMode.value) {
    await store.nextChapter()
    scrollToTop()
    return
  }

  await ensureContinuousChapterLoaded(targetIndex)
  const chapter = getContinuousChapter(targetIndex)
  if (!chapter) return
  setContinuousActiveChapter(targetIndex, chapter.content, 0)
  await nextTick()
  scrollToContinuousChapter(targetIndex)
}

function scrollToTop() {
  if (scrollContainerRef.value) {
    if (isHorizontalPageMode.value) {
      scrollContainerRef.value.scrollTo({ left: 0, behavior: 'smooth' })
    } else {
      scrollContainerRef.value.scrollTo({ top: 0, behavior: 'smooth' })
    }
  }
}

function scrollToBottom() {
  if (scrollContainerRef.value) {
    scrollContainerRef.value.scrollTo({ top: scrollContainerRef.value.scrollHeight, behavior: 'smooth' })
  }
}

function getFilteredParagraphs() {
  const roots = isContinuousMode.value
    ? Array.from(scrollContainerRef.value?.querySelectorAll('.chapter-text[data-role="continuous"]') || []) as HTMLElement[]
    : (chapterTextRef.value ? [chapterTextRef.value] : [])
  if (!roots.length) return [] as HTMLElement[]
  const allElements = roots.flatMap((root) => Array.from(root.querySelectorAll('p')) as HTMLElement[])
  const list: HTMLElement[] = []
  let lastText = ''
  allElements.forEach((el) => {
    const text = el.innerText.trim()
    if (text && text !== lastText) {
      list.push(el)
      lastText = text
    }
  })
  return list
}

function getCurrentParagraph() {
  const reading = chapterTextRef.value?.querySelector('.reading') as HTMLElement | null
  if (reading) return reading

  const container = scrollContainerRef.value
  if (!container) return null

  const list = getFilteredParagraphs()
  for (const paragraph of list) {
    const top = paragraph.offsetTop - container.scrollTop
    const bottom = top + paragraph.offsetHeight
    if (bottom > 40) {
      return paragraph
    }
  }

  return list[0] || null
}

function getPrevParagraph() {
  const current = getCurrentParagraph()
  const list = getFilteredParagraphs()
  const index = current ? list.indexOf(current) : -1
  if (index > 0) return list[index - 1]
  return null
}

function getNextParagraph() {
  const current = getCurrentParagraph()
  const list = getFilteredParagraphs()
  const index = current ? list.indexOf(current) : -1
  if (index >= 0 && index < list.length - 1) return list[index + 1]
  return null
}

function clearReadingClass() {
  scrollContainerRef.value?.querySelectorAll('.reading').forEach((el) => el.classList.remove('reading'))
}

function showParagraph(paragraph: HTMLElement | null, smooth = true) {
  const container = scrollContainerRef.value
  if (!container || !paragraph) return

  const targetTop = Math.max(0, paragraph.offsetTop - 24)
  container.scrollTo({
    top: targetTop,
    behavior: smooth ? 'smooth' : 'auto',
  })
}

function markReadingParagraph(paragraph: HTMLElement | null) {
  clearReadingClass()
  if (paragraph) {
    paragraph.classList.add('reading')
  }
}

// Click behavior
function handleBackgroundClick(e: Event) {
  // If clicked directly on the reader-view wrapper, toggle controls
  if ((e.target as HTMLElement).classList.contains('reader-view')) {
    showControls.value = false
    stopAutoScroll()
  }
}

function handleContextMenu(event: Event) {
  if (!disableSystemCallout.value) return
  event.preventDefault()
}

function handleGlobalClick(e: MouseEvent) {
  if (showControls.value && !store.activePanel) {
    showControls.value = false
    return
  }
  if (store.activePanel) return
  if (Date.now() < suppressNextTapUntil) return
  if (Date.now() < suppressSelectionCloseUntil) return
  if (selectionMenu.value.visible) {
    hideSelectionMenu()
    return
  }
  if (window.getSelection?.()?.toString().trim()) return

  const target = e.target as HTMLElement | null
  if (target?.closest('.tts-controls, .reader-search-bar, .selection-menu')) return
  
  if (isHorizontalPageMode.value && isMobile.value) {
    const x = e.clientX / window.innerWidth
    if (x < 0.3) {
      clickZoneAction('prev')
    } else if (x > 0.7) {
      clickZoneAction('next')
    } else {
      clickZoneAction('menu')
    }
  } else {
    const y = e.clientY / window.innerHeight
    if (y < 0.3) {
      clickZoneAction('prev')
    } else if (y > 0.7) {
      clickZoneAction('next')
    } else {
      clickZoneAction('menu')
    }
  }
}

function clickZoneAction(zone: 'prev' | 'menu' | 'next') {
  if (zone === 'menu') {
    if (isMobile.value) {
      showControls.value = !showControls.value
    }
    return
  }
  
  if (config.value.clickAction === 'none') return
  
  const container = scrollContainerRef.value
  if (!container) return
  
  if (isHorizontalPageMode.value) {
    if (zone === 'next') pageForward()
    else pageBackward()
    return
  }

  const h = container.clientHeight
  const delta = h * 0.8 // Page scroll amount

  if (config.value.clickAction === 'next') {
    pageForward()
    return
  }
  
  if (zone === 'next') {
    if (container.scrollTop + h >= container.scrollHeight - 10) {
      if (config.value.clickAction === 'auto') nextChapter()
    } else {
      container.scrollBy({ top: delta, behavior: 'smooth' })
    }
  } else {
    if (container.scrollTop === 0) {
      if (config.value.clickAction === 'auto') prevChapter()
    } else {
      container.scrollBy({ top: -delta, behavior: 'smooth' })
    }
  }
}

function handleScroll() {
  hideSelectionMenu()
  const container = scrollContainerRef.value
  if (container && isContinuousMode.value && continuousChapters.value.length) {
    const sections = getContinuousSections()
    if (sections.length) {
      const anchorLine = container.scrollTop + container.clientHeight * 0.3
      let activeSection = sections[0]
      for (const section of sections) {
        if (section.offsetTop <= anchorLine) {
          activeSection = section
        } else {
          break
        }
      }

      const activeIndex = Number(activeSection.dataset.chapterIndex || 0)
      const activeChapter = getContinuousChapter(activeIndex)
      const nextSection = sections[sections.indexOf(activeSection) + 1] || null
      const sectionRange = Math.max(
        1,
        (nextSection ? nextSection.offsetTop : container.scrollHeight) - activeSection.offsetTop,
      )
      const progress = Math.max(0, Math.min(1, (container.scrollTop - activeSection.offsetTop) / sectionRange))
      if (activeChapter) {
        if (store.currentIndex !== activeIndex || store.content !== activeChapter.content) {
          setContinuousActiveChapter(activeIndex, activeChapter.content, progress)
        } else {
          store.setChapterScrollProgress(progress)
        }
      }
    }

    if (container.scrollTop < 240) {
      loadContinuousPrev()
    }
    if (container.scrollHeight - (container.scrollTop + container.clientHeight) < 480) {
      loadContinuousNext()
    }
  } else if (container) {
    const maxScroll = isHorizontalPageMode.value
      ? Math.max(1, container.scrollWidth - container.clientWidth)
      : Math.max(1, container.scrollHeight - container.clientHeight)
    const progress = isHorizontalPageMode.value
      ? (container.scrollWidth <= container.clientWidth ? 1 : container.scrollLeft / maxScroll)
      : (container.scrollHeight <= container.clientHeight ? 1 : container.scrollTop / maxScroll)
    store.setChapterScrollProgress(progress)
    if (isHorizontalPageMode.value) {
      updateHorizontalMetrics()
      const maxPage = Math.max(0, horizontalPages.value.length - 1)
      horizontalPageIndex.value = Math.max(0, Math.min(maxPage, Math.round(container.scrollLeft / Math.max(1, horizontalPageStep.value))))
      updateHorizontalEndState()
    }
  }
  if (showControls.value && !store.activePanel) {
    showControls.value = false
  }
}

function handleTouchStart(event: TouchEvent) {
  stopAutoScroll()
  hideSelectionMenu()
  const touch = event.touches[0]
  if (!touch) return
  touchState.value = {
    startX: touch.clientX,
    startY: touch.clientY,
    startAt: Date.now(),
    moving: true,
    horizontalLocked: false,
  }
}

function handleTouchMove(event: TouchEvent) {
  if (!isMobile.value || config.value.readMethod !== '左右翻页' || !touchState.value.moving) return
  const selectedText = window.getSelection?.()?.toString().trim()
  if (selectedText) return
  // Keep long-press text selection gestures available on mobile.
  if (Date.now() - touchState.value.startAt > 220) return
  const touch = event.touches[0]
  if (!touch) return
  const deltaX = touch.clientX - touchState.value.startX
  const deltaY = touch.clientY - touchState.value.startY
  if (Math.abs(deltaX) > 12 && Math.abs(deltaX) > Math.abs(deltaY)) {
    touchState.value.horizontalLocked = true
    event.preventDefault()
  }
}

function handleTouchEnd(event: TouchEvent) {
  if (!isMobile.value || config.value.readMethod !== '左右翻页' || !touchState.value.moving) {
    touchState.value.moving = false
    return
  }
  const touchDuration = Date.now() - touchState.value.startAt
  const selectedText = window.getSelection?.()?.toString().trim()
  if (selectedText) {
    suppressNextTapUntil = Date.now() + 900
    touchState.value.moving = false
    scheduleSelectionMenuUpdate(260)
    return
  }
  const touch = event.changedTouches[0]
  if (!touch) {
    touchState.value.moving = false
    return
  }
  const deltaX = touch.clientX - touchState.value.startX
  const deltaY = touch.clientY - touchState.value.startY
  let didPageTurn = false
  if (Math.abs(deltaX) > 18 && Math.abs(deltaX) > Math.abs(deltaY)) {
    suppressNextTapUntil = Date.now() + 350
    if (deltaX < 0) {
      pageForward()
    } else {
      pageBackward()
    }
    didPageTurn = true
  }
  touchState.value.moving = false
  if (!didPageTurn && touchDuration > 260) {
    // Long-press should be reserved for native text selection, not page action.
    suppressNextTapUntil = Date.now() + 900
    scheduleSelectionMenuUpdate(260)
    return
  }
  if (!didPageTurn) {
    window.setTimeout(() => {
      alignHorizontalToNearestPage()
    }, 120)
  }
  scheduleSelectionMenuUpdate(260)
}

function openCachePanel() {
  store.togglePanel('cache')
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  const activeElement = document.activeElement as HTMLElement | null
  const tagName = activeElement?.tagName?.toLowerCase()
  if (tagName === 'input' || tagName === 'textarea' || tagName === 'select' || activeElement?.isContentEditable) {
    return
  }
  if (store.activePanel) return
  if (selectionMenu.value.visible && e.key === 'Escape') {
    hideSelectionMenu()
    return
  }
  const container = scrollContainerRef.value
  if (!container) return
  
  const h = container.clientHeight
  
  switch (e.key) {
    case ' ':
    case 'Space':
      e.preventDefault()
      pageForward()
      break
    case 'ArrowDown':
      e.preventDefault()
      container.scrollBy({ top: h * 0.8, behavior: 'smooth' })
      break
    case 'ArrowUp':
      e.preventDefault()
      container.scrollBy({ top: -(h * 0.8), behavior: 'smooth' })
      break
    case 'ArrowRight':
      e.preventDefault()
      nextChapter()
      break
    case 'ArrowLeft':
      e.preventDefault()
      prevChapter()
      break
    case 'Escape':
      if (showControls.value) showControls.value = false
      else showControls.value = true
      break
  }
}

// Search Logic
const showSearch = ref(false)
const searchQuery = ref('')
const searchIndex = ref(0)
const searchInputRef = ref<HTMLInputElement>()
const searchCount = computed(() => {
  if (!searchQuery.value || !store.content) return 0
  const matches = store.content.match(new RegExp(searchQuery.value, 'gi'))
  return matches ? matches.length : 0
})

function toggleSearch() {
  showSearch.value = !showSearch.value
  if (showSearch.value) {
    nextTick(() => searchInputRef.value?.focus())
  } else {
    searchQuery.value = ''
  }
}

function nextSearchResult() {
  if (!searchCount.value) return
  searchIndex.value = (searchIndex.value + 1) % searchCount.value
  scrollToMatch()
}

function prevSearchResult() {
  if (!searchCount.value) return
  searchIndex.value = (searchIndex.value - 1 + searchCount.value) % searchCount.value
  scrollToMatch()
}

function scrollToMatch() {
  nextTick(() => {
    const matches = document.querySelectorAll('.search-highlight')
    if (matches[searchIndex.value]) {
      matches[searchIndex.value].scrollIntoView({ block: 'center', behavior: 'smooth' })
      // Highlight current
      matches.forEach(m => m.classList.remove('current-match'))
      matches[searchIndex.value].classList.add('current-match')
    }
  })
}

// Toolbar actions
async function toggleBookmark() {
  const container = scrollContainerRef.value
  const pos = container ? container.scrollTop : 0
  await store.addBookmark(pos)
  appStore.showToast('已添加书签', 'success')
}

function handleTTS() {
  if (store.isSpeaking) {
    store.pauseTTS()
  } else {
    startSpeech()
  }
}

// Auto scroll logic
function startAutoScroll() {
  if (config.value.autoPageMode === 'paragraph') {
    if (autoParagraphTimer) return
    runAutoParagraph()
    return
  }
  if (autoScrollId) return
  runAutoScroll()
}

function stopAutoScroll() {
  store.isAutoScrolling = false
  autoReadingParagraphIndex = -1
  autoReadingProcessing = false
  if (autoScrollId) {
    cancelAnimationFrame(autoScrollId)
    autoScrollId = null
  }
  if (autoParagraphTimer) {
    clearTimeout(autoParagraphTimer)
    autoParagraphTimer = null
  }
  if (!store.isSpeaking) {
    clearReadingClass()
  }
}

watch(() => store.isAutoScrolling, (val) => {
  store.autoReading = val
  if (val) startAutoScroll()
  else stopAutoScroll()
})




function runAutoScroll() {
  if (!store.isAutoScrolling || !scrollContainerRef.value) return
  
  const container = scrollContainerRef.value
  // Speed mapping: config.pageSpeed (usually 1000-5000ms per screen?) 
  // Let's use a simpler pixels-per-frame based on speed (1-10)
  // pageSpeed is 1000 in defaultConfig, let's treat it as scale
  const speed = Math.max(1, config.value.scrollPixel) * (config.value.pageSpeed / 1000) * 0.5
  
  container.scrollTop += speed
  
  if (container.scrollTop + container.clientHeight >= container.scrollHeight - 2) {
     if (config.value.clickAction === 'auto' && store.hasNext) {
        nextChapter()
     } else {
        stopAutoScroll()
     }
  } else {
    autoScrollId = requestAnimationFrame(runAutoScroll)
  }
}

function runAutoParagraph() {
  if (!store.isAutoScrolling) return
  if (autoReadingProcessing) return

  const list = getFilteredParagraphs()
  if (!list.length) return

  autoReadingProcessing = true

  if (autoReadingParagraphIndex < 0) {
    const current = getCurrentParagraph()
    autoReadingParagraphIndex = current ? Math.max(0, list.indexOf(current)) : 0
  }

  if (autoReadingParagraphIndex >= list.length) {
    autoReadingParagraphIndex = -1
    autoReadingProcessing = false
    if (store.hasNext) {
      nextChapter().then(() => {
        window.setTimeout(() => {
          if (store.isAutoScrolling && config.value.autoPageMode === 'paragraph') {
            runAutoParagraph()
          }
        }, 300)
      })
    } else {
      stopAutoScroll()
    }
    return
  }

  const current = list[autoReadingParagraphIndex]
  markReadingParagraph(current)
  showParagraph(current)

  const estimatedLineCount = Math.max(1, Math.ceil(current.offsetHeight / (config.value.fontSize * config.value.lineHeight)))
  const delayTime = Math.max(300, config.value.pageSpeed * estimatedLineCount)

  autoReadingProcessing = false
  autoParagraphTimer = window.setTimeout(() => {
    autoReadingParagraphIndex += 1
    runAutoParagraph()
  }, delayTime)
}

function restartSpeechFromCurrentParagraph() {
  if (isSpeechTransitioning) return
  isSpeechTransitioning = true
  store.stopTTS(false)
  if (speechRestartTimer) {
    clearTimeout(speechRestartTimer)
  }
  speechRestartTimer = window.setTimeout(() => {
    isSpeechTransitioning = false
    startSpeech()
  }, 150)
}

function startSpeech(paragraph?: HTMLElement | null) {
  const current = paragraph || getCurrentParagraph()
  if (!current?.innerText.trim()) {
    speechNext()
    return
  }

  markReadingParagraph(current)
  showParagraph(current)
  store.startTTS(current.innerText, {
    onEnd: () => {
      speechNext()
    },
    onError: () => {
      clearReadingClass()
    },
  })
}

function speechPrev() {
  const prev = getPrevParagraph()
  if (prev) {
    restartSpeechTarget(prev)
    return
  }
  if (!store.hasPrev) {
    store.stopTTS()
    return
  }
  store.stopTTS(false)
  prevChapter().then(() => {
    window.setTimeout(() => {
      const list = getFilteredParagraphs()
      restartSpeechTarget(list[list.length - 1] || null)
    }, 120)
  })
}

function speechNext() {
  const next = getNextParagraph()
  if (next) {
    restartSpeechTarget(next)
    return
  }
  if (!store.hasNext) {
    store.stopTTS()
    clearReadingClass()
    return
  }
  store.stopTTS(false)
  nextChapter().then(() => {
    window.setTimeout(() => {
      restartSpeechTarget(getFilteredParagraphs()[0] || null)
    }, 120)
  })
}

function restartSpeechTarget(paragraph: HTMLElement | null) {
  if (!paragraph) {
    store.stopTTS()
    return
  }
  if (isSpeechTransitioning) return
  isSpeechTransitioning = true
  store.stopTTS(false)
  if (speechRestartTimer) {
    clearTimeout(speechRestartTimer)
  }
  speechRestartTimer = window.setTimeout(() => {
    isSpeechTransitioning = false
    startSpeech(paragraph)
  }, 150)
}

function changeVoice(name: string) {
  store.setVoiceName(name)
  if (store.isSpeaking && !store.isPaused) {
    restartSpeechFromCurrentParagraph()
  }
}

function adjustSpeechRate(delta: number) {
  const next = Math.max(0.5, Math.min(3, parseFloat((store.speechConfig.speechRate + delta).toFixed(1))))
  store.setSpeechRate(next)
  if (store.isSpeaking && !store.isPaused) {
    restartSpeechFromCurrentParagraph()
  }
}

function adjustSpeechPitch(delta: number) {
  const next = Math.max(0.5, Math.min(2, parseFloat((store.speechConfig.speechPitch + delta).toFixed(1))))
  store.setSpeechPitch(next)
  if (store.isSpeaking && !store.isPaused) {
    restartSpeechFromCurrentParagraph()
  }
}

function handleVoiceChange(event: Event) {
  const target = event.target as HTMLSelectElement | null
  changeVoice(target?.value || '')
}

function openSearch() { toggleSearch() }
function openInfo() { appStore.showToast('详情功能开发中', 'warning') }

function hideSelectionMenu() {
  selectionMenu.value.visible = false
}

function scheduleSelectionMenuUpdate(delay = 220) {
  if (selectionMenuUpdateTimer) {
    clearTimeout(selectionMenuUpdateTimer)
  }
  selectionMenuUpdateTimer = window.setTimeout(() => {
    updateSelectionMenu()
  }, delay)
}

function handleMouseUpSelection() {
  scheduleSelectionMenuUpdate(120)
}

function handleTouchEndSelection() {
  scheduleSelectionMenuUpdate(260)
}

function updateSelectionMenu() {
  if (config.value.selectAction !== 'popup') {
    hideSelectionMenu()
    return
  }

  const selection = window.getSelection?.()
  const text = selection?.toString().trim() || ''
  if (!selection || selection.rangeCount === 0 || !text) {
    return
  }

  const container = scrollContainerRef.value
  const range = selection.getRangeAt(0)
  const commonAncestor = range.commonAncestorContainer
  const targetNode = commonAncestor.nodeType === Node.TEXT_NODE ? commonAncestor.parentElement : commonAncestor as HTMLElement | null
  if (!container || !targetNode || !container.contains(targetNode)) {
    hideSelectionMenu()
    return
  }

  const rect = range.getBoundingClientRect()
  suppressSelectionCloseUntil = Date.now() + 250
  activeSelectionText.value = text
  selectionMenu.value = {
    visible: true,
    text: text.length > 48 ? `${text.slice(0, 48)}...` : text,
    top: Math.max(20, rect.top - 56),
    left: Math.min(window.innerWidth - 240, Math.max(16, rect.left + rect.width / 2 - 110)),
  }
}

async function addSelectionBookmark() {
  const selection = window.getSelection?.()
  const text = activeSelectionText.value || selection?.toString().trim() || ''
  if (!text) return
  try {
    const pos = scrollContainerRef.value?.scrollTop || 0
    await store.addBookmark(pos, text)
    selection?.removeAllRanges()
    activeSelectionText.value = ''
    hideSelectionMenu()
    appStore.showToast('已加入书签', 'success')
  } catch {
    appStore.showToast('加入书签失败', 'error')
  }
}

async function addSelectionReplaceRule(mode: 'book' | 'source') {
  const selection = window.getSelection?.()
  const text = activeSelectionText.value || selection?.toString().trim() || ''
  if (!text) return
  if (!store.book) return

  try {
    const scope = mode === 'source'
      ? `source:${store.book.origin}`
      : `book:${store.book.bookUrl}`
    await saveReplaceRule({
      id: 0,
      name: `${mode === 'source' ? '书源替换' : '本书替换'} ${store.replaceRules.length + 1}`,
      pattern: text,
      replacement: '',
      scope,
      isEnabled: true,
      isRegex: false,
      order: store.replaceRules.length + 1,
    })
    await store.fetchReplaceRules()
    selection?.removeAllRanges()
    activeSelectionText.value = ''
    hideSelectionMenu()
    appStore.showToast(mode === 'source' ? '已加入书源替换规则' : '已加入本书替换规则', 'success')
  } catch {
    appStore.showToast('加入替换规则失败', 'error')
  }
}

onMounted(async () => {
  if (!store.book) {
    router.replace('/')
    return
  }
  window.addEventListener('keydown', handleKeydown)
  document.addEventListener('mouseup', handleMouseUpSelection)
  document.addEventListener('touchend', handleTouchEndSelection)
  checkMedia()
  window.addEventListener('resize', checkMedia)
  store.fetchVoices()
  if (typeof window !== 'undefined' && window.speechSynthesis) {
    window.speechSynthesis.onvoiceschanged = () => store.fetchVoices()
  }
  await store.fetchBookmarks()
  updateHorizontalMetrics()
  await rebuildHorizontalPages()
  if (isContinuousMode.value) {
    await initializeContinuousChapters(store.currentIndex, false)
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('mouseup', handleMouseUpSelection)
  document.removeEventListener('touchend', handleTouchEndSelection)
  window.removeEventListener('resize', checkMedia)
  if (speechRestartTimer) clearTimeout(speechRestartTimer)
  if (continuousStateSyncTimer) clearTimeout(continuousStateSyncTimer)
  if (selectionMenuUpdateTimer) clearTimeout(selectionMenuUpdateTimer)
  stopAutoScroll()
  store.stopTTS()
  if (typeof window !== 'undefined' && window.speechSynthesis) {
    window.speechSynthesis.onvoiceschanged = null
  }
  store.closePanel()
})

watch(() => config.value.autoPageMode, () => {
  if (!store.isAutoScrolling) return
  stopAutoScroll()
  store.isAutoScrolling = true
  startAutoScroll()
})

watch(() => config.value.readMethod, async () => {
  activeSelectionText.value = ''
  hideSelectionMenu()
  if (isContinuousMode.value) {
    await initializeContinuousChapters(store.currentIndex, false)
  } else {
    continuousChapters.value = []
    await nextTick()
    if (scrollContainerRef.value) {
      scrollContainerRef.value.scrollTo({ top: 0, left: 0, behavior: 'auto' })
    }
  }
  horizontalPageIndex.value = 0
  if (isHorizontalPageMode.value && scrollContainerRef.value) {
    updateHorizontalMetrics()
    scrollContainerRef.value.scrollTo({ left: 0, behavior: 'auto' })
  }
  await rebuildHorizontalPages()
  updateHorizontalEndState()
})

watch(() => store.currentIndex, () => {
  if (!isHorizontalPageMode.value) return
  horizontalPageIndex.value = 0
  if (scrollContainerRef.value) {
    updateHorizontalMetrics()
    scrollContainerRef.value.scrollTo({ left: 0, behavior: 'auto' })
  }
  rebuildHorizontalPages()
  updateHorizontalEndState()
})

watch(
  [() => store.content, () => config.value.fontSize, () => config.value.fontWeight, () => config.value.lineHeight, () => config.value.paragraphSpacing, showSearch, searchQuery],
  () => {
    if (isHorizontalPageMode.value) {
      horizontalPageIndex.value = 0
      rebuildHorizontalPages()
    }
  },
)

watch(() => store.currentIndex, async () => {
  autoReadingParagraphIndex = -1
  if (!store.isSpeaking) {
    clearReadingClass()
  }
  if (isContinuousMode.value && !suppressContinuousSync) {
    await syncContinuousToStoreState()
  }
})

watch(() => store.content, () => {
  autoReadingParagraphIndex = -1
  if (isContinuousMode.value) {
    const current = getContinuousChapter(store.currentIndex)
    if (current) {
      current.content = store.content
      current.html = formatChapterHtml(store.content)
    }
  }
  if (store.isAutoScrolling && config.value.autoPageMode === 'paragraph') {
    if (autoParagraphTimer) {
      clearTimeout(autoParagraphTimer)
      autoParagraphTimer = null
    }
    window.setTimeout(() => {
      if (store.isAutoScrolling && config.value.autoPageMode === 'paragraph') {
        runAutoParagraph()
      }
    }, 100)
  }
})

watch([showSearch, searchQuery, () => config.value.paragraphSpacing], () => {
  if (isContinuousMode.value) {
    syncContinuousChapterHtml()
  }
})

watch(() => config.value.selectAction, (value) => {
  if (value !== 'popup') {
    activeSelectionText.value = ''
    hideSelectionMenu()
    window.getSelection?.()?.removeAllRanges()
  }
})

watch(() => store.isSpeaking, (speaking) => {
  if (!speaking && !store.isAutoScrolling) {
    clearReadingClass()
  }
})
</script>

<style scoped>
.reader-view {
  height: 100vh;
  width: 100vw;
  display: flex;
  position: relative;
  overflow: hidden;
  transition: background 0.3s, color 0.3s;
}

.reader-view.disable-system-callout .chapter-text,
.reader-view.disable-system-callout .horizontal-page-content,
.reader-view.disable-system-callout .continuous-reading {
  -webkit-touch-callout: none;
}

.reader-scroll-container {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  position: relative;
  scroll-behavior: smooth;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.reader-scroll-container.horizontal-page-mode {
  overflow-x: hidden;
  overflow-y: hidden;
  touch-action: auto;
  overscroll-behavior: none;
}

/* Hide scrollbar */
.reader-scroll-container::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}
.reader-scroll-container::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.1);
  border-radius: 4px;
}
.reader-view[style*="background: #1a1a2e"] .reader-scroll-container::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.1);
}

.content-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(0,0,0,0.1);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}
.reader-view[style*="background: #1a1a2e"] .loading-spinner {
  border-color: rgba(255,255,255,0.1);
  border-top-color: var(--color-primary);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.chapter-content {
  margin: 0 auto;
  padding: 80px 24px;
  min-height: 100%;
  transition: all 0.3s ease;
}

.chapter-content.horizontal-page-article {
  margin: 0;
  height: 100%;
  min-height: 100%;
  width: max-content;
  min-width: 100%;
  padding: 0;
}

.horizontal-page-layout {
  width: max-content;
  min-width: var(--reader-page-step);
  height: 100%;
}

.horizontal-content-page {
  width: max-content;
  min-width: var(--reader-page-step);
  height: 100%;
  min-height: 100%;
  padding: 0;
  box-sizing: border-box;
}

.horizontal-pages {
  display: flex;
  width: max-content;
  height: 100%;
  min-height: 100%;
}

.horizontal-page {
  width: var(--reader-page-step);
  min-width: var(--reader-page-step);
  height: 100%;
  min-height: 100%;
  padding: 24px var(--reader-side-padding);
  box-sizing: border-box;
}

.continuous-reading {
  margin: 0 auto;
  padding: 32px 0 80px;
}

.continuous-chapter {
  min-height: auto;
  padding-top: 48px;
  padding-bottom: 24px;
}

.chapter-title {
  font-size: 1.6em;
  font-weight: 700;
  margin-bottom: 2em;
  text-align: center;
  line-height: 1.4;
}

.chapter-text {
  word-break: normal;
  overflow-wrap: anywhere;
  text-align: justify;
  user-select: text;
  -webkit-user-select: text;
  -webkit-touch-callout: default;
}

.horizontal-page-content {
  height: 100%;
  overflow: hidden;
}

:deep(.horizontal-page-content .horizontal-flow-title) {
  margin: 0 0 1em 0;
  font-size: 1.5em;
  line-height: 1.35;
  font-weight: 700;
  text-align: center;
  break-inside: avoid;
}

:deep(.horizontal-page-content p:first-child) {
  margin-top: 0 !important;
}

:deep(.horizontal-page-content p:last-child) {
  margin-bottom: 0 !important;
}

:deep(.chapter-text p.reading) {
  background: rgba(201, 127, 58, 0.12);
  border-radius: 10px;
  box-shadow: inset 0 0 0 1px rgba(201, 127, 58, 0.18);
}

:deep(.chapter-text p) {
  user-select: text;
  -webkit-user-select: text;
}

.chapter-footer {
  margin-top: 60px;
  text-align: center;
  padding-bottom: 40px;
}

.horizontal-next-floating {
  position: absolute;
  left: 50%;
  bottom: 20px;
  transform: translateX(-50%);
  z-index: 12;
  pointer-events: none;
}

.horizontal-next-floating .next-btn {
  pointer-events: auto;
  background: rgba(255, 255, 255, 0.75);
  backdrop-filter: blur(6px);
}

.continuous-loading-inline {
  text-align: center;
  padding: 18px 24px;
  opacity: 0.6;
  font-size: 13px;
}

.next-btn {
  padding: 12px 36px;
  border-radius: 30px;
  background: transparent;
  border: 1px solid currentColor;
  color: inherit;
  font-size: 14px;
  opacity: 0.6;
  cursor: pointer;
  transition: all 0.2s;
}

.next-btn:hover:not(:disabled) {
  opacity: 1;
  background: rgba(0,0,0,0.05);
}

.next-btn:disabled {
  opacity: 0.2;
  cursor: not-allowed;
}



/* Slide Drawer Overlay */
.reader-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  z-index: 40;
}

.reader-drawer {
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  width: min(340px, 85vw);
  z-index: 50;
  box-shadow: 4px 0 24px rgba(0,0,0,0.15);
  transition: background 0.3s;
}

.tts-controls {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: var(--radius-full);
  box-shadow: 0 4px 20px rgba(0,0,0,0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  z-index: 30;
  min-width: 280px;
}

.tts-info {
  font-size: 12px;
  opacity: 0.7;
}

.tts-btns {
  display: flex;
  gap: 16px;
}

.tts-btns button {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 4px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.tts-voice-select {
  width: 100%;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(0, 0, 0, 0.08);
  background: rgba(255, 255, 255, 0.65);
  color: inherit;
}

.tts-tuning {
  width: 100%;
  display: flex;
  gap: 10px;
}

.tts-stepper {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.04);
  font-size: 12px;
}

.tts-stepper button {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: var(--color-primary);
  color: #fff;
  cursor: pointer;
}

.tts-label {
  opacity: 0.7;
}

.reader-search-bar {
  position: fixed;
  top: 24px;
  right: 80px;
  padding: 8px 16px;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0,0,0,0.15);
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 30;
  border: 1px solid rgba(0,0,0,0.05);
}

.selection-menu {
  position: fixed;
  z-index: 60;
  min-width: 220px;
  max-width: min(320px, calc(100vw - 32px));
  border-radius: 14px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.18);
  border: 1px solid rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

.selection-menu-text {
  padding: 12px 14px 8px;
  font-size: 13px;
  line-height: 1.5;
  opacity: 0.72;
  word-break: break-all;
}

.selection-menu-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  padding: 0 12px 12px;
}

.selection-menu-actions button {
  border: none;
  border-radius: 10px;
  padding: 10px 12px;
  background: var(--color-primary);
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}

.selection-menu-actions button:first-child {
  grid-column: 1 / -1;
}

.reader-search-bar input {
  background: transparent;
  border: none;
  outline: none;
  color: inherit;
  font-size: 14px;
  width: 180px;
}

.search-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  opacity: 0.8;
}

.search-actions button {
  background: rgba(0,0,0,0.05);
  border: none;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  cursor: pointer;
  color: inherit;
}

.close-search {
  font-size: 18px !important;
}

:deep(.search-highlight) {
  background: yellow;
  color: black;
  border-radius: 2px;
}

:deep(.search-highlight.current-match) {
  background: orange;
}

@media (max-width: 768px) {
  .tts-controls {
    bottom: 80px;
  }
  .reader-search-bar {
    top: auto;
    bottom: 80px;
    right: 16px;
    left: 16px;
  }
  .reader-search-bar input {
    width: auto;
    flex: 1;
  }
}

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.slide-left-enter-active, .slide-left-leave-active { transition: transform 0.35s cubic-bezier(0.2, 0.8, 0.2, 1); }
.slide-left-enter-from, .slide-left-leave-to { transform: translateX(-100%); }

.fade-slide-right-enter-active, .fade-slide-right-leave-active { transition: all 0.3s ease; }
.fade-slide-right-enter-from, .fade-slide-right-leave-to { transform: translateX(-20px); opacity: 0; }

.fade-slide-left-enter-active, .fade-slide-left-leave-active { transition: all 0.3s ease; }
.fade-slide-left-enter-from, .fade-slide-left-leave-to { transform: translateX(20px); opacity: 0; }
</style>
