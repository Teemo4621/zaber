<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { t } from "$lib/i18n.svelte";
  import { settings, updateSettings } from "$lib/settings.svelte";
  import { byHour, drink, todayMl, undoDrink } from "./water.svelte";

  let editing = $state(false);
  const w = $derived(settings.value.water);
  const total = $derived(todayMl());
  const hours = $derived(byHour());
  const peak = $derived(Math.max(w.bottleMl, ...hours.map((h) => h.ml)));
  const pad = (h: number) => String(h).padStart(2, "0");

  function setWater(key: keyof typeof w, value: number, min: number, max: number) {
    if (!Number.isFinite(value)) return;
    updateSettings({ water: { ...w, [key]: Math.min(max, Math.max(min, Math.round(value))) } });
  }
</script>

{#if editing}
  <form class="edit" onsubmit={(e) => (e.preventDefault(), (editing = false))}>
    <label>{t("water.goal")}<input type="number" min="250" max="6000" step="250" value={w.goalMl} onchange={(e) => setWater("goalMl", +e.currentTarget.value, 250, 6000)} /></label>
    <label>{t("water.bottle")}<input type="number" min="100" max="2000" step="50" value={w.bottleMl} onchange={(e) => setWater("bottleMl", +e.currentTarget.value, 100, 2000)} /></label>
    <label>{t("water.from")}<input type="number" min="0" max="23" value={w.fromHour} onchange={(e) => setWater("fromHour", +e.currentTarget.value, 0, w.toHour - 1)} /></label>
    <label>{t("water.to")}<input type="number" min="1" max="24" value={w.toHour} onchange={(e) => setWater("toHour", +e.currentTarget.value, w.fromHour + 1, 24)} /></label>
    <label>{t("water.every")}<input type="number" min="15" max="240" step="15" value={w.everyMin} onchange={(e) => setWater("everyMin", +e.currentTarget.value, 15, 240)} /></label>
    <button class="btn solid" type="submit" style:background="var(--accent-calm)">OK</button>
  </form>
{:else}
  <div class="water">
    <div class="left">
      <div class="read">
        <span class="big">{(total / 1000).toFixed(1)} <small>/ {(w.goalMl / 1000).toFixed(1)} L</small></span>
        <span class="dim">
          {total >= w.goalMl ? t("water.reached") : t("water.toGo", { ml: w.goalMl - total })} · {t("water.hours", { from: pad(w.fromHour), to: pad(w.toHour) })}
        </span>
      </div>
      <div class="meter"><span style:transform="scaleX({Math.min(1, total / w.goalMl)})"></span></div>
      <div class="row">
        <button class="btn solid" style:background="var(--accent-calm)" onclick={() => drink()}>
          <Icon name="plus" size={14} />{t("water.add", { ml: w.bottleMl })}
        </button>
        <button class="btn icon" aria-label={t("water.undo")} onclick={undoDrink} disabled={total === 0}><Icon name="reset" size={16} /></button>
        <button class="btn icon" aria-label={t("water.settings")} onclick={() => (editing = true)}><Icon name="gear" size={16} /></button>
      </div>
    </div>
    <div class="chart" role="img" aria-label={t("water.chart")}>
      <div class="bars">
        {#each hours as h (h.hour)}
          <span title="{pad(h.hour)}:00 · {h.ml} ml" style:height="{(h.ml / peak) * 100}%"></span>
        {/each}
      </div>
      <div class="axis">
        <span>{pad(w.fromHour)}</span><span>{pad(Math.round((w.fromHour + w.toHour) / 2))}</span><span>{pad(w.toHour)}</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .water {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px;
    height: 100%;
  }
  .left {
    display: grid;
    align-content: center;
    gap: 10px;
  }
  .read {
    display: grid;
    gap: 2px;
  }
  .big {
    font-size: 30px;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
  }
  .big small,
  .dim {
    font-size: 13px;
    color: var(--notch-fg-dim);
  }
  .big small {
    font-size: 16px;
  }
  .meter {
    height: 6px;
    border-radius: 3px;
    background: color-mix(in srgb, var(--accent-calm) 20%, transparent);
  }
  .meter span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--accent-calm);
    transform-origin: left;
    transition: transform 300ms ease-out;
  }
  .row {
    display: flex;
    gap: 6px;
  }
  .chart {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-top: 8px;
  }
  .bars {
    flex: 1;
    display: flex;
    align-items: flex-end;
    gap: 3px;
    border-bottom: 1px solid #ffffff22;
  }
  .bars span {
    flex: 1;
    min-height: 2px;
    border-radius: 3px 3px 0 0;
    background: var(--accent-calm);
  }
  :global(.glow) .meter span,
  :global(.glow) .bars span {
    box-shadow: 0 0 3px color-mix(in srgb, var(--accent-calm) 90%, transparent), 0 0 12px color-mix(in srgb, var(--accent-calm) 60%, transparent);
  }
  .axis {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--notch-fg-dim);
  }
  .edit {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px 12px;
    align-items: end;
  }
  .edit label {
    display: grid;
    gap: 4px;
    font-size: 11px;
    color: var(--notch-fg-dim);
  }
  .edit input {
    height: 30px;
    border: 1px solid #ffffff22;
    border-radius: 8px;
    background: var(--notch-surface);
    color: var(--notch-fg);
    font: inherit;
    font-size: 13px;
    padding: 0 8px;
    color-scheme: dark;
  }
</style>
