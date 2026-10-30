//! R15 / §5: bounded installation administration, sharing the request/grant/audit boundary.
use crate::{direct::Direct, journal::{Journal, Run}, taskq::Taskq};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use std::{path::Path, sync::Mutex, time::{Duration, Instant}};

pub const NAMES: [&str; 4] = ["project_register", "user_grant", "user_revoke", "topic_open"];
fn err(e: rusqlite::Error) -> String { e.to_string() }
fn now() -> i64 { crate::now() as i64 }

pub fn tools() -> Vec<Value> {
    let tool = |name, description, properties, required| json!({"type":"function","name":name,"description":description,
        "inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false}});
    let mut tools = vec![tool("project_register", "Projects administrators: register an existing repository, preparing a local root. Never creates a repository or a claim issue. Return filing readiness and missing prerequisites.",
        json!({"project":{"type":"string"},"repo":{"type":"string","description":"Exact owner/name named by the caller"},"board":{"type":"string","enum":["github","gitlab"]},"host":{"type":"string","description":"Board web host, e.g. github.com"}}), json!(["project","repo","board","host"]))];
    for name in ["user_grant", "user_revoke"] {
        tools.push(tool(name,"Users administrators: change named grants for an exact Telegram ID seen in this route, explicitly named or supplied by the current reply. Names never authorize. Owner identity is protected.",
            json!({"user":{"type":"integer"},"grants":{"type":"array","minItems":1,"items":{"type":"string"}},"route":{"type":"string","description":"CLI only: exact CHAT:TOPIC named by caller"}}),json!(["user","grants"])));
    }
    tools.push(tool("topic_open","Users administrator with project access: create a forum topic in this group and bind its default project. Requires bot can_manage_topics. Never repeat an uncertain creation.",
        json!({"project":{"type":"string"},"name":{"type":"string"},"shared":{"type":"boolean","description":"Default false; true only when caller explicitly asks for shared conversation"},"chat":{"type":"integer","description":"CLI only: exact forum supergroup ID named by caller"}}),json!(["project","name"])));
    tools
}

fn authorize(j: &Journal, r: &Run, tool: &str, args: &Value) -> Result<i64,String> {
    let user = j.sender(r.request)?;
    let grant = if tool == "project_register" { "projects" } else { "users" };
    if j.fenced(r)? || !j.has(user,"chat")? || !j.has(user,grant)? { return Err(format!("Administration refused: current chat and {grant} grants required, and run must be active.")); }
    if tool == "topic_open" {
        if args["shared"] == true && !shared_requested(&r.text) {
            return Err("Shared mode requires an explicit shared topic or shared conversation request; clarify the session policy. No topic created.".into());
        }
        let project = args["project"].as_str().ok_or("project required")?;
        if !(j.has(user,"projects")? || j.has(user,&format!("project:{project}"))?) { return Err("Project access required; no topic created.".into()); }
        if crate::direct::context_project(j,r,user)?.or(j.current_project(&r.session)?).as_deref() != Some(project) {
            return Err("Name one accessible project in your own message; no topic created.".into());
        }
    }
    Ok(user)
}

fn existing(j: &Journal, request: i64, tool: &str, args: &Value) -> Result<Option<Value>,String> {
    let row: Option<(String,Option<String>)> = j.0.query_row("SELECT args,receipt FROM admin_ops WHERE request=?1 AND tool=?2",params![request,tool],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(err)?;
    match row {
        None => Ok(None),
        Some((saved,receipt)) => {
            if serde_json::from_str::<Value>(&saved).map_err(|_|"Invalid saved administration intent")? != *args { return Err("This request already bound another operation of this type; send a new request.".into()); }
            receipt.map(|v|serde_json::from_str(&v).map(Some).map_err(|_|"Invalid administration receipt".into()))
                .unwrap_or_else(||Err("Administration outcome is uncertain or partial; automatic replay is blocked. Inspect the saved operation before sending a new request.".into()))
        }
    }
}
fn start(j: &Journal, r: &Run, tool: &str, args: &Value) -> Result<(),String> {
    authorize(j,r,tool,args)?;
    j.0.execute("INSERT INTO admin_ops(request,tool,args) VALUES (?1,?2,?3)",params![r.request,tool,args.to_string()]).map_err(err)?;
    j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'admin',?3,?4,'intent')",
        params![r.request,now(),tool,args.to_string()]).map_err(err)?;
    Ok(())
}
fn receipt(j: &Journal, request: i64, tool: &str, result: &Value) -> Result<(),String> {
    j.0.execute("UPDATE admin_ops SET receipt=?3 WHERE request=?1 AND tool=?2",params![request,tool,result.to_string()]).map_err(err)?;
    j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'admin',?3,?4)",params![request,now(),tool,result.to_string()]).map_err(err)?;
    Ok(())
}
fn route(j: &Journal, r: &Run, args: &Value) -> Result<(i64,i64),String> {
    let (front,chat,topic): (String,i64,i64) = j.0.query_row("SELECT front,chat,coalesce(topic,0) FROM requests WHERE id=?1",[r.request],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).map_err(err)?;
    if front == "telegram" { return Ok((chat,topic)); }
    let route = args["route"].as_str().ok_or("CLI user administration requires an explicit CHAT:TOPIC route")?;
    if !own_word(&r.text,route) { return Err("Name the exact route in your own request".into()); }
    let (chat,topic)=route.split_once(':').unwrap_or((route,"0"));
    Ok((chat.parse().map_err(|_|"Invalid chat ID")?,topic.parse().map_err(|_|"Invalid topic ID")?))
}
fn own_word(text: &str, word: &str) -> bool {
    crate::direct::own_text(text).split_whitespace().any(|w|w.trim_matches([',',';','(',')'])==word)
}
// Visibility changes need positive caller wording, never a model-selected boolean.
fn shared_requested(text: &str) -> bool {
    let own=crate::direct::own_text(text).to_lowercase();
    let words:Vec<_>=own.split(|c:char|!c.is_alphanumeric() && c!='\'').filter(|w|!w.is_empty()).collect();
    if words.iter().any(|w|["not","no","never","don't","dont","не","нет","без"].contains(w)) { return false; }
    words.windows(2).any(|w|matches!((w[0],w[1]),("shared","topic"|"conversation")|("общую","тему")|("общий","разговор")))
}
fn seen(j: &Journal, _r: &Run, target: i64, chat: i64, topic: i64) -> Result<bool,String> {
    // A quote is trusted Telegram envelope metadata, never the quoted text or a forwarded author.
    j.0.query_row("SELECT EXISTS(SELECT 1 FROM requests WHERE chat=?1 AND coalesce(topic,0)=?2 AND sender=?3 AND front='telegram') OR EXISTS(
        SELECT 1 FROM audit_events e JOIN requests q ON q.id=e.request_id WHERE e.kind='quote' AND q.chat=?1 AND coalesce(q.topic,0)=?2
        AND json_extract(e.item,'$.author')=?3)",params![chat,topic,target],|x|x.get(0)).map_err(err)
}
fn user_change(j: &mut Journal, r: &Run, tool: &str, args: &Value) -> Result<Value,String> {
    let actor=authorize(j,r,tool,args)?;
    if let Some(v)=existing(j,r.request,tool,args)? { return Ok(v); }
    let target=args["user"].as_i64().filter(|n|*n>0).ok_or("Exact positive Telegram ID required")?;
    if j.owner()? == Some(target) { return Err("Protected installation owner cannot be changed through chat tools.".into()); }
    let (chat,topic)=route(j,r,args)?;
    if !seen(j,r,target,chat,topic)? { return Err("Person has not been seen in this route; reply to their message with the exact Telegram ID.".into()); }
    let quoted: bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM audit_events WHERE request_id=?1 AND kind='quote' AND json_extract(item,'$.author')=?2)",params![r.request,target],|x|x.get(0)).map_err(err)?;
    if !quoted && !own_word(&r.text,&target.to_string()) { return Err("Name the exact Telegram ID in your own message; names do not authorize grants.".into()); }
    let names=args["grants"].as_array().filter(|a|!a.is_empty()).ok_or("Name at least one grant")?;
    if names.iter().any(|n|n.as_str().is_some_and(|s|s.starts_with("release:"))) {return Err("Release grants are protected CLI only".into());}
    let names:Vec<String>=names.iter().map(|n|n.as_str().filter(|g|crate::journal::valid_grant(g)).map(str::to_string).ok_or("Invalid grant name")).collect::<Result<_,_>>()?;
    if names.iter().any(|name|!own_word(&r.text,name)) {
        return Err("Name each exact grant in your own request; model arguments cannot authorize additional rights.".into());
    }
    let before=j.grants(target)?;
    let tx=j.0.unchecked_transaction().map_err(err)?;
    start(j,r,tool,args)?;
    let after=Journal::change_in(&tx,Some(actor),target,now(),|g| {
        if tool=="user_grant" { g.extend(names); } else { g.retain(|n|!names.contains(n)); }
    })?;
    let readback=j.grants(target)?;
    if readback!=after { return Err("Grant readback mismatch".into()); }
    let result=json!({"outcome":"accepted","user":target,"actor":actor,"before":before,"after":readback,"chat":chat,"topic":topic});
    receipt(j,r.request,tool,&result)?;
    tx.commit().map_err(err)?;
    Ok(result)
}

