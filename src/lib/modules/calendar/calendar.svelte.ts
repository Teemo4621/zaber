import { invoke } from "@tauri-apps/api/core";
import { showPeek } from "$lib/notch/peeks.svelte";
import { settings } from "$lib/settings.svelte";

export type CalEvent = {
  title: string;
  startMs: number;
  endMs: number;
  allDay: boolean;
  location: string;
  link: string | null;
  color: string;
};

export const calendar = $state<{ events: CalEvent[]; error: string | null; loaded: boolean }>({
  events: [],
  error: null,
  loaded: false,
});

export async function refreshCalendar() {
  try {
    calendar.events = await invoke<CalEvent[]>("calendar_events", { fromMs: new Date().setHours(0, 0, 0, 0), days: 2 });
    calendar.error = null;
  } catch (e) {
    calendar.error = String(e);
  }
  calendar.loaded = true;
}

const endOfToday = () => new Date().setHours(24, 0, 0, 0);
export const restOfToday = (now = Date.now()) => calendar.events.filter((e) => e.endMs > now && e.startMs < endOfToday());
export const tomorrowFirst = () => calendar.events.find((e) => e.startMs >= endOfToday() && !e.allDay);

refreshCalendar();
setInterval(refreshCalendar, 5 * 60_000);

const announced = new Set<string>();
setInterval(() => {
  const before = settings.value.calendarHeadsUp * 60_000;
  if (!before) return;
  const now = Date.now();
  const next = calendar.events.find((e) => !e.allDay && e.startMs > now && e.startMs - now <= before && !announced.has(e.title + e.startMs));
  if (!next) return;
  announced.add(next.title + next.startMs);
  showPeek({
    kind: "calendar",
    title: next.title,
    link: next.link,
    color: next.color,
    minutes: Math.max(1, Math.round((next.startMs - now) / 60_000)),
  });
}, 20_000);
