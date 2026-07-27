import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
  commands: [
    {
      id: 'validate',
      title: 'Validate KubeJS scripts',
      icon: 'shield',
      when: ['modpacks'],
      async run(context) {
        const pack = context.activePack()
        if (!pack) return context.notify('PackwandJS', 'Select a pack target first.', 'danger')
        context.output(`> PackwandJS: validate ${pack.id}`)
        const report = await context.kubejs.validate(pack.id)
        context.publishProblems('PackwandJS', report.issues)
        context.output(
          `PackwandJS: ${report.issues.length} issue(s) across ${report.checked} KubeJS script(s).`,
          report.issues.length ? 'error' : 'success',
        )
        context.notify(
          'PackwandJS',
          report.issues.length ? `${report.issues.length} issue(s) found.` : 'KubeJS scripts passed.',
          report.issues.length ? 'danger' : 'success',
        )
      },
    },
  ],
  views: [
    {
      id: 'scripts',
      title: 'KubeJS scripts',
      icon: 'editor',
      when: ['modpacks'],
      async rows(context) {
        const pack = context.activePack()
        if (!pack) return []
        return (await context.kubejs.scripts(pack.id)).map((script) => ({
          label: script.name,
          detail: script.path,
          icon: 'editor',
          run: () => context.editor.open(pack.id, script.path),
        }))
      },
    },
  ],
})
