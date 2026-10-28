//! Codex app-server manager runtime (pilotq.md § 3, § 4 Runs, § 12 Runtime). One adapter, one process per turn: the process is the
//! fault boundary, so stop and deadlines end it, and its thread ID carries the conversation to the next turn.

use crate::journal::{Journal, Run};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::{Duration, Instant};

/// Cancellation grace before the process group is killed (§ 7).
const GRACE: Duration = Duration::from_secs(5);
/// Item types that act outside the model; once one starts, a missing terminal outcome is uncertain, never a clean failure.
const TOOLS: [&str; 4] = ["commandExecution", "fileChange", "mcpToolCall", "dynamicToolCall"];

/// Section 7 budgets: first event, silence between events and the absolute turn deadline.
pub struct Limits {
    pub first_event: Duration,
    pub silent: Duration,
    pub deadline: Duration,
}

pub struct Codex {
    pub bin: String,
    /// Neutral workspace; no project directory becomes the runtime's current directory (§ 3).
    pub workspace: PathBuf,
    /// PilotQ's own Codex home: its subscription login and an empty config, so no personal MCP server or plugin loads.
    pub home: PathBuf,
    pub limits: Limits,
}

/// What one turn established. Only `Answered` is success.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Answered(String),
    Failed(String),
    /// A tool item started and the turn ended without a native terminal outcome: never replayed automatically.
    Uncertain(String),
    /// The journal fenced this run (stop or replacement); it may not record a reply.
    Fenced,
}

struct Proc {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
    /// Codex's last stderr line, the only reason it gives when it exits early. Operator log only: it can name paths or account
    /// details, so it never reaches a reply, the journal or telemetry.
    last_err: std::sync::Arc<Mutex<Option<String>>>,
}

/// Held from spawn until Codex answers `initialize`. Found in the A8 run: app-servers started together on a fresh Codex home race
/// its SQLite state setup and all but one exit with `failed to initialize sqlite state runtime`.
// ponytail: process-wide startup lock; a hung startup delays other conversations by at most the first-event bound
static START: Mutex<()> = Mutex::new(());

impl Proc {
    fn send(&mut self, v: Value) {
        // a closed pipe surfaces as EOF on the reader, which ends the turn
        let _ = writeln!(self.stdin, "{v}").and_then(|_| self.stdin.flush());
    }

