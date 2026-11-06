//! §9 host requirements. No package manager, credential or model operation.
use serde_json::{Value, json};
use std::{process::{Command, Stdio}, time::Duration};

pub fn log(command: &str, error: &str) {
    let keys = crate::keys::Keys(crate::tool().install.join("keys.json"));
    let scrub = |text: &str| {
        let mut safe = keys.scrub(text);
        for name in ["GH_TOKEN", "GITHUB_TOKEN", "GH_ENTERPRISE_TOKEN", "GITHUB_ENTERPRISE_TOKEN", "OPENAI_API_KEY"] {
            if let Ok(secret) = std::env::var(name) { if !secret.is_empty() { safe = safe.replace(&secret, "[credential]"); } }
        }
        safe
    };
    eprintln!("host tool: {}", one_line(&scrub(command), &scrub(error)));
}

/// #257: a multi-line child stderr (such as a Python traceback) becomes one journal line: the capped command, the
/// error's first line and its final summary. The summary is never cut by a long command. Full scrubbed diagnostics
/// stay in the request audit where a caller records them.
fn one_line(command: &str, error: &str) -> String {
    let cap = |text: &str, max: usize| if text.chars().count() <= max { text.to_string() } else { text.chars().take(max - 1).chain(['…']).collect() };
    let mut lines = error.lines().map(str::trim).filter(|l| !l.is_empty());
    let first = lines.next().unwrap_or("");
    let error = match lines.last() { Some(last) => format!("{} … {}", cap(first, 200), cap(last, 500)), None => cap(first, 500) };
    format!("{}: {error}", cap(command, 300))
}

pub fn output(c: &mut Command) -> Result<std::process::Output, String> {
    output_with_budget(c,Duration::from_secs(3))
}

pub fn output_with_budget(c: &mut Command, budget: Duration) -> Result<std::process::Output, String> {
    let command = format!("{:?} {:?}", c.get_program(), c.get_args().collect::<Vec<_>>());
    let result = coreq::process::output("pilotq.host", c.stdin(Stdio::null()), budget)
        .map_err(|e| match e { coreq::process::Failure::Exit(o) => format!("{}: {}",o.status,String::from_utf8_lossy(&o.stderr)), e=>e.to_string() }).and_then(|o| {
            if o.status.success() { Ok(o) } else { Err(format!("{}: {}", o.status, String::from_utf8_lossy(&o.stderr))) }
        });
    if let Err(e) = &result { log(&command,e); }
    result
}

/// §5 / #205: cleanup of an exited group is already complete.
#[cfg(unix)]
pub(crate) fn kill_group(pid: u32) -> Result<(), String> {
    if pid == 0 || pid > i32::MAX as u32 { return Err("invalid process group".into()); }
    let mut command = Command::new("/bin/kill");
    command.args(crate::manager::group_kill_args(pid)).env("LC_ALL", "C").stdin(Stdio::null());
    // ponytail: kill signals and exits, so it needs no deadline. coreq::process::output would record
    // the routine exited group as a failed operation on every bounded command.
    let result = coreq::telemetry::capture::operation("pilotq.cleanup", || {
        let o = command.output().map_err(|e| e.to_string())?;
        let stderr = String::from_utf8_lossy(&o.stderr);
        // procps/BSD kill renders ESRCH in the fixed C locale. Do not suppress other failures.
        if o.status.success() || stderr.trim_end().ends_with(": No such process") { Ok(()) } else { Err(format!("{}: {stderr}", o.status)) }
    }, |_| "cleanup-failed");
    if let Err(error) = &result { log("process-group cleanup", error); }
    result
}

pub(crate) fn version(raw: &str) -> Option<Vec<u64>> {
    let v = raw.trim().trim_start_matches('v');
    if v.is_empty() || !v.bytes().all(|b|b.is_ascii_digit() || b == b'.') { return None; }
    let mut nums = v.split('.').map(str::parse).collect::<Result<Vec<u64>,_>>().ok()?;
    if nums.len()>3 { return None; }
    nums.resize(3,0); Some(nums)
}

