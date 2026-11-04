<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import AgentLogo, { AGENTS, type AgentId } from "$lib/agents/AgentLogo.svelte";
  import Icon, { type IconName } from "$lib/Icon.svelte";
  import { t, type TKey } from "$lib/i18n.svelte";
  import { DEFAULT_SETTINGS, NOTCH_SCALE, settings, updateSettings, type PeekKind, type Settings, type TabId } from "$lib/settings.svelte";
  import { status } from "$lib/status.svelte";

  type Section = "general" | "tabs" | "peeks" | "replies" | "appearance" | "about";
  let section = $state<Section>("general");
  let query = $state("");
  const s = $derived(settings.value);

  const NAV: { id: Section; group: TKey; icon: IconName; label: TKey; hint: TKey; keys: string }[] = [
    { id: "general", group: "set.group.notch", icon: "gear", label: "set.general", hint: "set.generalHint", keys: "login language display delay fullscreen companion data" },
    { id: "tabs", group: "set.group.notch", icon: "grid", label: "set.tabs", hint: "set.tabsHint", keys: "media calendar focus todo water shelf lyrics order pill" },
    { id: "peeks", group: "set.group.notch", icon: "bell", label: "set.peeks", hint: "set.peeksPageHint", keys: "volume brightness caps keyboard battery wifi headphones usb clipboard copy notification hud" },
    { id: "replies", group: "set.group.notch", icon: "message", label: "set.replies", hint: "set.repliesHint", keys: "message discord line chat notification claude ai approval hook" },
    { id: "appearance", group: "set.group.look", icon: "palette", label: "set.appearance", hint: "set.appearanceHint", keys: "style size text motion color accent glow artwork wave" },
    { id: "about", group: "set.group.zaber", icon: "info", label: "set.about", hint: "set.aboutHint", keys: "version privacy welcome reset" },
  ];
  const GROUPS: TKey[] = ["set.group.notch", "set.group.look", "set.group.zaber"];
  const visible = $derived(
    NAV.filter((n) => {
      const q = query.trim().toLowerCase();
      return !q || `${t(n.label)} ${n.keys} ${n.id}`.toLowerCase().includes(q);
    }),
  );
  const page = $derived(NAV.find((n) => n.id === section)!);

  const OTHER_AGENTS: AgentId[] = ["codex", "antigravity", "hermes", "opencode"];
  const FEATURES: Record<TabId, { icon: IconName }> = {
    media: { icon: "music" },
    calendar: { icon: "calendar" },
    focus: { icon: "timer" },
    todo: { icon: "todo" },
    notes: { icon: "pencil" },
    water: { icon: "water" },
    shelf: { icon: "shelf" },
  };
  const PEEK_GROUPS: { cap: TKey; items: { kind: PeekKind; icon: IconName }[] }[] = [
    {
      cap: "set.cap.system",
      items: [
        { kind: "volume", icon: "volume" },
        { kind: "brightness", icon: "sun" },
        { kind: "caps", icon: "caps" },
        { kind: "layout", icon: "globe" },
        { kind: "mic", icon: "micOff" },
        { kind: "privacy", icon: "camera" },
      ],
    },
    {
      cap: "set.cap.power",
      items: [
        { kind: "power", icon: "charge" },
        { kind: "battery", icon: "batteryLow" },
        { kind: "deviceBattery", icon: "headphones" },
        { kind: "devices", icon: "drive" },
        { kind: "audio", icon: "headphones" },
        { kind: "network", icon: "wifiOff" },
      ],
    },
    {
      cap: "set.cap.apps",
      items: [
        { kind: "clipboard", icon: "copy" },
        { kind: "files", icon: "download" },
        { kind: "notification", icon: "bell" },
      ],
    },
  ];
  const PRESETS: [string, TKey][] = [
    ["#30d158", "color.green"], ["#64d2ff", "color.sky"], ["#0a84ff", "color.blue"], ["#bf5af2", "color.purple"],
    ["#ff375f", "color.pink"], ["#ff9f0a", "color.orange"], ["#ffd60a", "color.yellow"], ["#f5f5f7", "color.white"],
  ];
  const SIZES = ["compact", "standard", "large"] as const;

  let notifyAccess = $state<string>();
  invoke<string>("notify_access").then((a) => (notifyAccess = a)).catch(() => {});

  let claudeOn = $state(false);
  let claudeError = $state("");
  invoke<boolean>("claude_hook_status").then((on) => (claudeOn = on)).catch(() => {});
  async function setClaude(on: boolean) {
    claudeError = "";
    try {
      claudeOn = await invoke<boolean>("claude_hook", { on });
    } catch (e) {
      claudeError = String(e);
    }
  }

  let version = $state("");
  getVersion().then((v) => (version = v)).catch(() => {});

  let autostart = $state(false);
  isEnabled().then((on) => (autostart = on)).catch(() => {});
  async function setAutostart(on: boolean) {
    try {
      await (on ? enable() : disable());
      autostart = on;
    } catch {
      autostart = !on;
    }
  }

  let uninstall = $state<"idle" | "confirm" | string>("idle");
  async function removeApp() {
    if (uninstall !== "confirm") return void (uninstall = "confirm");
    try {
      await invoke("app_uninstall");
    } catch (e) {
      uninstall = String(e);
    }
  }

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => updateSettings({ [key]: value } as Partial<Settings>);

  function moveTab(i: number, d: number) {
    const tabs = [...s.tabs];
    [tabs[i], tabs[i + d]] = [tabs[i + d], tabs[i]];
    set("tabs", tabs);
  }
  const toggleTab = (id: TabId, on: boolean) => set("tabs", s.tabs.map((x) => (x.id === id ? { ...x, on } : x)));

  function resetAppearance() {
    const d = DEFAULT_SETTINGS;
    updateSettings({ notchStyle: d.notchStyle, notchSize: d.notchSize, textSize: d.textSize, motion: d.motion, accents: d.accents, useSystemAccent: false, followArtwork: false, glow: d.glow });
  }

  let confirmReset = $state(false);
  function resetAll() {
    if (!confirmReset) {
      confirmReset = true;
      setTimeout(() => (confirmReset = false), 3000);
      return;
    }
    confirmReset = false;
    settings.save({ ...DEFAULT_SETTINGS, welcomed: s.welcomed });
  }
