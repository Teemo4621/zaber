<script module lang="ts">
  let wakeNext = false;
  export const wakeCompanion = () => (wakeNext = true);
</script>

<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { companion } from "./companionCue.svelte";

  type Mood = "away" | "awake" | "yawn" | "asleep" | "wave" | "happy" | "startle";
  const NEXT: Record<Mood, [Mood, number] | null> = {
    away: ["awake", 8_000],
    wave: ["awake", 1_600],
    happy: ["awake", 2_600],
    startle: ["awake", 1_400],
    awake: ["yawn", 60_000],
    yawn: ["asleep", 2_400],
    asleep: null,
  };
  const GLANCES = [15_000, 30_000, 45_000];

  let { awake = false }: { awake?: boolean } = $props();
  let mood = $state<Mood>(untrack(() => awake) || wakeNext ? "awake" : (companion.cue ?? (companion.napped ? "wave" : "away")));
  wakeNext = false;
  companion.cue = null;
  let glancing = $state(false);

  let timers: ReturnType<typeof setTimeout>[] = [];
  const later = (ms: number, fn: () => void) => timers.push(setTimeout(fn, ms));
  function enter(next: Mood) {
    timers.forEach(clearTimeout);
    timers = [];
    mood = next;
    glancing = false;
    companion.napped = next === "asleep";
    if (next === "awake")
      for (const at of GLANCES) {
        later(at, () => (glancing = true));
        later(at + 1_300, () => (glancing = false));
      }
    const step = NEXT[next];
    if (step) later(step[1], () => enter(step[0]));
  }
  enter(untrack(() => mood));
  onDestroy(() => timers.forEach(clearTimeout));

  $effect(() => {
    const cue = companion.cue;
    if (!cue) return;
    companion.cue = null;
    untrack(() => enter(cue));
  });

  const look = $derived(companion.near !== null && (mood === "awake" || mood === "happy") ? companion.near : null);
</script>

<div class="eyes {mood}" class:glancing aria-hidden="true" style:--look={look === null ? undefined : `${look * 2}px`} class:looking={look !== null}>
  <i></i><i></i>
  {#if mood === "yawn"}<span class="mouth"></span><span class="z">z</span><span class="z two">z</span>{/if}
  {#if mood === "startle"}<span class="bang">!</span>{/if}
  {#if mood === "happy"}
    <svg class="heart" viewBox="0 0 24 24"><path d="M12 21s-8-5.2-8-11a4.5 4.5 0 0 1 8-2.8A4.5 4.5 0 0 1 20 10c0 5.8-8 11-8 11z" /></svg>
  {/if}
</div>

<style>
  .eyes {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    height: 12px;
    opacity: 1;
    transition:
      opacity 400ms ease-out,
      transform 300ms ease-out;
  }
  .away {
    opacity: 0;
  }
  .looking {
    transform: translate(var(--look), 1px);
  }
  i {
    box-sizing: border-box;
    width: 7px;
    height: 11px;
    border-radius: 4px;
    background: var(--notch-fg);
    transition:
      height 900ms ease-in-out,
      opacity 900ms ease-in-out;
  }
  .awake i {
    animation: blink 4s ease-in-out 1s 3;
  }
  .glancing {
    animation: glance 1.2s ease-in-out;
  }
  .asleep i {
    height: 2px;
    opacity: 0.55;
  }
  .yawn i {
    height: 5px;
  }
  .mouth {
    position: absolute;
    left: 50%;
    top: 12px;
    width: 7px;
    height: 6px;
    margin-left: -3.5px;
    box-sizing: border-box;
    border: 1.5px solid var(--notch-fg);
    border-radius: 50%;
    animation: mouth 1.6s ease-in-out forwards;
  }
  .z {
    position: absolute;
    right: -12px;
    top: -6px;
    font-size: 9px;
    font-weight: 700;
    line-height: 1;
    color: var(--notch-fg);
    opacity: 0;
    animation: z 1.8s ease-out 0.6s forwards;
  }
  .z.two {
    right: -18px;
    animation-delay: 0.9s;
  }
  .startle i {
    animation: wide 900ms ease-out;
  }
  .bang {
    position: absolute;
    right: -12px;
    top: -6px;
    font-size: 10px;
    font-weight: 800;
    line-height: 1;
    color: var(--accent-focus);
    animation: fadeout 1.2s ease-out forwards;
  }
  .wave,
  .happy {
    animation: hop 380ms ease-out 2;
  }
  .wave i,
  .happy i {
    width: 9px;
    height: 6px;
    background: none;
    border: 2px solid var(--notch-fg);
    border-bottom: 0;
    border-radius: 6px 6px 0 0;
    transition: none;
  }
  .heart {
    position: absolute;
    left: 50%;
    top: -2px;
    width: 10px;
    height: 10px;
    margin-left: -5px;
    fill: var(--accent-alert);
    opacity: 0;
    animation: heart 1.8s ease-out forwards;
  }
  @keyframes blink {
    0%,
    90%,
    100% {
      transform: scaleY(1);
    }
    95% {
      transform: scaleY(0.1);
    }
  }
  @keyframes hop {
    50% {
      transform: translateY(-2px);
    }
  }
  @keyframes glance {
    25%,
    40% {
      transform: translateX(-4px);
    }
    60%,
    75% {
      transform: translateX(4px);
    }
  }
  @keyframes mouth {
    0% {
      transform: scale(0);
    }
    30%,
    70% {
      transform: scale(1);
    }
    100% {
      transform: scale(0);
    }
  }
  @keyframes z {
    0% {
      opacity: 0;
      transform: translate(0, 0) scale(0.6);
    }
    25% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: translate(6px, -12px) scale(1);
    }
  }
  @keyframes wide {
    40% {
      transform: scale(1.35);
    }
  }
  @keyframes fadeout {
    0% {
      opacity: 0;
      transform: translateY(3px);
    }
    20%,
    60% {
      opacity: 1;
      transform: translateY(0);
    }
    100% {
      opacity: 0;
    }
  }
  @keyframes heart {
    0% {
      opacity: 0;
      transform: translateY(4px) scale(0.4);
    }
    25% {
      opacity: 1;
      transform: translateY(-4px) scale(1);
    }
    100% {
      opacity: 0;
      transform: translateY(-16px) scale(1);
    }
  }
  :global(.reduce-motion) .eyes,
  :global(.reduce-motion) .eyes :is(i, span, svg) {
    animation: none;
    transition: none;
  }
  :global(.reduce-motion) .eyes :is(.mouth, .z, .bang, .heart) {
    display: none;
  }
</style>
