import http from './http'
import type { BookSource } from '../types'

export function getBookSources() {
  return http.get<BookSource[]>('/getBookSources').then((r) => r.data)
}

export function loginBookSource(bookSourceUrl: string) {
  return http.post<{
    success: boolean
    status: number
    url: string
    checkResult?: string | null
    bodyPreview?: string
    bodyHtml?: string
  }>('/loginBookSource', { bookSourceUrl }).then((r) => r.data)
}

export function getBookSource(bookSourceUrl: string) {
  return http.post<BookSource>('/getBookSource', { bookSourceUrl }).then((r) => r.data)
}

export function saveBookSource(source: BookSource) {
  return http.post<{ saved: boolean }>('/saveBookSource', source).then((r) => r.data)
}

export function saveBookSources(sources: BookSource[]) {
  return http.post<{ saved: boolean; count: number }>('/saveBookSources', sources).then((r) => r.data)
}

export function deleteBookSource(bookSourceUrl: string) {
  return http.post<{ deleted: boolean }>('/deleteBookSource', { bookSourceUrl }).then((r) => r.data)
}

export function deleteBookSources(sources: { bookSourceUrl: string }[]) {
  return http.post<{ deleted: boolean }>('/deleteBookSources', sources).then((r) => r.data)
}

export function deleteAllBookSources() {
  return http.post<{ deleted: boolean }>('/deleteAllBookSources').then((r) => r.data)
}

export function readRemoteSourceFile(url: string) {
  return http.post<string[]>('/readRemoteSourceFile', { url }).then((r) => r.data)
}

export function readSourceFile(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return http.post<BookSource[]>('/readSourceFile', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  }).then((r) => r.data)
}

export function getInvalidBookSources() {
  return http.post<unknown[]>('/getInvalidBookSources').then((r) => r.data)
}
