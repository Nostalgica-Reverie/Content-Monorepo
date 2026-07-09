<script lang="ts">
  import SearchBox from "./SearchBox.svelte";
  import { navSections } from "$lib/generated/docs";
  import { latestMCData, windowInfo } from "$lib/stores.svelte";
  import SidebarTree from "./SidebarTree.svelte";
  import IconCollapse from "~icons/tabler/chevron-left";

  export async function handleKeyInput(
    e: KeyboardEvent & {
      currentTarget: EventTarget & Window;
    }
  ) {
    const doc = e.currentTarget.document;
    const notAnInput =
      !(doc.activeElement instanceof HTMLInputElement) && !(doc.activeElement instanceof HTMLTextAreaElement);
    if (e.key == "ArrowLeft" && windowInfo.isNavOpen && notAnInput) {
      windowInfo.isNavOpen = false;
    }

    if (e.key == "ArrowRight" && !windowInfo.isNavOpen && notAnInput) {
      windowInfo.isNavOpen = true;
    }
  }
</script>

<svelte:window on:keydown={handleKeyInput} />

<aside class="{windowInfo.isNavOpen ? 'fixed w-full sm:w-80' : 'w-fit hidden sm:flex'} flex flex-col bg-stone-800 items-center h-[calc(100dvh-3rem)] sm:sticky top-12 z-50 border-r border-stone-700">
  <div class="flex flex-col p-2 pt-1 grow overflow-y-auto w-full" id="nav_side">
    {#if windowInfo.isNavOpen}
      <SearchBox keyActivated />
    {/if}
    <div class="flex flex-col gap-3 h-full">
      {#each navSections as section}
        <section class="flex flex-col gap-1">
          {#if windowInfo.isNavOpen}
            <h2 class="px-1 pt-2 text-xs uppercase tracking-[0.2em] text-stone-500">{section.title}</h2>
          {/if}
          {#each section.children as node}
            <SidebarTree {node} />
          {/each}
        </section>
      {/each}
    </div>
  </div>
  <div class="flex text-sm text-stone-600 p-3 items-center w-full">
    {#if windowInfo.isNavOpen}
      <span class="grow flex flex-col">pack_format: {latestMCData.packFormat} ({latestMCData.gameVersion})</span>
    {/if}
    <button aria-label="{windowInfo.isNavOpen ? 'Collapse' : 'Expand'} Sidebar" class="hidden sm:block rounded-lg cursor-pointer text-stone-200 text-lg hover:bg-stone-700 hover:text-white motion-safe:transition-all focus-visible:outline-2 focus-visible:outline-mdw-yellow {windowInfo.isNavOpen ? 'rotate-0' : 'rotate-180'}" onclick={() => (windowInfo.isNavOpen = !windowInfo.isNavOpen)}>
      <IconCollapse />
    </button>
  </div>
  {#if windowInfo.isNavOpen}
    <span class="text-xs px-3 pb-3 text-stone-600">NOT AFFILIATED WITH OR ENDORSED BY MOJANG STUDIOS</span>
  {/if}
</aside>

