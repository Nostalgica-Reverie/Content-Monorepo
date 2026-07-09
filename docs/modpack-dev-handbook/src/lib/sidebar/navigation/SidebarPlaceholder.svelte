<script lang="ts">
  import { windowInfo } from "$lib/stores.svelte";

  type Props = {
    label: string;
    icon: any;
  };

  const { label, icon }: Props = $props();

  const Icon = $derived(icon);
</script>

<div class="cursor-not-allowed py-1 rounded-lg flex gap-2 pl-1 items-center text-stone-500">
  <Icon />

  {#if windowInfo.isNavOpen}
    <span>{label}</span>
  {/if}
</div>

