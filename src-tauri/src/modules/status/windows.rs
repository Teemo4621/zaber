use windows::Networking::Connectivity::{NetworkConnectivityLevel, NetworkInformation};
use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Globalization::LCIDToLocaleName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eConsole, eMultimedia, eRender};
use windows::Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, KEY_READ, RRF_RT_REG_QWORD, RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW};
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree, STGM_READ};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, VIRTUAL_KEY, VK_CAPITAL, VK_VOLUME_DOWN, VK_VOLUME_UP,
};
use windows::Win32::UI::Shell::{QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use std::thread;

use super::Status;

const SLOW_EVERY: u64 = 40;
const FULLSCREEN_AFTER: u8 = 20;
const LAYOUT_SETTLE: u8 = 10;

#[derive(Default)]
struct LayoutWatch {
    window: isize,
    actual: Option<String>,
    settle: u8,
}

impl LayoutWatch {
    fn next(&mut self, window: isize, actual: Option<String>, reported: &Option<String>) -> Option<String> {
        if window != self.window {
            self.window = window;
            self.settle = LAYOUT_SETTLE;
        }
        let changed = actual != self.actual;
        self.actual = actual.clone();
        if reported.is_none() {
            return actual;
        }
        if self.settle > 0 {
            self.settle -= 1;
            return reported.clone();
        }
        if changed { actual } else { reported.clone() }
    }
}

pub struct Probe {
    last: Status,
    enumerator: Option<IMMDeviceEnumerator>,
    volume: Option<IAudioEndpointVolume>,
    mic: Option<IAudioEndpointVolume>,
    ui: Option<UISettings>,
    busy_ticks: u8,
    layout: LayoutWatch,
}

impl Probe {
    pub fn new() -> Self {
        let enumerator = unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()
        };
        Self { last: Status::default(), enumerator, volume: None, mic: None, ui: UISettings::new().ok(), busy_ticks: 0, layout: LayoutWatch::default() }
    }

    pub fn poll(&mut self, tick: u64) -> Status {
        let mut s = self.last.clone();
        if tick.is_multiple_of(SLOW_EVERY) {
            self.refresh_device(&mut s);
            s.charging = None;
            s.battery = None;
            s.battery_minutes = None;
            let mut p = SYSTEM_POWER_STATUS::default();
            if unsafe { GetSystemPowerStatus(&mut p) }.is_ok() && p.BatteryFlag & 128 == 0 {
                s.charging = Some(p.ACLineStatus == 1);
                s.battery = (p.BatteryLifePercent <= 100).then_some(p.BatteryLifePercent);
                s.battery_minutes = (p.BatteryLifeTime != u32::MAX).then_some(p.BatteryLifeTime / 60);
            }
            s.online = NetworkInformation::GetInternetConnectionProfile()
                .and_then(|p| p.GetNetworkConnectivityLevel())
                .is_ok_and(|l| l == NetworkConnectivityLevel::InternetAccess);
            s.mic_apps = apps_using("microphone");
            s.camera_apps = apps_using("webcam");
            s.accent = self.ui.as_ref().and_then(|ui| ui.GetColorValue(UIColorType::Accent).ok()).map(|c| {
                format!("#{:02x}{:02x}{:02x}", c.R, c.G, c.B)
            });
        }
        unsafe {
            if let Some(v) = &self.volume {
                s.volume = v.GetMasterVolumeLevelScalar().ok().map(|f| (f * 100.0).round() as u8);
                s.muted = v.GetMute().is_ok_and(|m| m.as_bool());
            }
            s.mic_muted = self.mic.as_ref().and_then(|m| m.GetMute().ok()).map(|m| m.as_bool());
            s.caps_lock = GetKeyState(VK_CAPITAL.0 as i32) & 1 == 1;
            let window = GetForegroundWindow();
            let actual = layout_tag(GetKeyboardLayout(GetWindowThreadProcessId(window, None)).0 as usize);
            s.layout = self.layout.next(window.0 as isize, actual, &s.layout);
            let busy = SHQueryUserNotificationState()
                .is_ok_and(|q| q == QUNS_BUSY || q == QUNS_RUNNING_D3D_FULL_SCREEN || q == QUNS_PRESENTATION_MODE);
            self.busy_ticks = if busy { self.busy_ticks.saturating_add(1) } else { 0 };
            s.fullscreen = self.busy_ticks >= FULLSCREEN_AFTER;
        }
        self.last = s.clone();
        s
    }

    fn refresh_device(&mut self, s: &mut Status) {
        let Some(en) = &self.enumerator else { return };
        self.mic = unsafe { en.GetDefaultAudioEndpoint(eCapture, eConsole).and_then(|d| d.Activate(CLSCTX_ALL, None)).ok() };
        let found = unsafe {
            (|| -> windows::core::Result<(IAudioEndpointVolume, String)> {
                let dev = en.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
                let prop = dev.OpenPropertyStore(STGM_READ)?.GetValue(&PKEY_Device_FriendlyName)?;
                let name = PropVariantToStringAlloc(&prop)?;
                let text = name.to_string().unwrap_or_default();
                CoTaskMemFree(Some(name.0 as _));
                Ok((dev.Activate(CLSCTX_ALL, None)?, text))
            })()
        };
        match found {
            Ok((vol, name)) => {
                if s.audio_device.as_deref() != Some(&name) {
                    self.volume = Some(vol);
                    s.audio_device = Some(name);
                }
            }
            Err(_) => {
                self.volume = None;
                s.audio_device = None;
                s.volume = None;
            }
        }
    }
}

