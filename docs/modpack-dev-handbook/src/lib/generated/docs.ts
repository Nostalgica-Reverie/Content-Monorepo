export type DocMeta = {
  title: string;
  url: string;
  sourcePath: string;
  description: string;
  tags: string[];
};

export type NavNode = {
  title: string;
  url: string | null;
  children: NavNode[];
};

export type NavSection = {
  title: string;
  children: NavNode[];
};

export const docsIndex: DocMeta[] = [
  {
    "title": "Home",
    "url": "/",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/+page.svx",
    "description": "Unified handbook documentation for modpack development, pack publishing, and pack management tooling.",
    "tags": []
  },
  {
    "title": "Page Formatting",
    "url": "/contribute/formatting",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/contribute/formatting/+page.svx",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "tags": []
  },
  {
    "title": "Git Practices",
    "url": "/contribute/git-practices",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/contribute/git-practices/+page.svx",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "tags": []
  },
  {
    "title": "Credits",
    "url": "/credits",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/credits/+page.svx",
    "description": "The Modpack Dev Handbook Credits.",
    "tags": []
  },
  {
    "title": "Adding new blocks",
    "url": "/guide/custom-content/adding-blocks",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/custom-content/adding-blocks/+page.svx",
    "description": "Adding new blocks",
    "tags": []
  },
  {
    "title": "Adding new items",
    "url": "/guide/custom-content/adding-items",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/custom-content/adding-items/+page.svx",
    "description": "Adding new items",
    "tags": []
  },
  {
    "title": "Attribute Modification",
    "url": "/guide/custom-content/attribute-modification",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/custom-content/attribute-modification/+page.svx",
    "description": "Modifying attributes of items and entities",
    "tags": []
  },
  {
    "title": "Intro to Datapacks",
    "url": "/guide/intro/intro-datapack",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/intro/intro-datapack/+page.svx",
    "description": "Introduction and tutorial for datapacks",
    "tags": []
  },
  {
    "title": "Introduction to modpack development",
    "url": "/guide/intro/intro-intro",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/intro/intro-intro/+page.svx",
    "description": "The very basics for modpack development",
    "tags": []
  },
  {
    "title": "Intro to Mopdpacks",
    "url": "/guide/intro/intro-modpack",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/intro/intro-modpack/+page.svx",
    "description": "Introduction and tutorial for modpacks",
    "tags": []
  },
  {
    "title": "Intro to Resource Packs",
    "url": "/guide/intro/intro-resourcepack",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/intro/intro-resourcepack/+page.svx",
    "description": "Introduction and tutorial for resource packs",
    "tags": []
  },
  {
    "title": "Minecraft Concepts",
    "url": "/guide/intro/minecraft-concepts",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/intro/minecraft-concepts/+page.svx",
    "description": "How is Minecraft relevant to Minecraft Modpacks?",
    "tags": []
  },
  {
    "title": "Improving and Profiling Modpack Performance",
    "url": "/guide/performance",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/performance/+page.svx",
    "description": "How to improve performance in your modpack",
    "tags": []
  },
  {
    "title": "Removing Blocks",
    "url": "/guide/removals/removing-blocks",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/removals/removing-blocks/+page.svx",
    "description": "Strategies for removing naturally generated or unwanted blocks from a pack.",
    "tags": []
  },
  {
    "title": "Removing items",
    "url": "/guide/removals/removing-items",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/removals/removing-items/+page.svx",
    "description": "How to remove items from being obtained or used in a modpack",
    "tags": []
  },
  {
    "title": "Modifying mob spawns",
    "url": "/guide/worldgen/mob-spawns",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/worldgen/mob-spawns/+page.svx",
    "description": "Guide on modifying mob spawns with various methods",
    "tags": []
  },
  {
    "title": "Adding biomes to your modpack",
    "url": "/guide/worldgen/modifying-biomes/adding-biomes",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/worldgen/modifying-biomes/adding-biomes/+page.svx",
    "description": "Adding custom biomes to the game",
    "tags": []
  },
  {
    "title": "Removing biomes from your modpack",
    "url": "/guide/worldgen/modifying-biomes/removing-biomes",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/worldgen/modifying-biomes/removing-biomes/+page.svx",
    "description": "Removing biomes from the game",
    "tags": []
  },
  {
    "title": "Adding worldgen features to your modpack",
    "url": "/guide/worldgen/modifying-features/adding-features",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/worldgen/modifying-features/adding-features/+page.svx",
    "description": "Adding worldgen features to your game with datapacks and Lithostitched",
    "tags": []
  },
  {
    "title": "Removing features from your modpack",
    "url": "/guide/worldgen/modifying-features/removing-features",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/guide/worldgen/modifying-features/removing-features/+page.svx",
    "description": "Removing worldgen features to your game with datapacks and Lithostitched",
    "tags": []
  },
  {
    "title": "Evergreen Version Resources",
    "url": "/wiki/evergreen",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/evergreen/+page.svx",
    "description": "Resources and communities for packdev on older versions",
    "tags": []
  },
  {
    "title": "Data loading conditions",
    "url": "/wiki/info/data-loading-conditions",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/data-loading-conditions/+page.svx",
    "description": "Overview on modloader's data loading conditions and how to use them",
    "tags": []
  },
  {
    "title": "Free Multiplayer",
    "url": "/wiki/info/free-multiplayer",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/free-multiplayer/+page.svx",
    "description": "Exploring options for free multiplayer in modded Minecraft",
    "tags": []
  },
  {
    "title": "List of modpack launchers",
    "url": "/wiki/info/launchers",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/launchers/+page.svx",
    "description": "List of modpack launchers",
    "tags": []
  },
  {
    "title": "Licenses",
    "url": "/wiki/info/licenses",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/licenses/+page.svx",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "tags": []
  },
  {
    "title": "Regular Expressions",
    "url": "/wiki/info/regex",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/regex/+page.svx",
    "description": "Intro and examples of Regular Expressions (regex)",
    "tags": []
  },
  {
    "title": "Useful Tags and Terms",
    "url": "/wiki/info/useful-terms",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/useful-terms/+page.svx",
    "description": "Useful tags and terms",
    "tags": []
  },
  {
    "title": "Version Control Tools",
    "url": "/wiki/info/version-control-tools",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/info/version-control-tools/+page.svx",
    "description": "Git and pack-specific tooling choices for versioning a modpack project.",
    "tags": []
  },
  {
    "title": "Pack Management",
    "url": "/wiki/modpack-management",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/+page.svx",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "tags": []
  },
  {
    "title": "CurseForge",
    "url": "/wiki/modpack-management/curseforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/curseforge/+page.svx",
    "description": "Things to be aware of when submitting your modpack to CurseForge",
    "tags": []
  },
  {
    "title": "Marketing",
    "url": "/wiki/modpack-management/marketing",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/marketing/+page.svx",
    "description": "Notes on how to build an audience for your pack",
    "tags": []
  },
  {
    "title": "Modrinth",
    "url": "/wiki/modpack-management/modrinth",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/modrinth/+page.svx",
    "description": "Things to be aware of when submitting your modpack to Modrinth",
    "tags": []
  },
  {
    "title": "packwand",
    "url": "/wiki/modpack-management/packwand",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Building the native GUI app",
    "url": "/wiki/modpack-management/packwand/development/gui-build",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/development/gui-build/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Installation",
    "url": "/wiki/modpack-management/packwand/installation",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/installation/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Additional options",
    "url": "/wiki/modpack-management/packwand/reference/additional-options",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/reference/additional-options/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "index.toml",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/reference/pack-format/index-toml/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "manifest.json",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/reference/pack-format/manifest-json/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "mod.pw.toml",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/reference/pack-format/mod-toml/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "pack.toml",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/reference/pack-format/pack-toml/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": ".packwizignore",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/packwizignore",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/reference/pack-format/packwizignore/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Adding mods and resource packs",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/tutorials/creating/adding-mods/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Getting started",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/getting-started",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/tutorials/creating/getting-started/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Using packwand with Git",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/git",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/tutorials/creating/git/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Publishing to CurseForge",
    "url": "/wiki/modpack-management/packwand/tutorials/hosting/curseforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/tutorials/hosting/curseforge/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Publishing to Modrinth",
    "url": "/wiki/modpack-management/packwand/tutorials/hosting/modrinth",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/tutorials/hosting/modrinth/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Pack Installation using packwiz-installer",
    "url": "/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "packwiz",
    "url": "/wiki/modpack-management/packwiz",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Packwiz Components",
    "url": "/wiki/modpack-management/packwiz/components",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/components/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Bootstrap",
    "url": "/wiki/modpack-management/packwiz/components/bootstrap",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/components/bootstrap/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Building",
    "url": "/wiki/modpack-management/packwiz/components/building",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/components/building/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "packwiz-installer",
    "url": "/wiki/modpack-management/packwiz/components/installer",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/components/installer/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "modbrowserwebview",
    "url": "/wiki/modpack-management/packwiz/components/webview",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/components/webview/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Installation",
    "url": "/wiki/modpack-management/packwiz/installation",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/installation/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Additional options",
    "url": "/wiki/modpack-management/packwiz/reference/additional-options",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/reference/additional-options/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": ".packwizignore",
    "url": "/wiki/modpack-management/packwiz/reference/pack-format/packwizignore",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/reference/pack-format/packwizignore/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Adding mods and resource packs",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/adding-mods",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/tutorials/creating/adding-mods/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Getting started",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/getting-started",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/tutorials/creating/getting-started/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Using packwiz with Git",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/git",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/tutorials/creating/git/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Publishing to CurseForge",
    "url": "/wiki/modpack-management/packwiz/tutorials/hosting/curseforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/tutorials/hosting/curseforge/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Publishing to Modrinth",
    "url": "/wiki/modpack-management/packwiz/tutorials/hosting/modrinth",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/tutorials/hosting/modrinth/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Pack Installation using packwiz-installer",
    "url": "/wiki/modpack-management/packwiz/tutorials/installing/packwiz-installer",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/packwiz/tutorials/installing/packwiz-installer/+page.md",
    "description": "",
    "tags": []
  },
  {
    "title": "Project Management",
    "url": "/wiki/modpack-management/project-management",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/modpack-management/project-management/+page.svx",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "tags": []
  },
  {
    "title": "Ideation",
    "url": "/wiki/planning/ideation",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/planning/ideation/+page.svx",
    "description": "Creating a core concept for your pack",
    "tags": []
  },
  {
    "title": "Mod Selection",
    "url": "/wiki/planning/mod-selection",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/planning/mod-selection/+page.svx",
    "description": "Choosing which mods to add to your modpack",
    "tags": []
  },
  {
    "title": "Scope",
    "url": "/wiki/planning/scope",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/planning/scope/+page.svx",
    "description": "Modpack scope and how to limit it",
    "tags": []
  },
  {
    "title": "Useful Mods",
    "url": "/wiki/useful-mods",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/+page.svx",
    "description": "Library of useful mods",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Fabric 1.19.2",
    "url": "/wiki/useful-mods/bug_fixes/1.19.2/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.19.2/fabric/+page.svx",
    "description": "Bug Fixes mods for Fabric 1.19.2",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Forge 1.19.2",
    "url": "/wiki/useful-mods/bug_fixes/1.19.2/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.19.2/forge/+page.svx",
    "description": "Bug Fixes mods for Forge 1.19.2",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for NeoForge 1.19.2",
    "url": "/wiki/useful-mods/bug_fixes/1.19.2/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.19.2/neoforge/+page.svx",
    "description": "Bug Fixes mods for NeoForge 1.19.2",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Fabric 1.20.1",
    "url": "/wiki/useful-mods/bug_fixes/1.20.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.20.1/fabric/+page.svx",
    "description": "Bug Fixes mods for Fabric 1.20.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Forge 1.20.1",
    "url": "/wiki/useful-mods/bug_fixes/1.20.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.20.1/forge/+page.svx",
    "description": "Bug Fixes mods for Forge 1.20.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for NeoForge 1.20.1",
    "url": "/wiki/useful-mods/bug_fixes/1.20.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.20.1/neoforge/+page.svx",
    "description": "Bug Fixes mods for NeoForge 1.20.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Fabric 1.21.1",
    "url": "/wiki/useful-mods/bug_fixes/1.21.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.21.1/fabric/+page.svx",
    "description": "Bug Fixes mods for Fabric 1.21.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Forge 1.21.1",
    "url": "/wiki/useful-mods/bug_fixes/1.21.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.21.1/forge/+page.svx",
    "description": "Bug Fixes mods for Forge 1.21.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for NeoForge 1.21.1",
    "url": "/wiki/useful-mods/bug_fixes/1.21.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/1.21.1/neoforge/+page.svx",
    "description": "Bug Fixes mods for NeoForge 1.21.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Fabric 26.1",
    "url": "/wiki/useful-mods/bug_fixes/26.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/26.1/fabric/+page.svx",
    "description": "Bug Fixes mods for Fabric 26.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for Forge 26.1",
    "url": "/wiki/useful-mods/bug_fixes/26.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/26.1/forge/+page.svx",
    "description": "Bug Fixes mods for Forge 26.1",
    "tags": []
  },
  {
    "title": "Bug Fixes mods for NeoForge 26.1",
    "url": "/wiki/useful-mods/bug_fixes/26.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/bug_fixes/26.1/neoforge/+page.svx",
    "description": "Bug Fixes mods for NeoForge 26.1",
    "tags": []
  },
  {
    "title": "Documentation mods for Fabric 1.19.2",
    "url": "/wiki/useful-mods/documentation/1.19.2/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.19.2/fabric/+page.svx",
    "description": "Documentation mods for Fabric 1.19.2",
    "tags": []
  },
  {
    "title": "Documentation mods for Forge 1.19.2",
    "url": "/wiki/useful-mods/documentation/1.19.2/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.19.2/forge/+page.svx",
    "description": "Documentation mods for Forge 1.19.2",
    "tags": []
  },
  {
    "title": "Documentation mods for NeoForge 1.19.2",
    "url": "/wiki/useful-mods/documentation/1.19.2/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.19.2/neoforge/+page.svx",
    "description": "Documentation mods for NeoForge 1.19.2",
    "tags": []
  },
  {
    "title": "Documentation mods for Fabric 1.20.1",
    "url": "/wiki/useful-mods/documentation/1.20.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.20.1/fabric/+page.svx",
    "description": "Documentation mods for Fabric 1.20.1",
    "tags": []
  },
  {
    "title": "Documentation mods for Forge 1.20.1",
    "url": "/wiki/useful-mods/documentation/1.20.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.20.1/forge/+page.svx",
    "description": "Documentation mods for Forge 1.20.1",
    "tags": []
  },
  {
    "title": "Documentation mods for NeoForge 1.20.1",
    "url": "/wiki/useful-mods/documentation/1.20.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.20.1/neoforge/+page.svx",
    "description": "Documentation mods for NeoForge 1.20.1",
    "tags": []
  },
  {
    "title": "Documentation mods for Fabric 1.21.1",
    "url": "/wiki/useful-mods/documentation/1.21.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.21.1/fabric/+page.svx",
    "description": "Documentation mods for Fabric 1.21.1",
    "tags": []
  },
  {
    "title": "Documentation mods for Forge 1.21.1",
    "url": "/wiki/useful-mods/documentation/1.21.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.21.1/forge/+page.svx",
    "description": "Documentation mods for Forge 1.21.1",
    "tags": []
  },
  {
    "title": "Documentation mods for NeoForge 1.21.1",
    "url": "/wiki/useful-mods/documentation/1.21.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/1.21.1/neoforge/+page.svx",
    "description": "Documentation mods for NeoForge 1.21.1",
    "tags": []
  },
  {
    "title": "Documentation mods for Fabric 26.1",
    "url": "/wiki/useful-mods/documentation/26.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/26.1/fabric/+page.svx",
    "description": "Documentation mods for Fabric 26.1",
    "tags": []
  },
  {
    "title": "Documentation mods for Forge 26.1",
    "url": "/wiki/useful-mods/documentation/26.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/26.1/forge/+page.svx",
    "description": "Documentation mods for Forge 26.1",
    "tags": []
  },
  {
    "title": "Documentation mods for NeoForge 26.1",
    "url": "/wiki/useful-mods/documentation/26.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/documentation/26.1/neoforge/+page.svx",
    "description": "Documentation mods for NeoForge 26.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Fabric 1.19.2",
    "url": "/wiki/useful-mods/multiplayer/1.19.2/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.19.2/fabric/+page.svx",
    "description": "Free Multiplayer mods for Fabric 1.19.2",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Forge 1.19.2",
    "url": "/wiki/useful-mods/multiplayer/1.19.2/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.19.2/forge/+page.svx",
    "description": "Free Multiplayer mods for Forge 1.19.2",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for NeoForge 1.19.2",
    "url": "/wiki/useful-mods/multiplayer/1.19.2/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.19.2/neoforge/+page.svx",
    "description": "Free Multiplayer mods for NeoForge 1.19.2",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Fabric 1.20.1",
    "url": "/wiki/useful-mods/multiplayer/1.20.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.20.1/fabric/+page.svx",
    "description": "Free Multiplayer mods for Fabric 1.20.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Forge 1.20.1",
    "url": "/wiki/useful-mods/multiplayer/1.20.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.20.1/forge/+page.svx",
    "description": "Free Multiplayer mods for Forge 1.20.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for NeoForge 1.20.1",
    "url": "/wiki/useful-mods/multiplayer/1.20.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.20.1/neoforge/+page.svx",
    "description": "Free Multiplayer mods for NeoForge 1.20.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Fabric 1.21.1",
    "url": "/wiki/useful-mods/multiplayer/1.21.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.21.1/fabric/+page.svx",
    "description": "Free Multiplayer mods for Fabric 1.21.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Forge 1.21.1",
    "url": "/wiki/useful-mods/multiplayer/1.21.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.21.1/forge/+page.svx",
    "description": "Free Multiplayer mods for Forge 1.21.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for NeoForge 1.21.1",
    "url": "/wiki/useful-mods/multiplayer/1.21.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/1.21.1/neoforge/+page.svx",
    "description": "Free Multiplayer mods for NeoForge 1.21.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Fabric 26.1",
    "url": "/wiki/useful-mods/multiplayer/26.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/26.1/fabric/+page.svx",
    "description": "Free Multiplayer mods for Fabric 26.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for Forge 26.1",
    "url": "/wiki/useful-mods/multiplayer/26.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/26.1/forge/+page.svx",
    "description": "Free Multiplayer mods for Forge 26.1",
    "tags": []
  },
  {
    "title": "Free Multiplayer mods for NeoForge 26.1",
    "url": "/wiki/useful-mods/multiplayer/26.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/multiplayer/26.1/neoforge/+page.svx",
    "description": "Free Multiplayer mods for NeoForge 26.1",
    "tags": []
  },
  {
    "title": "Performance mods for Fabric 1.19.2",
    "url": "/wiki/useful-mods/performance/1.19.2/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.19.2/fabric/+page.svx",
    "description": "Performance mods for Fabric 1.19.2",
    "tags": []
  },
  {
    "title": "Performance mods for Forge 1.19.2",
    "url": "/wiki/useful-mods/performance/1.19.2/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.19.2/forge/+page.svx",
    "description": "Performance mods for Forge 1.19.2",
    "tags": []
  },
  {
    "title": "Performance mods for NeoForge 1.19.2",
    "url": "/wiki/useful-mods/performance/1.19.2/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.19.2/neoforge/+page.svx",
    "description": "Performance mods for NeoForge 1.19.2",
    "tags": []
  },
  {
    "title": "Performance mods for Fabric 1.20.1",
    "url": "/wiki/useful-mods/performance/1.20.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.20.1/fabric/+page.svx",
    "description": "Performance mods for Fabric 1.20.1",
    "tags": []
  },
  {
    "title": "Performance mods for Forge 1.20.1",
    "url": "/wiki/useful-mods/performance/1.20.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.20.1/forge/+page.svx",
    "description": "Performance mods for Forge 1.20.1",
    "tags": []
  },
  {
    "title": "Performance mods for NeoForge 1.20.1",
    "url": "/wiki/useful-mods/performance/1.20.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.20.1/neoforge/+page.svx",
    "description": "Performance mods for NeoForge 1.20.1",
    "tags": []
  },
  {
    "title": "Performance mods for Fabric 1.21.1",
    "url": "/wiki/useful-mods/performance/1.21.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.21.1/fabric/+page.svx",
    "description": "Performance mods for Fabric 1.21.1",
    "tags": []
  },
  {
    "title": "Performance mods for Forge 1.21.1",
    "url": "/wiki/useful-mods/performance/1.21.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.21.1/forge/+page.svx",
    "description": "Performance mods for Forge 1.21.1",
    "tags": []
  },
  {
    "title": "Performance mods for NeoForge 1.21.1",
    "url": "/wiki/useful-mods/performance/1.21.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/1.21.1/neoforge/+page.svx",
    "description": "Performance mods for NeoForge 1.21.1",
    "tags": []
  },
  {
    "title": "Performance mods for Fabric 26.1",
    "url": "/wiki/useful-mods/performance/26.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/26.1/fabric/+page.svx",
    "description": "Performance mods for Fabric 26.1",
    "tags": []
  },
  {
    "title": "Performance mods for Forge 26.1",
    "url": "/wiki/useful-mods/performance/26.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/26.1/forge/+page.svx",
    "description": "Performance mods for Forge 26.1",
    "tags": []
  },
  {
    "title": "Performance mods for NeoForge 26.1",
    "url": "/wiki/useful-mods/performance/26.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/performance/26.1/neoforge/+page.svx",
    "description": "Performance mods for NeoForge 26.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Fabric 1.19.2",
    "url": "/wiki/useful-mods/profiling/1.19.2/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.19.2/fabric/+page.svx",
    "description": "Profiling/Debugging mods for Fabric 1.19.2",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Forge 1.19.2",
    "url": "/wiki/useful-mods/profiling/1.19.2/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.19.2/forge/+page.svx",
    "description": "Profiling/Debugging mods for Forge 1.19.2",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for NeoForge 1.19.2",
    "url": "/wiki/useful-mods/profiling/1.19.2/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.19.2/neoforge/+page.svx",
    "description": "Profiling/Debugging mods for NeoForge 1.19.2",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Fabric 1.20.1",
    "url": "/wiki/useful-mods/profiling/1.20.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.20.1/fabric/+page.svx",
    "description": "Profiling/Debugging mods for Fabric 1.20.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Forge 1.20.1",
    "url": "/wiki/useful-mods/profiling/1.20.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.20.1/forge/+page.svx",
    "description": "Profiling/Debugging mods for Forge 1.20.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for NeoForge 1.20.1",
    "url": "/wiki/useful-mods/profiling/1.20.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.20.1/neoforge/+page.svx",
    "description": "Profiling/Debugging mods for NeoForge 1.20.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Fabric 1.21.1",
    "url": "/wiki/useful-mods/profiling/1.21.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.21.1/fabric/+page.svx",
    "description": "Profiling/Debugging mods for Fabric 1.21.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Forge 1.21.1",
    "url": "/wiki/useful-mods/profiling/1.21.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.21.1/forge/+page.svx",
    "description": "Profiling/Debugging mods for Forge 1.21.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for NeoForge 1.21.1",
    "url": "/wiki/useful-mods/profiling/1.21.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/1.21.1/neoforge/+page.svx",
    "description": "Profiling/Debugging mods for NeoForge 1.21.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Fabric 26.1",
    "url": "/wiki/useful-mods/profiling/26.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/26.1/fabric/+page.svx",
    "description": "Profiling/Debugging mods for Fabric 26.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for Forge 26.1",
    "url": "/wiki/useful-mods/profiling/26.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/26.1/forge/+page.svx",
    "description": "Profiling/Debugging mods for Forge 26.1",
    "tags": []
  },
  {
    "title": "Profiling/Debugging mods for NeoForge 26.1",
    "url": "/wiki/useful-mods/profiling/26.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/profiling/26.1/neoforge/+page.svx",
    "description": "Profiling/Debugging mods for NeoForge 26.1",
    "tags": []
  },
  {
    "title": "Utility mods for Fabric 1.19.2",
    "url": "/wiki/useful-mods/utility/1.19.2/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.19.2/fabric/+page.svx",
    "description": "Utility mods for Fabric 1.19.2",
    "tags": []
  },
  {
    "title": "Utility mods for Forge 1.19.2",
    "url": "/wiki/useful-mods/utility/1.19.2/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.19.2/forge/+page.svx",
    "description": "Utility mods for Forge 1.19.2",
    "tags": []
  },
  {
    "title": "Utility mods for NeoForge 1.19.2",
    "url": "/wiki/useful-mods/utility/1.19.2/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.19.2/neoforge/+page.svx",
    "description": "Utility mods for NeoForge 1.19.2",
    "tags": []
  },
  {
    "title": "Utility mods for Fabric 1.20.1",
    "url": "/wiki/useful-mods/utility/1.20.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.20.1/fabric/+page.svx",
    "description": "Utility mods for Fabric 1.20.1",
    "tags": []
  },
  {
    "title": "Utility mods for Forge 1.20.1",
    "url": "/wiki/useful-mods/utility/1.20.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.20.1/forge/+page.svx",
    "description": "Utility mods for Forge 1.20.1",
    "tags": []
  },
  {
    "title": "Utility mods for NeoForge 1.20.1",
    "url": "/wiki/useful-mods/utility/1.20.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.20.1/neoforge/+page.svx",
    "description": "Utility mods for NeoForge 1.20.1",
    "tags": []
  },
  {
    "title": "Utility mods for Fabric 1.21.1",
    "url": "/wiki/useful-mods/utility/1.21.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.21.1/fabric/+page.svx",
    "description": "Utility mods for Fabric 1.21.1",
    "tags": []
  },
  {
    "title": "Utility mods for Forge 1.21.1",
    "url": "/wiki/useful-mods/utility/1.21.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.21.1/forge/+page.svx",
    "description": "Utility mods for Forge 1.21.1",
    "tags": []
  },
  {
    "title": "Utility mods for NeoForge 1.21.1",
    "url": "/wiki/useful-mods/utility/1.21.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/1.21.1/neoforge/+page.svx",
    "description": "Utility mods for NeoForge 1.21.1",
    "tags": []
  },
  {
    "title": "Utility mods for Fabric 26.1",
    "url": "/wiki/useful-mods/utility/26.1/fabric",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/26.1/fabric/+page.svx",
    "description": "Utility mods for Fabric 26.1",
    "tags": []
  },
  {
    "title": "Utility mods for Forge 26.1",
    "url": "/wiki/useful-mods/utility/26.1/forge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/26.1/forge/+page.svx",
    "description": "Utility mods for Forge 26.1",
    "tags": []
  },
  {
    "title": "Utility mods for NeoForge 26.1",
    "url": "/wiki/useful-mods/utility/26.1/neoforge",
    "sourcePath": "docs/modpack-dev-handbook/src/routes/wiki/useful-mods/utility/26.1/neoforge/+page.svx",
    "description": "Utility mods for NeoForge 26.1",
    "tags": []
  }
];

