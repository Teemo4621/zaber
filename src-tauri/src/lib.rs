mod modules;
mod notch;
mod store;
mod uninstall;

use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg(not(target_os = "macos"))]
const KEYS: (&str, &str) = ("Ctrl+Alt+Space", "Ctrl+Alt+M");
#[cfg(target_os = "macos")]
const KEYS: (&str, &str) = ("Ctrl+Alt+N", "Ctrl+Alt+M");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_drag::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if KEYS.1.parse::<Shortcut>().is_ok_and(|mic| &mic == shortcut) {
                        modules::status::toggle_mic();
                    } else {
                        notch::take_keys(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))
        .manage(notch::HitRect::default())
        .invoke_handler(tauri::generate_handler![
            notch::notch_set_hit_rect,
            notch::notch_set_display,
            notch::settings_open,
            notch::app_launch,
            uninstall::app_uninstall,
            notch::notch_release_keys,
            store::store_get,
            store::store_set,
            store::store_open_dir,
            modules::media::media_now,
            modules::media::media_control,
            modules::calendar::calendar_events,
            modules::shelf::shelf_open,
            modules::shelf::shelf_icon,
            modules::devices::devices_open,
            modules::devices::devices_levels,
            modules::files::files_open,
            modules::files::files_icon,
            modules::notify::notify_open,
            modules::notify::notify_icon,
            modules::claude::claude_answer,
            modules::claude::claude_set_wait,
            modules::claude::claude_hook,
            modules::claude::claude_hook_status,
            modules::claude::claude_usage,
            modules::status::status_now,
            modules::status::status_volume_keys,
            modules::notify::notify_access,
        ])
        .setup(|app| {
            if std::env::args().any(|a| a == "--uninstall") {
                uninstall::clean(app.handle());
                std::process::exit(0);
            }
            let win = app.get_webview_window("notch").expect("notch window in tauri.conf.json");
            notch::setup(app.handle(), &win)?;
            modules::media::start(app.handle().clone());
            modules::status::start(app.handle().clone());
            modules::notify::start(app.handle().clone());
            modules::brightness::start(app.handle().clone());
            modules::clipboard::start(app.handle().clone());
            modules::devices::start(app.handle().clone());
            modules::claude::start(app.handle().clone());
            modules::files::start(app.handle().clone());
            let _ = app.global_shortcut().register(KEYS.0);
            let _ = app.global_shortcut().register(KEYS.1);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
