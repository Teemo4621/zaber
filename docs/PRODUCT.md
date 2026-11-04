# Zaber — Product spec

Notch app for Windows 10/11 and macOS.
Principles live in [`AGENTS.md`](../AGENTS.md#non-negotiable-product-principles).

## Behaviour

Screen map and sizes: `.agents/skills/notch-ui/SKILL.md`.

- Collapsed, idle: empty black pill at top-center — no clock. The resting companion lives here.
- Collapsed, media playing: artwork/thumbnail left, animated sound wave right. Paused → back to idle.
- Hover → springs open to the expanded panel: a tab row (Media · Focus · To-do · Water · Shelf) and
  the clock, then the current tab. Media shows the playing or paused item. Cursor leaves → collapses.
- Outside the pill/panel the window is click-through.
- Steps aside when a full-screen app is in front.
- Idle for a while → "Resting" companion: a pair of eyes appears, dozes off, waves when the user
  comes back.

## Features

Status: ✅ done · 🟡 partial · ⬜ not started. `Win` / `Mac` = per-OS status.

| # | Feature | What it does | Win | Mac |
|---|---|---|---|---|
| 1 | Notch shell | Pill ↔ panel, hover open/close, click-through, top-center placement | ✅ | 🟡 window can't sit over the menu bar yet |
| 2 | Media | Now playing from any player (Spotify, Apple Music, browser) + play/pause/next/prev | 🟡 artwork + wave pill done; no seek, no tab row yet | ⬜ |
| 3 | Synced lyrics | LRCLIB with retries on its frequent 503s, YouTube-title cleanup ("Artist - Song (Official MV)", "X - Topic"), free-text fallback, plain lyrics when no synced ones; Lyrics button splits the notch; current line highlighted; click a line to seek; lyrics stay on after the cursor leaves: the pill then shows the line being sung under the artwork + wave (one timer to the next line, no polling) until you turn lyrics off or the player goes away | ✅ | ⬜ macOS side not written yet |
| 4 | Focus timer | Pomodoro countdown in the notch (Focus / Short break), soft chime at break | ✅ | ⬜ macOS side not written yet |
| 5 | Water reminder | Intake vs daily goal, hourly chart, bottle size + goal + reminder hours settings | ✅ | ⬜ macOS side not written yet |
| 6 | File shelf | Real OS icons / thumbnails (IShellItemImageFactory); Drop files on the notch to keep them, drag out to apps, click to open | ✅ | ⬜ macOS side not written yet |
| 7 | To-do | Add/edit/check tasks with a time; notch alerts when due | ✅ | ⬜ macOS side not written yet |
| 8 | HUD overlays | Volume, brightness, Caps Lock (stays up while on), keyboard-language change (only a switch inside the focused window; Alt+Tab through stale windows is ignored) | ✅ | ⬜ macOS side not written yet |
| 9 | Status peeks | Charger plugged in, Wi-Fi lost, headphones connected (+ battery) | ✅ | ⬜ macOS side not written yet |
| 10 | Full-screen step aside | Hide while a full-screen app is focused | ✅ | ⬜ macOS side not written yet |
| 11 | Resting companion | Idle eyes: appear after 8 s idle, blink 3×, glance left-right at 15 / 30 / 45 s, yawn (squint, little "o", two z's float up once) then doze (eyes shut) at 60 s; ^ ^ hop when you come back. Eyes turn toward the cursor when it is within 300 px of the notch (Rust sends a 5-step direction only when it changes). Wakes with a start (wide eyes + !) when a peek pops up during a nap; ^ ^ + a heart after a small win (to-do done, water goal reached, Claude Code finished), played when the eyes are back on screen. Every move plays once (no loops); reduced motion: none. Toggle in Settings → General | ✅ | ✅ (cursor direction from Tauri's cursor position) |
| 15 | Notifications | Hover a notification → message panel: full text, Dismiss, "Open in <app>" (launches the sender by AppUserModelID; replying is impossible — Windows gives read-only access); Toasts from other apps re-shown as a 5 s peek (sender app icon from its Start menu entry, glowing in its own color with Glow on; title, body, app). Read-only. Windows `UserNotificationListener` (polled 1 s; user can block it in Privacy → Notifications). Toggle in Settings → Tabs → peeks | ✅ | ⬜ no public API to read other apps' notifications |
| 16 | Battery | Laptops only (hidden without a battery): gauge + % next to the clock in the open panel (green + bolt while charging, red ≤ 20 %); "Low battery" peek when it drops past 20 % and 10 % unplugged; charger peek shows the gauge. Toggle: Settings → Tabs → peeks | ✅ | ⬜ status probe is a stub (needs IOKit power) |
| 17 | Glow | Live elements light up in their own color: media progress bar, sound-wave bars, artwork (lit in its main color), current lyric line, volume bar, focus ring, water meter + chart. Static shadows (no extra animation). Toggle: Settings → Appearance | ✅ | ✅ |
| 18 | Volume keys | Zaber's HUD replaces the OS volume flyout: a low-level keyboard hook swallows only volume up/down/mute and Zaber sets the volume itself (2 % steps, unmute on volume key) — exact limit detection for the stretch. Falls back to the OS when off, hung (hook timeout) or quit. Some laptops' Fn keys bypass virtual keys and may still show the OS flyout. Toggle: Settings → Tabs → peeks | ✅ | ⬜ needs CGEventTap + Accessibility permission |
| 19 | Brightness HUD | Yellow bar on any brightness change: laptop panel via WMI (`WmiMonitorBrightness`, needs CoSetProxyBlanket), else first DDC/CI monitor (polled 2 s). No limit-stretch: laptop brightness keys never reach apps | ✅ | ⬜ needs DisplayServices |
| 20 | Copied peek | First line / link host / image size / file names of what was just copied; nothing stored; skips content marked private (ExcludeClipboardContentFromMonitorProcessing, CanIncludeInClipboardHistory=0) | ✅ | ⬜ NSPasteboard changeCount |
| 21 | Devices | USB drive connected (label, size, Open) / removed; mouse & keyboard connected / removed, named by product (device container) | ✅ | ⬜ IOKit / DiskArbitration |
| 22 | Device battery | Connected Bluetooth devices that report battery (DEVPKEY_Bluetooth_Battery on their service node, e.g. a headset's Hands-Free AG; only containers whose device node is connected, so old pairings don't count), polled 10 s: shown in the headphones peek (name + %, re-shown once the level arrives after connecting) and beside the clock in the open notch (headphones icon + %, the ones playing now), warn at 20 % and 10 % | ✅ | ⬜ IOBluetooth |
| 23 | First launch | Manual start: dot → pill → eyes (~2 s); very first run adds the welcome panel with 3 tips. Login start (`--autostart`) shows nothing. Replay in Settings → About | ✅ | ✅ |
| 24 | Notch size | Compact / Standard / Large (×0.88 / 1 / 1.16, CSS zoom; window 760×320 fits Large) | ✅ | ✅ |
| 25 | Claude Code | Settings → Replies & AI switch adds Zaber's HTTP hooks to `~/.claude/settings.json` (others kept, `.zaber-backup` saved; switch off removes only ours). PermissionRequest → peek (fades after a few seconds; hovering the notch reopens the request until it is answered, closed, or times out) + panel: Ask in terminal (close; hand back now) / Deny / Always allow (Claude's own suggested rule) / Allow once; MCP Elicitation form questions (enum / text / number / boolean); Stop → "Done in <project>"; idle prompt → "Claude is waiting". 127.0.0.1:47811 + secret token; no answer within the chosen time (1 / 5 / ~10 min) → empty reply, the terminal asks as usual. Zaber closed → hook fails fast, Claude Code unaffected. Not possible: the terminal's AskUserQuestion multiple-choice (no hook exposes it); URL-mode elicitation. Keys when opened by the shortcut (#27): Enter allow once, A always, Esc deny / decline | ✅ | ✅ (same code; untested on macOS) |
| 26 | AI activity | Claude Code sessions from extra activity hooks (UserPromptSubmit, PreToolUse, SessionEnd; answered at once, never decide anything): pill shows the logo (breathes once per step), project, "Working · m:ss" or "Needs you"; count badge with several sessions; with music playing the media pill swaps its wave for the logo + count. Hovering the pill opens a Claude tab: every session with what it's doing (working / waiting / done, kept 10 min). Above them the plan's limits: Session (5-hour) and Week, % used + time to reset, orange from 80 %; they come from a status line Zaber adds (curl to 127.0.0.1, prints "5h 23% · 7d 41%" in the terminal), only for Pro/Max, after the first reply, and not if you already have your own status line. Older installs show the Settings switch off until turned on again (adds the new hooks). Not possible: jumping to a session's terminal window | ✅ | ✅ (same code; untested on macOS) |
| 27 | Keyboard | Global shortcut (tauri-plugin-global-shortcut) opens the notch and gives it the keyboard: ← → switch tabs, Esc closes (focus goes back to the window you were in), Enter / A / Esc answer Claude. Ctrl+Alt+Space (Windows), Ctrl+Option+N (macOS: Ctrl+Option+Space is the input-source switch). Fixed keys for now | ✅ | 🟡 key focus stays on the notch after Esc until you click elsewhere |
| 28 | Notes | Notes tab: one plain-text scratchpad, saved (atomic JSON, `notes` store) 0.4 s after typing stops | ✅ | ✅ |
| 29 | Mic & camera | Privacy light: orange dot (mic) / green dot (camera) at the right of the pill while any app records, plus a peek naming the app when it starts. Windows privacy consent store (`CapabilityAccessManager\ConsentStore`, LastUsedTimeStop = 0 while in use; polled 2 s). Packaged apps show their package name. Mute: Ctrl+Alt+M mutes / unmutes the default microphone for every app; red "Microphone off" peek stays up while muted (also when muted elsewhere). Toggles: Settings → Tabs → peeks | ✅ | ⬜ needs CoreAudio (DeviceIsRunningSomewhere, mute) and CoreMediaIO |
| 30 | Screenshots & downloads | New screenshot (Pictures\Screenshots on Windows, PNGs on the Desktop on macOS) → peek with thumbnail + pixel size, Open / Keep (on the shelf); finished download (browser partial files ignored until renamed) → peek with the real file icon + size, Open / Show in folder. While a download runs (its partial file grows): the pill shows ↓ name + size so far + speed, or a small "↓ 2.3 MB/s" bubble beside the pill when music / the timer / Claude has it; paused (no growth for 20 s) ones drop off. No percentage: browsers don't write the total into the file. Only folder listings are read, when the folder changes (polled 1 s, one stat), plus one stat per running download. Toggle: Settings → Tabs → peeks | ✅ | 🟡 no thumbnails/icons (shelf icons not written for macOS); custom screenshot folder not read |
| 31 | Install & uninstall | `npm run tauri build` → Windows: NSIS setup `.exe` (per-user, no admin), uninstall from Settings → Apps or Zaber Settings → General → Uninstall (runs `uninstall.exe`; its pre-uninstall hook runs `Zaber.exe --uninstall`, which removes Zaber's Claude Code hooks and status line; the uninstaller removes launch-at-login and, if ticked, app data; skipped on updates). macOS: `.dmg` (drag to Applications); Settings → Uninstall removes the hooks, launch-at-login (LaunchAgent) and app data, then moves Zaber.app to the Trash via Finder (macOS asks once to let Zaber control Finder). Unsigned for now: SmartScreen / Gatekeeper warn on first open | ✅ | ✅ (untested on macOS) |
| 12 | Language | UI in EN / TH | ✅ | ⬜ macOS side not written yet |
| 14 | Calendar | Tab: today + the whole week (Monday first), rest-of-day agenda with each calendar's color, Join button for meeting links; with no events (or no access) the date and week still show and the right side says why ("No more events today" + tomorrow's first, "Calendar access needed", or where to add / allow it); peek N min before an event. Read-only, no accounts in Zaber: reads the OS calendars (Windows `Windows.ApplicationModel.Appointments` = accounts added to the Windows Calendar / Outlook app; macOS EventKit = the Calendar app's accounts, asks once for access without blocking: Info.plist `NSCalendarsFullAccessUsageDescription`) | 🟡 unpackaged apps may only see the local calendar (needs package identity) | ✅ (type-checked for aarch64-apple-darwin; untested on a Mac) |
| 13 | Settings window | Separate normal window (tray icon / right-click notch). **General**: launch at login, language, display, open delay, full-screen hide, resting companion. **Tabs & features**: on/off + order per tab, "open to", pill priority, lyrics, peeks on/off + duration. **Appearance**: notch style (Black / Graphite / Glass), text size, motion, colors for **Media player** and **HUD & peeks** only (presets + custom, or **use system accent color** — Windows `UISettings.GetColorValue(Accent)` / macOS `NSColor.controlAccentColor`, live-updating), wave follows artwork, reset. Timer, water and alerts keep fixed colors. Saved as JSON in app data dir; colors override `theme.css` tokens at runtime | ✅ | ⬜ macOS side not written yet |

Suggested build order: 1 → 2 (finish) → 10 → 4 → 7 → 6 → 5 → 9 → 8 → 3 → 11 → 12.

## Open problems

- **macOS media:** `MediaRemote.framework` is private and blocked for third-party apps since
  macOS 15.4. Options: bundle a helper run through `/usr/bin/perl` (the `mediaremote-adapter`
  approach), or AppleScript per player (Spotify, Music) with no browser support. Decide before
  implementing.
- **macOS window over the menu bar:** needs `NSWindow.level` above the status-bar level via `objc2`;
  Tauri's `alwaysOnTop` is not enough.
- **Windows calendar access:** `AppointmentManager.RequestStoreAsync` may need package identity
  (MSIX) and only sees calendars synced into the Windows Calendar/Outlook app. Verify on an unpackaged
  build first; fallback is a user-pasted ICS URL (needs network, opt-in).
- **Windows HUD:** replacing the system volume flyout isn't supported by an official API; show our
  HUD alongside it, or hide it via the known `NativeHWNDHost` trick (fragile).

## Out of scope (for now)

Accounts, cloud sync, plugins, themes marketplace, mobile.
