use std::fs;
use std::process::Command;

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::modules::claude;

pub fn clean(app: &AppHandle) {
    if app.path().home_dir().is_ok_and(|h| h.join(".claude").join("settings.json").exists()) {
        let _ = claude::claude_hook(app.clone(), false);
    }
    let _ = app.autolaunch().disable();
}

#[tauri::command]
pub fn app_uninstall(app: AppHandle) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if cfg!(windows) {
        let uninstaller = exe.with_file_name("uninstall.exe");
        if !uninstaller.exists() {
            return Err("Zaber isn't installed from the installer".into());
        }
        Command::new(uninstaller).spawn().map_err(|e| e.to_string())?;
    } else {
        let bundle = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).ok_or("Zaber isn't running from Zaber.app")?;
        clean(&app);
        if let Ok(dir) = app.path().app_data_dir() {
            let _ = fs::remove_dir_all(dir);
        }
        let script = format!("tell application \"Finder\" to delete POSIX file \"{}\"", bundle.display());
        Command::new("osascript").args(["-e", &script]).status().map_err(|e| e.to_string())?;
    }
    app.exit(0);
    Ok(())
}
