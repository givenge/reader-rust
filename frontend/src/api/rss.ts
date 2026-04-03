import http from './http'
import type { RssArticle, RssSource } from '../types'

export function getRssSources() {
  return http.get<RssSource[]>('/getRssSources').then((r) => r.data)
}

export function saveRssSource(source: RssSource) {
  return http.post<string>('/saveRssSource', source).then((r) => r.data)
}

export function saveRssSources(sources: RssSource[]) {
  return http.post<string>('/saveRssSources', sources).then((r) => r.data)
}

export function deleteRssSource(source: Pick<RssSource, 'sourceUrl' | 'sourceName'>) {
  return http.post<string>('/deleteRssSource', source).then((r) => r.data)
}

export function getRssArticles(params: {
  sourceUrl: string
  sortName?: string
  sortUrl?: string
  page?: number
}) {
  return http.post<{ first: RssArticle[]; second: null }>('/getRssArticles', params).then((r) => r.data)
}

export function getRssContent(params: {
  sourceUrl: string
  link: string
  origin: string
}) {
  return http.post<string>('/getRssContent', params).then((r) => r.data)
}

export function readRemoteRssSourceFile(url: string) {
  return http.post<string[]>('/readRemoteRssSourceFile', { url }).then((r) => r.data)
}

export function readRssSourceFile(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return http.post<RssSource[]>('/readRssSourceFile', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  }).then((r) => r.data)
}
