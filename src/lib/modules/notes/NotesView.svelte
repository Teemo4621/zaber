<script lang="ts">
  import { t } from "$lib/i18n.svelte";
  import { notes, setNotes } from "./notes.svelte";

  let text = $state("");
  let typing = $state(false);
  $effect(() => {
    if (!typing) text = notes.value.text;
  });
</script>

<textarea
  class="note"
  bind:value={text}
  placeholder={t("notes.placeholder")}
  aria-label={t("tab.notes")}
  spellcheck="false"
  onfocus={() => (typing = true)}
  onblur={() => (setNotes(text, true), (typing = false))}
  oninput={() => setNotes(text)}
></textarea>

<style>
  .note {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    resize: none;
    margin: 0;
    padding: 10px 12px;
    border: 1px solid #ffffff22;
    border-radius: 12px;
    background: var(--notch-surface);
    color: var(--notch-fg);
    font: inherit;
    font-size: 13px;
    line-height: 1.5;
    color-scheme: dark;
  }
  .note:focus {
    outline: none;
    border-color: #ffffff4d;
  }
  .note::placeholder {
    color: var(--notch-fg-dim);
  }
</style>
