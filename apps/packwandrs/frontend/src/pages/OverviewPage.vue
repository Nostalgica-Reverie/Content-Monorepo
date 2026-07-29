<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import Modal from '@/components/ui/Modal.vue'
import { diagnosticsPreflight, diagnosticsValidate } from '@/helpers/invoke/diagnostics'
import { modsRefresh } from '@/helpers/invoke/mods'
import { projectsCreate } from '@/helpers/invoke/projects'
import { workspaceSync, workspaceSyncPreview } from '@/helpers/invoke/workspace'
import type { NewProjectRequest, ValidationReport } from '@/helpers/types'
import { useShellStore } from '@/stores/shell'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const shell = useShellStore()
const toasts = useToastsStore()
const busy = ref('')
const createOpen = ref(false)
const form = reactive({
  category: 'modpacks' as NewProjectRequest['category'],
  id: '',
  name: '',
  minecraft: '1.21.1',
  loader: 'fabric',
  variants: '',
})

const filteredPacks = computed(() => {
  const query = workbench.search.trim().toLowerCase()
  return workbench.projectPacks.filter(
    (pack) => !query || (pack.name + ' ' + pack.id + ' ' + pack.path).toLowerCase().includes(query),
  )
})
const manifest = computed(() => workbench.selectedProject?.manifest)
const totalIndexedFiles = computed(() => workbench.projectPacks.reduce((total, pack) => total + pack.indexedFiles, 0))
const totalMetadataFiles = computed(() => workbench.projectPacks.reduce((total, pack) => total + pack.metadataFiles, 0))
const projectLoaders = computed(() => [...new Set(workbench.projectPacks.flatMap((pack) => pack.loaders))].sort())

/** The rail, sidebar, and palette all route "New project…" through the store. */
watch(
  () => shell.newProjectRequest,
  () => {
    createOpen.value = true
  },
)

async function run(name: string, work: () => Promise<unknown>) {
  busy.value = name
  shell.appendOutput(`> ${name}`)
  try {
    const result = await work()
    const issues = result && typeof result === 'object' && Array.isArray((result as ValidationReport).issues)
      ? (result as ValidationReport).issues
      : null
    if (issues) {
      shell.setProblems(name, issues)
      if (issues.length) shell.showDock('problems')
    }
    const count = issues?.length ?? 0
    shell.appendOutput(`${name}: ${count ? `${count} issue(s).` : 'completed.'}`, count ? 'error' : 'success')
    toasts.push(name, count ? `${count} issues found.` : 'Completed successfully.', count ? 'danger' : 'success')
  } catch (error) {
    shell.appendOutput(`${name} failed: ${String(error)}`, 'error')
    toasts.push(`${name} failed`, String(error), 'danger')
  } finally {
    busy.value = ''
  }
}

async function createProject() {
  if (!form.id.trim()) return
  busy.value = 'create'
  try {
    await projectsCreate({
      category: form.category,
      id: form.id.trim(),
      name: form.name.trim() || null,
      minecraft_version: form.minecraft.trim() || null,
      loader: form.loader || null,
      variants: form.variants.split(',').map((value) => value.trim()).filter(Boolean),
      role: 'none',
    })
    await workbench.refresh()
    workbench.selectProject(form.id.trim())
    createOpen.value = false
    shell.appendOutput(`Created project ${form.id.trim()}.`, 'success')
    toasts.push('Project created', form.id.trim(), 'success')
  } catch (error) {
    toasts.push('Project creation failed', String(error), 'danger')
  } finally {
    busy.value = ''
  }
}

const actions = [
  { label: 'Validate', icon: 'shield', hint: 'Manifests, roles, variants, releases', run: () => run('Validate workspace', diagnosticsValidate) },
  { label: 'Preflight', icon: 'check', hint: 'Composite release gate', run: () => run('Preflight', diagnosticsPreflight) },
  { label: 'Dry sync', icon: 'sync', hint: 'Preview base-to-consumer changes', run: () => run('Sync preview', workspaceSyncPreview) },
]
</script>

