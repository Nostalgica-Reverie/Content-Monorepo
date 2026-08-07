<script setup lang="ts">
/**
 * Pack file explorer, in the packwand sidebar.
 *
 * Packwand owns this navigation surface and opens files directly in Monaco.
 *
 * Directories expand lazily. A pack can hold thousands of files, and walking
 * the whole tree on selection would stall the sidebar for the common case of
 * opening a single config.
 */
import { ref, watch } from 'vue'

import FileTreeNode from '@/components/shell/FileTreeNode.vue'
import SideSection from '@/components/shell/SideSection.vue'
import { readLevel, type FileNode } from '@/helpers/filetree'

const props = defineProps<{ packId: string }>()
const emit = defineEmits<{ open: [path: string] }>()

const roots = ref<FileNode[]>([])
const rootError = ref<string | null>(null)
const loading = ref(false)

async function loadRoot() {
  if (!props.packId) {
    roots.value = []
    rootError.value = null
    return
  }
  loading.value = true
  rootError.value = null
  try {
    roots.value = await readLevel(props.packId, '')
  } catch (error) {
    rootError.value = String(error)
    roots.value = []
  } finally {
    loading.value = false
  }
}

async function toggle(node: FileNode) {
  if (!node.directory) {
    emit('open', node.path)
    return
  }
  if (node.expanded) {
    node.expanded = false
    return
  }
  node.expanded = true

  // Children are fetched once and kept. Re-reading on every expand would make
  // the tree flicker, and would lose nothing but staleness that the refresh
  // button already fixes.
  if (node.children) return

  node.loading = true
  node.error = null
  try {
    node.children = await readLevel(props.packId, node.path)
  } catch (error) {
    node.error = String(error)
    node.children = []
  } finally {
    node.loading = false
  }
}

watch(() => props.packId, loadRoot, { immediate: true })
defineExpose({ refresh: loadRoot })
</script>

<template>
  <SideSection title="Files" :count="roots.length || undefined">
    <p v-if="!props.packId" class="side-empty">Select a pack target to browse its files.</p>
    <p v-else-if="loading" class="side-empty">Reading pack…</p>
    <p v-else-if="rootError" class="side-empty">{{ rootError }}</p>
    <p v-else-if="!roots.length" class="side-empty">This pack has no files.</p>
    <ul v-else class="filetree">
      <FileTreeNode
        v-for="node in roots"
        :key="node.path"
        :node="node"
        :depth="0"
        @toggle="toggle"
        @open="emit('open', $event)"
      />
    </ul>
  </SideSection>
</template>

<style scoped>
.filetree {
  list-style: none;
  margin: 0;
  padding: 0;
}
</style>
