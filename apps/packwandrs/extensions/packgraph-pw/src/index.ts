import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
  commands: [{
    id: 'analyze',
    title: 'Analyze package graph',
    icon: 'git-branch',
    async run(context) {
      const pack = context.activePack()
      if (!pack) return context.notify('PackGraphPW', 'Select a pack target first.', 'danger')
      const graph = await context.graph.snapshot(pack.id)
      const providers = new Set(graph.nodes.map(node => node.provider)).size
      context.output(`PackGraphPW: ${graph.nodes.length} package(s), ${graph.edges.length} dependency edge(s), ${providers} provider(s).`, 'success')
      context.notify('PackGraphPW', `${graph.nodes.length} package(s) indexed.`, 'success')
    },
  }],
  views: [{
    id: 'packages',
    title: 'Package graph',
    icon: 'git-branch',
    async rows(context) {
      const pack = context.activePack()
      if (!pack) return []
      const graph = await context.graph.snapshot(pack.id)
      const incoming = new Map<string, number>()
      for (const edge of graph.edges) incoming.set(edge.to, (incoming.get(edge.to) ?? 0) + 1)
      return graph.nodes.map(node => ({
        label: node.name,
        detail: `${node.kind} · ${node.provider} · ${node.side} · ${incoming.get(node.id) ?? 0} dependent(s)`,
        icon: 'package',
        run: () => context.editor.open(pack.id, node.path),
      }))
    },
  }],
})