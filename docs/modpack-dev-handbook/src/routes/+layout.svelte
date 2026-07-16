<script lang="ts">
  import "../styles/app.css";
  import "../styles/fonts.css";

  import Sidebar from "$lib/sidebar/Sidebar.svelte";
  import Navbar from "../lib/Topbar.svelte";

  import { latestMCData, windowInfo } from "$lib/stores.svelte";
  import type { Snippet } from "svelte";
  import { innerWidth } from "svelte/reactivity/window";
  import { codeCopy } from "$lib/actions/codeCopy";
  import type { PageData } from "./$types";

  interface Props {
    children: Snippet;
    data: PageData;
  }

  let props: Props = $props();

  $effect(() => {
    latestMCData.packFormat = props.data.packFormat || 0;
    latestMCData.gameVersion = props.data.gameVersion || "1.0";
    windowInfo.width = innerWidth.current || 1920;
    windowInfo.isNavOpen = (innerWidth.current || 1920) >= 768;
  });
</script>

<div class="font-lexend h-full min-h-dvh flex flex-col text-stone-200">
  <Navbar />
  <div class="flex w-full h-[93%]">
    <Sidebar />
    <div use:codeCopy id="content" class="py-12 w-full min-h-dvh text-stone-200 bg-stone-900">
      {@render props.children()}
    </div>
  </div>
</div>