pub fn call(d: &Direct, j: &Mutex<Journal>, r: &Run, tool: &str, args: &Value, deadline: Instant, tq: Option<&Taskq>) -> (bool,String) {
    let result=(|| {
        if Instant::now()>=deadline { return Err("Administration deadline reached; nothing done".into()); }
        if matches!(tool,"user_grant"|"user_revoke") { return user_change(&mut j.lock().unwrap(),r,tool,args); }
        { let j=j.lock().unwrap(); authorize(&j,r,tool,args)?; if let Some(v)=existing(&j,r.request,tool,args)? { return Ok(v); } }
        match tool {
            "project_register" => register(d,j,r,args,deadline,tq.ok_or("TaskQ is not configured")?),
            "topic_open" => topic(j,r,args,deadline,tq.and_then(|t|t.bot.as_ref()).ok_or("Telegram bot is not configured")?),
            _ => Err("Unknown administration tool".into()),
        }
    })();
    match result { Ok(v)=>(v["outcome"]=="accepted",v.to_string()),Err(e)=>(false,e) }
}

fn component(s: &str) -> bool { !s.is_empty() && s.len()<=100 && s!="." && s!=".." && s.bytes().all(|c|c.is_ascii_alphanumeric() || b"-_.".contains(&c)) }
fn config_root(d: &Direct, project: &str, config: &Value) -> Result<std::path::PathBuf,String> {
    let base=d.keys.0.parent().ok_or("Installation directory unavailable")?.join("projects");
    std::fs::create_dir_all(&base).map_err(|_|"Cannot prepare projects directory")?;
    if std::fs::symlink_metadata(&base).map_err(|_|"Projects directory unavailable")?.file_type().is_symlink() { return Err("Projects directory must not be a symlink".into()); }
    let root=base.join(project);
    match std::fs::create_dir(&root) {
        Ok(())=>(), Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>(), Err(_)=>return Err("Cannot create project root".into()),
    }
    std::fs::File::open(&base).and_then(|f|f.sync_all()).map_err(|_|"Projects directory sync failed")?;
    let meta=std::fs::symlink_metadata(&root).map_err(|_|"Project root unavailable")?;
    if !meta.is_dir() || meta.file_type().is_symlink() { return Err("Project root must be an ordinary directory".into()); }
    let path=root.join("taskq.json");
    let mut options=std::fs::OpenOptions::new(); options.write(true).create_new(true);
    #[cfg(unix)] std::os::unix::fs::OpenOptionsExt::mode(&mut options,0o600);
    match options.open(&path) {
        Ok(mut file)=>{
            use std::io::Write;
            file.write_all(config.to_string().as_bytes()).and_then(|_|file.sync_all()).map_err(|_|"Project configuration write failed; inspect partial root")?;
            std::fs::File::open(&root).and_then(|f|f.sync_all()).map_err(|_|"Project directory sync failed")?;
        },
        Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{
            if std::fs::symlink_metadata(&path).map_err(|_|"Project configuration unavailable")?.file_type().is_symlink() { return Err("Project configuration symlink refused".into()); }
            let saved:Value=std::fs::read_to_string(&path).ok().and_then(|s|serde_json::from_str(&s).ok()).ok_or("Existing configuration unreadable; nothing overwritten")?;
            // Existing owner-provisioned claim settings are preserved.
            for key in ["repo","board","host"] { if saved[key]!=config[key] { return Err("Existing project configuration conflicts; nothing overwritten".into()); } }
        }, Err(_)=>return Err("Project configuration could not be created".into()),
    }
    std::fs::canonicalize(root).map_err(|_|"Project root unavailable".into())
}
fn register(d: &Direct, j: &Mutex<Journal>, r: &Run, args: &Value, deadline: Instant, tq: &Taskq) -> Result<Value,String> {
    let project=args["project"].as_str().filter(|s|component(s)).ok_or("Project must be a single safe directory name")?;
    let repo=args["repo"].as_str().ok_or("Repository required")?;
    let parts:Vec<_>=repo.split('/').collect();
    if parts.len()!=2 || !parts.iter().all(|s|component(s)) || !own_word(&r.text,repo) { return Err("Name the exact existing owner/repository in your own request".into()); }
    let board=args["board"].as_str().filter(|s|["github","gitlab"].contains(s)).ok_or("TaskQ board kind required")?;
    let host=args["host"].as_str().filter(|s|s.contains('.') && s.split('.').all(component)).ok_or("Exact board web host required")?;
    if !own_word(&r.text,host) && !matches!((board,host),("github","github.com")|("gitlab","gitlab.com")) { return Err("Name the custom board host explicitly".into()); }
    {
        let j=j.lock().unwrap();
        if j.projects()?.iter().any(|p|p.0==project) { return Err("Project name already registered; registration does not rebind it".into()); }
        let tx=j.0.unchecked_transaction().map_err(err)?;
        start(&j,r,"project_register",args)?; tx.commit().map_err(err)?;
    }
    let config=json!({"repo":repo,"board":board,"host":host});
    let root=config_root(d,project,&config)?;
    // Same bounded executor as installed reads: deadline, grant revocation, process group and output cap.
    let active=||authorize(&j.lock().unwrap(),r,"project_register",args).is_ok();
    let output=crate::commands::execute_bounded(crate::commands::command(&tq.bin,root.to_str().ok_or("Non-UTF8 project root")?,&["status","--json"]),deadline,&active,256*1024);
    let snapshot:Value=output["stdout"].as_str().and_then(|s|serde_json::from_str(s).ok()).unwrap_or(Value::Null);
    if output["status"]!="success" || snapshot["complete"]!=true || snapshot["board"]["repo"]!=repo || snapshot["board"]["kind"] .as_str().is_some_and(|v|v!=board)
        || snapshot["board"]["host"].as_str().is_some_and(|v|v!=host) {
        return Err("Local root prepared, but TaskQ board readback is unavailable or mismatched; project is not registered and filing is disabled. Inspect the partial registration.".into());
    }
    let j=j.lock().unwrap(); authorize(&j,r,"project_register",args)?;
    let tx=j.0.unchecked_transaction().map_err(err)?;
    j.add_project(j.sender(r.request)?,project,repo,root.to_str().unwrap(),now())?;
    let saved=j.projects()?.into_iter().find(|p|p.0==project).ok_or("Project registration readback failed")?;
    let config:Value=serde_json::from_str(&std::fs::read_to_string(root.join("taskq.json")).map_err(|_|"Configuration readback failed")?).map_err(|_|"Configuration readback invalid")?;
    let claim_configured=config["create_claim_issue"].as_i64().is_some_and(|n|n>0);
    crate::onboarding::queue(&j,project)?;
    let ready=false;
    let result=json!({"outcome":"accepted","project":saved.0,"repo":saved.1,"root":saved.2,"board":board,"host":host,"filing_ready":ready,
        "onboarding":"pending", "claim_configured":claim_configured,
        "filing_note":"Registered; setup checks are pending. The owner receives onboarding steps in Telegram. Filing is disabled until checks pass or the owner records a readiness override; claim prerequisites remain required."});
    receipt(&j,r.request,"project_register",&result)?; tx.commit().map_err(err)?; Ok(result)
}

