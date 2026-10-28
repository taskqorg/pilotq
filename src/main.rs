//! PilotQ service and operator CLI on coreq (pilotq.md § 9).

mod direct;
mod journal;
mod keys;
mod manager;
mod taskq;
mod telegram;

use coreq::Tool;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage: pilotq service install|run|stop|status | status [--json] | update [--check|--pause|--resume] | selftest | doctor [--json] | version | ask TEXT [--json]
  users list | users show ID | users grant ID GRANT.. | users revoke ID [GRANT..] | users remember ID KEY VALUE [--private] | users forget ID KEY
  routes list | routes set CHAT[:TOPIC] shared|isolated | projects list | projects add NAME ROOT
  keys list | keys add NAME PURPOSE TOOL.. (value on stdin) | keys grant NAME ID|route:CHAT[:TOPIC]|project:NAME.. | keys revoke NAME [WHO..]
  (users, routes, projects and keys accept --json)
  audit [--user ID] [--route cli|CHAT[:TOPIC]] [--project P] [--since UNIX_SECONDS] [--request ID] [--json]
  pilot start | pilot note intervention|outage TEXT | pilot report [--since UNIX_SECONDS] [--until UNIX_SECONDS] (pilot accepts --json)";
/// A status.json older than this reads unknown (coreq.md § status.json).
const FRESH: u64 = 60;
const PASS: Duration = Duration::from_secs(30);
/// Section 7 health sources, each reported separately.
const SOURCES: [&str; 5] = ["poll", "storage", "model", "taskq", "delivery"];
/// While this file exists in the install directory the service starts no new turn (§ 12 planned restart).
const DRAIN: &str = "drain";
/// Planned-restart grace: the 60 s direct-action budget plus the 5 s cancellation grace (§ 7).
const DRAIN_GRACE: Duration = Duration::from_secs(65);

/// Stops new turns and waits within the grace period; unfinished requests refuse activation.
fn drain(t: &Tool, grace: Duration) -> Result<Option<String>, String> {
    let path = t.install.join("journal.sqlite");
    if !path.exists() {
        return Ok(None);
    }
    std::fs::write(t.install.join(DRAIN), "").map_err(|e| format!("drain: {e}"))?;
    let j = journal::Journal::open(&path)?;
    j.draining(true)?;
    let end = std::time::Instant::now() + grace;
    loop {
        let n = j.running()?;
        if n == 0 {
            return Ok(Some("drain: ok".into()));
        }
        if std::time::Instant::now() >= end {
            return Err(format!("drain: {n} running request(s); update deferred"));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn resume_after_update(t: &Tool) -> Result<(), String> {
    let path = t.install.join("journal.sqlite");
    if path.exists() { journal::Journal::open(&path)?.draining(false)?; }
    match std::fs::remove_file(t.install.join(DRAIN)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("resume: {e}")),
        _ => Ok(()),
    }
}

/// §9 / R14: one update path, serialized across the CLI and service maintenance thread.
fn update(t: &Tool, check: bool) -> Result<(Option<String>, Option<String>), String> {
    std::fs::create_dir_all(&t.install).map_err(|e| e.to_string())?;
    let _lock = coreq::file_lock::try_lock(&t.install.join("update.lock")).map_err(|e| format!("update lock: {e}"))?;
    if check { return coreq::update::update(t, true).map(|v| (v, None)); }
    // Probe before draining: an unavailable or unchanged release must not interrupt conversation admission.
    if coreq::update::update(t, true)?.is_none() { return Ok((None, None)); }
    let result = (|| {
        let drained = drain(t, DRAIN_GRACE)?;
        coreq::update::update(t, false).map(|v| (v, drained))
    })();
    if !matches!(&result, Ok((Some(_), _))) {
        resume_after_update(t)?;
    }
    result
}

const UPDATE_INTERVAL: u64 = 300;

/// Persist before dispatch; a crash or unavailable release cannot create a startup retry storm.
fn scheduled_update(t: &Tool, at: u64) -> Result<Option<String>, String> {
    if t.install.join("update-paused").try_exists().map_err(|e| e.to_string())? { return Ok(None); }
    let path = t.install.join("update-at");
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            let previous = s.trim().parse::<u64>().map_err(|_| "update-at: invalid timestamp")?;
            if at.saturating_sub(previous) < UPDATE_INTERVAL { return Ok(None); }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("update-at: {e}")),
    }
    coreq::write_atomic(&path, &at.to_string())?;
    update(t, false).map(|(v, _)| v)
}

fn tool() -> Tool {
    Tool::new("pilotq", "PilotQ", "taskqorg/pilotq", env!("CARGO_PKG_VERSION"))
}

/// Protected bot settings in `<install>/config.json`, owner-only (§ 2: the owner ID is never taken from a message).
struct Config {
    token: String,
    owner: i64,
    api_url: Option<url::Url>,
    codex: manager::Codex,
    taskq: Option<taskq::Taskq>,
    direct: direct::Direct,
}

