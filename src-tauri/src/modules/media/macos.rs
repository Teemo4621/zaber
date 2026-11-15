use std::ptr::NonNull;
use std::sync::{Mutex, Once};
use std::thread;
use std::time::Instant;

use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::AnyThread;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSRunningApplication;
use objc2_foundation::{NSAppleScript, NSDictionary, NSDistributedNotificationCenter, NSNotification, NSNumber, NSString};

use super::{Action, NowPlaying};

const PLAYERS: [(&str, &str); 2] =
    [("com.apple.Music.playerInfo", "com.apple.Music"), ("com.spotify.client.PlaybackStateChanged", "com.spotify.client")];

struct Track {
    now: NowPlaying,
    at: Instant,
}

static NOW: Mutex<Option<Track>> = Mutex::new(None);
static SCRIPT: Mutex<()> = Mutex::new(());
static START: Once = Once::new();

fn listen() {
    DispatchQueue::main().exec_async(|| {
        let center = NSDistributedNotificationCenter::defaultCenter();
        for (name, app) in PLAYERS {
            let block = RcBlock::new(move |n: NonNull<NSNotification>| update(unsafe { n.as_ref() }, app));
            let token = unsafe { center.addObserverForName_object_queue_usingBlock(Some(&NSString::from_str(name)), None, None, &block) };
            std::mem::forget(token);
        }
    });
}

fn get(d: &NSDictionary<NSString, AnyObject>, key: &str) -> Option<objc2::rc::Retained<AnyObject>> {
    d.objectForKey(&NSString::from_str(key))
}

fn text(d: &NSDictionary<NSString, AnyObject>, key: &str) -> String {
    get(d, key).and_then(|v| v.downcast::<NSString>().ok()).map(|s| s.to_string()).unwrap_or_default()
}

fn number(d: &NSDictionary<NSString, AnyObject>, key: &str) -> Option<f64> {
    get(d, key).and_then(|v| v.downcast::<NSNumber>().ok()).map(|n| n.doubleValue())
}

fn position(t: &Track) -> i64 {
    let n = &t.now;
    if !n.playing {
        return n.position_ms;
    }
    let p = n.position_ms + t.at.elapsed().as_millis() as i64;
    if n.duration_ms > 0 { p.min(n.duration_ms) } else { p }
}

fn update(n: &NSNotification, app: &str) {
    let Some(info) = n.userInfo() else { return };
    let d: &NSDictionary<NSString, AnyObject> = unsafe { &*(objc2::rc::Retained::as_ptr(&info) as *const NSDictionary<NSString, AnyObject>) };
    let state = text(d, "Player State");
    let mut now = NOW.lock().unwrap();
    if state == "Stopped" {
        if now.as_ref().is_some_and(|t| t.now.source_app == app) {
            *now = None;
        }
        return;
    }
    let title = text(d, "Name");
    let same = now.as_ref().filter(|t| t.now.source_app == app && t.now.title == title);
    let position_ms = match number(d, "Playback Position") {
        Some(s) => (s * 1000.0) as i64,
        None => same.map(position).unwrap_or(0),
    };
    let duration_ms = number(d, "Duration").or_else(|| number(d, "Total Time")).unwrap_or(0.0) as i64;
    *now = Some(Track {
        now: NowPlaying {
            title,
            artist: text(d, "Artist"),
            album: text(d, "Album"),
            playing: state == "Playing",
            position_ms,
            duration_ms,
            source_app: app.to_string(),
            artwork: None,
        },
        at: Instant::now(),
    });
}

pub fn now_playing() -> Option<NowPlaying> {
    START.call_once(listen);
    let now = NOW.lock().unwrap();
    let t = now.as_ref()?;
    Some(NowPlaying { position_ms: position(t), ..t.now.clone() })
}

pub fn control(action: Action) -> Result<(), String> {
    let app = NOW.lock().unwrap().as_ref().map(|t| t.now.source_app.clone()).ok_or("nothing is playing")?;
    if NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(&app)).count() == 0 {
        return Err("the player is not running".into());
    }
    let command = match action {
        Action::PlayPause => "playpause".to_string(),
        Action::Next => "next track".to_string(),
        Action::Previous => "previous track".to_string(),
        Action::Seek(ms) => format!("set player position to {}", ms as f64 / 1000.0),
    };
    let source = format!("tell application id \"{app}\" to {command}");
    thread::spawn(move || {
        let _guard = SCRIPT.lock().unwrap();
        let Some(script) = NSAppleScript::initWithSource(NSAppleScript::alloc(), &NSString::from_str(&source)) else { return };
        let mut error = None;
        unsafe { script.executeAndReturnError(Some(&mut error)) };
        if error.is_none()
            && let Action::Seek(ms) = action
            && let Some(t) = NOW.lock().unwrap().as_mut()
        {
            t.now.position_ms = ms;
            t.at = Instant::now();
        }
    });
    Ok(())
}
