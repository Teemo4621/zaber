import { dismissPeek, showPeek } from "$lib/notch/peeks.svelte";
import { cheerCompanion } from "$lib/notch/companionCue.svelte";
import { drop } from "$lib/notch/nudge.svelte";
import { settings } from "$lib/settings.svelte";
import { persisted } from "$lib/store.svelte";

export type Drink = { at: number; ml: number };

const KEEP_MS = 30 * 86_400_000;
const asLog = (v: unknown): Drink[] => (Array.isArray(v) ? v.filter((d) => typeof d?.at === "number" && typeof d?.ml === "number") : []);
export const water = persisted<Drink[]>("water", [], asLog);

const startOfToday = () => new Date().setHours(0, 0, 0, 0);
export const todayDrinks = () => water.value.filter((d) => d.at >= startOfToday());
export const todayMl = () => todayDrinks().reduce((sum, d) => sum + d.ml, 0);

export function byHour() {
  const { fromHour, toHour } = settings.value.water;
  const hours = Array.from({ length: Math.max(1, toHour - fromHour) }, (_, i) => ({ hour: fromHour + i, ml: 0 }));
  for (const d of todayDrinks()) {
    const slot = hours.find((h) => h.hour === new Date(d.at).getHours());
    if (slot) slot.ml += d.ml;
  }
  return hours;
}

export function drink(ml = settings.value.water.bottleMl) {
  const goal = settings.value.water.goalMl;
  if (todayMl() < goal && todayMl() + ml >= goal) cheerCompanion();
  const cutoff = Date.now() - KEEP_MS;
  water.save([...water.value.filter((d) => d.at >= cutoff), { at: Date.now(), ml }]);
  dismissPeek("water");
  drop();
}

export const undoDrink = () => water.save(todayDrinks().length ? water.value.slice(0, -1) : water.value);

let lastReminder = 0;
setInterval(() => {
  const { fromHour, toHour, everyMin, goalMl } = settings.value.water;
  const now = Date.now();
  const hour = new Date().getHours();
  if (hour < fromHour || hour >= toHour || todayMl() >= goalMl) return;
  const lastDrink = Math.max(startOfToday() + fromHour * 3_600_000, ...todayDrinks().map((d) => d.at));
  const every = everyMin * 60_000;
  if (now - lastDrink >= every && now - lastReminder >= every) {
    lastReminder = now;
    showPeek({ kind: "water" }, 6);
  }
}, 60_000);