// Bot API errors contain token-bearing URLs; only fixed diagnostics cross this boundary.
fn api(j: &Mutex<Journal>, bot: &teloxide::Bot, method: &str, body: Value, end: Instant) -> Result<Value,String> {
    let duration=end.checked_duration_since(Instant::now()).ok_or("Administration deadline reached")?.min(Duration::from_secs(4));
    let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|"Telegram runtime unavailable")?;
    runtime.block_on(async {
        let url=bot.api_url().join(&format!("./bot{}/{method}",bot.token())).map_err(|_|"Invalid Telegram configuration")?;
        let response=bot.client().post(url).json(&body).timeout(duration).send().await.map_err(|_|"Telegram request unconfirmed")?;
        let value:Value=response.json().await.map_err(|_|"Telegram response unreadable")?;
        if value["ok"]!=true {
            if value["error_code"]==429 {
                let chat=body["chat_id"].as_i64().unwrap_or(0);
                let after=value["parameters"]["retry_after"].as_i64().unwrap_or(1).max(1);
                j.lock().unwrap().cool_telegram(chat,now().saturating_add(after).saturating_add(1))?;
            }
            return Err("Telegram refused the administration request".into());
        }
        Ok(value["result"].clone())
    })
}
fn topic(j: &Mutex<Journal>, r: &Run, args: &Value, deadline: Instant, bot: &teloxide::Bot) -> Result<Value,String> {
    let name=args["name"].as_str().filter(|s|!s.trim().is_empty() && s.chars().count()<=128).ok_or("Topic name must be 1–128 characters")?;
    let (front,mut chat):(String,i64)=j.lock().unwrap().0.query_row("SELECT front,chat FROM requests WHERE id=?1",[r.request],|x|Ok((x.get(0)?,x.get(1)?))).map_err(err)?;
    if front=="cli" { chat=args["chat"].as_i64().filter(|c|*c<0 && own_word(&r.text,&c.to_string())).ok_or("CLI topic creation requires an explicitly named negative chat ID")?; }
    if chat>=0 { return Err("Open project topics from a forum supergroup; no topic created.".into()); }
    let project=args["project"].as_str().ok_or("Project required")?;
    let binding=j.lock().unwrap().projects()?.into_iter().find(|p|p.0==project).ok_or("Project is not registered")?;
    let config:Value=std::fs::read_to_string(Path::new(&binding.2).join("taskq.json")).ok().and_then(|s|serde_json::from_str(&s).ok()).ok_or("Registered project configuration unavailable")?;
    if config["repo"]!=binding.1 { return Err("Registered board/root changed; no topic created".into()); }
    if j.lock().unwrap().telegram_cooldown(chat)?>now() { return Err("Telegram rate limit active; no topic created".into()); }
    let me=api(j,bot,"getMe",json!({}),deadline)?;
    let id=me["id"].as_i64().filter(|n|*n>0).ok_or("Bot identity unavailable")?;
    let group=api(j,bot,"getChat",json!({"chat_id":chat}),deadline)?;
    let member=api(j,bot,"getChatMember",json!({"chat_id":chat,"user_id":id}),deadline)?;
    if group["is_forum"]!=true || group["type"]!="supergroup" || member["status"]!="administrator" || member["can_manage_topics"]!=true {
        return Err("Cannot open topic: this must be a forum supergroup and the bot needs can_manage_topics permission.".into());
    }
    {
        let j=j.lock().unwrap();
        if j.telegram_cooldown(chat)?>now() { return Err("Telegram rate limit active; no topic created".into()); }
        if !j.projects()?.iter().any(|p|p.0==binding.0 && p.1==binding.1 && p.2==binding.2) { return Err("Project registration changed; no topic created".into()); }
        let tx=j.0.unchecked_transaction().map_err(err)?;
        start(&j,r,"topic_open",args)?; tx.commit().map_err(err)?;
    }
    let created=api(j,bot,"createForumTopic",json!({"chat_id":chat,"name":name}),deadline)
        .map_err(|_|"Topic creation is uncertain; automatic replay is blocked. Inspect the forum before requesting another topic.")?;
    let topic=created["message_thread_id"].as_i64().filter(|n|*n>0).ok_or("Topic response has no ID; creation is uncertain and will not be repeated")?;
    let external=json!({"chat":chat,"topic":topic,"project":project,"root":binding.2,"repo":binding.1,"shared":args["shared"].as_bool().unwrap_or(false)});
    let mut j=j.lock().unwrap();
    j.0.execute("UPDATE admin_ops SET external=?2 WHERE request=?1 AND tool='topic_open'",params![r.request,external.to_string()]).map_err(err)?;
    authorize(&j,r,"topic_open",args)?;
    bind_topic(&mut j,r.request,&external)
}
fn bind_topic(j: &mut Journal, request: i64, e: &Value) -> Result<Value,String> {
    let valid:bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM requests q JOIN runs r ON r.request_id=q.id JOIN conversations c ON c.session=r.session WHERE q.id=?1 AND q.state<>'cancelled' AND r.generation=c.generation AND r.fence=c.fence)",[request],|r|r.get(0)).map_err(err)?;
    if !valid { return Err("Topic exists but its originating request was stopped or reset; local binding is blocked".into()); }
    let actor=j.sender(request)?;
    let project=e["project"].as_str().ok_or("Saved project missing")?;
    if !j.has(actor,"chat")? || !j.has(actor,"users")? || !(j.has(actor,"projects")? || j.has(actor,&format!("project:{project}"))?) { return Err("Topic exists but route binding is blocked by revoked grants".into()); }
    if !j.projects()?.iter().any(|p|p.0==project && e["repo"]==p.1 && e["root"]==p.2) { return Err("Topic exists but registered project changed; binding blocked".into()); }
    let (chat,topic)=(e["chat"].as_i64().ok_or("Saved chat missing")?,e["topic"].as_i64().ok_or("Saved topic missing")?);
    let shared=e["shared"]==true;
    let text:String=j.0.query_row("SELECT text FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(err)?;
    if shared && !shared_requested(&text) { return Err("Topic exists but shared conversation was not explicitly authorized; local binding blocked".into()); }
    let root=j.projects()?.into_iter().find(|p|p.0==project).ok_or("Registered project missing")?.2;
    let config:Value=std::fs::read_to_string(Path::new(&root).join("taskq.json")).ok().and_then(|s|serde_json::from_str(&s).ok()).ok_or("Topic exists but project configuration is unavailable")?;
    if config["repo"]!=e["repo"] { return Err("Topic exists but project board changed; binding blocked".into()); }
    let tx=j.0.unchecked_transaction().map_err(err)?;
    tx.execute("INSERT INTO routes(chat,topic,shared,by,at) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(chat,topic) DO NOTHING",params![chat,topic,shared,actor,now()]).map_err(err)?;
    tx.execute("INSERT INTO route_projects(chat,topic,project) VALUES (?1,?2,?3) ON CONFLICT(chat,topic) DO NOTHING",params![chat,topic,project]).map_err(err)?;
    let saved:(bool,String)=tx.query_row("SELECT r.shared,p.project FROM routes r JOIN route_projects p USING(chat,topic) WHERE r.chat=?1 AND r.topic=?2",params![chat,topic],|x|Ok((x.get(0)?,x.get(1)?))).map_err(err)?;
    if saved!=(shared,project.to_string()) { return Err("Topic exists but route policy conflicts; inspect route configuration".into()); }
    let result=json!({"outcome":"accepted","chat":chat,"topic":topic,"project":project,"session":if shared {"shared"} else {"isolated"}});
    receipt(j,request,"topic_open",&result)?; tx.commit().map_err(err)?; Ok(result)
}

