#!/usr/bin/env bun
import * as fs from 'node:fs'
import * as path from 'node:path'

const CATEGORIES = ['mods', 'modpacks', 'datapacks', 'resourcepacks'] as const

/** Keep a Changelog section vocabulary, in canonical order. */
const SECTION_ORDER = ['added', 'changed', 'deprecated', 'removed', 'fixed', 'security'] as const

type Severity = 'error' | 'warning'
type Issue = { severity: Severity; project: string; path: string; message: string }

const repoRoot = path.resolve(import.meta.dir, '..')
const args = process.argv.slice(2)
const asJson = args.includes('--json')
const strict = args.includes('--strict')

/**
 * Drops SemVer build metadata: a manifest version of `26.07+LTS` is published
 * as `26.07`, and headings write it that way.
 */
function comparableVersion(version: string): string {
	return version.split('+')[0]!.trim()
}

function firstHeading(lines: string[]): string | undefined {
	return lines.find((line) => line.trimStart().startsWith('#'))
}

function hasBody(lines: string[]): boolean {
	return lines.some((line) => {
		const trimmed = line.trim()
		return trimmed.length > 0 && !trimmed.startsWith('#')
	})
}

/** `## Added` -> `added`; anything that is not a level-2 heading is skipped. */
function sectionNames(lines: string[]): string[] {
	return lines
		.map((line) => /^##\s+(.+?)\s*$/.exec(line.trim()))
		.filter((match): match is RegExpExecArray => match !== null)
		.map((match) => match[1]!.toLowerCase())
}

function lintProject(projectRoot: string, issues: Issue[]): void {
	const name = path.basename(projectRoot)
	const manifestPath = path.join(projectRoot, 'manifest.json')
	const changelogPath = path.join(projectRoot, 'changelog.md')
	const relative = path.relative(repoRoot, changelogPath).replaceAll('\\', '/')

	let version = ''
	try {
		version = JSON.parse(fs.readFileSync(manifestPath, 'utf8')).version ?? ''
	} catch {
		return // Manifest validity is `packwand validate`'s job, not ours.
	}

	if (!fs.existsSync(changelogPath)) {
		issues.push({
			severity: 'error',
			project: name,
			path: relative,
			message: 'changelog.md is missing',
		})
		return
	}

	const lines = fs.readFileSync(changelogPath, 'utf8').split(/\r?\n/)

	if (!hasBody(lines)) {
		issues.push({
			severity: 'error',
			project: name,
			path: relative,
			message: 'changelog has a heading but no description',
		})
	}

	const heading = firstHeading(lines)
	if (!heading) {
		issues.push({
			severity: 'error',
			project: name,
			path: relative,
			message: 'changelog has no heading',
		})
	} else if (version && !heading.includes(comparableVersion(version))) {
		issues.push({
			severity: 'warning',
			project: name,
			path: relative,
			message: `heading ${JSON.stringify(heading.trim())} does not name the manifest version ${version}`,
		})
	}

	const sections = sectionNames(lines)
	const known = sections.filter((section) => (SECTION_ORDER as readonly string[]).includes(section))
	// Only hold a changelog to the section vocabulary once it has opted in by
	// using at least one canonical section. Plenty of entries use `##` purely
	// as prose subheadings, and those are not section names.
	if (known.length === 0) return
	for (const section of sections) {
		if (!(SECTION_ORDER as readonly string[]).includes(section)) {
			issues.push({
				severity: 'warning',
				project: name,
				path: relative,
				message: `unknown section "${section}" (expected one of ${SECTION_ORDER.join(', ')})`,
			})
		}
	}
	const ranks = known.map((section) =>
		SECTION_ORDER.indexOf(section as (typeof SECTION_ORDER)[number]),
	)
	if (ranks.some((rank, index) => index > 0 && rank < ranks[index - 1]!)) {
		issues.push({
			severity: 'warning',
			project: name,
			path: relative,
			message: `sections out of order (expected ${SECTION_ORDER.join(' -> ')})`,
		})
	}
}

const issues: Issue[] = []
for (const category of CATEGORIES) {
	const categoryRoot = path.join(repoRoot, category)
	if (!fs.existsSync(categoryRoot)) continue
	for (const entry of fs.readdirSync(categoryRoot, { withFileTypes: true })) {
		if (!entry.isDirectory()) continue
		const projectRoot = path.join(categoryRoot, entry.name)
		if (fs.existsSync(path.join(projectRoot, 'manifest.json'))) lintProject(projectRoot, issues)
	}
}

const errors = issues.filter((issue) => issue.severity === 'error')
const warnings = issues.filter((issue) => issue.severity === 'warning')

if (asJson) {
	console.log(JSON.stringify({ errors: errors.length, warnings: warnings.length, issues }, null, 2))
} else {
	for (const issue of issues) {
		console.log(
			`${issue.severity === 'error' ? 'ERROR' : 'warn '}  ${issue.path}: ${issue.message}`,
		)
	}
	console.log(
		`\n${issues.length} issue(s): ${errors.length} error(s), ${warnings.length} warning(s)`,
	)
}

process.exit(errors.length > 0 || (strict && warnings.length > 0) ? 1 : 0)
