use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionMediaProperties as Props,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Storage::Streams::{Buffer, DataReader, InputStreamOptions};

use super::{Action, NowPlaying, choose};

static ARTWORK: Mutex<Option<(String, Option<String>)>> = Mutex::new(None);
static SHOWN: Mutex<Option<String>> = Mutex::new(None);

fn session() -> windows::core::Result<Session> {
    let mgr = Manager::RequestAsync()?.get()?;
    let sessions: Vec<Session> = mgr.GetSessions()?.into_iter().collect();
    let info: Vec<(String, bool)> = sessions
        .iter()
        .map(|s| {
            let id = s.SourceAppUserModelId().map(|h| h.to_string()).unwrap_or_default();
            let playing = s
                .GetPlaybackInfo()
                .and_then(|p| p.PlaybackStatus())
                .is_ok_and(|st| st == Status::Playing);
            (id, playing)
        })
        .collect();
    let current = mgr.GetCurrentSession().and_then(|s| s.SourceAppUserModelId()).map(|h| h.to_string()).ok();
    let mut shown = SHOWN.lock().unwrap();
    let i = choose(&info, shown.as_deref(), current.as_deref()).ok_or_else(windows::core::Error::empty)?;
    *shown = Some(info[i].0.clone());
    Ok(sessions[i].clone())
}

fn read_artwork(props: &Props) -> windows::core::Result<String> {
    let stream = props.Thumbnail()?.OpenReadAsync()?.get()?;
    let size = stream.Size()? as u32;
    let buf = stream.ReadAsync(&Buffer::Create(size)?, size, InputStreamOptions::None)?.get()?;
    let mut bytes = vec![0u8; buf.Length()? as usize];
    DataReader::FromBuffer(&buf)?.ReadBytes(&mut bytes)?;
    Ok(format!("data:{};base64,{}", stream.ContentType()?, STANDARD.encode(bytes)))
}

fn artwork(key: String, props: &Props) -> Option<String> {
    let mut cache = ARTWORK.lock().unwrap();
    if let Some((k, art)) = cache.as_ref()
        && *k == key
    {
        return art.clone();
    }
    let art = read_artwork(props).ok();
    *cache = Some((key, art.clone()));
    art
}

fn filetime_now() -> i64 {
    let unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    (unix.as_nanos() / 100) as i64 + 116_444_736_000_000_000
}

pub fn now_playing() -> Option<NowPlaying> {
    let read = || -> windows::core::Result<NowPlaying> {
        let s = session()?;
        let props = s.TryGetMediaPropertiesAsync()?.get()?;
        let timeline = s.GetTimelineProperties()?;
        let (title, artist, album) =
            (props.Title()?.to_string(), props.Artist()?.to_string(), props.AlbumTitle()?.to_string());
        let source_app = s.SourceAppUserModelId()?.to_string();
        let artwork = artwork(format!("{source_app}\0{title}\0{artist}\0{album}"), &props);
        let playing = s.GetPlaybackInfo()?.PlaybackStatus()? == Status::Playing;
        let duration_ms = timeline.EndTime()?.Duration / 10_000;
        let mut position_ms = timeline.Position()?.Duration / 10_000;
        if playing {
            position_ms += (filetime_now() - timeline.LastUpdatedTime()?.UniversalTime).max(0) / 10_000;
            if duration_ms > 0 {
                position_ms = position_ms.min(duration_ms);
            }
        }
        Ok(NowPlaying { title, artist, album, playing, position_ms, duration_ms, source_app, artwork })
    };
    read().ok()
}

pub fn control(action: Action) -> Result<(), String> {
    let run = || -> windows::core::Result<bool> {
        let s = session()?;
        match action {
            Action::PlayPause => s.TryTogglePlayPauseAsync()?.get(),
            Action::Next => s.TrySkipNextAsync()?.get(),
            Action::Previous => s.TrySkipPreviousAsync()?.get(),
            Action::Seek(ms) => s.TryChangePlaybackPositionAsync(ms * 10_000)?.get(),
        }
    };
    run().map(|_| ()).map_err(|e| e.to_string())
}
