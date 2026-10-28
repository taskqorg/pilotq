//! TaskQ CLI adapter and the manager's TaskQ tools (pilotq.md § 5 First TaskQ tool contract): registered boards, argument arrays,
//! JSON receipts, grants and fence checked at dispatch, and the scan that projects board state into Telegram.

use crate::journal::{Answer, Journal, PlanRun, Run, describe};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

/// The only board for direct issue writes; TaskQ tools use the registered target.
pub const BOARD: &str = "taskqorg/pilotq";

/// A plan run's lease while one caller files it; the scan takes over a run whose caller died.
const LEASE: i64 = 120;
/// Plans hold one to three tickets (U15); a larger plan is split before filing.
const TICKETS: usize = 3;

pub struct Taskq {
    pub bin: PathBuf,
    /// The project checkout: `TASKQ_ROOT` and the `taskq.json` that names the board.
    pub root: PathBuf,
    /// Board scan interval (§ 7 task notifications).
    pub poll: Duration,
}

/// What the manager may call (§ 5 table). Codex shows the model only these; PilotQ checks each call again at dispatch.
pub fn tools() -> Value {
    let ticket = json!({"type": "object", "properties": {
        "title": {"type": "string", "description": "Imperative task title, at most 80 characters"},
        "goal": {"type": "string", "description": "The user outcome, one or two sentences"},
        "acceptance": {"type": "array", "items": {"type": "string"}, "description": "1-8 observable checks, one line each"},
        "scope": {"type": "array", "items": {"type": "string"}, "description": "Optional boundaries, at most 3 lines"}},
        "required": ["title", "goal", "acceptance"]});
    let task = json!({"type": "string", "description": "Task number or registered GitHub board issue URL"});
    let tool = |name: &str, description: &str, schema: Value| json!({"type": "function", "name": name, "description": description, "inputSchema": schema});
    json!([
        tool("task_plan", "Save the next plan revision for this conversation (1-3 tickets). Files nothing. Show the person the revision number and tickets.",
            json!({"type": "object", "properties": {"project": {"type": "string"}, "tickets": {"type": "array", "items": ticket}}, "required": ["tickets"]})),
        tool("task_run_plan", "File the named plan revision on the saved registered board held, read it back and start it. Only when the person, in a later message, asks to run that revision.",
            json!({"type": "object", "properties": {"project": {"type": "string"}, "revision": {"type": "integer"}}, "required": ["revision"]})),
        tool("task_status", "Read one registered task's current state from the board. Never creates anything.", json!({"type": "object", "properties": {"project": {"type": "string"}, "task": task}, "required": ["task"]})),
        tool("task_park", "Park one registered task (TaskQ later) with the person's reason.",
            json!({"type": "object", "properties": {"project": {"type": "string"}, "task": task, "reason": {"type": "string"}}, "required": ["task", "reason"]})),
        tool("task_steer", "Send the person's instructions to a doing task at its current revision.",
            json!({"type": "object", "properties": {"project": {"type": "string"}, "task": task, "text": {"type": "string"}}, "required": ["task", "text"]})),
        tool("task_requeue", "Requeue an open task with the person's reason. Does not prove worker termination.",
            json!({"type": "object", "properties": {"project": {"type": "string"}, "task": task, "reason": {"type": "string"}}, "required": ["task", "reason"]})),
        tool("queue_pause", "Pause new TaskQ admission on this host. Running workers continue.", json!({"type": "object", "properties": {}})),
    ])
}

/// Model instructions for the TaskQ tools; the boundary enforces them regardless.
pub const INSTRUCTIONS: &str = "You are PilotQ, the conversational manager for registered TaskQ boards. Resolve one authorized project; never guess an ambiguous task number. Discussion never files work. To propose work, \
call task_plan and show the person the saved revision number and its tickets. Call task_run_plan only when the person, in a later message, asks to run a \
named revision. Never say a task exists, started or finished unless a tool result in this conversation says so; cite its URL. You cannot answer a worker's \
question: the person replies to the relayed question message. For task state call task_status; never guess it. Pausing the queue does not stop a running \
worker; say so.";

fn url(board: &str, task: i64) -> String {
    format!("https://github.com/{board}/issues/{task}")
}

