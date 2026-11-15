use objc2::runtime::NSObjectProtocol;
use objc2::sel;
use objc2_app_kit::{
    NSBitmapImageRep, NSPasteboard, NSPasteboardAccessBehavior, NSPasteboardTypeFileURL, NSPasteboardTypePNG,
    NSPasteboardTypeString, NSPasteboardTypeTIFF,
};
use objc2_foundation::{NSProcessInfo, NSString, NSURL};

use super::{Copied, summarize};

const PRIVATE: [&str; 2] = ["org.nspasteboard.ConcealedType", "org.nspasteboard.TransientType"];

pub fn sequence() -> u32 {
    NSPasteboard::generalPasteboard().changeCount() as u32
}

fn allowed(pb: &NSPasteboard) -> bool {
    if !pb.respondsToSelector(sel!(accessBehavior)) {
        return true;
    }
    let behavior = pb.accessBehavior();
    behavior == NSPasteboardAccessBehavior::AlwaysAllow
        || (behavior == NSPasteboardAccessBehavior::Default && NSProcessInfo::processInfo().operatingSystemVersion().majorVersion < 26)
}

pub fn read() -> Option<Copied> {
    let pb = NSPasteboard::generalPasteboard();
    if !allowed(&pb) {
        return None;
    }
    let types: Vec<String> = pb.types()?.iter().map(|t| t.to_string()).collect();
    if types.iter().any(|t| PRIVATE.contains(&t.as_str())) {
        return None;
    }
    let has = |t: &NSString| types.iter().any(|x| *x == t.to_string());
    if has(unsafe { NSPasteboardTypeFileURL }) {
        let items = pb.pasteboardItems()?;
        let urls: Vec<String> = items
            .iter()
            .filter_map(|i| i.stringForType(unsafe { NSPasteboardTypeFileURL }))
            .map(|s| s.to_string())
            .collect();
        if let Some(first) = urls.first() {
            let name = NSURL::URLWithString(&NSString::from_str(first))
                .and_then(|u| u.lastPathComponent())
                .map(|n| n.to_string())
                .unwrap_or_else(|| first.rsplit('/').next().unwrap_or(first).to_string());
            return Some(Copied::Files { name, count: urls.len() as u32 });
        }
    }
    if has(unsafe { NSPasteboardTypeString })
        && let Some(c) = pb.stringForType(unsafe { NSPasteboardTypeString }).and_then(|s| summarize(&s.to_string()))
    {
        return Some(c);
    }
    if has(unsafe { NSPasteboardTypePNG })
        && let Some(size) = pb.dataForType(unsafe { NSPasteboardTypePNG }).and_then(|d| png_size(&d.to_vec()))
    {
        return Some(Copied::Image { width: size.0, height: size.1 });
    }
    if has(unsafe { NSPasteboardTypeTIFF }) {
        let rep = pb.dataForType(unsafe { NSPasteboardTypeTIFF }).and_then(|d| NSBitmapImageRep::imageRepWithData(&d))?;
        return Some(Copied::Image { width: rep.pixelsWide() as u32, height: rep.pixelsHigh() as u32 });
    }
    None
}

fn png_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.len() < 24 || &b[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let be = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    Some((be(16), be(20)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn png_header_size() {
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        b.extend(800u32.to_be_bytes());
        b.extend(600u32.to_be_bytes());
        assert_eq!(super::png_size(&b), Some((800, 600)));
        assert_eq!(super::png_size(b"nope"), None);
    }
}
