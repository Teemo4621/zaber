<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { invoke } from "@tauri-apps/api/core";
  import Battery from "$lib/Battery.svelte";
  import Icon, { type IconName } from "$lib/Icon.svelte";
  import { locale, t } from "$lib/i18n.svelte";
  import { settings } from "$lib/settings.svelte";
  import { setDone, snooze } from "$lib/modules/todo/todo.svelte";
  import { drink } from "$lib/modules/water/water.svelte";
  import AgentLogo from "$lib/agents/AgentLogo.svelte";
  import { claude, heading } from "$lib/modules/claude/claude.svelte";
  import AppIcon from "$lib/modules/notify/AppIcon.svelte";
  import { addFiles } from "$lib/modules/shelf/shelf.svelte";
  import { bytes } from "$lib/modules/files/files.svelte";
  import { dismissPeek, levelOf, type Copied, type DeviceKind, type Peek } from "./peeks.svelte";

  let { peek }: { peek: Peek } = $props();

  const DEVICE_ICON: Record<DeviceKind, IconName> = { drive: "drive", mouse: "mouse", keyboard: "keyboard", headphones: "headphones" };
  const COPY_ICON: Record<Copied["kind"], IconName> = { text: "copy", link: "link", image: "image", files: "file" };
  function copiedText(c: Copied) {
    if (c.kind === "text") return `“${c.text}”`;
    if (c.kind === "link") return c.text;
    if (c.kind === "image") return t("clip.image", { w: c.width, h: c.height });
    return c.count > 1 ? t("clip.files", { name: c.name, n: c.count - 1 }) : c.name;
  }

  let fileIcon = $state<string | null>(null);
  $effect(() => {
    fileIcon = null;
    if (peek.kind !== "file") return;
    const path = peek.file.path;
    invoke<string | null>("files_icon", { path }).then((src) => peek.kind === "file" && peek.file.path === path && (fileIcon = src), () => {});
  });
  const openNew = (path: string, reveal: boolean) => (invoke("files_open", { path, reveal }).catch(() => {}), dismissPeek());

  const language = (tag: string) => {
    try {
      return new Intl.DisplayNames([tag], { type: "language" }).of(tag.split("-")[0]) ?? tag;
    } catch {
      return tag;
    }
  };
</script>

