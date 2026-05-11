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
      secureKeyRequired: boolean
      adminAuthorized: boolean
    }>('/getUserInfo')
    .then((r) => r.data)
}

export function getUserList() {
  return http.get<UserInfo[]>('/getUserList').then((r) => r.data)
}

export function addUser(username: string, password: string) {
  return http.post<UserInfo[]>('/addUser', { username, password }).then((r) => r.data)
}

export function resetPassword(username: string, password: string) {
  return http.post<string>('/resetPassword', { username, password }).then((r) => r.data)
}

export function changePassword(oldPassword: string, newPassword: string) {
  return http.post<string>('/changePassword', { oldPassword, newPassword }).then((r) => r.data)
}

export function updateUser(
  username: string,
  payload: { enableWebdav?: boolean; enableLocalStore?: boolean; enableAiModel?: boolean },
) {
  return http.post<UserInfo[]>('/updateUser', { username, ...payload }).then((r) => r.data)
}

export function deleteUsers(usernames: string[]) {
  return http.post<UserInfo[]>('/deleteUsers', usernames).then((r) => r.data)
}
