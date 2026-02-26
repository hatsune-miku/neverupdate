import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

import react from '@vitejs/plugin-react'

const rootDir = fileURLToPath(new URL('.', import.meta.url))

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(rootDir, 'src'),
    },
  },
  server: {
    port: 1214,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
