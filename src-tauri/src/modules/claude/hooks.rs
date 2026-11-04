use serde_json::{Map, Value, json};

pub fn url_prefix(port: u16) -> String {
    format!("http://127.0.0.1:{port}/zaber/")
}

const HOOKS: &[(&str, Option<&str>, &str, u32)] = &[
    ("PermissionRequest", None, "permission", 600),
    ("Elicitation", None, "elicitation", 600),
    ("Stop", None, "stop", 10),
    ("Notification", Some("idle_prompt"), "idle", 10),
    ("UserPromptSubmit", None, "prompt", 5),
    ("PreToolUse", None, "tool", 5),
    ("SessionEnd", None, "end", 5),
];

fn is_ours(handler: &Value, prefix: &str) -> bool {
    handler.get("url").and_then(Value::as_str).is_some_and(|u| u.starts_with(prefix))
}

fn status_line(prefix: &str, token: &str) -> Value {
    json!({ "type": "command", "command": format!("curl -s -m 2 --data-binary @- -H \"X-Zaber-Token: {token}\" {prefix}usage") })
}

fn is_our_status_line(settings: &Value, prefix: &str) -> bool {
    settings["statusLine"]["command"].as_str().is_some_and(|c| c.contains(prefix))
}

pub fn remove(mut settings: Value, prefix: &str) -> Value {
    if is_our_status_line(&settings, prefix) {
        settings.as_object_mut().map(|o| o.remove("statusLine"));
    }
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else { return settings };
    for groups in hooks.values_mut() {
        if let Some(groups) = groups.as_array_mut() {
            for g in groups.iter_mut() {
                if let Some(list) = g.get_mut("hooks").and_then(Value::as_array_mut) {
                    list.retain(|h| !is_ours(h, prefix));
                }
            }
            groups.retain(|g| g.get("hooks").and_then(Value::as_array).is_none_or(|l| !l.is_empty()));
        }
    }
    hooks.retain(|_, groups| groups.as_array().is_none_or(|g| !g.is_empty()));
    if hooks.is_empty() {
        settings.as_object_mut().map(|o| o.remove("hooks"));
    }
    settings
}

pub fn install(settings: Value, port: u16, token: &str) -> Result<Value, String> {
    let prefix = url_prefix(port);
    let mut settings = remove(settings, &prefix);
    let root = settings.as_object_mut().ok_or("settings.json is not a JSON object")?;
    root.entry("statusLine").or_insert_with(|| status_line(&prefix, token));
    let hooks = root.entry("hooks").or_insert_with(|| Value::Object(Map::new()));
    let hooks = hooks.as_object_mut().ok_or("\"hooks\" in settings.json is not an object")?;
    for (event, matcher, path, timeout) in HOOKS {
        let mut group = json!({
            "hooks": [{
                "type": "http",
                "url": format!("{prefix}{path}"),
                "headers": { "X-Zaber-Token": token },
                "timeout": timeout,
            }]
        });
        if let Some(m) = matcher {
            group["matcher"] = json!(m);
        }
        let list = hooks.entry(*event).or_insert_with(|| json!([]));
        list.as_array_mut().ok_or(format!("hooks.{event} in settings.json is not a list"))?.push(group);
    }
    Ok(settings)
}

pub fn installed(settings: &Value, port: u16) -> bool {
    let prefix = url_prefix(port);
    settings.get("statusLine").is_some()
        && HOOKS.iter().all(|(event, _, path, _)| {
        let url = format!("{prefix}{path}");
        settings["hooks"][event].as_array().is_some_and(|groups| {
            groups.iter().any(|g| g["hooks"].as_array().is_some_and(|l| l.iter().any(|h| h["url"] == url.as_str())))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn existing() -> Value {
        json!({
            "model": "opus",
            "hooks": {
                "Stop": [{ "hooks": [{ "type": "command", "command": "other-app.exe", "timeout": 10 }] }]
            }
        })
    }

    #[test]
    fn install_keeps_everything_else_and_remove_restores_it() {
        let before = existing();
        let after = install(before.clone(), 47811, "t0k").unwrap();
        assert_eq!(after["model"], "opus");
        assert_eq!(after["hooks"]["Stop"].as_array().unwrap().len(), 2, "other app's Stop hook kept");
        assert_eq!(after["hooks"]["PermissionRequest"][0]["hooks"][0]["headers"]["X-Zaber-Token"], "t0k");
        assert_eq!(after["hooks"]["Notification"][0]["matcher"], "idle_prompt");
        assert!(installed(&after, 47811));
        assert!(after["statusLine"]["command"].as_str().unwrap().ends_with("http://127.0.0.1:47811/zaber/usage"));
        assert_eq!(remove(after, &url_prefix(47811)), before);
    }

    #[test]
    fn the_users_own_status_line_is_kept() {
        let mine = json!({ "statusLine": { "type": "command", "command": "my-line.sh" } });
        let after = install(mine.clone(), 1, "a").unwrap();
        assert_eq!(after["statusLine"], mine["statusLine"]);
        assert!(installed(&after, 1));
        assert_eq!(remove(after, &url_prefix(1)), mine);
    }

    #[test]
    fn install_is_idempotent_and_works_on_empty_settings() {
        let once = install(json!({}), 1, "a").unwrap();
        let twice = install(once.clone(), 1, "a").unwrap();
        assert_eq!(once, twice);
        assert_eq!(remove(twice, &url_prefix(1)), json!({}));
        assert!(!installed(&json!({}), 1));
        assert!(install(json!([1]), 1, "a").is_err());
    }
}

#[cfg(test)]
mod real_file {
    #[test]
    #[ignore]
    fn round_trips_the_real_settings_in_memory() {
        let path = std::path::Path::new(&std::env::var("USERPROFILE").unwrap()).join(".claude/settings.json");
        let text = std::fs::read_to_string(path).unwrap();
        let before: serde_json::Value = serde_json::from_str(&text).unwrap();
        let after = super::install(before.clone(), 47811, "x".repeat(32).as_str()).unwrap();
        assert!(super::installed(&after, 47811));
        let back = super::remove(after, &super::url_prefix(47811));
        assert_eq!(serde_json::to_string(&back).unwrap(), serde_json::to_string(&before).unwrap(), "same keys, same order");
    }
}
