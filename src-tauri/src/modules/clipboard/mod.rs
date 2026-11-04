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

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Copied {
    Text { text: String },
    Link { text: String },
    Image { width: u32, height: u32 },
    Files { name: String, count: u32 },
}

#[cfg_attr(not(windows), allow(dead_code))]
const MAX_CHARS: usize = 80;

#[cfg_attr(not(windows), allow(dead_code))]
pub fn summarize(text: &str) -> Option<Copied> {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let short = |s: &str| {
        let mut out: String = s.chars().take(MAX_CHARS).collect();
        if s.chars().count() > MAX_CHARS {
            out.push('…');
        }
        out
    };
    if let Some(rest) = line.strip_prefix("https://").or_else(|| line.strip_prefix("http://"))
        && !rest.contains(char::is_whitespace)
    {
        let rest = rest.split(['?', '#']).next().unwrap_or(rest).trim_end_matches('/');
        return Some(Copied::Link { text: short(rest.strip_prefix("www.").unwrap_or(rest)) });
    }
    Some(Copied::Text { text: short(line) })
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut seen = platform::sequence();
        loop {
            thread::sleep(Duration::from_millis(200));
            let now = platform::sequence();
            if now == seen {
                continue;
            }
            seen = now;
            if let Some(c) = platform::read() {
                let _ = app.emit("clipboard:copied", c);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{Copied, summarize};

    #[test]
    fn first_line_and_links() {
        assert_eq!(summarize("\n  hello world \nsecond"), Some(Copied::Text { text: "hello world".into() }));
        assert_eq!(
            summarize("https://www.github.com/a/b/?tab=x"),
            Some(Copied::Link { text: "github.com/a/b".into() })
        );
        assert_eq!(summarize("http://x.com see this"), Some(Copied::Text { text: "http://x.com see this".into() }));
        assert_eq!(summarize("   \n "), None);
        let long = "ก".repeat(100);
        let Some(Copied::Text { text }) = summarize(&long) else { panic!() };
        assert_eq!(text.chars().count(), 81);
    }
}
