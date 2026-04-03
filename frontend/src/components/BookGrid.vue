<template>
  <div class="book-grid">
    <TransitionGroup name="card">
      <div
        v-for="book in books"
        :key="book.bookUrl"
        class="book-grid-item"
        :class="{
          'drop-target': sortable && dropTargetUrl === book.bookUrl,
          'is-dragging': sortable && draggedUrl === book.bookUrl,
        }"
        :draggable="sortable"
        @dragstart="handleDragStart(book)"
        @dragenter.prevent="handleDragEnter(book)"
        @dragover.prevent="handleDragOver(book)"
        @drop.prevent="handleDrop(book)"
        @dragend="handleDragEnd"
      >
        <BookCard
          :book="book"
          :edit-mode="editMode"
          :selected="selectedUrls?.has(book.bookUrl)"
          :is-search="isSearch"
          :dragging="sortable && draggedUrl === book.bookUrl"
          @click="$emit('click', $event)"
          @info="$emit('info', $event)"
          @delete="$emit('delete', $event)"
          @select="$emit('select', $event)"
          @addToShelf="$emit('addToShelf', $event)"
        />
      </div>
    </TransitionGroup>
    <div v-if="books.length === 0 && !loading" class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
        <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
      </svg>
      <p>{{ emptyText }}</p>
    </div>
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import BookCard from './BookCard.vue'
import type { Book, SearchBook } from '../types'

const props = defineProps<{
  books: (Book | SearchBook)[]
  editMode?: boolean
  selectedUrls?: Set<string>
  isSearch?: boolean
  loading?: boolean
  emptyText?: string
  sortable?: boolean
}>()

const emit = defineEmits<{
  click: [book: Book | SearchBook]
  info: [book: Book | SearchBook]
  delete: [book: Book | SearchBook]
  select: [book: Book | SearchBook]
  addToShelf: [book: Book | SearchBook]
  reorder: [payload: { draggedUrl: string; targetUrl: string }]
}>()

const draggedUrl = ref('')
const dropTargetUrl = ref('')

function getBookUrl(book: Book | SearchBook) {
  return book.bookUrl
}

function handleDragStart(book: Book | SearchBook) {
  if (!props.sortable) return
  draggedUrl.value = getBookUrl(book)
  dropTargetUrl.value = getBookUrl(book)
}

function handleDragEnter(book: Book | SearchBook) {
  if (!props.sortable || !draggedUrl.value) return
  dropTargetUrl.value = getBookUrl(book)
}

function handleDragOver(book: Book | SearchBook) {
  if (!props.sortable || !draggedUrl.value) return
  dropTargetUrl.value = getBookUrl(book)
}

function handleDrop(book: Book | SearchBook) {
  if (!props.sortable || !draggedUrl.value) return
  const sourceUrl = draggedUrl.value
  const targetUrl = getBookUrl(book)
  draggedUrl.value = ''
  dropTargetUrl.value = ''
  if (sourceUrl !== targetUrl) {
    emit('reorder', { draggedUrl: sourceUrl, targetUrl })
  }
}

function handleDragEnd() {
  draggedUrl.value = ''
  dropTargetUrl.value = ''
}
</script>

<style scoped>
.book-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--space-4);
  padding: var(--space-4) 0;
}

.book-grid-item {
  min-width: 0;
}

.book-grid-item.drop-target :deep(.book-card) {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(var(--color-primary-rgb), 0.16);
}

.book-grid-item.is-dragging {
  cursor: grabbing;
}

.empty-state {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-16) var(--space-6);
  color: var(--color-text-tertiary);
  gap: var(--space-4);
}

.empty-state svg {
  width: 64px;
  height: 64px;
  opacity: 0.3;
}

.empty-state p {
  font-size: var(--text-base);
}

.loading-state {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-16) var(--space-6);
  gap: var(--space-4);
  color: var(--color-text-tertiary);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Card transition group */
.card-enter-active {
  transition: all var(--duration-normal) var(--ease-out);
}
.card-leave-active {
  transition: all var(--duration-fast) var(--ease-out);
}
.card-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(8px);
}
.card-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
.card-move {
  transition: transform var(--duration-normal) var(--ease-out);
}

@media (max-width: 640px) {
  .book-grid {
    grid-template-columns: 1fr;
  }
}
</style>
