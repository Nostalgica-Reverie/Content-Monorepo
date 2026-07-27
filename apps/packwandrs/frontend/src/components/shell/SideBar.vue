<script setup lang="ts">
import { computed } from 'vue'
import ExtensionSection from '@/components/shell/ExtensionSection.vue'
import SideSection from '@/components/shell/SideSection.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { useExtensionsStore } from '@/stores/extensions'
import { useShellStore } from '@/stores/shell'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const shell = useShellStore()
const extensionsStore = useExtensionsStore()

const emit = defineEmits<{ newProject: [] }>()

/** Only the selected project expands its targets — keeps the tree scannable. */
const targets = computed(() => workbench.projectPacks)

function variantLabel(variant: Record<string, unknown>, index: number) {
  return String(variant.id ?? variant.name ?? variant.version ?? `Variant ${index + 1}`)
}
</script>

<template>
  <aside class="side" :style="{ width: shell.sidebarWidth + 'px' }" aria-label="Explorer">
    <div class="side-head">
      <span class="eyebrow">Explorer</span>
      <div class="panel-actions">
        <button class="icon-btn" title="New project" aria-label="New project" @click="emit('newProject')">
          <AppIcon name="plus" :size="15" />
        </button>
        <button
          class="icon-btn"
          title="Reindex workspace"
          aria-label="Reindex workspace"
          :disabled="workbench.loading"
          @click="workbench.refresh()"
        >
          <AppIcon name="refresh" :size="15" />
        </button>
      </div>
    </div>

    <div class="side-body">
      <SideSection title="Projects" :count="workbench.projects.length">
        <p v-if="!workbench.projects.length" class="side-empty">
          {{ workbench.loading ? 'Indexing…' : 'No projects indexed yet.' }}
        </p>
        <template v-for="project in workbench.projects" :key="project.manifest.id">
          <button
            class="tree-row"
            :class="{ active: workbench.selectedProject?.manifest.id === project.manifest.id }"
            :title="project.root"
            @click="workbench.selectProject(project.manifest.id)"
          >
            <AppIcon name="folder" :size="15" class="tree-row__icon" />
            <span class="tree-row__label">{{ project.manifest.name || project.manifest.id }}</span>
            <span class="tree-row__meta">{{ project.category }}</span>
          </button>
        </template>
      </SideSection>

      <SideSection title="Pack targets" :count="targets.length">
        <p v-if="!targets.length" class="side-empty">This project has no pack targets.</p>
        <button
          v-for="pack in targets"
          :key="pack.id"
          class="tree-row"
          :class="{ active: workbench.selectedPack?.id === pack.id }"
          :title="pack.path"
          @click="workbench.selectPack(pack.id)"
        >
          <AppIcon name="target" :size="15" class="tree-row__icon" />
          <span class="tree-row__label">{{ pack.name }}</span>
          <span class="tree-row__meta">{{ pack.minecraftVersion || '—' }}</span>
        </button>
      </SideSection>

      <SideSection
        v-if="workbench.selectedProject"
        title="Variants"
        :count="workbench.selectedProject.manifest.variants.length"
        :open="false"
      >
        <p v-if="!workbench.selectedProject.manifest.variants.length" class="side-empty">
          Root manifest, no named variants.
        </p>
        <div
          v-for="(variant, index) in workbench.selectedProject.manifest.variants"
          :key="index"
          class="tree-row tree-row--nested"
        >
          <span class="tree-row__label">{{ variantLabel(variant, index) }}</span>
        </div>
      </SideSection>

      <!-- Extension-contributed views sit below the built-in tree. -->
      <ExtensionSection v-for="entry in extensionsStore.views" :key="entry.id" :entry="entry" />
    </div>
  </aside>
</template>
