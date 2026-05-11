export const SECURE_KEY_STORAGE_KEY = 'secureKey'

type StorageLike = Pick<Storage, 'getItem'>

export function buildAuthHeaderValues(storage: StorageLike) {
  const accessToken = storage.getItem('accessToken')?.trim() || ''
  const secureKey = storage.getItem(SECURE_KEY_STORAGE_KEY)?.trim() || ''

  return {
    accessToken: accessToken || undefined,
    secureKey: secureKey || undefined,
  }
}

export function appendAuthQueryParams(params: URLSearchParams, storage: StorageLike = localStorage) {
  const { accessToken, secureKey } = buildAuthHeaderValues(storage)
  if (accessToken) {
    params.set('accessToken', accessToken)
  }
  if (secureKey) {
    params.set('secureKey', secureKey)
  }
}

export function computeNeedSecureKey(params: {
  secure: boolean
  secureKeyRequired: boolean
  adminAuthorized: boolean
}) {
  return params.secure && params.secureKeyRequired && !params.adminAuthorized
}

export function readStoredSecureKey(storage: StorageLike = localStorage) {
  return storage.getItem(SECURE_KEY_STORAGE_KEY)?.trim() || ''
}
