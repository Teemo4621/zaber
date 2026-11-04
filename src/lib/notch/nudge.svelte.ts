import { Spring } from "svelte/motion";
import { reduceMotion } from "$lib/theme.svelte";

export function feel(response: number, zeta: number) {
  const w = (2 * Math.PI) / (60 * response);
  return { stiffness: w * w, damping: 2 * zeta * w };
}

export const stretch = new Spring({ w: 0, h: 0, x: 0 }, feel(0.3, 1));

const FEEL = {
  tight: { ...feel(0.25, 1), hold: 90 },
  bouncy: { ...feel(0.35, 0.75), hold: 140 },
  rubber: { ...feel(0.4, 0.85), hold: 200 },
};

let release: ReturnType<typeof setTimeout> | undefined;
function push(w: number, h: number, x: number, f: (typeof FEEL)[keyof typeof FEEL]) {
  if (reduceMotion()) return;
  stretch.stiffness = f.stiffness;
  stretch.damping = f.damping;
  stretch.target = { w, h, x };
  clearTimeout(release);
  release = setTimeout(() => (stretch.target = { w: 0, h: 0, x: 0 }), f.hold);
}

export const press = () => push(-10, -3, 0, FEEL.tight);
export const lean = (dir: 1 | -1) => push(12, 0, dir * 6, FEEL.bouncy);
const POP = { s: [10, 3], m: [18, 5], l: [30, 12] } as const;
export const pop = (size: keyof typeof POP) => push(POP[size][0], POP[size][1], 0, FEEL.bouncy);
export const drop = () => push(4, 9, 0, FEEL.bouncy);

let strain = 0;
let lastStrain = 0;
export function strainAt(dir: 1 | -1, axis: "w" | "x" = "w") {
  const now = performance.now();
  strain = now - lastStrain < 450 ? strain + 1 : 1;
  lastStrain = now;
  const give = 1 - Math.exp(-strain / 2);
  if (axis === "x") push(14 * give, 0, dir * 18 * give, FEEL.rubber);
  else {
    const w = dir > 0 ? 64 * give : -36 * give;
    push(w, -Math.abs(w) * 0.08, 0, FEEL.rubber);
  }
}
