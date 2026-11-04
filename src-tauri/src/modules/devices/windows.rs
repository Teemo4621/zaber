use std::collections::HashMap;

use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows::Foundation::IReference;
use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
use windows::core::{GUID, HSTRING, IInspectable, Interface, PCWSTR};

use windows_collections::IIterable;

use super::{Device, Kind};

const DRIVE_REMOVABLE: u32 = 2;
const INPUT_EVERY: u64 = 3;
const KEYBOARD_CLASS: &str = "{4d36e96b-e325-11ce-bfc1-08002be10318}";
const MOUSE_CLASS: &str = "{4d36e96f-e325-11ce-bfc1-08002be10318}";
const CONTAINER: &str = "System.Devices.ContainerId";
const BATTERY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
const IS_CONNECTED: &str = "{83DA6326-97A6-4088-9453-A1923F573B29} 15";

pub struct Probe {
    input: Vec<Device>,
    names: HashMap<String, String>,
}

impl Probe {
    pub fn new() -> Self {
        Self { input: Vec::new(), names: HashMap::new() }
    }

    pub fn devices(&mut self, tick: u64) -> Vec<Device> {
        if tick.is_multiple_of(INPUT_EVERY) {
            self.input = [(KEYBOARD_CLASS, Kind::Keyboard), (MOUSE_CLASS, Kind::Mouse)]
                .into_iter()
                .flat_map(|(class, kind)| self.input_devices(class, kind))
                .collect();
        }
        let mut all = drives();
        all.extend(self.input.iter().cloned());
        all
    }

    fn input_devices(&mut self, class: &str, kind: Kind) -> Vec<Device> {
        let aqs = format!("System.Devices.ClassGuid:=\"{class}\" AND System.Devices.Present:=System.StructuredQueryType.Boolean#True");
        let Ok(list) = find(&aqs, &[CONTAINER], DeviceInformationKind::Device) else { return Vec::new() };
        let mut out: Vec<Device> = Vec::new();
        for d in list {
            let Some(container) = guid_prop(&d, CONTAINER) else { continue };
            if out.iter().any(|x| x.id == container) {
                continue;
            }
            let fallback = d.Name().map(|n| n.to_string()).unwrap_or_default();
            let name = self.names.entry(container.clone()).or_insert_with(|| container_name(&container).unwrap_or(fallback)).clone();
            out.push(Device { id: container, kind, name, detail: None, path: None });
        }
        out
    }

    pub fn batteries(&mut self) -> Vec<(String, Kind, String, u8)> {
        let Ok(list) = find("System.Devices.DeviceInstanceId:~<\"BTH\"", &[BATTERY, CONTAINER, IS_CONNECTED], DeviceInformationKind::Device) else { return Vec::new() };
        let prop = |d: &DeviceInformation, key: &str| d.Properties().ok().and_then(|p| p.Lookup(&HSTRING::from(key)).ok());
        let connected: Vec<String> = list
            .iter()
            .filter(|d| prop(d, IS_CONNECTED).and_then(|v| v.cast::<IReference<bool>>().ok()).and_then(|v| v.Value().ok()) == Some(true))
            .filter_map(|d| guid_prop(d, CONTAINER))
            .collect();
        let mut out: Vec<(String, Kind, String, u8)> = Vec::new();
        for d in list {
            let level = prop(&d, BATTERY).and_then(|v| v.cast::<IReference<u8>>().ok()).and_then(|v| v.Value().ok());
            let (Some(level), Some(container)) = (level, guid_prop(&d, CONTAINER)) else { continue };
            if !connected.contains(&container) || out.iter().any(|(id, ..)| *id == container) {
                continue;
            }
            let fallback = d.Name().map(|n| n.to_string().replace(" Hands-Free AG", "")).unwrap_or_default();
            let name = self.names.entry(container.clone()).or_insert_with(|| container_name(&container).unwrap_or(fallback)).clone();
            out.push((container, guess_kind(&name), name, level));
        }
        out
    }
}

fn find(aqs: &str, props: &[&str], kind: DeviceInformationKind) -> windows::core::Result<Vec<DeviceInformation>> {
    let props = IIterable::<HSTRING>::from(props.iter().map(|p| HSTRING::from(*p)).collect::<Vec<_>>());
    let list = DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(&HSTRING::from(aqs), &props, kind)?.get()?;
    Ok(list.into_iter().collect())
}

fn guid_prop(d: &DeviceInformation, key: &str) -> Option<String> {
    let v: IInspectable = d.Properties().ok()?.Lookup(&HSTRING::from(key)).ok()?;
    let g: GUID = v.cast::<IReference<GUID>>().ok()?.Value().ok()?;
    Some(format!("{{{g:?}}}"))
}

fn container_name(container: &str) -> Option<String> {
    let none = IIterable::<HSTRING>::from(Vec::<HSTRING>::new());
    let c = DeviceInformation::CreateFromIdAsyncWithKindAndAdditionalProperties(&HSTRING::from(container), &none, DeviceInformationKind::DeviceContainer)
        .ok()?
        .get()
        .ok()?;
    let name = c.Name().ok()?.to_string();
    (!name.is_empty()).then_some(name)
}

fn guess_kind(name: &str) -> Kind {
    let n = name.to_lowercase();
    if n.contains("mouse") || n.contains("mx ") {
        Kind::Mouse
    } else if n.contains("keyboard") || n.contains("keys") {
        Kind::Keyboard
    } else {
        Kind::Headphones
    }
}

fn drives() -> Vec<Device> {
    let mut out = Vec::new();
    let mask = unsafe { GetLogicalDrives() };
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let root = format!("{}:\\", char::from(b'A' + i as u8));
        let wide: Vec<u16> = root.encode_utf16().chain([0]).collect();
        let p = PCWSTR(wide.as_ptr());
        unsafe {
            if GetDriveTypeW(p) != DRIVE_REMOVABLE {
                continue;
            }
            let mut label = [0u16; 261];
            let mut total = 0u64;
            if GetVolumeInformationW(p, Some(&mut label), None, None, None, None).is_err() {
                continue;
            }
            let _ = GetDiskFreeSpaceExW(p, None, Some(&mut total), None);
            let label = String::from_utf16_lossy(&label[..label.iter().position(|&c| c == 0).unwrap_or(0)]);
            out.push(Device {
                id: root.clone(),
                kind: Kind::Drive,
                name: if label.is_empty() { format!("USB drive ({})", &root[..2]) } else { label },
                detail: (total > 0).then(|| size_label(total)),
                path: Some(root),
            });
        }
    }
    out
}

fn size_label(bytes: u64) -> String {
    let gb = bytes as f64 / 1e9;
    if gb >= 1.0 { format!("{gb:.0} GB") } else { format!("{:.0} MB", bytes as f64 / 1e6) }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn lists_devices_here() {
        let mut p = super::Probe::new();
        for d in p.devices(0) {
            println!("{:?} {:?} {:?} {:?}", d.kind, d.name, d.detail, d.path);
        }
        println!("batteries: {:?}", p.batteries());
    }
}