/// The key store and the GitHub CLI the direct tools use (§ 5, § 8).
fn direct(t: &Tool, gh: Option<&str>) -> direct::Direct {
    direct::Direct { gh: gh.unwrap_or("gh").into(), keys: keys::Keys(t.install.join("keys.json")) }
}

/// None when no bot is configured yet; an unsafe or malformed file is an error, never ignored.
fn config(t: &Tool) -> Result<Option<Config>, String> {
    let p = t.install.join("config.json");
    let Ok(text) = std::fs::read_to_string(&p) else { return Ok(None) };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&p).map_err(|e| e.to_string())?.permissions().mode();
        if mode & 0o077 != 0 {
            return Err(format!("{} must be owner-only (chmod 600)", p.display()));
        }
    }
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?;
    let token = v["bot_token"].as_str().filter(|s| !s.is_empty()).ok_or("config.json: bot_token missing")?.to_string();
    let owner = v["owner_id"].as_i64().ok_or("config.json: owner_id missing")?;
    let api_url = v["api_url"].as_str().map(url::Url::parse).transpose().map_err(|e| format!("config.json: api_url: {e}"))?;
    let secs = |k: &str, d: u64| v["codex_limits"][k].as_u64().map_or(Ok(Duration::from_secs(d)), |n| if n > 0 { Ok(Duration::from_secs(n)) } else { Err(format!("config.json: codex_limits.{k} must be positive")) });
    let limits = manager::Limits { first_event: secs("first_event", 30)?, silent: secs("silent", 30)?, deadline: secs("deadline", 60)? };
    let codex = manager::Codex { bin: v["codex"].as_str().unwrap_or("codex").to_string(), workspace: t.install.join("workspace"), home: t.install.join("codex"), limits };
    // § 5: the TaskQ CLI and the project root whose taskq.json names the board; absent, the manager has no TaskQ tools
    let tq = &v["taskq"];
    let taskq = match (tq["bin"].as_str(), tq["root"].as_str()) {
        (Some(bin), Some(root)) => Some(taskq::Taskq { bin: bin.into(), root: root.into(), poll: Duration::from_secs(tq["poll"].as_u64().filter(|n| *n > 0).unwrap_or(15)) }),
        (None, None) => None,
        _ => return Err("config.json: taskq needs bin and root".into()),
    };
    Ok(Some(Config { token, owner, api_url, codex, taskq, direct: direct(t, v["gh"].as_str()) }))
}

