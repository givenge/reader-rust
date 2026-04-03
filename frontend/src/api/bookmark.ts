import http from './http'
import type { Bookmark } from '../types'

/**
 * 获取所有书签
 */
export function getBookmarks() {
  return http.get<Bookmark[]>('/getBookmarks').then((r) => r.data)
}

/**
 * 保存单个书签
 */
export function saveBookmark(bookmark: Bookmark) {
  return http.post<string>('/saveBookmark', bookmark).then((r) => r.data)
}

/**
 * 批量保存书签
 */
export function saveBookmarks(bookmarks: Bookmark[]) {
  return http.post<string>('/saveBookmarks', bookmarks).then((r) => r.data)
}

/**
 * 删除单个书签
 */
export function deleteBookmark(bookmark: Bookmark) {
  return http.post<string>('/deleteBookmark', bookmark).then((r) => r.data)
}

/**
 * 批量删除书签
 */
export function deleteBookmarks(bookmarks: Bookmark[]) {
  return http.post<string>('/deleteBookmarks', bookmarks).then((r) => r.data)
}
