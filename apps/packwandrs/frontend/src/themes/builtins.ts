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
]

export const builtinThemeMap = new Map(builtinThemes.map(theme => [theme.id, theme]))
