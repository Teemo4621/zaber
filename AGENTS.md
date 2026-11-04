# AGENTS.md — Zaber

This is the single source of truth for every AI agent (Claude, Codex, Cursor, Gemini, Copilot, …)
working in this repo. Tool-specific files (`CLAUDE.md`, etc.) only point here.
Reply to the user in the language they write in.

## What we are building

**Zaber** is a notch app for **Windows 10/11 and macOS**: a small black pill at the top-center of
the screen that expands on hover to show live info (music, timer, to-dos, …) and collapses when the
cursor leaves. Full product spec and roadmap:
[`docs/PRODUCT.md`](docs/PRODUCT.md) — read it before adding or changing a feature.

### Non-negotiable product principles

1. **Hover to open, leave to close.** No pinning, no arranging, no settings needed to start.
2. **Invisible when idle.** ~0% CPU idle. Nothing animates unless something changed. No infinite CSS
   animations, no `requestAnimationFrame` loops left running.
3. **Never steal focus or clicks.** Outside the visible notch shape the window is click-through.
4. **Local only.** No accounts, no telemetry, no network calls except features that need them
   (e.g. lyrics). User data stays on the device.
5. **Same feature, both OSes.** A feature is "done" only when it works on Windows and macOS, or the
   missing OS is listed in `docs/PRODUCT.md` with the reason.
6. **Small.** Don't add a dependency for something a few lines can do.

## Stack

| Layer | Tech | Where |
|---|---|---|
| Native / OS access | Rust (edition 2024), Tauri 2 | `src-tauri/` |
| UI | Svelte 5 (runes) + SvelteKit static SPA, TypeScript | `src/` |
| Animation | `svelte/motion` (`Spring`, `Tween`) + `svelte/transition` | — |
| Windows APIs | `windows` crate (WinRT) | `*/windows.rs` |
| macOS APIs | `objc2` family crates (add when first needed) | `*/macos.rs` |

Package manager: **npm**. Do not introduce pnpm/yarn/bun lockfiles.

## Commands

```sh
npm install              # once
npm run tauri dev        # run the app (hot reload for UI, rebuilds Rust)
npm run check            # Svelte/TS type-check — must be 0 errors
cd src-tauri && cargo check && cargo clippy   # Rust — must be clean
npm run tauri build      # release bundle for the current OS
```

Before saying a change is done: `npm run check` and `cargo check` pass, and you ran
`npm run tauri dev` when the change is visible in the UI.

## Architecture

```
src-tauri/src/
  lib.rs                 # app builder: registers commands, starts modules
  notch.rs               # window placement, hover detection, click-through
  modules/
    mod.rs               # list of modules
    <module>/
      mod.rs             # shared types, #[tauri::command]s, background loop — OS-agnostic
      windows.rs         # Windows implementation (#[cfg(windows)])
      macos.rs           # macOS implementation (#[cfg(target_os = "macos")])
src/
  routes/+page.svelte    # mounts <Notch/>
  lib/theme.css          # design tokens — the only place for colors, radii, fonts
  lib/notch/Notch.svelte # the shell: collapsed pill ↔ expanded panel
  lib/modules/<module>/
    <module>.svelte.ts   # state ($state), invoke/listen wrappers, TS types mirroring Rust
    <Module>View.svelte  # UI shown inside the notch
```

### Data flow

- **Rust owns OS state, Svelte owns layout.** Rust watches the OS and **emits events**; Svelte calls
  **commands** for actions.
- Rust → UI: `app.emit("<module>:<event>", payload)` — emit only when the value changed.
- UI → Rust: `invoke("<module>_<verb>", args)` — validate every argument in Rust (it's a trust
  boundary), return `Result<T, String>`.
- Hover: `notch.rs` polls the cursor, emits `notch:hover` (bool) and toggles click-through. The UI
  reports the notch's current target rect via `notch_set_hit_rect`.

### Module contract (every feature follows this)

1. Rust: `src-tauri/src/modules/<name>/mod.rs` with serde types (`#[serde(rename_all = "camelCase")]`),
   commands named `<name>_<verb>`, events named `<name>:<event>`, and a `start(app)` if it watches
   the OS. OS code lives only in `windows.rs` / `macos.rs` behind the same functions.
2. Register in `modules/mod.rs` and `lib.rs` (`generate_handler!` + `start`).
3. Svelte: `src/lib/modules/<name>/<name>.svelte.ts` (types mirror Rust exactly) and `<Name>View.svelte`.
4. Mount the view in `Notch.svelte`.
5. Update the feature's row in `docs/PRODUCT.md`.

Reference implementation: the **media** module. Copy its shape.

## Code rules

- Match surrounding code. **No comments in code** (no `//`, `///`, `/* */`, `<!-- -->`), except
  tool directives such as `svelte-ignore` and `@ts-expect-error`. Explain the why and any known
  limitation in `docs/PRODUCT.md` or the commit message instead.
- No `unwrap()` on OS calls in Rust — OS APIs fail (no player open, permission denied). Return
  `Option`/`Result`; `unwrap()` is fine only on mutex locks.
- Never block the main thread: OS polling/waiting goes in a background thread or async task.
- Svelte 5 runes only (`$state`, `$derived`, `$effect`, `$props`). No Svelte 4 syntax
  (`export let`, `$:`, stores for local state, `on:click`).
- Colors/sizes/radii come from CSS variables in `src/lib/theme.css`.
- Accessibility: icon buttons get `aria-label`; respect `prefersReducedMotion`.
- Persisted user data (tasks, water log, shelf) → app data dir as JSON, written atomically
  (write temp file, then rename) so a crash never loses data.

## Skills (playbooks — read the file before doing the task)

Skills live in `.agents/skills/` (the shared Agent Skills folder that Codex, Gemini CLI, Cursor and
Hermes read). `.claude/skills` is a symlink to it for Claude Code; on Windows, clone with
Developer Mode on and `git config core.symlinks true`, or the link checks out as a text file.

| Task | Read |
|---|---|
| Add or extend a notch feature | [`.agents/skills/add-notch-module/SKILL.md`](.agents/skills/add-notch-module/SKILL.md) |
| Write Windows/macOS-specific Rust | [`.agents/skills/platform-native/SKILL.md`](.agents/skills/platform-native/SKILL.md) |
| UI, animation, visual design | [`.agents/skills/notch-ui/SKILL.md`](.agents/skills/notch-ui/SKILL.md) |

## Git

- Branch from `main`; small commits; message style: `feat(media): add seek`, `fix(notch): …`.
- Never commit `node_modules/`, `build/`, `src-tauri/target/`.
