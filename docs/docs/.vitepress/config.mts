import { defineSharedConfig } from '../../vitepress-shared.mts'

// Shared UI/UX (sidebar, search, social links, markdown/lightbox) comes from
// docs/vitepress-shared.mts; only this site's identity lives here.
export default defineSharedConfig({
	title: 'Lasting Legacy Docs',
	siteTitle: 'Documentation',
	hostname: 'https://docs.nostalgica.net/',
	description: 'Documentation for projects under Lasting Legacy, including Legacy4J and Re-Console',
	nav: [
		{ text: 'Home', link: '/' },
		{ text: 'Mods', link: '/mods' },
		{ text: 'Modpacks', link: '/modpacks' },
		{ text: 'Data Packs', link: '/datapacks' },
		{ text: 'Resource Packs', link: '/resource-packs' },
		{
			text: 'Other Docs',
			items: [
				{ text: 'Packwand', link: 'https://packwand.nostalgica.net/' },
				{ text: 'Packwiz Components', link: 'https://packwiz.nostalgica.net/' },
			],
		},
	],
})
