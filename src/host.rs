//! §9 host requirements. No package manager, credential or model operation.
use serde_json::{Value, json};
use std::{process::{Command, Stdio}, time::Duration};

pub fn log(command: &str, error: &str) {
    let mut safe = crate::keys::Keys(crate::tool().install.join("keys.json")).scrub(&format!("{command}: {error}"));
    for name in ["GH_TOKEN", "GITHUB_TOKEN", "GH_ENTERPRISE_TOKEN", "GITHUB_ENTERPRISE_TOKEN", "OPENAI_API_KEY"] {
        if let Ok(secret) = std::env::var(name) { if !secret.is_empty() { safe = safe.replace(&secret, "[credential]"); } }
    }
    eprintln!("host tool: {safe}");
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
    let result = coreq::process::output("pilotq.cleanup", &mut command, Duration::from_secs(3));
    match result {
        Ok(_) => Ok(()),
        // procps/BSD kill renders ESRCH in the fixed C locale. Do not suppress other failures.
        Err(coreq::process::Failure::Exit(o)) if String::from_utf8_lossy(&o.stderr).trim_end().ends_with(": No such process") => Ok(()),
        Err(e) => { let error=e.to_string(); log("process-group cleanup", &error); Err(error) }
    }
}

pub(crate) fn version(raw: &str) -> Option<Vec<u64>> {
    let v = raw.trim().trim_start_matches('v');
    if v.is_empty() || !v.bytes().all(|b|b.is_ascii_digit() || b == b'.') { return None; }
    let mut nums = v.split('.').map(str::parse).collect::<Result<Vec<u64>,_>>().ok()?;
    if nums.len()>3 { return None; }
    nums.resize(3,0); Some(nums)
}

pub(crate) fn probe(name: &str, bin: Option<&str>, args: &[&str], minimum: &str, prefix: &str, fix: &str) -> Value {
    let recorded = if name == "sherpa-onnx-offline" { bin.and_then(recorded_version) } else { None };
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

// Upstream 2.0.1 has no version command. Bind owner-provisioned release identity to the actual bytes.
fn recorded_version(bin: &str) -> Option<String> {
    let receipt:Value=serde_json::from_slice(&std::fs::read(format!("{bin}.pilotq-version.json")).ok()?).ok()?;
    let metadata=std::fs::metadata(bin).ok()?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; if metadata.permissions().mode() & 0o111 == 0 { return None; } }
    if !metadata.is_file() { return None; }
    let bytes=std::fs::read(bin).ok()?;
    coreq::update::verify(format!("{}  tool",receipt["sha256"].as_str()?).as_bytes(),"tool",&bytes).ok()?;
    Some(receipt["version"].as_str()?.to_string())
}

pub fn report(c: Option<&crate::Config>) -> Value {
    let voice = c.and_then(|c|c.voice.as_ref());
    let mut checks = vec![
        probe("codex",c.map(|c|c.codex.bin.as_str()),&["--version"],"0.162.0","codex-cli ","Install @openai/codex@0.162.0 or newer; set config.json codex to its absolute path"),
        probe("taskq",c.and_then(|c|c.taskq.as_ref()).and_then(|t|t.bin.to_str()),&["--version"],"4.6.46","","Update TaskQ CLI to 4.6.46 or newer; set taskq.bin"),
        probe("taskq_service",c.and_then(|c|c.taskq.as_ref()).and_then(|t|t.bin.to_str()),&["service","status","--json"],"4.6.40","","Update the existing TaskQ service to 4.6.40 or newer"),
        probe("ffmpeg",voice.map(|v|v.ffmpeg.as_str()),&["-version"],"6.1.0","ffmpeg version ","Install ffmpeg 6.1 or newer; set voice.ffmpeg"),
        probe("sherpa-onnx-offline",voice.map(|v|v.recognizer.as_str()),&["--version"],"2.0.1","","Install static sherpa-onnx 2.0.1 or newer; set voice.recognizer and create its verified .pilotq-version.json receipt (tester guide)"),
        probe("curl",Some("curl"),&["--version"],"7.68.0","curl ","Install curl 7.68 or newer on the service PATH"),
    ];
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

pub fn healthy(v: &Value) -> bool { v.as_array().is_some_and(|a|a.iter().all(|v|v["outcome"]=="ok")) }
pub fn gate(t: &coreq::Tool) -> Result<(),String> {
    let c = crate::config(t)?;
    let r = report(c.as_ref());
    if healthy(&r) { Ok(()) } else { Err(format!("Host requirements failed; complete these steps then rerun pilotq doctor --json: {r}")) }
}

#[cfg(all(test,unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
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
        for (minimum,old) in [("0.162.0","0.161.9"),("4.6.40","4.6.39"),("4.6.46","4.6.45"),("2.45.0","2.9.0"),("6.1.0","6.0.9"),("2.0.1","2.0.0"),("7.68.0","7.67.9"),("245","244"),("3.3.17","3.3.16"),("3.23","3.22")] {
            for (found,expected) in [(old,"failed"),(minimum,"ok"),("nonsense","failed"),("2.45.0-rc1","failed")] {
                std::fs::write(&bin,format!("#!/bin/sh\necho 'tool {found}'\n")).unwrap();
                std::fs::set_permissions(&bin,std::fs::Permissions::from_mode(0o700)).unwrap();
                assert_eq!(probe("fixture",bin.to_str(),&["--version"],minimum,"tool ","install tool")["outcome"],expected);
            }
        }
        // §9 / #154: TaskQ self-updates must not require a checksum receipt.
        for (found,expected) in [("4.6.45","failed"),("4.6.46","ok")] {
            std::fs::write(&bin,format!("#!/bin/sh\n[ \"$*\" = --version ] || exit 1\necho '{found}'\n")).unwrap();
            assert_eq!(probe("taskq",bin.to_str(),&["--version"],"4.6.46","","update taskq")["outcome"],expected);
        }
        // Known SHA-256 for the empty string; a changed executable cannot inherit its release identity.
        std::fs::write(&bin,"").unwrap();
        let receipt=format!("{}.pilotq-version.json",bin.display());
        std::fs::write(&receipt,json!({"version":"2.0.1","sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}).to_string()).unwrap();
        assert_eq!(recorded_version(bin.to_str().unwrap()).as_deref(),Some("2.0.1"));
        std::fs::write(&bin,"changed").unwrap();
        assert!(recorded_version(bin.to_str().unwrap()).is_none());
        std::fs::remove_dir_all(d).unwrap();
    }
}
