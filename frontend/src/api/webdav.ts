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
