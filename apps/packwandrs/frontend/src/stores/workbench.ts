import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { packsList } from '@/helpers/invoke/packs'
import { projectsList } from '@/helpers/invoke/projects'
import type { PackSummary, WorkspaceProject } from '@/helpers/types'
import type { GitDiffDocument } from '@/helpers/invoke/git'

function normalized(path: string) {
  return path.replaceAll('\\', '/').replace(/\/$/, '').toLowerCase()
}

export const useWorkbenchStore = defineStore('workbench', () => {
  const projects = ref<WorkspaceProject[]>([])
  const packs = ref<PackSummary[]>([])
  const selectedProjectId = ref(localStorage.getItem('packwand:selected-project') ?? '')
  const selectedPackId = ref(localStorage.getItem('packwand:selected-pack') ?? '')
  /**
   * Pack-relative path the sidebar file tree asked the editor to open.
   *
   * Deliberately not persisted: it seeds the embedded workbench's initial
   * editor, and restoring it on a cold start would reopen a file the user
   * clicked once, days ago, instead of whatever they were last editing.
   */
  const requestedFile = ref('')
  const requestedDiff = ref<GitDiffDocument | null>(null)
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
  function selectPack(id: string) {
    selectedPackId.value = id
    localStorage.setItem('packwand:selected-pack', id)
    // The path is pack-relative, so it means something different (or nothing)
    // in a different pack.
    requestedFile.value = ''
  }

  /** Ask the editor to open a pack-relative file. */
  function requestFile(path: string) { requestedFile.value = path }
  function requestDiff(diff: GitDiffDocument) { requestedDiff.value = diff }
  async function refresh() {
    loading.value = true; error.value = ''
    try {
      const [projectResult, packResult] = await Promise.allSettled([projectsList(), packsList()])
      if (projectResult.status === 'fulfilled') projects.value = projectResult.value
      if (packResult.status === 'fulfilled') packs.value = packResult.value
      const failures = [projectResult, packResult]
        .filter((result): result is PromiseRejectedResult => result.status === 'rejected')
        .map(result => String(result.reason))
      if (failures.length === 2) throw new Error(failures.join('; '))
      if (failures.length) error.value = `Workspace partially indexed: ${failures.join('; ')}`
      if (!projects.value.some((project) => project.manifest.id === selectedProjectId.value)) selectedProjectId.value = projects.value[0]?.manifest.id ?? ''
      if (!projectPacks.value.some((pack) => pack.id === selectedPackId.value)) selectedPackId.value = projectPacks.value[0]?.id ?? ''
      if (selectedProjectId.value) localStorage.setItem('packwand:selected-project', selectedProjectId.value)
      if (selectedPackId.value) localStorage.setItem('packwand:selected-pack', selectedPackId.value)
    } catch (caught) { error.value = String(caught); throw caught } finally { loading.value = false }
  }
  return { projects, packs, selectedProjectId, selectedPackId, selectedProject, projectPacks, selectedPack, title, summary, search, loading, error, selectProject, selectPack, refresh, requestedFile, requestFile, requestedDiff, requestDiff }
})
