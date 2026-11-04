---
name: platform-native
description: Use when writing Windows- or macOS-specific Rust in Zaber (media sessions, volume/brightness, battery, Wi-Fi, keyboard layout, window level, full-screen detection). Covers file layout, crates, API choices per feature and how to stay cross-platform.
---

# Platform-native Rust

## Layout rule

OS code lives **only** in `modules/<name>/windows.rs` and `modules/<name>/macos.rs`. Both expose the
same functions; `mod.rs` picks one:

```rust
#[cfg(target_os = "macos")] mod macos;
#[cfg(windows)] mod windows;
#[cfg(target_os = "macos")] use self::macos as platform;
#[cfg(windows)] use self::windows as platform;
```

Use `self::` — a module named `windows` otherwise clashes with the `windows` crate.
If one OS isn't done, keep a stub that returns `None` / `Err("not implemented on <os>")` and record
it in `docs/PRODUCT.md`. Never `todo!()` — it panics the app.

## Crates

- Windows: `windows` crate (already a dependency). Add only the `features = [...]` you need in
  `src-tauri/Cargo.toml` under `[target.'cfg(windows)'.dependencies]`. Async WinRT calls: `.get()`
  in a background thread.
- macOS: `objc2`, `objc2-foundation`, `objc2-app-kit` (add under
  `[target.'cfg(target_os = "macos")'.dependencies]` when first needed). No `cocoa`/`objc` legacy crates.

## API map

| Feature | Windows | macOS |
|---|---|---|
| Media | `Media::Control` GSMTC (done) | see "Open problems" in `docs/PRODUCT.md` |
| Volume | `Win32::Media::Audio` `IAudioEndpointVolume` + `IAudioEndpointVolumeCallback` | CoreAudio `AudioObjectAddPropertyListener` |
| Brightness | WMI `WmiMonitorBrightness` (laptops) | `DisplayServices` (private) / ddc for external |
| Battery / charger | `Win32::System::Power` `GetSystemPowerStatus` + `RegisterPowerSettingNotification` | IOKit `IOPSNotificationCreateRunLoopSource` |
| Wi-Fi | `Networking::Connectivity::NetworkInformation::NetworkStatusChanged` | `Network.framework` `nw_path_monitor` |
| Headphones (BT) | `Devices::Enumeration::DeviceWatcher` | IOBluetooth notifications |
| Caps Lock / layout | `GetKeyState(VK_CAPITAL)`, `GetKeyboardLayout` | `CGEventSourceFlagsState`, TIS input source notifications |
| Full-screen app | `SHQueryUserNotificationState` | `NSWorkspace` active-space / `CGWindowListCopyWindowInfo` |
| Window above menu bar | n/a | `NSWindow.setLevel(NSStatusWindowLevel + 1)` |

Prefer OS **change notifications** over polling. Polling is acceptable as a first version with a
≥ 500 ms interval; note the limit in `docs/PRODUCT.md`.

## Rules

- No `unwrap()`/`expect()` on OS calls — map to `Option`/`Result`.
- Long or blocking calls never run on the main thread.
- Convert units at the boundary (WinRT `TimeSpan` = 100 ns ticks → ms) so the UI sees one format.
- Payload types live in `mod.rs`, not in OS files, so both OSes serialize identically.

## Verify

- `cargo check` on your OS. Can't build the other OS? Run
  `cargo check --target x86_64-apple-darwin` / `x86_64-pc-windows-msvc` if the target is installed,
  and say clearly in your summary which OS was **not** run.
