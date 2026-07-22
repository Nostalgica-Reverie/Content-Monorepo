<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import PackwandMascot from '@/components/PackwandMascot.vue'
import Button from '@/components/ui/Button.vue'
import { normalizeBridgeError } from '@/helpers/errors'
import { markWorkspaceConfigured } from '@/router'
import { useWorkspaceStore } from '@/stores/workspace'

const workspace = useWorkspaceStore()
const router = useRouter()
const busy = ref(false)
const error = ref('')

async function selectWorkspace() {
  busy.value = true
  error.value = ''
  try {
    const path = await workspace.select()
    if (path) {
      markWorkspaceConfigured(path)
      await router.replace({ name: 'overview' })
    }
  } catch (caught) {
    error.value = normalizeBridgeError(caught).message
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <main class="setup-page">
    <div class="setup-card">
      <PackwandMascot />
      <p class="eyebrow">First run</p>
      <h1>Choose your workspace</h1>
      <p>Pick the repository root containing <code>mods</code>, <code>modpacks</code>, <code>datapacks</code>, or <code>resourcepacks</code>. Packwand discovers manifest projects and their <code>pack.toml</code> targets locally.</p>
      <div v-if="error" class="error-banner">{{ error }}</div>
      <Button :busy="busy" @click="selectWorkspace">Select workspace</Button>
      <small>Your path is stored in the local app configuration. No server is started.</small>
    </div>
  </main>
</template>
