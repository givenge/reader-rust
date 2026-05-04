import { describe, expect, it, beforeEach } from 'vitest'
import {
  DEFAULT_AI_BOOK_CONFIG,
  getAiBookConfig,
  isAiBookConfigReady,
  saveAiBookConfig,
} from './aiBookConfig'

beforeEach(() => {
  installLocalStorage()
  localStorage.clear()
})

describe('aiBookConfig', () => {
  it('persists model config by username', () => {
    saveAiBookConfig('alice', {
      baseUrl: 'https://llm.example.test/',
      apiKey: 'alice-key',
      textModel: 'story-text',
      imageModel: 'story-image',
      imageSize: '1024x1024',
    })
    saveAiBookConfig('bob', {
      baseUrl: 'https://other.example.test',
      apiKey: 'bob-key',
      textModel: 'other-text',
      imageModel: 'other-image',
      imageSize: '1792x1024',
    })

    expect(getAiBookConfig('alice')).toMatchObject({
      baseUrl: 'https://llm.example.test',
      apiKey: 'alice-key',
      textModel: 'story-text',
      imageModel: 'story-image',
      imageSize: '1024x1024',
    })
    expect(getAiBookConfig('bob').apiKey).toBe('bob-key')
  })

  it('falls back to defaults and reports readiness', () => {
    expect(getAiBookConfig('guest')).toEqual(DEFAULT_AI_BOOK_CONFIG)
    expect(isAiBookConfigReady(getAiBookConfig('guest'))).toBe(false)

    saveAiBookConfig('guest', {
      baseUrl: 'http://localhost:8825',
      apiKey: '',
      textModel: 'gpt-4o-mini',
      imageModel: 'gpt-image-1',
      imageSize: '1024x1024',
    })
    expect(isAiBookConfigReady(getAiBookConfig('guest'))).toBe(true)
  })
})

function installLocalStorage() {
  const memory = new Map<string, string>()
  Object.defineProperty(globalThis, 'localStorage', {
    value: {
      getItem: (key: string) => memory.get(key) || null,
      setItem: (key: string, value: string) => memory.set(key, value),
      removeItem: (key: string) => memory.delete(key),
      clear: () => memory.clear(),
    },
    configurable: true,
  })
}
