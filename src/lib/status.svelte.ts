import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Status = {
  volume: number | null;
  muted: boolean;
  audioDevice: string | null;
  capsLock: boolean;
  layout: string | null;
  charging: boolean | null;
  battery: number | null;
  batteryMinutes: number | null;
  online: boolean;
  fullscreen: boolean;
  accent: string | null;
  micMuted: boolean | null;
  micApps: string[];
  cameraApps: string[];
};

export const status = $state<{ now: Status | null }>({ now: null });

const watchers: ((prev: Status, next: Status) => void)[] = [];
export const onStatusChange = (fn: (prev: Status, next: Status) => void) => watchers.push(fn);

function set(next: Status | null) {
  const prev = status.now;
  status.now = next;
  if (prev && next) for (const fn of watchers) fn(prev, next);
}

invoke<Status | null>("status_now").then((s) => {
  if (!status.now) set(s);
});
listen<Status>("status:update", (e) => set(e.payload));
