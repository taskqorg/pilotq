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
base = pathlib.Path(__file__).resolve().parent
scope = json.loads(base.joinpath('taskq-scope.json').read_text())
for key, name in [('TASKQ_INSTALL', 'install'), ('TASKQ_ROOT', 'root')]:
    target = pathlib.Path(scope[name]).resolve(strict=True)
    if str(target) != scope[name]: sys.exit('noncanonical A40 scope')
    if key in os.environ and pathlib.Path(os.environ[key]).resolve() != target:
        sys.exit('foreign A40 ' + name)
# Reject explicit foreign project directories before any receipt or dispatch.
for parent in [pathlib.Path.cwd(), *pathlib.Path.cwd().parents]:
    if parent.joinpath('taskq.json').exists():
        if parent != pathlib.Path(scope['root']): sys.exit('foreign A40 project')
        break
if sys.argv[1:] == ['--a40-path-probe']:
    print(json.dumps(scope)); sys.exit(0)
env = {k:v for k,v in os.environ.items() if not k.startswith('TASKQ_')}
env.update(TASKQ_INSTALL=scope['install'], TASKQ_ROOT=scope['root'])
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
p = subprocess.run([str(base.joinpath('taskq-path').read_text().strip()), *args], capture_output=True, timeout=120, env=env, cwd=scope['root'])
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
    fixture_token: Option<Login>,
    update: i64,
    task: i64,
    run_request: i64,
    deadline: Instant,
}

pub(super) fn pause_tool_status(d: &Path, id: i64) -> String {
    query_text(d, &format!("SELECT result FROM audit_events WHERE request_id={id} AND kind='tool' AND target='dynamicToolCall' AND result='completed' AND EXISTS (SELECT 1 FROM audit_events WHERE request_id={id} AND kind='call' AND target='queue_pause') ORDER BY id DESC LIMIT 1"))
}

fn request_delivered(d: &Path, update: i64) -> bool {
    try_query(d, &format!("SELECT count(*) FROM requests q WHERE update_id={update} AND state IN ('answered','failed','uncertain') AND EXISTS (SELECT 1 FROM outbox o WHERE o.request_id=q.id AND (o.kind IN ('rich','rich_part','plan_answered','plan_problem') OR (o.kind='reply' AND o.text=q.reply)) AND o.state='delivered') AND NOT EXISTS (SELECT 1 FROM outbox o WHERE o.request_id=q.id AND o.kind IN ('rich','rich_part','plan_answered','plan_problem') AND o.state NOT IN ('delivered','superseded'))")) == Some(1)
}

// §5 / A40: observe the current delivered question, never a historical text hit.
pub(super) fn current_question(d: &Path, messages: &[Value], task: i64) -> Option<(i64, i64)> {
    let revision = try_query(d, &format!("SELECT revision FROM tasks WHERE project='pilotq' AND number={task} AND state='ask' AND closed=0"))?;
    let id = try_query(d, &format!("SELECT sent_message FROM outbox WHERE project='pilotq' AND task={task} AND revision={revision} AND kind='question' AND state='delivered' AND chat={CHAT} AND topic=5 ORDER BY id DESC LIMIT 1"))?;
    let message = messages.iter().rev().find(|m| m["id"] == id)?;
    (message["chat_id"] == CHAT && message["message_thread_id"] == 5
        && message["text"].as_str()?.starts_with("? "))
        .then_some((id, revision))
}