/// §4: finish a known topic's local binding; never replay an external create or a completed grant change.
pub fn recover(j: &mut Journal, at: i64) -> Result<(),String> {
    let pending:Vec<(i64,String,Option<String>)>={
        let mut q=j.0.prepare("SELECT request,tool,external FROM admin_ops WHERE receipt IS NULL").map_err(err)?;
        q.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(err)?.collect::<Result<_,_>>().map_err(err)?
    };
    for (request,tool,external) in pending {
        let result=if tool=="topic_open" {
            external.and_then(|e|serde_json::from_str(&e).ok()).map(|e|bind_topic(j,request,&e))
                .unwrap_or_else(||Err("Topic creation uncertain; it is never repeated automatically. Inspect the forum before creating another topic.".into()))
        } else { Err("Project registration partial; inspect its local root and TaskQ readback before sending a new request.".into()) };
        let text=match result { Ok(v)=>format!("Administration recovered: {v}"),Err(e)=>e };
        j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'admin_recovery',?3,?4)",params![request,at,tool,text]).map_err(err)?;
        // Existing outbox receipt/retry machinery owns delivery; recover each operation notice once.
        let key=format!("admin-recovered:{request}:{tool}");
        let tx=j.0.unchecked_transaction().map_err(err)?;
        let added=tx.execute("INSERT OR IGNORE INTO meta(key,value) VALUES (?1,?2)",params![key,at]).map_err(err)?;
        if added>0 {
            tx.execute("INSERT INTO outbox(request_id,chat,topic,text,state,next_at) SELECT id,chat,topic,?2,'pending',?3 FROM requests WHERE id=?1 AND front='telegram'",params![request,text,at]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::Input;
    use std::{io::{Read,Write},net::TcpListener,path::PathBuf};
    fn setup() -> (Journal,PathBuf,Direct) {
        let dir=std::env::temp_dir().join(format!("pilotq-admin-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap(); j.bootstrap(7,0).unwrap();
        let d=Direct{keys:crate::keys::Keys(dir.join("keys.json"))}; (j,dir,d)
    }
    fn input(user:i64,message:i64,text:&str) -> Input {
        Input{files:vec![],callback:None,media_group:None,quote:None,image:None,voice:None,language:None,chat:-100,topic:Some(5),sender:Some(user),name:None,message,kind:"text",text:Some(text.into()),reply_to:None}
    }
    fn run(j:&mut Journal,user:i64,message:i64,text:&str) -> Run {
        j.admit(&[(message,Some(input(user,message,text)))],message).unwrap();
        j.claim_next(4,message).unwrap().into_iter().find(|r|j.sender(r.request).unwrap()==user).unwrap()
    }
    fn finish(j:&mut Journal,r:&Run) { j.finish(r,None,None,&crate::manager::Outcome::Answered("saved".into()),100).unwrap(); }
    fn end() -> Instant { Instant::now()+Duration::from_secs(10) }
    fn fake_bot(responses:Vec<Option<Value>>) -> (teloxide::Bot,std::thread::JoinHandle<Vec<String>>) {
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let url=format!("http://{}/",listener.local_addr().unwrap());
        let handle=std::thread::spawn(move || {
            let mut calls=vec![];
            for response in responses {
                let (mut stream,_)=listener.accept().unwrap(); stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut bytes=vec![]; let mut b=[0;4096];
                loop {
                    let n=stream.read(&mut b).unwrap(); if n==0 {break;} bytes.extend_from_slice(&b[..n]);
                    if let Some(i)=bytes.windows(4).position(|w|w==b"\r\n\r\n") {
                        let headers=String::from_utf8_lossy(&bytes[..i]).to_lowercase();
                        let len=headers.lines().find_map(|l|l.strip_prefix("content-length: ").and_then(|n|n.parse::<usize>().ok())).unwrap_or(0);
                        if bytes.len()>=i+4+len {break;}
                    }
                }
                calls.push(String::from_utf8_lossy(&bytes).lines().next().unwrap().to_string());
                if let Some(result)=response {
                    let body=json!({"ok":true,"result":result}).to_string();
                    write!(stream,"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",body.len(),body).unwrap();
                }
            }
            calls
        });
        // Invented token, never a credential.
        (teloxide::Bot::new("123:invented").set_api_url(url.parse().unwrap()),handle)
    }
    fn registered(j:&Journal,dir:&Path) {
        let root=dir.join("demo");std::fs::create_dir_all(&root).unwrap();std::fs::write(root.join("taskq.json"),r#"{"repo":"invented/demo"}"#).unwrap();
        j.add_project(7,"demo","invented/demo",root.to_str().unwrap(),0).unwrap();
    }
    fn tq(bot:Option<teloxide::Bot>) -> Taskq { Taskq{bot,bin:"/unused".into(),root:"/unused".into(),poll:Duration::from_secs(15)} }

    // §4/§8: after grants change, native sessions must receive the newly scoped tools, not their old catalogue.
    #[test]
    fn admin_160_catalogue_refresh() {
        let (mut j,dir,_)=setup();
        let r=j.ask(7,"hello",1).unwrap();
        let ordinary=json!([{"name":"project_list"}]);
        j.catalogue(&r,&ordinary).unwrap();
        j.finish(&r,Some("invented-native-thread"),None,&crate::manager::Outcome::Answered("hello".into()),2).unwrap();
        let r=j.ask(7,"administer",3).unwrap();
        j.catalogue(&r,&ordinary).unwrap();assert!(!j.refresh_tools(&r).unwrap());
        j.catalogue(&r,&json!(tools())).unwrap();assert!(j.refresh_tools(&r).unwrap());
        j.rebuild(&r,4).unwrap();
        j.finish(&r,Some("invented-replacement-thread"),None,&crate::manager::Outcome::Answered("ready".into()),5).unwrap();
        assert!(!j.refresh_tools(&r).unwrap());
        let _=std::fs::remove_dir_all(dir);
    }

    // R10/R15/R17: authorization and route identity, atomic audit, no replay after later corrections.
    #[test]
    fn admin_160_grants_trace() {
        let (mut j,dir,d)=setup();
        j.grant(7,8,&["chat".into()],0).unwrap();
        let seen=run(&mut j,8,1,"hello"); finish(&mut j,&seen);
        let r=run(&mut j,7,2,"grant 8 project:demo");
        let args=json!({"user":8,"grants":["project:demo"]});let m=Mutex::new(j);
        assert!(call(&d,&m,&r,"user_grant",&args,end(),None).0);
        assert!(m.lock().unwrap().has(8,"project:demo").unwrap());
        m.lock().unwrap().revoke(7,8,&["project:demo".into()],3).unwrap();
        assert!(call(&d,&m,&r,"user_grant",&args,end(),None).0);
        assert!(!m.lock().unwrap().has(8,"project:demo").unwrap(),"saved receipt must not reapply a corrected grant");
        assert!(!call(&d,&m,&r,"user_grant",&json!({"user":9,"grants":["chat"]}),end(),None).0);
        finish(&mut m.lock().unwrap(),&r);
        let r=run(&mut m.lock().unwrap(),7,4,"grant 9 chat");
        assert!(!call(&d,&m,&r,"user_grant",&json!({"user":9,"grants":["chat"]}),end(),None).0,"unseen person refused");
        finish(&mut m.lock().unwrap(),&r);
        let mut quote=input(7,5,"grant this person chat");quote.quote=Some(json!({"author":9,"message":20}));quote.reply_to=Some(20);
        m.lock().unwrap().admit(&[(5,Some(quote))],5).unwrap();
        let claimed=m.lock().unwrap().claim_next(4,5).unwrap();
        let r=claimed.into_iter().find(|r|m.lock().unwrap().sender(r.request).unwrap()==7).unwrap();
        assert!(call(&d,&m,&r,"user_grant",&json!({"user":9,"grants":["chat"]}),end(),None).0);
        finish(&mut m.lock().unwrap(),&r);
        let r=run(&mut m.lock().unwrap(),7,6,"revoke 9 chat");
        assert!(call(&d,&m,&r,"user_revoke",&json!({"user":9,"grants":["chat"]}),end(),None).0);
        assert!(!m.lock().unwrap().has(9,"chat").unwrap());
        finish(&mut m.lock().unwrap(),&r);
        let r=run(&mut m.lock().unwrap(),8,7,"grant 9 chat");
        assert!(!call(&d,&m,&r,"user_grant",&json!({"user":9,"grants":["chat"]}),end(),None).0,"chat grant alone never permits administration");
        drop(m);let _=std::fs::remove_dir_all(dir);
    }

    // R10/§8: model arguments cannot expand the rights or conversation visibility requested by the caller.
    #[test]
    fn admin_160_caller_scope() {
        let (mut j,dir,_)=setup();j.grant(7,8,&["chat".into()],0).unwrap();
        let r=run(&mut j,8,1,"hello");finish(&mut j,&r);
        for (message,text,tool) in [(2,"grant 8 chat","user_grant"),(3,"revoke 8 chat","user_revoke"),(4,"grant 8 chat\n> users","user_grant")] {
            let r=run(&mut j,7,message,text);
            assert!(user_change(&mut j,&r,tool,&json!({"user":8,"grants":["users"]})).is_err(),"unrequested users right must be refused");
            assert_eq!(j.grants(8).unwrap(),vec!["chat".to_string()]);
            assert_eq!(j.0.query_row("SELECT count(*) FROM admin_ops WHERE request=?1",[r.request],|r|r.get::<_,i64>(0)).unwrap(),0);
            finish(&mut j,&r);
        }
        let _=std::fs::remove_dir_all(dir);
    }

    #[test]
    fn admin_160_caller_shared_scope() {
        let (mut j,dir,_)=setup();registered(&j,&dir);
        for (message,text) in [(5,"open topic demo"),(6,"open topic demo\n> shared topic"),(7,"open topic demo, not a shared topic")] {
            let r=run(&mut j,7,message,text);
            assert!(authorize(&j,&r,"topic_open",&json!({"project":"demo","name":"Demo","shared":true})).is_err(),"shared mode requires caller consent before Bot API");
            let binding=j.projects().unwrap().pop().unwrap();
            let external=json!({"chat":-100,"topic":42,"project":"demo","repo":binding.1,"root":binding.2,"shared":true});
            assert!(bind_topic(&mut j,r.request,&external).is_err(),"saved topic ID cannot expand authorized visibility during recovery");
            assert!(!j.routes().unwrap().iter().any(|route|route.topic==42));
            finish(&mut j,&r);
        }
        for (message,text) in [(8,"open shared topic demo"),(9,"открой общую тему demo")] {
            let r=run(&mut j,7,message,text);
            assert!(authorize(&j,&r,"topic_open",&json!({"project":"demo","name":"Demo","shared":true})).is_ok());
            finish(&mut j,&r);
        }
        let _=std::fs::remove_dir_all(dir);
    }

    // §4: injected receipt-write interruption rolls back grant, grant log and intent together.
    #[test]
    fn admin_160_grant_atomicity() {
        let (mut j,dir,d)=setup();j.grant(7,8,&["chat".into()],0).unwrap();
        let r=run(&mut j,8,1,"hello");finish(&mut j,&r);
        let r=run(&mut j,7,2,"grant 8 users");
        j.0.execute_batch("CREATE TRIGGER fail_receipt BEFORE UPDATE OF receipt ON admin_ops BEGIN SELECT RAISE(ABORT,'injected interruption'); END;").unwrap();
        let m=Mutex::new(j);let args=json!({"user":8,"grants":["users"]});
        assert!(!call(&d,&m,&r,"user_grant",&args,end(),None).0);
        assert!(!m.lock().unwrap().has(8,"users").unwrap());
        assert_eq!(m.lock().unwrap().grant_log(8).unwrap().len(),1);
        m.lock().unwrap().0.execute_batch("DROP TRIGGER fail_receipt").unwrap();
        assert!(call(&d,&m,&r,"user_grant",&args,end(),None).0);
        drop(m);let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();recover(&mut j,5).unwrap();assert!(j.has(8,"users").unwrap());
        let rebuilt=j.rebuild(&r,6).unwrap();
        assert!(rebuilt["receipts"].as_array().unwrap().iter().any(|e|e["kind"]=="admin" && e["result"].as_str().and_then(|s|serde_json::from_str::<Value>(s).ok()).is_some_and(|v|v["outcome"]=="accepted")),"native recovery must retain administration receipts rather than repeat effects");
        let _=std::fs::remove_dir_all(dir);
    }

    // #168 A4/A5 / §5: a claim and Codex slots do not make an ungranted project ready.
    #[cfg(unix)]
    #[test]
    fn onboarding_168_registration_is_not_readiness() {
        use std::os::unix::fs::PermissionsExt;
        let (mut j,dir,d)=setup();
        let root=d.keys.0.parent().unwrap().join("projects/cyprusbz-board");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("taskq.json"),r#"{"repo":"invented/cyprusbz-board","board":"github","host":"github.com","create_claim_issue":20}"#).unwrap();
        let bin=dir.join("fake-taskq");
        let snapshot=json!({"complete":true,"board":{"repo":"invented/cyprusbz-board","kind":"github","host":"github.com"},
            "capacity":{"limits":{"codex":4},"gate":"open"},"access":[{"runtime":"codex","grant":"none (taskq.json is a request)","refused":"no operator grant"}]});
        std::fs::write(&bin,format!("#!/bin/sh\nprintf '%s' '{}'\n",snapshot)).unwrap();
        std::fs::set_permissions(&bin,std::fs::Permissions::from_mode(0o700)).unwrap();
        let t=Taskq{bin,..tq(None)};
        let r=run(&mut j,7,1,"register invented/cyprusbz-board as cyprusbz-board");
        let m=Mutex::new(j);
        let (ok,out)=call(&d,&m,&r,"project_register",&json!({"project":"cyprusbz-board","repo":"invented/cyprusbz-board","board":"github","host":"github.com"}),end(),Some(&t));
        assert!(ok,"{out}");
        let receipt:Value=serde_json::from_str(&out).unwrap();
        assert_eq!(receipt["filing_ready"],false,"registration must await setup checks, even with a claim and codex limit 4");
        let _=std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn admin_160_project_trace() {
        use std::os::unix::fs::PermissionsExt;
        let (mut j,dir,d)=setup();let bin=dir.join("fake-taskq");
        std::fs::write(&bin,"#!/bin/sh\nprintf '%s' '{\"complete\":true,\"board\":{\"repo\":\"invented/demo\",\"kind\":\"github\",\"host\":\"github.com\"}}'\n").unwrap();std::fs::set_permissions(&bin,std::fs::Permissions::from_mode(0o700)).unwrap();
        let t=Taskq{bin,..tq(None)};
        let r=run(&mut j,7,1,"register invented/demo as demo");let m=Mutex::new(j);
        let args=json!({"project":"demo","repo":"invented/demo","board":"github","host":"github.com"});
        let (ok,out)=call(&d,&m,&r,"project_register",&args,end(),Some(&t));assert!(ok,"{out}");
        let v:Value=serde_json::from_str(&out).unwrap();assert_eq!(v["filing_ready"],false);
        assert_eq!(call(&d,&m,&r,"project_register",&args,end(),Some(&t)),(ok,out));
        assert_eq!(m.lock().unwrap().projects().unwrap().len(),1);
        assert_eq!(m.lock().unwrap().admit(&[(1,Some(input(7,1,"register invented/demo as demo")))],2).unwrap(),0,"duplicate input is never admitted twice");
        finish(&mut m.lock().unwrap(),&r);
        let r=run(&mut m.lock().unwrap(),7,2,"register invented/demo as ../escape");
        assert!(!call(&d,&m,&r,"project_register",&json!({"project":"../escape","repo":"invented/demo","board":"github","host":"github.com"}),end(),Some(&t)).0);
        finish(&mut m.lock().unwrap(),&r);
        m.lock().unwrap().grant(7,8,&["chat".into()],0).unwrap();
        let unprivileged=run(&mut m.lock().unwrap(),8,8,"register invented/demo as forbidden");
        let refused=json!({"project":"forbidden","repo":"invented/demo","board":"github","host":"github.com"});
        assert!(!call(&d,&m,&unprivileged,"project_register",&refused,end(),Some(&t)).0);
        assert!(!dir.join("projects/forbidden").exists(),"no filesystem effects without projects grant");
        finish(&mut m.lock().unwrap(),&unprivileged);
        let r=run(&mut m.lock().unwrap(),7,9,"register invented/demo as mismatch");
        std::fs::write(&t.bin,"#!/bin/sh\nprintf '%s' '{\"complete\":true,\"board\":{\"repo\":\"invented/other\"}}'\n").unwrap();
        let mismatch=json!({"project":"mismatch","repo":"invented/demo","board":"github","host":"github.com"});
        assert!(!call(&d,&m,&r,"project_register",&mismatch,end(),Some(&t)).0);
        assert_eq!(m.lock().unwrap().projects().unwrap().len(),1,"mismatched TaskQ board never registers");
        // Crash after a durable filesystem effect, before TaskQ readback: no registration or blind replay.
        finish(&mut m.lock().unwrap(),&r);
        let r=run(&mut m.lock().unwrap(),7,3,"register invented/demo as partial");
        let args=json!({"project":"partial","repo":"invented/demo","board":"github","host":"github.com"});
        start(&m.lock().unwrap(),&r,"project_register",&args).unwrap();
        config_root(&d,"partial",&json!({"repo":"invented/demo","board":"github","host":"github.com"})).unwrap();drop(m);
        let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();recover(&mut j,10).unwrap();let m=Mutex::new(j);
        assert!(!call(&d,&m,&r,"project_register",&args,end(),Some(&t)).0);
        assert_eq!(m.lock().unwrap().projects().unwrap().len(),1);drop(m);let _=std::fs::remove_dir_all(dir);
    }

    // R15/§4: real HTTP boundary with invented Bot API fixtures; one effect, correct default, no uncertain replay.
    #[test]
    fn admin_160_topic_trace() {
        for mode in ["allow","allow_shared","missing_permission","lost_response"] {
            let (mut j,dir,d)=setup();registered(&j,&dir);
            let r=run(&mut j,7,1,if mode=="allow_shared" {"open shared topic demo"} else {"open a topic for demo"});let m=Mutex::new(j);
            let mut replies=vec![Some(json!({"id":123})),Some(json!({"is_forum":true,"type":"supergroup"})),Some(json!({"status":"administrator","can_manage_topics":mode!="missing_permission"}))];
            if mode!="missing_permission" { replies.push(if mode=="lost_response" {None} else {Some(json!({"message_thread_id":42}))}); }
            let (bot,server)=fake_bot(replies);let t=tq(Some(bot));let args=json!({"project":"demo","name":"Demo","shared":mode=="allow_shared"});
            let (ok,out)=call(&d,&m,&r,"topic_open",&args,end(),Some(&t));let calls=server.join().unwrap();
            assert_eq!(ok,matches!(mode,"allow"|"allow_shared"),"{out}");
            assert_eq!(calls.iter().filter(|c|c.contains("createForumTopic")).count(),usize::from(mode!="missing_permission"));
            if matches!(mode,"allow"|"allow_shared") {
                let result:Value=serde_json::from_str(&out).unwrap();
                assert_eq!(result["session"],if mode=="allow_shared" {"shared"} else {"isolated"});
                assert_eq!(call(&d,&m,&r,"topic_open",&args,end(),Some(&t)),(ok,out));
                finish(&mut m.lock().unwrap(),&r);
                let mut message=input(7,2,"hello");message.topic=Some(42);m.lock().unwrap().admit(&[(2,Some(message))],2).unwrap();
                let next=m.lock().unwrap().claim_next(4,2).unwrap().pop().unwrap();assert_eq!(m.lock().unwrap().current_project(&next.session).unwrap().as_deref(),Some("demo"));
            } else if mode=="lost_response" {
                drop(m);let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();recover(&mut j,3).unwrap();let m=Mutex::new(j);
                assert!(!call(&d,&m,&r,"topic_open",&args,end(),Some(&t)).0,"lost create response must never repeat external effect");
                let _=std::fs::remove_dir_all(dir);continue;
            }
            drop(m);let _=std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn admin_160_topic_known_id_recovery_and_grants() {
        let (mut j,dir,d)=setup();registered(&j,&dir);
        j.grant(7,8,&["chat".into(),"project:demo".into()],0).unwrap();
        let r=run(&mut j,8,1,"open topic demo");let m=Mutex::new(j);let args=json!({"project":"demo","name":"Demo"});
        assert!(!call(&d,&m,&r,"topic_open",&args,end(),None).0,"project access without users refused before Bot API");
        finish(&mut m.lock().unwrap(),&r);
        m.lock().unwrap().grant(7,8,&["users".into()],0).unwrap();m.lock().unwrap().revoke(7,8,&["project:demo".into()],0).unwrap();
        let r=run(&mut m.lock().unwrap(),8,2,"open topic demo");assert!(!call(&d,&m,&r,"topic_open",&args,end(),None).0,"users without project access refused");finish(&mut m.lock().unwrap(),&r);
        let r=run(&mut m.lock().unwrap(),7,3,"open topic demo");start(&m.lock().unwrap(),&r,"topic_open",&args).unwrap();
        let p=m.lock().unwrap().projects().unwrap().pop().unwrap();
        let e=json!({"chat":-100,"topic":42,"project":"demo","root":p.2,"repo":p.1,"shared":false});
        m.lock().unwrap().0.execute("UPDATE admin_ops SET external=?2 WHERE request=?1",params![r.request,e.to_string()]).unwrap();drop(m);
        let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();recover(&mut j,5).unwrap();recover(&mut j,6).unwrap();
        assert!(existing(&j,r.request,"topic_open",&args).unwrap().is_some());
        assert_eq!(j.routes().unwrap().iter().filter(|r|r.topic==42).count(),1);
        let _=std::fs::remove_dir_all(dir);
    }
}
