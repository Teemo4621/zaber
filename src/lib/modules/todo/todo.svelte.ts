import { dismissPeek, peeks, showPeek } from "$lib/notch/peeks.svelte";
import { cheerCompanion } from "$lib/notch/companionCue.svelte";
import { pop } from "$lib/notch/nudge.svelte";
import { persisted } from "$lib/store.svelte";

export type Task = {
  id: string;
  title: string;
  time: string | null;
  day: string;
  done: boolean;
  remindAt?: number;
  alerted?: boolean;
};

export const today = (d = new Date()) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;

const asTasks = (v: unknown): Task[] => (Array.isArray(v) ? v.filter((t) => t && typeof t.title === "string") : []);
export const todo = persisted<Task[]>("todo", [], asTasks);

export const visibleTasks = () =>
  todo.value
    .filter((t) => !t.done || t.day === today())
    .toSorted((a, b) => +a.done - +b.done || (a.time ?? "99").localeCompare(b.time ?? "99"));

const put = (tasks: Task[]) => todo.save(tasks);
const patch = (id: string, p: Partial<Task>) => put(todo.value.map((t) => (t.id === id ? { ...t, ...p } : t)));

export function addTask(title: string, time: string | null) {
  const text = title.trim().slice(0, 200);
  if (!text) return;
  put([...todo.value, { id: crypto.randomUUID(), title: text, time: time || null, day: today(), done: false }]);
}
export const renameTask = (id: string, title: string) => title.trim() && patch(id, { title: title.trim().slice(0, 200) });
export const removeTask = (id: string) => put(todo.value.filter((t) => t.id !== id));

export function setDone(id: string, done: boolean) {
  patch(id, { done, day: done ? today() : todo.value.find((t) => t.id === id)?.day });
  if (done) pop("m");
  if (done) cheerCompanion();
  if (done && peeks.current?.kind === "todo" && peeks.current.id === id) dismissPeek("todo");
}

export function snooze(id: string, minutes = 10) {
  patch(id, { remindAt: Date.now() + minutes * 60_000, alerted: false });
  dismissPeek("todo");
}

function dueAt(t: Task): number | null {
  if (t.remindAt) return t.remindAt;
  if (!t.time) return null;
  const [h, m] = t.time.split(":").map(Number);
  const d = new Date(`${t.day}T00:00:00`);
  d.setHours(h, m, 0, 0);
  return d.getTime();
}

setInterval(() => {
  const now = Date.now();
  const due = todo.value.find((t) => !t.done && !t.alerted && (dueAt(t) ?? Infinity) <= now);
  if (!due) return;
  patch(due.id, { alerted: true });
  showPeek({ kind: "todo", id: due.id, title: due.title });
}, 20_000);