impl Taskq {
    /// Registered board identity and the create-claim prerequisite from the root's `taskq.json`.
    fn check(&self, board: &str) -> Result<bool, String> {
        let p = self.root.join("taskq.json");
        let v: Value = std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str(&t).ok()).ok_or_else(|| format!("TaskQ is unavailable: {} is unreadable", p.display()))?;
        if v["repo"] != board {
            return Err(format!("TaskQ is disabled: the configured root names board {}, not {board}", v["repo"]));
        }
        Ok(v["create_claim_issue"].as_i64().is_some_and(|n| n > 0))
    }

    /// Runs the CLI with an argument array; Telegram text is never interpolated into a shell. Returns stdout, else the error.
    fn cli(&self, args: &[String]) -> Result<String, String> {
        let o = Command::new(&self.bin)
            .args(args)
            .env("TASKQ_ROOT", &self.root)
            .env_remove("TASKQ_ATTEMPT")
            .env_remove("TASKQ_TASK")
            .env_remove("TASKQ_ROLE")
            .current_dir(&self.root)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("taskq did not start: {e}"))?;
        let out = String::from_utf8_lossy(&o.stdout).to_string();
        if o.status.success() || out.trim_start().starts_with('{') {
            return Ok(out);
        }
        let err = String::from_utf8_lossy(&o.stderr);
        let why = err.lines().chain(out.lines()).find(|l| !l.trim().is_empty()).unwrap_or("no output");
        Err(format!("taskq {} failed: {why}", args.first().map_or("", String::as_str)))
    }

    /// One write with `--op-id` and `--json`: its receipt `{task, op_id, outcome, state, revision, reason, next}`.
    fn write(&self, mut args: Vec<String>, op: &str) -> Result<Value, String> {
        args.extend(["--op-id".into(), op.into(), "--json".into()]);
        let out = self.cli(&args)?;
        serde_json::from_str(out.trim()).map_err(|_| format!("taskq {} returned no receipt", args[0]))
    }

    /// The board a fresh, complete `status --json` reads under this root (§ 5 Project registration readback).
    pub fn board(&self) -> Result<String, String> {
        self.snapshot()?["board"]["repo"].as_str().map(str::to_string).ok_or("taskq status names no board".into())
    }

    /// `status --json`; an incomplete snapshot is unknown, never a list of closed tasks.
    fn snapshot(&self) -> Result<Value, String> {
        let s: Value = serde_json::from_str(self.cli(&["status".into(), "--json".into()])?.trim()).map_err(|_| "taskq status returned no snapshot".to_string())?;
        if s["complete"] != true {
            return Err(format!("board snapshot incomplete: {}", s["reason"].as_str().unwrap_or("no reason given")));
        }
        let board = s["board"]["repo"].as_str().ok_or("snapshot names no board")?;
        self.check(board)?;
        Ok(s)
    }

    fn snapshot_for(&self, board: &str) -> Result<Value, String> {
        let s = self.snapshot()?;
        if s["board"]["repo"] != board { return Err("registered board mismatch; nothing was done".into()); }
        Ok(s)
    }

    fn registered(&self, j: &Journal, project: &str) -> Result<(Self, String), String> {
        let (_, board, root, ..) = j.projects()?.into_iter().find(|p| p.0 == project).ok_or(format!("{project} is not a registered project; nothing was done"))?;
        let tq = Self { bin: self.bin.clone(), root: root.into(), poll: self.poll };
        tq.check(&board)?;
        Ok((tq, board))
    }
}


fn find(s: &Value, task: i64) -> Option<&Value> {
    s["tasks"].as_array()?.iter().find(|t| t["n"] == task)
}

// Task messages render as Telegram HTML with a labeled link, never a bare URL (#82).
fn status_text(board: &str, task: i64, state: &str) -> String {
    format!("Task #{task} {}.\n[Open the task]({})", describe(state), url(board, task))
}

/// The card without its CLI `To answer:` line: in Telegram an option button or a reply answers (#82).
fn card(c: &str) -> String {
    c.lines().filter(|l| !l.starts_with("To answer")).collect::<Vec<_>>().join("\n")
}

fn question_text(board: &str, task: i64, c: &str) -> String {
    format!("Task #{task} asks:\n{}\nPress an option or reply to this message to answer.\n[Open the task]({})", card(c), url(board, task))
}

fn now() -> i64 {
    crate::now() as i64
}

/// Checks the caller's grants at dispatch (§ 8): project scope, plus `taskq` for a write.
fn allowed(j: &Journal, user: i64, project: &str, write: bool) -> Result<(), String> {
    let grant = format!("project:{project}");
    if !(j.has(user, &grant)? || j.has(user, "projects")?) {
        return Err(format!("{user} has no {grant} grant; nothing was done"));
    }
    if write && !j.has(user, "taskq")? {
        return Err(format!("{user} has no taskq grant; nothing was written"));
    }
    Ok(())
}

