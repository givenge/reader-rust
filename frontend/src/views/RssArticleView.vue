<template>
  <div class="rss-article-view">
    <header class="article-page-header">
      <button class="back-btn" @click="goBack">返回文章列表</button>
      <button class="ghost-btn" @click="goManage">管理订阅源</button>
    </header>

    <article class="article-page-card">
      <template v-if="store.activeArticle">
        <div class="article-page-head">
          <h1>{{ store.activeArticle.title || '正文' }}</h1>
          <div class="article-page-meta">
            <span v-if="store.activeArticle.pubDate">{{ formatRelativeTime(store.activeArticle.pubDate) }}</span>
            <span v-if="store.activeArticle.origin" class="meta-sep">·</span>
            <span v-if="store.activeArticle.origin">{{ store.activeArticle.origin }}</span>
          </div>
        </div>

        <div class="article-page-scroll">
          <div v-if="store.contentLoading" class="empty-box">正文加载中...</div>
          <div v-else-if="store.activeContent" class="content-html" v-html="store.activeContent"></div>
          <div v-else class="empty-box">这篇文章暂时没有可显示的正文。</div>
        </div>
      </template>
      <div v-else class="empty-box">文章不存在或已失效。</div>
    </article>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useRssStore } from '../stores/rss'
import { saveRecentReadBook } from '../utils/recentBooks'
import type { RssArticle } from '../types'

const route = useRoute()
const router = useRouter()
const store = useRssStore()

onMounted(async () => {
  const source = String(route.query.source || '')
  const link = String(route.query.link || '')
  if (!source || !link) return

  const existing = store.articles.find((item) => (item.variable || store.activeSourceUrl) === source && item.link === link)
  const article: RssArticle & { variable?: string } = existing || {
    origin: String(route.query.origin || ''),
    sort: '',
    title: String(route.query.title || ''),
    order: 0,
    link,
    pubDate: String(route.query.pubDate || ''),
    variable: source,
  }

  await store.openArticle(article)
  saveRecentReadBook({
    name: article.title || 'RSS 文章',
    author: article.origin || 'RSS',
    bookUrl: article.link,
    origin: `rss:${source}`,
    recentKind: 'rss',
    rssSourceUrl: source,
    rssLink: article.link,
    rssPubDate: article.pubDate,
    durChapterTime: Date.now(),
    intro: article.description,
  })
})

function goBack() {
  router.push('/rss')
}

function goManage() {
  router.push('/rss/manage')
}

function formatRelativeTime(value?: string) {
  if (!value) return ''
  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) return value

  const diff = Date.now() - timestamp
  if (diff < 0) return '刚刚'

  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour
  const month = 30 * day
  const year = 365 * day

  if (diff < minute) return '刚刚'
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`
  if (diff < month) return `${Math.floor(diff / day)} 天前`
  if (diff < year) return `${Math.floor(diff / month)} 个月前`
  return `${Math.floor(diff / year)} 年前`
}
</script>

<style scoped>
.rss-article-view {
  min-height: calc(100dvh - var(--header-height) - var(--safe-area-top) - var(--safe-area-bottom));
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
}

.article-page-header,
.article-page-card {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-light);
  border-radius: 24px;
}

.article-page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
}

.article-page-card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 16px;
  box-sizing: border-box;
}

.article-page-head {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 14px;
}

.article-page-head h1 {
  margin: 0;
  font-size: 30px;
  line-height: 1.25;
}

.article-page-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.article-page-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.back-btn,
.ghost-btn {
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid var(--color-border-light);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  font-size: 14px;
}

.content-html {
  line-height: 1.85;
  color: var(--color-text);
  max-width: 100%;
  overflow-wrap: anywhere;
  width: min(760px, 100%);
  margin: 0 auto;
}

.content-html :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 14px;
}

.content-html :deep(video),
.content-html :deep(iframe),
.content-html :deep(table),
.content-html :deep(pre),
.content-html :deep(code) {
  max-width: 100%;
}

.content-html :deep(pre) {
  overflow: auto;
}

.empty-box {
  flex: 1;
  min-height: 140px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--color-text-tertiary);
}

.meta-sep {
  opacity: 0.5;
}

@media (max-width: 640px) {
  .rss-article-view {
    box-sizing: border-box;
    padding: 10px;
    gap: 8px;
  }

  .article-page-header,
  .article-page-card {
    border-radius: 20px;
  }

  .article-page-card {
    padding: 14px;
  }

  .article-page-head h1 {
    font-size: 22px;
  }
}
</style>
