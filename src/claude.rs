//! #265: Claude Code manager turns through CoreQ `agent::Turn` (`claude -p`, one process per turn, resumed by session ID).
//! PilotQ's own tools reach Claude through a per-turn stdio MCP relay (`pilotq --mcp-bridge SOCKET`) that forwards each call
//! over a private Unix socket to the same tool boundary Codex uses; the relay holds no authority or state of its own.
use crate::manager::{Limits, Outcome};
use coreq::agent::{Client, Spec, Turn};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Claude {
    pub bin: String,
    /// PilotQ's own Claude config home with its subscription login, or None for the service user's default login. No API
    /// key is ever passed (R9).
    pub home: Option<PathBuf>,
    /// Per-session stream logs inside the installation, outside the model's workspace.
    pub logs: PathBuf,
    /// Claude has no catalogue call; `/model` offers these configured aliases.
    pub models: Vec<String>,
}

const SERVER: &str = "pilotq";
const TOKEN: &str = "PILOTQ_MCP_TOKEN";

fn b64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 { s.push(if i <= c.len() { A[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' }); }
    }
    s
}

/// PilotQ's Codex-shaped input items as one stream-json user message; images travel inline.
fn message(input: &Value) -> Result<String, String> {
    let mut content = vec![];
    for item in input.as_array().ok_or("invalid turn input")? {
        match item["type"].as_str() {
            Some("localImage") => {
                let path = Path::new(item["path"].as_str().ok_or("invalid image")?);
                let bytes = std::fs::read(path).map_err(|_| "image unreadable for this turn")?;
                let media = match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
                    "png" => "image/png", "webp" => "image/webp", "gif" => "image/gif", _ => "image/jpeg",
                };
                content.push(json!({"type": "image", "source": {"type": "base64", "media_type": media, "data": b64(&bytes)}}));
            }
            _ => content.push(json!({"type": "text", "text": item["text"].as_str().unwrap_or("")})),
        }
    }
    Ok(format!("{}\n", json!({"type": "user", "message": {"role": "user", "content": content}})))
}

/// The native item PilotQ records for a Claude tool use, in the Codex item vocabulary the journal and audit already use.
fn item(block: &Value) -> Value {
    let name = block["name"].as_str().unwrap_or("tool");
    let kind = match name {
        n if n.starts_with("mcp__pilotq__") => "dynamicToolCall",
        "Bash" => "commandExecution",
        "Write" | "Edit" | "NotebookEdit" => "fileChange",
        "WebSearch" | "WebFetch" => "webSearch",
        _ => "mcpToolCall",
    };
    let mut v = json!({"type": kind, "id": block["id"], "tool": name});
    if let Some(c) = block["input"]["command"].as_str() { v["command"] = json!(c); }
    if let Some(p) = block["input"]["file_path"].as_str() { v["changes"] = json!([{"path": p}]); }
    v
}

impl Claude {
    /// One Claude turn on `session` (a new one when None); the same callbacks and outcomes as `Codex::turn`.
    #[allow(clippy::too_many_arguments)]
    pub fn turn(
        &self,
        workspace: &Path,
        lim: &Limits,
        native: bool,
        session: Option<&str>,
        selected_model: Option<&str>,
        input: &Value,
        dynamic: Option<(&Value, &str)>,
        mut alive: impl FnMut() -> bool,
        mut tool: impl FnMut(&str, &str, Option<&str>),
        mut call: impl FnMut(&str, &Value, &str) -> (bool, String),
        mut rebuild: impl FnMut() -> Result<Value, String>,
        mut preview: impl FnMut(&str),
        mut observation: impl FnMut(&str, &Value),
        mut record: impl FnMut(&dyn Fn(&coreq::telemetry::Store) -> Result<(), String>),
        route: &str,
    ) -> (Option<String>, Option<String>, Outcome) {
        let mut input = input.clone();
        let mut session = session.map(str::to_string);
        loop {
            let resumed = session.is_some();
            match self.attempt(workspace, lim, native, session.as_deref(), selected_model, &input, dynamic, &mut alive, &mut tool, &mut call,
                &mut preview, &mut observation, &mut record, route) {
                // A resume that finds no native session is rebuilt once from the journal on a new session.
                Err(()) if resumed => {
                    let Ok(history) = rebuild() else {
                        return (session, None, Outcome::Failed("Conversation recovery could not be prepared; no work was repeated.".into()));
                    };
                    input.as_array_mut().unwrap().insert(0, json!({"type": "text", "text": history.to_string()}));
                    session = None;
                }
                Err(()) => return (None, None, Outcome::Failed("Claude could not open the conversation; no work was repeated.".into())),
                Ok(done) => return done,
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn attempt(
        &self,
        workspace: &Path,
        lim: &Limits,
        native: bool,
        resume: Option<&str>,
        selected_model: Option<&str>,
        input: &Value,
        dynamic: Option<(&Value, &str)>,
        alive: &mut impl FnMut() -> bool,
        tool: &mut impl FnMut(&str, &str, Option<&str>),
        call: &mut impl FnMut(&str, &Value, &str) -> (bool, String),
        preview: &mut impl FnMut(&str),
        observation: &mut impl FnMut(&str, &Value),
        record: &mut impl FnMut(&dyn Fn(&coreq::telemetry::Store) -> Result<(), String>),
        route: &str,
    ) -> Result<(Option<String>, Option<String>, Outcome), ()> {
        let failed = |e: String| Ok((resume.map(str::to_string), None, Outcome::Failed(e)));
        let session = resume.map(str::to_string).unwrap_or_else(|| coreq::agent::new_session(Client::Claude));
        // Private per-turn files: tool relay config, instructions and socket. The token proves a relay belongs to this turn.
        let token = coreq::agent::new_session(Client::Claude);
        let dir = workspace.join(".pilotq-turns").join(&token[..8]);
        let prepared = (|| {
            if let Some(home) = &self.home { std::fs::create_dir_all(home).map_err(|e| format!("claude home: {e}"))?; }
            std::fs::create_dir_all(&dir).map_err(|e| format!("turn directory: {e}"))?;
            #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?; }
            // SUN_LEN is 104 bytes on macOS: the socket lives directly in the OS temp dir, owner-only.
            let socket = std::env::temp_dir().join(format!("pilotq-{}.sock", &token[..12]));
            let exe = std::env::current_exe().map_err(|e| format!("pilotq executable: {e}"))?;
            let config = json!({"mcpServers": {SERVER: {"type": "stdio", "command": exe, "args": ["--mcp-bridge", socket], "env": {TOKEN: token}}}});
            std::fs::write(dir.join("mcp.json"), config.to_string()).map_err(|e| e.to_string())?;
            std::fs::write(dir.join("instructions.txt"), dynamic.map_or("", |d| d.1)).map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(&socket);
            let listener = UnixListener::bind(&socket).map_err(|e| format!("tool relay: {e}"))?;
            #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?; }
            listener.set_nonblocking(true).map_err(|e| e.to_string())?;
            Ok::<_, String>(listener)
        })();
        let _cleanup = Cleanup(vec![dir.clone(), std::env::temp_dir().join(format!("pilotq-{}.sock", &token[..12]))]);
        let listener = match prepared { Ok(l) => l, Err(e) => return failed(e) };
        let mut args: Vec<std::ffi::OsString> = ["--input-format", "stream-json", "--include-partial-messages", "--strict-mcp-config",
            "--setting-sources", "project", "--disable-slash-commands", "--system-prompt-snapshot", "off"].map(Into::into).into();
        args.extend(["--mcp-config".into(), dir.join("mcp.json").into(), "--append-system-prompt-file".into(), dir.join("instructions.txt").into()]);
        args.extend(["--allowedTools".into(), format!("mcp__{SERVER}").into()]);
        // §3 / #190: native tools and live web only for the owner or a shell grantee; others keep PilotQ's granted tools only
        if native { args.extend(["--permission-mode".into(), "bypassPermissions".into()]); } else { args.extend(["--tools".into(), "".into()]); }
        if let Some(m) = selected_model { args.extend(["--model".into(), m.into()]); }
        let spec = Spec { client: Client::Claude, root: workspace.into(), session: session.clone(), resume: resume.is_some(), args, home: self.home.clone(), env: vec![] };
        let text = match message(&input) { Ok(t) => t, Err(e) => return failed(e) };
        let log = self.logs.join(format!("{session}.jsonl"));
        let mut pos = std::fs::metadata(&log).map(|m| m.len()).unwrap_or(0);
        let started = Instant::now();
        let mut turn = match Turn::start(crate::manager::base(&self.bin), &spec, &log, &text) {
            Ok(t) => t,
            Err(e) => { crate::host::log("claude", &e); return failed(format!("claude did not start: {e}")); }
        };
        observation("spawned", &json!(turn.id()));
        let (mut first, mut last, mut tooled, mut answer) = (false, started, false, None::<String>);
        let (mut draft, mut window, mut used, mut missing) = (String::new(), None::<u64>, None::<u64>, false);
        let mut items = std::collections::HashMap::new();
        let version = "claude".to_string();
        let tools: Vec<Value> = dynamic.map(|d| d.0.as_array().cloned().unwrap_or_default()).unwrap_or_default().into_iter()
            .map(|t| json!({"name": t["name"], "description": t["description"], "inputSchema": t["inputSchema"]})).collect();
        let stop = |turn: &mut Turn| {
            #[cfg(unix)]
            let _ = crate::host::kill_group(turn.id());
            let _ = turn.kill();
            let _ = turn.wait(Some(Instant::now() + crate::manager::GRACE));
        };
        loop {
            if !alive() { stop(&mut turn); return Ok((Some(session), None, Outcome::Fenced)); }
            let mut progressed = false;
            // tool relay: one request per connection, answered by the same boundary as Codex dynamic tools
            while let Ok((stream, _)) = listener.accept() {
                progressed = true;
                let _ = relay(stream, &token, &tools, &mut *call, &version);
            }
            if let Ok(mut f) = std::fs::File::open(&log) {
                let mut rest = vec![];
                if f.seek(SeekFrom::Start(pos)).and_then(|_| f.read_to_end(&mut rest)).is_ok()
                    && let Some(end) = rest.iter().rposition(|b| *b == b'\n') {
                    pos += end as u64 + 1;
                    for line in String::from_utf8_lossy(&rest[..end]).lines() {
                        progressed = true;
                        missing |= line.contains("No conversation found with session ID");
                        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
                        match v["type"].as_str() {
                            Some("stream_event") => match v["event"]["type"].as_str() {
                                Some("message_start") => draft.clear(),
                                Some("content_block_delta") if v["event"]["delta"]["type"] == "text_delta" => {
                                    draft.push_str(v["event"]["delta"]["text"].as_str().unwrap_or(""));
                                    preview(&draft);
                                }
                                _ => {}
                            },
                            Some("assistant") => {
                                let u = &v["message"]["usage"];
                                used = ["input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens"].iter().map(|k| u[k].as_u64()).sum::<Option<u64>>().or(used);
                                for b in v["message"]["content"].as_array().into_iter().flatten().filter(|b| b["type"] == "tool_use") {
                                    tooled = true;
                                    let it = item(b);
                                    tool(it["type"].as_str().unwrap_or(""), it["id"].as_str().unwrap_or(""), None);
                                    observation("native_started", &it);
                                    items.insert(it["id"].as_str().unwrap_or("").to_string(), it);
                                }
                            }
                            Some("user") => for b in v["message"]["content"].as_array().into_iter().flatten().filter(|b| b["type"] == "tool_result") {
                                let status = if b["is_error"] == true { "failed" } else { "completed" };
                                let id = b["tool_use_id"].as_str().unwrap_or("");
                                let mut it = items.remove(id).unwrap_or_else(|| json!({"type": "mcpToolCall", "id": id}));
                                tool(it["type"].as_str().unwrap_or(""), id, Some(status));
                                it["status"] = json!(status);
                                observation("native_completed", &it);
                            },
                            Some("result") => {
                                answer = v["result"].as_str().map(str::to_string);
                                window = v["modelUsage"].as_object().and_then(|m| m.values().filter_map(|u| u["contextWindow"].as_u64()).max());
                            }
                            _ => {}
                        }
                    }
                }
            }
            if progressed { (first, last) = (true, Instant::now()); observation("activity", &Value::Null); }
            match turn.poll() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(e) => { stop(&mut turn); return Ok((Some(session), None, if tooled { Outcome::Uncertain(e) } else { Outcome::Failed(e) })); }
            }
            let (limit, why, secs) = if first { (last + lim.silent, "Claude sent no event for", lim.silent) } else { (started + lim.first_event, "Claude sent no first event within", lim.first_event) };
            if Instant::now() >= limit {
                let fenced = !alive();
                stop(&mut turn);
                let why = format!("{why} {} s", secs.as_secs());
                return Ok((Some(session), None, if fenced { Outcome::Fenced } else if tooled { Outcome::Uncertain(why) } else { Outcome::Failed(why) }));
            }
            if !progressed { std::thread::sleep(Duration::from_millis(100)); }
        }
        let r = turn.report();
        if missing && resume.is_some() && r.outcome != "ok" && !tooled { return Err(()); }
        // A5: one CoreQ usage observation per turn, attributed to this route
        let ctx = coreq::agent::Context { product: "pilotq", version: env!("CARGO_PKG_VERSION"), project: "other", task: None, route: Some(route), account: "default" };
        record(&|store| coreq::telemetry::Event::new(r.observation(Client::Claude, &ctx)).and_then(|e| store.append(&[e])).map(drop));
        if let (Some(used), Some(window)) = (used, window) {
            observation("context", &json!({"last": {"inputTokens": used}, "modelContextWindow": window}));
        }
        if r.limit.window.as_deref() == Some("seven_day") && let (Some(p), Some(at)) = (r.limit.used_percent, r.limit.resets_at) {
            observation("quota", &json!({"primary": {"usedPercent": p, "windowDurationMins": 10080, "resetsAt": at}}));
        }
        let model = r.usage.model.clone();
        let outcome = match (r.outcome, answer.or(r.text.clone())) {
            ("ok", Some(a)) if !a.trim().is_empty() => Outcome::Answered(a),
            ("ok", _) => Outcome::Failed("Claude ended the turn without an answer".into()),
            (o, _) => {
                let why = format!("Claude turn {o}: {}", r.error);
                if tooled { Outcome::Uncertain(why) } else { Outcome::Failed(why) }
            }
        };
        Ok((Some(session), model, outcome))
    }
}

struct Cleanup(Vec<PathBuf>);
impl Drop for Cleanup { fn drop(&mut self) { for p in &self.0 { let _ = if p.is_dir() { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) }; } } }

/// Service side of one relay request: `{"token","op":"list"|"call","name","arguments"}`, answered on the same connection.
fn relay(stream: UnixStream, token: &str, tools: &[Value], call: &mut impl FnMut(&str, &Value, &str) -> (bool, String), version: &str) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut line = String::new();
    BufReader::new(&stream).take(8 * 1024 * 1024).read_line(&mut line)?;
    let req: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
    let reply = if req["token"] != token {
        json!({"ok": false, "text": "tool relay refused"})
    } else if req["op"] == "list" {
        json!({"tools": tools})
    } else {
        let (ok, text) = call(req["name"].as_str().unwrap_or(""), &req["arguments"], version);
        json!({"ok": ok, "text": text})
    };
    (&stream).write_all(format!("{reply}\n").as_bytes())
}

/// `pilotq --mcp-bridge SOCKET`: the stdio MCP server Claude starts for PilotQ's tools. It relays and decides nothing.
pub fn bridge(socket: &str) {
    serve_bridge(socket, &std::env::var(TOKEN).unwrap_or_default(), std::io::stdin().lock(), std::io::stdout());
}

fn serve_bridge(socket: &str, token: &str, input: impl BufRead, mut out: impl Write) {
    let ask = |v: Value| -> Value {
        let mut req = v; req["token"] = json!(token);
        let Ok(mut s) = UnixStream::connect(socket) else { return json!({"ok": false, "text": "PilotQ tools are unavailable for this turn"}) };
        let _ = s.write_all(format!("{req}\n").as_bytes());
        let mut line = String::new();
        let _ = BufReader::new(s).read_line(&mut line);
        serde_json::from_str(&line).unwrap_or(json!({"ok": false, "text": "PilotQ tool outcome unknown"}))
    };
    for line in input.lines().map_while(Result::ok) {
        let Ok(m) = serde_json::from_str::<Value>(&line) else { continue };
        let Some(id) = m.get("id").cloned() else { continue };
        let result = match m["method"].as_str().unwrap_or("") {
            "initialize" => json!({"protocolVersion": m["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
                "capabilities": {"tools": {}}, "serverInfo": {"name": SERVER, "version": env!("CARGO_PKG_VERSION")}}),
            "ping" => json!({}),
            "tools/list" => json!({"tools": ask(json!({"op": "list"}))["tools"].as_array().cloned().unwrap_or_default()}),
            "tools/call" => {
                let r = ask(json!({"op": "call", "name": m["params"]["name"], "arguments": m["params"]["arguments"]}));
                json!({"content": [{"type": "text", "text": r["text"].as_str().unwrap_or("")}], "isError": r["ok"] != true})
            }
            _ => { let _ = writeln!(out, "{}", json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "unsupported"}})); let _ = out.flush(); continue; }
        };
        let _ = writeln!(out, "{}", json!({"jsonrpc": "2.0", "id": id, "result": result}));
        let _ = out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn base64_matches_rfc4648() {
        for (raw, enc) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foobar", "Zm9vYmFy")] { assert_eq!(b64(raw.as_bytes()), enc); }
    }

    // #265 A2/A3/A6: a fake Claude CLI receives one stream-json message, lists and calls a PilotQ tool through the per-turn
    // relay socket, streams a draft and answers; the next turn resumes the same session ID, and a missing session is rebuilt
    // once. Restricted principals get no built-in tools. No paid model is involved.
    #[test]
    fn claude_turn_and_tool_relay_265() {
        let dir = std::env::temp_dir().join(format!("pilotq-claude265-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("claude");
        std::fs::write(&bin, r#"#!/usr/bin/env python3
import sys,json,os,socket
a=sys.argv[1:]
def val(k): return a[a.index(k)+1]
sid=val('--resume') if '--resume' in a else val('--session-id')
native='--permission-mode' in a
assert native or val('--tools')==''
assert '--strict-mcp-config' in a and val('--allowedTools')=='mcp__pilotq'
assert 'ANTHROPIC_API_KEY' not in os.environ and os.environ['CLAUDE_CONFIG_DIR']
if sid=='00000000-0000-4000-8000-000000000000':
  print('No conversation found with session ID: '+sid, file=sys.stderr); sys.exit(1)
msg=json.loads(sys.stdin.readline())
text=msg['message']['content'][-1]['text']
srv=json.load(open(val('--mcp-config')))['mcpServers']['pilotq']
def ask(v):
  v['token']=srv['env']['PILOTQ_MCP_TOKEN']
  s=socket.socket(socket.AF_UNIX); s.connect(srv['args'][1]); s.sendall((json.dumps(v)+'\n').encode())
  return json.loads(s.makefile().readline())
assert [t['name'] for t in ask({'op':'list'})['tools']]==['project_list']
print(json.dumps({'type':'system','subtype':'init','session_id':sid}),flush=True)
print(json.dumps({'type':'assistant','message':{'id':'m1','model':'claude-invented','content':[{'type':'tool_use','id':'t1','name':'mcp__pilotq__project_list','input':{}}],'usage':{'input_tokens':10,'cache_read_input_tokens':0,'cache_creation_input_tokens':0,'output_tokens':1}}}),flush=True)
r=ask({'op':'call','name':'project_list','arguments':{}})
assert r=={'ok':True,'text':'granted result'}
print(json.dumps({'type':'user','message':{'content':[{'type':'tool_result','tool_use_id':'t1','content':'granted result'}]}}),flush=True)
print(json.dumps({'type':'stream_event','event':{'type':'content_block_delta','delta':{'type':'text_delta','text':'draft'}}}),flush=True)
print(json.dumps({'type':'result','subtype':'success','is_error':False,'result':'finished '+text,'session_id':sid,'modelUsage':{'claude-invented':{'contextWindow':100}}}),flush=True)
"#).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o700)).unwrap();
        let c = Claude { bin: bin.to_str().unwrap().into(), home: Some(dir.join("home")), logs: dir.join("logs"), models: vec![] };
        let lim = Limits { first_event: Duration::from_secs(10), silent: Duration::from_secs(10), deadline: Duration::from_secs(1) };
        let tools = json!([{"type": "function", "name": "project_list", "description": "invented", "inputSchema": {"type": "object"}}]);
        let run = |native: bool, session: Option<&str>| {
            let (mut drafts, mut seen, mut stored, mut context) = (vec![], vec![], 0, Value::Null);
            let out = c.turn(&dir.join("workspace"), &lim, native, session, None, &json!([{"type": "text", "text": "hello"}]), Some((&tools, "invented instructions")),
                || true, |_, id, status| seen.push((id.to_string(), status.map(str::to_string))), |name, _, _| (name == "project_list", "granted result".into()),
                || Ok(json!({"history": "invented"})), |d| drafts.push(d.to_string()), |k, v| if k == "context" { context = v.clone(); }, |_| stored += 1, "cli");
            (out, drafts, seen, stored, context)
        };
        let ((session, model, outcome), drafts, seen, stored, context) = run(false, None);
        assert_eq!(outcome, Outcome::Answered("finished hello".into()));
        assert_eq!(model.as_deref(), Some("claude-invented"));
        assert_eq!(drafts, ["draft"]);
        assert_eq!(seen, [("t1".to_string(), None), ("t1".to_string(), Some("completed".to_string()))]);
        assert_eq!(stored, 1, "one usage observation per turn");
        assert_eq!(context, json!({"last": {"inputTokens": 10}, "modelContextWindow": 100}));
        let session = session.unwrap();
        let ((again, _, outcome), ..) = run(true, Some(&session));
        assert_eq!((again.as_deref(), outcome), (Some(session.as_str()), Outcome::Answered("finished hello".into())));
        let ((rebuilt, _, outcome), ..) = run(true, Some("00000000-0000-4000-8000-000000000000"));
        assert!(rebuilt.is_some_and(|s| s != "00000000-0000-4000-8000-000000000000"));
        assert_eq!(outcome, Outcome::Answered("finished hello".into()));
        assert!(!dir.join("workspace/.pilotq-turns").read_dir().unwrap().any(|_| true), "per-turn relay files are removed");
        std::fs::remove_dir_all(dir).unwrap();
    }

    // #265: the stdio MCP relay answers initialize, lists and calls tools through the turn socket and never decides itself.
    #[test]
    fn mcp_bridge_relays_265() {
        let dir = std::env::temp_dir().join(format!("pilotq-bridge265-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let socket = dir.join("s.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let tools = vec![json!({"name": "project_list"})];
        let server = std::thread::spawn(move || for _ in 0..2 {
            let (s, _) = listener.accept().unwrap();
            relay(s, "invented-token", &tools, &mut |name, _, _| (false, format!("refused {name}")), "v").unwrap();
        });
        let input = [
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "project_list", "arguments": {}}}),
            json!({"jsonrpc": "2.0", "id": 4, "method": "resources/list"}),
        ].map(|v| v.to_string()).join("\n");
        let mut out = vec![];
        serve_bridge(socket.to_str().unwrap(), "invented-token", input.as_bytes(), &mut out);
        server.join().unwrap();
        let replies: Vec<Value> = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(replies[0]["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(replies[1]["result"]["tools"], json!([{"name": "project_list"}]));
        assert_eq!(replies[2]["result"], json!({"content": [{"type": "text", "text": "refused project_list"}], "isError": true}));
        assert_eq!(replies[3]["error"]["code"], -32601);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
