//! A40 release chain: real PilotQ/Codex/TaskQ, synthetic Telegram. See docs/acceptance.md.
use super::*;

const CHAT: i64 = -100;
const TOPIC: Option<i64> = Some(5);
const CASES: [&str; 14] = [
    "admission", "discussion", "plan", "create_start", "status", "steer", "pause", "park_requeue",
    "restart", "question_foreign", "answer_stale", "reviewer", "audit_delivery", "cleanup",
];

// Record intents before dispatch, including an add whose response is lost. Cleanup retries that exact argv/op ID.
// No token, environment or board snapshot is logged. Python is already used by the integration fixtures.
const RECORD_TASKQ: &str = r#"#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys
base = pathlib.Path(__file__).parent
args = sys.argv[1:]
if args and args[0] == 'add':
    # Bind the create claim itself, not a later label edit: start must never race a service.
    host = base.joinpath('taskq-host').read_text().strip()
    if not host.startswith('pilotq-e2e-'): sys.exit('invalid A40 host')
    if '--host' in args:
        i = args.index('--host')
        if args.count('--host') != 1 or args[i+1:i+2] != [host]: sys.exit('foreign A40 host')
    else: args += ['--host', host]
write = args and args[0] in ('add', 'start', 'steer', 'requeue', 'later', 'answer', 'service')
def save(row):
    with open(base / 'taskq-calls.jsonl', 'a') as f:
        f.write(json.dumps(row) + '\n'); f.flush(); os.fsync(f.fileno())
if write: save({'args': args})
p = subprocess.run([str(base.joinpath('taskq-path').read_text().strip()), *args], capture_output=True, timeout=120)
if write:
    try:
        v = json.loads(p.stdout)
        save({'args': args, 'receipt': {k:v.get(k) for k in ('task','op_id','outcome','state','revision')}})
    except (ValueError, AttributeError): pass
sys.stdout.buffer.write(p.stdout); sys.stderr.buffer.write(p.stderr)
sys.exit(p.returncode)
"#;

struct Run {
    d: PathBuf,
    st: State,
    root: PathBuf,
    report: PathBuf,
    data: Value,
    svc: Option<Svc>,
    login: Option<Login>,
    update: i64,
    task: i64,
    run_request: i64,
    deadline: Instant,
}

pub(super) fn pause_tool_status(d: &Path, id: i64) -> String {
    query_text(d, &format!("SELECT result FROM audit_events WHERE request_id={id} AND kind='tool' AND target='dynamicToolCall' AND result='completed' AND EXISTS (SELECT 1 FROM audit_events WHERE request_id={id} AND kind='call' AND target='queue_pause') ORDER BY id DESC LIMIT 1"))
}

impl Run {
    // TaskQ §2 permits trusted hand-authored entries. These are scripted fixtures, never real worker/reviewer claims.
    fn event(&mut self, body: &str, state: &str) -> i64 {
        let path = format!("repos/taskqorg/pilotq/issues/{}/comments", self.task);
        let rc = self.github("POST", &path, json!({"body":body}));
        let id = rc["id"].as_i64().unwrap();
        let inbox = self.taskq(&["inbox".into(), self.task.to_string()]).unwrap();
        let folded: Value = serde_json::from_str(inbox.lines().find_map(|l| l.strip_prefix("Folded: ")).expect("folded event readback")).unwrap();
        assert_eq!(folded["state"], state, "scripted event was not accepted");
        if body.starts_with("taskq: spawn ") {
            assert_eq!(folded["attempt"], id, "another attempt claimed the fixture");
        }
        let events = self.data.as_object_mut().unwrap().entry("scripted_events").or_insert(json!([]));
        events.as_array_mut().unwrap().push(json!({"comment":id,"body":body,"state":state}));
        self.save();
        id
    }

    fn github(&self, method: &str, path: &str, payload: Value) -> Value {
        let mut child = Command::new("gh").args(["api", "--method", method, path, "--input", "-"])
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(payload.to_string().as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "scripted board event failed: {}", out.status);
        serde_json::from_slice(&out.stdout).unwrap()
    }

    fn spawn_fixture(&mut self) -> i64 {
        let host = self.data["isolated_host"].as_str().unwrap();
        let attempt = self.event(&format!("taskq: spawn worker codex:a40-scripted {host}\nA40 synthetic lifecycle; no process is launched."), "doing");
        self.wait("scripted doing observation",60,|| self.state("doing"));
        self.data["fixture_attempt"] = json!(attempt);
        attempt
    }

    fn save(&mut self) {
        self.data["messages"] = json!(sent(&self.st));
        // Atomic replacement leaves the last complete report readable after interruption.
        let tmp = self.report.with_extension("json.tmp");
        let mut file = fs::File::create(&tmp).unwrap();
        file.set_permissions(fs::Permissions::from_mode(0o600)).unwrap();
        file.write_all(&serde_json::to_vec_pretty(&self.data).unwrap()).unwrap();
        file.sync_all().unwrap();
        fs::rename(tmp, &self.report).unwrap();
    }

