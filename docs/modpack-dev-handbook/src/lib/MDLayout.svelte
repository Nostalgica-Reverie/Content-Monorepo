<script lang="ts">
  import { page } from "$app/state";
  import Seo from "sk-seo";
  import type { Snippet } from "svelte";
  import { siteConfig } from "$lib/site";
  import Version from "./reusables/Version.svelte";

  type Props = {
    title: string;
    description: string;
    tags: string;
    version: string;
    children: Snippet;
  };

  let props: Props = $props();
  const tagsArr = $derived(props.tags.split(",").map((el: string) => el.trim()).filter(Boolean));
</script>

<Seo
  title="{props.title ? props.title + ' - ' : ''}{siteConfig.handbook.title}"
  description={props.description}
  author={siteConfig.handbook.author}
  siteName={siteConfig.handbook.title}
  keywords={siteConfig.handbook.keywords}
  name={siteConfig.handbook.title}
  schemaOrg={true}
  canonical={`${siteConfig.handbook.siteUrl}${page.url.pathname}`}
  socials={[siteConfig.handbook.discordUrl]} />

<main class="md px-4 md:px-8 lg:px-16 prose-headings:text-stone-200" id="main_content">
  {#if props.version}
    <Version version={props.version} />
  {/if}
  {@render props.children()}
  {#if props.tags}
    <div class="bg-stone-950/40 p-2 rounded-lg flex items-center flex-wrap gap-3 my-10">
      <span class="uppercase text-sm text-zinc-500">Tags:</span>
      {#each tagsArr as tag}
        <span class="border border-amber-500 px-1 text-yellow-500 rounded-lg uppercase text-sm">{tag}</span>
      {/each}
    </div>
  {/if}
</main>