//! §5: registered is not ready. Attach only to an existing healthy host; other host remedies stay owner operations.
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
    let origin:Option<i64>=j.0.query_row("SELECT request FROM admin_ops WHERE tool='project_register' AND json_extract(args,'$.project')=?1 ORDER BY request DESC LIMIT 1",[project],|r|r.get(0)).optional().map_err(err)?;
    if v["attach_request"].is_null() || v["registration_request"]!=json!(origin) {v["attach_request"]=json!(origin);}
    v["registration_request"]=json!(origin);
    save(j,project,&v)
}

fn card_request(j:&Journal,v:&mut Value,project:&str,owner:i64)->Result<i64,String> {
    if let Some(id)=v["card_request"].as_i64() {return Ok(id);}
    let update:i64=j.0.query_row("SELECT min(coalesce(min(update_id),0),0)-1 FROM requests",[],|r|r.get(0)).map_err(err)?;
    let id=crate::journal::insert(&j.0,update,"service",owner,None,owner,None,0,"onboarding",Some(&format!("Onboard {project}")),"answered",crate::now() as i64,&format!("dm:{owner}"),None)?;
    v["card_request"]=json!(id);Ok(id)
}

fn attach_authorized(j:&Journal,v:&Value,owner:i64)->Result<bool,String> {
    let Some(request)=v["attach_request"].as_i64() else {return Ok(j.has(owner,"chat")? && j.has(owner,"projects")?)};
    let (actor,state):(i64,String)=j.0.query_row("SELECT sender,state FROM requests WHERE id=?1",[request],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)?;
    let fenced:bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM runs r JOIN conversations c ON c.session=r.session WHERE r.request_id=?1 AND r.state='running' AND r.fence<>c.fence)",[request],|r|r.get(0)).map_err(err)?;
    Ok(!fenced && !matches!(state.as_str(),"cancelled"|"failed"|"uncertain") && j.has(actor,"chat")? && j.has(actor,"projects")?)
}

