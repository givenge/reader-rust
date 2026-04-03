import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { exploreBook } from '../api/explore'
import { useSourceStore } from './source'
import type { SearchBook, BookSource } from '../types'

export interface ExploreCategory {
  title: string
  url: string
}

export const useExploreStore = defineStore('explore', () => {
  const sourceStore = useSourceStore()

  const activeSourceUrl = ref<string>('')
  const activeCategoryUrl = ref<string>('')
  
  const books = ref<SearchBook[]>([])
  const loading = ref(false)
  const page = ref(1)
  const hasMore = ref(true)
  const error = ref<string | null>(null)

  // 筛选出启用了 explore 的书源
  const exploreSources = computed(() => {
    return sourceStore.sources.filter((s: BookSource) => s.enabledExplore && s.exploreUrl)
  })

  // 当前选中的书源对象
  const currentSource = computed(() => {
    return sourceStore.sources.find((s: BookSource) => s.bookSourceUrl === activeSourceUrl.value)
  })

  // 解析当前书源的 exploreUrl 分类
  const categories = computed<ExploreCategory[]>(() => {
    if (!currentSource.value || !currentSource.value.exploreUrl) return []
    const rule = currentSource.value.exploreUrl.trim()
    
    try {
      // 尝试解析 JSON 格式 (如 [{"title":"男频","url":"..."}])
      if (rule.startsWith('[')) {
        const arr = JSON.parse(rule)
        if (Array.isArray(arr)) return arr
      }
    } catch {
      // continue to fallback parsing
    }

    // 解析 Legado 常见的双冒号格式: 男频::https://... \n 女频::https://...
    const lines = rule.split(/\n|<br>/i).map((line: string) => line.trim()).filter(Boolean)
    const result: ExploreCategory[] = []
    
    for (const line of lines) {
      if (line.includes('::')) {
        const [title, url] = line.split('::').map((s: string) => s.trim())
        if (title && url) {
          result.push({ title, url })
        }
      }
    }
    return result
  })

  function setSource(url: string) {
    if (activeSourceUrl.value !== url) {
      activeSourceUrl.value = url
      // 默认选中第一个分类
      if (categories.value.length > 0) {
        setCategory(categories.value[0].url)
      } else {
        setCategory('')
      }
    }
  }

  function setCategory(url: string) {
    if (activeCategoryUrl.value !== url) {
      activeCategoryUrl.value = url
      resetAndFetch()
    }
  }

  async function resetAndFetch() {
    books.value = []
    page.value = 1
    hasMore.value = true
    error.value = null
    await fetchMore()
  }

  async function fetchMore() {
    if (loading.value || !hasMore.value || !activeSourceUrl.value || !activeCategoryUrl.value) return

    loading.value = true
    error.value = null
    try {
      const result = await exploreBook({
        bookSourceUrl: activeSourceUrl.value,
        ruleFindUrl: activeCategoryUrl.value,
        page: page.value,
      })

      if (result && result.length > 0) {
        books.value.push(...result)
        page.value++
      } else {
        hasMore.value = false
      }
    } catch (err: any) {
      error.value = err.message || '加载失败'
      hasMore.value = false
    } finally {
      loading.value = false
    }
  }

  // 初始化时加载书源数据
  async function init() {
    if (sourceStore.sources.length === 0) {
      await sourceStore.fetchSources()
    }
    if (exploreSources.value.length > 0 && !activeSourceUrl.value) {
      setSource(exploreSources.value[0].bookSourceUrl)
    }
  }

  return {
    activeSourceUrl,
    activeCategoryUrl,
    books,
    loading,
    page,
    hasMore,
    error,
    exploreSources,
    currentSource,
    categories,
    init,
    setSource,
    setCategory,
    fetchMore,
    resetAndFetch,
  }
})