    /// Interrupts the turn, waits up to GRACE for it to end, then kills the process group.
    fn stop(&mut self, thread: &Option<String>, turn: &Option<String>) {
        if let (Some(t), Some(u)) = (thread, turn) {
            self.send(json!({"id": 9, "method": "turn/interrupt", "params": {"threadId": t, "turnId": u}}));
            let end = Instant::now() + GRACE;
            while let Some(left) = end.checked_duration_since(Instant::now()) {
                match self.rx.recv_timeout(left) {
                    Ok(m) if m["method"] == "turn/completed" => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        // ponytail: `kill` on the process group; a pidfd/cgroup supervisor if children escape the group
        #[cfg(unix)]
        let _ = Command::new("kill").args(["-9", &format!("-{}", self.child.id())]).stderr(Stdio::null()).status();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Codex {
    fn spawn(&self) -> Result<Proc, String> {
        std::fs::create_dir_all(&self.workspace).map_err(|e| format!("workspace: {e}"))?;
        std::fs::create_dir_all(&self.home).map_err(|e| format!("codex home: {e}"))?;
        let mut c = Command::new(&self.bin);
        // subscription login only, no API-key fallback (R9); native tools that reach past PilotQ's effect boundary are off (§ 12)
        c.arg("app-server");
        for f in ["shell_tool", "apps", "plugins", "browser_use", "computer_use", "multi_agent", "image_generation"] {
            c.args(["--disable", f]);
        }
        // Found in the A7 run: Codex writes its environment into shell snapshots under its home, and the host's GitHub token was
        // there. It gets only what it needs, so no host credential or API key reaches the model's side (R9, § 8).
        c.env_clear();
        for k in ["PATH", "HOME", "USER", "LANG", "LC_ALL", "TMPDIR", "SSL_CERT_FILE"] {
            if let Some(v) = std::env::var_os(k) {
                c.env(k, v);
            }
        }
        c.args(["-c", "forced_login_method=\"chatgpt\"", "-c", "web_search=\"disabled\""])
            .env("CODEX_HOME", &self.home)
            .current_dir(&self.workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut c, 0);
        let mut child = c.spawn().map_err(|e| format!("codex app-server did not start: {e}"))?;
        let (stdin, stdout, stderr) = (child.stdin.take().unwrap(), child.stdout.take().unwrap(), child.stderr.take().unwrap());
        let last_err = std::sync::Arc::new(Mutex::new(None));
        let keep = last_err.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok).filter(|l| !l.trim().is_empty()) {
                *keep.lock().unwrap() = Some(line.chars().take(200).collect());
            }
        });
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                // ponytail: a non-JSON line is diagnostic output, never a control event
                if let Ok(v) = serde_json::from_str::<Value>(&line)
                    && tx.send(v).is_err()
                {
                    break;
                }
            }
        });
        Ok(Proc { child, stdin, rx, last_err })
    }

    /// Runs one turn on `thread` (a new thread when None). `alive` reports whether the journal still holds this run's fence;
    /// `tool` persists a tool item's type, ID and, once completed, its status. `dynamic` offers PilotQ's own tools on a new thread
    /// (their specs and instructions); `call` answers one complete call of them. Returns the thread ID and model Codex used and the outcome.
    #[allow(clippy::too_many_arguments)]
    pub fn turn(
        &self,
        thread: Option<&str>,
        input: &Value,
        dynamic: Option<(&Value, &str)>,
        mut alive: impl FnMut() -> bool,
        mut tool: impl FnMut(&str, &str, Option<&str>),
        mut call: impl FnMut(&str, &Value, &str) -> (bool, String),
        mut rebuild: impl FnMut() -> Result<Value, String>,
        mut preview: impl FnMut(&str),
    ) -> (Option<String>, Option<String>, Outcome) {
        // the first-event watch starts once this turn may spawn, before Codex runs (§ 12)
        let mut starting = Some(START.lock().unwrap_or_else(|e| e.into_inner()));
        let start = Instant::now();
        let mut p = match self.spawn() {
            Ok(p) => p,
            Err(e) => return (None, None, Outcome::Failed(e)),
        };
        let (mut thread, mut turn, mut answer, mut tooled, mut model) = (thread.map(str::to_string), None::<String>, String::new(), false, None::<String>);
        let (mut first, mut last, mut error) = (false, start, None::<String>);
        let mut input = input.clone();
        let mut resuming = thread.is_some();
        let (mut draft_item, mut draft) = (String::new(), String::new());
        let lim = &self.limits;
        let mut version = String::from("unknown");
        // dynamic tools are an experimental app-server field, negotiated only when PilotQ offers some
        let caps = dynamic.map(|_| json!({"experimentalApi": true}));
        p.send(json!({"id": 1, "method": "initialize", "params": {"clientInfo": {"name": "pilotq", "title": "PilotQ", "version": env!("CARGO_PKG_VERSION")}, "capabilities": caps}}));
        let fail = |tooled: bool, why: String| if tooled { Outcome::Uncertain(why) } else { Outcome::Failed(why) };
        loop {
            if !alive() {
                p.stop(&thread, &turn);
                return (thread, model, Outcome::Fenced);
            }
            let total = start + lim.deadline;
            let (limit, why) = if first { (last + lim.silent, "Codex sent no event for") } else { (start + lim.first_event, "Codex sent no first event within") };
            let (limit, why, secs) = if total < limit { (total, "Codex did not finish within", lim.deadline) } else { (limit, why, if first { lim.silent } else { lim.first_event }) };
            let wait = limit.saturating_duration_since(Instant::now()).min(Duration::from_secs(1));
            let m = match p.rx.recv_timeout(wait) {
                Ok(m) => m,
                Err(RecvTimeoutError::Timeout) if Instant::now() >= limit => {
                    p.stop(&thread, &turn);
                    return (thread, model, fail(tooled, format!("{why} {} s", secs.as_secs())));
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    // stdout closed; give the stderr reader a moment to take Codex's last words for the operator log
                    std::thread::sleep(Duration::from_millis(200));
                    if let Some(e) = p.last_err.lock().unwrap().as_deref() {
                        eprintln!("codex exited: {e}");
                    }
                    let why = error.map_or("Codex exited without a terminal event".to_string(), |e| format!("Codex exited without a terminal event after: {e}"));
                    return (thread, model, fail(tooled, why));
                }
            };
            (first, last) = (true, Instant::now());
            let method = m["method"].as_str().unwrap_or("");
            if method == "item/tool/call" && m.get("id").is_some() {
                // a complete call of PilotQ's own tool: the boundary checks grants and the fence and answers (§ 5, § 12)
                let (ok, out) = call(m["params"]["tool"].as_str().unwrap_or(""), &m["params"]["arguments"], &version);
                p.send(json!({"id": m["id"], "result": {"contentItems": [{"type": "inputText", "text": out}], "success": ok}}));
                continue;
            }
            if m.get("id").is_some() && !method.is_empty() {
                // any other server request (approval, user input): PilotQ grants no native capability
                p.send(json!({"id": m["id"], "error": {"code": -32601, "message": "PilotQ does not grant this request"}}));
                continue;
            }
            if let Some(e) = m.get("error") {
                if m["id"] == 2 && resuming && !tooled && turn.is_none() {
                    resuming = false;
                    let message = e["message"].as_str().unwrap_or("");
                    if message.contains("failed to read thread") || message.contains("thread not found") || message.contains("no rollout found") {
                        let history = match rebuild() {
                            Ok(v) => v,
                            Err(_) => return (thread, model, Outcome::Failed("Conversation recovery could not be prepared; no work was repeated.".into())),
                        };
                        input.as_array_mut().unwrap().insert(0, json!({"type": "text", "text": history.to_string()}));
                        thread = None;
                        let mut params = json!({"cwd": self.workspace, "approvalPolicy": "never", "sandbox": "read-only"});
                        if let Some((tools, instructions)) = dynamic {
                            params["dynamicTools"] = tools.clone();
                            params["developerInstructions"] = json!(instructions);
                        }
                        p.send(json!({"id": 2, "method": "thread/start", "params": params}));
                        continue;
                    }
                    return (thread, model, Outcome::Failed("Codex could not resume this conversation; check login and runtime health.".into()));
                }
                if m["id"] == 2 {
                    return (thread, model, Outcome::Failed("Codex could not open the conversation; no work was repeated.".into()));
                }
                p.stop(&thread, &turn);
                return (thread, model, fail(tooled, format!("Codex refused {}: {}", m["id"], e["message"].as_str().unwrap_or("error"))));
            }
            match (m["id"].as_i64(), method) {
                (Some(1), _) => {
                    version = m["result"]["userAgent"].as_str().unwrap_or("unknown").to_string();
                    drop(starting.take());
                    p.send(json!({"method": "initialized"}));
                    let mut params = json!({"cwd": self.workspace, "approvalPolicy": "never", "sandbox": "read-only"});
                    if let Some((tools, instructions)) = dynamic {
                        // Tools are fixed at thread creation; schema upgrades explicitly rebuild older catalogues from the journal.
                        params["dynamicTools"] = tools.clone();
                        params["developerInstructions"] = json!(instructions);
                    }
                    match &thread {
                        Some(t) => { params["threadId"] = json!(t); params.as_object_mut().unwrap().remove("dynamicTools"); p.send(json!({"id": 2, "method": "thread/resume", "params": params})); },
                        None => p.send(json!({"id": 2, "method": "thread/start", "params": params})),
                    }
                }
                (Some(2), _) => {
                    let Some(t) = m["result"]["thread"]["id"].as_str() else {
                        return (thread, model, Outcome::Failed("Codex returned no thread ID".into()));
                    };
                    thread = Some(t.to_string());
                    model = m["result"]["model"].as_str().map(str::to_string);
                    p.send(json!({"id": 3, "method": "turn/start", "params": {"threadId": t, "input": input}}));
                }
                (Some(3), _) => turn = m["result"]["turn"]["id"].as_str().map(str::to_string),
                (_, phase @ ("item/started" | "item/completed")) if TOOLS.contains(&m["params"]["item"]["type"].as_str().unwrap_or("")) => {
                    tooled = true;
                    let item = &m["params"]["item"];
                    let status = (phase == "item/completed").then(|| item["status"].as_str().unwrap_or("completed"));
                    tool(item["type"].as_str().unwrap_or(""), item["id"].as_str().unwrap_or(""), status);
                }
                (_, "item/agentMessage/delta") => {
                    let item = m["params"]["itemId"].as_str().unwrap_or("");
                    if item != draft_item { draft.clear(); draft_item=item.to_string(); }
                    draft.push_str(m["params"]["delta"].as_str().unwrap_or(""));
                    preview(&draft);
                }
                (_, "item/completed") if m["params"]["item"]["type"] == "agentMessage" => {
                    answer = m["params"]["item"]["text"].as_str().unwrap_or("").to_string();
                }
                (_, "error") => error = m["params"]["error"]["message"].as_str().map(str::to_string),
                (_, "turn/completed") => {
                    let t = &m["params"]["turn"];
                    let outcome = match t["status"].as_str() {
                        Some("completed") if !answer.trim().is_empty() => Outcome::Answered(answer),
                        Some("completed") => Outcome::Failed("Codex ended the turn without an answer".into()),
                        Some(s) => fail(tooled, format!("Codex turn {s}: {}", t["error"]["message"].as_str().or(error.as_deref()).unwrap_or("no reason given"))),
                        None => fail(tooled, "Codex sent a terminal event without a status".into()),
                    };
                    return (thread, model, outcome);
                }
                _ => {}
            }
        }
    }
}

