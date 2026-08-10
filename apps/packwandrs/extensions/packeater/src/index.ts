import { definePackwandExtension } from '@/extensions/api'

/**
 * Surfaces Packeater — the PackSquash fork in `crates/packeater_cli` — inside the
 * app. Compression itself runs as a Rust job, so a large resource pack does not
 * block the UI and its progress shows up in the dock's Jobs tab.
 */
export default definePackwandExtension({
	commands: [
		{
			id: 'compress',
			title: 'Compress pack with PackEater',
			icon: 'package',
			async run(context) {
				const pack = context.activePack()
				if (!pack) {
					context.notify('PackEater', 'Select a pack target first.', 'danger')
					return
				}
				context.output(`> PackEater: compress ${pack.id}`)
				try {
					const job = await context.optimizer.run(pack.id)
					context.output(`PackEater job started: ${job.label}`, 'success')
					context.notify('PackEater', 'Compression started; see the Jobs panel.', 'success')
				} catch (error) {
					context.output(`Packeater failed: ${String(error)}`, 'error')
					context.notify('Packeater failed', String(error), 'danger')
				}
			},
		},
		{
			id: 'preview',
			title: 'Preview PackEater run',
			icon: 'search',
			async run(context) {
				const pack = context.activePack()
				if (!pack) return context.notify('PackEater', 'Select a pack target first.', 'danger')
				const previews = await context.optimizer.preview(pack.id)
				if (!previews.length) return context.output(`PackEater: no markers under ${pack.id}.`)
				for (const preview of previews) {
					context.output(
						`PackEater preview: ${preview.directory} · ${preview.fileCount} files · ${preview.inputBytes} bytes -> ${preview.output}${preview.enabled ? '' : ' (disabled)'}`,
					)
				}
			},
		},
		{
			id: 'initialize',
			title: 'Create PackEater configuration',
			icon: 'new-file',
			async run(context) {
				const pack = context.activePack()
				if (!pack) return context.notify('PackEater', 'Select a pack target first.', 'danger')
				const marker = await context.optimizer.initialize(pack.id)
				context.output(`PackEater: created ${marker.path}`, 'success')
				context.notify('PackEater', 'Created packeater.json with safe defaults.', 'success')
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
					const markers = await context.optimizer.markers(pack.id)
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
				const markers = await context.optimizer.preview(pack.id)
				// Sidebar rows are narrow, so lead with the folder name and keep the full
				// path as the row's detail (which also becomes its tooltip).
				return markers.map((marker) => ({
					label: marker.directory.split('/').at(-1) || marker.path,
					detail: `${marker.enabled ? 'enabled' : 'disabled'} · ${marker.fileCount} files · ${marker.inputBytes} bytes`,
					icon: 'package',
				}))
			},
		},
	],
})
