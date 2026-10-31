//! #126: operator-approved read capabilities, never arbitrary shell execution (§5).
use crate::{direct::Direct, journal::{Journal, Run}};
use serde_json::{Value, json};
use std::{io::Read, path::Path, process::{Command, Stdio}, sync::{Mutex, mpsc}, time::{Duration, Instant}};

const CAP: usize = 16 * 1024;
// Internal board validation is not command output returned to the model (§5).
const BOARD_CAP: usize = 1024 * 1024;
pub const NAMES: [&str; 2] = ["command_list", "command_run"];

pub fn tools() -> Vec<Value> {
    vec![
        json!({"type":"function","name":"command_list","description":crate::catalog::purpose("command_list"),"inputSchema":{"type":"object","properties":{},"additionalProperties":false}}),
        json!({"type":"function","name":"command_run","description":crate::catalog::purpose("command_run"),"inputSchema":{"type":"object","properties":{"command":{"type":"string"},"args":{"type":"array","items":{"type":"string"}}},"required":["command","args"],"additionalProperties":false}}),
    ]
}

pub(crate) fn load(d: &Direct) -> Result<Value, String> {
    let path = d.keys.0.with_file_name("commands.json");
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(json!({})),
        Err(_) => return Err("command configuration unavailable".into()),
    };
    let meta = file.metadata().map_err(|_| "command configuration unavailable")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 { return Err("commands.json must be owner-only (chmod 600)".into()); }
    }
    if !meta.is_file() || meta.len() > 65536 { return Err("invalid command configuration".into()); }
    serde_json::from_reader(file).ok().filter(Value::is_object).ok_or("invalid command configuration".into())
}

fn authorize(j: &Journal, r: &Run, entry: &Value) -> Result<String, String> {
    let user = j.sender(r.request)?;
    if j.fenced(r)? || !j.has(user, "chat")? { return Err("command refused: stopped run or missing chat grant".into()); }
    let project = entry["project"].as_str().ok_or("command has no project")?;
    if !entry["users"].as_array().is_some_and(|a| a.contains(&json!(user)))
        || !(j.has(user, "projects")? || j.has(user, &format!("project:{project}"))?) {
        return Err("command permission denied".into());
    }
    let (_, board, root, ..) = j.projects()?.into_iter().find(|p| p.0 == project).ok_or("command project is not registered")?;
    if entry["board"] != board || entry["cwd"] != root { return Err("command registration changed".into()); }
    let config: Value = std::fs::read_to_string(Path::new(&root).join("taskq.json")).ok()
        .and_then(|s| serde_json::from_str(&s).ok()).ok_or("command project configuration unavailable")?;
    if config["repo"] != board { return Err("command board mismatch".into()); }
    Ok(project.into())
}

