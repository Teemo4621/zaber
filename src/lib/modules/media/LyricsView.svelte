<script lang="ts">
  import { t } from "$lib/i18n.svelte";
  import { onDestroy } from "svelte";
  import { currentLine, type LyricLine } from "./lrc";
  import { fetchLyrics, positionNow, seek, type NowPlaying } from "./media.svelte";

  let { now }: { now: NowPlaying } = $props();

  let lines = $state<LyricLine[] | null | undefined>(undefined);
  $effect(() => {
    const track = now.title + now.artist;
    lines = undefined;
    fetchLyrics(now).then((l) => {
      if (track === now.title + now.artist) lines = l;
    });
  });

  let clock = $state(Date.now());
  const tick = setInterval(() => now.playing && (clock = Date.now()), 250);
  onDestroy(() => clearInterval(tick));
  const synced = $derived(!!lines?.length && lines[0].ms >= 0);
  const index = $derived(lines && synced ? currentLine(lines, positionNow(clock)) : -1);
  const visible = $derived(lines ? lines.slice(Math.max(0, index - 2), Math.max(0, index - 2) + 5) : []);
</script>

<div class="lyrics" aria-live="off">
  {#if lines === undefined}
    <p class="note">{t("media.lyricsLoading")}</p>
  {:else if lines === null || lines.length === 0}
    <p class="note">{t("media.lyricsNone")}</p>
  {:else if !synced}
    <ul class="plain">
      {#each lines as line, i (i)}<li>{line.text || "…"}</li>{/each}
    </ul>
  {:else}
    {#each visible as line (line.ms)}
      <button class="line" class:now={lines[index] === line} onclick={() => seek(line.ms)}>{line.text || "…"}</button>
    {/each}
  {/if}
</div>

<style>
  .lyrics {
    border-left: 1px solid #ffffff1a;
    padding-left: 20px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 8px;
    overflow: hidden;
    min-width: 0;
  }
  .line {
    font-size: 14px;
    line-height: 1.35;
    color: #6e6e73;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition:
      color 200ms ease-out,
      font-size 200ms ease-out;
  }
  .line:hover {
    color: var(--notch-fg-dim);
  }
  .line.now {
    font-size: 17px;
    font-weight: 650;
    color: var(--accent-media);
  }
  :global(.glow) .line.now {
    text-shadow: 0 0 2px color-mix(in srgb, var(--accent-media) 70%, transparent), 0 0 14px color-mix(in srgb, var(--accent-media) 70%, transparent);
  }
  .plain {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    max-height: 100%;
    display: grid;
    gap: 4px;
    font-size: 13px;
    color: var(--notch-fg-dim);
  }
  .note {
    margin: 0;
    font-size: 13px;
    color: var(--notch-fg-dim);
  }
</style>
