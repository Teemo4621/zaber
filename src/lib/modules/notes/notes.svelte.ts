import { persisted } from "$lib/store.svelte";

export const notes = persisted<{ text: string }>("notes", { text: "" }, (v) => ({
  text: typeof (v as { text?: unknown })?.text === "string" ? (v as { text: string }).text : "",
}));

let timer: ReturnType<typeof setTimeout> | undefined;
export function setNotes(text: string, now = false) {
  clearTimeout(timer);
  if (now) notes.save({ text });
  else timer = setTimeout(() => notes.save({ text }), 400);
}
