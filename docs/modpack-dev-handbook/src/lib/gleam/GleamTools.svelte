<script lang="ts">
  import { compareVersions, slugifyPackName } from "./runtime/docsDemo";

  let versionA = $state("1.20.1");
  let versionB = $state("1.9");
  let packName = $state("My Cool Pack!");

  const verdict = $derived(compareVersions(versionA, versionB));
  const slug = $derived(slugifyPackName(packName));
</script>

<div class="gleam-demo not-prose">
  <p class="gleam-demo-title">Powered by Gleam</p>
  <div class="gleam-demo-row">
    <label>Compare Minecraft versions:
      <input bind:value={versionA} placeholder="1.20.1" />
      vs
      <input bind:value={versionB} placeholder="1.9" />
    </label>
    <p class="gleam-demo-out">{verdict}</p>
  </div>
  <div class="gleam-demo-row">
    <label>Slugify a pack name:
      <input bind:value={packName} placeholder="My Cool Pack!" />
    </label>
    <p class="gleam-demo-out"><code>{slug}</code></p>
  </div>
</div>

<style scoped>
  .gleam-demo {
    border: 1px solid rgb(68 64 60);
    border-radius: 0.75rem;
    padding: 1rem 1.25rem;
    margin: 1rem 0;
    background: rgb(28 25 23 / 0.6);
  }
  .gleam-demo-title {
    font-weight: 600;
    color: rgb(251 191 36);
    margin: 0 0 0.5rem;
  }
  .gleam-demo-row {
    margin: 0.75rem 0;
  }
  .gleam-demo input {
    border: 1px solid rgb(68 64 60);
    border-radius: 0.4rem;
    padding: 0.15rem 0.5rem;
    margin: 0 0.4rem;
    background: rgb(12 10 9);
    color: rgb(231 229 228);
    width: 7.5rem;
  }
  .gleam-demo-out {
    margin: 0.35rem 0 0;
    color: rgb(168 162 158);
  }
</style>
