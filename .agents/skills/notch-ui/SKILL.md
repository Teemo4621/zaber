---
name: notch-ui
description: Use when building or changing anything visual in Zaber — notch views, layout, colors, icons, animation, transitions, the resting companion. Defines the look, motion rules and performance limits.
---

# Notch UI

The rules below describe the approved design for every screen (Shell, Media, Lyrics, Focus, Water,
Shelf, To-do, HUD & peeks). Build to them.

## Screen map

| State | Size | Content |
|---|---|---|
| Idle (collapsed) | 180 × 32 | **Empty** — no clock. Reserved for the resting companion animation |
| Playing (collapsed) | 260 × 32 | Artwork left (22 px), sound-wave bars right |
| Focus running (collapsed) | 260 × 32 | Progress ring left, `24:13` right — beats the media pill |
| Peek (HUD, status, reminder, due alert) | 300–340 × 36–40 | Icon + short text/value; folds back after ~2 s (alerts stay until acted on) |
| Expanded (hover) | 480 × 200–220 | Tab row (Media · Focus · To-do · Water · Shelf) + clock on the right, then the tab's page |
| Lyrics open | 640 × 240 | Split: compact player left, lyric lines right |

Accent per feature: media/success `#30d158`, focus `#ff9f0a`, break/water `#64d2ff`,
brightness `#ffd60a`, error `#ff6961`. Add each as a `theme.css` token when its feature ships.
Users can recolor **only** the media accent and the HUD accent (Settings → Appearance), or make both
follow the OS accent color; focus, break/water and alert colors are fixed. Components must still read
the CSS variable, never the hex.

**Settings window** (artboards "Settings — …"): a normal desktop window, not the notch. Sidebar
(General · Tabs & features · Appearance & colors) + content of grouped cards; rows are title +
one-line description on the left, one control on the right. Dark only, same tokens as the notch.

## Look

- **Ears:** where the notch meets the screen edge it flares outward with concave fillets (`.ear` in
  `Notch.svelte`, 7 px on the pill → 14 px open), like a hardware notch. Keep them on every state.

- Pure black notch (`--notch-bg`) that reads as part of the hardware notch. White text, grey
  secondary text, one accent per feature (above). All values are tokens in `src/lib/theme.css` —
  add a token there instead of hard-coding a color.
- Collapsed pill: one line, ≤ 12–13 px text, ellipsis on overflow. Nothing that needs reading twice.
- Expanded panel: 16–24 px padding, one primary thing per view (track, timer, list). Max panel
  size ≈ 640 × 260 (the window size in `tauri.conf.json`); grow the window there if you truly need more.
- Font: system UI font (`--font`) — never web fonts (they need the network; see "Local only").
- Icons: inline SVG, `fill: currentColor`, 16–24 px, `aria-hidden="true"` on the `<svg>`.
  **Never emoji or Unicode symbols (⏮ ▶ ♪) as icons** — they render differently per OS and can
  turn into color emoji on Windows. No icon library dependency; paste the SVG path.

## Interaction

- Clickable targets ≥ 32 × 32 px (36 px for primary controls), ≥ 8 px apart.
- Every clickable element has a hover state (`--notch-hover` background, 150 ms ease-out) and a
  press state (`transform: scale(0.9)`). No instant 0 ms state changes.
- Hover is how the notch *opens*, but no action may be hover-only — everything inside works by click
  and keyboard too.
- Exit faster than enter: collapsing/fading out ≈ 70% of the opening duration.
- One meaning per motion: movement shows where content came from or went; no decorative motion.

## Motion

- The shell morph (pill ↔ panel) uses `Spring` in `Notch.svelte` — reuse it, don't add another.
- **No `filter` (blur) in animations** — in WebView2 it repaints the whole panel every frame and
  drops frames. Shell sizes are rounded to whole **even** px so the centered edges never sit on
  half pixels (shimmer). Nothing may hit the UI with big payloads mid-animation: Rust events
  carry changes only (media no longer re-sends artwork every second).
