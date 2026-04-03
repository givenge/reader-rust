import http from './http'
import type { ReplaceRule } from '../types'

/**
 * 获取所有净化规则
 */
export function getReplaceRules() {
  return http.get<ReplaceRule[]>('/getReplaceRules').then((r) => r.data)
}

/**
 * 保存单个净化规则
 */
export function saveReplaceRule(rule: ReplaceRule) {
  return http.post<string>('/saveReplaceRule', rule).then((r) => r.data)
}

/**
 * 批量保存净化规则
 */
export function saveReplaceRules(rules: ReplaceRule[]) {
  return http.post<string>('/saveReplaceRules', rules).then((r) => r.data)
}

/**
 * 删除单个净化规则
 */
export function deleteReplaceRule(rule: ReplaceRule) {
  return http.post<string>('/deleteReplaceRule', rule).then((r) => r.data)
}

/**
 * 批量删除净化规则
 */
export function deleteReplaceRules(rules: ReplaceRule[]) {
  return http.post<string>('/deleteReplaceRules', rules).then((r) => r.data)
}
