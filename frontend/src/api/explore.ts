import http from './http'
import type { SearchBook } from '../types'

export interface ExploreBookParams {
  bookSourceUrl: string
  ruleFindUrl: string
  page?: number
}

/**
 * 获取探索发现的书本列表
 * 对应后端 /reader3/exploreBook 接口
 */
export function exploreBook(params: ExploreBookParams) {
  return http.post<SearchBook[]>('/exploreBook', params).then((r) => r.data)
}