    fn start(&mut self) {
        // A long live run must not fill an unread stderr pipe and stall PilotQ.
        let log = fs::OpenOptions::new().create(true).append(true).open(self.d.join("service.log")).unwrap();
        self.svc = Some(Svc(service(&self.d, None).stderr(log).spawn().unwrap()));
        self.data["service_pid"] = json!(self.svc.as_ref().unwrap().id());
        self.save();
    }

    fn case(&mut self, name: &str, f: impl FnOnce(&mut Self) -> Value) {
        self.data["cases"][name] = json!({"status":"running"});
        self.save();
        eprintln!("A40: {name}");
        let started = Instant::now();
        let evidence = f(self);
        self.data["cases"][name] = json!({"status":"pass", "elapsed_ms":started.elapsed().as_millis(), "evidence":evidence});
        self.save();
    }

    fn wait(&self, label: &str, seconds: u64, mut ready: impl FnMut() -> bool) {
        let until = (Instant::now() + Duration::from_secs(seconds)).min(self.deadline);
        while !ready() {
            assert!(Instant::now() < until, "A40 timed out: {label}");
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    fn request(&mut self, who: i64, body: &str, question: Option<i64>) -> (i64, String) {
        self.update += 1;
        let u = self.update;
        push(&self.st, match question {
            Some(q) => reply(u, CHAT, TOPIC, who, q, body),
            None => text(u, CHAT, TOPIC, who, body),
        });
        self.wait("request terminal and delivered", 300, || {
            try_query(&self.d, &format!("SELECT count(*) FROM requests q WHERE update_id={u} AND state IN ('answered','failed','uncertain') AND EXISTS (SELECT 1 FROM outbox o WHERE o.request_id=q.id AND o.kind='reply' AND o.state='delivered')")) == Some(1)
        });
        let id = query(&self.d, &format!("SELECT id FROM requests WHERE update_id={u}"));
        let reply = query_text(&self.d, &format!("SELECT reply FROM requests WHERE id={id}"));
        (id, reply)
    }

    fn model(&mut self, body: &str) -> (i64, String) {
        let (id, reply) = self.request(OWNER, body, None);
        assert_eq!(query_text(&self.d, &format!("SELECT state FROM requests WHERE id={id}")), "answered", "request {id}: {reply}");
        (id, reply)
    }

    fn taskq(&self, args: &[String]) -> Result<String, String> {
        let out = Command::new(self.d.join("record-taskq"))
            .args(args).env("TASKQ_ROOT", &self.root).env_remove("TASKQ_ATTEMPT")
            .env_remove("TASKQ_TASK").env_remove("TASKQ_ROLE").current_dir(&self.root).output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err(format!("taskq {} failed ({})", args[0], out.status)); }
        String::from_utf8(out.stdout).map_err(|e| e.to_string())
    }

    fn calls(&self) -> Vec<Value> {
        fs::read_to_string(self.d.join("taskq-calls.jsonl")).unwrap_or_default().lines()
            .map(|l| serde_json::from_str(l).expect("complete recorded TaskQ intent")).collect()
    }

    fn control(&mut self, tool: &str, body: &str, command: &str) -> Value {
        let (id, reply) = self.model(body);
        let c = rusqlite::Connection::open(self.d.join("journal.sqlite")).unwrap();
        let (op, args, receipt): (String, String, String) = c.query_row(
            "SELECT op,args,receipt FROM task_effects WHERE request_id=?1 AND receipt IS NOT NULL", [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        let args: Value = serde_json::from_str(&args).unwrap();
        let rc: Value = serde_json::from_str(&receipt).unwrap();
        assert_eq!(args[0], command, "{tool}: {reply}");
        assert_eq!(args[1], self.task.to_string());
        assert!(args.as_array().unwrap().contains(&json!("--revision")));
        assert_eq!((&rc["outcome"], &rc["task"], &rc["op_id"]), (&json!("accepted"), &json!(self.task), &json!(op)));
        assert!(reply.contains(&format!("https://github.com/taskqorg/pilotq/issues/{}", self.task)));
        let inbox = self.taskq(&["inbox".into(), self.task.to_string()]).unwrap();
        assert!(inbox.contains(&op), "board does not read back {tool}'s operation ID");
        json!({"request":id, "operation":op, "args":args, "receipt":rc})
    }

    fn state(&self, state: &str) -> bool {
        try_query(&self.d, &format!("SELECT count(*) FROM tasks WHERE number={} AND state='{state}'", self.task)) == Some(1)
    }

    fn cleanup(&mut self) -> Result<Value, String> {
        // Drop stops and reaps PilotQ before replay reconciliation; its scan must no longer submit work.
        self.svc.take();
        self.data["service_pid"] = Value::Null;
        self.login.take();
        let mut errors = vec![];
        let mut tasks = vec![];
        let mut adds = std::collections::BTreeMap::new();
        for c in self.calls() {
            if c["args"][0] == "add" {
                let args: Vec<String> = serde_json::from_value(c["args"].clone()).unwrap();
                if let Some(i) = args.iter().position(|a| a == "--op-id") { adds.insert(args[i + 1].clone(), args); }
            }
        }
        for (op, args) in adds {
            // Same claim reconciles a committed create whose receipt never reached PilotQ; no new op ID or plain add.
            // A durable accepted create receipt already identifies the task. Replaying add can be
            // unavailable after the lifecycle advances; it must not prevent parking that known task.
            let recorded = self.calls().into_iter().find_map(|c| {
                let rc = &c["receipt"];
                (c["args"] == json!(args) && rc["op_id"] == op && rc["outcome"] == "accepted"
                    && rc["task"].as_i64().is_some_and(|n| n > 0)).then(|| rc.clone())
            });
            let rc = recorded.map(Ok).unwrap_or_else(|| self.taskq(&args)
                .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|e| e.to_string())));
            match rc {
                Ok(rc) if rc["outcome"] == "accepted" && rc["op_id"] == op => {
                    if let Some(n) = rc["task"].as_i64() { tasks.push(n); } else { errors.push("create receipt has no task".into()); }
                }
                Ok(rc) if rc["outcome"] == "rejected" => (),
                _ => errors.push(format!("cannot reconcile create {op}")),
            }
        }
        tasks.sort_unstable(); tasks.dedup();
        let mut outcomes = vec![];
        for n in tasks {
            let folded = self.taskq(&["inbox".into(), n.to_string()]).and_then(|body| {
                body.lines().find_map(|l| l.strip_prefix("Folded: ")).ok_or("missing folded board state".to_string())
                    .and_then(|s| serde_json::from_str::<Value>(s).map_err(|e| e.to_string()))
            });
            match folded {
                Ok(state) if state["state"] == "closed" => {
                    // A crash may separate the synthetic close log from GitHub's issue-state update.
                    let path = format!("repos/taskqorg/pilotq/issues/{n}");
                    self.github("PATCH", &path, json!({"state":"closed","state_reason":"completed"}));
                    let issue = gh_json(&path);
                    if issue["state"] == "closed" { outcomes.push(json!({"task":n,"state":"closed"})); }
                    else { errors.push(format!("task #{n} close was not read back")); }
                    continue;
                }
                Err(e) => { errors.push(format!("task #{n} readback: {e}")); continue; }
                _ => (),
            }
            let rc = self.taskq(&["later".into(), n.to_string(), "--text".into(), "Reason: A40 automatic run ended; retain evidence for investigation.".into(),
                "--op-id".into(), format!("a40-{}-{n}-cleanup", std::process::id()), "--json".into()])
                .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|e| e.to_string()));
            match rc {
                Ok(rc) if rc["outcome"] == "accepted" && rc["state"] == "later" => outcomes.push(json!({"task":n,"state":"later","receipt":rc})),
                _ => errors.push(format!("task #{n} was not verified parked")),
            }
        }
        // A pause intent can have succeeded before its response was lost. Restore only if this run dispatched one.
        if self.calls().iter().any(|c| c["args"][0] == "service" && c["args"][1] == "pause") {
            if self.root.join(".taskq/focus.paused").exists() {
                let saved = fs::read_to_string(self.root.join(".taskq/focus.paused")).ok();
                let current = fs::read_to_string(self.root.join(".taskq/focus")).ok();
                if saved.as_deref().map(str::trim) == self.data["original_focus"].as_str().map(str::trim) && current.as_deref().map(str::trim) == Some("off") {
                    if let Err(e) = self.taskq(&["service".into(), "resume".into()]) { errors.push(e); }
                } else if current.as_deref().map(str::trim) != self.data["original_focus"].as_str().map(str::trim) {
                    errors.push("queue configuration changed; PM reconciliation required".into());
                }
            }
            if let Some(original) = self.data["original_focus"].as_str() {
                if fs::read_to_string(self.root.join(".taskq/focus")).ok().as_deref().map(str::trim) != Some(original.trim()) { errors.push("queue focus was not restored".into()); }
            }
        }
        self.data["cleanup_tasks"] = json!(outcomes);
        if self.d.join("codex/auth.json").exists() { errors.push("copied login remains".into()); }
        if errors.is_empty() { Ok(json!({"tasks":outcomes,"copied_credentials_removed":true})) }
        else { Err(errors.join("; ")) }
    }
    fn finish(&mut self, result: Result<(), Box<dyn std::any::Any + Send>>) {
        if let Err(e) = &result {
            let reason = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or("panic".into());
            self.data["failure"] = json!(reason);
            for c in CASES {
                if self.data["cases"][c]["status"] == "running" {
                    self.data["cases"][c] = json!({"status":"failed", "reason":reason});
                }
            }
        }
        if self.d.join("journal.sqlite").exists() {
            let out = cli(&self.d, &["pilot", "report", "--since", "0", "--json"]);
            self.data["tool_sizes"] = if out.status.success() {
                serde_json::from_slice::<Value>(&out.stdout).ok().and_then(|v| v["tools"].as_array().cloned())
                    .map(Value::Array).unwrap_or_else(|| json!({"status":"unknown","reason":"invalid pilot report"}))
            } else { json!({"status":"unknown","reason":"pilot report failed"}) };
            self.data["tool_size_source"] = json!("journal tool_sizes: UTF-8 result bytes after secret scrubbing, before Codex delivery; audit_events stores status, not result text");
        }
        self.data["cases"]["cleanup"] = json!({"status":"running"});
        self.save();
        let cleanup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.cleanup()));
        self.data["cases"]["cleanup"] = match cleanup {
            Ok(Ok(evidence)) => json!({"status":"pass", "evidence":evidence}),
            Ok(Err(reason)) => json!({"status":"failed", "reason":reason}),
            Err(_) => json!({"status":"failed", "reason":"cleanup panicked; PM reconciliation required"}),
        };
        self.data["pass"] = json!(result.is_ok() && CASES.iter().all(|c| self.data["cases"][c]["status"] == "pass"));
        self.save();
    }

}

