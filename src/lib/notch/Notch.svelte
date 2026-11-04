<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Spring } from "svelte/motion";
  import { fade } from "svelte/transition";
  import { onDestroy } from "svelte";
  import Battery from "$lib/Battery.svelte";
  import Icon, { type IconName } from "$lib/Icon.svelte";
  import { locale, t, type TKey } from "$lib/i18n.svelte";
  import { NOTCH_SCALE, settings, updateSettings, type TabId } from "$lib/settings.svelte";
  import { status } from "$lib/status.svelte";
  import { reduceMotion } from "$lib/theme.svelte";
  import CalendarView from "$lib/modules/calendar/CalendarView.svelte";
  import FocusPill from "$lib/modules/focus/FocusPill.svelte";
  import FocusView from "$lib/modules/focus/FocusView.svelte";
  import { focus, running as focusRunning } from "$lib/modules/focus/focus.svelte";
  import MediaPill from "$lib/modules/media/MediaPill.svelte";
  import MediaView from "$lib/modules/media/MediaView.svelte";
  import Aura from "$lib/modules/media/Aura.svelte";
  import { media, mediaUi } from "$lib/modules/media/media.svelte";
  import ShelfView from "$lib/modules/shelf/ShelfView.svelte";
  import { addFiles, shelfUi } from "$lib/modules/shelf/shelf.svelte";
  import TodoView from "$lib/modules/todo/TodoView.svelte";
  import NotesView from "$lib/modules/notes/NotesView.svelte";
  import WaterView from "$lib/modules/water/WaterView.svelte";
  import "$lib/modules/notify/notify.svelte";
  import AppIcon from "$lib/modules/notify/AppIcon.svelte";
  import ClaudePanel from "$lib/modules/claude/ClaudePanel.svelte";
  import ClaudePill from "$lib/modules/claude/ClaudePill.svelte";
  import DownloadPill from "$lib/modules/files/DownloadPill.svelte";
  import { bytes, files } from "$lib/modules/files/files.svelte";
  import ClaudeSessions from "$lib/modules/claude/ClaudeSessions.svelte";
  import AgentLogo from "$lib/agents/AgentLogo.svelte";
  import { answer, busy, claude, prune } from "$lib/modules/claude/claude.svelte";
  import Companion, { wakeCompanion } from "./Companion.svelte";
  import { cubicOut } from "svelte/easing";
  import { untrack } from "svelte";
  import { feel, pop, strainAt, stretch } from "./nudge.svelte";
  import Peek from "./Peek.svelte";
  import { deviceLevels, dismissPeek, peeks, type Peek as PeekT } from "./peeks.svelte";

  type Tab = TabId | "agents";
  const TABS: Record<Tab, { icon: IconName; label: TKey }> = {
    agents: { icon: "sparkle", label: "claude.sessions" },
    notes: { icon: "pencil", label: "tab.notes" },
    media: { icon: "music", label: "tab.media" },
    calendar: { icon: "calendar", label: "tab.calendar" },
    focus: { icon: "timer", label: "tab.focus" },
    todo: { icon: "todo", label: "tab.todo" },
    water: { icon: "water", label: "tab.water" },
    shelf: { icon: "shelf", label: "tab.shelf" },
  };
  const IDLE = { w: 180, h: 32 };
  const PILL = { w: 260, h: 32 };
  const EXPANDED = { w: 480, h: 220 };
  const LYRICS = { w: 640, h: 240 };
  const PEEK_W: Record<PeekT["kind"], number> = {
    volume: 300, caps: 300, layout: 300, power: 300, network: 300, audio: 380, focus: 300, water: 320, todo: 360, calendar: 380,
    notification: 400, battery: 320, brightness: 300, clipboard: 340, device: 340, deviceBattery: 340,
    claude: 420, claudeNote: 360, mic: 300, privacy: 340, file: 420,
  };
  const ACTION_PEEKS: PeekT["kind"][] = ["water", "todo", "calendar", "device", "file"];
  const AGENT_PILL = { w: 300, h: 32 };
  const DOWNLOAD_PILL = { w: 340, h: 32 };
  const LIVE_LYRIC = { w: 360, h: 56 };
  const MESSAGE = { w: 440, h: 168 };
  const DOT = { w: 10, h: 10 };
  const WELCOME = { w: 420, h: 170 };

  let hovering = $state(false);
  let expanded = $state(false);
  let tab = $state<Tab>("media");
  let dir = 0;
  function go(id: Tab) {
    dir = Math.sign(tabs.indexOf(id) - tabs.indexOf(tab));
    tab = id;
  }
  let message = $state<Extract<PeekT, { kind: "notification" }> | null>(null);
  let asking = $state(false);
  const request = $derived(asking ? (claude.queue[0] ?? null) : null);
  let intro = $state<"dot" | "pill" | "eyes" | "welcome" | null>("dot");
  const scale = $derived(NOTCH_SCALE[settings.value.notchSize] ?? 1);

  const tabs = $derived<Tab[]>([...settings.value.tabs.filter((x) => x.on).map((x) => x.id), ...(claude.sessions.length || claude.usage ? (["agents"] as const) : [])]);
  let keys = $state(false);
  const agents = $derived(busy().length);
  const headphones = $derived.by(() => {
    const out = status.now?.audioDevice ?? "";
    const list = deviceLevels.list.filter((l) => l.kind === "headphones");
    return list.find((l) => out.includes(l.name)) ?? null;
  });
  const agentsWaiting = $derived(busy().some((s) => s.state === "waiting"));
  const privacy = $derived({ mic: !!status.now?.micApps.length, cam: !!status.now?.cameraApps.length });
  const playing = $derived(media.now?.playing ?? false);
  const timing = $derived(focus.phase !== "ready" && focusRunning());
  const collapsed = $derived.by(() => {
    if (peeks.current) return "peek";
    const order = settings.value.pillPriority === "focus" ? (["focus", "media"] as const) : (["media", "focus"] as const);
    return order.find((k) => (k === "focus" ? timing : playing)) ?? (agents ? "agents" : files.downloading.length ? "download" : "idle");
  });

  const pillBusy = $derived(!expanded && !message && !request && !intro && (collapsed === "media" || collapsed === "focus" || collapsed === "agents"));
  const target = $derived.by(() => {
    if (intro) return intro === "dot" ? DOT : intro === "welcome" ? WELCOME : IDLE;
    if (message) return MESSAGE;
    if (request) return { w: 480, h: request.kind === "permission" ? 196 : Math.min(300, 140 + request.fields.reduce((h, f) => h + (f.kind === "enum" ? f.options.length * 42 : 42), 0)) };
    if (expanded) return tab === "media" && mediaUi.lyrics && media.now ? LYRICS : EXPANDED;
    if (collapsed === "peek" && peeks.current) {
      const p = peeks.current;
      return { w: PEEK_W[p.kind], h: p.kind === "notification" || p.kind === "claude" ? 52 : ACTION_PEEKS.includes(p.kind) ? 40 : 36 };
    }
    if (collapsed === "media" && mediaUi.lyrics && mediaUi.line) return LIVE_LYRIC;
    return collapsed === "idle" ? IDLE : collapsed === "agents" ? AGENT_PILL : collapsed === "download" ? DOWNLOAD_PILL : PILL;
  });
  const OPEN = feel(0.42, 0.8);
  const CLOSE = feel(0.45, 1);
  const even = (n: number) => 2 * Math.round(n / 2);
  const size = Spring.of(() => target, OPEN);
  $effect(() => {
    const t = target;
    const grows = untrack(() => t.w * t.h >= size.current.w * size.current.h);
    Object.assign(size, grows ? OPEN : CLOSE);
  });
  const box = $derived.by(() => {
    if (reduceMotion()) return { ...target, x: 0 };
    const s = stretch.current;
    return { w: even(size.current.w + s.w), h: Math.round(size.current.h + s.h), x: Math.round(s.x) };
  });
  const inner = $derived(reduceMotion() ? target : { w: even(target.w + stretch.current.w), h: Math.round(target.h + stretch.current.h) });
  const radius = $derived(Math.min(28, 14 + Math.max(0, box.h - 40) * 0.1));
  const ear = $derived(Math.round(Math.min(14, 7 + Math.max(0, box.h - 40) * 0.04)));
  const aura = $derived(
    expanded && !intro && !message && !request && tab === "media" && media.now && settings.value.followArtwork ? mediaUi.artColor : null,
  );
  const fadeMs = (ms: number) => (reduceMotion() ? 0 : ms);

  function reveal(_: Element, { delay = 0, duration = 260 }) {
    return {
      delay: fadeMs(delay),
      duration: fadeMs(duration),
      easing: cubicOut,
      css: (t: number, u: number) => `opacity:${t};transform:translateY(${u * -6}px) scale(${1 - u * 0.03})`,
    };
  }

  function bubble(_: Element) {
    return {
      duration: fadeMs(260),
      easing: cubicOut,
      css: (t: number, u: number) => `opacity:${t};transform:translateX(${u * -10}px) scale(${0.6 + t * 0.4})`,
    };
  }

  function slide(_: Element, { out = false } = {}) {
    const d = out ? -dir : dir;
    return {
      duration: fadeMs(out ? 140 : 240),
      easing: cubicOut,
      css: (t: number, u: number) => `opacity:${t};transform:translateX(${u * d * 28}px)`,
    };
  }

  $effect(() => {
    const [w, h] = [target.w * scale, target.h * scale];
    invoke("notch_set_hit_rect", { rect: { x: (innerWidth - w) / 2, y: 0, w, h } });
  });

  $effect(() => {
    invoke("notch_set_display", { mode: settings.value.display });
  });

  $effect(() => {
    invoke("claude_set_wait", { secs: settings.value.claudeWait });
  });

  $effect(() => {
    invoke("status_volume_keys", { on: settings.value.volumeKeys && settings.value.peeks.volume });
  });

  let hidden = false;
  $effect(() => {
    const hide = settings.value.hideFullscreen && (status.now?.fullscreen ?? false);
    if (hide === hidden) return;
    hidden = hide;
    const win = getCurrentWindow();
    (hide ? win.hide() : win.show()).catch(() => {});
  });

  function open() {
    if (claude.queue.length && peeks.current?.kind !== "notification") {
      asking = true;
      return;
    }
    if (peeks.current?.kind === "notification") {
      message = peeks.current;
      return;
    }
    prune();
    if (!tabs.length) return;
    if (collapsed === "agents" && busy().some((s) => s.state === "waiting")) {
      tab = "agents";
      expanded = true;
      return;
    }
    const s = settings.value.openTo;
    const last = settings.value.lastTab;
    if (s === "first") tab = tabs[0];
    else if (s === "last") tab = last;
    else tab = timing && tabs.includes("focus") ? "focus" : playing && tabs.includes("media") ? "media" : last;
    if (!tabs.includes(tab)) tab = tabs[0];
    expanded = true;
  }

  $effect(() => {
    const t = tab;
    if (expanded && t !== untrack(() => settings.value.lastTab)) updateSettings({ lastTab: t });
  });

  let openTimer: ReturnType<typeof setTimeout> | undefined;
  const unlistenHover = listen<boolean>("notch:hover", (e) => {
    hovering = e.payload;
    clearTimeout(openTimer);
    if (keys) return;
    if (!hovering) {
      expanded = false;
      if (message) closeMessage();
      asking = false;
      if (intro === "welcome") endIntro();
      return;
    }
    if (intro === "welcome") return;
    if (intro) endIntro();
    openTimer = setTimeout(open, settings.value.openDelay);
  });

  const unlistenDrop = getCurrentWebview().onDragDropEvent((e) => {
    const p = e.payload;
    if (p.type === "enter" || p.type === "over") {
      if (!shelfUi.dragging && tabs.includes("shelf")) {
        shelfUi.dragging = true;
        tab = "shelf";
        clearTimeout(openTimer);
        expanded = true;
      }
    } else {
      shelfUi.dragging = false;
      if (p.type === "drop" && tabs.includes("shelf")) {
        addFiles(p.paths);
        pop("l");
      }
    }
  });
  const unlistenKeys = listen("notch:keys", () => {
    clearTimeout(openTimer);
    if (keys) return closeKeys();
    if (intro) endIntro();
    keys = true;
    open();
  });
  function closeKeys(refocus = true) {
    if (!keys) return;
    keys = false;
    expanded = false;
    asking = false;
    if (message) closeMessage();
    if (refocus) invoke("notch_release_keys");
  }
  const unlistenFocus = getCurrentWindow().onFocusChanged(({ payload }) => {
    if (!payload) closeKeys(false);
  });
  $effect(() => {
    if (keys && asking && !request) closeKeys();
  });
  function onKey(e: KeyboardEvent) {
    if (!keys) return;
    const typing = (e.target as HTMLElement).closest("input, textarea, select");
    if (e.key === "Escape") {
      e.preventDefault();
      if (request) answer(request, { kind: request.kind === "permission" ? "deny" : "decline" });
      else closeKeys();
      return;
    }
    if (typing || e.repeat) return;
    if (request?.kind === "permission") {
      if (e.key === "Enter") answer(request, { kind: "allow", always: false });
      else if (e.key.toLowerCase() === "a" && request.canAlways) answer(request, { kind: "allow", always: true });
    } else if (expanded && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      step(e.key === "ArrowLeft" ? -1 : 1);
    }
  }

  function closeMessage() {
    message = null;
    dismissPeek("notification");
  }
  function openMessageApp() {
    if (message?.appId) invoke("notify_open", { appId: message.appId }).catch(() => {});
    closeMessage();
  }

  let introTimers: ReturnType<typeof setTimeout>[] = [];
  function playIntro(welcome: boolean) {
    endIntro();
    if (reduceMotion()) {
      if (welcome) intro = "welcome";
      return;
    }
    const at = (ms: number, step: () => void) => introTimers.push(setTimeout(step, ms));
    intro = "dot";
    size.set(DOT, { instant: true });
    at(350, () => (intro = "pill"));
    at(1000, () => (intro = "eyes"));
    at(2000, () => (welcome ? (intro = "welcome") : endIntro()));
    if (welcome) at(8000, endIntro);
  }
  function endIntro() {
    introTimers.forEach(clearTimeout);
    introTimers = [];
    if (intro === "welcome" && !settings.value.welcomed) updateSettings({ welcomed: true });
    if (intro === "eyes" || intro === "welcome") wakeCompanion();
    intro = null;
  }
  let launch = $state<string>();
  invoke<string>("app_launch").then((k) => (launch = k), () => (launch = "login"));
  let introPlayed = false;
  $effect(() => {
    if (introPlayed || !launch || !settings.loaded) return;
    introPlayed = true;
    untrack(() => {
      if (launch === "manual") return playIntro(!settings.value.welcomed);
      endIntro();
      size.set(target, { instant: true });
    });
  });
  const unlistenWelcome = listen("notch:welcome", () => playIntro(true));

  onDestroy(() => {
    clearTimeout(openTimer);
    endIntro();
    unlistenWelcome.then((f) => f());
    unlistenHover.then((f) => f());
    unlistenDrop.then((f) => f());
    unlistenKeys.then((f) => f());
    unlistenFocus.then((f) => f());
  });

  let wheel = 0;
  let lastSwitch = 0;
  let lastWheel = 0;
  function onWheel(e: WheelEvent) {
    if (!expanded || tabs.length < 2) return;
    const scroller = (e.target as HTMLElement).closest("ul, .files");
    if (scroller && scroller.scrollHeight > scroller.clientHeight + 1 && Math.abs(e.deltaY) > Math.abs(e.deltaX)) return;
    e.preventDefault();
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    const now = performance.now();
    if (now - lastSwitch < 250) {
      wheel = 0;
      lastWheel = now;
      return;
    }
    if (now - lastWheel > 200 || Math.sign(delta) !== Math.sign(wheel)) wheel = 0;
    lastWheel = now;
    wheel += delta;
    if (Math.abs(wheel) < 40) return;
    const d = Math.sign(wheel);
    wheel = 0;
    lastSwitch = now;
    step(d);
  }
  function step(d: number) {
    const i = tabs.indexOf(tab) + d;
    if (i >= 0 && i < tabs.length) {
      go(tabs[i]);
    } else strainAt(i < 0 ? -1 : 1, "x");
  }

  let el = $state<HTMLDivElement>();
  $effect(() => {
    el?.addEventListener("wheel", onWheel, { passive: false });
    return () => el?.removeEventListener("wheel", onWheel);
  });

  const clock = () => new Date().toLocaleTimeString(locale(), { hour: "2-digit", minute: "2-digit" });