fn initial_project(j: &journal::Journal, c: &Config) -> Result<(), String> {
    if let Some(tq) = &c.taskq {
        let v: Value = std::fs::read_to_string(tq.root.join("taskq.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
        if v["repo"] == taskq::BOARD {
            if !j.projects()?.iter().any(|p| p.0 == "pilotq") {
                j.add_project(c.owner, "pilotq", taskq::BOARD, &tq.root.to_string_lossy(), now() as i64)?;
            }
            j.pin_legacy_taskq(&tq.root.to_string_lossy())?;
        }
    }
    Ok(())
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// One service pass's status.json. Nothing is observed yet, so every health source and count reads unknown, never ok or zero.
fn status_doc(t: &Tool, at: u64) -> Value {
    let host = std::process::Command::new("hostname").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let mut s = coreq::status::common(t, &host, "", at, None, None);
    let unknown = json!({"at": null, "age": null, "state": "unknown", "reason": "not-observed"});
    s["health"] = SOURCES.iter().map(|k| (k.to_string(), unknown.clone())).collect();
    for k in ["pending", "undelivered", "uncertain"] {
        s[k] = json!({"count": null, "reason": "not-implemented"});
    }
    s
}

/// Fresh status of the running service, else unknown with a reason.
fn status(t: &Tool) -> Value {
    coreq::status::read(t, &t.install, FRESH).unwrap_or_else(|| json!({"state": "unknown", "reason": "no status.json within 60 s; service not running"}))
}

fn selftest(t: &Tool) -> Result<(), String> {
    let s = status_doc(t, 1);
    let unknown = SOURCES.iter().all(|k| s["health"][k]["state"] == "unknown");
    if s["version"] != t.version || !unknown {
        return Err("fixture: status.json must name the version and report unobserved health as unknown".into());
    }
    println!("fixture: ok");
    // ponytail: no config or board read yet; the online part arrives with the journal and TaskQ in later slices.
    println!("pilotq {} selftest: ok", t.version);
    Ok(())
}

/// §9 setup checks; dependency diagnostics are deliberately withheld (§8).
fn doctor(t: &Tool) -> Value {
    use teloxide::prelude::*;
    use std::process::{Command, Stdio};
    let mut checks = json!({"configuration": "failed", "telegram": "unknown", "codex": "unknown", "taskq": "unknown"});
    let Ok(Some(c)) = config(t) else { return json!({"ok": false, "checks": checks}); };
    checks["configuration"] = json!("ok");
    let telegram = tokio::runtime::Builder::new_current_thread().enable_all().build().ok().is_some_and(|rt| rt.block_on(async {
        let mut bot = Bot::new(&c.token);
        if let Some(url) = &c.api_url { bot = bot.set_api_url(url.clone()); }
        matches!(tokio::time::timeout(Duration::from_secs(20), bot.get_me().send()).await, Ok(Ok(me)) if me.is_bot)
    }));
    checks["telegram"] = json!(if telegram { "ok" } else { "failed" });
    let mut cmd = Command::new(&c.codex.bin);
    cmd.env_clear();
    for k in ["PATH", "HOME", "USER", "LANG", "LC_ALL", "TMPDIR", "SSL_CERT_FILE"] {
        if let Some(v) = std::env::var_os(k) { cmd.env(k, v); }
    }
    let login = cmd.env("CODEX_HOME", &c.codex.home)
        .args(["-c", "forced_login_method=\"chatgpt\"", "login", "status"])
        .stdin(Stdio::null()).output().ok().is_some_and(|o| {
            o.status.success() && (String::from_utf8_lossy(&o.stdout).contains("Logged in using ChatGPT")
                || String::from_utf8_lossy(&o.stderr).contains("Logged in using ChatGPT"))
        });
    checks["codex"] = json!(if login { "ok" } else { "failed" });
    let board = c.taskq.as_ref().is_some_and(|tq| tq.board().is_ok());
    checks["taskq"] = json!(if board { "ok" } else { "failed" });
    json!({"ok": telegram && login && board, "checks": checks})
}

fn service_run(t: &Tool) -> ! {
    let started = std::time::Instant::now();
    let release = t.release_exe(t.version);
    let installed = std::env::current_exe().and_then(std::fs::canonicalize).ok().is_some_and(|e| Some(e) == release.canonicalize().ok());
    // readiness order (§ 12): config and storage, then the poll lock, then polling
    let cfg = config(t).unwrap_or_else(|e| t.die(&e));
    std::fs::create_dir_all(&t.install).unwrap_or_else(|e| t.die(&e.to_string()));
    let mut journal = journal::Journal::open(&t.install.join("journal.sqlite")).unwrap_or_else(|e| t.die(&e));
    if let Some(c) = &cfg {
        journal.bootstrap(c.owner, now() as i64).unwrap_or_else(|e| t.die(&e));
        initial_project(&journal, c).unwrap_or_else(|e| t.die(&e));
    }
    // § 12: reconcile runs a dead process left behind before anything new starts
    journal.reconcile(now() as i64).unwrap_or_else(|e| t.die(&e));
    // F04: an effect written without a receipt is read back, never repeated
    if let Some(c) = &cfg
        && let Err(e) = direct::reconcile(&c.direct, &journal)
    {
        eprintln!("effects: {e}");
    }
    // a drain left by an update applies to the process it restarted, not to this one
    journal.draining(false).unwrap_or_else(|e| t.die(&e));
    let _ = std::fs::remove_file(t.install.join(DRAIN));
    let j: telegram::Shared = Arc::new(Mutex::new(journal));
    let _lock = cfg.map(|c| {
        // one OS-held lock per installation; it is released only when this process ends
        let lock = coreq::file_lock::try_lock(&t.install.join("poll.lock")).unwrap_or_else(|e| match e {
            std::fs::TryLockError::WouldBlock => t.die("poll lock held: another pilotq service polls this bot"),
            std::fs::TryLockError::Error(e) => t.die(&format!("poll lock: {e}")),
        });
        let mut bot = teloxide::Bot::new(c.token);
        if let Some(u) = c.api_url {
            bot = bot.set_api_url(u);
        }
        let (j, wake) = (j.clone(), Arc::new(tokio::sync::Notify::new()));
        let deadline = c.codex.limits.deadline;
        let tq = c.taskq.map(Arc::new);
        let (mj, mwake, codex, drain, mtq, d) = (j.clone(), wake.clone(), c.codex, t.install.join(DRAIN), tq.clone(), Arc::new(c.direct));
        std::thread::spawn(move || manager::serve(codex, mj, mwake, || now() as i64, drain, mtq, d));
        if let Some(tq) = tq {
            let (wj, wwake) = (j.clone(), wake.clone());
            std::thread::spawn(move || taskq::watch(tq, wj, wwake));
        }
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
            rt.block_on(async {
                tokio::join!(telegram::poll(bot.clone(), j.clone(), wake.clone()), telegram::typing(bot.clone(), j.clone(), deadline), telegram::drafts(bot.clone(), j.clone()), telegram::deliver(bot, j, wake))
            });
        });
        lock
    });
    // Only installed Linux releases participate; source builds and other platforms never self-replace.
    if installed && cfg!(target_os = "linux") {
        let maintenance = tool();
        std::thread::spawn(move || loop {
            match scheduled_update(&maintenance, now()) {
                Ok(Some(v)) => {
                    eprintln!("update: {v}; restarting through the launcher");
                    std::process::exit(75);
                }
                Ok(None) => {}
                Err(e) => eprintln!("scheduled update: {e}"),
            }
            std::thread::sleep(PASS);
        });
    }
    eprintln!("pilotq {} service running", t.version);
    loop {
        let mut s = status_doc(t, now());
        if let Err(e) = j.lock().unwrap().status(&mut s, now() as i64) {
            s["health"]["storage"] = json!({"at": null, "age": null, "state": "unknown", "reason": e});
        }
        let r = coreq::status::write(t, &t.install, &s);
        if let Err(e) = &r {
            eprintln!("status: {e}");
        }
        // the launcher rolls back a release that never gets here (coreq.md § Launcher)
        if installed && r.is_ok() && started.elapsed() >= Duration::from_secs(60) {
            let _ = std::fs::write(release.with_file_name("healthy"), "");
        }
        std::thread::sleep(PASS);
    }
}

/// `pilotq ask` (§ 9, R13): one request through the Telegram path's grant check, request record and manager run, as the
/// configured owner. Prints the reply; a request that does not end answered exits nonzero.
fn ask(t: &Tool, text: &str) -> Result<Value, (Value, String)> {
    let plain = |e: String| (Value::Null, e);
    let cfg = config(t).map_err(plain)?.ok_or_else(|| plain("no config.json: the CLI acts as its configured owner_id".into()))?;
    std::fs::create_dir_all(&t.install).map_err(|e| plain(e.to_string()))?;
    let mut j = journal::Journal::open(&t.install.join("journal.sqlite")).map_err(plain)?;
    j.bootstrap(cfg.owner, now() as i64).map_err(plain)?;
    initial_project(&j, &cfg).map_err(plain)?;
    let r = j.ask(cfg.owner, text, now() as i64).map_err(plain)?;
    let j = Mutex::new(j);
    manager::run(&cfg.codex, &j, &r, || now() as i64, cfg.taskq.as_ref(), &cfg.direct).map_err(plain)?;
    let (state, reply, error) = j.lock().unwrap().request(r.request).map_err(plain)?;
    let v = json!({"request": r.request, "state": state, "reply": reply, "error": error});
    if state == "answered" { Ok(v) } else { Err((v, format!("request #{} {state}: {}", r.request, error.unwrap_or_default()))) }
}

/// `pilotq audit` (§ 9, R17) as the configured owner, who holds the `audit` grant; the read is recorded.
fn audit(t: &Tool, a: &[&str]) -> Result<Value, String> {
    let mut f = journal::Filter::default();
    let n = |k: &str, v: &str| v.parse::<i64>().map_err(|_| format!("{k} {v}: expected a number"));
    for pair in a.chunks(2) {
        match pair {
            ["--user", v] => f.user = Some(n("--user", v)?),
            ["--route", v] => f.route = Some(v.to_string()),
            ["--project", v] => f.project = Some(v.to_string()),
            ["--since", v] => f.since = Some(n("--since", v)?),
            ["--request", v] => f.request = Some(n("--request", v)?),
            _ => return Err(USAGE.into()),
        }
    }
    let cfg = config(t)?.ok_or("no config.json: the CLI acts as its configured owner_id")?;
    std::fs::create_dir_all(&t.install).map_err(|e| e.to_string())?;
    let mut j = journal::Journal::open(&t.install.join("journal.sqlite"))?;
    j.bootstrap(cfg.owner, now() as i64)?;
    initial_project(&j, &cfg)?;
    Ok(j.audit(cfg.owner, &f, now() as i64)?.into())
}

fn pilot_report(t: &Tool, a: &[&str]) -> Result<Value, String> {
    let (mut since, mut until) = (None, now() as i64 + 1);
    for pair in a.chunks(2) {
        match pair {
            ["--since", v] => since = Some(v.parse::<i64>().map_err(|_| "--since: expected Unix seconds")?),
            ["--until", v] => until = v.parse::<i64>().map_err(|_| "--until: expected Unix seconds")?,
            _ => return Err(USAGE.into()),
        }
    }
    let path = t.install.join("journal.sqlite");
    if !path.exists() { return Err("no journal: pilot measurements unavailable".into()); }
    let j = journal::Journal::open(&path)?;
    let mut report = j.pilot_report(since)?;
    let since = report["since"].as_i64().ok_or("pilot report has no start")?;
    if since < 0 || until <= since { return Err("expected 0 <= since < until".into()); }
    report["tools"] = j.tool_size_report(since, until)?["tools"].clone();
    report["until"] = until.into();
    Ok(report)
}

fn fact(f: &journal::Fact) -> Value {
    json!({"key": f.key, "value": f.value, "private": f.private, "source": f.source, "revision": f.revision})
}

/// `users`, `routes` and `projects` (§ 5, § 9, R10). The operator acts as the bootstrap owner from protected config; writes return the readback.
fn admin(t: &Tool, a: &[&str]) -> Result<Value, String> {
    let cfg = config(t)?.ok_or("no config.json: the CLI acts as its configured owner_id")?;
    std::fs::create_dir_all(&t.install).map_err(|e| e.to_string())?;
    let mut j = journal::Journal::open(&t.install.join("journal.sqlite"))?;
    let (me, at) = (cfg.owner, now() as i64);
    j.bootstrap(me, at)?;
    initial_project(&j, &cfg)?;
    let id = |s: &str| s.parse::<i64>().map_err(|_| format!("{s}: expected an exact Telegram user ID"));
    let user = |j: &journal::Journal, u: i64| -> Result<Value, String> {
        let log: Vec<Value> = j.grant_log(u)?.into_iter().map(|(at, by, before, after)| json!({"at": at, "by": by, "before": before, "after": after})).collect();
        Ok(json!({"id": u, "grants": j.grants(u)?, "facts": j.facts(u, me)?.iter().map(fact).collect::<Vec<_>>(), "log": log}))
    };
    let owned = |g: &[&str]| g.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    Ok(match a {
        ["users", "list"] => j.users()?.into_iter().map(|(u, g)| json!({"id": u, "grants": g})).collect(),
        ["users", "show", u] => user(&j, id(u)?)?,
        ["users", "grant", u, g @ ..] if !g.is_empty() => {
            j.grant(me, id(u)?, &owned(g), at)?;
            user(&j, id(u)?)?
        }
        ["users", "revoke", u, g @ ..] => {
            j.revoke(me, id(u)?, &owned(g), at)?;
            user(&j, id(u)?)?
        }
        ["users", "remember", u, key, value, flag @ ..] if flag.is_empty() || flag == ["--private"] => {
            j.remember(me, id(u)?, key, value, !flag.is_empty(), at)?;
            user(&j, id(u)?)?
        }
        ["users", "forget", u, key] => {
            if !j.forget(me, id(u)?, key)? {
                return Err(format!("{u} has no fact {key}"));
            }
            user(&j, id(u)?)?
        }
        ["projects", "list"] => j.projects()?.into_iter().map(|(name, repo, root, by, at)| json!({"name": name, "repo": repo, "root": root, "by": by, "at": at})).collect(),
        ["projects", "add", name, root] => {
            // § 5 Project registration: the root's taskq.json names the repository; a fresh TaskQ read must show the same board
            let tq = cfg.taskq.as_ref().ok_or("projects add needs taskq.bin in config.json to read the board back")?;
            let root = std::fs::canonicalize(root).map_err(|e| format!("{root}: {e}"))?;
            let tj: Value = std::fs::read_to_string(root.join("taskq.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).ok_or(format!("{}: no readable taskq.json", root.display()))?;
            let repo = tj["repo"].as_str().ok_or("taskq.json names no repo")?;
            let board = taskq::Taskq { bin: tq.bin.clone(), root: root.clone(), poll: tq.poll }.board()?;
            if !board.eq_ignore_ascii_case(repo) {
                return Err(format!("taskq reads board {board} under {}, not {repo}; nothing was registered", root.display()));
            }
            j.add_project(me, name, repo, &root.to_string_lossy(), at)?;
            let p = j.projects()?.into_iter().find(|p| p.0 == *name).ok_or("registration did not read back")?;
            json!({"name": p.0, "repo": p.1, "root": p.2, "by": p.3, "at": p.4})
        }
        ["routes", "list"] => j.routes()?.iter().map(|r| json!({"chat": r.chat, "topic": r.topic, "session": if r.shared { "shared" } else { "isolated" }, "by": r.by, "at": r.at})).collect(),
        ["routes", "set", route, policy @ ("shared" | "isolated")] => {
            let (chat, topic) = route.split_once(':').unwrap_or((route, "0"));
            let n = |s: &str| s.parse::<i64>().map_err(|_| format!("{route}: expected CHAT or CHAT:TOPIC"));
            let r = j.set_route(me, n(chat)?, n(topic)?, *policy == "shared", at)?;
            json!({"chat": r.chat, "topic": r.topic, "session": policy, "by": r.by, "at": r.at})
        }
        _ => return Err(USAGE.into()),
    })
}

/// `pilot` (section 10 step 5): the pilot start, intervention and outage notes as the configured owner, and the gate 4 report.
fn pilot(t: &Tool, a: &[&str]) -> Result<Value, String> {
    let cfg = config(t)?.ok_or("no config.json: the CLI acts as its configured owner_id")?;
    let j = journal::Journal::open(&t.install.join("journal.sqlite"))?;
    let at = now() as i64;
    match a {
        ["pilot", "start"] => Ok(json!({"pilot_start": j.pilot_start(at)?})),
        ["pilot", "note", kind, text @ ..] if !text.is_empty() => {
            j.pilot_note(cfg.owner, kind, &text.join(" "), at)?;
            j.pilot_report(None)
        }
        ["pilot", "report"] => j.pilot_report(None),
        ["pilot", "report", "--since", s] => j.pilot_report(Some(s.parse().map_err(|_| format!("--since {s}: expected a number"))?)),
        _ => Err(USAGE.into()),
    }
}

/// `keys` (§ 8 Key store, § 9). Keys enter only here; `add` reads the value from stdin and no command prints a value.
fn keys(t: &Tool, a: &[&str]) -> Result<Value, String> {
    let k = keys::Keys(t.install.join("keys.json"));
    std::fs::create_dir_all(&t.install).map_err(|e| e.to_string())?;
    let owned = |g: &[&str]| g.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    match a {
        ["keys", "list"] => k.list(),
        ["keys", "add", name, purpose, tools @ ..] => {
            let mut value = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut value).map_err(|e| format!("stdin: {e}"))?;
            k.add(name, purpose, &owned(tools), value.trim_end_matches(['\n', '\r']))
        }
        ["keys", "grant", name, who @ ..] if !who.is_empty() => k.grant(name, &owned(who)),
        ["keys", "revoke", name, who @ ..] => k.revoke(name, &owned(who)),
        _ => Err(USAGE.into()),
    }
}

/// Human form of an `admin` or `keys` readback: one line per person, route, project or key, facts and grant history indented.
fn show(v: &Value) {
    let one = |v: &Value| {
        if v.get("repo").is_some() {
            return println!("{} {} {} (registered by {})", v["name"].as_str().unwrap_or(""), v["repo"].as_str().unwrap_or(""), v["root"].as_str().unwrap_or(""), v["by"]);
        }
        if v.get("purpose").is_some() || v.get("removed").is_some() {
            let list = |k: &str| v[k].as_array().map_or(String::new(), |a| a.iter().map(|x| x.as_str().map_or(x.to_string(), str::to_string)).collect::<Vec<_>>().join(","));
            return println!("{} {}: tools {} users {} routes {} projects {} last use {}", v["name"].as_str().unwrap_or(""), v["purpose"].as_str().unwrap_or("removed"), list("tools"), list("users"), list("routes"), list("projects"), v["last_use"]);
        }
        if v.get("session").is_some() {
            let topic = if v["topic"] == 0 { String::new() } else { format!(":{}", v["topic"]) };
            return println!("{}{topic} {} (set by {})", v["chat"], v["session"].as_str().unwrap_or(""), v["by"]);
        }
        let g: Vec<&str> = v["grants"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        println!("{} grants: {}", v["id"], if g.is_empty() { "none (ignored)".to_string() } else { g.join(",") });
        for f in v["facts"].as_array().into_iter().flatten() {
            let scope = if f["private"] == true { "private" } else { "shared" };
            println!("  {} = {} ({scope}, revision {}, {})", f["key"].as_str().unwrap_or(""), f["value"].as_str().unwrap_or(""), f["revision"], f["source"].as_str().unwrap_or(""));
        }
        for l in v["log"].as_array().into_iter().flatten() {
            let by = if l["by"].is_null() { "config".to_string() } else { l["by"].to_string() };
            println!("  at {} by {by}: [{}] -> [{}]", l["at"], l["before"].as_str().unwrap_or(""), l["after"].as_str().unwrap_or(""));
        }
    };
    match v.as_array() {
        Some(rows) if rows.is_empty() => println!("none"),
        Some(rows) => rows.iter().for_each(one),
        None => one(v),
    }
}

fn main() {
    let t = tool();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // the installed launcher runs the current release (coreq launcher)
    if std::env::current_exe().ok() == Some(t.install.join(t.exe_name())) && args.first().map(String::as_str) != Some("update") {
        coreq::launcher::launch(&t, &args);
    }
    let unit = || coreq::service::unit(&t, &t.install, "");
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["service", "run"] => service_run(&t),
        ["service", "install"] => {
            let r = coreq::update::install_self(&t).and_then(|_| coreq::service::run_all(&unit().unload, true)).and_then(|_| coreq::service::host_unit(&t));
            let u = r.unwrap_or_else(|e| t.die(&e));
            println!("service install: ok ({})", u.path.display());
        }
        ["service", "stop"] => {
            coreq::service::run_all(&unit().stop, false).unwrap_or_else(|e| t.die(&e));
            println!("service stop: ok");
        }
        ["service", "status"] => {
            let u = unit();
            println!("unit: {} ({})", if u.path.exists() { "installed" } else { "absent" }, u.path.display());
            println!("status: {}", status(&t));
        }
        ["status", rest @ ..] if rest.iter().all(|x| *x == "--json") => {
            let s = status(&t);
            if rest.is_empty() {
                println!("version: {}", s["version"].as_str().unwrap_or("unknown"));
                for k in SOURCES {
                    let h = &s["health"][k];
                    println!("{k}: {} {}", h["state"].as_str().unwrap_or("unknown"), h["reason"].as_str().unwrap_or(s["reason"].as_str().unwrap_or("")));
                }
            } else {
                println!("{s}");
            }
        }
        ["update", rest @ ..] if rest.iter().all(|x| *x == "--check") => {
            let check = !rest.is_empty();
            let u = unit();
            let (version, drained) = update(&t, check).unwrap_or_else(|e| t.die(&e));
            match version {
                Some(v) if check => println!("update: {v} available"),
                Some(v) => {
                    println!("update: {v}");
                    if let Some(d) = drained { println!("{d}"); }
                    if u.path.exists() {
                        coreq::service::run_all(&u.unload, true).and_then(|_| coreq::service::run_all(&u.load, false))
                            .unwrap_or_else(|e| t.die(&format!("restart: {e}")));
                        println!("service restart: ok");
                    }
                    resume_after_update(&t).unwrap_or_else(|e| t.die(&e));
                }
                None => println!("update: up to date"),
            }
        }
        ["update", action @ ("--pause" | "--resume"), rest @ ..] if rest.iter().all(|x| *x == "--json") => {
            std::fs::create_dir_all(&t.install).unwrap_or_else(|e| t.die(&e.to_string()));
            let p = t.install.join("update-paused");
            if *action == "--pause" {
                coreq::write_atomic(&p, "paused").unwrap_or_else(|e| t.die(&e));
            } else if let Err(e) = std::fs::remove_file(&p) {
                if e.kind() != std::io::ErrorKind::NotFound { t.die(&e.to_string()); }
            }
            let paused = p.try_exists().unwrap_or_else(|e| t.die(&e.to_string()));
            if rest.is_empty() { println!("automatic updates: {}", if paused { "paused" } else { "enabled" }); }
            else { println!("{}", json!({"automatic_updates": if paused { "paused" } else { "enabled" }})); }
        }

        [cmd @ ("users" | "routes" | "projects" | "keys"), ..] => {
            let json = a.contains(&"--json");
            let rest: Vec<&str> = a.iter().copied().filter(|x| *x != "--json").collect();
            let v = if *cmd == "keys" { keys(&t, &rest) } else { admin(&t, &rest) }.unwrap_or_else(|e| t.die(&format!("{cmd}: {e}")));
            if json { println!("{v}") } else { show(&v) }
        }
        ["pilot", "report", rest @ ..] => {
            let args: Vec<&str> = rest.iter().copied().filter(|x| *x != "--json").collect();
            let v = pilot_report(&t, &args).unwrap_or_else(|e| t.die(&format!("pilot report: {e}")));
            if rest.contains(&"--json") { println!("{v}"); } else {
                println!("Pilot gate counts since {} (pilot start: {})", v["since"], v["pilot_start"]);
                for key in ["accepted", "simple", "users", "routes", "projects", "tasks", "interventions", "outages"] {
                    println!("{key}: {}", v[key]);
                }
                if v["failed"].is_null() {
                    println!("failed: unknown ({})", v["failed_reason"].as_str().unwrap_or("telemetry unavailable"));
                } else {
                    println!("failed: {}", v["failed"]);
                }
                println!("Tool result bytes [{} .. {})", v["since"], v["until"]);
                for row in v["tools"].as_array().into_iter().flatten() {
                    println!("{}: calls={} p95={} max={}", row["tool"].as_str().unwrap_or(""), row["calls"], row["p95_result_bytes"], row["max_result_bytes"]);
                }
            }
        }
        ["audit", ..] => {
            let json = a.contains(&"--json");
            let rest: Vec<&str> = a[1..].iter().copied().filter(|x| *x != "--json").collect();
            let v = audit(&t, &rest).unwrap_or_else(|e| t.die(&format!("audit: {e}")));
            if json {
                println!("{v}");
            }
            for r in v.as_array().into_iter().flatten().filter(|_| !json) {
                let w = &r["where"];
                let place = if w["front"] == "cli" { "cli".to_string() } else if w["topic"].is_null() { w["chat"].to_string() } else { format!("{}:{}", w["chat"], w["topic"]) };
                let who = r["who"]["name"].as_str().map_or(r["who"]["id"].to_string(), |n| format!("{} ({n})", r["who"]["id"]));
                println!("#{} at {} {place} by {who} {}: {}", r["request"], r["received"], r["outcome"].as_str().unwrap_or(""), r["text"].as_str().unwrap_or(""));
                for x in r["direct"].as_array().into_iter().flatten() {
                    println!("  direct {} {}", x["target"].as_str().unwrap_or(""), x["result"].as_str().unwrap_or(""));
                }
                for x in r["tools"].as_array().into_iter().flatten() {
                    println!("  tool {} {}", x["target"].as_str().unwrap_or(""), x["result"].as_str().unwrap_or("started"));
                }
                if let Some(reply) = r["reply"].as_str() {
                    println!("  reply: {reply}");
                }
            }
        }
        ["pilot", ..] => {
            let json = a.contains(&"--json");
            let rest: Vec<&str> = a.iter().copied().filter(|x| *x != "--json").collect();
            let v = pilot(&t, &rest).unwrap_or_else(|e| t.die(&format!("pilot: {e}")));
            if json {
                println!("{v}");
            } else if let Some(m) = v.as_object() {
                for (k, x) in m {
                    println!("{k}: {}", x.as_str().map_or(x.to_string(), str::to_string));
                }
            }
        }
        ["ask", rest @ ..] => {
            let json = rest.contains(&"--json");
            let text = rest.iter().copied().filter(|x| *x != "--json").collect::<Vec<_>>().join(" ");
            if text.trim().is_empty() {
                t.die(USAGE);
            }
            match ask(&t, &text) {
                Ok(v) if json => println!("{v}"),
                Ok(v) => println!("{}", v["reply"].as_str().unwrap_or("")),
                Err((v, e)) => {
                    if json && !v.is_null() {
                        println!("{v}");
                    }
                    t.die(&format!("ask: {e}"));
                }
            }
        }
        ["doctor", rest @ ..] if rest.iter().all(|a| *a == "--json") => {
            let report = doctor(&t);
            if rest.contains(&"--json") { println!("{report}"); }
            else { for (name, state) in report["checks"].as_object().unwrap() { println!("{name}: {}", state.as_str().unwrap()); } }
            if report["ok"] != true { std::process::exit(1); }
        }
        ["selftest"] => selftest(&t).unwrap_or_else(|e| t.die(&e)),
        ["version"] => println!("pilotq {}", t.version),
        _ => t.die(USAGE),
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;

    // §12: an expired drain may not authorize activation while a request is still running.
    #[test]
    fn update_drain_timeout_preserves_running_work() {
        let mut t = tool();
        t.install = std::env::temp_dir().join(format!("pilotq-update-timeout-{}", std::process::id()));
        std::fs::create_dir_all(&t.install).unwrap();
        let mut j = journal::Journal::open(&t.install.join("journal.sqlite")).unwrap();
        j.bootstrap(7, 0).unwrap();
        let r = j.ask(7, "fixture", 1).unwrap();
        assert!(drain(&t, Duration::ZERO).unwrap_err().contains("update deferred"));
        assert_eq!(j.request(r.request).unwrap().0, "running");
        assert!(matches!(j.ask(7, "blocked", 2), Err(e) if e.contains("drain")));
        resume_after_update(&t).unwrap();
        assert!(!t.install.join(DRAIN).exists());
        assert!(j.ask(7, "allowed", 3).is_ok());
        drop(j);
        std::fs::remove_dir_all(&t.install).unwrap();
    }

    // R14 / §9: pause, the five-minute retry window, clock rollback and corrupt state must not dispatch a download.
    #[test]
    fn scheduled_update_admission() {
        let mut t = tool();
        t.install = std::env::temp_dir().join(format!("pilotq-update-admission-{}", std::process::id()));
        std::fs::create_dir_all(&t.install).unwrap();
        std::fs::write(t.install.join("update-paused"), "paused").unwrap();
        assert_eq!(scheduled_update(&t, 1000).unwrap(), None);
        assert!(!t.install.join("update-at").exists());
        std::fs::remove_file(t.install.join("update-paused")).unwrap();
        std::fs::write(t.install.join("update-at"), "1000").unwrap();
        for at in [999, 1000, 1299] {
            assert_eq!(scheduled_update(&t, at).unwrap(), None);
            assert_eq!(std::fs::read_to_string(t.install.join("update-at")).unwrap(), "1000");
        }
        std::fs::write(t.install.join("update-at"), "invalid").unwrap();
        assert!(scheduled_update(&t, 1300).unwrap_err().contains("invalid timestamp"));
        let lock = coreq::file_lock::try_lock(&t.install.join("update.lock")).unwrap();
        assert!(matches!(update(&t, true), Err(e) if e.contains("update lock")), "a second updater cannot touch shared staging files");
        drop(lock);
        std::fs::remove_dir_all(&t.install).unwrap();
    }
}