/// Validates a `task_plan` argument against TaskQ's `add` limits before anything is saved.
fn tickets(args: &Value) -> Result<Value, String> {
    let t = args["tickets"].as_array().filter(|t| (1..=TICKETS).contains(&t.len())).ok_or(format!("a plan holds 1 to {TICKETS} tickets"))?;
    let line = |s: &str| !s.trim().is_empty() && !s.contains('\n') && s.chars().count() <= 240;
    let lines = |v: &Value, min: usize, max: usize| v.as_array().is_some_and(|a| (min..=max).contains(&a.len()) && a.iter().all(|x| x.as_str().is_some_and(line)));
    for (i, x) in t.iter().enumerate() {
        let ok = x["title"].as_str().is_some_and(|s| line(s) && s.chars().count() <= 80)
            && x["goal"].as_str().is_some_and(|s| !s.trim().is_empty() && s.lines().count() <= 2 && s.lines().all(line))
            && lines(&x["acceptance"], 1, 8)
            && (x["scope"].is_null() || lines(&x["scope"], 0, 3));
        if !ok {
            return Err(format!("ticket {}: title <= 80 characters, goal 1-2 lines, 1-8 acceptance lines and at most 3 scope lines, each <= 240 characters", i + 1));
        }
    }
    Ok(Value::Array(t.clone()))
}

/// One manager tool call (§ 5). Returns the result for the model and whether it succeeded; a refusal writes nothing.
pub fn call(tq: &Taskq, j: &Mutex<Journal>, r: &Run, name: &str, args: &Value) -> (bool, String) {
    match dispatch(tq, j, r, name, args) {
        Ok(text) => (true, text),
        Err(e) => (false, e),
    }
}

fn dispatch(tq: &Taskq, j: &Mutex<Journal>, r: &Run, name: &str, args: &Value) -> Result<String, String> {
    let sender = {
        let j = j.lock().unwrap();
        // a stopped or replaced run emits no authorized tool call (§ 4 Runs)
        if j.fenced(r)? {
            return Err("this run was stopped; nothing was done".into());
        }
        j.sender(r.request)?
    };
    // queue pause is host-scoped; other tools resolve the same target rules as issue tools.
    if name == "queue_pause" {
        let guard = j.lock().unwrap();
        if !guard.has(sender, "queue")? { return Err(format!("{sender} has no queue grant; nothing was paused")); }
        drop(guard);
        tq.cli(&["service".into(), "pause".into()])?;
        j.lock().unwrap().task_event(r.request, "service pause", "accepted", now())?;
        let running = tq.snapshot().map(|s| s["tasks"].as_array().into_iter().flatten().filter(|t| t["state"] == "doing").map(|t| format!("#{}", t["n"])).collect::<Vec<_>>().join(", ")).unwrap_or("unknown".into());
        return Ok(format!("TaskQ admission is paused on this host; no new task starts. Running workers continue. Still running: {running}."));
    }
    let saved = if name == "task_run_plan" {
        let rev = args["revision"].as_i64().ok_or("revision is required")?;
        Some(j.lock().unwrap().plan_target(&r.session, rev)?)
    } else { None };
    let (project, board, n) = {
        let j = j.lock().unwrap();
        let mut target = args.clone();
        target["issue"] = args.get("task").cloned().unwrap_or(if name == "task_plan" || name == "task_run_plan" { json!("1") } else { Value::Null });
        if let Some((project, _, _)) = &saved {
            if args["project"].as_str().is_some_and(|p| p != project) { return Err("saved plan cannot be retargeted; nothing was filed".into()); }
            allowed(&j, sender, project, true)?;
            (project.clone(), saved.as_ref().unwrap().1.clone(), 0)
        } else { crate::direct::target(&j, r, sender, &target)? }
    };
    let (bound, registered_board) = tq.registered(&j.lock().unwrap(), &project)?;
    let tq = &bound;
    if saved.as_ref().is_some_and(|(_, _, root)| registered_board != board || (!root.is_empty() && PathBuf::from(root) != tq.root)) {
        return Err("saved plan registration changed; nothing was filed".into());
    }
    let create = tq.check(&board)?;
    match name {
        "task_plan" => {
            allowed(&j.lock().unwrap(), sender, &project, false)?;
            let t = tickets(args)?;
            let rev = j.lock().unwrap().save_plan(r, &t, &project, &board, &tq.root.to_string_lossy(), now())?;
            let list: Vec<String> = t.as_array().unwrap().iter().enumerate().map(|(i, x)| format!("{}. {}", i + 1, x["title"].as_str().unwrap_or(""))).collect();
            Ok(format!("Plan revision {rev} saved; nothing is filed. Tickets:\n{}\nShow this revision; the person runs it by naming revision {rev} in a later message.", list.join("\n")))
        }
        "task_run_plan" => {
            let rev = args["revision"].as_i64().ok_or("revision is required")?;
            allowed(&j.lock().unwrap(), sender, &project, true)?;
            if !create {
                return Err("task creation is disabled: create_claim_issue is not configured for the resolved board, and plain add is never used. Nothing was filed.".into());
            }
            let run = j.lock().unwrap().authorize(r, rev, sender, LEASE, now())?;
            Ok(submit(tq, j, run, false))
        }
        "task_status" => {
            allowed(&j.lock().unwrap(), sender, &project, false)?;
            let s = tq.snapshot_for(&board)?;
            let text = match find(&s, n) {
                Some(t) => {
                    let mut text = format!("Task #{n} {} (revision {}, observed {}).\n{}", describe(t["state"].as_str().unwrap_or("")), t["revision"], s["observed"], url(&board, n));
                    for e in t["executions"].as_array().into_iter().flatten() {
                        text += &format!("\nAttempt {} ({}, {}): {}", e["attempt"], e["role"].as_str().unwrap_or(""), e["state"].as_str().unwrap_or(""), e["reason"].as_str().unwrap_or(""));
                    }
                    if let Some(c) = t["card"].as_str() {
                        text += &format!("\nOpen question:\n{}", card(c));
                    }
                    text
                }
                None => format!("Task #{n} is not open on {board} (closed, or not a task). {}", url(&board, n)),
            };
            j.lock().unwrap().task_event(r.request, &format!("{board} status #{n}"), "read", now())?;
            Ok(text)
        }
        "task_park" | "task_steer" | "task_requeue" => {
            allowed(&j.lock().unwrap(), sender, &project, true)?;
            let (cmd, field) = match name { "task_steer" => ("steer", "text"), "task_requeue" => ("requeue", "reason"), _ => ("later", "reason") };
            let value = args[field].as_str().filter(|s| !s.trim().is_empty()).ok_or(format!("{field} is required"))?;
            let text = if cmd == "steer" { value.to_string() } else { format!("Reason: {value}") };
            let limit = if cmd == "steer" { 4 } else { 1 };
            if text.lines().count() > limit || text.lines().any(|s| s.trim().is_empty() || s.chars().count() > 240) {
                return Err(format!("{field} must be 1-{limit} nonempty lines of at most 240 characters including the Reason prefix; nothing was written"));
            }
            let snapshot = tq.snapshot_for(&board)?;
            let task = find(&snapshot, n).ok_or("task is not open; nothing was written")?;
            if cmd == "steer" && task["state"] != "doing" { return Err("only a doing task can be steered; nothing was written".into()); }
            let revision = task["revision"].as_i64().ok_or("task revision is unknown; nothing was written")?;
            let argv = vec![cmd.into(), n.to_string(), "--text".into(), text, "--revision".into(), revision.to_string()];
            let op = {
                let j = j.lock().unwrap();
                if j.fenced(r)? { return Err("this run was stopped; nothing was written".into()); }
                allowed(&j, sender, &project, true)?;
                j.task_intent(r, &project, &board, &tq.root.to_string_lossy(), &argv, now())?
            };
            reconcile_effect(tq, j, &op, &board, &argv, false)
        }
        _ => Err(format!("unknown tool {name}")),
    }
}

