import type {
  AiBookConfig,
  AiBookMap,
  AiBookMemory,
  AiBookModelUpdate,
  Book,
  BookChapter,
} from '../types'
import { isAiBookConfigReady } from './aiBookConfig'

export type AiBookChatMessage = {
  role: 'system' | 'user'
  content: string
}

export interface BuildPromptParams {
  bookName: string
  chapterTitle: string
  chapterIndex: number
  chapterContent: string
  memory: AiBookMemory
}

export interface GenerateMemoryParams {
  config: AiBookConfig
  book: Book
  chapter: BookChapter
  chapterContent: string
  memory: AiBookMemory
  fetchImpl?: typeof fetch
}

export interface GenerateMapParams {
  config: AiBookConfig
  prompt: string
  fetchImpl?: typeof fetch
}

export interface UploadGeneratedMapParams {
  b64Json?: string
  imageUrl?: string
  filename: string
  fetchImpl?: typeof fetch
}

interface OpenAIChatResponse {
  choices?: Array<{
    message?: {
      content?: string
    }
  }>
}

interface OpenAIImageResponse {
  data?: Array<{
    b64_json?: string
    url?: string
  }>
}

interface AiBookRawModelUpdate {
  memory?: Partial<AiBookMemory>
  shouldRegenerateMap?: boolean
  mapPrompt?: string
  summary?: string
  worldview?: unknown
  characters?: unknown
  relationships?: unknown
  locations?: unknown
  mapDirty?: boolean
}

export function shouldRunAiBookAutoUpdate(
  memory: AiBookMemory | null | undefined,
  completedChapterIndex: number,
  config: AiBookConfig,
) {
  if (!memory?.enabled) return false
  if (!isAiBookConfigReady(config)) return false
  if (typeof memory.processedChapterIndex === 'number' && memory.processedChapterIndex >= completedChapterIndex) {
    return false
  }
  return true
}

export function buildAiBookPromptMessages({
  bookName,
  chapterTitle,
  chapterIndex,
  chapterContent,
  memory,
}: BuildPromptParams): AiBookChatMessage[] {
  return [
    {
      role: 'system',
      content: [
        '你是小说阅读辅助资料整理器。',
        '不得使用未读章节，不得补充未来剧情，不得剧透。',
        '只基于上一版结构化记忆和当前章节更新世界观、角色当前状态、人物关系、地点和地图信息。',
        '无法确认的信息必须标记为“推断”或“未知”。',
        '返回严格 JSON，不要 Markdown，不要解释。',
      ].join('\n'),
    },
    {
      role: 'user',
      content: JSON.stringify({
        task: 'incremental-ai-book-memory-update',
        schema: {
          memory: {
            summary: 'string',
            worldview: [{ title: 'string', content: 'string', confidence: '已知|推断|未知' }],
            characters: [{ name: 'string', aliases: ['string'], status: 'string', faction: 'string', location: 'string', description: 'string', lastSeenChapter: 'string' }],
            relationships: [{ source: 'string', target: 'string', relation: 'string', status: 'string', description: 'string' }],
            locations: [{ name: 'string', kind: 'string', description: 'string', status: 'string', relatedCharacters: ['string'], firstSeenChapter: 'string' }],
          },
          shouldRegenerateMap: 'boolean',
          mapPrompt: 'string when map should be regenerated',
        },
        bookName,
        chapter: {
          index: chapterIndex,
          title: chapterTitle,
          content: chapterContent.slice(0, 24000),
        },
        previousMemory: memory,
      }),
    },
  ]
}

export async function requestAiBookMemoryUpdate({
  config,
  book,
  chapter,
  chapterContent,
  memory,
  fetchImpl = fetch,
}: GenerateMemoryParams): Promise<AiBookModelUpdate> {
  const response = await fetchImpl(`${normalizeBaseUrl(config.baseUrl)}/v1/chat/completions`, {
    method: 'POST',
    headers: buildModelHeaders(config),
    body: JSON.stringify({
      model: config.textModel,
      messages: buildAiBookPromptMessages({
        bookName: book.name,
        chapterTitle: chapter.title,
        chapterIndex: chapter.index,
        chapterContent,
        memory,
      }),
      temperature: 0.2,
      response_format: { type: 'json_object' },
    }),
  })

  if (!response.ok) {
    throw new Error(await readModelError(response, 'AI 资料生成失败'))
  }

  const data = await response.json() as OpenAIChatResponse
  const content = data.choices?.[0]?.message?.content
  if (!content) {
    throw new Error('AI 资料生成结果为空')
  }

  return coerceModelUpdate(parseJsonContent(content), memory, book, chapter)
}

export async function requestAiBookMapImage({
  config,
  prompt,
  fetchImpl = fetch,
}: GenerateMapParams) {
  const response = await fetchImpl(`${normalizeBaseUrl(config.baseUrl)}/v1/images/generations`, {
    method: 'POST',
    headers: buildModelHeaders(config),
    body: JSON.stringify({
      model: config.imageModel,
      prompt,
      size: config.imageSize || '1024x1024',
      response_format: 'b64_json',
      n: 1,
    }),
  })

  if (!response.ok) {
    throw new Error(await readModelError(response, '地图生成失败'))
  }

  const data = await response.json() as OpenAIImageResponse
  const first = data.data?.[0]
  if (!first?.b64_json && !first?.url) {
    throw new Error('地图生成结果为空')
  }
  return {
    b64Json: first.b64_json,
    imageUrl: first.url,
  }
}