#[test]
#[ignore = "real subscription Codex and taskqorg/pilotq task; run before each release"]
fn a40_live_management() {
    let report = PathBuf::from(std::env::var("PILOTQ_A40_REPORT").expect("PILOTQ_A40_REPORT"));
    assert!(report.is_absolute() && !report.exists(), "choose a new absolute report path");
    let tag = format!("a40-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let (d, st) = setup(&tag, vec![]);
    fs::set_permissions(&d, fs::Permissions::from_mode(0o700)).unwrap();
    let mut h = Run {
        d, st, root: PathBuf::from(std::env::var("PILOTQ_A8_ROOT").expect("PILOTQ_A8_ROOT")), report,
        data: json!({"suite":"A40", "board":"taskqorg/pilotq", "pass":false, "cases":{}}),
        svc:None, login:None, update:0, task:0, run_request:0, deadline:Instant::now() + Duration::from_secs(10800),
    };
    for c in CASES { h.data["cases"][c] = json!({"status":"not_run"}); }
    h.data["install"] = json!(h.d);
    h.save();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let config: Value = serde_json::from_slice(&fs::read(h.root.join("taskq.json")).unwrap()).unwrap();
        assert_eq!(config["repo"], "taskqorg/pilotq");
        assert!(config["create_claim_issue"].as_i64().is_some_and(|n| n > 0));
        let focus = fs::read_to_string(h.root.join(".taskq/focus")).expect("existing queue focus required");
        assert!(focus.trim() != "off", "admission already paused");
        h.data["original_focus"] = json!(focus);
        // A fresh reserved host name has no service. Never enroll a service under pilotq-e2e-*.
        h.data["isolated_host"] = json!(format!("pilotq-e2e-{tag}"));
        fs::write(h.d.join("taskq-host"), h.data["isolated_host"].as_str().unwrap()).unwrap();
        fs::write(h.d.join("taskq-path"), std::env::var("PILOTQ_A8_TASKQ").expect("PILOTQ_A8_TASKQ")).unwrap();
        fs::write(h.d.join("record-taskq"), RECORD_TASKQ).unwrap();
        fs::set_permissions(h.d.join("record-taskq"), fs::Permissions::from_mode(0o700)).unwrap();
        h.login = Some(live_login(&h.d));
        limits(&h.d, Path::new("codex"), json!({"first_event":60,"silent":60,"deadline":240}));
        let mut cfg: Value = serde_json::from_slice(&fs::read(h.d.join("config.json")).unwrap()).unwrap();
        cfg["taskq"] = json!({"bin":h.d.join("record-taskq"),"root":h.root,"poll":15});
        fs::write(h.d.join("config.json"), cfg.to_string()).unwrap();
        for (key, bin, args) in [
            ("candidate", "git", vec!["rev-parse", "HEAD"]),
            ("binary_sha256", "sha256sum", vec![env!("CARGO_BIN_EXE_pilotq")]),
            ("codex_version", "codex", vec!["--version"]),
        ] {
            let o = Command::new(bin).args(args).output().unwrap();
            assert!(o.status.success());
            h.data[key] = json!(String::from_utf8_lossy(&o.stdout).trim());
        }
        let taskq_bin = std::env::var("PILOTQ_A8_TASKQ").unwrap();
        let hash = Command::new("sha256sum").arg(&taskq_bin).output().unwrap();
        assert!(hash.status.success());
        let dirty = Command::new("git").args(["status", "--porcelain"]).output().unwrap();
        assert!(dirty.status.success());
        h.data["worktree_dirty"] = json!(!dirty.stdout.is_empty());
        let sources = Command::new("sha256sum").args(["tests/telegram.rs", "tests/support/automatic.rs"]).output().unwrap();
        assert!(sources.status.success());
        h.data["test_source_sha256"] = json!(String::from_utf8_lossy(&sources.stdout));
        h.data["taskq_binary_sha256"] = json!(String::from_utf8_lossy(&hash.stdout).split_whitespace().next().unwrap());
        h.data["taskq_version"] = json!("unknown: CLI has no version command; exact binary hash recorded");
        let snapshot: Value = serde_json::from_str(&h.taskq(&["status".into(), "--json".into()]).unwrap()).unwrap();
        assert_eq!(snapshot["board"]["repo"], "taskqorg/pilotq");
        assert_eq!(snapshot["complete"], true, "board snapshot incomplete");
        h.save();
        h.start();
        h.case("admission", |h| {
            h.update += 1;
            push(&h.st, text(h.update, CHAT, TOPIC, 9, "Do not admit this synthetic stranger."));
            h.wait("ignored person", 30, || try_query(&h.d, "SELECT count(*) FROM updates WHERE update_id=1 AND disposition='ignored'") == Some(1));
            assert_eq!(query(&h.d, "SELECT count(*) FROM requests"), 0);
            assert!(sent(&h.st).is_empty() && h.st.lock().unwrap().actions.is_empty());
            ok(cli(&h.d, &["users","grant","8","chat"]));
            ok(cli(&h.d, &["routes","set","-100:5","shared"]));
            let (id, _) = h.request(8, "Reply hello in one word; do not call a tool.", None);
            assert_eq!(query(&h.d, &format!("SELECT sender FROM requests WHERE id={id}")), 8);
            json!({"ignored_update":1,"second_user_request":id})
        });
        h.case("discussion", |h| {
            let (id, reply) = h.model("Use project_list to show which registered projects I may access, then discuss briefly why release checks matter. Do not plan or execute anything.");
            assert_eq!(query(&h.d,"SELECT count(*) FROM plans"),0);
            assert!(!h.calls().iter().any(|c| c["args"][0] == "add"));
            assert_eq!(query(&h.d,&format!("SELECT count(*) FROM audit_events WHERE request_id={id} AND target='project_list' AND kind='call'")),1);
            assert!(h.st.lock().unwrap().reactions.iter().any(|r|r["chat_id"]==CHAT));
            assert!(!sent(&h.st).iter().any(|m|m["text"].as_str().unwrap_or("").contains("is queued")));
            json!({"request":id,"reply":reply,"project_list_called":true,"reaction_ack":true})
        });
        h.case("plan", |h| {
            let (id, _) = h.model("Use task_plan to save revision 1 for project pilotq, exactly one docs ticket: A40 scripted release-check navigation verification. Goal: exercise the automatic management lifecycle without code changes or ordinary workers. Acceptance: scripted question and answer, then scripted reviewer outcome; this is a fixture, not a request to implement or publish changes. Scope tests only. Do not run the plan yet. Show the saved revision.");
            assert_eq!(query(&h.d,"SELECT count(*) FROM plans WHERE revision=1 AND json_array_length(tickets)=1"),1);
            assert_eq!(query(&h.d,"SELECT count(*) FROM plan_runs"),0);
            json!({"request":id,"revision":1,"board_writes":0})
        });
        h.case("create_start", |h| {
            let (id, reply) = h.model("Run saved plan revision 1 on project pilotq now using task_run_plan.");
            h.run_request = id;
            assert_eq!(query(&h.d,"SELECT count(*) FROM plan_runs WHERE state='started'"),1);
            assert_eq!(query(&h.d,"SELECT count(*) FROM tickets WHERE started=1 AND task IS NOT NULL"),1);
            h.task = query(&h.d,"SELECT task FROM tickets");
            let issue_json = gh_json(&format!("repos/taskqorg/pilotq/issues/{}", h.task));
            let hosts: Vec<&str> = issue_json["labels"].as_array().unwrap().iter().filter_map(|l| l["name"].as_str()).filter(|s|s.starts_with("host-")).collect();
            assert_eq!(hosts, vec![format!("host-{}",h.data["isolated_host"].as_str().unwrap()).as_str()]);
            let issue = h.taskq(&["inbox".into(),h.task.to_string()]).unwrap();
            assert!(issue.contains("taskq: hold") && issue.contains("taskq: start"));
            assert!(!issue.contains("taskq: spawn"), "a service claimed the isolated fixture");
            assert!(reply.contains(&format!("https://github.com/taskqorg/pilotq/issues/{}",h.task)));
            assert_eq!(query(&h.d,"SELECT count(*) FROM audit_events WHERE kind='task' AND result='held'"),1);
            h.data["task_url"] = json!(format!("https://github.com/taskqorg/pilotq/issues/{}",h.task));
            json!({"request":id,"task":h.task,"held_readback":true,"started":true,"host_labels":hosts})
        });
        h.case("status", |h| {
            let (id, reply) = h.model(&format!("Use task_status on pilotq #{} and give its current state and URL.",h.task));
            assert_eq!(query(&h.d,&format!("SELECT count(*) FROM audit_events WHERE request_id={id} AND kind='task' AND target='taskqorg/pilotq status #{}' AND result='read'",h.task)),1);
            assert!(reply.contains(h.data["task_url"].as_str().unwrap()));
            json!({"request":id,"reply":reply})
        });
        h.case("steer", |h| {
            h.spawn_fixture();
            h.control("task_steer", &format!("Use task_steer on pilotq #{}: Keep README scope and ask the wording question before editing.",h.task),"steer")
        });
        h.case("pause", |h| {
            let (id, reply) = h.model("Use queue_pause now. Explain that existing workers continue.");
            assert_eq!(query(&h.d,&format!("SELECT count(*) FROM audit_events WHERE request_id={id} AND kind='task' AND target='service pause' AND result='accepted'")),1);
            assert_eq!(fs::read_to_string(h.root.join(".taskq/focus")).unwrap().trim(),"off");
            assert_eq!(fs::read_to_string(h.root.join(".taskq/focus.paused")).unwrap().trim(),h.data["original_focus"].as_str().unwrap().trim());
            let tool_status = pause_tool_status(&h.d, id);
            assert_eq!(tool_status, "completed");
            assert!(h.state("doing"), "queue pause must preserve the active worker");
            json!({"request":id,"reply":reply,"tool_status":tool_status,"admission":"off","worker":"doing"})
        });
        h.case("park_requeue", |h| {
            let park = h.control("task_park",&format!("Use task_park on pilotq #{}: park for this bounded automatic control check.",h.task),"later");
            assert_eq!(park["receipt"]["state"],"later");
            let requeue = h.control("task_requeue",&format!("Use task_requeue on pilotq #{}: resume the authorized README task after the automatic control check.",h.task),"requeue");
            assert_eq!(requeue["receipt"]["state"],"ready");
            h.taskq(&["service".into(),"resume".into()]).unwrap();
            assert_eq!(fs::read_to_string(h.root.join(".taskq/focus")).unwrap().trim(),h.data["original_focus"].as_str().unwrap().trim());
            json!({"park":park,"requeue":requeue,"admission_restored":true})
        });
        h.case("restart", |h| {
            h.spawn_fixture();
            let before = query(&h.d,"SELECT count(*) FROM tickets");
            let observed = query(&h.d,"SELECT at FROM health WHERE source='taskq'");
            h.svc.take();
            h.start();
            h.wait("fresh task observation after restart",60,|| query(&h.d,"SELECT at FROM health WHERE source='taskq' AND state='ok'") > observed);
            assert!(h.state("doing") || h.state("ask"));
            assert_eq!(query(&h.d,"SELECT count(*) FROM tickets"),before);
            json!({"task":h.task,"tickets":before})
        });
        let mut question = 0;
        h.case("question_foreign", |h| {
            let attempt = h.data["fixture_attempt"].as_i64().unwrap();
            h.event(&format!("taskq: ask attempt {attempt}\nDone: Scripted A40 fixture; no implementation requested.\nProblem: Choose wording for this synthetic navigation check.\n1. Short link ★\n2. Descriptive link\nTo answer: `taskq answer {}.K`",h.task), "ask");
            h.wait("worker question (required)",1800,|| h.state("ask") && sent(&h.st).iter().any(|s| s["text"].as_str().is_some_and(|t| t.starts_with(&format!("Task #{} asks:",h.task)))));
            // Only use the question at the currently observed revision, not one superseded by requeue.
            let revision = query(&h.d,&format!("SELECT revision FROM tasks WHERE number={}",h.task));
            question = query(&h.d,&format!("SELECT sent_message FROM outbox WHERE task={} AND revision={revision} AND kind='question' AND state='delivered' ORDER BY id DESC LIMIT 1",h.task));
            let count = h.calls().iter().filter(|c| c["args"][0] == "answer").count();
            let (id, reply) = h.request(8,"1",Some(question));
            assert!(reply.contains("no project:pilotq grant") || reply.contains("no taskq grant"),"{reply}");
            assert_eq!(h.calls().iter().filter(|c| c["args"][0] == "answer").count(),count);
            json!({"message":question,"revision":revision,"foreign_request":id,"refusal":reply})
        });
        h.case("answer_stale", |h| {
            let (id, answer) = h.request(OWNER,"1",Some(question));
            assert!(answer.contains(&format!("Answer written to task #{}",h.task)),"{answer}");
            let calls = h.calls();
            let rc = calls.iter().find(|c| c["args"][0] == "answer" && c["receipt"]["outcome"] == "accepted").expect("accepted answer receipt");
            assert!(rc["args"].as_array().unwrap().contains(&json!("--revision")));
            assert!(rc["receipt"]["op_id"].as_str().unwrap().starts_with("pilotq-"));
            let before = calls.iter().filter(|c| c["args"][0] == "answer").count();
            let (stale, refusal) = h.request(OWNER,"1",Some(question));
            assert!(refusal.contains("no longer current; nothing was written"),"{refusal}");
            assert_eq!(h.calls().iter().filter(|c| c["args"][0] == "answer").count(),before);
            json!({"request":id,"receipt":rc["receipt"],"stale_request":stale,"refusal":refusal})
        });
        h.case("reviewer", |h| {
            // The accepted answer resumes the same synthetic attempt; no ordinary worker runs.
            let inbox = h.taskq(&["inbox".into(),h.task.to_string()]).unwrap();
            let folded: Value = serde_json::from_str(inbox.lines().find_map(|l|l.strip_prefix("Folded: ")).unwrap()).unwrap();
            let attempt = folded["attempt"].as_i64().unwrap();
            assert_eq!(folded["attempt"],h.data["fixture_attempt"]);
            let candidate = h.data["candidate"].as_str().unwrap().to_string();
            h.event(&format!("taskq: result {candidate} attempt {attempt}\nChanged: Scripted A40 fixture reached result; no code changed or published.\nChecks: Synthetic management lifecycle only.\nOpen: none"), "review");
            h.wait("review delivery",60,|| sent(&h.st).iter().any(|s|s["text"].as_str().is_some_and(|t|t.starts_with(&format!("Task #{} review:",h.task)))));
            let host = h.data["isolated_host"].as_str().unwrap();
            let reviewer = h.event(&format!("taskq: spawn reviewer codex:a40-scripted-review {host}\nA40 synthetic reviewer; no process is launched."), "review");
            h.event(&format!("taskq: close attempt {reviewer}\nChanged: Scripted A40 reviewer outcome; no code review or publication claimed.\nChecks: Management notification fixture.\nOpen: none"), "closed");
            h.github("PATCH", &format!("repos/taskqorg/pilotq/issues/{}",h.task), json!({"state":"closed","state_reason":"completed"}));
            h.wait("reviewer outcome",7200,|| sent(&h.st).iter().any(|s| s["text"].as_str().is_some_and(|t|t.starts_with(&format!("Task #{} is closed on the board",h.task)))));
            assert!(sent(&h.st).iter().any(|s|s["text"].as_str().is_some_and(|t|t.starts_with(&format!("Task #{} review:",h.task)))));
            let issue = gh_json(&format!("repos/taskqorg/pilotq/issues/{}",h.task));
            assert_eq!((&issue["state"],&issue["state_reason"]),(&json!("closed"),&json!("completed")));
            let inbox = h.taskq(&["inbox".into(),h.task.to_string()]).unwrap();
            assert!(inbox.contains("taskq: result") && inbox.contains("taskq: close"),"missing result/reviewer close evidence");
            json!({"task":h.task,"state":"closed completed","scripted_result_and_close":true,"real_code_review":false})
        });
        h.case("audit_delivery", |h| {
            let out = cli(&h.d,&["audit","--json"]); assert!(out.status.success());
            let audit: Value = serde_json::from_slice(&out.stdout).unwrap();
            let run = audit.as_array().unwrap().iter().find(|a|a["request"] == h.run_request).unwrap();
            assert_eq!(run["who"]["id"],OWNER);
            assert_eq!(run["where"]["chat"], CHAT);
            assert_eq!(run["where"]["topic"], 5);
            assert_eq!(run["where"]["project"], "pilotq");
            assert_eq!(run["trace"],format!("pilotq.request.{}",h.run_request));
            assert!(run["tasks"].as_array().unwrap().iter().any(|t|t["target"] == format!("taskqorg/pilotq add #{}",h.task)));
            assert!(run["tasks"].as_array().unwrap().iter().any(|t|t["target"] == format!("taskqorg/pilotq start #{}",h.task)));
            let telemetry = coreq::telemetry::Reader::open(&h.d.join("telemetry.sqlite")).unwrap().export(0,0,1000).unwrap().to_string();
            assert!(telemetry.contains(&format!("pilotq.request.{}",h.run_request)));
            h.wait("all required deliveries", 60, || try_query(&h.d, "SELECT count(*) FROM outbox WHERE state NOT IN ('delivered','superseded')") == Some(0));
            let messages = sent(&h.st);
            assert!(messages.iter().all(|m|m["chat_id"] == CHAT && m["message_thread_id"] == 5));
            let terminal = messages.iter().find(|m|m["text"].as_str().is_some_and(|t|t.starts_with(&format!("Task #{} is closed on the board",h.task)))).unwrap();
            assert!(terminal.get("edit").is_none());
            assert!(!messages.iter().any(|m|m["edit"] == true && m["message_id"] == terminal["id"]));
            h.data["audit"] = audit;
            json!({"trace":format!("pilotq.request.{}",h.run_request),"all_messages_in_originating_topic":true})
        });
    }));
    h.finish(result);
    eprintln!("A40 report: {}",h.report.display());
    assert_eq!(h.data["pass"],true,"A40 incomplete; inspect the per-case report and cleanup");
}

