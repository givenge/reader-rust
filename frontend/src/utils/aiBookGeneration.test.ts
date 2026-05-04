import { describe, expect, it, vi } from 'vitest'
import type { AiBookConfig, AiBookMemory } from '../types'
import {
  buildAiBookPromptMessages,
  shouldRunAiBookAutoUpdate,
  uploadGeneratedMap,
} from './aiBookGeneration'

const readyConfig: AiBookConfig = {
  baseUrl: 'http://localhost:8825',
  apiKey: '',
  textModel: 'gpt-4o-mini',
  imageModel: 'gpt-image-1',
  imageSize: '1024x1024',
}

describe('aiBookGeneration', () => {
  it('skips auto update without config, disabled memory, or already processed chapter', () => {
    const memory: AiBookMemory = {
      bookUrl: 'book-1',
      enabled: true,
      processedChapterIndex: 4,
      worldview: [],
      characters: [],
      relationships: [],
      locations: [],
      updatedAt: 0,
    }

    expect(shouldRunAiBookAutoUpdate(memory, 5, { ...readyConfig, baseUrl: '' })).toBe(false)
    expect(shouldRunAiBookAutoUpdate({ ...memory, enabled: false }, 5, readyConfig)).toBe(false)
    expect(shouldRunAiBookAutoUpdate(memory, 4, readyConfig)).toBe(false)
    expect(shouldRunAiBookAutoUpdate(memory, 5, readyConfig)).toBe(true)
  })

  it('builds spoiler-safe incremental memory prompts', () => {
    const messages = buildAiBookPromptMessages({
      bookName: '山海旧事',
      chapterTitle: '第八章 北境',
      chapterIndex: 7,
      chapterContent: '主角抵达北境，只知道旧神传说真假未明。',
      memory: {
        bookUrl: 'book-1',
        enabled: true,
        processedChapterIndex: 6,
        summary: '主角离开帝都。',
        worldview: [],
        characters: [],
        relationships: [],
        locations: [],
        updatedAt: 0,
      },
    })

    const serialized = JSON.stringify(messages)
    expect(serialized).toContain('不得使用未读章节')
    expect(serialized).toContain('只基于上一版结构化记忆和当前章节')
    expect(serialized).toContain('第八章 北境')
  })

  it('uploads generated base64 maps through reader asset endpoint', async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        isSuccess: true,
        data: ['/assets/alice/ai-maps/map.png'],
      }),
    })) as unknown as typeof fetch

    const url = await uploadGeneratedMap({
      b64Json: btoa('fake-png'),
      filename: 'map.png',
      fetchImpl: fetchMock,
    })

    expect(url).toBe('/assets/alice/ai-maps/map.png')
    expect(fetchMock).toHaveBeenCalledWith(
      '/reader3/uploadFile?type=ai-maps',
      expect.objectContaining({
        method: 'POST',
        body: expect.any(FormData),
      }),
    )
  })
})
