import { defineSharedConfig } from '../../../vitepress-shared.mts'

// Shared UI/UX (sidebar, search, social links, markdown/lightbox) comes from
// docs/vitepress-shared.mts; only this site's identity lives here.
export default defineSharedConfig({
	title: 'Packwand Docs',
	siteTitle: 'Packwand',
	hostname: 'https://packwand.nostalgica.net/',
	description:
		'Documentation for packwand, the Minecraft modpack toolchain — packwiz core with multi-pack workspace management',
	nav: [
		{ text: 'Home', link: '/' },
		{ text: 'Installation', link: '/installation' },
		{ text: 'Tutorials', link: '/tutorials/creating/getting-started' },
		{ text: 'Reference', link: '/reference/additional-options' },
		{ text: 'Pack Format', link: '/reference/pack-format/pack-toml' },
		{
			text: 'Other Docs',
			items: [
				{ text: 'Lasting Legacy Docs', link: 'https://docs.nostalgica.net/' },
				{ text: 'Packwiz Components', link: 'https://packwiz.nostalgica.net/' },
			],
		},
	],
})
