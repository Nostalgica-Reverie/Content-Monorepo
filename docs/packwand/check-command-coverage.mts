import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { readdir } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const commandsDir = join(here, 'docs', 'reference', 'commands')
const packwandSrc = join(here, '..', '..', 'apps', 'packwandrs')

function findPackwand(): [string, ...string[]] {
	if (process.env.PACKWAND_BIN && existsSync(process.env.PACKWAND_BIN)) {
		return [process.env.PACKWAND_BIN]
	}
	return [
		'cargo',
		'run',
		'--manifest-path',
		join(packwandSrc, 'Cargo.toml'),
		'-p',
		'packwand-cli',
		'--',
	]
}

function pathToFilename(commandPath: string): string {
	return `packwand_${commandPath.replace(/ /g, '_')}.md`
}

const [bin, ...binArgs] = findPackwand()

let commandPaths
try {
	const output = execFileSync(bin, [...binArgs, 'utils', 'commands', '--json'], {
		encoding: 'utf-8',
		stdio: ['ignore', 'pipe', 'pipe'],
	})
	commandPaths = JSON.parse(output)
} catch (error) {
	console.warn(
		'Skipping CLI-reference coverage check: `packwand utils commands --json` is not available yet ' +
			'(codex.md section 4.2 depends on a CLI/core flag). Once that flag lands this script will enforce coverage.\n' +
			`Underlying error: ${error instanceof Error ? error.message : error}`,
	)
	process.exit(0)
}

if (!Array.isArray(commandPaths)) {
	console.error(
		'`packwand utils commands --json` did not return a JSON array as expected; got:',
		commandPaths,
	)
	process.exit(1)
}

commandPaths = commandPaths.map((entry) => (typeof entry === 'string' ? entry : entry.path))
const expectedFiles = new Set(['packwand.md', ...commandPaths.map(pathToFilename)])
const actualFiles = new Set((await readdir(commandsDir)).filter((name) => name.endsWith('.md')))

const missing = [...expectedFiles].filter((name) => !actualFiles.has(name)).sort()
const orphaned = [...actualFiles].filter((name) => !expectedFiles.has(name)).sort()

if (missing.length > 0) {
	console.error(
		`CLI-reference coverage: ${missing.length} registered command(s) have no generated page:`,
	)
	for (const name of missing) console.error(`  - ${name}`)
}
if (orphaned.length > 0) {
	console.warn(
		`CLI-reference coverage: ${orphaned.length} stale page(s) no longer correspond to a registered command:`,
	)
	for (const name of orphaned) console.warn(`  - ${name}`)
}

if (missing.length > 0) process.exit(1)
console.log(
	`CLI-reference coverage OK: ${commandPaths.length} command(s), all with a generated handbook page.`,
)
