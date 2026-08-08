import { builtinThemeMap, builtinThemes } from './builtins'
import { contrastRatio, contrastRequirements, validateHexColour, validateThemeId } from '@/core/packwand'
import { themeTokenNames, type PackwandTheme, type ResolvedTheme, type ThemeTokenName, type ThemeValidation } from './types'

export function validateTheme(value: unknown): ThemeValidation {
  const errors: string[] = []
  const warnings: string[] = []
  if (!value || typeof value !== 'object' || Array.isArray(value)) return { valid: false, errors: ['Theme must be a JSON object.'], warnings }
  const theme = value as Partial<PackwandTheme>
  if (theme.schemaVersion !== 1) errors.push('schemaVersion must be 1.')
  if (typeof theme.id !== 'string' || !validateThemeId(theme.id)) errors.push('id must be a lowercase builtin.* or user.* slug.')
  if (typeof theme.name !== 'string' || !theme.name.trim()) errors.push('name is required.')
  if (!['light', 'dark', 'high-contrast'].includes(String(theme.appearance))) errors.push('appearance must be light, dark, or high-contrast.')
  if (theme.extends !== undefined && (typeof theme.extends !== 'string' || !builtinThemeMap.has(theme.extends))) errors.push('extends must reference a bundled theme.')
  if (!theme.colors || typeof theme.colors !== 'object' || Array.isArray(theme.colors)) errors.push('colors must be an object.')
  else {
    for (const [key, color] of Object.entries(theme.colors)) {
      if (!themeTokenNames.includes(key as ThemeTokenName)) errors.push(`Unknown application color token: ${key}.`)
      if (typeof color !== 'string' || !validateHexColour(color)) errors.push(`${key} must be a #RRGGBB or #RRGGBBAA color.`)
    }
  }
  for (const [key, color] of Object.entries(theme.editor?.colors ?? {})) {
    if (!key || typeof color !== 'string' || !validateHexColour(color)) errors.push(`Invalid editor color: ${key}.`)
  }
  for (const rule of theme.editor?.rules ?? []) {
    if (!rule || typeof rule.token !== 'string' || !rule.token.trim()) errors.push('Every editor rule needs a token.')
    if (rule.foreground && !validateHexColour(rule.foreground)) errors.push(`Invalid token foreground for ${rule.token}.`)
    if (rule.background && !validateHexColour(rule.background)) errors.push(`Invalid token background for ${rule.token}.`)
    if (rule.fontStyle !== undefined && !['', 'bold', 'italic', 'underline', 'strikethrough'].includes(rule.fontStyle)) errors.push(`Invalid token fontStyle for ${rule.token}.`)
  }
  if (theme.id?.startsWith('builtin.')) warnings.push('Bundled theme IDs are reserved; imported themes will be copied to a user.* ID.')
  if (!errors.length) {
    const resolved = resolveTheme(theme as PackwandTheme)
    for (const { foreground, background, label, minimum } of contrastRequirements()) {
      const ratio = contrastRatio(
        resolved.colors[foreground as ThemeTokenName],
        resolved.colors[background as ThemeTokenName],
      )
      if (ratio === null) warnings.push(`${label} contrast could not be measured.`)
      else if (ratio < minimum) warnings.push(`${label} contrast is ${ratio.toFixed(2)}:1; target at least ${minimum}:1.`)
    }
  }
  return { valid: errors.length === 0, errors, warnings }
}

// Contrast now lives in `packwand/theme.gleam`. It was reimplemented here in
// TypeScript, which is exactly the sort of maths that drifts between two
// copies; the Gleam one is covered by tests pinned to the WCAG anchors
// (21:1 black-on-white, 1:1 self) and treats an unparseable colour as a
// failure rather than `NaN`.

export function resolveTheme(theme: PackwandTheme): ResolvedTheme {
  const base = theme.extends ? builtinThemeMap.get(theme.extends) : undefined
  const fallback = builtinThemes[0]
  const baseResolved = base && base.id !== theme.id ? resolveTheme(base) : undefined
  return {
    ...theme,
    colors: { ...(baseResolved?.colors ?? fallback.colors), ...theme.colors } as Record<ThemeTokenName, string>,
    editor: {
      colors: { ...(baseResolved?.editor.colors ?? fallback.editor?.colors), ...(theme.editor?.colors ?? {}) },
      rules: [...(baseResolved?.editor.rules ?? fallback.editor?.rules ?? []), ...(theme.editor?.rules ?? [])],
    },
  }
}

export function applyTheme(theme: ResolvedTheme) {
  const root = document.documentElement
  root.dataset.theme = theme.id
  root.dataset.themeAppearance = theme.appearance
  root.style.colorScheme = theme.appearance === 'light' ? 'light' : 'dark'
  for (const token of themeTokenNames) root.style.setProperty(`--${token}`, theme.colors[token])
  window.dispatchEvent(new CustomEvent('packwand:theme-changed', { detail: theme }))
}

export function portableTheme(theme: ResolvedTheme, id: string, name: string): PackwandTheme {
  return {
    schemaVersion: 1,
    id,
    name,
    author: theme.author,
    appearance: theme.appearance,
    extends: 'builtin.packwand-dark',
    colors: { ...theme.colors },
    editor: { colors: { ...theme.editor.colors }, rules: theme.editor.rules.map(rule => ({ ...rule })) },
  }
}
