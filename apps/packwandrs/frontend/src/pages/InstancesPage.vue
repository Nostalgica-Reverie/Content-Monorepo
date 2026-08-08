<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'

import AppIcon from '@/components/ui/AppIcon.vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import InstanceCard from '@/components/ui/InstanceCard.vue'
import Modal from '@/components/ui/Modal.vue'
import { instancesImport } from '@/helpers/invoke/instances'
import { useInstancesStore } from '@/stores/instances'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const store = useInstancesStore()
const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const router = useRouter()
const search = ref('')
const createOpen = ref(false)
const importOpen = ref(false)
const saving = ref(false)
const create = reactive({ name: '', source: 'linked' as 'linked' | 'owned', packId: '', gameVersion: '1.21.1', loader: 'fabric', loaderVersion: '' })
const importing = reactive({ archive: '', format: 'modrinth' as 'modrinth' | 'curse_forge' })

const filtered = computed(() => {
  const term = search.value.trim().toLowerCase()
  return store.items.filter(instance => !term || `${instance.name} ${instance.id} ${instance.loader} ${instance.gameVersion}`.toLowerCase().includes(term))
})

async function submitCreate() {
  saving.value = true
  try {
    const instance = await store.create({
      name: create.name,
      source: create.source,
      packId: create.source === 'linked' ? create.packId : undefined,
      gameVersion: create.source === 'owned' ? create.gameVersion : undefined,
      loader: create.source === 'owned' ? create.loader : undefined,
      loaderVersion: create.source === 'owned' && create.loader !== 'vanilla' ? create.loaderVersion || undefined : undefined,
    })
    createOpen.value = false
    await router.push(`/instances/${instance.id}`)
  } catch (error) { toasts.push('Could not create instance', String(error), 'danger') } finally { saving.value = false }
}

async function submitImport() {
  saving.value = true
  try {
    const instance = await instancesImport(importing.archive, importing.format)
    await store.refresh()
    importOpen.value = false
    await router.push(`/instances/${instance.id}`)
  } catch (error) { toasts.push('Could not import archive', String(error), 'danger') } finally { saving.value = false }
}

onMounted(async () => {
  await Promise.all([store.refresh(), workbench.packs.length ? Promise.resolve() : workbench.refresh()])
  create.packId = workbench.selectedPackId || workbench.packs[0]?.id || ''
})
</script>

<template>
  <section class="grid view-grid">
    <div class="panel span-12 instances-panel">
      <div class="panel-head">
        <div><h2>Instances</h2><p class="panel-copy">Isolated Minecraft installations backed by a workspace pack or a private standalone pack. Add <code>icon.png</code> and <code>bg.png</code> to a pack to give its instances custom artwork.</p></div>
        <div class="panel-actions"><Button variant="secondary" @click="importOpen = true">Import</Button><Button @click="createOpen = true">Create instance</Button></div>
      </div>
      <label class="instance-search"><AppIcon name="search" :size="15" /><input v-model="search" type="search" placeholder="Search instances…" /></label>
      <EmptyState v-if="!store.loading && !filtered.length" title="No instances found" message="Create a linked test instance or a standalone Minecraft installation." />
      <div v-else class="instance-list"><InstanceCard v-for="instance in filtered" :key="instance.id" :instance="instance" /></div>
    </div>
  </section>

  <Modal :open="createOpen" title="Create instance" @close="createOpen = false">
    <form class="form-grid" @submit.prevent="submitCreate">
      <label><span>Name</span><input v-model="create.name" required /></label>
      <label><span>Source</span><select v-model="create.source"><option value="linked">Workspace pack</option><option value="owned">Standalone</option></select></label>
      <label v-if="create.source === 'linked'"><span>Pack</span><select v-model="create.packId" required><option v-for="pack in workbench.packs" :key="pack.id" :value="pack.id">{{ pack.name }} — {{ pack.minecraftVersion }}</option></select></label>
      <template v-else>
        <label><span>Minecraft version</span><input v-model="create.gameVersion" required /></label>
        <label><span>Loader</span><select v-model="create.loader"><option>fabric</option><option>quilt</option><option>forge</option><option>neoforge</option><option>vanilla</option></select></label>
        <label v-if="create.loader !== 'vanilla'"><span>Loader version</span><input v-model="create.loaderVersion" placeholder="latest" /></label>
      </template>
      <div class="form-actions"><Button variant="quiet" @click="createOpen = false">Cancel</Button><Button type="submit" :busy="saving">Create</Button></div>
    </form>
  </Modal>

  <Modal :open="importOpen" title="Import instance" @close="importOpen = false">
    <form class="form-grid" @submit.prevent="submitImport">
      <label><span>Archive path</span><input v-model="importing.archive" required placeholder="C:\Downloads\pack.mrpack" /></label>
      <label><span>Format</span><select v-model="importing.format"><option value="modrinth">Modrinth .mrpack</option><option value="curse_forge">CurseForge .zip</option></select></label>
      <div class="form-actions"><Button variant="quiet" @click="importOpen = false">Cancel</Button><Button type="submit" :busy="saving">Import</Button></div>
    </form>
  </Modal>
</template>
