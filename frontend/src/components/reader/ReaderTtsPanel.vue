<template>
  <Transition name="slide-up">
    <div v-if="show" class="tts-controls" :style="{ background: theme.popup, color: theme.fontColor }">
      <div class="tts-head">
        <div class="tts-info">正在朗读: {{ chapterTitle }}</div>
        <button class="tts-close" @click="$emit('close')">×</button>
      </div>
      <div class="tts-btns">
        <button @click="$emit('prev')">上一段</button>
        <button @click="$emit('pause')">{{ isPaused ? '恢复' : '暂停' }}</button>
        <button @click="$emit('stop')">停止</button>
        <button @click="$emit('next')">下一段</button>
      </div>
      <select class="tts-voice-select" :value="voiceName" @change="$emit('voice-change', ($event.target as HTMLSelectElement).value)">
        <option value="">系统默认</option>
        <option v-for="voice in voices" :key="voice.name" :value="voice.name">
          {{ voice.name }} ({{ voice.lang }})
        </option>
      </select>
      <div class="tts-tuning">
        <div class="tts-stepper">
          <span class="tts-label">语速</span>
          <button @click="$emit('rate-change', -0.1)">-</button>
          <span>{{ rate.toFixed(1) }}</span>
          <button @click="$emit('rate-change', 0.1)">+</button>
        </div>
        <div class="tts-stepper">
          <span class="tts-label">语调</span>
          <button @click="$emit('pitch-change', -0.1)">-</button>
          <span>{{ pitch.toFixed(1) }}</span>
          <button @click="$emit('pitch-change', 0.1)">+</button>
        </div>
      </div>
      <div class="tts-timer-row">
        <span class="tts-label">定时停止</span>
        <div class="tts-timer-actions">
          <button :class="{ active: stopAfterMinutes === 0 }" @click="$emit('timer-change', 0)">关闭</button>
          <button :class="{ active: stopAfterMinutes === 15 }" @click="$emit('timer-change', 15)">15分</button>
          <button :class="{ active: stopAfterMinutes === 30 }" @click="$emit('timer-change', 30)">30分</button>
          <button :class="{ active: stopAfterMinutes === 60 }" @click="$emit('timer-change', 60)">60分</button>
        </div>
        <div v-if="timerText" class="tts-timer-text">{{ timerText }}</div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import type { ThemePreset } from '../../stores/reader'

defineProps<{
  show: boolean
  theme: ThemePreset | { popup: string; fontColor: string }
  chapterTitle?: string
  isPaused: boolean
  voices: SpeechSynthesisVoice[]
  voiceName: string
  rate: number
  pitch: number
  stopAfterMinutes: number
  timerText: string
}>()

defineEmits<{
  close: []
  prev: []
  pause: []
  stop: []
  next: []
  'voice-change': [value: string]
  'rate-change': [delta: number]
  'pitch-change': [delta: number]
  'timer-change': [minutes: number]
}>()
</script>

<style scoped>
.tts-controls {
  position: fixed;
  bottom: calc(24px + var(--safe-area-bottom));
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: var(--radius-full);
  box-shadow: 0 4px 20px rgba(0,0,0,0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  z-index: 30;
  min-width: 280px;
}

.tts-head {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.tts-info {
  font-size: 12px;
  opacity: 0.7;
}

.tts-close {
  border: none;
  background: transparent;
  color: inherit;
  font-size: 20px;
  line-height: 1;
  opacity: 0.6;
  cursor: pointer;
}

.tts-btns {
  display: flex;
  gap: 16px;
}

.tts-btns button {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 4px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.tts-voice-select {
  width: 100%;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(0, 0, 0, 0.08);
  background: rgba(255, 255, 255, 0.65);
  color: inherit;
}

.tts-tuning {
  width: 100%;
  display: flex;
  gap: 10px;
}

.tts-stepper {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.04);
  font-size: 12px;
}

.tts-stepper button {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: var(--color-primary);
  color: #fff;
  cursor: pointer;
}

.tts-label {
  opacity: 0.7;
}

.tts-timer-row {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tts-timer-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tts-timer-actions button {
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.04);
  color: inherit;
  padding: 6px 10px;
  font-size: 12px;
  cursor: pointer;
}

.tts-timer-actions button.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

.tts-timer-text {
  font-size: 12px;
  opacity: 0.65;
}

@media (max-width: 768px) {
  .tts-controls {
    bottom: calc(80px + var(--safe-area-bottom));
  }
}
</style>
