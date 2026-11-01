//! §7 / #265: one TaskQ host slot per manager turn through `taskq service slots` (taskq.slots/1). TaskQ owns capacity; PilotQ
//! persists each binding and its release key before sending, starts the runtime only on a fresh `reserved` receipt and releases
//! only after the whole turn, its runtime process group included, has ended.
use crate::journal::{Journal, Run};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone)]
pub struct Slots {
    pub bin: PathBuf,
    /// A registered TaskQ project root that all manager turns are counted under.
    // ponytail: one root for every conversation; the conversation's project root if per-project round-robin matters
    pub root: PathBuf,
}

/// The long-lived PilotQ process identity TaskQ binds every slot to; read once per process, retried after a failed read.
static OWNER: Mutex<Option<Value>> = Mutex::new(None);

impl Slots {
    fn exchange(&self, args: &[&str]) -> Result<Value, String> {
        let mut c = Command::new(&self.bin);
        c.args(args).env_remove("TASKQ_ATTEMPT");
        let o = crate::host::output_with_budget(&mut c, Duration::from_secs(20)).map_err(|_| "TaskQ host slot call failed".to_string())?;
        serde_json::from_slice(&o.stdout).map_err(|_| "TaskQ host slot reply unreadable".into())
    }

    fn call(&self, action: &str, request: &Value) -> Result<Value, String> {
        self.exchange(&["service", "slots", "--action", action, "--request", &request.to_string(), "--json"])
    }

