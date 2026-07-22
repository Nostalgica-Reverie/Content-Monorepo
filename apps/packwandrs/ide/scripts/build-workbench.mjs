import { existsSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ideRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)))
const workbenchRoot = path.join(ideRoot, 'workbench')
const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm'
const environment = {
  ...process.env,
  BUILD_SOURCEVERSION: '7e7950df89d055b5a378379db9ee14290772148a',
  ELECTRON_SKIP_BINARY_DOWNLOAD: '1',
  PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD: '1',
  VSCODE_SKIP_NODE_VERSION_CHECK: '1',
}

function run(command, args, extraEnvironment = {}) {
  const result = spawnSync(command, args, {
    cwd: workbenchRoot,
    env: { ...environment, ...extraEnvironment },
    stdio: 'inherit',
  })
  if (result.error) throw result.error
  if (result.status !== 0) process.exit(result.status ?? 1)
}

if (!existsSync(path.join(workbenchRoot, 'node_modules', '.package-lock.json'))) {
  run(npm, ['ci', '--ignore-scripts'])
  run(process.execPath, ['build/npm/postinstall.ts'], {
    npm_command: 'ci --ignore-scripts',
  })
}

run(npm, ['run', 'typecheck-client'])
run(npm, ['run', 'gulp', '--', 'vscode-web-min'])
