import { defineConfig } from 'vitepress'
import lightbox from "vitepress-plugin-lightbox"
import { generateSidebar } from "vitepress-sidebar";


// Mirrors docs/packwand/docs/.vitepress/config.mts so all monorepo doc sites
// share the same UI/UX, while remaining separately built and served.
export default defineConfig({
  cleanUrls: true,
  sitemap: {
    hostname: "https://packwiz.nostalgica.net/",
  },

  title: "Packwiz Components",
  head: [['link', { rel: 'icon', href: '/favicon.webp' }]],
  description: "Documentation for the packwiz-installer, bootstrap, and mod_browser_webview components maintained in the Lasting Legacy monorepo",
  themeConfig: {
    siteTitle: 'Packwiz Components',
    logo: '/logo.webp',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Installer', link: '/installer' },
      { text: 'Bootstrap', link: '/bootstrap' },
      { text: 'Webview', link: '/webview' },
      { text: 'Building', link: '/building' },
      {
        text: 'Other Docs',
        items: [
          { text: 'Lasting Legacy Docs', link: 'https://docs.nostalgica.net/' },
          { text: 'Packwand', link: 'https://packwand.nostalgica.net/' },
        ],
      },
    ],
    search: {
      provider: "local",
    },

    sidebar: generateSidebar({
      sortFolderTo: "bottom",
      documentRootPath: "/docs",
      useTitleFromFileHeading: true,
      useTitleFromFrontmatter: true,
      useFolderTitleFromIndexFile: true,
      collapsed: true,
      collapseDepth: 2,
      capitalizeFirst: true,
      capitalizeEachWords: false,
      rootGroupText: "Main",
      includeEmptyFolder: false,
    }),

    socialLinks: [
      { icon: 'forgejo', link: 'https://git.nostalgica.net/Lasting-Legacy' },
      { icon: 'discord', link: 'https://discord.gg/6pRkrYxbGW'}
      ],
    },
    markdown: {
      image: {
        lazyLoading: true,
    },
      config: (md) => {
        md.use(lightbox, {});
    },
  }
})
