use std::sync::mpsc;
use std::thread;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits, GetObjectW, HBITMAP,
    HGDIOBJ, ReleaseDC,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK};
use windows::core::HSTRING;

const PX: i32 = 64;

pub async fn icon(path: String) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let out = unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
            let out = render(&path);
            if init {
                CoUninitialize();
            }
            out
        };
        let _ = tx.send(out);
    });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten()).await.ok().flatten()
}

unsafe fn render(path: &str) -> Option<String> {
    unsafe {
        let item: IShellItemImageFactory = SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
        let bmp = item.GetImage(SIZE { cx: PX, cy: PX }, SIIGBF_BIGGERSIZEOK).ok()?;
        let rgba = pixels(bmp);
        let _ = DeleteObject(HGDIOBJ(bmp.0));
        let (w, h, rgba) = rgba?;
        let mut png = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png, w, h);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().ok()?.write_image_data(&rgba).ok()?;
        }
        Some(format!("data:image/png;base64,{}", STANDARD.encode(png)))
    }
}

unsafe fn pixels(bmp: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    unsafe {
        let mut info = BITMAP::default();
        if GetObjectW(HGDIOBJ(bmp.0), size_of::<BITMAP>() as i32, Some(&mut info as *mut _ as *mut _)) == 0 {
            return None;
        }
        let (w, h) = (info.bmWidth, info.bmHeight.abs());
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let dc = GetDC(None);
        let lines = GetDIBits(dc, bmp, 0, h as u32, Some(buf.as_mut_ptr().cast()), &mut bi, DIB_RGB_COLORS);
        ReleaseDC(None, dc);
        if lines == 0 {
            return None;
        }
        let opaque = buf.as_chunks::<4>().0.iter().all(|p| p[3] == 0);
        for p in buf.as_chunks_mut::<4>().0 {
            p.swap(0, 2);
            if opaque {
                p[3] = 255;
            } else if p[3] > 0 && p[3] < 255 {
                let a = p[3] as u16;
                for c in &mut p[..3] {
                    *c = ((*c as u16 * 255) / a).min(255) as u8;
                }
            }
        }
        Some((w as u32, h as u32, buf))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_a_real_file_icon_as_png() {
        let png = unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_APARTMENTTHREADED);
            super::render(r"C:\Windows\notepad.exe")
        };
        let png = png.expect("notepad has an icon");
        assert!(png.starts_with("data:image/png;base64,") && png.len() > 500);
    }
}
