<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { appIcons } from "./notify.svelte";

  let { appId, size = 24 }: { appId: string; size?: number } = $props();
  const icon = $derived(appIcons[appId] ?? null);
</script>

<span class="app" style:width="{size}px" style:height="{size}px" style:--glow={icon?.color ?? "var(--accent-hud)"}>
  {#if icon}
    <img src={icon.src} alt="" width={size} height={size} />
  {:else}
    <span class="bell"><Icon name="bell" size={Math.round(size * 0.6)} /></span>
  {/if}
</span>

<style>
  .app {
    flex: none;
    display: grid;
    place-items: center;
    margin-right: 8px;
  }
  img {
    display: block;
  }
  .bell {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--notch-surface);
    color: var(--accent-hud);
  }
  :global(.glow) img {
    filter: drop-shadow(0 0 2px color-mix(in srgb, var(--glow) 80%, transparent)) drop-shadow(0 0 8px color-mix(in srgb, var(--glow) 55%, transparent));
  }
  :global(.glow) .bell {
    box-shadow:
      0 0 3px color-mix(in srgb, var(--glow) 90%, transparent),
      0 0 12px color-mix(in srgb, var(--glow) 60%, transparent);
  }
</style>
