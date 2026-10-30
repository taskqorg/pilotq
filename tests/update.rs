// U22 first half (docs/acceptance.md): `pilotq update` refuses a release whose selftest fails and keeps the current
// binary; a good release becomes current. A fake `curl` on PATH serves the release files from a local folder.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use serde_json::Value;
#[path="support/host.rs"] mod host_support;
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

/// Publishes an invented release to the local download fixture.
fn publish(d: &Path, ver: &str, exe: &[u8]) {
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
    host_support::provision(&bin,&install.join("config.json"));
    // coreq runs `curl -fsSL --max-time 300 -o <to> <url>`
    executable(&bin.join("curl"), format!("#!/bin/sh\ncp \"{}/$(basename \"$6\")\" \"$5\"\n", site.display()).as_bytes());
    host_support::wrap(&bin.join("curl"),"--version","curl 7.68.0");
}

fn command(d: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_pilotq"));
    c.env("PILOTQ_INSTALL", d.join("install")).env("HOME", d)
        .env("PATH", format!("{}:{}", d.join("bin").display(), std::env::var("PATH").unwrap()));
    c
}

fn update(d: &Path, ver: &str, exe: &[u8]) -> Output {
    publish(d, ver, exe);
    command(d).arg("update").output().unwrap()
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

// §10 step 5 / A4: both CLI formats must expose gate counts; tool measurements retain their own window.
#[test]
fn pilot_report_prints_gate_counts_and_tool_window() {
    let d = scratch("pilot-report");
    fs::write(d.join("config.json"), r#"{"bot_token":"123:fake","owner_id":7}"#).unwrap();
    fs::set_permissions(d.join("config.json"), fs::Permissions::from_mode(0o600)).unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_pilotq")).args(args).env("PILOTQ_INSTALL", &d).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        out.stdout
    };
    run(&["pilot", "start", "--json"]);
    let db = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    db.execute("INSERT INTO pilot_notes(at, by, kind, text) VALUES (30, 7, 'intervention', 'invented fixture')", []).unwrap();
    for (at, bytes) in [(10, 99), (20, 42), (30, 100)] {
        db.execute("INSERT INTO tool_sizes(request_id, run_id, at, tool, result_bytes, codex_version, pilotq_version) VALUES (1,1,?1,'issue_read',?2,'fixture','fixture')", rusqlite::params![at, bytes]).unwrap();
    }
    let args = ["pilot", "report", "--since", "20", "--until", "30"];
    let text = String::from_utf8(run(&args)).unwrap();
    let mut json_args = args.to_vec();
    json_args.push("--json");
    let json: serde_json::Value = serde_json::from_slice(&run(&json_args)).unwrap();
    for key in ["accepted", "simple", "users", "routes", "projects", "tasks", "interventions", "outages"] {
        let expected = if key == "interventions" { 1 } else { 0 };
        assert_eq!(json[key], expected, "{key}");
        assert!(text.contains(&format!("{key}: {expected}\n")), "missing {key}: {text}");
    }
    assert!(json["failed"].is_null());
    assert!(json["failed_reason"].is_string());
    assert!(text.contains("failed: unknown ("), "{text}");
    assert!(text.contains(json["failed_reason"].as_str().unwrap()));
    assert!(text.contains("Tool result bytes [20 .. 30)"));
    assert!(text.contains("issue_read: calls=1 p95=42 max=42"));
    assert_eq!(json["tools"], serde_json::json!([{"tool":"issue_read","calls":1,"p95_result_bytes":42,"max_result_bytes":42}]));
    assert_eq!(json["since"], 20);
    assert_eq!(json["until"], 30);
    fs::remove_dir_all(d).unwrap();
}

// R14 / §9: the real installed service must dispatch automatic updates, preserve the old selection on
// selftest failure and exit only after verified activation; unit checks cover pause and persisted retry time.
#[test]
#[cfg(target_os = "linux")]
fn installed_service_updates_on_schedule() {
    use std::process::{Child, Stdio};
    use std::time::{Duration, Instant};
    struct Service(Child);
    impl Drop for Service {
        fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
    }
    fn wait(mut ready: impl FnMut() -> bool) {
        let end = Instant::now() + Duration::from_secs(15);
        while !ready() {
            assert!(Instant::now() < end, "service did not reach the expected update state");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let d = scratch("scheduled-update");
    let install = d.join("install");
    let current = env!("CARGO_PKG_VERSION");
    let old = install.join(format!("releases/{current}/pilotq"));
    fs::create_dir_all(old.parent().unwrap()).unwrap();
    let binary = fs::read(env!("CARGO_BIN_EXE_pilotq")).unwrap();
    executable(&old, &binary);
    fs::write(install.join("current"), current).unwrap();
    publish(&d, "9.0.0", b"#!/bin/sh\necho refused > \"$PILOTQ_INSTALL/refused\"\nexit 1\n");
    let start = || {
        let mut c = Command::new(&old);
        c.args(["service", "run"]).env("PILOTQ_INSTALL", &install).env("HOME", &d)
            .env("PATH", format!("{}:{}", d.join("bin").display(), std::env::var("PATH").unwrap()))
            .stdout(Stdio::null()).stderr(Stdio::null());
        Service(c.spawn().unwrap())
    };
    // Failure records the attempt and removes the drain, while health and service remain available.
    let mut service = start();
    wait(|| install.join(".pilotq/status.json").exists() && install.join("refused").exists() && !install.join("drain").exists());
    assert!(service.0.try_wait().unwrap().is_none());
    assert_eq!(fs::read_to_string(install.join("current")).unwrap(), current);
    drop(service);
    publish(&d, "9.0.0", &binary);
    assert!(command(&d).args(["update", "--pause"]).status().unwrap().success());
    assert!(install.join("update-paused").exists());
    assert!(command(&d).args(["update", "--resume"]).status().unwrap().success());
    assert!(!install.join("update-paused").exists());
    fs::write(install.join("update-at"), "0").unwrap();
    let mut service = start();
    let mut exit = None;
    wait(|| { exit = service.0.try_wait().unwrap(); exit.is_some() });
    assert_eq!(exit.unwrap().code(), Some(75), "native unit restarts through launcher");
    assert_eq!(fs::read_to_string(install.join("current")).unwrap(), "9.0.0");
    assert_eq!(fs::read_to_string(install.join("previous")).unwrap(), current);
    assert_eq!(fs::read(install.join("releases/9.0.0/pilotq")).unwrap(), binary);
    assert!(install.join("drain").exists(), "admission stays stopped until restart");
    drop(service);
    fs::remove_dir_all(d).unwrap();
}

// §9: absent setup must fail explicitly, without claiming healthy dependencies or writing state.
#[test]
fn doctor_missing_configuration_is_not_healthy() {
    let d = scratch("doctor-missing");
    let out = Command::new(env!("CARGO_BIN_EXE_pilotq"))
        .env("PILOTQ_INSTALL", &d).args(["doctor", "--json"]).output().unwrap();
    assert!(!out.status.success());
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(report["checks"]["configuration"], "failed");
    for key in ["telegram", "codex", "taskq"] { assert_eq!(report["checks"][key], "unknown"); }
    assert_eq!(fs::read_dir(&d).unwrap().count(), 0);
    fs::remove_dir_all(d).unwrap();
}

// #130 A2/A4, §9: an old executable must fail doctor with a concrete repair, never look healthy.
#[test]
#[cfg(unix)]
fn doctor_host_requirements_old_and_missing() {
    use std::os::unix::fs::PermissionsExt;
    let d = scratch("host-requirements");
    let gh = d.join("curl");
    for (body, found, outcome) in [
        ("echo 'curl 7.67.0 (invented)'", Some("7.67.0"), "failed"),
        ("echo 'curl 7.68.0 (invented)'", Some("7.68.0"), "ok"),
        ("echo invented-tool-error >&2; exit 1", None, "failed"),
    ] {
        fs::write(&gh, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&gh,fs::Permissions::from_mode(0o700)).unwrap();
        let out=Command::new(env!("CARGO_BIN_EXE_pilotq")).env("PILOTQ_INSTALL",&d).env("PATH",&d)
            .args(["doctor","--json"]).output().unwrap();
        let report:Value=serde_json::from_slice(&out.stdout).unwrap();
        let check=report["requirements"].as_array().expect("doctor must declare host requirements").iter().find(|c|c["tool"]=="curl").unwrap();
        assert_eq!(check["found"], serde_json::json!(found));
        assert_eq!(check["minimum"],"7.68.0");
        assert_eq!(check["outcome"],outcome);
        assert!(check["fix"].as_str().unwrap().contains("Install curl"));
        assert!(!out.status.success(),"missing configuration cannot pass");
        if found.is_none() { assert!(String::from_utf8_lossy(&out.stderr).contains("invented-tool-error")); }
        assert_eq!(report["checks"]["host_tools"],"failed");
    }
    fs::remove_file(gh).unwrap();
    let out=Command::new(env!("CARGO_BIN_EXE_pilotq")).env("PILOTQ_INSTALL",&d).env("PATH",&d)
        .args(["doctor","--json"]).output().unwrap();
    let report:Value=serde_json::from_slice(&out.stdout).unwrap();
    assert!(report["requirements"].as_array().unwrap().iter().filter(|c|c["tool"]!="kill_absolute").all(|c|c["outcome"]=="failed"));
    // Operator-approved commands also fail on undeclared, old or changed binary identities.
    let tool=d.join("approved");
    executable(&tool,b"");
    let receipt=format!("{}.pilotq-version.json",tool.display());
    for (minimum,declared,changed,expected) in [(None,"2.0.0",false,"failed"),(Some("2.0.1"),"2.0.0",false,"failed"),(Some("2.0.1"),"2.0.1",false,"ok"),(Some("2.0.1"),"2.0.1",true,"failed")] {
        fs::write(d.join("commands.json"),serde_json::json!({"approved":{"executable":tool,"minimum_version":minimum}}).to_string()).unwrap();
        fs::set_permissions(d.join("commands.json"),fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&receipt,serde_json::json!({"version":declared,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}).to_string()).unwrap();
        if changed { fs::write(&tool,"changed").unwrap(); }
        let out=Command::new(env!("CARGO_BIN_EXE_pilotq")).env("PILOTQ_INSTALL",&d).env("PATH",&d).args(["doctor","--json"]).output().unwrap();
        let report:Value=serde_json::from_slice(&out.stdout).unwrap();
        let check=report["requirements"].as_array().unwrap().iter().find(|c|c["tool"]=="command:approved").unwrap();
        assert_eq!(check["outcome"],expected);
    }
    fs::remove_dir_all(d).unwrap();
}

// §9 / #130 A3: unsupported host tools must stop install/update before activation or draining accepted work.
#[test]
fn host_requirements_block_install_and_update() {
    let d=scratch("host-gates");
    let bin=d.join("bin");
    let install=d.join("install");
    fs::create_dir_all(&install).unwrap();
    host_support::provision(&bin,&install.join("config.json"));
    executable(&bin.join("curl"),b"#!/bin/sh\necho 'curl 7.67.0 (invented)'\n");
    fs::write(install.join("current"),"0.0.1").unwrap();
    for args in [vec!["service","install"],vec!["update","--check"],vec!["update"]] {
        let out=command(&d).args(args).output().unwrap();
        assert!(!out.status.success());
        let error=String::from_utf8_lossy(&out.stderr);
        for expected in ["curl","7.67.0","7.68.0","Install curl"] { assert!(error.contains(expected),"{error}"); }
        assert_eq!(fs::read_to_string(install.join("current")).unwrap(),"0.0.1");
        assert!(!install.join("drain").exists());
        assert!(!install.join("pilotq").exists());
    }
    fs::remove_dir_all(d).unwrap();
}

// §9 / #135 A2: an installed start must refuse missing tools even in offline selftest mode.
#[test]
fn installed_start_refuses_missing_host_tool() {
    let d = scratch("start-missing-tool");
    let install = d.join("install");
    let bin = d.join("bin");
    fs::create_dir_all(&install).unwrap();
    host_support::provision(&bin, &install.join("config.json"));
    fs::remove_file(bin.join("codex")).unwrap();
    let release = install.join(format!("releases/{}/pilotq", env!("CARGO_PKG_VERSION")));
    fs::create_dir_all(release.parent().unwrap()).unwrap();
    executable(&release, &fs::read(env!("CARGO_BIN_EXE_pilotq")).unwrap());
    let mut child = Command::new(&release).args(["service", "run"])
        .env("PILOTQ_INSTALL", &install).env("HOME", &d)
        .env("PATH", &bin).env("PILOTQ_SELFTEST_OFFLINE", "1")
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).spawn().unwrap();
    let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() && std::time::Instant::now() < end {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let timed_out = child.try_wait().unwrap().is_none();
    if timed_out { child.kill().unwrap(); }
    let out = child.wait_with_output().unwrap();
    assert!(!timed_out, "installed start did not refuse missing codex");
    assert!(!out.status.success());
    let error = String::from_utf8_lossy(&out.stderr);
    for expected in ["Host requirements failed", "codex", "unavailable", "0.162.0"] {
        assert!(error.contains(expected), "{error}");
    }
    assert!(!install.join("journal.sqlite").exists(), "refuse before starting accepted work");
    fs::remove_dir_all(d).unwrap();
}
