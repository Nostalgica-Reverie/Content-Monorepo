<script lang="ts">
  // Page imports
  import SidebarPage from "$lib/sidebar/navigation/SidebarPage.svelte";
  import SidebarCategory from "$lib/sidebar/navigation/SidebarCategory.svelte";
  import SidebarHeading from "$lib/sidebar/navigation/SidebarHeading.svelte";

  // Icon imports
  // https://tabler-icons.io
  // import Icon<whatever name> from "~icons/tabler/<icon ID>"
  import IconHome from "~icons/tabler/home";
  import IconHammer from "~icons/tabler/hammer";
  import IconBook2 from "~icons/tabler/book-2";
  import IconInfo from "~icons/tabler/info-circle";
  import IconTrash from "~icons/tabler/trash";
  import IconCubePlus from "~icons/tabler/cube-plus";
  import IconAdjustmentsHorizontal from "~icons/tabler/adjustments-horizontal";
  import IconMeat from "~icons/tabler/meat";
  import IconCubeOff from "~icons/tabler/cube-off";
  import IconMeatOff from "~icons/tabler/meat-off";
  import IconMountain from "~icons/tabler/mountain";
  import IconMountainOff from "~icons/tabler/mountain-off";
</script>

<!-- ADD PAGES AND CATEGORIES BELOW -->

<SidebarPage label="Home" icon={IconHome} page="/" />

<SidebarCategory name="Introductory" icon={IconInfo}>
  <SidebarPage label="Intro to Modpacks" icon={IconBook2} page="/guide/intro/intro-modpack" />
  <SidebarPage label="Minecraft Concepts" icon={IconBook2} page="/guide/intro/minecraft-concepts" />
  <SidebarPage label="Intro to Datapacks" icon={IconBook2} page="/guide/intro/intro-datapack" />
</SidebarCategory>

<SidebarCategory name="Worldgen" icon={IconMountain}>
  <SidebarPage label="Changing Mob Spawns" icon={IconMountain} page="/guide/worldgen/mob-spawns" />
  <SidebarPage label="Adding Features" icon={IconMountain} page="/guide/worldgen/modifying-features/adding-features" />
  <SidebarPage label="Removing Features" icon={IconMountainOff} page="/guide/worldgen/modifying-features/removing-features" />
</SidebarCategory>

<SidebarCategory name="Removals" icon={IconTrash}>
  <SidebarPage label="Removing Items" icon={IconMeatOff} page="/guide/removals/removing-items" />
  <SidebarPage label="Removing Blocks" icon={IconCubeOff} page="/guide/removals/removing-blocks" />
</SidebarCategory>

<SidebarCategory name="Custom Content" icon={IconHammer}>
  <SidebarPage label="Attribute Modification" icon={IconAdjustmentsHorizontal} page="/guide/custom-content/attribute-modification" />
  <SidebarPage label="Adding Items" icon={IconMeat} page="/guide/custom-content/adding-items" />
  <SidebarPage label="Adding Blocks" icon={IconCubePlus} page="/guide/custom-content/adding-blocks" />
</SidebarCategory>

<SidebarCategory name="Advanced" icon={IconInfo}>
  <SidebarPage label="" icon={IconBook2} page="/guide/intro-datapack" />
  <SidebarPage label="" icon={IconBook2} page="/guide/intro-resourcepack" />
</SidebarCategory>
