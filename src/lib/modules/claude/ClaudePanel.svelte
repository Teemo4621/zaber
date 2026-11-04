<script lang="ts">
  import AgentLogo from "$lib/agents/AgentLogo.svelte";
  import { t } from "$lib/i18n.svelte";
  import { answer, claude, heading, type Request } from "./claude.svelte";

  let { request, keys = false }: { request: Request; keys?: boolean } = $props();

  let values = $state<Record<string, string | number | boolean>>({});
  $effect(() => {
    request.id;
    values = Object.fromEntries(request.fields.filter((f) => f.kind === "boolean").map((f) => [f.name, false]));
  });
  const ready = $derived(request.fields.every((f) => !f.required || (values[f.name] !== undefined && values[f.name] !== "")));
  const more = $derived(claude.queue.length - 1);
</script>

<div class="ask">
  <div class="head">
    <span class="badge"><AgentLogo agent="claude" /></span>
    <span class="who">
      <b>{heading(request)}</b>
      <span>Claude Code · {request.project}{more > 0 ? ` · ${t("claude.more", { n: more })}` : ""}</span>
    </span>
  </div>

  {#if request.kind === "permission"}
    <pre class="code">{request.summary}</pre>
    <div class="actions">
      <button class="btn ghost" onclick={() => answer(request, { kind: "pass" })}>{t("claude.close")}</button>
      <button class="btn" onclick={() => answer(request, { kind: "deny" })}>{t("claude.deny")}{#if keys}<kbd>Esc</kbd>{/if}</button>
      {#if request.canAlways}
        <button class="btn" onclick={() => answer(request, { kind: "allow", always: true })}>{t("claude.always")}{#if keys}<kbd>A</kbd>{/if}</button>
      {/if}
      <button class="btn solid" style:background="var(--accent-focus)" onclick={() => answer(request, { kind: "allow", always: false })}>{t("claude.allowOnce")}{#if keys}<kbd>Enter</kbd>{/if}</button>
    </div>
  {:else}
    <div class="fields">
      {#each request.fields as f (f.name)}
        {#if f.kind === "enum"}
          <div class="opts" role="radiogroup" aria-label={f.title}>
            {#each f.options as o, i (o)}
              <button class="opt" class:on={values[f.name] === o} role="radio" aria-checked={values[f.name] === o} onclick={() => (values[f.name] = o)}>
                <span class="k">{i + 1}</span>{o}
              </button>
            {/each}
          </div>
        {:else if f.kind === "boolean"}
          <label class="check"><input type="checkbox" bind:checked={values[f.name] as boolean} />{f.title}</label>
        {:else}
          <input
            class="field"
            type={f.kind === "number" ? "number" : "text"}
            placeholder={f.title}
            aria-label={f.title}
            oninput={(e) => (values[f.name] = f.kind === "number" ? +e.currentTarget.value : e.currentTarget.value)}
          />
        {/if}
      {/each}
    </div>
    <div class="actions">
      <button class="btn ghost" onclick={() => answer(request, { kind: "pass" })}>{t("claude.close")}</button>
      <button class="btn" onclick={() => answer(request, { kind: "decline" })}>{t("claude.decline")}{#if keys}<kbd>Esc</kbd>{/if}</button>
      <button class="btn solid" style:background="var(--accent-focus)" disabled={!ready} onclick={() => answer(request, { kind: "accept", content: values })}>{t("claude.answer")}</button>
    </div>
  {/if}
</div>

<style>
  kbd {
    font: inherit;
    font-size: 10px;
    margin-left: 6px;
    padding: 0 5px;
    border-radius: 4px;
    border: 1px solid currentColor;
    opacity: 0.7;
  }
  .ask {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .badge {
    width: 30px;
    height: 30px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    flex: none;
    background: var(--notch-surface);
  }
  .who {
    display: grid;
    min-width: 0;
    line-height: 1.3;
  }
  .who b,
  .who span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .who span {
    font-size: 12px;
    color: var(--notch-fg-dim);
  }
  .code {
    margin: 0;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 10px 12px;
    border-radius: 12px;
    background: var(--notch-surface);
    font: 12px "Cascadia Mono", Consolas, ui-monospace, monospace;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .fields {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: grid;
    gap: 6px;
    align-content: start;
  }
  .opts {
    display: grid;
    gap: 6px;
  }
  .opt {
    height: 36px;
    border: 1px solid #ffffff1f;
    border-radius: 12px;
    background: var(--notch-surface);
    padding: 0 12px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    transition: background-color 150ms ease-out;
  }
  .opt:hover {
    background: #2c2c2e;
  }
  .opt.on {
    border-color: var(--accent-focus);
    background: color-mix(in srgb, var(--accent-focus) 14%, transparent);
  }
  .k {
    font-size: 11px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
  .field {
    height: 36px;
    border-radius: 18px;
    border: 1px solid #ffffff26;
    background: var(--notch-surface);
    color: var(--notch-fg);
    font: inherit;
    font-size: 13px;
    padding: 0 14px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .actions .btn {
    height: 32px;
  }
  .actions .ghost {
    margin-right: auto;
    background: none;
    color: var(--notch-fg-dim);
  }
  .actions .ghost:hover {
    color: var(--notch-fg);
  }
</style>