fn reconcile_effect(tq: &Taskq, j: &Mutex<Journal>, op: &str, board: &str, args: &[String], notify: bool) -> Result<String, String> {
    let rc = match tq.write(args.to_vec(), op) {
        Ok(rc) => rc,
        Err(e) => return Err(format!("TaskQ effect {op} is uncertain ({e}); recovery reconciles the same operation ID")),
    };
    if !matches!(rc["outcome"].as_str(), Some("accepted" | "stale" | "rejected")) {
        return Err(format!("TaskQ effect {op} is uncertain; no verified readback"));
    }
    let n = args[1].parse::<i64>().unwrap_or(0);
    if rc["op_id"] != op || rc["task"] != n || (rc["outcome"] == "accepted" && (!rc["revision"].as_i64().is_some_and(|n| n > 0) || rc["state"].as_str().is_none())) {
        return Err(format!("TaskQ effect {op} is uncertain; receipt identity or revision is invalid"));
    }
    let message = format!("TaskQ effect {op} reconciled: {} {} at revision {}. {}", rc["outcome"], rc["state"], rc["revision"], url(board, n));
    j.lock().unwrap().task_receipt(op, &rc, notify.then_some(message.as_str()), now())?;
    if rc["outcome"] != "accepted" { return Err(format!("Task #{n} {}: {}. {}", rc["outcome"], rc["reason"].as_str().unwrap_or("no reason given"), url(board,n))); }
    Ok(format!("Task #{n}: {} accepted; TaskQ readback {} at revision {}. Worker termination is not implied. {}", args[0], rc["state"].as_str().unwrap_or("unknown"), rc["revision"], url(board,n)))
}

