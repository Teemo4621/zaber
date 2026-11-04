use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

const STORES: [&str; 5] = ["settings", "todo", "water", "shelf", "notes"];
const MAX_BYTES: usize = 1 << 20;

fn path(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    if !STORES.contains(&name) {
        return Err(format!("unknown store: {name}"));
    }
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{name}.json")))
}

pub fn read(app: &AppHandle, name: &str) -> Result<Value, String> {
    match fs::read_to_string(path(app, name)?) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Clone, Serialize)]
struct Update<'a> {
    name: &'a str,
    value: &'a Value,
}

#[tauri::command]
pub fn store_get(app: AppHandle, name: &str) -> Result<Value, String> {
    read(&app, name)
}

#[tauri::command]
pub fn store_set(app: AppHandle, name: &str, value: Value) -> Result<(), String> {
    let file = path(&app, name)?;
    let text = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    if text.len() > MAX_BYTES {
        return Err(format!("{name} is too large ({} bytes)", text.len()));
    }
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &file).map_err(|e| e.to_string())?;
    let _ = app.emit("store:update", Update { name, value: &value });
    Ok(())
}

#[tauri::command]
pub fn store_open_dir(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener().open_path(dir.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}
