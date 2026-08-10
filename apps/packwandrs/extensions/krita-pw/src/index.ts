import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
	commands: [
		{
			id: 'scan-assets',
			title: 'Scan Krita-compatible assets',
			icon: 'search',
			async run(context) {
				const pack = context.activePack()
				if (!pack) return context.notify('KritaPW', 'Select a pack target first.', 'danger')
				const assets = await context.krita.assets(pack.id)
				context.output(
					`KritaPW: ${assets.length} image/source asset(s) under ${pack.id}.`,
					'success',
				)
			},
		},
	],
	views: [
		{
			id: 'assets',
			title: 'Open in Krita',
			icon: 'editor',
			async rows(context) {
				const pack = context.activePack()
				if (!pack) return []
				return (await context.krita.assets(pack.id)).map((asset) => ({
					label: asset.name,
					detail: asset.path,
					icon: 'editor',
					run: async () => {
						await context.krita.open(pack.id, asset.path)
						context.output(`KritaPW: opened ${asset.path}`, 'success')
					},
				}))
			},
		},
	],
})
