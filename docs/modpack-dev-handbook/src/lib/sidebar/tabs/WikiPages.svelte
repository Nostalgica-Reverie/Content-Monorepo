<script lang="ts">
  // Page imports
  import SidebarPage from "$lib/sidebar/navigation/SidebarPage.svelte";
  import SidebarCategory from "$lib/sidebar/navigation/SidebarCategory.svelte";
  import SidebarHeading from "$lib/sidebar/navigation/SidebarHeading.svelte";
  import SidebarPlaceholder from "$lib/sidebar/navigation/SidebarPlaceholder.svelte";

  // Icon imports
  // https://tabler-icons.io
  // import Icon<whatever name> from "~icons/tabler/<icon ID>"

  import IconHome from "~icons/tabler/home";
  import IconZip from "~icons/tabler/file-zip";
  import IconPennant from "~icons/tabler/pennant";
  import IconUsers from "~icons/tabler/users";
  import IconGavel from "~icons/tabler/gavel";
  import IconHomeStar from "~icons/tabler/home-star";
  import IconRegex from "~icons/tabler/regex";
  import IconTagFilled from "~icons/tabler/tag-filled";
  import IconInfo from "~icons/tabler/info-circle";
  import IconMicroscope from "~icons/tabler/microscope";
  import IconShoppingCart from "~icons/tabler/shopping-cart";
  import IconLoader3 from "~icons/tabler/loader-3";
  import IconPencil from "~icons/tabler/pencil";
  import IconConcepts from "~icons/tabler/blocks";
  import IconGitMerge from "~icons/tabler/git-merge";
  import IconVocabulary from "~icons/tabler/vocabulary";
  import IconEvergreen from "~icons/tabler/christmas-tree";
</script>

<!-- ADD PAGES AND CATEGORIES BELOW -->

<SidebarPage label="Home" icon={IconHome} page="/" />

<SidebarCategory name="Info" icon={IconInfo}>
  <SidebarPage label="Free Multiplayer" icon={IconUsers} page="/wiki/info/free-multiplayer" />
  <SidebarPage label="Licenses" icon={IconGavel} page="/wiki/info/licenses" />
  <SidebarPage label="Data Loading Conditions" icon={IconLoader3} page="/wiki/info/data-loading-conditions" />
  <SidebarPage label="Regex" icon={IconRegex} page="/wiki/info/regex" />
  <SidebarPage label="Useful Tags" icon={IconTagFilled} page="/wiki/info/useful-tags" />
  <SidebarPage label="Version Control" icon={IconGitMerge} page="/wiki/info/version-control-tools" />
</SidebarCategory>

<SidebarCategory name="Useful Terms" icon={IconConcepts}>
  <SidebarPage label="Closed Source" icon={IconVocabulary} page="/wiki/info/useful-terms/closed-source" />
  <SidebarPage label="Open Source" icon={IconVocabulary} page="/wiki/info/useful-terms/open-source" />
  <SidebarPage label="Visible Source" icon={IconVocabulary} page="/wiki/info/useful-terms/visible-source" />
  <SidebarPage label="Kitchen Sink" icon={IconVocabulary} page="/wiki/info/useful-terms/kitchen-sink" />
  <SidebarPage label="Tags" icon={IconVocabulary} page="/wiki/info/useful-terms/tags" />
  <SidebarPage label="Vanilla+" icon={IconVocabulary} page="/wiki/info/useful-terms/vanilla+" />
  <SidebarPage label="worldgen-feature" icon={IconVocabulary} page="/wiki/info/useful-terms/worldgen-feature" />
  <SidebarPage label="worldgen-structure" icon={IconVocabulary} page="/wiki/info/useful-terms/worldgen-structure" />
</SidebarCategory>

<SidebarCategory name="Planning" icon={IconPencil}>
  <SidebarPage label="Ideation" icon={IconPennant} page="/wiki/planning/ideation" />
  <SidebarPage label="Mod Selection" icon={IconShoppingCart} page="/wiki/planning/mod-selection" />
  <SidebarPage label="Scope" icon={IconMicroscope} page="/wiki/planning/scope" />
</SidebarCategory>

<SidebarCategory name="Modpack Management" icon={IconZip}>
  <SidebarPage label="Project Management" icon={IconPennant} page="/wiki/modpack-management/project-management" />
  <SidebarPage label="CurseForge" icon={IconPennant} page="/wiki/modpack-management/curseforge" />
  <SidebarPage label="Modrinth" icon={IconPennant} page="/wiki/modpack-management/modrinth" />
  <SidebarPage label="Marketing" icon={IconPennant} page="/wiki/modpack-management/marketing" />
</SidebarCategory>

<SidebarPage label="Useful Mods" icon={IconHomeStar} page="/wiki/useful-mods" />
<SidebarPage label="Evergreen Resources" icon={IconEvergreen} page="/wiki/evergreen" />

