<script lang="ts">
  let { progress, size, stroke, color }: { progress: number; size: number; stroke: number; color: string } = $props();
  const r = $derived((size - stroke) / 2);
  const c = $derived(2 * Math.PI * r);
</script>

<svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true" style="transform: rotate(-90deg); flex: none">
  <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke={color} stroke-opacity="0.25" stroke-width={stroke} />
  <circle
    cx={size / 2}
    cy={size / 2}
    {r}
    fill="none"
    stroke={color}
    stroke-width={stroke}
    stroke-linecap="round"
    stroke-dasharray={c}
    stroke-dashoffset={c * (1 - Math.min(1, Math.max(0, progress)))}
    style:--ring={color}
  />
</svg>

<style>
  :global(.glow) circle:last-child {
    filter: drop-shadow(0 0 2px var(--ring)) drop-shadow(0 0 6px color-mix(in srgb, var(--ring) 70%, transparent));
  }
</style>