<template>
  <section v-if="!workbench.selectedProject" class="grid view-grid">
    <div class="panel span-12">
      <EmptyState
        title="No projects indexed"
        message="Create the first modpack, mod, data pack, or resource pack in this workspace."
      >
        <template #action><Button @click="createOpen = true">New project</Button></template>
      </EmptyState>
    </div>
  </section>

  <section v-else class="overview">
    <!-- Primary column: the pack targets are what the user actually acts on. -->
    <div class="overview__main">
      <div class="overview-metrics" aria-label="Project summary">
        <div class="overview-metric">
          <span>Pack targets</span>
          <strong>{{ workbench.projectPacks.length }}</strong>
          <small>{{ manifest?.variants.length || 0 }} declared variants</small>
        </div>
        <div class="overview-metric">
          <span>Indexed files</span>
          <strong>{{ totalIndexedFiles.toLocaleString() }}</strong>
          <small>{{ totalMetadataFiles.toLocaleString() }} metadata files</small>
        </div>
        <div class="overview-metric">
          <span>Loaders</span>
          <strong>{{ projectLoaders.length || (manifest?.loader ? 1 : 0) }}</strong>
          <small>{{ projectLoaders.join(', ') || manifest?.loader || 'Not specified' }}</small>
        </div>
      </div>

      <div v-if="workbench.selectedPack" class="selected-target">
        <div>
          <span class="eyebrow">Selected target</span>
          <h2>{{ workbench.selectedPack.name }}</h2>
          <p>{{ workbench.selectedPack.path }}</p>
        </div>
        <dl>
          <div><dt>Minecraft</dt><dd>{{ workbench.selectedPack.minecraftVersion || 'unset' }}</dd></div>
          <div><dt>Loader</dt><dd>{{ workbench.selectedPack.loaders.join(', ') || 'unset' }}</dd></div>
          <div><dt>Version</dt><dd>{{ workbench.selectedPack.version || 'unset' }}</dd></div>
        </dl>
      </div>

      <div class="section-head">
        <h2>Pack targets</h2>
        <span class="pill">{{ filteredPacks.length }} of {{ workbench.projectPacks.length }}</span>
      </div>
      <div v-if="filteredPacks.length" class="list target-list">
        <button
          v-for="pack in filteredPacks"
          :key="pack.id"
          class="row target-row"
          :class="{ selected: workbench.selectedPack?.id === pack.id }"
          @click="workbench.selectPack(pack.id)"
        >
          <span class="target-row__id">
            <strong>{{ pack.name }}</strong>
            <small>{{ pack.id }}</small>
          </span>
          <span class="target-row__meta">
            {{ pack.minecraftVersion || '—' }} · {{ pack.loaders.join(', ') || '—' }}
            <em>{{ pack.indexedFiles }} files</em>
          </span>
        </button>
      </div>
      <p v-else class="empty-note">No pack targets match the current search.</p>

      <div class="section-head section-head--spaced">
        <h2>Workspace</h2>
      </div>
      <div class="quiet-actions">
        <button v-for="action in actions" :key="action.label" class="quiet-action" @click="action.run()">
          <AppIcon :name="action.icon" :size="15" />
          <span><strong>{{ action.label }}</strong><small>{{ action.hint }}</small></span>
        </button>
        <button
          class="quiet-action"
          :disabled="!workbench.selectedPack"
          @click="run('Refresh pack', () => modsRefresh(workbench.selectedPack!.id))"
        >
          <AppIcon name="refresh" :size="15" />
          <span><strong>Refresh pack</strong><small>Rebuild hashes and metadata</small></span>
        </button>
      </div>
      <!-- One accented control on the view; everything else stays quiet. -->
      <div class="action-row panel-bottom-actions">
        <Button :busy="busy === 'Apply sync'" @click="run('Apply sync', workspaceSync)">Apply workspace sync</Button>
        <Button variant="quiet" @click="createOpen = true">New project</Button>
      </div>
    </div>

    <!-- Secondary column: reference facts, deliberately lower contrast. -->
    <aside class="overview__aside">
      <div class="section-head">
        <h2>Project</h2>
        <span class="pill">{{ workbench.selectedProject.category }}</span>
      </div>
      <dl class="facts">
        <div><dt>ID</dt><dd>{{ manifest?.id }}</dd></div>
        <div><dt>Version</dt><dd>{{ manifest?.version || 'unset' }}</dd></div>
        <div><dt>Lifecycle</dt><dd>{{ manifest?.lifecycle || 'active' }}</dd></div>
        <div><dt>Minecraft</dt><dd>{{ manifest?.mc_version || 'per variant' }}</dd></div>
        <div><dt>Loader</dt><dd>{{ manifest?.loader || 'per variant' }}</dd></div>
        <div><dt>Targets</dt><dd>{{ workbench.projectPacks.length }}</dd></div>
        <div><dt>Variants</dt><dd>{{ manifest?.variants.length || 0 }}</dd></div>
      </dl>
      <p class="project-root">{{ workbench.selectedProject.root }}</p>
    </aside>
  </section>

  <Modal :open="createOpen" title="Create project" @close="createOpen = false">
    <form class="form-grid" @submit.prevent="createProject">
      <label>
        <span>Category</span>
        <select v-model="form.category">
          <option value="modpacks">Modpack</option>
          <option value="mods">Mod</option>
          <option value="datapacks">Datapack</option>
          <option value="resourcepacks">Resource pack</option>
        </select>
      </label>
      <label><span>Project ID</span><input v-model="form.id" required placeholder="my-project" /></label>
      <label><span>Display name</span><input v-model="form.name" placeholder="Defaults to ID" /></label>
      <label><span>Minecraft</span><input v-model="form.minecraft" /></label>
      <label>
        <span>Loader</span>
        <select v-model="form.loader">
          <option>fabric</option>
          <option>neoforge</option>
          <option>forge</option>
          <option>quilt</option>
        </select>
      </label>
      <label><span>Variants</span><input v-model="form.variants" placeholder="1.21.1-fabric, 1.21.4-fabric" /></label>
      <div class="form-actions">
        <Button variant="quiet" @click="createOpen = false">Cancel</Button>
        <Button type="submit" :busy="busy === 'create'">Create</Button>
      </div>
    </form>
  </Modal>
</template>
