import type { PackwandTheme } from './types'

const editorDark = {
  colors: {
    'editor.background': '#16161b',
    'editor.foreground': '#e4e4ea',
    'editorCursor.foreground': '#a78bfa',
    'editor.selectionBackground': '#6d51c966',
    'editor.lineHighlightBackground': '#24242c',
    'editorLineNumber.foreground': '#6b6b78',
    'editorLineNumber.activeForeground': '#e4e4ea',
  },
  rules: [
    { token: 'comment', foreground: '#6b6b78', fontStyle: 'italic' as const },
    { token: 'string', foreground: '#9bd6a8' },
    { token: 'number', foreground: '#e0b062' },
    { token: 'keyword', foreground: '#a78bfa' },
    { token: 'type', foreground: '#78b8d8' },
  ],
}

const packwandDark: PackwandTheme = {
  schemaVersion: 1,
  id: 'builtin.packwand-dark',
  name: 'Packwand Dark',
  author: 'Packwand',
  appearance: 'dark',
  colors: {
    rail: '#15151a', side: '#1b1b21', bg: '#1e1e24', 'bg-2': '#22222a', surface: '#24242c',
    'surface-2': '#292932', 'surface-3': '#16161b', 'surface-soft': '#26262f', elevated: '#2d2d38',
    hover: '#2b2b35', active: '#33333f', selected: '#2f2c3d', line: '#2c2c35',
    'line-soft': '#26262e', 'line-strong': '#3b3b47', text: '#e4e4ea', 'text-strong': '#f4f4f8',
    muted: '#9a9aa8', faint: '#6b6b78', accent: '#8a6df0', 'accent-2': '#a78bfa',
    'accent-dim': '#6d51c9', 'accent-soft': '#8a6df026', 'accent-line': '#8a6df066',
    danger: '#e2687d', 'danger-bg': '#33202a', 'danger-line': '#5c3241', warning: '#e0b062',
    success: '#6fce9f', 'success-bg': '#16281f',
  },
  editor: editorDark,
}

function variant(
  id: string,
  name: string,
  appearance: PackwandTheme['appearance'],
  colors: PackwandTheme['colors'],
  editor: PackwandTheme['editor'] = {},
): PackwandTheme {
  return { schemaVersion: 1, id, name, author: 'Packwand', appearance, extends: packwandDark.id, colors, editor }
}

