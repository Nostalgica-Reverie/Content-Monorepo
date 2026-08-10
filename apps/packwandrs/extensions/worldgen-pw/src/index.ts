import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
	commands: [
		{
			id: 'check',
			title: 'Validate world generation',
			icon: 'shield',
			when: ['datapacks', 'modpacks'],
			async run(context) {
				const pack = context.activePack()
				if (!pack) return context.notify('WorldgenPW', 'Select a pack target first.', 'danger')
				const [assets, report] = await Promise.all([
					context.worldgen.assets(pack.id),
					context.diagnostics.contentLint(pack.id),
				])
				const issues = report.issues.filter((issue) =>
					issue.path.replaceAll('\\', '/').includes('/worldgen/'),
				)
				context.publishProblems('WorldgenPW', issues)
				const kinds = new Set(assets.map((asset) => asset.kind)).size
				context.output(
					`WorldgenPW: ${assets.length} definition(s) across ${kinds} registry type(s); ${issues.length} issue(s).`,
					issues.length ? 'error' : 'success',
				)
				context.notify(
					'WorldgenPW',
					issues.length
						? `${issues.length} worldgen issue(s).`
						: `${assets.length} definition(s) indexed.`,
					issues.length ? 'danger' : 'success',
				)
			},
		},
	],
	views: [
		{
			id: 'worldgen',
			title: 'World generation',
			icon: 'target',
			when: ['datapacks', 'modpacks'],
			async rows(context) {
				const pack = context.activePack()
				if (!pack) return []
				return (await context.worldgen.assets(pack.id)).map((asset) => ({
					label: asset.id,
					detail: `${asset.kind} � ${asset.path}`,
					icon: 'target',
					run: () => context.editor.open(pack.id, asset.path),
				}))
			},
		},
	],
})
