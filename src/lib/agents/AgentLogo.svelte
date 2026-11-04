<script module lang="ts">
  import antigravity from "./antigravity.svg?raw";
  import claude from "./claude.svg?raw";
  import codex from "./codex.svg?raw";
  import hermes from "./hermes.svg?raw";
  import opencode from "./opencode.svg?raw";

  export const AGENTS = {
    claude: { name: "Claude Code", svg: claude },
    codex: { name: "Codex", svg: codex },
    antigravity: { name: "Antigravity", svg: antigravity },
    hermes: { name: "Hermes Agent", svg: hermes },
    opencode: { name: "opencode", svg: opencode },
  } as const;
  export type AgentId = keyof typeof AGENTS;
</script>

<script lang="ts">
  let { agent, size = 18 }: { agent: AgentId; size?: number } = $props();
</script>

<span class="logo" style:--size="{size}px" aria-hidden="true">{@html AGENTS[agent].svg}</span>

<style>
  .logo {
    display: inline-grid;
    flex: none;
  }
  .logo :global(svg) {
    width: var(--size);
    height: var(--size);
  }
</style>