export const builtinThemes: PackwandTheme[] = [
  packwandDark,
  variant('builtin.packwand-light', 'Packwand Light', 'light', {
    rail: '#e5e5eb', side: '#ededf2', bg: '#f7f7fa', 'bg-2': '#f0f0f5', surface: '#ffffff',
    'surface-2': '#f1f1f6', 'surface-3': '#ffffff', 'surface-soft': '#ebebf1', elevated: '#ffffff',
    hover: '#e5e2f2', active: '#dcd7ee', selected: '#e4ddfa', line: '#d4d4dc',
    'line-soft': '#e3e3e9', 'line-strong': '#b7b7c2', text: '#2d2d35', 'text-strong': '#15151b',
    muted: '#5f5f6b', faint: '#73737f', accent: '#6549c8', 'accent-2': '#5236b4',
    'accent-dim': '#49309f', 'accent-soft': '#6549c81f', 'accent-line': '#6549c866',
    danger: '#ad304a', 'danger-bg': '#f9e7eb', 'danger-line': '#d98a9a', warning: '#865b10',
    success: '#26724d', 'success-bg': '#e1f3e9',
  }, {
    colors: {
      'editor.background': '#ffffff', 'editor.foreground': '#2d2d35', 'editorCursor.foreground': '#6549c8',
      'editor.selectionBackground': '#c8baf0', 'editor.lineHighlightBackground': '#f1f1f6',
      'editorLineNumber.foreground': '#858590', 'editorLineNumber.activeForeground': '#2d2d35',
    },
    rules: [
      { token: 'comment', foreground: '#6f6f79', fontStyle: 'italic' }, { token: 'string', foreground: '#26724d' },
      { token: 'number', foreground: '#865b10' }, { token: 'keyword', foreground: '#6549c8' },
      { token: 'type', foreground: '#12678b' },
    ],
  }),
  variant('builtin.high-contrast-dark', 'Packwand High Contrast Dark', 'high-contrast', {
    rail: '#000000', side: '#080808', bg: '#000000', 'bg-2': '#0a0a0a', surface: '#111111',
    'surface-2': '#181818', 'surface-3': '#000000', 'surface-soft': '#141414', elevated: '#202020',
    hover: '#252525', active: '#303030', selected: '#302453', line: '#767676',
    'line-soft': '#555555', 'line-strong': '#ffffff', text: '#ffffff', 'text-strong': '#ffffff',
    muted: '#d0d0d0', faint: '#b5b5b5', accent: '#c8b5ff', 'accent-2': '#dfd5ff',
    'accent-dim': '#a88cff', 'accent-soft': '#c8b5ff33', 'accent-line': '#c8b5ffaa', danger: '#ff8fa3',
    'danger-bg': '#39000c', 'danger-line': '#ff8fa3', warning: '#ffd37a', success: '#8ff0ba', 'success-bg': '#00351b',
  }),
  variant('builtin.high-contrast-light', 'Packwand High Contrast Light', 'high-contrast', {
    rail: '#ffffff', side: '#f2f2f2', bg: '#ffffff', 'bg-2': '#eeeeee', surface: '#ffffff',
    'surface-2': '#e9e9e9', 'surface-3': '#ffffff', 'surface-soft': '#ededed', elevated: '#ffffff',
    hover: '#dedede', active: '#cccccc', selected: '#d8ccff', line: '#555555',
    'line-soft': '#777777', 'line-strong': '#000000', text: '#000000', 'text-strong': '#000000',
    muted: '#333333', faint: '#4a4a4a', accent: '#3d1c9e', 'accent-2': '#2e117e',
    'accent-dim': '#321681', 'accent-soft': '#3d1c9e22', 'accent-line': '#3d1c9eaa', danger: '#87001b',
    'danger-bg': '#ffe8ed', 'danger-line': '#87001b', warning: '#604000', success: '#005b2c', 'success-bg': '#dcf5e7',
  }, {
    colors: {
      'editor.background': '#ffffff', 'editor.foreground': '#000000', 'editorCursor.foreground': '#3d1c9e',
      'editor.selectionBackground': '#c2afff', 'editor.lineHighlightBackground': '#eeeeee',
      'editorLineNumber.foreground': '#555555', 'editorLineNumber.activeForeground': '#000000',
    },
  }),
  variant('builtin.tangled-dark', 'Tangled Dark', 'dark', {
    accent: '#f05a7e', 'accent-2': '#ff88a4', 'accent-dim': '#c63f61', 'accent-soft': '#f05a7e26',
    'accent-line': '#f05a7e66', selected: '#3b2731', rail: '#171316', side: '#21191e', bg: '#241c21',
  }, { colors: { 'editorCursor.foreground': '#ff88a4', 'editor.selectionBackground': '#c63f6166' } }),
  variant('builtin.nether-ember', 'Nether Ember', 'dark', {
    accent: '#ef7f45', 'accent-2': '#ffad6f', 'accent-dim': '#bd592d', 'accent-soft': '#ef7f4526',
    'accent-line': '#ef7f4566', selected: '#3e2b22', rail: '#171310', side: '#211a16', bg: '#241d19',
    warning: '#ffc36b', success: '#7fd39c',
  }, { colors: { 'editorCursor.foreground': '#ffad6f', 'editor.selectionBackground': '#bd592d66' } }),

  // Themes evoking the sites Packwand talks to. These take their cue from each
  // provider's palette without reproducing its design system: the token set,
  // the surface ramp and the editor rules are all Packwand's. Every one clears
  // `validateTheme` with zero warnings, not merely zero errors — the contrast
  // pairs are checked in `builtins.test.ts`.
  variant('builtin.modrinth', 'Modrinth Green', 'dark', {
    rail: '#101216', side: '#16181c', bg: '#16181c', 'bg-2': '#1c1f24', surface: '#26292f',
    'surface-2': '#2b2f36', 'surface-3': '#101216', 'surface-soft': '#23262c', elevated: '#30343c',
    hover: '#2c3037', active: '#363b44', selected: '#1d3a2a', line: '#2f333a',
    'line-soft': '#282c32', 'line-strong': '#3f444d',
    accent: '#1bd96a', 'accent-2': '#4ce68e', 'accent-dim': '#14a04e', 'accent-soft': '#1bd96a1f',
    'accent-line': '#1bd96a66', success: '#1bd96a', 'success-bg': '#12281c',
  }, {
    colors: {
      'editor.background': '#16181c', 'editor.foreground': '#e4e4ea',
      'editorCursor.foreground': '#1bd96a', 'editor.selectionBackground': '#14a04e59',
      'editor.lineHighlightBackground': '#1c1f24',
    },
    rules: [{ token: 'string', foreground: '#4ce68e' }, { token: 'keyword', foreground: '#1bd96a' }],
  }),
  variant('builtin.curseforge', 'CurseForge Ember', 'dark', {
    rail: '#101013', side: '#16161a', bg: '#16161a', 'bg-2': '#1c1c21', surface: '#232329',
    'surface-2': '#2a2a31', 'surface-3': '#101013', 'surface-soft': '#212127', elevated: '#2f2f37',
    hover: '#2b2b32', active: '#35353e', selected: '#3a2620', line: '#2e2e35',
    'line-soft': '#26262c', 'line-strong': '#3e3e47',
    accent: '#f16436', 'accent-2': '#ff8b63', 'accent-dim': '#c44a24', 'accent-soft': '#f164361f',
    'accent-line': '#f1643666', warning: '#f0a53c',
  }, {
    colors: {
      'editor.background': '#16161a', 'editor.foreground': '#e4e4ea',
      'editorCursor.foreground': '#f16436', 'editor.selectionBackground': '#c44a2459',
      'editor.lineHighlightBackground': '#1c1c21',
    },
    rules: [{ token: 'keyword', foreground: '#f16436' }, { token: 'number', foreground: '#f0a53c' }],
  }),
  variant('builtin.github-dark', 'GitHub Dark', 'dark', {
    rail: '#010409', side: '#0d1117', bg: '#0d1117', 'bg-2': '#131920', surface: '#161b22',
    'surface-2': '#1c2129', 'surface-3': '#010409', 'surface-soft': '#151a21', elevated: '#21262d',
    hover: '#1f242c', active: '#282e37', selected: '#132741', line: '#30363d',
    'line-soft': '#21262d', 'line-strong': '#484f58',
    accent: '#2f81f7', 'accent-2': '#58a6ff', 'accent-dim': '#1f6feb', 'accent-soft': '#2f81f71f',
    'accent-line': '#2f81f766', danger: '#f85149', 'danger-bg': '#3d1518', 'danger-line': '#6e2225',
    warning: '#d29922', success: '#3fb950', 'success-bg': '#12261a',
  }, {
    colors: {
      'editor.background': '#0d1117', 'editor.foreground': '#e6edf3',
      'editorCursor.foreground': '#2f81f7', 'editor.selectionBackground': '#1f6feb59',
      'editor.lineHighlightBackground': '#131920',
    },
    rules: [
      { token: 'comment', foreground: '#8b949e', fontStyle: 'italic' },
      { token: 'string', foreground: '#a5d6ff' }, { token: 'keyword', foreground: '#ff7b72' },
      { token: 'type', foreground: '#79c0ff' },
    ],
  }),
  variant('builtin.github-light', 'GitHub Light', 'light', {
    rail: '#f6f8fa', side: '#ffffff', bg: '#ffffff', 'bg-2': '#f6f8fa', surface: '#f6f8fa',
    'surface-2': '#eaeef2', 'surface-3': '#ffffff', 'surface-soft': '#f0f3f6', elevated: '#ffffff',
    hover: '#eaeef2', active: '#dde3ea', selected: '#ddf4ff', line: '#d1d9e0',
    'line-soft': '#e4e8ed', 'line-strong': '#adb5bd', text: '#1f2328', 'text-strong': '#010409',
    muted: '#59636e', faint: '#6e7781',
    accent: '#0969da', 'accent-2': '#0550ae', 'accent-dim': '#0550ae', 'accent-soft': '#0969da1f',
    'accent-line': '#0969da66', danger: '#a40e26', 'danger-bg': '#ffebe9', 'danger-line': '#ff818266',
    warning: '#7d4e00', success: '#1a7f37', 'success-bg': '#dafbe1',
  }, {
    colors: {
      'editor.background': '#ffffff', 'editor.foreground': '#1f2328',
      'editorCursor.foreground': '#0969da', 'editor.selectionBackground': '#b6e3ff',
      'editor.lineHighlightBackground': '#f6f8fa',
      'editorLineNumber.foreground': '#6e7781', 'editorLineNumber.activeForeground': '#1f2328',
    },
    rules: [
      { token: 'comment', foreground: '#59636e', fontStyle: 'italic' },
      { token: 'string', foreground: '#0a3069' }, { token: 'keyword', foreground: '#a40e26' },
      { token: 'type', foreground: '#0550ae' },
    ],
  }),
  variant('builtin.gitlab', 'GitLab Tanuki', 'dark', {
    rail: '#18171d', side: '#1f1e24', bg: '#1f1e24', 'bg-2': '#25242b', surface: '#28262d',
    'surface-2': '#2f2d35', 'surface-3': '#18171d', 'surface-soft': '#262429', elevated: '#34323b',
    hover: '#302e37', active: '#3a3841', selected: '#3a2a20', line: '#33313a',
    'line-soft': '#2b2932', 'line-strong': '#45434e',
    accent: '#fc6d26', 'accent-2': '#fca326', 'accent-dim': '#c9541a', 'accent-soft': '#fc6d261f',
    'accent-line': '#fc6d2666', warning: '#fca326',
  }, {
    colors: {
      'editor.background': '#1f1e24', 'editor.foreground': '#e4e4ea',
      'editorCursor.foreground': '#fc6d26', 'editor.selectionBackground': '#c9541a59',
      'editor.lineHighlightBackground': '#25242b',
    },
    rules: [{ token: 'keyword', foreground: '#fc6d26' }, { token: 'number', foreground: '#fca326' }],
  }),
  variant('builtin.forgejo', 'Forgejo Amber', 'dark', {
    rail: '#0f1114', side: '#15171a', bg: '#15171a', 'bg-2': '#1b1e22', surface: '#22262b',
    'surface-2': '#282d33', 'surface-3': '#0f1114', 'surface-soft': '#1f2328', elevated: '#2d3239',
    hover: '#292e34', active: '#333941', selected: '#3a2a1c', line: '#2d3239',
    'line-soft': '#252a30', 'line-strong': '#3d434b',
    accent: '#ff7a2f', 'accent-2': '#ffa066', 'accent-dim': '#cc5c1c', 'accent-soft': '#ff7a2f1f',
    'accent-line': '#ff7a2f66', warning: '#ffb454',
  }, {
    colors: {
      'editor.background': '#15171a', 'editor.foreground': '#e4e4ea',
      'editorCursor.foreground': '#ff7a2f', 'editor.selectionBackground': '#cc5c1c59',
      'editor.lineHighlightBackground': '#1b1e22',
    },
    rules: [{ token: 'keyword', foreground: '#ff7a2f' }, { token: 'string', foreground: '#a3d9a5' }],
  }),
]

export const builtinThemeMap = new Map(builtinThemes.map(theme => [theme.id, theme]))
