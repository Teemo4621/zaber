<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { t } from "$lib/i18n.svelte";
  import { clearShelf, dragOut, extension, icons, loadIcon, openFile, removeFile, shelf, shelfUi } from "./shelf.svelte";

  const COLORS: Record<string, string> = { PDF: "#ff6961", PNG: "#64d2ff", JPG: "#64d2ff", JPEG: "#64d2ff", XLSX: "#30d158", ZIP: "#ffd60a" };

  let press: { path: string; x: number; y: number } | null = null;
  function down(e: PointerEvent, path: string) {
    if (e.button === 0) press = { path, x: e.clientX, y: e.clientY };
  }
  function move(e: PointerEvent) {
    if (press && Math.hypot(e.clientX - press.x, e.clientY - press.y) > 5) {
      dragOut(press.path);
      press = null;
    }
  }
</script>

<svelte:window onpointerup={() => (press = null)} />

{#if shelfUi.dragging}
  <div class="drop">
    <Icon name="download" size={28} />
    <span>{t("shelf.drop")}</span>
  </div>
{:else if shelf.value.length === 0}
  <div class="drop idle">
    <Icon name="shelf" size={26} />
    <span>{t("shelf.empty")}</span>
  </div>
{:else}
  <div class="files" onpointermove={move} role="list">
    {#each shelf.value as item (item.path)}
      {@const ext = extension(item.name)}
      {@const icon = (loadIcon(item.path), icons[item.path])}
      <div class="item" role="listitem">
        <button
          class="file"
          title={item.path}
          aria-label={t("shelf.open", { name: item.name })}
          onpointerdown={(e) => down(e, item.path)}
          onclick={() => openFile(item.path)}
          oncontextmenu={(e) => (e.preventDefault(), e.stopPropagation(), openFile(item.path, true))}
        >
          {#if icon}
            <img class="icon" src={icon} alt="" draggable="false" />
          {:else}
            <span class="doc" style:background={COLORS[ext] ?? "#c7c7cc"}>{ext}</span>
          {/if}
          <span class="name">{item.name}</span>
        </button>
        <button class="remove" aria-label={t("shelf.remove", { name: item.name })} onclick={() => removeFile(item.path)}>
          <Icon name="x" size={12} />
        </button>
      </div>
    {/each}
    <button class="file clear" onclick={clearShelf}>
      <span class="doc ghost"><Icon name="trash" size={16} /></span>
      <span class="name">{t("shelf.clear")}</span>
    </button>
  </div>
{/if}

<style>
  .drop {
    height: 100%;
    box-sizing: border-box;
    border: 2px dashed var(--accent-media);
    border-radius: 18px;
    background: color-mix(in srgb, var(--accent-media) 8%, transparent);
    color: var(--accent-media);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
  }
  .drop.idle {
    border-color: #48484a;
    background: none;
    color: var(--notch-fg-dim);
    font-weight: 500;
    font-size: 13px;
  }
  .files {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
    overflow-x: auto;
  }
  .item {
    position: relative;
    flex: none;
  }
  .file {
    width: 76px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 6px 0;
    border-radius: 12px;
    cursor: grab;
  }
  .file:hover {
    background: #ffffff14;
  }
  .doc {
    width: 40px;
    height: 48px;
    border-radius: 6px;
    display: grid;
    place-items: end center;
    padding-bottom: 6px;
    box-sizing: border-box;
    font-size: 10px;
    font-weight: 700;
    color: #000;
  }
  .doc.ghost {
    border: 1.5px dashed #48484a;
    color: var(--notch-fg-dim);
    place-items: center;
    padding: 0;
  }
  .clear {
    cursor: pointer;
    color: var(--notch-fg-dim);
  }
  .name {
    font-size: 11px;
    max-width: 70px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .remove {
    position: absolute;
    top: 0;
    right: 4px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #3a3a3c;
    display: grid;
    place-items: center;
    opacity: 0;
  }
  .item:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }
  .icon {
    width: 44px;
    height: 44px;
    object-fit: contain;
    border-radius: 6px;
  }
</style>
