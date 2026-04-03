import http from './http'

/**
 * SSE-based book caching. Returns an EventSource.
 */
export function cacheBookSSE(params: {
  bookUrl: string
  tocUrl?: string
  count?: number
  refresh?: number
  concurrentCount?: number
}) {
  const query = new URLSearchParams()
  query.set('bookUrl', params.bookUrl)
  if (params.tocUrl) query.set('tocUrl', params.tocUrl)
  if (params.count) query.set('count', String(params.count))
  if (params.refresh) query.set('refresh', String(params.refresh))
  if (params.concurrentCount) query.set('concurrentCount', String(params.concurrentCount))

  const token = localStorage.getItem('accessToken')
  if (token) query.set('accessToken', token)

  return new EventSource(`/reader3/cacheBookSSE?${query.toString()}`)
}

/**
 * Delete all content cache for a book
 */
export function deleteBookCache(bookUrl: string) {
  return http.post('/deleteBookCache', { bookUrl }).then((r) => r.data)
}
