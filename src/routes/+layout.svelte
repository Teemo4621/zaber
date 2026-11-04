<script lang="ts">
  import "$lib/theme.css";
  import { applyTheme } from "$lib/theme.svelte";

  let { children } = $props();
  applyTheme();
</script>

{@render children()}
