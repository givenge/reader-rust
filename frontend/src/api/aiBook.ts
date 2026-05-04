import http from './http'
import type { AiBookMemory } from '../types'

export function getAiBookMemory(bookUrl: string) {
  return http.post<AiBookMemory | null>('/getAiBookMemory', { bookUrl }).then((r) => r.data)
}

export function saveAiBookMemory(memory: AiBookMemory) {
  return http.post<AiBookMemory>('/saveAiBookMemory', memory).then((r) => r.data)
}

export function deleteAiBookMemory(bookUrl: string) {
  return http.post<{ deleted: boolean }>('/deleteAiBookMemory', { bookUrl }).then((r) => r.data)
}
