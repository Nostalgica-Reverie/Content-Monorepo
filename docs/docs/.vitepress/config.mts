import { defineConfig } from 'vitepress'
import lightbox from "vitepress-plugin-lightbox"
import { generateSidebar } from "vitepress-sidebar";


// https://vitepress.dev/reference/site-config
export default defineConfig({
  cleanUrls: true,
  sitemap: {
    hostname: "https://docs.nostalgica.net/",
  },

  title: "Lasting Legacy Docs",
  head: [['link', { rel: 'icon', href: '/favicon.webp' }]],
  description: "Documentation for projects under Lasting Legacy, including Legacy4J and Re-Console",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    siteTitle: 'Documentation',
    logo: '/logo.webp',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Mods', link: '/mods' },
      { text: 'Modpacks', link: '/modpacks' },
      { text: 'Data Packs', link: '/datapacks' },
      { text: 'Resource Packs', link: '/resource-packs' }
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