export const navSections: NavSection[] = [
  {
    "title": "Start Here",
    "children": [
      {
        "title": "Home",
        "url": "/",
        "children": []
      }
    ]
  },
  {
    "title": "Guides",
    "children": [
      {
        "title": "Guides",
        "url": null,
        "children": [
          {
            "title": "Custom Content",
            "url": null,
            "children": [
              {
                "title": "Adding new blocks",
                "url": "/guide/custom-content/adding-blocks",
                "children": []
              },
              {
                "title": "Adding new items",
                "url": "/guide/custom-content/adding-items",
                "children": []
              },
              {
                "title": "Attribute Modification",
                "url": "/guide/custom-content/attribute-modification",
                "children": []
              }
            ]
          },
          {
            "title": "Intro",
            "url": null,
            "children": [
              {
                "title": "Intro to Datapacks",
                "url": "/guide/intro/intro-datapack",
                "children": []
              },
              {
                "title": "Intro to Mopdpacks",
                "url": "/guide/intro/intro-modpack",
                "children": []
              },
              {
                "title": "Intro to Resource Packs",
                "url": "/guide/intro/intro-resourcepack",
                "children": []
              },
              {
                "title": "Introduction to modpack development",
                "url": "/guide/intro/intro-intro",
                "children": []
              },
              {
                "title": "Minecraft Concepts",
                "url": "/guide/intro/minecraft-concepts",
                "children": []
              }
            ]
          },
          {
            "title": "Removals",
            "url": null,
            "children": [
              {
                "title": "Removing Blocks",
                "url": "/guide/removals/removing-blocks",
                "children": []
              },
              {
                "title": "Removing items",
                "url": "/guide/removals/removing-items",
                "children": []
              }
            ]
          },
          {
            "title": "Worldgen",
            "url": null,
            "children": [
              {
                "title": "Modifying Biomes",
                "url": null,
                "children": [
                  {
                    "title": "Adding biomes to your modpack",
                    "url": "/guide/worldgen/modifying-biomes/adding-biomes",
                    "children": []
                  },
                  {
                    "title": "Removing biomes from your modpack",
                    "url": "/guide/worldgen/modifying-biomes/removing-biomes",
                    "children": []
                  }
                ]
              },
              {
                "title": "Modifying Features",
                "url": null,
                "children": [
                  {
                    "title": "Adding worldgen features to your modpack",
                    "url": "/guide/worldgen/modifying-features/adding-features",
                    "children": []
                  },
                  {
                    "title": "Removing features from your modpack",
                    "url": "/guide/worldgen/modifying-features/removing-features",
                    "children": []
                  }
                ]
              },
              {
                "title": "Modifying mob spawns",
                "url": "/guide/worldgen/mob-spawns",
                "children": []
              }
            ]
          },
          {
            "title": "Improving and Profiling Modpack Performance",
            "url": "/guide/performance",
            "children": []
          }
        ]
      }
    ]
  },
  {
    "title": "Reference",
    "children": [
      {
        "title": "Info",
        "url": null,
        "children": [
          {
            "title": "Data loading conditions",
            "url": "/wiki/info/data-loading-conditions",
            "children": []
          },
          {
            "title": "Free Multiplayer",
            "url": "/wiki/info/free-multiplayer",
            "children": []
          },
          {
            "title": "Licenses",
            "url": "/wiki/info/licenses",
            "children": []
          },
          {
            "title": "List of modpack launchers",
            "url": "/wiki/info/launchers",
            "children": []
          },
          {
            "title": "Regular Expressions",
            "url": "/wiki/info/regex",
            "children": []
          },
          {
            "title": "Useful Tags and Terms",
            "url": "/wiki/info/useful-terms",
            "children": []
          },
          {
            "title": "Version Control Tools",
            "url": "/wiki/info/version-control-tools",
            "children": []
          }
        ]
      },
      {
        "title": "Planning",
        "url": null,
        "children": [
          {
            "title": "Ideation",
            "url": "/wiki/planning/ideation",
            "children": []
          },
          {
            "title": "Mod Selection",
            "url": "/wiki/planning/mod-selection",
            "children": []
          },
          {
            "title": "Scope",
            "url": "/wiki/planning/scope",
            "children": []
          }
        ]
      },
      {
        "title": "Useful Mods",
        "url": "/wiki/useful-mods",
        "children": [
          {
            "title": "Bug Fixes",
            "url": null,
            "children": [
              {
                "title": "1.19.2",
                "url": null,
                "children": [
                  {
                    "title": "Bug Fixes mods for Fabric 1.19.2",
                    "url": "/wiki/useful-mods/bug_fixes/1.19.2/fabric",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for Forge 1.19.2",
                    "url": "/wiki/useful-mods/bug_fixes/1.19.2/forge",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for NeoForge 1.19.2",
                    "url": "/wiki/useful-mods/bug_fixes/1.19.2/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.20.1",
                "url": null,
                "children": [
                  {
                    "title": "Bug Fixes mods for Fabric 1.20.1",
                    "url": "/wiki/useful-mods/bug_fixes/1.20.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for Forge 1.20.1",
                    "url": "/wiki/useful-mods/bug_fixes/1.20.1/forge",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for NeoForge 1.20.1",
                    "url": "/wiki/useful-mods/bug_fixes/1.20.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.21.1",
                "url": null,
                "children": [
                  {
                    "title": "Bug Fixes mods for Fabric 1.21.1",
                    "url": "/wiki/useful-mods/bug_fixes/1.21.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for Forge 1.21.1",
                    "url": "/wiki/useful-mods/bug_fixes/1.21.1/forge",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for NeoForge 1.21.1",
                    "url": "/wiki/useful-mods/bug_fixes/1.21.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "26.1",
                "url": null,
                "children": [
                  {
                    "title": "Bug Fixes mods for Fabric 26.1",
                    "url": "/wiki/useful-mods/bug_fixes/26.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for Forge 26.1",
                    "url": "/wiki/useful-mods/bug_fixes/26.1/forge",
                    "children": []
                  },
                  {
                    "title": "Bug Fixes mods for NeoForge 26.1",
                    "url": "/wiki/useful-mods/bug_fixes/26.1/neoforge",
                    "children": []
                  }
                ]
              }
            ]
          },
          {
            "title": "Documentation",
            "url": null,
            "children": [
              {
                "title": "1.19.2",
                "url": null,
                "children": [
                  {
                    "title": "Documentation mods for Fabric 1.19.2",
                    "url": "/wiki/useful-mods/documentation/1.19.2/fabric",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for Forge 1.19.2",
                    "url": "/wiki/useful-mods/documentation/1.19.2/forge",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for NeoForge 1.19.2",
                    "url": "/wiki/useful-mods/documentation/1.19.2/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.20.1",
                "url": null,
                "children": [
                  {
                    "title": "Documentation mods for Fabric 1.20.1",
                    "url": "/wiki/useful-mods/documentation/1.20.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for Forge 1.20.1",
                    "url": "/wiki/useful-mods/documentation/1.20.1/forge",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for NeoForge 1.20.1",
                    "url": "/wiki/useful-mods/documentation/1.20.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.21.1",
                "url": null,
                "children": [
                  {
                    "title": "Documentation mods for Fabric 1.21.1",
                    "url": "/wiki/useful-mods/documentation/1.21.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for Forge 1.21.1",
                    "url": "/wiki/useful-mods/documentation/1.21.1/forge",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for NeoForge 1.21.1",
                    "url": "/wiki/useful-mods/documentation/1.21.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "26.1",
                "url": null,
                "children": [
                  {
                    "title": "Documentation mods for Fabric 26.1",
                    "url": "/wiki/useful-mods/documentation/26.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for Forge 26.1",
                    "url": "/wiki/useful-mods/documentation/26.1/forge",
                    "children": []
                  },
                  {
                    "title": "Documentation mods for NeoForge 26.1",
                    "url": "/wiki/useful-mods/documentation/26.1/neoforge",
                    "children": []
                  }
                ]
              }
            ]
          },
          {
            "title": "Multiplayer",
            "url": null,
            "children": [
              {
                "title": "1.19.2",
                "url": null,
                "children": [
                  {
                    "title": "Free Multiplayer mods for Fabric 1.19.2",
                    "url": "/wiki/useful-mods/multiplayer/1.19.2/fabric",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for Forge 1.19.2",
                    "url": "/wiki/useful-mods/multiplayer/1.19.2/forge",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for NeoForge 1.19.2",
                    "url": "/wiki/useful-mods/multiplayer/1.19.2/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.20.1",
                "url": null,
                "children": [
                  {
                    "title": "Free Multiplayer mods for Fabric 1.20.1",
                    "url": "/wiki/useful-mods/multiplayer/1.20.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for Forge 1.20.1",
                    "url": "/wiki/useful-mods/multiplayer/1.20.1/forge",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for NeoForge 1.20.1",
                    "url": "/wiki/useful-mods/multiplayer/1.20.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.21.1",
                "url": null,
                "children": [
                  {
                    "title": "Free Multiplayer mods for Fabric 1.21.1",
                    "url": "/wiki/useful-mods/multiplayer/1.21.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for Forge 1.21.1",
                    "url": "/wiki/useful-mods/multiplayer/1.21.1/forge",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for NeoForge 1.21.1",
                    "url": "/wiki/useful-mods/multiplayer/1.21.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "26.1",
                "url": null,
                "children": [
                  {
                    "title": "Free Multiplayer mods for Fabric 26.1",
                    "url": "/wiki/useful-mods/multiplayer/26.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for Forge 26.1",
                    "url": "/wiki/useful-mods/multiplayer/26.1/forge",
                    "children": []
                  },
                  {
                    "title": "Free Multiplayer mods for NeoForge 26.1",
                    "url": "/wiki/useful-mods/multiplayer/26.1/neoforge",
                    "children": []
                  }
                ]
              }
            ]
          },
          {
            "title": "Performance",
            "url": null,
            "children": [
              {
                "title": "1.19.2",
                "url": null,
                "children": [
                  {
                    "title": "Performance mods for Fabric 1.19.2",
                    "url": "/wiki/useful-mods/performance/1.19.2/fabric",
                    "children": []
                  },
                  {
                    "title": "Performance mods for Forge 1.19.2",
                    "url": "/wiki/useful-mods/performance/1.19.2/forge",
                    "children": []
                  },
                  {
                    "title": "Performance mods for NeoForge 1.19.2",
                    "url": "/wiki/useful-mods/performance/1.19.2/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.20.1",
                "url": null,
                "children": [
                  {
                    "title": "Performance mods for Fabric 1.20.1",
                    "url": "/wiki/useful-mods/performance/1.20.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Performance mods for Forge 1.20.1",
                    "url": "/wiki/useful-mods/performance/1.20.1/forge",
                    "children": []
                  },
                  {
                    "title": "Performance mods for NeoForge 1.20.1",
                    "url": "/wiki/useful-mods/performance/1.20.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.21.1",
                "url": null,
                "children": [
                  {
                    "title": "Performance mods for Fabric 1.21.1",
                    "url": "/wiki/useful-mods/performance/1.21.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Performance mods for Forge 1.21.1",
                    "url": "/wiki/useful-mods/performance/1.21.1/forge",
                    "children": []
                  },
                  {
                    "title": "Performance mods for NeoForge 1.21.1",
                    "url": "/wiki/useful-mods/performance/1.21.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "26.1",
                "url": null,
                "children": [
                  {
                    "title": "Performance mods for Fabric 26.1",
                    "url": "/wiki/useful-mods/performance/26.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Performance mods for Forge 26.1",
                    "url": "/wiki/useful-mods/performance/26.1/forge",
                    "children": []
                  },
                  {
                    "title": "Performance mods for NeoForge 26.1",
                    "url": "/wiki/useful-mods/performance/26.1/neoforge",
                    "children": []
                  }
                ]
              }
            ]
          },
          {
            "title": "Profiling",
            "url": null,
            "children": [
              {
                "title": "1.19.2",
                "url": null,
                "children": [
                  {
                    "title": "Profiling/Debugging mods for Fabric 1.19.2",
                    "url": "/wiki/useful-mods/profiling/1.19.2/fabric",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for Forge 1.19.2",
                    "url": "/wiki/useful-mods/profiling/1.19.2/forge",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for NeoForge 1.19.2",
                    "url": "/wiki/useful-mods/profiling/1.19.2/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.20.1",
                "url": null,
                "children": [
                  {
                    "title": "Profiling/Debugging mods for Fabric 1.20.1",
                    "url": "/wiki/useful-mods/profiling/1.20.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for Forge 1.20.1",
                    "url": "/wiki/useful-mods/profiling/1.20.1/forge",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for NeoForge 1.20.1",
                    "url": "/wiki/useful-mods/profiling/1.20.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.21.1",
                "url": null,
                "children": [
                  {
                    "title": "Profiling/Debugging mods for Fabric 1.21.1",
                    "url": "/wiki/useful-mods/profiling/1.21.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for Forge 1.21.1",
                    "url": "/wiki/useful-mods/profiling/1.21.1/forge",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for NeoForge 1.21.1",
                    "url": "/wiki/useful-mods/profiling/1.21.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "26.1",
                "url": null,
                "children": [
                  {
                    "title": "Profiling/Debugging mods for Fabric 26.1",
                    "url": "/wiki/useful-mods/profiling/26.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for Forge 26.1",
                    "url": "/wiki/useful-mods/profiling/26.1/forge",
                    "children": []
                  },
                  {
                    "title": "Profiling/Debugging mods for NeoForge 26.1",
                    "url": "/wiki/useful-mods/profiling/26.1/neoforge",
                    "children": []
                  }
                ]
              }
            ]
          },
          {
            "title": "Utility",
            "url": null,
            "children": [
              {
                "title": "1.19.2",
                "url": null,
                "children": [
                  {
                    "title": "Utility mods for Fabric 1.19.2",
                    "url": "/wiki/useful-mods/utility/1.19.2/fabric",
                    "children": []
                  },
                  {
                    "title": "Utility mods for Forge 1.19.2",
                    "url": "/wiki/useful-mods/utility/1.19.2/forge",
                    "children": []
                  },
                  {
                    "title": "Utility mods for NeoForge 1.19.2",
                    "url": "/wiki/useful-mods/utility/1.19.2/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.20.1",
                "url": null,
                "children": [
                  {
                    "title": "Utility mods for Fabric 1.20.1",
                    "url": "/wiki/useful-mods/utility/1.20.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Utility mods for Forge 1.20.1",
                    "url": "/wiki/useful-mods/utility/1.20.1/forge",
                    "children": []
                  },
                  {
                    "title": "Utility mods for NeoForge 1.20.1",
                    "url": "/wiki/useful-mods/utility/1.20.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "1.21.1",
                "url": null,
                "children": [
                  {
                    "title": "Utility mods for Fabric 1.21.1",
                    "url": "/wiki/useful-mods/utility/1.21.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Utility mods for Forge 1.21.1",
                    "url": "/wiki/useful-mods/utility/1.21.1/forge",
                    "children": []
                  },
                  {
                    "title": "Utility mods for NeoForge 1.21.1",
                    "url": "/wiki/useful-mods/utility/1.21.1/neoforge",
                    "children": []
                  }
                ]
              },
              {
                "title": "26.1",
                "url": null,
                "children": [
                  {
                    "title": "Utility mods for Fabric 26.1",
                    "url": "/wiki/useful-mods/utility/26.1/fabric",
                    "children": []
                  },
                  {
                    "title": "Utility mods for Forge 26.1",
                    "url": "/wiki/useful-mods/utility/26.1/forge",
                    "children": []
                  },
                  {
                    "title": "Utility mods for NeoForge 26.1",
                    "url": "/wiki/useful-mods/utility/26.1/neoforge",
                    "children": []
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  },
  {
    "title": "Pack Management",
    "children": [
      {
        "title": "Pack Management",
        "url": "/wiki/modpack-management",
        "children": [
          {
            "title": "Packwand",
            "url": null,
            "children": [
              {
                "title": "Development",
                "url": null,
                "children": [
                  {
                    "title": "Building the native GUI app",
                    "url": "/wiki/modpack-management/packwand/development/gui-build",
                    "children": []
                  }
                ]
              },
              {
                "title": "Reference",
                "url": null,
                "children": [
                  {
                    "title": "Pack Format",
                    "url": null,
                    "children": [
                      {
                        "title": ".packwizignore",
                        "url": "/wiki/modpack-management/packwand/reference/pack-format/packwizignore",
                        "children": []
                      },
                      {
                        "title": "index.toml",
                        "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml",
                        "children": []
                      },
                      {
                        "title": "manifest.json",
                        "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json",
                        "children": []
                      },
                      {
                        "title": "mod.pw.toml",
                        "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml",
                        "children": []
                      },
                      {
                        "title": "pack.toml",
                        "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml",
                        "children": []
                      }
                    ]
                  },
                  {
                    "title": "Additional options",
                    "url": "/wiki/modpack-management/packwand/reference/additional-options",
                    "children": []
                  }
                ]
              },
              {
                "title": "Tutorials",
                "url": null,
                "children": [
                  {
                    "title": "Creating",
                    "url": null,
                    "children": [
                      {
                        "title": "Adding mods and resource packs",
                        "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods",
                        "children": []
                      },
                      {
                        "title": "Getting started",
                        "url": "/wiki/modpack-management/packwand/tutorials/creating/getting-started",
                        "children": []
                      },
                      {
                        "title": "Using packwand with Git",
                        "url": "/wiki/modpack-management/packwand/tutorials/creating/git",
                        "children": []
                      }
                    ]
                  },
                  {
                    "title": "Hosting",
                    "url": null,
                    "children": [
                      {
                        "title": "Publishing to CurseForge",
                        "url": "/wiki/modpack-management/packwand/tutorials/hosting/curseforge",
                        "children": []
                      },
                      {
                        "title": "Publishing to Modrinth",
                        "url": "/wiki/modpack-management/packwand/tutorials/hosting/modrinth",
                        "children": []
                      }
                    ]
                  },
                  {
                    "title": "Installing",
                    "url": null,
                    "children": [
                      {
                        "title": "Pack Installation using packwiz-installer",
                        "url": "/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer",
                        "children": []
                      }
                    ]
                  }
                ]
              },
              {
                "title": "Installation",
                "url": "/wiki/modpack-management/packwand/installation",
                "children": []
              }
            ]
          },
          {
            "title": "Packwiz",
            "url": null,
            "children": [
              {
                "title": "Components",
                "url": null,
                "children": [
                  {
                    "title": "Bootstrap",
                    "url": "/wiki/modpack-management/packwiz/components/bootstrap",
                    "children": []
                  },
                  {
                    "title": "Building",
                    "url": "/wiki/modpack-management/packwiz/components/building",
                    "children": []
                  },
                  {
                    "title": "modbrowserwebview",
                    "url": "/wiki/modpack-management/packwiz/components/webview",
                    "children": []
                  },
                  {
                    "title": "packwiz-installer",
                    "url": "/wiki/modpack-management/packwiz/components/installer",
                    "children": []
                  }
                ]
              },
              {
                "title": "Reference",
                "url": null,
                "children": [
                  {
                    "title": "Pack Format",
                    "url": null,
                    "children": [
                      {
                        "title": ".packwizignore",
                        "url": "/wiki/modpack-management/packwiz/reference/pack-format/packwizignore",
                        "children": []
                      }
                    ]
                  },
                  {
                    "title": "Additional options",
                    "url": "/wiki/modpack-management/packwiz/reference/additional-options",
                    "children": []
                  }
                ]
              },
              {
                "title": "Tutorials",
                "url": null,
                "children": [
                  {
                    "title": "Creating",
                    "url": null,
                    "children": [
                      {
                        "title": "Adding mods and resource packs",
                        "url": "/wiki/modpack-management/packwiz/tutorials/creating/adding-mods",
                        "children": []
                      },
                      {
                        "title": "Getting started",
                        "url": "/wiki/modpack-management/packwiz/tutorials/creating/getting-started",
                        "children": []
                      },
                      {
                        "title": "Using packwiz with Git",
                        "url": "/wiki/modpack-management/packwiz/tutorials/creating/git",
                        "children": []
                      }
                    ]
                  },
                  {
                    "title": "Hosting",
                    "url": null,
                    "children": [
                      {
                        "title": "Publishing to CurseForge",
                        "url": "/wiki/modpack-management/packwiz/tutorials/hosting/curseforge",
                        "children": []
                      },
                      {
                        "title": "Publishing to Modrinth",
                        "url": "/wiki/modpack-management/packwiz/tutorials/hosting/modrinth",
                        "children": []
                      }
                    ]
                  },
                  {
                    "title": "Installing",
                    "url": null,
                    "children": [
                      {
                        "title": "Pack Installation using packwiz-installer",
                        "url": "/wiki/modpack-management/packwiz/tutorials/installing/packwiz-installer",
                        "children": []
                      }
                    ]
                  }
                ]
              },
              {
                "title": "Installation",
                "url": "/wiki/modpack-management/packwiz/installation",
                "children": []
              },
              {
                "title": "Packwiz Components",
                "url": "/wiki/modpack-management/packwiz/components",
                "children": []
              }
            ]
          },
          {
            "title": "CurseForge",
            "url": "/wiki/modpack-management/curseforge",
            "children": []
          },
          {
            "title": "Marketing",
            "url": "/wiki/modpack-management/marketing",
            "children": []
          },
          {
            "title": "Modrinth",
            "url": "/wiki/modpack-management/modrinth",
            "children": []
          },
          {
            "title": "packwand",
            "url": "/wiki/modpack-management/packwand",
            "children": []
          },
          {
            "title": "packwiz",
            "url": "/wiki/modpack-management/packwiz",
            "children": []
          },
          {
            "title": "Project Management",
            "url": "/wiki/modpack-management/project-management",
            "children": []
          }
        ]
      }
    ]
  },
  {
    "title": "Contribute",
    "children": [
      {
        "title": "Contribute",
        "url": null,
        "children": [
          {
            "title": "Git Practices",
            "url": "/contribute/git-practices",
            "children": []
          },
          {
            "title": "Page Formatting",
            "url": "/contribute/formatting",
            "children": []
          }
        ]
      },
      {
        "title": "Credits",
        "url": "/credits",
        "children": []
      }
    ]
  }
];

export function normalizeDocUrl(url: string): string {
  if (url.length > 1 && url.endsWith('/')) return url.slice(0, -1);
  return url || '/';
}

const docsByUrl = new Map(docsIndex.map((doc) => [normalizeDocUrl(doc.url), doc]));

export function findDocByUrl(url: string): DocMeta | undefined {
  return docsByUrl.get(normalizeDocUrl(url));
}
