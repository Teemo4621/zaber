use std::fs;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSNumber, NSString, NSURL, NSURLVolumeIsEjectableKey, NSURLVolumeLocalizedNameKey, NSURLVolumeTotalCapacityKey};

use super::{Device, Kind};

pub struct Probe {
    names: Vec<String>,
    drives: Vec<Device>,
}

impl Probe {
    pub fn new() -> Self {
        Self { names: Vec::new(), drives: Vec::new() }
    }

    pub fn devices(&mut self, _tick: u64) -> Vec<Device> {
        let mut names: Vec<String> = fs::read_dir("/Volumes")
            .map(|d| d.flatten().filter(|e| e.file_type().is_ok_and(|t| t.is_dir())).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        if names != self.names {
            self.drives = names.iter().filter_map(|n| drive(&format!("/Volumes/{n}"))).collect();
            self.names = names;
        }
        self.drives.clone()
    }

    pub fn batteries(&mut self) -> Vec<(String, Kind, String, u8)> {
        Vec::new()
    }
}

fn value(url: &NSURL, key: &NSString) -> Option<Retained<AnyObject>> {
    let mut out = None;
    unsafe { url.getResourceValue_forKey_error(&mut out, key) }.ok()?;
    out
}

fn drive(path: &str) -> Option<Device> {
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));
    let ejectable = value(&url, unsafe { NSURLVolumeIsEjectableKey })?.downcast::<NSNumber>().ok()?.boolValue();
    if !ejectable {
        return None;
    }
    let name = value(&url, unsafe { NSURLVolumeLocalizedNameKey })
        .and_then(|n| n.downcast::<NSString>().ok())
        .map(|n| n.to_string())
        .unwrap_or_else(|| path.trim_start_matches("/Volumes/").to_string());
    let total = value(&url, unsafe { NSURLVolumeTotalCapacityKey }).and_then(|n| n.downcast::<NSNumber>().ok()).map(|n| n.longLongValue().max(0) as u64);
    Some(Device { id: path.to_string(), kind: Kind::Drive, name, detail: total.filter(|&t| t > 0).map(size_label), path: Some(path.to_string()) })
}

fn size_label(bytes: u64) -> String {
    let gb = bytes as f64 / 1e9;
    if gb >= 1.0 { format!("{gb:.0} GB") } else { format!("{:.0} MB", bytes as f64 / 1e6) }
}
