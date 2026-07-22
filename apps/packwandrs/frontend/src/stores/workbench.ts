import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { packsList } from '@/helpers/invoke/packs'
import { projectsList } from '@/helpers/invoke/projects'
import type { PackSummary, WorkspaceProject } from '@/helpers/types'

function normalized(path: string) {
  return path.replaceAll('\\', '/').replace(/\/$/, '').toLowerCase()
}

export const useWorkbenchStore = defineStore('workbench', () => {
  const projects = ref<WorkspaceProject[]>([])
  const packs = ref<PackSummary[]>([])
  const selectedProjectId = ref(localStorage.getItem('packwand:selected-project') ?? '')
  const selectedPackId = ref(localStorage.getItem('packwand:selected-pack') ?? '')
  const search = ref('')
  const loading = ref(false)
  const error = ref('')
  const selectedProject = computed(() => projects.value.find((project) => project.manifest.id === selectedProjectId.value) ?? projects.value[0] ?? null)
  const projectPacks = computed(() => {
    if (!selectedProject.value) return packs.value
    const root = normalized(selectedProject.value.root)
    return packs.value.filter((pack) => normalized(pack.path) === root || normalized(pack.path).startsWith(root + '/'))
  })
  const selectedPack = computed(() => projectPacks.value.find((pack) => pack.id === selectedPackId.value) ?? projectPacks.value[0] ?? null)
  const title = computed(() => selectedProject.value?.manifest.name || selectedProject.value?.manifest.id || selectedPack.value?.name || 'Packwand')
  const summary = computed(() => {
    const project = selectedProject.value
    if (!project) return selectedPack.value?.path ?? 'No projects indexed'
    return [project.category, project.manifest.mc_version, project.manifest.loader, project.manifest.version].filter(Boolean).join(' · ')
  })
  function selectProject(id: string) {
    selectedProjectId.value = id
    localStorage.setItem('packwand:selected-project', id)
    selectedPackId.value = projectPacks.value[0]?.id ?? ''
    if (selectedPackId.value) localStorage.setItem('packwand:selected-pack', selectedPackId.value)
  }
  function selectPack(id: string) { selectedPackId.value = id; localStorage.setItem('packwand:selected-pack', id) }
  async function refresh() {
    loading.value = true; error.value = ''
    try {
      const [nextProjects, nextPacks] = await Promise.all([projectsList(), packsList()])
      projects.value = nextProjects; packs.value = nextPacks
      if (!projects.value.some((project) => project.manifest.id === selectedProjectId.value)) selectedProjectId.value = projects.value[0]?.manifest.id ?? ''
      if (!projectPacks.value.some((pack) => pack.id === selectedPackId.value)) selectedPackId.value = projectPacks.value[0]?.id ?? ''
      if (selectedProjectId.value) localStorage.setItem('packwand:selected-project', selectedProjectId.value)
      if (selectedPackId.value) localStorage.setItem('packwand:selected-pack', selectedPackId.value)
    } catch (caught) { error.value = String(caught); throw caught } finally { loading.value = false }
  }
  return { projects, packs, selectedProjectId, selectedPackId, selectedProject, projectPacks, selectedPack, title, summary, search, loading, error, selectProject, selectPack, refresh }
})