#[allow(clippy::too_many_arguments)]
pub fn call(d: &Direct, j: &Mutex<Journal>, r: &Run, name: &str, args: &Value, deadline: Instant, tq: Option<&crate::taskq::Taskq>) -> (bool, String) {
    let result = (|| {
        let entries = load(d)?;
        if name == "command_list" {
            let j = j.lock().unwrap();
            if j.fenced(r)? || !j.has(j.sender(r.request)?, "chat")? { return Err("command discovery refused".into()); }
            let list: Vec<_> = entries.as_object().unwrap().iter().filter(|(_,e)| authorize(&j,r,e).is_ok())
                .map(|(name,e)| json!({"command":name,"description":e["description"],"args":e["args"]})).collect();
            j.direct_event(r.request,"","command_list",None,"read approved capabilities",crate::now() as i64)?;
            return Ok(json!({"commands":list}).to_string());
        }
        let selected = args["command"].as_str().ok_or("command name is required")?;
        let entry = entries.get(selected).ok_or("unknown or unapproved command; use command_list")?;
        let argv = args["args"].as_array().filter(|a| a.iter().all(Value::is_string)).ok_or("args must be a string array")?;
        if !entry["args"].as_array().is_some_and(|a| a.contains(&args["args"])) { return Err("command arguments are not approved".into()); }
        let bin = entry["executable"].as_str().filter(|p|Path::new(p).is_absolute()).ok_or("command executable must be absolute")?;
        let cwd = entry["cwd"].as_str().filter(|p|Path::new(p).is_absolute()).ok_or("command cwd must be absolute")?;
        let end = deadline.min(Instant::now() + Duration::from_secs(10));
        if Instant::now() >= end { return Err("command timeout: request deadline reached".into()); }
        let project = authorize(&j.lock().unwrap(),r,entry)?;
        let active = || load(d).is_ok_and(|v|v.get(selected)==Some(entry))
            && authorize(&j.lock().unwrap(),r,entry).is_ok();
        // §5: a complete fresh board observation is required before project reads.
        // The probe and selected command share a deadline; each has bounded output.
        let tq = tq.ok_or("TaskQ is not configured; cannot verify the registered board")?;
        let probe = execute_bounded(command(&tq.bin,cwd,&["status","--json"]),end,&active,BOARD_CAP);
        let snapshot: Value = probe["stdout"].as_str().and_then(|s|serde_json::from_str(s).ok()).unwrap_or(Value::Null);
        if probe["status"] != "success" || snapshot["complete"] != true || snapshot["board"]["repo"] != entry["board"] {
            crate::host::log(&format!("{:?} status --json",tq.bin),
                &format!("invalid, incomplete or mismatched board snapshot; stderr: {}",probe["stderr"].as_str().unwrap_or("unavailable")));
            return Err(format!("registered board unavailable or mismatched ({}); selected command was not executed",probe["status"]));
        }
        {
            let j = j.lock().unwrap();
            authorize(&j,r,entry)?;
            j.direct_event(r.request,&project,&format!("command_run {selected}"),None,&format!("start args={}",args["args"]),crate::now() as i64)?;
        }
        let argv: Vec<_> = argv.iter().map(|v|v.as_str().unwrap()).collect();
        let command = command(bin,cwd,&argv);
        // Authorization is checked again immediately before dispatch.
        if !active() { return Err("command capability changed or was revoked; nothing was executed".into()); }
        let output = execute(command,end,active);
        let out = d.keys.scrub(&output.to_string());
        j.lock().unwrap().direct_event(r.request,&project,&format!("command_run {selected}"),None,&out,crate::now() as i64)?;
        Ok(out)
    })();
    match result {
        Ok(s) => { let ok = name == "command_list" || serde_json::from_str::<Value>(&s).is_ok_and(|v|v["status"]=="success"); (ok,s) },
        Err(e) => (false,e),
    }
}

pub(crate) fn command(bin: impl AsRef<std::ffi::OsStr>, cwd: &str, argv: &[&str]) -> Command {
    let mut command = Command::new(bin);
    command.args(argv).current_dir(cwd).env_clear()
        .env("PATH","/usr/bin:/bin").env("TASKQ_ROOT",cwd).env("LANG","C.UTF-8")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(home) = std::env::var_os("HOME") { command.env("HOME",home); }
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command,0);
    command
}

fn execute(command: Command, end: Instant, active: impl Fn() -> bool) -> Value {
    execute_bounded(command,end,active,CAP)
}

