use std::ffi::c_void;
use std::ptr;

use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSColor, NSColorSpace};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

use super::Status;

const SLOW_EVERY: u64 = 20;
const LAYOUT_EVERY: u64 = 5;
const FULLSCREEN_EVERY: u64 = 10;

#[repr(C)]
struct Address {
    selector: u32,
    scope: u32,
    element: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[repr(C)]
struct SockAddrIn {
    len: u8,
    family: u8,
    port: u16,
    addr: u32,
    zero: [u8; 8],
}

#[link(name = "CoreAudio", kind = "framework")]
unsafe extern "C" {
    fn AudioObjectGetPropertyData(id: u32, addr: *const Address, qsize: u32, qdata: *const c_void, size: *mut u32, data: *mut c_void) -> i32;
    fn AudioObjectSetPropertyData(id: u32, addr: *const Address, qsize: u32, qdata: *const c_void, size: u32, data: *const c_void) -> i32;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceFlagsState(state: i32) -> u64;
    fn CGWindowListCopyWindowInfo(option: u32, relative: u32) -> *mut AnyObject;
    fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayBounds(display: u32) -> Rect;
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOPSCopyPowerSourcesInfo() -> *mut AnyObject;
    fn IOPSCopyPowerSourcesList(blob: *const AnyObject) -> *mut NSArray<AnyObject>;
    fn IOPSGetPowerSourceDescription(blob: *const AnyObject, source: *const AnyObject) -> *mut NSDictionary<NSString, AnyObject>;
}

#[link(name = "SystemConfiguration", kind = "framework")]
unsafe extern "C" {
    fn SCNetworkReachabilityCreateWithAddress(alloc: *const c_void, addr: *const SockAddrIn) -> *mut AnyObject;
    fn SCNetworkReachabilityGetFlags(target: *const AnyObject, flags: *mut u32) -> u8;
}

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyInputSourceLanguages: *const c_void;
    fn TISCopyCurrentKeyboardInputSource() -> *mut AnyObject;
    fn TISGetInputSourceProperty(source: *const AnyObject, key: *const c_void) -> *const NSArray<NSString>;
}

const fn code(s: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*s)
}

const SYSTEM: u32 = 1;
const GLOBAL: u32 = code(b"glob");
const OUTPUT: u32 = code(b"outp");
const INPUT: u32 = code(b"inpt");
const CAPS: u64 = 0x10000;

fn get<T: Copy + Default>(id: u32, selector: u32, scope: u32, element: u32) -> Option<T> {
    let addr = Address { selector, scope, element };
    let mut v = T::default();
    let mut size = size_of::<T>() as u32;
    let r = unsafe { AudioObjectGetPropertyData(id, &addr, 0, ptr::null(), &mut size, (&mut v as *mut T).cast()) };
    (r == 0).then_some(v)
}

fn device_name(id: u32) -> Option<String> {
    let raw: usize = get(id, code(b"lnam"), GLOBAL, 0)?;
    let name = unsafe { Retained::from_raw(raw as *mut NSString) }?;
    Some(name.to_string())
}

pub struct Probe {
    last: Status,
    output: Option<u32>,
    input: Option<u32>,
    reach: Option<Retained<AnyObject>>,
}

impl Probe {
    pub fn new() -> Self {
        let zero = SockAddrIn { len: size_of::<SockAddrIn>() as u8, family: 2, port: 0, addr: 0, zero: [0; 8] };
        let reach = unsafe { Retained::from_raw(SCNetworkReachabilityCreateWithAddress(ptr::null(), &zero)) };
        Self { last: Status { online: true, ..Status::default() }, output: None, input: None, reach }
    }

    pub fn poll(&mut self, tick: u64) -> Status {
        let mut s = self.last.clone();
        if tick.is_multiple_of(SLOW_EVERY) {
            self.output = get(SYSTEM, code(b"dOut"), GLOBAL, 0).filter(|&id: &u32| id != 0);
            s.audio_device = self.output.and_then(device_name);
            self.input = input();
            (s.charging, s.battery, s.battery_minutes) = battery();
            s.online = self.reach.as_ref().is_none_or(|r| {
                let mut flags = 0u32;
                unsafe { SCNetworkReachabilityGetFlags(Retained::as_ptr(r), &mut flags) != 0 && flags & 2 != 0 && flags & 4 == 0 }
            });
            s.accent = accent();
        }
        match self.output {
            Some(id) => {
                let level = get::<f32>(id, code(b"vmvc"), OUTPUT, 0).or_else(|| get::<f32>(id, code(b"volm"), OUTPUT, 1));
                s.volume = level.map(|f| (f * 100.0).round().clamp(0.0, 100.0) as u8);
                s.muted = get::<u32>(id, code(b"mute"), OUTPUT, 0).is_some_and(|m| m != 0);
            }
            None => s.volume = None,
        }
        s.mic_muted = self.input.and_then(|id| get::<u32>(id, code(b"mute"), INPUT, 0)).map(|m| m != 0);
        s.caps_lock = unsafe { CGEventSourceFlagsState(0) } & CAPS != 0;
        if tick.is_multiple_of(LAYOUT_EVERY) {
            s.layout = layout();
        }
        if tick.is_multiple_of(FULLSCREEN_EVERY) {
            s.fullscreen = fullscreen();
        }
        self.last = s.clone();
        s
    }
}

