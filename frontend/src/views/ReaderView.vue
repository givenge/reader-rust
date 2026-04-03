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
        <div v-if="store.activePanel" class="reader-drawer" :style="{ background: chromeTheme.popup }">
          <ReaderCatalog v-if="store.activePanel === 'catalog' || store.activePanel === 'bookmark'" :initial-tab="store.activePanel === 'bookmark' ? 'bookmarks' : 'chapters'" />
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

    <ReaderTtsPanel
      :show="showTTSPanel"
      :theme="chromeTheme"
      :chapter-title="store.currentChapter?.title"
      :is-paused="store.isPaused"
      :voices="store.voiceList"
      :voice-name="store.speechConfig.voiceName"
      :rate="store.speechConfig.speechRate"
      :pitch="store.speechConfig.speechPitch"
      :stop-after-minutes="store.speechConfig.stopAfterMinutes"
      :timer-text="speechTimerText"
      @close="showTTSPanel = false"
      @prev="speechPrev"
      @pause="store.pauseTTS()"
      @stop="store.stopTTS()"
      @next="speechNext"
      @voice-change="changeVoice"
      @rate-change="adjustSpeechRate"
      @pitch-change="adjustSpeechPitch"
      @timer-change="setSpeechTimer"
    />

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

      <div v-else-if="offlineBannerText" class="offline-banner">
        {{ offlineBannerText }}
      </div>

      <article
        v-if="!store.loading && !isContinuousMode"
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



    <ReaderSearchPanel
      :show="showSearch"
      :theme="chromeTheme"
      :query="searchQuery"
      :results="searchResults"
      :active-index="searchIndex"
      :count="searchCount"
      :status="bookSearchStatus"
      @close="closeSearch"
      @search="runSearch"
      @next="nextSearchResult"
      @prev="prevSearchResult"
      @update:query="searchQuery = $event"
      @jump="jumpToSearchResult"
    />

    <Transition name="fade">
      <div
        v-if="selectionMenu.visible"
        class="selection-menu"
        @click.stop
        :style="{
          top: selectionMenu.top + 'px',
          left: selectionMenu.left + 'px',
          background: chromeTheme.popup,
          color: chromeTheme.fontColor,
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

    <BookDetailModal
      v-model="showBookInfo"
      :book="bookInfoBook"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick, defineAsyncComponent } from 'vue'
import { useRouter } from 'vue-router'
import { useReaderStore, fontPresets } from '../stores/reader'
import { useAppStore } from '../stores/app'
import { getBookInfo } from '../api/bookshelf'
import { applySystemTheme } from '../utils/systemUi'
import { countBrowserBookCache } from '../utils/browserCache'
import type { Book } from '../types'

import ReaderSidebar from '../components/reader/ReaderSidebar.vue'
import ReaderToolbar from '../components/reader/ReaderToolbar.vue'
import ReaderMobileControls from '../components/reader/ReaderMobileControls.vue'
import { useReaderSearch } from '../composables/useReaderSearch'
import { useReaderSelection } from '../composables/useReaderSelection'
import { useHorizontalPaging } from '../composables/useHorizontalPaging'
import { useContinuousReading } from '../composables/useContinuousReading'
import { useReaderAutoPlayback } from '../composables/useReaderAutoPlayback'

const ReaderCatalog = defineAsyncComponent(() => import('../components/reader/ReaderCatalog.vue'))
const ReadSettings = defineAsyncComponent(() => import('../components/reader/ReadSettings.vue'))
const ReaderBookshelf = defineAsyncComponent(() => import('../components/reader/ReaderBookshelf.vue'))
const ReaderSource = defineAsyncComponent(() => import('../components/reader/ReaderSource.vue'))
const ReplaceRuleManager = defineAsyncComponent(() => import('../components/reader/ReplaceRuleManager.vue'))
const CacheManager = defineAsyncComponent(() => import('../components/reader/CacheManager.vue'))
const BookDetailModal = defineAsyncComponent(() => import('../components/BookDetailModal.vue'))
const ReaderTtsPanel = defineAsyncComponent(() => import('../components/reader/ReaderTtsPanel.vue'))
const ReaderSearchPanel = defineAsyncComponent(() => import('../components/reader/ReaderSearchPanel.vue'))