</script>

<svelte:head><title>Zaber — {t("set.title")}</title></svelte:head>

{#snippet badge(icon: IconName)}
  <span class="rb"><Icon name={icon} size={18} /></span>
{/snippet}

<div class="app">
  <nav aria-label={t("set.title")}>
    <div class="brand">
      <span class="mini"><i></i><i></i></span>
      <span class="brand-text"><strong>Zaber</strong><span>{t("set.title")}</span></span>
    </div>
    <label class="search">
      <Icon name="search" size={15} />
      <input type="search" bind:value={query} placeholder={t("set.search")} aria-label={t("set.search")} />
    </label>
    {#each GROUPS as g (g)}
      {@const items = visible.filter((n) => n.group === g)}
      {#if items.length}
        <p class="grp">{t(g)}</p>
        {#each items as n (n.id)}
          <button class="nav" class:on={section === n.id} aria-current={section === n.id ? "page" : undefined} onclick={() => (section = n.id)}>
            <span class="nb"><Icon name={n.icon} size={15} /></span>
            <span>{t(n.label)}</span>
          </button>
        {/each}
      {/if}
    {/each}
    {#if visible.length === 0}
      <p class="none">{t("set.noMatch", { q: query })}</p>
    {/if}
  </nav>

  <main>
    <header>
      <span class="nb big"><Icon name={page.icon} size={24} /></span>
      <div class="ht">
        <h1>{t(page.label)}</h1>
        <p>{t(page.hint)}</p>
      </div>
      {#if section === "appearance"}<button class="sbtn" onclick={resetAppearance}>{t("set.reset")}</button>{/if}
    </header>

    {#if !settings.loaded}
      <p class="hint">…</p>
    {:else if section === "general"}
      <p class="cap">{t("set.cap.startup")}</p>
      <div class="card">
        <div class="row">
          {@render badge("power")}
          <div class="rt"><b>{t("set.login")}</b><span>{t("set.loginHint")}</span></div>
          <input class="switch" type="checkbox" checked={autostart} onchange={(e) => setAutostart(e.currentTarget.checked)} aria-label={t("set.login")} />
        </div>
        <div class="row">
          {@render badge("language")}
          <div class="rt"><b>{t("set.language")}</b><span>{t("set.languageHint")}</span></div>
          <div class="seg" role="radiogroup" aria-label={t("set.language")}>
            {#each [["en", "English"], ["th", "ไทย"]] as [id, label] (id)}
              <label><input type="radio" name="lang" checked={s.language === id} onchange={() => set("language", id as Settings["language"])} />{label}</label>
            {/each}
          </div>
        </div>
        <div class="row">
          {@render badge("monitor")}
          <div class="rt"><b>{t("set.display")}</b><span>{t("set.displayHint")}</span></div>
          <select value={s.display} onchange={(e) => set("display", e.currentTarget.value as Settings["display"])} aria-label={t("set.display")}>
            <option value="main">{t("set.displayMain")}</option>
            <option value="cursor">{t("set.displayCursor")}</option>
          </select>
        </div>
      </div>
      <p class="cap">{t("set.cap.behavior")}</p>
      <div class="card">
        <div class="row">
          {@render badge("pointer")}
          <div class="rt"><b>{t("set.delay")}</b><span>{t("set.delayHint")}</span></div>
          <div class="range">
            <input type="range" min="0" max="600" step="50" value={s.openDelay} onchange={(e) => set("openDelay", +e.currentTarget.value)} aria-label={t("set.delay")} />
            <span class="pill">{s.openDelay} ms</span>
          </div>
        </div>
        <div class="row">
          {@render badge("fullscreen")}
          <div class="rt"><b>{t("set.fullscreen")}</b><span>{t("set.fullscreenHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.hideFullscreen} onchange={(e) => set("hideFullscreen", e.currentTarget.checked)} aria-label={t("set.fullscreen")} />
        </div>
        <div class="row">
          {@render badge("eye")}
          <div class="rt"><b>{t("set.companion")}</b><span>{t("set.companionHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.companion} onchange={(e) => set("companion", e.currentTarget.checked)} aria-label={t("set.companion")} />
        </div>
      </div>
      <p class="cap">{t("set.cap.privacy")}</p>
      <div class="card">
        <div class="row">
          {@render badge("shield")}
          <div class="rt"><b>{t("set.data")}</b><span>{t("set.dataHint")}</span></div>
          <button class="sbtn" onclick={() => invoke("store_open_dir")}><Icon name="folder" size={15} />{t("set.dataOpen")}</button>
        </div>
        <div class="row">
          {@render badge("trash")}
          <div class="rt">
            <b>{t("set.uninstall")}</b>
            <span>{uninstall === "idle" || uninstall === "confirm" ? t("set.uninstallHint") : uninstall}</span>
          </div>
          <button class="sbtn" class:danger={uninstall === "confirm"} onclick={removeApp} onblur={() => uninstall === "confirm" && (uninstall = "idle")}>
            {uninstall === "confirm" ? t("set.uninstallConfirm") : t("set.uninstallOpen")}
          </button>
        </div>
      </div>
    {:else if section === "tabs"}
      <p class="cap">{t("set.cap.order")}</p>
      <div class="card">
        {#each s.tabs as tab, i (tab.id)}
          {@const f = FEATURES[tab.id]}
          {@const name = t(`tab.${tab.id}` as TKey)}
          <div class="row">
            <span class="pos">{i + 1}</span>
            {@render badge(f.icon)}
            <div class="rt"><b>{name}</b></div>
            <div class="updown">
              <button class="ibtn" aria-label={t("set.moveUp", { name })} disabled={i === 0} onclick={() => moveTab(i, -1)}><Icon name="up" size={15} /></button>
              <button class="ibtn" aria-label={t("set.moveDown", { name })} disabled={i === s.tabs.length - 1} onclick={() => moveTab(i, 1)}><Icon name="down" size={15} /></button>
            </div>
            <input class="switch" type="checkbox" checked={tab.on} onchange={(e) => toggleTab(tab.id, e.currentTarget.checked)} aria-label={t("set.showTab", { name })} />
          </div>
        {/each}
      </div>
      <p class="cap">{t("set.cap.open")}</p>
      <div class="card">
        <div class="row">
          {@render badge("door")}
          <div class="rt"><b>{t("set.openTo")}</b><span>{t("set.openToHint")}</span></div>
          <select value={s.openTo} onchange={(e) => set("openTo", e.currentTarget.value as Settings["openTo"])} aria-label={t("set.openTo")}>
            <option value="live">{t("set.openLive")}</option>
            <option value="last">{t("set.openLast")}</option>
            <option value="first">{t("set.openFirst")}</option>
          </select>
        </div>
        <div class="row">
          {@render badge("layers")}
          <div class="rt"><b>{t("set.pill")}</b><span>{t("set.pillHint")}</span></div>
          <select value={s.pillPriority} onchange={(e) => set("pillPriority", e.currentTarget.value as Settings["pillPriority"])} aria-label={t("set.pill")}>
            <option value="focus">{t("set.pillFocus")}</option>
            <option value="media">{t("set.pillMedia")}</option>
          </select>
        </div>
        <div class="row">
          {@render badge("lyrics")}
          <div class="rt"><b>{t("set.lyrics")}</b><span>{t("set.lyricsHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.lyrics} onchange={(e) => set("lyrics", e.currentTarget.checked)} aria-label={t("set.lyrics")} />
        </div>
        <div class="row">
          {@render badge("calendar")}
          <div class="rt"><b>{t("set.headsUp")}</b><span>{t("set.headsUpHint")}</span></div>
          <select value={s.calendarHeadsUp} onchange={(e) => set("calendarHeadsUp", +e.currentTarget.value)} aria-label={t("set.headsUp")}>
            {#each [1, 5, 10, 15] as n (n)}<option value={n}>{t("set.minBefore", { n })}</option>{/each}
            <option value={0}>{t("set.off")}</option>
          </select>
        </div>
      </div>
    {:else if section === "peeks"}
      <p class="cap">{t("set.cap.timing")}</p>
      <div class="card">
        <div class="row">
          {@render badge("clock")}
          <div class="rt"><b>{t("set.showFor")}</b><span>{t("set.peekTimeHint")}</span></div>
          <div class="seg" role="radiogroup" aria-label={t("set.showFor")}>
            {#each [2, 4, 6] as n (n)}
              <label><input type="radio" name="peek-time" checked={s.peekSeconds === n} onchange={() => set("peekSeconds", n)} />{n} s</label>
            {/each}
          </div>
        </div>
        {#if s.peeks.volume}
          <div class="row">
            {@render badge("volume")}
            <div class="rt"><b>{t("set.volumeKeys")}</b><span>{t("set.volumeKeysHint")}</span></div>
            <input class="switch" type="checkbox" checked={s.volumeKeys} onchange={(e) => set("volumeKeys", e.currentTarget.checked)} aria-label={t("set.volumeKeys")} />
          </div>
        {/if}
      </div>
      {#each PEEK_GROUPS as g (g.cap)}
        <p class="cap">{t(g.cap)}</p>
        <div class="card">
          {#each g.items as p (p.kind)}
            {@const label = t(`set.peek.${p.kind}` as TKey)}
            <div class="row small">
              {@render badge(p.icon)}
              <div class="rt"><b class="light">{label}</b><span>{t(`set.peekHint.${p.kind}` as TKey)}</span></div>
              <input class="switch" type="checkbox" checked={s.peeks[p.kind]} onchange={(e) => set("peeks", { ...s.peeks, [p.kind]: e.currentTarget.checked })} aria-label={label} />
            </div>
          {/each}
        </div>
      {/each}
    {:else if section === "replies"}
      <p class="cap">{t("set.cap.messages")}</p>
      <div class="card">
        <div class="row">
          {@render badge("message")}
          <div class="rt"><b>{t("set.messages")}</b><span>{t("set.messagesHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.peeks.notification} onchange={(e) => set("peeks", { ...s.peeks, notification: e.currentTarget.checked })} aria-label={t("set.messages")} />
        </div>
        {#if s.peeks.notification && notifyAccess && notifyAccess !== "allowed"}
          <div class="row small">
            {@render badge("info")}
            <div class="rt"><span>{notifyAccess === "denied" ? t("set.notifyBlocked") : t("set.notifyUnsupported")}</span></div>
            {#if notifyAccess === "denied"}
              <button class="sbtn" onclick={() => openUrl("ms-settings:privacy-notifications")}>{t("set.notifyOpen")}</button>
            {/if}
          </div>
        {/if}
      </div>
      <p class="cap">{t("set.cap.ai")}</p>
      <div class="card">
        <div class="row">
          <span class="rb"><AgentLogo agent="claude" size={18} /></span>
          <div class="rt"><b>{t("set.claude")}</b><span>{t("set.claudeHint")}</span></div>
          <input class="switch" type="checkbox" checked={claudeOn} onchange={(e) => setClaude(e.currentTarget.checked)} aria-label={t("set.claude")} />
        </div>
        {#if claudeError}
          <div class="row small">
            {@render badge("info")}
            <div class="rt"><span>{t("set.claudeError", { e: claudeError })}</span></div>
          </div>
        {/if}
        {#if claudeOn}
          <div class="row">
            {@render badge("clock")}
            <div class="rt"><b>{t("set.claudeWait")}</b><span>{t("set.claudeWaitHint")}</span></div>
            <select value={s.claudeWait} onchange={(e) => set("claudeWait", +e.currentTarget.value)} aria-label={t("set.claudeWait")}>
              {#each [60, 300, 0] as n (n)}<option value={n}>{t(`set.claudeWait.${n}` as TKey)}</option>{/each}
            </select>
          </div>
        {/if}
        <div class="row">
          {@render badge("sparkle")}
          <div class="rt">
            <b>{t("set.agents")}</b><span>{t("set.agentsHint")}</span>
            <span class="agents">
              {#each OTHER_AGENTS as a (a)}
                <span class="agent"><AgentLogo agent={a} size={14} />{AGENTS[a].name}<i>{t("set.soon")}</i></span>
              {/each}
            </span>
          </div>
        </div>
      </div>
    {:else if section === "appearance"}
      <div class="preview" aria-hidden="true">
        <span class="pv-label">{t("set.preview")}</span>
        <div class="pv-row" style:zoom={NOTCH_SCALE[s.notchSize]}>
          <div class="pv" style:width="200px">
            <span class="art"></span>
            <span class="wave"><i></i><i></i><i></i><i></i></span>
          </div>
          <div class="pv" style:width="240px">
            <Icon name="volume" size={16} />
            <span class="bar"><span></span></span>
            <span class="num">64</span>
          </div>
        </div>
      </div>

      <p class="cap">{t("set.cap.notch")}</p>
      <div class="card">
        <div class="row">
          {@render badge("notch")}
          <div class="rt"><b>{t("set.style")}</b><span>{t(`set.styleHint.${s.notchStyle}` as TKey)}</span></div>
          <div class="seg" role="radiogroup" aria-label={t("set.style")}>
            {#each ["black", "graphite", "glass"] as const as id (id)}
              <label><input type="radio" name="style" checked={s.notchStyle === id} onchange={() => set("notchStyle", id)} />{t(`set.style.${id}`)}</label>
            {/each}
          </div>
        </div>
        <div class="row">
          {@render badge("resize")}
          <div class="rt"><b>{t("set.size")}</b><span>{t(`set.sizeHint.${s.notchSize}` as TKey)}</span></div>
          <div class="seg" role="radiogroup" aria-label={t("set.size")}>
            {#each SIZES as id (id)}
              <label><input type="radio" name="notch-size" checked={s.notchSize === id} onchange={() => set("notchSize", id)} />{t(`set.size.${id}`)}</label>
            {/each}
          </div>
        </div>
        <div class="row">
          {@render badge("text")}
          <div class="rt"><b>{t("set.textSize")}</b><span>{t("set.textSizeHint")}</span></div>
          <div class="seg" role="radiogroup" aria-label={t("set.textSize")}>
            <label><input type="radio" name="size" checked={s.textSize === "standard"} onchange={() => set("textSize", "standard")} />{t("set.textStandard")}</label>
            <label><input type="radio" name="size" checked={s.textSize === "large"} onchange={() => set("textSize", "large")} />{t("set.textLarge")}</label>
          </div>
        </div>
        <div class="row">
          {@render badge("motion")}
          <div class="rt"><b>{t("set.motion")}</b><span>{t("set.motionHint")}</span></div>
          <select value={s.motion} onchange={(e) => set("motion", e.currentTarget.value as Settings["motion"])} aria-label={t("set.motion")}>
            <option value="system">{t("set.motionSystem")}</option>
            <option value="reduce">{t("set.motionReduce")}</option>
          </select>
        </div>
      </div>

      <p class="cap">{t("set.cap.colors")}</p>
      <div class="card">
        <div class="row">
          {@render badge("palette")}
          <div class="rt"><b>{t("set.systemAccent")}</b><span>{t("set.systemAccentHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.useSystemAccent} onchange={(e) => set("useSystemAccent", e.currentTarget.checked)} aria-label={t("set.systemAccent")} />
        </div>
        {#each ["media", "hud"] as const as key (key)}
          {@const name = t(`set.accent.${key}`)}
          <div class="row">
            {@render badge(key === "media" ? "wave" : "volume")}
            <div class="rt"><b class="light">{name}</b><span>{t(`set.accent.${key}Hint`)}</span></div>
            {#if s.useSystemAccent}
              <span class="sys"><span style:background={status.now?.accent ?? s.accents[key]}></span>{t("set.systemAccentShort")}</span>
            {:else}
              <div class="swatches" role="radiogroup" aria-label={name}>
                {#each PRESETS as [color, label] (color)}
                  <label class="sw" style:background={color}>
                    <input type="radio" name="accent-{key}" checked={s.accents[key].toLowerCase() === color} onchange={() => set("accents", { ...s.accents, [key]: color })} aria-label={t(label)} />
                  </label>
                {/each}
                <input type="color" value={s.accents[key]} onchange={(e) => set("accents", { ...s.accents, [key]: e.currentTarget.value })} aria-label={t("set.custom", { name })} />
              </div>
            {/if}
          </div>
        {/each}
        <div class="row">
          {@render badge("wave")}
          <div class="rt"><b>{t("set.followArt")}</b><span>{t("set.followArtHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.followArtwork} onchange={(e) => set("followArtwork", e.currentTarget.checked)} aria-label={t("set.followArt")} />
        </div>
        <div class="row">
          {@render badge("sparkle")}
          <div class="rt"><b>{t("set.glow")}</b><span>{t("set.glowHint")}</span></div>
          <input class="switch" type="checkbox" checked={s.glow} onchange={(e) => set("glow", e.currentTarget.checked)} aria-label={t("set.glow")} />
        </div>
      </div>
    {:else}
      <div class="card">
        <div class="row tall">
          <span class="mini big"><i></i><i></i></span>
          <div class="rt"><b>Zaber</b><span>{t("set.version", { v: version || "…" })}</span></div>
        </div>
        <div class="row">
          {@render badge("shield")}
          <div class="rt"><b>{t("set.localOnly")}</b><span>{t("set.localOnlyHint")}</span></div>
        </div>
        <div class="row">
          {@render badge("reset")}
          <div class="rt"><b>{t("set.welcome")}</b><span>{t("set.welcomeHint")}</span></div>
          <button class="sbtn" onclick={() => emit("notch:welcome")}>{t("set.replay")}</button>
        </div>
      </div>
      <div class="card">
        <div class="row">
          {@render badge("trash")}
          <div class="rt"><b>{t("set.resetAll")}</b><span>{t("set.resetAllHint")}</span></div>
          <button class="sbtn danger" onclick={resetAll}>{confirmReset ? t("set.resetConfirm") : t("set.resetAllBtn")}</button>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  :global(html),
  :global(body) {
    background: #121214;
    color-scheme: dark;
  }
  .app {
    min-height: 100vh;
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
  }
  nav {
    flex: 1 1 220px;
    max-width: 100%;
    box-sizing: border-box;
    padding: 22px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    position: sticky;
    top: 0;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 10px 16px;
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    font-size: 15px;
  }
  .brand-text span {
    font-size: 11px;
    color: #8e8e93;
  }
  .mini {
    width: 36px;
    height: 16px;
    border-radius: 0 0 8px 8px;
    background: #000;
    box-shadow: 0 0 0 1px #ffffff33;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    flex: none;
  }
  .mini i {
    width: 3px;
    height: 5px;
    border-radius: 2px;
    background: #f5f5f7;
  }
  .mini.big {
    width: 56px;
    height: 24px;
    border-radius: 0 0 12px 12px;
    gap: 8px;
  }
  .mini.big i {
    width: 4px;
    height: 7px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    margin: 0 2px 6px;
    padding: 0 10px;
    border-radius: 9px;
    background: #ffffff0f;
    color: #8e8e93;
  }
  .search:focus-within {
    outline: 2px solid #0a84ff;
    outline-offset: 2px;
  }
  .search input {
    all: unset;
    flex: 1;
    min-width: 0;
    font-size: 13px;
    color: #f5f5f7;
  }
  .search input::placeholder {
    color: #8e8e93;
  }
  .grp {
    margin: 14px 10px 4px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #8e8e93;
  }
  .none {
    margin: 10px;
    font-size: 13px;
    color: #8e8e93;
  }
  .nav {
    box-sizing: border-box;
    width: 100%;
    height: 38px;
    padding: 0 10px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: #d1d1d6;
    transition: background-color 150ms ease-out;
  }
  .nav:hover {
    background: #ffffff0d;
  }
  .nav.on {
    background: #ffffff1a;
    color: #fff;
    font-weight: 600;
  }
  .nb {
    width: 20px;
    display: grid;
    place-items: center;
    color: #8e8e93;
    flex: none;
  }
  .nav.on .nb {
    color: #fff;
  }
  .nb.big {
    width: 32px;
    color: #f5f5f7;
  }
  main {
    flex: 999 1 520px;
    min-width: 0;
    box-sizing: border-box;
    padding: 30px 32px 48px;
    border-left: 1px solid #ffffff12;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 840px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
  }
  .ht {
    flex: 1 1 300px;
  }
  header h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 650;
  }
  header p,
  .hint {
    margin: 3px 0 0;
    font-size: 14px;
    color: #a1a1a6;
  }
  .cap {
    margin: 0 4px -14px;
    font-size: 12px;
    font-weight: 600;
    color: #8e8e93;
  }
  .card {
    background: #1c1c1e;
    border: 1px solid #ffffff12;
    border-radius: 14px;
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 58px;
    padding: 10px 16px;
    box-sizing: border-box;
    flex-wrap: wrap;
  }
  .row.small {
    min-height: 52px;
  }
  .row.tall {
    min-height: 84px;
  }
  .row + .row {
    border-top: 1px solid #ffffff12;
  }
  .rb {
    width: 22px;
    height: 32px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    flex: none;
    color: #a1a1a6;
  }
  .rt {
    flex: 1 1 220px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .rt b {
    font-size: 14px;
    font-weight: 600;
  }
  .rt b.light {
    font-weight: 500;
  }
  .agents {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }
  .rt .agent {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 10px;
    border-radius: 13px;
    background: #ffffff12;
    color: var(--fg, #f5f5f7);
    font-size: 12px;
  }
  .agent i {
    font-style: normal;
    font-size: 10px;
    opacity: 0.6;
  }
  .rt span {
    font-size: 12.5px;
    line-height: 1.35;
    color: #a1a1a6;
  }
  .pos {
    width: 18px;
    font-size: 12px;
    color: #8e8e93;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .switch {
    appearance: none;
    margin: 0;
    flex: none;
    width: 44px;
    height: 26px;
    border-radius: 13px;
    background: #48484a;
    position: relative;
    cursor: pointer;
    transition: background-color 150ms ease-out;
  }
  .switch::before {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px #0006;
    transition: transform 180ms cubic-bezier(0.3, 1.3, 0.5, 1);
  }
  .switch:checked {
    background: var(--accent-media);
  }
  .switch:checked::before {
    transform: translateX(18px);
  }
  select {
    height: 34px;
    border-radius: 9px;
    border: 1px solid #ffffff1f;
    background: #2c2c2e;
    color: #f5f5f7;
    font: inherit;
    font-size: 13px;
    padding: 0 10px;
    max-width: 100%;
  }
  .seg {
    display: flex;
    padding: 3px;
    border-radius: 10px;
    background: #2c2c2e;
    gap: 2px;
  }
  .seg label {
    position: relative;
    height: 30px;
    padding: 0 14px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    font-size: 13px;
    color: #c7c7cc;
    cursor: pointer;
    transition: background-color 150ms ease-out;
  }
  .seg input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }
  .seg label:has(input:checked) {
    background: #636366;
    color: #fff;
    font-weight: 600;
    box-shadow: 0 1px 2px #0005;
  }
  .seg label:has(input:focus-visible) {
    outline: 2px solid #0a84ff;
  }
  .range {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .range input {
    width: 170px;
    accent-color: var(--accent-media);
  }
  .pill {
    min-width: 44px;
    text-align: center;
    font-size: 11px;
    font-weight: 700;
    padding: 3px 8px;
    border-radius: 6px;
    background: #ffffff14;
    color: #c7c7cc;
    font-variant-numeric: tabular-nums;
  }
  .ibtn {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: #c7c7cc;
  }
  .ibtn:hover:not(:disabled) {
    background: #ffffff1a;
  }
  .ibtn:disabled {
    color: #48484a;
  }
  .updown {
    display: flex;
    gap: 2px;
  }
  .sbtn {
    height: 34px;
    border: 1px solid #ffffff1f;
    border-radius: 9px;
    background: #2c2c2e;
    color: #f5f5f7;
    font-size: 13px;
    font-weight: 600;
    padding: 0 14px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    box-sizing: border-box;
  }
  .sbtn:hover {
    background: #3a3a3c;
  }
  .sbtn.danger {
    color: #ff6961;
  }
  .swatches {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .sw {
    position: relative;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
  }
  .sw input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }
  .sw:has(input:checked) {
    box-shadow:
      0 0 0 2px #1c1c1e,
      0 0 0 4px #f5f5f7;
  }
  .sw:has(input:focus-visible) {
    outline: 2px solid #0a84ff;
    outline-offset: 4px;
  }
  input[type="color"] {
    width: 34px;
    height: 30px;
    padding: 0;
    border: 1px solid #ffffff26;
    border-radius: 8px;
    background: #2c2c2e;
    cursor: pointer;
  }
  .sys {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: #a1a1a6;
  }
  .sys span {
    width: 20px;
    height: 20px;
    border-radius: 50%;
  }
  .preview {
    position: relative;
    height: 150px;
    border-radius: 14px;
    background:
      url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 400 100' preserveAspectRatio='none'%3E%3Cpath d='M0 58 L38 40 L70 52 L112 26 L150 46 L196 22 L238 44 L282 30 L326 50 L364 34 L400 46 V100 H0Z' fill='%23474c78'/%3E%3Cpath d='M0 78 L46 60 L92 72 L140 54 L188 70 L232 58 L276 74 L322 60 L366 72 L400 64 V100 H0Z' fill='%2326283f'/%3E%3C/svg%3E")
        bottom / 100% 62% no-repeat,
      radial-gradient(circle at 72% 78%, #ffd9a0 0 14px, #ffb37a66 15px, transparent 60px),
      linear-gradient(180deg, #141a33 0%, #2c3a6b 40%, #8a6a8f 72%, #e59a72 100%);
    overflow: hidden;
  }
  .pv-label {
    position: absolute;
    left: 12px;
    bottom: 10px;
    font-size: 11px;
    font-weight: 600;
    color: #ffffffcc;
  }
  .pv-row {
    display: flex;
    justify-content: center;
    gap: 16px;
    flex-wrap: wrap;
  }
  .pv {
    height: 34px;
    border-radius: 0 0 14px 14px;
    box-sizing: border-box;
    padding: 0 14px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    background: var(--notch-bg);
    border: var(--notch-border);
    border-top: 0;
    color: var(--notch-fg);
    font-size: var(--pill-text);
  }
  .pv .art {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: #5e5ce6;
  }
  .pv .wave {
    display: flex;
    gap: 2px;
    align-items: center;
    height: 14px;
  }
  .pv .wave i {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: var(--accent-media);
  }
  .pv .wave i:nth-child(2n) {
    height: 60%;
  }
  .pv .bar {
    flex: 1;
    height: 5px;
    border-radius: 3px;
    background: #ffffff26;
  }
  .pv .bar span {
    display: block;
    width: 64%;
    height: 100%;
    border-radius: 3px;
    background: var(--accent-hud);
  }
  :global(.glow) .pv .bar span,
  :global(.glow) .pv .wave i {
    box-shadow:
      0 0 3px color-mix(in srgb, var(--accent-media) 90%, transparent),
      0 0 12px color-mix(in srgb, var(--accent-media) 60%, transparent);
  }
  .pv .num {
    font-size: 12px;
    color: #8e8e93;
    font-variant-numeric: tabular-nums;
  }
  @media (prefers-reduced-motion: reduce) {
    .switch::before,
    .nav,
    .seg label {
      transition: none;
    }
  }
</style>
