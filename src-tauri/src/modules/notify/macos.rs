use super::Notice;

pub fn request_access() {}

pub fn access() -> &'static str {
    "unsupported"
}

pub fn list() -> Option<Vec<Notice>> {
    None
}

pub fn icon_path(_app_id: &str) -> Option<String> {
    None
}

pub fn open(_app_id: &str) -> Result<(), String> {
    Err("not supported on macOS".into())
}
