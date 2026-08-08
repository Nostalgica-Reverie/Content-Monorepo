import { describe, expect, test } from 'bun:test'

import { defaultLayout, reconcileLayout } from '@/stores/layout'
import type { ShellLayout } from '@/helpers/types'

describe('reconcileLayout', () => {
  test('an absent layout is the default arrangement', () => {
    expect(reconcileLayout(null)).toEqual(defaultLayout())
    expect(reconcileLayout(undefined)).toEqual(defaultLayout())
  })

  test('the default puts the sidebar on the left', () => {
    expect(defaultLayout().sidebarSide).toBe('left')
  })

  test('a well-formed layout is preserved', () => {
    const stored: ShellLayout = { version: 2, sidebarSide: 'right', sizes: { side: 240 } }
    const result = reconcileLayout(stored)
    expect(result.sidebarSide).toBe('right')
    expect(result.sizes).toEqual({ side: 240 })
  })

  // The property the feature rests on: customization is unsupported, so
  // anything on disk may be wrong, and none of it may make the shell
  // unrenderable.
  test('a nonsense value resolves to a renderable arrangement', () => {
    for (const wrecked of [
      { version: 2, sidebarSide: 'nowhere' },
      { version: 2 },
      { version: 2, sidebarSide: null, sizes: 'not an object' },
      { sidebarSide: 'right' },
      'a string entirely',
      42,
    ]) {
      const result = reconcileLayout(wrecked as unknown as ShellLayout)
      expect(['left', 'right']).toContain(result.sidebarSide)
      expect(result.version).toBe(2)
    }
  })

  // Version 1 modelled seven placeable panels, which the shell could not
  // honour. A layout saved by that build must reset rather than be coerced.
  test('a version-1 layout is discarded rather than migrated', () => {
    const old = { version: 1, slots: [{ id: 'fileTree', region: 'right', order: 0 }] }
    expect(reconcileLayout(old as unknown as ShellLayout)).toEqual(defaultLayout())
  })

  test('an unknown future version resets rather than being trusted', () => {
    const future = { version: 99, sidebarSide: 'right' }
    expect(reconcileLayout(future as unknown as ShellLayout)).toEqual(defaultLayout())
  })
})
