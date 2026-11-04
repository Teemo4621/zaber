<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { bytes, files } from "./files.svelte";

  const d = $derived(files.downloading[0]);
  const more = $derived(files.downloading.length - 1);
</script>

{#if d}
  <div class="pill">
    <span class="what">
      <span class="ic"><Icon name="download" size={14} /></span>
      <span class="ellipsis">{d.name}</span>
      {#if more > 0}<span class="more">+{more}</span>{/if}
    </span>
    <span class="num">{bytes(d.bytes)} · {bytes(d.speed)}/s</span>
  </div>
{/if}

<style>
  .pill {
    width: 100%;
    padding: 0 12px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: var(--pill-text);
    font-weight: 600;
  }
  .what {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .ic {
    display: grid;
    color: var(--accent-calm);
  }
  .ellipsis {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .more {
    font-size: 11px;
    color: var(--notch-fg-dim);
  }
  .num {
    flex: none;
    font-size: 12px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
</style>