    /// Waits for admission until the request's deadline or a fence. The binding is durable before the first send, so a lost
    /// reply is reconciled with the same identity, never with a new one.
    pub fn reserve(&self, j: &Mutex<Journal>, r: &Run, runtime: &str, now: i64, mut alive: impl FnMut() -> bool) -> Result<Value, String> {
        let owner = {
            let mut owner = OWNER.lock().unwrap_or_else(|e| e.into_inner());
            if owner.is_none() { *owner = Some(self.exchange(&["service", "slots", "--identity", &std::process::id().to_string(), "--json"])?); }
            owner.clone().unwrap()
        };
        let hex = || coreq::agent::new_session(coreq::agent::Client::Claude).replace('-', "");
        let request = json!({"binding": {"schema": "taskq.slots/1", "caller": "pilotq", "op_id": format!("pilotq-{}", hex()),
            "root": self.root, "runtime": runtime, "pid": owner["pid"], "started": owner["started"], "boot": owner["boot"],
            "expires_at": now + 300}, "key": format!("{}{}", hex(), hex())});
        j.lock().unwrap().set_slot(r.id, Some(&json!({"request": request, "bin": self.bin})))?;
        let end = std::time::Instant::now() + Duration::from_secs(300);
        let mut action = "reserve";
        loop {
            let reply = self.call(action, &request);
            action = "observe";
            match reply.as_ref().map(|v| v["state"].as_str().unwrap_or("unknown")) {
                Ok("reserved") => return Ok(request),
                Ok("denied") | Ok("released") => {
                    let why = reply.as_ref().ok().and_then(|v| v["reason"].as_str()).unwrap_or("no reason given").to_string();
                    release(j, r.id, &request, &self.bin)?;
                    return Err(format!("No host slot for this turn: {why}"));
                }
                _ => {}
            }
            if !alive() || std::time::Instant::now() >= end {
                // Release before admission cancels the request, also when its admission reply was lost.
                release(j, r.id, &request, &self.bin)?;
                return Err(if alive() { "No host slot became free within 300 s; nothing was started.".into() } else { "fenced".into() });
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

/// Attests that the turn ended, then waits for TaskQ's `released` readback before forgetting the binding.
pub fn release(j: &Mutex<Journal>, run: i64, request: &Value, bin: &std::path::Path) -> Result<(), String> {
    let s = Slots { bin: bin.into(), root: PathBuf::new() };
    for _ in 0..30 {
        if s.call("release", request).is_ok_and(|v| v["state"] == "released" || v["state"] == "denied") {
            return j.lock().unwrap().set_slot(run, None);
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err("TaskQ host slot release unconfirmed; the binding is kept for the next start".into())
}

/// Startup: a slot left by an earlier process is released only after its recorded runtime group is gone. An unreadable
/// identity keeps the slot, so occupancy stays unknown rather than free.
pub fn recover(j: &Mutex<Journal>) {
    let left = j.lock().unwrap().slots().unwrap_or_default();
    for (run, v) in left {
        let pid = v["pid"].as_u64().map(|p| p as u32);
        if let Some(pid) = pid {
            match coreq::process::identity(pid) {
                Ok(Some(id)) if Some(id.started.as_str()) == v["started"].as_str() => {
                    #[cfg(unix)]
                    let _ = crate::host::kill_group(pid);
                    std::thread::sleep(Duration::from_secs(1));
                    if coreq::process::identity(pid).map(|p| p.is_some_and(|p| Some(p.started.as_str()) == v["started"].as_str())).unwrap_or(true) {
                        eprintln!("slot of run {run}: runtime still running; slot kept");
                        continue;
                    }
                }
                Ok(_) => {}
                Err(_) => { eprintln!("slot of run {run}: runtime liveness unknown; slot kept"); continue; }
            }
        }
        if let Some(bin) = v["bin"].as_str()
            && let Err(e) = release(j, run, &v["request"], std::path::Path::new(bin)) { eprintln!("slot of run {run}: {e}"); }
    }
}

/// Records the runtime's process group with its slot, so a later start can prove it ended.
pub fn spawned(j: &Mutex<Journal>, run: i64, pid: u32) {
    let guard = j.lock().unwrap();
    let Ok(Some(mut v)) = guard.slot(run) else { return };
    v["pid"] = json!(pid);
    v["started"] = json!(coreq::process::identity(pid).ok().flatten().map(|p| p.started));
    if let Err(e) = guard.set_slot(run, Some(&v)) { eprintln!("slot of run {run}: {e}"); }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    // #265 A5: a turn is admitted only on `reserved`, its binding is durable until TaskQ reads back `released`, and a
    // restart releases a left slot whose runtime is gone. The fake records every request it receives.
    #[test]
    fn slot_lifecycle_265() {
        let dir = std::env::temp_dir().join(format!("pilotq-slots265-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("taskq");
        std::fs::write(&bin, format!(r#"#!/bin/sh
echo "$@" >> {log}
case "$3" in
  --identity) echo '{{"schema":"taskq.slots/1","pid":'$4',"started":"invented","boot":1}}';;
  *) case "$4" in reserve) echo '{{"state":"queued"}}';; observe) echo '{{"state":"reserved"}}';; release) echo '{{"state":"released"}}';; esac;;
esac
"#, log = dir.join("log").display())).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut journal = Journal::open(std::path::Path::new(":memory:")).unwrap();
        journal.bootstrap(7, 0).unwrap();
        let j = Mutex::new(journal);
        let r = j.lock().unwrap().ask(7, "invented", 1).unwrap();
        let s = Slots { bin: bin.clone(), root: "/invented/root".into() };
        let request = s.reserve(&j, &r, "codex", 1, || true).unwrap();
        assert_eq!(request["binding"]["root"], "/invented/root");
        assert_eq!(request["key"].as_str().unwrap().len(), 64);
        assert!(j.lock().unwrap().slot(r.id).unwrap().is_some(), "binding must stay durable while the turn runs");
        release(&j, r.id, &request, &bin).unwrap();
        assert!(j.lock().unwrap().slot(r.id).unwrap().is_none());
        let log = std::fs::read_to_string(dir.join("log")).unwrap();
        let actions: Vec<_> = log.lines().filter_map(|l| l.split(' ').nth(3)).collect();
        assert_eq!(actions, [std::process::id().to_string().as_str(), "reserve", "observe", "release"], "identity once, then one reserve");
        // a slot left by a crashed process whose runtime is gone is released at the next start
        let left = j.lock().unwrap().ask(7, "invented left", 2).unwrap();
        j.lock().unwrap().set_slot(left.id, Some(&json!({"request": request, "bin": bin, "pid": 999_999_999u32, "started": "gone"}))).unwrap();
        recover(&j);
        assert!(j.lock().unwrap().slots().unwrap().is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
