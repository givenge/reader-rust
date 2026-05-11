import http from './http'
import type { SearchBook } from '../types'
import { appendAuthQueryParams } from '../utils/secureAccess'

export function searchBookMulti(params: {
  key: string
  page?: number
  bookSourceGroup?: string
  bookSourceUrl?: string
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
  bookSourceUrl?: string
  concurrentCount?: number
  searchSize?: number
}) {
  const query = new URLSearchParams()
  query.set('key', params.key)
  if (params.bookSourceGroup) query.set('bookSourceGroup', params.bookSourceGroup)
  if (params.bookSourceUrl) query.set('bookSourceUrl', params.bookSourceUrl)
  if (params.concurrentCount) query.set('concurrentCount', String(params.concurrentCount))
  if (params.searchSize) query.set('searchSize', String(params.searchSize))

  appendAuthQueryParams(query)

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
  origin?: string
  refresh?: number
  lastIndex?: number
  resultLimit?: number
  concurrentCount?: number
}) {
  return http
    .post<SearchBook[] | AvailableBookSourceResult>('/getAvailableBookSource', params)
    .then((r) => normalizeAvailableBookSourceResult(r.data))
}

export function getAvailableBookSourceSSE(params: {
  url?: string
  name?: string
  author?: string
  origin?: string
  refresh?: number
  lastIndex?: number
  concurrentCount?: number
}) {
  const query = new URLSearchParams()
  if (params.url) query.set('url', params.url)
  if (params.name) query.set('name', params.name)
  if (params.author) query.set('author', params.author)
  if (params.origin) query.set('origin', params.origin)
  if (typeof params.refresh !== 'undefined') query.set('refresh', String(params.refresh))
  query.set('lastIndex', String(params.lastIndex ?? -1))
  query.set('concurrentCount', String(params.concurrentCount ?? 8))

  appendAuthQueryParams(query)

  return new EventSource(`/reader3/getAvailableBookSourceSSE?${query.toString()}`)
}

export interface AvailableBookSourceResult {
  books: SearchBook[]
  lastIndex: number
  hasMore: boolean
}

function normalizeAvailableBookSourceResult(
  data: SearchBook[] | AvailableBookSourceResult,
): AvailableBookSourceResult {
  if (Array.isArray(data)) {
    return {
      books: data,
      lastIndex: data.length - 1,
      hasMore: false,
    }
  }
  return data
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

  appendAuthQueryParams(query)

  return new EventSource(`/reader3/searchBookSourceSSE?${query.toString()}`)
}
