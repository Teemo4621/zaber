#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Drive,
    Mouse,
    Keyboard,
    Headphones,
}

#[derive(Clone, Debug)]
pub struct Device {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub detail: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub kind: Kind,
    pub name: String,
    pub detail: Option<String>,
    pub path: Option<String>,
    pub connected: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LowBattery {
    pub kind: Kind,
    pub name: String,
    pub level: u8,
}

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub kind: Kind,
    pub name: String,
    pub level: u8,
}

static LEVELS: Mutex<Vec<Level>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn devices_levels() -> Vec<Level> {
    LEVELS.lock().unwrap().clone()
}

static DRIVES: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn devices_open(app: AppHandle, path: String) -> Result<(), String> {
    if !DRIVES.lock().unwrap().contains(&path) {
        return Err("not a connected drive".into());
    }
    app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())
}

fn diff(before: &HashMap<String, Device>, now: &HashMap<String, Device>) -> (Vec<Device>, Vec<Device>) {
    let added = now.iter().filter(|(k, _)| !before.contains_key(*k)).map(|(_, d)| d.clone()).collect();
    let removed = before.iter().filter(|(k, _)| !now.contains_key(*k)).map(|(_, d)| d.clone()).collect();
    (added, removed)
}

fn battery_step(warned: &mut HashMap<String, u8>, id: &str, level: u8) -> bool {
    let step = if level <= 10 { 10 } else if level <= 20 { 20 } else { 100 };
    let last = warned.insert(id.to_string(), step).unwrap_or(100);
    step < last
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut probe = platform::Probe::new();
        let mut known: Option<HashMap<String, Device>> = None;
        let mut warned: HashMap<String, u8> = HashMap::new();
        let mut tick = 0u64;
        loop {
            let now: HashMap<String, Device> = probe.devices(tick).into_iter().map(|d| (d.id.clone(), d)).collect();
            *DRIVES.lock().unwrap() = now.values().filter_map(|d| d.path.clone()).collect();
            if let Some(before) = &known {
                let (added, removed) = diff(before, &now);
                for (d, connected) in added.into_iter().map(|d| (d, true)).chain(removed.into_iter().map(|d| (d, false))) {
                    let _ = app.emit(
                        "devices:change",
                        Change { kind: d.kind, name: d.name, detail: d.detail, path: d.path, connected },
                    );
                }
            }
            known = Some(now);

            if tick.is_multiple_of(10) {
                let levels = probe.batteries();
                let now: Vec<Level> = levels.iter().map(|(_, kind, name, level)| Level { kind: *kind, name: name.clone(), level: *level }).collect();
                let mut last = LEVELS.lock().unwrap();
                if *last != now {
                    let _ = app.emit("devices:levels", &now);
                    *last = now;
                }
                drop(last);
                let present: HashSet<&String> = levels.iter().map(|(id, ..)| id).collect();
                warned.retain(|id, _| present.contains(id));
                for (id, kind, name, level) in &levels {
                    if battery_step(&mut warned, id, *level) {
                        let _ = app.emit("devices:battery", LowBattery { kind: *kind, name: name.clone(), level: *level });
                    }
                }
            }
            tick = tick.wrapping_add(1);
            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(id: &str) -> Device {
        Device { id: id.into(), kind: Kind::Mouse, name: id.into(), detail: None, path: None }
    }

    #[test]
    fn diff_finds_connected_and_removed() {
        let before: HashMap<_, _> = [("a", dev("a")), ("b", dev("b"))].into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let now: HashMap<_, _> = [("b", dev("b")), ("c", dev("c"))].into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let (added, removed) = diff(&before, &now);
        assert_eq!(added.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["c"]);
        assert_eq!(removed.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["a"]);
    }

    #[test]
    fn battery_warns_at_20_then_10_and_rearms() {
        let mut w = HashMap::new();
        assert!(!battery_step(&mut w, "x", 60));
        assert!(battery_step(&mut w, "x", 19));
        assert!(!battery_step(&mut w, "x", 15));
        assert!(battery_step(&mut w, "x", 9));
        assert!(!battery_step(&mut w, "x", 5));
        assert!(!battery_step(&mut w, "x", 80));
        assert!(battery_step(&mut w, "x", 20));
    }
}