fn recover_effects(tq: &Taskq, j: &Mutex<Journal>) -> bool {
    let mut recovered = false;
    let pending = j.lock().unwrap().pending_task_effects().unwrap_or_default();
    for (op, request, project, board, root, args) in pending {
        let actor = j.lock().unwrap().sender(request).unwrap_or(0);
        if allowed(&j.lock().unwrap(), actor, &project, true).is_err() { continue; }
        let Ok((bound, registered)) = tq.registered(&j.lock().unwrap(), &project) else { continue };
        if registered != board || bound.root != PathBuf::from(root) { continue; }
        if bound.snapshot_for(&board).is_err() { continue; }
        let _ = reconcile_effect(&bound, j, &op, &board, &args, true);
        recovered = true;
    }
    recovered
}

/// Files a plan run's remaining tickets held with their op IDs, then starts them once every ticket read back held (§ 5, U15).
/// An unavailable board defers the rest with its reason; the scan retries the same op IDs. `notify` tells the route.
pub fn submit(tq: &Taskq, j: &Mutex<Journal>, id: i64, notify: bool) -> String {
    let at = now();
    let run = match j.lock().unwrap().plan_run(id) {
        Ok(r) => r,
        Err(e) => return format!("plan run {id}: {e}"),
    };
    let end = |run: &PlanRun, state: &str, reason: Option<&str>, text: String| {
        let next = if state == "creating" { at + tq.poll.as_secs() as i64 } else { at };
        if let Err(e) = j.lock().unwrap().run_outcome(run, state, reason, notify.then_some(text.as_str()), next, at) {
            eprintln!("plan run {id}: {e}");
        }
        text
    };
    let resolved = tq.registered(&j.lock().unwrap(), &run.project);
    let (bound, board) = match resolved {
        Ok((t, b)) if b == run.board && (run.root.is_empty() || t.root == PathBuf::from(&run.root)) => (t, b),
        _ => return end(&run, "failed", Some("registration changed or unavailable"), "Plan submission blocked: registration changed or unavailable".into()),
    };
    let tq = &bound;
    // grants are rechecked before every retry (§ 5)
    let ok = allowed(&j.lock().unwrap(), run.actor, &run.project, true);
    if let Err(e) = ok {
        return end(&run, "failed", Some(&e), format!("Plan revision {} was not filed: {e}.", run.revision));
    }
    let filed = |run: &PlanRun| run.tickets.iter().filter_map(|t| t.task).map(|n| format!("#{n} {}", url(&board, n))).collect::<Vec<_>>().join(", ");
    let mut run = run;
    if let Err(e) = tq.snapshot_for(&board) {
        return end(&run, "creating", Some(&e), format!("Plan revision {}: ticket 1 is unsent ({e}). Filed so far (held, not started): {}. PilotQ retries with the same operation IDs.", run.revision, or_none(filed(&run))));
    }
    for i in 0..run.tickets.len() {
        let t = run.tickets[i].clone();
        if t.task.is_some() {
            continue;
        }
        if let Err(e) = allowed(&j.lock().unwrap(), run.actor, &run.project, true).and_then(|_| tq.check(&board).and_then(|ok| if ok { Ok(()) } else { Err("create_claim_issue is not configured".into()) })) {
            return end(&run, "failed", Some(&e), e.clone());
        }
        let p = &t.plan;
        let mut args = vec!["add".to_string(), p["title"].as_str().unwrap_or("").to_string(), "--goal".into(), p["goal"].as_str().unwrap_or("").to_string()];
        for a in p["acceptance"].as_array().into_iter().flatten() {
            args.extend(["--acceptance".into(), a.as_str().unwrap_or("").to_string()]);
        }
        for s in p["scope"].as_array().into_iter().flatten() {
            args.extend(["--scope".into(), s.as_str().unwrap_or("").to_string()]);
        }
        // the real author, route, request and approved revision travel with the task (§ 5)
        let route = if run.chat == run.actor { format!("DM {}", run.chat) } else { format!("chat {}{}", run.chat, run.topic.map_or(String::new(), |t| format!(" topic {t}"))) };
        args.extend(["--scope".into(), format!("Filed by PilotQ {} for Telegram user {} in {route}, plan revision {}.", crate::journal::trace_id(run.request), run.actor, run.revision)]);
        let rc = tq.write(args, &t.op_id);
        match rc.as_ref().map(|r| (r["outcome"].as_str(), r["state"].as_str(), r["task"].as_i64())) {
            Ok((Some("accepted"), Some("held"), Some(n))) => {
                let rev = rc.as_ref().unwrap()["revision"].as_i64().unwrap_or(0);
                let r = j.lock().unwrap().filed(&run, t.idx, n, rev, now());
                if let Err(e) = r {
                    return end(&run, "creating", Some(&e), format!("Task #{n} is held but PilotQ could not record it: {e}"));
                }
                run.tickets[i].task = Some(n);
            }
            Ok((Some("rejected"), ..)) => {
                let why = rc.as_ref().unwrap()["reason"].as_str().unwrap_or("rejected").to_string();
                return end(&run, "failed", Some(&why), format!("Plan revision {}: TaskQ rejected ticket {}: {why}. Filed so far (held): {}.", run.revision, t.idx, or_none(filed(&run))));
            }
            _ => {
                let why = match &rc {
                    Err(e) => e.clone(),
                    Ok(r) => format!("no held readback: {r}"),
                };
                return end(&run, "creating", Some(&why), format!("Plan revision {}: ticket {} is unsent ({why}). Filed so far (held, not started): {}. PilotQ retries with the same operation IDs.", run.revision, t.idx, or_none(filed(&run))));
            }
        }
    }
    // start only after every ticket of the revision read back held
    for t in run.tickets.clone().iter().filter(|t| !t.started) {
        if let Err(e) = allowed(&j.lock().unwrap(), run.actor, &run.project, true) { return end(&run, "failed", Some(&e), e.clone()); }
        let n = t.task.unwrap_or_default();
        let rc = tq.write(vec!["start".into(), n.to_string()], &format!("{}.start", t.op_id));
        match rc.as_ref().map(|r| (r["outcome"].as_str(), r["state"].as_str())) {
            Ok((Some("accepted"), Some(state))) => {
                if let Err(e) = j.lock().unwrap().started(&run, t.idx, n, state, now()) {
                    eprintln!("plan run {id}: {e}");
                }
            }
            _ => {
                let why = match &rc {
                    Err(e) => e.clone(),
                    Ok(r) => format!("no start readback: {r}"),
                };
                return end(&run, "creating", Some(&why), format!("Plan revision {}: every ticket is held ({}); starting #{n} is pending ({why}). PilotQ retries.", run.revision, filed(&run)));
            }
        }
    }
    end(&run, "started", None, format!("Plan revision {}: filed held, read back and started: {}.", run.revision, filed(&run)))
}

