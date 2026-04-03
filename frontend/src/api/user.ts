import http from './http'
import type { UserInfo } from '../types'

export function login(username: string, password: string) {
  return http
    .post<UserInfo>('/login', { username, password, isLogin: true })
    .then((r) => r.data)
}

export function register(username: string, password: string, code?: string) {
  return http
    .post<UserInfo>('/login', { username, password, isLogin: false, code })
    .then((r) => r.data)
}

export function logout() {
  return http.post('/logout').catch(() => {
    // Logout always clears local state
  })
}

export function getUserInfo() {
  return http
    .get<{
      userInfo: UserInfo | null
      secure: boolean
      secureKey: boolean
    }>('/getUserInfo')
    .then((r) => r.data)
}