</script>

<svelte:window onkeydown={onKey} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="shell"
  style:width="{box.w}px"
  style:--ear="{ear}px"
  style:zoom={scale === 1 ? undefined : scale}
  style:transform={box.x ? `translateX(${box.x}px)` : undefined}
>
<span class="ear left" aria-hidden="true"></span>
<span class="ear right" aria-hidden="true"></span>
<span class="sides">
  {#if agents && pillBusy && collapsed !== "agents"}
    <span class="side" class:wait={agentsWaiting} transition:bubble aria-label="Claude Code · {agents}">
      <AgentLogo agent="claude" size={14} /><b>{agents}</b>
    </span>
  {/if}
  {#if files.downloading.length && pillBusy}
    <span class="side" transition:bubble aria-label={files.downloading[0].name}>
      <span class="dl"><Icon name="download" size={13} /></span>{bytes(files.downloading[0].speed)}/s
    </span>
  {/if}
</span>
<div
  class="notch"
  style:height="{box.h}px"
  style:--radius="{radius}px"
  bind:this={el}
  oncontextmenu={(e) => (e.preventDefault(), invoke("settings_open"))}
>
  {#if aura}<Aura color={aura} />{/if}
  {#if !expanded && !message && !request && !intro && collapsed !== "peek" && (privacy.mic || privacy.cam)}
    <span class="privacy" aria-hidden="true">
      {#if privacy.mic}<i style:background="var(--accent-focus)"></i>{/if}
      {#if privacy.cam}<i style:background="var(--accent-media)"></i>{/if}
    </span>
  {/if}
  <!-- svelte-ignore block_empty -->
  {#if intro === "welcome"}
    <div class="layer welcome" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 80, duration: 280 }} out:fade={{ duration: fadeMs(90) }}>
      <Companion awake />
      <strong>{t("welcome.title")}</strong>
      <span class="hints">
        <kbd>{t("welcome.hover")}</kbd>{t("welcome.hoverHint")}
        <kbd>{t("welcome.scroll")}</kbd>{t("welcome.scrollHint")}
        <kbd>{t("welcome.right")}</kbd>{t("welcome.rightHint")}
      </span>
      <button class="btn solid" style:background="var(--notch-fg)" onclick={endIntro}>{t("welcome.ok")}</button>
    </div>
  {:else if intro && intro !== "eyes"}
  {:else if request}
    <div class="body" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 80, duration: 280 }} out:fade={{ duration: fadeMs(90) }}>
      <ClaudePanel {request} {keys} />
    </div>
  {:else if message}
    <div class="body message" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 80, duration: 280 }} out:fade={{ duration: fadeMs(90) }}>
      <div class="msg-head">
        <AppIcon appId={message.appId} size={30} />
        <span class="msg-who"><b>{message.title || message.app}</b>{#if message.title && message.app}<span>{message.app}</span>{/if}</span>
      </div>
      {#if message.body}<p class="msg-body">{message.body}</p>{/if}
      <div class="msg-actions">
        <button class="btn" onclick={closeMessage}>{t("msg.dismiss")}</button>
        {#if message.appId}
          <button class="btn solid" style:background="var(--notch-fg)" onclick={openMessageApp}>
            <Icon name="external" size={14} />{message.app ? t("msg.open", { app: message.app }) : t("msg.openApp")}
          </button>
        {/if}
      </div>
    </div>
  {:else if expanded}
    <div
      class="body"
      style:width="{inner.w}px"
      style:height="{inner.h}px"
      in:reveal={{ delay: 80, duration: 280 }}
      out:fade={{ duration: fadeMs(90) }}
    >
      <div class="tabrow">
        <div class="tabs" role="tablist">
          {#if tabs.includes(tab)}
            <span class="tab-on" aria-hidden="true" style:transform="translateX({tabs.indexOf(tab) * 32}px)" class:still={reduceMotion()}></span>
          {/if}
          {#each tabs as id (id)}
            <button class="tab" class:on={tab === id} role="tab" aria-selected={tab === id} aria-label={t(TABS[id].label)} onclick={() => go(id)}>
              {#if id === "agents"}<span class="agent-tab" class:busy={agents}><AgentLogo agent="claude" size={16} /></span>{:else}<Icon name={TABS[id].icon} />{/if}
            </button>
          {/each}
        </div>
        <span class="meta">
          {#if keys}<span class="keys"><kbd>← →</kbd>{t("keys.tabs")}<kbd>Esc</kbd>{t("keys.close")}</span>{:else}
          {#if headphones}
            <span class="battery" title="{headphones.name} · {headphones.level}%">
              <Icon name="headphones" size={14} /><span style:color={headphones.level <= 20 ? "var(--accent-alert)" : undefined}>{headphones.level}%</span>
            </span>
          {/if}
          {#if status.now?.battery != null}
            <span class="battery" title="{status.now.battery}%">
              <Battery level={status.now.battery} charging={status.now.charging ?? false} />{status.now.battery}%
            </span>
          {/if}
          <span>{clock()}</span>
          {/if}
        </span>
      </div>
      <div class="page" role="tabpanel">
        {#key tab}
        <div class="pane" in:slide out:slide={{ out: true }}>
        {#if tab === "media"}
          {#if media.now}
            <MediaView now={media.now} />
          {:else}
            <div class="empty">
              <Icon name="music" size={26} />
              <strong>{t("media.nothing")}</strong>
              <span>{t("media.nothingHint")}</span>
            </div>
          {/if}
        {:else if tab === "calendar"}
          <CalendarView />
        {:else if tab === "focus"}
          <FocusView />
        {:else if tab === "todo"}
          <TodoView />
        {:else if tab === "notes"}
          <NotesView />
        {:else if tab === "agents"}
          <ClaudeSessions />
        {:else if tab === "water"}
          <WaterView />
        {:else if tab === "shelf"}
          <ShelfView />
        {/if}
        </div>
        {/key}
      </div>
    </div>
  {:else if collapsed === "peek" && peeks.current}
    <div class="layer" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 60, duration: 220 }} out:fade={{ duration: fadeMs(80) }}><Peek peek={peeks.current} /></div>
  {:else if collapsed === "focus"}
    <div class="layer" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 60, duration: 220 }} out:fade={{ duration: fadeMs(80) }}><FocusPill /></div>
  {:else if collapsed === "media" && media.now}
    <div class="layer" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 60, duration: 220 }} out:fade={{ duration: fadeMs(80) }}><MediaPill now={media.now} /></div>
  {:else if collapsed === "agents"}
    <div class="layer" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 60, duration: 220 }} out:fade={{ duration: fadeMs(80) }}><ClaudePill /></div>
  {:else if collapsed === "download"}
    <div class="layer" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 60, duration: 220 }} out:fade={{ duration: fadeMs(80) }}><DownloadPill /></div>
  {:else if intro === "eyes" || settings.value.companion}
    <div class="layer" style:width="{inner.w}px" style:height="{inner.h}px" in:reveal={{ delay: 60, duration: 220 }} out:fade={{ duration: fadeMs(80) }}><Companion awake={intro === "eyes"} /></div>
  {/if}
</div>
</div>

<style>
  .shell {
    position: relative;
    margin: 0 auto;
  }
  .sides {
    position: absolute;
    top: 3px;
    left: calc(100% + var(--ear) + 6px);
    display: flex;
    gap: 6px;
    pointer-events: none;
  }
  .side {
    height: 26px;
    padding: 0 10px 0 8px;
    box-sizing: border-box;
    border-radius: 13px;
    background: var(--notch-bg);
    border: var(--notch-border);
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    color: var(--notch-fg);
    transform-origin: left center;
    white-space: nowrap;
  }
  .side.wait b {
    color: var(--accent-focus);
  }
  .dl {
    display: grid;
    color: var(--accent-calm);
  }
  .ear {
    position: absolute;
    top: 0;
    width: var(--ear);
    height: var(--ear);
    pointer-events: none;
  }
  .ear.left {
    right: 100%;
    background: radial-gradient(circle at 0 100%, transparent calc(var(--ear) - 0.5px), var(--notch-bg) var(--ear));
  }
  .ear.right {
    left: 100%;
    background: radial-gradient(circle at 100% 100%, transparent calc(var(--ear) - 0.5px), var(--notch-bg) var(--ear));
  }
  .notch {
    position: relative;
    isolation: isolate;
    width: 100%;
    background: var(--notch-bg);
    border: var(--notch-border);
    border-top: 0;
    box-sizing: border-box;
    border-radius: 0 0 var(--radius) var(--radius);
    overflow: hidden;
    display: grid;
    place-items: center;
  }
  .notch > * {
    grid-area: 1 / 1;
  }
  .layer {
    flex: none;
    transform-origin: top center;
    display: grid;
    place-items: center;
  }
  .body {
    flex: none;
    transform-origin: top center;
    align-self: start;
    padding: 12px 20px 16px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .welcome {
    flex-direction: column;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    align-self: start;
  }
  .welcome strong {
    font-size: 17px;
  }
  .hints {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--notch-fg-dim);
  }
  .hints kbd {
    font: inherit;
    font-size: 11px;
    color: var(--notch-fg);
    padding: 2px 7px;
    border-radius: 6px;
    background: var(--notch-surface);
    border: 1px solid #ffffff26;
  }
  .hints kbd:not(:first-child) {
    margin-left: 6px;
  }
  .message {
    gap: 10px;
  }
  .msg-head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .msg-who {
    display: grid;
    min-width: 0;
    line-height: 1.3;
  }
  .msg-who b,
  .msg-who span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .msg-who span {
    font-size: 12px;
    color: var(--notch-fg-dim);
  }
  .msg-body {
    margin: 0;
    flex: 1;
    min-height: 0;
    font-size: 13px;
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .msg-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .msg-actions .btn {
    height: 32px;
  }
  .tabrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .tabs {
    position: relative;
    display: flex;
    gap: 2px;
  }
  .tab-on {
    position: absolute;
    left: 0;
    top: 0;
    width: 30px;
    height: 32px;
    border-radius: 10px;
    background: var(--notch-hover);
    transition: transform 320ms cubic-bezier(0.34, 1.3, 0.64, 1);
    pointer-events: none;
  }
  .privacy {
    position: absolute;
    right: 5px;
    top: 50%;
    transform: translateY(-50%);
    display: grid;
    gap: 3px;
    pointer-events: none;
  }
  .privacy i {
    width: 5px;
    height: 5px;
    border-radius: 50%;
  }
  .agent-tab {
    display: grid;
    opacity: 0.6;
  }
  .tab.on .agent-tab,
  .agent-tab.busy {
    opacity: 1;
  }
  .keys {
    display: flex;
    align-items: center;
    gap: 2px;
    font-size: 11px;
  }
  .keys kbd {
    font: inherit;
    color: var(--notch-fg);
    padding: 1px 6px;
    border-radius: 5px;
    background: var(--notch-surface);
    border: 1px solid #ffffff26;
  }
  .keys kbd:not(:first-child) {
    margin-left: 4px;
  }
  .tab-on.still {
    transition: none;
  }
  .tab {
    position: relative;
    width: 30px;
    height: 32px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    color: var(--notch-fg-dim);
    transition:
      background-color 150ms ease-out,
      color 150ms ease-out;
  }
  .tab:hover {
    background: #ffffff14;
  }
  .tab.on {
    color: var(--notch-fg);
  }
  .tab.on:hover {
    background: none;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--notch-fg-dim);
    font-variant-numeric: tabular-nums;
  }
  .battery {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--notch-fg-dim);
  }
  .page {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template: minmax(0, 1fr) / minmax(0, 1fr);
  }
  .pane {
    grid-area: 1 / 1;
    min-width: 0;
    min-height: 0;
  }
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    color: var(--notch-fg-dim);
    font-size: 13px;
  }
  .empty strong {
    color: var(--notch-fg);
    font-size: 15px;
  }
</style>
