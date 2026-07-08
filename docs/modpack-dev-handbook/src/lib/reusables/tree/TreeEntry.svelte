<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<!-- add border-l-2 for line border -->

<li class="w-2 ml-1.5 flex flex-col">
  <div class="w-auto pl-3">{@render children()}</div>
  <!-- <div class="h-0.5 w-2 bg-white relative"></div> -->
</li>