pub fn gate(j:&Journal,project:&str)->Result<(),String> {
    // Existing installations are queued during service startup before claims are admitted.
    let Some(v)=saved(j,project)? else { return Ok(()) };
    let (board,root)=binding(j,project)?;
    let passed=v["ready"]==true && v["report"]["ready"]==true && v["report"]["gaps"].as_array().is_some_and(|g|g.is_empty());
    let overridden=v["override"]["actor"].as_i64().is_some_and(|a|Some(a)==j.owner().ok().flatten())
        && v["override"]["request"].as_i64().is_some_and(|id|id>0) && v["override"]["report"]==v["report"];
    if v["board"]==board && v["root"]==root && v["pending"]==false && (passed || overridden) { return Ok(()) }
    Err(format!("Project {project} setup is not ready. Complete its onboarding steps and Recheck; the protected owner may write /onboard {project} override. Nothing started."))
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

/// #245: members receive a factual refusal and a durable owner escalation, without owner remedies.
pub(crate) fn member_refusal(j:&Journal,r:&crate::journal::Run,project:&str,language:Option<&str>)->Result<Option<String>,String> {
    let Some(reason)=refusal(j,project,language)? else {return Ok(None)};
    Ok(Some(setup_refusal(j,r.request,project,language,&reason)?))
}

pub(crate) fn setup_refusal(j:&Journal,request:i64,project:&str,language:Option<&str>,reason:&str)->Result<String,String> {
    if j.owner()?==Some(j.sender(request)?) {return Ok(reason.into());}
    let ru=language.unwrap_or("en").starts_with("ru");
    let notice=crate::direct::queue_blocker(j,request,project,reason).unwrap_or_else(|_|if ru {"Не удалось поставить сообщение владельцу в очередь; доставка не подтверждена.".into()} else {"The owner notice could not be queued; delivery is not confirmed.".into()});
    Ok(if ru {format!("{project}: настройка не завершена. Ничего не запущено. {notice}")}
        else {format!("Project {project} setup is not ready. Nothing started. {notice}")})
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
    let fix=fix.into();
    items.push(json!({"check":name,"owner_only":!fix.starts_with("PM:"),"fix":fix}));
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
        gap(&mut gaps,"claim","PM: complete the create_claim_issue prerequisite through TaskQ Add project setup for this board using existing on-host credentials, verify it through TaskQ, then Recheck");
    }
    // An absent project observation proves no missing grant/capacity. Restore service
    // evidence first instead of asking the owner to change unknown host policy.
    if snapshot["complete"]==true && fresh(&snapshot["at"],at) {
    let access=snapshot["access"].as_array().and_then(|a|a.iter().find(|a|a["runtime"]=="codex"));
    let granted=access.is_some_and(|a|a["grant"].as_str().is_some_and(|g|!g.trim().is_empty() && !g.starts_with("none")));
    let usable=access.is_some_and(|a|matches!(a["effective"].as_str(),Some("restricted"|"workspace"|"full")) && a["refused"].is_null());
    if !granted {
        gap(&mut gaps,"codex_grant",format!("PM: review project-scoped Workspace access, then run {} and Recheck (owner or explicit shell grant required)",tq_command(tq,"service permissions --runtime codex --mode workspace --scope project")));
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
    }
    let route=if config["board"]=="gitlab" {json!({"outcome":"skipped","reason":"TaskQ service status fallback has no qualified GitLab merge-request route probe; TaskQ retains execution checks"})} else {Value::Null};
    json!({"setup_revision":2,"merge_request_route":route,"ready":gaps.is_empty(),"gaps":gaps,"checked_at":at,"source":"taskq service status --json (setup-check fallback)","service_identity":if service["schema"]=="taskq.service/1" {json!([service["service"]["instance"],service["service"]["started"],service["service"]["version"]])} else {Value::Null}})
}

/// Inspect grants through the metadata-only key API: no take(), credential injection or last-use write.
#[cfg(test)]
fn issue_key_step(keys:&crate::keys::Keys,project:&str,owner:i64)->Option<String> {
    issue_key_step_for_board(keys,project,owner,&Value::Null)
}
fn issue_key_step_for_board(keys:&crate::keys::Keys,project:&str,owner:i64,config:&Value)->Option<String> {
    let entries=match keys.list() {
        Ok(v)=>v,
        Err(_)=>return Some("Owner: repair the protected keys.json permissions/content, then run pilotq keys list and Recheck".into()),
    };
    let gitlab=config["board"]=="gitlab";
    let candidates:Vec<_>=entries.as_array().into_iter().flatten().filter(|k|
        (!gitlab || k["name"]=="pilotq-gitlab") &&
        k["tools"].as_array().is_some_and(|t|t.contains(&json!("issue_read")))
        && k["name"].as_str().is_some_and(|n|!n.is_empty() && n.len()<=32 && n.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_".contains(&c)))).collect();
    let holds=|k:&Value,field:&str,value:Value|k[field].as_array().is_some_and(|a|a.contains(&value));
    let allows=|k:&Value,route:&str|holds(k,"projects",json!(project)) || holds(k,"users",json!(owner)) || holds(k,"routes",json!(route));
    if candidates.iter().any(|k|allows(k,"cli")) && candidates.iter().any(|k|allows(k,&owner.to_string())) {return None;}
    if candidates.is_empty() && gitlab {return Some(format!("Owner: missing GitLab issue_read key; run pilotq keys add pilotq-gitlab 'GitLab issue access' issue_read with the approved on-host token via protected stdin, then grant that key to project:{project} and Recheck"));}
    if candidates.is_empty() {return Some(format!("Owner: provision an approved issue_read key through pilotq keys add (value via protected stdin), then grant it to project:{project}; pilotq keys list shows key names"));}
    let commands=candidates.iter().filter_map(|k|k["name"].as_str()).map(|name|format!("pilotq keys grant {name} project:{project}")).collect::<Vec<_>>().join(" OR ");
    Some(format!("Owner: choose the approved issue_read key and run {commands}, then Recheck"))
}

fn probe(bin:&Path,root:&str,args:&[&str],end:Instant,active:impl Fn()->bool)->Value {
    let o=crate::commands::execute_bounded(crate::commands::command(bin,root,args),end,active,512*1024);
    if o["status"]!="success" {return Value::Null;}
    serde_json::from_str(o["stdout"].as_str().unwrap_or("")).unwrap_or(Value::Null)
}

// TaskQ v4.6.56 host-service contract: explicit line registry + poke. Never call
// service install here: it can upgrade an image or create/change the native unit.
fn attach(tq:&Taskq,j:&std::sync::Mutex<Journal>,project:&str,epoch:&Value,end:Instant,active:impl Fn()->bool)->Result<(),String> {
    use std::fs;
    let service=probe(&tq.bin,&tq.root.to_string_lossy(),&["service","status","--json"],end,&active);
    if service["schema"]!="taskq.service/1" || service["service"]["state"]!="running" || service["service"]["fresh"]!=true
        || !fresh(&service["service"]["at"],crate::now() as i64) || !service["registry_error"].is_null() {
        return Err("Owner: restore a healthy installed TaskQ service, then Recheck".into());
    }
    if service["projects"].as_array().is_some_and(|entries|entries.iter().any(|p|p["root"].as_str()==tq.root.to_str()
        && p["status"]["complete"]==true && fresh(&p["status"]["at"],crate::now() as i64) && p["error"].is_null()
        && (p["host_binding"].is_null() || p["host_binding"]=="current"))) {return Ok(());}
    if service["service"]["install_protocol"]!=1 {
        return Err("Owner: update/reconcile the TaskQ host through its coordinated migration route to install_protocol 1, then Recheck".into());
    }
    let exe=fs::canonicalize(&tq.bin).map_err(|_|"TaskQ installation unavailable")?;
    let parent=exe.parent().ok_or("TaskQ installation unavailable")?;
    let install=if parent.parent().is_some_and(|p|p.file_name().is_some_and(|n|n=="releases")) {
        parent.parent().and_then(Path::parent).ok_or("TaskQ installation unavailable")?
    } else {parent};
    // Refuse source binaries and missing installations without creating their directories.
    if !install.join("host-policy-migration/install.json").is_file() || !install.join("resource-state").is_dir() {
        return Err("Owner: install or repair the TaskQ host service, then Recheck".into());
    }
    for name in ["resource-state","host-policy-migration","host-policy-migration/install.json","service","service.lock","projects.lock","update.lock","resource-state/lock","resource-state/state.json","poke"] {
        match fs::symlink_metadata(install.join(name)) {
            Ok(m) if m.file_type().is_symlink()=>return Err("TaskQ installation state symlink refused; owner must repair it, then Recheck".into()),
            Err(e) if e.kind()!=std::io::ErrorKind::NotFound=>return Err("TaskQ installation state unavailable; Recheck".into()),
            _=>{},
        }
    }
    let _update=coreq::file_lock::try_lock(&install.join("update.lock")).map_err(|_|"TaskQ update in progress; Recheck")?;
    let _projects=coreq::file_lock::try_lock(&install.join("projects.lock")).map_err(|_|"TaskQ project registry busy; Recheck")?;
    // Serialize lifecycle submissions as TaskQ does. A fresh service alone does not
    // authorize editing its registry during a retained stop/install transition.
    let _resources=coreq::file_lock::try_lock(&install.join("resource-state/lock")).map_err(|_|"TaskQ lifecycle store busy; Recheck")?;
    for name in ["projects.pending","projects.installing","projects.installing.pending","projects.upgrading","resource-state/state.json.pending","restart","update"] {
        match fs::symlink_metadata(install.join(name)) {
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},
            _=>return Err("TaskQ has a pending host transition; owner must reconcile it, then Recheck".into()),
        }
    }
    let state=match fs::read(install.join("resource-state/state.json")) {
        Ok(bytes)=>serde_json::from_slice::<Value>(&bytes).map_err(|_|"TaskQ lifecycle store unreadable; Recheck")?,
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>json!({"ops":{}}),
        Err(_)=>return Err("TaskQ lifecycle store unavailable; Recheck".into()),
    };
    let ops=state["ops"].as_object().ok_or("TaskQ lifecycle store unsupported; Recheck")?;
    if ops.values().any(|op|["local_restart","local_stop","local_uninstall"].iter().any(|k|!op["bound"][k].is_null()) && op["receipt"]["state"]!="observed") {
        return Err("TaskQ lifecycle operation pending; owner must reconcile it, then Recheck".into());
    }
    let owner:Value=serde_json::from_slice(&fs::read(install.join("service")).map_err(|_|"TaskQ service identity unavailable")?).map_err(|_|"TaskQ service identity unreadable")?;
    if owner["schema"]!=1 || ["pid","started","instance","version"].iter().any(|k|owner[k]!=service["service"][k]) {
        return Err("TaskQ service identity changed; Recheck".into());
    }
    let pid=owner["pid"].as_u64().and_then(|n|u32::try_from(n).ok()).filter(|p|*p>0).ok_or("TaskQ service identity invalid")?;
    if !coreq::process::identity(pid).map_err(|_|"TaskQ service liveness unknown")?.is_some_and(|p|Some(p.started.as_str())==owner["started"].as_str()) {
        return Err("TaskQ service exited; owner must restore it, then Recheck".into());
    }
    match coreq::file_lock::try_lock(&install.join("service.lock")) {
        Err(std::fs::TryLockError::WouldBlock)=>{},
        _=>return Err("TaskQ service ownership unavailable; Recheck".into()),
    }
    let path=install.join("projects");
    if !fs::symlink_metadata(&path).is_ok_and(|m|m.is_file() && !m.file_type().is_symlink()) {return Err("TaskQ registry must be an existing ordinary file; Recheck".into());}
    let before=fs::read_to_string(&path).map_err(|_|"TaskQ registry unreadable; Recheck")?;
    let root=fs::canonicalize(&tq.root).map_err(|_|"Project root unavailable")?;
    let root_text=root.to_str().filter(|s|!s.contains(['\r','\n'])).ok_or("Project root cannot be represented in TaskQ registry")?;
    let roots=before.lines().filter(|l|!l.is_empty()).map(|s| {
        let p=Path::new(s);if !p.is_absolute() {return Err("TaskQ registry contains a relative root");}
        fs::canonicalize(p).ok().filter(|p|p.is_dir()).ok_or("TaskQ registry contains an unavailable root")
    }).collect::<Result<Vec<_>,_>>()?;
    if !active() {return Err("Onboarding authorization changed; attach refused".into());}
    let mut intent=json!({"root":root,"service":owner,"state":"intent"});
    {
        let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;
        let mut v=saved(&j,project)?.ok_or("Onboarding missing")?;
        if &v["epoch"]!=epoch {return Err("Onboarding changed; Recheck".into());}
        v["attach"]=intent.clone();save(&j,project,&v)?;
        let request=v["attach_request"].as_i64().or(v["card_request"].as_i64()).ok_or("Onboarding audit source unavailable")?;
        j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'project_attach',?3,?4,'intent')",params![request,crate::now() as i64,project,intent.to_string()]).map_err(err)?;
        tx.commit().map_err(err)?;
    }
    if !roots.contains(&root) {
        // TaskQ service install also defaults new roots to focus off; attaching never admits tasks.
        let dot=root.join(".taskq");fs::create_dir_all(&dot).map_err(|_|"Project state directory unavailable")?;
        if fs::symlink_metadata(&dot).map_err(|_|"Project state directory unavailable")?.file_type().is_symlink() {return Err("Project state symlink refused".into());}
        use std::io::Write;
        match fs::OpenOptions::new().write(true).create_new(true).open(dot.join("focus")) {
            Ok(mut f)=>{f.write_all(b"off\n").and_then(|_|f.sync_all()).map_err(|_|"Project focus write failed")?;},
            Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{
                if !fs::symlink_metadata(dot.join("focus")).is_ok_and(|m|m.is_file() && !m.file_type().is_symlink()) {return Err("Project focus must be an ordinary file".into());}
            },
            Err(_)=>return Err("Project focus unavailable".into()),
        }
        fs::File::open(&dot).and_then(|f|f.sync_all()).and_then(|_|fs::File::open(&root)?.sync_all()).map_err(|_|"Project state sync failed")?;
        let next=format!("{before}{}{root_text}\n",if before.is_empty() || before.ends_with('\n') {""} else {"\n"});
        if !active() {return Err("Onboarding authorization changed; attach refused".into());}
        // Preserve registry permissions before atomic publication, including after a crash.
        let pending=path.with_extension("pending");
        let mut options=fs::OpenOptions::new();options.write(true).create_new(true);
        #[cfg(unix)] {use std::os::unix::fs::{OpenOptionsExt,PermissionsExt};options.mode(fs::metadata(&path).map_err(|_|"TaskQ registry metadata unavailable")?.permissions().mode() & 0o777);}
        let mut file=options.open(&pending).map_err(|_|"TaskQ registry prepare unavailable; owner must reconcile it, then Recheck")?;
        file.set_permissions(fs::metadata(&path).map_err(|_|"TaskQ registry metadata unavailable")?.permissions()).map_err(|_|"TaskQ registry permissions unavailable")?;
        file.write_all(next.as_bytes()).and_then(|_|file.sync_all()).map_err(|_|"TaskQ registry prepare failed; owner must reconcile it, then Recheck")?;
        fs::rename(&pending,&path).and_then(|_|fs::File::open(install)?.sync_all()).map_err(|_|"TaskQ registry publication uncertain; Recheck reads back the same root")?;
        if fs::read_to_string(&path).map_err(|_|"TaskQ registry readback unavailable")?!=next {return Err("TaskQ registry readback differs; attach uncertain".into());}
    }
    coreq::write_durable(&install.join("poke"),"")?;
    intent["state"]=json!("registered");
    let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;
    let mut v=saved(&j,project)?.ok_or("Onboarding missing")?;
    if &v["epoch"]!=epoch {return Err("Onboarding changed; Recheck".into());}
    v["attach"]=intent.clone();save(&j,project,&v)?;
    let request=v["attach_request"].as_i64().or(v["card_request"].as_i64()).ok_or("Onboarding audit source unavailable")?;
    j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'project_attach',?3,?4,'registered')",params![request,crate::now() as i64,project,intent.to_string()]).map_err(err)?;
    tx.commit().map_err(err)?;
    Ok(())
}

