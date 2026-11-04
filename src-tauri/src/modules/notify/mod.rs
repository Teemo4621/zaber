#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use std::collections::HashSet;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub id: u32,
    pub app: String,
    pub app_id: String,
    pub title: String,
    pub body: String,
}

static SENDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn notify_open(app_id: String) -> Result<(), String> {
    if app_id.is_empty() || !SENDERS.lock().unwrap().contains(&app_id) {
        return Err("unknown app".into());
    }
    platform::open(&app_id)
}

#[tauri::command]
pub async fn notify_icon(app_id: String) -> Result<Option<String>, String> {
    if app_id.is_empty() || !SENDERS.lock().unwrap().contains(&app_id) {
        return Err("unknown app".into());
    }
    Ok(match platform::icon_path(&app_id) {
        Some(path) => crate::modules::shelf::shell_icon(path).await,
        None => None,
    })
}

#[tauri::command]
pub fn notify_access() -> &'static str {
    platform::access()
}

fn fresh(seen: Option<&HashSet<u32>>, list: Vec<Notice>) -> Vec<Notice> {
    match seen {
        Some(seen) => list.into_iter().filter(|n| !seen.contains(&n.id)).collect(),
        None => Vec::new(),
    }
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        platform::request_access();
        let mut seen: Option<HashSet<u32>> = None;
        loop {
            if let Some(list) = platform::list() {
                let ids = list.iter().map(|n| n.id).collect();
                for n in fresh(seen.as_ref(), list) {
                    let mut senders = SENDERS.lock().unwrap();
                    if !n.app_id.is_empty() && !senders.contains(&n.app_id) {
                        senders.push(n.app_id.clone());
                    }
                    drop(senders);
                    let _ = app.emit("notify:new", n);
                }
                seen = Some(ids);
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: u32) -> Notice {
        Notice { id, app: String::new(), app_id: String::new(), title: String::new(), body: String::new() }
    }

    #[test]
    fn first_poll_shows_nothing_then_only_new() {
        assert!(fresh(None, vec![n(1), n(2)]).is_empty());
        let seen = HashSet::from([1, 2]);
        let out = fresh(Some(&seen), vec![n(1), n(2), n(3)]);
        assert_eq!(out.iter().map(|n| n.id).collect::<Vec<_>>(), vec![3]);
    }
}
