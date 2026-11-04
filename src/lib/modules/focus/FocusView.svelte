<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { t } from "$lib/i18n.svelte";
  import FocusRing from "./FocusRing.svelte";
  import { SESSIONS, clockText, extend, finish, focus, pause, remainingMs, reset, running, start, totalMs } from "./focus.svelte";

  const left = $derived(remainingMs());
  const onBreak = $derived(focus.phase === "short" || focus.phase === "long");
  const color = $derived(onBreak ? "var(--accent-calm)" : "var(--accent-focus)");
  const label = $derived(
    { ready: t("focus.ready"), focus: t("focus.focus"), short: t("focus.short"), long: t("focus.long") }[focus.phase],
  );
</script>

<div class="focus">
  <FocusRing progress={1 - left / totalMs()} size={96} stroke={8} {color} />
  <div class="side">
    <div class="read">
      <span class="label" style:color>{label} · {t("focus.session", { n: focus.session, total: SESSIONS })}</span>
      <span class="big">{clockText(left)}</span>
    </div>
    <div class="row">
      {#if onBreak}
        <button class="btn solid" style:background={color} onclick={() => (finish(), start())}>{t("focus.startNext")}</button>
        <button class="btn" onclick={() => extend()}>{t("focus.extend")}</button>
      {:else if running()}
        <button class="btn solid" style:background={color} onclick={pause}><Icon name="pause" size={14} filled />{t("focus.pause")}</button>
        <button class="btn" onclick={finish}>{t("focus.skip")}</button>
        <button class="btn icon" aria-label={t("focus.reset")} onclick={reset}><Icon name="reset" size={16} /></button>
      {:else}
        <button class="btn solid" style:background={color} onclick={start}>
          <Icon name="play" size={14} filled />{focus.phase === "ready" ? t("focus.start") : t("focus.resume")}
        </button>
        {#if focus.phase !== "ready"}
          <button class="btn icon" aria-label={t("focus.reset")} onclick={reset}><Icon name="reset" size={16} /></button>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .focus {
    display: flex;
    align-items: center;
    gap: 24px;
    height: 100%;
  }
  .side {
    flex: 1;
    display: grid;
    gap: 10px;
  }
  .read {
    display: grid;
  }
  .label {
    font-size: 13px;
    font-weight: 600;
  }
  .big {
    font-size: 40px;
    font-weight: 650;
    line-height: 1.1;
    font-variant-numeric: tabular-nums;
  }
  .row {
    display: flex;
    gap: 8px;
  }
</style>
