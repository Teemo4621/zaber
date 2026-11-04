<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "$lib/Icon.svelte";
  import { locale, t } from "$lib/i18n.svelte";
  import { calendar, refreshCalendar, restOfToday, tomorrowFirst } from "./calendar.svelte";

  refreshCalendar();

  const now = Date.now();
  const time = (ms: number) => new Date(ms).toLocaleTimeString(locale(), { hour: "2-digit", minute: "2-digit" });
  const today = new Date();
  const week = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(today);
    d.setDate(today.getDate() - ((today.getDay() + 6) % 7) + i);
    return d;
  });
  const events = $derived(restOfToday(now));
  const next = $derived(tomorrowFirst());
  const mac = navigator.userAgent.includes("Mac");
  const problem = $derived.by(() => {
    const e = calendar.error;
    if (!e) return null;
    if (e.includes("pending")) return { title: t("cal.asking"), hint: t("cal.askingHint") };
    return { title: t("cal.unavailable"), hint: mac ? t("cal.unavailableHintMac") : t("cal.unavailableHint") };
  });
</script>

<div class="cal">
    <div class="day">
      <div class="date">
        <span class="wd">{today.toLocaleDateString(locale(), { weekday: "long" })}</span>
        <span class="big">{today.toLocaleDateString(locale(), { day: "numeric", month: "short" })}</span>
      </div>
      <div class="week">
        {#each week as d (d.getDate())}
          <span class="wday" class:today={d.toDateString() === today.toDateString()}>
            {d.toLocaleDateString(locale(), { weekday: "narrow" })}<b>{d.getDate()}</b>
          </span>
        {/each}
      </div>
    </div>
    {#if problem}
      <div class="empty">
        <Icon name="calendar" size={22} />
        <strong>{problem.title}</strong>
        <span>{problem.hint}</span>
      </div>
    {:else if calendar.loaded && events.length === 0}
      <div class="empty">
        <Icon name="calendarCheck" size={22} />
        <strong>{t("cal.nothingLeft")}</strong>
        <span>{next ? t("cal.tomorrowStarts", { title: next.title, time: time(next.startMs) }) : t("cal.tomorrowFree")}</span>
      </div>
    {:else}
    <ul>
      {#each events as e (e.title + e.startMs)}
        {@const live = e.startMs <= now}
        <li class:live>
          <time>{e.allDay ? t("cal.allDay") : live ? t("cal.now") : time(e.startMs)}</time>
          <i style:background={e.color}></i>
          <span class="what">
            {e.title}
            {#if live && !e.allDay}<small>{t("cal.until", { time: time(e.endMs) })}{e.location && !e.link ? ` · ${e.location}` : ""}</small>{/if}
          </span>
          {#if e.link}
            <button class="btn solid join" onclick={() => openUrl(e.link!)}>{t("cal.join")}</button>
          {/if}
        </li>
      {/each}
    </ul>
    {/if}
</div>

<style>
  .cal {
    display: flex;
    gap: 18px;
    height: 100%;
    min-height: 0;
  }
  .day {
    width: 168px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .date {
    display: grid;
  }
  .wd {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent-alert);
  }
  .big {
    font-size: 30px;
    font-weight: 650;
    line-height: 1.1;
  }
  .week {
    display: flex;
    justify-content: space-between;
  }
  .wday {
    width: 22px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    font-size: 11px;
    color: var(--notch-fg-dim);
  }
  .wday b {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 12px;
    color: var(--notch-fg);
  }
  .wday.today b {
    background: var(--accent-alert);
    color: #000;
  }
  ul {
    flex: 1;
    min-width: 0;
    list-style: none;
    margin: 0;
    padding: 0 0 0 12px;
    border-left: 1px solid #ffffff1a;
    overflow-y: auto;
  }
  li {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 36px;
    padding: 0 8px;
    border-radius: 10px;
    font-size: 13px;
  }
  li.live {
    background: #ffffff0f;
  }
  time {
    width: 46px;
    flex: none;
    font-size: 12px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
  i {
    width: 4px;
    height: 22px;
    border-radius: 2px;
    flex: none;
  }
  .what {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  small {
    display: block;
    font-size: 12px;
    color: var(--notch-fg-dim);
  }
  .join {
    height: 26px;
    background: var(--accent-media);
  }
  .empty {
    flex: 1;
    min-width: 0;
    padding-left: 12px;
    border-left: 1px solid #ffffff1a;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    color: var(--notch-fg-dim);
    font-size: 13px;
    text-align: center;
  }
  .empty strong {
    color: var(--notch-fg);
    font-size: 15px;
  }
</style>
