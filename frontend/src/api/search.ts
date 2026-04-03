import http from './http'
import type { SearchBook } from '../types'

export function searchBookMulti(params: {
  key: string
  page?: number
  bookSourceGroup?: string
}) {
  return http.post<SearchBook[]>('/searchBookMulti', params).then((r) => r.data)
}

/**
 * SSE-based multi-source search. Returns an EventSource.
 * Caller is responsible for closing the connection.
 */
export function searchBookMultiSSE(params: {
  key: string
  bookSourceGroup?: string
  concurrentCount?: number
  searchSize?: number
}) {
  const query = new URLSearchParams()
  query.set('key', params.key)
  if (params.bookSourceGroup) query.set('bookSourceGroup', params.bookSourceGroup)
  if (params.concurrentCount) query.set('concurrentCount', String(params.concurrentCount))
  if (params.searchSize) query.set('searchSize', String(params.searchSize))

  const token = localStorage.getItem('accessToken')
  if (token) query.set('accessToken', token)

  return new EventSource(`/reader3/searchBookMultiSSE?${query.toString()}`)
}

export function exploreBook(params: {
  ruleFindUrl: string
  bookSourceUrl: string
  page?: number
}) {
  return http.post<SearchBook[]>('/exploreBook', params).then((r) => r.data)
}

export function getAvailableBookSource(params: {
  url?: string
  name?: string
  author?: string
}) {
  return http.post<SearchBook[]>('/getAvailableBookSource', params).then((r) => r.data)
}

export function searchBookSourceSSE(params: {
  url: string
  bookSourceGroup?: string
  lastIndex?: number
  concurrentCount?: number
  searchSize?: number
  refresh?: number
}) {
  const query = new URLSearchParams()
  query.set('url', params.url)
  query.set('concurrentCount', String(params.concurrentCount ?? 24))
  query.set('lastIndex', String(params.lastIndex ?? -1))
  if (typeof params.refresh !== 'undefined') query.set('refresh', String(params.refresh))
  if (params.bookSourceGroup !== undefined) query.set('bookSourceGroup', params.bookSourceGroup)
  if (params.searchSize) query.set('searchSize', String(params.searchSize))

  const token = localStorage.getItem('accessToken')
  if (token) query.set('accessToken', token)

  return new EventSource(`/reader3/searchBookSourceSSE?${query.toString()}`)
}
