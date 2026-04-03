import axios from 'axios'
import type { ApiResponse } from '../types'

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
        localStorage.removeItem('accessToken')
        window.dispatchEvent(new CustomEvent('need-login'))
      }
      return Promise.reject(new Error(data.errorMsg || '请求失败'))
    }
    // Return unwrapped data
    response.data = data.data
    return response
  },
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('accessToken')
      window.dispatchEvent(new CustomEvent('need-login'))
    }
    return Promise.reject(error)
  }
)

export default http