pub(crate) fn probe(name: &str, bin: Option<&str>, args: &[&str], minimum: &str, prefix: &str, fix: &str) -> Value {
    let recorded = if matches!(name, "whisper-cli" | "sherpa-onnx-offline") { bin.and_then(recorded_version) } else { None };
    let found = recorded.or_else(|| bin.and_then(|bin|output(Command::new(bin).args(args)).ok()).and_then(|o| {
        let text = String::from_utf8_lossy(&o.stdout).to_string()+&String::from_utf8_lossy(&o.stderr);
        if name == "taskq_service" {
            return serde_json::from_str::<Value>(&text).ok()?["service"]["version"].as_str().map(str::to_string);
        }
        // Approved executables may print a banner such as pwd (GNU coreutils) 8.30.
        // Read only the first line; later copyright text is not a version.
        if name == "command" {
            return text.lines().next()?.split_whitespace().last().map(str::to_string);
        }
        text.lines().find_map(|l|l.trim().strip_prefix(prefix)?.split_whitespace().next().map(str::to_string))
    }));
    // Only ffmpeg's distribution package suffix is accepted; prereleases are unqualified.
    let found = found.map(|v| if name == "ffmpeg" { v.split('-').next().unwrap().to_string() } else { v })
        .filter(|v|version(v).is_some());
    let ok = found.as_deref().and_then(version).is_some_and(|v|v>=version(minimum).unwrap());
    if !ok { log(name,&format!("found {}; minimum {minimum}; {fix}",found.as_deref().unwrap_or("unavailable"))); }
    json!({"tool":name,"found":found,"minimum":minimum,"outcome":if ok {"ok"} else {"failed"},"fix":fix})
}

// Sherpa 2.0.1 and whisper.cpp 1.8.2 have no version command. Bind owner-provisioned release identity to the actual bytes.
fn recorded_version(bin: &str) -> Option<String> {
    let receipt:Value=serde_json::from_slice(&std::fs::read(format!("{bin}.pilotq-version.json")).ok()?).ok()?;
    let metadata=std::fs::metadata(bin).ok()?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; if metadata.permissions().mode() & 0o111 == 0 { return None; } }
    if !metadata.is_file() { return None; }
    let bytes=std::fs::read(bin).ok()?;
    coreq::update::verify(format!("{}  tool",receipt["sha256"].as_str()?).as_bytes(),"tool",&bytes).ok()?;
    Some(receipt["version"].as_str()?.to_string())
}

/// §1/#344: pinned community export, not official checkpoint-equivalence attestation.
/// Provisioning never downloads or rewrites an asset; doctor verifies bytes before loading it.
pub const CTC_ORIGIN: &str = "https://huggingface.co/fussraider/GigaAM-Multilingual-sherpa-onnx-ctc/resolve/9f5a77e8975211abe8511693accd3a63ee1e9f43";
pub const CTC_ASSETS: [(&str, u64, &str); 3] = [
    ("model.int8.onnx", 591645642, "6b6f195026b0f90721cd4593c664becf009a71131550b664eec71446ec351c81"),
    ("tokens.txt", 391, "9b5df7987cb4ca52c1a468649ce897fab1cd182067416e29fef49dfaa7a856c2"),
    ("LICENSE", 1070, "f00de6715714c7a63d08639cdbfaa40224eefc407302614bd19f1a8b98c875aa"),
];
pub fn verify_ctc_assets(dir: &std::path::Path) -> Result<(), String> {
    for (name, size, sha) in CTC_ASSETS {
        let path = dir.join(name);
        let fix = || format!("voice model verification failed: provision {name} from {CTC_ORIGIN}/{}; see docs/tester-guide.md", if name == "LICENSE" { name.to_string() } else { format!("large/{name}") });
        if !std::fs::metadata(&path).is_ok_and(|m| m.is_file() && m.len() == size) { return Err(fix()); }
        let bytes = std::fs::read(path).map_err(|_| fix())?;
        coreq::update::verify(format!("{sha}  {name}").as_bytes(), name, &bytes).map_err(|_| fix())?;
    }
    Ok(())
}

