// F01, F02 and F05 (docs/acceptance.md) against a fake Bot API: `pilotq service run` polls and sends through a local HTTP
// server that keeps Telegram's offset confirmation and injects send faults. The live test-bot run is separate evidence.
#![cfg(unix)]

use serde_json::{Value, json};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const OWNER: i64 = 7;

#[derive(Default)]
struct Fake {
    updates: Vec<Value>,
    /// Bodies of sendMessage calls Telegram accepted, including one whose response was lost.
    sent: Vec<Value>,
    attempts: usize,
    faults: VecDeque<&'static str>,
    conflict: bool,
    polls: usize,
    /// Hold an empty getUpdates for its `timeout`, as Telegram does.
    hold: bool,
}

type State = Arc<Mutex<Fake>>;

fn update(id: i64, chat: i64, sender: i64) -> Value {
    json!({"update_id": id, "message": {"message_id": 100 + id, "date": 0, "chat": {"id": chat, "type": "private", "first_name": "A"},
        "from": {"id": sender, "is_bot": false, "first_name": "A"}, "text": "hi"}})
}

fn respond(s: &mut impl Write, code: u16, body: &str) {
    let _ = write!(s, "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
}

fn handle(stream: std::net::TcpStream, st: &State) {
    let mut r = BufReader::new(stream.try_clone().unwrap());
    let (mut line, mut len) = (String::new(), 0);
    r.read_line(&mut line).unwrap();
    let method = line.split_whitespace().nth(1).unwrap_or("").rsplit('/').next().unwrap().to_lowercase();
    loop {
        let mut h = String::new();
        r.read_line(&mut h).unwrap();
        if h.trim().is_empty() {
            break;
        }
        if let Some(v) = h.to_lowercase().strip_prefix("content-length:") {
            len = v.trim().parse().unwrap();
        }
    }
    let mut body = vec![0; len];
    r.read_exact(&mut body).unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let mut out = stream;
    if method == "getupdates" {
        let offset = body["offset"].as_i64().unwrap_or(0);
        let ready = {
            let mut f = st.lock().unwrap();
            f.polls += 1;
            if f.conflict {
                drop(f);
                return respond(&mut out, 409, r#"{"ok":false,"error_code":409,"description":"Conflict: terminated by other getUpdates request; make sure that only one bot instance is running"}"#);
            }
            f.updates.retain(|u| u["update_id"].as_i64().unwrap() >= offset); // Telegram forgets confirmed updates
            f.updates.clone()
        };
        if ready.is_empty() {
            let hold = st.lock().unwrap().hold;
            std::thread::sleep(if hold { Duration::from_secs(body["timeout"].as_u64().unwrap_or(0)) } else { Duration::from_millis(100) });
        }
        return respond(&mut out, 200, &json!({"ok": true, "result": ready}).to_string());
    }
    let mut f = st.lock().unwrap();
    f.attempts += 1;
    let ok = json!({"ok": true, "result": {"message_id": 900 + f.attempts, "date": 0, "chat": {"id": body["chat_id"], "type": "private", "first_name": "A"}, "text": body["text"]}});
    match f.faults.pop_front() {
        Some("drop") => f.sent.push(body), // accepted, then the response is lost
        Some("parse") if body.get("parse_mode").is_some() => respond(&mut out, 400, r#"{"ok":false,"error_code":400,"description":"Bad Request: can't parse entities: unsupported start tag"}"#),
        Some("429") => respond(&mut out, 429, r#"{"ok":false,"error_code":429,"description":"Too Many Requests: retry after 1","parameters":{"retry_after":1}}"#),
        Some("down") => {
            let _ = write!(out, "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html\r\nContent-Length: 11\r\nConnection: close\r\n\r\nBad Gateway");
        }
        Some("forbidden") => respond(&mut out, 403, r#"{"ok":false,"error_code":403,"description":"Forbidden: bot was blocked by the user"}"#),
        _ => {
            f.sent.push(body);
            respond(&mut out, 200, &ok.to_string());
        }
    }
}

/// Starts the fake Bot API and an install directory whose owner-only config points at it.
fn setup(tag: &str, updates: Vec<Value>) -> (PathBuf, State) {
    let d = std::env::temp_dir().join(format!("pilotq-tg-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    let st: State = Arc::new(Mutex::new(Fake { updates, ..Default::default() }));
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    let s = st.clone();
    std::thread::spawn(move || {
        for c in l.incoming().flatten() {
            let s = s.clone();
            std::thread::spawn(move || handle(c, &s));
        }
    });
    let cfg = d.join("config.json");
    fs::write(&cfg, json!({"bot_token": "123:fake", "owner_id": OWNER, "api_url": format!("http://127.0.0.1:{port}/")}).to_string()).unwrap();
    fs::set_permissions(&cfg, fs::Permissions::from_mode(0o600)).unwrap();
    (d, st)
}

fn spawn(d: &Path, fault: Option<&str>) -> Child {
    let mut c = Command::new(env!("CARGO_BIN_EXE_pilotq"));
    c.args(["service", "run"]).env("PILOTQ_INSTALL", d).stdout(Stdio::null()).stderr(Stdio::piped());
    if let Some(f) = fault {
        c.env("PILOTQ_FAULT", f);
    }
    c.spawn().unwrap()
}

fn wait(what: &str, secs: u64, mut ok: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !ok() {
        assert!(Instant::now() < end, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn query(d: &Path, sql: &str) -> i64 {
    try_query(d, sql).unwrap()
}

/// None until the service has created the journal.
fn try_query(d: &Path, sql: &str) -> Option<i64> {
    let c = rusqlite::Connection::open_with_flags(d.join("journal.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    c.query_row(sql, [], |r| r.get(0)).ok()
}

fn killed(mut c: Child) {
    let s = c.wait().unwrap();
    assert_eq!(s.signal(), Some(9), "the fault point must SIGKILL the service: {s:?}");
}

// F01: SIGKILL before the inbox commit, after it, and after cursor advancement. No acknowledged input disappears,
// Telegram's redelivery admits the same update once, and each request is acknowledged once.
#[test]
fn f01_crash_points_keep_each_input_once() {
    let (d, st) = setup("f01", vec![update(1, 11, OWNER)]);
    killed(spawn(&d, Some("after-fetch")));
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 0, "killed before commit: nothing admitted, cursor unchanged");
    assert_eq!(query(&d, "SELECT count(*) FROM meta WHERE key='cursor'"), 0);

    killed(spawn(&d, Some("after-commit")));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE update_id=1"), 1, "Telegram redelivered update 1; admitted once");
    assert_eq!(query(&d, "SELECT value FROM meta WHERE key='cursor'"), 2);
    assert!(st.lock().unwrap().sent.is_empty(), "killed before delivery");

    st.lock().unwrap().updates.push(update(2, 11, OWNER));
    let c = spawn(&d, None);
    wait("both acknowledgements", 15, || st.lock().unwrap().sent.len() == 2);
    wait("cursor past update 2", 15, || try_query(&d, "SELECT value FROM meta WHERE key='cursor'") == Some(3));
    let mut c = c;
    c.kill().unwrap(); // after cursor advancement
    c.wait().unwrap();

    let c = spawn(&d, None);
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 2);
    let sent = st.lock().unwrap().sent.clone();
    assert_eq!(sent.len(), 2, "no acknowledgement repeated after restart: {sent:?}");
    for (i, s) in sent.iter().enumerate() {
        assert_eq!(s["chat_id"], 11);
        assert_eq!(s["reply_parameters"]["message_id"], 101 + i as i64);
        assert!(s["text"].as_str().unwrap().contains(&format!("#{}", i + 1)));
    }
    let mut c = c;
    c.kill().unwrap();
    c.wait().unwrap();
    fs::remove_dir_all(d).unwrap();
}

// F02: a duplicate update admits one request; a stranger is ignored silently; a second local process cannot poll;
// a cross-host conflict stops polling and is visible as poll health.
#[test]
fn f02_duplicate_lock_and_conflict() {
    let (d, st) = setup("f02", vec![update(5, 21, OWNER), update(5, 21, OWNER), update(6, 22, 999)]);
    let mut c = spawn(&d, None);
    wait("acknowledgement", 15, || st.lock().unwrap().sent.len() == 1);
    wait("dispositions", 15, || try_query(&d, "SELECT count(*) FROM updates") == Some(2));
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 1);
    assert_eq!(query(&d, "SELECT count(*) FROM updates WHERE update_id=6 AND disposition='ignored' AND request_id IS NULL"), 1);

    let second = spawn(&d, None).wait_with_output().unwrap();
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("poll lock held"), "{}", String::from_utf8_lossy(&second.stderr));

    st.lock().unwrap().conflict = true;
    wait("conflict health", 15, || try_query(&d, "SELECT count(*) FROM health WHERE source='poll' AND state='conflict'") == Some(1));
    let polls = st.lock().unwrap().polls;
    std::thread::sleep(Duration::from_millis(1000));
    assert_eq!(st.lock().unwrap().polls, polls, "no retry loop after a conflict");
    assert!(c.try_wait().unwrap().is_none(), "the service stays up for status and delivery");
    assert!(st.lock().unwrap().sent.iter().all(|s| s["chat_id"] == 21), "nothing sent to the stranger");
    c.kill().unwrap();
    c.wait().unwrap();
    fs::remove_dir_all(d).unwrap();
}

// Live F02 run found every idle poll failing: Telegram holds an empty getUpdates for the whole timeout,
// which exceeded the HTTP client timeout. An idle long poll must end as poll health ok.
#[test]
fn idle_long_poll_is_healthy() {
    let (d, st) = setup("idle", vec![]);
    st.lock().unwrap().hold = true;
    let mut c = spawn(&d, None);
    wait("idle poll health", 25, || try_query(&d, "SELECT count(*) FROM health WHERE source='poll'") == Some(1));
    c.kill().unwrap();
    c.wait().unwrap();
    assert_eq!(query(&d, "SELECT count(*) FROM health WHERE source='poll' AND state='ok'"), 1, "idle long poll reported as failed");
    fs::remove_dir_all(d).unwrap();
}

// F05: lost send response, formatting rejection, 429 and an outage. The result stays durable, delivery is bounded,
// falls back to plain text and records the possible visible duplicate; the request is not admitted or run again.
#[test]
fn f05_delivery_faults_are_bounded() {
    let (d, st) = setup("f05", vec![update(9, 31, OWNER), update(10, 32, OWNER)]);
    st.lock().unwrap().faults = VecDeque::from(["drop", "forbidden", "parse", "429", "down"]);
    let mut c = spawn(&d, None);
    wait("delivery", 30, || try_query(&d, "SELECT count(*) FROM outbox WHERE state IN ('delivered','held')") == Some(2));
    c.kill().unwrap();
    c.wait().unwrap();
    assert_eq!(query(&d, "SELECT count(*) FROM outbox WHERE chat=31 AND state='delivered' AND attempts=5 AND possible_duplicate=1 AND html IS NULL"), 1);
    assert_eq!(query(&d, "SELECT count(*) FROM outbox WHERE chat=32 AND state='held'"), 1, "a blocked chat is held, never rerouted or marked delivered");
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 2);
    let f = st.lock().unwrap();
    let to31: Vec<_> = f.sent.iter().filter(|s| s["chat_id"] == 31).collect();
    assert_eq!(to31.len(), 2, "the lost response and the final plain send: {to31:?}");
    assert!(to31[0].get("parse_mode").is_some() && to31[1].get("parse_mode").is_none());
    drop(f);
    fs::remove_dir_all(d).unwrap();
}

fn cli(d: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pilotq")).args(args).env("PILOTQ_INSTALL", d).output().unwrap()
}

// U18 through the CLI and the running service: a stranger is ignored silently; `users grant` admits only new requests;
// revoke ignores them again. Writes print a readback, --json parses, and a refused write exits nonzero.
#[test]
fn u18_cli_grant_admits_only_new_requests() {
    let (d, st) = setup("u18", vec![update(1, 41, 41)]);
    let mut c = spawn(&d, None);
    wait("stranger ignored", 15, || try_query(&d, "SELECT count(*) FROM updates WHERE update_id=1 AND disposition='ignored'") == Some(1));

    let g = cli(&d, &["users", "grant", "41", "chat", "--json"]);
    assert!(g.status.success(), "{}", String::from_utf8_lossy(&g.stderr));
    let v: Value = serde_json::from_slice(&g.stdout).unwrap();
    assert_eq!(v["grants"], json!(["chat"]));
    assert_eq!(v["log"][0]["by"], OWNER, "the CLI acts as the configured owner");
    assert!(!cli(&d, &["users", "grant", "@alice", "chat"]).status.success(), "names are not user IDs");
    assert!(!cli(&d, &["users", "grant", "41", "root"]).status.success());
    assert!(String::from_utf8_lossy(&cli(&d, &["users", "show", "41"]).stdout).contains("41 grants: chat"));

    st.lock().unwrap().updates.push(update(2, 41, 41));
    wait("new request acknowledged", 15, || st.lock().unwrap().sent.len() == 1);
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 1, "the ignored update is not replayed");
    assert_eq!(query(&d, "SELECT update_id FROM requests"), 2);

    assert!(cli(&d, &["users", "revoke", "41"]).status.success());
    st.lock().unwrap().updates.push(update(3, 41, 41));
    wait("revoked input ignored", 15, || try_query(&d, "SELECT count(*) FROM updates WHERE update_id=3 AND disposition='ignored'") == Some(1));
    assert_eq!(st.lock().unwrap().sent.len(), 1, "no reply to revoked input");
    c.kill().unwrap();
    c.wait().unwrap();
    fs::remove_dir_all(d).unwrap();
}