fn or_none(s: String) -> String {
    if s.is_empty() { "none".into() } else { s }
}

/// Checks and writes one answer (§ 5): the replier needs `taskq` and project scope, and the question must be the current one.
/// A refusal writes nothing and says why; a lost write is uncertain and never repeated with a new op ID.
fn answer(tq: &Taskq, j: &Mutex<Journal>, a: &Answer, resumed: bool) {
    let at = now();
    let end = |state: &str, reply: String, error: Option<&str>| {
        if let Err(e) = j.lock().unwrap().end_answer(a, state, &reply, error, at) {
            eprintln!("answer {}: {e}", a.request);
        }
    };
    let n = a.task;
    let (bound, board) = match tq.registered(&j.lock().unwrap(), &a.project) {
        Ok(x) => x, Err(e) => return end("failed", e, Some("registered board unavailable")),
    };
    let tq = &bound;
    if !j.lock().unwrap().task_target_matches(&a.project, n, &board, &tq.root.to_string_lossy()).unwrap_or(false) {
        return end("failed", "Task registration changed; nothing was written".into(), Some("registration changed"));
    }
    let ok = allowed(&j.lock().unwrap(), a.sender, &a.project, true);
    if let Err(e) = ok {
        return end("failed", format!("Not written to task #{n}: {e}."), Some("not authorized"));
    }
    let lines: Vec<&str> = a.text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() || lines.len() > 4 || lines.iter().any(|l| l.chars().count() > 240) {
        return end("failed", format!("Not written to task #{n}: an answer is 1-4 lines of at most 240 characters."), Some("answer too long"));
    }
    let current = |j: &Mutex<Journal>| -> String {
        let fresh = tq.snapshot_for(&board).ok().and_then(|s| find(&s, n).cloned());
        match fresh.or_else(|| j.lock().unwrap().task(&a.project, n).ok().flatten().map(|(s, r, c)| json!({"state": s, "revision": r, "card": c}))) {
            Some(t) if t["state"] == "ask" => format!("Task #{n} now asks:\n{}", card(t["card"].as_str().unwrap_or(""))),
            Some(t) => format!("Task #{n} is {}.", describe(t["state"].as_str().unwrap_or(""))),
            None => format!("Task #{n} is not open."),
        }
    };
    // a reply to an older question writes nothing (U08); a resumed in-flight write keeps its op ID instead
    let stored = j.lock().unwrap().task(&a.project, n).ok().flatten();
    if !resumed && stored.as_ref().is_some_and(|(s, r, _)| *r != a.revision || s != "ask") {
        return end("failed", format!("That question is no longer current; nothing was written. {}", current(j)), Some("stale question"));
    }
    if !resumed {
        let fresh = match tq.snapshot_for(&board) { Ok(s) => s, Err(e) => return end("failed", e, Some("fresh question unavailable")) };
        if !find(&fresh, n).is_some_and(|t| t["state"] == "ask" && t["revision"] == a.revision) {
            return end("failed", format!("That question is no longer current; nothing was written. {}", current(j)), Some("stale question"));
        }
    }
    if let Err(e) = j.lock().unwrap().take_answer(a.request) {
        return eprintln!("answer {}: {e}", a.request);
    }
    let text = lines.join("\n");
    let pick = text.trim().parse::<u8>().ok().filter(|k| (1..=4).contains(k));
    let mut args = match pick {
        Some(k) => vec!["answer".to_string(), format!("{n}.{k}")],
        None => vec!["answer".to_string(), n.to_string(), "--text".into(), text],
    };
    args.extend(["--revision".into(), a.revision.to_string()]);
    let op = match j.lock().unwrap().instance() {
        Ok(i) => format!("pilotq-{i}-a{}", a.request),
        Err(e) => return eprintln!("answer {}: {e}", a.request),
    };
    match tq.write(args, &op) {
        Ok(rc) if rc["outcome"] == "accepted" => end("answered", format!("Answer written to task #{n}; it is now {}.", describe(rc["state"].as_str().unwrap_or(""))), None),
        Ok(rc) if rc["outcome"] == "stale" => end("failed", format!("That question is no longer current; nothing was written. {}", current(j)), Some("stale question")),
        Ok(rc) if rc["outcome"] == "rejected" => {
            end("failed", format!("TaskQ refused the answer to task #{n}: {}. {}", rc["reason"].as_str().unwrap_or("no reason given"), current(j)), Some("rejected"))
        }
        Ok(rc) => end("uncertain", format!("The answer to task #{n} may have been written ({}); it is not repeated automatically.", rc["outcome"]), Some("unknown receipt")),
        Err(e) => end("uncertain", format!("The answer to task #{n} may have been written ({e}); it is not repeated automatically."), Some("unknown receipt")),
    }
}

