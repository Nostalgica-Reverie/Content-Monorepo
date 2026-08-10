import * as fs from 'node:fs'
import * as path from 'node:path'
import * as crypto from 'node:crypto'

function runGit(args: string[]): string {
	const proc = Bun.spawnSync(['git', ...args], { stdout: 'pipe', stderr: 'pipe' })
	if (!proc.success) {
		throw new Error(`git ${args.join(' ')} failed: ${proc.stderr.toString().trim()}`)
	}
	return proc.stdout.toString().trim()
}

function listModBlobsAtRef(ref: string, repoRelDir: string): Map<string, string> {
	const blobs = new Map<string, string>()
	try {
		const out = runGit(['ls-tree', '-r', ref, '--', repoRelDir])
		for (const line of out.split('\n')) {
			const tab = line.indexOf('\t')
			if (tab < 0) continue
			const filePath = line.slice(tab + 1).trim()
			if (!filePath.endsWith('.pw.toml')) continue
			const meta = line.slice(0, tab).split(/\s+/)
			const sha = meta[2]
			if (sha) blobs.set(filePath, sha)
		}
	} catch {
		return blobs
	}
	return blobs
}

function fileAtRef(ref: string, repoRelPath: string): string {
	const proc = Bun.spawnSync(['git', 'show', `${ref}:${repoRelPath}`], {
		stdout: 'pipe',
		stderr: 'pipe',
	})
	return proc.success ? proc.stdout.toString() : ''
}

// blobContents reads many blobs with a single `git cat-file --batch` process
// instead of spawning `git show` once per file.
function blobContents(shas: string[]): Map<string, string> {
	const contents = new Map<string, string>()
	if (shas.length === 0) return contents
	const proc = Bun.spawnSync(['git', 'cat-file', '--batch'], {
		stdin: Buffer.from(shas.join('\n') + '\n'),
		stdout: 'pipe',
		stderr: 'pipe',
	})
	if (!proc.success) return contents
	const data = Buffer.from(proc.stdout)
	let offset = 0
	while (offset < data.length) {
		const nl = data.indexOf(10, offset)
		if (nl < 0) break
		const header = data.toString('utf-8', offset, nl).split(' ')
		offset = nl + 1
		const [sha, type, sizeStr] = header
		if (!sha || type !== 'blob' || !sizeStr) continue // "<sha> missing" has no body
		const size = Number.parseInt(sizeStr, 10)
		contents.set(sha, data.toString('utf-8', offset, offset + size))
		offset += size + 1 // skip trailing newline after each object body
	}
	return contents
}

function changeSignal(content: string): string {
	const hashMatch = content.match(/hash\s*=\s*"([^"]+)"/)
	if (hashMatch?.[1]) return hashMatch[1]
	const verMatch = content.match(/version\s*=\s*"([^"]+)"/)
	if (verMatch?.[1]) return verMatch[1]
	return crypto.createHash('sha1').update(content).digest('hex')
}

function modNameFromPath(p: string): string {
	return path.basename(p).replace(/\.pw\.toml$/, '')
}

interface DiffResult {
	added: string[]
	updated: string[]
	removed: string[]
}

function diffMods(oldRef: string, newRef: string, repoRelDir: string): DiffResult {
	const oldBlobs = listModBlobsAtRef(oldRef, repoRelDir)
	const newBlobs = listModBlobsAtRef(newRef, repoRelDir)

	const oldByName = new Map<string, string>()
	for (const p of oldBlobs.keys()) oldByName.set(modNameFromPath(p), p)
	const newByName = new Map<string, string>()
	for (const p of newBlobs.keys()) newByName.set(modNameFromPath(p), p)

	const added: string[] = []
	const removed: string[] = []
	const candidates: Array<{ name: string; oldSha: string; newSha: string }> = []

	for (const [name, newPath] of newByName) {
		const oldPath = oldByName.get(name)
		if (!oldPath) {
			added.push(name)
			continue
		}
		const oldSha = oldBlobs.get(oldPath) ?? ''
		const newSha = newBlobs.get(newPath) ?? ''
		if (oldSha === newSha) continue
		candidates.push({ name, oldSha, newSha })
	}
	for (const name of oldByName.keys()) {
		if (!newByName.has(name)) removed.push(name)
	}

	const contents = blobContents(candidates.flatMap((c) => [c.oldSha, c.newSha]))
	const updated = candidates
		.filter(
			(c) =>
				changeSignal(contents.get(c.oldSha) ?? '') !== changeSignal(contents.get(c.newSha) ?? ''),
		)
		.map((c) => c.name)

	added.sort()
	updated.sort()
	removed.sort()
	return { added, updated, removed }
}

