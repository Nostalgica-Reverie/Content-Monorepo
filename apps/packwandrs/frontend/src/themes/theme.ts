import { builtinThemeMap, builtinThemes } from './builtins'
import { validateHexColour, validateThemeId } from '@/core/packwand'
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
    for (const [foreground, background, label, minimum] of [
      ['text', 'bg', 'body text', 4.5], ['text-strong', 'surface', 'strong text', 4.5], ['accent', 'bg', 'accent controls', 3],
    ] as const) {
      const ratio = contrast(resolved.colors[foreground], resolved.colors[background])
      if (ratio < minimum) warnings.push(`${label} contrast is ${ratio.toFixed(2)}:1; target at least ${minimum}:1.`)
    }
  }
  return { valid: errors.length === 0, errors, warnings }
}

function contrast(left: string, right: string) {
  const luminance = (color: string) => {
    const channels = [1, 3, 5].map(index => Number.parseInt(color.slice(index, index + 2), 16) / 255)
      .map(value => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4)
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
  }
  const values = [luminance(left), luminance(right)].sort((a, b) => b - a)
  return (values[0] + 0.05) / (values[1] + 0.05)
}

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
