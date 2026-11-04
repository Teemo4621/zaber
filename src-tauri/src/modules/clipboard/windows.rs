use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{CF_DIB, CF_HDROP, CF_UNICODETEXT};
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::core::w;

use super::{Copied, summarize};

pub fn sequence() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

pub fn read() -> Option<Copied> {
    unsafe {
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if !opened {
            return None;
        }
        let out = read_open();
        let _ = CloseClipboard();
        out
    }
}

unsafe fn private() -> bool {
    unsafe {
        let exclude = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
        let ignore = RegisterClipboardFormatW(w!("Clipboard Viewer Ignore"));
        if IsClipboardFormatAvailable(exclude).is_ok() || IsClipboardFormatAvailable(ignore).is_ok() {
            return true;
        }
        let history = RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory"));
        if let Ok(h) = GetClipboardData(history) {
            let p = GlobalLock(HGLOBAL(h.0)) as *const u32;
            let allowed = p.is_null() || *p != 0;
            let _ = GlobalUnlock(HGLOBAL(h.0));
            return !allowed;
        }
        false
    }
}

unsafe fn read_open() -> Option<Copied> {
    unsafe {
        if private() {
            return None;
        }
        if let Ok(h) = GetClipboardData(CF_HDROP.0 as u32) {
            let drop = HDROP(h.0);
            let count = DragQueryFileW(drop, u32::MAX, None);
            if count > 0 {
                let mut buf = [0u16; 520];
                let n = DragQueryFileW(drop, 0, Some(&mut buf)) as usize;
                let path = String::from_utf16_lossy(&buf[..n]);
                let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
                return Some(Copied::Files { name, count });
            }
        }
        if let Ok(h) = GetClipboardData(CF_UNICODETEXT.0 as u32) {
            let text = locked_text(h);
            if let Some(c) = text.as_deref().and_then(summarize) {
                return Some(c);
            }
        }
        if let Ok(h) = GetClipboardData(CF_DIB.0 as u32) {
            let g = HGLOBAL(h.0);
            let p = GlobalLock(g) as *const BITMAPINFOHEADER;
            let size = if p.is_null() || GlobalSize(g) < size_of::<BITMAPINFOHEADER>() {
                None
            } else {
                Some(((*p).biWidth.unsigned_abs(), (*p).biHeight.unsigned_abs()))
            };
            let _ = GlobalUnlock(g);
            if let Some((width, height)) = size {
                return Some(Copied::Image { width, height });
            }
        }
        None
    }
}

unsafe fn locked_text(h: HANDLE) -> Option<String> {
    unsafe {
        let g = HGLOBAL(h.0);
        let p = GlobalLock(g) as *const u16;
        if p.is_null() {
            return None;
        }
        let max = (GlobalSize(g) / 2).min(4096);
        let mut n = 0;
        while n < max && *p.add(n) != 0 {
            n += 1;
        }
        let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
        let _ = GlobalUnlock(g);
        Some(s)
    }
}