pub(crate) fn execute_bounded(mut command: Command, end: Instant, active: impl Fn() -> bool, cap: usize) -> Value {
    if !active() { return json!({"status":"cancelled","exit_code":null}); }
    if Instant::now() >= end { return json!({"status":"timeout","exit_code":null}); }
    let diagnostic=format!("{:?} {:?}",command.get_program(),command.get_args().collect::<Vec<_>>());
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => { crate::host::log(&diagnostic,&e.to_string()); return json!({"status":if e.kind()==std::io::ErrorKind::NotFound {"missing"} else {"spawn_failed"},"exit_code":null}); },
    };
    let (tx,rx) = mpsc::sync_channel(4);
    fn reader(mut pipe: impl Read + Send + 'static, tx: mpsc::SyncSender<(bool,Result<Vec<u8>,std::io::Error>)>, stderr: bool) {
        std::thread::spawn(move || {
            let mut buffer = [0;1024];
            loop {
                match pipe.read(&mut buffer) {
                    Ok(n) => { if tx.send((stderr,Ok(buffer[..n].to_vec()))).is_err() || n==0 { break; } },
                    Err(e) if e.kind()==std::io::ErrorKind::Interrupted => continue,
                    Err(e) => { let _=tx.send((stderr,Err(e))); break; },
                }
            }
        });
    }
    reader(child.stdout.take().unwrap(),tx.clone(),false);
    reader(child.stderr.take().unwrap(),tx,true);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut received = 0;
    let mut limited = false;
    let mut read_failed = false;
    let mut exit = None;
    let status = loop {
        // Wait for data rather than throttling every four buffered chunks with
        // a fixed sleep. Board snapshots must share the deadline, not lose it
        // to reader-channel backpressure (§5 / #137).
        if let Ok((err,bytes)) = rx.recv_timeout(Duration::from_millis(10)) {
            match bytes {
                Ok(bytes) if bytes.is_empty() => received += 1,
                Ok(bytes) => {
                    let left = cap-stdout.len()-stderr.len();
                    limited |= bytes.len()>left;
                    let output = if err { &mut stderr } else { &mut stdout };
                    output.extend_from_slice(&bytes[..bytes.len().min(left)]);
                },
                Err(_) => { read_failed=true; },
            }
        }
        if read_failed { break "read_failed"; }
        if limited { break "output_limit"; }
        if !active() { break "cancelled"; }
        if Instant::now() >= end { break "timeout"; }
        match child.try_wait() {
            Ok(Some(code)) => { exit=Some(code); if received == 2 { break if code.success() {"success"} else {"exit_nonzero"}; } }
            Ok(None) => {},
            Err(_) => break "wait_failed",
        }
    };
    #[cfg(unix)]
    let _ = crate::host::kill_group(child.id());
    let _ = child.kill();
    let _ = child.wait();
    stdout.truncate(cap);
    stderr.truncate(cap-stdout.len());
    if status != "success" { crate::host::log(&diagnostic,&format!("{status}: {}",String::from_utf8_lossy(&stderr))); }
    json!({"status":status,"exit_code":exit.and_then(|c|c.code()),"stdout":String::from_utf8_lossy(&stdout),"stderr":String::from_utf8_lossy(&stderr)})
}

#[cfg(all(test,unix))]
mod tests {
    use super::*;
    use crate::keys::Keys;
    use std::os::unix::fs::PermissionsExt;