pub fn report(c: Option<&crate::Config>) -> Value {
    let voice = c.and_then(|c|c.voice.as_ref());
    let mut checks = vec![
        probe("taskq",c.and_then(|c|c.taskq.as_ref()).and_then(|t|t.bin.to_str()),&["--version"],"4.6.46","","Update TaskQ CLI to 4.6.46 or newer; set taskq.bin"),
        probe("taskq_service",c.and_then(|c|c.taskq.as_ref()).and_then(|t|t.bin.to_str()),&["service","status","--json"],"4.6.40","","Update the existing TaskQ service to 4.6.40 or newer"),
        probe("ffmpeg",voice.map(|v|v.ffmpeg.as_str()),&["-version"],"6.1.0","ffmpeg version ","Install ffmpeg 6.1 or newer; set voice.ffmpeg"),
        if voice.is_some_and(|v| v.whisper) {
            probe("whisper-cli",voice.map(|v|v.recognizer.as_str()),&["--version"],"1.8.2","","Provision verified CPU whisper.cpp 1.8.2 or newer; set voice.recognizer and its verified .pilotq-version.json receipt (tester guide)")
        } else { probe("sherpa-onnx-offline",voice.map(|v|v.recognizer.as_str()),&["--version"],"2.0.1","","Provision sherpa-onnx-offline 2.0.1 or newer; set voice.recognizer and the pinned GigaAM-Multilingual large model directory (docs/tester-guide.md); preserve legacy assets") },
        probe("curl",Some("curl"),&["--version"],"7.68.0","curl ","Install curl 7.68 or newer on the service PATH"),
    ];
    // #265/#359: each runtime the installation or a route runs the manager on; without config, the default runtime
    let used = c.map_or_else(|| vec![crate::manager::default_runtime()], |c| c.runtime.used());
    for r in crate::manager::supported().filter(|r| used.contains(&r.descriptor().name)) {
        let agent = c.and_then(|c| c.runtime.agents.get(r.descriptor().name));
        checks.push(runtime_check(r, agent.map_or(r.descriptor().name, |a| a.bin.as_str()), agent.and_then(|a| a.home.as_deref())));
    }
    if cfg!(target_os="linux") {
        checks.extend([
            probe("systemctl",Some("systemctl"),&["--version"],"245","systemd ","Provision systemd 245 or newer and its user service manager"),
            probe("kill",Some("kill"),&["--version"],"3.3.17","kill from procps-ng ","Install procps-ng 3.3.17 or newer on the service PATH"),
            probe("kill_absolute",Some("/bin/kill"),&["--version"],"3.3.17","kill from procps-ng ","Install procps-ng 3.3.17 or newer at /bin/kill"),
            probe("hostname",Some("hostname"),&["--version"],"3.23","hostname ","Install hostname 3.23 or newer on the service PATH"),
        ]);
    }
    let direct = c.map(|c|&c.direct);
    let default_direct=crate::direct(&crate::tool());
    match crate::commands::load(direct.unwrap_or(&default_direct)) {
        Ok(entries) => for (name,entry) in entries.as_object().unwrap() {
            let minimum=entry["minimum_version"].as_str().filter(|s|version(s).is_some());
            let check=if let Some(minimum)=minimum {
                let mut v=probe("command",entry["executable"].as_str(),&["--version"],minimum,"","Provision the approved executable with a working --version command");
                v["tool"]=json!(format!("command:{name}"));v
            } else {
                json!({"tool":format!("command:{name}"),"found":null,"minimum":null,"outcome":"failed","fix":"Declare minimum_version in the protected commands.json entry and provision its executable with --version"})
            };
            checks.push(check);
        },
        Err(e) => checks.push(json!({"tool":"installed_commands","found":null,"minimum":null,"outcome":"failed","fix":e})),
    }
    checks.extend(crate::release::host_checks(&crate::tool().install.join("journal.sqlite")));
    json!(checks)
}

/// § 9 runtime row from the trait's readiness (#363): install and minimum version; login is doctor's runtime check.
pub(crate) fn runtime_check(r: &dyn agentq_runtime::agent::Runtime, bin: &str, home: Option<&std::path::Path>) -> Value {
    let d = r.descriptor();
    let ready = r.ready(Some(std::path::Path::new(bin)), home);
    let ok = ready.install.state == "ok" && ready.version.state == "ok";
    let fix = format!("Install {} {} or newer; set config.json runtimes.{}.bin to its absolute path", d.label, d.minimum, d.name);
    let found = (ready.version.state != "unknown").then(|| ready.version.detail.clone());
    if !ok { log(d.name, &format!("found {}; minimum {}; {fix}", found.as_deref().unwrap_or("unavailable"), d.minimum)); }
    json!({"tool": d.name, "found": found, "minimum": d.minimum, "outcome": if ok {"ok"} else {"failed"}, "fix": fix})
}

