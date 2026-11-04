mod hooks;

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Emitter, Manager};

pub const PORT: u16 = 47811;
const MAX_BODY: usize = 1 << 20;
const MAX_WAIT: u64 = 580;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub title: String,
    pub kind: String,
    pub options: Vec<String>,
    pub required: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub id: u64,
    pub kind: &'static str,
    pub project: String,
    pub session: String,
    pub tool: Option<String>,
    pub summary: String,
    pub message: Option<String>,
    pub fields: Vec<Field>,
    pub can_always: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub project: String,
    pub session: String,
    pub text: String,
}

fn note(body: &Value, text: &str) -> Note {
    Note { project: project(body), session: body["session_id"].as_str().unwrap_or_default().to_string(), text: text.chars().take(200).collect() }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limit {
    pub used: f64,
    pub resets_at: u64,
}

#[derive(Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub session: Option<Limit>,
    pub week: Option<Limit>,
}

fn usage(body: &Value) -> Usage {
    let limit = |w: &Value| Some(Limit { used: w["used_percentage"].as_f64()?, resets_at: w["resets_at"].as_u64()? });
    Usage { session: limit(&body["rate_limits"]["five_hour"]), week: limit(&body["rate_limits"]["seven_day"]) }
}

fn usage_line(u: &Usage) -> String {
    [("5h", &u.session), ("7d", &u.week)]
        .iter()
        .filter_map(|(k, l)| l.as_ref().map(|l| format!("{k} {:.0}%", l.used)))
        .collect::<Vec<_>>()
        .join(" · ")
}

static USAGE: Mutex<Option<Usage>> = Mutex::new(None);

#[tauri::command]
pub fn claude_usage() -> Option<Usage> {
    USAGE.lock().unwrap().clone()
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Answer {
    Allow { always: bool },
    Deny,
    Accept { content: Map<String, Value> },
    Decline,
    Pass,
}

struct Pending {
    kind: &'static str,
    suggestions: Vec<Value>,
    fields: Vec<String>,
    tx: Sender<Value>,
}

static PENDING: Mutex<Option<HashMap<u64, Pending>>> = Mutex::new(None);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static WAIT_SECS: AtomicU64 = AtomicU64::new(60);

#[tauri::command]
pub fn claude_answer(id: u64, answer: Answer) -> Result<(), String> {
    let p = PENDING.lock().unwrap().as_mut().and_then(|m| m.remove(&id)).ok_or("request is no longer waiting")?;
    let reply = decision(&p, answer)?;
    p.tx.send(reply).map_err(|_| "Claude Code stopped waiting".to_string())
}

#[tauri::command]
pub fn claude_set_wait(secs: u64) {
    WAIT_SECS.store(if secs == 0 { MAX_WAIT } else { secs.clamp(10, MAX_WAIT) }, Ordering::Relaxed);
}

#[tauri::command]
pub fn claude_hook(app: AppHandle, on: bool) -> Result<bool, String> {
    let path = settings_path(&app)?;
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "{}".into(),
        Err(e) => return Err(e.to_string()),
    };
    let current: Value = serde_json::from_str(&text).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    let next = if on { hooks::install(current, PORT, &token(&app)?)? } else { hooks::remove(current, &hooks::url_prefix(PORT)) };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        fs::copy(&path, path.with_extension("json.zaber-backup")).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.zaber-tmp");
    fs::write(&tmp, serde_json::to_string_pretty(&next).map_err(|e| e.to_string())? + "\n").map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(hooks::installed(&next, PORT))
}

#[tauri::command]
pub fn claude_hook_status(app: AppHandle) -> bool {
    settings_path(&app)
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .is_some_and(|v| hooks::installed(&v, PORT))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().home_dir().map_err(|e| e.to_string())?.join(".claude").join("settings.json"))
}

fn token(app: &AppHandle) -> Result<String, String> {
    let path = app.path().app_data_dir().map_err(|e| e.to_string())?.join("claude-token");
    if let Ok(t) = fs::read_to_string(&path)
        && t.trim().len() >= 32
    {
        return Ok(t.trim().to_string());
    }
    let r = || RandomState::new().build_hasher().finish();
    let t = format!("{:016x}{:016x}", r(), r());
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(&path, &t).map_err(|e| e.to_string())?;
    Ok(t)
}

fn decision(p: &Pending, answer: Answer) -> Result<Value, String> {
    Ok(match (p.kind, answer) {
        ("permission", Answer::Allow { always }) => {
            let mut d = json!({ "behavior": "allow" });
            if always {
                let rules: Vec<Value> = p.suggestions.iter().filter(|s| s["behavior"] == "allow").cloned().collect();
                if !rules.is_empty() {
                    d["updatedPermissions"] = json!(rules);
                }
            }
            json!({ "hookSpecificOutput": { "hookEventName": "PermissionRequest", "decision": d } })
        }
        ("permission", Answer::Deny) => json!({
            "hookSpecificOutput": { "hookEventName": "PermissionRequest", "decision": { "behavior": "deny", "message": "Denied from the Zaber notch" } }
        }),
        ("question", Answer::Accept { content }) => {
            let content: Map<String, Value> = content.into_iter().filter(|(k, _)| p.fields.contains(k)).collect();
            json!({ "hookSpecificOutput": { "hookEventName": "Elicitation", "action": "accept", "content": content } })
        }
        (_, Answer::Pass) => Value::Null,
        ("question", Answer::Decline) => json!({ "hookSpecificOutput": { "hookEventName": "Elicitation", "action": "decline" } }),
        _ => return Err("that answer doesn't fit this request".into()),
    })
}

