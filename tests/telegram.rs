// F01, F02, F05 and the manager cases U01, U12, U16, U19, U23 and F03 (docs/acceptance.md) against a fake Bot API and a fake
// Codex app-server: `pilotq service run` polls and sends through a local HTTP server that keeps Telegram's offset confirmation
// and injects send faults; a shell script speaks the app-server JSON-RPC lines. Live test-bot and real Codex runs are separate evidence.
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
    /// getUpdates fails with 401 while sendMessage keeps working (F06).
    poll_fail: bool,
    /// sendChatAction bodies (typing), kept apart from messages.
    actions: Vec<Value>,
    /// editMessageText answers "message to edit not found" (a deleted status message, L12).
    edit_gone: bool,
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
            if f.poll_fail {
                drop(f);
                return respond(&mut out, 401, r#"{"ok":false,"error_code":401,"description":"Unauthorized"}"#);
            }
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
    if method == "sendchataction" {
        f.actions.push(body);
        return respond(&mut out, 200, r#"{"ok":true,"result":true}"#);
    }
    let mut body = body;
    if method == "editmessagetext" {
        if f.edit_gone {
            return respond(&mut out, 400, r#"{"ok":false,"error_code":400,"description":"Bad Request: message to edit not found"}"#);
        }
        body["edit"] = json!(true);
    }
    f.attempts += 1;
    let id = if method == "editmessagetext" { body["message_id"].as_i64().unwrap() } else { 900 + f.attempts as i64 };
    let ok = json!({"ok": true, "result": {"message_id": id, "date": 0, "chat": {"id": body["chat_id"], "type": "private", "first_name": "A"}, "text": body["text"]}});
    match f.faults.pop_front() {
        Some("drop") => f.sent.push(body), // accepted, then the response is lost
        Some("parse") if body.get("parse_mode").is_some() => respond(&mut out, 400, r#"{"ok":false,"error_code":400,"description":"Bad Request: can't parse entities: unsupported start tag"}"#),
        Some("429") => respond(&mut out, 429, r#"{"ok":false,"error_code":429,"description":"Too Many Requests: retry after 1","parameters":{"retry_after":1}}"#),
        Some("down") => {
            let _ = write!(out, "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html\r\nContent-Length: 11\r\nConnection: close\r\n\r\nBad Gateway");
        }
        Some("forbidden") => respond(&mut out, 403, r#"{"ok":false,"error_code":403,"description":"Forbidden: bot was blocked by the user"}"#),
        _ => {
            body["id"] = json!(id);
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
    // a Codex that never answers keeps the ingress and delivery tests free of model replies
    let mute = script(&d, "mute", "while read -r l; do :; done");
    let c = json!({"bot_token": "123:fake", "owner_id": OWNER, "api_url": format!("http://127.0.0.1:{port}/"), "codex": mute, "codex_limits": {"first_event": 600}});
    fs::write(&cfg, c.to_string()).unwrap();
    fs::set_permissions(&cfg, fs::Permissions::from_mode(0o600)).unwrap();
    (d, st)
}

fn script(d: &Path, name: &str, body: &str) -> PathBuf {
    let p = d.join(name);
    fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    p
}

/// A fake `codex app-server`: logs every line it receives; answers "re: TEXT @ THREAD", resuming the requested thread.
/// The turn text picks a fault: slow, stall, exit, tool (exits after a tool starts), empty, failed, chatty, ask, once (sleeps on
/// its first run only), diag (an unknown optional event first) or env (answers with the API key it sees).
const FAKE: &str = r#"d=$(dirname "$0")
while IFS= read -r l; do
  printf '%s\n' "$l" >> "$d/codex.log"
  case "$l" in
  *'"method":"initialize"'*) echo '{"id":1,"result":{}}' ;;
  *'"method":"thread/start"'*) n=$(($(cat "$d/threads" 2>/dev/null || echo 0) + 1)); echo $n > "$d/threads"; th=th-$n
    echo '{"id":2,"result":{"thread":{"id":"'$th'"},"model":"gpt-test"}}' ;;
  *'"method":"thread/resume"'*) th=$(printf '%s' "$l" | sed 's/.*"threadId":"\([^"]*\)".*/\1/')
    echo '{"id":2,"result":{"thread":{"id":"'$th'"}}}' ;;
  *'"method":"turn/start"'*) text=$(printf '%s' "$l" | sed 's/.*"text":"\([^"]*\)".*/\1/')
    echo '{"id":3,"result":{"turn":{"id":"u1","status":"inProgress","items":[]}}}'
    done='{"method":"turn/completed","params":{"threadId":"'$th'","turn":{"id":"u1","status":"completed","items":[]}}}'
    case "$text" in
    slow*) sleep 20 ;;
    stall*) sleep 20; exit 0 ;;
    exit*) exit 0 ;;
    tool*) echo '{"method":"item/started","params":{"threadId":"'$th'","turnId":"u1","item":{"type":"commandExecution","id":"c1","command":"touch x","commandActions":[],"cwd":"/","status":"inProgress"}}}'; exit 0 ;;
    empty*) echo "$done"; continue ;;
    failed*) echo '{"method":"turn/completed","params":{"threadId":"'$th'","turn":{"id":"u1","status":"failed","items":[],"error":{"message":"usage limit reached"}}}}'; continue ;;
    chatty*) while :; do echo '{"method":"item/agentMessage/delta","params":{"delta":"."}}'; sleep 0.5; done ;;
    once*) [ -f "$d/once" ] || { touch "$d/once"; sleep 30; } ;;
    diag*) echo '{"method":"pilotq/unknownDiagnostic","params":{"x":1}}' ;;
    env*) text="key=${OPENAI_API_KEY:-none}${CODEX_API_KEY:-}" ;;
    ask*) echo '{"id":77,"method":"item/commandExecution/requestApproval","params":{}}'; IFS= read -r r; printf '%s\n' "$r" >> "$d/codex.log" ;;
    esac
    echo '{"method":"item/completed","params":{"threadId":"'$th'","turnId":"u1","completedAtMs":0,"item":{"type":"agentMessage","id":"m1","text":"re: '"$text"' @ '"$th"'"}}}'
    echo "$done" ;;
  esac
done"#;

/// Points the install's config at `codex` with short section 7 limits.
fn use_codex(d: &Path, codex: &Path) {
    limits(d, codex, json!({"first_event": 2, "silent": 2, "deadline": 4}));
}

fn limits(d: &Path, codex: &Path, limits: Value) {
    let cfg = d.join("config.json");
    let mut c: Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    c["codex"] = json!(codex);
    c["codex_limits"] = limits;
    fs::write(&cfg, c.to_string()).unwrap();
}

fn text(id: i64, chat: i64, topic: Option<i64>, sender: i64, text: &str) -> Value {
    let mut u = update(id, chat, sender);
    u["message"]["text"] = json!(text);
    if chat < 0 {
        u["message"]["chat"] = json!({"id": chat, "type": "supergroup", "title": "G", "is_forum": topic.is_some()});
    }
    if let Some(t) = topic {
        u["message"]["message_thread_id"] = json!(t);
        u["message"]["is_topic_message"] = json!(true);
    }
    u
}

fn texts(st: &State) -> Vec<String> {
    st.lock().unwrap().sent.iter().map(|s| s["text"].as_str().unwrap_or("").to_string()).collect()
}

fn service(d: &Path, fault: Option<&str>) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_pilotq"));
    c.args(["service", "run"]).env("PILOTQ_INSTALL", d).stdout(Stdio::null()).stderr(Stdio::piped());
    if let Some(f) = fault {
        c.env("PILOTQ_FAULT", f);
    }
    c
}

/// A running `pilotq service run` that is killed and reaped when dropped, so a failed or panicking test leaves no orphan.
struct Svc(Child);

impl std::ops::Deref for Svc {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}

impl std::ops::DerefMut for Svc {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}

