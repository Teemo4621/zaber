use base64::Engine;
use objc2::AnyThread;
use base64::engine::general_purpose::STANDARD;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};

const PX: f64 = 64.0;

pub async fn icon(path: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || render(&path)).await.ok().flatten()
}

fn render(path: &str) -> Option<String> {
    let image = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(path));
    let mut rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(PX, PX));
    let cg = unsafe { image.CGImageForProposedRect_context_hints(&mut rect, None, None) }?;
    let rep = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &cg);
    let png = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) }?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(png.to_vec())))
}
