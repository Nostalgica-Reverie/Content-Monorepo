<script lang="ts">
  import { windowInfo } from "$lib/stores.svelte";
  import { ensureSearchReady, search, type SearchResult } from "../search";
  import { base } from "$app/paths";

  type Props = {
    results: SearchResult[];
    keyActivated?: boolean;
  };

  let { results = $bindable([]), keyActivated }: Props = $props();
  let dialog: HTMLDialogElement;

  let searchTerm = $state("");
  let searchState: "idle" | "loading" | "ready" | "error" = $state("idle");

  export async function showModalWithEvent(e: KeyboardEvent) {
    if (!keyActivated) {
      return;
    }
    e.preventDefault();
    await showModal();
  }

  export async function showModal() {
    dialog.showModal();
    if (searchState === "idle") {
      searchState = "loading";
      try {
        await ensureSearchReady();
        searchState = "ready";
      } catch (error) {
        console.error("Failed to load search data", error);
        searchState = "error";
      }
    }
  }

  $effect(() => {
    if (searchState === "ready") {
      results = search(searchTerm);
      return;
    }

    results = [];
  });
</script>

<svelte:window
  onkeydown={e => (e.key == "k" && e.ctrlKey ? showModalWithEvent(e) : null)}
  onclick={e => {
    e.target === dialog ? dialog.close() : null;
  }} />

<dialog
  bind:this={dialog}
  class="w-[90%] md:w-3/4 lg:w-1/2 xl:w-1/3 bg-transparent backdrop:bg-black/50 backdrop:backdrop-blur-sm mx-auto top-1/3 not-prose">
  <div class="bg-stone-800 w-full p-4 gap-1 rounded-md">
    <input
      class="bg-stone-900 w-full rounded-sm focus:outline-0 text-white p-2 placeholder:text-stone-500 disabled:cursor-not-allowed disabled:bg-stone-900/50"
      disabled={searchState === "loading"}
      id="search-box"
      autocomplete="off"
      spellcheck="false"
      type="search"
      placeholder="Search for a page..."
      bind:value={searchTerm} />
    <div class="overflow-y-auto max-h-[50vh]">
      {#each results as result}
        <a
          onclick={() => {
            dialog.close();
            if (windowInfo.width < 640) {
              windowInfo.isNavOpen = false;
            }
          }}
          href={`${base}${result.url}`}>
          <div class="p-2 my-2 rounded-sm hover:bg-black/20 motion-safe:transition-all">
            <p class="text-stone-200 text-lg">
              {@html result.title}
              <span class="text-stone-400 text-xs">{result.url}</span>
            </p>
            <p class="text-stone-400 line-clamp-2">{@html result.content}</p>
          </div>
        </a>
      {/each}
    </div>
    <p class="text-stone-400 mt-2">
      {searchTerm === ""
        ? ""
        : searchState === "loading"
          ? "Loading search index..."
          : searchState === "error"
            ? "Search is unavailable"
            : results.length === 0
              ? "No results"
              : results.length + " result(s) found"}
    </p>

    <button class="bg-stone-700 w-full rounded-sm p-2 mt-2 text-white cursor-pointer" onclick={() => dialog.close()}
      >Close</button>
  </div>
</dialog>
