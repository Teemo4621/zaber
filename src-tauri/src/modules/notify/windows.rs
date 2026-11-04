use windows::UI::Notifications::Management::{
    UserNotificationListener as Listener, UserNotificationListenerAccessStatus as Access,
};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds, UserNotification};

use super::Notice;

pub fn request_access() {
    if let Ok(l) = Listener::Current()
        && l.GetAccessStatus().is_ok_and(|a| a == Access::Unspecified)
        && let Ok(op) = l.RequestAccessAsync()
    {
        let _ = op.get();
    }
}

pub fn access() -> &'static str {
    match Listener::Current().and_then(|l| l.GetAccessStatus()) {
        Ok(Access::Allowed) => "allowed",
        Ok(_) => "denied",
        Err(_) => "unsupported",
    }
}

pub fn list() -> Option<Vec<Notice>> {
    let l = Listener::Current().ok()?;
    let all = l.GetNotificationsAsync(NotificationKinds::Toast).ok()?.get().ok()?;
    Some(all.into_iter().filter_map(|n| notice(&n).ok()).collect())
}

fn notice(n: &UserNotification) -> windows::core::Result<Notice> {
    let info = n.AppInfo().ok();
    let app = info.as_ref().and_then(|a| a.DisplayInfo().ok()).and_then(|d| d.DisplayName().ok()).map(|s| s.to_string());
    let app_id = info.and_then(|a| a.AppUserModelId().ok()).map(|s| s.to_string()).unwrap_or_default();
    let binding = n.Notification()?.Visual()?.GetBinding(&KnownNotificationBindings::ToastGeneric()?)?;
    let mut texts = binding
        .GetTextElements()?
        .into_iter()
        .filter_map(|t| t.Text().ok().map(|h| h.to_string()))
        .filter(|s| !s.is_empty());
    Ok(Notice {
        id: n.Id()?,
        app: app.unwrap_or_default(),
        app_id,
        title: texts.next().unwrap_or_default(),
        body: texts.collect::<Vec<_>>().join(" "),
    })
}

pub fn icon_path(app_id: &str) -> Option<String> {
    Some(format!("shell:AppsFolder\\{app_id}"))
}

pub fn open(app_id: &str) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg(format!("shell:AppsFolder\\{app_id}"))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_an_app_icon_from_its_app_id() {
        let path = super::icon_path("windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel").unwrap();
        let png = tauri::async_runtime::block_on(crate::modules::shelf::shell_icon(path)).expect("an icon");
        assert!(png.starts_with("data:image/png;base64,") && png.len() > 500);
    }
}
