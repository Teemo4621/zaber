import { pop } from "$lib/notch/nudge.svelte";
import { showPeek } from "$lib/notch/peeks.svelte";

export type Phase = "ready" | "focus" | "short" | "long";

export const DURATION_MS: Record<Exclude<Phase, "ready">, number> = {
  focus: 25 * 60_000,
  short: 5 * 60_000,
  long: 15 * 60_000,
};
export const SESSIONS = 4;

export const focus = $state({
  phase: "ready" as Phase,
  session: 1,
  endsAt: null as number | null,
  leftMs: DURATION_MS.focus,
  now: Date.now(),
});

let tick: ReturnType<typeof setInterval> | undefined;

export const running = () => focus.endsAt !== null;
export const remainingMs = () => (focus.endsAt !== null ? Math.max(0, focus.endsAt - focus.now) : focus.leftMs);
export const totalMs = () => (focus.phase === "ready" ? DURATION_MS.focus : DURATION_MS[focus.phase]);

function run(ms: number) {
  focus.now = Date.now();
  focus.endsAt = focus.now + ms;
  clearInterval(tick);
  tick = setInterval(() => {
    focus.now = Date.now();
    if (remainingMs() === 0) finish();
  }, 1000);
}

function stop() {
  clearInterval(tick);
  tick = undefined;
  focus.endsAt = null;
}

export function start() {
  if (focus.phase === "ready") focus.phase = "focus";
  run(focus.endsAt === null && focus.leftMs > 0 ? focus.leftMs : DURATION_MS.focus);
}

export function pause() {
  focus.leftMs = remainingMs();
  stop();
}

export function reset() {
  stop();
  focus.phase = "ready";
  focus.session = 1;
  focus.leftMs = DURATION_MS.focus;
}

export function finish() {
  stop();
  chime();
  pop("m");
  if (focus.phase === "focus") {
    focus.phase = focus.session % SESSIONS === 0 ? "long" : "short";
    showPeek({ kind: "focus", phase: "break" }, 4);
    run(DURATION_MS[focus.phase]);
  } else {
    focus.session = (focus.session % SESSIONS) + 1;
    focus.phase = "ready";
    focus.leftMs = DURATION_MS.focus;
    showPeek({ kind: "focus", phase: "focus" }, 4);
  }
}

export function extend(ms = 5 * 60_000) {
  if (focus.endsAt !== null) focus.endsAt += ms;
  else focus.leftMs += ms;
}

function chime() {
  try {
    const ctx = new AudioContext();
    [660, 880].forEach((hz, i) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      const t0 = ctx.currentTime + i * 0.18;
      osc.frequency.value = hz;
      gain.gain.setValueAtTime(0.0001, t0);
      gain.gain.exponentialRampToValueAtTime(0.15, t0 + 0.02);
      gain.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.9);
      osc.connect(gain).connect(ctx.destination);
      osc.start(t0);
      osc.stop(t0 + 1);
    });
    setTimeout(() => ctx.close(), 1500);
  } catch {
  }
}

export const clockText = (ms: number) =>
  `${String(Math.floor(ms / 60_000)).padStart(2, "0")}:${String(Math.floor((ms % 60_000) / 1000)).padStart(2, "0")}`;
