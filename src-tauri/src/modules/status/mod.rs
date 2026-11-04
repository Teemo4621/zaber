#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub volume: Option<u8>,
    pub muted: bool,
    pub audio_device: Option<String>,
    pub caps_lock: bool,
    pub layout: Option<String>,
    pub charging: Option<bool>,
    pub battery: Option<u8>,
    pub battery_minutes: Option<u32>,
    pub online: bool,
    pub fullscreen: bool,
    pub accent: Option<String>,
    pub mic_muted: Option<bool>,
    pub mic_apps: Vec<String>,
    pub camera_apps: Vec<String>,
}

static LAST: Mutex<Option<Status>> = Mutex::new(None);

#[tauri::command]
pub fn status_now() -> Option<Status> {
    LAST.lock().unwrap().clone()
}

#[tauri::command]
pub fn status_volume_keys(app: AppHandle, on: bool) {
    platform::take_volume_keys(&app, on);
}

pub fn toggle_mic() {
    thread::spawn(platform::toggle_mic);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VolumeKey {
    Up,
    Down,
    Mute,
}

pub fn next_volume(level: f32, up: bool) -> f32 {
    let step = (level * 50.0).round() + if up { 1.0 } else { -1.0 };
    step.clamp(0.0, 50.0) / 50.0
}

fn limit(before: Option<u8>, now: Option<u8>, up: bool, down: bool) -> Option<&'static str> {
    match (before, now) {
        (Some(100), Some(100)) if up => Some("up"),
        (Some(0), Some(0)) if down => Some("down"),
        _ => None,
    }
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut probe = platform::Probe::new();
        let mut tick = 0u64;
        loop {
            let now = probe.poll(tick);
            tick = tick.wrapping_add(1);
            let mut last = LAST.lock().unwrap();
            let (up, down) = if platform::volume_keys_taken() { (false, false) } else { platform::volume_keys() };
            if let Some(dir) = limit(last.as_ref().and_then(|l| l.volume), now.volume, up, down) {
                let _ = app.emit("status:limit", dir);
            }
            if last.as_ref() != Some(&now) {
                let _ = app.emit("status:update", &now);
                *last = Some(now);
            }
            drop(last);
            thread::sleep(Duration::from_millis(50));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{limit, next_volume};

    #[test]
    fn volume_steps_like_windows() {
        assert_eq!(next_volume(0.50, true), 0.52);
        assert_eq!(next_volume(0.51, false), 0.5);
        assert_eq!(next_volume(1.0, true), 1.0);
        assert_eq!(next_volume(0.0, false), 0.0);
        assert_eq!(next_volume(0.99, true), 1.0);
    }

    #[test]
    fn limit_only_when_already_at_the_end() {
        assert_eq!(limit(Some(100), Some(100), true, false), Some("up"));
        assert_eq!(limit(Some(96), Some(100), true, false), None);
        assert_eq!(limit(Some(0), Some(0), false, true), Some("down"));
        assert_eq!(limit(Some(100), Some(100), false, true), None);
        assert_eq!(limit(None, None, true, true), None);
    }
}
