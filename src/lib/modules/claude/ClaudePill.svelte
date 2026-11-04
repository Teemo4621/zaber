<script lang="ts">
  import AgentLogo from "$lib/agents/AgentLogo.svelte";
  import { t } from "$lib/i18n.svelte";
  import { busy } from "./claude.svelte";

  const list = $derived(busy());
  const head = $derived(list.find((s) => s.state === "waiting") ?? list[0]);

  let now = $state(Date.now());
  $effect(() => {
    if (head?.state !== "working") return;
    const id = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(id);
  });
  const elapsed = $derived.by(() => {
    const s = Math.max(0, Math.floor((now - (head?.since ?? now)) / 1000));
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  });
</script>

{#if head}
  <div class="pill">
    <span class="what">
      {#key head.detail}<span class="logo"><AgentLogo agent="claude" size={16} /></span>{/key}
      <span class="ellipsis">{head.project}</span>
      {#if list.length > 1}<span class="count">{list.length}</span>{/if}
    </span>
    {#if head.state === "waiting"}
      <span class="state" style:color="var(--accent-focus)">{t("claude.needsYou")}</span>
    {:else}
      <span class="state">{t("claude.working")} · {elapsed}</span>
    {/if}
  </div>
{/if}

<style>
  .pill {
    width: 100%;
    padding: 0 12px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: var(--pill-text);
    font-weight: 600;
  }
  .what {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .ellipsis {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .logo {
    display: grid;
    animation: breathe 700ms ease-in-out;
  }
  @keyframes breathe {
    50% {
      transform: scale(0.82);
      opacity: 0.7;
    }
  }
  :global(.reduce-motion) .logo {
    animation: none;
  }
  .count {
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    box-sizing: border-box;
    border-radius: 8px;
    background: var(--notch-surface);
    font-size: 11px;
    display: grid;
    place-items: center;
  }
  .state {
    flex: none;
    font-size: 12px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
</style>
