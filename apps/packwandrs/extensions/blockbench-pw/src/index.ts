import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
  commands: [
    {
      id: 'scan-models',
      title: 'Scan Blockbench models',
      icon: 'search',
      async run(context) {
        const pack = context.activePack()
        if (!pack) return context.notify('BlockbenchPW', 'Select a pack target first.', 'danger')
        const assets = await context.blockbench.assets(pack.id)
        context.output(`BlockbenchPW: ${assets.length} model/project asset(s) under ${pack.id}.`, 'success')
      },
    },
  ],
  views: [
    {
      id: 'models',
      title: 'Open in Blockbench',
      icon: 'package',
      async rows(context) {
        const pack = context.activePack()
        if (!pack) return []
        return (await context.blockbench.assets(pack.id)).map((asset) => ({
          label: asset.name,
          detail: asset.path,
          icon: 'package',
          run: async () => {
            await context.blockbench.open(pack.id, asset.path)
            context.output(`BlockbenchPW: opened ${asset.path}`, 'success')
          },
        }))
      },
    },
  ],
})
