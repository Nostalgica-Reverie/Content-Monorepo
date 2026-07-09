<script lang="ts">
  type Props = {
    children: any;
    color?: string;
  };
  const { children, color = "FFFF00" }: Props = $props();
</script>

<span style="--background-color: {color}BB;" class="dph-highlight">
  {@render children()}
</span>

<style>
  span.dph-highlight {
    background-color: var(--background-color);
    padding: 0.2rem;
  }
</style>

