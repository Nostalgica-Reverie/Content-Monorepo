import { createRouter, createWebHashHistory } from 'vue-router'
import { workspaceGet } from '@/helpers/invoke/workspace'
import AppLayout from '@/layouts/AppLayout.vue'
import ChangelogPage from '@/pages/ChangelogPage.vue'
import ExportsPage from '@/pages/ExportsPage.vue'
import InstancesPage from '@/pages/InstancesPage.vue'
import InstanceDetailPage from '@/pages/InstanceDetailPage.vue'
import JobsPage from '@/pages/JobsPage.vue'
import BrowsePage from '@/pages/BrowsePage.vue'
import ModsPage from '@/pages/ModsPage.vue'
import OverviewPage from '@/pages/OverviewPage.vue'
import SettingsPage from '@/pages/SettingsPage.vue'
import SetupPage from '@/pages/SetupPage.vue'

const EditorPage = () => import('@/pages/EditorPage.vue')
// Pulls in the mcdoc renderer and its schema tables; kept out of the initial
// bundle since most sessions never open a generator.
const GeneratorPage = () => import('@/pages/GeneratorPage.vue')

const router = createRouter({ history: createWebHashHistory(), routes: [
  { path: '/setup', name: 'setup', component: SetupPage },
  { path: '/', component: AppLayout, children: [
    { path: '', name: 'overview', component: OverviewPage },
    { path: 'editor', name: 'editor', component: EditorPage },
    { path: 'instances', name: 'instances', component: InstancesPage },
    { path: 'instances/:id', name: 'instance-detail', component: InstanceDetailPage },
    { path: 'exports', name: 'exports', component: ExportsPage },
    { path: 'mods', name: 'mods', component: ModsPage },
    { path: 'browse', name: 'browse', component: BrowsePage },
    { path: 'generator', name: 'generator', component: GeneratorPage },
    { path: 'changelog', name: 'changelog', component: ChangelogPage },
    { path: 'logs', name: 'logs', component: JobsPage },
    { path: 'settings', name: 'settings', component: SettingsPage },
    { path: 'jobs', redirect: '/logs' }, { path: 'projects', redirect: '/' },
    { path: 'diagnostics', redirect: '/' }, { path: 'operations', redirect: '/' }, { path: 'api', redirect: '/settings' },
  ] },
] })

let cachedWorkspace: string | null | undefined
router.beforeEach(async (to) => {
  if (cachedWorkspace === undefined) { try { cachedWorkspace = await workspaceGet() } catch { cachedWorkspace = null } }
  if (!cachedWorkspace && to.name !== 'setup') return { name: 'setup' }
  if (cachedWorkspace && to.name === 'setup') return { name: 'overview' }
  return true
})
export function markWorkspaceConfigured(path: string) { cachedWorkspace = path }
export default router
