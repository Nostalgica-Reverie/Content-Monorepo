<script lang="ts">
  import type { Snippet } from "svelte";

  import IconThing from "~icons/tabler/CaretDown";
  interface Props {
    label?: string;
    children?: Snippet;
  }

  let { label = "No Label!", children }: Props = $props();

  let open = $state(true);
</script>

<ul class="not-prose flex flex-col h-auto">
  <button class="flex items-center h-7 w-max" onclick={() => (open = !open)}>
    <IconThing class="{open ? 'rotate-0' : '-rotate-90'} motion-safe:transition-all size-4" />
    <p class="w-max">{label}</p>
  </button>
  {#if open}
    {#if children}
      {@render children?.()}
    {:else}
      No Content!
    {/if}
  {/if}
</ul>

