export type SearchDocument = {
  kind: 'page' | 'section';
  title: string;
  pageTitle: string;
  sectionTitle: string;
  description: string;
  content: string;
  url: string;
  tags: string[];
};

export const searchDocuments: SearchDocument[] = [
  {
    "kind": "page",
    "title": "Home",
    "pageTitle": "Home",
    "sectionTitle": "",
    "description": "Unified handbook documentation for modpack development, pack publishing, and pack management tooling.",
    "content": "Modpack Dev Handbook This handbook is the single documentation site for modpack development guidance and pack management tooling for Minecraft Modding. Handbook pages cover planning, design, debugging, and evergreen reference material. Pack management pages cover packwand, packwiz, publishing targets, and installer/runtime components. Generated command docs for packwand now live directly inside the handbook route tree instead of being imported from a separate site. Pack Management Start with Pack Management for the practical toolchain: publishing platforms, pack formats, packwand, packwiz, and related components. Gleam Utilities We also use Gleam where it is a good fit for small deterministic helpers. The example below uses Gleam compiled functions to compare Minecraft versions and slugify pack names. Contribute All content in this handbook is source controlled in this repository. Use the Source button in the top bar to jump directly to the backing file for the current page.",
    "url": "/",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Home / Contribute",
    "pageTitle": "Home",
    "sectionTitle": "Contribute",
    "description": "Unified handbook documentation for modpack development, pack publishing, and pack management tooling.",
    "content": "All content in this handbook is source controlled in this repository. Use the Source button in the top bar to jump directly to the backing file for the current page.",
    "url": "/#contribute",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Home / Gleam Utilities",
    "pageTitle": "Home",
    "sectionTitle": "Gleam Utilities",
    "description": "Unified handbook documentation for modpack development, pack publishing, and pack management tooling.",
    "content": "We also use Gleam where it is a good fit for small deterministic helpers. The example below uses Gleam compiled functions to compare Minecraft versions and slugify pack names.",
    "url": "/#gleam-utilities",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Home / Pack Management",
    "pageTitle": "Home",
    "sectionTitle": "Pack Management",
    "description": "Unified handbook documentation for modpack development, pack publishing, and pack management tooling.",
    "content": "Start with Pack Management for the practical toolchain: publishing platforms, pack formats, packwand, packwiz, and related components.",
    "url": "/#pack-management",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Page Formatting",
    "pageTitle": "Page Formatting",
    "sectionTitle": "",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "content": "Page Formatting Last Updated: 7 08 2026 This section goes over how we format wiki pages. We like to be consistent, so please read through this section carefully and use these features to your advantage: Basic Writing Guidelines When writing for the wiki, write in a way that is easy to understand and easy for beginners to understand. Put yourself in the viewer's shoes. What confused you when you first learned about the topic? What new terms did you learn? Avoid using technical terms without explaining them or point to a resource that explains them. Documentation and tutorials are a great way to learn, but sometimes you don't need to read every part of a page to understand it. When writing for the wiki, write in a way that makes it easy to scan and understand quickly. Some recommendations are using white space to your advantage to break your page into easily digestible chunks. When learning a concept, it is helpful to have concrete examples that people can refer to instead of just using a concept. This will help wiki goers understand the concept better and make it easier to remember. Writing Style The datapacking community is a diverse group of people with different backgrounds; many people don't speak English as their first language! When writing, try to follow these 5 guidelines: 1. Use the active voice. For example, instead of The pig is teleported by the command, write The command teleported the pig. 2. Don't use unnecessary adverbs or adjectives 3. Try not to use the words: obvious, simple, basic, easy, actual, just, clear, and however 4. Explicitly reference what you are explaining 5. Use 's for indicating possession Technical information All content on the website (except a few small exceptions) are made using a technology called mdsvex. This technology enables people like you to insert Markdown with svelte components. It is recommended to know what the proper way to format Markdown is in order to stay consistent and prevent confusion. The front matter title is the same as the title in the sidebar and the title on the page (heading 1 or single ) Use bold and italics sparingly and only when emphasis is needed Use headings to break up the page into sections Code blocks are used to show code snippets or commands Admonitions are used to show important information unrelated to the content of the page Tables are used to show large amounts of data Each page is made of 3 parts: front matter (metadata about the page such as title, description, tags, version, etc.) content (the actual content of the page) components (custom components that allow for interactivity or other features not able to be reproduced with markdown) Each is crucial to making the page look and feel how it does. Frontmatter We try to keep the front matter as minimal as possible, but it is still required. Without it, the page will not display correctly on search engines or other sites. The front matter for this page looks like this: Front matter is denoted with triple hyphens ( ) at the top of the page and the end of the front matter. The title should be the same as the title in the sidebar in order to reduce confusion. The description should be a short summary of the content of the page in order to show people what all is covered in the article. The version should be set to the latest version that the page has been and works in. If the page works in 1.21.4 but not in 1.21.5 or later, this should be set to 1.21.4. Custom Elements Our markdown system adds unlimited customizability to the way we format our pages. As of the time of writing, we have the following features: Admonitions Code Titles MCFunction Formatting (Thanks Snave!) Highlighting Admonitions are a way to warnings, info or tips, or other important information to your page. :::info This is an example of an info box. ::: Code blocks are a way to format code in your page. These code blocks come with the option to add a title to the code block for clarity. The Modpack Dev Wiki supports syntax highlighting for MCFunction which are used for code samples whenever possible. Highlighting is a way to highlight specific text. It isn't commonly used, but exists. Highlighted Text like this.",
    "url": "/contribute/formatting",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Page Formatting / Basic Writing Guidelines",
    "pageTitle": "Page Formatting",
    "sectionTitle": "Basic Writing Guidelines",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "content": "When writing for the wiki, write in a way that is easy to understand and easy for beginners to understand. Put yourself in the viewer's shoes. What confused you when you first learned about the topic? What new terms did you learn? Avoid using technical terms without explaining them or point to a resource that explains them. Documentation and tutorials are a great way to learn, but sometimes you don't need to read every part of a page to understand it. When writing for the wiki, write in a way that makes it easy to scan and understand quickly. Some recommendations are using white space to your advantage to break your page into easily digestible chunks. When learning a concept, it is helpful to have concrete examples that people can refer to instead of just using a concept. This will help wiki goers understand the concept better and make it easier to remember.",
    "url": "/contribute/formatting#basic-writing-guidelines",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Page Formatting / Custom Elements",
    "pageTitle": "Page Formatting",
    "sectionTitle": "Custom Elements",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "content": "Our markdown system adds unlimited customizability to the way we format our pages. As of the time of writing, we have the following features: Admonitions Code Titles MCFunction Formatting (Thanks Snave!) Highlighting Admonitions are a way to warnings, info or tips, or other important information to your page. :::info This is an example of an info box. ::: Code blocks are a way to format code in your page. These code blocks come with the option to add a title to the code block for clarity. The Modpack Dev Wiki supports syntax highlighting for MCFunction which are used for code samples whenever possible. Highlighting is a way to highlight specific text. It isn't commonly used, but exists. Highlighted Text like this.",
    "url": "/contribute/formatting#custom-elements",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Page Formatting / Frontmatter",
    "pageTitle": "Page Formatting",
    "sectionTitle": "Frontmatter",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "content": "We try to keep the front matter as minimal as possible, but it is still required. Without it, the page will not display correctly on search engines or other sites. The front matter for this page looks like this: Front matter is denoted with triple hyphens ( ) at the top of the page and the end of the front matter. The title should be the same as the title in the sidebar in order to reduce confusion. The description should be a short summary of the content of the page in order to show people what all is covered in the article. The version should be set to the latest version that the page has been and works in. If the page works in 1.21.4 but not in 1.21.5 or later, this should be set to 1.21.4.",
    "url": "/contribute/formatting#frontmatter",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Page Formatting / Technical information",
    "pageTitle": "Page Formatting",
    "sectionTitle": "Technical information",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "content": "All content on the website (except a few small exceptions) are made using a technology called mdsvex. This technology enables people like you to insert Markdown with svelte components. It is recommended to know what the proper way to format Markdown is in order to stay consistent and prevent confusion. The front matter title is the same as the title in the sidebar and the title on the page (heading 1 or single ) Use bold and italics sparingly and only when emphasis is needed Use headings to break up the page into sections Code blocks are used to show code snippets or commands Admonitions are used to show important information unrelated to the content of the page Tables are used to show large amounts of data Each page is made of 3 parts: front matter (metadata about the page such as title, description, tags, version, etc.) content (the actual content of the page) components (custom components that allow for interactivity or other features not able to be reproduced with markdown) Each is crucial to making the page look and feel how it does.",
    "url": "/contribute/formatting#technical-information",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Page Formatting / Writing Style",
    "pageTitle": "Page Formatting",
    "sectionTitle": "Writing Style",
    "description": "This page is an introduction to formatting page content, and details about how the wiki handles formatting.",
    "content": "The datapacking community is a diverse group of people with different backgrounds; many people don't speak English as their first language! When writing, try to follow these 5 guidelines: 1. Use the active voice. For example, instead of The pig is teleported by the command, write The command teleported the pig. 2. Don't use unnecessary adverbs or adjectives 3. Try not to use the words: obvious, simple, basic, easy, actual, just, clear, and however 4. Explicitly reference what you are explaining 5. Use 's for indicating possession",
    "url": "/contribute/formatting#writing-style",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Git Practices",
    "pageTitle": "Git Practices",
    "sectionTitle": "",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "content": "Git Practices Last Updated: 07 08 2026 This page is an introduction to how we use Git in the wiki repository . In order to keep the wiki consistent and reputable, we have a few rules that we follow. Git provides a lot of features that are great for collaboration, and we try to use them as much as possible. :::info This guide assumes you already have experience using Git before. ::: Forking and PRs We currently do not accept PR's from outside members to the wiki, due to limitations around our ForgeJo. If you would like to contribute, please send @omo50 a friend request on Discord. Branches Branches are useful additions to help separate features in your fork. Please allocate your username and the feature you are working on for the branch you are working on. IE. omo50 works on a new KubeJS section, that would be under the branch omo/kubejs. Commit Messages Abide by the Conventional Commit standard as outlined in CONTRIBUTING.md at the repository root. Merging Whenever you start working on a new branch or features, pull the latest changes from the main branch. This will ensure that you have the most up to date changes. Other Important Information Make a description of your changes in your PR. Reviewers: Proofread changes before approving them.",
    "url": "/contribute/git-practices",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Git Practices / Branches",
    "pageTitle": "Git Practices",
    "sectionTitle": "Branches",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "content": "Branches are useful additions to help separate features in your fork. Please allocate your username and the feature you are working on for the branch you are working on. IE. omo50 works on a new KubeJS section, that would be under the branch omo/kubejs.",
    "url": "/contribute/git-practices#branches",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Git Practices / Commit Messages",
    "pageTitle": "Git Practices",
    "sectionTitle": "Commit Messages",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "content": "Abide by the Conventional Commit standard as outlined in CONTRIBUTING.md at the repository root.",
    "url": "/contribute/git-practices#commit-messages",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Git Practices / Forking and PRs",
    "pageTitle": "Git Practices",
    "sectionTitle": "Forking and PRs",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "content": "We currently do not accept PR's from outside members to the wiki, due to limitations around our ForgeJo. If you would like to contribute, please send @omo50 a friend request on Discord.",
    "url": "/contribute/git-practices#forking-and-prs",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Git Practices / Merging",
    "pageTitle": "Git Practices",
    "sectionTitle": "Merging",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "content": "Whenever you start working on a new branch or features, pull the latest changes from the main branch. This will ensure that you have the most up to date changes.",
    "url": "/contribute/git-practices#merging",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Git Practices / Other Important Information",
    "pageTitle": "Git Practices",
    "sectionTitle": "Other Important Information",
    "description": "This page is an introduction to our Git practices as a page for the wiki.",
    "content": "Make a description of your changes in your PR. Reviewers: Proofread changes before approving them.",
    "url": "/contribute/git-practices#other-important-information",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Credits",
    "pageTitle": "Credits",
    "sectionTitle": "",
    "description": "The Modpack Dev Handbook Credits.",
    "content": "Credits The Modpack Dev Handbook is a fork of the Modpack Development Knowledgebase, licensed under MIT. Developed and Maintained by Reverie Projects @ nostalgica.net.",
    "url": "/credits",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Adding new blocks",
    "pageTitle": "Adding new blocks",
    "sectionTitle": "",
    "description": "Adding new blocks",
    "content": "Adding new Blocks Resources Each block must have four assets, the block texture itself, models for both the item and block, and blockstate definitions. In your resourcepack's asset folder: assets/packid/textures/block/ Block textures (.png files) should be placed here assets/packid/models/block/ Model file (.json) should be placed here assets/packid/models/item/ The item form of your block's file (.json) should be placed here assets/packid/models/blockstates/ Blockstate Json files (.json) should be placed here Models can be made using Blockbench, a free program that allows you to model, texture, and animate any kind of element in Minecraft. Since every block (should) also be an item, your item model json can have the block model as a parent. Blockstates can be slightly more complex. If you're using something like KubeJs, the BlockState json might be generated automatically for you. If you block has more complex states (think a furnace being rotatable and also having an on and off state), you will need a custom blockstate json. You can find more information in this in the Minecraft Wiki. :::info If your model is not rendering in the world properly but the model file looks perfect, it likely is an issue with your blockstate json! :::: KubeJs KubeJs handles some parts of the block creation process easier than other programs. For starters, you have access to the kubejs/assets folder, which is a dynamic resource pack. It also automatically generates basic BlockState json unless you specify a different one using the builder. All block registration scripts need to be added in KubeJs' startup scripts directory. The full KubeJs wiki page can be found here",
    "url": "/guide/custom-content/adding-blocks",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding new blocks / KubeJs",
    "pageTitle": "Adding new blocks",
    "sectionTitle": "KubeJs",
    "description": "Adding new blocks",
    "content": "KubeJs handles some parts of the block creation process easier than other programs. For starters, you have access to the kubejs/assets folder, which is a dynamic resource pack. It also automatically generates basic BlockState json unless you specify a different one using the builder. All block registration scripts need to be added in KubeJs' startup scripts directory. The full KubeJs wiki page can be found here",
    "url": "/guide/custom-content/adding-blocks#kubejs",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding new blocks / Resources",
    "pageTitle": "Adding new blocks",
    "sectionTitle": "Resources",
    "description": "Adding new blocks",
    "content": "Each block must have four assets, the block texture itself, models for both the item and block, and blockstate definitions. In your resourcepack's asset folder: assets/packid/textures/block/ Block textures (.png files) should be placed here assets/packid/models/block/ Model file (.json) should be placed here assets/packid/models/item/ The item form of your block's file (.json) should be placed here assets/packid/models/blockstates/ Blockstate Json files (.json) should be placed here Models can be made using Blockbench, a free program that allows you to model, texture, and animate any kind of element in Minecraft. Since every block (should) also be an item, your item model json can have the block model as a parent. Blockstates can be slightly more complex. If you're using something like KubeJs, the BlockState json might be generated automatically for you. If you block has more complex states (think a furnace being rotatable and also having an on and off state), you will need a custom blockstate json. You can find more information in this in the Minecraft Wiki. :::info If your model is not rendering in the world properly but the model file looks perfect, it likely is an issue with your blockstate json! ::::",
    "url": "/guide/custom-content/adding-blocks#resources",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Adding new items",
    "pageTitle": "Adding new items",
    "sectionTitle": "",
    "description": "Adding new items",
    "content": "Adding new items Resources Each item must have two assets, the item texture itself, and the model that tells which texture to use. In your resourcepack's asset folder: assets/packid/textures/item/ Item textures (.png files) should be placed here assets/packid/models/item/ Model files (.json files) should be placed here Items can have any kind of models if you're familiar with a program like blockbench, but in most cases the default item model for flat icons looks like this: KubeJs KubeJs handles some parts of the item creation process easier than other programs. For starters, you have access to the kubejs/assets folder, which is a dynamic resource pack. It also automatically generates basic models unless you specify a different one using the builder. All item registration scripts need to be added in KubeJs' startup scripts directory. The full KubeJs wiki page can be found here",
    "url": "/guide/custom-content/adding-items",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding new items / KubeJs",
    "pageTitle": "Adding new items",
    "sectionTitle": "KubeJs",
    "description": "Adding new items",
    "content": "KubeJs handles some parts of the item creation process easier than other programs. For starters, you have access to the kubejs/assets folder, which is a dynamic resource pack. It also automatically generates basic models unless you specify a different one using the builder. All item registration scripts need to be added in KubeJs' startup scripts directory. The full KubeJs wiki page can be found here",
    "url": "/guide/custom-content/adding-items#kubejs",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding new items / Resources",
    "pageTitle": "Adding new items",
    "sectionTitle": "Resources",
    "description": "Adding new items",
    "content": "Each item must have two assets, the item texture itself, and the model that tells which texture to use. In your resourcepack's asset folder: assets/packid/textures/item/ Item textures (.png files) should be placed here assets/packid/models/item/ Model files (.json files) should be placed here Items can have any kind of models if you're familiar with a program like blockbench, but in most cases the default item model for flat icons looks like this:",
    "url": "/guide/custom-content/adding-items#resources",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Attribute Modification",
    "pageTitle": "Attribute Modification",
    "sectionTitle": "",
    "description": "Modifying attributes of items and entities",
    "content": "Attribute Modification Both Attribute Setter and KubeJS can add attributes to items, armor, and curios. This will cover how to do both. Attribute Setter Attribute Setter relies on datapacks to apply attributes. Their schema and example files can be found here. data/my namespace/attributesetter/entity/modify entity.json :::tip UUIDs for the uuid field can be obtained through UUID generator sites such as https://www.uuidgenerator.net ::: KubeJS WIP :::info Other mods that have this functionality are Attributizer and Custom Item Attributes, but they were not included in this guide for brevity and redundancy. :::",
    "url": "/guide/custom-content/attribute-modification",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Attribute Modification / Attribute Setter",
    "pageTitle": "Attribute Modification",
    "sectionTitle": "Attribute Setter",
    "description": "Modifying attributes of items and entities",
    "content": "Attribute Setter relies on datapacks to apply attributes. Their schema and example files can be found here. data/my namespace/attributesetter/entity/modify entity.json :::tip UUIDs for the uuid field can be obtained through UUID generator sites such as https://www.uuidgenerator.net :::",
    "url": "/guide/custom-content/attribute-modification#attribute-setter",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Attribute Modification / KubeJS",
    "pageTitle": "Attribute Modification",
    "sectionTitle": "KubeJS",
    "description": "Modifying attributes of items and entities",
    "content": "WIP :::info Other mods that have this functionality are Attributizer and Custom Item Attributes, but they were not included in this guide for brevity and redundancy. :::",
    "url": "/guide/custom-content/attribute-modification#kubejs",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Intro to Datapacks",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "",
    "description": "Introduction and tutorial for datapacks",
    "content": "Intro to Datapacks A datapack is a collection of data stored in a folder or .zip file used to modify elements of the game. Due to them being predominately JSON based, they are relatively easy to get into with no prior experience. Both Vanilla Minecraft and most mods use data in their implementation, meaning that they can be modified via datapacks. Common applications of data in vanilla/mods are recipes, tags, advancements, and worldgen, meaning that these elements of the game can be added, removed, and modified with datapacks. Making a datapack 1. Create a folder This can be done by right clicking your desktop or inside another directory, and pressing \"New\" \"Folder\". You can also press CTRL + Shift + N as a shortcut. You can name this folder anything. 2. Open the folder and create the data folder and pack.mcmeta file After you've opened the folder you just created, create another folder inside of it named data. Then, create a file named pack.mcmeta by right clicking, pressing \"New\" \"Text Document\", then renaming the entire thing to pack.mcmeta, including the file extension. Your screen should look similar to the image below. :::tip Enabling file extensions is a must have when doing nearly anything modpack related. On Windows, they can be turned on by going to Settings System Advanced File Explorer, then clicking the \"Show File Extensions\" tick. ::: 3. Put information into the pack.mcmeta file pack.mcmeta files include information on what Minecraft version a datapack is compatible for, among other things. To supply the correct information on what to include in a datapack, use Misode's pack.mcmeta generator site to create the contents of the file. The only thing that matters here is the pack format field, which should be 15 if the datapack is made for Minecraft version 1.20.1, and 48 if made for Minecraft version 1.21.1. Other pack formats for different versions can be found here. pack.mcmeta 1.20.1 pack.mcmeta 1.21.1 4. Create a \"Namespace\" folder inside data folder A Namespace determines \"who\" a set of data belongs to in a datapack. If you're adding custom content with a datapack, and not editing anything from the base game or another mod, you should create your own namespace, ex; my namespace. But if you're modifying files in vanilla or in other mods, you should use theirs instead, otherwise they wouldn't be overridden correctly, ex; minecraft, oreganized. For the purposes of this guide, I will be doing all three. 5. Create \"data type\" folders inside of the namespace folders The data type folders determine what \"kind\" of data is to be created/modified. For the purposes of this example, we will be changing recipes. On 1.20.1 and below, the data type is called recipes, while on 1.21.1 and above, it's called recipe. To find the correct data type for what you want to override/create, there are a couple options. For vanilla Minecraft, you can use sites like MCasset or Misode to see the default data for the game. For mods, you can either view their source code (usually provided on their Curseforge / Modrinth sites, though not always) or open the mod .jar file in your modpack /mods directory with a program such as WinRAR or 7 Zip. Oreganized mod page Oreganized source 6. Create the data file Once you've figured out exactly what you want to change by looking at the base data, you can override it by copying the exact filepath presented. In this case, we will be doing the following: Changing Minecraft's \"Stick\" recipe Changing Oreganized's \"Lead Bolt\" recipe Adding a new recipe converting diamonds to dirt my datapack/data/minecraft/recipe/stick.json my datapack/data/oreganized/recipe/lead bolt.json my datapack/data/my namespace/recipe/dirt to diamond.json :::tip You can use Misode's datapack generator to easily create data files using vanilla data types ::: 7. Zip the datapack (optional) Once you've created the datapack, you can zip/compress it to make it easier to share standalone, or leave it as a folder. To zip the file, select both the data folder and pack.mcmeta file in the original folder, right click compress to ZIP file. Zipped files are easier to share but harder to edit, so if you're going to actively be making changes to the datapack, you may want to leave it as a folder. 8. Loading the datapack To load datapacks, ordinarily you'd have to manually place the file into your worlds datapacks folder, and it would not persist between worlds. However, in a modded environment, there are mods such as Open Loader and Paxi that automatically load datapacks placed in their config folder, or in other directories such as the /datapacks directory that many launchers support. Additionally, KubeJS can act as a datapack loader, with /kubejs/data/... loading data files put in it. Data loaded through KubeJS will have higher priority than ones loaded through Open Loader or Paxi. Related topics: Data Loading Conditions",
    "url": "/guide/intro/intro-datapack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 1. Create a folder",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "1. Create a folder",
    "description": "Introduction and tutorial for datapacks",
    "content": "This can be done by right clicking your desktop or inside another directory, and pressing \"New\" \"Folder\". You can also press CTRL + Shift + N as a shortcut. You can name this folder anything.",
    "url": "/guide/intro/intro-datapack#1-create-a-folder",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 2. Open the folder and create the data folder and pack.mcmeta file",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "2. Open the folder and create the data folder and pack.mcmeta file",
    "description": "Introduction and tutorial for datapacks",
    "content": "After you've opened the folder you just created, create another folder inside of it named data. Then, create a file named pack.mcmeta by right clicking, pressing \"New\" \"Text Document\", then renaming the entire thing to pack.mcmeta, including the file extension. Your screen should look similar to the image below. :::tip Enabling file extensions is a must have when doing nearly anything modpack related. On Windows, they can be turned on by going to Settings System Advanced File Explorer, then clicking the \"Show File Extensions\" tick. :::",
    "url": "/guide/intro/intro-datapack#2-open-the-folder-and-create-the-data-folder-and-packmcmeta-file",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 3. Put information into the pack.mcmeta file",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "3. Put information into the pack.mcmeta file",
    "description": "Introduction and tutorial for datapacks",
    "content": "pack.mcmeta files include information on what Minecraft version a datapack is compatible for, among other things. To supply the correct information on what to include in a datapack, use Misode's pack.mcmeta generator site to create the contents of the file. The only thing that matters here is the pack format field, which should be 15 if the datapack is made for Minecraft version 1.20.1, and 48 if made for Minecraft version 1.21.1. Other pack formats for different versions can be found here. pack.mcmeta 1.20.1 pack.mcmeta 1.21.1",
    "url": "/guide/intro/intro-datapack#3-put-information-into-the-packmcmeta-file",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 4. Create a \"Namespace\" folder inside data folder",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "4. Create a \"Namespace\" folder inside data folder",
    "description": "Introduction and tutorial for datapacks",
    "content": "A Namespace determines \"who\" a set of data belongs to in a datapack. If you're adding custom content with a datapack, and not editing anything from the base game or another mod, you should create your own namespace, ex; my namespace. But if you're modifying files in vanilla or in other mods, you should use theirs instead, otherwise they wouldn't be overridden correctly, ex; minecraft, oreganized. For the purposes of this guide, I will be doing all three.",
    "url": "/guide/intro/intro-datapack#4-create-a-namespace-folder-inside-data-folder",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 5. Create \"data type\" folders inside of the namespace folders",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "5. Create \"data type\" folders inside of the namespace folders",
    "description": "Introduction and tutorial for datapacks",
    "content": "The data type folders determine what \"kind\" of data is to be created/modified. For the purposes of this example, we will be changing recipes. On 1.20.1 and below, the data type is called recipes, while on 1.21.1 and above, it's called recipe. To find the correct data type for what you want to override/create, there are a couple options. For vanilla Minecraft, you can use sites like MCasset or Misode to see the default data for the game. For mods, you can either view their source code (usually provided on their Curseforge / Modrinth sites, though not always) or open the mod .jar file in your modpack /mods directory with a program such as WinRAR or 7 Zip. Oreganized mod page Oreganized source",
    "url": "/guide/intro/intro-datapack#5-create-data-type-folders-inside-of-the-namespace-folders",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 6. Create the data file",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "6. Create the data file",
    "description": "Introduction and tutorial for datapacks",
    "content": "Once you've figured out exactly what you want to change by looking at the base data, you can override it by copying the exact filepath presented. In this case, we will be doing the following: Changing Minecraft's \"Stick\" recipe Changing Oreganized's \"Lead Bolt\" recipe Adding a new recipe converting diamonds to dirt my datapack/data/minecraft/recipe/stick.json my datapack/data/oreganized/recipe/lead bolt.json my datapack/data/my namespace/recipe/dirt to diamond.json :::tip You can use Misode's datapack generator to easily create data files using vanilla data types :::",
    "url": "/guide/intro/intro-datapack#6-create-the-data-file",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 7. Zip the datapack (optional)",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "7. Zip the datapack (optional)",
    "description": "Introduction and tutorial for datapacks",
    "content": "Once you've created the datapack, you can zip/compress it to make it easier to share standalone, or leave it as a folder. To zip the file, select both the data folder and pack.mcmeta file in the original folder, right click compress to ZIP file. Zipped files are easier to share but harder to edit, so if you're going to actively be making changes to the datapack, you may want to leave it as a folder.",
    "url": "/guide/intro/intro-datapack#7-zip-the-datapack-optional",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / 8. Loading the datapack",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "8. Loading the datapack",
    "description": "Introduction and tutorial for datapacks",
    "content": "To load datapacks, ordinarily you'd have to manually place the file into your worlds datapacks folder, and it would not persist between worlds. However, in a modded environment, there are mods such as Open Loader and Paxi that automatically load datapacks placed in their config folder, or in other directories such as the /datapacks directory that many launchers support. Additionally, KubeJS can act as a datapack loader, with /kubejs/data/... loading data files put in it. Data loaded through KubeJS will have higher priority than ones loaded through Open Loader or Paxi.",
    "url": "/guide/intro/intro-datapack#8-loading-the-datapack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / Making a datapack",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "Making a datapack",
    "description": "Introduction and tutorial for datapacks",
    "content": "Making a datapack",
    "url": "/guide/intro/intro-datapack#making-a-datapack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Datapacks / Related topics:",
    "pageTitle": "Intro to Datapacks",
    "sectionTitle": "Related topics:",
    "description": "Introduction and tutorial for datapacks",
    "content": "Data Loading Conditions",
    "url": "/guide/intro/intro-datapack#related-topics",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Introduction to modpack development",
    "pageTitle": "Introduction to modpack development",
    "sectionTitle": "",
    "description": "The very basics for modpack development",
    "content": "Introduction to modpack development New to packdev? This article will serve as an introduction to getting started with modpack development, covering introductory topics such as launchers, versions, modloaders, logs, and datapacks. Launchers To play modded Minecraft, you first need a launcher . Launchers are what allow you to easily add/remove mods from your game, as well as create different instances containing different mods and configurations. Using the Vanilla game launcher to run mods, while sometimes possible, can run into issues, and it a lot less convenient than using any of the options listed below. Launcher Where mods are downloaded from Additional Notes Curseforge Curseforge Recommended to download without Overwolf Prism Curseforge, Modrinth Has access to both CF and Modrinth Modrinth Modrinth Only has access to Modrinth ATlauncher :idk: :idk: While launchers have their differences, features, and issues, they are mainly up to personal preference. Versions and Modloaders While launchers are how you start the game, modloaders are what platform the mods you're using run on. Mods made on Forge generally cannot run on Fabric, and vice versa. While there are many loaders out there today, only three are relevant for modern modpack development; Forge, Neoforge, and Fabric. Version Loaders 1.20.1 Forge, Fabric 1.21.1 NeoForge, Fabric :::info The performance of a modpack is attributed almost entirely to the content of the mods it has, not the loader. Different modloaders are not inherently more/less performant for running mods than each other. ::: Logs",
    "url": "/guide/intro/intro-intro",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Introduction to modpack development / Launchers",
    "pageTitle": "Introduction to modpack development",
    "sectionTitle": "Launchers",
    "description": "The very basics for modpack development",
    "content": "To play modded Minecraft, you first need a launcher . Launchers are what allow you to easily add/remove mods from your game, as well as create different instances containing different mods and configurations. Using the Vanilla game launcher to run mods, while sometimes possible, can run into issues, and it a lot less convenient than using any of the options listed below. Launcher Where mods are downloaded from Additional Notes Curseforge Curseforge Recommended to download without Overwolf Prism Curseforge, Modrinth Has access to both CF and Modrinth Modrinth Modrinth Only has access to Modrinth ATlauncher :idk: :idk: While launchers have their differences, features, and issues, they are mainly up to personal preference.",
    "url": "/guide/intro/intro-intro#launchers",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Introduction to modpack development / Logs",
    "pageTitle": "Introduction to modpack development",
    "sectionTitle": "Logs",
    "description": "The very basics for modpack development",
    "content": "Logs",
    "url": "/guide/intro/intro-intro#logs",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Introduction to modpack development / Versions and Modloaders",
    "pageTitle": "Introduction to modpack development",
    "sectionTitle": "Versions and Modloaders",
    "description": "The very basics for modpack development",
    "content": "While launchers are how you start the game, modloaders are what platform the mods you're using run on. Mods made on Forge generally cannot run on Fabric, and vice versa. While there are many loaders out there today, only three are relevant for modern modpack development; Forge, Neoforge, and Fabric. Version Loaders 1.20.1 Forge, Fabric 1.21.1 NeoForge, Fabric :::info The performance of a modpack is attributed almost entirely to the content of the mods it has, not the loader. Different modloaders are not inherently more/less performant for running mods than each other. :::",
    "url": "/guide/intro/intro-intro#versions-and-modloaders",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Intro to Mopdpacks",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "",
    "description": "Introduction and tutorial for modpacks",
    "content": "Intro to Modpacks Loosely defined, a modpack is a collection of Minecraft mods that players can download and play. Anyone can put one together, and upload it to CurseForge and Modrinth with little resistance. While this wiki focuses on doing a bit more than just throwing your favorite mods into a pack and calling it a day, you have to learn the basics somewhere! This guide is best for those not 100% familiar with the process, and will get you started. You should aim to follow this guide with a basic pack idea if it's your first time making a modpack. If you want to do something more ambitious, treat it as a trial run and then afterwards go over the more in depth planning section of the wiki. Making a modpack 1. Create an instance in your launcher To start, you'll want to create a fresh instance to develop on. There are a few different launchers to pick from, which you can find here, but this tutorial will work on the assumption that you're using Prism. Pressing the \"Add Instance\" button in the top left will bring you to the version and loader selection screen. This is likely the most important choice you will make in the pack dev process as it will determine some extremely important aspects of development: What mods are available to you How supported those mods are by devs some mods support multiple versions, some only the newest Technical support from others popular and evergreen versions will have more people able to help What audience your pack will bring Some types of players will prefer older packs, while vanilla players might prefer the latest versions of Minecraft If you're dedicated enough to modded Minecraft you'll likely know what to pick, but if not just start off with whatever version is supported by your most important mod. 2. Select mods With your fresh new instance, you can start adding mods to your pack. It's best to start off with the standard performance and utility mods for your version. From there, add your core and complimentary mods, occasionally launching the issue to see if there are any crashes or easily apparent issues. The Mod Selection page contains detailed information on the mod selection process. It's another incredibly important part of the pack dev process, so be sure to take time here to really consider what mods you are adding to the pack. Less can be more! 3. Configuring mods This step can be done while adding mods or after your mod list is stable, whatever works best for your workflow. It involves digging through the config directory of your instance, as well as the serverconfigs folder of new worlds. :::tip The defaultconfigs directory can contain any configuration files in the serverconfig folder that are made when you create a world (or join a world with the mod freshly installed/updated). Copy and paste files from serverconfigs to defaultconfigs to allow your changes to be added to the world whenever a new world is created ::: Every config file of every mod should be scanned and tweaked according to your pack's goals. Quark as an example, has an extensive configuration (with an in game ui) that allows you to completely remove many of its additions. In some cases, this will directly tie to the balance of your pack. If you're making a technical pack for example, some tech mods may include power consumption and speed configs. Be aware that updating mods may change config files! New ones can be added and existing ones can be taken away, so be very careful when updating mods! 4. Making additional changes Datapacks In modern versions, mod devs have gradually adopted Mojang's data driven approach to configuring mods. If you don't know what that means, it basically boils down to modpack developers having more control over things like worldgen, recipes, and other features! Making datapack changes is a bit involved, but once you learn the basics it will give you a ton of power when making changes to your pack. For example: Have two identical ores generating in the world? Datapack the one you don't like away Want to add custom recipes for a mod's strange machine? Create a bunch of recipes using datapacks Need to prevent a mob from dropping an item? Override its loot table using datapacks You can find the full tutorial on Intro to Datapacks. Custom recipes No matter what kind of modpack you are making, you will have to change some recipes. Sometimes this is for balance (get rid of a mod's coal to diamond recipe), or for unification (two different recipes craft the same thing). The Polymorph mod can be used to easily find recipe conflicts using /polymorph conflicts, outputting them to the logs folder. You should avoid having the mod in your final release for performance if you have the time to fix them all. The primary way you modify recipes is datapacks, though KubeJS has some useful utilities to make this faster. 5. Sharing your pack With the modpack created and playtesting done, you can now share the pack with friends and strangers! This can be done using any launcher's export feature. In Prism, you can right click the instance, then click export, selecting which platform to export to. Make sure to include any folders that you've made changes to. Usually this means: config, defaultconfigs, kubejs, mods, and resourcepacks. Before uploading, import your pack into your launcher and run the pack to see if everything has exported properly and the experience is exactly how you want players to have. From there, you can safely upload your pack to your platform and show it off to the world once its approved! :::warning Read up on a platform's posting rules before uploading your modpack! Your pack can be rejected or taken down if it includes mods, resourcepacks, or datapacks not on the platform. Double check the export file for any override mods to correct them so mod devs get credit for their work. ::: You can find more information on this on the CurseForge and Modrinth pages.",
    "url": "/guide/intro/intro-modpack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / 1. Create an instance in your launcher",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "1. Create an instance in your launcher",
    "description": "Introduction and tutorial for modpacks",
    "content": "To start, you'll want to create a fresh instance to develop on. There are a few different launchers to pick from, which you can find here, but this tutorial will work on the assumption that you're using Prism. Pressing the \"Add Instance\" button in the top left will bring you to the version and loader selection screen. This is likely the most important choice you will make in the pack dev process as it will determine some extremely important aspects of development: What mods are available to you How supported those mods are by devs some mods support multiple versions, some only the newest Technical support from others popular and evergreen versions will have more people able to help What audience your pack will bring Some types of players will prefer older packs, while vanilla players might prefer the latest versions of Minecraft If you're dedicated enough to modded Minecraft you'll likely know what to pick, but if not just start off with whatever version is supported by your most important mod.",
    "url": "/guide/intro/intro-modpack#1-create-an-instance-in-your-launcher",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / 2. Select mods",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "2. Select mods",
    "description": "Introduction and tutorial for modpacks",
    "content": "With your fresh new instance, you can start adding mods to your pack. It's best to start off with the standard performance and utility mods for your version. From there, add your core and complimentary mods, occasionally launching the issue to see if there are any crashes or easily apparent issues. The Mod Selection page contains detailed information on the mod selection process. It's another incredibly important part of the pack dev process, so be sure to take time here to really consider what mods you are adding to the pack. Less can be more!",
    "url": "/guide/intro/intro-modpack#2-select-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / 3. Configuring mods",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "3. Configuring mods",
    "description": "Introduction and tutorial for modpacks",
    "content": "This step can be done while adding mods or after your mod list is stable, whatever works best for your workflow. It involves digging through the config directory of your instance, as well as the serverconfigs folder of new worlds. :::tip The defaultconfigs directory can contain any configuration files in the serverconfig folder that are made when you create a world (or join a world with the mod freshly installed/updated). Copy and paste files from serverconfigs to defaultconfigs to allow your changes to be added to the world whenever a new world is created ::: Every config file of every mod should be scanned and tweaked according to your pack's goals. Quark as an example, has an extensive configuration (with an in game ui) that allows you to completely remove many of its additions. In some cases, this will directly tie to the balance of your pack. If you're making a technical pack for example, some tech mods may include power consumption and speed configs. Be aware that updating mods may change config files! New ones can be added and existing ones can be taken away, so be very careful when updating mods!",
    "url": "/guide/intro/intro-modpack#3-configuring-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / 4. Making additional changes",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "4. Making additional changes",
    "description": "Introduction and tutorial for modpacks",
    "content": "4. Making additional changes",
    "url": "/guide/intro/intro-modpack#4-making-additional-changes",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / 5. Sharing your pack",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "5. Sharing your pack",
    "description": "Introduction and tutorial for modpacks",
    "content": "With the modpack created and playtesting done, you can now share the pack with friends and strangers! This can be done using any launcher's export feature. In Prism, you can right click the instance, then click export, selecting which platform to export to. Make sure to include any folders that you've made changes to. Usually this means: config, defaultconfigs, kubejs, mods, and resourcepacks. Before uploading, import your pack into your launcher and run the pack to see if everything has exported properly and the experience is exactly how you want players to have. From there, you can safely upload your pack to your platform and show it off to the world once its approved! :::warning Read up on a platform's posting rules before uploading your modpack! Your pack can be rejected or taken down if it includes mods, resourcepacks, or datapacks not on the platform. Double check the export file for any override mods to correct them so mod devs get credit for their work. ::: You can find more information on this on the CurseForge and Modrinth pages.",
    "url": "/guide/intro/intro-modpack#5-sharing-your-pack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / Custom recipes",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "Custom recipes",
    "description": "Introduction and tutorial for modpacks",
    "content": "No matter what kind of modpack you are making, you will have to change some recipes. Sometimes this is for balance (get rid of a mod's coal to diamond recipe), or for unification (two different recipes craft the same thing). The Polymorph mod can be used to easily find recipe conflicts using /polymorph conflicts, outputting them to the logs folder. You should avoid having the mod in your final release for performance if you have the time to fix them all. The primary way you modify recipes is datapacks, though KubeJS has some useful utilities to make this faster.",
    "url": "/guide/intro/intro-modpack#custom-recipes",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / Datapacks",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "Datapacks",
    "description": "Introduction and tutorial for modpacks",
    "content": "In modern versions, mod devs have gradually adopted Mojang's data driven approach to configuring mods. If you don't know what that means, it basically boils down to modpack developers having more control over things like worldgen, recipes, and other features! Making datapack changes is a bit involved, but once you learn the basics it will give you a ton of power when making changes to your pack. For example: Have two identical ores generating in the world? Datapack the one you don't like away Want to add custom recipes for a mod's strange machine? Create a bunch of recipes using datapacks Need to prevent a mob from dropping an item? Override its loot table using datapacks You can find the full tutorial on Intro to Datapacks.",
    "url": "/guide/intro/intro-modpack#datapacks",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Mopdpacks / Making a modpack",
    "pageTitle": "Intro to Mopdpacks",
    "sectionTitle": "Making a modpack",
    "description": "Introduction and tutorial for modpacks",
    "content": "Making a modpack",
    "url": "/guide/intro/intro-modpack#making-a-modpack",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Intro to Resource Packs",
    "pageTitle": "Intro to Resource Packs",
    "sectionTitle": "",
    "description": "Introduction and tutorial for resource packs",
    "content": "Intro to Resource Packs A resource pack is a collection of assets stored in a folder or .zip file used to modify how elements of the game, such as blocks, items, and GUIs look. They mainly consist of textures, models, and lang. Making a Resource Pack 1. Create a folder This can be done by right clicking your desktop or inside another directory, and pressing \"New\" \"Folder\". You can also press CTRL + Shift + N as a shortcut. You can name this folder anything.",
    "url": "/guide/intro/intro-resourcepack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Resource Packs / 1. Create a folder",
    "pageTitle": "Intro to Resource Packs",
    "sectionTitle": "1. Create a folder",
    "description": "Introduction and tutorial for resource packs",
    "content": "This can be done by right clicking your desktop or inside another directory, and pressing \"New\" \"Folder\". You can also press CTRL + Shift + N as a shortcut. You can name this folder anything.",
    "url": "/guide/intro/intro-resourcepack#1-create-a-folder",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Intro to Resource Packs / Making a Resource Pack",
    "pageTitle": "Intro to Resource Packs",
    "sectionTitle": "Making a Resource Pack",
    "description": "Introduction and tutorial for resource packs",
    "content": "Making a Resource Pack",
    "url": "/guide/intro/intro-resourcepack#making-a-resource-pack",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Minecraft Concepts",
    "pageTitle": "Minecraft Concepts",
    "sectionTitle": "",
    "description": "How is Minecraft relevant to Minecraft Modpacks?",
    "content": "Minecraft Concepts No matter what type of pack you are creating, you will be building upon the game that is Minecraft. Your ability to design your gameplay and make your modpack will directly correspond to how well you know the game itself inside and out. General Concepts The Minecraft Wiki is the best resource for the base general concepts of Minecraft, so to avoid repeating information here is a shortlist of pages to be aware of: Ticks A cycle of the game loop. This is important for optimizing performance of your pack. JSON The file format used in data and resource packs. Very important to learn and fairly human readable. NBT How arbitrary data is stored in blocks/items/saves. This is partially replaced by Data Components in newer versions. Data Components (1.21.1+) Arbitrary data for items and partially entities. Tags) Groupings of items, blocks, entities, biomes, and more. Block Entity Blocks that do things such as a furnace. Block States Data that primarily controls block appearance such as rotation. :::tip These links are best used as references! There's no need to dig into every detail before you mack modpacks, but these concepts will come up at some point. ::: Mod Loader concepts The Mod Loader of your pack is a larger piece of software that allows Minecraft mods to run on Minecraft. The most commonly used ones are NeoForge (Forge on 1.20.1 and below) and Fabric. Since they add their own code that \"hooks\" into Minecraft, it's important to have an awareness of them in your modpack. Similar to versions of Minecraft, you can only run mods that are built with a mod loader in mind. Events Events are pieces of code triggered when something happens in the game. These systems are loader specific, and very useful for pack developers to add custom content to. For example: Event: Player right clicks with item Give them a random item and delete the item Event: Player loads into the world Send a message to the player about your latest updated These are most easily used by KubeJs to create custom behaviors in your modpack. NeoForge documentation Fabric Documentation Version Mixing As mentioned above, you generally can't run mods that run on one loader on a different one. There is a recent exception to this rule with the mod Sinytra Connector that allows Fabric mods to run on Forge and NeoForge. It is generally not recommended to use Sinytra Connector in modpacks. It is still in active development, and has many incompatibilities that will make your modpack less stable and harder to debug.",
    "url": "/guide/intro/minecraft-concepts",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Minecraft Concepts / Events",
    "pageTitle": "Minecraft Concepts",
    "sectionTitle": "Events",
    "description": "How is Minecraft relevant to Minecraft Modpacks?",
    "content": "Events are pieces of code triggered when something happens in the game. These systems are loader specific, and very useful for pack developers to add custom content to. For example: Event: Player right clicks with item Give them a random item and delete the item Event: Player loads into the world Send a message to the player about your latest updated These are most easily used by KubeJs to create custom behaviors in your modpack. NeoForge documentation Fabric Documentation",
    "url": "/guide/intro/minecraft-concepts#events",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Minecraft Concepts / General Concepts",
    "pageTitle": "Minecraft Concepts",
    "sectionTitle": "General Concepts",
    "description": "How is Minecraft relevant to Minecraft Modpacks?",
    "content": "The Minecraft Wiki is the best resource for the base general concepts of Minecraft, so to avoid repeating information here is a shortlist of pages to be aware of: Ticks A cycle of the game loop. This is important for optimizing performance of your pack. JSON The file format used in data and resource packs. Very important to learn and fairly human readable. NBT How arbitrary data is stored in blocks/items/saves. This is partially replaced by Data Components in newer versions. Data Components (1.21.1+) Arbitrary data for items and partially entities. Tags) Groupings of items, blocks, entities, biomes, and more. Block Entity Blocks that do things such as a furnace. Block States Data that primarily controls block appearance such as rotation. :::tip These links are best used as references! There's no need to dig into every detail before you mack modpacks, but these concepts will come up at some point. :::",
    "url": "/guide/intro/minecraft-concepts#general-concepts",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Minecraft Concepts / Mod Loader concepts",
    "pageTitle": "Minecraft Concepts",
    "sectionTitle": "Mod Loader concepts",
    "description": "How is Minecraft relevant to Minecraft Modpacks?",
    "content": "The Mod Loader of your pack is a larger piece of software that allows Minecraft mods to run on Minecraft. The most commonly used ones are NeoForge (Forge on 1.20.1 and below) and Fabric. Since they add their own code that \"hooks\" into Minecraft, it's important to have an awareness of them in your modpack. Similar to versions of Minecraft, you can only run mods that are built with a mod loader in mind.",
    "url": "/guide/intro/minecraft-concepts#mod-loader-concepts",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Minecraft Concepts / Version Mixing",
    "pageTitle": "Minecraft Concepts",
    "sectionTitle": "Version Mixing",
    "description": "How is Minecraft relevant to Minecraft Modpacks?",
    "content": "As mentioned above, you generally can't run mods that run on one loader on a different one. There is a recent exception to this rule with the mod Sinytra Connector that allows Fabric mods to run on Forge and NeoForge. It is generally not recommended to use Sinytra Connector in modpacks. It is still in active development, and has many incompatibilities that will make your modpack less stable and harder to debug.",
    "url": "/guide/intro/minecraft-concepts#version-mixing",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Improving and Profiling Modpack Performance",
    "pageTitle": "Improving and Profiling Modpack Performance",
    "sectionTitle": "",
    "description": "How to improve performance in your modpack",
    "content": "Improving Modpack Performance Performance is one of the most important aspects of modpack development! You don't want people to be unable to play something you've made due to issues with optimization. Below is a guide on how to optimize and profile your modpack to keep performance stable. 1. Before you profile a laggy pack Before you bother taking profilers of your pack, you should check the following: How much RAM is being assigned to the modpack Most packs should take between 4 10GB RAM to run, depending on how much content you have. Additionally, you shouldn't assign more than half your machine's total RAM to a modpack, as over assigning memory can lead to issues. Your machine specs Make sure your graphics drivers are up to date and being used by your instance If you're unsure on how to do this, ask your preferred search engine \"How to update graphics drivers for \\ \". Make sure you have relevant performance mods installed if you're unsure on what to install, read the guide below. 2. Install performance mods Installing performance mods is the easiest way to improve performance! Our Our Useful Mods List provides a competent selection of performance mods for versions/loaders you're likely to make a modpack in, and we recommend to check it out for mods to add to your modpacks, especially ones marked with an . :::warning It is your responsibility as a modpack developer to ensure all mods in your pack work together. We recommend against adding every mod listed on the page at once if you lack the skills to debug issues that may arise from it, even if these mods are curated for compatibility and performance. We cannot guarantee that no issues or crashes may arise from using any of these mods. ::: When adding performance mods to your pack, you should watch out for mods that have these red flags: Don't have source listed Never use closed source performance mods Don't provide benchmarks If a mod doesn't tell you how much the mod affects performance with a defined setup, it probably isn't worth using High amount of open issues on issue tracker compared to closed ones If a developer isn't on top of issues that the mod has, it may be buggy or unperformant Small number of downloads If a mod has a small download count, it may be unstable since issues or incompatibilities have not been discovered and reported yet 3. Taking a Profiler There are many issues in modpacks that can be diagnosed via profilers . A profiler is a collection of information of your game instance that can be used to pinpoint sources of issues. Depending on the issue at hand, you'll need to take different profilers to find the root cause. For this guide, we recommend installing Spark and Modernfix for the majority of profiling usages. Issue Type Server Client Startup Memory / GC Common Symptoms Ghost blocks, lag spikes, Mobs moving irregularly FPS drops Long startup time WIP Profiler Method Spark Server Profiler, Modernfix MCfunctions profiler. Spark Client Profiler Spark + Modernfix Startup Profiler WIP Server Profiler One of the most common areas of lag in modpacks is server lag.",
    "url": "/guide/performance",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Improving and Profiling Modpack Performance / 1. Before you profile a laggy pack",
    "pageTitle": "Improving and Profiling Modpack Performance",
    "sectionTitle": "1. Before you profile a laggy pack",
    "description": "How to improve performance in your modpack",
    "content": "Before you bother taking profilers of your pack, you should check the following: How much RAM is being assigned to the modpack Most packs should take between 4 10GB RAM to run, depending on how much content you have. Additionally, you shouldn't assign more than half your machine's total RAM to a modpack, as over assigning memory can lead to issues. Your machine specs Make sure your graphics drivers are up to date and being used by your instance If you're unsure on how to do this, ask your preferred search engine \"How to update graphics drivers for \\ \". Make sure you have relevant performance mods installed if you're unsure on what to install, read the guide below.",
    "url": "/guide/performance#1-before-you-profile-a-laggy-pack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Improving and Profiling Modpack Performance / 2. Install performance mods",
    "pageTitle": "Improving and Profiling Modpack Performance",
    "sectionTitle": "2. Install performance mods",
    "description": "How to improve performance in your modpack",
    "content": "Installing performance mods is the easiest way to improve performance! Our Our Useful Mods List provides a competent selection of performance mods for versions/loaders you're likely to make a modpack in, and we recommend to check it out for mods to add to your modpacks, especially ones marked with an . :::warning It is your responsibility as a modpack developer to ensure all mods in your pack work together. We recommend against adding every mod listed on the page at once if you lack the skills to debug issues that may arise from it, even if these mods are curated for compatibility and performance. We cannot guarantee that no issues or crashes may arise from using any of these mods. ::: When adding performance mods to your pack, you should watch out for mods that have these red flags: Don't have source listed Never use closed source performance mods Don't provide benchmarks If a mod doesn't tell you how much the mod affects performance with a defined setup, it probably isn't worth using High amount of open issues on issue tracker compared to closed ones If a developer isn't on top of issues that the mod has, it may be buggy or unperformant Small number of downloads If a mod has a small download count, it may be unstable since issues or incompatibilities have not been discovered and reported yet",
    "url": "/guide/performance#2-install-performance-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Improving and Profiling Modpack Performance / 3. Taking a Profiler",
    "pageTitle": "Improving and Profiling Modpack Performance",
    "sectionTitle": "3. Taking a Profiler",
    "description": "How to improve performance in your modpack",
    "content": "There are many issues in modpacks that can be diagnosed via profilers . A profiler is a collection of information of your game instance that can be used to pinpoint sources of issues. Depending on the issue at hand, you'll need to take different profilers to find the root cause. For this guide, we recommend installing Spark and Modernfix for the majority of profiling usages. Issue Type Server Client Startup Memory / GC Common Symptoms Ghost blocks, lag spikes, Mobs moving irregularly FPS drops Long startup time WIP Profiler Method Spark Server Profiler, Modernfix MCfunctions profiler. Spark Client Profiler Spark + Modernfix Startup Profiler WIP",
    "url": "/guide/performance#3-taking-a-profiler",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Improving and Profiling Modpack Performance / Server Profiler",
    "pageTitle": "Improving and Profiling Modpack Performance",
    "sectionTitle": "Server Profiler",
    "description": "How to improve performance in your modpack",
    "content": "One of the most common areas of lag in modpacks is server lag.",
    "url": "/guide/performance#server-profiler",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Removing Blocks",
    "pageTitle": "Removing Blocks",
    "sectionTitle": "",
    "description": "Strategies for removing naturally generated or unwanted blocks from a pack.",
    "content": "Removing Blocks Removing blocks from world generation is usually best done at the source of generation rather than by blunt post processing. If a block comes from a feature, biome modifier, or datapack driven generation rule, remove or replace that generation rule directly. If you only replace the finished block afterward, you can create air pockets, broken shapes, or other artifacts. Reliable Replacer Reliable Replacer can replace existing blocks through JSON configuration and is useful when direct generation edits are not practical. json title=\"config/reliable replacer/swapper.json\" { \"swapper\": { \"oreganized:lead door\": \"supplementaries:netherite door\", \"farmersdelight:rope\": \"supplementaries:rope\", \"minecraft:dirt\": \"minecraft:stone\" } } Prefer generation level fixes when possible For feature based generation, prefer removing or editing the feature itself. That keeps terrain logic predictable and avoids broad replacement side effects.",
    "url": "/guide/removals/removing-blocks",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Removing Blocks / Prefer generation-level fixes when possible",
    "pageTitle": "Removing Blocks",
    "sectionTitle": "Prefer generation-level fixes when possible",
    "description": "Strategies for removing naturally generated or unwanted blocks from a pack.",
    "content": "For feature based generation, prefer removing or editing the feature itself. That keeps terrain logic predictable and avoids broad replacement side effects.",
    "url": "/guide/removals/removing-blocks#prefer-generation-level-fixes-when-possible",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Removing Blocks / Reliable Replacer",
    "pageTitle": "Removing Blocks",
    "sectionTitle": "Reliable Replacer",
    "description": "Strategies for removing naturally generated or unwanted blocks from a pack.",
    "content": "Reliable Replacer can replace existing blocks through JSON configuration and is useful when direct generation edits are not practical. json title=\"config/reliable replacer/swapper.json\" { \"swapper\": { \"oreganized:lead door\": \"supplementaries:netherite door\", \"farmersdelight:rope\": \"supplementaries:rope\", \"minecraft:dirt\": \"minecraft:stone\" } }",
    "url": "/guide/removals/removing-blocks#reliable-replacer",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Removing items",
    "pageTitle": "Removing items",
    "sectionTitle": "",
    "description": "How to remove items from being obtained or used in a modpack",
    "content": "Removing items from the game Adding content to your modpack is great, but sometimes mods can either add things that just don't fit into your vision for the pack, or maybe just have overlap with another mod you have installed. This article will go into some methods of streamlining your pack and removing items from your pack. Some mods that're useful for removing items from your game are Reliable Remover, KubeJS, and Registry Blocker. Reliable Remover Reliable Remover is a simple json based tool that can remove the functionality and obtainment methods for various items. More details of the mods functionality, as well as examples of usages can be found on its wiki. :::tip Running the in game commands /rremover hand, /rremover hotbar, and /rremover inventory are quick ways to get the IDs to put into the array. ::: KubeJS This KubeJS script removes everything in the global.nukelist array from all tags, recipes, and recipe viewers. Additional functionality can be achieved if LootJS is installed, granting it the ability to remove items from many types of loot tables. :::warning The following script only works for 1.20.1 ::: The following can be put in client scripts to add a tooltip notifying the user that an item has been removed, and to report the issue to the modpack developers in cases of the nukelist not being thorough. :::tip Running the in game commands /kubejs hand, /kubejs hotbar, and /kubejs inventory are quick ways to get the IDs to put into the array. ::: :::info Note that neither Reliable Remover nor KubeJS fully removes an item from being registered in game, and only attempts to remove methods of obtaining said item. ::: Registry Blocker Registry Blocker is a mod that blocks registries. It is as invasive and destructive as it sounds. Instructions on how to use it is documented on the mod page. :::warning Messing with the game registry with mods such as Registry Blocker is unsafe, and may lead to issues such as data validation errors, log spam, and even crashes. It should only be considered as a last resort if neither a mods config, KubeJS, Reliable Remover, or datapacks work to remove something. :::",
    "url": "/guide/removals/removing-items",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Removing items / KubeJS",
    "pageTitle": "Removing items",
    "sectionTitle": "KubeJS",
    "description": "How to remove items from being obtained or used in a modpack",
    "content": "This KubeJS script removes everything in the global.nukelist array from all tags, recipes, and recipe viewers. Additional functionality can be achieved if LootJS is installed, granting it the ability to remove items from many types of loot tables. :::warning The following script only works for 1.20.1 ::: The following can be put in client scripts to add a tooltip notifying the user that an item has been removed, and to report the issue to the modpack developers in cases of the nukelist not being thorough. :::tip Running the in game commands /kubejs hand, /kubejs hotbar, and /kubejs inventory are quick ways to get the IDs to put into the array. ::: :::info Note that neither Reliable Remover nor KubeJS fully removes an item from being registered in game, and only attempts to remove methods of obtaining said item. :::",
    "url": "/guide/removals/removing-items#kubejs",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Removing items / Registry Blocker",
    "pageTitle": "Removing items",
    "sectionTitle": "Registry Blocker",
    "description": "How to remove items from being obtained or used in a modpack",
    "content": "Registry Blocker is a mod that blocks registries. It is as invasive and destructive as it sounds. Instructions on how to use it is documented on the mod page. :::warning Messing with the game registry with mods such as Registry Blocker is unsafe, and may lead to issues such as data validation errors, log spam, and even crashes. It should only be considered as a last resort if neither a mods config, KubeJS, Reliable Remover, or datapacks work to remove something. :::",
    "url": "/guide/removals/removing-items#registry-blocker",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Removing items / Reliable Remover",
    "pageTitle": "Removing items",
    "sectionTitle": "Reliable Remover",
    "description": "How to remove items from being obtained or used in a modpack",
    "content": "Reliable Remover is a simple json based tool that can remove the functionality and obtainment methods for various items. More details of the mods functionality, as well as examples of usages can be found on its wiki. :::tip Running the in game commands /rremover hand, /rremover hotbar, and /rremover inventory are quick ways to get the IDs to put into the array. :::",
    "url": "/guide/removals/removing-items#reliable-remover",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Modifying mob spawns",
    "pageTitle": "Modifying mob spawns",
    "sectionTitle": "",
    "description": "Guide on modifying mob spawns with various methods",
    "content": "Editing mob spawns There are several methods to edit mob spawns, each with varying levels of complexity and control, with the main ones being: Lithostitched Simplest of the three, allows for basic worldgen modifiers to edit spawns In Control! Spawn control with extremely verbose JSON KubeJS skript dat shit bay bee! With Lithostitched Lithostitched can edit mob spawns a datapack adding a worldgen modifier. You have the option of either adding or removing spawns from biomes. Some examples pulled directly from the Lithostitched wiki are found below. Adding spawns: Adds witch spawns to swamps Removing spawns: Removes zombie spawns from the overworld These modifiers can be generated through the Lithostitched generator site. :::info Biome modifiers are also possible through Neo/Forge biome modifiers, but was not included in this page due to brevity and redundancy. Lithostitched covers most biome modifiers Neo/Forge has, but is more versatile and not loader specific. ::: In Control! In Control! uses extremely verbose JSON syntax to edit spawns, and is very useful if you'd like more control over spawns than simply adding mobs to the spawn pool. Additional features over Lithostitched includes, but is not limited to: Game Stages integration Adding gear/NBT to mobs Y level control Day counter control Weather control While it does have a wiki, it is hard to follow at times, and does not include many tangible examples. :::warning In Control is not available on Fabric. ::: (page needs useful IC examples) KubeJS literally noone knows how tf this works at all bruuu",
    "url": "/guide/worldgen/mob-spawns",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Adding biomes to your modpack",
    "pageTitle": "Adding biomes to your modpack",
    "sectionTitle": "",
    "description": "Adding custom biomes to the game",
    "content": "Adding Biomes",
    "url": "/guide/worldgen/modifying-biomes/adding-biomes",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Removing biomes from your modpack",
    "pageTitle": "Removing biomes from your modpack",
    "sectionTitle": "",
    "description": "Removing biomes from the game",
    "content": "Removing Biomes",
    "url": "/guide/worldgen/modifying-biomes/removing-biomes",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Adding worldgen features to your modpack",
    "pageTitle": "Adding worldgen features to your modpack",
    "sectionTitle": "",
    "description": "Adding worldgen features to your game with datapacks and Lithostitched",
    "content": "Adding worldgen features Features need three things to generate in your world: Configured Feature: Determines the type and parameters of the feature you want to generate. Placed Feature: Determines where a Configured Feature should be attempted to be placed using placement modifiers. Worldgen modifier: Determines what biomes the Placed Feature can generate in. Implemented through Lithostitched. Configured features consist of a type and a config field, where type is the kind of feature generated, and config is any customization for the feature. Available config values depend on what the feature type is. For vanilla feature types, you can use Misode's Configured Feature Generator to easily create feature files. For more information on feature types and configs, see the Minecraft Configured Feature Wiki. Example Configured Feature: Defines a feature that replaces minecraft:stone with an ore feature using minecraft:amethyst, with a size of 8 Placed features consist of feature and placement parameters. The feature parameter references a Configured Feature ID, while the placement parameters define where in a biome the feature can spawn. Again, you can use Misode's Placed Feature Generator to assist in making feature files, and see the Minecraft Placed Feature Wiki for more information on placement parameters. Example Placed Feature: Places the amethyst ore:amethyst configured feature between y= 20 and y=60, with a count multiplier of 32 Lithostitched Worldgen Modifiers consist of type, biomes, features, and step parameters. type defines what action the modifier will do, biomes consist of a string or array of biome or biome tag IDs, features consist of a string or array of feature or feature tag IDs, and the step parameter defines what phase of worldgen the feature will generate in. More details on worldgen modifiers from Lithostitched can be found on their wiki. Additionally, Lithostitched has a generator site similar to Misode to aid in production of files. Example Biome Modifier using Lithostitched: Adds the amethyst ore:amethyst placed placed feature into all biomes with the minecraft:is overworld tag :::tip Biome and Biome Tag IDs can be found by going through /locate biome in game, and using the arrow keys to navigate Commonly used biome tags can be found here If KubeJS is installed, you have a variety of options of dumping registries into your latest.log. Run /kubejs dump registry minecraft:worldgen/... in game to see the available options. ::: :::info Biome modifiers are also possible through Neo/Forge biome modifiers, but was not included in this page due to brevity and redundancy. Lithostitched covers most biome modifiers Neo/Forge has, but is more versatile and not loader specific. :::",
    "url": "/guide/worldgen/modifying-features/adding-features",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Removing features from your modpack",
    "pageTitle": "Removing features from your modpack",
    "sectionTitle": "",
    "description": "Removing worldgen features to your game with datapacks and Lithostitched",
    "content": "Removing worldgen features Removing worldgen features is most easily done through worldgen modifiers, such as those provided through Lithostitched. More information on Lithostitched Worldgen Modifiers can be found on their wiki, and can be created using their generator site. Example worldgen modifier Removes Coal Ore generation from the Overworld. :::info Biome modifiers are also possible through Neo/Forge biome modifiers, but was not included in this page due to brevity and redundancy. Lithostitched covers most biome modifiers Neo/Forge has, but is more versatile and not loader specific. :::",
    "url": "/guide/worldgen/modifying-features/removing-features",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Evergreen Version Resources",
    "pageTitle": "Evergreen Version Resources",
    "sectionTitle": "",
    "description": "Resources and communities for packdev on older versions",
    "content": "Evergreen Version Resources While this wiki does not support Minecraft versions below 1.20.1, we are can provide resources for evergreen packdev versions (particularly 1.12.2) that would be useful for anyone making a modpack on those versions! If you develop on older versions, we recommend that you check out these resources, as the rest of this wiki's contents and guides are widely inapplicable to these versions. 1.12.2 Master Doc Compilation of Discord servers, sites, mod lists, tutorials, and other resources tailored towards 1.12.2 Modernized 1.12 Template modpack featuring performance, content, and QoL mods 1.12 Coalition Discord community centered around developing 1.12.2 mods and modpacks",
    "url": "/wiki/evergreen",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Data loading conditions",
    "pageTitle": "Data loading conditions",
    "sectionTitle": "",
    "description": "Overview on modloader's data loading conditions and how to use them",
    "content": "Data loading conditions Both Forge, Neoforge, and Fabric have additional datapack functionality where both modders and modpack developers can dynamically disable/enable data files based on given criteria. These criteria can range from checking if a tag/item exists in the registry, or simply disabling the file outright. A common application of this is disabling files with false conditions as shown below. Forge NeoForge Fabric :::info More information on the types of loading conditions supported by modloaders can be found on their respective documentations, which are listed below. Forge Neoforge Fabric :::",
    "url": "/wiki/info/data-loading-conditions",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer",
    "pageTitle": "Free Multiplayer",
    "sectionTitle": "",
    "description": "Exploring options for free multiplayer in modded Minecraft",
    "content": "Free multiplayer options for modded Minecraft Mod options There are many mods that allow for free multiplayer across different loaders and versions. We've compiled a list of them in our Useful Mods List under the \"Multiplayer\" section. These are by far the easiest methods for free multiplayer, as they require little setup or technical knowledge. The main drawback is that the server is hosted through the owner's machine, so if they go offline, the whole server does as well. Oracle Free Tier Oracle Free Tier is a service offered by the Oracle corporation to offer free 24GB servers to users who apply. The catch is that it's more time and effort to set up and maintain than any of the mods shown here, since you'd be setting up an unmanaged Linux server to play Minecraft from scratch. However, some online guides do exist to aid through the process. Server host partnership Many companies such as Bisect Hosting can provide you with free servers if you partner with them. These are usually premium, 6 12gb RAM servers that are free of cost to you. Aternos Aternos is a server host that provides free servers, but are generally poor quality and prone to many issues. We recommend to try the above options before Aternos. :::warning Aternos usually only assigns around 2400 MB of RAM to individual servers [1] , which is insufficient to run most modpacks, as well as only allowing certain mods to be used on their servers [2] , and they have been known to globally blacklist mods from their platform without notifying users [3] , leading to other cascading issues. :::",
    "url": "/wiki/info/free-multiplayer",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Free Multiplayer / Aternos",
    "pageTitle": "Free Multiplayer",
    "sectionTitle": "Aternos",
    "description": "Exploring options for free multiplayer in modded Minecraft",
    "content": "Aternos is a server host that provides free servers, but are generally poor quality and prone to many issues. We recommend to try the above options before Aternos. :::warning Aternos usually only assigns around 2400 MB of RAM to individual servers [1] , which is insufficient to run most modpacks, as well as only allowing certain mods to be used on their servers [2] , and they have been known to globally blacklist mods from their platform without notifying users [3] , leading to other cascading issues. :::",
    "url": "/wiki/info/free-multiplayer#aternos",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Free Multiplayer / Mod options",
    "pageTitle": "Free Multiplayer",
    "sectionTitle": "Mod options",
    "description": "Exploring options for free multiplayer in modded Minecraft",
    "content": "There are many mods that allow for free multiplayer across different loaders and versions. We've compiled a list of them in our Useful Mods List under the \"Multiplayer\" section. These are by far the easiest methods for free multiplayer, as they require little setup or technical knowledge. The main drawback is that the server is hosted through the owner's machine, so if they go offline, the whole server does as well.",
    "url": "/wiki/info/free-multiplayer#mod-options",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Free Multiplayer / Oracle Free Tier",
    "pageTitle": "Free Multiplayer",
    "sectionTitle": "Oracle Free Tier",
    "description": "Exploring options for free multiplayer in modded Minecraft",
    "content": "Oracle Free Tier is a service offered by the Oracle corporation to offer free 24GB servers to users who apply. The catch is that it's more time and effort to set up and maintain than any of the mods shown here, since you'd be setting up an unmanaged Linux server to play Minecraft from scratch. However, some online guides do exist to aid through the process.",
    "url": "/wiki/info/free-multiplayer#oracle-free-tier",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Free Multiplayer / Server host partnership",
    "pageTitle": "Free Multiplayer",
    "sectionTitle": "Server host partnership",
    "description": "Exploring options for free multiplayer in modded Minecraft",
    "content": "Many companies such as Bisect Hosting can provide you with free servers if you partner with them. These are usually premium, 6 12gb RAM servers that are free of cost to you.",
    "url": "/wiki/info/free-multiplayer#server-host-partnership",
    "tags": []
  },
  {
    "kind": "page",
    "title": "List of modpack launchers",
    "pageTitle": "List of modpack launchers",
    "sectionTitle": "",
    "description": "List of modpack launchers",
    "content": "List of modpack launchers Modpack launchers are software made to make playing modded Minecraft more accessible. They often include features such as profile instancing, mod browsing/downloading, and modpack exporting/importing features Launcher Curseforge support Modrinth support Features Additional notes Curseforge V X Recommended to download using the \"download standalone\" option, as Overwolf is generally considered bloatware. Modrinth X V Prism V V Recommended for non git pack development. ATLauncher V V Not to be confused with \"TLauncher\", a pirated launcher. Launchers with known issues Lunar Client Because of the modifications Lunar Client includes out of the box, it should not be used to install or develop modpacks. Official Minecraft Launcher The official Minecraft Launcher does not natively support modded instances, and therefore cannot really be used to develop modpacks.",
    "url": "/wiki/info/launchers",
    "tags": []
  },
  {
    "kind": "section",
    "title": "List of modpack launchers / Launchers with known issues",
    "pageTitle": "List of modpack launchers",
    "sectionTitle": "Launchers with known issues",
    "description": "List of modpack launchers",
    "content": "Launchers with known issues",
    "url": "/wiki/info/launchers#launchers-with-known-issues",
    "tags": []
  },
  {
    "kind": "section",
    "title": "List of modpack launchers / Lunar Client",
    "pageTitle": "List of modpack launchers",
    "sectionTitle": "Lunar Client",
    "description": "List of modpack launchers",
    "content": "Because of the modifications Lunar Client includes out of the box, it should not be used to install or develop modpacks.",
    "url": "/wiki/info/launchers#lunar-client",
    "tags": []
  },
  {
    "kind": "section",
    "title": "List of modpack launchers / Official Minecraft Launcher",
    "pageTitle": "List of modpack launchers",
    "sectionTitle": "Official Minecraft Launcher",
    "description": "List of modpack launchers",
    "content": "The official Minecraft Launcher does not natively support modded instances, and therefore cannot really be used to develop modpacks.",
    "url": "/wiki/info/launchers#official-minecraft-launcher",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Licenses",
    "pageTitle": "Licenses",
    "sectionTitle": "",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "Licenses Licenses are ways for developers to define what others can or cannot do with their work. Licenses can be applied to nearly everything, but Mods, Modpacks, and Resource Packs are the most relevant here. You should be aware of a projects license if you choose to do any of the following: Distribute the project on a hosting site that the original work is not hosted on Use any part of the projects assets or code in any way, besides simply including it in a modpack Port or fork the project :::info No matter what a project's license says, if it is hosted on a platform such as Curseforge or Modrinth, you are free to include it in a modpack as long as it is hosted on the same platform. Both CurseForge and Modrinth have clauses that prevent project developers from restricting access to users using their content on their platforms. CurseForge clause Modrinth clause ::: :::warning The following is simply an overview of common licenses, and is not legal advice! If you're ever unsure on what you can/can't do with someone's work, either consult an attorney or reach out to the author to get explicit permission to do what you want. ::: Common Licenses If existing licenses aren't to your liking, it is possible to use multiple licenses for different parts of your project. For example, you could license code under MIT and assets under ARR, meaning that users are free to distribute and modify your code as long as you are credited, but unable to reuse your assets for their own projects or republish the project in it's entirety without changing the assets. This is more advised than creating a license from scratch due to users and sites being familiar with existing licenses, meaning that enforcement and interpretation is more consistent. All Rights Reserved / ARR Author keeps ALL rights Users are not permitted to use the project in any way Projects without licenses should be treated as All Rights Reserved Ask the author before doing anything with the project! MIT Author waives most rights Users are free to do anything they'd like, as long as credit to the original author is provided Public Domain Author waives all rights Users are free to do anything they'd like, no credit or attribution required GNU GPL v3 \"Viral License\" derivatives of this project must use the same license as the original If this software or parts of this software is used in a larger project, the entire project must be GNU GPL Licensed Users are free to do anything they'd like, as long as credit to original author is provided and project derivative is GNU GPL licensed LGPL v3 \"Viral License\" derivatives of this project must use the same license as the original If this software or parts of this software is used in a larger project, the project as a whole does not have to be LGPL Licensed Users are free to do anything they'd like, as long as credit to original author is provided and the original LGPL licensed work is still LGPL licensed in the project derivative About “Custom Licenses” Unless you have legal experience, creating your own license from scratch is generally advised against. This includes licenses made with or assisted by LLMs such as ChatGPT. Licenses not made or approved by an attorney may not be legally valid, meaning that loopholes or unintended interpretations are a risk when using them. Using a license with no prior precedent may dissuade users from modifying, distributing, or contributing to your project.",
    "url": "/wiki/info/licenses",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / About “Custom Licenses”",
    "pageTitle": "Licenses",
    "sectionTitle": "About “Custom Licenses”",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "Unless you have legal experience, creating your own license from scratch is generally advised against. This includes licenses made with or assisted by LLMs such as ChatGPT. Licenses not made or approved by an attorney may not be legally valid, meaning that loopholes or unintended interpretations are a risk when using them. Using a license with no prior precedent may dissuade users from modifying, distributing, or contributing to your project.",
    "url": "/wiki/info/licenses#about-custom-licenses",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / All Rights Reserved / ARR",
    "pageTitle": "Licenses",
    "sectionTitle": "All Rights Reserved / ARR",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "Author keeps ALL rights Users are not permitted to use the project in any way Projects without licenses should be treated as All Rights Reserved Ask the author before doing anything with the project!",
    "url": "/wiki/info/licenses#all-rights-reserved-arr",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / Common Licenses",
    "pageTitle": "Licenses",
    "sectionTitle": "Common Licenses",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "If existing licenses aren't to your liking, it is possible to use multiple licenses for different parts of your project. For example, you could license code under MIT and assets under ARR, meaning that users are free to distribute and modify your code as long as you are credited, but unable to reuse your assets for their own projects or republish the project in it's entirety without changing the assets. This is more advised than creating a license from scratch due to users and sites being familiar with existing licenses, meaning that enforcement and interpretation is more consistent.",
    "url": "/wiki/info/licenses#common-licenses",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / GNU GPL v3",
    "pageTitle": "Licenses",
    "sectionTitle": "GNU GPL v3",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "\"Viral License\" derivatives of this project must use the same license as the original If this software or parts of this software is used in a larger project, the entire project must be GNU GPL Licensed Users are free to do anything they'd like, as long as credit to original author is provided and project derivative is GNU GPL licensed",
    "url": "/wiki/info/licenses#gnu-gpl-v3",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / LGPL v3",
    "pageTitle": "Licenses",
    "sectionTitle": "LGPL v3",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "\"Viral License\" derivatives of this project must use the same license as the original If this software or parts of this software is used in a larger project, the project as a whole does not have to be LGPL Licensed Users are free to do anything they'd like, as long as credit to original author is provided and the original LGPL licensed work is still LGPL licensed in the project derivative",
    "url": "/wiki/info/licenses#lgpl-v3",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / Licenses",
    "pageTitle": "Licenses",
    "sectionTitle": "Licenses",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "Licenses are ways for developers to define what others can or cannot do with their work. Licenses can be applied to nearly everything, but Mods, Modpacks, and Resource Packs are the most relevant here. You should be aware of a projects license if you choose to do any of the following: Distribute the project on a hosting site that the original work is not hosted on Use any part of the projects assets or code in any way, besides simply including it in a modpack Port or fork the project :::info No matter what a project's license says, if it is hosted on a platform such as Curseforge or Modrinth, you are free to include it in a modpack as long as it is hosted on the same platform. Both CurseForge and Modrinth have clauses that prevent project developers from restricting access to users using their content on their platforms. CurseForge clause Modrinth clause ::: :::warning The following is simply an overview of common licenses, and is not legal advice! If you're ever unsure on what you can/can't do with someone's work, either consult an attorney or reach out to the author to get explicit permission to do what you want. :::",
    "url": "/wiki/info/licenses#licenses",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / MIT",
    "pageTitle": "Licenses",
    "sectionTitle": "MIT",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "Author waives most rights Users are free to do anything they'd like, as long as credit to the original author is provided",
    "url": "/wiki/info/licenses#mit",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Licenses / Public Domain",
    "pageTitle": "Licenses",
    "sectionTitle": "Public Domain",
    "description": "An overview of a few common licenses, what they mean, and which one is right for you",
    "content": "Author waives all rights Users are free to do anything they'd like, no credit or attribution required",
    "url": "/wiki/info/licenses#public-domain",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Regular Expressions",
    "pageTitle": "Regular Expressions",
    "sectionTitle": "",
    "description": "Intro and examples of Regular Expressions (regex)",
    "content": "Intro to Regex Regular Expressions or \"regex\" are sequences of characters used to find patterns in text. Common applications of these in modded Minecraft are in mods like KubeJS or Reliable Remover, where one could bulk group blocks/items together and perform operations on them. Examples Regex string Caught registries /^minecraft:.+?bed/ Items ending in \"bed\" under the Minecraft namespace /^minecraft:pink .+/, Items beginning with \"pink\" under the Minecraft namespace /^minecraft:. quartz. / Items containing \"quartz\" under the Minecraft namespace /^ ingot. / Items containing \"ingot\" under any namespace More Information https://regex101.com",
    "url": "/wiki/info/regex",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Regular Expressions / Examples",
    "pageTitle": "Regular Expressions",
    "sectionTitle": "Examples",
    "description": "Intro and examples of Regular Expressions (regex)",
    "content": "Regex string Caught registries /^minecraft:.+?bed/ Items ending in \"bed\" under the Minecraft namespace /^minecraft:pink .+/, Items beginning with \"pink\" under the Minecraft namespace /^minecraft:. quartz. / Items containing \"quartz\" under the Minecraft namespace /^ ingot. / Items containing \"ingot\" under any namespace",
    "url": "/wiki/info/regex#examples",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Regular Expressions / Intro to Regex",
    "pageTitle": "Regular Expressions",
    "sectionTitle": "Intro to Regex",
    "description": "Intro and examples of Regular Expressions (regex)",
    "content": "Regular Expressions or \"regex\" are sequences of characters used to find patterns in text. Common applications of these in modded Minecraft are in mods like KubeJS or Reliable Remover, where one could bulk group blocks/items together and perform operations on them.",
    "url": "/wiki/info/regex#intro-to-regex",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Regular Expressions / More Information",
    "pageTitle": "Regular Expressions",
    "sectionTitle": "More Information",
    "description": "Intro and examples of Regular Expressions (regex)",
    "content": "https://regex101.com",
    "url": "/wiki/info/regex#more-information",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Useful Tags and Terms",
    "pageTitle": "Useful Tags and Terms",
    "sectionTitle": "",
    "description": "Useful tags and terms",
    "content": "Closed Source Closed Source refers to projects that does not have the source code publicly available, in most cases prohibiting modification and distribution of the project. Kitchen Sink Kitchen Sink generally refers to modpacks that possess one or more of the following qualities: High mod count Little to no theme or focus Little to no integration or custom content This term can have negative connotations, as some use it to label modpacks with low effort or quality, while others use it as a neutral categorization term for packs that're broader in theme or have a higher mod count. Open Source Open Source refers to projects that both have a publicly available source code published and a license prohibiting redistribution and modifications, such as MIT or GPL. Tags Tags act as references to groups of registry entries, letting Minecraft treat multiple items, blocks, or entities as a single category. For example, the tag minecraft:logs represents all log blocks, so commands or recipes using that tag automatically apply to oak logs, birch logs, and any other block included in it. Tags are one of the most powerful data pack features for organizing and modifying game behavior without editing core files. More information on tags: https://minecraft.wiki/w/Tag (Java Edition) Vanilla+ Vanilla+ is used to refer to concepts or features resembling similarities to the base \"Vanilla\" game. The term is widely regarded as a buzzword due to it being used inappropriately often or as a filler adjective that doesn't adequately describe the concepts or features at hand. Instead of \"Vanilla+\", it is more productive in conversations to describe which aspect of vanilla is being expanded upon. \"Does anyone know any dungeon mods that include structures similar in size and complexity to ones in the base game?\" is a lot better question than \"Does anyone know any vanilla+ dungeon mods?\", since the latter has a myriad of possible interpretations. A vanilla+ dungeon mod to some people could mean one that expands on the existing structures in the game, such as the Yung's structure series, or it could entail a mod that uses vanilla's block pallet and mobs, such as When Dungeons Arise. While these mods have their similarities, they are vastly different in implementation, and it is important to effectively communicate what kinds of concepts and features you're looking for in content. Worldgen Feature A worldgen feature is a chunk generation element such as an ore vein, patch of flowers, tree placement, lake, or other small terrain decoration. Features usually run after the base terrain and biome shape are chosen. They are configured through placed features, configured features, and biome modification hooks depending on the mod loader and Minecraft version. How features differ from structures Structures are large authored generation units such as villages, temples, dungeons, or custom set pieces. Features are usually smaller, more repeatable, and more data driven. If you are removing an unwanted ore, plant, or stone patch, you are usually editing a feature or the biome step that places it rather than a structure. Structure Structures (also known as a \"generated structure\" or \"structure feature\") are naturally generated formations that can be located using /locate structure in game, such as Ancient Cities, Igloos, and Woodland Mansions. They are defined via NBT as opposed to features which generate dynamically. More information on structures: https://minecraft.wiki/w/Structure Useful Tags These are tags added by mods, modloaders, or even sometimes the base game that have potentially useful behaviors for modpack developers. On versions 1.20.1 and below, tags added by the Forge modloader used the forge namespace (ex: forge:relocation not supported), while loaders like Fabric and NeoForge use tags with the namespace c, standing for \"Common\". Tag name Description Tag Type Additional Notes c:relocation not supported forge:relocation not supported Prevents a block from being \"relocated,\" such as via Create Contraptions or Mekanism Cardboard Boxes. Block c:hidden from recipe viewers Hides an item from a recipe viewer index. Block, Item, Fluid EMI respects this on 1.20.1 minecraft:dirt Allows plants to be placed on top of the block. Block terrablender:overworld regions terrablender:nether regions terrablender:aether regions Determines what dimensions are considered as for Terrablender biome injection purposes. Useful if you want to redirect the injection to an alternate dimension (say, an alternative overworld) or if you want to remove Terrablender biomes from generating. Dimension terrablender:aether regions is added by Aeroblender lychee:lightning immune Self explanatory Entity From Lychee lychee:lightning fire immune Self explanatory Entity lychee:fire immune Self explanatory Item lychee:dispenser placement Self explanatory Item Additionally, you can find all biome tags used by Minecraft, Forge, and NeoForge / Fabric here. https://gist.github.com/TelepathicGrunt/b768ce904baa4598b21c3ca42f137f23 what tag entries exist that i can use",
    "url": "/wiki/info/useful-terms",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Useful Tags and Terms / How features differ from structures",
    "pageTitle": "Useful Tags and Terms",
    "sectionTitle": "How features differ from structures",
    "description": "Useful tags and terms",
    "content": "Structures are large authored generation units such as villages, temples, dungeons, or custom set pieces. Features are usually smaller, more repeatable, and more data driven. If you are removing an unwanted ore, plant, or stone patch, you are usually editing a feature or the biome step that places it rather than a structure. Structure Structures (also known as a \"generated structure\" or \"structure feature\") are naturally generated formations that can be located using /locate structure in game, such as Ancient Cities, Igloos, and Woodland Mansions. They are defined via NBT as opposed to features which generate dynamically. More information on structures: https://minecraft.wiki/w/Structure Useful Tags These are tags added by mods, modloaders, or even sometimes the base game that have potentially useful behaviors for modpack developers. On versions 1.20.1 and below, tags added by the Forge modloader used the forge namespace (ex: forge:relocation not supported), while loaders like Fabric and NeoForge use tags with the namespace c, standing for \"Common\". Tag name Description Tag Type Additional Notes c:relocation not supported forge:relocation not supported Prevents a block from being \"relocated,\" such as via Create Contraptions or Mekanism Cardboard Boxes. Block c:hidden from recipe viewers Hides an item from a recipe viewer index. Block, Item, Fluid EMI respects this on 1.20.1 minecraft:dirt Allows plants to be placed on top of the block. Block terrablender:overworld regions terrablender:nether regions terrablender:aether regions Determines what dimensions are considered as for Terrablender biome injection purposes. Useful if you want to redirect the injection to an alternate dimension (say, an alternative overworld) or if you want to remove Terrablender biomes from generating. Dimension terrablender:aether regions is added by Aeroblender lychee:lightning immune Self explanatory Entity From Lychee lychee:lightning fire immune Self explanatory Entity lychee:fire immune Self explanatory Item lychee:dispenser placement Self explanatory Item Additionally, you can find all biome tags used by Minecraft, Forge, and NeoForge / Fabric here. https://gist.github.com/TelepathicGrunt/b768ce904baa4598b21c3ca42f137f23 what tag entries exist that i can use",
    "url": "/wiki/info/useful-terms#how-features-differ-from-structures",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Version Control Tools",
    "pageTitle": "Version Control Tools",
    "sectionTitle": "",
    "description": "Git and pack-specific tooling choices for versioning a modpack project.",
    "content": "Version Control Tools Version control lets you track changes, recover mistakes, and collaborate on a pack over time instead of relying on ad hoc backups. Git Git is the standard version control system used across software projects and works well for modpacks when paired with a clean .gitignore and a metadata driven pack format. A beginner friendly starting point is GitHub Desktop, but the main concepts apply equally well on Forgejo, GitLab, and other Git hosting. At minimum, keep generated runtime content such as /saves and downloaded /mods out of source control unless you explicitly intend to redistribute them. Pack specific tooling The modpack ecosystem also has tools that treat the pack manifest itself as the source of truth rather than a loose folder of JARs. packwand packwiz Pakku These tools are best when you want reproducible exports, clean reviewable changes, and automation for publishing or updating packs.",
    "url": "/wiki/info/version-control-tools",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Version Control Tools / Git",
    "pageTitle": "Version Control Tools",
    "sectionTitle": "Git",
    "description": "Git and pack-specific tooling choices for versioning a modpack project.",
    "content": "Git is the standard version control system used across software projects and works well for modpacks when paired with a clean .gitignore and a metadata driven pack format. A beginner friendly starting point is GitHub Desktop, but the main concepts apply equally well on Forgejo, GitLab, and other Git hosting. At minimum, keep generated runtime content such as /saves and downloaded /mods out of source control unless you explicitly intend to redistribute them.",
    "url": "/wiki/info/version-control-tools#git",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Version Control Tools / Pack-specific tooling",
    "pageTitle": "Version Control Tools",
    "sectionTitle": "Pack-specific tooling",
    "description": "Git and pack-specific tooling choices for versioning a modpack project.",
    "content": "The modpack ecosystem also has tools that treat the pack manifest itself as the source of truth rather than a loose folder of JARs. packwand packwiz Pakku These tools are best when you want reproducible exports, clean reviewable changes, and automation for publishing or updating packs.",
    "url": "/wiki/info/version-control-tools#pack-specific-tooling",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Pack Management",
    "pageTitle": "Pack Management",
    "sectionTitle": "",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "content": "Pack Management This section covers the practical toolchain around building, versioning, exporting, testing, and publishing packs. Platforms CurseForge Modrinth Project management Tools packwand packwiz packwiz components Pakku Choosing a tool Need Best fit Why Maintain one or more packs as metadata in Git packwand Adds higher level workflows on top of packwiz, including workspace operations, publishing, diffing, validation, and a local GUI/API Maintain a single pack with a smaller CLI surface packwiz Mature TOML based pack format with straightforward install/export workflows Distribute updates to players or servers packwiz components The bootstrap and installer handle launch time updates and optional mods Produce a hosted .mrpack from an existing manifest Pakku Useful when you want an external exporter around the packwiz style ecosystem Recommended workflows 1. Author the pack as metadata, not as a folder of downloaded JARs. 2. Keep the pack in Git so recipe, config, and dependency changes are reviewable. 3. Use packwand when you need repository aware workflows such as workspace sync, bulk updates, publishing plans, or diagnostics. 4. Use packwiz directly when you want the smaller original toolchain and you do not need the packwand specific repository automation. 5. Test the player install path with the bootstrap and installer before you publish a release. Mental model packwiz defines the core manifest format and the classic CLI workflow. packwand builds on that format and adds repository automation, publishing, diagnostics, and multi pack support. packwiz installer and the bootstrap are runtime delivery tools for players and servers. Hosting targets such as CurseForge and Modrinth are output formats and distribution channels, not your source of truth.",
    "url": "/wiki/modpack-management",
    "tags": []
  },
  {
    "kind": "page",
    "title": "CurseForge",
    "pageTitle": "CurseForge",
    "sectionTitle": "",
    "description": "Things to be aware of when submitting your modpack to CurseForge",
    "content": "CurseForge CurseForge is a popular platform for Minecraft mods and modpacks owned by Overwolf, and as a modpack developer it's important you have a good understanding of how it works and impacts your development. You can find the platform specific moderation policies here. One of the more important rules to be aware of is the third party mods guidelines. Typically when you create a client pack export, your zip file will not actually contain mods if the export was done correctly. They will instead contain a manifest that allows the mods to download from CurseForge when a user imports the pack. Therefore, if you see a mods folder in your export, that means you have either done something wrong or you've used mods that are not on CurseForge. All platforms have their own general rules for these mods which you can find on the above link. Paying attention the the licenses of third party mods will help your pack not be denied by CurseForge moderation. CurseForge also maintains a spreadsheet of pre cleared third party mods here. These mods will automatically be approved by CurseForge moderation if they are included in your pack's mod overrides. Client pack The CurseForge Launcher has its own process for exporting modpacks outlined in their first party guide here. Third party launchers or CLI tools such as Pakku can also be used to create a CurseForge applicable client pack export, though the process may differ depending on the platform. Server Pack Server packs are specially made exports made for servers to install your modpack. They are uploaded as \"Additional Files\" after uploading a pack version, and they differ in a few small ways from client packs: They contain mod files. This is an important distinction as you have to be extra careful when making sure your pack export is done properly They only contain mods with server functionality. Mods with client side functionality only may crash the server on startup Certain utility scripts/or files can be added, such as server icons or start scripts. :::warning Not all mods are tagged correctly on CurseForge! Some mods may be marked as Client/Server despite only having client functionality. Third party server pack creation tools can allow you to manually add taggingsfor mods, but be sure to always test server packs if you're able to. ::: Exporting a server pack can be tricky and not a completely solved problem, though a few good solutions exist. Curseforge's tutorial has a manual solution. Server Pack Creator is a specialized tool for server packs, with GUI and command line options Pakku and PackWiz both over solutions for creating a server pack at the same time as a client pack.",
    "url": "/wiki/modpack-management/curseforge",
    "tags": []
  },
  {
    "kind": "section",
    "title": "CurseForge / Client pack",
    "pageTitle": "CurseForge",
    "sectionTitle": "Client pack",
    "description": "Things to be aware of when submitting your modpack to CurseForge",
    "content": "The CurseForge Launcher has its own process for exporting modpacks outlined in their first party guide here. Third party launchers or CLI tools such as Pakku can also be used to create a CurseForge applicable client pack export, though the process may differ depending on the platform.",
    "url": "/wiki/modpack-management/curseforge#client-pack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "CurseForge / Server Pack",
    "pageTitle": "CurseForge",
    "sectionTitle": "Server Pack",
    "description": "Things to be aware of when submitting your modpack to CurseForge",
    "content": "Server packs are specially made exports made for servers to install your modpack. They are uploaded as \"Additional Files\" after uploading a pack version, and they differ in a few small ways from client packs: They contain mod files. This is an important distinction as you have to be extra careful when making sure your pack export is done properly They only contain mods with server functionality. Mods with client side functionality only may crash the server on startup Certain utility scripts/or files can be added, such as server icons or start scripts. :::warning Not all mods are tagged correctly on CurseForge! Some mods may be marked as Client/Server despite only having client functionality. Third party server pack creation tools can allow you to manually add taggingsfor mods, but be sure to always test server packs if you're able to. ::: Exporting a server pack can be tricky and not a completely solved problem, though a few good solutions exist. Curseforge's tutorial has a manual solution. Server Pack Creator is a specialized tool for server packs, with GUI and command line options Pakku and PackWiz both over solutions for creating a server pack at the same time as a client pack.",
    "url": "/wiki/modpack-management/curseforge#server-pack",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Marketing",
    "pageTitle": "Marketing",
    "sectionTitle": "",
    "description": "Notes on how to build an audience for your pack",
    "content": "Marketing Marketing a modpack is not a simple task, there's a very low barrier to entry for creating one, and it's very easy for it to get lost in the sea of other packs. One important thing to note about modpacks is that unless you're a larger creator, there will be no large blockbuster release like you'd see for a video game. It's likely that if your pack picks up steam, it'll be after more than a few updates. Maybe a bigger name in the community notices your pack, or it gets featured on a platform like CurseForge or Modrinth. Don't get discouraged if you release your packs to crickets! Modpack page Your modpack's page is essentially the face of the project. Screenshots Do not just fill your modpack's page with worldgen and structure screenshots! Anyone can download a few worldgen mods, slap on a shader, and call it a day. Players will not be particularly impressed unless your modpack features custom or at least unique worldgen and structures. Your goal should be to highlight what makes your pack unique in as few screenshots as possible. Some ideas to get you started: If you're making a tech pack, build out a visually impressive factory that inspires players Custom mechanics make for great screenshots, as they prove at least some level of effort went into the pack Quest books can be very attractive to certain types of players, and screenshots of well done quest book pages well help filter for those players A note on generated images The use of images generated by AI models (i.e. AI generated images) is a net negative for your project. Modding is a primarily passion driven fields, and many users will be put off by AI generated logos of images in your description, even if they are minor. The use of these images can make your project come across as low effort, even if the actual contents of your pack aren't! If you're bad at art, the Minecraft Title Generator plugin for BlockBench offers lots of customization for making a text based logo for your page. Alternatively, throwing something together in paint or sourcing from the community can be a good option. If you've put tons of effort into your pack, just put a tiny bit more into your logo!",
    "url": "/wiki/modpack-management/marketing",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Marketing / A note on generated images",
    "pageTitle": "Marketing",
    "sectionTitle": "A note on generated images",
    "description": "Notes on how to build an audience for your pack",
    "content": "The use of images generated by AI models (i.e. AI generated images) is a net negative for your project. Modding is a primarily passion driven fields, and many users will be put off by AI generated logos of images in your description, even if they are minor. The use of these images can make your project come across as low effort, even if the actual contents of your pack aren't! If you're bad at art, the Minecraft Title Generator plugin for BlockBench offers lots of customization for making a text based logo for your page. Alternatively, throwing something together in paint or sourcing from the community can be a good option. If you've put tons of effort into your pack, just put a tiny bit more into your logo!",
    "url": "/wiki/modpack-management/marketing#a-note-on-generated-images",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Marketing / Modpack page",
    "pageTitle": "Marketing",
    "sectionTitle": "Modpack page",
    "description": "Notes on how to build an audience for your pack",
    "content": "Your modpack's page is essentially the face of the project.",
    "url": "/wiki/modpack-management/marketing#modpack-page",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Marketing / Screenshots",
    "pageTitle": "Marketing",
    "sectionTitle": "Screenshots",
    "description": "Notes on how to build an audience for your pack",
    "content": "Do not just fill your modpack's page with worldgen and structure screenshots! Anyone can download a few worldgen mods, slap on a shader, and call it a day. Players will not be particularly impressed unless your modpack features custom or at least unique worldgen and structures. Your goal should be to highlight what makes your pack unique in as few screenshots as possible. Some ideas to get you started: If you're making a tech pack, build out a visually impressive factory that inspires players Custom mechanics make for great screenshots, as they prove at least some level of effort went into the pack Quest books can be very attractive to certain types of players, and screenshots of well done quest book pages well help filter for those players",
    "url": "/wiki/modpack-management/marketing#screenshots",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Modrinth",
    "pageTitle": "Modrinth",
    "sectionTitle": "",
    "description": "Things to be aware of when submitting your modpack to Modrinth",
    "content": "Modrinth Modrinth is a popular platform for Minecraft mods and modpacks owned by Spark Universe, and as a modpack developer it's important you have a good understanding of how it works and impacts your development. You can find the platform specific moderation policies here. One of the more important rules to be aware of is the third party mods guidelines. Typically when you create a client pack export, your .mrpack file will not actually contain mods if the export was done correctly. They will instead contain a manifest that allows the mods to download from Modrinth when a user imports the pack. Therefore, if you see a mods folder in your export, that means you have either done something wrong or you've used mods that are not on Modrinth. All platforms have their own general rules for these mods which you can find on the above link. Paying attention the the licenses of third party mods will help your pack not be denied by Modrinth moderation. Handling Non Modrinth mods You may research how Modrinth handles off platform mods in their recent blogpost, accessible here FTB Checker FTB Checker is a mod specifically made for the FTB suite of mods. It renders a screen with download links to mods if they are not installed. On 1.21.1 and above, this mod also supports non FTB mods Missing Mods Checker Missing Mods Checker does a similar thing as the above, but works on 1.20.1 and also contains a button to download mods at once. This mod currently does not generate a config, lacks documentation and is not implemented anywhere in any official Luna Pixel Studios modpacks. Please note that this may not be able to be turned functional without direct help from the Developer. Client pack The Modrinth Launcher has its own process for exporting modpacks outlined in their first party guide here. Third party launchers or CLI tools such as Pakku can also be used to create a Modrinth applicable mrpack export, though the process may differ depending on the platform.",
    "url": "/wiki/modpack-management/modrinth",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Modrinth / Client pack",
    "pageTitle": "Modrinth",
    "sectionTitle": "Client pack",
    "description": "Things to be aware of when submitting your modpack to Modrinth",
    "content": "The Modrinth Launcher has its own process for exporting modpacks outlined in their first party guide here. Third party launchers or CLI tools such as Pakku can also be used to create a Modrinth applicable mrpack export, though the process may differ depending on the platform.",
    "url": "/wiki/modpack-management/modrinth#client-pack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Modrinth / FTB Checker",
    "pageTitle": "Modrinth",
    "sectionTitle": "FTB Checker",
    "description": "Things to be aware of when submitting your modpack to Modrinth",
    "content": "FTB Checker is a mod specifically made for the FTB suite of mods. It renders a screen with download links to mods if they are not installed. On 1.21.1 and above, this mod also supports non FTB mods",
    "url": "/wiki/modpack-management/modrinth#ftb-checker",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Modrinth / Handling Non-Modrinth mods",
    "pageTitle": "Modrinth",
    "sectionTitle": "Handling Non-Modrinth mods",
    "description": "Things to be aware of when submitting your modpack to Modrinth",
    "content": "You may research how Modrinth handles off platform mods in their recent blogpost, accessible here",
    "url": "/wiki/modpack-management/modrinth#handling-non-modrinth-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Modrinth / Missing Mods Checker",
    "pageTitle": "Modrinth",
    "sectionTitle": "Missing Mods Checker",
    "description": "Things to be aware of when submitting your modpack to Modrinth",
    "content": "Missing Mods Checker does a similar thing as the above, but works on 1.20.1 and also contains a button to download mods at once. This mod currently does not generate a config, lacks documentation and is not implemented anywhere in any official Luna Pixel Studios modpacks. Please note that this may not be able to be turned functional without direct help from the Developer.",
    "url": "/wiki/modpack-management/modrinth#missing-mods-checker",
    "tags": []
  },
  {
    "kind": "page",
    "title": "packwand",
    "pageTitle": "packwand",
    "sectionTitle": "",
    "description": "",
    "content": "packwand packwand is a Minecraft modpack toolchain that keeps the packwiz metadata format but adds repository aware workflows, multi pack workspace management, publishing, and diagnostics. Instead of handling downloaded JARs directly, packwand keeps modpacks in TOML backed metadata that is version controlled, reviewable, and exportable. When to reach for packwand Use packwand when you need one or more of these: A single source of truth in Git for mods, configs, scripts, and exports Multiple related packs in one repository with shared content or synchronized updates First class publishing workflows for Modrinth, CurseForge, and internal targets Diagnostics such as diffing, validation, content linting, and test installs A local GUI or HTTP API on top of the manifest driven workflow If you only need the smaller original CLI for a single pack, packwiz is still a good fit. What packwand adds on top of packwiz Workspace operations across many packs in the same repository Publishing commands that plan, build, upload, and verify release artifacts Repository aware commands such as diff, pages, workspace status, and workspace sync Extra automation surfaces: HTTP API, local GUI, automation plans, and richer diagnostics A broader installer/export/testing story for teams maintaining long lived packs Typical repository flow 1. Create or enter a pack repository. 2. Run packwand init for a single pack or packwand new when you want packwand's scaffolding. 3. Add mods from Modrinth, CurseForge, or forge hosted releases with metadata commands instead of dropping JARs into mods/. 4. Commit the resulting manifest changes to Git. 5. Use packwand refresh, validate, content lint, and test as quality gates. 6. Build and publish from the same metadata when the pack is ready. Single pack vs multi pack use Single pack packwand still works well for one pack when you want publishing, validation, a local GUI, or a more opinionated CLI than packwiz offers. Multi pack repository packwand becomes more compelling when you maintain variants such as: client/server splits loader ports long term support branches regional or platform specific releases \"base pack\" content reused by consumer packs In these cases, workspace, packs, diff, and publish remove a lot of manual repository work. Usage Pack Management add Add a mod to all or a specific pack's Modrinth and CurseForge subdirs curseforge Manage CurseForge based mods forgejo Manage projects released on Forgejo, Gitea, or Codeberg freeze Pin mods so updates skip them github Manage projects released on GitHub gitlab Manage projects released on GitLab or self hosted GitLab instances import Import an .mrpack or CurseForge zip as a new modpack init Initialise a packwiz modpack modrinth Manage Modrinth based mods new Scaffold a new pack pin Pin a file so it does not get updated automatically port Compare Modrinth and CurseForge subdirs and port missing mods rehash Migrate all hashes to a specific format remove Remove an external file from the modpack side Check or fix a mod's side across all subdirs in a pack unfreeze Unpin mods so updates can apply to them again unpin Unpin a file so it receives updates url Add external files from a direct download link Updates & Refresh migrate Migrate Minecraft, loader, or pack format generations refresh Refresh the index file update Update an external file or all external files in the modpack Build & Export build Build modpack exports and zip packs from git changed targets bump Bump the manifest version export Export packs locally publish Build, upload, verify, or list publish targets for a pack Workspace packs Look up or edit any pack's manifest fields by id workspace Multi pack workspace operations across all packs Diagnostics content lint Lint pack content doctor Check that tools, repo root, and manifests are healthy lint Check JSON and .pw.toml files for syntax errors list List all the mods in the modpack test Spin up packwand serve and validate a pack with packwiz installer validate Validate pack manifests version Print the packwand version Other api Run and inspect the Packwand HTTP API automation Query effective automation settings for a pack cache Inspect and maintain the shared download cache diff Show mod additions, removals, and updates between two git refs gui Run the local Packwand web GUI modlist Write a crash assistant modlist.json from a pack's mods/ directory nix Nix integration pages Regenerate modlist.md files and the projects index run Execute a user defined script from pack.toml serve Run a local development server settings Manage pack settings utils Utilities for managing packwiz itself Flags cache Override the shared download cache directory config Select the packwand config file meta folder Change where new metadata files are written meta folder base Resolve meta folder relative to another base directory no refresh Skip index and pack.toml refresh after modifications pack file Select the pack metadata file y, yes Accept default prompts in non interactive mode Getting started Install packwand Create your first modpack Command reference Pack format reference Repository Releases",
    "url": "/wiki/modpack-management/packwand",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Building the native GUI app",
    "pageTitle": "Building the native GUI app",
    "sectionTitle": "",
    "description": "",
    "content": "Building the native GUI app The Packwand GUI ships in two forms: the browser based packwand gui command, and a native desktop app built with Tauri v2 that wraps the same frontend and server. The Tauri shell lives in apps/packwand/gui/tauri/. Architecture The app follows the pattern used by the Modrinth App: a small Rust backend acts as the privileged bridge, and the webview renders the existing Gleam frontend. On launch, a bundled boot page calls the single exposed IPC command, backend url. The Rust backend locates the packwand binary (PACKWAND BIN, next to the app executable, then PATH), spawns packwand gui no open port 0 as a managed child process, and reads the bound http://127.0.0.1: / address from its startup banner. The window then navigates to the local server. From that point everything works exactly like the browser GUI — same Gleam frontend, same HTTP API, same SSE job streams. The server pages are deliberately given no Tauri IPC access (the capability only covers the boot page), so the webview cannot reach system APIs beyond what the packwand HTTP API already exposes. The backend process is terminated when the app exits. Prerequisites Follow the Tauri v2 prerequisites guide for your platform. In short: Rust (stable, via rustup) Go 1.25+ (builds the packwand backend the app spawns) Node.js 22.18+ (only needed when rebuilding the Gleam frontend via gui/ui/build.mts; the build script is TypeScript run via Node’s native type stripping) The Tauri CLI: cargo install tauri cli version \"^2\" locked Platform specific webview dependencies: Platform Requirement Windows WebView2 runtime (preinstalled on Windows 11) and the Microsoft C++ Build Tools Linux webkit2gtk 4.1, libgtk 3 dev, build essential, libssl dev, libayatana appindicator3 dev, librsvg2 dev (names vary by distro — see the Tauri guide) macOS Xcode Command Line Tools (xcode select install) Building From the repository root: This builds the packwand CLI first, then runs cargo tauri build in apps/packwand/gui/tauri, producing a platform installer/bundle under apps/packwand/gui/tauri/src tauri/target/release/bundle/. ::: warning The packaged app expects a packwand executable next to it or on PATH (or PACKWAND BIN set). When distributing, ship the packwand binary alongside the app bundle. ::: Development tauri dev starts packwand gui no open port 8654 (via beforeDevCommand) and points the window at it, so frontend/API changes are picked up by restarting the server. To iterate on the Gleam frontend, rebuild it with task gui frontend (the server serves the embedded static files, so rebuild the Go binary — or just restart cargo tauri dev — after changing them). Security boundaries tauri.conf.json sets a strict CSP for bundled assets and enables no Tauri plugins. capabilities/default.json grants only core:default to the boot page; no filesystem, shell, or HTTP scopes are exposed to the webview. All pack management operations flow through the packwand gui HTTP API on 127.0.0.1, which binds to the loopback interface only.",
    "url": "/wiki/modpack-management/packwand/development/gui-build",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building the native GUI app / Architecture",
    "pageTitle": "Building the native GUI app",
    "sectionTitle": "Architecture",
    "description": "",
    "content": "The app follows the pattern used by the Modrinth App: a small Rust backend acts as the privileged bridge, and the webview renders the existing Gleam frontend. On launch, a bundled boot page calls the single exposed IPC command, backend url. The Rust backend locates the packwand binary (PACKWAND BIN, next to the app executable, then PATH), spawns packwand gui no open port 0 as a managed child process, and reads the bound http://127.0.0.1: / address from its startup banner. The window then navigates to the local server. From that point everything works exactly like the browser GUI — same Gleam frontend, same HTTP API, same SSE job streams. The server pages are deliberately given no Tauri IPC access (the capability only covers the boot page), so the webview cannot reach system APIs beyond what the packwand HTTP API already exposes. The backend process is terminated when the app exits.",
    "url": "/wiki/modpack-management/packwand/development/gui-build#architecture",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building the native GUI app / Building",
    "pageTitle": "Building the native GUI app",
    "sectionTitle": "Building",
    "description": "",
    "content": "From the repository root: This builds the packwand CLI first, then runs cargo tauri build in apps/packwand/gui/tauri, producing a platform installer/bundle under apps/packwand/gui/tauri/src tauri/target/release/bundle/. ::: warning The packaged app expects a packwand executable next to it or on PATH (or PACKWAND BIN set). When distributing, ship the packwand binary alongside the app bundle. :::",
    "url": "/wiki/modpack-management/packwand/development/gui-build#building",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building the native GUI app / Development",
    "pageTitle": "Building the native GUI app",
    "sectionTitle": "Development",
    "description": "",
    "content": "tauri dev starts packwand gui no open port 8654 (via beforeDevCommand) and points the window at it, so frontend/API changes are picked up by restarting the server. To iterate on the Gleam frontend, rebuild it with task gui frontend (the server serves the embedded static files, so rebuild the Go binary — or just restart cargo tauri dev — after changing them).",
    "url": "/wiki/modpack-management/packwand/development/gui-build#development",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building the native GUI app / Prerequisites",
    "pageTitle": "Building the native GUI app",
    "sectionTitle": "Prerequisites",
    "description": "",
    "content": "Follow the Tauri v2 prerequisites guide for your platform. In short: Rust (stable, via rustup) Go 1.25+ (builds the packwand backend the app spawns) Node.js 22.18+ (only needed when rebuilding the Gleam frontend via gui/ui/build.mts; the build script is TypeScript run via Node’s native type stripping) The Tauri CLI: cargo install tauri cli version \"^2\" locked Platform specific webview dependencies: Platform Requirement Windows WebView2 runtime (preinstalled on Windows 11) and the Microsoft C++ Build Tools Linux webkit2gtk 4.1, libgtk 3 dev, build essential, libssl dev, libayatana appindicator3 dev, librsvg2 dev (names vary by distro — see the Tauri guide) macOS Xcode Command Line Tools (xcode select install)",
    "url": "/wiki/modpack-management/packwand/development/gui-build#prerequisites",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building the native GUI app / Security boundaries",
    "pageTitle": "Building the native GUI app",
    "sectionTitle": "Security boundaries",
    "description": "",
    "content": "tauri.conf.json sets a strict CSP for bundled assets and enables no Tauri plugins. capabilities/default.json grants only core:default to the boot page; no filesystem, shell, or HTTP scopes are exposed to the webview. All pack management operations flow through the packwand gui HTTP API on 127.0.0.1, which binds to the loopback interface only.",
    "url": "/wiki/modpack-management/packwand/development/gui-build#security-boundaries",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Installation",
    "pageTitle": "Installation",
    "sectionTitle": "",
    "description": "",
    "content": "Installation Prebuilt binaries Prebuilt binaries for Linux, Windows, and macOS (amd64 and arm64) are published on the Forgejo releases page . Download the archive for your platform, extract it, and add the folder containing the executable to your PATH environment variable (see tutorial for Windows here) or move it to where you want to use it. Verify the download against checksums.txt (SHA 256) attached to the release. go install With Go 1.26 or newer installed, a single command builds and installs the latest packwand from the repository: The binary is placed in $(go env GOPATH)/bin make sure that directory is on your PATH. ::: tip @latest resolves through the public Go module proxy, which can lag the tip of main by up to 30 minutes. To fetch the newest commit straight from the repository, bypass the proxy: ::: Building from source 1. Install Go (1.26 or newer) from https://golang.org/dl/ 2. Clone the repository and build: Be patient the first time Go has to download and compile dependencies as well. Which install path should you choose? Use the release archive if you just want a stable binary on your workstation. Use go install if you already have Go installed and want the CLI on your developer machine quickly. Build from source when you need to modify packwand itself, test a branch, or produce binaries in CI. ::: tip Tools in this repository that shell out to packwand respect the PACKWAND BIN environment variable if you want to point them at a specific binary. :::",
    "url": "/wiki/modpack-management/packwand/installation",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Installation / Building from source",
    "pageTitle": "Installation",
    "sectionTitle": "Building from source",
    "description": "",
    "content": "1. Install Go (1.26 or newer) from https://golang.org/dl/ 2. Clone the repository and build: Be patient the first time Go has to download and compile dependencies as well.",
    "url": "/wiki/modpack-management/packwand/installation#building-from-source",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Installation / go install",
    "pageTitle": "Installation",
    "sectionTitle": "go install",
    "description": "",
    "content": "With Go 1.26 or newer installed, a single command builds and installs the latest packwand from the repository: The binary is placed in $(go env GOPATH)/bin make sure that directory is on your PATH. ::: tip @latest resolves through the public Go module proxy, which can lag the tip of main by up to 30 minutes. To fetch the newest commit straight from the repository, bypass the proxy: :::",
    "url": "/wiki/modpack-management/packwand/installation#go-install",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Installation / Prebuilt binaries",
    "pageTitle": "Installation",
    "sectionTitle": "Prebuilt binaries",
    "description": "",
    "content": "Prebuilt binaries for Linux, Windows, and macOS (amd64 and arm64) are published on the Forgejo releases page . Download the archive for your platform, extract it, and add the folder containing the executable to your PATH environment variable (see tutorial for Windows here) or move it to where you want to use it. Verify the download against checksums.txt (SHA 256) attached to the release.",
    "url": "/wiki/modpack-management/packwand/installation#prebuilt-binaries",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Installation / Which install path should you choose?",
    "pageTitle": "Installation",
    "sectionTitle": "Which install path should you choose?",
    "description": "",
    "content": "Use the release archive if you just want a stable binary on your workstation. Use go install if you already have Go installed and want the CLI on your developer machine quickly. Build from source when you need to modify packwand itself, test a branch, or produce binaries in CI. ::: tip Tools in this repository that shell out to packwand respect the PACKWAND BIN environment variable if you want to point them at a specific binary. :::",
    "url": "/wiki/modpack-management/packwand/installation#which-install-path-should-you-choose",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Additional options",
    "pageTitle": "Additional options",
    "sectionTitle": "",
    "description": "",
    "content": "Additional options Additional options can be configured in the [options] section of pack.toml, as follows: acceptable game versions A list of additional Minecraft versions to accept when installing or updating mods (see Adding mods) acceptable game loaders A list of additional mod loaders to accept when installing or updating mods, beyond those implied by the pack's [versions] (quilt already accepts fabric mods, and neoforge accepts forge mods) meta folder The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used) mods folder is deprecated; aliased to meta folder meta folder base The base folder from which meta folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour) no internal hashes If this is set to true, packwand will not generate hashes of local files, to prevent merge conflicts and inconsistent hashes when using git/etc. packwand refresh build can be used in this mode to generate internal hashes for distributing the pack with packwiz installer datapack folder The folder in which datapacks are to be added; specific to the datapack loader mod you use, and must be set to add datapacks (that are not bundled as mods) Scripts Packs can define runnable scripts in a [scripts] section of pack.toml, executed with packwand run : Global configuration These are set in packwand's own config file (.packwand.toml in your platform config directory) or via flags/environment, not in pack.toml: cache.directory Overrides the download cache location (also the cache global flag) github.token A GitHub API token, to avoid rate limits when installing/updating GitHub mods gitlab.token / gitlab. .token GitLab API token(s) forgejo.token / forgejo. .token Forgejo/Gitea/Codeberg API token(s) Environment variables PACKWAND CONCURRENCY Cap on parallel workers for workspace operations (SOMNUS CONCURRENCY is still honored for existing automation) PACKWAND NETWORK CONCURRENCY Cap on parallel API/download requests PACKWAND HASH CONCURRENCY Cap on parallel local hashing PACKWAND CACHE SLOTS Cap on concurrent export operations against the pack cache PACKWAND BIN Path to the packwand binary, used by tooling that shells out to packwand (PACKWIZ BIN is deprecated but still honored) MODPACKS DIR Overrides the workspace pack root (default modpacks)",
    "url": "/wiki/modpack-management/packwand/reference/additional-options",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Additional options / Environment variables",
    "pageTitle": "Additional options",
    "sectionTitle": "Environment variables",
    "description": "",
    "content": "PACKWAND CONCURRENCY Cap on parallel workers for workspace operations (SOMNUS CONCURRENCY is still honored for existing automation) PACKWAND NETWORK CONCURRENCY Cap on parallel API/download requests PACKWAND HASH CONCURRENCY Cap on parallel local hashing PACKWAND CACHE SLOTS Cap on concurrent export operations against the pack cache PACKWAND BIN Path to the packwand binary, used by tooling that shells out to packwand (PACKWIZ BIN is deprecated but still honored) MODPACKS DIR Overrides the workspace pack root (default modpacks)",
    "url": "/wiki/modpack-management/packwand/reference/additional-options#environment-variables",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Additional options / Global configuration",
    "pageTitle": "Additional options",
    "sectionTitle": "Global configuration",
    "description": "",
    "content": "These are set in packwand's own config file (.packwand.toml in your platform config directory) or via flags/environment, not in pack.toml: cache.directory Overrides the download cache location (also the cache global flag) github.token A GitHub API token, to avoid rate limits when installing/updating GitHub mods gitlab.token / gitlab. .token GitLab API token(s) forgejo.token / forgejo. .token Forgejo/Gitea/Codeberg API token(s)",
    "url": "/wiki/modpack-management/packwand/reference/additional-options#global-configuration",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Additional options / Scripts",
    "pageTitle": "Additional options",
    "sectionTitle": "Scripts",
    "description": "",
    "content": "Packs can define runnable scripts in a [scripts] section of pack.toml, executed with packwand run :",
    "url": "/wiki/modpack-management/packwand/reference/additional-options#scripts",
    "tags": []
  },
  {
    "kind": "page",
    "title": "index.toml",
    "pageTitle": "index.toml",
    "sectionTitle": "",
    "description": "",
    "content": "index.toml The index file of the modpack, storing references to every file to be downloaded (or verified) in the pack. hash format String, required. The default hash format for every file in the index. If missing, consumers assume sha512. packwand writes sha512; when it loads an index using an older format it transparently upgrades the index to sha512 on the next packwand refresh. [[files]] Array of tables, optional (defaults to an empty list). One entry per file in the pack. Key Type Description file path, required The path to the file, relative to the index file, in forward slash format. hash string The hash of the file, in the index's hash format (or this entry's override). May be omitted when no internal hashes is enabled. hash format string Overrides the index level hash format for this file only. Omitted when equal to the index's format, to save space. metafile boolean, default false True when this entry points to a .pw.toml metadata file, which references a file outside the pack. preserve boolean, default false When true, the file is not overwritten on update if it already exists, preserving user changes. alias string The name with which this file should be downloaded, instead of the filename in file. Not compatible with metafile. Multiple entries may share the same file with different aliases. Entries are sorted by file (then alias) when packwand writes the index, so diffs stay stable under version control. Ignored files Files matching the pack's .packwizignore rules (or the built in defaults) are never added to the index. The pack file, the index itself, and .packwizignore are always excluded. Example",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml",
    "tags": []
  },
  {
    "kind": "section",
    "title": "index.toml / Example",
    "pageTitle": "index.toml",
    "sectionTitle": "Example",
    "description": "",
    "content": "Example",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml#example",
    "tags": []
  },
  {
    "kind": "section",
    "title": "index.toml / [[files]]",
    "pageTitle": "index.toml",
    "sectionTitle": "[[files]]",
    "description": "",
    "content": "Array of tables, optional (defaults to an empty list). One entry per file in the pack. Key Type Description file path, required The path to the file, relative to the index file, in forward slash format. hash string The hash of the file, in the index's hash format (or this entry's override). May be omitted when no internal hashes is enabled. hash format string Overrides the index level hash format for this file only. Omitted when equal to the index's format, to save space. metafile boolean, default false True when this entry points to a .pw.toml metadata file, which references a file outside the pack. preserve boolean, default false When true, the file is not overwritten on update if it already exists, preserving user changes. alias string The name with which this file should be downloaded, instead of the filename in file. Not compatible with metafile. Multiple entries may share the same file with different aliases. Entries are sorted by file (then alias) when packwand writes the index, so diffs stay stable under version control.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml#files",
    "tags": []
  },
  {
    "kind": "section",
    "title": "index.toml / hash-format",
    "pageTitle": "index.toml",
    "sectionTitle": "hash-format",
    "description": "",
    "content": "String, required. The default hash format for every file in the index. If missing, consumers assume sha512. packwand writes sha512; when it loads an index using an older format it transparently upgrades the index to sha512 on the next packwand refresh.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml#hash-format",
    "tags": []
  },
  {
    "kind": "section",
    "title": "index.toml / Ignored files",
    "pageTitle": "index.toml",
    "sectionTitle": "Ignored files",
    "description": "",
    "content": "Files matching the pack's .packwizignore rules (or the built in defaults) are never added to the index. The pack file, the index itself, and .packwizignore are always excluded.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/index-toml#ignored-files",
    "tags": []
  },
  {
    "kind": "page",
    "title": "manifest.json",
    "pageTitle": "manifest.json",
    "sectionTitle": "",
    "description": "",
    "content": "manifest.json The packwand manifest is the root metadata file for a pack directory. It stores the pack's identity, loader/version matrix, publishing identifiers, role, lifecycle, and automation settings. packwand reads and writes manifest.json in each pack directory. Commands such as packwand new, packwand validate, packwand publish, packwand automation, and the workspace operations all treat it as the pack's source of truth. Required fields id Unique pack identifier, usually the directory name name Human readable pack name type Pack kind, such as modpack, datapack, or resourcepack role Pack role, usually none, base, or a consumer/base mapping object Common fields loader Primary loader for the pack mc version Primary Minecraft version for the pack variants Optional variant list for multi version packs version Pack release version release type Release channel label used by publish workflows description Short pack description $schema Optional schema URL for editor tooling modrinth id, curseforge id, github id, gitea id, gitlab id External publishing identifiers shared assets Shared asset path used by base/consumer pack layouts lifecycle Pack maintenance state: active, maintenance, archived, or eol Variants Each entry in variants is an object with: id Optional variant identifier name Optional display name mc version Minecraft version for that variant loader Optional loader override for that variant version Optional variant specific pack version Role role is deliberately flexible so the pack can describe both simple and workspace aware setups. \"none\" is the default for ordinary packs \"base\" marks a performance base pack { \"performance base\": { \"pack\": \"...\", \"mappings\": [...] } } marks a consumer pack that syncs content from a base pack Automation automation is optional. It controls unattended update and release behavior. auto update enables or disables automatic update flows server promo marks a pack for server promotion workflows sync exclude lists paths to skip during workspace sync freeze maps subdirs to frozen mod slugs that should not update full auto.enabled opts into the end to end packwand automation run pipeline full auto.tests is an optional list of shell commands run before the manifest version bump Example",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json",
    "tags": []
  },
  {
    "kind": "section",
    "title": "manifest.json / Automation",
    "pageTitle": "manifest.json",
    "sectionTitle": "Automation",
    "description": "",
    "content": "automation is optional. It controls unattended update and release behavior. auto update enables or disables automatic update flows server promo marks a pack for server promotion workflows sync exclude lists paths to skip during workspace sync freeze maps subdirs to frozen mod slugs that should not update full auto.enabled opts into the end to end packwand automation run pipeline full auto.tests is an optional list of shell commands run before the manifest version bump",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json#automation",
    "tags": []
  },
  {
    "kind": "section",
    "title": "manifest.json / Common fields",
    "pageTitle": "manifest.json",
    "sectionTitle": "Common fields",
    "description": "",
    "content": "loader Primary loader for the pack mc version Primary Minecraft version for the pack variants Optional variant list for multi version packs version Pack release version release type Release channel label used by publish workflows description Short pack description $schema Optional schema URL for editor tooling modrinth id, curseforge id, github id, gitea id, gitlab id External publishing identifiers shared assets Shared asset path used by base/consumer pack layouts lifecycle Pack maintenance state: active, maintenance, archived, or eol",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json#common-fields",
    "tags": []
  },
  {
    "kind": "section",
    "title": "manifest.json / Example",
    "pageTitle": "manifest.json",
    "sectionTitle": "Example",
    "description": "",
    "content": "Example",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json#example",
    "tags": []
  },
  {
    "kind": "section",
    "title": "manifest.json / Required fields",
    "pageTitle": "manifest.json",
    "sectionTitle": "Required fields",
    "description": "",
    "content": "id Unique pack identifier, usually the directory name name Human readable pack name type Pack kind, such as modpack, datapack, or resourcepack role Pack role, usually none, base, or a consumer/base mapping object",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json#required-fields",
    "tags": []
  },
  {
    "kind": "section",
    "title": "manifest.json / Role",
    "pageTitle": "manifest.json",
    "sectionTitle": "Role",
    "description": "",
    "content": "role is deliberately flexible so the pack can describe both simple and workspace aware setups. \"none\" is the default for ordinary packs \"base\" marks a performance base pack { \"performance base\": { \"pack\": \"...\", \"mappings\": [...] } } marks a consumer pack that syncs content from a base pack",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json#role",
    "tags": []
  },
  {
    "kind": "section",
    "title": "manifest.json / Variants",
    "pageTitle": "manifest.json",
    "sectionTitle": "Variants",
    "description": "",
    "content": "Each entry in variants is an object with: id Optional variant identifier name Optional display name mc version Minecraft version for that variant loader Optional loader override for that variant version Optional variant specific pack version",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/manifest-json#variants",
    "tags": []
  },
  {
    "kind": "page",
    "title": "mod.pw.toml",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "",
    "description": "",
    "content": "mod.pw.toml A metadata file which references an external file from a URL (or a metadata based downloader). This allows for side only mods, optional mods, and pinning, and stores metadata to allow finding updates on Modrinth, CurseForge, GitHub, GitLab, and Forgejo. The \"mod\" terminology is used a lot here, but this works for any file — resource packs, shader packs, datapacks, and plain files. Metadata files use the .pw.toml extension and are marked with metafile = true in the index. name String, required. The name of the mod, displayed in user interfaces. Does not need to be unique. filename Path, required. The destination filename of the downloaded file, relative to this metadata file. side String, default \"both\". The physical Minecraft side this file should be installed on: \"client\" (client and integrated server), \"server\" (dedicated server), or \"both\". An empty string is equivalent to \"both\". pin Boolean, default false. (packwand extension.) When true, the file is pinned: packwand update skips it until it is unpinned (packwand pin / packwand unpin ). [download] Table, required. How to obtain the file. Key Type Description url string The URL to download from. Required when mode is \"url\" or omitted. mode string The download mode. \"url\" (or omitted/empty) downloads from url. \"metadata:curseforge\" resolves the download through the CurseForge API using the [update.curseforge] metadata — required by CurseForge's distribution rules; such files have no url. hash format string, required The hash format of hash. packwand writes sha512 where the source provides it. hash string, required The hash of the file, used for integrity verification. [option] Table, optional. The optional state of this file. When absent, the file is not optional. Key Type Description optional boolean, required, default false Whether the file is optional. description string Shown to the user when selecting optional mods; should explain why they might want it. default boolean, default false Whether the file is enabled by default. If a target pack format does not support optional mods but supports disabled mods, files defaulting to disabled are exported disabled. [update] Table, optional. How tools may update this file. If absent or empty, the file is never auto updated. Each sub table is one update source; if several are defined, the tool chooses one (which one is implementation defined — do not rely on the order). Consumers must fail to load a metadata file that declares an update source they do not recognise. [update.curseforge] Key Type Description project id integer, required The CurseForge project ID. Updating retrieves the latest valid file for this project (matching game version, release channel, and loader). file id integer, required The currently installed file ID. [update.modrinth] Key Type Description mod id string, required The Modrinth project ID. version string, required The currently installed version ID. [update.github] (packwand extension.) Updates from GitHub release assets. Key Type Description slug string, required The repository, as owner/repo. tag string The currently installed release tag. branch string Restrict updates to releases targeting this branch. regex string A regular expression an asset filename must match to be selected. [update.gitlab] (packwand extension.) Updates from GitLab release assets. Key Type Description instance string The GitLab instance hostname; defaults to gitlab.com. slug string, required The project path, as owner/repo. tag string The currently installed release tag. regex string A regular expression an asset filename must match to be selected. [update.forgejo] (packwand extension.) Updates from Forgejo/Gitea release assets (including Codeberg). Key Type Description instance string The Forgejo/Gitea instance hostname; defaults to codeberg.org. slug string, required The repository, as owner/repo. tag string The currently installed release tag. branch string Restrict updates to releases targeting this branch. regex string A regular expression an asset filename must match to be selected. Example A CurseForge metadata mode file:",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [download]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[download]",
    "description": "",
    "content": "Table, required. How to obtain the file. Key Type Description url string The URL to download from. Required when mode is \"url\" or omitted. mode string The download mode. \"url\" (or omitted/empty) downloads from url. \"metadata:curseforge\" resolves the download through the CurseForge API using the [update.curseforge] metadata — required by CurseForge's distribution rules; such files have no url. hash format string, required The hash format of hash. packwand writes sha512 where the source provides it. hash string, required The hash of the file, used for integrity verification.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#download",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / Example",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "Example",
    "description": "",
    "content": "A CurseForge metadata mode file:",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#example",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / filename",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "filename",
    "description": "",
    "content": "Path, required. The destination filename of the downloaded file, relative to this metadata file.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#filename",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / name",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "name",
    "description": "",
    "content": "String, required. The name of the mod, displayed in user interfaces. Does not need to be unique.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#name",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [option]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[option]",
    "description": "",
    "content": "Table, optional. The optional state of this file. When absent, the file is not optional. Key Type Description optional boolean, required, default false Whether the file is optional. description string Shown to the user when selecting optional mods; should explain why they might want it. default boolean, default false Whether the file is enabled by default. If a target pack format does not support optional mods but supports disabled mods, files defaulting to disabled are exported disabled.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#option",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / pin",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "pin",
    "description": "",
    "content": "Boolean, default false. (packwand extension.) When true, the file is pinned: packwand update skips it until it is unpinned (packwand pin / packwand unpin ).",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#pin",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / side",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "side",
    "description": "",
    "content": "String, default \"both\". The physical Minecraft side this file should be installed on: \"client\" (client and integrated server), \"server\" (dedicated server), or \"both\". An empty string is equivalent to \"both\".",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#side",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [update]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[update]",
    "description": "",
    "content": "Table, optional. How tools may update this file. If absent or empty, the file is never auto updated. Each sub table is one update source; if several are defined, the tool chooses one (which one is implementation defined — do not rely on the order). Consumers must fail to load a metadata file that declares an update source they do not recognise.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#update",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [update.curseforge]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[update.curseforge]",
    "description": "",
    "content": "Key Type Description project id integer, required The CurseForge project ID. Updating retrieves the latest valid file for this project (matching game version, release channel, and loader). file id integer, required The currently installed file ID.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#updatecurseforge",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [update.forgejo]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[update.forgejo]",
    "description": "",
    "content": "(packwand extension.) Updates from Forgejo/Gitea release assets (including Codeberg). Key Type Description instance string The Forgejo/Gitea instance hostname; defaults to codeberg.org. slug string, required The repository, as owner/repo. tag string The currently installed release tag. branch string Restrict updates to releases targeting this branch. regex string A regular expression an asset filename must match to be selected.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#updateforgejo",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [update.github]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[update.github]",
    "description": "",
    "content": "(packwand extension.) Updates from GitHub release assets. Key Type Description slug string, required The repository, as owner/repo. tag string The currently installed release tag. branch string Restrict updates to releases targeting this branch. regex string A regular expression an asset filename must match to be selected.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#updategithub",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [update.gitlab]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[update.gitlab]",
    "description": "",
    "content": "(packwand extension.) Updates from GitLab release assets. Key Type Description instance string The GitLab instance hostname; defaults to gitlab.com. slug string, required The project path, as owner/repo. tag string The currently installed release tag. regex string A regular expression an asset filename must match to be selected.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#updategitlab",
    "tags": []
  },
  {
    "kind": "section",
    "title": "mod.pw.toml / [update.modrinth]",
    "pageTitle": "mod.pw.toml",
    "sectionTitle": "[update.modrinth]",
    "description": "",
    "content": "Key Type Description mod id string, required The Modrinth project ID. version string, required The currently installed version ID.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/mod-toml#updatemodrinth",
    "tags": []
  },
  {
    "kind": "page",
    "title": "pack.toml",
    "pageTitle": "pack.toml",
    "sectionTitle": "",
    "description": "",
    "content": "pack.toml The main metadata file for a packwand modpack. This is the first file loaded, so that a modpack downloader can download all the files in the modpack. pack format String, required for new packs. A version string identifying the pack format. packwand writes packwand:26 for new packs. Two families of values are accepted: packwand: — the packwand format. The suffix is a single integer generation number (currently 26). Consumers must fail to load the pack if the generation is not a valid integer. Consumers must fail to load the pack if the generation predates the minimum they support; packwand migrate format upgrades old packs. Consumers should warn (but continue) if the generation is newer than the version they implement. packwiz: — the legacy packwiz format, accepted for backward compatibility. The suffix must be valid semver; versions matching 1 are accepted, and packs with a feature version above 1.1 produce an upgrade suggestion. packwiz:1.0.0 is migrated to packwiz:1.1.0 automatically on load. If the field is missing entirely, consumers assume packwiz:1.1.0 for compatibility with very old packs. name String, required. The name of the modpack. Displayed in user interfaces to identify the pack; does not need to be unique between packs. author String, optional. The author(s) of the modpack. Output when exporting to the CurseForge pack format. version String, optional. The version of the modpack. Output when exporting to CurseForge and Modrinth pack formats. Must not be used to determine whether the modpack is outdated. description String, optional. A short description of the modpack. Output when exporting to the Modrinth pack format. [index] Table, required. Information about the index file of this modpack. Key Type Description file path, required The path to the index file, relative to pack.toml (forward slashes). Defaults to index.toml when empty. hash format string, required The hash format of the index hash. packwand writes sha512. hash string The hash of the index file. May be omitted when no internal hashes is enabled. [versions] Table of strings, required. The versions of components used by this modpack — Minecraft and the mod loader(s). The existence of a component implies it should be installed; tools also use these values to decide which mod versions are compatible. Key Description Example minecraft Required. The Minecraft version, in the format used by version.json files. \"1.20.1\", \"26.1.2\" fabric The Fabric loader version. \"0.16.9\" forge The Forge version, without the Minecraft version prefix. \"14.23.5.2838\" neoforge The NeoForge version. \"21.1.77\" quilt The Quilt loader version. \"0.27.0\" liteloader The LiteLoader version. \"1.12.2 SNAPSHOT\" Additional string keys are permitted. A pack with quilt is also considered compatible with fabric mods, and a pack with neoforge is also considered compatible with forge mods. [options] Table, optional. Tool configuration read at load time; see Additional options. Keys include acceptable game versions, acceptable game loaders, meta folder, meta folder base, no internal hashes, and datapack folder. [scripts] Table of strings, optional. (packwand extension, not in packwiz.) Named commands runnable with packwand run : [export] Table of tables, optional. Per platform export configuration, e.g. [export.curseforge] and [export.modrinth] settings used by the corresponding export commands. Hash formats All hash values in the pack are lowercase strings. Consumers must support: Format Notes sha512 Default. Used by packwand for all new files and index entries. sha256 Used as the download cache key format. sha1 Legacy; provided by some remote APIs. md5 Legacy; provided by some remote APIs. murmur2 The CurseForge variant: 32 bit MurmurHash2 (seed 1) with whitespace bytes (9, 10, 13, 32) removed before hashing, stored as an unsigned decimal integer. Example",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / author",
    "pageTitle": "pack.toml",
    "sectionTitle": "author",
    "description": "",
    "content": "String, optional. The author(s) of the modpack. Output when exporting to the CurseForge pack format.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#author",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / description",
    "pageTitle": "pack.toml",
    "sectionTitle": "description",
    "description": "",
    "content": "String, optional. A short description of the modpack. Output when exporting to the Modrinth pack format.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#description",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / Example",
    "pageTitle": "pack.toml",
    "sectionTitle": "Example",
    "description": "",
    "content": "Example",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#example",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / [export]",
    "pageTitle": "pack.toml",
    "sectionTitle": "[export]",
    "description": "",
    "content": "Table of tables, optional. Per platform export configuration, e.g. [export.curseforge] and [export.modrinth] settings used by the corresponding export commands.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#export",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / Hash formats",
    "pageTitle": "pack.toml",
    "sectionTitle": "Hash formats",
    "description": "",
    "content": "All hash values in the pack are lowercase strings. Consumers must support: Format Notes sha512 Default. Used by packwand for all new files and index entries. sha256 Used as the download cache key format. sha1 Legacy; provided by some remote APIs. md5 Legacy; provided by some remote APIs. murmur2 The CurseForge variant: 32 bit MurmurHash2 (seed 1) with whitespace bytes (9, 10, 13, 32) removed before hashing, stored as an unsigned decimal integer.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#hash-formats",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / [index]",
    "pageTitle": "pack.toml",
    "sectionTitle": "[index]",
    "description": "",
    "content": "Table, required. Information about the index file of this modpack. Key Type Description file path, required The path to the index file, relative to pack.toml (forward slashes). Defaults to index.toml when empty. hash format string, required The hash format of the index hash. packwand writes sha512. hash string The hash of the index file. May be omitted when no internal hashes is enabled.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#index",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / name",
    "pageTitle": "pack.toml",
    "sectionTitle": "name",
    "description": "",
    "content": "String, required. The name of the modpack. Displayed in user interfaces to identify the pack; does not need to be unique between packs.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#name",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / [options]",
    "pageTitle": "pack.toml",
    "sectionTitle": "[options]",
    "description": "",
    "content": "Table, optional. Tool configuration read at load time; see Additional options. Keys include acceptable game versions, acceptable game loaders, meta folder, meta folder base, no internal hashes, and datapack folder.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#options",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / pack-format",
    "pageTitle": "pack.toml",
    "sectionTitle": "pack-format",
    "description": "",
    "content": "String, required for new packs. A version string identifying the pack format. packwand writes packwand:26 for new packs. Two families of values are accepted: packwand: — the packwand format. The suffix is a single integer generation number (currently 26). Consumers must fail to load the pack if the generation is not a valid integer. Consumers must fail to load the pack if the generation predates the minimum they support; packwand migrate format upgrades old packs. Consumers should warn (but continue) if the generation is newer than the version they implement. packwiz: — the legacy packwiz format, accepted for backward compatibility. The suffix must be valid semver; versions matching 1 are accepted, and packs with a feature version above 1.1 produce an upgrade suggestion. packwiz:1.0.0 is migrated to packwiz:1.1.0 automatically on load. If the field is missing entirely, consumers assume packwiz:1.1.0 for compatibility with very old packs.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#pack-format",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / [scripts]",
    "pageTitle": "pack.toml",
    "sectionTitle": "[scripts]",
    "description": "",
    "content": "Table of strings, optional. (packwand extension, not in packwiz.) Named commands runnable with packwand run :",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#scripts",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / version",
    "pageTitle": "pack.toml",
    "sectionTitle": "version",
    "description": "",
    "content": "String, optional. The version of the modpack. Output when exporting to CurseForge and Modrinth pack formats. Must not be used to determine whether the modpack is outdated.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#version",
    "tags": []
  },
  {
    "kind": "section",
    "title": "pack.toml / [versions]",
    "pageTitle": "pack.toml",
    "sectionTitle": "[versions]",
    "description": "",
    "content": "Table of strings, required. The versions of components used by this modpack — Minecraft and the mod loader(s). The existence of a component implies it should be installed; tools also use these values to decide which mod versions are compatible. Key Description Example minecraft Required. The Minecraft version, in the format used by version.json files. \"1.20.1\", \"26.1.2\" fabric The Fabric loader version. \"0.16.9\" forge The Forge version, without the Minecraft version prefix. \"14.23.5.2838\" neoforge The NeoForge version. \"21.1.77\" quilt The Quilt loader version. \"0.27.0\" liteloader The LiteLoader version. \"1.12.2 SNAPSHOT\" Additional string keys are permitted. A pack with quilt is also considered compatible with fabric mods, and a pack with neoforge is also considered compatible with forge mods.",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/pack-toml#versions",
    "tags": []
  },
  {
    "kind": "page",
    "title": ".packwizignore",
    "pageTitle": ".packwizignore",
    "sectionTitle": "",
    "description": "",
    "content": ".packwizignore .packwizignore is an optional file at the root of a pack that excludes files from the pack index, using the same format as gitignore. Place patterns in it (one per line) and run packwand refresh; matching files are not added to index.toml and are not distributed with the pack. The pack file (pack.toml), the index file, and .packwizignore itself are always excluded. Default rules The following defaults are always applied, whether or not a .packwizignore file exists. They can be overridden with a negating pattern (preceded with !):",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/packwizignore",
    "tags": []
  },
  {
    "kind": "section",
    "title": ".packwizignore / Default rules",
    "pageTitle": ".packwizignore",
    "sectionTitle": "Default rules",
    "description": "",
    "content": "The following defaults are always applied, whether or not a .packwizignore file exists. They can be overridden with a negating pattern (preceded with !):",
    "url": "/wiki/modpack-management/packwand/reference/pack-format/packwizignore#default-rules",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Adding mods and resource packs",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "",
    "description": "",
    "content": "Adding mods and resource packs To add external files to your modpack, such as mods and resource packs, you'll need .pw.toml metadata files to define how to download them. The modrinth install and curseforge install commands can automatically create these for you with all the necessary metadata! CurseForge and Modrinth Mods and resource packs from CurseForge and Modrinth can be easily added with the modrinth install and curseforge install commands. They can also be updated with the packwand update command; pass all to update all your mods at once. Mods can be passed in multiple forms to these commands: packwand curseforge install indium (by slug) packwand curseforge install category texture packs unity (by slug; category and game can be specified with the corresponding flags) packwand curseforge install https://www.curseforge.com/minecraft/mc mods/indium (by mod page URL) packwand curseforge install https://www.curseforge.com/minecraft/mc mods/indium/files/3535202 (by file page URL) packwand curseforge install Indium (by search) packwand curseforge install addon id 459496 file id 3535202 (if all else fails) packwand modrinth install indium (by slug) packwand modrinth install https://modrinth.com/mod/indium (by mod page URL) packwand modrinth install https://modrinth.com/mod/indium/version/mfNlBb6U (by file page URL) packwand modrinth install Fabric Rendering Sodium (by search) packwand modrinth install Orvt0mRa (by ID) Dependencies are automatically picked up for you if you don't have them already, you'll be prompted whether you want to install them. packwand also checks if your mods are being installed for the wrong version; but you can tell it to allow more versions using the acceptable game versions field in pack.toml. Just add the following to the bottom of pack.toml, replacing the versions listed here with those you want to allow: ::: tip Several aliases exist for the curseforge and modrinth commands to speed up your workflow. Try packwand cf add or packwand mr add! ::: GitHub, GitLab, and Forgejo packwand can also install mods directly from software forges, downloading release assets and keeping them updated: packwand github install owner/repo (or a full GitHub URL) packwand gitlab install owner/repo (defaults to gitlab.com; other instances via URL) packwand forgejo install owner/repo (defaults to codeberg.org; works with any Forgejo/Gitea instance URL) Internal files (config files, scripts, etc.) Configuration files for your modpack can simply be placed in a config folder (in the same place as the mods folder) and they'll be copied to the config folder when installing the modpack. This works for any file (including quests/scripts) place it in the modpack and it'll be installed into the corresponding location in the game folder. Make sure you run packwand refresh so that the index is up to date! This works for mods that aren't available elsewhere online too (e.g. custom mods or forks); just drop them in the mods folder alongside the .pw.toml files. This isn't ideal for Git as it's not great at handling large binary files; you could use Git LFS or you may prefer to upload them elsewhere manually and reference them from the pack see the section below. ::: tip If you don't want to include files in the modpack, you can add them to a file called .packwizignore in your modpack directory. This uses the same format as gitignore; see the .packwizignore reference for the defaults that are always applied. ::: Other external files If you have external files/mods that aren't from CurseForge or Modrinth, you'll need to create the .pw.toml files manually. See the following for an example of how you could lay it out: You can even create them for files that aren't mods (such as resource packs) just make sure to use the .pw.toml extension and run packwand refresh, so that packwand knows that the file contains metadata.",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / CurseForge and Modrinth",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "CurseForge and Modrinth",
    "description": "",
    "content": "Mods and resource packs from CurseForge and Modrinth can be easily added with the modrinth install and curseforge install commands. They can also be updated with the packwand update command; pass all to update all your mods at once. Mods can be passed in multiple forms to these commands: packwand curseforge install indium (by slug) packwand curseforge install category texture packs unity (by slug; category and game can be specified with the corresponding flags) packwand curseforge install https://www.curseforge.com/minecraft/mc mods/indium (by mod page URL) packwand curseforge install https://www.curseforge.com/minecraft/mc mods/indium/files/3535202 (by file page URL) packwand curseforge install Indium (by search) packwand curseforge install addon id 459496 file id 3535202 (if all else fails) packwand modrinth install indium (by slug) packwand modrinth install https://modrinth.com/mod/indium (by mod page URL) packwand modrinth install https://modrinth.com/mod/indium/version/mfNlBb6U (by file page URL) packwand modrinth install Fabric Rendering Sodium (by search) packwand modrinth install Orvt0mRa (by ID) Dependencies are automatically picked up for you if you don't have them already, you'll be prompted whether you want to install them. packwand also checks if your mods are being installed for the wrong version; but you can tell it to allow more versions using the acceptable game versions field in pack.toml. Just add the following to the bottom of pack.toml, replacing the versions listed here with those you want to allow: ::: tip Several aliases exist for the curseforge and modrinth commands to speed up your workflow. Try packwand cf add or packwand mr add! :::",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods#curseforge-and-modrinth",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / GitHub, GitLab, and Forgejo",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "GitHub, GitLab, and Forgejo",
    "description": "",
    "content": "packwand can also install mods directly from software forges, downloading release assets and keeping them updated: packwand github install owner/repo (or a full GitHub URL) packwand gitlab install owner/repo (defaults to gitlab.com; other instances via URL) packwand forgejo install owner/repo (defaults to codeberg.org; works with any Forgejo/Gitea instance URL)",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods#github-gitlab-and-forgejo",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / Internal files (config files, scripts, etc.)",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "Internal files (config files, scripts, etc.)",
    "description": "",
    "content": "Configuration files for your modpack can simply be placed in a config folder (in the same place as the mods folder) and they'll be copied to the config folder when installing the modpack. This works for any file (including quests/scripts) place it in the modpack and it'll be installed into the corresponding location in the game folder. Make sure you run packwand refresh so that the index is up to date! This works for mods that aren't available elsewhere online too (e.g. custom mods or forks); just drop them in the mods folder alongside the .pw.toml files. This isn't ideal for Git as it's not great at handling large binary files; you could use Git LFS or you may prefer to upload them elsewhere manually and reference them from the pack see the section below. ::: tip If you don't want to include files in the modpack, you can add them to a file called .packwizignore in your modpack directory. This uses the same format as gitignore; see the .packwizignore reference for the defaults that are always applied. :::",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods#internal-files-config-files-scripts-etc",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / Other external files",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "Other external files",
    "description": "",
    "content": "If you have external files/mods that aren't from CurseForge or Modrinth, you'll need to create the .pw.toml files manually. See the following for an example of how you could lay it out: You can even create them for files that aren't mods (such as resource packs) just make sure to use the .pw.toml extension and run packwand refresh, so that packwand knows that the file contains metadata.",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/adding-mods#other-external-files",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Getting started",
    "pageTitle": "Getting started",
    "sectionTitle": "",
    "description": "",
    "content": "Getting started To use the packwand CLI, first you'll want to create a folder to develop your modpack in. This should not be the same as your .minecraft or a MultiMC instance folder; this folder holds metadata and files for your modpack so it can be managed by packwand. Then open your command line (Command Prompt/Terminal) and use the cd command to move into the folder you created. Creating a new modpack To create the files for your new modpack, just run packwand init in the folder you created! It'll ask you for a few details, then create a pack.toml and index.toml based on your answers. pack.toml is the main file of your modpack and defines several crucial details; including the name of your modpack, the version of Minecraft and the version of the mod loader you're using. Optionally, you can include a version (required for exporting to Modrinth packs) and a description for your modpack. index.toml is the index of your modpack which lists the files in your modpack with their hashes (for integrity checking). You're unlikely to need to touch this yourself, but you'll need to run the packwand refresh command when you manually add, remove or edit files in the pack. Importing an existing modpack Have an existing CurseForge modpack? You can use the packwand curseforge import command with the path to the modpack .zip file, which will import all the mods and files from the pack into your current directory. If this isn't your own modpack, please make sure you have permission (or a license) to redistribute the modpack you import! ::: warning If you have existing files in your modpack, importing will overwrite them. It's a good idea to use version control systems (such as Git) with packwand! ::: Cheat Sheet You'll get more information in the tutorials following this one (and the reference pages), but this is a quick summary of the most useful commands: packwand init creates a modpack in the current folder packwand curseforge import [zip path] imports a CurseForge modpack packwand refresh updates the modpack index packwand curseforge install [mod] installs a mod from CurseForge packwand modrinth install [mod] installs a mod from Modrinth packwand update [mod] updates a mod packwand update all updates all the mods in the modpack packwand curseforge export exports the modpack in the format supported by the CurseForge Launcher packwand modrinth export exports the modpack in the format supported by Modrinth packwand curseforge detect to detect files that are available on CurseForge and make them downloaded from there packwand workspace status shows every pack in a multi pack repository Use the help flag for more information about any command!",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/getting-started",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Getting started / Cheat Sheet",
    "pageTitle": "Getting started",
    "sectionTitle": "Cheat Sheet",
    "description": "",
    "content": "You'll get more information in the tutorials following this one (and the reference pages), but this is a quick summary of the most useful commands: packwand init creates a modpack in the current folder packwand curseforge import [zip path] imports a CurseForge modpack packwand refresh updates the modpack index packwand curseforge install [mod] installs a mod from CurseForge packwand modrinth install [mod] installs a mod from Modrinth packwand update [mod] updates a mod packwand update all updates all the mods in the modpack packwand curseforge export exports the modpack in the format supported by the CurseForge Launcher packwand modrinth export exports the modpack in the format supported by Modrinth packwand curseforge detect to detect files that are available on CurseForge and make them downloaded from there packwand workspace status shows every pack in a multi pack repository Use the help flag for more information about any command!",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/getting-started#cheat-sheet",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Getting started / Creating a new modpack",
    "pageTitle": "Getting started",
    "sectionTitle": "Creating a new modpack",
    "description": "",
    "content": "To create the files for your new modpack, just run packwand init in the folder you created! It'll ask you for a few details, then create a pack.toml and index.toml based on your answers. pack.toml is the main file of your modpack and defines several crucial details; including the name of your modpack, the version of Minecraft and the version of the mod loader you're using. Optionally, you can include a version (required for exporting to Modrinth packs) and a description for your modpack. index.toml is the index of your modpack which lists the files in your modpack with their hashes (for integrity checking). You're unlikely to need to touch this yourself, but you'll need to run the packwand refresh command when you manually add, remove or edit files in the pack.",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/getting-started#creating-a-new-modpack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Getting started / Importing an existing modpack",
    "pageTitle": "Getting started",
    "sectionTitle": "Importing an existing modpack",
    "description": "",
    "content": "Have an existing CurseForge modpack? You can use the packwand curseforge import command with the path to the modpack .zip file, which will import all the mods and files from the pack into your current directory. If this isn't your own modpack, please make sure you have permission (or a license) to redistribute the modpack you import! ::: warning If you have existing files in your modpack, importing will overwrite them. It's a good idea to use version control systems (such as Git) with packwand! :::",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/getting-started#importing-an-existing-modpack",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Using packwand with Git",
    "pageTitle": "Using packwand with Git",
    "sectionTitle": "",
    "description": "",
    "content": "Using packwand with Git On Windows, line ending conversion causes the hashes to change when files are uploaded to Git, so you'll get invalid hash errors when trying to install the pack. You'll want to add a .gitattributes file to disable line ending conversion. See the example pack for example .gitignore and .gitattributes files! If you have existing files committed to Git, you'll need to run git add renormalize . to reset line ending conversion after adding .gitattributes. You'll also want a .packwizignore file so that Git metadata isn't included in the pack index — though packwand ignores .git/ , .gitignore, and .gitattributes by default.",
    "url": "/wiki/modpack-management/packwand/tutorials/creating/git",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Publishing to CurseForge",
    "pageTitle": "Publishing to CurseForge",
    "sectionTitle": "",
    "description": "",
    "content": "Publishing to CurseForge Exporting a CurseForge pack is as simple as running packwand curseforge export this gives you a .zip in your pack folder that you can upload to CurseForge! Since this pack format doesn't support side only mods, packwand can't create a pack that differs between server and client. You can use the side flag to specify which mods should be exported by default it exports a pack for Minecraft clients (containing mods with side client or both). Mods without the necessary CurseForge metadata (such as those installed from Modrinth) will be placed as JARs into the modpack zip; these must be approved manually by CurseForge staff. Be wary of including files that you don't want (the packwand executable, and the modpack zip itself) in the pack! packwand's default ignore rules exclude the executable and .zip files at the pack root. The CurseForge pack format doesn't really support optional mods. The user won't be prompted about optional mods, but if they default to being disabled they will be disabled in the CurseForge launcher. See the corresponding reference page for the full flag list.",
    "url": "/wiki/modpack-management/packwand/tutorials/hosting/curseforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Publishing to Modrinth",
    "pageTitle": "Publishing to Modrinth",
    "sectionTitle": "",
    "description": "",
    "content": "Publishing to Modrinth Exporting a Modrinth pack is as simple as running packwand modrinth export this gives you a .mrpack in your pack folder that you can upload to Modrinth! Unlike CurseForge, this pack format does support side only mods. When exporting, packwand will export a pack with side information provided from Modrinth or as specified in the mod's .pw.toml file. The official Modrinth launcher will automatically filter out serverside mods, and the pack can be used on the server using tools like mrpack install or packwiz installer. Mods without the necessary Modrinth metadata (such as those installed from CurseForge) will be placed as JARs into the modpack zip; make sure that you have the licenses for these mods as it is your responsibility as a pack creator. Be wary of including files that you don't want (the packwand executable, and the modpack zip itself) in the pack! packwand's default ignore rules exclude the executable and .mrpack files. Keep in mind that since the official Modrinth app doesn't support optional mods, the user won't be prompted for optional mods. The official launcher will automatically install all optional mods (even if they default to being disabled!). If you'd like to be able to use optional mods, use Prism Launcher's \"Import Instance\" section to install the exported .mrpack file. See the corresponding reference page for the full flag list.",
    "url": "/wiki/modpack-management/packwand/tutorials/hosting/modrinth",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Pack Installation using packwiz-installer",
    "pageTitle": "Pack Installation using packwiz-installer",
    "sectionTitle": "",
    "description": "",
    "content": "Pack Installation using packwiz installer packwiz installer is a Java based installer that allows for automatic installation and updates of packwiz format packs (including packwand packs)! It can be used with MultiMC/Prism/ATLauncher as a prelaunch task, or on servers as part of your start script, and supports side only mods as well as optional mods with a fancy GUI. To distribute a modpack, you'll first want to set up a web hosting service (such as Netlify, GitHub Pages, GitLab Pages) so that your pack files are accessible from a HTTP/HTTPS link. For testing, you can use the packwand serve command to run a local HTTP server, that serves your pack at http://localhost:8080/pack.toml it'll refresh the index whenever it's queried so you don't need to refresh it manually! Creating a MultiMC instance for your modpack To distribute the modpack as a MultiMC instance: 1. Create a barebones MultiMC instance, with the modloader and Minecraft version you want (memory allocation overrides are also a good idea) 2. Download packwiz installer bootstrap from https://github.com/packwiz/packwiz installer bootstrap/releases and place it in the instance Minecraft folder ::: info This is the same folder as options.txt MultiMC will call it .minecraft or minecraft depending on your system. ::: 3. Go to Edit Instance Settings Custom commands, then check the Custom Commands box and paste the following command into the pre launch command field: \"$INST JAVA\" jar packwiz installer bootstrap.jar https://[your server]/pack.toml (where https://[your server]/pack.toml is the HTTP URL your pack.toml file is hosted at) 4. Use the Export Instance function to export your pack as a .zip file (which can be distributed similarly to your pack via a web hosting service) To install your pack, users just need to add it with Add instance Import from zip then packwiz installer does the rest, keeping it up to date every time the game is launched! Using a modpack with a server You can use packwiz installer to download non client mods (side either both or server), for example: java jar packwiz installer bootstrap.jar g s server https://[your server]/pack.toml g flag to disable the GUI s server to download only server side mods. itzg's docker minecraft server has built in support for packwiz format packs. You can pass the PACKWIZ URL environment variable pointing to your pack's TOML file, and the container will bootstrap packwiz installer and install/update the provided pack. See the documentation for more information. ::: tip For local validation, packwand test spins up packwand serve and runs packwiz installer against it automatically (requires Java; the bootstrap jar is downloaded into Packwand's cache automatically, with PACKWAND INSTALLER JAR available as an override). :::",
    "url": "/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Installation using packwiz-installer / Creating a MultiMC instance for your modpack",
    "pageTitle": "Pack Installation using packwiz-installer",
    "sectionTitle": "Creating a MultiMC instance for your modpack",
    "description": "",
    "content": "To distribute the modpack as a MultiMC instance: 1. Create a barebones MultiMC instance, with the modloader and Minecraft version you want (memory allocation overrides are also a good idea) 2. Download packwiz installer bootstrap from https://github.com/packwiz/packwiz installer bootstrap/releases and place it in the instance Minecraft folder ::: info This is the same folder as options.txt MultiMC will call it .minecraft or minecraft depending on your system. ::: 3. Go to Edit Instance Settings Custom commands, then check the Custom Commands box and paste the following command into the pre launch command field: \"$INST JAVA\" jar packwiz installer bootstrap.jar https://[your server]/pack.toml (where https://[your server]/pack.toml is the HTTP URL your pack.toml file is hosted at) 4. Use the Export Instance function to export your pack as a .zip file (which can be distributed similarly to your pack via a web hosting service) To install your pack, users just need to add it with Add instance Import from zip then packwiz installer does the rest, keeping it up to date every time the game is launched!",
    "url": "/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer#creating-a-multimc-instance-for-your-modpack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Installation using packwiz-installer / Using a modpack with a server",
    "pageTitle": "Pack Installation using packwiz-installer",
    "sectionTitle": "Using a modpack with a server",
    "description": "",
    "content": "You can use packwiz installer to download non client mods (side either both or server), for example: java jar packwiz installer bootstrap.jar g s server https://[your server]/pack.toml g flag to disable the GUI s server to download only server side mods. itzg's docker minecraft server has built in support for packwiz format packs. You can pass the PACKWIZ URL environment variable pointing to your pack's TOML file, and the container will bootstrap packwiz installer and install/update the provided pack. See the documentation for more information. ::: tip For local validation, packwand test spins up packwand serve and runs packwiz installer against it automatically (requires Java; the bootstrap jar is downloaded into Packwand's cache automatically, with PACKWAND INSTALLER JAR available as an override). :::",
    "url": "/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer#using-a-modpack-with-a-server",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Build & Export",
    "pageTitle": "packwand",
    "sectionTitle": "Build & Export",
    "description": "",
    "content": "build Build modpack exports and zip packs from git changed targets bump Bump the manifest version export Export packs locally publish Build, upload, verify, or list publish targets for a pack",
    "url": "/wiki/modpack-management/packwand#build-export",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Diagnostics",
    "pageTitle": "packwand",
    "sectionTitle": "Diagnostics",
    "description": "",
    "content": "content lint Lint pack content doctor Check that tools, repo root, and manifests are healthy lint Check JSON and .pw.toml files for syntax errors list List all the mods in the modpack test Spin up packwand serve and validate a pack with packwiz installer validate Validate pack manifests version Print the packwand version",
    "url": "/wiki/modpack-management/packwand#diagnostics",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Flags",
    "pageTitle": "packwand",
    "sectionTitle": "Flags",
    "description": "",
    "content": "cache Override the shared download cache directory config Select the packwand config file meta folder Change where new metadata files are written meta folder base Resolve meta folder relative to another base directory no refresh Skip index and pack.toml refresh after modifications pack file Select the pack metadata file y, yes Accept default prompts in non interactive mode",
    "url": "/wiki/modpack-management/packwand#flags",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Getting started",
    "pageTitle": "packwand",
    "sectionTitle": "Getting started",
    "description": "",
    "content": "Install packwand Create your first modpack Command reference Pack format reference Repository Releases",
    "url": "/wiki/modpack-management/packwand#getting-started",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Multi-pack repository",
    "pageTitle": "packwand",
    "sectionTitle": "Multi-pack repository",
    "description": "",
    "content": "packwand becomes more compelling when you maintain variants such as: client/server splits loader ports long term support branches regional or platform specific releases \"base pack\" content reused by consumer packs In these cases, workspace, packs, diff, and publish remove a lot of manual repository work.",
    "url": "/wiki/modpack-management/packwand#multi-pack-repository",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Other",
    "pageTitle": "packwand",
    "sectionTitle": "Other",
    "description": "",
    "content": "api Run and inspect the Packwand HTTP API automation Query effective automation settings for a pack cache Inspect and maintain the shared download cache diff Show mod additions, removals, and updates between two git refs gui Run the local Packwand web GUI modlist Write a crash assistant modlist.json from a pack's mods/ directory nix Nix integration pages Regenerate modlist.md files and the projects index run Execute a user defined script from pack.toml serve Run a local development server settings Manage pack settings utils Utilities for managing packwiz itself",
    "url": "/wiki/modpack-management/packwand#other",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Pack Management",
    "pageTitle": "packwand",
    "sectionTitle": "Pack Management",
    "description": "",
    "content": "add Add a mod to all or a specific pack's Modrinth and CurseForge subdirs curseforge Manage CurseForge based mods forgejo Manage projects released on Forgejo, Gitea, or Codeberg freeze Pin mods so updates skip them github Manage projects released on GitHub gitlab Manage projects released on GitLab or self hosted GitLab instances import Import an .mrpack or CurseForge zip as a new modpack init Initialise a packwiz modpack modrinth Manage Modrinth based mods new Scaffold a new pack pin Pin a file so it does not get updated automatically port Compare Modrinth and CurseForge subdirs and port missing mods rehash Migrate all hashes to a specific format remove Remove an external file from the modpack side Check or fix a mod's side across all subdirs in a pack unfreeze Unpin mods so updates can apply to them again unpin Unpin a file so it receives updates url Add external files from a direct download link",
    "url": "/wiki/modpack-management/packwand#pack-management",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Single pack",
    "pageTitle": "packwand",
    "sectionTitle": "Single pack",
    "description": "",
    "content": "packwand still works well for one pack when you want publishing, validation, a local GUI, or a more opinionated CLI than packwiz offers.",
    "url": "/wiki/modpack-management/packwand#single-pack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Single-pack vs multi-pack use",
    "pageTitle": "packwand",
    "sectionTitle": "Single-pack vs multi-pack use",
    "description": "",
    "content": "Single-pack vs multi-pack use",
    "url": "/wiki/modpack-management/packwand#single-pack-vs-multi-pack-use",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Typical repository flow",
    "pageTitle": "packwand",
    "sectionTitle": "Typical repository flow",
    "description": "",
    "content": "1. Create or enter a pack repository. 2. Run packwand init for a single pack or packwand new when you want packwand's scaffolding. 3. Add mods from Modrinth, CurseForge, or forge hosted releases with metadata commands instead of dropping JARs into mods/. 4. Commit the resulting manifest changes to Git. 5. Use packwand refresh, validate, content lint, and test as quality gates. 6. Build and publish from the same metadata when the pack is ready.",
    "url": "/wiki/modpack-management/packwand#typical-repository-flow",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Updates & Refresh",
    "pageTitle": "packwand",
    "sectionTitle": "Updates & Refresh",
    "description": "",
    "content": "migrate Migrate Minecraft, loader, or pack format generations refresh Refresh the index file update Update an external file or all external files in the modpack",
    "url": "/wiki/modpack-management/packwand#updates-refresh",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Usage",
    "pageTitle": "packwand",
    "sectionTitle": "Usage",
    "description": "",
    "content": "Usage",
    "url": "/wiki/modpack-management/packwand#usage",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / What packwand adds on top of packwiz",
    "pageTitle": "packwand",
    "sectionTitle": "What packwand adds on top of packwiz",
    "description": "",
    "content": "Workspace operations across many packs in the same repository Publishing commands that plan, build, upload, and verify release artifacts Repository aware commands such as diff, pages, workspace status, and workspace sync Extra automation surfaces: HTTP API, local GUI, automation plans, and richer diagnostics A broader installer/export/testing story for teams maintaining long lived packs",
    "url": "/wiki/modpack-management/packwand#what-packwand-adds-on-top-of-packwiz",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / When to reach for packwand",
    "pageTitle": "packwand",
    "sectionTitle": "When to reach for packwand",
    "description": "",
    "content": "Use packwand when you need one or more of these: A single source of truth in Git for mods, configs, scripts, and exports Multiple related packs in one repository with shared content or synchronized updates First class publishing workflows for Modrinth, CurseForge, and internal targets Diagnostics such as diffing, validation, content linting, and test installs A local GUI or HTTP API on top of the manifest driven workflow If you only need the smaller original CLI for a single pack, packwiz is still a good fit.",
    "url": "/wiki/modpack-management/packwand#when-to-reach-for-packwand",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwand / Workspace",
    "pageTitle": "packwand",
    "sectionTitle": "Workspace",
    "description": "",
    "content": "packs Look up or edit any pack's manifest fields by id workspace Multi pack workspace operations across all packs",
    "url": "/wiki/modpack-management/packwand#workspace",
    "tags": []
  },
  {
    "kind": "page",
    "title": "packwiz",
    "pageTitle": "packwiz",
    "sectionTitle": "",
    "description": "",
    "content": "packwiz packwiz is a command line tool for authoring Minecraft modpacks as TOML metadata rather than as a checked in folder of downloaded JARs. It is the core manifest format that packwand builds on. If you want a smaller, established CLI for a single pack, packwiz is often the simplest place to start. Where packwiz fits best packwiz is well suited to: Single pack repositories Private packs for friends, servers, or internal testing Creator workflows where the manifest format matters more than repository automation Teams that want a stable, Git friendly format without the packwand specific surfaces Where packwiz is intentionally smaller packwiz is not trying to be a repository orchestration tool. It is lighter on: multi pack workspace management release planning and verification workflows repository diffing and diagnostics local GUI or API surfaces If you want those higher level workflows, move up to packwand. How packwiz relates to the rest of this section packwiz is the authoring CLI and metadata format. packwiz installer is the runtime updater players and servers execute. The bootstrap is the tiny launcher facing shim that updates and starts the installer. packwand uses the same general pack format but adds more repository aware tooling on top. Recommended author workflow 1. Create a clean repository for the pack. 2. Initialize the manifest with packwiz. 3. Add mods through metadata commands rather than copying JARs into source control. 4. Commit the resulting manifest and config changes to Git. 5. Test distribution through a local server, export, or installer flow before publishing. Features Git friendly TOML based metadata format Java based pack installer/updater (works with MultiMC and ATLauncher), with support for optional mods and fast automatic updates Pack distribution with HTTP servers, with a built in local server for testing Easy installation and updating of multiple mods at once from CurseForge and Modrinth Exporting to CurseForge and Modrinth packs Importing from CurseForge packs Server only and client only mod handling Creation of remote file metadata from JAR files for CurseForge mods Useful links packwiz repository example pack third party GUI project upstream Discord",
    "url": "/wiki/modpack-management/packwiz",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Packwiz Components",
    "pageTitle": "Packwiz Components",
    "sectionTitle": "",
    "description": "",
    "content": "Packwiz Components This site documents the packwiz ecosystem components shipped alongside packwand . They install and update packs in the packwiz/packwand format on end user machines. Component Language Location Purpose packwiz installer Kotlin lib/packwiz installer Downloads and updates pack contents on launch, with optional mod UI and side only filtering bootstrap Go (new) / Java (legacy) apps/packwand/cmd/packwiz bootstrap, lib/packwiz installer/bootstrap Verifies a JDK, keeps packwiz installer up to date, and launches it mod browser webview Rust (wry) apps/mod browser webview Native webview for downloading CurseForge files that disallow API distribution; bridged into the packwand GUI All three are built from this repository see Building. How they fit together 1. A launcher instance (MultiMC/Prism/ATLauncher) or server start script runs the bootstrap as a pre launch command. 2. The bootstrap verifies Java, updates packwiz installer if needed, and hands over your pack URL. 3. packwiz installer reads pack.toml, downloads changed files, prompts for optional mods, and writes its state to packwiz.json. 4. For CurseForge files that cannot be downloaded through the API, tooling can open mod browser webview so the user downloads them from the real CurseForge site; the resulting CDN URLs are captured programmatically.",
    "url": "/wiki/modpack-management/packwiz/components",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bootstrap",
    "pageTitle": "Bootstrap",
    "sectionTitle": "",
    "description": "",
    "content": "Bootstrap The bootstrap is the small program a launcher actually invokes. It verifies that a suitable Java runtime is available, keeps packwiz installer.jar up to date, launches it with your arguments, and passes the exit code through. Two implementations are maintained: Go bootstrap (recommended) Source: apps/packwand/cmd/packwiz bootstrap. A single native binary with no Java requirement of its own, following packwand's CLI conventions. Option Description java Path to the java executable (otherwise $JAVA HOME/bin/java, then PATH) min java Minimum Java major version to accept (defaults to 8) jar Location of packwiz installer.jar (defaults to next to the bootstrap executable) download url URL to download packwiz installer.jar from when missing sha256 Expected SHA 256 of a downloaded jar (verified before first use) g, no gui Passed through to the installer: disable the GUI s, side Passed through to the installer: client or server Behaviour: 1. Locates and verifies Java (java version must report at least min java). 2. Ensures the installer jar exists; downloads it from download url if missing (with optional SHA 256 verification). 3. Runs java jar packwiz installer.jar and exits with the installer's exit code. Example (MultiMC/Prism pre launch command): Legacy Java bootstrap Source: lib/packwiz installer/bootstrap (built as a Gradle subproject of packwiz installer). Kept for compatibility with existing instances that already ship packwiz installer bootstrap.jar. Option Description bootstrap update url GitHub API URL for checking for updates bootstrap update token GitHub API access token, for private repositories bootstrap no update Don't update packwiz installer bootstrap main jar Location of the packwiz installer JAR file g, no gui Don't display a GUI to show update progress h, help Display usage (includes the installer's options when the jar is present) All other arguments are passed through to packwiz installer.",
    "url": "/wiki/modpack-management/packwiz/components/bootstrap",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Bootstrap / Go bootstrap (recommended)",
    "pageTitle": "Bootstrap",
    "sectionTitle": "Go bootstrap (recommended)",
    "description": "",
    "content": "Source: apps/packwand/cmd/packwiz bootstrap. A single native binary with no Java requirement of its own, following packwand's CLI conventions. Option Description java Path to the java executable (otherwise $JAVA HOME/bin/java, then PATH) min java Minimum Java major version to accept (defaults to 8) jar Location of packwiz installer.jar (defaults to next to the bootstrap executable) download url URL to download packwiz installer.jar from when missing sha256 Expected SHA 256 of a downloaded jar (verified before first use) g, no gui Passed through to the installer: disable the GUI s, side Passed through to the installer: client or server Behaviour: 1. Locates and verifies Java (java version must report at least min java). 2. Ensures the installer jar exists; downloads it from download url if missing (with optional SHA 256 verification). 3. Runs java jar packwiz installer.jar and exits with the installer's exit code. Example (MultiMC/Prism pre launch command):",
    "url": "/wiki/modpack-management/packwiz/components/bootstrap#go-bootstrap-recommended",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Bootstrap / Legacy Java bootstrap",
    "pageTitle": "Bootstrap",
    "sectionTitle": "Legacy Java bootstrap",
    "description": "",
    "content": "Source: lib/packwiz installer/bootstrap (built as a Gradle subproject of packwiz installer). Kept for compatibility with existing instances that already ship packwiz installer bootstrap.jar. Option Description bootstrap update url GitHub API URL for checking for updates bootstrap update token GitHub API access token, for private repositories bootstrap no update Don't update packwiz installer bootstrap main jar Location of the packwiz installer JAR file g, no gui Don't display a GUI to show update progress h, help Display usage (includes the installer's options when the jar is present) All other arguments are passed through to packwiz installer.",
    "url": "/wiki/modpack-management/packwiz/components/bootstrap#legacy-java-bootstrap",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Building",
    "pageTitle": "Building",
    "sectionTitle": "",
    "description": "",
    "content": "Building All components build from the repository root with Task (Taskfile.yml), or directly with their native toolchains. Prerequisites JDK 17+ (JDK 25 verified) for packwiz installer â€” Gradle 9 is fetched by the wrapper Rust (cargo) for mod browser webview Go 1.25+ for the Go bootstrap and packwand With Task Directly ::: info The installer's R8 shrunk distribution jar is opt in: ./gradlew build PshrinkDist=true. The default build ships the shadow jar, because R8 8.5 cannot read the class files of very new JDKs (e.g. Java 25) when they are passed as its library. :::",
    "url": "/wiki/modpack-management/packwiz/components/building",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building / Directly",
    "pageTitle": "Building",
    "sectionTitle": "Directly",
    "description": "",
    "content": "::: info The installer's R8 shrunk distribution jar is opt in: ./gradlew build PshrinkDist=true. The default build ships the shadow jar, because R8 8.5 cannot read the class files of very new JDKs (e.g. Java 25) when they are passed as its library. :::",
    "url": "/wiki/modpack-management/packwiz/components/building#directly",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building / Prerequisites",
    "pageTitle": "Building",
    "sectionTitle": "Prerequisites",
    "description": "",
    "content": "JDK 17+ (JDK 25 verified) for packwiz installer â€” Gradle 9 is fetched by the wrapper Rust (cargo) for mod browser webview Go 1.25+ for the Go bootstrap and packwand",
    "url": "/wiki/modpack-management/packwiz/components/building#prerequisites",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Building / With Task",
    "pageTitle": "Building",
    "sectionTitle": "With Task",
    "description": "",
    "content": "With Task",
    "url": "/wiki/modpack-management/packwiz/components/building#with-task",
    "tags": []
  },
  {
    "kind": "page",
    "title": "packwiz-installer",
    "pageTitle": "packwiz-installer",
    "sectionTitle": "",
    "description": "",
    "content": "packwiz installer A Kotlin/JVM installer that downloads and updates packwiz/packwand format packs on launch. It runs as a pre launch task in MultiMC/Prism/ATLauncher, or in a server start script, and supports side only mods as well as optional mods with a GUI (and a fully non interactive mode for servers). Source: lib/packwiz installer. Build output: lib/packwiz installer/build/dist/packwiz installer.jar. Usage packwiz installer is normally launched through the bootstrap, which handles updates: Running the JAR directly also works (no auto update): Options Option Description s, side Side to install mods from (client/server, defaults to client) title Title of the installer window pack folder Folder to install the pack to (defaults to the JAR directory) multimc folder The MultiMC pack folder (defaults to the parent of the pack directory) meta file JSON file to store pack metadata, relative to the pack folder (defaults to packwiz.json) t, timeout Seconds to wait before automatically launching when asking about optional mods (defaults to 10) g, no gui Don't display a GUI to show update progress (for servers/CI) h, help Display usage The bootstrap options are accepted (and ignored) so that the bootstrap can pass its own arguments through. Server usage g disables the GUI s server downloads only server side mods (side server or both) State Installed file state is tracked in packwiz.json (configurable with meta file) so that removed files are cleaned up, preserved files are not overwritten, and unchanged files are not re downloaded.",
    "url": "/wiki/modpack-management/packwiz/components/installer",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz-installer / Options",
    "pageTitle": "packwiz-installer",
    "sectionTitle": "Options",
    "description": "",
    "content": "Option Description s, side Side to install mods from (client/server, defaults to client) title Title of the installer window pack folder Folder to install the pack to (defaults to the JAR directory) multimc folder The MultiMC pack folder (defaults to the parent of the pack directory) meta file JSON file to store pack metadata, relative to the pack folder (defaults to packwiz.json) t, timeout Seconds to wait before automatically launching when asking about optional mods (defaults to 10) g, no gui Don't display a GUI to show update progress (for servers/CI) h, help Display usage The bootstrap options are accepted (and ignored) so that the bootstrap can pass its own arguments through.",
    "url": "/wiki/modpack-management/packwiz/components/installer#options",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz-installer / Server usage",
    "pageTitle": "packwiz-installer",
    "sectionTitle": "Server usage",
    "description": "",
    "content": "g disables the GUI s server downloads only server side mods (side server or both)",
    "url": "/wiki/modpack-management/packwiz/components/installer#server-usage",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz-installer / State",
    "pageTitle": "packwiz-installer",
    "sectionTitle": "State",
    "description": "",
    "content": "Installed file state is tracked in packwiz.json (configurable with meta file) so that removed files are cleaned up, preserved files are not overwritten, and unchanged files are not re downloaded.",
    "url": "/wiki/modpack-management/packwiz/components/installer#state",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz-installer / Usage",
    "pageTitle": "packwiz-installer",
    "sectionTitle": "Usage",
    "description": "",
    "content": "packwiz installer is normally launched through the bootstrap, which handles updates: Running the JAR directly also works (no auto update):",
    "url": "/wiki/modpack-management/packwiz/components/installer#usage",
    "tags": []
  },
  {
    "kind": "page",
    "title": "modbrowserwebview",
    "pageTitle": "modbrowserwebview",
    "sectionTitle": "",
    "description": "",
    "content": "mod browser webview A native webview (Rust, using wry) that displays real CurseForge or Modrinth project pages so users can download files that may not be distributed through the CurseForge API. Host applications drive it over a simple stdin/stdout line protocol and receive the resolved CDN download URLs. Source: apps/mod browser webview. Build output: apps/mod browser webview/target/release/mod browser webview. Platform requirements Windows : the WebView2 runtime (preinstalled on Windows 11) Linux : WebKitGTK (webkit2gtk) macOS : WKWebView (built in) Protocol The provider is selected with a CLI flag: provider curseforge (default) or provider modrinth. The host writes to the webview's stdin , one request per line, then DONE: Each request line is a file/version ID, a space, and the project page URL. For CurseForge the ID is numeric and the URL must match https://(www. beta.)curseforge.com/ / / ; for Modrinth the ID is an alphanumeric version ID and the URL must match https://modrinth.com/ / (the file page becomes /version/ ). The webview then opens the file page for each request in turn. Navigation is sandboxed: only pages for the requested file are allowed, curseforge:// and other external links prompt the user, and unrelated links open in the system browser. A Reload and Skip menu are available; skipping a file advances to the next one without emitting output. The host reads stdout : Each line reports the download URL captured for the request at that (zero based) index. On failure, a line reading ERROR is printed followed by error details. The process exits when every request has been downloaded or skipped, or when the window is closed. packwand GUI integration The packwand GUI (packwand gui) bridges this protocol over HTTP + Server Sent Events: POST /api/webview/open with {\"provider\": \"curseforge\", \"files\": [{\"file id\": \"3643025\", \"slug\": \"jei\"}]} (or an explicit \"url\"; provider may be \"modrinth\") spawns the webview and returns a job ID. The job's event stream (GET /api/jobs/{id}/events) then carries a DOWNLOAD line for every captured file, live, followed by a summary line. The binary is located via MOD BROWSER WEBVIEW BIN, the in repo cargo output (apps/mod browser webview/target/{release,debug}), or PATH. In the GUI's Mods view, CurseForge and Modrinth mods with a known file/version ID show a CF Fetch / MR Fetch button that opens the webview for that mod and streams the captured URL into the Logs view. Licenses page The About menu shows bundled third party licenses from src/licenses.html. Regenerate it after dependency changes with task gen licenses (or the commands in the README).",
    "url": "/wiki/modpack-management/packwiz/components/webview",
    "tags": []
  },
  {
    "kind": "section",
    "title": "modbrowserwebview / Licenses page",
    "pageTitle": "modbrowserwebview",
    "sectionTitle": "Licenses page",
    "description": "",
    "content": "The About menu shows bundled third party licenses from src/licenses.html. Regenerate it after dependency changes with task gen licenses (or the commands in the README).",
    "url": "/wiki/modpack-management/packwiz/components/webview#licenses-page",
    "tags": []
  },
  {
    "kind": "section",
    "title": "modbrowserwebview / packwand GUI integration",
    "pageTitle": "modbrowserwebview",
    "sectionTitle": "packwand GUI integration",
    "description": "",
    "content": "The packwand GUI (packwand gui) bridges this protocol over HTTP + Server Sent Events: POST /api/webview/open with {\"provider\": \"curseforge\", \"files\": [{\"file id\": \"3643025\", \"slug\": \"jei\"}]} (or an explicit \"url\"; provider may be \"modrinth\") spawns the webview and returns a job ID. The job's event stream (GET /api/jobs/{id}/events) then carries a DOWNLOAD line for every captured file, live, followed by a summary line. The binary is located via MOD BROWSER WEBVIEW BIN, the in repo cargo output (apps/mod browser webview/target/{release,debug}), or PATH. In the GUI's Mods view, CurseForge and Modrinth mods with a known file/version ID show a CF Fetch / MR Fetch button that opens the webview for that mod and streams the captured URL into the Logs view.",
    "url": "/wiki/modpack-management/packwiz/components/webview#packwand-gui-integration",
    "tags": []
  },
  {
    "kind": "section",
    "title": "modbrowserwebview / Platform requirements",
    "pageTitle": "modbrowserwebview",
    "sectionTitle": "Platform requirements",
    "description": "",
    "content": "Windows : the WebView2 runtime (preinstalled on Windows 11) Linux : WebKitGTK (webkit2gtk) macOS : WKWebView (built in)",
    "url": "/wiki/modpack-management/packwiz/components/webview#platform-requirements",
    "tags": []
  },
  {
    "kind": "section",
    "title": "modbrowserwebview / Protocol",
    "pageTitle": "modbrowserwebview",
    "sectionTitle": "Protocol",
    "description": "",
    "content": "The provider is selected with a CLI flag: provider curseforge (default) or provider modrinth. The host writes to the webview's stdin , one request per line, then DONE: Each request line is a file/version ID, a space, and the project page URL. For CurseForge the ID is numeric and the URL must match https://(www. beta.)curseforge.com/ / / ; for Modrinth the ID is an alphanumeric version ID and the URL must match https://modrinth.com/ / (the file page becomes /version/ ). The webview then opens the file page for each request in turn. Navigation is sandboxed: only pages for the requested file are allowed, curseforge:// and other external links prompt the user, and unrelated links open in the system browser. A Reload and Skip menu are available; skipping a file advances to the next one without emitting output. The host reads stdout : Each line reports the download URL captured for the request at that (zero based) index. On failure, a line reading ERROR is printed followed by error details. The process exits when every request has been downloaded or skipped, or when the window is closed.",
    "url": "/wiki/modpack-management/packwiz/components/webview#protocol",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Packwiz Components / How they fit together",
    "pageTitle": "Packwiz Components",
    "sectionTitle": "How they fit together",
    "description": "",
    "content": "1. A launcher instance (MultiMC/Prism/ATLauncher) or server start script runs the bootstrap as a pre launch command. 2. The bootstrap verifies Java, updates packwiz installer if needed, and hands over your pack URL. 3. packwiz installer reads pack.toml, downloads changed files, prompts for optional mods, and writes its state to packwiz.json. 4. For CurseForge files that cannot be downloaded through the API, tooling can open mod browser webview so the user downloads them from the real CurseForge site; the resulting CDN URLs are captured programmatically.",
    "url": "/wiki/modpack-management/packwiz/components#how-they-fit-together",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Installation",
    "pageTitle": "Installation",
    "sectionTitle": "",
    "description": "",
    "content": "Installation Prebuilt binaries are available from GitHub Actions . The UI is awkward, but the general flow is to open the latest successful build and download the artifact zip for your system from the artifacts section. To run the executable, add the folder where you downloaded it to your PATH environment variable (see tutorial for Windows here) or move it somewhere already on PATH. If you do not have a GitHub account or cannot download directly from GitHub, you can also use nightly.link . You can also compile from source: 1. Install Go (1.19 or newer) from https://golang.org/dl/ 2. Run go install github.com/packwiz/packwiz@latest Be patient on the first run; Go needs to download and compile dependencies. Choosing an install path Use the prebuilt archive if you only need the CLI. Use go install if you already work in Go and want the fastest developer setup. Pair packwiz with the bootstrap and installer when you are validating the player update path.",
    "url": "/wiki/modpack-management/packwiz/installation",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Installation / Choosing an install path",
    "pageTitle": "Installation",
    "sectionTitle": "Choosing an install path",
    "description": "",
    "content": "Use the prebuilt archive if you only need the CLI. Use go install if you already work in Go and want the fastest developer setup. Pair packwiz with the bootstrap and installer when you are validating the player update path.",
    "url": "/wiki/modpack-management/packwiz/installation#choosing-an-install-path",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Additional options",
    "pageTitle": "Additional options",
    "sectionTitle": "",
    "description": "",
    "content": "Additional options Additional options can be configured in the [options] section of pack.toml, as follows: acceptable game versions A list of additional Minecraft versions to accept when installing or updating mods (see Adding mods) meta folder The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used) mods folder is now deprecated; aliassed to meta folder meta folder base The base folder from which meta folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour) no internal hashes If this is set to true, packwiz will not generate hashes of local files, to prevent merge conflicts and inconsistent hashes when using git/etc. packwiz refresh build can be used in this mode to generate internal hashes for distributing the pack with [packwiz installer] datapack folder The folder in which datapacks are to be added; specific to the datapack loader mod you use, and must be set to add datapacks (that are not bundled as mods)",
    "url": "/wiki/modpack-management/packwiz/reference/additional-options",
    "tags": []
  },
  {
    "kind": "page",
    "title": ".packwizignore",
    "pageTitle": ".packwizignore",
    "sectionTitle": "",
    "description": "",
    "content": ".packwizignore .packwizignore works like .gitignore, but for packwiz refresh and export generation. Use it to exclude files that should exist in your working directory without being indexed into the pack manifest. Common entries When to use it Add entries when a file is part of your local workflow but should not be treated as pack content. Typical examples include Git metadata, exported archives, local notes, or temporary tooling output. If a file should never be downloaded by players or mirrored into the pack index, .packwizignore is the right place to exclude it.",
    "url": "/wiki/modpack-management/packwiz/reference/pack-format/packwizignore",
    "tags": []
  },
  {
    "kind": "section",
    "title": ".packwizignore / Common entries",
    "pageTitle": ".packwizignore",
    "sectionTitle": "Common entries",
    "description": "",
    "content": "Common entries",
    "url": "/wiki/modpack-management/packwiz/reference/pack-format/packwizignore#common-entries",
    "tags": []
  },
  {
    "kind": "section",
    "title": ".packwizignore / When to use it",
    "pageTitle": ".packwizignore",
    "sectionTitle": "When to use it",
    "description": "",
    "content": "Add entries when a file is part of your local workflow but should not be treated as pack content. Typical examples include Git metadata, exported archives, local notes, or temporary tooling output. If a file should never be downloaded by players or mirrored into the pack index, .packwizignore is the right place to exclude it.",
    "url": "/wiki/modpack-management/packwiz/reference/pack-format/packwizignore#when-to-use-it",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Adding mods and resource packs",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "",
    "description": "",
    "content": "Adding mods and resource packs To add external files to your modpack, such as mods and resource packs, you'll need .pw.toml metadata files to define how to download them. The modrinth install and curseforge install commands can automatically create these for you with all the necessary metadata! CurseForge and Modrinth Mods and resource packs from CurseForge and Modrinth can be easily added with the modrinth install and curseforge install commands. They can also be updated with the packwiz update command; pass all to update all your mods at once. Mods can be passed in multiple forms to these commands: packwiz curseforge install indium (by slug) packwiz curseforge install category texture packs unity (by slug; category and game can be specified with the corresponding flags) packwiz curseforge install https://www.curseforge.com/minecraft/mc mods/indium (by mod page URL) packwiz curseforge install https://www.curseforge.com/minecraft/mc mods/indium/files/3535202 (by file page URL) packwiz curseforge install Indium (by search) packwiz curseforge install addon id 459496 file id 3535202 (if all else fails) packwiz modrinth install indium (by slug) packwiz modrinth install https://modrinth.com/mod/indium (by mod page URL) packwiz modrinth install https://modrinth.com/mod/indium/version/mfNlBb6U (by file page URL) packwiz modrinth install Fabric Rendering Sodium (by search) packwiz modrinth install Orvt0mRa (by ID) Dependencies are automatically picked up for you if you don't have them already, you'll be prompted whether you want to install them. packwiz also checks if your mods are being installed for the wrong version; but you can tell it to allow more versions using the acceptable game versions field in pack.toml. Just add the following to the bottom of pack.toml, replacing the versions listed here with those you want to allow: !!! tip Several aliases exist for the curseforge and modrinth commands to speed up your workflow. Try packwiz cf add or packwiz mr add! Internal files (config files, scripts, etc.) Configuration files for your modpack can simply be placed in a config folder (in the same place as the mods folder) and they'll be copied to the config folder when installing the modpack. This works for any file (including quests/scripts) place it in the modpack and it'll be installed into the corresponding location in the game folder. Make sure you run packwiz refresh so that the index is up to date! This works for mods that aren't available elsewhere online too (e.g. custom mods or forks); just drop them in the mods folder alongside the .pw.toml files. This isn't ideal for Git as it's not great at handling large binary files; you could use Git LFS or you may prefer to upload them elsewhere manually and reference them from the pack see the section below. !!! tip If you don't want to include files in the modpack, you can add them to a file called .packwizignore in your modpack directory. This uses the same format as gitignore; see the example pack for an example! Other external files If you have external files/mods that aren't from CurseForge or Modrinth, you'll need to create the .pw.toml files manually. See the following for an example of how you could lay it out: You can even create them for files that aren't mods (such as resource packs) just make sure to use the .pw.toml extension and run packwiz refresh, so that packwiz knows that the file contains metadata.",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/adding-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / CurseForge and Modrinth",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "CurseForge and Modrinth",
    "description": "",
    "content": "Mods and resource packs from CurseForge and Modrinth can be easily added with the modrinth install and curseforge install commands. They can also be updated with the packwiz update command; pass all to update all your mods at once. Mods can be passed in multiple forms to these commands: packwiz curseforge install indium (by slug) packwiz curseforge install category texture packs unity (by slug; category and game can be specified with the corresponding flags) packwiz curseforge install https://www.curseforge.com/minecraft/mc mods/indium (by mod page URL) packwiz curseforge install https://www.curseforge.com/minecraft/mc mods/indium/files/3535202 (by file page URL) packwiz curseforge install Indium (by search) packwiz curseforge install addon id 459496 file id 3535202 (if all else fails) packwiz modrinth install indium (by slug) packwiz modrinth install https://modrinth.com/mod/indium (by mod page URL) packwiz modrinth install https://modrinth.com/mod/indium/version/mfNlBb6U (by file page URL) packwiz modrinth install Fabric Rendering Sodium (by search) packwiz modrinth install Orvt0mRa (by ID) Dependencies are automatically picked up for you if you don't have them already, you'll be prompted whether you want to install them. packwiz also checks if your mods are being installed for the wrong version; but you can tell it to allow more versions using the acceptable game versions field in pack.toml. Just add the following to the bottom of pack.toml, replacing the versions listed here with those you want to allow: !!! tip Several aliases exist for the curseforge and modrinth commands to speed up your workflow. Try packwiz cf add or packwiz mr add!",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/adding-mods#curseforge-and-modrinth",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / Internal files (config files, scripts, etc.)",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "Internal files (config files, scripts, etc.)",
    "description": "",
    "content": "Configuration files for your modpack can simply be placed in a config folder (in the same place as the mods folder) and they'll be copied to the config folder when installing the modpack. This works for any file (including quests/scripts) place it in the modpack and it'll be installed into the corresponding location in the game folder. Make sure you run packwiz refresh so that the index is up to date! This works for mods that aren't available elsewhere online too (e.g. custom mods or forks); just drop them in the mods folder alongside the .pw.toml files. This isn't ideal for Git as it's not great at handling large binary files; you could use Git LFS or you may prefer to upload them elsewhere manually and reference them from the pack see the section below. !!! tip If you don't want to include files in the modpack, you can add them to a file called .packwizignore in your modpack directory. This uses the same format as gitignore; see the example pack for an example!",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/adding-mods#internal-files-config-files-scripts-etc",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Adding mods and resource packs / Other external files",
    "pageTitle": "Adding mods and resource packs",
    "sectionTitle": "Other external files",
    "description": "",
    "content": "If you have external files/mods that aren't from CurseForge or Modrinth, you'll need to create the .pw.toml files manually. See the following for an example of how you could lay it out: You can even create them for files that aren't mods (such as resource packs) just make sure to use the .pw.toml extension and run packwiz refresh, so that packwiz knows that the file contains metadata.",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/adding-mods#other-external-files",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Getting started",
    "pageTitle": "Getting started",
    "sectionTitle": "",
    "description": "",
    "content": "Getting started To use the packwiz CLI, first you'll want to create a folder to develop your modpack in. This should not be the same as your .minecraft or a MultiMC instance folder; this folder holds metadata and files for your modpack so it can be managed by packwiz. Then open your command line (Command Prompt/Terminal) and use the cd command to move into the folder you created. Creating a new modpack To create the files for your new modpack, just run packwiz init in the folder you created! It'll ask you for a few details, then create a pack.toml and index.toml based on your answers. pack.toml is the main file of your modpack and defines several crucial details; including the name of your modpack, the version of Minecraft and the version of the mod loader you're using. Optionally, you can include a version (required for exporting to Modrinth packs) and a description for your modpack. index.toml is the index of your modpack which lists the files in your modpack with their hashes (for integrity checking). You're unlikely to need to touch this yourself, but you'll need to run the packwiz refresh command when you manually add, remove or edit files in the pack. Importing an existing modpack Have an existing CurseForge modpack? You can use the packwiz curseforge import command with the path to the modpack .zip file, which will import all the mods and files from the pack into your current directory. If this isn't your own modpack, please make sure you have permission (or a license) to redistribute the modpack you import! !!! warning If you have existing files in your modpack, importing will overwrite them. It's a good idea to use version control systems (such as Git) with packwiz! Cheat Sheet You'll get more information in the tutorials following this one (and the reference pages), but this is a quick summary of the most useful commands: packwiz init creates a modpack in the current folder packwiz curseforge import [zip path] imports a CurseForge modpack packwiz refresh updates the modpack index packwiz curseforge install [mod] installs a mod from CurseForge packwiz modrinth install [mod] installs a mod from Modrinth packwiz update [mod] updates a mod packwiz update all updates all the mods in the modpack packwiz curseforge export exports the modpack in the format supported by the CurseForge Launcher packwiz modrinth export exports the modpack in the format supported by Modrinth (and their in progress launcher) packwiz curseforge detect to detect files that are available on CurseForge and make them downloaded from there Use the help flag for more information about any command! [packwiz installer]: ../installing/packwiz installer.md",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/getting-started",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Getting started / Cheat Sheet",
    "pageTitle": "Getting started",
    "sectionTitle": "Cheat Sheet",
    "description": "",
    "content": "You'll get more information in the tutorials following this one (and the reference pages), but this is a quick summary of the most useful commands: packwiz init creates a modpack in the current folder packwiz curseforge import [zip path] imports a CurseForge modpack packwiz refresh updates the modpack index packwiz curseforge install [mod] installs a mod from CurseForge packwiz modrinth install [mod] installs a mod from Modrinth packwiz update [mod] updates a mod packwiz update all updates all the mods in the modpack packwiz curseforge export exports the modpack in the format supported by the CurseForge Launcher packwiz modrinth export exports the modpack in the format supported by Modrinth (and their in progress launcher) packwiz curseforge detect to detect files that are available on CurseForge and make them downloaded from there Use the help flag for more information about any command! [packwiz installer]: ../installing/packwiz installer.md",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/getting-started#cheat-sheet",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Getting started / Creating a new modpack",
    "pageTitle": "Getting started",
    "sectionTitle": "Creating a new modpack",
    "description": "",
    "content": "To create the files for your new modpack, just run packwiz init in the folder you created! It'll ask you for a few details, then create a pack.toml and index.toml based on your answers. pack.toml is the main file of your modpack and defines several crucial details; including the name of your modpack, the version of Minecraft and the version of the mod loader you're using. Optionally, you can include a version (required for exporting to Modrinth packs) and a description for your modpack. index.toml is the index of your modpack which lists the files in your modpack with their hashes (for integrity checking). You're unlikely to need to touch this yourself, but you'll need to run the packwiz refresh command when you manually add, remove or edit files in the pack.",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/getting-started#creating-a-new-modpack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Getting started / Importing an existing modpack",
    "pageTitle": "Getting started",
    "sectionTitle": "Importing an existing modpack",
    "description": "",
    "content": "Have an existing CurseForge modpack? You can use the packwiz curseforge import command with the path to the modpack .zip file, which will import all the mods and files from the pack into your current directory. If this isn't your own modpack, please make sure you have permission (or a license) to redistribute the modpack you import! !!! warning If you have existing files in your modpack, importing will overwrite them. It's a good idea to use version control systems (such as Git) with packwiz!",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/getting-started#importing-an-existing-modpack",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Using packwiz with Git",
    "pageTitle": "Using packwiz with Git",
    "sectionTitle": "",
    "description": "",
    "content": "Using packwiz with Git On Windows, line ending conversion causes the hashes to change when files are uploaded to Git, so you'll get invalid hash errors when trying to install the pack. You'll want to add a .gitattributes file to disable line ending conversion. See the example pack for example .gitignore and .gitattributes files! If you have existing files committed to Git, you'll need to run git add renormalize . to reset line ending conversion after adding .gitattributes.",
    "url": "/wiki/modpack-management/packwiz/tutorials/creating/git",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Publishing to CurseForge",
    "pageTitle": "Publishing to CurseForge",
    "sectionTitle": "",
    "description": "",
    "content": "Publishing to CurseForge Exporting a CurseForge pack is as simple as running packwiz curseforge export this gives you a .zip in your pack folder that you can upload to CurseForge! Since this pack format doesn't support side only mods, packwiz can't create a pack that differs between server and client. You can use the side flag to specify which mods should be exported by default it exports a pack for Minecraft clients (containing mods with side client or both). Mods without the necessary CurseForge metadata (such as those installed from Modrinth) will be placed as JARs into the modpack zip; these must be approved manually by CurseForge staff. Be wary of including files that you don't want (the packwiz executable, and the modpack zip itself) in the pack! The CurseForge pack format doesn't really support optional mods. The user won't be prompted about optional mods, but if they default to being disabled they will be disabled in the CurseForge launcher. (though I can't speak for third party support since I don't think the launcher usually exports disabled mods)",
    "url": "/wiki/modpack-management/packwiz/tutorials/hosting/curseforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Publishing to Modrinth",
    "pageTitle": "Publishing to Modrinth",
    "sectionTitle": "",
    "description": "",
    "content": "Publishing to Modrinth Exporting a Modrinth pack is as simple as running packwiz modrinth export this gives you a .mrpack in your pack folder that you can upload to Modrinth! Unlike CurseForge, this pack format does support side only mods. When exporting, packwiz will export a pack with side information provided from Modrinth or as specified in the mod's mod.pw.toml file. The official Modrinth launcher will automatically filter out serverside mods, and the pack can be used on the server using tools like mrpack install or packwiz installer. Mods without the necessary Modrinth metadata (such as those installed from CurseForge) will be placed as JARs into the modpack zip; make sure that you have the licenses for these mods as it is your responsibility as a pack creator to. Be wary of including files that you don't want (the packwiz executable, and the modpack zip itself) in the pack! Keep in mind that since the official Modrinth app doesn't support optional mods, the user won't be prompted for optional mods. The official launcher will automatically install all optional mods (even if they default to being disabled!). If you'd like to be able to use optional mods, you use Prism Launcher's \"Import Instance\" section to install the exported .mrpack file. Note that if you use Prism Launcher's \"Modrinth\" section to install the pack, you will not be prompted for optional mods.",
    "url": "/wiki/modpack-management/packwiz/tutorials/hosting/modrinth",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Pack Installation using packwiz-installer",
    "pageTitle": "Pack Installation using packwiz-installer",
    "sectionTitle": "",
    "description": "",
    "content": "Pack Installation using packwiz installer [packwiz installer] is a Java based installer that allows for automatic installation and updates of packwiz packs! It can be used with MultiMC/ATLauncher as a prelaunch task, or on servers as part of your start script, and supports side only mods as well as optional mods with a fancy GUI. To distribute a packwiz modpack, you'll first want to set up a web hosting service (such as Netlify, Github Pages, GitLab Pages) so that your pack files are accessible from a HTTP/HTTPS link. For testing, you can use the packwiz serve command to run a local HTTP server, that serves your pack at http://localhost:8080/pack.toml it'll refresh the index whenever it's queried so you don't need to refresh it manually! Creating a MultiMC instance for your modpack To distribute the modpack as a MultiMC instance: 1. Create a barebones MultiMC instance, with the modloader and Minecraft version you want (memory allocation overrides are also a good idea) 2. Download packwiz installer bootstrap from https://github.com/packwiz/packwiz installer bootstrap/releases and place it in the instance Minecraft folder !!! info This is the same folder as options.txt MultiMC will call it .minecraft or minecraft depending on your system. 3. Go to Edit Instance Settings Custom commands, then check the Custom Commands box and paste the following command into the pre launch command field: \"$INST JAVA\" jar packwiz installer bootstrap.jar https://[your server]/pack.toml (where https://[your server]/pack.toml is the HTTP URL your pack.toml file is hosted at) 4. Use the Export Instance function to export your pack as a .zip file (which can be distributed similarly to your pack via a web hosting service) To install your pack, users just need to add it with Add instance Import from zip then packwiz installer does the rest, keeping it up to date every time the game is launched! Using a modpack with a server You can use [packwiz installer] to download non client mods (side either both or server), for example: java jar packwiz installer bootstrap.jar g s server https://[your server]/pack.toml g flag to disable the GUI s server to download only server side mods. itzg's docker minecraft server has built in support for packwiz. You can pass the PACKWIZ URL environment variable pointing to your pack's TOML file, and the container will bootstrap packwiz installer and install/update the provided pack. See the documentation for more information. [packwiz installer]: https://github.com/packwiz/packwiz installer",
    "url": "/wiki/modpack-management/packwiz/tutorials/installing/packwiz-installer",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Installation using packwiz-installer / Creating a MultiMC instance for your modpack",
    "pageTitle": "Pack Installation using packwiz-installer",
    "sectionTitle": "Creating a MultiMC instance for your modpack",
    "description": "",
    "content": "To distribute the modpack as a MultiMC instance: 1. Create a barebones MultiMC instance, with the modloader and Minecraft version you want (memory allocation overrides are also a good idea) 2. Download packwiz installer bootstrap from https://github.com/packwiz/packwiz installer bootstrap/releases and place it in the instance Minecraft folder !!! info This is the same folder as options.txt MultiMC will call it .minecraft or minecraft depending on your system. 3. Go to Edit Instance Settings Custom commands, then check the Custom Commands box and paste the following command into the pre launch command field: \"$INST JAVA\" jar packwiz installer bootstrap.jar https://[your server]/pack.toml (where https://[your server]/pack.toml is the HTTP URL your pack.toml file is hosted at) 4. Use the Export Instance function to export your pack as a .zip file (which can be distributed similarly to your pack via a web hosting service) To install your pack, users just need to add it with Add instance Import from zip then packwiz installer does the rest, keeping it up to date every time the game is launched!",
    "url": "/wiki/modpack-management/packwiz/tutorials/installing/packwiz-installer#creating-a-multimc-instance-for-your-modpack",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Installation using packwiz-installer / Using a modpack with a server",
    "pageTitle": "Pack Installation using packwiz-installer",
    "sectionTitle": "Using a modpack with a server",
    "description": "",
    "content": "You can use [packwiz installer] to download non client mods (side either both or server), for example: java jar packwiz installer bootstrap.jar g s server https://[your server]/pack.toml g flag to disable the GUI s server to download only server side mods. itzg's docker minecraft server has built in support for packwiz. You can pass the PACKWIZ URL environment variable pointing to your pack's TOML file, and the container will bootstrap packwiz installer and install/update the provided pack. See the documentation for more information. [packwiz installer]: https://github.com/packwiz/packwiz installer",
    "url": "/wiki/modpack-management/packwiz/tutorials/installing/packwiz-installer#using-a-modpack-with-a-server",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz / Features",
    "pageTitle": "packwiz",
    "sectionTitle": "Features",
    "description": "",
    "content": "Git friendly TOML based metadata format Java based pack installer/updater (works with MultiMC and ATLauncher), with support for optional mods and fast automatic updates Pack distribution with HTTP servers, with a built in local server for testing Easy installation and updating of multiple mods at once from CurseForge and Modrinth Exporting to CurseForge and Modrinth packs Importing from CurseForge packs Server only and client only mod handling Creation of remote file metadata from JAR files for CurseForge mods",
    "url": "/wiki/modpack-management/packwiz#features",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz / How packwiz relates to the rest of this section",
    "pageTitle": "packwiz",
    "sectionTitle": "How packwiz relates to the rest of this section",
    "description": "",
    "content": "packwiz is the authoring CLI and metadata format. packwiz installer is the runtime updater players and servers execute. The bootstrap is the tiny launcher facing shim that updates and starts the installer. packwand uses the same general pack format but adds more repository aware tooling on top.",
    "url": "/wiki/modpack-management/packwiz#how-packwiz-relates-to-the-rest-of-this-section",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz / Recommended author workflow",
    "pageTitle": "packwiz",
    "sectionTitle": "Recommended author workflow",
    "description": "",
    "content": "1. Create a clean repository for the pack. 2. Initialize the manifest with packwiz. 3. Add mods through metadata commands rather than copying JARs into source control. 4. Commit the resulting manifest and config changes to Git. 5. Test distribution through a local server, export, or installer flow before publishing.",
    "url": "/wiki/modpack-management/packwiz#recommended-author-workflow",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz / Useful links",
    "pageTitle": "packwiz",
    "sectionTitle": "Useful links",
    "description": "",
    "content": "packwiz repository example pack third party GUI project upstream Discord",
    "url": "/wiki/modpack-management/packwiz#useful-links",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz / Where packwiz fits best",
    "pageTitle": "packwiz",
    "sectionTitle": "Where packwiz fits best",
    "description": "",
    "content": "packwiz is well suited to: Single pack repositories Private packs for friends, servers, or internal testing Creator workflows where the manifest format matters more than repository automation Teams that want a stable, Git friendly format without the packwand specific surfaces",
    "url": "/wiki/modpack-management/packwiz#where-packwiz-fits-best",
    "tags": []
  },
  {
    "kind": "section",
    "title": "packwiz / Where packwiz is intentionally smaller",
    "pageTitle": "packwiz",
    "sectionTitle": "Where packwiz is intentionally smaller",
    "description": "",
    "content": "packwiz is not trying to be a repository orchestration tool. It is lighter on: multi pack workspace management release planning and verification workflows repository diffing and diagnostics local GUI or API surfaces If you want those higher level workflows, move up to packwand.",
    "url": "/wiki/modpack-management/packwiz#where-packwiz-is-intentionally-smaller",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Project Management",
    "pageTitle": "Project Management",
    "sectionTitle": "",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "Project Management Modpack development is usually a long running project. Good organization matters because content, testing, balance work, and publishing all compete for time. Start with a release target Before you expand a pack, define what a first public release actually needs to include. That is your minimum viable project: the smallest complete version that players can install and enjoy. A narrow but polished release is usually better than a broad, unfinished one. If you can finish one progression tier, one gameplay loop, or one content pillar completely, you will understand the rest of the pack much better. Break work into reviewable chunks Once the direction is clear, split work into concrete tasks such as: integrating one mod fully finishing one quest chapter balancing one progression tier testing one worldgen pass preparing one publishable release candidate That makes estimation easier and keeps the project from feeling permanently half finished. Working with mod authors When you hit a mod bug or need a feature, start with the project's issue tracker and bring logs, versions, reproduction steps, and a minimal test case when possible. Avoid direct messages unless the author explicitly invites them. Clear bug reports and patience usually get better results than urgency. Working without upstream support Sometimes a mod is effectively unmaintained on the version you need. In that case your choices are usually: configure around the bug remove the affected content patch it locally if you have the technical ability replace the mod entirely The safest defense is still early testing. Validate a mod before it becomes central to your progression. Avoid scope creep Scope creep happens when many small additions slowly turn the project into something much larger than planned. A few extra mods, systems, or side mechanics can multiply testing and integration work. Keep a written plan and be conservative about additions after the pack's direction is stable. Utility fixes are usually fine. New major systems usually are not. Playtest deliberately A full playthrough matters, but repeated early game playtesting matters even more. The first hours of a pack shape whether players stay long enough to see the rest. Replaying the opening several times and fixing friction there is usually worth more than polishing an endgame few players will reach.",
    "url": "/wiki/modpack-management/project-management",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Project Management / Avoid scope creep",
    "pageTitle": "Project Management",
    "sectionTitle": "Avoid scope creep",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "Scope creep happens when many small additions slowly turn the project into something much larger than planned. A few extra mods, systems, or side mechanics can multiply testing and integration work. Keep a written plan and be conservative about additions after the pack's direction is stable. Utility fixes are usually fine. New major systems usually are not.",
    "url": "/wiki/modpack-management/project-management#avoid-scope-creep",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Project Management / Break work into reviewable chunks",
    "pageTitle": "Project Management",
    "sectionTitle": "Break work into reviewable chunks",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "Once the direction is clear, split work into concrete tasks such as: integrating one mod fully finishing one quest chapter balancing one progression tier testing one worldgen pass preparing one publishable release candidate That makes estimation easier and keeps the project from feeling permanently half finished.",
    "url": "/wiki/modpack-management/project-management#break-work-into-reviewable-chunks",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Project Management / Playtest deliberately",
    "pageTitle": "Project Management",
    "sectionTitle": "Playtest deliberately",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "A full playthrough matters, but repeated early game playtesting matters even more. The first hours of a pack shape whether players stay long enough to see the rest. Replaying the opening several times and fixing friction there is usually worth more than polishing an endgame few players will reach.",
    "url": "/wiki/modpack-management/project-management#playtest-deliberately",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Project Management / Start with a release target",
    "pageTitle": "Project Management",
    "sectionTitle": "Start with a release target",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "Before you expand a pack, define what a first public release actually needs to include. That is your minimum viable project: the smallest complete version that players can install and enjoy. A narrow but polished release is usually better than a broad, unfinished one. If you can finish one progression tier, one gameplay loop, or one content pillar completely, you will understand the rest of the pack much better.",
    "url": "/wiki/modpack-management/project-management#start-with-a-release-target",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Project Management / Working with mod authors",
    "pageTitle": "Project Management",
    "sectionTitle": "Working with mod authors",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "When you hit a mod bug or need a feature, start with the project's issue tracker and bring logs, versions, reproduction steps, and a minimal test case when possible. Avoid direct messages unless the author explicitly invites them. Clear bug reports and patience usually get better results than urgency.",
    "url": "/wiki/modpack-management/project-management#working-with-mod-authors",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Project Management / Working without upstream support",
    "pageTitle": "Project Management",
    "sectionTitle": "Working without upstream support",
    "description": "How to scope, organize, and finish a modpack project without losing momentum.",
    "content": "Sometimes a mod is effectively unmaintained on the version you need. In that case your choices are usually: configure around the bug remove the affected content patch it locally if you have the technical ability replace the mod entirely The safest defense is still early testing. Validate a mod before it becomes central to your progression.",
    "url": "/wiki/modpack-management/project-management#working-without-upstream-support",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Management / Choosing a tool",
    "pageTitle": "Pack Management",
    "sectionTitle": "Choosing a tool",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "content": "Need Best fit Why Maintain one or more packs as metadata in Git packwand Adds higher level workflows on top of packwiz, including workspace operations, publishing, diffing, validation, and a local GUI/API Maintain a single pack with a smaller CLI surface packwiz Mature TOML based pack format with straightforward install/export workflows Distribute updates to players or servers packwiz components The bootstrap and installer handle launch time updates and optional mods Produce a hosted .mrpack from an existing manifest Pakku Useful when you want an external exporter around the packwiz style ecosystem",
    "url": "/wiki/modpack-management#choosing-a-tool",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Management / Mental model",
    "pageTitle": "Pack Management",
    "sectionTitle": "Mental model",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "content": "packwiz defines the core manifest format and the classic CLI workflow. packwand builds on that format and adds repository automation, publishing, diagnostics, and multi pack support. packwiz installer and the bootstrap are runtime delivery tools for players and servers. Hosting targets such as CurseForge and Modrinth are output formats and distribution channels, not your source of truth.",
    "url": "/wiki/modpack-management#mental-model",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Management / Platforms",
    "pageTitle": "Pack Management",
    "sectionTitle": "Platforms",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "content": "CurseForge Modrinth Project management",
    "url": "/wiki/modpack-management#platforms",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Management / Recommended workflows",
    "pageTitle": "Pack Management",
    "sectionTitle": "Recommended workflows",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "content": "1. Author the pack as metadata, not as a folder of downloaded JARs. 2. Keep the pack in Git so recipe, config, and dependency changes are reviewable. 3. Use packwand when you need repository aware workflows such as workspace sync, bulk updates, publishing plans, or diagnostics. 4. Use packwiz directly when you want the smaller original toolchain and you do not need the packwand specific repository automation. 5. Test the player install path with the bootstrap and installer before you publish a release.",
    "url": "/wiki/modpack-management#recommended-workflows",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Pack Management / Tools",
    "pageTitle": "Pack Management",
    "sectionTitle": "Tools",
    "description": "Tools, publishing targets, and workflows for managing Minecraft packs in this repository.",
    "content": "packwand packwiz packwiz components Pakku",
    "url": "/wiki/modpack-management#tools",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Ideation",
    "pageTitle": "Ideation",
    "sectionTitle": "",
    "description": "Creating a core concept for your pack",
    "content": "Ideation Despite modpacks being loose collections of mods, there's a lot of work that goes into \"gluing\" these mods together to make a cohesive experience. This involves planning out progression, removing unused or unwanted contents from mods, and just general game design. This page will take you through the planning phase of your modpack, and what you can expect to have to take on when making a modpack. Prerequisites Most of this works involves editing json files, writing KubeJS/Craftweaker scripts, and editing configs. It's essential you have have the following (or equivalents) to keep yourself and your pack properly organized: Text editor like VSCodium or Visual Studio Code Version Control for ensuring work doesn't get deleted Some tool to take notes such as Obsidian or any other text editors/paper Some other things that will improve your experience creating modpacks is technical knowledge of Minecraft, experience playing lots of different kinds of modpacks, and general programming experience. None of these things are required though by any means. Most importantly, you must pick a version of Minecraft to build your pack on. Often times this decision isn't really up to you. You already likely have a core mod or two you're centering your pack on, so the best call is to build your pack on a version those mod authors support. Try not to get swept up in conversations about which version of Minecraft is superior for modded minecraft. Ultimately this question only comes down to how good a given version is for your pack specifically. At the same time, know your audience. If you're making a more vanilla like modpack, your audience will have less of a tolerance for older versions of Minecraft. If you're making a Gregtech modpack, that audience might feel the exact opposite way! Picking a concept Deciding to create a modpack will likely come from having a specific idea or type of pack you'd like to see. Other times it can be based off an existing game or game genre. Often times inspiration can come from playing a pack and noticing things you would tweak or add to. Maybe a core mod such as Create can be replaced by a more niche option you prefer such as Crossroads. Playing packs is a good way to shape and solidify your preferences. At worst, playing other modpacks will give you a better sense at what mods are out there, and give ideas on how to use them. If you're just starting out, the best concept you might want to try is a \"stuff I like pack\". Just throw together a bunch of mods, and tweak some recipes to add progression. Experimentation can lead you to a stronger idea later on, while still building your skills and experience.",
    "url": "/wiki/planning/ideation",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Ideation / Picking a concept",
    "pageTitle": "Ideation",
    "sectionTitle": "Picking a concept",
    "description": "Creating a core concept for your pack",
    "content": "Deciding to create a modpack will likely come from having a specific idea or type of pack you'd like to see. Other times it can be based off an existing game or game genre. Often times inspiration can come from playing a pack and noticing things you would tweak or add to. Maybe a core mod such as Create can be replaced by a more niche option you prefer such as Crossroads. Playing packs is a good way to shape and solidify your preferences. At worst, playing other modpacks will give you a better sense at what mods are out there, and give ideas on how to use them. If you're just starting out, the best concept you might want to try is a \"stuff I like pack\". Just throw together a bunch of mods, and tweak some recipes to add progression. Experimentation can lead you to a stronger idea later on, while still building your skills and experience.",
    "url": "/wiki/planning/ideation#picking-a-concept",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Ideation / Prerequisites",
    "pageTitle": "Ideation",
    "sectionTitle": "Prerequisites",
    "description": "Creating a core concept for your pack",
    "content": "Most of this works involves editing json files, writing KubeJS/Craftweaker scripts, and editing configs. It's essential you have have the following (or equivalents) to keep yourself and your pack properly organized: Text editor like VSCodium or Visual Studio Code Version Control for ensuring work doesn't get deleted Some tool to take notes such as Obsidian or any other text editors/paper Some other things that will improve your experience creating modpacks is technical knowledge of Minecraft, experience playing lots of different kinds of modpacks, and general programming experience. None of these things are required though by any means. Most importantly, you must pick a version of Minecraft to build your pack on. Often times this decision isn't really up to you. You already likely have a core mod or two you're centering your pack on, so the best call is to build your pack on a version those mod authors support. Try not to get swept up in conversations about which version of Minecraft is superior for modded minecraft. Ultimately this question only comes down to how good a given version is for your pack specifically. At the same time, know your audience. If you're making a more vanilla like modpack, your audience will have less of a tolerance for older versions of Minecraft. If you're making a Gregtech modpack, that audience might feel the exact opposite way!",
    "url": "/wiki/planning/ideation#prerequisites",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Mod Selection",
    "pageTitle": "Mod Selection",
    "sectionTitle": "",
    "description": "Choosing which mods to add to your modpack",
    "content": "Mod Selection The mod selection phase is both the most important most fun part of planning a modpack. This is not a good thing! Scrolling through the list of mods and adding whatever catches your eye can be incredibly appealing, and will often lead to you being saddled with mods that are a bad fit for your pack and difficult to remove. A good way to avoid this trap is by having two separate instances for a modpack: one for mods you're certain you will include, and one for just trying out random mods. You can do this easily by first picking out performance/utility mods that are set in stone and duplicating that instance in the launcher you use. While these tips aren't exhaustive nor something needed to be followed to the letter, here are some general guidelines for curating a modlist suitable for building a pack out of. Avoid overlap This can come in many forms, whether it be in similar blocks/items/mechanics/systems between mods. Two mods adding different magic systems or having duplicate blocksets can be confusing to players and add more work to the developer to make work together cohesively. However, this isn't to say that you should never add similar mods, as streamlining and integrating mods together is something you can do to make your pack unique! Just be aware of the extra workload it may take to pull it off. Avoid adding mods that don't compliment the premise of the pack It sounds obvious, but it's very easy to get carried away in what mods to add! You may want to reconsider what value a mod like Create would bring to an adventure pack, or what a mod like Apotheosis would bring to a tech pack. Be wary of closed source mods Due to not having easily accessible source code, closed source mods are less likely to be supported by other mod developers in cases of compatibility issues or integration requests. Additionally, closed source mods are less likely to receive contributions, and if the developer of the mod is unwilling or unable to update the mod in cases of fixing critical issues or modloader / version ports, it may be left in an unusable state for the purposes of your modpack. Similar issues may be present in mods that are visible source but with a restrictive license such as All Rights Reserved, inhibiting the ability to make forks / fixes for the mod. Performance/utility mods These mods are the bedrock of every pack, and will likely not vary too much no matter what kind of modpack you are creating. They tend to fall under a few categories: Performance mods such as Sodium, Embeddium, and Modernfix. A recipe viewer like JEI, EMI, or equivalents. Pack tweaking utility mods such as KubeJS or Craftweaker. Some template packs may exist depending on the chosen minecraft version, though it's important to look through every mod and tweak the list if it doesn't fit your pack's needs. Core mods Some mods are non negotiable to your pack. They will be what you center progression and gameplay around, so it's important to pick reliable and easily customizable mods for this category. Additionally, they must have enough of their own content to provide a cohesive experience, with help from maybe a handful of supporting mods. Some mods like these are Ars Nouveau, Gregtech, Create, etc. They are your anchor points, and centering your pack around them will make planning significantly easier. You can also choose to forgo major content mods and center your pack around non content mods. Skyblock Burgeria for example is a modpack that centers around a custom mechanic written in KubeJS. Supporting mods There are some mods that act as \"glue\" in a given pack. More often than not, they fulfill one or a few different roles that are needed to support the core mod(s) in your modpack. Some easy examples of these are storage mods such as Applied Energistics 2 or Functional Storage, item transportation mods like Pretty Pipes, or biome mods such as Regions Unexplored. These mods should have a purpose in your pack, and each one will increase your overall workload as they'll bring more content to integrate (change recipes, remove items, etc). You don't want to mindlessly add them into your modpack, but a lack of these might be frustrating to your target audience. Fluff mods \"Fluff\" mods are those that don't contribute to the core gameplay of your pack, but add something meaningful certain types of players might enjoy, or you just have a preference for. This is also fairly genre and audience dependent. Mods centered around technical automation mods will appeal to players that aren't necessarily interested in normal decor mods. Look for ones that add more factory themed blocks or ones that give players a clean palette. It can be dangerous to add too many of these mods! Think carefully about each one and don't just install every fluff mod that catches your eye.",
    "url": "/wiki/planning/mod-selection",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Mod Selection / Core mods",
    "pageTitle": "Mod Selection",
    "sectionTitle": "Core mods",
    "description": "Choosing which mods to add to your modpack",
    "content": "Some mods are non negotiable to your pack. They will be what you center progression and gameplay around, so it's important to pick reliable and easily customizable mods for this category. Additionally, they must have enough of their own content to provide a cohesive experience, with help from maybe a handful of supporting mods. Some mods like these are Ars Nouveau, Gregtech, Create, etc. They are your anchor points, and centering your pack around them will make planning significantly easier. You can also choose to forgo major content mods and center your pack around non content mods. Skyblock Burgeria for example is a modpack that centers around a custom mechanic written in KubeJS.",
    "url": "/wiki/planning/mod-selection#core-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Mod Selection / Fluff mods",
    "pageTitle": "Mod Selection",
    "sectionTitle": "Fluff mods",
    "description": "Choosing which mods to add to your modpack",
    "content": "\"Fluff\" mods are those that don't contribute to the core gameplay of your pack, but add something meaningful certain types of players might enjoy, or you just have a preference for. This is also fairly genre and audience dependent. Mods centered around technical automation mods will appeal to players that aren't necessarily interested in normal decor mods. Look for ones that add more factory themed blocks or ones that give players a clean palette. It can be dangerous to add too many of these mods! Think carefully about each one and don't just install every fluff mod that catches your eye.",
    "url": "/wiki/planning/mod-selection#fluff-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Mod Selection / Performance/utility mods",
    "pageTitle": "Mod Selection",
    "sectionTitle": "Performance/utility mods",
    "description": "Choosing which mods to add to your modpack",
    "content": "These mods are the bedrock of every pack, and will likely not vary too much no matter what kind of modpack you are creating. They tend to fall under a few categories: Performance mods such as Sodium, Embeddium, and Modernfix. A recipe viewer like JEI, EMI, or equivalents. Pack tweaking utility mods such as KubeJS or Craftweaker. Some template packs may exist depending on the chosen minecraft version, though it's important to look through every mod and tweak the list if it doesn't fit your pack's needs.",
    "url": "/wiki/planning/mod-selection#performanceutility-mods",
    "tags": []
  },
  {
    "kind": "section",
    "title": "Mod Selection / Supporting mods",
    "pageTitle": "Mod Selection",
    "sectionTitle": "Supporting mods",
    "description": "Choosing which mods to add to your modpack",
    "content": "There are some mods that act as \"glue\" in a given pack. More often than not, they fulfill one or a few different roles that are needed to support the core mod(s) in your modpack. Some easy examples of these are storage mods such as Applied Energistics 2 or Functional Storage, item transportation mods like Pretty Pipes, or biome mods such as Regions Unexplored. These mods should have a purpose in your pack, and each one will increase your overall workload as they'll bring more content to integrate (change recipes, remove items, etc). You don't want to mindlessly add them into your modpack, but a lack of these might be frustrating to your target audience.",
    "url": "/wiki/planning/mod-selection#supporting-mods",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Scope",
    "pageTitle": "Scope",
    "sectionTitle": "",
    "description": "Modpack scope and how to limit it",
    "content": "Scope The scope of a project is its defined set of features, planned or implemented. In modpack development this can be quantified in a few ways: How many major content mods a pack has How much custom content is planned How many mechanics are in a pack It may be tempting to start off planning the largest and most interesting pack you can dream of. An easy example of this could be: \"A fully fledged RPG modpack with dialog, custom loot system, player leveling, and interesting boss fights.\" This project seems interesting enough, and there are enough RPG games out there to draw inspiration from and come up with a lot of good ideas. The problems will arrive when you start to implement all these systems. Going through the list of features, you can break down each of them into a series of sub tasks. Dialog system: Find a mod that can support dialogs, custom NPCs, or both Make or source skins for each of the NPCs Figure out how to put them in the world Write hundreds of lines of high effort dialog Come up and implement reasons to engage with the dialog system, such as quests Find suitable items to put as quest rewards. Maybe money? Add shops the player can purchase items from But what about selling items? Should you come up with an economy system? This is a ton of work just for one of the 4 core features of a modpack. Even this one alone might take a month or so of development time, assuming you're working for several hours a day. Instead of trying to make a pack that does all of these things, try focusing on doing one or two things really well. It allows you to flesh out systems in your modpack while also leaving room to add the other two later on after the pack is released. You'll also notice in the above example something called scope creep . This term refers to the act of adding small features here and there that both directly and indirectly increase the scope of your project. In this example, something that starts off as a simple dialog system has turned into a full questing and economy system just in the planning phase! It's important to be your own biggest hater! Always assume things will be much more difficult to do than you think they are, and if you have good ideas, write them down and save them for future updates. The Project Management page has more information on preventing scope creep and following through on plans you create.",
    "url": "/wiki/planning/scope",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Useful Mods",
    "pageTitle": "Useful Mods",
    "sectionTitle": "",
    "description": "Library of useful mods",
    "content": "Useful Mods Library List of useful mods to include in modpacks based on category and Minecraft version Category 1.19.2 1.20.1 1.21.1 26.1 Performance Forge Fabric Forge Fabric Neoforge Fabric Neoforge Fabric Utility Forge Fabric Forge Fabric Neoforge Fabric Neoforge Fabric Bug Fixes Forge Fabric Forge Fabric Neoforge Fabric Neoforge Fabric Profiling Forge Fabric Forge Fabric Neoforge Fabric Neoforge Fabric Documentation Forge Fabric Forge Fabric Neoforge Fabric Neoforge Fabric Multiplayer Forge Fabric Forge Fabric Neoforge Fabric Neoforge Fabric",
    "url": "/wiki/useful-mods",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Fabric 1.19.2",
    "pageTitle": "Bug Fixes mods for Fabric 1.19.2",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Fabric 1.19.2",
    "content": "Bug Fixes mods for Fabric 1.19.2 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.19.2/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Forge 1.19.2",
    "pageTitle": "Bug Fixes mods for Forge 1.19.2",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Forge 1.19.2",
    "content": "Bug Fixes mods for Forge 1.19.2 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.19.2/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for NeoForge 1.19.2",
    "pageTitle": "Bug Fixes mods for NeoForge 1.19.2",
    "sectionTitle": "",
    "description": "Bug Fixes mods for NeoForge 1.19.2",
    "content": "Bug Fixes mods for NeoForge 1.19.2 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.19.2/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Fabric 1.20.1",
    "pageTitle": "Bug Fixes mods for Fabric 1.20.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Fabric 1.20.1",
    "content": "Bug Fixes mods for Fabric 1.20.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. FTB Quests Freeze Fix CF / MR Caches the FTB Quests menu on startup to prevent freezing in game FTB Quests Kill Task Tweaks CF Improves kill contribution tracking for FTB Quests Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.20.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Forge 1.20.1",
    "pageTitle": "Bug Fixes mods for Forge 1.20.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Forge 1.20.1",
    "content": "Bug Fixes mods for Forge 1.20.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. FTB Quests Freeze Fix CF / MR Caches the FTB Quests menu on startup to prevent freezing in game FTB Quests Kill Task Tweaks CF Improves kill contribution tracking for FTB Quests Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.20.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for NeoForge 1.20.1",
    "pageTitle": "Bug Fixes mods for NeoForge 1.20.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for NeoForge 1.20.1",
    "content": "Bug Fixes mods for NeoForge 1.20.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. FTB Quests Kill Task Tweaks CF Improves kill contribution tracking for FTB Quests Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.20.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Fabric 1.21.1",
    "pageTitle": "Bug Fixes mods for Fabric 1.21.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Fabric 1.21.1",
    "content": "Bug Fixes mods for Fabric 1.21.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. FTB Quests Kill Task Tweaks CF Improves kill contribution tracking for FTB Quests Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.21.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Forge 1.21.1",
    "pageTitle": "Bug Fixes mods for Forge 1.21.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Forge 1.21.1",
    "content": "Bug Fixes mods for Forge 1.21.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. FTB Quests Kill Task Tweaks CF Improves kill contribution tracking for FTB Quests Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.21.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for NeoForge 1.21.1",
    "pageTitle": "Bug Fixes mods for NeoForge 1.21.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for NeoForge 1.21.1",
    "content": "Bug Fixes mods for NeoForge 1.21.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. FTB Quests Kill Task Tweaks CF Improves kill contribution tracking for FTB Quests Load My Fucking Tags CF / MR Prevents Incorrect Tag Entries from breaking an entire Tag Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/1.21.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Fabric 26.1",
    "pageTitle": "Bug Fixes mods for Fabric 26.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Fabric 26.1",
    "content": "Bug Fixes mods for Fabric 26.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/26.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for Forge 26.1",
    "pageTitle": "Bug Fixes mods for Forge 26.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for Forge 26.1",
    "content": "Bug Fixes mods for Forge 26.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/26.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Bug Fixes mods for NeoForge 26.1",
    "pageTitle": "Bug Fixes mods for NeoForge 26.1",
    "sectionTitle": "",
    "description": "Bug Fixes mods for NeoForge 26.1",
    "content": "Bug Fixes mods for NeoForge 26.1 These mods fix bugs present in Vanilla Minecraft, other mods, or in loaders. Name Links Description Alternatives AttributeFix CF / MR Removes arbitrary limits on Minecraft's attribute system. Able to modify the base attributes of players/mobs, only change the caps. For that functionality, check out our guide here. Max Health Fix CF / MR Fixes an old bug in Minecraft that causes the \"Max Health\" attribute to be ignored when a player joins the game. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Simple Snowy Fix CF / MR) Fixes incorrect snow generation on tree leaves Too Fast CF / MR Removes the server side limitations on player speed that result in “player XYZ moved too fast” messages in the console and rubber banding. I'm Fast \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/bug_fixes/26.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Fabric 1.19.2",
    "pageTitle": "Documentation mods for Fabric 1.19.2",
    "sectionTitle": "",
    "description": "Documentation mods for Fabric 1.19.2",
    "content": "Documentation mods for Fabric 1.19.2 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.19.2/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Forge 1.19.2",
    "pageTitle": "Documentation mods for Forge 1.19.2",
    "sectionTitle": "",
    "description": "Documentation mods for Forge 1.19.2",
    "content": "Documentation mods for Forge 1.19.2 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.19.2/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for NeoForge 1.19.2",
    "pageTitle": "Documentation mods for NeoForge 1.19.2",
    "sectionTitle": "",
    "description": "Documentation mods for NeoForge 1.19.2",
    "content": "Documentation mods for NeoForge 1.19.2 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.19.2/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Fabric 1.20.1",
    "pageTitle": "Documentation mods for Fabric 1.20.1",
    "sectionTitle": "",
    "description": "Documentation mods for Fabric 1.20.1",
    "content": "Documentation mods for Fabric 1.20.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.20.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Forge 1.20.1",
    "pageTitle": "Documentation mods for Forge 1.20.1",
    "sectionTitle": "",
    "description": "Documentation mods for Forge 1.20.1",
    "content": "Documentation mods for Forge 1.20.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod GuideME CF / MR A guidebook toolkit for mods and modpack makers alike with comfortable markdown formatting, and live 3d scenes Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.20.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for NeoForge 1.20.1",
    "pageTitle": "Documentation mods for NeoForge 1.20.1",
    "sectionTitle": "",
    "description": "Documentation mods for NeoForge 1.20.1",
    "content": "Documentation mods for NeoForge 1.20.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod GuideME CF / MR A guidebook toolkit for mods and modpack makers alike with comfortable markdown formatting, and live 3d scenes Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.20.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Fabric 1.21.1",
    "pageTitle": "Documentation mods for Fabric 1.21.1",
    "sectionTitle": "",
    "description": "Documentation mods for Fabric 1.21.1",
    "content": "Documentation mods for Fabric 1.21.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives Domix's Guidebook CF / MR Simple Data Driven Library for creating Guidebooks. FTB Quests CF GUI based questing mod Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system Wikiful CF / MR A simple data driven tip and wiki library mod \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.21.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Forge 1.21.1",
    "pageTitle": "Documentation mods for Forge 1.21.1",
    "sectionTitle": "",
    "description": "Documentation mods for Forge 1.21.1",
    "content": "Documentation mods for Forge 1.21.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod GuideME CF / MR A guidebook toolkit for mods and modpack makers alike with comfortable markdown formatting, and live 3d scenes Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.21.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for NeoForge 1.21.1",
    "pageTitle": "Documentation mods for NeoForge 1.21.1",
    "sectionTitle": "",
    "description": "Documentation mods for NeoForge 1.21.1",
    "content": "Documentation mods for NeoForge 1.21.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives FTB Quests CF GUI based questing mod GuideME CF / MR A guidebook toolkit for mods and modpack makers alike with comfortable markdown formatting, and live 3d scenes Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Patchouli CF / MR Accessible, Data Driven, Dependency Free Documentation for Minecraft Modders and Pack Makers QuestLog CF / MR Adds an intuitive questing system Wikiful CF / MR A simple data driven tip and wiki library mod \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/1.21.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Fabric 26.1",
    "pageTitle": "Documentation mods for Fabric 26.1",
    "sectionTitle": "",
    "description": "Documentation mods for Fabric 26.1",
    "content": "Documentation mods for Fabric 26.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Wikiful CF / MR A simple data driven tip and wiki library mod \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/26.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for Forge 26.1",
    "pageTitle": "Documentation mods for Forge 26.1",
    "sectionTitle": "",
    "description": "Documentation mods for Forge 26.1",
    "content": "Documentation mods for Forge 26.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives GuideME CF / MR A guidebook toolkit for mods and modpack makers alike with comfortable markdown formatting, and live 3d scenes Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/26.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Documentation mods for NeoForge 26.1",
    "pageTitle": "Documentation mods for NeoForge 26.1",
    "sectionTitle": "",
    "description": "Documentation mods for NeoForge 26.1",
    "content": "Documentation mods for NeoForge 26.1 These mods help provide information to the player and guide them through content in the modpack. Name Links Description Alternatives GuideME CF / MR A guidebook toolkit for mods and modpack makers alike with comfortable markdown formatting, and live 3d scenes Modonomicon CF / MR Data driven minecraft in game documentation with progress visualization Wikiful CF / MR A simple data driven tip and wiki library mod \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/documentation/26.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Fabric 1.19.2",
    "pageTitle": "Free Multiplayer mods for Fabric 1.19.2",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Fabric 1.19.2",
    "content": "Free Multiplayer mods for Fabric 1.19.2 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.19.2/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Forge 1.19.2",
    "pageTitle": "Free Multiplayer mods for Forge 1.19.2",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Forge 1.19.2",
    "content": "Free Multiplayer mods for Forge 1.19.2 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.19.2/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for NeoForge 1.19.2",
    "pageTitle": "Free Multiplayer mods for NeoForge 1.19.2",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for NeoForge 1.19.2",
    "content": "Free Multiplayer mods for NeoForge 1.19.2 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.19.2/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Fabric 1.20.1",
    "pageTitle": "Free Multiplayer mods for Fabric 1.20.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Fabric 1.20.1",
    "content": "Free Multiplayer mods for Fabric 1.20.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. LAN Server Properties CF / MR Enhance the vanilla \"Open to LAN\" Gui for online mode and port customization Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.20.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Forge 1.20.1",
    "pageTitle": "Free Multiplayer mods for Forge 1.20.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Forge 1.20.1",
    "content": "Free Multiplayer mods for Forge 1.20.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. LAN Server Properties CF / MR Enhance the vanilla \"Open to LAN\" Gui for online mode and port customization Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.20.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for NeoForge 1.20.1",
    "pageTitle": "Free Multiplayer mods for NeoForge 1.20.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for NeoForge 1.20.1",
    "content": "Free Multiplayer mods for NeoForge 1.20.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. LAN Server Properties CF / MR Enhance the vanilla \"Open to LAN\" Gui for online mode and port customization Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.20.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Fabric 1.21.1",
    "pageTitle": "Free Multiplayer mods for Fabric 1.21.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Fabric 1.21.1",
    "content": "Free Multiplayer mods for Fabric 1.21.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. LAN Server Properties CF / MR Enhance the vanilla \"Open to LAN\" Gui for online mode and port customization Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.21.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Forge 1.21.1",
    "pageTitle": "Free Multiplayer mods for Forge 1.21.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Forge 1.21.1",
    "content": "Free Multiplayer mods for Forge 1.21.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. LAN Server Properties CF / MR Enhance the vanilla \"Open to LAN\" Gui for online mode and port customization Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.21.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for NeoForge 1.21.1",
    "pageTitle": "Free Multiplayer mods for NeoForge 1.21.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for NeoForge 1.21.1",
    "content": "Free Multiplayer mods for NeoForge 1.21.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. LAN Server Properties CF / MR Enhance the vanilla \"Open to LAN\" Gui for online mode and port customization Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/1.21.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Fabric 26.1",
    "pageTitle": "Free Multiplayer mods for Fabric 26.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Fabric 26.1",
    "content": "Free Multiplayer mods for Fabric 26.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/26.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for Forge 26.1",
    "pageTitle": "Free Multiplayer mods for Forge 26.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for Forge 26.1",
    "content": "Free Multiplayer mods for Forge 26.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/26.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Free Multiplayer mods for NeoForge 26.1",
    "pageTitle": "Free Multiplayer mods for NeoForge 26.1",
    "sectionTitle": "",
    "description": "Free Multiplayer mods for NeoForge 26.1",
    "content": "Free Multiplayer mods for NeoForge 26.1 These mods allow for free multiplayer without the cost or hassle of setting up a server. Name Links Description Alternatives E4MC CF / MR Open a LAN server to anyone, anywhere, anytime. Relies on donations to tunnel servers. Modflared CF / MR Automatically connects you to a Cloudflare tunnel without having to install cloudflared separately. CloudFlared for Forge 1.20.1. Windows antivirus can delete the downloaded exe from Cloudflare, causing a crash on startup. \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/multiplayer/26.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Fabric 1.19.2",
    "pageTitle": "Performance mods for Fabric 1.19.2",
    "sectionTitle": "",
    "description": "Performance mods for Fabric 1.19.2",
    "content": "Performance mods for Fabric 1.19.2 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Bobby CF / MR Allows for render distances greater than the server's view distance setting. C2ME MR Allows chunkloading to be multithreaded, increasing worldgen speeds. Has frequent issues with mods that touch worldgen or threading Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds Ferritecore CF / MR Memory usage optimizations ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Quick Pack CF / MR Improves datapack / resourcepack zip file loading times QuickBench CF / MR FastBench for Fabric ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.19.2/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Forge 1.19.2",
    "pageTitle": "Performance mods for Forge 1.19.2",
    "sectionTitle": "",
    "description": "Performance mods for Forge 1.19.2",
    "content": "Performance mods for Forge 1.19.2 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Bobby Reforged CF Allows for render distances greater than the server’s view distance setting Farsight, though this mod lacks a chunk caching feature. Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds FastFurnace CF A performance upgrade for the furnace FastSuite CF A performance upgrade for the JSON recipe system Recipe Essentials Ferritecore CF / MR Memory usage optimizations ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Quick Pack CF / MR Improves datapack / resourcepack zip file loading times ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Structure Essentials CF Performance improvements for structure searching \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.19.2/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for NeoForge 1.19.2",
    "pageTitle": "Performance mods for NeoForge 1.19.2",
    "sectionTitle": "",
    "description": "Performance mods for NeoForge 1.19.2",
    "content": "Performance mods for NeoForge 1.19.2 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds FastFurnace CF A performance upgrade for the furnace FastSuite CF A performance upgrade for the JSON recipe system Recipe Essentials Ferritecore CF / MR Memory usage optimizations ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Quick Pack CF / MR Improves datapack / resourcepack zip file loading times ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.19.2/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Fabric 1.20.1",
    "pageTitle": "Performance mods for Fabric 1.20.1",
    "sectionTitle": "",
    "description": "Performance mods for Fabric 1.20.1",
    "content": "Performance mods for Fabric 1.20.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Bobby CF / MR Allows for render distances greater than the server's view distance setting. C2ME MR Allows chunkloading to be multithreaded, increasing worldgen speeds. Has frequent issues with mods that touch worldgen or threading Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds Ferritecore CF / MR Memory usage optimizations FTB Quests Freeze Fix CF / MR Caches the FTB Quests menu on startup to prevent freezing in game Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times QuickBench CF / MR FastBench for Fabric ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching TerraBlenderFix MR Performance mod targeting Terrablender surface rule injection methods. Reportedly breaks surface rule injections with certain Terrablender mods \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.20.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Forge 1.20.1",
    "pageTitle": "Performance mods for Forge 1.20.1",
    "sectionTitle": "",
    "description": "Performance mods for Forge 1.20.1",
    "content": "Performance mods for Forge 1.20.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Bobby Reforged CF Allows for render distances greater than the server’s view distance setting Farsight, though this mod lacks a chunk caching feature. Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds FastFurnace CF A performance upgrade for the furnace FastSuite CF A performance upgrade for the JSON recipe system Recipe Essentials Ferritecore CF / MR Memory usage optimizations Flerovium CF / MR Greatly improve your fps with virtually no side effects on graphics quality FTB Quests Freeze Fix CF / MR Caches the FTB Quests menu on startup to prevent freezing in game Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times Radium CF / MR Radium is an Unofficial Fork of CaffeineMC's 'Lithium' Lithium on relevant versions ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Structure Essentials CF Performance improvements for structure searching TerraBlenderFix MR Performance mod targeting Terrablender surface rule injection methods. Reportedly breaks surface rule injections with certain Terrablender mods \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.20.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for NeoForge 1.20.1",
    "pageTitle": "Performance mods for NeoForge 1.20.1",
    "sectionTitle": "",
    "description": "Performance mods for NeoForge 1.20.1",
    "content": "Performance mods for NeoForge 1.20.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds FastFurnace CF A performance upgrade for the furnace FastSuite CF A performance upgrade for the JSON recipe system Recipe Essentials Ferritecore CF / MR Memory usage optimizations Flerovium CF / MR Greatly improve your fps with virtually no side effects on graphics quality Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times Radium CF / MR Radium is an Unofficial Fork of CaffeineMC's 'Lithium' Lithium on relevant versions ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching TerraBlenderFix MR Performance mod targeting Terrablender surface rule injection methods. Reportedly breaks surface rule injections with certain Terrablender mods \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.20.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Fabric 1.21.1",
    "pageTitle": "Performance mods for Fabric 1.21.1",
    "sectionTitle": "",
    "description": "Performance mods for Fabric 1.21.1",
    "content": "Performance mods for Fabric 1.21.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Bobby CF / MR Allows for render distances greater than the server's view distance setting. C2ME MR Allows chunkloading to be multithreaded, increasing worldgen speeds. Has frequent issues with mods that touch worldgen or threading Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds Ferritecore CF / MR Memory usage optimizations Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times QuickBench CF / MR FastBench for Fabric ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching TerraBlenderFix MR Performance mod targeting Terrablender surface rule injection methods. Reportedly breaks surface rule injections with certain Terrablender mods \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.21.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Forge 1.21.1",
    "pageTitle": "Performance mods for Forge 1.21.1",
    "sectionTitle": "",
    "description": "Performance mods for Forge 1.21.1",
    "content": "Performance mods for Forge 1.21.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds FastFurnace CF A performance upgrade for the furnace FastSuite CF A performance upgrade for the JSON recipe system Recipe Essentials Ferritecore CF / MR Memory usage optimizations Flerovium CF / MR Greatly improve your fps with virtually no side effects on graphics quality Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times Radium CF / MR Radium is an Unofficial Fork of CaffeineMC's 'Lithium' Lithium on relevant versions ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Structure Essentials CF Performance improvements for structure searching TerraBlenderFix MR Performance mod targeting Terrablender surface rule injection methods. Reportedly breaks surface rule injections with certain Terrablender mods \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.21.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for NeoForge 1.21.1",
    "pageTitle": "Performance mods for NeoForge 1.21.1",
    "sectionTitle": "",
    "description": "Performance mods for NeoForge 1.21.1",
    "content": "Performance mods for NeoForge 1.21.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Embeddium CF / MR Rendering optimizations Xenon and Sodium for relevant versions Fast Noise CF / MR Optimizes worldgen speeds FastFurnace CF A performance upgrade for the furnace FastSuite CF A performance upgrade for the JSON recipe system Recipe Essentials Ferritecore CF / MR Memory usage optimizations Flerovium CF / MR Greatly improve your fps with virtually no side effects on graphics quality Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times Radium CF / MR Radium is an Unofficial Fork of CaffeineMC's 'Lithium' Lithium on relevant versions ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching TerraBlenderFix MR Performance mod targeting Terrablender surface rule injection methods. Reportedly breaks surface rule injections with certain Terrablender mods \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/1.21.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Fabric 26.1",
    "pageTitle": "Performance mods for Fabric 26.1",
    "sectionTitle": "",
    "description": "Performance mods for Fabric 26.1",
    "content": "Performance mods for Fabric 26.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Bobby CF / MR Allows for render distances greater than the server's view distance setting. C2ME MR Allows chunkloading to be multithreaded, increasing worldgen speeds. Has frequent issues with mods that touch worldgen or threading Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Fast Noise CF / MR Optimizes worldgen speeds Ferritecore CF / MR Memory usage optimizations Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/26.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for Forge 26.1",
    "pageTitle": "Performance mods for Forge 26.1",
    "sectionTitle": "",
    "description": "Performance mods for Forge 26.1",
    "content": "Performance mods for Forge 26.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Fast Noise CF / MR Optimizes worldgen speeds Ferritecore CF / MR Memory usage optimizations Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Structure Essentials CF Performance improvements for structure searching \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/26.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Performance mods for NeoForge 26.1",
    "pageTitle": "Performance mods for NeoForge 26.1",
    "sectionTitle": "",
    "description": "Performance mods for NeoForge 26.1",
    "content": "Performance mods for NeoForge 26.1 These mods directly improve modpack performance through code optimizations. Name Links Description Alternatives Alternate Current CF / MR Vanilla compatible redstone optimizations Connectivity CF Fix Login timeouts, Packet sizes errors, Payloads errors, ghostblocks and more Packet Fixer Dynamic FPS CF / MR Improve performance when Minecraft is in the background FPS Reducer Fast Noise CF / MR Optimizes worldgen speeds Ferritecore CF / MR Memory usage optimizations Gnetum CF / MR Distribute HUD updates over multiple frames to improve performance ImmediatelyFast CF / MR Speed up immediate mode rendering in Minecraft Ixeris CF / MR Buffered raw input and threaded event polling Lithium CF / MR General purpose serverside optimization mod Radium on relevant versions ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Particle Core CF / MR Particle optimizations Quick Pack CF / MR Improves datapack / resourcepack zip file loading times ServerCore CF / MR Various serverside optimizations. Breaks vanilla compatibility Sodium CF / MR Rendering optimizations Embeddium on relevant versions Structure Essentials CF Performance improvements for structure searching \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/performance/26.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Fabric 1.19.2",
    "pageTitle": "Profiling/Debugging mods for Fabric 1.19.2",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Fabric 1.19.2",
    "content": "Profiling/Debugging mods for Fabric 1.19.2 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.19.2/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Forge 1.19.2",
    "pageTitle": "Profiling/Debugging mods for Forge 1.19.2",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Forge 1.19.2",
    "content": "Profiling/Debugging mods for Forge 1.19.2 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.19.2/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for NeoForge 1.19.2",
    "pageTitle": "Profiling/Debugging mods for NeoForge 1.19.2",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for NeoForge 1.19.2",
    "content": "Profiling/Debugging mods for NeoForge 1.19.2 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.19.2/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Fabric 1.20.1",
    "pageTitle": "Profiling/Debugging mods for Fabric 1.20.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Fabric 1.20.1",
    "content": "Profiling/Debugging mods for Fabric 1.20.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.20.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Forge 1.20.1",
    "pageTitle": "Profiling/Debugging mods for Forge 1.20.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Forge 1.20.1",
    "content": "Profiling/Debugging mods for Forge 1.20.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.20.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for NeoForge 1.20.1",
    "pageTitle": "Profiling/Debugging mods for NeoForge 1.20.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for NeoForge 1.20.1",
    "content": "Profiling/Debugging mods for NeoForge 1.20.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.20.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Fabric 1.21.1",
    "pageTitle": "Profiling/Debugging mods for Fabric 1.21.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Fabric 1.21.1",
    "content": "Profiling/Debugging mods for Fabric 1.21.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.21.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Forge 1.21.1",
    "pageTitle": "Profiling/Debugging mods for Forge 1.21.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Forge 1.21.1",
    "content": "Profiling/Debugging mods for Forge 1.21.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.21.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for NeoForge 1.21.1",
    "pageTitle": "Profiling/Debugging mods for NeoForge 1.21.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for NeoForge 1.21.1",
    "content": "Profiling/Debugging mods for NeoForge 1.21.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. Cyanide CF / MR Reduces data pack world generation pain by providing useful, informative error messages. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Observable CF / MR Profiles (tile) entities and shows you what’s taking up tick time and where Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/1.21.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Fabric 26.1",
    "pageTitle": "Profiling/Debugging mods for Fabric 26.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Fabric 26.1",
    "content": "Profiling/Debugging mods for Fabric 26.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/26.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for Forge 26.1",
    "pageTitle": "Profiling/Debugging mods for Forge 26.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for Forge 26.1",
    "content": "Profiling/Debugging mods for Forge 26.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/26.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Profiling/Debugging mods for NeoForge 26.1",
    "pageTitle": "Profiling/Debugging mods for NeoForge 26.1",
    "sectionTitle": "",
    "description": "Profiling/Debugging mods for NeoForge 26.1",
    "content": "Profiling/Debugging mods for NeoForge 26.1 These mods help profile and diagnose issues for modpacks, so that issues can be found and fixed. Name Links Description Alternatives Console Filter CF Allows you to stop entries from being logged in game console through phrases or regex Log Begone Crash Assistant CF / MR Shows a GUI after Minecraft crashes, immediately showing and analyzing all affected logs. ModernFix CF / MR All in one mod that improves performance, reduces memory usage, and fixes many bugs Spark CF / MR Performance profiler for Minecraft clients, servers and proxies \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/profiling/26.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Fabric 1.19.2",
    "pageTitle": "Utility mods for Fabric 1.19.2",
    "sectionTitle": "",
    "description": "Utility mods for Fabric 1.19.2",
    "content": "Utility mods for Fabric 1.19.2 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery CF / MR Create custom machines via JSON FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.19.2/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Forge 1.19.2",
    "pageTitle": "Utility mods for Forge 1.19.2",
    "sectionTitle": "",
    "description": "Utility mods for Forge 1.19.2",
    "content": "Utility mods for Forge 1.19.2 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources Bad Mobs CF / MR Simple blacklist for mob spawns Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery CF / MR Create custom machines via JSON Emendatus Enigmatica CF / MR Dynamic material registry system, with world generation, compat, and many other features FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Game Stages CF / MR An API for adding stages for modpacks to use AStages In Control CF / MR Control mob spawns via verbose JSON configuration JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. One Enough Enchantment CF / MR Dynamically control the weight of enchantments or even completely remove enchantments. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi ServerConfigUpdater CF Keeps DefaultConfigs in sync with worlds’ serverConfigs between modpack updates \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.19.2/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for NeoForge 1.19.2",
    "pageTitle": "Utility mods for NeoForge 1.19.2",
    "sectionTitle": "",
    "description": "Utility mods for NeoForge 1.19.2",
    "content": "Utility mods for NeoForge 1.19.2 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources Bad Mobs CF / MR Simple blacklist for mob spawns Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery CF / MR Create custom machines via JSON Emendatus Enigmatica CF / MR Dynamic material registry system, with world generation, compat, and many other features FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge In Control CF / MR Control mob spawns via verbose JSON configuration JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. One Enough Enchantment CF / MR Dynamically control the weight of enchantments or even completely remove enchantments. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.19.2/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Fabric 1.20.1",
    "pageTitle": "Utility mods for Fabric 1.20.1",
    "sectionTitle": "",
    "description": "Utility mods for Fabric 1.20.1",
    "content": "Utility mods for Fabric 1.20.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery (Fork) CF Create custom machines via JSON FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.20.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Forge 1.20.1",
    "pageTitle": "Utility mods for Forge 1.20.1",
    "sectionTitle": "",
    "description": "Utility mods for Forge 1.20.1",
    "content": "Utility mods for Forge 1.20.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources AStages CF / MR Restrict nearly every aspect of gameplay based on progression Game Stages Bad Mobs CF / MR Simple blacklist for mob spawns Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery (Fork) CF Create custom machines via JSON FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. Game Stages CF / MR An API for adding stages for modpacks to use AStages Iglee's Modpack Utilities CF Allows for farmland soil configuration, compressed block creation, and lore additions In Control CF / MR Control mob spawns via verbose JSON configuration JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Multiblocked2 CF / MR Powerful visual custom machine/multiblock mod Neutron Tools CF / MR Configure creative inventory, portal wait time, and hunger speeds One Enough Enchantment CF / MR Dynamically control the weight of enchantments or even completely remove enchantments. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block ServerConfigUpdater CF Keeps DefaultConfigs in sync with worlds’ serverConfigs between modpack updates Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.20.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for NeoForge 1.20.1",
    "pageTitle": "Utility mods for NeoForge 1.20.1",
    "sectionTitle": "",
    "description": "Utility mods for NeoForge 1.20.1",
    "content": "Utility mods for NeoForge 1.20.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources AStages CF / MR Restrict nearly every aspect of gameplay based on progression Game Stages Bad Mobs CF / MR Simple blacklist for mob spawns Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. Iglee's Modpack Utilities CF Allows for farmland soil configuration, compressed block creation, and lore additions In Control CF / MR Control mob spawns via verbose JSON configuration JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Multiblocked2 CF / MR Powerful visual custom machine/multiblock mod Neutron Tools CF / MR Configure creative inventory, portal wait time, and hunger speeds One Enough Enchantment CF / MR Dynamically control the weight of enchantments or even completely remove enchantments. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.20.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Fabric 1.21.1",
    "pageTitle": "Utility mods for Fabric 1.21.1",
    "sectionTitle": "",
    "description": "Utility mods for Fabric 1.21.1",
    "content": "Utility mods for Fabric 1.21.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery CF / MR Create custom machines via JSON Defaulted CF / MR Adds registry to control default item components. FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.21.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Forge 1.21.1",
    "pageTitle": "Utility mods for Forge 1.21.1",
    "sectionTitle": "",
    "description": "Utility mods for Forge 1.21.1",
    "content": "Utility mods for Forge 1.21.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources AStages CF / MR Restrict nearly every aspect of gameplay based on progression Game Stages Bad Mobs CF / MR Simple blacklist for mob spawns Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery CF / MR Create custom machines via JSON Emendatus Enigmatica CF / MR Dynamic material registry system, with world generation, compat, and many other features FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. Iglee's Modpack Utilities CF Allows for farmland soil configuration, compressed block creation, and lore additions In Control CF / MR Control mob spawns via verbose JSON configuration JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Multiblocked2 CF / MR Powerful visual custom machine/multiblock mod Neutron Tools CF / MR Configure creative inventory, portal wait time, and hunger speeds One Enough Enchantment CF / MR Dynamically control the weight of enchantments or even completely remove enchantments. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.21.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for NeoForge 1.21.1",
    "pageTitle": "Utility mods for NeoForge 1.21.1",
    "sectionTitle": "",
    "description": "Utility mods for NeoForge 1.21.1",
    "content": "Utility mods for NeoForge 1.21.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Almost Unified CF / MR Unify all resources AStages CF / MR Restrict nearly every aspect of gameplay based on progression Game Stages Bad Mobs CF / MR Simple blacklist for mob spawns Biome Replacer CF / MR A quick way to get rid of a biome CraftTweaker CF / MR Change recipes, script events, add new commands, and change item properties with ZenScript Custom Machinery CF / MR Create custom machines via JSON Defaulted CF / MR Adds registry to control default item components. Emendatus Enigmatica CF / MR Dynamic material registry system, with world generation, compat, and many other features FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. Iglee's Modpack Utilities CF Allows for farmland soil configuration, compressed block creation, and lore additions In Control CF / MR Control mob spawns via verbose JSON configuration JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs KubeJS CF / MR Edit recipes, add new custom items, script world events through JavaScript Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Modular Machinery Reborn CF / MR Create custom machines via JSON or KubeJS Multiblocked2 CF / MR Powerful visual custom machine/multiblock mod Neutron Tools CF / MR Configure creative inventory, portal wait time, and hunger speeds One Enough Enchantment CF / MR Dynamically control the weight of enchantments or even completely remove enchantments. One Enough Item CF / MR Replace duplicate items with a single designated representative item Open Loader CF / MR Allows data packs and resource packs to be applied globally across all save files in a game instance. Paxi Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/1.21.1/neoforge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Fabric 26.1",
    "pageTitle": "Utility mods for Fabric 26.1",
    "sectionTitle": "",
    "description": "Utility mods for Fabric 26.1",
    "content": "Utility mods for Fabric 26.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Biome Replacer CF / MR A quick way to get rid of a biome Defaulted CF / MR Adds registry to control default item components. FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/26.1/fabric",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for Forge 26.1",
    "pageTitle": "Utility mods for Forge 26.1",
    "sectionTitle": "",
    "description": "Utility mods for Forge 26.1",
    "content": "Utility mods for Forge 26.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Biome Replacer CF / MR A quick way to get rid of a biome FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/26.1/forge",
    "tags": []
  },
  {
    "kind": "page",
    "title": "Utility mods for NeoForge 26.1",
    "pageTitle": "Utility mods for NeoForge 26.1",
    "sectionTitle": "",
    "description": "Utility mods for NeoForge 26.1",
    "content": "Utility mods for NeoForge 26.1 These are mods that are essential for creating, removing, and editing aspects of the game to create custom content and unique experiences for a modpack. Name Links Description Alternatives Biome Replacer CF / MR A quick way to get rid of a biome Defaulted CF / MR Adds registry to control default item components. FancyMenu CF / MR Customize Minecraft menus in an in game GUI editor PackMenu for Neo/Forge Featurify CF / MR A worldgen feature configuration mod that eliminates the need for datapacks. Add, remove, and tweak features in any biome. JSON Things CF A mod that enables players to define blocks and items (and more!) via thingpacks, zip files similar to datapacks and resource packs Lithostitched CF / MR Library mod with new configurability and compatibility enhancements for worldgen Lychee CF / MR Define in world crafting & interactions using JSON recipes, such as item interaction, burning, touching fluid, anvil, crushing, lightning, and exploding. Mob Control CF / MR Rule based mob spawning and AI configuration. Closed Source. Recreative CF / MR Dynamic, data driven Creative Mode tabs via simple JSON configuration Neutron Tools has similar Creative Tab customization features. Reliable Recipes CF / MR Dynamic recipe and tag manipulation via simple JSON configuration Reliable Remover CF / MR Allows for completely removing items via simple JSON configuration. Item Obliterator, though this mod was made to address critical issues of it. Reliable Replacer CF / MR Allows for replacing one block with another, both during worldgen and afterward. Block Swap, though this mod was made to address critical issues of it. One Enough Block Structurify CF / MR GUI based structure configuration \\ A must have mod for nearly any modpack. Only in very rare circumstances should you not use this mod or an equivalent. Prone to issues or conflicts with other mods",
    "url": "/wiki/useful-mods/utility/26.1/neoforge",
    "tags": []
  }
];