const router = useRouter()
const store = useReaderStore()
const appStore = useAppStore()

const config = computed(() => store.config)
const theme = computed(() => store.currentTheme)
const chromeTheme = computed(() => {
  if (store.isNight || appStore.theme === 'dark') {
    return {
      ...store.currentTheme,
      popup: 'var(--color-bg-elevated)',
      fontColor: 'var(--color-text)',
    }
  }
  return store.currentTheme
})

const scrollContainerRef = ref<HTMLElement>()
const chapterTextRef = ref<HTMLElement>()
const showControls = ref(false)
const isMobile = ref(false)
let speechTimerTicker: number | null = null
let suppressNextTapUntil = 0
const isContinuousMode = computed(() =>
  config.value.readMethod === '上下滚动' || config.value.readMethod === '上下滚动2',
)
const isHorizontalPageMode = computed(() => config.value.readMethod === '左右翻页')
const disableSystemCallout = computed(() => {
  const ua = typeof navigator !== 'undefined' ? navigator.userAgent : ''
  const isIOS = /iPhone|iPad|iPod/i.test(ua)
  return isIOS && isMobile.value && config.value.selectAction === 'popup'
})
const touchState = ref({
  startX: 0,
  startY: 0,
  startAt: 0,
  moving: false,
  horizontalLocked: false,
})
const showBookInfo = ref(false)
const bookInfoBook = ref<Book | null>(null)
const showTTSPanel = ref(false)
const offlineCachedCount = ref(0)
const speechTimerNow = ref(Date.now())
const speechTimerText = computed(() => {
  if (!store.speechStopAt) return ''
  const remainMs = store.speechStopAt - speechTimerNow.value
  if (remainMs <= 0) return ''
  const totalMinutes = Math.ceil(remainMs / 60000)
  if (totalMinutes >= 60) {
    const hours = Math.floor(totalMinutes / 60)
    const minutes = totalMinutes % 60
    return minutes ? `${hours}小时${minutes}分钟后停止` : `${hours}小时后停止`
  }
  return `${totalMinutes}分钟后停止`
})
const {
  showSearch,
  searchQuery,
  searchResults,
  searchIndex,
  searchCount,
  bookSearchStatus,
  toggleSearch,
  openSearch,
  closeSearch,
  runSearch,
  nextSearchResult,
  prevSearchResult,
  jumpToSearchResult,
  handleContentUpdated,
  handlePresentationUpdated,
} = useReaderSearch(store)
const {
  selectionMenu,
  suppressSelectionCloseUntil,
  hideSelectionMenu,
  scheduleSelectionMenuUpdate,
  handleMouseUpSelection,
  handleTouchEndSelection,
  handleSelectionChange,
  addSelectionBookmark,
  addSelectionReplaceRule,
  clearSelectionState,
  disposeSelection,
} = useReaderSelection(
  store,
  appStore,
  computed(() => ({ selectAction: config.value.selectAction })),
  scrollContainerRef,
)

const offlineBannerText = computed(() => {
  if (appStore.isOnline) return ''
  if (offlineCachedCount.value > 0) {
    return `离线模式：当前书已缓存 ${offlineCachedCount.value} 章，可继续阅读已缓存章节`
  }
  return '离线模式：当前书尚未缓存到浏览器，未缓存章节将无法打开'
})

async function refreshOfflineCacheState() {
  if (!store.book) {
    offlineCachedCount.value = 0
    return
  }
  offlineCachedCount.value = await countBrowserBookCache(store.book.bookUrl).catch(() => 0)
}

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