fn input() -> Option<u32> {
    get(SYSTEM, code(b"dIn "), GLOBAL, 0).filter(|&id: &u32| id != 0)
}

fn number(d: &NSDictionary<NSString, AnyObject>, key: &str) -> Option<f64> {
    d.objectForKey(&NSString::from_str(key))?.downcast::<NSNumber>().ok().map(|n| n.doubleValue())
}

fn text(d: &NSDictionary<NSString, AnyObject>, key: &str) -> Option<String> {
    d.objectForKey(&NSString::from_str(key))?.downcast::<NSString>().ok().map(|n| n.to_string())
}

fn battery() -> (Option<bool>, Option<u8>, Option<u32>) {
    unsafe {
        let Some(blob) = Retained::from_raw(IOPSCopyPowerSourcesInfo()) else { return (None, None, None) };
        let Some(list) = Retained::from_raw(IOPSCopyPowerSourcesList(Retained::as_ptr(&blob))) else { return (None, None, None) };
        for source in list.iter() {
            let Some(d) = Retained::retain(IOPSGetPowerSourceDescription(Retained::as_ptr(&blob), Retained::as_ptr(&source))) else { continue };
            if text(&d, "Type").as_deref() != Some("InternalBattery") {
                continue;
            }
            let ac = text(&d, "Power Source State").as_deref() == Some("AC Power");
            let level = match (number(&d, "Current Capacity"), number(&d, "Max Capacity")) {
                (Some(c), Some(m)) if m > 0.0 => Some((c * 100.0 / m).round().clamp(0.0, 100.0) as u8),
                _ => None,
            };
            let minutes = number(&d, "Time to Empty").filter(|&m| !ac && m > 0.0).map(|m| m as u32);
            return (Some(ac), level, minutes);
        }
        (None, None, None)
    }
}

fn accent() -> Option<String> {
    let c = NSColor::controlAccentColor().colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
    let b = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Some(format!("#{:02x}{:02x}{:02x}", b(c.redComponent()), b(c.greenComponent()), b(c.blueComponent())))
}

fn layout() -> Option<String> {
    let mut out = None;
    DispatchQueue::main().exec_sync(|| unsafe {
        let Some(source) = Retained::from_raw(TISCopyCurrentKeyboardInputSource()) else { return };
        let langs = TISGetInputSourceProperty(Retained::as_ptr(&source), kTISPropertyInputSourceLanguages);
        out = langs.as_ref().and_then(|l| l.firstObject()).map(|s| s.to_string());
    });
    out
}

fn fullscreen() -> bool {
    let mut ids = [0u32; 8];
    let mut count = 0u32;
    if unsafe { CGGetActiveDisplayList(ids.len() as u32, ids.as_mut_ptr(), &mut count) } != 0 {
        return false;
    }
    let screens: Vec<Rect> = ids[..count as usize].iter().map(|&d| unsafe { CGDisplayBounds(d) }).collect();
    let Some(list) = (unsafe { Retained::from_raw(CGWindowListCopyWindowInfo(1 | 16, 0)) }) else { return false };
    let Ok(list) = list.downcast::<NSArray>() else { return false };
    let me = std::process::id() as f64;
    for w in list.iter() {
        let Ok(w) = w.downcast::<NSDictionary>() else { continue };
        let w: &NSDictionary<NSString, AnyObject> = unsafe { &*(Retained::as_ptr(&w) as *const NSDictionary<NSString, AnyObject>) };
        if number(w, "kCGWindowLayer") != Some(0.0) || number(w, "kCGWindowOwnerPID") == Some(me) || number(w, "kCGWindowAlpha") == Some(0.0) {
            continue;
        }
        let Some(b) = w.objectForKey(&NSString::from_str("kCGWindowBounds")).and_then(|b| b.downcast::<NSDictionary>().ok()) else { return false };
        let b: &NSDictionary<NSString, AnyObject> = unsafe { &*(Retained::as_ptr(&b) as *const NSDictionary<NSString, AnyObject>) };
        let r = Rect {
            x: number(b, "X").unwrap_or(-1.0),
            y: number(b, "Y").unwrap_or(-1.0),
            w: number(b, "Width").unwrap_or(0.0),
            h: number(b, "Height").unwrap_or(0.0),
        };
        return screens.iter().any(|s| (s.x - r.x).abs() < 1.0 && (s.y - r.y).abs() < 1.0 && (s.w - r.w).abs() < 1.0 && (s.h - r.h).abs() < 1.0);
    }
    false
}

pub fn toggle_mic() {
    let Some(id) = input() else { return };
    let muted = get::<u32>(id, code(b"mute"), INPUT, 0).is_some_and(|m| m != 0);
    let addr = Address { selector: code(b"mute"), scope: INPUT, element: 0 };
    let next: u32 = (!muted).into();
    unsafe {
        AudioObjectSetPropertyData(id, &addr, 0, ptr::null(), size_of::<u32>() as u32, (&next as *const u32).cast());
    }
}

pub fn volume_keys() -> (bool, bool) {
    (false, false)
}

pub fn take_volume_keys(_app: &tauri::AppHandle, _on: bool) {}

pub fn volume_keys_taken() -> bool {
    false
}
