<script lang="ts">
  import { fly } from "svelte/transition";
  import Icon from "$lib/Icon.svelte";
  import { reduceMotion } from "$lib/theme.svelte";
  import { t } from "$lib/i18n.svelte";
  import { addTask, removeTask, renameTask, setDone, visibleTasks } from "./todo.svelte";

  let title = $state("");
  let time = $state("");
  let editing = $state<string | null>(null);

  function submit(e: SubmitEvent) {
    e.preventDefault();
    addTask(title, time || null);
    title = "";
    time = "";
    picking = false;
  }

  const tasks = $derived(visibleTasks());
  const now = () => new Date().toTimeString().slice(0, 5);
  let justDone = $state<string>();

  let picking = $state(false);
  const pad = (n: number) => String(n).padStart(2, "0");
  const hm = (d: Date) => `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  function soon() {
    const d = new Date(Date.now() + 5 * 60_000);
    d.setMinutes(d.getMinutes() - (d.getMinutes() % 5));
    return hm(d);
  }
  function openPicker() {
    if (!time) time = soon();
    picking = !picking;
  }
  function step(min: number) {
    const [h, m] = (time || soon()).split(":").map(Number);
    const total = (((h * 60 + m + min) % 1440) + 1440) % 1440;
    time = `${pad(Math.floor(total / 60))}:${pad(total % 60)}`;
  }
  function inMinutes(min: number) {
    time = hm(new Date(Date.now() + min * 60_000));
    picking = false;
  }
</script>

<div class="todo">
  <form onsubmit={submit}>
    <input class="field" bind:value={title} placeholder={t("todo.add")} aria-label={t("todo.newTask")} maxlength="200" />
    <button type="button" class="when" class:set={time} class:open={picking} aria-label={t("todo.dueTime")} aria-expanded={picking} onclick={openPicker}>
      <Icon name="clock" size={14} />{time || t("todo.time")}
    </button>
    <button class="btn icon" type="submit" aria-label={t("todo.newTask")}><Icon name="plus" size={16} /></button>
  </form>
  {#if picking}
    <div class="picker" transition:fly={{ y: -6, duration: reduceMotion() ? 0 : 160 }}>
      <div class="spin" role="group" aria-label={t("todo.dueTime")}>
        <button type="button" class="ibtn" aria-label={t("todo.earlier")} onclick={() => step(-60)}><Icon name="down" size={14} /></button>
        <b>{time.slice(0, 2)}</b>
        <button type="button" class="ibtn" aria-label={t("todo.later")} onclick={() => step(60)}><Icon name="up" size={14} /></button>
        <span class="colon">:</span>
        <button type="button" class="ibtn" aria-label={t("todo.earlier")} onclick={() => step(-5)}><Icon name="down" size={14} /></button>
        <b>{time.slice(3, 5)}</b>
        <button type="button" class="ibtn" aria-label={t("todo.later")} onclick={() => step(5)}><Icon name="up" size={14} /></button>
      </div>
      <div class="chips">
        <button type="button" class="chip" onclick={() => inMinutes(15)}>{t("todo.in", { n: 15 })}</button>
        <button type="button" class="chip" onclick={() => inMinutes(60)}>{t("todo.in", { n: 60 })}</button>
        <button type="button" class="chip" onclick={() => ((time = ""), (picking = false))}>{t("todo.noTime")}</button>
        <button type="button" class="chip ok" aria-label={t("todo.setTime")} onclick={() => (picking = false)}><Icon name="check" size={14} /></button>
      </div>
    </div>
  {/if}
  <ul>
    {#each tasks as task (task.id)}
      <li class:done={task.done}>
        <button
          class="check"
          class:on={task.done}
          class:pop={justDone === task.id}
          role="checkbox"
          aria-checked={task.done}
          aria-label={task.title}
          onclick={() => {
            justDone = task.done ? undefined : task.id;
            setDone(task.id, !task.done);
          }}
        >
          <Icon name="check" size={12} />
        </button>
        {#if editing === task.id}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="field edit"
            value={task.title}
            autofocus
            aria-label={t("todo.newTask")}
            onblur={(e) => (renameTask(task.id, e.currentTarget.value), (editing = null))}
            onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()}
          />
        {:else}
          <button class="title" ondblclick={() => (editing = task.id)}>{task.title}</button>
        {/if}
        {#if task.time}
          <span class="at" class:soon={!task.done && task.time <= now()}>{task.time}</span>
        {/if}
        <button class="del" aria-label={t("todo.delete", { title: task.title })} onclick={() => removeTask(task.id)}>
          <Icon name="x" size={14} />
        </button>
      </li>
    {:else}
      <li class="empty">{t("todo.empty")}</li>
    {/each}
  </ul>
</div>

<style>
  .todo {
    display: flex;
    flex-direction: column;
    gap: 6px;
    height: 100%;
    min-height: 0;
  }
  form {
    display: flex;
    gap: 6px;
  }
  .field {
    flex: 1;
    min-width: 0;
    height: 32px;
    box-sizing: border-box;
    border: 1px solid #ffffff22;
    border-radius: 10px;
    background: var(--notch-surface);
    color: var(--notch-fg);
    font: inherit;
    font-size: 13px;
    padding: 0 12px;
    color-scheme: dark;
  }
  .field:focus {
    outline: none;
    border-color: #ffffff4d;
  }
  .when {
    flex: none;
    height: 32px;
    padding: 0 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    border-radius: 10px;
    border: 1px solid #ffffff22;
    background: var(--notch-surface);
    color: var(--notch-fg-dim);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    transition:
      background-color 150ms ease-out,
      color 150ms ease-out;
  }
  .when:hover,
  .when.open {
    background: #2c2c2e;
  }
  .when.set {
    color: var(--accent-focus);
    border-color: color-mix(in srgb, var(--accent-focus) 40%, transparent);
  }
  .picker {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 12px;
    background: var(--notch-surface);
  }
  .spin {
    display: flex;
    align-items: center;
    gap: 2px;
    font-variant-numeric: tabular-nums;
  }
  .spin b {
    width: 24px;
    text-align: center;
    font-size: 16px;
  }
  .colon {
    color: var(--notch-fg-dim);
    margin: 0 2px;
  }
  .ibtn {
    width: 24px;
    height: 24px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    color: var(--notch-fg-dim);
  }
  .ibtn:hover {
    background: #ffffff14;
    color: var(--notch-fg);
  }
  .chips {
    display: flex;
    gap: 4px;
  }
  .chip {
    height: 26px;
    padding: 0 10px;
    border-radius: 13px;
    background: #ffffff14;
    font-size: 12px;
    display: grid;
    place-items: center;
    transition: background-color 150ms ease-out;
  }
  .chip:hover {
    background: #ffffff2e;
  }
  .chip.ok {
    width: 26px;
    padding: 0;
    color: #000;
    background: var(--accent-focus);
  }
  .field.edit {
    height: 26px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }
  li {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 32px;
    padding: 0 8px;
    border-radius: 10px;
    font-size: 13px;
  }
  li:hover {
    background: #ffffff14;
  }
  .check {
    flex: none;
    width: 18px;
    height: 18px;
    box-sizing: border-box;
    border-radius: 6px;
    border: 1.5px solid #ffffff59;
    display: grid;
    place-items: center;
    color: transparent;
    transition:
      background-color 150ms ease-out,
      border-color 150ms ease-out,
      color 150ms ease-out;
  }
  .check:hover {
    border-color: var(--accent-media);
  }
  .check.on {
    border-color: var(--accent-media);
    background: var(--accent-media);
    color: #000;
  }
  :global(.glow) .check.on {
    box-shadow:
      0 0 3px color-mix(in srgb, var(--accent-media) 90%, transparent),
      0 0 12px color-mix(in srgb, var(--accent-media) 60%, transparent);
  }
  .title {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  li.done {
    color: var(--notch-fg-dim);
  }
  .title {
    background: linear-gradient(currentColor, currentColor) left 55% / 0 1px no-repeat;
    transition: background-size 280ms ease-out;
  }
  li.done .title {
    background-size: 100% 1px;
  }
  .check.pop {
    animation: tick 320ms cubic-bezier(0.3, 1.6, 0.5, 1);
  }
  @keyframes tick {
    40% {
      transform: scale(1.35);
    }
  }
  :global(.reduce-motion) .check.pop {
    animation: none;
  }
  .at {
    font-size: 12px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
  .at.soon {
    color: var(--accent-focus);
  }
  .del {
    color: var(--notch-fg-dim);
    opacity: 0;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
  }
  li:hover .del,
  .del:focus-visible {
    opacity: 1;
  }
  .empty {
    color: var(--notch-fg-dim);
    justify-content: center;
  }
  .empty:hover {
    background: none;
  }
</style>
