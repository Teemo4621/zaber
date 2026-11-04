import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { lean, press } from "$lib/notch/nudge.svelte";
import { vividColor } from "./color";
import { cleanTrack, parseLrc, parsePlain, type LyricLine } from "./lrc";

export type NowPlaying = {
  title: string;
  artist: string;
  album: string;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  sourceApp: string;
  artwork: string | null;
};

export type MediaAction = "play_pause" | "next" | "previous";

export const media = $state<{ now: NowPlaying | null; receivedAt: number }>({ now: null, receivedAt: 0 });

function set(n: NowPlaying | null) {
  if (n?.artwork !== media.now?.artwork) {
    mediaUi.artColor = null;
    const src = n?.artwork;
    if (src) artworkColor(src).then((c) => media.now?.artwork === src && (mediaUi.artColor = c));
  }
  media.now = n;
  if (!n) mediaUi.lyrics = false;
  media.receivedAt = Date.now();
}
invoke<NowPlaying | null>("media_now").then(set);
listen<NowPlaying | null>("media:update", (e) => set(e.payload));

const FEEDBACK: Partial<Record<MediaAction, () => void>> = {
  next: () => lean(1),
  previous: () => lean(-1),
  play_pause: press,
};
export function control(action: MediaAction) {
  FEEDBACK[action]?.();
  return invoke("media_control", { action });
}
export const seek = (positionMs: number) => invoke("media_control", { action: "seek", positionMs });

export function positionNow(at = Date.now()) {
  const n = media.now;
  if (!n) return 0;
  const ms = n.positionMs + (n.playing ? at - media.receivedAt : 0);
  return n.durationMs > 0 ? Math.min(ms, n.durationMs) : ms;
}

export const formatTime = (ms: number) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(s / 3600);
  const mm = String(Math.floor((s % 3600) / 60)).padStart(h ? 2 : 1, "0");
  return `${h ? h + ":" : ""}${mm}:${String(s % 60).padStart(2, "0")}`;
};

const lyricsCache = new Map<string, LyricLine[] | null>();
type LrcHit = { syncedLyrics?: string | null; plainLyrics?: string | null; duration?: number };

async function lrclib(path: "get" | "search", params: Record<string, string>): Promise<LrcHit | LrcHit[] | null | undefined> {
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      const res = await fetch(`https://lrclib.net/api/${path}?${new URLSearchParams(params)}`);
      if (res.ok) return await res.json();
      if (res.status !== 429 && res.status < 500) return null;
    } catch {
    }
    await new Promise((r) => setTimeout(r, 700 * (attempt + 1)));
  }
  return undefined;
}

export async function fetchLyrics(n: NowPlaying): Promise<LyricLine[] | null> {
  const key = `${n.title}\0${n.artist}`;
  if (lyricsCache.has(key)) return lyricsCache.get(key)!;
  const { track, artist } = cleanTrack(n.title, n.artist);
  const secs = n.durationMs > 0 ? Math.round(n.durationMs / 1000) : 0;
  const close = (h: LrcHit) => !secs || !h.duration || Math.abs(h.duration - secs) <= 3;
  let busy = false;
  let plain: string | null = null;
  const pick = (body: LrcHit | LrcHit[] | null | undefined): string | null => {
    if (body === undefined) busy = true;
    const hits = (Array.isArray(body) ? body : body ? [body] : []).filter(close);
    plain ??= hits.find((h) => h.plainLyrics)?.plainLyrics ?? null;
    return hits.find((h) => h.syncedLyrics)?.syncedLyrics ?? null;
  };

  const exact: Record<string, string> = { track_name: track, artist_name: artist };
  if (n.album && track === n.title) exact.album_name = n.album;
  if (secs) exact.duration = String(secs);
  const synced =
    pick(await lrclib("get", exact)) ??
    pick(await lrclib("search", { track_name: track, artist_name: artist })) ??
    pick(await lrclib("search", { q: `${artist} ${track}`.trim() }));

  const lines = synced ? parseLrc(synced) : plain ? parsePlain(plain) : null;
  if (lines || !busy) lyricsCache.set(key, lines);
  return lines;
}

export async function artworkColor(src: string): Promise<string | null> {
  const img = new Image();
  img.src = src;
  try {
    await img.decode();
  } catch {
    return null;
  }
  const c = document.createElement("canvas");
  c.width = c.height = 16;
  const ctx = c.getContext("2d")!;
  ctx.drawImage(img, 0, 0, 16, 16);
  return vividColor(ctx.getImageData(0, 0, 16, 16).data);
}

export const mediaUi = $state<{ lyrics: boolean; line: string | null; artColor: string | null }>({ lyrics: false, line: null, artColor: null });
