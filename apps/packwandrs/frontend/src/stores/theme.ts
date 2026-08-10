import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { themesDelete, themesList, themesSave } from '@/helpers/invoke/themes'
import { core } from '@/core/packwand'
import { builtinThemeMap, builtinThemes } from '@/themes/builtins'
import { applyTheme, portableTheme, resolveTheme, validateTheme } from '@/themes/theme'
import type { PackwandTheme, ResolvedTheme, ThemeValidation } from '@/themes/types'

import { useSettingsStore } from './settings'
import { useAppCoreStore } from './appCore'

const legacyStorageKey = 'packwand.ide-theme'
const fallbackId = 'builtin.packwand-dark'

function slug(value: string) {
	return (
		value
			.toLowerCase()
			.trim()
			.replace(/[^a-z0-9]+/g, '-')
			.replace(/^-|-$/g, '') || 'theme'
	)
}

export const useThemeStore = defineStore('theme', () => {
	const settings = useSettingsStore()
	const appCore = useAppCoreStore()
	const customThemes = ref<PackwandTheme[]>([])
	const currentId = ref(fallbackId)
	const loading = ref(false)
	const diagnostics = ref<string[]>([])

	const themes = computed(() => [...builtinThemes, ...customThemes.value])
	const currentTheme = computed(
		() => themes.value.find((theme) => theme.id === currentId.value) ?? builtinThemes[0],
	)
	const resolved = computed(() => resolveTheme(currentTheme.value))

	function activate(id: string) {
		currentId.value = themes.value.some((theme) => theme.id === id) ? id : fallbackId
		applyTheme(resolved.value)
	}

	/** Apply a settings event without writing the same settings back to Rust. */
	function applySetting(id: string) {
		activate(id)
	}

	async function initialize() {
		loading.value = true
		diagnostics.value = []
		try {
			const stored = await themesList()
			customThemes.value = stored.flatMap((record) => {
				if (!record.theme) {
					diagnostics.value.push(`${record.fileName}: ${record.error ?? 'could not be loaded'}`)
					return []
				}
				const validation = validateTheme(record.theme)
				if (!validation.valid) {
					diagnostics.value.push(`${record.fileName}: ${validation.errors.join(' ')}`)
					return []
				}
				return [record.theme]
			})

			let selected = settings.value?.themeId || fallbackId
			const legacy = localStorage.getItem(legacyStorageKey)
			if (legacy) {
				selected = legacy === 'Tangled Dark' ? 'builtin.tangled-dark' : fallbackId
				localStorage.removeItem(legacyStorageKey)
				if (settings.value) await settings.save({ ...settings.value, themeId: selected })
			}
			if (!themes.value.some((theme) => theme.id === selected)) {
				diagnostics.value.push(`Selected theme ${selected} is unavailable; using Packwand Dark.`)
				selected = fallbackId
			}
			activate(selected)
		} finally {
			loading.value = false
		}
	}

	async function setTheme(id: string) {
		activate(id)
		const effects = appCore.dispatch(core.Message$SelectTheme(currentId.value))
		for (const effect of effects) {
			if (core.Effect$isPersistTheme(effect) && settings.value) {
				await settings.save({ ...settings.value, themeId: core.Effect$PersistTheme$0(effect) })
			}
		}
	}

	function preview(theme: PackwandTheme): ThemeValidation {
		const validation = validateTheme(theme)
		if (validation.valid) applyTheme(resolveTheme(theme))
		return validation
	}

	function cancelPreview() {
		applyTheme(resolved.value)
	}

	async function saveCustom(theme: PackwandTheme) {
		const validation = validateTheme(theme)
		if (!validation.valid) throw new Error(validation.errors.join(' '))
		const saved = await themesSave(theme)
		customThemes.value = [...customThemes.value.filter((item) => item.id !== saved.id), saved].sort(
			(left, right) => left.name.localeCompare(right.name),
		)
		await setTheme(saved.id)
		return saved
	}

	async function removeCustom(id: string) {
		await themesDelete(id)
		customThemes.value = customThemes.value.filter((theme) => theme.id !== id)
		if (currentId.value === id) await setTheme(fallbackId)
	}

	function duplicate(source: PackwandTheme = currentTheme.value) {
		const base = resolveTheme(source)
		let id = `user.${slug(source.name)}`
		let suffix = 2
		while (themes.value.some((theme) => theme.id === id))
			id = `user.${slug(source.name)}-${suffix++}`
		return portableTheme(base, id, `${source.name} Copy`)
	}

	function importText(source: string): PackwandTheme {
		const parsed = JSON.parse(source) as PackwandTheme
		if (parsed.id?.startsWith('builtin.')) parsed.id = `user.${slug(parsed.name || parsed.id)}`
		const validation = validateTheme(parsed)
		if (!validation.valid) throw new Error(validation.errors.join(' '))
		return parsed
	}

	function exportText(theme: PackwandTheme = currentTheme.value) {
		return JSON.stringify(portableTheme(resolveTheme(theme), theme.id, theme.name), null, 2) + '\n'
	}

	return {
		themes,
		currentId,
		currentTheme,
		resolved,
		loading,
		diagnostics,
		initialize,
		setTheme,
		applySetting,
		preview,
		cancelPreview,
		saveCustom,
		removeCustom,
		duplicate,
		importText,
		exportText,
	}
})
