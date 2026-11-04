<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { t } from "$lib/i18n.svelte";
  import { settings } from "$lib/settings.svelte";
  import { onDestroy } from "svelte";
  import Artwork from "./Artwork.svelte";
  import LyricsView from "./LyricsView.svelte";
  import { control, formatTime, mediaUi, positionNow, seek, type NowPlaying } from "./media.svelte";

  let { now }: { now: NowPlaying } = $props();

  let clock = $state(Date.now());
  const tick = setInterval(() => now.playing && (clock = Date.now()), 1000);
  onDestroy(() => clearInterval(tick));
  const position = $derived(positionNow(clock));
  const progress = $derived(now.durationMs > 0 ? position / now.durationMs : 0);
  const artColor = $derived(settings.value.followArtwork ? mediaUi.artColor : null);
  const source = $derived(now.sourceApp.split(/[\\/!]/).pop()?.replace(/\.exe$/i, "") ?? "");

  function seekTo(e: MouseEvent) {
    if (now.durationMs <= 0) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    seek(Math.round(((e.clientX - r.left) / r.width) * now.durationMs));
  }
</script>

<div class="media" class:split={mediaUi.lyrics} style:--accent-media={artColor}>
  <div class="player">
    <div class="track">
      <Artwork src={now.artwork} size={mediaUi.lyrics ? 48 : 56} />
      <div class="meta">
        <strong>{now.title}</strong>
        <span>{now.artist || now.album}{source ? ` · ${source}` : ""}{now.playing ? "" : ` · ${t("media.paused")}`}</span>
      </div>
      {#if settings.value.lyrics && !mediaUi.lyrics}
        <button class="ctl dim" aria-label={t("media.showLyrics")} onclick={() => (mediaUi.lyrics = true)}>
          <Icon name="lyrics" />
        </button>
      {/if}
    </div>
    <div class="time">
      <span>{formatTime(position)}</span>
      <button class="bar" aria-label={formatTime(position)} onclick={seekTo} disabled={now.durationMs <= 0}>
        <span class="fill" class:paused={!now.playing} style:transform="scaleX({progress})"></span>
      </button>
      <span>{now.durationMs > 0 ? formatTime(now.durationMs) : ""}</span>
    </div>
    <div class="controls">
      <button class="ctl back" aria-label={t("media.previous")} onclick={() => control("previous")}><Icon name="prev" size={20} filled /></button>
      <button class="ctl" aria-label={now.playing ? t("media.pause") : t("media.play")} onclick={() => control("play_pause")}>
        <Icon name={now.playing ? "pause" : "play"} size={20} filled />
      </button>
      <button class="ctl fwd" aria-label={t("media.next")} onclick={() => control("next")}><Icon name="next" size={20} filled /></button>
      {#if mediaUi.lyrics}
        <button class="ctl on" aria-label={t("media.hideLyrics")} onclick={() => (mediaUi.lyrics = false)}><Icon name="lyrics" /></button>
      {/if}
    </div>
  </div>
  {#if mediaUi.lyrics}
    <LyricsView {now} />
  {/if}
</div>

<style>
  .media.split {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px;
    height: 100%;
  }
  .player {
    display: grid;
    gap: 10px;
    align-content: center;
    min-width: 0;
  }
  .track {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }
  .meta {
    flex: 1;
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .meta > * {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta strong {
    font-size: 15px;
  }
  .meta span {
    color: var(--notch-fg-dim);
    font-size: 13px;
  }
  .time {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 11px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
  .bar {
    flex: 1;
    height: 12px;
    display: flex;
    align-items: center;
    background: linear-gradient(#ffffff22, #ffffff22) center / 100% 4px no-repeat;
    border-radius: 2px;
  }
  .fill {
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--accent-media);
    transform-origin: left;
    transition: background-color 400ms ease-out;
  }
  .fill.paused {
    background: var(--notch-fg-dim);
  }
  :global(.glow) .fill:not(.paused) {
    box-shadow: 0 0 3px color-mix(in srgb, var(--accent-media) 90%, transparent), 0 0 12px color-mix(in srgb, var(--accent-media) 60%, transparent);
  }
  .controls {
    display: flex;
    justify-content: center;
    gap: 16px;
  }
  .ctl {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    transition:
      background-color 150ms ease-out,
      transform 150ms ease-out;
  }
  .ctl:hover {
    background: var(--notch-hover);
  }
  .ctl:active {
    transform: scale(0.9);
  }
  .ctl :global(svg) {
    transition: transform 160ms ease-out;
  }
  .ctl.fwd:active :global(svg) {
    transform: translateX(3px);
  }
  .ctl.back:active :global(svg) {
    transform: translateX(-3px);
  }
  .ctl.dim {
    color: var(--notch-fg-dim);
  }
  .ctl.on {
    color: var(--accent-media);
    background: color-mix(in srgb, var(--accent-media) 14%, transparent);
  }
</style>