/// Checks never hold the journal lock across subprocesses, run a model or poll updates.
pub fn check(t:&coreq::Tool,j:&Shared,project:&str)->Result<Value,String> {
    let config=crate::config(t)?.ok_or("PilotQ configuration unavailable")?;
    let tq=config.taskq.as_ref().ok_or("TaskQ configuration unavailable")?;
    check_registered(&config.direct,tq,j,project,Instant::now()+Duration::from_secs(60))
}

pub fn check_registered(d:&crate::direct::Direct,tq:&Taskq,j:&std::sync::Mutex<Journal>,project:&str,end:Instant)->Result<Value,String> {
    let exe=std::env::current_exe().map_err(|_|"PilotQ binary unavailable")?;
    check_with(d,tq,j,project,end,&exe)
}

// A5 uses the same registration/check/apply chain with invented executable and board boundaries.
pub(crate) fn check_with(d:&crate::direct::Direct,tq:&Taskq,j:&std::sync::Mutex<Journal>,project:&str,end:Instant,exe:&Path)->Result<Value,String> {
    let owner=j.lock().unwrap().owner()?.ok_or("Owner unavailable")?;
    let (board,root,epoch)={
        let j=j.lock().unwrap();
        if !j.has(owner,"chat")? || !j.has(owner,"projects")? {return Err("Owner access revoked".into());}
        let (board,root)=binding(&j,project)?;
        let tx=j.0.unchecked_transaction().map_err(err)?;
        let mut v=saved(&j,project)?.ok_or("Onboarding not queued")?;
        if v["root"]!=root || v["board"]!=board {return Err("Registration changed; Recheck must queue the new binding before checks".into());}
        if !attach_authorized(&j,&v,owner)? {return Err("Registration authority revoked or stopped; the owner must Recheck".into());}
        let created=v["card_request"].is_null();
        let source=card_request(&j,&mut v,project,owner)?;
        let now=crate::now() as i64;
        if v["checking_until"].as_i64().is_some_and(|until|until>now) {return Err("Setup checks already running; wait for the card update".into());}
        v["checking_until"]=json!(now+65);save(&j,project,&v)?;
        tx.commit().map_err(err)?;
        if created {j.trace(source,now);}
        (board,root,v["epoch"].clone())
    };
    let active=|| {let j=j.lock().unwrap(); j.has(owner,"chat").unwrap_or(false) && j.has(owner,"projects").unwrap_or(false) && binding(&j,project).is_ok_and(|b|b==(board.clone(),root.clone()))
        && saved(&j,project).ok().flatten().is_some_and(|v|v["epoch"]==epoch && v["root"]==root && v["board"]==board && attach_authorized(&j,&v,owner).unwrap_or(false))};
    let fallback=Taskq{bot:None,bin:tq.bin.clone(),root:root.clone().into(),poll:Duration::from_secs(15)};
    let status=probe(&fallback.bin,&root,&["status","--json"],end.min(Instant::now()+Duration::from_secs(10)),&active);
    let config:Value=std::fs::read(fallback.root.join("taskq.json")).ok().and_then(|s|serde_json::from_slice(&s).ok()).unwrap_or(Value::Null);
    let matched=status["complete"]==true && status["board"]["repo"]==board && config["repo"]==board
        && ["kind","host"].iter().all(|k|status["board"][k].is_null() || status["board"][k]==config[if *k=="kind" {"board"} else {"host"}]);
    let attached=if matched {attach(&fallback,j,project,&epoch,end.min(Instant::now()+Duration::from_secs(10)),&active)}
        else {Err("Owner: restore matching TaskQ configuration and board readback, then Recheck".into())};
    // Doctor must use the same install and service PATH. Only its JSON crosses this boundary.
    let mut cmd=std::process::Command::new(&exe);
    cmd.args(["doctor","--json"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    #[cfg(unix)] std::os::unix::process::CommandExt::process_group(&mut cmd,0);
    let output=crate::commands::execute_bounded(cmd,end.min(Instant::now()+Duration::from_secs(40)),&active,128*1024);
    let doctor=if matches!(output["status"].as_str(),Some("success"|"exit_nonzero")) {serde_json::from_str(output["stdout"].as_str().unwrap_or("")).unwrap_or(Value::Null)} else {Value::Null};
    let service=probe(&fallback.bin,&root,&["service","status","--json"],end.min(Instant::now()+Duration::from_secs(10)),&active);
    let mut result=report(&fallback,&board,&doctor,&status,&service,crate::now() as i64);
    let waiting=attached.is_ok() && service["service"]["state"]=="running" && service["service"]["fresh"]==true
        && result["gaps"].as_array().unwrap().iter().any(|g|g["check"]=="service");
    if waiting {
        for g in result["gaps"].as_array_mut().unwrap().iter_mut().filter(|g|g["check"]=="service") {
            g["fix"]=json!("PilotQ: waiting for TaskQ to observe the attached project; checks retry automatically.");g["owner_only"]=json!(false);
        }
        result["retry"]=json!(true);
    }
    if let Err(reason)=attached {
        // Keep the failed boundary explicit; never turn missing service evidence into readiness.
        result["gaps"].as_array_mut().unwrap().retain(|g|g["check"]!="service");
        let owner_only=reason.starts_with("Owner:");
        result["gaps"].as_array_mut().unwrap().push(json!({"check":"service","fix":d.keys.scrub(&reason),"owner_only":owner_only}));
        result["retry"]=json!(reason.contains("busy") || reason.contains("in progress") || reason.contains("identity changed"));
        result["ready"]=json!(false);
    }
    if let Some(step)=issue_key_step_for_board(&d.keys,project,owner,&config) {
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
        else if result["gaps"].as_array().is_some_and(|g|g.iter().any(|g|g["owner_only"]==false)) {
            if ru {format!("{project}: проект пока не готов. Результаты настройки:")} else {format!("{project}: not ready. Setup findings:")}
        } else if ru {format!("{project}: проект пока не готов. Шаги для владельца:")} else {format!("{project}: not ready. Owner steps:")};
    let mut lines=vec![title];
    if result["merge_request_route"]["outcome"]=="skipped" {
        lines.push(format!("merge_request_route: skipped; {}",result["merge_request_route"]["reason"].as_str().unwrap_or("qualification unavailable")));
    }
    for g in result["gaps"].as_array().into_iter().flatten() {lines.push(format!("{}: {}",g["check"].as_str().unwrap_or("setup"),g["fix"].as_str().unwrap_or("Owner: run pilotq doctor --json")));}
    if result["ready"]!=true {lines.push(if ru {"После исправления нажмите «Проверить снова»."} else {"After completing the listed steps, tap Recheck."}.into());}
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
    let source=card_request(j,&mut v,project,owner)?;
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
            let mut v=saved(&j,&project)?.ok_or("Onboarding unavailable")?;
            v["attach_request"]=json!(r.id);save(&j,&project,&v)?;
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
    for (project,..) in j.projects()? { if saved(j,&project)?.is_none_or(|v|v["report"]["issue_read_key_checked"]!=true || v["report"]["setup_revision"]!=2) {queue(j,&project)?;} }
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
            v["pending"]==true || v["report"]["retry"]==true
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
        assert!(line.contains("codex_grant: PM:") && line.contains("service permissions --runtime codex --mode workspace --scope project"),"{line}");
        assert!(!line.contains("setup passed"));
        for (doc,board,svc,at) in [(Value::Null,status(),service(&tq),1000),(doctor(),Value::Null,service(&tq),1000),
            (doctor(),status(),Value::Null,1000),(doctor(),status(),service(&tq),1201),
            (doctor(),json!({"complete":true,"board":{"repo":"invented/other"}}),service(&tq),1000)] {
            assert_eq!(report(&tq,"invented/cyprusbz-board",&doc,&board,&svc,at)["ready"],false);
        }
        let _=std::fs::remove_dir_all(dir);
    }
    // #251 A2/A3 / §5: PM remedies never masquerade as owner-only work; unknown MR route is non-gating.
    #[test]
    fn onboarding_251_pm_steps_and_gitlab_route() {
        let (_,dir,tq)=setup("pm-251");
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"invented/cyprusbz-board","board":"gitlab"}"#).unwrap();
        let mut svc=service(&tq);svc["projects"][0]["status"]["access"]=json!([]);
        let result=report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&svc,1000);
        for check in ["claim","codex_grant"] {
            let g=result["gaps"].as_array().unwrap().iter().find(|g|g["check"]==check).unwrap();
            assert_eq!(g["owner_only"],false,"{g}");
            assert!(g["fix"].as_str().unwrap().starts_with("PM:"));
        }
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"invented/cyprusbz-board","board":"gitlab","create_claim_issue":20}"#).unwrap();
        let mut st=status();st["merge_request_route"]=json!("unknown");
        let result=report(&tq,"invented/cyprusbz-board",&doctor(),&st,&service(&tq),1000);
        assert_eq!(result["ready"],true,"{result}");
        assert_eq!(result["merge_request_route"]["outcome"],"skipped");
        assert!(summary("pah",&result).contains("merge_request_route: skipped"));
        assert!(result["merge_request_route"]["reason"].as_str().unwrap().contains("fallback"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    // #251 / R5: reports from older binaries must not retain GitHub-only readiness after restart.
    #[test]
    fn onboarding_251_refresh_old_readiness() {
        let (j,dir,tq)=setup("upgrade-251");
        queue(&j,"cyprusbz-board").unwrap();
        let mut v=saved(&j,"cyprusbz-board").unwrap().unwrap();
        let mut old=report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&service(&tq),1000);
        old.as_object_mut().unwrap().remove("setup_revision");old["issue_read_key_checked"]=json!(true);
        v["report"]=old;v["pending"]=json!(false);v["ready"]=json!(true);save(&j,"cyprusbz-board",&v).unwrap();
        startup(&j).unwrap();
        assert!(gate(&j,"cyprusbz-board").is_err(),"old GitHub-only readiness survived startup");
        let mut v=saved(&j,"cyprusbz-board").unwrap().unwrap();
        v["report"]=report(&tq,"invented/cyprusbz-board",&doctor(),&status(),&service(&tq),1000);
        v["report"]["issue_read_key_checked"]=json!(true);v["pending"]=json!(false);v["ready"]=json!(true);
        save(&j,"cyprusbz-board",&v).unwrap();startup(&j).unwrap();
        assert!(gate(&j,"cyprusbz-board").is_ok(),"current reports should not be requeued on every restart");
        std::fs::remove_dir_all(dir).unwrap();
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

    // #251 A4 / R16: a scoped GitHub key must not hide a missing GitLab key; values stay protected.
    #[test]
    fn onboarding_251_gitlab_key() {
        let (_,dir,_)=setup("gitlab-key-251");let keys=crate::keys::Keys(dir.join("keys.json"));
        keys.add("pilotq-github","invented",&["issue_read".into()],"invented-github-credential").unwrap();
        keys.grant("pilotq-github",&["project:pah".into()]).unwrap();
        let config=json!({"board":"gitlab","host":"gitlab.example.test"});
        let missing=issue_key_step_for_board(&keys,"pah",7,&config).expect("GitHub key cannot qualify GitLab");
        assert_eq!(missing.matches("pilotq-gitlab").count(),1,"{missing}");
        assert!(!missing.contains("invented-github-credential"));
        keys.add("pilotq-gitlab","invented",&["issue_read".into()],"invented-gitlab-credential").unwrap();
        let step=issue_key_step_for_board(&keys,"pah",7,&config).unwrap();
        assert!(step.contains("pilotq keys grant pilotq-gitlab project:pah"),"{step}");
        keys.grant("pilotq-gitlab",&["project:pah".into()]).unwrap();
        let before=keys.list().unwrap();
        assert!(issue_key_step_for_board(&keys,"pah",7,&config).is_none());
        assert_eq!(before,keys.list().unwrap(),"readiness must not consume credentials or modify grants");
        std::fs::remove_dir_all(dir).unwrap();
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

    // #236 A1 / R11, §5: setup can close after authorization; the durable
    // receipt must retain the localized remedy through finish and chat delivery.
    #[test]
    fn onboarding_236_gate_closes_after_authorization() {
        for lang in ["en","ru"] {
            let (mut j,dir,tq)=setup(&format!("236-late-{lang}"));
            let mut input=press(1,7,0,0);input.callback=None;input.reply_to=None;input.text=Some("plan cyprusbz-board".into());
            j.admit(&[(1,Some(input))],1).unwrap();
            let draft=j.claim_next(4,1).unwrap().remove(0);
            j.save_plan(&draft,&json!([{"title":"Invented task"}]),"cyprusbz-board","invented/cyprusbz-board",dir.to_str().unwrap(),1).unwrap();
            j.finish(&draft,None,None,&crate::manager::Outcome::Answered("Prepared".into()),2).unwrap();
            let mut input=press(2,7,0,0);input.callback=None;input.reply_to=None;input.text=Some("run plan".into());
            j.admit(&[(2,Some(input))],3).unwrap();
            let launch=j.claim_next(4,3).unwrap().remove(0);
            j.0.execute("UPDATE requests SET language=?2 WHERE id=?1",params![launch.request,lang]).unwrap();
            assert!(gate(&j,"cyprusbz-board").is_ok());
            j.authorize(&launch,1,7,30,3).unwrap();
            queue(&j,"cyprusbz-board").unwrap();
            let expected=refusal(&j,"cyprusbz-board",Some(lang)).unwrap().unwrap();
            let shared=std::sync::Mutex::new(j);
            let reply=crate::taskq::launch(&tq,&shared,&launch).unwrap().unwrap();
            assert_eq!(reply,expected,"late gate refusal lost at launch");
            let j=shared.into_inner().unwrap();
            j.finish(&launch,None,None,&crate::manager::Outcome::Answered(reply),4).unwrap();
            assert_eq!(j.request(launch.request).unwrap().1.unwrap(),expected);
            let delivered:String=j.0.query_row("SELECT text FROM outbox WHERE request_id=?1 AND kind='plan_problem'",[launch.request],|r|r.get(0)).unwrap();
            assert!(delivered.contains(&expected),"{delivered}");
            let filed:i64=j.0.query_row("SELECT count(*) FROM tickets WHERE task IS NOT NULL",[],|r|r.get(0)).unwrap();
            assert_eq!(filed,0);
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
