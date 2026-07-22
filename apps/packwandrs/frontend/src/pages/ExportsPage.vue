<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import { exportBuild, exportPlan, publishBuild, publishInspect, publishTargets, publishUpload, publishVerify } from '@/helpers/invoke/exports'
import type { ExportPlan, JobRecord, PublishTarget } from '@/helpers/types'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const packId = computed(() => workbench.selectedPack?.id ?? '')
const projectId = computed(() => workbench.selectedProject?.manifest.id ?? '')
const variant = ref<string | null>(null)
const variants = ref<Array<string | null>>([])
const plan = ref<ExportPlan | null>(null)
const target = ref<PublishTarget | null>(null)
const job = ref<JobRecord | null>(null)
const busy = ref('')
const publishConfigured = computed(() => Boolean(workbench.selectedProject?.manifest.modrinth_id || workbench.selectedProject?.manifest.curseforge_id))

watch(projectId, async (id) => {
  target.value = null; variant.value = null; variants.value = []
  if (!id) return
  try { const rows = await publishTargets(id); variants.value = [...new Set(rows.map((row) => row.variant))]; variant.value = variants.value[0] ?? null }
  catch (error) { if (publishConfigured.value) toasts.push('Publish matrix failed', String(error), 'danger') }
}, { immediate: true })

async function execute(name: string, work: () => Promise<JobRecord>) {
  busy.value = name
  try { job.value = await work(); toasts.push(name + ' started', job.value.label, 'success') }
  catch (error) { toasts.push(name + ' failed', String(error), 'danger') } finally { busy.value = '' }
}
async function inspectArchive() { busy.value = 'Archive plan'; try { plan.value = await exportPlan(packId.value) } catch (error) { toasts.push('Plan failed', String(error), 'danger') } finally { busy.value = '' } }
async function inspectPublish() { busy.value = 'Inspect publish'; try { target.value = await publishInspect(projectId.value, variant.value) } catch (error) { toasts.push('Inspect failed', String(error), 'danger') } finally { busy.value = '' } }
async function liveUpload() { if (window.confirm('Upload these artifacts to the configured public release platforms? This is not reversible.')) await execute('Live upload', () => publishUpload(projectId.value, variant.value, true)) }
</script>

<template>
  <section class="grid view-grid">
    <article class="panel span-6">
      <div class="panel-head"><h2>Pack archive</h2><span class="status-badge">{{ workbench.selectedPack?.packFormat || 'no target' }}</span></div>
      <p class="panel-copy">Build a launcher-ready Modrinth or CurseForge archive from the active pack target.</p>
      <label class="field-stack"><span>Pack target</span><select :value="workbench.selectedPackId" @change="workbench.selectPack(($event.target as HTMLSelectElement).value)"><option v-for="pack in workbench.projectPacks" :key="pack.id" :value="pack.id">{{ pack.name }} · {{ pack.id }}</option></select></label>
      <div class="action-row"><Button variant="quiet" :busy="busy === 'Archive plan'" :disabled="!packId" @click="inspectArchive">Inspect plan</Button><Button :disabled="!packId" @click="execute('Archive build', () => exportBuild(packId))">Build archive</Button></div>
    </article>
    <article class="panel span-6">
      <div class="panel-head"><h2>Publish release</h2><span :class="['status-badge', { integrated: publishConfigured }]">{{ publishConfigured ? 'configured' : 'not configured' }}</span></div>
      <p class="panel-copy">Build, upload, and verify the selected manifest variant against its configured platforms.</p>
      <label class="field-stack"><span>Variant</span><select v-model="variant"><option v-for="item in variants" :key="item ?? 'default'" :value="item">{{ item || 'Default' }}</option></select></label>
      <div class="action-row"><Button variant="quiet" :disabled="!publishConfigured" @click="inspectPublish">Inspect</Button><Button :disabled="!publishConfigured" @click="execute('Publish build', () => publishBuild(projectId, variant))">Build release</Button></div>
    </article>

    <article v-if="plan" class="panel span-12 compact-panel">
      <div class="panel-head"><h2>Archive plan</h2><span class="pill">{{ plan.outputStem }}</span></div>
      <div class="details"><div class="detail"><span>Pack</span><strong>{{ plan.packName }} {{ plan.packVersion }}</strong></div><div class="detail"><span>Indexed files</span><strong>{{ plan.indexedFiles }}</strong></div><div class="detail"><span>Metadata files</span><strong>{{ plan.metadataFiles }}</strong></div></div>
    </article>

    <article v-if="target" class="panel span-12 release-panel">
      <div class="panel-head"><div><h2>{{ target.name }} {{ target.version }}</h2><p class="panel-copy">{{ target.minecraftVersion }} · {{ target.loader }} · {{ target.releaseType }}</p></div><span class="pill">{{ target.variant || 'default' }}</span></div>
      <div class="artifact-list"><div v-for="artifact in target.artifacts" :key="artifact.platform" class="artifact-row"><strong>{{ artifact.platform }}</strong><code>{{ artifact.path }}</code><span :class="['status-badge', { integrated: artifact.exists }]">{{ artifact.exists ? artifact.bytes + ' bytes' : 'not built' }}</span></div></div>
      <div class="action-row release-actions"><Button variant="quiet" @click="execute('Upload dry-run', () => publishUpload(projectId, variant, false))">Dry-run upload</Button><Button variant="secondary" @click="execute('Verification', () => publishVerify(projectId, variant))">Verify release</Button><Button variant="danger" @click="liveUpload">Live upload</Button></div>
    </article>
    <EmptyState v-if="!packId" title="No export target" message="Select a project containing a pack.toml target." />
    <div v-if="job" class="success-banner span-12">{{ job.label }} queued. Follow progress and output in Logs.</div>
  </section>
</template>
