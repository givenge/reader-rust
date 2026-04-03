<template>
  <div
    class="book-card"
    :class="{ 'edit-mode': editMode, 'selected': selected, 'dragging': dragging }"
    @click="handleCardClick"
  >
    <!-- Cover -->
    <div class="card-cover" @click.stop="handleCoverClick">
      <img
        v-if="coverSrc"
        :src="coverSrc"
        :alt="book.name"
        class="cover-img"
        loading="lazy"
        @error="coverFailed = true"
      />
      <div v-else class="cover-placeholder">
        <span class="cover-title">{{ book.name }}</span>
        <span class="cover-author">{{ book.author }}</span>
      </div>

      <!-- Unread badge -->
      <div v-if="unreadCount > 0 && !editMode" class="unread-badge">
        {{ unreadCount > 99 ? '99+' : unreadCount }}
      </div>

      <!-- Selection overlay -->
      <div v-if="editMode" class="selection-overlay">
        <div class="checkbox" :class="{ checked: selected }">
          <svg v-if="selected" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M20 6 9 17l-5-5" />
          </svg>
        </div>
      </div>
    </div>

    <!-- Info -->
    <div class="card-info">
      <h3 class="book-name">{{ book.name }}</h3>
      <div class="book-meta">
        <span class="book-author">{{ book.author || '未知作者' }}</span>
        <span v-if="asBook.totalChapterNum" class="meta-dot">·</span>
        <span v-if="asBook.totalChapterNum" class="book-chapters">共{{ asBook.totalChapterNum }}章</span>
      </div>
      <p v-if="asBook.durChapterTitle && !isSearch" class="book-progress">
        已读：{{ asBook.durChapterTitle }}
      </p>
      <p v-if="asBook.latestChapterTitle" class="book-latest">
        最新：{{ asBook.latestChapterTitle }}
      </p>
      <div v-if="!isSearch && (browserCachedCount > 0 || serverCachedCount > 0)" class="book-cache-row">
        <span v-if="browserCachedCount > 0" class="cache-chip primary">离线 {{ browserCachedCount }} 章</span>
        <span v-if="serverCachedCount > 0" class="cache-chip">服务端 {{ serverCachedCount }} 章</span>
      </div>
      <!-- Search mode: add to shelf -->
      <button
        v-if="isSearch"
        class="add-shelf-btn"
        @click.stop="$emit('addToShelf', book)"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
          <path d="M12 5v14M5 12h14" />
        </svg>
        加入书架
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { getCoverUrl } from '../api/bookshelf'
import type { Book, SearchBook } from '../types'

const props = defineProps<{
  book: Book | SearchBook
  editMode?: boolean
  selected?: boolean
  isSearch?: boolean
  dragging?: boolean
}>()

const emit = defineEmits<{
  click: [book: Book | SearchBook]
  info: [book: Book | SearchBook]
  delete: [book: Book | SearchBook]
  addToShelf: [book: Book | SearchBook]
  select: [book: Book | SearchBook]
}>()

function handleCardClick() {
  if (props.editMode) {
    emit('select', props.book)
  } else {
    emit('click', props.book)
  }
}

function handleCoverClick() {
  if (props.editMode) {
    emit('select', props.book)
  } else {
    emit('info', props.book)
  }
}

const coverFailed = ref(false)

const asBook = computed(() => props.book as Book)

const coverSrc = computed(() => {
  if (coverFailed.value) return ''
  const url = (props.book as Book).customCoverUrl || props.book.coverUrl
  if (!url) return ''
  return getCoverUrl(url)
})

const unreadCount = computed(() => {
  const b = props.book as Book
  if (!b.totalChapterNum || b.durChapterIndex === undefined) return 0
  return Math.max(0, b.totalChapterNum - 1 - b.durChapterIndex)
})

const browserCachedCount = computed(() => Math.max(0, asBook.value.browserCachedChapterCount || 0))
const serverCachedCount = computed(() => Math.max(0, asBook.value.cachedChapterCount || 0))
</script>

<style scoped>
.book-card {
  display: flex;
  gap: var(--space-4);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  cursor: pointer;
  transition: all var(--duration-normal) var(--ease-out);
  position: relative;
  overflow: hidden;
}

.book-card:hover {
  border-color: var(--color-primary-border);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.book-card:active {
  transform: translateY(0);
}

.card-cover {
  width: 90px;
  height: 120px;
  flex-shrink: 0;
  border-radius: var(--radius-sm);
  overflow: hidden;
  position: relative;
  background: var(--color-bg-sunken);
}

.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-2);
  background: linear-gradient(135deg, var(--color-primary-bg), var(--color-bg-sunken));
  text-align: center;
  gap: var(--space-1);
}

.cover-title {
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--color-primary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: var(--leading-tight);
}

.cover-author {
  font-size: 10px;
  color: var(--color-text-tertiary);
}

.unread-badge {
  position: absolute;
  top: var(--space-1);
  right: var(--space-1);
  background: var(--color-primary);
  color: white;
  font-size: 10px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: var(--radius-full);
  min-width: 18px;
  text-align: center;
  line-height: 16px;
}

.selection-overlay {
  position: absolute;
  inset: 0;
  background: rgba(var(--color-primary-rgb), 0.1);
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  padding: var(--space-1);
  opacity: 1;
}

.checkbox {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-full);
  border: 2px solid white;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration-fast);
}

.checkbox.checked {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.checkbox svg {
  width: 14px;
  height: 14px;
  color: white;
}

.book-card.selected {
  border-color: var(--color-primary);
  background: rgba(var(--color-primary-rgb), 0.05);
}

.book-card.dragging {
  opacity: 0.55;
  transform: scale(0.98);
  box-shadow: none;
}

.edit-btn {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration-fast);
}

.edit-btn svg {
  width: 18px;
  height: 18px;
}

.edit-btn.delete {
  background: var(--color-danger);
  color: white;
}

.edit-btn.delete:hover {
  background: #ff4d4f;
  transform: scale(1.1);
}

.card-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-1) 0;
}

.book-name {
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--color-text);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: var(--leading-tight);
}

.book-meta {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.book-author {
  font-size: var(--text-xs);
}

.meta-dot {
  font-size: 10px;
}

.book-progress,
.book-latest {
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: var(--leading-normal);
}

.book-latest {
  color: var(--color-text-tertiary);
}

.book-cache-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.cache-chip {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.05);
  color: var(--color-text-secondary);
  font-size: 11px;
  line-height: 1.4;
}

.cache-chip.primary {
  background: rgba(201, 127, 58, 0.12);
  color: var(--color-primary);
}

.add-shelf-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  margin-top: var(--space-2);
  padding: var(--space-1) var(--space-3);
  background: var(--color-primary);
  color: white;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: 500;
  transition: all var(--duration-fast);
  align-self: flex-start;
}

.add-shelf-btn:hover {
  background: var(--color-primary-dark);
  transform: scale(1.02);
}

.add-shelf-btn:active {
  transform: scale(0.98);
}
</style>
