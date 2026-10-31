//! §5 / #168: registered is not ready. All host remedies remain owner operations.
use crate::{journal::Journal, taskq::Taskq, telegram::Shared};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use std::{path::Path, time::{Duration, Instant}};

fn err(e:rusqlite::Error)->String { e.to_string() }
fn key(project:&str)->String { format!("onboarding:{project}") }
fn saved(j:&Journal,project:&str)->Result<Option<Value>,String> {
    let text:Option<String>=j.0.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[key(project)],|r|r.get(0)).optional().map_err(err)?;
    text.map(|s|serde_json::from_str(&s).map_err(|_|"Onboarding record unreadable".into())).transpose()
}
fn save(j:&Journal,project:&str,value:&Value)->Result<(),String> {
    j.0.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![key(project),value.to_string()]).map(drop).map_err(err)
}
fn binding(j:&Journal,project:&str)->Result<(String,String),String> {
    j.projects()?.into_iter().find(|p|p.0==project).map(|p|(p.1,p.2)).ok_or("Project is not registered".into())
}

/// Called in the registration transaction. Existing reports cannot authorize a rebound root.
pub fn queue(j:&Journal,project:&str)->Result<(),String> {
    let (board,root)=binding(j,project)?;
    let mut v=saved(j,project)?.filter(|v|v["board"]==board && v["root"]==root).unwrap_or(json!({"epoch":0}));
    v["board"]=json!(board);v["root"]=json!(root);v["pending"]=json!(true);v["ready"]=json!(false);v["override"]=Value::Null;v["checking_until"]=Value::Null;
    v["epoch"]=json!(v["epoch"].as_i64().unwrap_or(0)+1);
    save(j,project,&v)
}

pub fn gate(j:&Journal,project:&str)->Result<(),String> {
    // Existing installations are queued during service startup before claims are admitted.
    let Some(v)=saved(j,project)? else { return Ok(()) };
    let (board,root)=binding(j,project)?;
    let passed=v["ready"]==true && v["report"]["ready"]==true && v["report"]["gaps"].as_array().is_some_and(|g|g.is_empty());
    let overridden=v["override"]["actor"].as_i64().is_some_and(|a|Some(a)==j.owner().ok().flatten())
        && v["override"]["request"].as_i64().is_some_and(|id|id>0) && v["override"]["report"]==v["report"];
    if v["board"]==board && v["root"]==root && v["pending"]==false && (passed || overridden) { return Ok(()) }
    Err(format!("Project {project} setup is not ready. The owner must complete its onboarding steps and Recheck, or write /onboard {project} override. Nothing started."))
}

/// R11 / #236: the same localized remedy is used by cards, tools and durable replies.
pub fn refusal(j:&Journal,project:&str,language:Option<&str>)->Result<Option<String>,String> {
    if gate(j,project).is_ok() {return Ok(None);}
    let checks=saved(j,project).ok().flatten().into_iter().flat_map(|v|v["report"]["gaps"].as_array().cloned().unwrap_or_default())
        .filter_map(|g|g["check"].as_str().map(str::to_owned)).collect::<Vec<_>>().join(", ");
    let why=if checks.is_empty() {String::new()} else {format!(" ({checks})")};
    Ok(Some(if language.unwrap_or("en").starts_with("ru") {
        format!("{project}: настройка не завершена{why}. Владелец должен нажать «Проверить снова» (Recheck). Ничего не запущено.")
    } else {format!("Project {project} setup is not ready{why}. The owner must tap Recheck. Nothing started.")}))
}

fn service_gap(g:&Value)->bool {
    matches!(g["check"].as_str(),Some("service"|"admission"|"codex_grant"|"codex_access"|"codex_capacity"))
}

/// Refresh only volatile service evidence. Doctor, key and other owner findings retain authority.
/// External calls run outside the journal lock; apply fences the registration epoch and owner.
pub fn launch_gate(tq:&Taskq,j:&std::sync::Mutex<Journal>,project:&str,language:Option<&str>,end:Instant)->Result<(),String> {
    let refreshed=refresh_service(tq,j,project,end);
    let j=j.lock().unwrap();
    match refusal(&j,project,language)? {Some(text)=>Err(text),None=>refreshed}
}

fn refresh_service(tq:&Taskq,j:&std::sync::Mutex<Journal>,project:&str,end:Instant)->Result<(),String> {
    let refresh={
        let j=j.lock().unwrap();
        let v=saved(&j,project)?;
        v.filter(|v|gate(&j,project).is_err() && v["pending"]==false
            && v["report"]["gaps"].as_array().is_some_and(|g|g.iter().any(|g|matches!(g["check"].as_str(),Some("admission"|"service"))))
            && v["checking_until"].as_i64().is_none_or(|until|until<=crate::now() as i64))
            .filter(|v|v["root"].as_str()==tq.root.to_str() && binding(&j,project).is_ok_and(|(b,r)|v["board"]==b && v["root"]==r))
            .map(|mut v|{v["checking_until"]=json!(crate::now()+15);save(&j,project,&v).map(|_|v)}).transpose()?
    };
    if let Some(v)=refresh {
        let active=||{let j=j.lock().unwrap();saved(&j,project).ok().flatten().is_some_and(|now|now["epoch"]==v["epoch"] && now["root"]==v["root"] && now["board"]==v["board"])};
        let service=probe(&tq.bin,&tq.root.to_string_lossy(),&["service","status","--json"],end.min(Instant::now()+Duration::from_secs(10)),active);
        let observed=report(tq,v["board"].as_str().unwrap_or(""),&Value::Null,&Value::Null,&service,crate::now() as i64);
        let mut result=v["report"].clone();
        let gaps=result["gaps"].as_array_mut().ok_or("Onboarding report unreadable")?;
        gaps.retain(|g|!service_gap(g));
        gaps.extend(observed["gaps"].as_array().into_iter().flatten().filter(|g|service_gap(g)).cloned());
        result["ready"]=json!(gaps.is_empty());
        result["service_identity"]=observed["service_identity"].clone();
        result["service_checked_at"]=json!(crate::now());
        apply(&j.lock().unwrap(),project,&v["epoch"],&result)?;
    }
    Ok(())
}

