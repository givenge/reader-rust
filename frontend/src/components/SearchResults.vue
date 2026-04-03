<template>
  <div class="search-results">
    <div class="search-header">
      <h2>
        搜索 "{{ searchKey }}"
        <span v-if="isSearching" class="searching-indicator">
          <span class="dot-pulse"></span>
          搜索中...
        </span>
        <span v-else class="result-count">({{ results.length }} 个结果)</span>
      </h2>
      <button class="back-btn" @click="$emit('back')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
        返回书架
      </button>
    </div>

    <BookGrid
      :books="results"
      :is-search="true"
      :loading="isSearching && results.length === 0"
      empty-text="未找到相关书籍"
      @click="handleBookClick"
      @addToShelf="handleAddToShelf"
    />
  </div>
</template>

<script setup lang="ts">
import { watch, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useBookshelfStore } from '../stores/bookshelf'
import { useReaderStore } from '../stores/reader'
import { useAppStore } from '../stores/app'
import { searchBookMultiSSE } from '../api/search'
import { saveBook } from '../api/bookshelf'
import BookGrid from './BookGrid.vue'
import type { Book, SearchBook } from '../types'

import { storeToRefs } from 'pinia'

const router = useRouter()
const shelfStore = useBookshelfStore()
const readerStore = useReaderStore()
const appStore = useAppStore()

const { searchKey, searchResults: results, isSearching } = storeToRefs(shelfStore)

let eventSource: EventSource | null = null

function doSearch(key: string) {
  // Clean up previous
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }

  shelfStore.searchResults = []
  shelfStore.isSearching = true

  eventSource = searchBookMultiSSE({ key, concurrentCount: 24 })

  eventSource.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data)
      if (data.data && Array.isArray(data.data)) {
        // Deduplicate
        const existing = new Set(shelfStore.searchResults.map((r) => r.bookUrl))
        const newBooks = data.data.filter((b: SearchBook) => !existing.has(b.bookUrl))
        shelfStore.searchResults = [...shelfStore.searchResults, ...newBooks]
      }
    } catch { /* skip */ }
  }

  eventSource.addEventListener('end', () => {
    shelfStore.isSearching = false
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
  })

  eventSource.addEventListener('error', () => {
    shelfStore.isSearching = false
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
  })

  eventSource.onerror = () => {
    shelfStore.isSearching = false
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
  }
}

watch(
  () => shelfStore.searchKey,
  (key) => {
    if (key) {
      doSearch(key)
    }
  },
  { immediate: true }
)

onUnmounted(() => {
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }
})

async function handleBookClick(book: Book | SearchBook) {
  const b = book as Book
  if (b.origin && b.bookUrl) {
    await readerStore.loadBook(b)
    await readerStore.loadChapter(b.durChapterIndex || 0)
    router.push('/reader')
  }
}

async function handleAddToShelf(book: Book | SearchBook) {
  try {
    await saveBook({
      name: book.name,
      author: book.author,
      bookUrl: book.bookUrl,
      origin: book.origin,
      coverUrl: book.coverUrl,
    })
    appStore.showToast(`"${book.name}" 已加入书架`, 'success')
    shelfStore.fetchBooks()
  } catch (e: unknown) {
    appStore.showToast((e as Error).message, 'error')
  }
}

defineEmits<{
  back: []
}>()
</script>

<style scoped>
.search-results {
  padding: 0 var(--space-6);
}

.search-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) 0;
  gap: var(--space-4);
}

.search-header h2 {
  font-size: var(--text-xl);
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.result-count {
  font-size: var(--text-sm);
  font-weight: 400;
  color: var(--color-text-tertiary);
}

.searching-indicator {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  font-weight: 400;
  color: var(--color-primary);
}

.dot-pulse {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-primary);
  animation: pulse 1.2s infinite ease-in-out;
}

@keyframes pulse {
  0%, 80%, 100% {
    transform: scale(0.6);
    opacity: 0.5;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
}

.back-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  transition: all var(--duration-fast);
}

.back-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}
</style>
