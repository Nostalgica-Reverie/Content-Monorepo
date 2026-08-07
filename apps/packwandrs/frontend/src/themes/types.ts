export const themeTokenNames = [
  'rail', 'side', 'bg', 'bg-2', 'surface', 'surface-2', 'surface-3', 'surface-soft', 'elevated',
  'hover', 'active', 'selected', 'line', 'line-soft', 'line-strong', 'text', 'text-strong',
  'muted', 'faint', 'accent', 'accent-2', 'accent-dim', 'accent-soft', 'accent-line',
  'danger', 'danger-bg', 'danger-line', 'warning', 'success', 'success-bg',
] as const

export type ThemeTokenName = (typeof themeTokenNames)[number]
export type ThemeAppearance = 'light' | 'dark' | 'high-contrast'

export interface ThemeTokenRule {
  token: string
  foreground?: string
  background?: string
  fontStyle?: 'bold' | 'italic' | 'underline' | 'strikethrough' | ''
}

export interface PackwandTheme {
  $schema?: string
  schemaVersion: 1
  id: string
  name: string
  author?: string
  appearance: ThemeAppearance
  extends?: string
  colors: Partial<Record<ThemeTokenName, string>>
  editor?: {
    colors?: Record<string, string>
    rules?: ThemeTokenRule[]
  }
}

export interface ResolvedTheme extends Omit<PackwandTheme, 'colors' | 'editor'> {
  colors: Record<ThemeTokenName, string>
  editor: {
    colors: Record<string, string>
    rules: ThemeTokenRule[]
  }
}

export interface ThemeValidation {
  valid: boolean
  errors: string[]
  warnings: string[]
}

export const themeFileSuffix = '.packwand-theme.json'
