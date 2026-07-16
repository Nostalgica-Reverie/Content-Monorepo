export const DOCS_BASE = "/modpack-dev-handbook";

export const siteConfig = {
  handbook: {
    title: "Modpack Dev Handbook",
    siteUrl: "https://docs.nostalgica.net",
    author: "Reverie Projects",
    keywords: "minecraft, modpack development, packwand, packwiz, handbook",
    discordUrl: "https://discord.gg/6pRkrYxbGW",
    repoUrl: "https://git.nostalgica.net/Reverie-Projects/monorepo",
    sourceBrowseRoot: "https://git.nostalgica.net/Reverie-Projects/monorepo/src/branch/main/",
    releasesUrl: "https://git.nostalgica.net/Reverie-Projects/monorepo/releases",
  },
  packwand: {
    repoUrl: "https://git.nostalgica.net/Reverie-Projects/monorepo",
    releasesUrl: "https://git.nostalgica.net/Reverie-Projects/monorepo/releases",
    sourceBrowseRoot: "https://git.nostalgica.net/Reverie-Projects/monorepo/src/branch/main/apps/packwand/",
    goModule: "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand",
  },
  packwiz: {
    repoUrl: "https://github.com/packwiz/packwiz",
    actionsUrl: "https://github.com/packwiz/packwiz/actions",
    nightlyUrl: "https://nightly.link/packwiz/packwiz/workflows/go/main",
    examplePackUrl: "https://github.com/packwiz/packwiz-example-pack",
    guiUrl: "https://github.com/ExoPlant/packwiz-gui",
    discordUrl: "https://discord.gg/Csh8zbbhCt",
  },
} as const;

export function handbookSourceUrl(sourcePath: string) {
  return `${siteConfig.handbook.sourceBrowseRoot}${sourcePath}`;
}