impl Drop for Svc {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn(d: &Path, fault: Option<&str>) -> Svc {
    Svc(service(d, fault).spawn().unwrap())
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

fn killed(mut c: Svc) {
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

    let second = service(&d, None).output().unwrap();
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

fn sent_has(st: &State, needle: &str) -> bool {
    texts(st).iter().any(|t| t.contains(needle))
}

fn stop(mut c: Svc, d: PathBuf) {
    c.kill().unwrap();
    c.wait().unwrap();
    fs::remove_dir_all(d).unwrap();
}

// U01: a question and a follow-up in one DM. The follow-up resumes the same Codex thread; each request ends answered with a
// delivered reply after its acknowledgement.
#[test]
fn u01_follow_up_resumes_the_conversation() {
    let (d, st) = setup("u01", vec![text(1, OWNER, None, OWNER, "first")]);
    use_codex(&d, &script(&d, "fake-codex", FAKE));
    let c = spawn(&d, None);
    wait("first answer", 15, || sent_has(&st, "re: first @ th-1"));
    st.lock().unwrap().updates.push(text(2, OWNER, None, OWNER, "follow"));
    wait("follow-up answer", 15, || sent_has(&st, "re: follow @ th-1"));
    wait("deliveries recorded", 5, || try_query(&d, "SELECT count(*) FROM outbox WHERE state='delivered'") == Some(4));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE state='answered' AND reply LIKE 're: %'"), 2);
    let log = fs::read_to_string(d.join("codex.log")).unwrap();
    assert_eq!(log.matches("thread/start").count(), 1, "{log}");
    assert!(log.contains(r#""method":"thread/resume","params":{"threadId":"th-1""#), "{log}");
    let t = texts(&st);
    assert!(t[0].contains("<b>#1</b> is queued") && t[1].starts_with("re: first"), "acknowledgement precedes the answer: {t:?}");
    stop(c, d);
}

// U12: a slow turn, a follow-up and /stop in one topic, a question in another topic. /stop cancels only the running turn,
// the follow-up stays queued and then runs, the other topic answers meanwhile, and the stopped run never replies.
#[test]
fn u12_stop_is_scoped_and_fences_the_run() {
    let (d, st) = setup("u12", vec![text(1, -100, Some(5), OWNER, "slow one")]);
    use_codex(&d, &script(&d, "fake-codex", FAKE));
    let c = spawn(&d, None);
    wait("slow turn running", 15, || try_query(&d, "SELECT count(*) FROM runs WHERE state='running'") == Some(1));
    st.lock().unwrap().updates.extend([text(2, -100, Some(5), OWNER, "follow"), text(3, -100, Some(5), OWNER, "/stop"), text(4, -100, Some(6), OWNER, "other")]);
    wait("other topic answered", 15, || sent_has(&st, "re: other @"));
    wait("stop acknowledged", 15, || sent_has(&st, "Stopped request #1"));
    wait("follow-up answered", 20, || sent_has(&st, "re: follow @"));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE id=1 AND state='cancelled'"), 1);
    wait("stopped run fenced", 15, || try_query(&d, "SELECT count(*) FROM runs WHERE request_id=1 AND state='fenced'") == Some(1));
    std::thread::sleep(Duration::from_millis(500));
    assert!(!sent_has(&st, "re: slow"), "a fenced run cannot reply: {:?}", texts(&st));
    let f = st.lock().unwrap();
    let other = f.sent.iter().find(|s| s["text"].as_str().unwrap().starts_with("re: other")).unwrap();
    assert_eq!(other["message_thread_id"], 6, "the reply goes to its own topic");
    drop(f);
    stop(c, d);
}

// U16: with Codex unavailable the request fails visibly, while /status, /new and /stop answer without the model;
// /new keeps earlier requests and receipts.
#[test]
fn u16_controls_work_without_the_model() {
    let (d, st) = setup("u16", vec![text(1, OWNER, None, OWNER, "hello")]);
    use_codex(&d, &d.join("missing-codex"));
    let c = spawn(&d, None);
    wait("visible failure", 15, || sent_has(&st, "Request #1 failed: codex app-server did not start"));
    st.lock().unwrap().updates.extend([text(2, OWNER, None, OWNER, "/status"), text(3, OWNER, None, OWNER, "/new@pilotq_bot"), text(4, OWNER, None, OWNER, "/stop")]);
    wait("control replies", 15, || sent_has(&st, "Nothing is running in this conversation."));
    assert!(sent_has(&st, "Nothing is running or queued in this conversation."));
    assert!(sent_has(&st, "New conversation started."));
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 4, "/new keeps earlier requests");
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE kind='control' AND state='answered'"), 3);
    assert_eq!(query(&d, "SELECT generation FROM conversations WHERE session='dm:7'"), 1);
    assert_eq!(query(&d, "SELECT count(*) FROM health WHERE source='model' AND state='failed'"), 1);
    stop(c, d);
}

// U19: the same question through `pilotq ask` and a Telegram DM uses one grant check and request record. --json parses;
// a failed request exits nonzero.
#[test]
fn u19_cli_and_telegram_share_one_path() {
    let (d, st) = setup("u19", vec![]);
    use_codex(&d, &script(&d, "fake-codex", FAKE));
    let a = cli(&d, &["ask", "same", "--json"]);
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let v: Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!((v["state"].as_str(), v["reply"].as_str()), (Some("answered"), Some("re: same @ th-1")));
    let plain = cli(&d, &["ask", "plain"]);
    assert_eq!(String::from_utf8_lossy(&plain.stdout).trim(), "re: plain @ th-1", "the CLI conversation resumes too");
    let bad = cli(&d, &["ask", "--json", "failed"]);
    assert!(!bad.status.success());
    assert_eq!(serde_json::from_slice::<Value>(&bad.stdout).unwrap()["state"], "failed");
    assert!(!cli(&d, &["ask"]).status.success(), "no text is a usage error");

    st.lock().unwrap().updates.push(text(1, OWNER, None, OWNER, "same"));
    let c = spawn(&d, None);
    wait("Telegram answer", 15, || sent_has(&st, "re: same @ th-2"));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE sender=7 AND text='same' AND state='answered'"), 2);
    assert_eq!(query(&d, "SELECT count(DISTINCT front) FROM requests"), 2);
    stop(c, d);
}

// F03: stall, disconnect before and after a tool, an empty or failed terminal event, endless output and a server request.
// Every case ends bounded and never as false success; after a tool started the outcome is uncertain, not retryable.
#[test]
fn f03_provider_faults_are_bounded() {
    let (d, _st) = setup("f03", vec![]);
    use_codex(&d, &script(&d, "fake-codex", FAKE));
    let cases = [
        ("stall", "failed", "Codex sent no event for 2 s"),
        ("exit", "failed", "Codex exited without a terminal event"),
        ("tool", "uncertain", "Codex exited without a terminal event"),
        ("empty", "failed", "Codex ended the turn without an answer"),
        ("failed", "failed", "usage limit reached"),
        ("chatty", "failed", "Codex did not finish within 4 s"),
    ];
    for (input, state, error) in cases {
        let t = Instant::now();
        let o = cli(&d, &["ask", "--json", input]);
        assert!(t.elapsed() < Duration::from_secs(12), "{input}: unbounded ({:?})", t.elapsed());
        assert!(!o.status.success(), "{input}: no false success");
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["state"], state, "{input}: {v}");
        assert!(v["error"].as_str().unwrap().contains(error), "{input}: {v}");
    }
    // L07: an unknown optional event is ignored safely; L08: no API key reaches Codex, so there is no billed fallback
    let o = cli(&d, &["ask", "diag"]);
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "re: diag @ th-1", "{}", String::from_utf8_lossy(&o.stderr));
    let o = Command::new(env!("CARGO_BIN_EXE_pilotq")).args(["ask", "env"]).env("PILOTQ_INSTALL", &d).env("OPENAI_API_KEY", "sk-aaaaaaaa").env("CODEX_API_KEY", "sk-bbbbbbbb").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "re: key=none @ th-1", "API keys are removed from the Codex environment");
    let o = cli(&d, &["ask", "ask me"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(fs::read_to_string(d.join("codex.log")).unwrap().contains(r#"{"id":77,"error":"#), "the server request is refused");
    assert_eq!(query(&d, "SELECT count(*) FROM runs WHERE tool=1"), 1);

    // no first event at all
    use_codex(&d, &d.join("mute"));
    let o = cli(&d, &["ask", "--json", "hello"]);
    assert_eq!(serde_json::from_slice::<Value>(&o.stdout).unwrap()["error"], "Codex sent no first event within 2 s");
    fs::remove_dir_all(d).unwrap();
}

// U23 through the service and the CLI: a person in a DM, another in a forum topic and the CLI owner each ask; one fails after a
// tool started. `pilotq audit` as the owner shows every record with who, where, text, tools and outcome; each request ID
// finds its CoreQ trace, which carries no message text; each person's /audit DM shows only their own; every read is recorded.
#[test]
fn u23_audit_through_service_and_cli() {
    let (d, st) = setup("u23", vec![]);
    use_codex(&d, &script(&d, "fake-codex", FAKE));
    for u in ["8", "9"] {
        assert!(cli(&d, &["users", "grant", u, "chat"]).status.success());
    }
    assert!(cli(&d, &["ask", "owner question"]).status.success());
    st.lock().unwrap().updates.extend([text(1, 8, None, 8, "dm question"), text(2, -100, Some(5), 9, "tool topic question")]);
    let c = spawn(&d, None);
    wait("both outcomes", 20, || sent_has(&st, "re: dm question") && sent_has(&st, "uncertain outcome"));
    assert!(sent_has(&st, "Your requests are recorded (metadata 365 days"), "admission tells the person");

    let a = cli(&d, &["audit", "--json"]);
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let all: Value = serde_json::from_slice(&a.stdout).unwrap();
    let all = all.as_array().unwrap();
    assert_eq!(all.len(), 3, "one record per request: {all:?}");
    assert_eq!((all[0]["where"]["front"].as_str(), all[0]["outcome"].as_str()), (Some("cli"), Some("answered")));
    let dm = &all[1];
    assert_eq!((dm["who"]["id"].as_i64(), dm["who"]["name"].as_str(), dm["text"].as_str()), (Some(8), Some("A"), Some("dm question")));
    assert_eq!((dm["runs"][0]["model"].as_str(), dm["deliveries"][0]["state"].as_str()), (Some("gpt-test"), Some("delivered")));
    let topic = &all[2];
    assert_eq!((topic["where"]["chat"].as_i64(), topic["where"]["topic"].as_i64(), topic["outcome"].as_str()), (Some(-100), Some(5), Some("uncertain")));
    assert_eq!(topic["tools"][0]["target"], "commandExecution");
    let one = cli(&d, &["audit", "--request", "2"]);
    assert!(String::from_utf8_lossy(&one.stdout).starts_with("#2 at "), "{}", String::from_utf8_lossy(&one.stdout));
    assert!(!cli(&d, &["audit", "--user", "x"]).status.success(), "a malformed filter exits nonzero");

    st.lock().unwrap().updates.extend([text(3, 8, None, 8, "/audit"), text(4, 9, None, 9, "/audit")]);
    wait("audit replies", 15, || texts(&st).iter().filter(|t| t.contains("Latest:")).count() == 2);
    let t = texts(&st);
    let mine = |who: &str| t.iter().find(|x| x.contains("Latest:") && x.contains(who)).cloned().unwrap_or_default();
    assert!(!mine("dm question").contains("topic question") && !mine("dm question").contains("owner"), "{t:?}");
    assert!(!mine("topic question").contains("dm question"), "{t:?}");
    assert_eq!(query(&d, "SELECT count(*) FROM audit_reads"), 4, "every read is recorded");
    let mut c = c;
    c.kill().unwrap();
    c.wait().unwrap();

    let tel = coreq::telemetry::Reader::open(&d.join("telemetry.sqlite")).unwrap().export(0, 0, 100).unwrap().to_string();
    for r in all {
        assert!(tel.contains(&format!("\"{}\"", r["trace"].as_str().unwrap())), "request {} has no CoreQ trace", r["request"]);
    }
    assert!(!tel.contains("question"), "no message text in telemetry");
    fs::remove_dir_all(d).unwrap();
}

// F06, L18: polling fails with 401 while sending still works. Poll health turns failed with its reason, delivery health stays ok
// and the committed acknowledgement is still delivered.
#[test]
fn f06_poll_failure_is_visible_while_delivery_works() {
    let (d, st) = setup("f06", vec![update(1, 61, OWNER)]);
    st.lock().unwrap().faults = VecDeque::from(["429"]);
    let c = spawn(&d, None);
    wait("admitted", 15, || try_query(&d, "SELECT count(*) FROM requests") == Some(1));
    st.lock().unwrap().poll_fail = true;
    wait("poll failed", 15, || try_query(&d, "SELECT count(*) FROM health WHERE source='poll' AND state='failed' AND reason IS NOT NULL") == Some(1));
    wait("delivered", 15, || try_query(&d, "SELECT count(*) FROM outbox WHERE state='delivered'") == Some(1));
    assert_eq!(query(&d, "SELECT count(*) FROM health WHERE source='delivery' AND state='ok'"), 1);
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(query(&d, "SELECT count(*) FROM health WHERE source='poll' AND state='failed'"), 1, "no later success hides the failure");
    stop(c, d);
}

// Step 2 crash qualification (F01, F03, L15): SIGKILL the service during a Codex turn. On restart the run is interrupted, the pure
// model request is requeued within its attempt budget and answered once; the acknowledgement is not repeated.
#[test]
fn crash_during_turn_requeues_once() {
    let (d, st) = setup("crash", vec![text(1, OWNER, None, OWNER, "once")]);
    limits(&d, &script(&d, "fake-codex", FAKE), json!({"first_event": 5, "silent": 60, "deadline": 60}));
    let mut c = spawn(&d, None);
    wait("turn running", 15, || d.join("once").exists() && try_query(&d, "SELECT count(*) FROM runs WHERE state='running'") == Some(1));
    wait("acknowledged", 15, || st.lock().unwrap().sent.len() == 1);
    c.kill().unwrap();
    c.wait().unwrap();
    let c = spawn(&d, None);
    wait("answer after restart", 20, || sent_has(&st, "re: once @ th-2"));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE state='answered'"), 1);
    assert_eq!(query(&d, "SELECT count(*) FROM runs WHERE request_id=1 AND state='interrupted'"), 1);
    assert_eq!(texts(&st).iter().filter(|t| t.contains("is queued")).count(), 1, "the acknowledgement is not repeated: {:?}", texts(&st));
    stop(c, d);
}

// Found in the A8 run: three conversations started Codex together on a fresh Codex home, and all but one app-server exited with
// `failed to initialize sqlite state runtime`, read by PilotQ only as "Codex exited without a terminal event". Startup is now
// serialized until `initialize` answers, so every request is answered; a Codex that exits early names its stderr reason.
#[test]
fn concurrent_codex_starts_do_not_race() {
    let (d, st) = setup("race", vec![text(1, 11, None, OWNER, "a"), text(2, -100, Some(5), OWNER, "b"), text(3, -100, Some(6), OWNER, "c")]);
    script(&d, "fake-codex", FAKE);
    // like Codex's state setup: an app-server starting while another initializes the home fails
    let racy = script(&d, "racy-codex", r#"d=$(dirname "$0")
mkdir "$d/init.lock" 2>/dev/null || { echo 'Error: failed to initialize sqlite state runtime' >&2; exit 1; }
sleep 0.5; rmdir "$d/init.lock"; exec "$d/fake-codex""#);
    use_codex(&d, &racy);
    let c = spawn(&d, None);
    wait("three answers", 20, || ["re: a @", "re: b @", "re: c @"].iter().all(|t| sent_has(&st, t)));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE state='answered'"), 3);
    stop(c, d);

    let (d, _st) = setup("dead", vec![]);
    use_codex(&d, &script(&d, "dead-codex", "echo 'Error: boom' >&2; exit 1"));
    let o = cli(&d, &["ask", "--json", "hi"]);
    assert_eq!(serde_json::from_slice::<Value>(&o.stdout).unwrap()["error"], "Codex exited without a terminal event after: Error: boom");
    fs::remove_dir_all(d).unwrap();
}

// Live test-bot run found Telegram service messages (forum topic created, group migrated, group created) answered as unsupported
// input. They leave an ID-only disposition and no reply; a person's photo still gets the explicit unsupported outcome (U14).
#[test]
fn service_messages_are_not_input() {
    let mut topic = text(1, -100, Some(3), OWNER, "x");
    topic["message"].as_object_mut().unwrap().remove("text");
    topic["message"]["forum_topic_created"] = json!({"name": "Topic 2", "icon_color": 7322096});
    let mut migrated = update(2, -50, OWNER);
    migrated["message"]["chat"] = json!({"id": -50, "type": "group", "title": "G"});
    migrated["message"].as_object_mut().unwrap().remove("text");
    migrated["message"]["migrate_to_chat_id"] = json!(-100);
    let mut created = migrated.clone();
    created["update_id"] = json!(3);
    created["message"].as_object_mut().unwrap().remove("migrate_to_chat_id");
    created["message"]["group_chat_created"] = json!(true);
    let mut photo = text(4, -100, Some(3), OWNER, "x");
    photo["message"].as_object_mut().unwrap().remove("text");
    photo["message"]["photo"] = json!([{"file_id": "f", "file_unique_id": "u", "width": 1, "height": 1}]);
    let (d, st) = setup("service", vec![topic, migrated, created, photo]);
    let c = spawn(&d, None);
    wait("dispositions", 15, || try_query(&d, "SELECT count(*) FROM updates") == Some(4));
    wait("photo outcome", 15, || sent_has(&st, "not supported yet"));
    assert_eq!(query(&d, "SELECT count(*) FROM requests"), 1, "only the photo is a request");
    assert_eq!(query(&d, "SELECT count(*) FROM updates WHERE disposition='unsupported' AND request_id IS NULL"), 3);
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(st.lock().unwrap().sent.len(), 1, "no reply to service messages: {:?}", texts(&st));
    stop(c, d);
}

// Section 10 step 3 (task #18): the TaskQ tools against a fake TaskQ CLI and a fake Codex that makes one dynamic tool call per
// turn. Board state lives in the fake's JSON file; tests change it as workers and reviewers would. The live board is separate.

/// A fake `taskq` CLI: logs every argument array, keeps tasks and op-id receipts in `taskq-state.json`, returns the first
/// receipt for a seen op ID, refuses an answer at another revision as stale. `board-down` fails every call; `lose-add-K`
/// creates the K-th task, then loses the response and takes the board down.
const FAKE_TASKQ: &str = r#"#!/usr/bin/env python3
import json, os, sys, fcntl
d = os.path.dirname(os.path.abspath(__file__))
a = sys.argv[1:]
with open(os.path.join(d, "taskq.log"), "a") as f:
    f.write(json.dumps({"args": a, "root": os.environ.get("TASKQ_ROOT"), "attempt": os.environ.get("TASKQ_ATTEMPT")}) + "\n")
lock = open(os.path.join(d, "taskq.lock"), "w"); fcntl.flock(lock, fcntl.LOCK_EX)
path = os.path.join(d, "taskq-state.json")
st = json.load(open(path)) if os.path.exists(path) else {"tasks": {}, "ops": {}, "next": 19, "rev": 100, "creates": 0}
def save():
    json.dump(st, open(path + ".tmp", "w")); os.replace(path + ".tmp", path)
def opt(k):
    return a[a.index(k) + 1] if k in a else None
def bump():
    st["rev"] += 1; return st["rev"]
def receipt(n, outcome, reason=None):
    t = st["tasks"].get(str(n), {})
    return {"task": n, "op_id": opt("--op-id"), "outcome": outcome, "state": t.get("state"), "revision": t.get("revision"), "reason": reason, "next": None}
def out(r, code=0):
    if r["outcome"] == "accepted" and opt("--op-id"): st["ops"][opt("--op-id")] = r
    save(); print(json.dumps(r)); sys.exit(code)
if os.path.exists(os.path.join(d, "board-down")):
    print("Error: board unreachable: connection refused", file=sys.stderr); sys.exit(1)
cmd, op = a[0], opt("--op-id")
if op in st["ops"]:
    print(json.dumps(st["ops"][op])); sys.exit(0)
if cmd == "status":
    tasks = [dict(t, n=int(n), url="https://github.com/taskqorg/pilotq/issues/" + n) for n, t in st["tasks"].items() if not t.get("closed")]
    print(json.dumps({"schema": 1, "source": "board", "observed": 1, "complete": True, "reason": None, "board": {"repo": "taskqorg/pilotq"}, "tasks": tasks})); sys.exit(0)
if cmd == "add":
    st["creates"] += 1; n = st["next"]; st["next"] += 1
    st["tasks"][str(n)] = {"title": a[1], "state": "held", "revision": bump(), "card": None, "args": a}
    r = receipt(n, "accepted")
    lose = os.path.join(d, "lose-add-%d" % st["creates"])
    if os.path.exists(lose):
        os.remove(lose); st["ops"][op] = r; save(); open(os.path.join(d, "board-down"), "w").close()
        print("Error: connection reset by peer", file=sys.stderr); sys.exit(1)
    out(r)
if cmd == "service":
    st["paused"] = True; save(); print("service pause: ok"); sys.exit(0)
n = a[1].split(".")[0]; t = st["tasks"][n]
if cmd == "start":
    t["state"] = "ready"; t["revision"] = bump(); out(receipt(int(n), "accepted"))
if cmd == "answer":
    if t["state"] != "ask": out(receipt(int(n), "rejected", "not in ask"), 1)
    if int(opt("--revision")) != t["revision"]: out(receipt(int(n), "stale", "current revision %d" % t["revision"]), 1)
    t.update(state="doing", card=None, answer=opt("--text") or a[1], revision=bump()); out(receipt(int(n), "accepted"))
if cmd == "later":
    t["state"] = "later"; t["revision"] = bump(); out(receipt(int(n), "accepted"))
print("Error: unknown command", file=sys.stderr); sys.exit(1)
"#;

/// A fake `codex app-server` in Python: `call NAME JSON` makes one complete dynamic tool call and answers "re: ok: RESULT" or
/// "re: refused: RESULT"; `slow S` sleeps S seconds; anything else answers "re: TEXT".
const FAKE_TOOLS: &str = r#"#!/usr/bin/env python3
import json, os, sys, time
d = os.path.dirname(os.path.abspath(__file__))
log = open(os.path.join(d, "codex.log"), "a")
def send(v):
    print(json.dumps(v), flush=True)
th = None
for line in sys.stdin:
    log.write(line); log.flush()
    m = json.loads(line); meth = m.get("method")
    if meth == "initialize":
        send({"id": 1, "result": {}})
    elif meth == "thread/start":
        p = os.path.join(d, "threads"); n = int(open(p).read()) + 1 if os.path.exists(p) else 1
        open(p, "w").write(str(n)); th = "th-%d" % n
        send({"id": 2, "result": {"thread": {"id": th}, "model": "gpt-test"}})
    elif meth == "thread/resume":
        th = m["params"]["threadId"]; send({"id": 2, "result": {"thread": {"id": th}}})
    elif meth == "turn/start":
        text = m["params"]["input"][0]["text"]
        send({"id": 3, "result": {"turn": {"id": "u1", "status": "inProgress", "items": []}}})
        if text.startswith("slow "):
            time.sleep(float(text.split()[1]))
        if text.startswith("call "):
            _, name, args = text.split(" ", 2)
            item = {"type": "dynamicToolCall", "id": "k1", "tool": name, "arguments": json.loads(args), "status": "inProgress"}
            send({"method": "item/started", "params": {"threadId": th, "turnId": "u1", "item": item}})
            send({"id": 50, "method": "item/tool/call", "params": {"threadId": th, "turnId": "u1", "callId": "k1", "tool": name, "arguments": json.loads(args)}})
            r = {}
            while r.get("id") != 50:
                r = json.loads(sys.stdin.readline()); log.write(json.dumps(r) + "\n"); log.flush()
            res = r["result"]
            text = ("ok: " if res["success"] else "refused: ") + res["contentItems"][0]["text"]
            send({"method": "item/completed", "params": {"threadId": th, "turnId": "u1", "item": dict(item, status="completed")}})
        send({"method": "item/completed", "params": {"threadId": th, "turnId": "u1", "completedAtMs": 0, "item": {"type": "agentMessage", "id": "m1", "text": "re: " + text}}})
        send({"method": "turn/completed", "params": {"threadId": th, "turn": {"id": "u1", "status": "completed", "items": []}}})
"#;

/// An install wired to the fake Codex and fake TaskQ with a 1 s board scan; `claim` sets `create_claim_issue` on the root.
fn tq_setup(tag: &str, claim: bool) -> (PathBuf, State) {
    let (d, st) = setup(tag, vec![]);
    let (codex, taskq) = (d.join("fake-codex"), d.join("fake-taskq"));
    for (p, body) in [(&codex, FAKE_TOOLS), (&taskq, FAKE_TASKQ)] {
        fs::write(p, body).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::create_dir_all(d.join("root")).unwrap();
    root(&d, claim);
    limits(&d, &codex, json!({"first_event": 10, "silent": 10, "deadline": 20}));
    let cfg = d.join("config.json");
    let mut c: Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    c["taskq"] = json!({"bin": taskq, "root": d.join("root"), "poll": 1});
    fs::write(&cfg, c.to_string()).unwrap();
    (d, st)
}

fn root(d: &Path, claim: bool) {
    let mut t = json!({"board": "github", "repo": "taskqorg/pilotq", "publish": "pr"});
    if claim {
        t["create_claim_issue"] = json!(1);
    }
    fs::write(d.join("root").join("taskq.json"), t.to_string()).unwrap();
}

fn board(d: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(d.join("taskq-state.json")).unwrap_or("{}".into())).unwrap()
}

/// Changes one fake task as a worker, reviewer or the board would, and gives it a newer revision unless `f` sets one.
fn set_task(d: &Path, n: i64, f: impl FnOnce(&mut Value)) {
    let mut s = board(d);
    s["rev"] = json!(s["rev"].as_i64().unwrap() + 1);
    let rev = s["rev"].clone();
    let t = &mut s["tasks"][n.to_string()];
    t["revision"] = rev;
    f(t);
    fs::write(d.join("taskq-state.json.t"), s.to_string()).unwrap();
    fs::rename(d.join("taskq-state.json.t"), d.join("taskq-state.json")).unwrap();
}

/// TaskQ argument arrays in call order, without the periodic status reads.
fn tq_calls(d: &Path) -> Vec<Vec<String>> {
    let log = fs::read_to_string(d.join("taskq.log")).unwrap_or_default();
    let v = log.lines().map(|l| serde_json::from_str::<Value>(l).unwrap());
    v.filter(|v| v["args"][0] != "status").map(|v| v["args"].as_array().unwrap().iter().map(|a| a.as_str().unwrap().to_string()).collect()).collect()
}

fn call(id: i64, chat: i64, topic: Option<i64>, sender: i64, tool: &str, args: Value) -> Value {
    text(id, chat, topic, sender, &format!("call {tool} {args}"))
}

fn plan(titles: &[&str]) -> Value {
    json!({"tickets": titles.iter().map(|t| json!({"title": t, "goal": "Make the README name the ask command.", "acceptance": ["README names pilotq ask."]})).collect::<Vec<_>>()})
}

fn reply(id: i64, chat: i64, topic: Option<i64>, sender: i64, to: i64, body: &str) -> Value {
    let mut u = text(id, chat, topic, sender, body);
    u["message"]["reply_to_message"] = json!({"message_id": to, "date": 0, "chat": u["message"]["chat"].clone(), "text": "question"});
    u
}

fn push(st: &State, u: Value) {
    st.lock().unwrap().updates.push(u);
}

/// Messages and edits Telegram accepted, in order.
fn sent(st: &State) -> Vec<Value> {
    st.lock().unwrap().sent.clone()
}

// U05, U06, L01, L03, L12, F14: three plan revisions start nothing; authorizing revision 2 cannot run revision 3; revision 3
// is filed held with an op ID, read back and started, with the author, route and revision in the task. One status message
// names what the task waits for and is edited in place; a stale observation changes nothing; the closing outcome is its own
// message and nothing edits it afterwards; no hung or stall warning appears.
#[test]
fn u06_plan_revisions_then_held_create_start_and_one_status_message() {
    let (d, st) = tq_setup("u06", true);
    let c = spawn(&d, None);
    for (i, t) in ["Draft one", "Draft two", "Draft three"].into_iter().enumerate() {
        push(&st, call(i as i64 + 1, OWNER, None, OWNER, "task_plan", plan(&[t])));
        wait("plan revision", 15, || sent_has(&st, &format!("re: ok: Plan revision {} saved", i + 1)));
    }
    assert!(tq_calls(&d).is_empty(), "a plan-only request writes nothing to the board: {:?}", tq_calls(&d));
    push(&st, call(4, OWNER, None, OWNER, "task_run_plan", json!({"revision": 2})));
    wait("revision 2 refused", 15, || sent_has(&st, "re: refused: revision 2 is not the latest plan revision (3)"));
    assert!(tq_calls(&d).is_empty(), "an older revision files nothing");

    push(&st, call(5, OWNER, None, OWNER, "task_run_plan", json!({"revision": 3})));
    wait("filed and started", 15, || sent_has(&st, "re: ok: Plan revision 3: filed held, read back and started: #19 https://github.com/taskqorg/pilotq/issues/19"));
    let calls = tq_calls(&d);
    assert_eq!((calls.len(), calls[0][0].as_str(), calls[0][1].as_str(), calls[1][0].as_str(), calls[1][1].as_str()), (2, "add", "Draft three", "start", "19"), "{calls:?}");
    let op = calls[0].iter().skip_while(|a| *a != "--op-id").nth(1).unwrap().clone();
    assert!(op.starts_with("pilotq-") && op.ends_with("-t1"), "{op}");
    assert!(calls[1].contains(&format!("{op}.start")), "{calls:?}");
    assert!(calls[0].contains(&"Filed by PilotQ pilotq.request.5 for Telegram user 7 in DM 7, plan revision 3.".to_string()), "{calls:?}");
    let log = fs::read_to_string(d.join("taskq.log")).unwrap();
    assert!(log.contains(&format!("\"root\": \"{}\"", d.join("root").display())) && log.contains("\"attempt\": null"), "{log}");

    wait("status message", 15, || sent_has(&st, "Task #19 ready: waiting for a free worker."));
    let status = sent(&st).into_iter().find(|s| s["text"].as_str().unwrap().starts_with("Task #19 ready")).unwrap();
    let id = status["id"].as_i64().unwrap();
    set_task(&d, 19, |t| t["state"] = json!("doing"));
    wait("edit to doing", 15, || sent(&st).iter().any(|s| s["edit"] == true && s["message_id"] == id && s["text"].as_str().unwrap().starts_with("Task #19 doing")));
    // F14: an observation older than the stored revision is ignored
    set_task(&d, 19, |t| {
        t["state"] = json!("ready");
        t["revision"] = json!(50);
    });
    std::thread::sleep(Duration::from_millis(2500));
    assert!(!sent(&st).iter().any(|s| s["edit"] == true && s["text"].as_str().unwrap().starts_with("Task #19 ready")), "a stale projection overwrote a newer status");
    set_task(&d, 19, |t| t["state"] = json!("review"));
    wait("edit to review", 15, || sent(&st).iter().any(|s| s["edit"] == true && s["message_id"] == id && s["text"].as_str().unwrap().contains("a reviewer checks it")));
    set_task(&d, 19, |t| t["closed"] = json!(true));
    wait("closing outcome", 15, || sent_has(&st, "Task #19 is closed on the board"));
    let fin = sent(&st).into_iter().find(|s| s["text"].as_str().unwrap().starts_with("Task #19 is closed")).unwrap();
    assert!(fin.get("edit").is_none() && fin["id"] != id, "the outcome is its own message");
    std::thread::sleep(Duration::from_millis(2000));
    let all = sent(&st);
    assert!(!all.iter().any(|s| s["edit"] == true && s["message_id"] == fin["id"]), "nothing edits the final outcome");
    assert_eq!(all.iter().filter(|s| s.get("edit").is_none() && s["text"].as_str().unwrap().starts_with("Task #19 ")).count(), 2, "one status message and one outcome: {all:?}");
    assert!(!all.iter().any(|s| s["text"].as_str().unwrap().contains("stall") || s["text"].as_str().unwrap().contains("hung")));
    let a: Value = serde_json::from_slice(&cli(&d, &["audit", "--request", "5", "--json"]).stdout).unwrap();
    assert_eq!(a[0]["tasks"].as_array().unwrap().iter().map(|t| t["target"].as_str().unwrap()).collect::<Vec<_>>(), ["add #19", "start #19"]);
    stop(c, d);
}

// U15: a three-ticket plan loses the network after the board created ticket two. Tickets one and two stay held and nothing
// starts; after a restart the scan retries the same op IDs, finds ticket two instead of creating it again, files ticket three
// and starts all three only after every ticket read back held.
#[test]
fn u15_network_loss_after_ticket_two_reconciles_the_same_ids() {
    let (d, st) = tq_setup("u15", true);
    fs::write(d.join("lose-add-2"), "").unwrap();
    let c = spawn(&d, None);
    push(&st, call(1, OWNER, None, OWNER, "task_plan", plan(&["One", "Two", "Three"])));
    wait("plan", 15, || sent_has(&st, "Plan revision 1 saved"));
    push(&st, call(2, OWNER, None, OWNER, "task_run_plan", json!({"revision": 1})));
    wait("unsent reply", 15, || sent_has(&st, "ticket 2 is unsent (taskq add failed: Error: connection reset by peer). Filed so far (held, not started): #19"));
    let b = board(&d);
    assert_eq!((b["tasks"]["19"]["state"].as_str(), b["tasks"]["20"]["state"].as_str()), (Some("held"), Some("held")), "created tickets stay held");
    assert!(tq_calls(&d).iter().all(|c| c[0] == "add"), "nothing starts before every ticket reads back");
    drop(c);

    fs::remove_file(d.join("board-down")).unwrap();
    let c = spawn(&d, None);
    wait("started after restart", 20, || sent_has(&st, "Plan revision 1: filed held, read back and started: #19"));
    let b = board(&d);
    assert_eq!(b["creates"], 3, "ticket two was found by its op ID, not created again");
    assert!(["19", "20", "21"].iter().all(|n| b["tasks"][n]["state"] == "ready"), "{b}");
    let calls = tq_calls(&d);
    let op = |c: &Vec<String>| c.iter().skip_while(|a| *a != "--op-id").nth(1).cloned().unwrap_or_default();
    let adds: Vec<String> = calls.iter().filter(|c| c[0] == "add").map(op).collect();
    assert!(adds.iter().filter(|o| o.ends_with("-t2")).count() >= 2 && adds.iter().filter(|o| o.ends_with("-t1")).count() == 1, "{adds:?}");
    let first_start = calls.iter().position(|c| c[0] == "start").unwrap();
    assert!(calls[first_start..].iter().all(|c| c[0] == "start") && calls[first_start..].len() == 3, "start only after every create: {calls:?}");
    assert!(sent_has(&st, "#20 https://github.com/taskqorg/pilotq/issues/20, #21"));
    stop(c, d);
}

// U07, U08, L02: the worker question reaches the originating forum topic; a reply to the older question writes nothing and
// shows the current one; a person without the taskq grant writes nothing and is told why; the owner's reply to the current
// question is written with its revision and op ID. Typing goes to the same topic while a turn runs and stops after it.
#[test]
fn u07_question_relay_and_bound_answers() {
    let (d, st) = tq_setup("u07", true);
    assert!(cli(&d, &["users", "grant", "8", "chat"]).status.success());
    let c = spawn(&d, None);
    let topic = Some(5);
    push(&st, text(1, -100, topic, OWNER, "slow 5"));
    wait("typing in the topic", 15, || st.lock().unwrap().actions.iter().any(|a| a["chat_id"] == -100 && a["message_thread_id"] == 5 && a["action"] == "typing"));
    wait("slow answer", 15, || sent_has(&st, "re: slow 5"));
    let typed = st.lock().unwrap().actions.len();
    std::thread::sleep(Duration::from_secs(5));
    assert_eq!(st.lock().unwrap().actions.len(), typed, "typing stops after the terminal state");
    assert!(st.lock().unwrap().actions.iter().all(|a| a["message_thread_id"] == 5), "typing only in the originating topic");

    push(&st, call(2, -100, topic, OWNER, "task_plan", plan(&["Docs"])));
    wait("plan", 15, || sent_has(&st, "Plan revision 1 saved"));
    push(&st, call(3, -100, topic, OWNER, "task_run_plan", json!({"revision": 1})));
    wait("started", 15, || sent_has(&st, "started: #19"));
    set_task(&d, 19, |t| {
        t["state"] = json!("ask");
        t["card"] = json!("Done: drafted\nProblem: pick a wording\n1. short ★\n2. long");
    });
    let question = |needle: &str| sent(&st).into_iter().find(|s| s["text"].as_str().unwrap().starts_with("Task #19 asks:") && s["text"].as_str().unwrap().contains(needle));
    wait("first question", 15, || question("short ★").is_some());
    let q1 = question("short ★").unwrap();
    assert_eq!((q1["chat_id"].as_i64(), q1["message_thread_id"].as_i64()), (Some(-100), Some(5)), "the question reaches the originating topic");
    set_task(&d, 19, |t| t["card"] = json!("Done: drafted twice\nProblem: pick again\n1. tiny ★\n2. huge"));
    wait("second question", 15, || question("tiny ★").is_some());
    let q2 = question("tiny ★").unwrap();

    push(&st, reply(4, -100, topic, OWNER, q1["id"].as_i64().unwrap(), "short"));
    wait("stale answer refused", 15, || sent_has(&st, "That question is no longer current; nothing was written. Task #19 now asks:\nDone: drafted twice"));
    push(&st, reply(5, -100, topic, 8, q2["id"].as_i64().unwrap(), "2"));
    wait("foreign answer refused", 15, || sent_has(&st, "Not written to task #19: 8 has no project:pilotq grant"));
    assert!(!tq_calls(&d).iter().any(|c| c[0] == "answer"), "a stale or foreign answer writes nothing");

    push(&st, reply(6, -100, topic, OWNER, q2["id"].as_i64().unwrap(), "2"));
    wait("answer written", 15, || sent_has(&st, "Answer written to task #19; it is now doing: a worker is on it."));
    let answers: Vec<_> = tq_calls(&d).into_iter().filter(|c| c[0] == "answer").collect();
    let rev = board(&d)["tasks"]["19"]["revision"].as_i64().unwrap() - 1;
    assert_eq!(answers.len(), 1);
    assert_eq!(&answers[0][..4], ["answer", "19.2", "--revision", &rev.to_string()], "{answers:?}");
    assert!(answers[0].iter().any(|a| a.starts_with("pilotq-") && a.contains("-a")), "{answers:?}");
    assert!(sent(&st).iter().filter(|s| s["text"].as_str().unwrap().contains("written")).all(|s| s["message_thread_id"] == 5));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE kind='answer'"), 3, "each reply is one recorded request");
    stop(c, d);
}

// U10, U13, L06, L12, L16: a transition made while PilotQ is down is found by the scan after restart; /new keeps the task;
// status by issue URL creates nothing; another board's URL is refused; a queue pause reports that the worker still runs;
// parking reports TaskQ's later state; a person without the grant changes nothing; a deleted status message is replaced.
#[test]
fn u10_restart_new_status_pause_and_park() {
    let (d, st) = tq_setup("u10", true);
    assert!(cli(&d, &["users", "grant", "8", "chat"]).status.success());
    let c = spawn(&d, None);
    push(&st, call(1, OWNER, None, OWNER, "task_plan", plan(&["Docs"])));
    wait("plan", 15, || sent_has(&st, "Plan revision 1 saved"));
    push(&st, call(2, OWNER, None, OWNER, "task_run_plan", json!({"revision": 1})));
    wait("status", 15, || sent_has(&st, "Task #19 ready"));
    drop(c);
    set_task(&d, 19, |t| t["state"] = json!("doing"));
    let c = spawn(&d, None);
    wait("missed transition recovered", 15, || sent(&st).iter().any(|s| s["edit"] == true && s["text"].as_str().unwrap().starts_with("Task #19 doing")));
    push(&st, text(3, OWNER, None, OWNER, "/new"));
    wait("new", 15, || sent_has(&st, "New conversation started"));
    push(&st, call(4, OWNER, None, OWNER, "task_status", json!({"task": "https://github.com/taskqorg/pilotq/issues/19"})));
    wait("status by URL", 15, || sent_has(&st, "re: ok: Task #19 doing: a worker is on it"));
    push(&st, call(5, OWNER, None, OWNER, "task_status", json!({"task": "https://github.com/taskqorg/other/issues/19"})));
    wait("other board refused", 15, || sent_has(&st, "re: refused: https://github.com/taskqorg/other/issues/19 is not a task on taskqorg/pilotq"));
    push(&st, call(6, 8, None, 8, "task_park", json!({"task": "19", "reason": "x"})));
    wait("ungranted park refused", 15, || sent_has(&st, "re: refused: 8 has no project:pilotq grant; nothing was done"));
    push(&st, call(7, OWNER, None, OWNER, "queue_pause", json!({})));
    wait("pause", 15, || sent_has(&st, "re: ok: TaskQ admission is paused on this host; no new task starts. Running workers continue. Still running: #19."));
    assert_eq!(board(&d)["tasks"]["19"]["state"], "doing", "a queue pause does not stop the worker");
    // L12: the status message was deleted, so the next update goes out as a new message in the same route
    st.lock().unwrap().edit_gone = true;
    push(&st, call(8, OWNER, None, OWNER, "task_park", json!({"task": "#19", "reason": "owner asked"})));
    wait("park", 15, || sent_has(&st, "re: ok: Task #19 is parked: TaskQ state later."));
    wait("status parked", 15, || sent(&st).iter().any(|s| s.get("edit").is_none() && s["chat_id"] == OWNER && s["text"].as_str().unwrap().starts_with("Task #19 parked")));
    let b = board(&d);
    assert_eq!((b["creates"].as_i64(), b["paused"].as_bool()), (Some(1), Some(true)), "status reads never create");
    assert_eq!(tq_calls(&d).iter().map(|c| c[0].as_str()).collect::<Vec<_>>(), ["add", "start", "service", "later"]);
    stop(c, d);
}

// F07, L16: without create_claim_issue creation is disabled with that reason and plain add never runs; the conversation goes
// on. With the claim but the board down, the authorized create stays unsent with its reason, and the periodic scan files and
// starts it once the board is back, telling the route.
#[test]
fn f07_board_unavailable_or_no_create_claim() {
    let (d, st) = tq_setup("f07", false);
    let c = spawn(&d, None);
    push(&st, call(1, OWNER, None, OWNER, "task_plan", plan(&["Docs"])));
    wait("plan", 15, || sent_has(&st, "Plan revision 1 saved"));
    push(&st, call(2, OWNER, None, OWNER, "task_run_plan", json!({"revision": 1})));
    wait("disabled", 15, || sent_has(&st, "re: refused: task creation is disabled: create_claim_issue is not configured"));
    push(&st, text(3, OWNER, None, OWNER, "hello"));
    wait("conversation continues", 15, || sent_has(&st, "re: hello"));
    assert!(tq_calls(&d).is_empty(), "no plain add fallback");

    root(&d, true);
    fs::write(d.join("board-down"), "").unwrap();
    push(&st, call(4, OWNER, None, OWNER, "task_run_plan", json!({"revision": 1})));
    wait("unsent", 15, || sent_has(&st, "ticket 1 is unsent (taskq add failed: Error: board unreachable: connection refused)"));
    wait("taskq health", 15, || try_query(&d, "SELECT count(*) FROM plan_runs WHERE state='creating' AND reason LIKE '%board unreachable%'") == Some(1));
    fs::remove_file(d.join("board-down")).unwrap();
    wait("scan files it", 15, || sent_has(&st, "Plan revision 1: filed held, read back and started: #19"));
    assert_eq!(board(&d)["creates"], 1);
    stop(c, d);
}

/// Runs `taskq` against the real board from `root`, without any attempt identity.
fn real_taskq(root: &Path, args: &[&str]) -> String {
    let o = Command::new(std::env::var("PILOTQ_A8_TASKQ").unwrap()).args(args).env("TASKQ_ROOT", root).env_remove("TASKQ_ATTEMPT").env_remove("TASKQ_TASK").env_remove("TASKQ_ROLE").current_dir(root).output().unwrap();
    String::from_utf8_lossy(&o.stdout).to_string()
}

// A8 (owner, 2026-10-23): the real PilotQ binary, real Codex under PilotQ's subscription login, the real taskqorg/pilotq board
// and TaskQ CLI. Telegram is the local fake Bot API with two synthetic users. One small real docs task goes from discussion to
// the reviewer outcome with a PilotQ restart while it is `doing`; the journal, audit record, board and every recorded message
// are checked, U23 links included, and the U02/U18 two-user cases are rerun. It files and publishes a real task, so it runs
// only on request:
//   PILOTQ_A8_ROOT=<pilotq checkout with create_claim_issue> PILOTQ_A8_TASKQ=<taskq> PILOTQ_A8_CODEX_HOME=<logged-in Codex home> \
//   cargo test --locked --test telegram a8_live -- --ignored --nocapture
#[test]
#[ignore = "files and publishes a real task on taskqorg/pilotq"]
fn a8_live_board_run() {
    let root = PathBuf::from(std::env::var("PILOTQ_A8_ROOT").expect("PILOTQ_A8_ROOT"));
    let (d, st) = setup("a8", vec![]);
    // PilotQ's own Codex home: only the subscription login is copied, so no personal config loads
    fs::create_dir_all(d.join("codex")).unwrap();
    fs::copy(PathBuf::from(std::env::var("PILOTQ_A8_CODEX_HOME").expect("PILOTQ_A8_CODEX_HOME")).join("auth.json"), d.join("codex/auth.json")).unwrap();
    fs::set_permissions(d.join("codex/auth.json"), fs::Permissions::from_mode(0o600)).unwrap();
    limits(&d, Path::new("codex"), json!({"first_event": 60, "silent": 60, "deadline": 240}));
    let cfg = d.join("config.json");
    let mut c: Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    c["taskq"] = json!({"bin": std::env::var("PILOTQ_A8_TASKQ").unwrap(), "root": root, "poll": 15});
    fs::write(&cfg, c.to_string()).unwrap();
    let say = |m: &str| eprintln!("a8 {}: {m}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    let model = |id: i64| query(&d, &format!("SELECT count(*) FROM requests WHERE id={id} AND state='answered'")) == 1;

    // U18: an ungranted person is ignored silently and not replayed after the grant
    let mut c = spawn(&d, None);
    push(&st, text(1, 8, None, 8, "hello before any grant"));
    wait("stranger ignored", 30, || try_query(&d, "SELECT count(*) FROM updates WHERE update_id=1 AND disposition='ignored'") == Some(1));
    assert!(cli(&d, &["users", "grant", "8", "chat"]).status.success());
    // U02: a DM per person, then two people in one shared forum topic
    assert!(cli(&d, &["routes", "set", "-100:5", "shared"]).status.success());
    push(&st, text(2, 8, None, 8, "Reply with one short sentence: what is PilotQ?"));
    push(&st, text(3, -100, Some(5), 8, "Reply with one word: hello."));
    push(&st, text(4, -100, Some(5), OWNER, "Reply with one word: thanks."));
    wait("three answers", 300, || [1, 2, 3].iter().all(|id| try_query(&d, &format!("SELECT count(*) FROM requests WHERE id={id} AND state='answered'")) == Some(1)));
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE update_id=1"), 0, "the ignored update is not replayed");
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE session='-100:5' AND sender IN (7, 8)"), 2, "one shared topic conversation keeps both authors");
    assert_eq!(query(&d, "SELECT count(*) FROM requests WHERE id=1 AND session='dm:8'"), 1);
    say("U02/U18 answered");

    // U05: discussion files nothing
    push(&st, text(5, OWNER, None, OWNER, "Plan one pilotq docs task with task_plan and show me the saved revision; do not run it. The task: in docs/projects.md, add one sentence saying that PilotQ files its own tasks on the pilotq board."));
    wait("plan answered", 300, || try_query(&d, "SELECT count(*) FROM requests WHERE id=4 AND state IN ('answered','failed','uncertain')") == Some(1));
    assert!(model(4), "plan request: {:?}", texts(&st));
    assert_eq!((query(&d, "SELECT count(*) FROM plans WHERE revision=1"), query(&d, "SELECT count(*) FROM plan_runs")), (1, 0), "a plan revision is saved and nothing is filed");
    say("plan revision 1 saved");

    // U06: an authorized revision is filed held, read back and started
    push(&st, text(6, OWNER, None, OWNER, "Run plan revision 1 now with task_run_plan."));
    wait("run answered", 300, || try_query(&d, "SELECT count(*) FROM requests WHERE id=5 AND state IN ('answered','failed','uncertain')") == Some(1));
    assert_eq!(query(&d, "SELECT count(*) FROM plan_runs WHERE state='started'"), 1, "plan run: {:?}", texts(&st));
    let n = query(&d, "SELECT number FROM tasks");
    let url = format!("https://github.com/taskqorg/pilotq/issues/{n}");
    let run_reply = query_text(&d, "SELECT reply FROM requests WHERE id=5");
    assert!(run_reply.contains(&format!("#{n}")), "the reply names the task: {run_reply}");
    let issue = real_taskq(&root, &["inbox", &n.to_string()]);
    assert!(issue.contains("Filed by PilotQ pilotq.request.5 for Telegram user 7 in DM 7, plan revision 1."), "{issue}");
    assert!(issue.contains("taskq: hold") && issue.contains("taskq: start"), "held, then started: {issue}");
    say(&format!("filed and started #{n}"));

    let status = |s: &str| format!("Task #{n} {s}.\n{url}");
    let doing = status("doing: a worker is on it");
    let exact = |t: &str| sent(&st).iter().any(|s| s["text"] == t);
    wait("doing", 1800, || exact(&doing));
    // a PilotQ restart while the worker runs: TaskQ keeps the task; the scan keeps reporting it
    c.kill().unwrap();
    c.wait().unwrap();
    say("PilotQ killed while doing; restarting");
    c = spawn(&d, None);
    let closed = format!("Task #{n} is closed on the board; its verdict and evidence are on the issue: {url}");
    let end = Instant::now() + Duration::from_secs(7200);
    while !exact(&closed) {
        assert!(Instant::now() < end, "timed out waiting for the reviewer outcome: {:?}", texts(&st));
        // a worker question is answered by the synthetic owner with its recommended option, by reply
        let q = sent(&st).into_iter().filter(|s| s["text"].as_str().unwrap().starts_with(&format!("Task #{n} asks:"))).last();
        if let Some(q) = q.filter(|q| try_query(&d, &format!("SELECT count(*) FROM requests WHERE reply_to={}", q["id"])) == Some(0)) {
            let k = q["text"].as_str().unwrap().lines().find(|l| l.ends_with('★')).and_then(|l| l.split('.').next()).unwrap_or("1").to_string();
            say(&format!("answering the worker question with {k}"));
            let id = 100 + query(&d, "SELECT count(*) FROM requests");
            push(&st, reply(id, OWNER, None, OWNER, q["id"].as_i64().unwrap(), &k));
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    say("reviewer outcome reached");
    std::thread::sleep(Duration::from_secs(20));

    // recorded messages: one status message edited in place, the outcome its own message, nothing after it, all in the DM
    let all = sent(&st);
    let task_msgs: Vec<&Value> = all.iter().filter(|s| s["text"].as_str().unwrap().starts_with(&format!("Task #{n} "))).collect();
    let news: Vec<&&Value> = task_msgs.iter().filter(|s| s.get("edit").is_none() && !s["text"].as_str().unwrap().contains(" asks:")).collect();
    assert_eq!(news.len(), 2, "one status message and one outcome: {task_msgs:?}");
    assert!(news[0]["text"].as_str().unwrap().starts_with(&format!("Task #{n} ")) && news[1]["text"] == closed.as_str());
    let edits = task_msgs.iter().filter(|s| s["edit"] == true);
    assert!(edits.clone().all(|s| s["message_id"] == news[0]["id"]), "every update edits the status message");
    assert!(edits.clone().any(|s| s["text"] == doing.as_str()) && edits.clone().any(|s| s["text"] == status("review: the worker handed in a result; a reviewer checks it").as_str()));
    assert!(task_msgs.iter().all(|s| s["chat_id"] == OWNER), "task messages stay in the originating route");
    assert!(!all.iter().any(|s| s["edit"] == true && s["message_id"] == news[1]["id"]), "nothing edits the outcome");
    assert!(!all.iter().any(|s| { let t = s["text"].as_str().unwrap(); t.contains("stall") || t.contains("hung") }));
    for (id, chat, topic) in [(1, 8, None), (2, -100, Some(5)), (3, -100, Some(5))] {
        let r = query_text(&d, &format!("SELECT reply FROM requests WHERE id={id}"));
        assert!(all.iter().any(|s| s["text"] == r.as_str() && s["chat_id"] == chat && s.get("message_thread_id").and_then(Value::as_i64) == topic), "reply {id} in its route");
    }
    let acks = all.iter().filter(|s| s["text"].as_str().unwrap().starts_with("Received. Request #")).count() as i64;
    assert_eq!(acks, query(&d, "SELECT count(*) FROM requests"), "one acknowledgement per request");

    // U23: the audit record joins the request to its task and trace; the task text carries the trace ID
    let a: Value = serde_json::from_slice(&cli(&d, &["audit", "--request", "5", "--json"]).stdout).unwrap();
    assert_eq!(a[0]["tasks"].as_array().unwrap().iter().map(|t| t["target"].as_str().unwrap().to_string()).collect::<Vec<_>>(), [format!("add #{n}"), format!("start #{n}")]);
    assert_eq!((a[0]["trace"].as_str(), a[0]["who"]["id"].as_i64(), a[0]["outcome"].as_str()), (Some("pilotq.request.5"), Some(OWNER), Some("answered")));
    c.kill().unwrap();
    c.wait().unwrap();
    let tel = coreq::telemetry::Reader::open(&d.join("telemetry.sqlite")).unwrap().export(0, 0, 1000).unwrap().to_string();
    assert!(tel.contains("\"pilotq.request.5\""), "the request's CoreQ trace exists");
    let state = Command::new("gh").args(["api", &format!("repos/taskqorg/pilotq/issues/{n}"), "--jq", ".state + \" \" + .state_reason"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&state.stdout).trim(), "closed completed", "the board shows the task completed");
    say(&format!("A8 passed: task #{n}"));
    fs::remove_dir_all(d).unwrap();
}

fn query_text(d: &Path, sql: &str) -> String {
    let c = rusqlite::Connection::open_with_flags(d.join("journal.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    c.query_row(sql, [], |r| r.get(0)).unwrap()
}
