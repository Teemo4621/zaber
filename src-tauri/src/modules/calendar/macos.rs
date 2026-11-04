use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_app_kit::NSColorSpace;
use objc2_event_kit::{EKAuthorizationStatus, EKEntityType, EKEventStore};
use objc2_foundation::{NSDate, NSError};

use super::{Event, first_url};

static ASKED: AtomicBool = AtomicBool::new(false);

pub fn events(from_ms: i64, days: i64) -> Result<Vec<Event>, String> {
    unsafe {
        let status = EKEventStore::authorizationStatusForEntityType(EKEntityType::Event);
        if status == EKAuthorizationStatus::NotDetermined {
            if !ASKED.swap(true, Ordering::Relaxed) {
                let store = EKEventStore::new();
                let done = RcBlock::new(|_granted: Bool, _err: *mut NSError| {});
                #[allow(deprecated)]
                store.requestAccessToEntityType_completion(EKEntityType::Event, RcBlock::as_ptr(&done));
            }
            return Err("calendar access pending".into());
        }
        if status != EKAuthorizationStatus::FullAccess {
            return Err("calendar access denied".into());
        }

        let store = EKEventStore::new();
        let start = NSDate::dateWithTimeIntervalSince1970(from_ms as f64 / 1000.0);
        let end = NSDate::dateWithTimeIntervalSince1970((from_ms + days * 86_400_000) as f64 / 1000.0);
        let predicate = store.predicateForEventsWithStartDate_endDate_calendars(&start, &end, None);
        let srgb = NSColorSpace::sRGBColorSpace();
        Ok(store
            .eventsMatchingPredicate(&predicate)
            .iter()
            .map(|e| {
                let start_ms = (e.startDate().timeIntervalSince1970() * 1000.0) as i64;
                let location = e.location().map(|s| s.to_string()).unwrap_or_default();
                let link = e
                    .URL()
                    .and_then(|u| u.absoluteString())
                    .and_then(|s| first_url(&s.to_string()))
                    .or_else(|| first_url(&location))
                    .or_else(|| e.notes().and_then(|n| first_url(&n.to_string())));
                let color = e
                    .calendar()
                    .and_then(|c| c.color().colorUsingColorSpace(&srgb))
                    .map(|c| {
                        let b = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                        format!("#{:02x}{:02x}{:02x}", b(c.redComponent()), b(c.greenComponent()), b(c.blueComponent()))
                    })
                    .unwrap_or_else(|| "#8e8e93".into());
                Event {
                    title: e.title().to_string(),
                    start_ms,
                    end_ms: (e.endDate().timeIntervalSince1970() * 1000.0) as i64,
                    all_day: e.isAllDay(),
                    location,
                    link,
                    color,
                }
            })
            .collect())
    }
}
