import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { cp, readFile, stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'

const frontendRoot = fileURLToPath(new URL('.', import.meta.url))
const ideHostRoot = path.resolve(frontendRoot, '../ide/host')
const ideCoreRoot = path.resolve(frontendRoot, '../ide/vscode-web')

const contentTypes: Record<string, string> = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.ico': 'image/x-icon',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.map': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.ttf': 'font/ttf',
  '.wasm': 'application/wasm',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
}

function packwandIdeAssets() {
  return {
    name: 'packwand-ide-assets',
    configureServer(server: import('vite').ViteDevServer) {
      server.middlewares.use(async (request, response, next) => {
        const pathname = decodeURIComponent(new URL(request.url ?? '/', 'http://packwand.local').pathname)
        if (!pathname.startsWith('/packwand-ide/')) return next()
        const relative = pathname.slice('/packwand-ide/'.length) || 'index.html'
        const core = relative.startsWith('core/')
        const root = core ? ideCoreRoot : ideHostRoot
        const candidate = path.resolve(root, core ? relative.slice(5) : relative)
        if (path.relative(root, candidate).startsWith('..')) return next()
        try {
          const info = await stat(candidate)
          if (!info.isFile()) return next()
          response.statusCode = 200
          response.setHeader('Content-Type', contentTypes[path.extname(candidate)] ?? 'application/octet-stream')
          response.end(await readFile(candidate))
        } catch {
          next()
        }
      })
    },
    async writeBundle(options: { dir?: string }) {
      const outputRoot = path.resolve(frontendRoot, options.dir ?? 'dist', 'packwand-ide')
      await cp(ideHostRoot, outputRoot, { recursive: true })
      await cp(ideCoreRoot, path.join(outputRoot, 'core'), { recursive: true })
    },
  }
}

export default defineConfig({
  plugins: [vue(), tailwindcss(), packwandIdeAssets()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
})
