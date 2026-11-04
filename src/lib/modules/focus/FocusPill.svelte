<script lang="ts">
  import FocusRing from "./FocusRing.svelte";
  import { clockText, focus, remainingMs, totalMs } from "./focus.svelte";

  const left = $derived(remainingMs());
  const color = $derived(focus.phase === "focus" ? "var(--accent-focus)" : "var(--accent-calm)");
  const text = $derived(clockText(left));
</script>

<div class="pill">
  <FocusRing progress={1 - left / totalMs()} size={18} stroke={3} {color} />
  <span style:color>{text}</span>
</div>

<style>
  .pill {
    width: 100%;
    padding: 0 14px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  span {
    font-size: var(--pill-text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
</style>