#[test]
fn a40_host_isolation() {
    assert!(Command::new("python3").arg(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/automatic_host.py")).status().unwrap().success());
}

// R5/R17: a failed case must remain failed while cleanup reconciles an interrupted add, parks it and restores our pause.
#[test]
fn a40_failure_report_and_cleanup() {
    let (d, st) = setup("a40-cleanup", vec![]);
    let root = d.join("root"); fs::create_dir_all(root.join(".taskq")).unwrap();
    fs::write(root.join(".taskq/focus"), "off").unwrap();
    fs::write(root.join(".taskq/focus.paused"), "all\n").unwrap();
    fs::create_dir_all(d.join("codex")).unwrap();
    fs::write(d.join("codex/auth.json"), "invented-test-credential").unwrap();
    let add = json!(["add","Verify README navigation","--op-id","pilotq-fixture-p1-t1","--json"]);
    fs::write(d.join("taskq-calls.jsonl"), format!("{}\n{}\n",json!({"args":add,"receipt":{"task":19,"op_id":"pilotq-fixture-p1-t1","outcome":"accepted","state":"held"}}),json!({"args":["service","pause"]}))).unwrap();
    let fake = r#"#!/usr/bin/env python3
import json, pathlib, sys
p = pathlib.Path(__file__).parent
a = sys.argv[1:]
with open(p/'cleanup-calls.jsonl','a') as f: f.write(json.dumps(a)+'\n')
if a[0] == 'inbox': print('Folded: {"state":"ready"}')
elif a[:2] == ['service','resume']:
    r=p/'root/.taskq'
    (r/'focus').write_text((r/'focus.paused').read_text())
elif a[0] == 'add':
    if not (p/'allow-reconcile').exists(): sys.exit('create reconciliation unavailable')
    print(json.dumps({'task':19,'op_id':'pilotq-fixture-p1-t1','outcome':'accepted','state':'held'}))
elif a[0] == 'later': print(json.dumps({'task':19,'outcome':'accepted','state':'later'}))
else: sys.exit(1)
"#;
    script(&d,"record-taskq",""); fs::write(d.join("record-taskq"),fake).unwrap();
    let mut h = Run {
        report:d.join("report.json"),root,data:json!({"pass":false,"original_focus":"all\n","cases":{}}),
        login:Some(Login(vec![d.join("codex/auth.json")])), d:d.clone(),st,svc:None,update:0,task:0,run_request:0,
        deadline:Instant::now()+Duration::from_secs(10),
    };
    for c in CASES { h.data["cases"][c]=json!({"status":"not_run"}); }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||h.case("discussion", |_|panic!("injected missing native answer"))));
    h.finish(result);
    let report: Value=serde_json::from_slice(&fs::read(&h.report).unwrap()).unwrap();
    assert_eq!(report["pass"],false);
    assert_eq!(report["cases"]["discussion"]["status"],"failed");
    assert_eq!(report["cases"]["reviewer"]["status"],"not_run");
    assert_eq!(report["cases"]["cleanup"]["status"],"pass");
    assert_eq!(report["cleanup_tasks"][0]["state"],"later");
    assert!(!d.join("codex/auth.json").exists());
    assert_eq!(fs::read_to_string(h.root.join(".taskq/focus")).unwrap(),"all\n");
    let calls:Vec<Value>=fs::read_to_string(d.join("cleanup-calls.jsonl")).unwrap().lines().map(|l|serde_json::from_str(l).unwrap()).collect();
    assert!(!calls.iter().any(|a|a[0]=="add"),"a durable accepted receipt identifies the task without replay");
    assert_eq!(calls.iter().filter(|a|a[0]=="later").count(),1);
    // Lost receipts still reconcile the exact claim, never a new operation or plain add.
    fs::write(d.join("allow-reconcile"), "").unwrap();
    fs::write(d.join("taskq-calls.jsonl"), format!("{}\n",json!({"args":add}))).unwrap();
    h.cleanup().unwrap();
    let calls:Vec<Value>=fs::read_to_string(d.join("cleanup-calls.jsonl")).unwrap().lines().map(|l|serde_json::from_str(l).unwrap()).collect();
    assert_eq!(calls.iter().find(|a|a[0]=="add").unwrap(), &add);
    fs::write(d.join("taskq-calls.jsonl"), format!("{}\n",json!({"args":["service","pause"]}))).unwrap();
    // Concurrent operator changes are unknown ownership: report failure without restoring over them.
    fs::write(h.root.join(".taskq/focus.paused"), "different-focus").unwrap();
    fs::write(h.root.join(".taskq/focus"), "off").unwrap();
    assert!(h.cleanup().unwrap_err().contains("queue configuration changed"));
    assert_eq!(fs::read_to_string(h.root.join(".taskq/focus.paused")).unwrap(), "different-focus");
    assert_eq!(fs::read_to_string(h.root.join(".taskq/focus")).unwrap(), "off");
    fs::remove_dir_all(d).unwrap();
}
