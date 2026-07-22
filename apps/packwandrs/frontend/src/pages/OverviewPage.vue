<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import Modal from '@/components/ui/Modal.vue'
import { diagnosticsPreflight, diagnosticsValidate } from '@/helpers/invoke/diagnostics'
import { modsRefresh } from '@/helpers/invoke/mods'
import { projectsCreate } from '@/helpers/invoke/projects'
import { workspaceSync, workspaceSyncPreview } from '@/helpers/invoke/workspace'
import type { NewProjectRequest } from '@/helpers/types'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const busy = ref('')
const createOpen = ref(false)
const form = reactive({ category: 'modpacks' as NewProjectRequest['category'], id: '', name: '', minecraft: '1.21.1', loader: 'fabric', variants: '' })
const filteredPacks = computed(() => {
  const query = workbench.search.trim().toLowerCase()
  return workbench.projectPacks.filter((pack) => !query || (pack.name + ' ' + pack.id + ' ' + pack.path).toLowerCase().includes(query))
})
const manifest = computed(() => workbench.selectedProject?.manifest)

function variantName(variant: Record<string, unknown>, index: number) {
  return String(variant.id ?? variant.name ?? variant.version ?? 'Variant ' + (index + 1))
}
async function run(name: string, work: () => Promise<unknown>) {
  busy.value = name
  try { const result = await work(); const issueCount = typeof result === 'object' && result && 'issues' in result ? (result as { issues: unknown[] }).issues.length : 0; toasts.push(name, issueCount ? issueCount + ' issues found.' : 'Completed successfully.', issueCount ? 'danger' : 'success') }
  catch (error) { toasts.push(name + ' failed', String(error), 'danger') } finally { busy.value = '' }
}
async function createProject() {
  if (!form.id.trim()) return
  busy.value = 'create'
  try {
    await projectsCreate({ category: form.category, id: form.id.trim(), name: form.name.trim() || null, minecraft_version: form.minecraft.trim() || null, loader: form.loader || null, variants: form.variants.split(',').map((value) => value.trim()).filter(Boolean), role: 'none' })
    await workbench.refresh(); workbench.selectProject(form.id.trim()); createOpen.value = false
    toasts.push('Project created', form.id.trim(), 'success')
  } catch (error) { toasts.push('Project creation failed', String(error), 'danger') } finally { busy.value = '' }
}
</script>

<template>
  <section v-if="!workbench.selectedProject" class="grid view-grid">
    <div class="panel span-12 empty-workspace"><EmptyState title="No projects indexed" message="Create the first modpack, mod, data pack, or resource pack in this workspace."><template #action><Button @click="createOpen = true">New project</Button></template></EmptyState></div>
  </section>
  <section v-else class="grid view-grid">
    <article class="panel span-5">
      <div class="panel-head"><h2>Project</h2><span class="pill">{{ workbench.selectedProject.category }}</span></div>
      <div class="details">
        <div class="detail"><span>ID</span><strong>{{ manifest?.id }}</strong></div>
        <div class="detail"><span>Version</span><strong>{{ manifest?.version || 'unset' }}</strong></div>
        <div class="detail"><span>Lifecycle</span><strong>{{ manifest?.lifecycle || 'active' }}</strong></div>
        <div class="detail"><span>Minecraft</span><strong>{{ manifest?.mc_version || 'per variant' }}</strong></div>
        <div class="detail"><span>Loader</span><strong>{{ manifest?.loader || 'per variant' }}</strong></div>
        <div class="detail"><span>Targets</span><strong>{{ workbench.projectPacks.length }}</strong></div>
      </div>
      <p class="project-root">{{ workbench.selectedProject.root }}</p>
    </article>

    <article class="panel span-7">
      <div class="panel-head"><h2>Pack targets</h2><span class="status-badge integrated">{{ filteredPacks.length }}</span></div>
      <div v-if="filteredPacks.length" class="list target-list">
        <button v-for="pack in filteredPacks" :key="pack.id" class="row target-row" :class="{ selected: workbench.selectedPack?.id === pack.id }" @click="workbench.selectPack(pack.id)">
          <span><strong>{{ pack.name }}</strong><small>{{ pack.id }}</small></span>
          <span>{{ pack.minecraftVersion || '—' }} · {{ pack.loaders.join(', ') || '—' }}</span>
        </button>
      </div>
      <p v-else class="empty-note">No pack targets match the current search.</p>
    </article>

    <article class="panel span-6">
      <div class="panel-head"><h2>Workspace actions</h2><span class="status-badge">native</span></div>
      <p class="panel-copy">The legacy operational shortcuts, now backed directly by the Rust command surface.</p>
      <div class="action-cards">
        <button @click="run('Validate workspace', diagnosticsValidate)"><strong>Validate</strong><span>Check manifests, roles, variants, and releases.</span></button>
        <button @click="run('Preflight', diagnosticsPreflight)"><strong>Preflight</strong><span>Run the composite release gate.</span></button>
        <button @click="run('Sync preview', workspaceSyncPreview)"><strong>Dry sync</strong><span>Preview base-to-consumer changes.</span></button>
        <button :disabled="!workbench.selectedPack" @click="run('Refresh metadata', () => modsRefresh(workbench.selectedPack!.id))"><strong>Refresh pack</strong><span>Rebuild hashes and metadata for this target.</span></button>
      </div>
      <div class="action-row panel-bottom-actions"><Button variant="danger" :busy="busy === 'Apply sync'" @click="run('Apply sync', workspaceSync)">Apply workspace sync</Button><Button variant="quiet" @click="createOpen = true">New project</Button></div>
    </article>

    <article class="panel span-6">
      <div class="panel-head"><h2>Variants</h2><span class="status-badge">{{ manifest?.variants.length || 0 }}</span></div>
      <div v-if="manifest?.variants.length" class="variant-list">
        <div v-for="(variant, index) in manifest.variants" :key="index" class="mini-row"><strong>{{ variantName(variant, index) }}</strong><span>{{ Object.entries(variant).filter(([key]) => key !== 'id' && key !== 'name').map(([key, value]) => key + ': ' + String(value)).join(' · ') || 'Default project settings' }}</span></div>
      </div>
      <p v-else class="empty-note">This project uses its root manifest without named variants.</p>
    </article>
  </section>

  <Modal :open="createOpen" title="Create project" @close="createOpen = false">
    <form class="form-grid" @submit.prevent="createProject">
      <label><span>Category</span><select v-model="form.category"><option value="modpacks">Modpack</option><option value="mods">Mod</option><option value="datapacks">Datapack</option><option value="resourcepacks">Resource pack</option></select></label>
      <label><span>Project ID</span><input v-model="form.id" required placeholder="my-project" /></label>
      <label><span>Display name</span><input v-model="form.name" placeholder="Defaults to ID" /></label>
      <label><span>Minecraft</span><input v-model="form.minecraft" /></label>
      <label><span>Loader</span><select v-model="form.loader"><option>fabric</option><option>neoforge</option><option>forge</option><option>quilt</option></select></label>
      <label><span>Variants</span><input v-model="form.variants" placeholder="1.21.1-fabric, 1.21.4-fabric" /></label>
      <div class="form-actions"><Button variant="quiet" @click="createOpen = false">Cancel</Button><Button type="submit" :busy="busy === 'create'">Create</Button></div>
    </form>
  </Modal>
</template>
