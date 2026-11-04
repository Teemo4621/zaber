---
name: add-notch-module
description: Use when adding a new notch feature (timer, to-do, water, shelf, HUD, lyrics, status peeks…) or extending an existing one in Zaber. Walks through the Rust module, per-OS files, Svelte state/view, registration and spec update.
---

# Add a notch module

Reference implementation: `src-tauri/src/modules/media/` + `src/lib/modules/media/`. Copy its shape;
don't invent a new one.

## 0. Before code

- Find the feature row in `docs/PRODUCT.md`. If it's not there, add it and confirm scope with the user.
- Decide: does it need the OS? (media, HUD, status → yes; timer, to-do, water → Rust only for
  persistence/notifications, maybe no per-OS file at all.)
- Pick names: module `focus`, commands `focus_start`, `focus_stop`, event `focus:update`.

## 1. Rust — `src-tauri/src/modules/<name>/mod.rs`

```rust
//! <Name>: one-line purpose.
//! Events: `<name>:update` (<Type>). Commands: `<name>_<verb>`.

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct <Name>State { /* … */ }

#[tauri::command]
pub fn <name>_<verb>(arg: &str) -> Result<(), String> {
    // validate `arg` here — the UI is a trust boundary
}

pub fn start(app: AppHandle) { /* background thread; emit only on change */ }
```

- OS-specific? Add `windows.rs` + `macos.rs` exposing the **same function signatures**, select with
  `use self::windows as platform;` under `#[cfg]` like media does. See the `platform-native` skill.
- Pure logic (timer math, due-time checks, parsing) → plain functions with a `#[cfg(test)] mod tests`.
- Persistence → JSON in `app.path().app_data_dir()`, write temp + rename.

## 2. Register

- `src-tauri/src/modules/mod.rs`: `pub mod <name>;`
- `src-tauri/src/lib.rs`: add commands to `generate_handler!`, call `modules::<name>::start(...)` in `setup`.
- New Tauri plugin? Add its permission to `src-tauri/capabilities/default.json`.

## 3. Svelte — `src/lib/modules/<name>/`

`<name>.svelte.ts`:
```ts
export type <Name>State = { /* same fields as Rust, camelCase */ };
export const <name> = $state<{ value: <Name>State | null }>({ value: null });
listen<<Name>State>("<name>:update", (e) => (<name>.value = e.payload));
export const <verb> = (arg: string) => invoke("<name>_<verb>", { arg });
```

`<Name>View.svelte`: props in, `$derived` for computed values, tokens from `theme.css`.
Follow the `notch-ui` skill for layout and motion.

## 4. Mount

In `src/lib/notch/Notch.svelte`, decide what the **collapsed pill** shows (if anything) and where
the view goes in the **expanded panel**. If the panel needs a different size, change `EXPANDED`
(the hit rect follows automatically).

## 5. Verify and record

- `npm run check` → 0 errors; `cd src-tauri && cargo check && cargo test`.
- `npm run tauri dev` → hover the notch, use the feature, confirm the pill collapses and clicks pass
  through elsewhere.
- Update the row in `docs/PRODUCT.md` (Win/Mac status, limits).
