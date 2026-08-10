import { defineConfig, type DefaultTheme, type UserConfig } from 'vitepress'
import lightbox from 'vitepress-plugin-lightbox'
import { generateSidebar } from 'vitepress-sidebar'

// Shared VitePress configuration for the three monorepo doc sites
// (docs/docs, docs/packwand, docs/packwiz). Everything that gives the sites
// one UI/UX — sidebar generation, search, social links, markdown/lightbox,
// favicon/logo — lives here exactly once; each site's config.mts supplies
// only its own identity (title, hostname, description, nav).

export interface SharedSiteOptions {
	/** Browser-tab / SEO title, e.g. "Packwand Docs". */
	title: string
	/** Sitemap hostname, e.g. "https://packwand.nostalgica.net/". */
	hostname: string
	/** Meta description. */
	description: string
	/** Navbar site title (defaults to `title` when omitted). */
	siteTitle?: string
	/** Site-specific top navigation. */
	nav: DefaultTheme.NavItem[]
}

export function defineSharedConfig(site: SharedSiteOptions): UserConfig<DefaultTheme.Config> {
	return defineConfig({
		cleanUrls: true,
		sitemap: {
			hostname: site.hostname,
		},

		title: site.title,
		head: [['link', { rel: 'icon', href: '/favicon.webp' }]],
		description: site.description,
		themeConfig: {
			siteTitle: site.siteTitle ?? site.title,
			logo: '/logo.webp',
			nav: site.nav,
			search: {
				provider: 'local',
			},

			sidebar: generateSidebar({
				sortFolderTo: 'bottom',
				documentRootPath: '/docs',
				useTitleFromFileHeading: true,
				useTitleFromFrontmatter: true,
				useFolderTitleFromIndexFile: true,
				collapsed: true,
				collapseDepth: 2,
				capitalizeFirst: true,
				capitalizeEachWords: false,
				rootGroupText: 'Main',
				includeEmptyFolder: false,
			}),

			socialLinks: [
				{ icon: 'forgejo', link: 'https://git.nostalgica.net/Lasting-Legacy' },
				{ icon: 'discord', link: 'https://discord.gg/6pRkrYxbGW' },
			],
		},
		markdown: {
			image: {
				lazyLoading: true,
			},
			config: (md) => {
				md.use(lightbox, {})
			},
		},
	})
}
