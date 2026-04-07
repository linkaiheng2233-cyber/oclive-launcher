import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  base: './',
  clearScreen: false,
  server: {
    // 与 tauri.conf.json 的 devPath（127.0.0.1）一致，避免 Tauri 卡在 Waiting for dev server
    host: '127.0.0.1',
    port: 5174,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
})
