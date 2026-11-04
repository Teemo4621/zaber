import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { settings } from "$lib/settings.svelte";
import { onStatusChange, status } from "$lib/status.svelte";
import { startleCompanion } from "./companionCue.svelte";
import { drop, pop, press, strainAt } from "./nudge.svelte";

export type Peek =
  | { kind: "volume"; level: number; muted: boolean }
  | { kind: "caps"; on: boolean }
  | { kind: "layout"; tag: string }
  | { kind: "power"; charging: boolean; battery: number | null }
  | { kind: "network" }
  | { kind: "audio"; name: string }
  | { kind: "water" }
  | { kind: "todo"; id: string; title: string }
  | { kind: "calendar"; title: string; link: string | null; color: string; minutes: number }
  | { kind: "focus"; phase: "focus" | "break" }
  | { kind: "notification"; app: string; appId: string; title: string; body: string }
  | { kind: "battery"; level: number; minutes: number | null }
  | { kind: "brightness"; level: number }
  | { kind: "clipboard"; copied: Copied }
  | { kind: "device"; device: DeviceKind; name: string; detail: string | null; path: string | null; connected: boolean }
  | { kind: "deviceBattery"; device: DeviceKind; name: string; level: number }
  | { kind: "mic"; muted: boolean }
  | { kind: "privacy"; device: "mic" | "camera"; app: string }
  | { kind: "file"; file: NewFile }
  | { kind: "claude"; count: number }
  | { kind: "claudeNote"; tone: "done" | "waiting" | "handed" | "ok" | "no"; title: string; sub: string };

export type Copied =
  | { kind: "text"; text: string }
  | { kind: "link"; text: string }
  | { kind: "image"; width: number; height: number }
  | { kind: "files"; name: string; count: number };
export type NewFile = { source: "screenshot" | "download"; path: string; name: string; bytes: number; size: [number, number] | null };
export type DeviceKind = "drive" | "mouse" | "keyboard" | "headphones";

const STICKY: Peek["kind"][] = ["network", "todo", "calendar"];

export const peeks = $state<{ current: Peek | null }>({ current: null });
let timer: ReturnType<typeof setTimeout> | undefined;
let held: Peek | null = null;

const isSticky = (p: Peek) =>
  STICKY.includes(p.kind) || (p.kind === "battery" && p.level <= 10) || (p.kind === "caps" && p.on) || (p.kind === "mic" && p.muted);

export function showPeek(peek: Peek, seconds = settings.value.peekSeconds) {
  if (peek.kind === "notification" || STICKY.includes(peek.kind)) drop();
  const sticky = isSticky(peek);
  if (sticky || held?.kind === peek.kind) held = null;
  if (!sticky && peeks.current && isSticky(peeks.current) && peeks.current.kind !== peek.kind) held = peeks.current;
  peeks.current = peek;
  startleCompanion();
  clearTimeout(timer);
  if (!sticky) timer = setTimeout(dismissPeek, seconds * 1000);
}

export function dismissPeek(kind?: Peek["kind"]) {
  if (kind && held?.kind === kind) held = null;
  if (kind && peeks.current?.kind !== kind) return;
  clearTimeout(timer);
  peeks.current = held;
  held = null;
}

onStatusChange((prev, next) => {
  const on = settings.value.peeks;
  if (on.volume && next.volume !== null && (next.volume !== prev.volume || next.muted !== prev.muted))
    showPeek({ kind: "volume", level: next.volume, muted: next.muted });
  if (on.caps && next.capsLock !== prev.capsLock) {
    showPeek({ kind: "caps", on: next.capsLock });
    (next.capsLock ? () => pop("s") : press)();
  }
  if (on.layout && next.layout && next.layout !== prev.layout) showPeek({ kind: "layout", tag: next.layout });
  if (on.power && next.charging !== null && next.charging !== prev.charging && next.charging)
    showPeek({ kind: "power", charging: true, battery: next.battery });
  const crossed = [20, 10].some((n) => prev.battery !== null && next.battery !== null && prev.battery > n && next.battery <= n);
  if (on.battery && crossed && !next.charging) showPeek({ kind: "battery", level: next.battery!, minutes: next.batteryMinutes }, 5);
  if (next.charging && !prev.charging) dismissPeek("battery");
  if (on.network && next.online !== prev.online) {
    if (next.online) dismissPeek("network");
    else showPeek({ kind: "network" });
  }
  if (on.mic && next.micMuted !== null && prev.micMuted !== null && next.micMuted !== prev.micMuted) showPeek({ kind: "mic", muted: next.micMuted });
  if (next.micMuted === null && prev.micMuted) dismissPeek("mic");
  if (on.privacy) {
    const mic = next.micApps.find((a) => !prev.micApps.includes(a));
    const cam = next.cameraApps.find((a) => !prev.cameraApps.includes(a));
    if (cam) showPeek({ kind: "privacy", device: "camera", app: cam }, 3);
    else if (mic) showPeek({ kind: "privacy", device: "mic", app: mic }, 3);
  }
  if (on.audio && next.audioDevice && prev.audioDevice && next.audioDevice !== prev.audioDevice)
    showPeek({ kind: "audio", name: next.audioDevice }, 3);
});

listen<"up" | "down">("status:limit", ({ payload }) => {
  if (!settings.value.peeks.volume) return;
  const level = status.now?.volume ?? (payload === "up" ? 100 : 0);
  showPeek({ kind: "volume", level, muted: status.now?.muted ?? false });
  strainAt(payload === "up" ? 1 : -1);
});

listen<{ level: number }>("brightness:update", ({ payload }) => {
  if (settings.value.peeks.brightness) showPeek({ kind: "brightness", level: payload.level });
});

listen<Copied>("clipboard:copied", ({ payload }) => {
  if (settings.value.peeks.clipboard) showPeek({ kind: "clipboard", copied: payload });
});

type Change = { kind: DeviceKind; name: string; detail: string | null; path: string | null; connected: boolean };
listen<Change>("devices:change", ({ payload: d }) => {
  if (!settings.value.peeks.devices) return;
  showPeek({ kind: "device", device: d.kind, name: d.name, detail: d.detail, path: d.path, connected: d.connected }, d.path && d.connected ? 5 : undefined);
});

listen<NewFile>("files:new", ({ payload }) => {
  if (settings.value.peeks.files) showPeek({ kind: "file", file: payload }, 6);
});

type Level = { kind: DeviceKind; name: string; level: number };
export const deviceLevels = $state<{ list: Level[] }>({ list: [] });
export const levelOf = (name: string) => deviceLevels.list.find((l) => name.includes(l.name) || l.name.includes(name))?.level ?? null;
invoke<Level[]>("devices_levels").then((l) => (deviceLevels.list = l), () => {});
listen<Level[]>("devices:levels", ({ payload }) => {
  const fresh = payload.find((l) => l.kind === "headphones" && !deviceLevels.list.some((o) => o.name === l.name));
  deviceLevels.list = payload;
  const out = status.now?.audioDevice;
  if (fresh && settings.value.peeks.audio && out && levelOf(out) !== null) showPeek({ kind: "audio", name: out }, 3);
});

listen<{ kind: DeviceKind; name: string; level: number }>("devices:battery", ({ payload: d }) => {
  if (settings.value.peeks.deviceBattery) showPeek({ kind: "deviceBattery", device: d.kind, name: d.name, level: d.level }, 5);
});
