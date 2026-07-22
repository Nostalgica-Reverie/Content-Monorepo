<script setup lang="ts">
import { computed, ref } from 'vue'

import PackwandWorkbench from '@/components/PackwandWorkbench.vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import { normalizeBridgeError } from '@/helpers/errors'
import { diagnosticsPreflight } from '@/helpers/invoke/diagnostics'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const reload = ref(0)
const packId = computed(() => workbench.selectedPack?.id ?? '')

async function validate() {
  try {
    const result = await diagnosticsPreflight()
    toasts.push('Preflight complete', result.issues.length ? `${result.issues.length} issues found.` : 'All checks passed.', result.issues.length ? 'danger' : 'success')
  } catch (error) {
    toasts.push('Preflight failed', normalizeBridgeError(error).message, 'danger')
  }
}
</script>

<template>
  <section class="grid view-grid">
    <div class="panel span-12 ide-editor-panel">
      <div class="panel-head">
        <div>
          <h2>Packwand IDE</h2>
          <p class="panel-copy">The Packwand-focused Code OSS workbench, confined to {{ workbench.selectedPack?.name || 'the active pack' }}.</p>
        </div>
        <div class="panel-actions">
          <Button variant="quiet" @click="reload++">Reload IDE</Button>
          <Button @click="validate">Validate pack</Button>
        </div>
      </div>
      <EmptyState v-if="!packId" title="No pack target" message="Select a project with at least one pack.toml target." />
      <PackwandWorkbench v-else :pack-id="packId" :reload="reload" />
    </div>
  </section>
</template>
