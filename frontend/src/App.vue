<template>
  <div id="app">
    <AppHeader v-if="showHeader" @explore="router.push('/explore')" @rss="router.push('/rss')" />
    <router-view />
    <SettingsDrawer v-model="appStore.showSettingsDrawer" />
    <LoginModal v-model="appStore.showLoginModal" />
    <SourceManager v-model="appStore.showSourceManager" />
    <WebdavManager v-model="appStore.showWebdavManager" />

    <!-- Toast notifications -->
    <div class="toast-container">
      <TransitionGroup name="slide-up">
        <div
          v-for="toast in appStore.toasts"
          :key="toast.id"
          class="toast"
          :class="toast.type"
        >
          {{ toast.message }}
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from './stores/app'
import AppHeader from './components/AppHeader.vue'
import SettingsDrawer from './components/SettingsDrawer.vue'
import LoginModal from './components/LoginModal.vue'
import SourceManager from './components/SourceManager.vue'
import WebdavManager from './components/WebdavManager.vue'

const route = useRoute()
const router = useRouter()
const appStore = useAppStore()

const showHeader = computed(() => route.name !== 'reader')

// Listen for need-login events from API layer
window.addEventListener('need-login', () => {
  appStore.showLoginModal = true
})
</script>

<style>
#app {
  min-height: 100vh;
}
</style>