pub fn healthy(v: &Value) -> bool { v.as_array().is_some_and(|a|a.iter().all(|v|v["outcome"]=="ok")) }
pub fn gate(t: &coreq::Tool) -> Result<(),String> {
    let c = crate::config(t)?;
    let r = report(c.as_ref());
    if healthy(&r) { Ok(()) } else { Err(format!("Host requirements failed; complete these steps then rerun pilotq doctor --json: {r}")) }
}

#[cfg(all(test,unix))]
mod tests {
    use super::*;
    // #257 A5: an adapter's HTTPS handshake timeout traceback is logged as one short line ending in its summary.
    #[test]
    fn traceback_logs_one_line() {
        let trace = "exit_nonzero: Traceback (most recent call last):\n  File \"/opt/adapter\", line 9, in <module>\n    with urllib.request.urlopen(req, timeout=4) as r:\n         ~~~~^^^^\nTimeoutError: _ssl.c:1011: The handshake operation timed out\n";
        let line = one_line("\"/opt/adapter\" [\"541\"]", trace);
        assert_eq!(line, "\"/opt/adapter\" [\"541\"]: exit_nonzero: Traceback (most recent call last): … TimeoutError: _ssl.c:1011: The handshake operation timed out");
        // A >500-char command (long macOS temp paths) still ends in the error summary.
        let long = one_line(&"x".repeat(900), trace);
        assert!(long.chars().count() < 900 && long.ends_with("TimeoutError: _ssl.c:1011: The handshake operation timed out"), "{long}");
        assert!(one_line(&"x".repeat(900), "exit status: 1: private-dependency-output").ends_with("private-dependency-output"));
    }

    // #205 A2 / R5: ESRCH during cleanup must succeed without host-error logging.
    #[test]
    fn live_gaps_205_missing_group() {
        // Linux PIDs cannot reach i32::MAX; this must not target an existing group.
        assert!(kill_group(i32::MAX as u32).is_ok());
    }

    // §9 / #130: numeric ordering, oldest supported versions and unreadable identities must fail closed.
    #[test]
    fn host_requirement_versions_and_receipts() {
        let d=std::env::temp_dir().join(format!("pilotq-host-versions-{}",std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let bin=d.join("tool");
        for (minimum,old) in [("0.162.0","0.161.9"),("4.6.40","4.6.39"),("4.6.46","4.6.45"),("2.45.0","2.9.0"),("6.1.0","6.0.9"),("2.0.1","2.0.0"),("1.8.2","1.8.1"),("7.68.0","7.67.9"),("245","244"),("3.3.17","3.3.16"),("3.23","3.22")] {
            for (found,expected) in [(old,"failed"),(minimum,"ok"),("nonsense","failed"),("2.45.0-rc1","failed")] {
                crate::test_support::write_script(&bin,format!("#!/bin/sh\necho 'tool {found}'\n")).unwrap();
                assert_eq!(probe("fixture",bin.to_str(),&["--version"],minimum,"tool ","install tool")["outcome"],expected);
            }
        }
        // §9 / #154: TaskQ self-updates must not require a checksum receipt.
        for (found,expected) in [("4.6.45","failed"),("4.6.46","ok")] {
            crate::test_support::write_script(&bin,format!("#!/bin/sh\n[ \"$*\" = --version ] || exit 1\necho '{found}'\n")).unwrap();
            assert_eq!(probe("taskq",bin.to_str(),&["--version"],"4.6.46","","update taskq")["outcome"],expected);
        }
        // Known SHA-256 for the empty string; a changed executable cannot inherit its release identity.
        std::fs::write(&bin,"").unwrap();
        let receipt=format!("{}.pilotq-version.json",bin.display());
        std::fs::write(&receipt,json!({"version":"2.0.1","sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}).to_string()).unwrap();
        assert_eq!(recorded_version(bin.to_str().unwrap()).as_deref(),Some("2.0.1"));
        assert_eq!(probe("sherpa-onnx-offline",bin.to_str(),&["--version"],"2.0.1","","retain verified receipt")["outcome"],"ok");
        std::fs::write(&bin,"changed").unwrap();
        assert!(recorded_version(bin.to_str().unwrap()).is_none());
        std::fs::remove_dir_all(d).unwrap();
    }
}
