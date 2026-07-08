<script lang="ts">
  import { base } from '$app/paths';
  import { windowInfo } from "$lib/stores.svelte";

  type Props = {
    icon: any;
    page: string;
    label: string;
  };

  const { icon, page, label }: Props = $props();

  const Icon = $derived(icon);
</script>

<a
  href={`${base}${page}`}
  onclick={() => (windowInfo.width < 768 ? (windowInfo.isNavOpen = !windowInfo.isNavOpen) : null)}
  class="hover:bg-stone-700 hover:text-white hover:font-medium py-1 rounded-lg flex gap-2 pl-1 items-center focus-visible:outline-2 focus-visible:outline-mdw-yellow">
  <Icon />

  {#if windowInfo.isNavOpen}
    <span>{label}</span>
  {/if}
</a>

