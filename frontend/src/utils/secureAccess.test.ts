import { describe, expect, it } from 'vitest'
import {
  appendAuthQueryParams,
  buildAuthHeaderValues,
  computeNeedSecureKey,
  SECURE_KEY_STORAGE_KEY,
} from './secureAccess'

describe('secureAccess', () => {
  it('adds stored secure key to auth headers', () => {
    const storage = {
      getItem(key: string) {
        if (key === 'accessToken') return 'token-123'
        if (key === SECURE_KEY_STORAGE_KEY) return 'secure-456'
        return null
      },
    }

    expect(buildAuthHeaderValues(storage)).toEqual({
      accessToken: 'token-123',
      secureKey: 'secure-456',
    })
  })

  it('adds stored auth values to event source query params', () => {
    const storage = {
      getItem(key: string) {
        if (key === 'accessToken') return 'alice-token'
        if (key === SECURE_KEY_STORAGE_KEY) return 'admin-key'
        return null
      },
    }
    const params = new URLSearchParams()

    appendAuthQueryParams(params, storage)

    expect(params.get('accessToken')).toBe('alice-token')
    expect(params.get('secureKey')).toBe('admin-key')
  })

  it('requires secure key only when the server requires it and current request is not admin authorized', () => {
    expect(computeNeedSecureKey({
      secure: true,
      secureKeyRequired: true,
      adminAuthorized: false,
    })).toBe(true)

    expect(computeNeedSecureKey({
      secure: true,
      secureKeyRequired: true,
      adminAuthorized: true,
    })).toBe(false)

    expect(computeNeedSecureKey({
      secure: true,
      secureKeyRequired: false,
      adminAuthorized: false,
    })).toBe(false)
  })
})
