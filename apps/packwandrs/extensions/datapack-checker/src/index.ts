import { definePackwandExtension } from '@/extensions/api'
import type { ContentRegistry, ValidationReport } from '@/helpers/types'

/**
 * Datapack validation. Every check runs in `packwand-diagnostics` on the Rust
 * side — `content_lint` already resolves namespaces, resource-location charsets,
 * and function/tag references — so this extension only invokes it and presents
 * the result. Re-implementing any of it here would create a second source of
 * truth that drifts from the CLI.
 */
export default definePackwandExtension({
  commands: [
    {
      id: 'check',
      title: 'Check datapack content',
      icon: 'shield',
      when: ['datapacks', 'modpacks'],
      async run(context) {
        context.output('> Datapack Checker: content lint')
        try {
          const report = await context.invoke<ValidationReport>('diagnostics_content_lint')
          context.publishProblems('Datapack Checker', report.issues)
          const count = report.issues.length
          context.output(
            `Datapack Checker: ${count} issue(s) across ${report.checked} file(s).`,
            count ? 'error' : 'success',
          )
          context.notify(
            'Datapack Checker',
            count ? `${count} issue(s) found.` : 'No issues found.',
            count ? 'danger' : 'success',
          )
        } catch (error) {
          context.output(`Datapack Checker failed: ${String(error)}`, 'error')
          context.notify('Datapack Checker failed', String(error), 'danger')
        }
      },
    },
  ],
  views: [
    {
      id: 'registry',
      title: 'Datapack registry',
      icon: 'package',
      when: ['datapacks', 'modpacks'],
      async rows(context) {
        const registries = await context.invoke<ContentRegistry[]>('diagnostics_registries')
        // Registry entries cross the bridge untyped, so read them defensively.
        return registries
          .filter((registry) => registry.kind === 'datapack')
          .flatMap((registry) => registry.entries)
          .map((entry) => ({
            label: String(entry.path ?? '(unnamed)'),
            detail: entry.kind ? String(entry.kind) : undefined,
            icon: 'folder',
          }))
      },
    },
  ],
})
