import { definePackwandExtension } from '@/extensions/api'
import type { PackeaterMarker } from '@/helpers/invoke/packeater'
import type { JobRecord } from '@/helpers/types'

/**
 * Surfaces Packeater — the PackSquash fork in `crates/packeater_cli` — inside the
 * app. Compression itself runs as a Rust job, so a large resource pack does not
 * block the UI and its progress shows up in the dock's Jobs tab.
 */
export default definePackwandExtension({
  commands: [
    {
      id: 'compress',
      title: 'Compress pack with Packeater',
      icon: 'package',
      async run(context) {
        const pack = context.activePack()
        if (!pack) {
          context.notify('Packeater', 'Select a pack target first.', 'danger')
          return
        }
        context.output(`> Packeater: compress ${pack.id}`)
        try {
          const job = await context.invoke<JobRecord>('packeater_run', { id: pack.id, output: null })
          context.output(`Packeater job started: ${job.label}`, 'success')
          context.notify('Packeater', 'Compression started; see the Jobs panel.', 'success')
        } catch (error) {
          context.output(`Packeater failed: ${String(error)}`, 'error')
          context.notify('Packeater failed', String(error), 'danger')
        }
      },
    },
    {
      id: 'list-markers',
      title: 'List Packeater markers',
      icon: 'search',
      async run(context) {
        const pack = context.activePack()
        if (!pack) {
          context.notify('Packeater', 'Select a pack target first.', 'danger')
          return
        }
        try {
          const markers = await context.invoke<PackeaterMarker[]>('packeater_markers', {
            id: pack.id,
          })
          if (!markers.length) {
            context.output(`Packeater: no packeater.json markers under ${pack.id}.`)
            return
          }
          context.output(`Packeater: ${markers.length} marker(s) under ${pack.id}:`)
          for (const marker of markers) context.output(`  ${marker.path}`)
        } catch (error) {
          context.output(`Packeater marker scan failed: ${String(error)}`, 'error')
        }
      },
    },
  ],
  views: [
    {
      id: 'markers',
      title: 'Packeater markers',
      icon: 'package',
      async rows(context) {
        const pack = context.activePack()
        if (!pack) return []
        const markers = await context.invoke<PackeaterMarker[]>('packeater_markers', {
          id: pack.id,
        })
        // Sidebar rows are narrow, so lead with the folder name and keep the full
        // path as the row's detail (which also becomes its tooltip).
        return markers.map((marker) => ({
          label: marker.directory.split('/').at(-1) || marker.path,
          detail: marker.directory || marker.path,
          icon: 'package',
        }))
      },
    },
  ],
})