<div class="peek">
  {#if peek.kind === "volume"}
    <Icon name={peek.muted ? "mute" : "volume"} size={16} />
    <span class="track"><span style:transform="scaleX({peek.muted ? 0 : peek.level / 100})"></span></span>
    <span class="num">{peek.muted ? 0 : peek.level}</span>
  {:else if peek.kind === "caps"}
    <span class="what"><Icon name="caps" size={16} />{t("peek.caps")}</span>
    <span class="hud">{peek.on ? t("peek.on") : t("peek.off")}</span>
  {:else if peek.kind === "layout"}
    <span class="what"><Icon name="globe" size={16} />{language(peek.tag)}</span>
    <span class="chip">{(peek.tag.split("-")[1] ?? peek.tag).toUpperCase()}</span>
  {:else if peek.kind === "power"}
    <span class="what"><Battery level={peek.battery ?? 100} charging />{t("peek.charging")}</span>
    {#if peek.battery !== null}<span class="num" style:color="var(--accent-media)">{peek.battery}%</span>{/if}
  {:else if peek.kind === "network"}
    <span class="what"><span style:color="var(--accent-alert)"><Icon name="wifiOff" size={16} /></span>{t("peek.offline")}</span>
    <span class="dim">{t("peek.offlineShort")}</span>
  {:else if peek.kind === "audio"}
    {@const level = levelOf(peek.name)}
    <span class="what grow"><Icon name={level !== null || /head|ear|buds|airpods|หูฟัง/i.test(peek.name) ? "headphones" : "volume"} size={16} /><span class="ellipsis">{peek.name}</span></span>
    {#if level !== null}
      <span class="what"><Battery {level} charging={false} /><span class="num" style:color={level <= 20 ? "var(--accent-alert)" : undefined}>{level}%</span></span>
    {/if}
  {:else if peek.kind === "water"}
    <span class="what"><span style:color="var(--accent-calm)"><Icon name="water" size={16} /></span>{t("water.reminder")}</span>
    <button class="btn solid small" style:background="var(--accent-calm)" onclick={() => drink()}>+{settings.value.water.bottleMl} ml</button>
  {:else if peek.kind === "todo"}
    <span class="what grow"><span style:color="var(--accent-focus)"><Icon name="bell" size={16} /></span><span class="ellipsis">{peek.title} · {t("todo.dueNow")}</span></span>
    <button class="btn small" onclick={() => snooze(peek.id)}>{t("todo.snooze")}</button>
    <button class="btn solid small" style:background="var(--accent-media)" onclick={() => setDone(peek.id, true)}>{t("todo.done")}</button>
  {:else if peek.kind === "calendar"}
    <i style:background={peek.color}></i>
    <span class="what grow"><span class="ellipsis">{peek.title} · {t("cal.inMin", { n: peek.minutes })}</span></span>
    <button class="btn icon small" aria-label={t("cal.dismiss")} onclick={() => dismissPeek()}><Icon name="x" size={14} /></button>
    {#if peek.link}
      <button class="btn solid small" style:background="var(--accent-media)" onclick={() => (openUrl(peek.link!), dismissPeek())}>{t("cal.join")}</button>
    {/if}
  {:else if peek.kind === "focus"}
    <span class="what" style:color={peek.phase === "break" ? "var(--accent-calm)" : "var(--accent-focus)"}>
      <Icon name="timer" size={16} /><span style:color="var(--notch-fg)">{peek.phase === "break" ? t("focus.breakTime") : t("focus.backToWork")}</span>
    </span>
    <span class="dim">{new Date().toLocaleTimeString(locale(), { hour: "2-digit", minute: "2-digit" })}</span>
  {:else if peek.kind === "battery"}
    <span class="what"><Battery level={peek.level} charging={false} />{peek.level <= 10 ? t("peek.plugIn") : t("peek.lowBattery")}</span>
    <span class="num" style:color="var(--accent-alert)">{peek.level}%{peek.minutes ? ` · ${t("peek.minLeft", { n: peek.minutes })}` : ""}</span>
  {:else if peek.kind === "brightness"}
    <span style:color="var(--accent-bright)"><Icon name="sun" size={16} /></span>
    <span class="track"><span class="bright" style:transform="scaleX({peek.level / 100})"></span></span>
    <span class="num">{peek.level}</span>
  {:else if peek.kind === "clipboard"}
    <span class="what grow"><span style:color="var(--accent-media)"><Icon name={COPY_ICON[peek.copied.kind]} size={16} /></span><span class="ellipsis">{copiedText(peek.copied)}</span></span>
    <span class="dim">{t("clip.copied")}</span>
  {:else if peek.kind === "device"}
    <span class="what grow">
      <span style:color={peek.connected ? "var(--accent-calm)" : "var(--notch-fg-dim)"}><Icon name={DEVICE_ICON[peek.device]} size={16} /></span>
      <span class="ellipsis">{peek.name}{peek.detail ? ` · ${peek.detail}` : ""}</span>
    </span>
    {#if peek.connected && peek.path}
      <button class="btn small" onclick={() => (invoke("devices_open", { path: peek.path }), dismissPeek())}>{t("dev.open")}</button>
    {:else}
      <span class="dim">{peek.connected ? t("dev.connected") : t("dev.removed")}</span>
    {/if}
  {:else if peek.kind === "deviceBattery"}
    <span class="what grow"><Icon name={DEVICE_ICON[peek.device]} size={16} /><span class="ellipsis">{peek.name}</span></span>
    <span class="what"><Battery level={peek.level} charging={false} /><span class="num" style:color="var(--accent-alert)">{peek.level}%</span></span>
  {:else if peek.kind === "mic"}
    <span class="what">
      <span style:color={peek.muted ? "var(--accent-alert)" : undefined}><Icon name={peek.muted ? "micOff" : "mic"} size={16} /></span>
      {peek.muted ? t("peek.micOff") : t("peek.micOn")}
    </span>
    {#if peek.muted}<span class="chip">Ctrl+Alt+M</span>{/if}
  {:else if peek.kind === "privacy"}
    <span class="what">
      <i class="led" style:background={peek.device === "mic" ? "var(--accent-focus)" : "var(--accent-media)"}></i>
      {peek.device === "mic" ? t("peek.micOn") : t("peek.camOn")}
    </span>
    <span class="dim app ellipsis">{peek.app}</span>
  {:else if peek.kind === "file"}
    {@const f = peek.file}
    <span class="what grow">
      {#if fileIcon}<img class="thumb" class:shot={f.source === "screenshot"} src={fileIcon} alt="" />{:else}<Icon name={f.source === "screenshot" ? "image" : "file"} size={16} />{/if}
      <span class="ellipsis">{f.source === "screenshot" ? t("files.screenshot") : f.name}</span>
      <span class="dim">{f.source === "screenshot" && f.size ? `${f.size[0]} × ${f.size[1]}` : bytes(f.bytes)}</span>
    </span>
    <button class="btn small" onclick={() => openNew(f.path, false)}>{t("files.open")}</button>
    {#if f.source === "screenshot"}
      <button class="btn solid small" style:background="var(--accent-focus)" onclick={() => (addFiles([f.path]), dismissPeek())}>{t("files.keep")}</button>
    {:else}
      <button class="btn small" onclick={() => openNew(f.path, true)}>{t("files.show")}</button>
    {/if}
  {:else if peek.kind === "claude" && claude.queue[0]}
    {@const r = claude.queue[0]}
    <span class="cc"><AgentLogo agent="claude" /></span>
    <span class="note">
      <b class="ellipsis">{heading(r)}</b>
      <span class="ellipsis dim">{r.kind === "permission" ? r.summary : `Claude Code · ${r.project}`}</span>
    </span>
    {#if peek.count > 1}<span class="count">+{peek.count - 1}</span>{/if}
  {:else if peek.kind === "claudeNote"}
    <span class="what grow">
      <span style:color={peek.tone === "no" ? "var(--accent-alert)" : peek.tone === "ok" ? "var(--accent-media)" : undefined}>
        {#if peek.tone === "ok" || peek.tone === "no"}<Icon name={peek.tone === "ok" ? "check" : "x"} size={16} />{:else}<AgentLogo agent="claude" size={16} />{/if}
      </span>
      <span class="ellipsis">{peek.title}</span>
    </span>
    {#if peek.sub}<span class="dim app ellipsis">{peek.sub}</span>{/if}
  {:else if peek.kind === "notification"}
    <AppIcon appId={peek.appId} size={24} />
    <span class="note">
      <b class="ellipsis">{peek.title || peek.app}</b>
      {#if peek.body}<span class="ellipsis dim">{peek.body}</span>{/if}
    </span>
    {#if peek.title && peek.app}<span class="dim app ellipsis">{peek.app}</span>{/if}
  {/if}
</div>

<style>
  .led {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }
  .thumb {
    width: 22px;
    height: 22px;
    object-fit: contain;
    flex: none;
  }
  .thumb.shot {
    width: 34px;
    object-fit: cover;
    border-radius: 4px;
  }
  .peek {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    padding: 0 8px 0 14px;
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
  .grow {
    flex: 1;
  }
  .ellipsis {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .track {
    flex: 1;
    height: 5px;
    border-radius: 3px;
    background: #ffffff26;
  }
  .track span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--accent-hud);
    transform-origin: left;
    transition: transform 120ms ease-out;
  }
  .track .bright {
    background: var(--accent-bright);
  }
  :global(.glow) .track .bright {
    box-shadow:
      0 0 3px color-mix(in srgb, var(--accent-bright) 90%, transparent),
      0 0 12px color-mix(in srgb, var(--accent-bright) 60%, transparent);
  }
  :global(.glow) .track span {
    box-shadow: 0 0 3px color-mix(in srgb, var(--accent-hud) 90%, transparent), 0 0 12px color-mix(in srgb, var(--accent-hud) 60%, transparent);
  }
  .num {
    min-width: 26px;
    text-align: right;
    font-size: 12px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
    margin-right: 6px;
  }
  .hud {
    color: var(--accent-hud);
    margin-right: 6px;
  }
  .dim {
    color: var(--notch-fg-dim);
    font-weight: 500;
    margin-right: 6px;
  }
  .chip {
    padding: 2px 8px;
    border-radius: 6px;
    background: #ffffff1f;
    font-size: 12px;
    margin-right: 6px;
  }
  i {
    width: 4px;
    height: 18px;
    border-radius: 2px;
    flex: none;
  }
  .note {
    flex: 1;
    min-width: 0;
    display: grid;
    line-height: 1.3;
  }
  .note .dim {
    font-size: 12px;
    margin: 0;
  }
  .app {
    max-width: 30%;
    font-size: 12px;
  }
  .cc {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    flex: none;
    background: var(--notch-surface);
  }
  .count {
    height: 22px;
    min-width: 22px;
    padding: 0 7px;
    box-sizing: border-box;
    border-radius: 11px;
    background: var(--accent-focus);
    color: #000;
    font-size: 12px;
    font-weight: 700;
    display: grid;
    place-items: center;
    margin-right: 6px;
  }
  .small {
    height: 28px;
    font-size: 12px;
  }
</style>