export async function uploadGeneratedMap({
  b64Json,
  imageUrl,
  filename,
  fetchImpl = fetch,
}: UploadGeneratedMapParams) {
  const blob = b64Json
    ? base64ToBlob(b64Json, 'image/png')
    : await fetchImageBlob(imageUrl || '', fetchImpl)

  const formData = new FormData()
  formData.append('file', blob, filename)

  const headers: Record<string, string> = {}
  const token = safeLocalStorageGet('accessToken')
  if (token) {
    headers.Authorization = token
  }

  const response = await fetchImpl('/reader3/uploadFile?type=ai-maps', {
    method: 'POST',
    headers,
    body: formData,
  })
  if (!response.ok) {
    throw new Error(await readModelError(response, '地图上传失败'))
  }

  const data = await response.json() as {
    isSuccess?: boolean
    errorMsg?: string
    data?: string[]
  }
  if (data.isSuccess === false) {
    throw new Error(data.errorMsg || '地图上传失败')
  }
  const url = Array.isArray(data.data) ? data.data[0] : ''
  if (!url) {
    throw new Error('地图上传结果为空')
  }
  return url
}

export function createEmptyAiBookMemory(book: Book): AiBookMemory {
  return {
    bookUrl: book.bookUrl,
    bookName: book.name,
    author: book.author,
    enabled: false,
    processedChapterIndex: undefined,
    processedChapterTitle: undefined,
    updatedAt: Date.now(),
    summary: '',
    worldview: [],
    characters: [],
    relationships: [],
    locations: [],
    map: null,
    mapDirty: false,
    lastError: undefined,
  }
}

export function applyMapToMemory(memory: AiBookMemory, map: AiBookMap): AiBookMemory {
  return {
    ...memory,
    map,
    mapDirty: false,
    updatedAt: Date.now(),
  }
}

function coerceModelUpdate(raw: AiBookRawModelUpdate, previous: AiBookMemory, book: Book, chapter: BookChapter): AiBookModelUpdate {
  const rawMemory = raw.memory || raw
  const memory: AiBookMemory = {
    ...previous,
    ...rawMemory,
    bookUrl: book.bookUrl,
    bookName: book.name,
    author: book.author,
    enabled: previous.enabled,
    processedChapterIndex: chapter.index,
    processedChapterTitle: chapter.title,
    updatedAt: Date.now(),
    summary: typeof rawMemory.summary === 'string' ? rawMemory.summary : previous.summary || '',
    worldview: Array.isArray(rawMemory.worldview) ? rawMemory.worldview as AiBookMemory['worldview'] : previous.worldview,
    characters: Array.isArray(rawMemory.characters) ? rawMemory.characters as AiBookMemory['characters'] : previous.characters,
    relationships: Array.isArray(rawMemory.relationships) ? rawMemory.relationships as AiBookMemory['relationships'] : previous.relationships,
    locations: Array.isArray(rawMemory.locations) ? rawMemory.locations as AiBookMemory['locations'] : previous.locations,
    map: previous.map || null,
    mapDirty: Boolean(raw.shouldRegenerateMap || raw.mapDirty),
    lastError: undefined,
  }

  return {
    memory,
    shouldRegenerateMap: Boolean(raw.shouldRegenerateMap || raw.mapDirty),
    mapPrompt: raw.mapPrompt,
  }
}

function buildModelHeaders(config: AiBookConfig) {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (config.apiKey.trim()) {
    headers.Authorization = `Bearer ${config.apiKey.trim()}`
  }
  return headers
}

function normalizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, '')
}

function parseJsonContent(content: string): AiBookRawModelUpdate {
  const trimmed = content.trim()
  const json = trimmed
    .replace(/^```(?:json)?/i, '')
    .replace(/```$/i, '')
    .trim()
  return JSON.parse(json) as AiBookRawModelUpdate
}

async function readModelError(response: Response, fallback: string) {
  try {
    const contentType = response.headers.get('content-type') || ''
    if (contentType.includes('application/json')) {
      const data = await response.json() as {
        error?: { message?: string }
        errorMsg?: string
      }
      return data.error?.message || data.errorMsg || `${fallback} (${response.status})`
    }
    const text = await response.text()
    return text.trim() || `${fallback} (${response.status})`
  } catch {
    return `${fallback} (${response.status})`
  }
}

function base64ToBlob(value: string, contentType: string) {
  const binary = atob(value)
  const bytes = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index)
  }
  return new Blob([bytes], { type: contentType })
}

async function fetchImageBlob(imageUrl: string, fetchImpl: typeof fetch) {
  if (!imageUrl) {
    throw new Error('地图图片地址为空')
  }
  const response = await fetchImpl(imageUrl)
  if (!response.ok) {
    throw new Error(await readModelError(response, '地图图片下载失败'))
  }
  return response.blob()
}

function safeLocalStorageGet(key: string) {
  try {
    return localStorage.getItem(key) || ''
  } catch {
    return ''
  }
}
