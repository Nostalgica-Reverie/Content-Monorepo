<script lang="ts">
  import { base } from "$app/paths";
  import type { NavNode } from "$lib/generated/docs";
  import { windowInfo } from "$lib/stores.svelte";
  import IconChevron from "~icons/tabler/chevron-right";
  import IconFile from "~icons/tabler/file-text";
  import IconFolder from "~icons/tabler/folder";
  import SidebarTreeNode from "./SidebarTree.svelte";

  type Props = {
    node: NavNode;
    depth?: number;
  };

  let { node, depth = 0 }: Props = $props();
  const hasChildren = $derived(node.children.length > 0);

  function maybeCloseNav() {
    if (windowInfo.width < 768) {
      windowInfo.isNavOpen = false;
    }
  }
</script>

{#if hasChildren}
  <details class="w-full group marker:hidden" open={depth < 1}>
    <summary class="rounded-lg cursor-pointer p-1 flex gap-2 items-center text-left hover:bg-stone-700 hover:text-white hover:font-medium marker:hidden focus-visible:outline-2 focus-visible:outline-mdw-yellow">
      <IconFolder class="shrink-0" />
      {#if windowInfo.isNavOpen}
        <span class="grow">{node.title}</span>
        <IconChevron class="motion-safe:transition-all group-open:rotate-90 rotate-0 select-none" />
      {/if}
    </summary>
    {#if windowInfo.isNavOpen}
      <div class="flex flex-col ml-4 pb-2 gap-0.5">
        {#if node.url}
          <a href={`${base}${node.url}`} onclick={maybeCloseNav} class="hover:bg-stone-700 hover:text-white hover:font-medium py-1 rounded-lg flex gap-2 pl-1 items-center focus-visible:outline-2 focus-visible:outline-mdw-yellow text-stone-300">
            <IconFile class="shrink-0" />
            <span>Overview</span>
          </a>
        {/if}
        {#each node.children as child}
          <SidebarTreeNode node={child} depth={depth + 1} />
        {/each}
      </div>
    {/if}
  </details>
{:else if node.url}
  <a href={`${base}${node.url}`} onclick={maybeCloseNav} class="hover:bg-stone-700 hover:text-white hover:font-medium py-1 rounded-lg flex gap-2 pl-1 items-center focus-visible:outline-2 focus-visible:outline-mdw-yellow">
    <IconFile class="shrink-0" />
    {#if windowInfo.isNavOpen}
      <span>{node.title}</span>
    {/if}
  </a>
{/if}
