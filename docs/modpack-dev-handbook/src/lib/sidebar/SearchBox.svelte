<script lang="ts">
  import IconSearch from "~icons/tabler/search";
  import type { SvelteComponent } from "svelte";
  import type { SearchResult } from "$lib/search";
  import SidebarSearchDialog from "./SidebarSearchDialog.svelte";

  let dialog: SvelteComponent;

  let results: SearchResult[] = $state([]);

  let { keyActivated }: { keyActivated?: boolean } = $props();
</script>

<button
  aria-label="Open Search Modal"
  class="bg-black/45 px-3 py-1 rounded-lg flex justify-between items-center mt-1 mb-2 w-full focus-visible:outline-2 focus-visible:outline-mdw-yellow cursor-text"
  onclick={async () => await dialog.showModal()}>
  <div class="flex items-center gap-2">
    <IconSearch />
    <span class="py-1 text-stone-500">Search...</span>
  </div>
  <p class="text-stone-400 text-xs md:block hidden font-mono">
    <span class="bg-stone-800 px-1.5 py-1 rounded-sm">Ctrl</span>
    <span class="bg-stone-800 px-1.5 py-1 rounded-sm">K</span>
  </p>
</button>

<SidebarSearchDialog {keyActivated} bind:results bind:this={dialog} />