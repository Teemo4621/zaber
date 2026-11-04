#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub title: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub all_day: bool,
    pub location: String,
    pub link: Option<String>,
    pub color: String,
}

pub fn first_url(text: &str) -> Option<String> {
    let start = text.find("https://")?;
    let url: String = text[start..]
        .chars()
        .take_while(|c| !c.is_whitespace() && !matches!(c, '"' | '<' | '>' | ')'))
        .collect();
    (url.len() > "https://".len()).then_some(url)
}

#[tauri::command]
pub fn calendar_events(from_ms: i64, days: i64) -> Result<Vec<Event>, String> {
    if !(1..=7).contains(&days) || from_ms < 0 {
        return Err("calendar range must be 1–7 days from a valid time".into());
    }
    let mut events = platform::events(from_ms, days)?;
    events.sort_by_key(|e| e.start_ms);
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::first_url;

    #[test]
    fn finds_meeting_links_in_text() {
        assert_eq!(
            first_url("Join: <https://teams.microsoft.com/l/abc> now").as_deref(),
            Some("https://teams.microsoft.com/l/abc")
        );
        assert_eq!(first_url("Room 4B").as_deref(), None);
        assert_eq!(first_url("https://").as_deref(), None);
    }
}
