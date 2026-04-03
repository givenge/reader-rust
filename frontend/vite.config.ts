import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/reader3': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules/opencc-js')) {
            return 'opencc'
          }
          if (id.includes('node_modules')) {
            return 'vendor'
          }
          if (id.includes('/src/components/reader/') || id.includes('/src/composables/useReader')) {
            return 'reader-panels'
          }
        },
      },
    },
  },
})