fn fresh(v:&Value,at:i64)->bool {
    v.as_i64().is_some_and(|t|t<=at+5 && at-t<=120)
}
fn gap(items:&mut Vec<Value>,name:&str,fix:impl Into<String>) {
    items.push(json!({"check":name,"fix":fix.into(),"owner_only":true}));
}
// Shell quoting is for displayed owner commands only. Execution always uses argument arrays.
fn quote(s:&str)->String { format!("'{}'",s.replace('\'',"'\\''")) }
fn tq_command(tq:&Taskq,args:&str)->String { format!("TASKQ_ROOT={} {} {args}",quote(&tq.root.to_string_lossy()),quote(&tq.bin.to_string_lossy())) }

/// The fallback has no setup schema yet: use a matched status snapshot plus the service/access projection.
fn report(tq:&Taskq,board:&str,doctor:&Value,status:&Value,service:&Value,at:i64)->Value {
    let mut gaps=vec![];
    for (check,step) in [
        ("configuration","Owner: complete protected PilotQ config.json, then run pilotq doctor --json"),
        ("telegram","Owner: verify the protected bot token and Telegram connectivity; run pilotq doctor --json"),
        ("codex","Owner: restore the isolated Codex ChatGPT login configured for PilotQ; run pilotq doctor --json"),
        ("taskq","Owner: verify the configured TaskQ binary, root and board; run pilotq doctor --json"),
        ("voice","Owner: configure voice.ffmpeg, voice.recognizer and GigaAM model files using docs/tester-guide.md; run pilotq doctor --json"),
    ] {
        if doctor["checks"][check]!="ok" { gap(&mut gaps,check,step); }
    }
    if doctor["checks"]["host_tools"]!="ok" {
        // Fixes are from PilotQ's fixed requirements catalogue, never external diagnostic text.
        let mut found=false;
        for item in doctor["requirements"].as_array().into_iter().flatten().filter(|i|i["outcome"]!="ok") {
            if let (Some(tool),Some(fix))=(item["tool"].as_str(),item["fix"].as_str()) {
                gap(&mut gaps,tool,format!("Owner: {fix}"));found=true;
            }
        }
        if !found {gap(&mut gaps,"host_tools","Owner: run pilotq doctor --json and resolve its host-tool requirements");}
    }
    if doctor["ok"]!=true && gaps.is_empty() { gap(&mut gaps,"doctor","Owner: run pilotq doctor --json and resolve its failed setup checks"); }
    if status["complete"]!=true || status["board"]["repo"]!=board {
        gap(&mut gaps,"board",format!("Owner: verify taskq.json names {board}, then run {}",tq_command(tq,"status --json")));
    }
    let entry=service["projects"].as_array().and_then(|p|p.iter().find(|p|p["root"].as_str()==tq.root.to_str()));
    let snapshot=entry.map(|p|&p["status"]).unwrap_or(&Value::Null);
    if service["schema"]!="taskq.service/1" || service["service"]["state"]!="running" || service["service"]["fresh"]!=true
        || !fresh(&service["service"]["at"],at) || snapshot["complete"]!=true || !fresh(&snapshot["at"],at)
        || entry.is_some_and(|p|!p["error"].is_null()) {
        gap(&mut gaps,"service",format!("Owner: inspect {}; restore the TaskQ service for this registered root, then Recheck",tq_command(tq,"service status --json")));
    }
    let config:Value=std::fs::read(tq.root.join("taskq.json")).ok().and_then(|s|serde_json::from_slice(&s).ok()).unwrap_or(Value::Null);
    if config["repo"]!=board || !config["create_claim_issue"].as_i64().is_some_and(|n|n>0) {
        gap(&mut gaps,"claim","Owner: provision create_claim_issue for this board through TaskQ, then Recheck");
    }
    let access=snapshot["access"].as_array().and_then(|a|a.iter().find(|a|a["runtime"]=="codex"));
    let granted=access.is_some_and(|a|a["grant"].as_str().is_some_and(|g|!g.trim().is_empty() && !g.starts_with("none")));
    let usable=access.is_some_and(|a|matches!(a["effective"].as_str(),Some("restricted"|"workspace"|"full")) && a["refused"].is_null());
    if !granted {
        gap(&mut gaps,"codex_grant",format!("Owner: review project-scoped Workspace access, then run {} and Recheck (no automatic grant)",tq_command(tq,"service permissions --runtime codex --mode workspace --scope project")));
    }
    if granted && !usable {
        gap(&mut gaps,"codex_access",format!("Owner: inspect {} and resolve the refused or unsupported runtime containment through TaskQ setup, then Recheck",tq_command(tq,"service permissions")));
    }
    let capacity=&snapshot["capacity"];
    // TaskQ defines an omitted runtime limit in a known limits object as no ceiling (§3).
    let limits=capacity["limits"].as_object();
    if limits.is_none() || limits.is_some_and(|l|l.get("codex").is_some_and(|n|n.as_u64().is_none_or(|n|n==0))) {
        gap(&mut gaps,"codex_capacity","Owner: configure a positive Codex runtime limit in TaskQ host policy, then Recheck");
    }
    if capacity["gate"]!="open" {
        gap(&mut gaps,"admission",format!("Owner: inspect {} and resolve the reported admission hold, then Recheck",tq_command(tq,"service status --json")));
    }
    json!({"ready":gaps.is_empty(),"gaps":gaps,"checked_at":at,"source":"taskq service status --json (setup-check fallback)","service_identity":if service["schema"]=="taskq.service/1" {json!([service["service"]["instance"],service["service"]["started"],service["service"]["version"]])} else {Value::Null}})
}

