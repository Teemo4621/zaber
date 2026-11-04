use super::{Action, NowPlaying};

pub fn now_playing() -> Option<NowPlaying> {
    None
}

pub fn control(_action: Action) -> Result<(), String> {
    Err("media control is not implemented on macOS yet".into())
}
