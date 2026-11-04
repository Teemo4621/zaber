<script lang="ts">
  import AgentLogo from "$lib/agents/AgentLogo.svelte";
  import { t } from "$lib/i18n.svelte";
  import { claude, type Limit, type Session } from "./claude.svelte";

  function state(s: Session) {
    if (s.state === "waiting") return t("claude.needsYou");
    if (s.state === "working") return t("claude.working");
    const min = Math.floor((Date.now() - s.since) / 60_000);
    return min < 1 ? t("claude.doneNow") : t("claude.doneAgo", { n: min });
  }

  function left(l: Limit) {
    const min = Math.max(0, Math.round((l.resetsAt * 1000 - Date.now()) / 60_000));
    const d = Math.floor(min / 1440), h = Math.floor((min % 1440) / 60), m = min % 60;
    return d ? `${d}d ${h}h` : h ? `${h}h ${m}m` : `${m}m`;
  }
  const used = (l: Limit) => (l.resetsAt * 1000 <= Date.now() ? 0 : Math.round(l.used));
  const limits = $derived(
    claude.usage
      ? ([["claude.limitSession", claude.usage.session], ["claude.limitWeek", claude.usage.week]] as const).filter((x): x is readonly [typeof x[0], Limit] => !!x[1])
      : [],
  );
</script>

<div class="list">
  <span class="title"><AgentLogo agent="claude" size={14} />{t("claude.sessions")}</span>
  {#if limits.length}
    <div class="limits">
      {#each limits as [label, l] (label)}
        {@const pct = used(l)}
        <div class="limit">
          <span class="lrow"><span>{t(label)}</span><b class:high={pct >= 80}>{pct}%</b></span>
          <span class="bar" role="meter" aria-label={t(label)} aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100}>
            <span class:high={pct >= 80} style:width="{pct}%"></span>
          </span>
          <span class="reset">{t("claude.resetsIn", { t: left(l) })}</span>
        </div>
      {/each}
    </div>
  {/if}
  {#if !claude.sessions.length}<span class="empty">{t("claude.noSessions")}</span>{/if}
  {#each claude.sessions.slice(0, limits.length ? 2 : 4) as s (s.id)}
    <div class="row" class:done={s.state === "done"}>
      <span class="txt">
        <b class="ellipsis">{s.project}</b>
        {#if s.detail}<span class="ellipsis">{s.detail}</span>{/if}
      </span>
      <span class="state" class:wait={s.state === "waiting"} class:work={s.state === "working"}>{state(s)}</span>
    </div>
  {/each}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--notch-fg-dim);
    margin-bottom: 4px;
  }
  .limits {
    display: flex;
    gap: 8px;
    margin-bottom: 4px;
  }
  .limit {
    flex: 1;
    padding: 6px 10px;
    border-radius: 10px;
    background: var(--notch-surface);
    display: grid;
    gap: 4px;
    font-size: 12px;
  }
  .lrow {
    display: flex;
    justify-content: space-between;
    color: var(--notch-fg-dim);
  }
  .lrow b {
    color: var(--notch-fg);
    font-variant-numeric: tabular-nums;
  }
  .bar {
    height: 4px;
    border-radius: 2px;
    background: var(--notch-hover);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: var(--notch-fg);
    transition: width 0.3s;
  }
  .high {
    color: var(--accent-focus) !important;
  }
  .bar .high {
    background: var(--accent-focus);
  }
  .reset {
    font-size: 11px;
    color: var(--notch-fg-dim);
  }
  .empty {
    font-size: 12px;
    color: var(--notch-fg-dim);
    padding: 6px 10px;
  }
  .row {
    height: 40px;
    padding: 0 10px;
    border-radius: 10px;
    background: var(--notch-surface);
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }
  .row.done {
    background: none;
  }
  .txt {
    flex: 1;
    min-width: 0;
    display: grid;
    line-height: 1.3;
  }
  .txt span {
    font-size: 12px;
    color: var(--notch-fg-dim);
  }
  .ellipsis {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .state {
    flex: none;
    font-size: 12px;
    color: var(--notch-fg-dim);
  }
  .state.work {
    color: var(--notch-fg);
  }
  .state.wait {
    color: var(--accent-focus);
  }
</style>
