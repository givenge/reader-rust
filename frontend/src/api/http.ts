import axios from 'axios'
import type { ApiResponse } from '../types'

let lastNeedLoginDispatchAt = 0

function dispatchNeedLogin() {
  const now = Date.now()
  if (now - lastNeedLoginDispatchAt < 1500) return
  lastNeedLoginDispatchAt = now
  window.dispatchEvent(new CustomEvent('need-login'))
}

const http = axios.create({
  baseURL: '/reader3',
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' },
})

// ─── Request interceptor: attach token ───
http.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken')
  if (token) {
    config.headers.Authorization = token
  }
  return config
})

// ─── Response interceptor: unwrap ApiResponse ───
http.interceptors.response.use(
  (response) => {
    const data = response.data as ApiResponse
    // Some endpoints return raw data (cover, file etc.)
    if (data.isSuccess === undefined) {
      return response
    }
    if (!data.isSuccess) {
      if (data.errorMsg === 'NEED_LOGIN' || data.data === 'NEED_LOGIN') {
        dispatchNeedLogin()
      }
      return Promise.reject(new Error(data.errorMsg || '请求失败'))
    }
    // Return unwrapped data
    response.data = data.data
    return response
  },
  (error) => {
    const data = error.response?.data as Partial<ApiResponse> | undefined
    if (data && typeof data === 'object') {
      if (data.errorMsg === 'NEED_LOGIN' || data.data === 'NEED_LOGIN') {
        dispatchNeedLogin()
      }
      if (typeof data.errorMsg === 'string' && data.errorMsg.trim()) {
        return Promise.reject(new Error(data.errorMsg))
      }
    }
    if (error.response?.status === 401) {
      dispatchNeedLogin()
    }
    return Promise.reject(new Error(error.message || '请求失败'))
  }
)

export default http
