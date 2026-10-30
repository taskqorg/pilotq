//! Invented oldest-supported tools for §9 setup/update checks; no host dependency or login.
use std::{fs,path::Path,os::unix::fs::PermissionsExt};
use serde_json::{Value,json};
fn script(p:&Path,body:&str) { fs::write(p,format!("#!/bin/sh\n{body}\n")).unwrap(); fs::set_permissions(p,fs::Permissions::from_mode(0o700)).unwrap(); }
pub fn wrap(p:&Path,args:&str,reply:&str) {
    let real=p.with_extension("real");
    if fs::read_to_string(p).is_ok_and(|s|s.contains(&format!("exec '{}'",real.display()))) { return; }
    if p.exists() { fs::rename(p,&real).unwrap(); }
    script(p,&format!("if [ \"$1\" = {args} ]; then echo '{reply}'; exit 0; fi\nexec '{}' \"$@\"",real.display()));
}
pub fn provision(d:&Path,config:&Path) {
    fs::create_dir_all(d).unwrap();
    let mut c:Value=fs::read_to_string(config).ok().map(|s|serde_json::from_str(&s).unwrap()).unwrap_or(json!({"bot_token":"123:invented","owner_id":7}));
    for (name,reply) in [("gh","gh version 2.45.0 (invented)"),("curl","curl 7.68.0"),("systemctl","systemd 245"),("kill","kill from procps-ng 3.3.17"),("hostname","hostname 3.23")] {
        let p=d.join(name);
        if p.exists() { wrap(&p,"--version",reply); } else { script(&p,&format!("echo '{reply}'")); }
    }
    for (key,name,reply) in [("codex","codex","codex-cli 0.162.0"),("recognizer","recognizer","2.0.1"),("ffmpeg","ffmpeg","ffmpeg version 6.1.0")] {
        let current=if key=="codex" { c[key].as_str() } else { c["voice"][key].as_str() };
        let p=current.map(std::path::PathBuf::from).unwrap_or(d.join(name));
        if p.exists() { wrap(&p,if key=="ffmpeg" {"-version"} else {"--version"},reply); }
        else { script(&p,&format!("echo '{reply}'")); }
        if key=="codex" { c[key]=json!(p); } else { c["voice"][key]=json!(p); }
    }
    if c["taskq"]["bin"].is_null() {
        let p=d.join("taskq");script(&p,"echo '{\"service\":{\"version\":\"4.6.40\"}}'");c["taskq"]=json!({"bin":p,"root":d});
    } else { wrap(Path::new(c["taskq"]["bin"].as_str().unwrap()),"service",r#"{"service":{"version":"4.6.40"}}"#); }
    let taskq=Path::new(c["taskq"]["bin"].as_str().unwrap());
    let sum=std::process::Command::new("sh").args(["-c","sha256sum \"$1\" 2>/dev/null || shasum -a 256 \"$1\"","receipt",taskq.to_str().unwrap()]).output().unwrap();
    let digest=String::from_utf8(sum.stdout).unwrap().split_whitespace().next().unwrap().to_string();
    fs::write(format!("{}.pilotq-version.json",taskq.display()),json!({"version":"4.6.43","sha256":digest}).to_string()).unwrap();
    if c["voice"]["model"].is_null() { c["voice"]["model"]=json!(d); }
    fs::write(config,c.to_string()).unwrap();fs::set_permissions(config,fs::Permissions::from_mode(0o600)).unwrap();
}
