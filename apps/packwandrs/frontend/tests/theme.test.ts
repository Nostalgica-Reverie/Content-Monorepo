import { describe, expect, it } from 'bun:test'

import { builtinThemes } from '@/themes/builtins'
import { portableTheme, resolveTheme, validateTheme } from '@/themes/theme'

describe('Packwand themes', () => {
  it('ships six valid and uniquely identified themes', () => {
    expect(builtinThemes).toHaveLength(6)
    expect(new Set(builtinThemes.map(theme => theme.id)).size).toBe(6)
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
