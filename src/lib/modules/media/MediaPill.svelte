<script lang="ts">
  import { settings } from "$lib/settings.svelte";
  import { fade } from "svelte/transition";
  import { reduceMotion } from "$lib/theme.svelte";
  import Artwork from "./Artwork.svelte";
  import { currentLine, type LyricLine } from "./lrc";
  import { fetchLyrics, media, mediaUi, positionNow, type NowPlaying } from "./media.svelte";

  let { now }: { now: NowPlaying } = $props();

  let lines = $state<LyricLine[] | null>(null);
  $effect(() => {
    lines = null;
    if (!mediaUi.lyrics) return;
    const track = now.title + now.artist;
    fetchLyrics(now).then((l) => {
      if (track === now.title + now.artist && l?.length && l[0].ms >= 0) lines = l;
    });
  });
  let index = $state(-1);
  $effect(() => {
    const l = lines;
    const playing = now.playing;
    void media.receivedAt;
    if (!l) return void (index = -1);
    let timer: ReturnType<typeof setTimeout> | undefined;
    const tick = () => {
      const at = positionNow();
      index = currentLine(l, at);
      const next = l[index + 1];
      if (playing && next) timer = setTimeout(tick, Math.max(50, next.ms - at + 30));
    };
    tick();
    return () => clearTimeout(timer);
  });
  const line = $derived(lines && index >= 0 ? lines[index].text || "♪" : null);
  $effect(() => {
    mediaUi.line = line;
    return () => (mediaUi.line = null);
  });

  const artColor = $derived(settings.value.followArtwork ? mediaUi.artColor : null);
</script>

<div class="live" class:on={line}>
<div class="pill">
  <Artwork src={now.artwork} size={22} />
  <div class="wave" class:paused={!now.playing} aria-hidden="true" style:--wave={artColor ?? "var(--accent-media)"}>
    {#each [0, 1, 2, 3] as i (i)}
      <span style:animation-delay="{i * -0.27}s"></span>
    {/each}
  </div>
</div>
{#if line}
  {#key line}
    <p class="line" style:--wave={artColor ?? "var(--accent-media)"} in:fade={{ duration: reduceMotion() ? 0 : 220 }}>{line}</p>
  {/key}
{/if}
</div>

<style>
  .live {
    width: 100%;
    height: 100%;
    display: grid;
    grid-template-rows: 32px;
    align-content: start;
  }
  .live.on {
    grid-template-rows: 32px 1fr;
  }
  .line {
    margin: 0;
    padding: 0 14px 6px;
    font-size: 13px;
    font-weight: 600;
    text-align: center;
    color: var(--wave);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pill {
    width: 100%;
    padding: 0 12px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .wave {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 14px;
  }
  .wave span {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: var(--wave);
    transform-origin: center;
    animation: wave 0.9s ease-in-out infinite alternate;
  }
  :global(.glow) .wave:not(.paused) span {
    box-shadow: 0 0 3px color-mix(in srgb, var(--wave) 90%, transparent), 0 0 12px color-mix(in srgb, var(--wave) 60%, transparent);
  }
  .wave.paused span {
    animation-play-state: paused;
  }
  @keyframes wave {
    from {
      transform: scaleY(0.25);
    }
    to {
      transform: scaleY(1);
    }
  }
  :global(.reduce-motion) .wave span {
    animation: none;
    transform: scaleY(0.6);
  }
</style>
