// U22 first half (docs/acceptance.md): `pilotq update` refuses a release whose selftest fails and keeps the current
// binary; a good release becomes current. A fake `curl` on PATH serves the release files from a local folder.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("pilotq-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn executable(p: &Path, text: &[u8]) {
    fs::write(p, text).unwrap();
    fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Publishes `exe` as release `ver` and runs `pilotq update` against it.
fn update(d: &Path, ver: &str, exe: &[u8]) -> Output {
    let (site, bin, install) = (d.join("site"), d.join("bin"), d.join("install"));
    for p in [&site, &bin, &install] {
        fs::create_dir_all(p).unwrap();
    }
    let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let name = format!("pilotq-{ver}-{target}");
    fs::write(site.join(&name), exe).unwrap();
    let sum = Command::new("sh").args(["-c", "sha256sum \"$0\" 2>/dev/null || shasum -a 256 \"$0\"", &name]).current_dir(&site).output().unwrap();
    fs::write(site.join("SHA256SUMS"), sum.stdout).unwrap();
    fs::write(site.join("latest.json"), format!(r#"{{"version":"{ver}","files":{{"{target}":"{name}"}}}}"#)).unwrap();
    // coreq runs `curl -fsSL --max-time 300 -o <to> <url>`
    executable(&bin.join("curl"), format!("#!/bin/sh\ncp \"{}/$(basename \"$6\")\" \"$5\"\n", site.display()).as_bytes());
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    // HOME points at the scratch folder, so the host's own launchd unit is never seen or restarted by a test
    Command::new(env!("CARGO_BIN_EXE_pilotq")).arg("update").env("PILOTQ_INSTALL", &install).env("PATH", path).env("HOME", d).output().unwrap()
}

#[test]
fn update_refuses_a_release_whose_selftest_fails() {
    let d = scratch("update-broken");
    let install = d.join("install");
    fs::create_dir_all(install.join("releases/0.0.1")).unwrap();
    fs::write(install.join("current"), "0.0.1").unwrap();
    let out = update(&d, "9.0.0", b"#!/bin/sh\necho broken >&2\nexit 1\n");
    assert!(!out.status.success(), "a broken release must fail the update");
    assert!(String::from_utf8_lossy(&out.stderr).contains("9.0.0 selftest failed"), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(fs::read_to_string(install.join("current")).unwrap(), "0.0.1", "current stays");
    assert!(!install.join("releases/9.0.0").exists() && !install.join("releases/9.0.0.tmp").exists(), "broken release removed");
    assert!(!install.join("pilotq").exists(), "launcher untouched");
    fs::remove_dir_all(d).unwrap();
}

#[test]
fn update_installs_a_release_whose_selftest_passes() {
    let d = scratch("update-good");
    let install = d.join("install");
    let out = update(&d, "9.0.0", &fs::read(env!("CARGO_BIN_EXE_pilotq")).unwrap());
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "update: 9.0.0");
    assert_eq!(fs::read_to_string(install.join("current")).unwrap(), "9.0.0");
    assert!(install.join("releases/9.0.0/pilotq").is_file() && install.join("pilotq").is_file());
    fs::remove_dir_all(d).unwrap();
}

#[test]
fn selftest_and_status_report_unknown_health() {
    let d = scratch("status");
    let run = |args: &[&str]| Command::new(env!("CARGO_BIN_EXE_pilotq")).args(args).env("PILOTQ_INSTALL", &d).output().unwrap();
    let out = run(&["selftest"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(fs::read_dir(&d).unwrap().count(), 0, "selftest writes nothing");
    let s: serde_json::Value = serde_json::from_slice(&run(&["status", "--json"]).stdout).unwrap();
    assert_eq!(s["state"], "unknown", "no service: unknown, never healthy");
    fs::remove_dir_all(d).unwrap();
}

// U22 drain and the step 2 "update during a turn": `pilotq update` stops new turns and waits for the running request to end
// before it switches, then clears the drain marker. A fake Codex answers after 3 s; a CLI request is the running turn.
#[test]
fn update_waits_for_the_running_turn() {
    let d = scratch("update-drain");
    let install = d.join("install");
    fs::create_dir_all(&install).unwrap();
    let codex = d.join("codex");
    executable(
        &codex,
        br#"#!/bin/sh
while IFS= read -r l; do
  case "$l" in
  *'"method":"initialize"'*) echo '{"id":1,"result":{}}' ;;
  *'"method":"thread/start"'*) echo '{"id":2,"result":{"thread":{"id":"t1"}}}' ;;
  *'"method":"turn/start"'*) echo '{"id":3,"result":{"turn":{"id":"u1"}}}'; sleep 3
    echo '{"method":"item/completed","params":{"item":{"type":"agentMessage","id":"m1","text":"drained"}}}'
    echo '{"method":"turn/completed","params":{"turn":{"id":"u1","status":"completed"}}}' ;;
  esac
done
"#,
    );
    let cfg = install.join("config.json");
    fs::write(&cfg, format!(r#"{{"bot_token":"123:fake","owner_id":7,"codex":"{}"}}"#, codex.display())).unwrap();
    fs::set_permissions(&cfg, fs::Permissions::from_mode(0o600)).unwrap();
    let ask = Command::new(env!("CARGO_BIN_EXE_pilotq")).args(["ask", "slow"]).env("PILOTQ_INSTALL", &install).stdout(std::process::Stdio::piped()).spawn().unwrap();
    let running = || {
        rusqlite::Connection::open(install.join("journal.sqlite")).ok().and_then(|c| c.query_row("SELECT count(*) FROM requests WHERE state='running'", [], |r| r.get::<_, i64>(0)).ok())
    };
    let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while running() != Some(1) {
        assert!(std::time::Instant::now() < end, "the turn never started");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let out = update(&d, "9.0.0", &fs::read(env!("CARGO_BIN_EXE_pilotq")).unwrap());
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).lines().collect::<Vec<_>>(), ["update: 9.0.0", "drain: ok"]);
    assert_eq!(running(), Some(0), "the switch came after the turn ended");
    let ask = ask.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&ask.stdout).trim(), "drained");
    assert!(!install.join("drain").exists(), "the drain marker is cleared");
    fs::remove_dir_all(d).unwrap();
}
