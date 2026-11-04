#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Brightness {
    pub level: u8,
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut probe = platform::Probe::new();
        let mut last: Option<u8> = None;
        let mut tick = 0u64;
        loop {
            if let Some(level) = probe.level(tick)
                && last != Some(level)
            {
                if last.is_some() {
                    let _ = app.emit("brightness:update", Brightness { level });
                }
                last = Some(level);
            }
            tick = tick.wrapping_add(1);
            thread::sleep(Duration::from_millis(250));
        }
    });
}