pub fn volume_keys() -> (bool, bool) {
    let pressed = |vk: VIRTUAL_KEY| unsafe { GetAsyncKeyState(vk.0 as i32) } as u16 & 0x8001 != 0;
    (pressed(VK_VOLUME_UP), pressed(VK_VOLUME_DOWN))
}

pub fn toggle_mic() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let Ok(en) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else { return };
        let Ok(mic) = en.GetDefaultAudioEndpoint(eCapture, eConsole).and_then(|d| d.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)) else { return };
        let muted = mic.GetMute().is_ok_and(|m| m.as_bool());
        let _ = mic.SetMute(!muted, std::ptr::null());
    }
}

const CONSENT: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";

fn apps_using(capability: &str) -> Vec<String> {
    let base = format!(r"{CONSENT}\{capability}");
    let mut apps = Vec::new();
    for key in [base.clone(), format!(r"{base}\NonPackaged")] {
        for name in subkeys(&key) {
            let path = format!(r"{key}\{name}");
            if name != "NonPackaged" && qword(&path, "LastUsedTimeStop") == Some(0) && qword(&path, "LastUsedTimeStart").is_some_and(|t| t > 0) {
                apps.push(app_name(&name));
            }
        }
    }
    apps.sort();
    apps.dedup();
    apps
}

fn app_name(key: &str) -> String {
    let last = key.rsplit('#').next().unwrap_or(key);
    let last = last.strip_suffix(".exe").or_else(|| last.strip_suffix(".EXE")).unwrap_or(last);
    let last = last.split('_').next().unwrap_or(last);
    if key.contains('#') { last.to_string() } else { last.rsplit('.').next().unwrap_or(last).to_string() }
}

fn subkeys(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, &HSTRING::from(path), None, KEY_READ, &mut key).is_err() {
            return out;
        }
        let mut buf = [0u16; 512];
        for i in 0.. {
            let mut len = buf.len() as u32;
            if RegEnumKeyExW(key, i, Some(PWSTR(buf.as_mut_ptr())), &mut len, None, None, None, None).is_err() {
                break;
            }
            out.push(String::from_utf16_lossy(&buf[..len as usize]));
        }
        let _ = RegCloseKey(key);
    }
    out
}

fn qword(path: &str, value: &str) -> Option<u64> {
    let mut v = 0u64;
    let mut size = 8u32;
    unsafe {
        RegGetValueW(HKEY_CURRENT_USER, &HSTRING::from(path), &HSTRING::from(value), RRF_RT_REG_QWORD, None, Some(&mut v as *mut u64 as _), Some(&mut size))
    }
    .is_ok()
    .then_some(v)
}

fn layout_tag(hkl: usize) -> Option<String> {
    let mut buf = [0u16; 85];
    let len = unsafe { LCIDToLocaleName((hkl & 0xffff) as u32, Some(&mut buf), 0) };
    (len > 1).then(|| String::from_utf16_lossy(&buf[..len as usize - 1]))
}

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};