function formatDiff(d: DiffResult): string {
	if (d.added.length === 0 && d.updated.length === 0 && d.removed.length === 0) return ''
	const summary = `**${d.added.length} added, ${d.updated.length} updated, ${d.removed.length} removed**`
	const blocks: string[] = []
	if (d.added.length > 0) {
		blocks.push('### Added\n\n' + d.added.map((m) => `- 🟢 \`${m}\``).join('\n'))
	}
	if (d.updated.length > 0) {
		blocks.push('### Updated\n\n' + d.updated.map((m) => `- 🟠 \`${m}\``).join('\n'))
	}
	if (d.removed.length > 0) {
		blocks.push('### Removed\n\n' + d.removed.map((m) => `- 🔴 \`${m}\``).join('\n'))
	}
	return `${summary}\n\n${blocks.join('\n\n')}`
}

function normalizeMarkdown(s: string): string {
	return (
		s
			.replace(/\r\n/g, '\n')
			.replace(/\n{3,}/g, '\n\n')
			.trimEnd() + '\n'
	)
}

interface ManifestVariant {
	id?: string
	mc_version?: string
}

function parseCalVer(v: string): { cycle: string; patch: string } | null {
	const m = v.match(/^(\d{2}\.\d{2})(?:\.(\d+))?/)
	if (!m || !m[1]) return null
	return { cycle: m[1], patch: m[2] ?? '0' }
}

function previousVersionFromGit(pathRef: string): string | null {
	try {
		const log = runGit(['log', '-n', '2', '--format=%H', '--', pathRef])
		const hashes = log.split('\n').filter(Boolean)
		if (hashes.length < 2 || !hashes[1]) return null
		const old = fileAtRef(hashes[1], pathRef)
		if (!old) return null
		const parsed = JSON.parse(old) as { version?: string }
		return parsed.version ?? null
	} catch {
		return null
	}
}

function versionHeader(rawName: string, version: string, pathRef: string): string {
	const today = new Date().toISOString().slice(0, 10)
	let tag = ''
	const cur = parseCalVer(version)
	if (cur) {
		const prev = previousVersionFromGit(pathRef)
		const prevParsed = prev ? parseCalVer(prev) : null
		if (prevParsed && prevParsed.cycle !== cur.cycle) {
			tag = ' — New monthly cycle'
		}
	}
	return `# ${rawName} ${version} — ${today}${tag}`
}

interface ManifestFile {
	name?: string
	type?: string
	version?: string
	mc_version?: string
	variants?: ManifestVariant[]
	automation?: { server_promo?: boolean }
}

function serverPromoEnabled(pDir: string, manifest: ManifestFile): boolean {
	// packwand merges manifest.json automation with the legacy opt-out.json;
	// CI puts it on PATH via .forgejo/actions/setup-packwand.
	const packwand = process.env.PACKWAND_BIN ?? 'packwand'
	const proc = Bun.spawnSync([packwand, 'automation', 'get', pDir], {
		stdout: 'pipe',
		stderr: 'pipe',
	})
	if (proc.success) {
		try {
			const auto = JSON.parse(proc.stdout.toString()) as { server_promo?: boolean }
			if (typeof auto.server_promo === 'boolean') return auto.server_promo
			return true
		} catch {
			/* fall through */
		}
	}
	// Fallback when packwand is unavailable: read from manifest directly
	const fromManifest = manifest.automation?.server_promo
	if (typeof fromManifest === 'boolean') return fromManifest
	return true
}

