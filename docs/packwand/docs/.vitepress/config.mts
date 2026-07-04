import { defineConfig } from 'vitepress'
import lightbox from "vitepress-plugin-lightbox"
import { generateSidebar } from "vitepress-sidebar";


// Mirrors the main Lasting Legacy docs site config (docs/docs/.vitepress/config.mts)
// so both sites share the same UI/UX, while remaining separately built and served.
export default defineConfig({
  cleanUrls: true,
  sitemap: {
    hostname: "https://packwand.nostalgica.net/",
  },

  title: "Packwand Docs",
  head: [['link', { rel: 'icon', href: '/favicon.webp' }]],
  description: "Documentation for packwand, the Minecraft modpack toolchain — packwiz core with multi-pack workspace management",
  themeConfig: {
    siteTitle: 'Packwand',
    logo: '/logo.webp',
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
