import { persisted } from "./store.svelte";

export const TAB_IDS = ["media", "calendar", "focus", "todo", "notes", "water", "shelf"] as const;
export type TabId = (typeof TAB_IDS)[number];
export const PEEK_KINDS = ["volume", "caps", "layout", "power", "network", "audio", "notification", "battery", "brightness", "clipboard", "devices", "deviceBattery", "mic", "privacy", "files"] as const;
export type PeekKind = (typeof PEEK_KINDS)[number];

export type Settings = {
  language: "en" | "th";
  display: "main" | "cursor";
  openDelay: number;
  hideFullscreen: boolean;
  companion: boolean;
  tabs: { id: TabId; on: boolean }[];
  openTo: "live" | "last" | "first";
  lastTab: TabId | "agents";
  pillPriority: "focus" | "media";
  lyrics: boolean;
  calendarHeadsUp: number;
  peeks: Record<PeekKind, boolean>;
  peekSeconds: number;
  volumeKeys: boolean;
  claudeWait: number;
  notchStyle: "black" | "graphite" | "glass";
  notchSize: "compact" | "standard" | "large";
  textSize: "standard" | "large";
  motion: "system" | "reduce";
  accents: { media: string; hud: string };
  useSystemAccent: boolean;
  followArtwork: boolean;
  glow: boolean;
  welcomed: boolean;
  water: { goalMl: number; bottleMl: number; fromHour: number; toHour: number; everyMin: number };
};

export const DEFAULT_SETTINGS: Settings = {
  language: "en",
  display: "main",
  openDelay: 150,
  hideFullscreen: true,
  companion: true,
  tabs: TAB_IDS.map((id) => ({ id, on: true })),
  openTo: "last",
  lastTab: "media",
  pillPriority: "focus",
  lyrics: true,
  calendarHeadsUp: 5,
  peeks: { volume: true, caps: true, layout: true, power: true, network: true, audio: true, notification: true, battery: true, brightness: true, clipboard: true, devices: true, deviceBattery: true, mic: true, privacy: true, files: true },
  peekSeconds: 2,
  volumeKeys: true,
  claudeWait: 60,
  notchStyle: "black",
  notchSize: "standard",
  textSize: "standard",
  motion: "system",
  accents: { media: "#30d158", hud: "#f5f5f7" },
  useSystemAccent: false,
  followArtwork: false,
  glow: true,
  welcomed: false,
  water: { goalMl: 2000, bottleMl: 500, fromHour: 8, toHour: 20, everyMin: 60 },
};

export function mergeSettings(saved: unknown): Settings {
  const s = (saved && typeof saved === "object" ? saved : {}) as Partial<Settings>;
  const tabs = (Array.isArray(s.tabs) ? s.tabs : []).filter((t) => TAB_IDS.includes(t?.id));
  for (const id of TAB_IDS) if (!tabs.some((t) => t.id === id)) tabs.push({ id, on: true });
  const d = DEFAULT_SETTINGS;
  return {
    ...d,
    ...s,
    tabs,
    lastTab: TAB_IDS.includes(s.lastTab as TabId) || s.lastTab === "agents" ? s.lastTab! : d.lastTab,
    peeks: { ...d.peeks, ...s.peeks },
    accents: { ...d.accents, ...s.accents },
    water: { ...d.water, ...s.water },
  };
}

export const settings = persisted("settings", DEFAULT_SETTINGS, mergeSettings);

export const updateSettings = (patch: Partial<Settings>) => settings.save({ ...settings.value, ...patch });

export const NOTCH_SCALE: Record<Settings["notchSize"], number> = { compact: 0.88, standard: 1, large: 1.16 };
