import { describe, expect, it, test } from 'bun:test'

import { builtinThemes } from '@/themes/builtins'
import { portableTheme, resolveTheme, validateTheme } from '@/themes/theme'
import { themeTokenNames } from '@/themes/types'

describe('Packwand themes', () => {
	// Deliberately not a fixed count: themes are data, and adding one should not
	// require editing a number here. What must hold is that every one is valid
	// and that no two share an id — a duplicate would make theme selection
	// resolve to whichever happened to be later in the array.
	it('ships valid, uniquely identified themes', () => {
		expect(builtinThemes.length).toBeGreaterThanOrEqual(6)
		expect(new Set(builtinThemes.map((theme) => theme.id)).size).toBe(builtinThemes.length)
		for (const theme of builtinThemes) expect(validateTheme(theme).valid).toBe(true)
	})

	it('resolves partial themes against a bundled base', () => {
		const resolved = resolveTheme({
			schemaVersion: 1,
			id: 'user.rose',
			name: 'Rose',
			appearance: 'dark',
			extends: 'builtin.packwand-dark',
			colors: { accent: '#ff6688' },
		})
		expect(resolved.colors.accent).toBe('#ff6688')
		expect(resolved.colors.text).toBeTruthy()
		expect(resolved.editor.colors['editor.background']).toBeTruthy()
	})

	it('exports a portable copy and rejects unsafe identifiers', () => {
		const exported = portableTheme(resolveTheme(builtinThemes[0]), 'user.copy', 'Copy')
		expect(validateTheme(exported).valid).toBe(true)
		expect(validateTheme({ ...exported, id: '../escape' }).valid).toBe(false)
	})
})

describe('bundled themes', () => {
	// The gate for every theme that ships with Packwand: not merely "no errors"
	// but no *quality* warnings either. Warnings here are contrast failures, so
	// letting one through would ship a theme whose body text is unreadable while
	// the validator quietly said so.
	//
	// The one advisory that is dropped is the reserved-id notice. It exists to
	// tell someone *importing* a theme that a `builtin.*` id will be rewritten to
	// `user.*`, so it fires for every bundled theme by construction and says
	// nothing about the theme's quality.
	const RESERVED_ID_ADVISORY = 'Bundled theme IDs are reserved'
	for (const theme of builtinThemes) {
		test(`${theme.id} validates with no errors and no contrast warnings`, () => {
			const result = validateTheme(theme)
			const quality = result.warnings.filter((warning) => !warning.startsWith(RESERVED_ID_ADVISORY))
			expect({ id: theme.id, errors: result.errors, warnings: quality }).toEqual({
				id: theme.id,
				errors: [],
				warnings: [],
			})
		})
	}

	test('every theme id is unique', () => {
		const ids = builtinThemes.map((theme) => theme.id)
		expect(new Set(ids).size).toBe(ids.length)
	})

	test('the provider themes are all present', () => {
		const ids = new Set(builtinThemes.map((theme) => theme.id))
		for (const id of [
			'builtin.modrinth',
			'builtin.curseforge',
			'builtin.github-dark',
			'builtin.github-light',
			'builtin.gitlab',
			'builtin.forgejo',
		]) {
			expect(ids.has(id)).toBe(true)
		}
	})

	// Resolution fills every token from the base theme, so a variant that only
	// overrides a handful still hands the DOM a complete set of custom
	// properties — a missing one renders as an empty string, not a fallback.
	test('resolving a theme yields every application token', () => {
		for (const theme of builtinThemes) {
			const resolved = resolveTheme(theme)
			for (const token of themeTokenNames) {
				expect(typeof resolved.colors[token]).toBe('string')
				expect(resolved.colors[token]).not.toBe('')
			}
		}
	})
})
