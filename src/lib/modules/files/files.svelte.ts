import { listen } from "@tauri-apps/api/event";
import { settings } from "$lib/settings.svelte";

export type Downloading = { name: string; bytes: number; speed: number };

export const files = $state<{ downloading: Downloading[] }>({ downloading: [] });

listen<Downloading[]>("files:downloading", ({ payload }) => (files.downloading = settings.value.peeks.files ? payload : []));

export const bytes = (n: number) =>
  n < 1e6 ? `${Math.max(1, Math.round(n / 1e3))} KB` : n < 1e9 ? `${(n / 1e6).toFixed(1)} MB` : `${(n / 1e9).toFixed(2)} GB`;