/// Runs one claimed request through Codex and records the outcome under its fence. The CLI and Telegram share this path (R13),
/// with the direct tools and, when TaskQ is configured, the TaskQ tools, at most `direct::BUDGET` calls per request.
pub fn run(codex: &Codex, j: &Mutex<Journal>, r: &Run, now: impl Fn() -> i64, tq: Option<&crate::taskq::Taskq>, d: &crate::direct::Direct) -> Result<Outcome, String> {
    use crate::direct;
    let mut tools = direct::tools();
    let instructions = match tq {
        Some(_) => {
            tools.extend(crate::taskq::tools().as_array().cloned().unwrap_or_default());
            format!("{} {}", crate::taskq::INSTRUCTIONS, direct::INSTRUCTIONS)
        }
        None => format!("You are PilotQ. {}", direct::INSTRUCTIONS),
    };
    let instructions = format!("{instructions} Write user-facing answers in Markdown. Do not include internal request numbers in ordinary answers; those belong in status, audit, failure and stop messages.");
    let tools = Value::Array(tools);
    // Read on every turn, including resumed threads and empty profiles after forget (§ 6, R10, R13).
    // Keep the person's text separate: neither profile values nor old summaries are authorization.
    let profile = j.lock().unwrap().profile(r.request)?;
    let refresh = j.lock().unwrap().refresh_tools(r)?;
    let mut input = json!([{"type": "text", "text": profile.to_string()}, {"type": "text", "text": r.text}]);
    if refresh {
        let context=j.lock().unwrap().rebuild(r,now())?;
        input.as_array_mut().unwrap().insert(0,json!({"type":"text","text":context.to_string()}));
    }
    let (thread, model, outcome) = codex.turn(
        if refresh {None} else {r.thread.as_deref()},
        &input,
        Some((&tools, &instructions)),
        || j.lock().unwrap().fenced(r).map(|f| !f).unwrap_or(true),
        |kind, item, result| {
            if let Err(e) = j.lock().unwrap().tool(r, kind, item, result, now()) {
                eprintln!("run {}: {e}", r.id);
            }
        },
        |name, args, version| {
            let (ok, out) = (|| {
                // § 7 direct action: a request's sixth tool call, counting earlier runs of it, is refused and writes nothing (U11)
                match j.lock().unwrap().spend(r, name, direct::BUDGET, now()) {
                    Ok(true) => {}
                    Ok(false) => return (false, format!("This request reached its budget of {} tool calls; nothing was done. Answer with the evidence so far and offer to plan the rest as a task, or ask the person.", direct::BUDGET)),
                    Err(e) => return (false, format!("the tool-call budget could not be read ({e}); nothing was done")),
                }
                let (ok, out) = match tq {
                    _ if direct::NAMES.contains(&name) => direct::call(d, j, r, name, args),
                    Some(tq) => crate::taskq::call(tq, j, r, name, args),
                    None => (false, "TaskQ is not configured".into()),
                };
                // no key value reaches the model, whichever tool printed it (§ 8)
                (ok, d.keys.scrub(&out))
            })();
            if let Err(e) = j.lock().unwrap().tool_size(r, name, out.len(), version, now()) {
                eprintln!("run {} tool size: {e}", r.id);
            }
            (ok, out)
        },
        || j.lock().unwrap().rebuild(r, now()),
        |text| { let _ = j.lock().unwrap().preview(r, &d.keys.scrub(text)); },
    );
    let j = j.lock().unwrap();
    let health = match &outcome {
        Outcome::Answered(_) => ("ok", None),
        Outcome::Failed(e) | Outcome::Uncertain(e) => ("failed", Some(e.as_str())),
        Outcome::Fenced => ("ok", None),
    };
    j.set_health("model", now(), health.0, health.1)?;
    j.finish(r, thread.as_deref(), model.as_deref(), &outcome, now())?;
    Ok(outcome)
}