const SERVER_PROMO =
	'# Need a server?\n\n' +
	'[![BisectHosting Partnership](https://cdn.modrinth.com/data/cached_images/3d811a958c28645cf1007ccc3d90cb282921bf7f.webp)](https://bh.naomieow.xyz/raamviot50)'

function insertAfterFirstHeading(notes: string, block: string): string {
	const lines = notes.split('\n')
	const idx = lines.findIndex((l) => l.trimStart().startsWith('# '))
	if (idx < 0) return `${block}\n\n${notes.trim()}`
	lines.splice(idx + 1, 0, '', block)
	return lines.join('\n')
}

function generateChangelog(manifestPathStr: string): string {
	const manifestPath = path.resolve(manifestPathStr)
	const pDir = path.dirname(manifestPath)
	const filename = path.basename(manifestPath)
	const isExperimental = filename === 'manifest-experimental.json'

	if (!fs.existsSync(manifestPath)) {
		throw new Error(`manifest not found: ${manifestPathStr}`)
	}

	const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8')) as ManifestFile
	const rawName: string = manifest.name ?? path.basename(pDir)

	let prevHash: string | null = null
	try {
		if (isExperimental) {
			const packLog = runGit(['log', '-n', '2', '--format=%H', '--', pDir])
			const hashes = packLog.split('\n').filter(Boolean)
			if (hashes.length > 1 && hashes[1]) prevHash = hashes[1]
		} else {
			const prevBumpLog = runGit(['log', '-n', '2', '--format=%H', '--', manifestPathStr])
			const hashes = prevBumpLog.split('\n').filter(Boolean)
			if (hashes.length > 1 && hashes[1]) prevHash = hashes[1]
		}
	} catch (e) {
		console.warn(`could not read git log for anchor: ${e}`)
	}

	function subdirKeys(): string[] {
		if (Array.isArray(manifest.variants)) {
			return manifest.variants
				.map((v) => v.id ?? v.mc_version)
				.filter((k): k is string => typeof k === 'string' && k !== '')
		}
		if (manifest.mc_version) return [manifest.mc_version]
		return []
	}

	function buildModUpdatesBlock(): string {
		if (!prevHash || manifest.type !== 'modpack') return ''
		const anchor = prevHash
		const keys = subdirKeys()
		const sections: string[] = []

		for (const platform of ['mr', 'cf'] as const) {
			const label = platform === 'mr' ? 'Modrinth' : 'CurseForge'
			const groups = new Map<string, { keys: string[]; formatted: string }>()
			let present = 0

			for (const key of keys) {
				const subdir = path.join(pDir, `${key}-${platform}`)
				if (!fs.existsSync(subdir)) continue
				present++
				const repoRel = path.relative(process.cwd(), subdir).split(path.sep).join('/')
				const diff = diffMods(anchor, 'HEAD', repoRel)
				const formatted = formatDiff(diff)
				if (!formatted) continue
				const sig = JSON.stringify(diff)
				const group = groups.get(sig)
				if (group) group.keys.push(key)
				else groups.set(sig, { keys: [key], formatted })
			}

			for (const group of groups.values()) {
				let variantLabel = ''
				if (keys.length > 1) {
					variantLabel =
						group.keys.length === present && present > 1
							? ' (all variants)'
							: ` (${group.keys.join(', ')})`
				}
				sections.push(`## ${label}${variantLabel}\n\n${group.formatted}`)
			}
		}

		if (sections.length === 0) return ''
		return `# Mod Updates\n\n${sections.join('\n\n')}\n`
	}

	const version = manifest.version
	if (!isExperimental) {
		const changelogFile = path.join(pDir, 'changelog.md')
		let notes = fs.existsSync(changelogFile)
			? fs.readFileSync(changelogFile, 'utf-8')
			: `update for ${rawName}`

		if (version) {
			const header = versionHeader(rawName, version, manifestPathStr)
			if (!notes.trimStart().startsWith('# ')) {
				notes = `${header}\n\n${notes.trim()}`
			}
		}

		if (manifest.type === 'modpack' && serverPromoEnabled(pDir, manifest)) {
			notes = insertAfterFirstHeading(notes, SERVER_PROMO)
		}

		const modUpdatesBlock = buildModUpdatesBlock()
		if (modUpdatesBlock) {
			if (notes.includes('# Meta-changes')) {
				notes = notes.replace('# Meta-changes', `${modUpdatesBlock}\n# Meta-changes`)
			} else {
				notes = `${notes.trim()}\n\n${modUpdatesBlock}`
			}
		}

		const commitLines = collectCommitLines(prevHash, pDir)
		if (commitLines.length > 0) {
			if (!notes.includes('# Meta-changes')) notes += '\n\n# Meta-changes\n'
			notes += '\n## Automated Commit Log\n\n'
			notes += commitLines.map((line) => `- ${line}`).join('\n') + '\n'
		}

		return normalizeMarkdown(notes)
	}

	let notes = `_Experimental commit build. Unfinished work for technical users. Here be dragons._\n`
	const modUpdatesBlock = buildModUpdatesBlock()
	if (modUpdatesBlock) notes += `\n${modUpdatesBlock}`

	const commitLines = collectCommitLines(prevHash, pDir)
	if (commitLines.length > 0) {
		notes += '\n# Meta-changes\n\n## Automated Commit Log\n\n'
		notes += commitLines.map((line) => `- ${line}`).join('\n') + '\n'
	} else {
		notes += '\n_No commits to report since last experimental build._\n'
	}
	return normalizeMarkdown(notes)
}

