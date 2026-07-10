<script lang="ts">
  import IconDiscord from "~icons/tabler/brand-discord";
  import IconShare from "~icons/tabler/share";
  import IconMenu from "~icons/tabler/menu-2";
  import IconEdit from "~icons/tabler/pencil";

  import { windowInfo } from "$lib/stores.svelte";
  import { page } from "$app/state";
  import { base } from "$app/paths";
  import { findDocByUrl } from "$lib/generated/docs";
  import { handbookSourceUrl, siteConfig } from "$lib/site";

  let shareText = $state("Share");

  function copyUrl() {
    if (navigator.maxTouchPoints > 0 && navigator.share) {
      navigator.share({ url: window.location.href });
    } else {
      navigator.clipboard.writeText(window.location.href);
      shareText = "Copied!";
      setTimeout(() => {
        shareText = "Share";
      }, 2000);
    }
  }

  let lastFewInputs: string[] = [];
  let logoFlipped = $state(false);
  let logoBonked = $state(false);

  const currentDoc = $derived(findDocByUrl(page.url.pathname.replace(base, "") || "/"));
  const editHref = $derived(currentDoc ? handbookSourceUrl(currentDoc.sourcePath) : siteConfig.handbook.sourceBrowseRoot);

  export async function handleKeyInput(
    e: KeyboardEvent & {
      currentTarget: EventTarget & Window;
    }
  ) {
    const doc = e.currentTarget.document;
    const notAnInput = !(doc.activeElement instanceof HTMLInputElement) && !(doc.activeElement instanceof HTMLTextAreaElement);
    if (!notAnInput) return;
    lastFewInputs.push(e.key);
    if (lastFewInputs.length > 8) lastFewInputs.shift();
    if (lastFewInputs.join("").includes("dataflip")) logoFlipped = !logoFlipped;
    if (lastFewInputs.join("").includes("databonk")) logoBonked = !logoBonked;
  }
</script>

<svelte:window onkeydown={e => handleKeyInput(e)} />

<div class="bg-stone-800 flex w-full items-center justify-between p-2 h-12 sticky top-0 border-b border-stone-700 z-20">
  <a class="absolute -translate-y-30 -translate-x-1/2 left-1/2 focus-visible:outline-2 outline-blue-500 focus-visible:translate-y-0" href="#nav_side">Go To Nav</a>
  <a class="absolute -translate-y-30 -translate-x-1/2 left-1/2 focus-visible:outline-2 outline-blue-500 focus-visible:translate-y-0" href="#main_content">Go To Content</a>
  <div class="flex items-center grow">
    <button class="px-2 pr-3 sm:hidden focus-visible:outline-2 focus-visible:outline-mdw-yellow" aria-label="{windowInfo.isNavOpen ? 'Collapse' : 'Expand'} Sidebar" onclick={() => (windowInfo.isNavOpen = !windowInfo.isNavOpen)}><IconMenu /></button>
    <a class="flex items-center hover:text-white p-1 focus-visible:outline-2 focus-visible:outline-mdw-yellow" href={`${base}`}>
      <img alt="Modpack Dev Handbook Logo" src={`${base}/logos/dph.svg`} class="h-8 mr-2 {logoFlipped ? 'rotate-180' : ''} {logoBonked ? 'scale-y-50' : ''} transition-transform" width="32" height="32" />
      <h1 class="font-bold hidden text-lg lg:text-xl sm:block">{siteConfig.handbook.title}</h1>
    </a>
  </div>
  <div class="flex items-center gap-2">
    <a href={editHref} class="p-2 rounded-lg py-1 flex items-center gap-2 hover:bg-stone-700 hover:text-white hover:font-medium aspect-square sm:aspect-auto focus-visible:outline-2 focus-visible:outline-mdw-yellow" aria-label="Edit"><IconEdit /><span class="hidden sm:block">Source</span></a>
    <button class="p-2 rounded-lg py-1 flex items-center gap-2 hover:bg-stone-700 hover:text-white hover:font-medium aspect-square sm:aspect-auto focus-visible:outline-2 focus-visible:outline-mdw-yellow cursor-pointer" aria-label="Copy URL" onclick={copyUrl}><IconShare /><span class="hidden sm:block">{shareText}</span></button>
    <a href={siteConfig.handbook.discordUrl} class="p-2 rounded-lg py-1 flex items-center gap-2 hover:bg-stone-700 hover:text-white hover:font-medium aspect-square sm:aspect-auto focus-visible:outline-2 focus-visible:outline-mdw-yellow" aria-label="Discord"><IconDiscord /><span class="hidden sm:block">Discord</span></a>
  </div>
</div>