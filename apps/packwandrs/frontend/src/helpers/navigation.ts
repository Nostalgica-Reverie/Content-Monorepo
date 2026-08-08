export interface NavItem {
  name: string
  path: string
  label: string
  icon: string
}

/** Views reachable from the activity rail, in rail order. */
export const primaryNav: NavItem[] = [
  { name: 'overview', path: '/', label: 'Overview', icon: 'overview' },
  { name: 'editor', path: '/editor', label: 'Editor', icon: 'editor' },
  { name: 'mods', path: '/mods', label: 'Mods', icon: 'mods' },
  { name: 'browse', path: '/browse', label: 'Browse', icon: 'package' },
  { name: 'generator', path: '/generator', label: 'Generators', icon: 'package' },
  { name: 'instances', path: '/instances', label: 'Instances', icon: 'instances' },
  { name: 'exports', path: '/exports', label: 'Exports', icon: 'exports' },
  { name: 'changelog', path: '/changelog', label: 'Changelog', icon: 'changelog' },
  { name: 'logs', path: '/logs', label: 'Jobs', icon: 'logs' },
]

/** Pinned to the bottom of the rail, away from the task-oriented views. */
export const endNav: NavItem[] = [{ name: 'settings', path: '/settings', label: 'Settings', icon: 'settings' }]

export const allNav = [...primaryNav, ...endNav]

export function navByName(name: string): NavItem | undefined {
  return allNav.find((item) => item.name === name)
}
