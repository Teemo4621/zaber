import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { artworkColor } from "$lib/modules/media/media.svelte";
import { settings } from "$lib/settings.svelte";
import { showPeek } from "$lib/notch/peeks.svelte";

export type Notice = { id: number; app: string; appId: string; title: string; body: string };

export const appIcons = $state<Record<string, { src: string; color: string | null } | null>>({});

async function loadAppIcon(appId: string) {
  if (!appId || appId in appIcons) return;
  appIcons[appId] = null;
  const src = await invoke<string | null>("notify_icon", { appId }).catch(() => null);
  if (src) appIcons[appId] = { src, color: await artworkColor(src) };
}

listen<Notice>("notify:new", ({ payload: n }) => {
  if (!settings.value.peeks.notification || !(n.title || n.body)) return;
  loadAppIcon(n.appId);
  showPeek({ kind: "notification", app: n.app, appId: n.appId, title: n.title, body: n.body }, Math.max(settings.value.peekSeconds, 5));
});