fn project(body: &Value) -> String {
    let cwd = body["cwd"].as_str().unwrap_or_default();
    cwd.rsplit(['\\', '/']).find(|s| !s.is_empty()).unwrap_or(cwd).to_string()
}

fn doing(tool: &str, input: &Value) -> String {
    let s = summary(tool, input);
    let s = s.lines().next().unwrap_or_default();
    let s = if input.get("file_path").is_some() { s.rsplit(['\\', '/']).next().unwrap_or(s) } else { s };
    format!("{tool} {}", s.chars().take(60).collect::<String>()).trim().to_string()
}

fn summary(tool: &str, input: &Value) -> String {
    let pick = ["command", "file_path", "url", "pattern", "path", "query", "prompt"]
        .iter()
        .find_map(|k| input.get(*k).and_then(Value::as_str));
    let s = match pick {
        Some(s) => s.to_string(),
        None if input.is_null() => tool.to_string(),
        None => input.to_string(),
    };
    s.chars().take(300).collect()
}

fn fields(schema: &Value) -> Vec<Field> {
    let required: Vec<&str> = schema["required"].as_array().map(|r| r.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    let Some(props) = schema["properties"].as_object() else { return Vec::new() };
    props
        .iter()
        .map(|(name, p)| {
            let options: Vec<String> = p["enum"].as_array().map(|e| e.iter().map(|v| v.as_str().map_or(v.to_string(), str::to_string)).collect()).unwrap_or_default();
            let kind = if !options.is_empty() {
                "enum"
            } else {
                match p["type"].as_str() {
                    Some("boolean") => "boolean",
                    Some("number" | "integer") => "number",
                    _ => "string",
                }
            };
            Field {
                name: name.clone(),
                title: p["title"].as_str().or(p["description"].as_str()).unwrap_or(name).to_string(),
                kind: kind.into(),
                options,
                required: required.contains(&name.as_str()),
            }
        })
        .collect()
}

pub fn start(app: AppHandle) {
    let Ok(token) = token(&app) else { return };
    thread::spawn(move || {
        let Ok(listener) = TcpListener::bind(("127.0.0.1", PORT)) else { return };
        for stream in listener.incoming().flatten() {
            let (app, token) = (app.clone(), token.clone());
            thread::spawn(move || {
                let _ = serve(&app, stream, &token);
            });
        }
    });
}

fn respond(mut stream: &TcpStream, status: &str, body: &str) -> std::io::Result<()> {
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}

fn serve(app: &AppHandle, stream: TcpStream, token: &str) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let path = line.split_whitespace().nth(1).unwrap_or_default().to_string();
    let (mut len, mut authorized) = (0usize, false);
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        let (k, v) = line.split_once(':').unwrap_or((&line, ""));
        match k.trim().to_ascii_lowercase().as_str() {
            "content-length" => len = v.trim().parse().unwrap_or(0),
            "x-zaber-token" => authorized = v.trim() == token,
            _ => {}
        }
    }
    if !authorized || len > MAX_BODY || !line.trim().is_empty() {
        return respond(&stream, "403 Forbidden", "");
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    match path.as_str() {
        "/zaber/stop" => {
            let text = body["last_assistant_message"].as_str().unwrap_or_default().lines().find(|l| !l.trim().is_empty()).unwrap_or_default();
            let _ = app.emit("claude:done", note(&body, text));
            respond(&stream, "200 OK", "")
        }
        "/zaber/idle" => {
            let _ = app.emit("claude:waiting", note(&body, body["message"].as_str().unwrap_or_default()));
            respond(&stream, "200 OK", "")
        }
        "/zaber/prompt" => {
            let _ = app.emit("claude:working", note(&body, ""));
            respond(&stream, "200 OK", "")
        }
        "/zaber/tool" => {
            let _ = app.emit("claude:working", note(&body, &doing(body["tool_name"].as_str().unwrap_or("tool"), &body["tool_input"])));
            respond(&stream, "200 OK", "")
        }
        "/zaber/usage" => {
            let u = usage(&body);
            let line = usage_line(&u);
            if (u.session.is_some() || u.week.is_some()) && USAGE.lock().unwrap().replace(u.clone()).as_ref() != Some(&u) {
                let _ = app.emit("claude:usage", u);
            }
            respond(&stream, "200 OK", &line)
        }
        "/zaber/end" => {
            let _ = app.emit("claude:ended", note(&body, ""));
            respond(&stream, "200 OK", "")
        }
        "/zaber/permission" | "/zaber/elicitation" => {
            let permission = path.ends_with("permission");
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let tool = body["tool_name"].as_str().map(str::to_string);
            let suggestions = body["permission_suggestions"].as_array().cloned().unwrap_or_default();
            let fields = if permission { Vec::new() } else { fields(&body["requested_schema"]) };
            let req = Request {
                id,
                kind: if permission { "permission" } else { "question" },
                project: project(&body),
                session: body["session_id"].as_str().unwrap_or_default().to_string(),
                summary: if permission { summary(tool.as_deref().unwrap_or("tool"), &body["tool_input"]) } else { body["mcp_server_name"].as_str().unwrap_or_default().to_string() },
                message: body["message"].as_str().map(str::to_string),
                can_always: suggestions.iter().any(|s| s["behavior"] == "allow"),
                fields: fields.clone(),
                tool,
            };
            if !permission && body["mode"].as_str() == Some("url") {
                return respond(&stream, "200 OK", "");
            }
            let (tx, rx) = channel();
            PENDING.lock().unwrap().get_or_insert_with(HashMap::new).insert(
                id,
                Pending { kind: req.kind, suggestions, fields: fields.into_iter().map(|f| f.name).collect(), tx },
            );
            let _ = app.emit("claude:request", req);
            match rx.recv_timeout(Duration::from_secs(WAIT_SECS.load(Ordering::Relaxed))) {
                Ok(Value::Null) => respond(&stream, "200 OK", ""),
                Ok(reply) => respond(&stream, "200 OK", &reply.to_string()),
                Err(_) => {
                    PENDING.lock().unwrap().as_mut().map(|m| m.remove(&id));
                    let _ = app.emit("claude:expired", id);
                    respond(&stream, "200 OK", "")
                }
            }
        }
        _ => respond(&stream, "404 Not Found", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(kind: &'static str, suggestions: Vec<Value>) -> Pending {
        Pending { kind, suggestions, fields: vec!["db".into()], tx: channel().0 }
    }

    #[test]
    fn decisions_match_the_hook_schema() {
        let rule = json!({ "type": "addRules", "rules": [{ "toolName": "Bash", "ruleContent": "npm test" }], "behavior": "allow", "destination": "localSettings" });
        let p = pending("permission", vec![rule.clone()]);
        let once = decision(&p, Answer::Allow { always: false }).unwrap();
        assert_eq!(once["hookSpecificOutput"]["decision"], json!({ "behavior": "allow" }));
        let always = decision(&p, Answer::Allow { always: true }).unwrap();
        assert_eq!(always["hookSpecificOutput"]["decision"]["updatedPermissions"], json!([rule]));
        assert_eq!(decision(&p, Answer::Deny).unwrap()["hookSpecificOutput"]["decision"]["behavior"], "deny");
        assert!(decision(&p, Answer::Decline).is_err(), "question answer to a permission");
        assert!(decision(&p, Answer::Pass).unwrap().is_null(), "closed: no decision, the terminal asks");

        let q = pending("question", vec![]);
        let mut content = Map::new();
        content.insert("db".into(), json!("sqlite"));
        content.insert("sneaky".into(), json!(1));
        let a = decision(&q, Answer::Accept { content }).unwrap();
        assert_eq!(a["hookSpecificOutput"]["content"], json!({ "db": "sqlite" }), "only asked-for fields");
        assert_eq!(a["hookSpecificOutput"]["action"], "accept");
    }

    #[test]
    fn usage_reads_the_status_line_and_skips_missing_windows() {
        let u = usage(&json!({ "rate_limits": { "five_hour": { "used_percentage": 23.5, "resets_at": 1738425600 } } }));
        assert_eq!(u.session, Some(Limit { used: 23.5, resets_at: 1738425600 }));
        assert_eq!(u.week, None);
        assert_eq!(usage_line(&u), "5h 24%");
        assert_eq!(usage_line(&usage(&json!({}))), "");
    }

    #[test]
    fn summary_and_project() {
        assert_eq!(doing("Edit", &json!({ "file_path": "C:\\a\\Notch.svelte" })), "Edit Notch.svelte");
        assert_eq!(doing("Bash", &json!({ "command": "npm test\necho" })), "Bash npm test");
        assert_eq!(summary("Bash", &json!({ "command": "npm i", "description": "x" })), "npm i");
        assert_eq!(summary("Edit", &json!({ "file_path": "C:\\a\\b.ts" })), "C:\\a\\b.ts");
        assert_eq!(project(&json!({ "cwd": "C:\\Users\\me\\zaber\\" })), "zaber");
        let f = fields(&json!({ "properties": { "db": { "type": "string", "enum": ["pg", "sqlite"], "title": "Database" }, "n": { "type": "integer" } }, "required": ["db"] }));
        assert_eq!((f[0].kind.as_str(), f[0].required, f[0].options.len()), ("enum", true, 2));
        assert_eq!(f[1].kind, "number");
    }
}