/// One board scan: each subscribed task's fresh state becomes its status, question or closing message. Returns whether
/// anything was queued. An unreachable board or incomplete snapshot reads unknown and closes nothing.
fn scan(tq: &Taskq, j: &Mutex<Journal>) -> bool {
    let subs = match j.lock().unwrap().subscriptions() {
        Ok(s) if s.is_empty() => return false,
        Ok(s) => s,
        Err(e) => {
            eprintln!("taskq scan: {e}");
            return false;
        }
    };
    let mut queued = false;
    let mut failure = None;
    let mut snapshots = std::collections::HashMap::new();
    for (project, n, _) in subs {
        let bound = tq.registered(&j.lock().unwrap(), &project);
        let (bound, board) = match bound { Ok(x) => x, Err(e) => { failure = Some(e); continue; } };
        if !j.lock().unwrap().task_target_matches(&project, n, &board, &bound.root.to_string_lossy()).unwrap_or(false) {
            failure = Some(format!("{project} registration changed; subscription held"));
            continue;
        }
        let snapshot = snapshots.entry(project.clone()).or_insert_with(|| bound.snapshot_for(&board));
        let s = match snapshot { Ok(s) => s, Err(e) => { failure = Some(e.clone()); continue; } };
        let j = j.lock().unwrap();
        let r = match find(s, n) {
            Some(t) => {
                let (state, card) = (t["state"].as_str().unwrap_or(""), t["card"].as_str());
                j.observe(&project, n, state, t["revision"].as_i64().unwrap_or(0), card, &status_text(&board, n, state), card.map(|c| question_text(&board, n, c)).as_deref(), now())
            }
            None => j.close_task(&project, n, &format!("Task #{n} is closed on the board; its verdict and evidence are on the [issue]({}).", url(&board, n)), now()),
        };
        match r {
            Ok(q) => queued |= q,
            Err(e) => eprintln!("taskq scan #{n}: {e}"),
        }
    }
    let _ = j.lock().unwrap().set_health("taskq", now(), if failure.is_some() { "failed" } else { "ok" }, failure.as_deref());
    queued
}

