import { prefersReducedMotion } from "svelte/motion";
import { settings } from "./settings.svelte";
import { status } from "./status.svelte";

const NOTCH_BG = { black: "#000", graphite: "#2c2c2e", glass: "rgba(28, 28, 30, 0.78)" };

export const reduceMotion = () => settings.value.motion === "reduce" || prefersReducedMotion.current;

export function applyTheme() {
  $effect(() => {
    const s = settings.value;
    const system = s.useSystemAccent ? status.now?.accent : null;
    const root = document.documentElement.style;
    root.setProperty("--accent-media", system ?? s.accents.media);
    root.setProperty("--accent-hud", system ?? s.accents.hud);
    root.setProperty("--notch-bg", NOTCH_BG[s.notchStyle]);
    root.setProperty("--notch-border", s.notchStyle === "glass" ? "1px solid #ffffff22" : "0");
    root.setProperty("--pill-text", s.textSize === "large" ? "15px" : "13px");
    document.documentElement.classList.toggle("reduce-motion", reduceMotion());
    document.documentElement.classList.toggle("glow", s.glow);
    document.documentElement.lang = s.language;
  });
}
