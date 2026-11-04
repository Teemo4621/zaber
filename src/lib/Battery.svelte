<script lang="ts">
  let { level, charging }: { level: number; charging: boolean } = $props();
  const color = $derived(charging ? "var(--accent-media)" : level <= 20 ? "var(--accent-alert)" : "var(--notch-fg)");
</script>

<svg width="25" height="12" viewBox="0 0 25 12" aria-hidden="true">
  <rect x="0.75" y="0.75" width="20.5" height="10.5" rx="3" fill="none" stroke="currentColor" stroke-opacity="0.45" stroke-width="1.5" />
  <rect x="22.5" y="4" width="1.75" height="4" rx="0.9" fill="currentColor" fill-opacity="0.45" />
  <rect x="2.5" y="2.5" width={Math.max(1.5, (17 * level) / 100)} height="7" rx="1.6" fill={color} style:transition="width 300ms ease-out" />
  {#if charging}
    <path d="M12.2 1.6 8.4 6.6h2.8l-1.4 3.8 3.8-5H10.8z" fill="var(--notch-bg)" stroke="var(--notch-bg)" stroke-width="0.6" stroke-linejoin="round" />
    <path d="M12.2 1.6 8.4 6.6h2.8l-1.4 3.8 3.8-5H10.8z" fill="var(--notch-fg)" />
  {/if}
</svg>