/// Service loop (§ 5, L16): retries deferred plan runs, writes queued answers, and scans the board every `poll`, so a missed
/// transition is found without a nudge. Every pass rereads the journal; a timer is a wakeup, never the only copy of work.
pub fn watch(tq: std::sync::Arc<Taskq>, j: crate::telegram::Shared, wake: std::sync::Arc<tokio::sync::Notify>) {
    let mut last = std::time::Instant::now() - tq.poll;
    loop {
        let mut work = last.elapsed() >= tq.poll && recover_effects(&tq, &j);
        let runs = j.lock().unwrap().due_runs(now(), LEASE);
        for id in runs.unwrap_or_else(|e| {
            eprintln!("taskq: {e}");
            vec![]
        }) {
            submit(&tq, &j, id, true);
            work = true;
        }
        let answers = j.lock().unwrap().answers();
        for (a, resumed) in answers.unwrap_or_else(|e| {
            eprintln!("taskq: {e}");
            vec![]
        }) {
            answer(&tq, &j, &a, resumed);
            work = true;
        }
        if work || last.elapsed() >= tq.poll {
            last = std::time::Instant::now();
            work |= scan(&tq, &j);
        }
        if work {
            wake.notify_one();
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // §5 / A40: TaskQ start removes q-held before adding taskq. A complete list can
    // omit the just-filed task in that interval; it must not end its subscription.
    #[cfg(unix)]
    #[test]
    fn a40_start_label_gap_keeps_subscription() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("pilotq-start-gap-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("taskq.json"), r#"{"repo":"taskqorg/pilotq","create_claim_issue":1}"#).unwrap();
        let bin = dir.join("taskq");
        std::fs::write(&bin, "#!/bin/sh\ncat snapshot.json\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut journal = Journal::open(&dir.join("journal.sqlite")).unwrap();
        journal.bootstrap(7, 0).unwrap();
        journal.add_project(7, "pilotq", BOARD, &dir.to_string_lossy(), 0).unwrap();
        let plan = journal.ask(7, "plan", 1).unwrap();
        journal.save_plan(&plan, &json!([{"title":"fixture"}]), "pilotq", BOARD, &dir.to_string_lossy(), 1).unwrap();
        let request = journal.ask(7, "run plan", 2).unwrap();
        let id = journal.authorize(&request, 1, 7, LEASE, 2).unwrap();
        let run = journal.plan_run(id).unwrap();
        journal.filed(&run, 1, 19, 10, 3).unwrap();
        let j = Mutex::new(journal);
        let tq = Taskq { bin, root: dir.clone(), poll: Duration::from_secs(15) };
        let snapshot = |tasks: Value| {
            std::fs::write(dir.join("snapshot.json"), json!({"complete":true,"board":{"repo":BOARD},"tasks":tasks}).to_string()).unwrap();
        };
        snapshot(json!([])); // The manager is still submitting; eligibility labels are between writes.
        assert!(!scan(&tq, &j), "an in-flight start must not emit a closing outcome");
        j.lock().unwrap().started(&run, 1, 19, "ready", 4).unwrap();
        j.lock().unwrap().run_outcome(&run, "started", None, None, 0, 4).unwrap();
        snapshot(json!([{"n":19,"state":"doing","revision":12}]));
        assert!(scan(&tq, &j));
        assert_eq!(j.lock().unwrap().task("pilotq", 19).unwrap().unwrap().0, "doing");
        // Retain F14 stale protection and normal closure after submission is settled.
        snapshot(json!([{"n":19,"state":"ready","revision":11}]));
        assert!(!scan(&tq, &j));
        assert_eq!(j.lock().unwrap().task("pilotq", 19).unwrap().unwrap().0, "doing");
        snapshot(json!([]));
        assert!(scan(&tq, &j));
        assert!(j.lock().unwrap().subscriptions().unwrap().is_empty());
        drop(j);
        std::fs::remove_dir_all(dir).unwrap();
    }

    // §4/§5: a replaced run cannot create an intent or invoke a CLI, even with owner grants.
    #[test]
    fn u39_fenced_controls_write_nothing() {
        let dir = std::env::temp_dir().join(format!("pilotq-fenced-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut journal = Journal::open(&dir.join("journal.sqlite")).unwrap();
        journal.bootstrap(7, 0).unwrap();
        let old = journal.ask(7, "steer pilotq #19", 1).unwrap();
        let _replacement = journal.ask(7, "new request", 2).unwrap();
        let j = Mutex::new(journal);
        let tq = Taskq { bin: dir.join("must-not-run"), root: dir.clone(), poll: Duration::from_secs(1) };
        for name in ["task_steer", "task_requeue"] {
            let (ok, reason) = call(&tq, &j, &old, name, &json!({"project":"pilotq","task":"19","text":"x","reason":"x"}));
            assert!(!ok && reason.contains("this run was stopped"), "{reason}");
        }
        assert!(j.lock().unwrap().pending_task_effects().unwrap().is_empty());
        drop(j);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
