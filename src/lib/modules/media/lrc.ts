export type LyricLine = { ms: number; text: string };

export function parseLrc(lrc: string): LyricLine[] {
  const lines: LyricLine[] = [];
  for (const raw of lrc.split(/\r?\n/)) {
    const stamps = [...raw.matchAll(/\[(\d+):(\d{1,2})(?:[.:](\d{1,3}))?\]/g)];
    if (stamps.length === 0) continue;
    const text = raw.slice(stamps.at(-1)!.index! + stamps.at(-1)![0].length).trim();
    for (const [, m, s, frac = "0"] of stamps) {
      const ms = (+m * 60 + +s) * 1000 + Math.round(+frac.padEnd(3, "0"));
      lines.push({ ms, text });
    }
  }
  return lines.sort((a, b) => a.ms - b.ms);
}

export function currentLine(lines: LyricLine[], ms: number): number {
  let i = -1;
  while (i + 1 < lines.length && lines[i + 1].ms <= ms) i++;
  return i;
}

export const parsePlain = (text: string): LyricLine[] => text.split(/\r?\n/).map((t) => ({ ms: -1, text: t.trim() }));

const NOISE = /\s*[([【「].*?(official|video|audio|lyric|mv|m\/v|visualizer|remaster|hd|4k|live|เนื้อเพลง).*?[)\]】」]\s*/gi;
const FEAT = /\s+(ft\.?|feat\.?|featuring)\s.+$/i;

export function cleanTrack(title: string, artist: string): { track: string; artist: string } {
  let a = artist.replace(/\s*-\s*Topic$/i, "").replace(/VEVO$/i, "").replace(/\s*(official)$/i, "").trim();
  let t = title.replace(NOISE, " ").replace(/\s+/g, " ").trim();
  const parts = t.split(/\s+[-–—|]\s+/);
  if (parts.length >= 2) {
    const [left, ...rest] = parts;
    t = rest.join(" - ");
    if (!a || !left.toLowerCase().includes(a.toLowerCase()) || left.length >= a.length) a = left;
  }
  return { track: t.replace(FEAT, "").trim(), artist: a.replace(FEAT, "").trim() };
}