/// Service loop: claims queued Telegram requests, one run per conversation and `MODELS` at once, oldest first. While the
/// `drain` file exists (`pilotq update`), nothing new starts; running turns finish (§ 12 planned restart).
#[allow(clippy::too_many_arguments)]
pub fn serve(
    codex: Codex,
    j: crate::telegram::Shared,
    wake: std::sync::Arc<tokio::sync::Notify>,
    now: fn() -> i64,
    drain: PathBuf,
    tq: Option<std::sync::Arc<crate::taskq::Taskq>>,
    d: std::sync::Arc<crate::direct::Direct>,
) {
    const MODELS: usize = 4;
    let codex = std::sync::Arc::new(codex);
    loop {
        let claimed = if drain.exists() { Ok(vec![]) } else { j.lock().unwrap().claim_next(MODELS, now()) };
        match claimed {
            Ok(runs) => {
                for r in runs {
                    let (codex, j, wake, tq, d) = (codex.clone(), j.clone(), wake.clone(), tq.clone(), d.clone());
                    std::thread::spawn(move || {
                        if let Err(e) = run(&codex, &j, &r, now, tq.as_deref(), &d) {
                            eprintln!("run {}: {e}", r.id);
                        }
                        wake.notify_one();
                    });
                }
            }
            Err(e) => eprintln!("manager: {e}"),
        }
        // ponytail: 0.5 s journal poll; a wake channel from ingress if admission latency matters
        std::thread::sleep(Duration::from_millis(500));
    }
}