- **Spring values are set with `feel(response, zeta)`** (`nudge.svelte.ts`), never raw
  stiffness/damping — Svelte's units are per-frame and easy to get wildly bouncy. Values: open `feel(0.42, 0.8)`, shrink `feel(0.45, 1)`, presses
  `feel(0.25, 1)`, pops `feel(0.35, 0.75)`, rubber release `feel(0.4, 0.85)`. **Never ζ < 0.75**, and
  shrinking is always ζ = 1 (no overshoot, no dip below the resting size).
- Content is laid out at the **target size + elastic stretch** (`inner` in `Notch.svelte`): the
  open/close spring only reveals it (no reflow while opening), but a stretch carries the content
  with it — the volume bar stretches when the notch does. Build peek/pill rows with one `flex: 1`
  part (bar, title) so stretching lands there, not on icons or text. It enters with `reveal`
  (opacity + 6 px rise + 0.97 scale, 60–80 ms after the shell starts) and leaves with a fast fade.
- Content inside: `svelte/transition` (`fade`, `fly`, `scale`) with ~150–250 ms, or a `Spring`/`Tween`
  for numbers (timer, progress).
- Animate **`transform` and `opacity`** only. The shell's width/height is the single exception.
- **Idle = zero frames.** No `infinite` CSS animations, no permanent `setInterval` faster than 1 s,
  no `requestAnimationFrame` loop that keeps running when nothing changes. The resting eyes animate
  for a few seconds, then stop until the next state change. Only exception: an `infinite` animation
  that *shows* live activity (the media sound wave) — it must stop the moment the activity stops.
- Always honor `prefersReducedMotion.current` (from `svelte/motion`): jump to the end state.

### Elastic feedback (`nudge.svelte.ts`)

Actions push the notch's edges and it springs back (`src/lib/notch/nudge.svelte.ts`: one shared
spring added on top of the notch size; settles to zero; skipped under reduced motion). Each
gesture has its own feel — use these five, don't invent new motions:

| Gesture | Feel | Used for |
|---|---|---|
| `strainAt(dir)` | rubber: each quick repeat stretches further with growing resistance (64 px max), flattens as it widens, smooth snap-back; the volume bar stretches with it — no shaking (key repeat would make it vibrate) | volume key at 100 % / 0 % |
| `strainAt(dir, "x")` | rubber lean that way | scrolling past the first/last tab |
| `lean(dir)` | that side's edge pulled out, one soft wobble | next / previous track |
| `press()` | gives in a little, no wobble | play / pause, Caps Lock off |
| `pop("s" \| "m" \| "l")` | grows and settles | Caps on (s), to-do done / focus phase ends (m), files dropped (l) |
| `drop()` | sags downward | notification / alert peek, drinking water |

Content reacts with the shell: the volume bar stretches with the notch, next/previous icons move
their way while pressed, a ticked to-do's check pops and its strike draws across.

## Svelte 5

- Runes only: `$state`, `$derived`, `$effect`, `$props`; events as `onclick={…}`.
- State shared across components lives in the module's `<name>.svelte.ts`, not in a component.
- `$effect` only for side effects (IPC, timers); computed values use `$derived`.
- Clean up listeners/timers with `onDestroy` (see `Notch.svelte`).

## Accessibility

- Every icon-only button has `aria-label`. Focus ring comes from `theme.css` — don't remove it.
- Text contrast ≥ 4.5:1 on black (`--notch-fg-dim` is the lowest allowed). Don't convey state by
  color alone — pair it with an icon or text.
- Text ≥ 11 px in the pill, ≥ 13 px in the panel.

## Pre-delivery checklist

- [ ] No emoji/Unicode icons; SVGs are `aria-hidden`, buttons have `aria-label`
- [ ] Hover + press state on every clickable element; focus ring visible
- [ ] Colors only from `theme.css` tokens; contrast ≥ 4.5:1
- [ ] `prefersReducedMotion` → no spring/transition, end state shown
- [ ] Idle for 10 s → no running animations or timers faster than 1 s

## Verify

`npm run tauri dev`, then: hover → opens smoothly, leave → closes, clicks outside the notch reach
the app below, Task Manager / Activity Monitor shows ~0% CPU after 10 s idle.