// R8 / §5: regression for run7; canonical question receipt plus exact revision/route.
#[test]
fn a40_current_question_delivery() {
    let d = std::env::temp_dir().join(format!("pilotq-a40-question-{}", std::process::id()));
    fs::create_dir_all(&d).unwrap();
    let c = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    c.execute_batch("DROP TABLE IF EXISTS tasks; DROP TABLE IF EXISTS outbox;
        CREATE TABLE tasks(project TEXT, number INTEGER, state TEXT, closed INTEGER, revision INTEGER);
        CREATE TABLE outbox(project TEXT, task INTEGER, revision INTEGER, kind TEXT, state TEXT, chat INTEGER, topic INTEGER, id INTEGER, sent_message INTEGER);
        INSERT INTO tasks VALUES('pilotq',19,'ask',0,42);
        INSERT INTO outbox VALUES('pilotq',19,42,'question','delivered',-100,5,1,77);").unwrap();
    let mut messages = vec![json!({"id":77,"chat_id":CHAT,"message_thread_id":5,"text":"? <a href=\"https://github.com/taskqorg/pilotq/issues/19\">pilotq#19 — Docs</a>\n✓ drafted\n⚑ choose\n1. short ★\n2. long"})];
    assert_eq!(current_question(&d, &messages, 19), Some((77,42)), "current canonical card must be reachable");
    for sql in ["UPDATE outbox SET state='pending'", "UPDATE outbox SET revision=41", "UPDATE outbox SET task=20", "UPDATE outbox SET project='other'", "UPDATE outbox SET kind='status'", "UPDATE outbox SET topic=6", "UPDATE tasks SET state='doing'", "UPDATE tasks SET closed=1"] {
        c.execute_batch(sql).unwrap();
        assert_eq!(current_question(&d, &messages, 19), None, "{sql}");
        c.execute_batch("UPDATE tasks SET state='ask', closed=0;
            UPDATE outbox SET project='pilotq', task=19, revision=42, kind='question', state='delivered', topic=5;").unwrap();
    }
    messages[0]["message_thread_id"] = json!(6);
    assert_eq!(current_question(&d, &messages, 19), None);
    messages[0]["message_thread_id"] = json!(5);
    messages[0]["id"] = json!(78);
    assert_eq!(current_question(&d, &messages, 19), None);
    drop(c);
    fs::remove_dir_all(d).unwrap();
}

// §4 / A40: final delivery must not be confused with a privacy notice or a partial fallback.
#[test]
fn a40_final_reply_delivery() {
    let d = std::env::temp_dir().join(format!("pilotq-a40-final-{}", std::process::id()));
    fs::create_dir_all(&d).unwrap();
    let c = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    c.execute_batch("DROP TABLE IF EXISTS requests; DROP TABLE IF EXISTS outbox;
        CREATE TABLE requests(id INTEGER, update_id INTEGER, state TEXT, reply TEXT);
        CREATE TABLE outbox(request_id INTEGER, kind TEXT, state TEXT, text TEXT);
        INSERT INTO requests VALUES(1,1,'answered','Final answer');
        INSERT INTO outbox VALUES(1,'rich','delivered','Final answer');").unwrap();
    assert!(request_delivered(&d, 1), "rich final without a privacy notice");
    c.execute_batch("UPDATE outbox SET state='pending';
        INSERT INTO outbox VALUES(1,'reply','delivered','Your requests are recorded');").unwrap();
    assert!(!request_delivered(&d, 1), "privacy notice alone is not final delivery");
    c.execute_batch("UPDATE outbox SET state='superseded' WHERE kind='rich';
        INSERT INTO outbox VALUES(1,'rich_part','delivered','Final ');
        INSERT INTO outbox VALUES(1,'rich_part','pending','answer');").unwrap();
    assert!(!request_delivered(&d, 1), "every fallback chunk needs a receipt");
    c.execute("UPDATE outbox SET state='delivered' WHERE kind='rich_part'", []).unwrap();
    assert!(request_delivered(&d, 1), "complete HTML/plain fallback");
    c.execute_batch("DELETE FROM outbox; INSERT INTO outbox VALUES(1,'reply','delivered','Final answer');").unwrap();
    assert!(request_delivered(&d, 1), "plain final reply");
    c.execute_batch("DELETE FROM outbox; INSERT INTO outbox VALUES(1,'plan_progress','delivered','Creating tasks');").unwrap();
    assert!(!request_delivered(&d, 1), "#165 progress is not a launch receipt");
    c.execute_batch("INSERT INTO outbox VALUES(1,'plan_answered','pending','Tasks submitted');").unwrap();
    assert!(!request_delivered(&d, 1), "#165 pending source-card edit is not delivered");
    c.execute("UPDATE outbox SET state='delivered' WHERE kind='plan_answered'", []).unwrap();
    assert!(request_delivered(&d, 1), "#165 source-card receipt is final delivery");
    c.execute("UPDATE requests SET state='running'", []).unwrap();
    assert!(!request_delivered(&d, 1), "delivery cannot replace terminal state");
    drop(c);
    fs::remove_dir_all(d).unwrap();
}

// §2, §4–5 / A40: owner DMs need exact onboarding or independently owned board-event receipts.
fn routed_message(d: &Path, m: &Value, launched_task: i64) -> bool {
    let (Some(id), Some(chat)) = (m["id"].as_i64(), m["chat_id"].as_i64()) else { return false; };
    let topic = match m.get("message_thread_id") {
        None | Some(Value::Null) => None,
        Some(value) => match value.as_i64() { Some(topic) => Some(topic), None => return false },
    };
    let c = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    let mut stmt=c.prepare("SELECT o.text,o.html FROM outbox o JOIN requests q ON q.id=o.request_id
        WHERE o.sent_message=?1 AND o.chat=?2 AND o.topic IS ?3 AND o.state='delivered'
        AND ((o.chat=-100 AND o.topic=5 AND q.chat=-100 AND q.topic=5)
          OR (o.chat=7 AND o.topic IS NULL AND o.kind='onboarding' AND o.project='pilotq'
            AND q.front='service' AND q.kind='onboarding' AND q.state='answered'
            AND q.sender=7 AND q.chat=7 AND q.topic IS NULL AND q.update_id<0)
          OR (o.chat=7 AND o.topic IS NULL AND o.kind IN ('status','question')
            AND q.front='telegram' AND q.kind='board_event' AND q.state='answered'
            AND q.sender=7 AND q.chat=7 AND q.topic IS NULL AND q.update_id<0
            AND EXISTS(SELECT 1 FROM tasks t JOIN audit_events a ON a.request_id=t.request_id
              WHERE t.request_id=q.id AND t.project='pilotq' AND t.number<>?4
              AND ((t.project=o.project AND t.number=o.task)
                OR (o.project='' AND o.task=0 AND EXISTS(SELECT 1 FROM outbox original
                  WHERE original.request_id=o.request_id AND original.sent_message=o.sent_message
                  AND original.chat=o.chat AND original.topic IS o.topic AND original.state='delivered'
                  AND original.kind=o.kind AND original.project=t.project AND original.task=t.number)))
              AND t.chat=o.chat AND t.topic IS o.topic AND q.text=t.project||'#'||t.number
              AND a.kind='board_event' AND a.target=q.text)))").unwrap();
    stmt.query_map(rusqlite::params![id,chat,topic,launched_task], |r| Ok((r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?))).unwrap()
        .any(|row| {
            let (text,html)=row.unwrap();
            let expected=if m.get("rich_message").is_some() {
                telegram_text::markdown(&text).replace("  \n","\n")
            } else if m["parse_mode"]=="HTML" {
                html.unwrap_or_else(||telegram_text::html(&text))
            } else { text };
            m["text"].as_str()==Some(expected.as_str())
        })
}

#[test]
fn a40_final_route_receipts() {
    let d = std::env::temp_dir().join(format!("pilotq-a40-route-{}", std::process::id()));
    fs::create_dir_all(&d).unwrap();
    let c = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    c.execute_batch("DROP TABLE IF EXISTS requests; DROP TABLE IF EXISTS outbox; DROP TABLE IF EXISTS tasks; DROP TABLE IF EXISTS audit_events;
      CREATE TABLE requests(id INTEGER,front TEXT,kind TEXT,state TEXT,sender INTEGER,chat INTEGER,topic INTEGER,update_id INTEGER,text TEXT);
      CREATE TABLE tasks(request_id INTEGER,project TEXT,number INTEGER,chat INTEGER,topic INTEGER);
      CREATE TABLE audit_events(request_id INTEGER,kind TEXT,target TEXT);
      CREATE TABLE outbox(request_id INTEGER,sent_message INTEGER,chat INTEGER,topic INTEGER,state TEXT,kind TEXT,project TEXT,text TEXT,html TEXT,task INTEGER);
      INSERT INTO requests VALUES(1,'service','onboarding','answered',7,7,NULL,-1,'Onboard pilotq');
      INSERT INTO outbox VALUES(1,903,7,NULL,'delivered','onboarding','pilotq','Onboard pilotq',NULL,NULL);").unwrap();
    let mut m=json!({"id":903,"chat_id":7,"text":"Onboard pilotq"});
    assert!(routed_message(&d,&m,298), "run7 verified owner onboarding is not a topic leak");
    for (key,value) in [("id",json!(904)),("chat_id",json!(8)),("message_thread_id",json!(5)),("text",json!("foreign content"))] {
        let mut wrong=m.clone(); wrong[key]=value;
        assert!(!routed_message(&d,&wrong,298), "reject changed {key}");
    }
    for sql in ["UPDATE requests SET front='telegram'", "UPDATE requests SET sender=8", "UPDATE requests SET update_id=1", "UPDATE requests SET kind='text'", "UPDATE requests SET state='running'", "UPDATE outbox SET kind='reply'", "UPDATE outbox SET state='pending'", "UPDATE outbox SET project='other'"] {
        c.execute_batch(sql).unwrap();
        assert!(!routed_message(&d,&m,298), "{sql}");
        c.execute_batch("UPDATE requests SET front='service',sender=7,update_id=-1,kind='onboarding',state='answered'; UPDATE outbox SET kind='onboarding',state='delivered',project='pilotq'").unwrap();
    }
    c.execute_batch("INSERT INTO requests VALUES(2,'telegram','text','answered',7,-100,5,2,'Run task');
      INSERT INTO outbox VALUES(2,907,-100,5,'delivered','status','pilotq','Task closed; see the result',NULL,298);").unwrap();
    let topic=json!({"id":907,"chat_id":CHAT,"message_thread_id":5,"text":"Task closed; see the result"});
    assert!(routed_message(&d,&topic,298));
    let mut wrong=topic.clone(); wrong["text"]=json!("foreign content");
    assert!(!routed_message(&d,&wrong,298), "topic content also needs its exact receipt");
    wrong=topic.clone(); wrong["message_thread_id"]=json!(6);
    assert!(!routed_message(&d,&wrong,298), "foreign topic is never admitted");
    c.execute_batch("INSERT INTO outbox VALUES(2,907,-100,5,'delivered','status','pilotq','Task is doing',NULL,298);").unwrap();
    let mut historical=topic.clone(); historical["text"]=json!("Task is doing"); historical["edit"]=json!(true);
    assert!(routed_message(&d,&historical,298), "historical edits retain their own exact receipt content");
    for text in ["Task closed; see the result", "Read docs/my file.md & <raw>\n[task](https://example.invalid)"] {
        c.execute("UPDATE outbox SET text=?1 WHERE sent_message=907",[text]).unwrap();
        let mut rich=topic.clone();
        let markdown=telegram_text::markdown(text);
        rich["rich_message"]=json!({"markdown":markdown});
        rich["text"]=json!(markdown.replace("  \n","\n"));
        assert!(routed_message(&d,&rich,298), "exact rich rendering");
        rich["text"]=json!("foreign content");
        assert!(!routed_message(&d,&rich,298));
        let mut html=topic.clone(); html["parse_mode"]=json!("HTML"); html["text"]=json!(telegram_text::html(text));
        assert!(routed_message(&d,&html,298), "exact HTML fallback");
    }
    c.execute_batch("INSERT INTO requests VALUES(12,'telegram','board_event','answered',7,7,NULL,-12,'pilotq#294');
      INSERT INTO tasks VALUES(12,'pilotq',294,7,NULL);
      INSERT INTO audit_events VALUES(12,'board_event','pilotq#294');
      INSERT INTO outbox VALUES(12,921,7,NULL,'delivered','status','pilotq','Task 294 status',NULL,294);").unwrap();
    let event=json!({"id":921,"chat_id":7,"text":"Task 294 status"});
    assert!(routed_message(&d,&event,298), "run7 board observer owns its owner-DM fallback");
    // Run7 outbox39 owns message921; refreshes42/43/44 omit project/task metadata.
    for (row, text) in [(42,"Task 294 refreshed"),(43,"Task 294 reviewed"),(44,"Task 294 closed")] {
        c.execute("INSERT INTO outbox VALUES(12,921,7,NULL,'delivered','status','','temporary',NULL,0)", []).unwrap();
        c.execute("UPDATE outbox SET text=?1 WHERE rowid=(SELECT max(rowid) FROM outbox)",[text]).unwrap();
        let refresh=json!({"id":921,"chat_id":7,"text":text});
        assert!(routed_message(&d,&refresh,298), "run7 refresh outbox{row} uses original ownership receipt");
        for sql in ["UPDATE outbox SET state='pending' WHERE task=294", "UPDATE outbox SET sent_message=922 WHERE task=294", "UPDATE outbox SET request_id=13 WHERE task=294", "UPDATE outbox SET chat=8 WHERE task=294", "UPDATE outbox SET topic=5 WHERE task=294", "UPDATE outbox SET kind='question' WHERE task=294"] {
            c.execute_batch(sql).unwrap();
            assert!(!routed_message(&d,&refresh,298), "refresh must retain original identity: {sql}");
            c.execute_batch("UPDATE outbox SET state='delivered',sent_message=921,request_id=12,chat=7,topic=NULL,kind='status' WHERE task=294").unwrap();
        }
        let mut wrong=refresh.clone(); wrong["text"]=json!("task 298 leaked");
        assert!(!routed_message(&d,&wrong,298));

    }
    for sql in ["UPDATE tasks SET request_id=13", "UPDATE tasks SET number=298", "UPDATE tasks SET chat=-100", "UPDATE tasks SET topic=5", "UPDATE tasks SET project='other'", "UPDATE audit_events SET target='pilotq#298'", "UPDATE requests SET text='pilotq#298' WHERE id=12", "UPDATE requests SET sender=8 WHERE id=12", "UPDATE requests SET update_id=12 WHERE id=12", "UPDATE outbox SET task=298 WHERE sent_message=921"] {
        c.execute_batch(sql).unwrap();
        assert!(!routed_message(&d,&event,298), "reject unowned service destination: {sql}");
        c.execute_batch("UPDATE tasks SET request_id=12,number=294,chat=7,topic=NULL,project='pilotq'; UPDATE audit_events SET target='pilotq#294'; UPDATE requests SET text='pilotq#294',sender=7,update_id=-12 WHERE id=12; UPDATE outbox SET task=294 WHERE sent_message=921").unwrap();
    }
    c.execute_batch("UPDATE tasks SET number=298; UPDATE audit_events SET target='pilotq#298'; UPDATE requests SET text='pilotq#298' WHERE id=12; UPDATE outbox SET task=298 WHERE sent_message=921").unwrap();
    assert!(!routed_message(&d,&event,298), "the launched task can never use owner-DM fallback");
    c.execute_batch("UPDATE tasks SET number=294; UPDATE audit_events SET target='pilotq#294'; UPDATE requests SET text='pilotq#294' WHERE id=12; UPDATE outbox SET task=294 WHERE sent_message=921").unwrap();
    let mut wrong=event.clone(); wrong["text"]=json!("task 298 content");
    assert!(!routed_message(&d,&wrong,298));
    wrong=event.clone(); wrong["chat_id"]=json!(-100); wrong["message_thread_id"]=json!(6);
    assert!(!routed_message(&d,&wrong,298));
    m["id"]=json!(910); m["text"]=Value::Null;
    assert!(!routed_message(&d,&m,298), "run7 phantom delete is not a delivery");
    drop(c); fs::remove_dir_all(d).unwrap();
}

// R10 / §2: only the owner's §5 startup onboarding may precede the first admitted input.
// Scope stranger checks to ingress identity, rather than suppressing legitimate service work.
fn assert_stranger_ignored(d: &Path, st: &State, update: i64) {
    assert_eq!(query(d, &format!("SELECT count(*) FROM updates WHERE update_id={update} AND disposition='ignored' AND request_id IS NULL")), 1);
    assert_eq!(query(d, &format!("SELECT count(*) FROM requests WHERE update_id={update} OR sender=9")), 0);
    assert_eq!(query(d, "SELECT count(*) FROM requests WHERE NOT (front='service' AND kind='onboarding' AND state='answered' AND sender=7 AND chat=7 AND topic IS NULL AND update_id<0)"), 0);
    assert_eq!(query(d, "SELECT count(*) FROM outbox WHERE kind<>'onboarding' OR chat<>7 OR topic IS NOT NULL"), 0);
    assert_eq!(query(d, "SELECT count(*) FROM runs"), 0, "stranger must not start a model");
    assert_eq!(query(d, "SELECT count(*) FROM plans"), 0);
    assert_eq!(query(d, "SELECT count(*) FROM tickets"), 0, "stranger must not file a task");
    let f = st.lock().unwrap();
    assert!(f.sent.iter().all(|m| m["chat_id"] == OWNER && m["message_thread_id"].is_null()), "only owner onboarding messages are allowed");
    assert!(f.actions.is_empty(), "stranger must not trigger typing");
    assert!(f.reactions.is_empty(), "stranger must not trigger a reaction");
}

// R10 / §5: exercise the admission fixture with real startup onboarding and fake dependencies.
#[test]
fn a40_admission_startup_onboarding() {
    let (d, st) = tq_setup("a40-admission-startup", true);
    let db = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    db.execute("DELETE FROM meta WHERE key='onboarding:pilotq'", []).unwrap();
    drop(db);
    let svc = spawn(&d, None);
    wait("startup onboarding", 15, || try_query(&d, "SELECT count(*) FROM requests WHERE kind='onboarding' AND state='answered'") == Some(1));
    push(&st, text(1, CHAT, TOPIC, 9, "Do not admit this synthetic stranger."));
    wait("ignored stranger", 15, || try_query(&d, "SELECT count(*) FROM updates WHERE update_id=1 AND disposition='ignored' AND request_id IS NULL") == Some(1));
    assert_stranger_ignored(&d, &st, 1);
    assert!(!fs::read_to_string(d.join("codex.log")).unwrap_or_default().contains("turn/start"));
    for line in fs::read_to_string(d.join("taskq.log")).unwrap_or_default().lines() {
        let call: Value = serde_json::from_str(line).unwrap();
        assert_ne!(call["args"][0], "add", "stranger must not call TaskQ add");
    }
    let removed = d.clone();
    stop(svc, d);
    assert!(!removed.exists(), "A40 fixture survived teardown");
}

// §5: prepare only the synthetic install through the protected owner's supported control.
// Keep actual findings: an override permits launch, never asserts that setup passed.
fn prepare_readiness(d: &Path) -> Value {
    wait("completed startup findings", 30, || try_query(d, "SELECT count(*) FROM meta WHERE key='onboarding:pilotq' AND json_extract(CAST(value AS TEXT),'$.pending')=0") == Some(1));
    let saved = || -> Value {
        serde_json::from_str(&query_text(d, "SELECT CAST(value AS TEXT) FROM meta WHERE key='onboarding:pilotq'")).unwrap()
    };
    let before = saved();
    assert_eq!(before["pending"], false);
    assert!(before["report"].is_object(), "startup findings required");
    let receipt = ok(cli(d, &["ask", "/onboard pilotq override", "--json"]));
    assert_eq!(receipt["state"], "answered");
    let after = saved();
    for key in ["board", "root", "epoch", "pending", "ready", "report"] {
        assert_eq!(after[key], before[key], "override changed {key}");
    }
    assert_eq!(after["override"]["actor"], OWNER);
    assert_eq!(after["override"]["request"], receipt["request"]);
    assert_eq!(after["override"]["report"], before["report"]);
    assert_eq!(query(d, "SELECT count(*) FROM audit_events WHERE kind='onboarding_override' AND target='pilotq'"), 1);
    json!({"request":receipt["request"], "ready":after["ready"], "report":after["report"], "override":after["override"]})
}

// §5 / #288: real startup findings must not prevent this synthetic management fixture
// from reaching create/start; R10 admission still runs before owner preparation.
#[test]
fn a40_readiness_create_start() {
    let (d, st) = tq_setup("a40-readiness-create", true);
    let db = rusqlite::Connection::open(d.join("journal.sqlite")).unwrap();
    db.execute("DELETE FROM meta WHERE key='onboarding:pilotq'", []).unwrap();
    drop(db);
    let svc = spawn(&d, None);
    wait("startup onboarding", 15, || try_query(&d, "SELECT count(*) FROM requests WHERE kind='onboarding' AND state='answered'") == Some(1));
    push(&st, text(1, CHAT, TOPIC, 9, "Do not admit this synthetic stranger."));
    wait("ignored stranger", 15, || try_query(&d, "SELECT count(*) FROM updates WHERE update_id=1 AND disposition='ignored'") == Some(1));
    assert_stranger_ignored(&d, &st, 1);
    let evidence = prepare_readiness(&d);
    assert_eq!(evidence["ready"], false, "fixture must retain real gaps");
    assert!(!evidence["report"]["gaps"].as_array().unwrap().is_empty());
    assert!(!tq_calls(&d).iter().any(|a| a[0] == "add"));
    push(&st, call(2, CHAT, TOPIC, OWNER, "task_plan", plan(&["A40 synthetic docs"])));
    wait("saved plan", 15, || try_query(&d, "SELECT count(*) FROM plans WHERE revision=1") == Some(1));
    push(&st, call(3, CHAT, TOPIC, OWNER, "task_run_plan", json!({"revision":1})));
    wait("launch outcome", 15, || try_query(&d, "SELECT count(*) FROM requests WHERE update_id=3 AND state IN ('answered','failed')") == Some(1));
    assert_eq!(query(&d, "SELECT count(*) FROM plan_runs WHERE state='started'"), 1, "{:?}", texts(&st));
    assert_eq!(query(&d, "SELECT count(*) FROM tickets WHERE started=1 AND task IS NOT NULL"), 1);
    assert!(tq_calls(&d).iter().any(|a| a[0] == "add"));
    assert!(tq_calls(&d).iter().any(|a| a[0] == "start"));
    let removed = d.clone();
    stop(svc, d);
    assert!(!removed.exists(), "A40 fixture survived teardown");
}

impl Run {
    fn status_message(&self, message: &Value, label: &str) -> bool {
        let id=try_query(&self.d,&format!("SELECT sent_message FROM outbox WHERE request_id={} AND kind='status' AND state='delivered' ORDER BY id LIMIT 1",self.run_request));
        id.is_some_and(|id|message["id"]==id) && message["chat_id"]==CHAT && message["message_thread_id"]==5
            && message["text"].as_str().is_some_and(|t|t.contains(label) && t.contains(self.data["task_url"].as_str().unwrap()))
    }

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

    fn host_body(&self) -> String {
        format!("A40 explicitly synthetic unserved fixture: {}\nfocus off\nNo service, worker or reviewer exists. Never enroll this host.", self.data["isolated_host"].as_str().unwrap())
    }

    fn validate_host(&self, row: &Value) {
        let host = self.data["isolated_host"].as_str().unwrap();
        assert!(host.starts_with("pilotq-e2e-"), "foreign host namespace");
        assert_eq!(row["repository_url"], "https://api.github.com/repos/taskqorg/pilotq", "foreign host project");
        assert_eq!(row["title"], format!("taskq host {host}"), "foreign host title");
        assert_eq!(row["body"], self.host_body(), "foreign reserved host metadata");
        assert!(row["labels"].as_array().unwrap().iter().any(|v| v == "taskq-host" || v["name"] == "taskq-host"), "missing host label");
        assert!(row["number"].as_u64().is_some_and(|n| n > 0));
    }

    // Listing is discovery only, including uncertain POST reconciliation. Never proves absence.
    fn fixture_hosts(&self) -> Vec<Value> {
        let host = self.data["isolated_host"].as_str().unwrap();
        assert!(host.starts_with("pilotq-e2e-"), "foreign host namespace");
        let title = format!("taskq host {host}");
        let mut found = vec![];
        for page in 1..=10 {
            let rows = gh_json(&format!("repos/taskqorg/pilotq/issues?state=all&labels=taskq-host&per_page=100&page={page}"));
            let rows = rows.as_array().expect("complete host listing");
            for row in rows.iter().filter(|r| r["title"] == title) {
                self.validate_host(row);
                found.push(row.clone());
            }
            if rows.len() < 100 {
                assert!(found.len() <= 1, "ambiguous reserved host metadata");
                return found;
            }
        }
        panic!("host reconciliation listing exceeded ten pages");
    }

    fn exact_host(&mut self) -> Value {
        let id = self.data["host_metadata"].as_u64().filter(|n| *n > 0).expect("unresolved host identity");
        let row = gh_json(&format!("repos/taskqorg/pilotq/issues/{id}"));
        self.data["host_metadata_readback"] = row.clone();
        self.save();
        self.validate_host(&row);
        assert_eq!(row["number"], id, "foreign host issue identity");
        row
    }

    fn prepare_host(&mut self) {
        if self.data["host_metadata"].as_u64().is_none() {
            let prior_intent = self.data["host_metadata_intent"] == true;
            let rows = self.fixture_hosts();
            self.data["host_metadata_intent"] = json!(true);
            self.save();
            let row = rows.into_iter().next().unwrap_or_else(|| {
                assert!(!prior_intent, "unresolved host POST; do not retry");
                self.github("POST", "repos/taskqorg/pilotq/issues",
                    json!({"title":format!("taskq host {}",self.data["isolated_host"].as_str().unwrap()),
                        "body":self.host_body(),"labels":["taskq-host"]}))
            });
            self.data["host_metadata"] = row["number"].clone();
            self.data["host_metadata_create_receipt"] = row;
            self.save();
        }
        assert_eq!(self.exact_host()["state"], "open", "retired host cannot be reused");
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
        self.svc = Some(Svc(service(&self.d, None).env("HOME", self.d.join("home")).env_remove("TASKQ_INSTALL").stderr(log).spawn().unwrap()));
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
            request_delivered(&self.d, u)
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
            .args(args).env("TASKQ_ROOT", &self.root).env_remove("TASKQ_INSTALL").env_remove("TASKQ_ATTEMPT")
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
        // Keep host validation available until all interrupted adds and task parks are reconciled.
        if errors.is_empty() && self.data["host_metadata_intent"] == true {
            self.data["host_metadata_retired"] = json!(false);
            self.save();
            if self.data["host_metadata"].as_u64().is_none() {
                let rows = self.fixture_hosts();
                if let Some(row) = rows.first() {
                    self.data["host_metadata"] = row["number"].clone();
                    self.save();
                } else {
                    errors.push("unresolved host POST; missing listing is not retirement".into());
                }
            }
            if self.data["host_metadata"].as_u64().is_some() {
                let row = self.exact_host();
                let path = format!("repos/taskqorg/pilotq/issues/{}", row["number"]);
                if row["state"] != "closed" {
                    let receipt = self.github("PATCH", &path, json!({"state":"closed","state_reason":"completed"}));
                    self.data["host_metadata_retirement_receipt"] = receipt;
                    self.save();
                }
                if self.exact_host()["state"] == "closed" {
                    self.data["host_metadata_retired"] = json!(true);
                } else {
                    errors.push("fixture host retirement exact readback is not closed".into());
                }
            }
            self.save();
        }
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
        // Run even if board reconciliation panics; remove only the validated disposable token copy.
        let token_cleanup = self.data["fixture_install"].as_str().map(|install| {
            let token = Path::new(install).join("token");
            match fs::remove_file(&token) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(_) => Err("copied board credential remains".to_string()),
            }
        }).unwrap_or(Ok(()));
        let cleanup = match token_cleanup { Ok(()) => cleanup, Err(e) => Ok(Err(e)) };
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
        svc:None, login:None, fixture_token:None, update:0, task:0, run_request:0, deadline:Instant::now() + Duration::from_secs(10800),
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
        // §9: the PM provisions a disposable TaskQ installation and checkout together.
        // Never infer fixture ownership from an ordinary installation's environment.
        let install = PathBuf::from(std::env::var("PILOTQ_A40_TASKQ_INSTALL").expect("PILOTQ_A40_TASKQ_INSTALL"));
        let fixture = install.parent().expect("fixture parent").canonicalize().unwrap();
        assert!(fixture.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(fixture.file_name().unwrap().to_string_lossy().starts_with("pilotq-e2e-"));
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::metadata(&fixture).unwrap();
        assert_eq!(metadata.mode() & 0o777, 0o700, "fixture must be private");
        assert_eq!(metadata.uid(), fs::metadata(&h.d).unwrap().uid(), "foreign fixture owner");
        let install = install.canonicalize().unwrap();
        h.root = h.root.canonicalize().unwrap();
        assert_eq!(install.parent(), Some(fixture.as_path()));
        assert_eq!(h.root.parent(), Some(fixture.as_path()));
        assert_ne!(install, h.root);
        let projects = fs::read_to_string(install.join("projects")).expect("fixture project registry");
        assert_eq!(projects.trim(), h.root.to_str().unwrap(), "foreign fixture projects");
        h.fixture_token = Some(Login(vec![install.join("token")]));
        h.data["fixture_install"] = json!(install);
        h.save();
        fs::write(h.d.join("taskq-scope.json"), json!({"install":install,"root":h.root}).to_string()).unwrap();
        std::os::unix::fs::symlink(h.d.join("record-taskq"), h.d.join("taskq")).unwrap();
        let home_bin = h.d.join("home/.local/share/taskq");
        fs::create_dir_all(&home_bin).unwrap();
        std::os::unix::fs::symlink(h.d.join("record-taskq"), home_bin.join("taskq")).unwrap();
        // Model children receive only manager.rs's production allowlist. Probe that boundary
        // without a model call, token access, board read or service mutation.
        let native = Command::new("/bin/sh").args(["-c", "taskq --a40-path-probe"])
            .env_clear().env("PATH", format!("{}:{}", h.d.display(), std::env::var("PATH").unwrap_or_default()))
            .env("HOME", h.d.join("home")).current_dir(&h.d).output().unwrap();
        assert!(native.status.success(), "native A40 preflight failed");
        let scope: Value = serde_json::from_slice(&native.stdout).unwrap();
        assert_eq!(scope, json!({"install":install,"root":h.root}));
        h.data["native_taskq_scope"] = scope;
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
        h.prepare_host();
        h.save();
        h.start();
        h.case("admission", |h| {
            h.update += 1;
            push(&h.st, text(h.update, CHAT, TOPIC, 9, "Do not admit this synthetic stranger."));
            h.wait("ignored person", 30, || try_query(&h.d, "SELECT count(*) FROM updates WHERE update_id=1 AND disposition='ignored'") == Some(1));
            h.wait("startup onboarding", 30, || try_query(&h.d, "SELECT count(*) FROM requests WHERE front='service' AND kind='onboarding' AND state='answered'") == Some(1));
            assert_stranger_ignored(&h.d, &h.st, h.update);
            assert!(!h.calls().iter().any(|c| c["args"][0] == "add"));
            let onboarding = query(&h.d, "SELECT id FROM requests WHERE front='service' AND kind='onboarding'");
            ok(cli(&h.d, &["users","grant","8","chat"]));
            ok(cli(&h.d, &["routes","set","-100:5","shared"]));
            let (id, _) = h.request(8, "Reply hello in one word; do not call a tool.", None);
            assert_eq!(query(&h.d, &format!("SELECT sender FROM requests WHERE id={id}")), 8);
            json!({"ignored_update":1,"startup_onboarding_request":onboarding,"second_user_request":id})
        });
        // R10 admission is already verified. Prepare the private fixture before planning/launch;
        // this changes neither host setup nor TaskQ admission checks (§5).
        h.data["readiness_preparation"] = prepare_readiness(&h.d);
        h.save();
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
            let revision = h.event(&format!("taskq: ask attempt {attempt}\nDone: Scripted A40 fixture; no implementation requested.\nProblem: Choose wording for this synthetic navigation check.\n1. Short link ★\n2. Descriptive link\nTo answer: `taskq answer {}.K`",h.task), "ask");
            h.wait("worker question (required)",1800,|| current_question(&h.d, &sent(&h.st), h.task).is_some_and(|(_,r)| r == revision));
            question = current_question(&h.d, &sent(&h.st), h.task).unwrap().0;
            let count = h.calls().iter().filter(|c| c["args"][0] == "answer").count();
            let (id, reply) = h.request(8,"1",Some(question));
            assert_eq!(reply, "Answer was not saved; check the current question and your access.");
            assert_eq!(query_text(&h.d,&format!("SELECT error FROM requests WHERE id={id}")), "not authorized");
            assert_eq!(h.calls().iter().filter(|c| c["args"][0] == "answer").count(),count);
            json!({"message":question,"revision":revision,"foreign_request":id,"refusal":reply})
        });
        h.case("answer_stale", |h| {
            let attempt = h.data["fixture_attempt"].as_i64().unwrap();
            let revision = h.event(&format!("taskq: ask attempt {attempt}\nDone: Scripted wording revised.\nProblem: Choose current wording.\n1. Short link ★\n2. Descriptive link\nTo answer: `taskq answer {}.K`",h.task), "ask");
            h.wait("revised question",60,|| current_question(&h.d,&sent(&h.st),h.task).is_some_and(|(_,r)|r==revision));
            let current = current_question(&h.d,&sent(&h.st),h.task).unwrap().0;
            assert_ne!(question,current);
            let before = h.calls().iter().filter(|c|c["args"][0]=="answer").count();
            let (stale, refusal) = h.request(OWNER,"1",Some(question));
            assert_eq!(refusal,"Answer was not saved; check the current question and your access.");
            assert_eq!(query_text(&h.d,&format!("SELECT error FROM requests WHERE id={stale}")),"stale question");
            assert_eq!(h.calls().iter().filter(|c|c["args"][0]=="answer").count(),before);
            let (id, answer) = h.request(OWNER,"1",Some(current));
            assert!(answer.starts_with("Answered by ") && answer.contains("Short link") && answer.contains("TaskQ:"),"{answer}");
            let calls = h.calls();
            let rc = calls.iter().find(|c| c["args"][0] == "answer" && c["receipt"]["outcome"] == "accepted").expect("accepted answer receipt");
            let args=rc["args"].as_array().unwrap();
            let fence=args.iter().position(|a|a=="--revision").unwrap();
            assert_eq!(args[fence+1],revision.to_string());
            assert_eq!(rc["receipt"]["task"],h.task);
            assert!(rc["receipt"]["op_id"].as_str().unwrap().starts_with("pilotq-"));
            let before = calls.iter().filter(|c| c["args"][0] == "answer").count();
            let (repeat, confirmed) = h.request(OWNER,"2",Some(current));
            let receipt=query_text(&h.d,&format!("SELECT text FROM outbox WHERE task={} AND revision={revision} AND kind='question_answered' LIMIT 1",h.task));
            assert!(receipt.starts_with("Answered by ") && receipt.contains("Short link"));
            assert_eq!(confirmed,receipt,"repeated reply returns the saved actor/choice, never a new decision");
            assert_eq!(h.calls().iter().filter(|c| c["args"][0] == "answer").count(),before);
            json!({"request":id,"receipt":rc["receipt"],"stale_request":stale,"refusal":refusal,"repeat_request":repeat,"repeat_receipt":confirmed})
        });
        h.case("reviewer", |h| {
            // The accepted answer resumes the same synthetic attempt; no ordinary worker runs.
            let inbox = h.taskq(&["inbox".into(),h.task.to_string()]).unwrap();
            let folded: Value = serde_json::from_str(inbox.lines().find_map(|l|l.strip_prefix("Folded: ")).unwrap()).unwrap();
            let attempt = folded["attempt"].as_i64().unwrap();
            assert_eq!(folded["attempt"],h.data["fixture_attempt"]);
            let candidate = h.data["candidate"].as_str().unwrap().to_string();
            h.event(&format!("taskq: result {candidate} attempt {attempt}\nChanged: Scripted A40 fixture reached result; no code changed or published.\nChecks: Synthetic management lifecycle only.\nOpen: none"), "review");
            h.wait("review delivery",60,|| sent(&h.st).iter().any(|s|h.status_message(s,"Result is being reviewed")));
            let host = h.data["isolated_host"].as_str().unwrap();
            let reviewer = h.event(&format!("taskq: spawn reviewer codex:a40-scripted-review {host}\nA40 synthetic reviewer; no process is launched."), "review");
            h.event(&format!("taskq: close attempt {reviewer}\nChanged: Scripted A40 reviewer outcome; no code review or publication claimed.\nChecks: Management notification fixture.\nOpen: none"), "closed");
            h.github("PATCH", &format!("repos/taskqorg/pilotq/issues/{}",h.task), json!({"state":"closed","state_reason":"completed"}));
            h.wait("reviewer outcome",7200,|| sent(&h.st).iter().any(|s| h.status_message(s,"Task closed; see the result")));
            assert!(sent(&h.st).iter().any(|s|h.status_message(s,"Result is being reviewed")));
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
            let update = query(&h.d, &format!("SELECT update_id FROM requests WHERE id={}", h.run_request));
            assert!(request_delivered(&h.d, update), "audit requires final delivery, not a privacy notice");
            let messages = sent(&h.st);
            assert!(messages.iter().all(|m|routed_message(&h.d,m,h.task)), "every visible message needs an originating-route or verified startup onboarding receipt");
            let terminal = messages.iter().find(|m|h.status_message(m,"Task closed; see the result")).unwrap();
            assert_eq!(terminal["edit"],true,"#165 closure edits the launch status card");
            let status = query(&h.d,&format!("SELECT sent_message FROM outbox WHERE request_id={} AND kind='status' AND state='delivered' ORDER BY id LIMIT 1",h.run_request));
            assert_eq!(terminal["id"],status);
            assert_eq!(query(&h.d,&format!("SELECT count(DISTINCT sent_message) FROM outbox WHERE request_id={} AND kind='status' AND state='delivered'",h.run_request)),1);
            let latest=messages.iter().rev().find(|m|m["id"]==status).unwrap();
            assert!(latest["text"].as_str().unwrap().contains("Task closed; see the result"),"terminal outcome cannot be overwritten");
            h.data["audit"] = audit;
            json!({"trace":format!("pilotq.request.{}",h.run_request),"all_messages_have_route_receipts":true})
        });
    }));
    h.finish(result);
    eprintln!("A40 report: {}",h.report.display());
    assert_eq!(h.data["pass"],true,"A40 incomplete; inspect the per-case report and cleanup");
}

// R9/R16, §3: exercise the actual manager env_clear boundary with a harmless
// fake app-server child. No subscription, token, board or service control is used.
#[test]
fn a40_native_manager_scope() {
    let (d, st) = setup("a40-native-scope", vec![]);
    let root = d.join("root");
    let install = d.join("install");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&install).unwrap();
    let scope = json!({"install":install,"root":root});
    fs::write(d.join("taskq-scope.json"), scope.to_string()).unwrap();
    fs::write(d.join("record-taskq"), RECORD_TASKQ).unwrap();
    fs::set_permissions(d.join("record-taskq"), fs::Permissions::from_mode(0o700)).unwrap();
    std::os::unix::fs::symlink(d.join("record-taskq"), d.join("taskq")).unwrap();
    let fake = script(&d, "probe-codex", &FAKE.replace("d=$(dirname \"$0\")", "d=$(dirname \"$0\")\necho $$ > \"$d/native-runtime.pid\"\ntaskq --a40-path-probe > \"$d/native-scope.json\""));
    use_codex(&d, &fake);
    let end = Instant::now() + Duration::from_secs(10);
    let c = spawn(&d, None);
    push(&st, text(1, OWNER, None, OWNER, "hello"));
    wait("native path probe", 10, || d.join("native-scope.json").exists() && fs::metadata(d.join("native-scope.json")).unwrap().len() > 0);
    let observed: Value = serde_json::from_slice(&fs::read(d.join("native-scope.json")).unwrap()).unwrap();
    assert_eq!(observed, scope);
    // The probe is an intermediate runtime write, not a completed turn. The manager
    // has its own process group: killing only the service can leave it writing here.
    wait("native probe request delivered", end.saturating_duration_since(Instant::now()).as_secs(), || request_delivered(&d, 1));
    let pid = fs::read_to_string(d.join("native-runtime.pid")).unwrap();
    wait("native runtime reaped", end.saturating_duration_since(Instant::now()).as_secs(), || !Command::new("kill")
        .args(["-0", "--", pid.trim()]).stderr(Stdio::null()).status().unwrap().success());
    let removed = d.clone();
    stop(c, d);
    assert!(!removed.exists(), "native fixture survived teardown");
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
        login:Some(Login(vec![d.join("codex/auth.json")])), fixture_token:None, d:d.clone(),st,svc:None,update:0,task:0,run_request:0,
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

// §9/R5/R16: real harness host preparation/retirement against a fake forge, no board or login.
#[test]
fn a40_reserved_host_metadata() {
    const CHILD: &str = "PILOTQ_A40_HOST_TEST";
    if std::env::var_os(CHILD).is_none() {
        let (d, _) = setup("a40-host-metadata", vec![]);
        let bin = d.join("bin"); fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("gh"), r#"#!/usr/bin/env python3
import json, os, pathlib, sys
p = pathlib.Path(os.environ['PILOTQ_A40_HOST_TEST'])
a = sys.argv[1:]; path = next(x for x in a if x.startswith('repos/'))
f = p/'hosts.json'; rows = json.loads(f.read_text()) if f.exists() else []
if '--method' in a:
    method = a[a.index('--method')+1]; v = json.load(sys.stdin)
    if method == 'POST':
        v.update(number=901, state='open', repository_url='https://api.github.com/repos/taskqorg/pilotq'); rows.append(v)
        f.write_text(json.dumps(rows))
        if (p/'lose-response').exists(): sys.exit(1)
    else:
        v = rows[0]
        if not (p/'refuse-close').exists(): v['state'] = 'closed'
        f.write_text(json.dumps(rows))
    print(json.dumps(v))
elif '?' in path:
    # Prove pagination: the matching issue is after one full unrelated page.
    print(json.dumps([] if (p/'delayed-list').exists() else ([{'title':'ordinary host'}]*100 if path.endswith('&page=1') else rows)))
else: print(json.dumps(rows[0]))
"#).unwrap();
        fs::set_permissions(bin.join("gh"), fs::Permissions::from_mode(0o700)).unwrap();
        let out = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "automatic::a40_reserved_host_metadata", "--nocapture"])
            .env(CHILD, &d).env("PATH", format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()))
            .output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
        fs::remove_dir_all(d).unwrap();
        return;
    }
    let d = PathBuf::from(std::env::var_os(CHILD).unwrap());
    let (_, st) = setup("a40-host-child", vec![]);
    let mut h = Run { d:d.clone(), st, root:d.clone(), report:d.join("report.json"),
        data:json!({"isolated_host":"pilotq-e2e-invented-fixture","cases":{}}),
        svc:None, login:None, fixture_token:None, update:0, task:0, run_request:0, deadline:Instant::now()+Duration::from_secs(30) };
    fs::write(d.join("delayed-list"), "").unwrap();
    h.prepare_host(); h.prepare_host();
    fs::write(d.join("refuse-close"), "").unwrap();
    assert!(h.cleanup().is_err(), "open exact issue must fail retirement");
    assert_ne!(h.data["host_metadata_retired"], true);
    fs::remove_file(d.join("refuse-close")).unwrap();
    h.cleanup().unwrap();
    assert_eq!(h.data["host_metadata_retired"], true);
    fs::remove_file(d.join("delayed-list")).unwrap();
    // Continue existing ambiguity and credential-cleanup protections with an open fixture.
    let mut rows: Value = serde_json::from_str(&fs::read_to_string(d.join("hosts.json")).unwrap()).unwrap();
    rows[0]["state"] = json!("open");
    fs::write(d.join("hosts.json"), rows.to_string()).unwrap();
    assert_eq!(h.fixture_hosts().len(), 1, "repeat preparation must not create another host");
    let original = h.fixture_hosts();
    fs::write(d.join("hosts.json"), serde_json::to_string(&vec![original[0].clone(), original[0].clone()]).unwrap()).unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||h.fixture_hosts())).is_err());
    fs::write(d.join("hosts.json"), serde_json::to_string(&original).unwrap()).unwrap();
    h.cleanup().unwrap();
    assert_eq!(h.fixture_hosts()[0]["state"], "closed");
    fs::remove_file(d.join("hosts.json")).unwrap();
    h.data.as_object_mut().unwrap().remove("host_metadata");
    h.data.as_object_mut().unwrap().remove("host_metadata_intent");
    fs::write(d.join("lose-response"), "").unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||h.prepare_host())).is_err());
    assert_eq!(h.fixture_hosts().len(), 1, "lost response retains reconcilable intent");
    fs::write(d.join("delayed-list"), "").unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||h.prepare_host())).is_err());
    assert_eq!(serde_json::from_str::<Value>(&fs::read_to_string(d.join("hosts.json")).unwrap()).unwrap().as_array().unwrap().len(), 1, "uncertain POST must not duplicate creation");
    assert!(h.cleanup().is_err(), "unknown issue and empty listing must remain unresolved");
    assert_eq!(h.data["host_metadata_retired"], false);
    let report: Value = serde_json::from_str(&fs::read_to_string(&h.report).unwrap()).unwrap();
    assert_eq!(report["host_metadata_intent"], true);
    assert_eq!(report["host_metadata_retired"], false);
    fs::remove_file(d.join("delayed-list")).unwrap();
    let original = h.fixture_hosts()[0].clone();
    h.data["host_metadata"] = original["number"].clone();
    for (key, value) in [("repository_url", json!("https://api.github.com/repos/invented/foreign")),
        ("title", json!("taskq host ordinary")), ("body", json!("foreign service")),
        ("labels", json!([])), ("number", json!(902))] {
        let mut row = original.clone(); row[key] = value;
        fs::write(d.join("hosts.json"), json!([row]).to_string()).unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||h.cleanup())).is_err(), "foreign {key} must prevent retirement");
    }
    fs::write(d.join("hosts.json"), json!([original]).to_string()).unwrap();
    // Even failed cleanup must remove the disposable credential and retain the failed case.
    let install = d.join("install"); fs::create_dir_all(&install).unwrap();
    fs::write(install.join("token"), "invented-copy").unwrap();
    h.data["fixture_install"] = json!(install);
    let mut rows = h.fixture_hosts(); rows[0]["body"] = json!("foreign ordinary service");
    fs::write(d.join("hosts.json"), serde_json::to_string(&rows).unwrap()).unwrap();
    h.finish(Err(Box::new("injected failure".to_string())));
    assert_eq!(h.data["cases"]["cleanup"]["status"], "failed");
    assert!(!install.join("token").exists());
    assert_eq!(h.data["pass"], false);
    fs::write(install.join("token"), "invented-copy").unwrap();
    h.fixture_token = Some(Login(vec![install.join("token")]));
    drop(h);
    assert!(!install.join("token").exists(), "unwinding/report errors retain the token cleanup guard");
}
