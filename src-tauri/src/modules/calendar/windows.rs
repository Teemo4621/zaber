use std::collections::HashMap;

use windows::ApplicationModel::Appointments::{
    AppointmentManager, AppointmentProperties, AppointmentStoreAccessType, FindAppointmentsOptions,
};
use windows::Foundation::{DateTime, TimeSpan};

use super::{Event, first_url};

const EPOCH_DIFF_100NS: i64 = 116_444_736_000_000_000;
const DAY_100NS: i64 = 864_000_000_000;

pub fn events(from_ms: i64, days: i64) -> Result<Vec<Event>, String> {
    let run = || -> windows::core::Result<Vec<Event>> {
        let store = AppointmentManager::RequestStoreAsync(AppointmentStoreAccessType::AllCalendarsReadOnly)?.get()?;
        let colors: HashMap<String, String> = store
            .FindAppointmentCalendarsAsync()?
            .get()?
            .into_iter()
            .filter_map(|c| {
                let col = c.DisplayColor().ok()?;
                Some((c.LocalId().ok()?.to_string(), format!("#{:02x}{:02x}{:02x}", col.R, col.G, col.B)))
            })
            .collect();

        let opts = FindAppointmentsOptions::new()?;
        let fetch = opts.FetchProperties()?;
        for name in [
            AppointmentProperties::Subject()?,
            AppointmentProperties::StartTime()?,
            AppointmentProperties::Duration()?,
            AppointmentProperties::AllDay()?,
            AppointmentProperties::Location()?,
            AppointmentProperties::OnlineMeetingLink()?,
            AppointmentProperties::Details()?,
        ] {
            fetch.Append(&name)?;
        }
        let start = DateTime { UniversalTime: from_ms * 10_000 + EPOCH_DIFF_100NS };
        let span = TimeSpan { Duration: days * DAY_100NS };
        let found = store.FindAppointmentsAsyncWithOptions(start, span, &opts)?.get()?;

        Ok(found
            .into_iter()
            .filter_map(|a| {
                let start_ms = (a.StartTime().ok()?.UniversalTime - EPOCH_DIFF_100NS) / 10_000;
                let location = a.Location().map(|h| h.to_string()).unwrap_or_default();
                let link = a
                    .OnlineMeetingLink()
                    .ok()
                    .and_then(|h| first_url(&h.to_string()))
                    .or_else(|| first_url(&location))
                    .or_else(|| a.Details().ok().and_then(|d| first_url(&d.to_string())));
                Some(Event {
                    title: a.Subject().ok()?.to_string(),
                    start_ms,
                    end_ms: start_ms + a.Duration().map(|d| d.Duration / 10_000).unwrap_or(0),
                    all_day: a.AllDay().unwrap_or(false),
                    location,
                    link,
                    color: a
                        .CalendarId()
                        .ok()
                        .and_then(|id| colors.get(&id.to_string()).cloned())
                        .unwrap_or_else(|| "#8e8e93".into()),
                })
            })
            .collect())
    };
    run().map_err(|e| format!("calendar unavailable: {e}"))
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn reads_this_weeks_events() {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        use windows::ApplicationModel::Appointments::{AppointmentManager, AppointmentStoreAccessType};
        let store = AppointmentManager::RequestStoreAsync(AppointmentStoreAccessType::AllCalendarsReadOnly).unwrap().get().unwrap();
        for c in store.FindAppointmentCalendarsAsync().unwrap().get().unwrap() {
            println!("calendar: {} hidden={:?}", c.DisplayName().unwrap_or_default(), c.IsHidden());
        }
        match super::events(now - 86_400_000, 7) {
            Ok(list) => {
                println!("{} events", list.len());
                for e in list.iter().take(5) {
                    println!("{} {} {}", e.title, e.start_ms, e.color);
                }
            }
            Err(e) => println!("error: {e}"),
        }
    }
}
