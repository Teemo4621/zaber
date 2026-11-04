import { listen } from "@tauri-apps/api/event";

export type Cue = "happy" | "startle";

export const companion = $state<{ cue: Cue | null; napped: boolean; near: number | null }>({ cue: null, napped: false, near: null });

listen<number | null>("notch:near", ({ payload }) => (companion.near = payload));

export const cheerCompanion = () => (companion.cue = "happy");

export function startleCompanion() {
  if (companion.napped) companion.cue ??= "startle";
}