    // §5 / #126 A1–A6: the common boundary must run approved argv only,
    // retain factual failures, and refuse revocation, stale registration and fences.
    #[test]
    fn approved_command_boundary() {
        let dir=std::env::temp_dir().join(format!("pilotq-commands-{}",std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"taskqorg/pilotq"}"#).unwrap();
        let mut journal=Journal::open(&dir.join("journal.sqlite")).unwrap();
        journal.bootstrap(7,0).unwrap();
        journal.add_project(7,"pilotq","taskqorg/pilotq",dir.to_str().unwrap(),0).unwrap();
        let r=journal.ask(7,"pwd",1).unwrap();
        let j=Mutex::new(journal);
        let d=Direct{keys:Keys(dir.join("keys.json"))};
        let entry=|bin:&str,argv:Value|json!({"description":"invented test capability","executable":bin,"cwd":dir,"project":"pilotq","board":"taskqorg/pilotq","users":[7],"args":argv});
        let config=dir.join("commands.json");
        let entries=json!({"pwd":entry("/bin/pwd",json!([[]])),"exit":entry("/bin/false",json!([[]])),"missing":entry("/invented/missing-command",json!([[]])),"sleep":entry("/bin/sleep",json!([["1"]])),"flood":entry("/usr/bin/yes",json!([[]]))});
        std::fs::write(&config,entries.to_string()).unwrap();
        std::fs::set_permissions(&config,std::fs::Permissions::from_mode(0o600)).unwrap();
        let probe = dir.join("taskq-probe");
        std::fs::write(&probe,"#!/bin/sh\nprintf '%s' '{\"complete\":true,\"board\":{\"repo\":\"taskqorg/pilotq\"}}'\n").unwrap();
        std::fs::set_permissions(&probe,std::fs::Permissions::from_mode(0o700)).unwrap();
        let tq=crate::taskq::Taskq{bot:None,bin:probe.to_str().unwrap().into(),root:dir.clone(),poll:Duration::from_secs(1)};
        let call=|name:&str,args:Value,deadline|super::call(&d,&j,&r,name,&args,deadline,Some(&tq));
        let deadline=||Instant::now()+Duration::from_secs(2);
        // #137 / §5: a growing board must not consume the command's 16 KiB output budget.
        let board = json!({"complete":true,"board":{"repo":"taskqorg/pilotq"},"padding":"x".repeat(64 * 1024)});
        std::fs::write(&probe,format!("#!/bin/sh\nprintf '%s' '{}'\n",board)).unwrap();
        let (ok,list)=call("command_list",json!({}),deadline());
        assert!(ok && list.contains("pwd"));
        for (command,args,status) in [("pwd",json!([]),"success"),("exit",json!([]),"exit_nonzero"),("missing",json!([]),"missing"),("flood",json!([]),"output_limit")] {
            let (ok,out)=call("command_run",json!({"command":command,"args":args}),deadline());
            let v:Value=serde_json::from_str(&out).unwrap();
            assert_eq!(v["status"],status,"{out}");
            assert_eq!(ok,status=="success");
            if command=="pwd" { assert_eq!(v["stdout"],format!("{}\n",dir.display())); }
            assert!(v["stdout"].as_str().unwrap_or("").len()+v["stderr"].as_str().unwrap_or("").len()<=CAP);
        }
        let (_,out)=call("command_run",json!({"command":"sleep","args":["1"]}),Instant::now()+Duration::from_millis(50));
        assert!(out.contains("timeout"));
        for args in [json!({"command":"pwd","args":["; touch forbidden"]}),json!({"command":"unknown","args":[]})] {
            assert!(!call("command_run",args,deadline()).0);
        }
        let (_, unknown)=call("command_run",json!({"command":"unknown","args":[]}),deadline());
        assert!(unknown.contains("unknown or unapproved command"));
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"taskqorg/other"}"#).unwrap();
        assert!(!call("command_run",json!({"command":"pwd","args":[]}),deadline()).0);
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"taskqorg/pilotq"}"#).unwrap();
        let saved_probe = std::fs::read(&probe).unwrap();
        // The separate board budget remains bounded and refuses dispatch on overflow.
        std::fs::write(&probe,"#!/bin/sh\nexec /usr/bin/yes\n").unwrap();
        let (ok, overflow)=call("command_run",json!({"command":"pwd","args":[]}),deadline());
        assert!(!ok && overflow.contains("output_limit") && overflow.contains("selected command was not executed"));
        std::fs::write(&probe,"#!/bin/sh\nprintf '%s' '{\"complete\":false}'\n").unwrap();
        assert!(!call("command_run",json!({"command":"pwd","args":[]}),deadline()).0);
        std::fs::write(&probe,saved_probe).unwrap();
        let mut stale = r.clone();
        stale.fence += 1;
        assert!(!super::call(&d,&j,&stale,"command_run",&json!({"command":"pwd","args":[]}),deadline(),Some(&tq)).0);
        let mut sleeping = Command::new("/bin/sleep");
        sleeping.arg("10").stdout(Stdio::piped()).stderr(Stdio::piped());
        std::os::unix::process::CommandExt::process_group(&mut sleeping,0);
        let checks = std::sync::atomic::AtomicUsize::new(0);
        assert_eq!(execute(sleeping,deadline(),||checks.fetch_add(1,std::sync::atomic::Ordering::Relaxed)<2)["status"],"cancelled");
        let mut denied = entries.clone();
        denied["pwd"]["users"] = json!([]);
        std::fs::write(&config,denied.to_string()).unwrap();
        let (_, refusal)=call("command_run",json!({"command":"pwd","args":[]}),deadline());
        assert_eq!(refusal,"command permission denied");
        assert!(!call("command_run",json!({"command":"pwd","args":[]}),deadline()).0);
        std::fs::write(&config,entries.to_string()).unwrap();
        j.lock().unwrap().revoke(7,7,&["projects".into()],2).unwrap();
        assert!(!call("command_run",json!({"command":"pwd","args":[]}),deadline()).0);
        std::fs::set_permissions(&config,std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!call("command_list",json!({}),deadline()).0);
        drop(j);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
