import * as monaco from 'monaco-editor'
import 'monaco-editor/language/css/monaco.contribution.js'
import 'monaco-editor/language/html/monaco.contribution.js'
import 'monaco-editor/language/json/monaco.contribution.js'
import 'monaco-editor/language/typescript/monaco.contribution.js'
import CssWorker from 'monaco-editor/language/css/css.worker.js?worker'
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker'
import HtmlWorker from 'monaco-editor/language/html/html.worker.js?worker'
import JsonWorker from 'monaco-editor/language/json/json.worker.js?worker'
import TypeScriptWorker from 'monaco-editor/language/typescript/ts.worker.js?worker'

import type { ResolvedTheme } from '@/themes/types'

;(
	self as typeof self & {
		MonacoEnvironment: { getWorker(moduleId: string, label: string): Worker }
	}
).MonacoEnvironment = {
	getWorker(_moduleId: string, label: string) {
		if (label === 'json') return new JsonWorker()
		if (label === 'css' || label === 'scss' || label === 'less') return new CssWorker()
		if (label === 'html' || label === 'handlebars' || label === 'razor') return new HtmlWorker()
		if (label === 'typescript' || label === 'javascript') return new TypeScriptWorker()
		return new EditorWorker()
	},
}

let languagesRegistered = false

export function registerPackwandLanguages() {
	if (languagesRegistered) return
	languagesRegistered = true
	monaco.languages.register({ id: 'toml', extensions: ['.toml'] })
	monaco.languages.setMonarchTokensProvider('toml', {
		tokenizer: {
			root: [
				[/#.*$/, 'comment'],
				[/\[\[?.*?\]\]?/, 'type'],
				[/^[\w.-]+(?=\s*=)/, 'key'],
				[/"(?:[^"\\]|\\.)*"/, 'string'],
				[/'[^']*'/, 'string'],
				[/\b(?:true|false)\b/, 'keyword'],
				[/[+-]?(?:\d+\.?\d*|\.\d+)/, 'number'],
			],
		},
	})
	monaco.languages.register({ id: 'mcfunction', extensions: ['.mcfunction'] })
	monaco.languages.setMonarchTokensProvider('mcfunction', {
		tokenizer: {
			root: [
				[/#.*$/, 'comment'],
				[/^\s*\/?[a-z_][\w:]*/, 'keyword'],
				[/@[pares](?:\[[^\]]*\])?/, 'type'],
				[/\b(?:true|false)\b/, 'keyword'],
				[/[~-]?\d+(?:\.\d+)?/, 'number'],
				[/"(?:[^"\\]|\\.)*"/, 'string'],
				[/[a-z0-9_.-]+:[a-z0-9_./-]+/i, 'string'],
			],
		},
	})
}

export function languageForPath(path: string) {
	const extension = path.split('.').pop()?.toLowerCase()
	return (
		(
			{
				json: 'json',
				json5: 'json',
				toml: 'toml',
				mcfunction: 'mcfunction',
				js: 'javascript',
				mjs: 'javascript',
				cjs: 'javascript',
				ts: 'typescript',
				mts: 'typescript',
				css: 'css',
				scss: 'scss',
				html: 'html',
				md: 'markdown',
				xml: 'xml',
				yml: 'yaml',
				yaml: 'yaml',
				properties: 'ini',
				txt: 'plaintext',
				log: 'plaintext',
			} as Record<string, string>
		)[extension ?? ''] ?? 'plaintext'
	)
}

export function installMonacoTheme(theme: ResolvedTheme) {
	const name = `packwand-${theme.id.replaceAll('.', '-')}`
	const highContrastBase = theme.colors.bg.toLowerCase() === '#ffffff' ? 'hc-light' : 'hc-black'
	monaco.editor.defineTheme(name, {
		base:
			theme.appearance === 'light'
				? 'vs'
				: theme.appearance === 'high-contrast'
					? highContrastBase
					: 'vs-dark',
		inherit: true,
		colors: theme.editor.colors,
		rules: theme.editor.rules.map((rule) => ({
			token: rule.token,
			foreground: rule.foreground?.slice(1),
			background: rule.background?.slice(1),
			fontStyle: rule.fontStyle,
		})),
	})
	monaco.editor.setTheme(name)
	return name
}

export { monaco }
