import { createRequire } from 'node:module'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const ideRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)))
const require = createRequire(pathToFileURL(path.join(ideRoot, 'workbench', 'package.json')))
const { chromium } = require('playwright-core')
const candidates = process.platform === 'win32'
  ? ['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', 'C:/Program Files/Microsoft/Edge/Application/msedge.exe']
  : process.platform === 'darwin'
    ? ['/Applications/Google Chrome.app/Contents/MacOS/Google Chrome']
    : ['/usr/bin/microsoft-edge', '/usr/bin/google-chrome', '/usr/bin/chromium']
const executablePath = process.env.PACKWAND_BROWSER ?? candidates.find(existsSync)
if (!executablePath) throw new Error('Set PACKWAND_BROWSER to a Chromium-compatible browser executable.')

const baseUrl = process.env.PACKWAND_DEV_URL ?? 'http://127.0.0.1:1420'

// A minimal valid 1x1 transparent PNG, used to prove the media-preview
// extension's webview can actually load bytes end-to-end through the
// packwand: filesystem provider -> webview resource loader -> webview
// host iframe chain (not just the plain-text editor path).
const pngBytes = Array.from(Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64',
))

const browser = await chromium.launch({ executablePath, headless: true })
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } })
  const pageErrors = []
  page.on('pageerror', error => pageErrors.push(error.message))
  await page.goto(`${baseUrl}/`)
  await page.evaluate((pngBytes) => {
    const encoder = new TextEncoder()
    const files = new Map([
      ['pack.toml', Array.from(encoder.encode('name = "Smoke Pack"\npack-format = "packwiz:1.1.0"\n'))],
      ['packeater.json', Array.from(encoder.encode('{\n  "preset": "aggressive"\n}\n'))],
      ['sample.png', pngBytes],
    ])
    window.addEventListener('message', event => {
      const request = event.data
      if (request?.channel !== 'packwand:ide-fs' || request.direction !== 'request') return
      const requestedPath = request.parameters?.path ?? ''
      let result
      let error
      if (request.method === 'stat') {
        if (requestedPath === '') result = { fileType: 2, size: 0, ctime: 0, mtime: Date.now() }
        else if (files.has(requestedPath)) result = { fileType: 1, size: files.get(requestedPath).length, ctime: 0, mtime: Date.now() }
        else error = { kind: 'not_found', message: `${requestedPath} was not found` }
      } else if (request.method === 'readDir') {
        result = requestedPath === '' ? [...files.keys()].map(name => ({ name, fileType: 1 })) : []
      } else if (request.method === 'readFile') {
        result = files.get(requestedPath)
        if (!result) error = { kind: 'not_found', message: `${requestedPath} was not found` }
      } else {
        result = null
      }
      event.source.postMessage({ channel: request.channel, direction: 'response', id: request.id, result, error }, event.origin)
    })
    document.body.innerHTML = '<iframe title="Packwand IDE smoke" src="/packwand-ide/index.html" style="width:100%;height:780px;border:0"></iframe>'
  }, pngBytes)
  pageErrors.length = 0

  const ide = page.frameLocator('iframe[title="Packwand IDE smoke"]')
  const packFile = ide.locator('.explorer-folders-view .monaco-list-row').filter({ hasText: 'pack.toml' }).first()
  await packFile.waitFor({ timeout: 30_000 })
  await packFile.dblclick()
  await ide.locator('.tab').filter({ hasText: 'pack.toml' }).first().waitFor({ timeout: 15_000 })
  await page.waitForTimeout(1_000)
  const text = await ide.locator('body').innerText()
  const editorText = (await ide.locator('.view-lines').allInnerTexts()).join('\n')
  const normalizedEditorText = editorText.replaceAll('\u00a0', ' ')

  // Open the PNG fixture through the built-in image-preview extension. This
  // exercises the webview stack end-to-end: extension host -> asWebviewUri
  // -> webview host iframe -> service worker resource fetch -> back through
  // the packwand: filesystem provider bridge. The extension only posts a
  // 'size' message (rendered into the status bar) after the <img> actually
  // fires its 'load' event, so waiting for that text is a reliable signal
  // that real image bytes made it all the way into the webview -- not just
  // that the tab opened.
  const pngFile = ide.locator('.explorer-folders-view .monaco-list-row').filter({ hasText: 'sample.png' }).first()
  await pngFile.waitFor({ timeout: 15_000 })
  await pngFile.dblclick()
  await ide.locator('.tab').filter({ hasText: 'sample.png' }).first().waitFor({ timeout: 15_000 })

  let imageSizeText = ''
  let hasImagePreview = false
  try {
    const sizeStatusEntry = ide.getByText(/^\s*\d+\s*x\s*\d+\s*$/).first()
    await sizeStatusEntry.waitFor({ timeout: 20_000 })
    imageSizeText = (await sizeStatusEntry.innerText()).trim()
    hasImagePreview = /^\d+x\d+$/.test(imageSizeText)
  } catch {
    hasImagePreview = false
  }

  const result = {
    hasExplorerFile: text.includes('packeater.json'),
    hasEditorContent: normalizedEditorText.includes('Smoke Pack'),
    editorSample: normalizedEditorText.slice(0, 160),
    hasImagePreview,
    imageSizeText,
    pageErrors,
  }
  console.log(JSON.stringify(result))
  if (!result.hasExplorerFile || !result.hasEditorContent || !result.hasImagePreview || result.pageErrors.length) process.exitCode = 1
} finally {
  await browser.close()
}
