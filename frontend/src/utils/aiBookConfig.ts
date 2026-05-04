import type { AiBookConfig } from '../types'

export const DEFAULT_AI_BOOK_CONFIG: AiBookConfig = {
  baseUrl: '',
  apiKey: '',
  textModel: 'gpt-4o-mini',
  imageModel: 'gpt-image-1',
  imageSize: '1024x1024',
}

const AI_BOOK_CONFIG_PREFIX = 'reader-ai-book-config:'

function normalizeUsername(username?: string | null) {
  return (username || 'default').trim() || 'default'
}

function storageKey(username?: string | null) {
  return `${AI_BOOK_CONFIG_PREFIX}${normalizeUsername(username)}`
}

function normalizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, '')
}

export function getAiBookConfig(username?: string | null): AiBookConfig {
  try {
    const raw = localStorage.getItem(storageKey(username))
    if (!raw) return { ...DEFAULT_AI_BOOK_CONFIG }
    const parsed = JSON.parse(raw) as Partial<AiBookConfig>
    return {
      ...DEFAULT_AI_BOOK_CONFIG,
      ...parsed,
      baseUrl: normalizeBaseUrl(parsed.baseUrl || DEFAULT_AI_BOOK_CONFIG.baseUrl),
      apiKey: (parsed.apiKey || '').trim(),
      textModel: (parsed.textModel || DEFAULT_AI_BOOK_CONFIG.textModel).trim(),
      imageModel: (parsed.imageModel || DEFAULT_AI_BOOK_CONFIG.imageModel).trim(),
      imageSize: (parsed.imageSize || DEFAULT_AI_BOOK_CONFIG.imageSize).trim(),
    }
  } catch {
    return { ...DEFAULT_AI_BOOK_CONFIG }
  }
}

export function saveAiBookConfig(username: string | null | undefined, config: AiBookConfig) {
  const next: AiBookConfig = {
    ...DEFAULT_AI_BOOK_CONFIG,
    ...config,
    baseUrl: normalizeBaseUrl(config.baseUrl),
    apiKey: config.apiKey.trim(),
    textModel: config.textModel.trim(),
    imageModel: config.imageModel.trim(),
    imageSize: config.imageSize.trim(),
  }
  localStorage.setItem(storageKey(username), JSON.stringify(next))
  return next
}

export function isAiBookConfigReady(config: AiBookConfig) {
  return Boolean(config.baseUrl.trim() && config.textModel.trim())
}

export function aiBookConfigStorageKey(username?: string | null) {
  return storageKey(username)
}
