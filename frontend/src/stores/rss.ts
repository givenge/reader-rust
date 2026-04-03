import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { deleteRssSource, getRssArticles, getRssContent, getRssSources, saveRssSource, saveRssSources } from '../api/rss'
import type { RssArticle, RssSource } from '../types'

export const useRssStore = defineStore('rss', () => {
  const sources = ref<RssSource[]>([])
  const activeSourceUrl = ref('')
  const articles = ref<RssArticle[]>([])
  const page = ref(1)
  const hasMore = ref(true)
  const loading = ref(false)
  const contentLoading = ref(false)
  const activeArticle = ref<RssArticle | null>(null)
  const activeContent = ref('')

  const activeSource = computed(() => sources.value.find((item) => item.sourceUrl === activeSourceUrl.value) || null)

  async function fetchSources() {
    sources.value = await getRssSources().catch(() => [])
    if (!activeSourceUrl.value && sources.value.length) {
      activeSourceUrl.value = sources.value[0].sourceUrl
    }
  }

  async function addSource(source: RssSource) {
    await saveRssSource(source)
    await fetchSources()
  }

  async function addSources(list: RssSource[]) {
    await saveRssSources(list)
    await fetchSources()
  }

  async function removeSource(source: RssSource) {
    await deleteRssSource({ sourceUrl: source.sourceUrl, sourceName: source.sourceName })
    if (activeSourceUrl.value === source.sourceUrl) {
      activeSourceUrl.value = ''
      articles.value = []
      activeArticle.value = null
      activeContent.value = ''
    }
    await fetchSources()
  }

  async function fetchArticles(reset = false) {
    if (!activeSourceUrl.value || loading.value) return
    if (reset) {
      page.value = 1
      hasMore.value = true
      articles.value = []
    }
    loading.value = true
    try {
      const res = await getRssArticles({
        sourceUrl: activeSourceUrl.value,
        sortName: '默认',
        sortUrl: activeSource.value?.sortUrl || activeSourceUrl.value,
        page: page.value,
      })
      const list = res.first || []
      articles.value = reset ? list : articles.value.concat(list)
      hasMore.value = list.length >= 50
      if (list.length && !activeArticle.value) {
        await openArticle(list[0])
      }
      if (list.length) {
        page.value += 1
      }
    } finally {
      loading.value = false
    }
  }

  async function openArticle(article: RssArticle) {
    if (!activeSourceUrl.value) return
    activeArticle.value = article
    contentLoading.value = true
    try {
      activeContent.value = await getRssContent({
        sourceUrl: activeSourceUrl.value,
        link: article.link,
        origin: article.origin || activeSourceUrl.value,
      })
    } finally {
      contentLoading.value = false
    }
  }

  async function setSource(url: string) {
    activeSourceUrl.value = url
    activeArticle.value = null
    activeContent.value = ''
    await fetchArticles(true)
  }

  return {
    sources,
    activeSourceUrl,
    activeSource,
    articles,
    page,
    hasMore,
    loading,
    contentLoading,
    activeArticle,
    activeContent,
    fetchSources,
    addSource,
    addSources,
    removeSource,
    fetchArticles,
    openArticle,
    setSource,
  }
})
