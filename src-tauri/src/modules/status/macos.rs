use super::Status;

pub struct Probe;

impl Probe {
    pub fn new() -> Self {
        Self
    }

    pub fn poll(&mut self, _tick: u64) -> Status {
        Status { online: true, ..Status::default() }
    }
}

pub fn toggle_mic() {}

pub fn volume_keys() -> (bool, bool) {
    (false, false)
}

pub fn take_volume_keys(_app: &tauri::AppHandle, _on: bool) {}

pub fn volume_keys_taken() -> bool {
    false
}