function collectCommitLines(prevHash: string | null, pDir: string): string[] {
	if (!prevHash) {
		console.warn('no prior manifest bump found; skipping automated commit log')
		return []
	}
	const out: string[] = []
	try {
		const logs = runGit(['log', `${prevHash}..HEAD`, '--format=%h%x09%s%x09%an', '--', pDir])
		for (const line of logs.split('\n')) {
			const parts = line.split('\t')
			if (parts.length !== 3) continue
			const [hash = '', subject = '', author = ''] = parts
			if (!subject.includes(': ')) continue
			if (author === 'forgejo-actions[bot]') continue
			out.push(`${hash} ${subject} - ${author}`)
		}
	} catch (e) {
		console.warn(`could not fetch git logs for ${pDir}: ${e}`)
	}
	return out
}

const args = process.argv.slice(2)
const target = args[0]
if (!target) {
	console.error('usage: bun generate-changelog.ts <path/to/manifest.json> [output-file]')
	process.exit(1)
}

let finalNotes: string
try {
	finalNotes = generateChangelog(target)
} catch (e) {
	console.error(`${e instanceof Error ? e.message : e}`)
	process.exit(1)
}

const outFile = args[1]
if (outFile) {
	fs.writeFileSync(outFile, finalNotes.trim() + '\n')
	console.log(`wrote changelog for ${target} to ${outFile}`)
}

const outPath = process.env.GITHUB_OUTPUT
if (outPath) {
	const delimiter = `EOF_${crypto.randomBytes(8).toString('hex')}`
	fs.appendFileSync(outPath, `notes<<${delimiter}\n${finalNotes.trim()}\n${delimiter}\n`)
	console.log(`wrote changelog for ${target} to GITHUB_OUTPUT`)
} else if (!outFile) {
	console.log('\n--- CHANGELOG PREVIEW ---\n')
	console.log(finalNotes.trim())
	console.log('\n--- END ---\n')
}
