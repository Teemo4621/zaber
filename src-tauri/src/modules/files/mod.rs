use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewFile {
    pub source: &'static str,
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub size: Option<(u32, u32)>,
}

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloading {
    pub name: String,
    pub bytes: u64,
    pub speed: u64,
}

const STALLED: Duration = Duration::from_secs(20);

const PARTIAL: [&str; 6] = ["crdownload", "part", "partial", "tmp", "download", "opdownload"];
static SHOWN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn check(path: &str) -> Result<(), String> {
    if !SHOWN.lock().unwrap().iter().any(|p| p == path) {
        return Err("not a file Zaber announced".into());
    }
    if !Path::new(path).exists() {
        return Err("file was moved or deleted".into());
    }
    Ok(())
}

#[tauri::command]
pub fn files_open(app: AppHandle, path: String, reveal: bool) -> Result<(), String> {
    check(&path)?;
    let opener = app.opener();
    if reveal { opener.reveal_item_in_dir(&path) } else { opener.open_path(&path, None::<&str>) }.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn files_icon(path: String) -> Result<Option<String>, String> {
    check(&path)?;
    Ok(super::shelf::shell_icon(path).await)
}

fn screenshot_dir(app: &AppHandle) -> Option<PathBuf> {
    if cfg!(target_os = "macos") { app.path().desktop_dir().ok() } else { app.path().picture_dir().ok().map(|p| p.join("Screenshots")) }
}

fn partial_base(name: &str) -> Option<&str> {
    name.rsplit_once('.').filter(|(_, ext)| PARTIAL.contains(&ext.to_ascii_lowercase().as_str())).map(|(base, _)| base)
}

struct Partial {
    bytes: u64,
    at: Instant,
    grew: Instant,
    speed: f64,
}

fn is_partial(name: &str) -> bool {
    name.starts_with('.') || name.starts_with("~$") || name.rsplit_once('.').is_some_and(|(_, ext)| PARTIAL.contains(&ext.to_ascii_lowercase().as_str()))
}

fn png_size(path: &Path) -> Option<(u32, u32)> {
    let mut head = [0u8; 24];
    File::open(path).ok()?.read_exact(&mut head).ok()?;
    let be = |i: usize| u32::from_be_bytes([head[i], head[i + 1], head[i + 2], head[i + 3]]);
    (head[..8] == *b"\x89PNG\r\n\x1a\n").then(|| (be(16), be(20)))
}

struct Watch {
    source: &'static str,
    dir: PathBuf,
    changed: Option<SystemTime>,
    seen: HashSet<String>,
    partials: HashMap<String, Partial>,
}

impl Watch {
    fn new(source: &'static str, dir: PathBuf) -> Self {
        Self { source, dir, changed: None, seen: HashSet::new(), partials: HashMap::new() }
    }

    fn progress(&mut self) -> Vec<Downloading> {
        let now = Instant::now();
        let mut out: Vec<Downloading> = Vec::new();
        for (name, p) in &mut self.partials {
            let Ok(bytes) = fs::metadata(self.dir.join(name)).map(|m| m.len()) else { continue };
            let secs = now.duration_since(p.at).as_secs_f64().max(0.001);
            let rate = bytes.saturating_sub(p.bytes) as f64 / secs;
            p.speed = if p.speed == 0.0 { rate } else { 0.6 * p.speed + 0.4 * rate };
            if bytes > p.bytes {
                p.grew = now;
            }
            p.bytes = bytes;
            p.at = now;
            if now.duration_since(p.grew) < STALLED {
                out.push(Downloading { name: partial_base(name).unwrap_or(name).to_string(), bytes, speed: p.speed as u64 });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    fn names(&self) -> Vec<String> {
        fs::read_dir(&self.dir).into_iter().flatten().flatten().filter_map(|e| e.file_name().into_string().ok()).collect()
    }

    fn poll(&mut self) -> Option<NewFile> {
        let changed = fs::metadata(&self.dir).and_then(|m| m.modified()).ok();
        if changed == self.changed {
            return None;
        }
        let first = self.changed.is_none();
        self.changed = changed;
        let names = self.names();
        if self.source == "download" {
            self.partials.retain(|n, _| names.contains(n));
            let now = Instant::now();
            for n in names.iter().filter(|n| partial_base(n).is_some()) {
                let grew = if first { now.checked_sub(STALLED).unwrap_or(now) } else { now };
                let bytes = fs::metadata(self.dir.join(n)).map(|m| m.len()).unwrap_or(0);
                self.partials.entry(n.clone()).or_insert(Partial { bytes, at: now, grew, speed: 0.0 });
            }
        }
        let mut fresh: Vec<(SystemTime, PathBuf, u64)> = Vec::new();
        for name in names {
            if !self.seen.insert(name.clone()) || first || is_partial(&name) {
                continue;
            }
            let path = self.dir.join(&name);
            if let Ok(m) = fs::metadata(&path)
                && m.is_file()
                && m.len() > 0
            {
                fresh.push((m.modified().unwrap_or(SystemTime::UNIX_EPOCH), path, m.len()));
            }
        }
        let (_, path, bytes) = fresh.into_iter().max_by_key(|f| f.0)?;
        Some(NewFile {
            source: self.source,
            name: path.file_name()?.to_string_lossy().into_owned(),
            size: png_size(&path),
            path: path.to_string_lossy().into_owned(),
            bytes,
        })
    }
}

pub fn start(app: AppHandle) {
    let mut watches: Vec<Watch> = [("screenshot", screenshot_dir(&app)), ("download", app.path().download_dir().ok())]
        .into_iter()
        .filter_map(|(source, dir)| Some(Watch::new(source, dir?)))
        .collect();
    thread::spawn(move || {
        let mut downloading: Vec<Downloading> = Vec::new();
        loop {
            for w in &mut watches {
                if w.source == "download" {
                    let now = w.progress();
                    if now != downloading {
                        let _ = app.emit("files:downloading", &now);
                        downloading = now;
                    }
                }
                let Some(f) = w.poll() else { continue };
                if f.source == "screenshot" && f.size.is_none() {
                    continue;
                }
                let mut shown = SHOWN.lock().unwrap();
                shown.push(f.path.clone());
                let keep = shown.len().saturating_sub(20);
                shown.drain(..keep);
                drop(shown);
                let _ = app.emit("files:new", f);
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finished_downloads_only() {
        assert!(is_partial("report.pdf.crdownload"));
        assert!(is_partial("movie.mkv.PART"));
        assert!(is_partial(".DS_Store"));
        assert!(!is_partial("report.pdf"));
    }

    #[test]
    fn announces_the_new_file_not_the_old_ones() {
        let dir = std::env::temp_dir().join(format!("zaber-files-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("old.txt"), "x").unwrap();
        let mut w = Watch::new("download", dir.clone());
        assert!(w.poll().is_none(), "first look only takes stock");
        fs::write(dir.join("new.zip.crdownload"), "x").unwrap();
        w.changed = Some(SystemTime::UNIX_EPOCH);
        assert!(w.poll().is_none(), "still downloading");
        fs::write(dir.join("new.zip.crdownload"), "xxxx").unwrap();
        let p = w.progress();
        assert_eq!((p.len(), p[0].name.as_str(), p[0].bytes), (1, "new.zip", 4), "a growing download shows");
        fs::rename(dir.join("new.zip.crdownload"), dir.join("new.zip")).unwrap();
        w.changed = Some(SystemTime::UNIX_EPOCH);
        assert_eq!(w.poll().map(|f| f.name), Some("new.zip".into()));
        fs::remove_dir_all(dir).unwrap();
    }
}
