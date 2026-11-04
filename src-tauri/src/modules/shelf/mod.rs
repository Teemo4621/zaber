#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(windows)]
use self::windows as platform;

use std::path::Path;

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::store;

fn check(app: &AppHandle, path: &str) -> Result<(), String> {
    let on_shelf = store::read(app, "shelf")?
        .as_array()
        .is_some_and(|items| items.iter().any(|i| i.get("path").and_then(|p| p.as_str()) == Some(path)));
    if !on_shelf {
        return Err("file is not on the shelf".into());
    }
    if !Path::new(path).exists() {
        return Err("file was moved or deleted".into());
    }
    Ok(())
}

#[tauri::command]
pub fn shelf_open(app: AppHandle, path: String, reveal: bool) -> Result<(), String> {
    check(&app, &path)?;
    let opener = app.opener();
    if reveal { opener.reveal_item_in_dir(&path) } else { opener.open_path(&path, None::<&str>) }.map_err(|e| e.to_string())
}

pub async fn shell_icon(path: String) -> Option<String> {
    platform::icon(path).await
}

#[tauri::command]
pub async fn shelf_icon(app: AppHandle, path: String) -> Result<Option<String>, String> {
    check(&app, &path)?;
    Ok(platform::icon(path).await)
}
