import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
	commands: [
		{
			id: 'check',
			title: 'Validate game content',
			icon: 'shield',
			when: ['datapacks', 'resourcepacks', 'modpacks'],
			async run(context) {
				const pack = context.activePack()
				if (!pack) return context.notify('GamePW Studio', 'Select a pack target first.', 'danger')
				context.output('> GamePW Studio: content validation')
				const report = await context.diagnostics.contentLint(pack.id)
				context.publishProblems('GamePW Studio', report.issues)
				const count = report.issues.length
				context.output(
					`GamePW Studio: ${count} issue(s) across ${report.checked} file(s).`,
					count ? 'error' : 'success',
				)
				context.notify(
					'GamePW Studio',
					count ? `${count} issue(s) found.` : 'Game content is valid.',
					count ? 'danger' : 'success',
				)
			},
		},
		{
			id: 'recipes',
			title: 'Inspect recipes',
			icon: 'package',
			when: ['datapacks', 'modpacks'],
			async run(context) {
				const pack = context.activePack()
				if (!pack) return context.notify('GamePW Studio', 'Select a pack target first.', 'danger')
				const recipes = await context.game.recipes(pack.id)
				const namespaces = new Set(recipes.map((recipe) => recipe.namespace)).size
				context.output(
					`GamePW Studio: indexed ${recipes.length} recipe(s) in ${namespaces} namespace(s).`,
					'success',
				)
				context.notify('Recipe Studio', `${recipes.length} recipe(s) indexed.`, 'success')
			},
		},
		{
			id: 'generate',
			title: 'New datapack file from a schema',
			icon: 'package',
			when: ['datapacks', 'modpacks'],
			run(context) {
				// The form is drawn by the shell from mcdoc schemas. GamePW Studio
				// only decides which one to author, so a new registry becomes a new
				// schema rather than a new screen here.
				context.generator.open()
			},
		},
		{
			id: 'parity',
			title: 'Check variant parity',
			icon: 'sync',
			async run(context) {
				const reports = await context.diagnostics.parity()
				const drifted = reports.filter(
					(report) =>
						(report.only_mr as unknown[] | undefined)?.length ||
						(report.only_cf as unknown[] | undefined)?.length ||
						(report.file_drift as unknown[] | undefined)?.length,
				)
				context.output(
					`GamePW Studio: ${drifted.length} of ${reports.length} variant(s) drifted.`,
					drifted.length ? 'error' : 'success',
				)
				context.notify(
					'Variant parity',
					drifted.length ? `${drifted.length} variant(s) drifted.` : 'All variants agree.',
					drifted.length ? 'danger' : 'success',
				)
			},
		},
	],
	views: [
		{
			id: 'content',
			title: 'Game content',
			icon: 'package',
			when: ['datapacks', 'resourcepacks', 'modpacks'],
			async rows(context) {
				const pack = context.activePack()
				if (!pack) return []
				const registries = await context.diagnostics.registries(pack.id)
				return registries
					.filter((registry) => registry.kind === 'datapack' || registry.kind === 'resourcepack')
					.flatMap((registry) =>
						registry.entries.map((entry) => {
							const origin = String(entry.origin ?? '')
							const path = String(entry.path ?? '')
							const editorPath = origin && origin !== '.' ? `${origin}/${path}` : path
							return {
								label: String(entry.id ?? path ?? '(unnamed)'),
								detail: `${String(entry.kind ?? registry.kind)} � ${path}`,
								icon: entry.kind === 'texture' ? 'editor' : 'package',
								run: path ? () => context.editor.open(pack.id, editorPath) : undefined,
							}
						}),
					)
			},
		},
		{
			id: 'recipes',
			title: 'Recipe Studio',
			icon: 'package',
			when: ['datapacks', 'modpacks'],
			async rows(context) {
				const pack = context.activePack()
				if (!pack) return []
				return (await context.game.recipes(pack.id)).map((recipe) => ({
					label: recipe.id,
					detail: recipe.path,
					icon: 'package',
					run: () => context.editor.open(pack.id, recipe.path),
				}))
			},
		},
	],
})
