use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitors, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoSetProxyBlanket, EOAC_NONE,
    RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE,
    WbemLocator,
};
use windows::Win32::System::Variant::{VARIANT, VariantClear, VariantToInt32};
use windows::core::{BOOL, BSTR, w};

const DDC_EVERY: u64 = 8;

pub struct Probe {
    wmi: Option<IWbemServices>,
    external: bool,
    last_external: Option<u8>,
}

impl Probe {
    pub fn new() -> Self {
        let wmi = unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            CoCreateInstance::<_, IWbemLocator>(&WbemLocator, None, CLSCTX_INPROC_SERVER)
                .and_then(|l| l.ConnectServer(&BSTR::from("ROOT\\WMI"), &BSTR::new(), &BSTR::new(), &BSTR::new(), 0, &BSTR::new(), None))
                .and_then(|svc| {
                    CoSetProxyBlanket(&svc, RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE, None, RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE, None, EOAC_NONE)?;
                    Ok(svc)
                })
                .ok()
        };
        let mut p = Self { wmi, external: false, last_external: None };
        p.external = p.panel().is_none();
        p
    }

    pub fn level(&mut self, tick: u64) -> Option<u8> {
        if !self.external {
            return self.panel();
        }
        if tick.is_multiple_of(DDC_EVERY) {
            self.last_external = external_level();
        }
        self.last_external
    }

    fn panel(&self) -> Option<u8> {
        let wmi = self.wmi.as_ref()?;
        unsafe {
            let rows = wmi
                .ExecQuery(
                    &BSTR::from("WQL"),
                    &BSTR::from("SELECT CurrentBrightness FROM WmiMonitorBrightness"),
                    WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                    None,
                )
                .ok()?;
            let mut row: [Option<IWbemClassObject>; 1] = [None];
            let mut got = 0u32;
            let _ = rows.Next(WBEM_INFINITE, &mut row, &mut got);
            let row = row[0].take()?;
            let mut v = VARIANT::default();
            row.Get(w!("CurrentBrightness"), 0, &mut v, None, None).ok()?;
            let n = VariantToInt32(&v);
            let _ = VariantClear(&mut v);
            n.ok().and_then(|n| u8::try_from(n).ok())
        }
    }
}

fn external_level() -> Option<u8> {
    unsafe extern "system" fn collect(m: HMONITOR, _: HDC, _: *mut RECT, out: LPARAM) -> BOOL {
        unsafe { (*(out.0 as *mut Vec<HMONITOR>)).push(m) };
        true.into()
    }
    let mut monitors: Vec<HMONITOR> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(collect), LPARAM(&mut monitors as *mut _ as isize));
        for m in monitors {
            let mut n = 0u32;
            if GetNumberOfPhysicalMonitorsFromHMONITOR(m, &mut n).is_err() || n == 0 {
                continue;
            }
            let mut phys = vec![PHYSICAL_MONITOR::default(); n as usize];
            if GetPhysicalMonitorsFromHMONITOR(m, &mut phys).is_err() {
                continue;
            }
            let (mut min, mut cur, mut max) = (0u32, 0u32, 0u32);
            let ok = GetMonitorBrightness(phys[0].hPhysicalMonitor, &mut min, &mut cur, &mut max) != 0;
            let _ = DestroyPhysicalMonitors(&phys);
            if ok && max > min {
                return u8::try_from((cur.saturating_sub(min)) * 100 / (max - min)).ok();
            }
        }
    }
    None
}
