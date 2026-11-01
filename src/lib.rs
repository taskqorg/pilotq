//! Portable manager launch boundary (pilotq.md R9, §6). Service policy and delivery stay in the binary.
use agentq_runtime::{agent::{Client, Spec}, optimization::{Intent, Optimization}};
use serde_json::Value;
use std::process::Command;

pub fn optimization(value: &Value) -> Result<Intent, String> {
    if value.is_null() { return Ok(Intent::Off); }
    value.as_str().and_then(Intent::parse).ok_or_else(|| "config.json: optimization must be off, inherit or on".into())
}

pub fn prepare(intent: Intent, spec: &mut Spec) -> Optimization {
    // Claude excludes user settings; their AgentQ hooks cannot prove effective compression for this launch.
    let mode = if spec.client == Client::Claude { Intent::Off } else { intent };
    let mut state = agentq_runtime::optimization::prepare(mode, "pilotq", spec, agentq_runtime::optimization::install_root().as_deref());
    state.intent = intent;
    if spec.client == Client::Claude && intent != Intent::Off { state.reason = "settings_excluded"; }
    state
}

/// Only runtime necessities reach the child; host credentials and API keys cannot become a paid fallback (R9, R16).
pub fn base(bin: &str) -> Command {
    let mut c = Command::new(bin);
    c.env_clear().env("AGENTQ_LAUNCHER", "pilotq");
    for k in ["PATH", "HOME", "USER", "LANG", "LC_ALL", "TMPDIR", "SSL_CERT_FILE"] {
        if let Some(v) = std::env::var_os(k) { c.env(k, v); }
    }
    #[cfg(windows)]
    if let Some(v) = std::env::var_os("SystemRoot") { c.env("SystemRoot", v); }
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut c, 0);
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentq_runtime::{agent::{AppSession, Client, Context, Spec}, optimization::prepare};
    use serde_json::json;

    fn spec(client: Client, root: &std::path::Path) -> Spec {
        Spec { client, root: root.into(), session: "-".into(), resume: false,
            args: vec![], home: Some(root.join("home")), env: vec![] }
    }

    // R9, §6/#294: optional intent cannot install/enable an optimizer or leak a host credential into a launch.
    #[test]
    fn launch_294_optimization_is_optional_and_child_only() {
        assert_eq!(optimization(&Value::Null).unwrap(), Intent::Off);
        for code in ["off", "inherit", "on"] { assert_eq!(optimization(&json!(code)).unwrap().code(), code); }
        for value in [json!(false), json!(1), json!("invalid")] { assert!(optimization(&value).is_err()); }
        let root = std::env::temp_dir().join(format!("pilotq-294-{}", agentq_runtime::agent::new_session(Client::Claude)));
        std::fs::create_dir_all(root.join("home")).unwrap();
        let config = "model_provider = \"agentq\"\n";
        std::fs::write(root.join("home/config.toml"), config).unwrap();
        for client in [Client::Codex, Client::Claude] {
            let mut launch = spec(client, &root);
            launch.env.push(("AGENTQ_OPTIMIZATION".into(), "off".into()));
            let state = prepare(Intent::On, "pilotq", &mut launch, Some(&root));
            assert_eq!((state.intent, state.effective, state.reason), (Intent::On, "off", "launch_off"));
            let command = agentq_runtime::agent::command(base("invented-never-started"), &launch);
            let env: std::collections::BTreeMap<_, _> = command.get_envs().collect();
            assert_eq!(env.get(std::ffi::OsStr::new("AGENTQ_LAUNCHER")).copied().flatten(), Some(std::ffi::OsStr::new("pilotq")));
            for key in ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GH_TOKEN", "GITHUB_TOKEN"] {
                assert!(!env.contains_key(std::ffi::OsStr::new(key)));
            }
            assert_eq!(std::fs::read_to_string(root.join("home/config.toml")).unwrap(), config);
            assert!(!root.join("data").exists(), "launch must not install an optimizer");
        }
        let mut plain = spec(Client::Codex, &root);
        plain.env.push(("AGENTQ_OPTIMIZATION".into(), "off".into()));
        assert_eq!(prepare(Intent::Inherit, "pilotq", &mut plain, None).effective, "off");
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(root.join("home/settings.json"), r#"{"hooks":{"PostToolUse":[{"hooks":[{"command":"/invented/agentq hook claude PostToolUse"}]}]}}"#).unwrap();
        let mut excluded = spec(Client::Claude, &root);
        excluded.args = ["--setting-sources", "project"].map(Into::into).into();
        excluded.env.push(("AGENTQ_OPTIMIZATION".into(), "on".into()));
        assert_eq!(prepare(Intent::On, "pilotq", &mut excluded, Some(&root)).effective, "on",
            "reproduce the adapter's home-only observation despite the launch excluding user settings");
        let corrected = super::prepare(Intent::On, &mut excluded);
        assert_eq!((corrected.effective, corrected.reason), ("off", "settings_excluded"));
        for intent in [Intent::Off, Intent::Inherit, Intent::On] {
            let mut claude = spec(Client::Claude, &root);
            let state = super::prepare(intent, &mut claude);
            assert_eq!((state.intent, state.effective), (intent, "off"));
            assert_eq!(state.reason, if intent == Intent::Off { "launch_off" } else { "settings_excluded" });
            assert!(claude.env.iter().any(|(k,v)| k == "AGENTQ_OPTIMIZATION" && v == "off"));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    fn answer(s: &mut AppSession, request: Value, result: Value) {
        s.observe(&json!({"id":request["id"],"result":result})).unwrap();
    }

    // §3/6/7/#294: fake native events preserve tools, drafts, catalogue, exact session recovery and one usage record.
    // CoreQ appears here only to prove rollback compatibility with the previous immutable consumer.
    #[test]
    fn sessions_294_preserve_protocol_recovery_and_usage() {
        let root = std::env::temp_dir().join(format!("pilotq-294-session-{}", agentq_runtime::agent::new_session(Client::Claude)));
        std::fs::create_dir_all(&root).unwrap();
        let launch = spec(Client::Codex, &root);
        let mut s = AppSession::new(&launch).unwrap();
        let init = s.initialize(json!({"capabilities":{"experimentalApi":true}})).unwrap();
        answer(&mut s, init, json!({}));
        s.initialized().unwrap();
        for method in ["model/list", "account/rateLimits/read"] {
            let request = s.request(method, json!({})).unwrap();
            s.observe(&json!({"id":request["id"],"error":{"message":"unavailable"}})).unwrap();
            assert!(s.report().is_none(), "a utility read is not inference");
        }
        let tools = json!([{"name":"project_list","inputSchema":{"type":"object"}}]);
        let open = s.open(json!({"dynamicTools":tools,"sandbox":"read-only","approvalPolicy":"never","model":"unchanged"})).unwrap();
        assert_eq!(open["params"]["dynamicTools"], tools);
        assert_eq!(open["params"]["sandbox"], "read-only");
        answer(&mut s, open, json!({"thread":{"id":"invented-thread-294"},"model":"unchanged"}));
        let turn = s.start_turn(json!({"input":[{"type":"text","text":"invented"}]})).unwrap();
        answer(&mut s, turn, json!({"turn":{"id":"invented-turn-294"}}));
        let call = json!({"id":77,"method":"item/tool/call","params":{"tool":"project_list","arguments":{}}});
        s.observe(&call).unwrap();
        let event = |method: &str, mut params: Value| {
            params["threadId"] = json!("invented-thread-294"); params["turnId"] = json!("invented-turn-294");
            json!({"method":method,"params":params})
        };
        let draft = event("item/agentMessage/delta", json!({"delta":"private draft"}));
        s.observe(&draft).unwrap();
        assert!(s.report().is_none(), "a private draft is not completion");
        let checkpoint = s.checkpoint();
        let legacy = coreq::agent::Spec { client:coreq::agent::Client::Codex, root:launch.root.clone(), session:launch.session.clone(),
            resume:launch.resume, args:launch.args.clone(), home:launch.home.clone(), env:launch.env.clone() };
        let old = coreq::agent::AppSession::recover(&legacy, &checkpoint).unwrap();
        s = AppSession::recover(&launch, &old.checkpoint()).unwrap();
        assert_eq!(s.session(), Some("invented-thread-294"));
        assert_eq!(s.turn_id(), Some("invented-turn-294"));
        assert!(s.start_turn(json!({})).is_err(), "unknown original turn must not be replayed");
        let mut foreign = launch.clone(); foreign.home = Some(root.join("other"));
        assert!(AppSession::recover(&foreign, &checkpoint).is_err(), "other account scope cannot recover");
        s.observe(&event("thread/tokenUsage/updated",json!({"tokenUsage":{"last":{"inputTokens":10,"outputTokens":2,"cachedInputTokens":0}}}))).unwrap();
        s.observe(&event("item/completed",json!({"item":{"type":"agentMessage","text":"finished"}}))).unwrap();
        s.observe(&event("turn/completed",json!({"turn":{"id":"invented-turn-294","status":"completed"}}))).unwrap();
        let report = s.report().unwrap();
        assert_eq!((report.outcome, report.text.as_deref()), ("ok", Some("finished")));
        let path = root.join("telemetry.sqlite");
        let store = coreq::telemetry::Store::open(&path).unwrap();
        let ctx = Context { product:"pilotq",version:env!("CARGO_PKG_VERSION"),project:"other",task:None,route:Some("cli"),account:"default" };
        s.record(&ctx, &store).unwrap().unwrap().recorded.unwrap();
        s.record(&ctx, &store).unwrap().unwrap().recorded.unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM events", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1, "same native turn is counted once");
        let exported = coreq::telemetry::Reader::open(&path).unwrap().export(0, 0, 100).unwrap().to_string();
        assert!(exported.contains("cli") && exported.contains("pilotq"), "usage keeps consumer and route attribution");
        drop(conn); drop(store); drop(s);
        std::fs::remove_dir_all(root).unwrap();
    }

    // A5/#294: real Windows child I/O with invented provider events; never starts Codex, Claude or a subscription call.
    #[cfg(windows)]
    #[test]
    fn fake_clients_294_windows_native_transports() {
        use agentq_runtime::agent::{AppServer, Turn};
        use std::time::{Duration, Instant};
        let root = std::env::temp_dir().join(format!("pilotq-294-clients-{}", agentq_runtime::agent::new_session(Client::Claude)));
        std::fs::create_dir_all(root.join("home")).unwrap();
        let script = r#"
$ErrorActionPreference = 'Stop'
if ($env:AGENTQ_LAUNCHER -ne 'pilotq' -or $env:AGENTQ_OPTIMIZATION -ne 'off') { throw 'launch markers missing' }
if ($env:OPENAI_API_KEY -or $env:ANTHROPIC_API_KEY -or $env:GH_TOKEN) { throw 'host credential leaked' }
if ($args[0] -eq 'app-server') {
  while ($line = [Console]::ReadLine()) {
    $m = $line | ConvertFrom-Json
    $r = switch ($m.method) {
      'initialize' { @{} }
      'model/list' { @{data=@(@{model='invented-model'})} }
      'account/rateLimits/read' { @{rateLimits=@{}} }
      'thread/resume' { @{thread=@{id=$m.params.threadId};model='unchanged'} }
    }
    @{id=$m.id;result=$r} | ConvertTo-Json -Depth 10 -Compress
  }
} else {
  $flag = if ($args -contains '--resume') { '--resume' } else { '--session-id' }
  $sid = $args[[Array]::IndexOf($args,$flag)+1]
  if (-not ([Console]::ReadLine()).Contains('invented input')) { throw 'original input lost' }
  @{type='system';subtype='init';session_id=$sid} | ConvertTo-Json -Compress
  @{type='result';subtype='success';is_error=$false;session_id=$sid;result='invented result'} | ConvertTo-Json -Compress
}
"#;
        let child = || {
            let mut c = base("powershell.exe");
            c.args(["-NoProfile", "-NonInteractive", "-Command"]).arg(format!("& {{ {script} }}")); c
        };
        let mut codex = spec(Client::Codex, &root);
        codex.session = "invented-native-294".into(); codex.resume = true;
        prepare(Intent::Off, "pilotq", &mut codex, None);
        let mut server = AppServer::start(child(), &codex, &root.join("codex.log")).unwrap();
        let mut s = AppSession::new(&codex).unwrap();
        for (method, params) in [("initialize", json!({})), ("model/list", json!({})), ("account/rateLimits/read", json!({})),
            ("thread/resume", json!({"threadId":"invented-native-294"}))] {
            let request = if method == "initialize" { s.initialize(params).unwrap() }
                else if method == "thread/resume" { s.open(params).unwrap() } else { s.request(method, params).unwrap() };
            server.send(&request, Instant::now()+Duration::from_secs(10)).unwrap();
            let result = server.receive(Instant::now()+Duration::from_secs(10)).unwrap().unwrap();
            if method == "model/list" { assert_eq!(result["result"]["data"][0]["model"], "invented-model"); }
            s.observe(&result).unwrap();
        }
        assert_eq!(s.session(), Some("invented-native-294"));
        assert!(s.report().is_none(), "catalogue/account/session reads do not count a model turn");
        server.kill().unwrap();
        let mut claude = spec(Client::Claude, &root);
        claude.session = agentq_runtime::agent::new_session(Client::Claude);
        prepare(Intent::Off, "pilotq", &mut claude, None);
        for resume in [false, true] {
            claude.resume = resume;
            let mut turn = Turn::start(child(), &claude, &root.join("claude.log"), "invented input\n").unwrap();
            turn.wait(Some(Instant::now()+Duration::from_secs(10))).unwrap();
            let report = turn.report();
            assert!(report.exit.is_some(), "child termination must be confirmed after the native result");
            assert_eq!((report.session.as_deref(), report.outcome, report.text.as_deref()),
                (Some(claude.session.as_str()), "ok", Some("invented result")));
        }
        drop(server);
        std::fs::remove_dir_all(root).unwrap();
    }
}
