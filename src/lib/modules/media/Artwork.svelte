<script lang="ts">
  import { mediaUi } from "./media.svelte";

  let { src, size }: { src: string | null; size: number } = $props();
</script>

<div class="art" style:width="{size}px" style:height="{size}px" style:--art-glow={mediaUi.artColor ?? "var(--accent-media)"}>
  {#if src}
    <img {src} alt="" />
  {:else}
    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 4v11.5a3.5 3.5 0 1 0 2 3.2V8h7V4z" /></svg>
  {/if}
</div>

<style>
  .art {
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 22%;
    overflow: hidden;
    background: var(--notch-surface);
    color: var(--notch-fg-dim);
  }
  :global(.glow) .art {
    box-shadow: 0 0 18px 1px color-mix(in srgb, var(--art-glow) 70%, transparent);
    transition: box-shadow 400ms ease-out;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  svg {
    width: 55%;
    height: 55%;
    fill: currentColor;
  }
</style>
