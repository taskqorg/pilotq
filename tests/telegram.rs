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
