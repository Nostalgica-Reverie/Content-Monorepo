<script setup lang="ts">
/**
 * One row of the pack file tree, recursing into its own children.
 *
 * A separate component rather than nested `<ul>`s in the parent template
 * because a template can only nest to a depth someone typed out, and packs nest
 * deeper than anyone wants to type. Vue resolves a `<script setup>` component
 * by filename, so referencing `FileTreeNode` inside its own template is the
 * supported way to recurse.
 */
import AppIcon from '@/components/ui/AppIcon.vue'
import type { FileNode } from '@/helpers/filetree'

defineProps<{ node: FileNode; depth: number }>()
const emit = defineEmits<{ toggle: [node: FileNode]; open: [path: string] }>()
</script>

<template>
  <li class="filetree__item">
    <button
      class="tree-row"
      :title="node.path"
      :style="{ paddingLeft: `${8 + depth * 12}px` }"
      @click="emit('toggle', node)"
    >
      <AppIcon
        v-if="node.directory"
        :name="node.expanded ? 'chevron-down' : 'chevron-right'"
        :size="12"
        class="filetree__twisty"
      />
      <!-- `editor` is the document glyph; the icon set has no dedicated
           `file`, and inventing one for this is not worth a new path. -->
      <AppIcon
        :name="node.directory ? 'folder' : 'editor'"
        :size="15"
        class="tree-row__icon"
      />
      <span class="tree-row__label">{{ node.name }}</span>
    </button>

    <ul v-if="node.directory && node.expanded" class="filetree">
      <li v-if="node.loading" class="side-empty">Reading…</li>
      <li v-else-if="node.error" class="side-empty">{{ node.error }}</li>
      <FileTreeNode
        v-for="child in node.children ?? []"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        @toggle="emit('toggle', $event)"
        @open="emit('open', $event)"
      />
    </ul>
  </li>
</template>

<style scoped>
.filetree {
  list-style: none;
  margin: 0;
  padding: 0;
}

.filetree__item {
  list-style: none;
}

.filetree__twisty {
  flex: none;
  opacity: 0.7;
}
</style>
