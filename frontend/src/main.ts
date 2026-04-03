import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './styles/global.css'
import { useAppStore } from './stores/app'
import { registerPwa } from './utils/pwa'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
registerPwa(useAppStore(pinia))

app.mount('#app')
