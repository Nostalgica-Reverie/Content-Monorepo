<script lang="ts">
  import IconDiscord from "~icons/tabler/brand-discord";
  import { siteConfig } from "$lib/site";
</script>

<div class="bg-stone-950 w-full min-w-[25%] border-l-4 border-mdw-yellow p-3 pr-5 my-2 mt-10">
  <div class="flex items-center gap-3">
    <img alt="Reverie Projects Discord" src="./logos/dph.webp" class="not-prose h-16 m-2 hidden md:block" />
    <div class="flex flex-col not-prose">
      <div class="flex text-sm gap-2 items-center text-amber-400 mb-1">
        <IconDiscord />
        <span>JOIN OUR DISCORD SERVER</span>
      </div>
      <p>
        If you need any more help with modpack development, don't hesitate to join the
        <span class="text-amber-400">Reverie Projects</span> Discord server!
        <br />
        Join us now!
        <a href={siteConfig.handbook.discordUrl} class="underline text-orange-400">{siteConfig.handbook.discordUrl}</a>
      </p>
    </div>
  </div>
</div>


