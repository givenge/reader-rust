import http from './http'

export interface WebdavFileEntry {
  name: string
  size: number
  path: string
  lastModified: number
  isDirectory: boolean
}

export function getWebdavFileList(path = '/') {
  return http.get<WebdavFileEntry[]>('/getWebdavFileList', {
    params: { path },
  }).then((r) => r.data)
}

export function getWebdavFileText(path: string) {
  return http.get<string>('/getWebdavFile', {
    params: { path },
    responseType: 'text',
    transformResponse: [(value) => value],
  }).then((r) => r.data as unknown as string)
}

export function getWebdavFileBlob(path: string) {
  return http.get<Blob>('/getWebdavFile', {
    params: { path },
    responseType: 'blob',
  }).then((r) => r.data)
}

export function uploadFilesToWebdav(files: Array<{ file: Blob; name: string }>, path = '/') {
  const formData = new FormData()
  formData.append('path', path)
  files.forEach((item, index) => {
    formData.append(`file${index}`, item.file, item.name)
  })
  return http.post<WebdavFileEntry[]>('/uploadFileToWebdav', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  }).then((r) => r.data)
}

export function uploadTextToWebdav(content: string, filename: string, path = '/') {
  const blob = new Blob([content], { type: 'application/json;charset=utf-8' })
  return uploadFilesToWebdav([{ file: blob, name: filename }], path)
}

export function deleteWebdavFile(path: string) {
  return http.post<string>('/deleteWebdavFile', { path }).then((r) => r.data)
}

export function deleteWebdavFileList(paths: string[]) {
  return http.post<string>('/deleteWebdavFileList', { path: paths }).then((r) => r.data)
}

// ==================== 远程 WebDAV 客户端 ====================

export interface WebdavConfig {
    serverUrl: string
    username: string
    password: string
    enabled: boolean
}

export function saveWebdavConfig(config: WebdavConfig) {
    return http.post('/saveWebdavConfig', config)
}

export function getWebdavConfig() {
    return http.get<WebdavConfig>('/getWebdavConfig')
}

export function testWebdavConnection(config: WebdavConfig) {
    return http.post<TestResult>('/testWebdavConnection', config)
}

export function backupToRemoteWebdav(path = '/reader-backups/') {
    return http.post<BackupResult>('/backupToRemoteWebdav', { path })
}

export function getRemoteWebdavFileList(path = '/') {
    return http.get<RemoteWebdavFileEntry[]>('/getRemoteWebdavFileList', {
        params: { path },
    })
}

export function restoreFromRemoteWebdav(path: string) {
    return http.post<RestoreResult>('/restoreFromRemoteWebdav', { path })
}

export interface TestResult {
    connected: boolean
    message: string
}

export interface BackupResult {
    fileName: string
    size: number
}

export interface RemoteWebdavFileEntry {
    name: string
    size: number
    path: string
    lastModified: number
    isDirectory: boolean
}

export interface RestoreResult {
    restored: boolean
}
