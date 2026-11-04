use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{
    AppHandle, Emitter, LogicalPosition, Manager, Monitor, PhysicalPosition, Theme, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

#[derive(Clone, Copy, Default, Deserialize)]
pub struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Default)]
pub struct HitRect(Mutex<Rect>);

static FOLLOW_CURSOR: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn notch_set_hit_rect(rect: Rect, state: tauri::State<HitRect>) {
    *state.0.lock().unwrap() = rect;
}

#[tauri::command]
pub fn notch_set_display(app: AppHandle, mode: &str) -> Result<(), String> {
    FOLLOW_CURSOR.store(
        match mode {
            "main" => false,
            "cursor" => true,
            other => return Err(format!("unknown display mode: {other}")),
        },
        Ordering::Relaxed,
    );
    let win = app.get_webview_window("notch").ok_or("no notch window")?;
    let monitor = if FOLLOW_CURSOR.load(Ordering::Relaxed) { cursor_monitor(&win) } else { win.primary_monitor().ok().flatten() };
    if let Some(m) = monitor {
        place(&win, &m).map_err(|e| e.to_string())?;
    }
    Ok(())
}

static BEFORE: AtomicIsize = AtomicIsize::new(0);

pub fn take_keys(app: &AppHandle) {
    let Some(win) = app.get_webview_window("notch") else { return };
    #[cfg(windows)]
    BEFORE.store(unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }.0 as isize, Ordering::Relaxed);
    let _ = win.set_focus();
    let _ = app.emit("notch:keys", ());
}

#[tauri::command]
pub fn notch_release_keys() {
    let before = BEFORE.swap(0, Ordering::Relaxed);
    #[cfg(windows)]
    if before != 0 {
        use windows::Win32::Foundation::HWND;
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(HWND(before as _)) };
    }
    let _ = before;
}

#[tauri::command]
pub fn app_launch() -> &'static str {
    if std::env::args().any(|a| a == "--autostart") { "login" } else { "manual" }
}

#[tauri::command]
pub async fn settings_open(app: AppHandle) -> Result<(), String> {
    open_settings(&app).map_err(|e| e.to_string())
}

fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("settings") {
        w.unminimize()?;
        w.show()?;
        return w.set_focus();
    }
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings".into()))
        .title("Zaber Settings")
        .inner_size(920.0, 680.0)
        .min_inner_size(560.0, 480.0)
        .theme(Some(Theme::Dark))
        .build()?;
    Ok(())
}

pub fn setup(app: &AppHandle, win: &WebviewWindow) -> tauri::Result<()> {
    {
        use tauri_plugin_autostart::ManagerExt;
        if app.autolaunch().is_enabled().unwrap_or(false) {
            let _ = app.autolaunch().enable();
        }
    }
    if let Some(m) = win.primary_monitor()? {
        place(win, &m)?;
    }
    win.set_ignore_cursor_events(true)?;
    watch_hover(win.clone());

    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Zaber", true, None::<&str>)?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("Zaber")
        .menu(&Menu::with_items(app, &[&settings, &quit])?)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "settings" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = open_settings(&app);
                });
            }
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn place(win: &WebviewWindow, monitor: &Monitor) -> tauri::Result<()> {
    let (m, w) = (monitor.position(), win.outer_size()?);
    let x = m.x + (monitor.size().width as i32 - w.width as i32) / 2;
    win.set_position(PhysicalPosition::new(x, m.y))
}

fn cursor_monitor(win: &WebviewWindow) -> Option<Monitor> {
    let c = win.cursor_position().ok()?;
    win.monitor_from_point(c.x, c.y).ok().flatten()
}

const NEAR: f64 = 300.0;

fn near(p: LogicalPosition<f64>, r: Rect) -> Option<i8> {
    let dx = p.x - (r.x + r.w / 2.0);
    (p.y >= 0.0 && p.y < NEAR && dx.abs() < NEAR).then(|| (dx / NEAR * 2.5).round().clamp(-2.0, 2.0) as i8)
}

fn watch_hover(win: WebviewWindow) {
    thread::spawn(move || {
        let mut hovering = false;
        let mut looking: Option<i8> = None;
        let mut display: Option<String> = None;
        let mut tick = 0u64;
        #[cfg(windows)]
        let mut on_top = (0isize, 0u8);
        loop {
            thread::sleep(Duration::from_millis(50));
            tick = tick.wrapping_add(1);
            #[cfg(windows)]
            keep_on_top(&win, &mut on_top);
            if tick.is_multiple_of(20)
                && FOLLOW_CURSOR.load(Ordering::Relaxed)
                && let Some(m) = cursor_monitor(&win)
                && display.as_deref() != m.name().map(String::as_str)
            {
                display = m.name().cloned();
                let _ = place(&win, &m);
            }
            let (Ok(cursor), Ok(origin), Ok(scale)) =
                (win.cursor_position(), win.outer_position(), win.scale_factor())
            else {
                continue;
            };
            let p: LogicalPosition<f64> =
                PhysicalPosition::new(cursor.x - origin.x as f64, cursor.y - origin.y as f64).to_logical(scale);
            let r = *win.state::<HitRect>().0.lock().unwrap();
            let inside = p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;
            let step = near(p, r);
            if step != looking {
                looking = step;
                let _ = win.emit("notch:near", step);
            }
            if inside != hovering {
                hovering = inside;
                let _ = win.set_ignore_cursor_events(!inside);
                let _ = win.emit("notch:hover", inside);
            }
        }
    });
}

#[cfg(windows)]
fn keep_on_top(win: &WebviewWindow, state: &mut (isize, u8)) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, HWND_TOPMOST, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
    };
    let fg = unsafe { GetForegroundWindow() }.0 as isize;
    if fg != state.0 {
        *state = (fg, 10);
    }
    if state.1 == 0 {
        return;
    }
    state.1 -= 1;
    if let Ok(h) = win.hwnd() {
        unsafe {
            let flags = SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_ASYNCWINDOWPOS;
            let _ = SetWindowPos(HWND(h.0), Some(HWND_TOPMOST), 0, 0, 0, 0, flags);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LogicalPosition, Rect, near};

    #[test]
    fn eyes_look_toward_a_near_cursor_only() {
        let r = Rect { x: 290.0, y: 0.0, w: 180.0, h: 32.0 };
        let at = |x: f64, y: f64| near(LogicalPosition::new(x, y), r);
        assert_eq!(at(380.0, 100.0), Some(0));
        assert_eq!(at(140.0, 50.0), Some(-2));
        assert_eq!(at(560.0, 50.0), Some(2));
        assert_eq!(at(380.0, 400.0), None, "too far below");
        assert_eq!(at(900.0, 10.0), None, "too far to the side");
    }
}