const {
  horizontalPageIndex,
  horizontalPageStep,
  horizontalPageStepStyle,
  horizontalPages,
  isHorizontalAtEnd,
  rebuildHorizontalPages,
  updateHorizontalMetrics,
  updateHorizontalEndState,
  alignHorizontalToNearestPage,
  resetHorizontalPagePosition,
} = useHorizontalPaging(
  store,
  computed(() => ({
    fontSize: config.value.fontSize,
    fontWeight: config.value.fontWeight,
    lineHeight: config.value.lineHeight,
  })),
  currentFontFamily,
  formattedContent,
  isHorizontalPageMode,
  scrollContainerRef,
)
const {
  continuousChapters,
  continuousLoadingNext,
  continuousLoadingPrev,
  suppressContinuousSync,
  syncContinuousChapterHtml,
  getContinuousChapter,
  setContinuousActiveChapter,
  initializeContinuousChapters,
  syncContinuousToStoreState,
  loadContinuousNext,
  loadContinuousPrev,
  ensureContinuousChapterLoaded,
  getContinuousSections,
  scrollToContinuousChapter,
  clearContinuousChapters,
  disposeContinuousReading,
} = useContinuousReading(
  store,
  formatChapterHtml,
  isContinuousMode,
  scrollContainerRef,
)

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

