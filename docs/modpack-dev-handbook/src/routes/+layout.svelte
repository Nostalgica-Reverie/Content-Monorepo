<script lang="ts">
  import "../styles/app.css";
  import "../styles/fonts.css";

  import Sidebar from "$lib/sidebar/Sidebar.svelte";
  import { siteConfig } from "$lib/site";
  import Navbar from "../lib/Topbar.svelte";

  import { onMount } from "svelte";
  import { latestMCData, windowInfo } from "$lib/stores.svelte";
  import type { Snippet } from "svelte";
  import { innerWidth } from "svelte/reactivity/window";
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

  $effect(() => {
    console.log(`%c${siteConfig.handbook.title}`, "color: oklch(69.27% 0.2042 40.82); font-size: 24pt; font-weight: 600;");
    console.log("Docs are sourced from the handbook plus the packwand and packwiz trees in this repository.");
  });

  onMount(() => {
    document.querySelectorAll("pre").forEach((block) => {
      const container = document.createElement("div");
      container.className = "code-wrapper";
      const copyButton = document.createElement("button");
      copyButton.innerText = "Copy";
      copyButton.className = "copy-button";
      copyButton.addEventListener("click", () => {
        const text = block.innerText;
        navigator.clipboard.writeText(text).then(() => {
          copyButton.innerText = "Copied!";
          setTimeout(() => (copyButton.innerText = "Copy"), 2000);
        });
      });
      if (block.parentNode) {
        block.parentNode.insertBefore(container, block);
        container.appendChild(copyButton);
        container.appendChild(block);
      }
    });
  });
</script>

<div class="font-lexend h-full min-h-dvh flex flex-col text-stone-200">
  <Navbar />
  <div class="flex w-full h-[93%]">
    <Sidebar />
    <div id="content" class="py-12 w-full min-h-dvh text-stone-200 bg-stone-900">
      {@render props.children()}
    </div>
  </div>
</div>

