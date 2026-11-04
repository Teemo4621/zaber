#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    pub position_ms: i64,
    pub duration_ms: i64,
    pub source_app: String,
    pub artwork: Option<String>,
}

#[derive(Clone, Copy)]
pub enum Action {
    PlayPause,
    Next,
    Previous,
    Seek(i64),
}

pub fn choose(sessions: &[(String, bool)], shown: Option<&str>, current: Option<&str>) -> Option<usize> {
    let find = |id: Option<&str>| id.and_then(|id| sessions.iter().position(|(s, _)| s == id));
    let shown_i = find(shown);
    let current_i = find(current);
    if let Some(i) = shown_i.filter(|&i| sessions[i].1) {
        return Some(i);
    }
    if let Some(i) = current_i.filter(|&i| sessions[i].1) {
        return Some(i);
    }
    if let Some(i) = sessions.iter().position(|(_, playing)| *playing) {
        return Some(i);
    }
    shown_i.or(current_i).or(if sessions.is_empty() { None } else { Some(0) })
}

#[tauri::command]
pub fn media_now() -> Option<NowPlaying> {
    platform::now_playing()
}

#[tauri::command]
pub fn media_control(action: &str, position_ms: Option<i64>) -> Result<(), String> {
    let action = match (action, position_ms) {
        ("play_pause", _) => Action::PlayPause,
        ("next", _) => Action::Next,
        ("previous", _) => Action::Previous,
        ("seek", Some(ms)) if ms >= 0 => Action::Seek(ms),
        (other, _) => return Err(format!("invalid media action: {other}")),
    };
    platform::control(action)
}

fn changed(last: Option<&NowPlaying>, now: Option<&NowPlaying>, elapsed_ms: i64) -> bool {
    match (last, now) {
        (Some(a), Some(b)) => {
            let expected = a.position_ms + if a.playing { elapsed_ms } else { 0 };
            NowPlaying { position_ms: 0, ..a.clone() } != NowPlaying { position_ms: 0, ..b.clone() }
                || (b.position_ms - expected).abs() > 1500
        }
        (a, b) => a.is_some() != b.is_some(),
    }
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut last: Option<NowPlaying> = None;
        let mut sent = Instant::now();
        loop {
            let now = platform::now_playing();
            if changed(last.as_ref(), now.as_ref(), sent.elapsed().as_millis() as i64) {
                let _ = app.emit("media:update", &now);
                last = now;
                sent = Instant::now();
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{NowPlaying, changed, choose};

    fn track(position_ms: i64, playing: bool) -> NowPlaying {
        NowPlaying {
            title: "t".into(),
            artist: String::new(),
            album: String::new(),
            playing,
            position_ms,
            duration_ms: 200_000,
            source_app: "spotify".into(),
            artwork: None,
        }
    }

    #[test]
    fn position_ticking_on_is_not_a_change_but_a_seek_is() {
        let a = track(10_000, true);
        assert!(!changed(Some(&a), Some(&track(13_100, true)), 3_000));
        assert!(changed(Some(&a), Some(&track(60_000, true)), 3_000));
        assert!(changed(Some(&a), Some(&track(13_000, false)), 3_000));
        assert!(!changed(Some(&track(5_000, false)), Some(&track(5_000, false)), 9_000));
        assert!(changed(None, Some(&a), 0));
        assert!(!changed(None, None, 0));
    }

    fn s(list: &[(&str, bool)]) -> Vec<(String, bool)> {
        list.iter().map(|(id, p)| (id.to_string(), *p)).collect()
    }

    #[test]
    fn keeps_shown_while_it_plays_even_if_os_current_changes() {
        let list = s(&[("spotify", true), ("chrome", true)]);
        assert_eq!(choose(&list, Some("spotify"), Some("chrome")), Some(0));
    }

    #[test]
    fn switches_to_the_player_that_is_playing() {
        let list = s(&[("spotify", false), ("chrome", true)]);
        assert_eq!(choose(&list, Some("spotify"), Some("spotify")), Some(1));
    }

    #[test]
    fn stays_on_paused_shown_player_when_nothing_plays() {
        let list = s(&[("spotify", false), ("chrome", false)]);
        assert_eq!(choose(&list, Some("chrome"), Some("spotify")), Some(1));
    }

    #[test]
    fn falls_back_to_os_current_then_first() {
        let list = s(&[("a", false), ("b", false)]);
        assert_eq!(choose(&list, Some("gone"), Some("b")), Some(1));
        assert_eq!(choose(&list, None, None), Some(0));
        assert_eq!(choose(&[], None, None), None);
    }
}
