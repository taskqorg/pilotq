//! Per-turn stdio MCP relay for every runtime. Calls reach PilotQ's grant/fence boundary; the relay owns no authority.
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const SERVER: &str = "pilotq";
const TOKEN: &str = "PILOTQ_MCP_TOKEN";

/// One turn's private relay: MCP config, instructions and socket. The token proves a relay connection belongs to this turn.
pub struct Relay {
    pub config: PathBuf,
    pub instructions: PathBuf,
    token: String,
    tools: Vec<Value>,
    listener: UnixListener,
    _cleanup: Cleanup,
}

impl Relay {
    pub fn open(workspace: &Path, tools: (&Value, &str)) -> Result<Relay, String> {
        let token = crate::manager::token();
        let dir = workspace.join(".pilotq-turns").join(&token[..8]);
        // SUN_LEN is 104 bytes on macOS: the socket lives directly in the OS temp dir, owner-only.
        let socket = std::env::temp_dir().join(format!("pilotq-{}.sock", &token[..12]));
        let cleanup = Cleanup(vec![dir.clone(), socket.clone()]);
        std::fs::create_dir_all(&dir).map_err(|e| format!("turn directory: {e}"))?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
        let exe = std::env::current_exe().map_err(|e| format!("pilotq executable: {e}"))?;
        let config = json!({"mcpServers": {SERVER: {"command": exe, "args": ["--mcp-bridge", socket], "env": {TOKEN: token}}}});
        std::fs::write(dir.join("mcp.json"), config.to_string()).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("instructions.txt"), tools.1).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).map_err(|e| format!("tool relay: {e}"))?;
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let tools = tools.0.as_array().cloned().unwrap_or_default().into_iter()
            .map(|t| json!({"name": t["name"], "description": t["description"], "inputSchema": t["inputSchema"]})).collect();
        Ok(Relay { config: dir.join("mcp.json"), instructions: dir.join("instructions.txt"), token, tools, listener, _cleanup: cleanup })
    }

    /// Answers every pending relay request, one per connection; true when one arrived.
    pub fn serve(&self, call: &mut impl FnMut(&str, &Value, &str) -> (bool, String), version: &str) -> bool {
        let mut served = false;
        while let Ok((stream, _)) = self.listener.accept() {
            served = true;
            let _ = relay(stream, &self.token, &self.tools, call, version);
        }
        served
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

/// `pilotq --mcp-bridge SOCKET`: the stdio MCP server the runtime starts for PilotQ's tools. It relays and decides nothing.
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
    use crate::manager::{Agent, Limits, Outcome, Runtime};
    use agentq_runtime::optimization::Intent;
    use std::os::unix::fs::PermissionsExt;

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
assert not os.environ.get('ANTHROPIC_API_KEY') and not os.environ.get('ANTHROPIC_AUTH_TOKEN') and os.environ['CLAUDE_CONFIG_DIR'], 'R9: no host API key reaches the runtime'
assert os.environ['AGENTQ_LAUNCHER']=='pilotq' and os.environ['AGENTQ_OPTIMIZATION']=='off'
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
        let c = Runtime { workspace: dir.join("workspace"), logs: dir.join("logs"), optimization: Intent::Off,
            limits: Limits { first_event: Duration::from_secs(10), silent: Duration::from_secs(10), deadline: Duration::from_secs(1) },
            agents: [("claude".to_string(), Agent { bin: bin.to_str().unwrap().into(), home: Some(dir.join("home")), models: vec![] })].into(),
            default: "claude".into(), routes: Default::default(), slots: None };
        let store=coreq::telemetry::Store::open(&dir.join("telemetry.sqlite")).unwrap();
        let tools = json!([{"type": "function", "name": "project_list", "description": "invented", "inputSchema": {"type": "object"}}]);
        let run = |native: bool, session: Option<&str>| {
            let (mut drafts, mut seen, mut stored, mut context) = (vec![], vec![], 0, Value::Null);
            let out = c.turn("claude", native, session, None, &json!([{"type": "text", "text": "hello"}]), (&tools, "invented instructions"),
                || true, |_, id, status| seen.push((id.to_string(), status.map(str::to_string))), |name, _, _| (name == "project_list", "granted result".into()),
                || Ok(json!({"history": "invented"})), |d| drafts.push(d.to_string()), |k, v| if k == "context" { context = v.clone(); }, |record| { record(&store).unwrap(); stored += 1; }, "cli");
            (out, drafts, seen, stored, context)
        };
        let ((session, model, outcome), drafts, seen, stored, context) = run(false, None);
        assert_eq!(outcome, Outcome::Answered("finished hello".into()));
        assert_eq!(model.as_deref(), Some("claude-invented"));
        assert_eq!(drafts, ["draft", "finished hello"]);
        assert_eq!(seen, [("t1".to_string(), None), ("t1".to_string(), Some("completed".to_string()))]);
        assert_eq!(stored, 1, "one usage observation per turn");
        // #374 A3: the real runtime record boundary supplies both kinds; PilotQ never parses calls.
        let rows=store.export(0,0,100).unwrap();
        let events=rows["events"].as_array().unwrap();
        assert_eq!(events.iter().filter(|e|e["observation"]["kind"]=="agent.turn").count(),1);
        let turn=events.iter().find(|e|e["observation"]["kind"]=="agent.turn").unwrap();
        assert_eq!(turn["observation"]["outcome"],"ok","cleanup must not relabel a completed turn as cancelled");
        let calls:Vec<_>=events.iter().filter(|e|e["observation"]["kind"]=="agent.call").collect();
        assert_eq!(calls.len(),1);
        assert_eq!(calls[0]["observation"]["input_tokens"],10);
        assert!(calls[0]["observation"]["reasoning_tokens"].is_null());
        assert!(calls[0]["observation"]["unknown"]["reasoning_tokens"].is_string());
        assert_eq!(context, json!({"used": 10, "window": 100}));
        let session = session.unwrap();
        let ((again, _, outcome), ..) = run(true, Some(&session));
        assert_eq!((again.as_deref(), outcome), (Some(session.as_str()), Outcome::Answered("finished hello".into())));
        let ((rebuilt, _, outcome), ..) = run(true, Some("00000000-0000-4000-8000-000000000000"));
        assert!(rebuilt.is_some_and(|s| s != "00000000-0000-4000-8000-000000000000"));
        assert_eq!(outcome, Outcome::Answered("finished hello".into()));
        assert!(!dir.join("workspace/.pilotq-turns").read_dir().unwrap().any(|_| true), "per-turn relay files are removed");
        std::fs::remove_dir_all(dir).unwrap();
    }

    // #383 A3 / R16: another turn's token must never dispatch a tool, including through the stdio bridge.
    #[test]
    fn mcp_bridge_refuses_foreign_token_383() {
        let socket=std::env::temp_dir().join(format!("pilotq-token383-{}.sock",std::process::id()));
        let _=std::fs::remove_file(&socket);
        let listener=UnixListener::bind(&socket).unwrap();
        let server=std::thread::spawn(move || {
            let (stream,_)=listener.accept().unwrap();
            relay(stream,"invented-current-token",&[],&mut |_,_,_| panic!("foreign token dispatched"),"v").unwrap();
        });
        let input=json!({"id":1,"method":"tools/call","params":{"name":"project_list","arguments":{}}}).to_string()+"\n";
        let mut out=vec![];
        serve_bridge(socket.to_str().unwrap(),"invented-old-token",input.as_bytes(),&mut out);
        server.join().unwrap();
        let reply:Value=serde_json::from_slice(&out).unwrap();
        assert_eq!(reply["result"]["isError"],true);
        assert_eq!(reply["result"]["content"][0]["text"],"tool relay refused");
        std::fs::remove_file(socket).unwrap();
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
