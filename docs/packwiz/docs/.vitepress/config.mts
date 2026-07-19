import { defineSharedConfig } from '../../../vitepress-shared.mts'

// Shared UI/UX (sidebar, search, social links, markdown/lightbox) comes from
// docs/vitepress-shared.mts; only this site's identity lives here.
export default defineSharedConfig({
  title: "Packwiz Components",
  hostname: "https://packwiz.nostalgica.net/",
  description: "Documentation for the packwiz-installer, bootstrap, and mod_browser_webview components maintained in the Lasting Legacy monorepo",
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
})
