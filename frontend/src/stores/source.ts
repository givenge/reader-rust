import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getBookSources } from '../api/source'
import type { BookSource } from '../types'

export const useSourceStore = defineStore('source', () => {
  const sources = ref<BookSource[]>([])
  const loading = ref(false)

  async function fetchSources() {
    if (loading.value) return
    loading.value = true
    try {
      sources.value = await getBookSources()
    } finally {
      loading.value = false
    }
  }

  return {
    sources,
    loading,
    fetchSources
  }
})