// Navigation
function goHome() {
  router.replace('/')
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

const {
  clearReadingClass,
  startAutoScroll,
  stopAutoScroll,
  startSpeech,
  speechPrev,
  speechNext,
  restartSpeechFromCurrentParagraph,
  resetAutoParagraphIndex,
  handleContentChanged,
  disposeAutoPlayback,
} = useReaderAutoPlayback(
  store,
  computed(() => ({
    autoPageMode: config.value.autoPageMode,
    clickAction: config.value.clickAction,
    scrollPixel: config.value.scrollPixel,
    pageSpeed: config.value.pageSpeed,
    fontSize: config.value.fontSize,
    lineHeight: config.value.lineHeight,
  })),
  isContinuousMode,
  scrollContainerRef,
  chapterTextRef,
  nextChapter,
  prevChapter,
)

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
  if (Date.now() < suppressSelectionCloseUntil.value) return
  if (selectionMenu.value.visible) {
    hideSelectionMenu()
    return
  }
  if (window.getSelection?.()?.toString().trim()) return

  const target = e.target as HTMLElement | null
  if (target?.closest('.tts-controls, .reader-search-panel, .selection-menu')) return
  
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
      if (maxPage > 0 && horizontalPageIndex.value >= maxPage - 1) {
        store.preloadAroundChapter(store.currentIndex)
      }
    } else if (container.scrollHeight - (container.scrollTop + container.clientHeight) < container.clientHeight * 1.5) {
      store.preloadAroundChapter(store.currentIndex)
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
        alignHorizontalToNearestPage(touchState.value.moving)
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

// Toolbar actions
async function toggleBookmark() {
  store.togglePanel('bookmark')
}

function handleTTS() {
  showTTSPanel.value = true
  if (store.isSpeaking) {
    store.pauseTTS()
  } else {
    startSpeech()
  }
}

watch(() => store.isAutoScrolling, (val) => {
  store.autoReading = val
  if (val) startAutoScroll()
  else stopAutoScroll()
})

function changeVoice(name: string) {
  store.setVoiceName(name)
  showTTSPanel.value = true
  if (store.isSpeaking && !store.isPaused) {
    restartSpeechFromCurrentParagraph()
  }
}

function adjustSpeechRate(delta: number) {
  const next = Math.max(0.5, Math.min(3, parseFloat((store.speechConfig.speechRate + delta).toFixed(1))))
  store.setSpeechRate(next)
  showTTSPanel.value = true
  if (store.isSpeaking && !store.isPaused) {
    restartSpeechFromCurrentParagraph()
  }
}

function adjustSpeechPitch(delta: number) {
  const next = Math.max(0.5, Math.min(2, parseFloat((store.speechConfig.speechPitch + delta).toFixed(1))))
  store.setSpeechPitch(next)
  showTTSPanel.value = true
  if (store.isSpeaking && !store.isPaused) {
    restartSpeechFromCurrentParagraph()
  }
}

function setSpeechTimer(minutes: number) {
  store.setSpeechStopTimer(minutes)
  showTTSPanel.value = true
}
async function openInfo() {
  if (!store.book) return
  showBookInfo.value = true
  bookInfoBook.value = {
    ...store.book,
    durChapterIndex: store.currentIndex,
    durChapterTitle: store.currentChapter?.title || store.book.durChapterTitle,
  }
  try {
    const latest = await getBookInfo(store.book.bookUrl, store.book.origin)
    bookInfoBook.value = {
      ...store.book,
      ...latest,
      durChapterIndex: store.currentIndex,
      durChapterTitle: store.currentChapter?.title || latest.durChapterTitle || store.book.durChapterTitle,
    }
  } catch {
    appStore.showToast('获取书籍详情失败，已显示当前缓存信息', 'warning')
  }
}

onMounted(async () => {
  appStore.startReadingSession()
  if (!store.book) {
    const restored = await store.restorePersistedSession()
    if (!restored) {
      router.replace('/')
      return
    }
    appStore.showToast('已恢复最近阅读的离线章节', 'success')
  }
  window.addEventListener('keydown', handleKeydown)
  document.addEventListener('mouseup', handleMouseUpSelection)
  document.addEventListener('touchend', handleTouchEndSelection)
  document.addEventListener('selectionchange', handleSelectionChange)
  checkMedia()
  window.addEventListener('resize', checkMedia)
  store.fetchVoices()
  applySystemTheme(store.isNight ? 'dark' : appStore.theme, store.currentTheme.body)
  if (typeof window !== 'undefined' && window.speechSynthesis) {
    window.speechSynthesis.onvoiceschanged = () => store.fetchVoices()
  }
  speechTimerTicker = window.setInterval(() => {
    speechTimerNow.value = Date.now()
  }, 15000)
  await Promise.all([
    store.fetchBookmarks(),
    store.fetchReplaceRules(),
  ])
  await refreshOfflineCacheState()
  updateHorizontalMetrics()
  await rebuildHorizontalPages()
  if (isContinuousMode.value) {
    await initializeContinuousChapters(store.currentIndex, false)
  }
})

onUnmounted(() => {
  appStore.stopReadingSession()
  window.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('mouseup', handleMouseUpSelection)
  document.removeEventListener('touchend', handleTouchEndSelection)
  document.removeEventListener('selectionchange', handleSelectionChange)
  window.removeEventListener('resize', checkMedia)
  if (speechTimerTicker) clearInterval(speechTimerTicker)
  disposeSelection()
  disposeContinuousReading()
  disposeAutoPlayback()
  store.stopTTS()
  if (typeof window !== 'undefined' && window.speechSynthesis) {
    window.speechSynthesis.onvoiceschanged = null
  }
  applySystemTheme(appStore.theme)
  store.closePanel()
})

watch(() => config.value.autoPageMode, () => {
  if (!store.isAutoScrolling) return
  stopAutoScroll()
  store.isAutoScrolling = true
  startAutoScroll()
})

watch(() => config.value.readMethod, async () => {
  clearSelectionState()
  if (isContinuousMode.value) {
    await initializeContinuousChapters(store.currentIndex, false)
  } else {
    clearContinuousChapters()
    await nextTick()
    if (scrollContainerRef.value) {
      scrollContainerRef.value.scrollTo({ top: 0, left: 0, behavior: 'auto' })
    }
  }
  if (isHorizontalPageMode.value && scrollContainerRef.value) {
    resetHorizontalPagePosition()
  }
  await rebuildHorizontalPages()
  updateHorizontalEndState()
})

watch(() => store.currentIndex, () => {
  if (!isHorizontalPageMode.value) return
  resetHorizontalPagePosition()
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
  resetAutoParagraphIndex()
  if (!store.isSpeaking) {
    clearReadingClass()
  }
  if (!isContinuousMode.value) {
    store.preloadAroundChapter(store.currentIndex)
  }
  if (isContinuousMode.value && !suppressContinuousSync.value) {
    await syncContinuousToStoreState()
  }
  void refreshOfflineCacheState()
})

watch(() => store.content, () => {
  resetAutoParagraphIndex()
  if (isContinuousMode.value) {
    const current = getContinuousChapter(store.currentIndex)
    if (current) {
      current.content = store.content
      current.html = formatChapterHtml(store.content)
    }
  }
  handleContentChanged()
  handleContentUpdated()
  void refreshOfflineCacheState()
})

watch(() => store.book?.bookUrl, () => {
  void refreshOfflineCacheState()
})

watch([showSearch, searchQuery, () => config.value.paragraphSpacing], () => {
  if (isContinuousMode.value) {
    syncContinuousChapterHtml()
  }
  handlePresentationUpdated()
})

watch(() => config.value.selectAction, (value) => {
  if (value !== 'popup') {
    clearSelectionState()
  }
})

watch(() => store.isSpeaking, (speaking) => {
  if (speaking) {
    showTTSPanel.value = true
  }
  if (!speaking && !store.isAutoScrolling) {
    clearReadingClass()
  }
})

watch(
  [() => store.isNight, () => store.currentTheme.body, () => appStore.theme],
  ([isNight, body]) => {
    applySystemTheme(isNight ? 'dark' : appStore.theme, body)
  },
  { immediate: true },
)
</script>

<style scoped>
.reader-view {
  height: 100vh;
  height: 100dvh;
  width: 100vw;
  display: flex;
  position: relative;
  overflow: hidden;
  transition: background 0.3s, color 0.3s;
  padding-top: var(--safe-area-top);
  box-sizing: border-box;
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
  overscroll-behavior: contain;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.reader-scroll-container.horizontal-page-mode {
  overflow-x: auto;
  overflow-y: hidden;
  touch-action: pan-x pinch-zoom;
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

.offline-banner {
  position: sticky;
  top: 0;
  z-index: 6;
  margin: 0 auto;
  width: min(100%, 880px);
  padding: 10px 16px;
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-primary);
  border-bottom: 1px solid rgba(201, 127, 58, 0.18);
  font-size: 13px;
  line-height: 1.5;
  text-align: center;
  backdrop-filter: blur(6px);
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
  bottom: calc(20px + var(--safe-area-bottom));
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
  top: var(--safe-area-top);
  bottom: var(--safe-area-bottom);
  left: 0;
  width: min(340px, 85vw);
  z-index: 50;
  box-shadow: 4px 0 24px rgba(0,0,0,0.15);
  transition: background 0.3s;
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

:deep(.search-highlight) {
  background: yellow;
  color: black;
  border-radius: 2px;
}

:deep(.search-highlight.current-match) {
  background: orange;
}

@media (max-width: 768px) {
  .chapter-content {
    padding: 24px 20px 8px;
    min-height: auto;
    height: auto;
  }

  .continuous-reading {
    padding: 16px 0 8px;
  }

  .continuous-chapter {
    padding-top: 20px;
    padding-bottom: 8px;
  }

  .chapter-title {
    margin-bottom: 0.9em;
  }

  .chapter-footer {
    margin-top: 12px;
    padding-bottom: 0;
  }

  .reader-drawer {
    top: var(--safe-area-top);
    bottom: var(--safe-area-bottom);
    width: min(340px, 85vw);
    padding-top: var(--safe-area-top);
    padding-bottom: var(--safe-area-bottom);
    box-sizing: border-box;
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
