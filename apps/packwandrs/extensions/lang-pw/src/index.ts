import { definePackwandExtension } from '@/extensions/api'

export default definePackwandExtension({
  commands: [{
    id: 'check',
    title: 'Check translations',
    icon: 'shield',
    async run(context) {
      const pack = context.activePack()
      if (!pack) return context.notify('LangPW', 'Select a pack target first.', 'danger')
      const snapshot = await context.language.snapshot(pack.id)
      const files = new Map(snapshot.files.map(file => [`${file.namespace}:${file.locale}`, file.path]))
      const issues = snapshot.gaps.map(gap => ({
        severity: 'warning' as const,
        path: files.get(`${gap.namespace}:${gap.locale}`) ?? '',
        message: `Missing translation key ${gap.key} (reference: ${gap.referenceLocale})`,
      }))
      context.publishProblems('LangPW', issues)
      context.output(`LangPW: ${snapshot.files.length} language file(s), ${snapshot.gaps.length} missing key(s).`, issues.length ? 'error' : 'success')
      context.notify('LangPW', issues.length ? `${issues.length} missing translation key(s).` : 'Translations are complete.', issues.length ? 'danger' : 'success')
    },
  }],
  views: [{
    id: 'languages',
    title: 'Languages',
    icon: 'editor',
    async rows(context) {
      const pack = context.activePack()
      if (!pack) return []
      const snapshot = await context.language.snapshot(pack.id)
      const missing = new Map<string, number>()
      for (const gap of snapshot.gaps) {
        const key = `${gap.namespace}:${gap.locale}`
        missing.set(key, (missing.get(key) ?? 0) + 1)
      }
      return snapshot.files.map(file => ({
        label: `${file.namespace} · ${file.locale}`,
        detail: `${file.keys} key(s) · ${missing.get(`${file.namespace}:${file.locale}`) ?? 0} missing`,
        icon: 'editor',
        run: () => context.editor.open(pack.id, file.path),
      }))
    },
  }],
})