use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_VOLUME_MUTE;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_SYSKEYDOWN,
};

use super::{VolumeKey, next_volume};

static TAKEOVER: AtomicBool = AtomicBool::new(false);
static KEYS: OnceLock<Sender<VolumeKey>> = OnceLock::new();

pub fn volume_keys_taken() -> bool {
    TAKEOVER.load(Ordering::Relaxed)
}

pub fn take_volume_keys(app: &AppHandle, on: bool) {
    TAKEOVER.store(on, Ordering::Relaxed);
    if !on || KEYS.get().is_some() {
        return;
    }
    let (tx, rx) = channel();
    if KEYS.set(tx).is_err() {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        let en: Option<IMMDeviceEnumerator> = unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()
        };
        let Some(en) = en else { return };
        for key in rx {
            if let Some(dir) = press(&en, key) {
                let _ = app.emit("status:limit", dir);
            }
        }
    });
    thread::spawn(hook_loop);
}

fn press(en: &IMMDeviceEnumerator, key: VolumeKey) -> Option<&'static str> {
    unsafe {
        let vol: IAudioEndpointVolume =
            en.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?.Activate(CLSCTX_ALL, None).ok()?;
        let muted = vol.GetMute().is_ok_and(|m| m.as_bool());
        if key == VolumeKey::Mute {
            let _ = vol.SetMute(!muted, std::ptr::null());
            return None;
        }
        if muted {
            let _ = vol.SetMute(false, std::ptr::null());
        }
        let level = vol.GetMasterVolumeLevelScalar().ok()?;
        let next = next_volume(level, key == VolumeKey::Up);
        if (next - level).abs() < f32::EPSILON {
            return Some(if key == VolumeKey::Up { "up" } else { "down" });
        }
        let _ = vol.SetMasterVolumeLevelScalar(next, std::ptr::null());
        None
    }
}

fn hook_loop() {
    unsafe {
        let module = GetModuleHandleW(None).ok().map(|m| HINSTANCE(m.0));
        if SetWindowsHookExW(WH_KEYBOARD_LL, Some(on_key), module, 0).is_err() {
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
    }
}

unsafe extern "system" fn on_key(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && TAKEOVER.load(Ordering::Relaxed) {
        let vk = unsafe { (*(lparam.0 as *const KBDLLHOOKSTRUCT)).vkCode } as u16;
        let key = match VIRTUAL_KEY(vk) {
            VK_VOLUME_UP => Some(VolumeKey::Up),
            VK_VOLUME_DOWN => Some(VolumeKey::Down),
            VK_VOLUME_MUTE => Some(VolumeKey::Mute),
            _ => None,
        };
        if let Some(key) = key {
            let down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
            if down && let Some(tx) = KEYS.get() {
                let _ = tx.send(key);
            }
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

#[cfg(test)]
mod privacy_tests {
    use super::app_name;

    #[test]
    fn names_from_consent_keys() {
        assert_eq!(app_name("C:#Program Files#Zoom#bin#Zoom.exe"), "Zoom");
        assert_eq!(app_name("Microsoft.WindowsCamera_8wekyb3d8bbwe"), "WindowsCamera");
    }

    #[test]
    #[ignore]
    fn lists_apps_using_the_mic_now() {
        println!("mic {:?} camera {:?}", super::apps_using("microphone"), super::apps_using("webcam"));
    }
}

#[cfg(test)]
mod layout_tests {
    use super::{LAYOUT_SETTLE, LayoutWatch};

    #[test]
    fn alt_tab_through_a_stale_window_is_not_a_switch() {
        let (th, en) = (Some("th-TH".to_string()), Some("en-US".to_string()));
        let mut w = LayoutWatch::default();
        let mut shown = w.next(1, th.clone(), &None);
        assert_eq!(shown, th, "first reading is taken as is");
        shown = w.next(2, en.clone(), &shown);
        assert_eq!(shown, th);
        shown = w.next(2, th.clone(), &shown);
        assert_eq!(shown, th);
        for _ in 0..LAYOUT_SETTLE {
            shown = w.next(2, th.clone(), &shown);
        }
        assert_eq!(shown, th);
        shown = w.next(2, en.clone(), &shown);
        assert_eq!(shown, en);
    }
}
