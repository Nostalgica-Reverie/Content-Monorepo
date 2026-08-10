import vue from '@vitejs/plugin-vue'
import { execFile } from 'node:child_process'
import { fileURLToPath, URL } from 'node:url'
import { promisify } from 'node:util'
import { defineConfig } from 'vite'

const exec = promisify(execFile)
const coreRoot = fileURLToPath(new URL('./core', import.meta.url))

function gleamCore() {
	return {
		name: 'packwand-gleam-core',
		async handleHotUpdate(context: {
			file: string
			server: { ws: { send(message: { type: string }): void } }
		}) {
			if (!context.file.endsWith('.gleam') || !context.file.startsWith(coreRoot)) return
			await exec('gleam', ['build', '--target', 'javascript'], { cwd: coreRoot })
			context.server.ws.send({ type: 'full-reload' })
			return []
		},
	}
}

export default defineConfig({
	plugins: [gleamCore(), vue()],
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
