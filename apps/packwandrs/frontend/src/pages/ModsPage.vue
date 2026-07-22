<script setup lang="ts">
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import Modal from '@/components/ui/Modal.vue'
import { modPin, modRemove, modSideSet, modsList, modsRefresh } from '@/helpers/invoke/mods'
import { providerAdd } from '@/helpers/invoke/providers'
import type { ModSummary, ProviderKind } from '@/helpers/types'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const client = useQueryClient()
const id = computed(() => workbench.selectedPack?.id ?? '')
const working = ref<string | null>(null)
const addOpen = ref(false)
const adding = ref(false)
const addForm = ref({ provider: 'modrinth' as ProviderKind, project: '', gameVersions: '', loaders: '', token: '', instance: '', branch: '', pattern: '' })
const mods = useQuery({ queryKey: ['mods', id], queryFn: () => modsList(id.value), enabled: () => Boolean(id.value) })
const filtered = computed(() => {
  const term = workbench.search.trim().toLowerCase()
  return mods.data.value?.filter((mod) => !term || (mod.name + ' ' + mod.filename + ' ' + mod.metadataPath).toLowerCase().includes(term)) ?? []
})

async function mutate(mod: ModSummary, action: () => Promise<unknown>, success: string) {
  working.value = mod.metadataPath
  try { await action(); await client.invalidateQueries({ queryKey: ['mods', id.value] }); toasts.push(success, mod.name, 'success') }
  catch (error) { toasts.push('Mod operation failed', String(error), 'danger') } finally { working.value = null }
}
const pin = (mod: ModSummary) => mutate(mod, () => modPin(id.value, mod.metadataPath, !mod.pinned), mod.pinned ? 'Mod unpinned' : 'Mod pinned')
const side = (mod: ModSummary, value: string) => mutate(mod, () => modSideSet(id.value, mod.metadataPath, value), 'Side updated')
async function remove(mod: ModSummary) { if (window.confirm('Remove ' + mod.name + '?')) await mutate(mod, () => modRemove(id.value, mod.metadataPath), 'Removal queued') }
async function refresh() { try { const job = await modsRefresh(id.value); toasts.push('Refresh queued', job.label, 'success') } catch (error) { toasts.push('Refresh failed', String(error), 'danger') } }
async function add() {
  adding.value = true
  try {
    const path = await providerAdd(id.value, addForm.value.provider, {
      project: addForm.value.project.trim(), game_versions: addForm.value.gameVersions.split(',').map((value) => value.trim()).filter(Boolean),
      loaders: addForm.value.loaders.split(',').map((value) => value.trim()).filter(Boolean), channels: ['release', 'beta', 'alpha'],
      branch: addForm.value.branch.trim() || null, asset_pattern: addForm.value.pattern.trim() || null,
    }, addForm.value.token || null, addForm.value.instance || null)
    await client.invalidateQueries({ queryKey: ['mods', id.value] }); toasts.push('Project added', path, 'success'); addOpen.value = false
  } catch (error) { toasts.push('Provider add failed', String(error), 'danger') } finally { adding.value = false }
}
</script>

<template>
  <section class="grid view-grid">
    <div class="panel span-12 mods-panel">
      <div class="panel-head">
        <div><h2>Mods</h2><p class="panel-copy">Provider metadata in {{ workbench.selectedPack?.name || 'the active pack target' }}.</p></div>
        <div class="panel-actions"><span class="pill">{{ filtered.length }} mods</span><Button variant="quiet" :disabled="!id" @click="refresh">Refresh metadata</Button><Button :disabled="!id" @click="addOpen = true">Add project</Button></div>
      </div>
      <EmptyState v-if="!id" title="No pack target" message="Choose a project containing a pack.toml target." />
      <EmptyState v-else-if="!mods.isPending.value && !filtered.length" title="No mod metadata" message="Add a Modrinth, CurseForge, GitHub, Forgejo, or GitLab project to this target." />
      <div v-else class="mod-list list">
        <div v-for="mod in filtered" :key="mod.metadataPath" class="row mod-row">
          <div><strong>{{ mod.name }}</strong><span>{{ mod.filename }} · {{ mod.metadataPath }}</span></div>
          <span class="provider-label">{{ mod.providers.join(', ') || 'direct' }}</span>
          <select :value="mod.side || 'both'" :disabled="working === mod.metadataPath" @change="side(mod, ($event.target as HTMLSelectElement).value)"><option value="both">Both sides</option><option value="client">Client</option><option value="server">Server</option></select>
          <button class="pin-toggle" :class="{ active: mod.pinned }" :disabled="working === mod.metadataPath" @click="pin(mod)">{{ mod.pinned ? 'Pinned' : 'Updates on' }}</button>
          <Button variant="danger" :busy="working === mod.metadataPath" @click="remove(mod)">Remove</Button>
        </div>
      </div>
    </div>
  </section>
  <Modal :open="addOpen" title="Add provider project" @close="addOpen = false">
    <form class="form-grid" @submit.prevent="add">
      <label><span>Provider</span><select v-model="addForm.provider"><option value="modrinth">Modrinth</option><option value="curse_forge">CurseForge</option><option value="git_hub">GitHub</option><option value="forgejo">Forgejo / Codeberg</option><option value="git_lab">GitLab</option></select></label>
      <label><span>Project ID, slug, or URL</span><input v-model="addForm.project" required placeholder="fabric-api or owner/repo" /></label>
      <label><span>Minecraft versions</span><input v-model="addForm.gameVersions" placeholder="1.21.1, 1.21.4" /></label>
      <label><span>Loaders</span><input v-model="addForm.loaders" placeholder="fabric, neoforge" /></label>
      <label><span>API token / key</span><input v-model="addForm.token" type="password" placeholder="Optional for most providers" /></label>
      <label><span>Self-hosted instance</span><input v-model="addForm.instance" placeholder="git.example.com" /></label>
      <label><span>Branch</span><input v-model="addForm.branch" placeholder="Optional" /></label>
      <label><span>Asset regular expression</span><input v-model="addForm.pattern" placeholder="Optional" /></label>
      <div class="form-actions"><Button variant="quiet" @click="addOpen = false">Cancel</Button><Button type="submit" :busy="adding">Resolve and add</Button></div>
    </form>
  </Modal>
</template>