/// Inspect grants through the metadata-only key API: no take(), credential injection or last-use write.
fn issue_key_step(keys:&crate::keys::Keys,project:&str,owner:i64)->Option<String> {
    let entries=match keys.list() {
        Ok(v)=>v,
        Err(_)=>return Some("Owner: repair the protected keys.json permissions/content, then run pilotq keys list and Recheck".into()),
    };
    let candidates:Vec<_>=entries.as_array().into_iter().flatten().filter(|k|
        k["tools"].as_array().is_some_and(|t|t.contains(&json!("issue_read")))
        && k["name"].as_str().is_some_and(|n|!n.is_empty() && n.len()<=32 && n.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_".contains(&c)))).collect();
    let holds=|k:&Value,field:&str,value:Value|k[field].as_array().is_some_and(|a|a.contains(&value));
    let allows=|k:&Value,route:&str|holds(k,"projects",json!(project)) || holds(k,"users",json!(owner)) || holds(k,"routes",json!(route));
    if candidates.iter().any(|k|allows(k,"cli")) && candidates.iter().any(|k|allows(k,&owner.to_string())) {return None;}
    if candidates.is_empty() {return Some(format!("Owner: provision an approved issue_read key through pilotq keys add (value via protected stdin), then grant it to project:{project}; pilotq keys list shows key names"));}
    let commands=candidates.iter().filter_map(|k|k["name"].as_str()).map(|name|format!("pilotq keys grant {name} project:{project}")).collect::<Vec<_>>().join(" OR ");
    Some(format!("Owner: choose the approved issue_read key and run {commands}, then Recheck"))
}

fn probe(bin:&Path,root:&str,args:&[&str],end:Instant,active:impl Fn()->bool)->Value {
    let o=crate::commands::execute_bounded(crate::commands::command(bin,root,args),end,active,512*1024);
    if o["status"]!="success" {return Value::Null;}
    serde_json::from_str(o["stdout"].as_str().unwrap_or("")).unwrap_or(Value::Null)
}

/// Checks never hold the journal lock or run a model, poll updates or mutate host setup.
pub fn check(t:&coreq::Tool,j:&Shared,project:&str)->Result<Value,String> {
    let config=crate::config(t)?.ok_or("PilotQ configuration unavailable")?;
    let (board,root,epoch)={
        let j=j.lock().unwrap();
        if !j.has(config.owner,"chat")? {return Err("Owner access revoked".into());}
        let (board,root)=binding(&j,project)?;
        let tx=j.0.unchecked_transaction().map_err(err)?;
        let mut v=saved(&j,project)?.ok_or("Onboarding not queued")?;
        let now=crate::now() as i64;
        if v["checking_until"].as_i64().is_some_and(|until|until>now) {return Err("Setup checks already running; wait for the card update".into());}
        v["checking_until"]=json!(now+65);save(&j,project,&v)?;
        tx.commit().map_err(err)?;
        (board,root,v["epoch"].clone())
    };
    let active=|| {let j=j.lock().unwrap(); j.has(config.owner,"chat").unwrap_or(false) && binding(&j,project).is_ok_and(|b|b==(board.clone(),root.clone()))
        && saved(&j,project).ok().flatten().is_some_and(|v|v["epoch"]==epoch)};
    let end=Instant::now()+Duration::from_secs(60);
    let exe=std::env::current_exe().map_err(|_|"PilotQ binary unavailable")?;
    // Doctor must use the same install and service PATH. Only its JSON crosses this boundary.
    let mut cmd=std::process::Command::new(&exe);
    cmd.args(["doctor","--json"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    #[cfg(unix)] std::os::unix::process::CommandExt::process_group(&mut cmd,0);
    let output=crate::commands::execute_bounded(cmd,end.min(Instant::now()+Duration::from_secs(40)),&active,128*1024);
    let doctor=if matches!(output["status"].as_str(),Some("success"|"exit_nonzero")) {serde_json::from_str(output["stdout"].as_str().unwrap_or("")).unwrap_or(Value::Null)} else {Value::Null};
    let fallback=Taskq{bot:None,bin:config.taskq.as_ref().map(|q|q.bin.clone()).unwrap_or_default(),root:root.clone().into(),poll:Duration::from_secs(15)};
    let status=probe(&fallback.bin,&root,&["status","--json"],end.min(Instant::now()+Duration::from_secs(10)),&active);
    let service=probe(&fallback.bin,&root,&["service","status","--json"],end.min(Instant::now()+Duration::from_secs(10)),&active);
    let mut result=report(&fallback,&board,&doctor,&status,&service,crate::now() as i64);
    if let Some(step)=issue_key_step(&config.direct.keys,project,config.owner) {
        result["gaps"].as_array_mut().unwrap().push(json!({"check":"issue_read_key","fix":step,"owner_only":true}));
        result["ready"]=json!(false);
    }
    result["issue_read_key_checked"]=json!(true);
    if !active() {return Err("Onboarding changed or owner access was revoked; Recheck required".into());}
    let j=j.lock().unwrap();
    apply(&j,project,&epoch,&result)?;
    Ok(result)
}

pub fn summary(project:&str,result:&Value)->String { text(project,result,false) }

fn text(project:&str,result:&Value,ru:bool)->String {
    let title=if result["ready"]==true {if ru {format!("{project}: настройки проверены, проект готов к первой задаче.")} else {format!("{project}: setup passed; ready for the first task.")}}
        else if ru {format!("{project}: проект пока не готов. Шаги для владельца:")} else {format!("{project}: not ready. Owner steps:")};
    let mut lines=vec![title];
    for g in result["gaps"].as_array().into_iter().flatten() {lines.push(format!("{}: {}",g["check"].as_str().unwrap_or("setup"),g["fix"].as_str().unwrap_or("Owner: run pilotq doctor --json")));}
    if result["ready"]!=true {lines.push(if ru {"После исправления нажмите «Проверить снова»."} else {"After the owner steps, tap Recheck."}.into());}
    lines.join("\n")
}

fn apply(j:&Journal,project:&str,epoch:&Value,result:&Value)->Result<(),String> {
    let tx=j.0.unchecked_transaction().map_err(err)?;
    let mut v=saved(j,project)?.ok_or("Onboarding not queued")?;
    if &v["epoch"]!=epoch {return Err("Onboarding changed; Recheck required".into());}
    let (board,root)=binding(j,project)?;
    if v["board"]!=board || v["root"]!=root {return Err("Registration changed; Recheck required".into());}
    let owner=j.owner()?.ok_or("Owner unavailable")?;
    if !j.has(owner,"chat")? {return Err("Owner access revoked".into());}
    v["service_identity"]=result["service_identity"].clone();v["ready"]=result["ready"].clone();v["report"]=result.clone();v["pending"]=json!(false);v["checking_until"]=Value::Null;
    let at=crate::now() as i64;
    let session=format!("dm:{owner}");
    let created=v["card_request"].is_null();
    let source=if let Some(id)=v["card_request"].as_i64() {id} else {
        let update:i64=j.0.query_row("SELECT min(coalesce(min(update_id),0),0)-1 FROM requests",[],|r|r.get(0)).map_err(err)?;
        let id=crate::journal::insert(&j.0,update,"service",owner,None,owner,None,0,"onboarding",Some(&format!("Onboard {project}")),"answered",at,&session,None)?;
        v["card_request"]=json!(id);id
    };
    let generation:i64=j.0.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&session],|r|r.get(0)).map_err(err)?;
    v["generation"]=json!(generation);
    let language:Option<String>=j.0.query_row("SELECT language FROM requests WHERE sender=?1 AND language IS NOT NULL ORDER BY id DESC LIMIT 1",[owner],|r|r.get(0)).optional().map_err(err)?;
    let message=text(project,result,language.as_deref().unwrap_or("en").starts_with("ru"));
    j.0.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND kind='onboarding' AND state='pending'",[source]).map_err(err)?;
    crate::journal::outbox(&j.0,source,owner,None,None,Some(&crate::telegram::html(&message)),&message,at)?;
    j.0.execute("UPDATE outbox SET kind='onboarding',project=?2,revision=?3 WHERE id=?1",params![j.0.last_insert_rowid(),project,epoch.as_i64()]).map_err(err)?;
    j.0.execute("UPDATE requests SET reply=?2 WHERE id=?1",params![source,message]).map_err(err)?;
    j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'onboarding',?3,?4)",params![source,at,project,result.to_string()]).map_err(err)?;
    save(j,project,&v)?;tx.commit().map_err(err)?;
    if created {j.trace(source,at);}
    Ok(())
}

pub fn buttons(j:&Journal,delivery:i64)->Result<Vec<(String,String)>,String> {
    let row:Option<(String,i64,i64)>=j.0.query_row("SELECT project,request_id,revision FROM outbox WHERE id=?1 AND kind='onboarding'",[delivery],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(err)?;
    let Some((project,source,epoch))=row else {return Ok(vec![])};
    let v=saved(j,&project)?.ok_or("Onboarding unavailable")?;
    if v["ready"]==true || v["epoch"]!=epoch || v["pending"]==true {return Ok(vec![])}
    Ok(vec![(format!("onboard:{source}:{epoch}"),"Recheck".into())])
}

pub fn control(t:&coreq::Tool,j:&Shared,r:&crate::builtins::Request)->Result<(),String> {
    let mut args=r.text.split_whitespace();let cmd=args.next().unwrap_or("").split('@').next().unwrap_or("");
    let value=args.next().ok_or("Use /onboard NAME or /onboard NAME override")?;
    let action=args.next();if args.next().is_some() {return Err("Use /onboard NAME or /onboard NAME override".into());}
    let project={
        let j=j.lock().unwrap();
        if j.owner()?!=Some(r.sender) || !j.builtin_allowed(r)? {return Err("Only the configured owner can manage onboarding".into());}
        if cmd=="/onboard-button" {
            let (source,epoch)=value.split_once(':').ok_or("Invalid onboarding button")?;
            let source=source.parse::<i64>().map_err(|_|"Invalid onboarding button")?;
            let epoch=epoch.parse::<i64>().map_err(|_|"Invalid onboarding button")?;
            let project:Option<String>=j.0.query_row("SELECT o.project FROM outbox o JOIN requests q ON q.id=o.request_id JOIN requests press ON press.id=?2 WHERE o.request_id=?1 AND o.kind='onboarding' AND o.state='delivered' AND o.sent_message=press.message AND o.chat=press.chat AND o.topic IS press.topic AND q.sender=press.sender",params![source,r.id],|x|x.get(0)).optional().map_err(err)?;
            let project=project.ok_or("This onboarding button is stale")?;
            let v=saved(&j,&project)?.ok_or("This onboarding button is stale")?;
            let generation:i64=j.0.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&r.session],|x|x.get(0)).map_err(err)?;
            let current=binding(&j,&project)?;
            if action.is_some() || v["epoch"]!=epoch || v["generation"]!=generation || v["pending"]==true || v["board"]!=current.0 || v["root"]!=current.1 {return Err("This onboarding button is stale".into());}
            project
        } else {value.to_string()}
    };
    {
        let j=j.lock().unwrap();
        let tx=j.0.unchecked_transaction().map_err(err)?;
        if j.owner()?!=Some(r.sender) || !j.builtin_allowed(r)? {return Err("Only the configured owner can manage onboarding".into());}
        let (board,root)=binding(&j,&project)?;
        if action==Some("override") && cmd=="/onboard" {
            let mut v=saved(&j,&project)?.ok_or("Recheck this project before overriding")?;
            if v["board"]!=board || v["root"]!=root || v["pending"]!=false {return Err("Recheck this project before overriding".into());}
            v["override"]=json!({"actor":r.sender,"request":r.id,"at":crate::now(),"report":v["report"]});
            save(&j,&project,&v)?;
            j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'onboarding_override',?3,?4)",params![r.id,crate::now() as i64,project,v["override"].to_string()]).map_err(err)?;
        } else {
            if action.is_some() {return Err("Use /onboard NAME or /onboard NAME override".into());}
            queue(&j,&project)?;
        }
        // A valid callback gets its toast before potentially slow dependency checks.
        let callback:Option<Vec<u8>>=j.0.query_row("SELECT value FROM meta WHERE key=?1",[format!("callback:{}",r.id)],|x|x.get(0)).optional().map_err(err)?;
        if let Some(callback)=callback {
            let callback=String::from_utf8(callback).map_err(|_|"Invalid callback")?;
            crate::journal::outbox(&j.0,r.id,r.sender,None,None,None,"Recheck received.",crate::now() as i64)?;
            j.0.execute("UPDATE outbox SET kind=?2 WHERE id=?1",params![j.0.last_insert_rowid(),format!("toast:{callback}")]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
    }
    let message=if action==Some("override") {format!("{project}: readiness override recorded; gaps remain. TaskQ admission and grants still apply.")}
        else {let result=check(t,j,&project)?;text(&project,&result,false)};
    j.lock().unwrap().builtin_finish(r.id,&message,true,crate::now() as i64)
}

/// Startup queues old registrations before new manager claims; no root discovery.
pub fn startup(j:&Journal)->Result<(),String> {
    for (project,..) in j.projects()? { if saved(j,&project)?.is_none_or(|v|v["report"]["issue_read_key_checked"]!=true) {queue(j,&project)?;} }
    Ok(())
}

/// Scan just registered roots. Service generation changes detect installs/restarts; diagnostics never enter Telegram.
pub fn serve(t:coreq::Tool,j:Shared) {
    loop {
        if let Err(e)=scan(&t,&j) {eprintln!("onboarding: {e}");}
        std::thread::sleep(Duration::from_secs(60));
    }
}
fn scan(t:&coreq::Tool,j:&Shared)->Result<(),String> {
    let Some(config)=crate::config(t)? else {return Ok(())};
    let Some(tq)=config.taskq else {return Ok(())};
    let projects=j.lock().unwrap().projects()?;
    // One host service observation supplies the install generation for every registered project.
    let service=probe(&tq.bin,tq.root.to_str().ok_or("Invalid TaskQ root")?,&["service","status","--json"],Instant::now()+Duration::from_secs(10),||true);
    let identity=if service["schema"]=="taskq.service/1" {json!([service["service"]["instance"],service["service"]["started"],service["service"]["version"]])} else {Value::Null};
    for (project,..) in projects {
        let pending={
            let j=j.lock().unwrap();
            if !j.has(config.owner,"chat")? {return Ok(())}
            let mut v=saved(&j,&project)?.unwrap_or(Value::Null);
            if !identity.is_null() && v["service_identity"]!=identity {queue(&j,&project)?;v=saved(&j,&project)?.unwrap();v["service_identity"]=identity.clone();save(&j,&project,&v)?;}
            v["pending"]==true
        };
        if pending {let _=check(t,j,&project);}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::{Input,Sent};
    fn setup(tag:&str)->(Journal,std::path::PathBuf,Taskq) {
        let dir=std::env::temp_dir().join(format!("pilotq-onboarding-{tag}-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&dir);std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"invented/cyprusbz-board","create_claim_issue":20}"#).unwrap();
        let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();j.bootstrap(7,1).unwrap();
        j.add_project(7,"cyprusbz-board","invented/cyprusbz-board",dir.to_str().unwrap(),1).unwrap();
        let tq=Taskq{bot:None,bin:"/invented/taskq".into(),root:dir.clone(),poll:Duration::from_secs(15)};
        (j,dir,tq)
    }
    fn doctor()->Value {json!({"ok":true,"checks":{"configuration":"ok","telegram":"ok","codex":"ok","taskq":"ok","voice":"ok","host_tools":"ok"}})}
    fn service(tq:&Taskq)->Value {
        json!({"schema":"taskq.service/1","service":{"state":"running","version":"4.6.40","fresh":true,"at":1000},
            "projects":[{"root":tq.root,"status":{"complete":true,"at":1000,"capacity":{"limits":{"codex":4},"gate":"open"},
            "access":[{"runtime":"codex","grant":"workspace (project) rev 3 by invented","effective":"workspace","refused":null}]}}]})
    }
    fn status()->Value {json!({"complete":true,"board":{"repo":"invented/cyprusbz-board"}})}
    // A5 / §5: pds18's reported limit 4 must never substitute for an operator grant.
    // Retains protection for incomplete/stale dependencies and project/board isolation.
    #[test]
    fn onboarding_168_pds18_missing_grant_and_unknown_checks() {
        let (_,dir,tq)=setup("pds18");
        let good=report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&service(&tq),1000);
        assert_eq!(good["ready"],true,"{good}");
        let mut unlimited=service(&tq);unlimited["projects"][0]["status"]["capacity"]["limits"]=json!({});
        assert_eq!(report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&unlimited,1000)["ready"],true,"TaskQ's omitted runtime limit means no ceiling");
        let mut refused=service(&tq);refused["projects"][0]["status"]["access"][0]["refused"]=json!("containment unavailable");
        let refused=report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&refused,1000);
        assert!(refused["gaps"].as_array().unwrap().iter().any(|g|g["check"]=="codex_access"));
        let mut no_grant=service(&tq);
        no_grant["projects"][0]["status"]["access"][0]=json!({"runtime":"codex","grant":"none (taskq.json is a request)","effective":"legacy","refused":"no operator grant"});
        let bad=report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&no_grant,1000);
        assert_eq!(bad["ready"],false);
        let line=summary("cyprusbz-board",&bad);
        assert!(line.contains("codex_grant: Owner:") && line.contains("service permissions --runtime codex --mode workspace --scope project"),"{line}");
        assert!(!line.contains("setup passed"));
        for (doc,board,svc,at) in [(Value::Null,status(),service(&tq),1000),(doctor(),Value::Null,service(&tq),1000),
            (doctor(),status(),Value::Null,1000),(doctor(),status(),service(&tq),1201),
            (doctor(),json!({"complete":true,"board":{"repo":"invented/other"}}),service(&tq),1000)] {
            assert_eq!(report(&tq,"invented/cyprusbz-board",&doc,&board,&svc,at)["ready"],false);
        }
        let _=std::fs::remove_dir_all(dir);
    }
    // #168 continuation / §8: project-only issue key grants do not cover other registered projects.
    #[test]
    fn onboarding_168_issue_key_project_grants() {
        let (_,dir,_)=setup("issue-key");let keys=crate::keys::Keys(dir.join("keys.json"));
        assert!(issue_key_step(&keys,"cyprusbz",7).unwrap().contains("provision an approved issue_read key"));
        // Invented credential; metadata inspection must never return or consume it.
        keys.add("pilotq-github","invented",&["issue_read".into()],"invented-issue-key-value").unwrap();
        keys.grant("pilotq-github",&["project:pilotq".into()]).unwrap();
        let before=keys.list().unwrap();
        for project in ["cyprusbz","tight","agentq"] {
            let fix=issue_key_step(&keys,project,7).unwrap();
            assert!(fix.contains(&format!("pilotq keys grant pilotq-github project:{project}")),"{fix}");
            assert!(!fix.contains("invented-issue-key-value"));
        }
        assert!(issue_key_step(&keys,"pilotq",7).is_none());
        assert_eq!(keys.list().unwrap(),before,"setup never changes grants or last_use");
        keys.grant("pilotq-github",&["project:cyprusbz".into()]).unwrap();
        assert!(issue_key_step(&keys,"cyprusbz",7).is_none());
        assert!(issue_key_step(&keys,"tight",7).is_some());
        keys.grant("pilotq-github",&["7".into()]).unwrap();
        assert!(issue_key_step(&keys,"tight",7).is_none(),"existing user-wide grants retain §8 semantics");
        keys.revoke("pilotq-github",&["7".into()]).unwrap();
        keys.grant("pilotq-github",&["route:cli".into()]).unwrap();
        assert!(issue_key_step(&keys,"tight",7).is_some(),"CLI-only access does not qualify the owner DM");
        keys.grant("pilotq-github",&["route:7".into()]).unwrap();
        assert!(issue_key_step(&keys,"tight",7).is_none());
        std::fs::write(&keys.0,"malformed invented-issue-key-value").unwrap();
        let unknown=issue_key_step(&keys,"tight",7).unwrap();
        assert!(unknown.contains("repair the protected keys.json") && !unknown.contains("invented-issue-key-value"));
        let _=std::fs::remove_dir_all(dir);
    }

    fn press(message:i64,sender:i64,source:i64,epoch:i64)->Input {
        Input{files:vec![],callback:Some(format!("invented-{sender}-{message}")),media_group:None,quote:None,image:None,voice:None,language:None,chat:7,topic:None,sender:Some(sender),name:Some("Invented".into()),message,kind:"text",text:Some(format!("/onboard-button {source}:{epoch}")),reply_to:Some(message)}
    }
    // A3/A4 / R10/R13/R17: one durable card, gate across restart, owner-only written override,
    // stale/foreign callbacks, rebound registration and successful recheck remove buttons.
    #[test]
    fn onboarding_168_card_gate_and_recovery() {
        let (mut j,dir,tq)=setup("card");queue(&j,"cyprusbz-board").unwrap();
        assert!(gate(&j,"cyprusbz-board").is_err());
        let draft=j.ask(7,"plan cyprusbz-board",1).unwrap();
        j.save_plan(&draft,&json!([{"title":"invented first task"}]),"cyprusbz-board","invented/cyprusbz-board",dir.to_str().unwrap(),1).unwrap();
        let request=j.ask(7,"launch",2).unwrap();
        let run_id=j.authorize(&request,1,7,120,2).unwrap();
        let shared=std::sync::Mutex::new(j);
        let refused=crate::taskq::submit(&tq,&shared,run_id,false);
        assert!(refused.contains("setup is not ready"),"{refused}");
        let mut j=shared.into_inner().unwrap();
        assert!(j.plan_run(run_id).unwrap().tickets.iter().all(|t|t.task.is_none() && !t.started),"setup gate must run before any TaskQ add/start");
        let epoch=saved(&j,"cyprusbz-board").unwrap().unwrap()["epoch"].clone();
        let report=report(&tq,"invented/cyprusbz-board",&Value::Null,&status(),&service(&tq),1000);
        apply(&j,"cyprusbz-board",&epoch,&report).unwrap();
        let d=j.due(crate::now() as i64).unwrap().0.unwrap();assert_eq!(d.kind,"onboarding");
        assert_eq!(d.chat,7);assert_eq!(d.buttons.len(),1);assert!(d.text.contains("not ready"));
        j.record(d.id,&Sent::Ok(55),crate::now() as i64).unwrap();
        let source=saved(&j,"cyprusbz-board").unwrap().unwrap()["card_request"].as_i64().unwrap();
        j.grant(7,8,&["chat".into()],1).unwrap();
        let j=std::sync::Arc::new(std::sync::Mutex::new(j));
        for (id,sender) in [(1,8),(2,7)] {
            let mut locked=j.lock().unwrap();
            locked.admit(&[(id,Some(press(55,sender,source,0)))],2).unwrap();
            let r=locked.builtin_next().unwrap().unwrap();drop(locked);
            assert!(control(&crate::tool(),&j,&r).is_err());
            j.lock().unwrap().builtin_finish(r.id,"This onboarding button is stale",false,3).unwrap();
        }
        // Unavailable checks persist across restart and cannot start a task.
        drop(j);let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();
        assert!(gate(&j,"cyprusbz-board").is_err());
        let id=j.builtin_cli(7,"/onboard cyprusbz-board override",4).unwrap().unwrap();
        let r=j.builtin_take(id).unwrap().unwrap();let j=std::sync::Arc::new(std::sync::Mutex::new(j));
        control(&crate::tool(),&j,&r).unwrap();
        {let locked=j.lock().unwrap();assert!(gate(&locked,"cyprusbz-board").is_ok());assert_eq!(saved(&locked,"cyprusbz-board").unwrap().unwrap()["ready"],false);}
        let locked=j.lock().unwrap();queue(&locked,"cyprusbz-board").unwrap();
        let epoch2=saved(&locked,"cyprusbz-board").unwrap().unwrap()["epoch"].clone();
        assert!(gate(&locked,"cyprusbz-board").is_err());
        assert!(apply(&locked,"cyprusbz-board",&epoch,&json!({"ready":true,"gaps":[]})).is_err());
        apply(&locked,"cyprusbz-board",&epoch2,&json!({"ready":true,"gaps":[]})).unwrap();
        assert!(gate(&locked,"cyprusbz-board").is_ok());
        let outbox:i64=locked.0.query_row("SELECT max(id) FROM outbox WHERE kind='onboarding'",[],|r|r.get(0)).unwrap();
        assert!(buttons(&locked,outbox).unwrap().is_empty());
        let toasts:i64=locked.0.query_row("SELECT count(*) FROM outbox WHERE kind LIKE 'toast:%' AND text='This onboarding button is stale'",[],|r|r.get(0)).unwrap();
        assert_eq!(toasts,2); // stale taps are toasts, never extra gap lists
        locked.add_project(7,"cyprusbz-board","invented/other",dir.to_str().unwrap(),5).unwrap();
        assert!(gate(&locked,"cyprusbz-board").is_err());
        drop(locked);drop(j);let _=std::fs::remove_dir_all(dir);
    }
    // #236 A1/A3 / R11, §5: closed setup must hide launch and survive the final reply boundary.
    #[test]
    fn onboarding_236_refusal_and_plan_card() {
        for (lang,model_reply) in [("en",false),("ru",false),("en",true),("ru",true)] {
            let (mut j,dir,_)=setup(&format!("236-{lang}-{model_reply}"));
            queue(&j,"cyprusbz-board").unwrap();
            let mut input=press(1,7,0,0);input.callback=None;input.reply_to=None;input.text=Some("plan cyprusbz-board".into());
            j.admit(&[(1,Some(input))],1).unwrap();
            let r=j.claim_next(4,1).unwrap().remove(0);
            j.0.execute("UPDATE requests SET language=?2 WHERE id=?1",params![r.request,lang]).unwrap();
            j.save_plan(&r,&json!([{"title":"Invented task"}]),"cyprusbz-board","invented/cyprusbz-board",dir.to_str().unwrap(),1).unwrap();
            j.finish(&r,None,None,&crate::manager::Outcome::Answered("Prepared".into()),2).unwrap();
            let delivery=j.0.query_row("SELECT id FROM outbox WHERE request_id=?1 AND kind='rich'",[r.request],|r|r.get(0)).unwrap();
            assert!(!j.card_buttons(delivery).unwrap().iter().any(|(id,_)|id.starts_with("plan:")),"closed setup offered Run plan");
            let reply=j.request(r.request).unwrap().1.unwrap();
            assert!(reply.contains("cyprusbz-board") && reply.contains(if lang=="ru" {"Проверить снова"} else {"Recheck"}),"{reply}");
            let mut input=press(2,7,0,0);input.callback=None;input.reply_to=None;input.text=Some("run plan".into());
            j.admit(&[(2,Some(input))],3).unwrap();
            let launch=j.claim_next(4,3).unwrap().remove(0);
            j.0.execute("UPDATE requests SET language=?2 WHERE id=?1",params![launch.request,lang]).unwrap();
            let shared=std::sync::Mutex::new(j);
            let tq=Taskq{bot:None,bin:"/invented/taskq".into(),root:dir.clone(),poll:Duration::from_secs(15)};
            let (ok,refusal)=crate::taskq::call(&tq,&shared,&launch,"task_run_plan",&json!({"revision":1}));
            assert!(!ok && refusal.contains("cyprusbz-board") && refusal.contains(if lang=="ru" {"Проверить снова"} else {"Recheck"}),"{refusal}");
            let j=shared.into_inner().unwrap();
            let outcome=if model_reply {crate::manager::Outcome::Answered("The request failed; try again or check /status.".into())} else {crate::manager::Outcome::Failed(refusal.clone())};
            j.finish(&launch,None,None,&outcome,4).unwrap();
            assert_eq!(j.request(launch.request).unwrap().1.unwrap(),refusal);
            let delivery=j.0.query_row("SELECT id FROM outbox WHERE request_id=?1 AND kind='rich'",[launch.request],|r|r.get::<_,i64>(0)).unwrap();
            assert!(delivery>0,"refusal must reach the chat, not only a callback toast");
            drop(j);let _=std::fs::remove_dir_all(dir);
        }
    }

    // #236 A2 / §5: a saved hold must recover without owner intervention; unrelated gaps,
    // stale evidence, changed roots and revoked grants must still block launch.
    #[cfg(unix)]
    #[test]
    fn onboarding_236_refresh_saved_service_gaps() {
        use std::os::unix::fs::PermissionsExt;
        let (j,dir,mut tq)=setup("236-refresh");queue(&j,"cyprusbz-board").unwrap();
        let epoch=saved(&j,"cyprusbz-board").unwrap().unwrap()["epoch"].clone();
        tq.bin=dir.join("taskq-fixture");
        std::fs::write(&tq.bin,"#!/bin/sh\ncat \"$TASKQ_ROOT/service.json\"\n").unwrap();
        std::fs::set_permissions(&tq.bin,std::fs::Permissions::from_mode(0o700)).unwrap();
        let shared=std::sync::Mutex::new(j);
        for old in ["admission","service"] {
            for outcome in ["open","closed","stale","unavailable","other-gap","revoked","rebound"] {
                let mut svc=service(&tq);
                let at=crate::now() as i64;
                svc["service"]["at"]=json!(at);svc["projects"][0]["status"]["at"]=json!(at);
                match outcome {
                    "closed"=>svc["projects"][0]["status"]["capacity"]["gate"]=json!("closed"),
                    "stale"=>svc["service"]["at"]=json!(at-121),
                    "unavailable"=>svc=Value::Null,
                    "revoked"=>svc["projects"][0]["status"]["access"]=json!([]),
                    "rebound"=>svc["projects"][0]["root"]=json!("/invented/other"),
                    _=>{},
                }
                std::fs::write(dir.join("service.json"),svc.to_string()).unwrap();
                let mut gaps=vec![json!({"check":old,"fix":"Owner Recheck"})];
                if outcome=="other-gap" {gaps.push(json!({"check":"issue_read_key","fix":"Owner provisions key"}));}
                apply(&shared.lock().unwrap(),"cyprusbz-board",&epoch,&json!({"ready":false,"gaps":gaps})).unwrap();
                let result=launch_gate(&tq,&shared,"cyprusbz-board",Some("en"),Instant::now()+Duration::from_secs(2));
                assert_eq!(result.is_ok(),outcome=="open","{old}/{outcome}: {result:?}");
                let j=shared.lock().unwrap();
                let v=saved(&j,"cyprusbz-board").unwrap().unwrap();
                if outcome=="other-gap" {assert!(v["report"]["gaps"].as_array().unwrap().iter().any(|g|g["check"]=="issue_read_key"));}
            }
        }
        let _=std::fs::remove_dir_all(dir);
    }

}
