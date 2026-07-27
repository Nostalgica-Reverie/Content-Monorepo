<script setup lang="ts">
import { reactive, ref, toRaw, watch } from 'vue'
import Button from '@/components/ui/Button.vue'
import { automationPlan, automationRun } from '@/helpers/invoke/automation'
import { projectBump, projectManifestUpdate } from '@/helpers/invoke/projects'
import type { AppSettings, AutomationPlan, ProjectManifest } from '@/helpers/types'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'
import { useWorkspaceStore } from '@/stores/workspace'

const workbench = useWorkbenchStore()
const workspace = useWorkspaceStore()
const auth = useAuthStore()
const settings = useSettingsStore()
const toasts = useToastsStore()
const busy = ref('')
const manifest = reactive<ProjectManifest>({ id: '', name: '', type: '', variants: [], version: '' })
const appForm = reactive<AppSettings>({ workspacePath: null, javaDefaults: {}, memoryMb: 4096, msaClientId: null })
const automation = ref<AutomationPlan | null>(null)

// structuredClone cannot clone Vue's reactive proxies, so unwrap to the raw
// store object before copying it into the local editing form.
const detached = <T,>(value: T): T => structuredClone(toRaw(value))
watch(() => workbench.selectedProject, (project) => { if (project) Object.assign(manifest, detached(project.manifest)) }, { immediate: true })
watch(() => settings.value, (value) => { if (value) Object.assign(appForm, detached(value)) }, { immediate: true })
async function saveManifest() {
  busy.value = 'manifest'
  try { await projectManifestUpdate(workbench.selectedProjectId, manifest); await workbench.refresh(); toasts.push('Manifest saved', manifest.id, 'success') }
  catch (error) { toasts.push('Manifest save failed', String(error), 'danger') } finally { busy.value = '' }
}
async function bump() {
  const next = window.prompt('New version', manifest.version)
  if (!next || next === manifest.version) return
  try { await projectBump(manifest.id, next); manifest.version = next; await workbench.refresh(); toasts.push('Version bumped', next, 'success') }
  catch (error) { toasts.push('Version bump failed', String(error), 'danger') }
}
async function saveApp() {
  busy.value = 'app'
  try { await settings.save({ ...appForm }); toasts.push('Application settings saved', 'Local settings were updated.', 'success') }
  catch (error) { toasts.push('Settings save failed', String(error), 'danger') } finally { busy.value = '' }
}
async function chooseWorkspace() {
  try { if (await workspace.select()) { await workbench.refresh(); if (settings.value) Object.assign(appForm, settings.value) } }
  catch (error) { toasts.push('Workspace selection failed', String(error), 'danger') }
}
async function inspectAutomation() {
  busy.value = 'automation'
  try { automation.value = await automationPlan(manifest.id) }
  catch (error) { toasts.push('Automation inspection failed', String(error), 'danger') } finally { busy.value = '' }
}
async function runAutomation(dryRun: boolean) {
  if (!dryRun && !window.confirm('Run updates, synchronization, tests, and version bump for this project?')) return
  try { const job = await automationRun(manifest.id, dryRun); toasts.push(dryRun ? 'Automation dry-run started' : 'Automation started', job.label, 'success') }
  catch (error) { toasts.push('Automation failed', String(error), 'danger') }
}
</script>

<template>
  <section class="grid view-grid settings-view">
    <article class="panel span-7">
      <div class="panel-head"><h2>Project manifest</h2><div class="panel-actions"><Button variant="quiet" @click="bump">Bump version</Button><Button :busy="busy === 'manifest'" @click="saveManifest">Save manifest</Button></div></div>
      <form class="form-grid manifest-form" @submit.prevent="saveManifest">
        <label><span>ID</span><input v-model="manifest.id" disabled /></label>
        <label><span>Name</span><input v-model="manifest.name" /></label>
        <label><span>Version</span><input v-model="manifest.version" /></label>
        <label><span>Type</span><input v-model="manifest.type" /></label>
        <label><span>Minecraft</span><input v-model="manifest.mc_version" placeholder="Per variant" /></label>
        <label><span>Loader</span><input v-model="manifest.loader" placeholder="Per variant" /></label>
        <label><span>Release type</span><select v-model="manifest.release_type"><option value="">Unset</option><option value="release">Release</option><option value="beta">Beta</option><option value="alpha">Alpha</option></select></label>
        <label><span>Lifecycle</span><select v-model="manifest.lifecycle"><option value="active">Active</option><option value="maintenance">Maintenance</option><option value="archived">Archived</option></select></label>
        <label><span>Modrinth ID</span><input v-model="manifest.modrinth_id" placeholder="Optional" /></label>
        <label><span>CurseForge ID</span><input v-model="manifest.curseforge_id" placeholder="Optional" /></label>
        <label class="span-form"><span>Description</span><input v-model="manifest.description" /></label>
      </form>
    </article>
    <article class="panel span-5">
      <div class="panel-head"><h2>Application</h2><span class="status-badge integrated">local</span></div>
      <label class="field-stack"><span>Workspace</span><div class="compound-field"><input :value="workspace.path ?? ''" readonly /><Button variant="quiet" @click="chooseWorkspace">Change</Button></div></label>
      <label class="field-stack"><span>Default memory (MB)</span><input v-model.number="appForm.memoryMb" type="number" min="1024" step="512" /></label>
      <label class="field-stack"><span>Microsoft client ID override</span><input v-model="appForm.msaClientId" placeholder="Environment-gated when blank" /></label>
      <Button :busy="busy === 'app'" @click="saveApp">Save application settings</Button>
    </article>
    <article class="panel span-6">
      <div class="panel-head"><h2>Release automation</h2><span :class="['status-badge', { integrated: automation?.enabled }]">{{ automation?.enabled ? 'enabled' : 'inspect' }}</span></div>
      <p class="panel-copy">Inspect or run this project's opt-in update, sync, validation, test, and CalVer pipeline.</p>
      <div v-if="automation" class="details"><div class="detail"><span>Current</span><strong>{{ automation.version }}</strong></div><div class="detail"><span>Next</span><strong>{{ automation.nextVersion }}</strong></div><div class="detail"><span>Targets</span><strong>{{ automation.subdirs.length }}</strong></div></div>
      <div class="action-row panel-bottom-actions"><Button variant="quiet" :busy="busy === 'automation'" @click="inspectAutomation">Inspect</Button><Button variant="secondary" @click="runAutomation(true)">Dry-run</Button><Button variant="danger" @click="runAutomation(false)">Run pipeline</Button></div>
    </article>
    <article class="panel span-6">
      <div class="panel-head"><h2>Account</h2><span class="status-badge">{{ auth.state }}</span></div>
      <p class="panel-copy">{{ auth.label }}. Microsoft authentication is still environment-gated in this build and the UI does not pretend otherwise.</p>
      <div class="notice account-notice">Pack management, validation, exports, and publishing do not require a Minecraft account.</div>
    </article>
  </section>
</template>
