import { readFile, readdir } from 'node:fs/promises'
import { resolve } from 'node:path'

const root = resolve(import.meta.dirname, '../../src-tauri')
const files = await readdir(resolve(root, 'capabilities'))
const contents = await Promise.all(
  files.filter((file) => file.endsWith('.json')).map((file) => readFile(resolve(root, 'capabilities', file), 'utf8')),
)
const config = JSON.parse(await readFile(resolve(root, 'tauri.conf.json'), 'utf8'))
delete config.build?.devUrl
contents.push(JSON.stringify(config))
const source = contents.join('\n').toLowerCase()
if (source.includes('remote.urls') || source.includes('"remote"')) {
  throw new Error('remote capability grant found')
}
if (/https?:\/\/(localhost|127\.0\.0\.1)/.test(source)) {
  throw new Error('loopback runtime URL found outside build.devUrl')
}
console.log('Capability audit passed: local grants only, no remote URL scope.')
