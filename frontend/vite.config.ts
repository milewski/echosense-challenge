import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { viteSingleFile } from 'vite-plugin-singlefile'
import { createHtmlPlugin } from 'vite-plugin-html'

export default defineConfig({
    esbuild: { legalComments: 'none' },
    plugins: [
        vue(),
        viteSingleFile(),
        createHtmlPlugin({ minify: true }),
    ],
})
