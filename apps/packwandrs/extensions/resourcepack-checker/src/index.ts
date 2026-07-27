import { definePackwandExtension } from '@/extensions/api'
import type { ValidationReport } from '@/helpers/types'

/**
 * Resource pack validation, again delegating to `packwand-diagnostics`.
 *
 * `content_lint` covers the two failure modes that actually bite: a missing or
 * malformed `pack.mcmeta`, and model/texture references that resolve on Windows
 * but break on a case-sensitive server filesystem. Its hygiene pass (duplicate
 * files, resource-location charset) is included here — unlike in preflight,
 * where packaging problems should not block a release.
 */
export default definePackwandExtension({
  commands: [
    {
      id: 'check',
      title: 'Check resourcepack content',
      icon: 'shield',
      when: ['resourcepacks', 'modpacks'],
      async run(context) {
        context.output('> Resourcepack Checker: content lint')
        try {
          const report = await context.invoke<ValidationReport>('diagnostics_content_lint')
          context.publishProblems('Resourcepack Checker', report.issues)
          const count = report.issues.length
          context.output(
            `Resourcepack Checker: ${count} issue(s) across ${report.checked} file(s).`,
            count ? 'error' : 'success',
          )
          context.notify(
            'Resourcepack Checker',
            count ? `${count} issue(s) found.` : 'No issues found.',
            count ? 'danger' : 'success',
          )
        } catch (error) {
          context.output(`Resourcepack Checker failed: ${String(error)}`, 'error')
          context.notify('Resourcepack Checker failed', String(error), 'danger')
        }
      },
    },
    {
      id: 'parity',
      title: 'Check variant parity across platforms',
      icon: 'sync',
      async run(context) {
        context.output('> Resourcepack Checker: variant parity')
        try {
          const reports = await context.invoke<Array<Record<string, unknown>>>('diagnostics_parity')
          const drifted = reports.filter(
            (report) =>
              (report.only_mr as unknown[] | undefined)?.length ||
              (report.only_cf as unknown[] | undefined)?.length ||
              (report.file_drift as unknown[] | undefined)?.length,
          )
          context.output(
            `Variant parity: ${drifted.length} of ${reports.length} variant(s) drifted.`,
            drifted.length ? 'error' : 'success',
          )
          for (const report of drifted) {
            context.output(`  ${String(report.pack)} / ${String(report.variant)} differs`, 'error')
          }
          context.notify(
            'Variant parity',
            drifted.length ? `${drifted.length} variant(s) drifted.` : 'All variants agree.',
            drifted.length ? 'danger' : 'success',
          )
        } catch (error) {
          context.output(`Variant parity failed: ${String(error)}`, 'error')
          context.notify('Variant parity failed', String(error), 'danger')
        }
      },
    },
  ],
})
