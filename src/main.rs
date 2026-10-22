//! PilotQ service and operator CLI on coreq (pilotq.md § 9).

mod journal;
mod telegram;

use coreq::Tool;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage: pilotq service install|run|stop|status | status [--json] | update [--check] | selftest | version";
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
    let j: telegram::Shared = Arc::new(Mutex::new(journal::Journal::open(&t.install.join("journal.sqlite")).unwrap_or_else(|e| t.die(&e))));
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
            rt.block_on(async { tokio::join!(telegram::poll(bot.clone(), j.clone(), c.owner, wake.clone()), telegram::deliver(bot, j, wake)) });
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
        ["selftest"] => selftest(&t).unwrap_or_else(|e| t.die(&e)),
        ["version"] => println!("pilotq {}", t.version),
        _ => t.die(USAGE),
    }
}
