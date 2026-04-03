import { computed, nextTick, ref } from 'vue'
import type { ComputedRef, Ref } from 'vue'
import type { useReaderStore } from '../stores/reader'

type ReaderStore = ReturnType<typeof useReaderStore>

export function useHorizontalPaging(
  store: ReaderStore,
  config: ComputedRef<{ fontSize: number; fontWeight: number; lineHeight: number }>,
  currentFontFamily: ComputedRef<string>,
  formattedContent: ComputedRef<string>,
  isHorizontalPageMode: ComputedRef<boolean>,
  scrollContainerRef: Ref<HTMLElement | undefined>,
) {
  const horizontalPageIndex = ref(0)
  const horizontalPageStep = ref(1)
  const horizontalPageStepStyle = computed(() => `${Math.max(1, horizontalPageStep.value)}px`)
  const horizontalPages = ref<string[]>([])
  const isHorizontalAtEnd = ref(false)

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
      if (idx > 0) delete styleObj['text-indent']
      if (idx < segments.length - 1) delete styleObj['margin-bottom']
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
    return Array.from(root.querySelectorAll('p')).map((node) => node.outerHTML)
  }

  function updateHorizontalMetrics() {
    const container = scrollContainerRef.value
    if (!container || !isHorizontalPageMode.value) return
    horizontalPageStep.value = Math.max(1, container.clientWidth)
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
      pages[0] = `${titleHtml}${pages[0]}`
    }

    document.body.removeChild(measurer)
    horizontalPages.value = pages
    horizontalPageIndex.value = Math.min(horizontalPageIndex.value, pages.length - 1)

    const targetLeft = horizontalPageIndex.value * horizontalPageStep.value
    container.scrollTo({ left: targetLeft, behavior: 'auto' })
    updateHorizontalEndState()
  }

  function alignHorizontalToNearestPage(touchMoving: boolean) {
    const container = scrollContainerRef.value
    if (!container || !isHorizontalPageMode.value || touchMoving) return
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

  function resetHorizontalPagePosition() {
    horizontalPageIndex.value = 0
    const container = scrollContainerRef.value
    if (!container) return
    updateHorizontalMetrics()
    container.scrollTo({ left: 0, behavior: 'auto' })
  }

  return {
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
  }
}
