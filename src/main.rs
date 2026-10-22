//! PilotQ service and operator CLI on coreq (pilotq.md § 9).

mod journal;
mod telegram;

use coreq::Tool;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage: pilotq service install|run|stop|status | status [--json] | update [--check] | selftest | version
  users list | users show ID | users grant ID GRANT.. | users revoke ID [GRANT..] | users remember ID KEY VALUE [--private] | users forget ID KEY
  routes list | routes set CHAT[:TOPIC] shared|isolated        (users and routes accept --json)";
/// A status.json older than this reads unknown (coreq.md § status.json).
const FRESH: u64 = 60;
const PASS: Duration = Duration::from_secs(30);
/// Section 7 health sources, each reported separately.
const SOURCES: [&str; 5] = ["poll", "storage", "model", "taskq", "delivery"];

fn tool() -> Tool {
    Tool::new("pilotq", "PilotQ", "taskqorg/pilotq", env!("CARGO_PKG_VERSION"))
}

/// Protected bot settings in `<install>/config.json`, owner-only (§ 2: the owner ID is never taken from a message).
struct Config {
    token: String,
    owner: i64,
    api_url: Option<url::Url>,
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
    Ok(Some(Config { token, owner, api_url }))
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
    }
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
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
            rt.block_on(async { tokio::join!(telegram::poll(bot.clone(), j.clone(), wake.clone()), telegram::deliver(bot, j, wake)) });
        });
        lock
    });
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

fn fact(f: &journal::Fact) -> Value {
    json!({"key": f.key, "value": f.value, "private": f.private, "source": f.source, "revision": f.revision})
}

/// `users` and `routes` (§ 9, R10). The operator acts as the bootstrap owner from protected config; writes return the readback.
fn admin(t: &Tool, a: &[&str]) -> Result<Value, String> {
    let cfg = config(t)?.ok_or("no config.json: the CLI acts as its configured owner_id")?;
    std::fs::create_dir_all(&t.install).map_err(|e| e.to_string())?;
    let mut j = journal::Journal::open(&t.install.join("journal.sqlite"))?;
    let (me, at) = (cfg.owner, now() as i64);
    j.bootstrap(me, at)?;
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

/// Human form of an `admin` readback: one line per person or route, facts and grant history indented.
fn show(v: &Value) {
    let one = |v: &Value| {
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
            match coreq::update::update(&t, check).unwrap_or_else(|e| t.die(&e)) {
                Some(v) if check => println!("update: {v} available"),
                Some(v) => {
                    println!("update: {v}");
                    // ponytail: no drain yet (nothing runs a turn); section 12 drain joins with the journal
                    if u.path.exists() {
                        coreq::service::run_all(&u.unload, true).and_then(|_| coreq::service::run_all(&u.load, false)).unwrap_or_else(|e| t.die(&format!("restart: {e}")));
                        println!("service restart: ok");
                    }
                }
                None => println!("update: up to date"),
            }
        }
        [cmd @ ("users" | "routes"), ..] => {
            let json = a.contains(&"--json");
            let rest: Vec<&str> = a.iter().copied().filter(|x| *x != "--json").collect();
            let v = admin(&t, &rest).unwrap_or_else(|e| t.die(&format!("{cmd}: {e}")));
            if json { println!("{v}") } else { show(&v) }
        }
        ["selftest"] => selftest(&t).unwrap_or_else(|e| t.die(&e)),
        ["version"] => println!("pilotq {}", t.version),
        _ => t.die(USAGE),
    }
}
