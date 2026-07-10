import eslint from '@eslint/js'
import prettierEslint from 'eslint-plugin-prettier/recommended'
import simpleImportSort from 'eslint-plugin-simple-import-sort'
import tseslint from 'typescript-eslint'

export default tseslint.config(
	eslint.configs.recommended,
	tseslint.configs.recommended,
	prettierEslint,
	{
		plugins: {
			'simple-import-sort': simpleImportSort,
		},
		rules: {
			'simple-import-sort/imports': 'error',
			'simple-import-sort/exports': 'error',
		},
	},
	{
		rules: {
			'@typescript-eslint/no-type-alias': 'off',
			'@typescript-eslint/no-explicit-any': 'off',
			'@typescript-eslint/ban-ts-comment': 'off',
			'@typescript-eslint/prefer-literal-enum-member': 'off',
			'@typescript-eslint/no-unused-expressions': 'off',
		},
	},
	{
		languageOptions: {
			parserOptions: {
				warnOnUnsupportedTypeScriptVersion: false,
			},
		},
	},
	{
		ignores: ['node_modules/', 'dist/', 'build/'],
	},
)
