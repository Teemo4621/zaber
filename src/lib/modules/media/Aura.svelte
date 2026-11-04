<script lang="ts">
  import { fade } from "svelte/transition";
  import { reduceMotion } from "$lib/theme.svelte";

  let { color }: { color: string } = $props();

  const STARS = [
    [38, 18, 2], [52, 9, 1.5], [63, 26, 2], [71, 12, 1.5], [82, 20, 2.5], [91, 8, 1.5],
    [46, 82, 1.5], [58, 90, 2], [67, 76, 1.5], [78, 88, 2], [88, 70, 1.5], [95, 52, 2],
    [74, 46, 1.5],
  ];
</script>

{#key color}
  <div class="aura" class:still={reduceMotion()} style:--c={color} transition:fade={{ duration: reduceMotion() ? 0 : 400 }} aria-hidden="true">
    {#each STARS as [x, y, s], i (i)}
      <span style:left="{x}%" style:top="{y}%" style:width="{s}px" style:height="{s}px" style:animation-delay="{(i * 97) % 900}ms"></span>
    {/each}
  </div>
{/key}

<style>
  .aura {
    position: absolute;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    background: radial-gradient(ellipse 55% 75% at 12% 62%, color-mix(in srgb, var(--c) 20%, transparent), transparent 70%);
  }
  span {
    position: absolute;
    border-radius: 50%;
    background: color-mix(in srgb, var(--c) 45%, white);
    box-shadow: 0 0 4px color-mix(in srgb, var(--c) 70%, transparent);
    opacity: 0.55;
    animation: twinkle 1400ms ease-out 1 both;
  }
  .still span {
    animation: none;
  }
  @keyframes twinkle {
    0% {
      opacity: 0;
      transform: scale(0.4);
    }
    45% {
      opacity: 1;
      transform: scale(1.4);
    }
    100% {
      opacity: 0.55;
      transform: scale(1);
    }
  }
</style>
