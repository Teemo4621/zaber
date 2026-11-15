<script lang="ts">
  import { settings } from "$lib/settings.svelte";
  import { fade } from "svelte/transition";
  import { reduceMotion } from "$lib/theme.svelte";
  import Artwork from "./Artwork.svelte";
  import { currentLine, type LyricLine } from "./lrc";
  import { fetchLyrics, media, mediaUi, positionNow, type NowPlaying } from "./media.svelte";

  let { now, inline = true }: { now: NowPlaying; inline?: boolean } = $props();

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

  const BARS = [0, 1, 2, 3];
  let bars = $state(BARS.map(() => 0.6));
  $effect(() => {
    if (!now.playing || reduceMotion()) return;
    const start = performance.now();
    const timer = setInterval(() => {
      const t = (performance.now() - start) / 1000;
      bars = BARS.map((i) => 0.25 + 0.75 * Math.abs(Math.sin(t * (3.1 + i * 0.7) + i * 1.3)));
    }, 1000 / 12);
    return () => clearInterval(timer);
  });
</script>

<div class="live" class:on={line && !inline}>
<div class="pill">
  <Artwork src={now.artwork} size={22} />
  {#if line && inline}
    <span class="slot">
      {#key line}
        <p class="line" style:--wave={artColor ?? "var(--accent-media)"} in:fade={{ duration: reduceMotion() ? 0 : 220 }}>{line}</p>
      {/key}
    </span>
  {/if}
  <div class="wave" class:paused={!now.playing} aria-hidden="true" style:--wave={artColor ?? "var(--accent-media)"}>
    {#each BARS as i (i)}
      <span style:transform="scaleY({bars[i].toFixed(2)})"></span>
    {/each}
  </div>
</div>
{#if line && !inline}
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
    grid-template-rows: 100%;
    align-content: start;
  }
  .live.on {
    grid-template-rows: calc(100% - 24px) 24px;
  }
  .slot {
    flex: 1;
    min-width: 0;
    display: grid;
  }
  .slot .line {
    grid-area: 1 / 1;
    padding: 0 10px;
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
  }
  :global(.glow) .wave:not(.paused) span {
    box-shadow: 0 0 3px color-mix(in srgb, var(--wave) 90%, transparent), 0 0 12px color-mix(in srgb, var(--wave) 60%, transparent);
  }
</style>
