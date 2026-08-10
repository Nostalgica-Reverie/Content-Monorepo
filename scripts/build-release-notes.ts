#!/usr/bin/env bun
import * as fs from 'node:fs'
import * as path from 'node:path'

const SECTION_ORDER = ['added', 'changed', 'deprecated', 'removed', 'fixed', 'security'] as const

const repoRoot = path.resolve(import.meta.dir, '..')

type Manifest = {
	id?: string
	name?: string
	version?: string
	role?: unknown
}

/** Heading text plus the lines beneath it, excluding the document title. */
type Section = { title: string; body: string[] }

type Changelog = { title: string; preamble: string[]; sections: Section[] }

function readManifest(manifestPath: string): Manifest {
	return JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as Manifest
}

function displayName(manifest: Manifest): string {
	return manifest.name?.trim() || manifest.id?.trim() || 'base pack'
}

/** The base pack id when the manifest consumes one, else undefined. */
function basePackId(manifest: Manifest): string | undefined {
	const role = manifest.role
	if (!role || typeof role !== 'object') return undefined
	const performanceBase = (role as Record<string, unknown>).performance_base
	if (!performanceBase || typeof performanceBase !== 'object') return undefined
	const pack = (performanceBase as Record<string, unknown>).pack
	return typeof pack === 'string' && pack.trim() ? pack.trim() : undefined
}

function findProjectById(id: string): string | undefined {
	for (const category of ['mods', 'modpacks', 'datapacks', 'resourcepacks']) {
		const categoryRoot = path.join(repoRoot, category)
		if (!fs.existsSync(categoryRoot)) continue
		for (const entry of fs.readdirSync(categoryRoot, { withFileTypes: true })) {
			if (!entry.isDirectory()) continue
			const projectRoot = path.join(categoryRoot, entry.name)
			const manifestPath = path.join(projectRoot, 'manifest.json')
			if (!fs.existsSync(manifestPath)) continue
			try {
				if (readManifest(manifestPath).id === id) return projectRoot
			} catch {
				continue
			}
		}
	}
	return undefined
}

function parseChangelog(markdown: string): Changelog {
	const lines = markdown.split(/\r?\n/)
	let title = ''
	const preamble: string[] = []
	const sections: Section[] = []
	let current: Section | undefined

	for (const line of lines) {
		const trimmed = line.trim()
		if (!title && trimmed.startsWith('# ')) {
			title = trimmed.slice(2).trim()
			continue
		}
		const heading = /^##\s+(.+?)\s*$/.exec(trimmed)
		if (heading) {
			current = { title: heading[1]!, body: [] }
			sections.push(current)
			continue
		}
		;(current ? current.body : preamble).push(line)
	}
	return { title, preamble, sections }
}

function trimBlank(lines: string[]): string[] {
	const copy = [...lines]
	while (copy.length && !copy[0]!.trim()) copy.shift()
	while (copy.length && !copy[copy.length - 1]!.trim()) copy.pop()
	return copy
}

function rank(title: string): number {
	const index = (SECTION_ORDER as readonly string[]).indexOf(title.toLowerCase())
	return index === -1 ? SECTION_ORDER.length : index
}

/** Prefixes each bullet with the base pack's name so its origin is obvious. */
function attribute(body: string[], label: string): string[] {
	return trimBlank(body).map((line) => {
		const bullet = /^(\s*)([-*])\s+(.*)$/.exec(line)
		return bullet ? `${bullet[1]}${bullet[2]} **${label}:** ${bullet[3]}` : line
	})
}

function merge(consumer: Changelog, base: Changelog | undefined, baseLabel: string): string {
	const out: string[] = []
	if (consumer.title) out.push(`# ${consumer.title}`, '')

	const preamble = trimBlank(consumer.preamble)
	if (preamble.length) out.push(...preamble, '')

	const sections = consumer.sections.map((section) => ({ ...section, body: [...section.body] }))

	if (base) {
		const baseSections = base.sections.filter((section) => trimBlank(section.body).length > 0)
		const consumerUsesSections = sections.length > 0

		if (consumerUsesSections && baseSections.length > 0) {
			for (const baseSection of baseSections) {
				const match = sections.find(
					(section) => section.title.toLowerCase() === baseSection.title.toLowerCase(),
				)
				const attributed = attribute(baseSection.body, baseLabel)
				if (match) match.body.push(...attributed)
				else sections.push({ title: baseSection.title, body: attributed })
			}
		} else {
			// No shared section structure to merge into: give the base its own
			// section rather than silently interleaving prose.
			const body = baseSections.length
				? baseSections.flatMap((section) => [
						`**${section.title}**`,
						...trimBlank(section.body),
						'',
					])
				: trimBlank(base.preamble)
			if (body.length) sections.push({ title: `From ${baseLabel}`, body })
		}
	}

	// Canonical order first; unrecognised headings keep their relative order.
	const ordered = sections
		.map((section, index) => ({ section, index }))
		.sort(
			(left, right) =>
				rank(left.section.title) - rank(right.section.title) || left.index - right.index,
		)
		.map((entry) => entry.section)

	for (const section of ordered) {
		const body = trimBlank(section.body)
		if (!body.length) continue
		out.push(`## ${section.title}`, '', ...body, '')
	}
	return `${trimBlank(out).join('\n')}\n`
}

const args = process.argv.slice(2)
const dryRun = args.includes('--dry-run')
const positional = args.filter((arg) => !arg.startsWith('--'))
const [manifestArg, outputArg] = positional

if (!manifestArg) {
	console.error(
		'usage: bun run.ts build-release-notes <path/to/manifest.json> [output-file] [--dry-run]',
	)
	process.exit(1)
}

const manifestPath = path.resolve(manifestArg)
if (!fs.existsSync(manifestPath)) {
	console.error(`error: no manifest at ${manifestPath}`)
	process.exit(1)
}

const manifest = readManifest(manifestPath)
const projectRoot = path.dirname(manifestPath)
const changelogPath = path.join(projectRoot, 'changelog.md')
if (!fs.existsSync(changelogPath)) {
	console.error(`error: no changelog.md beside ${manifestPath}`)
	process.exit(1)
}
const consumer = parseChangelog(fs.readFileSync(changelogPath, 'utf8'))

let base: Changelog | undefined
let baseLabel = ''
const baseId = basePackId(manifest)
if (baseId) {
	const baseRoot = findProjectById(baseId)
	if (!baseRoot) {
		console.error(
			`warning: performance base ${JSON.stringify(baseId)} was not found; notes cover this pack only`,
		)
	} else {
		const baseChangelog = path.join(baseRoot, 'changelog.md')
		if (fs.existsSync(baseChangelog)) {
			base = parseChangelog(fs.readFileSync(baseChangelog, 'utf8'))
			baseLabel = displayName(readManifest(path.join(baseRoot, 'manifest.json')))
		} else {
			console.error(
				`warning: performance base ${baseId} has no changelog.md; notes cover this pack only`,
			)
		}
	}
}

const notes = merge(consumer, base, baseLabel)

if (dryRun || !outputArg) {
	process.stdout.write(notes)
} else {
	fs.mkdirSync(path.dirname(path.resolve(outputArg)), { recursive: true })
	fs.writeFileSync(path.resolve(outputArg), notes)
	console.log(`wrote ${outputArg}${base ? ` (merged with ${baseLabel})` : ''}`)
}
