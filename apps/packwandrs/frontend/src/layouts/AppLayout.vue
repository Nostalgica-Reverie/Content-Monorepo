<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import AppIcon from '@/components/ui/AppIcon.vue'
import Button from '@/components/ui/Button.vue'
import { apiInspect } from '@/helpers/invoke/api'
import { diagnosticsLint, diagnosticsPreflight } from '@/helpers/invoke/diagnostics'
import { workspaceSyncPreview } from '@/helpers/invoke/workspace'
import { useAuthStore } from '@/stores/auth'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'
import { useWorkspaceStore } from '@/stores/workspace'

const workbench = useWorkbenchStore()
const workspace = useWorkspaceStore()
const auth = useAuthStore()
const toasts = useToastsStore()
const route = useRoute()
const busy = ref('')
const logoUrl = new URL('../../../src-tauri/icons/icon.png', import.meta.url).href
const navigation = [
  { to: '/', label: 'Overview', icon: 'overview' }, { to: '/editor', label: 'Editor', icon: 'editor' },
  { to: '/instances', label: 'Instances', icon: 'instances' }, { to: '/exports', label: 'Exports', icon: 'exports' },
  { to: '/mods', label: 'Mods', icon: 'mods' }, { to: '/changelog', label: 'Changelog', icon: 'changelog' },
  { to: '/logs', label: 'Logs', icon: 'logs' }, { to: '/settings', label: 'Settings', icon: 'settings' },
]

onMounted(async () => {
  try { await workbench.refresh() }
  catch (error) { toasts.push('Could not index workspace', String(error), 'danger') }
})

async function action(name: string, work: () => Promise<unknown>) {
  busy.value = name
  try {
    const result = await work()
    const issues = typeof result === 'object' && result && 'issues' in result ? (result as { issues: unknown[] }).issues.length : null
    toasts.push(name, issues === null ? 'Completed successfully.' : issues ? issues + ' issues found.' : 'All checks passed.', issues ? 'danger' : 'success')
  } catch (error) {
    toasts.push(name + ' failed', String(error), 'danger')
  } finally { busy.value = '' }
}

const showToolbar = () => ['overview', 'logs', 'settings'].includes(String(route.name))
</script>

<template>
  <div class="app workbench-shell" :data-current-view="route.name">
    <aside class="sidebar">
      <div class="brand">
        <img :src="logoUrl" class="mark" alt="" />
        <div><strong>Packwand</strong><span :title="workspace.path ?? ''">{{ workspace.path || 'No workspace' }}</span></div>
      </div>
      <div class="explorer-title"><span>EXPLORER</span><span class="branch-option">repository</span></div>
      <label class="field-label" for="projectSelect">Current project</label>
      <select id="projectSelect" :value="workbench.selectedProjectId" @change="workbench.selectProject(($event.target as HTMLSelectElement).value)">
        <option v-if="!workbench.projects.length" value="">No projects indexed</option>
        <option v-for="project in workbench.projects" :key="project.manifest.id" :value="project.manifest.id">{{ project.manifest.name || project.manifest.id }}</option>
      </select>
      <nav class="activity-nav" aria-label="Primary navigation">
        <RouterLink v-for="item in navigation" :key="item.to" :to="item.to" class="nav-btn" :class="{ active: route.path === item.to }">
          <span class="nav-icon"><AppIcon :name="item.icon" /></span><span class="nav-label">{{ item.label }}</span>
        </RouterLink>
      </nav>
      <div class="sidebar-footer"><span><i class="connection-dot" /> Native Rust workspace</span><span>packwand 26.2.0</span></div>
    </aside>
    <main class="workbench-main">
      <header class="topbar">
        <div class="project-heading"><h1>{{ workbench.title }}</h1><p id="projectMeta">{{ workbench.summary }}</p></div>
        <div class="top-actions">
          <label class="search-wrap"><AppIcon name="search" /><input v-model="workbench.search" type="search" placeholder="Search current project…" /></label>
          <span v-if="workbench.search.trim()" class="pill">filtering</span>
          <Button variant="quiet" :busy="workbench.loading" @click="workbench.refresh()"><AppIcon name="refresh" /> Refresh</Button>
        </div>
      </header>
      <section v-if="showToolbar()" class="toolbar command-toolbar">
        <Button variant="secondary" :busy="busy === 'Status'" @click="action('Status', () => apiInspect('/health'))">Status</Button>
        <Button variant="secondary" :busy="busy === 'Preflight'" @click="action('Preflight', diagnosticsPreflight)">Preflight</Button>
        <Button variant="secondary" :busy="busy === 'Lint'" @click="action('Lint', diagnosticsLint)">Lint</Button>
        <Button variant="quiet" :busy="busy === 'Dry sync'" @click="action('Dry sync', workspaceSyncPreview)">Dry sync</Button>
      </section>
      <div v-if="workbench.error" class="error-banner">{{ workbench.error }}</div>
      <RouterView />
      <footer class="workbench-status"><span>{{ workbench.selectedPack?.id || 'No pack target selected' }}</span><span>{{ auth.label }}</span><span>IPC connected</span></footer>
    </main>
  </div>
</template>
