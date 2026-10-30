//! #174 / §5: protected registration, DEV observations and single-use release intents.
use crate::{journal::{Journal,Run},taskq::Taskq,telegram::Shared};
use rusqlite::{params,OptionalExtension};
use serde_json::{json,Value};
use std::{path::Path,time::{Duration,Instant}};
fn err(e:rusqlite::Error)->String {e.to_string()}
fn get(j:&Journal,key:&str)->Result<Option<Value>,String> {
    let s:Option<String>=j.0.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[key],|r|r.get(0)).optional().map_err(err)?;
    s.map(|s|serde_json::from_str(&s).map_err(|_|"Release record unreadable".into())).transpose()
}
fn put(j:&Journal,key:&str,v:&Value)->Result<(),String> {j.0.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![key,v.to_string()]).map(drop).map_err(err)}
fn sha(s:&str)->bool {s.len()==40 && s.bytes().all(|c|c.is_ascii_hexdigit())}

/// Only TaskQ's trusted terminal delivery receipt proves a merge identity.
pub fn merged_sha(issue:&Value)->Option<String> {
    if let Some(s)=issue["merged_sha"].as_str().filter(|s|sha(s)) {return Some(s.into());}
    issue["comments"].as_array()?.iter().rev().filter(|c|c["trusted"]==true)
        .filter_map(|c|c["body"].as_str()).filter(|b|b.starts_with("taskq: close\n"))
        .flat_map(|b|b.lines().rev()).find_map(|line| {
            if !line.starts_with("delivery ") || !line.contains("; merge-result CI green") {return None;}
            let (_,part)=line.split_once(" as ")?;let s=part.split(';').next()?;sha(s).then(||s.into())
        })
}

fn binding(j:&Journal,p:&str)->Result<(String,String),String> {j.projects()?.into_iter().find(|x|x.0==p).map(|x|(x.1,x.2)).ok_or("Project is not registered".into())}
fn authorize(j:&Journal,user:i64,p:&str)->Result<(),String> {
    if j.owner()?!=Some(user) || !j.has(user,"chat")? || !j.has(user,&format!("release:{p}"))?
        || !(j.has(user,"projects")? || j.has(user,&format!("project:{p}"))?) {return Err("Release refused: protected owner and separate release grant required".into());} Ok(())
}
fn config(j:&Journal,p:&str)->Result<Value,String> {
    let (board,root)=binding(j,p)?;
    let c=get(j,&format!("release-config:{p}"))?.ok_or("Release configuration missing")?;
    if c["board"]!=board || c["root"]!=root {return Err("Release registration changed".into());}
    let local:Value=std::fs::read(Path::new(&root).join("taskq.json")).ok().and_then(|b|serde_json::from_slice(&b).ok()).ok_or("Registered root unreadable")?;
    if local["repo"]!=board || std::fs::canonicalize(&root).ok().as_deref()!=Some(Path::new(&root)) {return Err("Registered root changed".into());} Ok(c)
}
fn argv(v:&Value)->Result<Vec<String>,String> {
    let a:Vec<String>=serde_json::from_value(v.clone()).map_err(|_|"Command must be a string array")?;
    if a.is_empty() || !Path::new(&a[0]).is_absolute() || a.iter().any(|s|s.contains('\0')) {return Err("Command executable must be absolute".into());} Ok(a)
}
pub fn configure(j:&Journal,user:i64,p:&str,mut c:Value)->Result<Value,String> {
    if j.owner()?!=Some(user) {return Err("Only the protected owner configures releases".into());}
    let (board,root)=binding(j,p)?;
    if !c["host"].as_str().is_some_and(|s|!s.is_empty() && s.chars().all(|c|c.is_ascii_alphanumeric() || "-_.".contains(c))) {return Err("Configured local TaskQ host identity required".into());}
    let url=url::Url::parse(c["dev_health_url"].as_str().ok_or("DEV health URL required")?).map_err(|_|"Invalid DEV URL")?;
    if url.scheme()!="https" || !url.username().is_empty() || url.password().is_some() {return Err("DEV URL must use HTTPS without credentials".into());}
    if !c["git"].as_str().is_some_and(|s|Path::new(s).is_absolute()) {return Err("Absolute Git executable required".into());}
    for key in ["dry_run","release","pipeline","preflight","backup"] {argv(&c[key])?;}
    for key in ["release","pipeline"] {if !argv(&c[key])?.contains(&"{sha}".into()) {return Err(format!("{key} requires a separate {{sha}} argument"));}}
    for key in ["main_ref","live_ref"] {let s=c[key].as_str().ok_or("main_ref/live_ref required")?;if !s.starts_with("refs/") || s.chars().any(char::is_whitespace) {return Err("Use explicit refs/... references".into());}}
    if !c["migration_paths"].as_array().is_some_and(|a|!a.is_empty() && a.iter().all(|s|s.as_str().is_some_and(|s|!s.is_empty() && !s.starts_with('/') && !s.contains("..")))) {return Err("Explicit relative migration_paths required".into());}
    for bin in std::iter::once(c["git"].as_str().unwrap().to_string()).chain(["dry_run","release","pipeline","preflight","backup"].into_iter().map(|k|argv(&c[k]).unwrap()[0].clone())) {
        let minimum=c["requirements"][&bin].as_str().and_then(crate::host::version).ok_or("Every release executable requires a minimum version in requirements")?;
        if c["git"]==bin && minimum<crate::host::version("2.25.0").unwrap() {return Err("Git minimum must be at least 2.25.0".into());}
    }
    c["board"]=json!(board);c["root"]=json!(root);
    let tx=j.0.unchecked_transaction().map_err(err)?;
    put(j,&format!("release-config:{p}"),&c)?;
    let update:i64=j.0.query_row("SELECT min(coalesce(min(update_id),0),0)-1 FROM requests",[],|r|r.get(0)).map_err(err)?;
    let id=crate::journal::insert(&j.0,update,"cli",user,None,user,None,0,"release_config",Some(&format!("Configure release {p}")),"answered",crate::now() as i64,&format!("cli:{user}"),None)?;
    audit(j,id,p,"configuration",&json!({"board":board,"root":root,"actor":user}))?;
    let readback=config(j,p)?;tx.commit().map_err(err)?;j.trace(id,crate::now() as i64);Ok(readback)
}
pub fn host_checks(path:&Path)->Vec<Value> {
    if !path.exists() {return vec![];}
    let result=(|| -> Result<Vec<Value>,String> {
        let db=rusqlite::Connection::open_with_flags(path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(err)?;
        let mut q=db.prepare("SELECT CAST(value AS TEXT) FROM meta WHERE key LIKE 'release-config:%'").map_err(err)?;
        let rows=q.query_map([],|r|r.get::<_,String>(0)).and_then(Iterator::collect::<Result<Vec<_>,_>>).map_err(err)?;
        let mut requirements=std::collections::BTreeMap::new();
        for row in rows {let c:Value=serde_json::from_str(&row).map_err(|_|"Release config unreadable")?;
            for (bin,min) in c["requirements"].as_object().ok_or("Release requirements missing")? {
                let min=min.as_str().ok_or("Release minimum missing")?;
                let v=crate::host::version(min).ok_or("Release minimum invalid")?;
                let item=requirements.entry(bin.clone()).or_insert((v.clone(),min.to_string()));if v>item.0 {*item=(v,min.to_string());}
            }
        }
        Ok(requirements.into_iter().map(|(bin,(_,min))|{
            let mut v=crate::host::probe("command",Some(&bin),&["--version"],&min,"","Provision this registered release executable with --version, then rerun doctor");
            v["tool"]=json!(format!("release:{bin}"));v
        }).collect())
    })();
    result.unwrap_or_else(|e|vec![json!({"tool":"release_requirements","found":null,"minimum":null,"outcome":"failed","fix":e})])
}
fn read(tq:&Taskq,board:&str,args:&[&str],end:Instant)->Result<Value,String> {
    let output=crate::commands::execute_bounded(crate::commands::command(&tq.bin,tq.root.to_str().ok_or("Registered root unavailable")?,args),end,||true,1024*1024);
    if output["status"]!="success" {return Err("TaskQ release evidence unavailable".into());}
    let v:Value=serde_json::from_str(output["stdout"].as_str().unwrap_or("")).map_err(|_|"TaskQ release evidence unreadable")?;
    if args[0]=="status" && (v["complete"]!=true || v["board"]["repo"]!=board || !v["tasks"].is_array()) {return Err("TaskQ release snapshot incomplete or mismatched".into());}
    Ok(v)
}
fn run(c:&Value,field:&str,pinned:&str,end:Instant,active:impl Fn()->bool)->Value {
    let a=match argv(&c[field]) {Ok(a)=>a,Err(e)=>return json!({"status":"invalid","reason":e})};
    let a:Vec<_>=a.iter().map(|s|if s=="{sha}" {pinned} else {s.as_str()}).collect();
    crate::commands::execute_bounded(crate::commands::command(a[0],c["root"].as_str().unwrap(),&a[1..]),end,active,65536)
}
fn git(c:&Value,args:&[&str])->Result<String,String> {git_until(c,args,Instant::now()+Duration::from_secs(5))}
fn git_until(c:&Value,args:&[&str],end:Instant)->Result<String,String> {
    let out=crate::commands::execute_bounded(crate::commands::command(c["git"].as_str().ok_or("Git unavailable")?,c["root"].as_str().ok_or("Root unavailable")?,args),end,||true,65536);
    if out["status"]!="success" {return Err("Git read failed; deployment state unknown".into());} Ok(out["stdout"].as_str().ok_or("Git output unavailable")?.trim().into())
}
fn resolve(c:&Value,reference:&str)->Result<String,String> {resolve_until(c,reference,Instant::now()+Duration::from_secs(5))}
fn resolve_until(c:&Value,reference:&str,end:Instant)->Result<String,String> {let s=git_until(c,&["rev-parse","--verify",&format!("{reference}^{{commit}}")],end)?;if !sha(&s) {return Err("Commit identity unavailable".into());}Ok(s)}
fn ancestor(c:&Value,a:&str,b:&str)->Result<bool,String> {ancestor_until(c,a,b,Instant::now()+Duration::from_secs(5))}
fn ancestor_until(c:&Value,a:&str,b:&str,end:Instant)->Result<bool,String> {
    if !sha(a)||!sha(b) {return Err("Merged/deployed SHA unavailable".into());}
    // Distinguish a negative ancestry result from missing commits and Git errors.
    resolve_until(c,a,end)?;resolve_until(c,b,end)?;
    let out=crate::commands::execute_bounded(crate::commands::command(c["git"].as_str().ok_or("Git unavailable")?,c["root"].as_str().ok_or("Root unavailable")?,&["merge-base","--is-ancestor",a,b]),end,||true,1024);
    match (out["status"].as_str(),out["exit_code"].as_i64()) {(Some("success"),Some(0))=>Ok(true),(Some("exit_nonzero"),Some(1))=>Ok(false),_=>Err("DEV ancestry unknown".into())}
}
fn health(c:&Value)->Result<Value,String> {
    let url=c["dev_health_url"].as_str().ok_or("DEV URL missing")?.to_string();
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|"DEV runtime unavailable")?.block_on(async {
            let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(10)).build().map_err(|_|"DEV client unavailable")?;
            let mut response=client.get(url).send().await.map_err(|_|"DEV health read failed")?.error_for_status().map_err(|_|"DEV health status failed")?;
            let mut bytes=Vec::new();while let Some(chunk)=response.chunk().await.map_err(|_|"DEV health read failed")? {if bytes.len()+chunk.len()>65536 {return Err("DEV health too large");}bytes.extend_from_slice(&chunk);}
            let v:Value=serde_json::from_slice(&bytes).map_err(|_|"DEV health unreadable")?;
            if !v["sha"].as_str().is_some_and(sha) {return Err("DEV health SHA unavailable");} Ok(v)
        })
    }).join().map_err(|_|"DEV health unavailable")?.map_err(String::from)
}
fn dev_read(j:&Journal,p:&str,c:&Value,merged:Option<&str>,h:&Value)->Result<Value,String> {
    let a=merged.ok_or("Merged SHA unavailable")?;let b=h["sha"].as_str().ok_or("DEV SHA unavailable")?;
    let included=ancestor(c,a,b)?;
    if !included {return Ok(json!({"state":"not yet on DEV","sha":b,"serves_target_exactly":a==b,"observed_at":crate::now()}));}
    let key=format!("dev-first:{p}:{a}");
    let first=get(j,&key)?.unwrap_or(json!(crate::now()));put(j,&key,&first)?;
    let time=h["deployed_at"].as_str().or_else(||h["started_at"].as_str()).filter(|s|!s.is_empty() && s.len()<100);
    Ok(json!({"state":match time {Some(t)=>format!("on DEV since {t}"),None=>format!("on DEV since {} (first observed)",first)},"sha":b,"serves_target_exactly":a==b,"observed_at":crate::now()}))
}
pub fn dev(j:&Journal,p:&str,merged:Option<&str>)->Value {
    let result=(||{let c=config(j,p)?;let h=health(&c)?;dev_read(j,p,&c,merged,&h)})();
    result.unwrap_or_else(|e:String|json!({"state":"unknown","reason":e,"observed_at":crate::now()}))
}
fn project_numbers(j:&Journal,p:&str,tasks:&Value)->Result<std::collections::BTreeSet<i64>,String> {
    let mut numbers=std::collections::BTreeSet::new();for t in tasks.as_array().ok_or("Task list unavailable")? {if let Some(n)=t["n"].as_i64(){numbers.insert(n);}}
    let mut q=j.0.prepare("SELECT number FROM tasks WHERE project=?1 ORDER BY observed DESC LIMIT 12").map_err(err)?;
    let cached=q.query_map([p],|r|r.get::<_,i64>(0)).and_then(Iterator::collect::<Result<Vec<_>,_>>).map_err(err)?;numbers.extend(cached);Ok(numbers)
}
pub fn project_status_dev(tq:&Taskq,j:&std::sync::Mutex<Journal>,p:&str,tasks:&Value)->Value {
    let result=(|| -> Result<Value,String> {
        let (board,numbers)={let j=j.lock().unwrap();config(&j,p)?;let (board,_)=binding(&j,p)?;
            let numbers=project_numbers(&j,p,tasks)?;(board,numbers)};
        if numbers.len()>20 {return Err("Too many tasks for a bounded DEV observation".into());}
        let end=Instant::now()+Duration::from_secs(15);let mut projected=vec![];
        for n in numbers {let issue=read(tq,&board,&["issue",&n.to_string(),"--json"],end);
            let merged=issue.ok().filter(|i|i["number"]==n).as_ref().and_then(merged_sha);
            projected.push(json!({"n":n,"merged_sha":merged}));}
        Ok(project_dev(&j.lock().unwrap(),p,&json!(projected)))
    })();result.unwrap_or_else(|e|json!({"state":"unknown","reason":e}))
}
pub fn project_dev(j:&Journal,p:&str,tasks:&Value)->Value {
    let result=(|| {let c=config(j,p)?;let h=health(&c)?;let main=resolve(&c,c["main_ref"].as_str().ok_or("Main ref missing")?)?;
        let list=tasks.as_array().ok_or("Task list unavailable")?.iter().map(|t|json!({"task":t["n"],"dev":dev_read(j,p,&c,t["merged_sha"].as_str(),&h).unwrap_or_else(|e|json!({"state":"unknown","reason":e}))})).collect::<Vec<_>>();
        Ok(json!({"main":dev_read(j,p,&c,Some(&main),&h)?,"tasks":list}))})();
    result.unwrap_or_else(|e:String|json!({"state":"unknown","reason":e}))
}
fn repository_origin(raw:&str)->String {
    let s=raw.trim().trim_end_matches('/').trim_end_matches(".git");
    if let Ok(u)=url::Url::parse(s) {if let Some(host)=u.host_str() {return format!("{host}/{}",u.path().trim_start_matches('/')).to_lowercase();}}
    let s=s.strip_prefix("git@").unwrap_or(s);
    s.replacen(':',"/",1).to_lowercase()
}
fn blockers(tq:&Taskq,j:&Journal,p:&str,c:&Value)->Result<(),String> {
    let end=Instant::now()+Duration::from_secs(15);
    let host=c["host"].as_str().ok_or("Local TaskQ host identity unavailable")?;
    let origin=repository_origin(&git(c,&["remote","get-url","origin"])?);
    for (name,board,root,..) in j.projects()? {
        if name==p {continue;}
        let mut other=c.clone();other["root"]=json!(root);
        if repository_origin(&git(&other,&["remote","get-url","origin"])?)!=origin {continue;}
        let (bound,_)=tq.registered(j,&name)?;
        let snapshot=read(&bound,&board,&["status","--json"],end)?;
        for task in snapshot["tasks"].as_array().ok_or("Blocker snapshot incomplete")? {
            let attempts=match task["executions"].as_array() {
                Some(a)=>a,
                None if matches!(task["state"].as_str(),Some("held"|"ready"|"later"|"waiting"|"ask"|"failed"))=>continue,
                None=>return Err("Running attempt evidence unavailable".into()),
            };
            for attempt in attempts {
                if matches!(attempt["state"].as_str(),Some("running"|"doing")) {
                    if attempt["host"].as_str().is_some_and(|h|h!=host) {continue;}
                    return Err(format!("Release blocked by {name} task #{} attempt {}",task["n"],attempt["attempt"]));
                }
            }
        }
    } Ok(())
}
fn audit(j:&Journal,id:i64,p:&str,stage:&str,v:&Value)->Result<(),String> {j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'release',?3,?4,?5)",params![id,crate::now() as i64,p,stage,v.to_string()]).map(drop).map_err(err)}
fn result_card(j:&Journal,request:i64,preview:i64,text:&str)->Result<(),String> {
    let (front,chat,topic):(String,i64,Option<i64>)=j.0.query_row("SELECT front,chat,topic FROM requests WHERE id=?1",[request],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(err)?;
    if front=="telegram" {
        crate::journal::outbox(&j.0,request,chat,topic,None,None,text,crate::now() as i64)?;
        j.0.execute("UPDATE outbox SET kind='release_result',revision=?2 WHERE id=?1",params![j.0.last_insert_rowid(),preview]).map_err(err)?;
    } Ok(())
}
fn card(j:&Journal,id:i64,p:&str,text:&str)->Result<(),String> {
    let (front,chat,topic):(String,i64,Option<i64>)=j.0.query_row("SELECT front,chat,topic FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(err)?;
    if matches!(front.as_str(),"telegram"|"cli") {
        crate::journal::outbox(&j.0,id,chat,topic,None,None,text,crate::now() as i64)?;
        j.0.execute("UPDATE outbox SET kind='release_preview',project=?2 WHERE id=?1",params![j.0.last_insert_rowid(),p]).map_err(err)?;
    } Ok(())
}
pub fn preview(tq:&Taskq,j:&std::sync::Mutex<Journal>,r:&Run,p:&str)->Result<String,String> {
    let user=j.lock().unwrap().sender(r.request)?;
    let started:i64=j.lock().unwrap().0.query_row("SELECT started_at FROM runs WHERE id=?1",[r.id],|x|x.get(0)).map_err(err)?;
    let remaining=60i64.saturating_sub((crate::now() as i64).saturating_sub(started)).max(0) as u64;
    preview_for(tq,j,r.request,&r.session,r.generation,user,p,Instant::now()+Duration::from_secs(remaining.min(45)),||!j.lock().unwrap().fenced(r).unwrap_or(true))
}
fn preview_for(tq:&Taskq,j:&std::sync::Mutex<Journal>,id:i64,session:&str,generation:i64,user:i64,p:&str,end:Instant,active:impl Fn()->bool)->Result<String,String> {
    if Instant::now()>=end {return Err("Preview deadline reached".into());}
    {
        let j=j.lock().unwrap();authorize(&j,user,p)?;
        if let Some(saved)=get(&j,&format!("release-preview:{id}"))? {
            if saved["project"]!=p || saved["state"]!="preview" {return Err("This request already has an immutable release preview".into());}
            return Ok(saved["text"].as_str().unwrap_or("Preview already recorded; use its Release button").into());
        }
    }
    let (session,generation)={
        let j=j.lock().unwrap();
        let front:String=j.0.query_row("SELECT front FROM requests WHERE id=?1",[id],|r|r.get(0)).map_err(err)?;
        if front=="cli" {let session=format!("dm:{user}");let generation=j.0.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&session],|r|r.get::<_,i64>(0)).map_err(err)?;(session,generation)}else{(session.to_string(),generation)}
    };
    let c={let j=j.lock().unwrap();authorize(&j,user,p)?;let c=config(&j,p)?;blockers(tq,&j,p,&c)?;c};
    let (bound,board)={let j=j.lock().unwrap();tq.registered(&j,p)?};
    let snapshot=read(&bound,&board,&["status","--json"],end)?;
    let target=resolve_until(&c,c["main_ref"].as_str().unwrap(),end)?;let live=resolve_until(&c,c["live_ref"].as_str().unwrap(),end)?;
    if !ancestor_until(&c,&live,&target,end)? {return Err("LIVE is not an ancestor of the target".into());}
    let range=format!("{live}..{target}");
    let count=git_until(&c,&["rev-list","--count",&range],end)?.parse::<u64>().map_err(|_|"Commit count unavailable")?;
    let authors=git_until(&c,&["log","--format=%an <%ae>",&range],end)?;
    let paths=git_until(&c,&["diff","--name-only",&live,&target],end)?;
    let migrations=paths.lines().any(|path|c["migration_paths"].as_array().unwrap().iter().any(|v|path.starts_with(v.as_str().unwrap())));
    let dry=run(&c,"dry_run",&target,end,||active());
    if dry["status"]!="success" {return Err("Release dry run failed; no Release button created".into());}
    // Readbacks supply task URLs; commit text never creates links or authority.
    let mut links=vec![];
    let messages=git_until(&c,&["log","--format=%B",&range],end)?;
    let mut numbers=std::collections::BTreeSet::new();
    for (_,tail) in messages.match_indices('#').map(|(i,_)|(i,&messages[i+1..])) {
        let digits:String=tail.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(n)=digits.parse::<i64>() {if n>0 {numbers.insert(n);}}
    }
    for t in snapshot["tasks"].as_array().ok_or("Task snapshot unavailable")? {if let Some(n)=t["n"].as_i64() {numbers.insert(n);}}
    if numbers.len()>20 {return Err("Too many task references for bounded preview; narrow candidate".into());}
    for n in numbers {
        if Instant::now()>=end {return Err("Preview deadline reached".into());}
        let issue=read(&bound,&board,&["issue",&n.to_string(),"--json"],end)?;
        if issue["number"]!=n || !issue["comments"].is_array() {return Err("TaskQ issue evidence incomplete".into());}
        if let Some(s)=merged_sha(&issue) {
            if ancestor_until(&c,&s,&target,end)? && !ancestor_until(&c,&s,&live,end)? {
                let u=issue["url"].as_str().filter(|u|crate::taskq::valid_url(u)).ok_or("Task URL unavailable")?;links.push(u.to_string());
            }
        }
    }
    if Instant::now()>=end {return Err("Preview deadline reached".into());}
    let observed=dev(&j.lock().unwrap(),p,Some(&target));
    if observed["state"]=="unknown" {return Err("DEV state unknown; preview refused".into());}
    let text=format!("Release {p}\nTarget SHA: {target}\nCommits since LIVE: {count}\nAuthors:\n{authors}\nTasks: {}\nMigrations: {}\nDEV: {} (serves target exactly: {})",if links.is_empty(){"none reported by TaskQ".into()}else{links.join("\n")},if migrations {"yes"}else{"no"},observed["state"].as_str().unwrap_or("unknown"),observed["serves_target_exactly"]);
    if text.len()>3500 {return Err("Release preview too large; narrow the candidate".into());}
    if !active() || Instant::now()>=end {return Err("Preview cancelled or deadline reached".into());}
    let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;
    authorize(&j,user,p)?;
    if get(&j,&format!("release-preview:{id}"))?.is_some() {return Err("This request already has an immutable release preview".into());}
    if config(&j,p)?!=c {return Err("Configuration changed during preview".into());}
    put(&j,&format!("release-preview:{id}"),&json!({"project":p,"owner":user,"session":session,"generation":generation,"config":c,"sha":target,"live":live,"migrations":migrations,"created":crate::now(),"state":"preview","text":text}))?;
    put(&j,&format!("release-latest:{session}"),&json!(id))?;
    audit(&j,id,p,"preview",&json!({"sha":target,"dry_run":"success"}))?;card(&j,id,p,&text)?;
    tx.commit().map_err(err)?;
    Ok(format!("{text}\nPress Release to authorize this SHA. The Release button is delivered to the owner conversation."))
}
pub fn buttons(j:&Journal,delivery:i64)->Result<Vec<(String,String)>,String> {
    let id:Option<i64>=j.0.query_row("SELECT request_id FROM outbox WHERE id=?1 AND kind='release_preview'",[delivery],|r|r.get(0)).optional().map_err(err)?;
    let Some(id)=id else {return Ok(vec![])};
    let Some(v)=get(j,&format!("release-preview:{id}"))? else {return Ok(vec![])};
    if v["state"]!="preview" || crate::now().saturating_sub(v["created"].as_u64().unwrap_or(0))>900 || get(j,&format!("release-latest:{}",v["session"].as_str().unwrap_or("")))?!=Some(json!(id)) {return Ok(vec![]);}
    Ok(vec![(format!("release:{id}"),"Release".into())])
}
fn validate(j:&Journal,r:&crate::builtins::Request,id:i64)->Result<Value,String> {
    let v=get(j,&format!("release-preview:{id}"))?.ok_or("Release button is stale")?;
    let p=v["project"].as_str().ok_or("Release preview invalid")?;authorize(j,r.sender,p)?;
    let generation:i64=j.0.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&r.session],|x|x.get(0)).map_err(err)?;
    if v["state"]!="preview" || v["owner"]!=r.sender || v["session"]!=r.session || v["generation"]!=generation || get(j,&format!("release-latest:{}",r.session))?!=Some(json!(id)) || crate::now().saturating_sub(v["created"].as_u64().unwrap_or(0))>900 || config(j,p)?!=v["config"] {return Err("Release button is stale or belongs to another person".into());}
    let front:String=j.0.query_row("SELECT front FROM requests WHERE id=?1",[r.id],|x|x.get(0)).map_err(err)?;
    if front!="telegram" {return Err("Use the Release button in Telegram; text cannot authorize release".into());}
    let callback:bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM meta WHERE key=?1)",[format!("callback:{}",r.id)],|x|x.get(0)).map_err(err)?;
    if !callback {return Err("Use the Release button; text cannot authorize release".into());}
    {
        let matched:bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM outbox o JOIN requests q ON q.id=?2 WHERE o.request_id=?1 AND o.kind='release_preview' AND o.state='delivered' AND o.sent_message=q.message AND o.chat=q.chat AND o.topic IS q.topic)",params![id,r.id],|x|x.get(0)).map_err(err)?;
        if !matched {return Err("Release button source does not match".into());}
    } Ok(v)
}
fn confirm(tq:&Taskq,j:&Shared,r:&crate::builtins::Request,id:i64)->Result<String,String> {
    let v={let j=j.lock().unwrap();validate(&j,r,id)?};
    let c=&v["config"];let p=v["project"].as_str().unwrap();let target=v["sha"].as_str().ok_or("Pinned SHA unavailable")?;
    if !sha(target) {return Err("Pinned SHA invalid".into());}
    // ponytail: one installation-wide release lock; per-repository locks only if simultaneous releases are needed.
    let lock_path={let j=j.lock().unwrap();Path::new(j.0.path().ok_or("Journal path unavailable")?).with_file_name("release.lock")};
    let _lock=coreq::file_lock::try_lock(&lock_path).map_err(|_|"Another release is active")?;
    let (bound,board)={let j=j.lock().unwrap();tq.registered(&j,p)?};
    read(&bound,&board,&["status","--json"],Instant::now()+Duration::from_secs(15))?;
    {let j=j.lock().unwrap();blockers(tq,&j,p,c)?;}
    if resolve(c,c["live_ref"].as_str().unwrap())?!=v["live"] {return Err("LIVE changed; request a fresh preview".into());}
    resolve(c,target)?;
    {
        let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;let mut current=validate(&j,r,id)?;
        let draining:bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM meta WHERE key='draining' AND value=1)",[],|x|x.get(0)).map_err(err)?;
        if draining {return Err("Service is draining; release refused".into());}
        current["state"]=json!("intent");current["confirmation"]=json!(r.id);
        put(&j,&format!("release-preview:{id}"),&current)?;audit(&j,r.id,p,"intent",&current)?;
        result_card(&j,r.id,id,&format!("{p}: release started for {target}."))?;
        let callback:Vec<u8>=j.0.query_row("SELECT CAST(value AS BLOB) FROM meta WHERE key=?1",[format!("callback:{}",r.id)],|x|x.get(0)).map_err(err)?;
        let callback=String::from_utf8(callback).map_err(|_|"Invalid callback")?;
        let (chat,topic):(i64,Option<i64>)=j.0.query_row("SELECT chat,topic FROM requests WHERE id=?1",[r.id],|x|Ok((x.get(0)?,x.get(1)?))).map_err(err)?;
        crate::journal::outbox(&j.0,r.id,chat,topic,None,None,"Release started.",crate::now() as i64)?;
        j.0.execute("UPDATE outbox SET kind=?2 WHERE id=?1",params![j.0.last_insert_rowid(),format!("toast:{callback}")]).map_err(err)?;
        tx.commit().map_err(err)?;
    }
    let end=Instant::now()+Duration::from_secs(900);
    let active=|| {let j=j.lock().unwrap();authorize(&j,r.sender,p).is_ok() && config(&j,p).is_ok_and(|x|x==*c)};
    let mut stages=vec![];let mut uncertain=false;
    for field in ["preflight","backup","release","pipeline"] {
        if matches!(field,"preflight"|"backup") && v["migrations"]!=true {continue;}
        audit(&j.lock().unwrap(),r.id,p,&format!("{field}_intent"),&json!({"sha":target}))?;
        let output=run(c,field,target,end,&active);
        // Never put command output or credentials in the journal/model. Structured readback is validated below.
        let receipt=json!({"stage":field,"status":output["status"],"exit_code":output["exit_code"],"sha":target});
        audit(&j.lock().unwrap(),r.id,p,field,&receipt)?;
        stages.push(format!("{field}: {}",output["status"].as_str().unwrap_or("unknown")));
        if output["status"]!="success" {uncertain=true;break;}
        if field=="pipeline" {
            let report:Value=output["stdout"].as_str().and_then(|s|serde_json::from_str(s).ok()).unwrap_or(Value::Null);
            let valid=report["sha"]==target && report["live_sha"]==target && ["release","smoke","backmerge"].iter().all(|s|report["stages"][s]["state"]=="success" && report["stages"][s]["url"].as_str().is_some_and(crate::taskq::valid_url));
            if valid {
                let mut verified=json!({"sha":target,"live_sha":target,"stages":{}});
                for s in ["release","smoke","backmerge"] {
                    let link=crate::keys::Keys(crate::tool().install.join("keys.json")).scrub(report["stages"][s]["url"].as_str().unwrap());
                    stages.push(format!("{s}: success {link}"));verified["stages"][s]=json!({"state":"success","url":link});
                }
                stages.push(format!("LIVE SHA: {target}"));audit(&j.lock().unwrap(),r.id,p,"verified",&verified)?;
            }
            else {stages.push("Release/smoke/backmerge or LIVE SHA unconfirmed".into());uncertain=true;}
        }
    }
    if uncertain {
        if !stages.iter().any(|s|s.starts_with("release:")) {stages.push("release: not run".into());}
        stages.extend(["smoke: unconfirmed".into(),"backmerge: unconfirmed".into(),"LIVE SHA: unknown".into()]);
    }
    let message=format!("{p}: {}\n{}",if uncertain {"release outcome uncertain; no automatic retry"}else{"release verified"},stages.join("\n"));
    let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;
    let mut saved=get(&j,&format!("release-preview:{id}"))?.ok_or("Release intent unavailable")?;saved["state"]=json!(if uncertain {"uncertain"}else{"done"});saved["result"]=json!(message);
    put(&j,&format!("release-preview:{id}"),&saved)?;result_card(&j,r.id,id,&message)?;tx.commit().map_err(err)?;Ok(message)
}
pub fn control(t:&coreq::Tool,j:&Shared,r:&crate::builtins::Request)->Result<String,String> {
    let cfg=crate::config(t)?.ok_or("Configuration unavailable")?;let tq=cfg.taskq.as_ref().ok_or("TaskQ unavailable")?;
    let args=r.text.split_whitespace().collect::<Vec<_>>();if args.len()!=2 {return Err("Use /release NAME or /release-confirm PREVIEW".into());}
    if args[0].split('@').next()==Some("/release") {
        let generation=j.lock().unwrap().0.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&r.session],|x|x.get(0)).map_err(err)?;
        preview_for(tq,j,r.id,&r.session,generation,r.sender,args[1],Instant::now()+Duration::from_secs(45),||j.lock().unwrap().builtin_allowed(r).unwrap_or(false))
    } else {confirm(tq,j,r,args[1].parse().map_err(|_|"Invalid Release button")?)}
}
pub fn recover(j:&Journal)->Result<(),String> {
    let mut q=j.0.prepare("SELECT key,CAST(value AS TEXT) FROM meta WHERE key LIKE 'release-preview:%'").map_err(err)?;
    let rows=q.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).and_then(Iterator::collect::<Result<Vec<_>,_>>).map_err(err)?;
    for (key,s) in rows {let mut v:Value=serde_json::from_str(&s).map_err(|_|"Release record unreadable")?;
        if v["state"]=="intent" {let tx=j.0.unchecked_transaction().map_err(err)?;v["state"]=json!("uncertain");put(j,&key,&v)?;
            if let Some(id)=v["confirmation"].as_i64() {let (chat,topic):(i64,Option<i64>)=j.0.query_row("SELECT chat,topic FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)?;
                let _=(chat,topic);
                let preview=key.strip_prefix("release-preview:").and_then(|s|s.parse().ok()).ok_or("Invalid preview identity")?;
                result_card(j,id,preview,"Release interrupted; outcome uncertain. It will not be repeated automatically.")?;}
            tx.commit().map_err(err)?;}
    } Ok(())
}
pub fn active(j:&Journal)->Result<i64,String> {j.0.query_row("SELECT count(*) FROM meta WHERE key LIKE 'release-preview:%' AND json_extract(CAST(value AS TEXT),'$.state')='intent'",[],|r|r.get(0)).map_err(err)}

#[cfg(all(test,unix))]
mod tests {
    use super::*;
    use std::{sync::{Arc,Mutex},os::unix::fs::PermissionsExt};
    struct Fixture { dir:std::path::PathBuf,j:Shared,tq:Taskq,c:Value,a:String,b:String,r:Run }
    impl Drop for Fixture {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.dir);}}
    fn fixture(tag:&str)->Fixture {
        let dir=std::env::temp_dir().join(format!("pilotq-release-{tag}-{}",std::process::id()));let _=std::fs::remove_dir_all(&dir);std::fs::create_dir_all(&dir).unwrap();
        let cmd=|args:&[&str]| {let o=std::process::Command::new("/usr/bin/git").current_dir(&dir).args(args).output().unwrap();assert!(o.status.success(),"{:?}: {}",args,String::from_utf8_lossy(&o.stderr));String::from_utf8_lossy(&o.stdout).trim().to_string()};
        cmd(&["init","-q"]);cmd(&["config","user.name","Invented tester"]);cmd(&["config","user.email","invented@example.invalid"]);cmd(&["remote","add","origin","https://example.invalid/project.git"]);
        std::fs::write(dir.join("a"),"a").unwrap();cmd(&["add","a"]);cmd(&["commit","-qm","initial"]);let a=cmd(&["rev-parse","HEAD"]);cmd(&["update-ref","refs/heads/live",&a]);
        std::fs::write(dir.join("a"),"b").unwrap();cmd(&["commit","-qam","next"]);let b=cmd(&["rev-parse","HEAD"]);cmd(&["update-ref","refs/heads/main",&b]);
        std::fs::write(dir.join("taskq.json"),r#"{"repo":"invented/project"}"#).unwrap();
        let script=dir.join("runner");std::fs::write(&script,"#!/bin/sh\nprintf '%s\\n' \"$@\" >> calls\n").unwrap();std::fs::set_permissions(&script,std::fs::Permissions::from_mode(0o700)).unwrap();
        let probe=dir.join("taskq");std::fs::write(&probe,"#!/bin/sh\ncat snapshot.json\n").unwrap();std::fs::set_permissions(&probe,std::fs::Permissions::from_mode(0o700)).unwrap();std::fs::write(dir.join("snapshot.json"),r#"{"complete":true,"board":{"repo":"invented/project"},"tasks":[]}"#).unwrap();
        let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();j.bootstrap(7,0).unwrap();j.add_project(7,"sample","invented/project",dir.to_str().unwrap(),0).unwrap();
        let r=j.ask(7,"release sample",crate::now() as i64).unwrap();
        let c=json!({"host":"invented-local","git":"/usr/bin/git","dev_health_url":"https://example.invalid/__health","main_ref":"refs/heads/main","live_ref":"refs/heads/live","migration_paths":["migrations/"],"dry_run":[script,"dry"],"release":[script,"release","{sha}"],"pipeline":[script,"pipeline","{sha}"],"preflight":[script,"preflight"],"backup":[script,"backup"]});
        let mut c=c;c["requirements"]=json!({"/usr/bin/git":"2.25.0",script.to_str().unwrap():"1.0.0"});
        let c=configure(&j,7,"sample",c).unwrap();
        let tq=Taskq{bot:None,bin:probe,root:dir.clone(),poll:Duration::from_secs(1)};
        Fixture{dir,j:Arc::new(Mutex::new(j)),tq,c,a,b,r}
    }
    fn save_preview(f:&Fixture)->i64 {
        let id=f.r.request;let j=f.j.lock().unwrap();
        j.0.execute("UPDATE requests SET front='telegram',message=99 WHERE id=?1",[id]).unwrap();
        j.0.execute("INSERT INTO meta(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![format!("callback:{id}"),b"invented".as_slice()]).unwrap();
        j.0.execute("INSERT INTO outbox(request_id,chat,topic,text,next_at,kind,state,sent_message) SELECT ?1,7,NULL,'invented preview',0,'release_preview','delivered',99 WHERE NOT EXISTS(SELECT 1 FROM outbox WHERE request_id=?1 AND kind='release_preview')",[id]).unwrap();
        put(&j,&format!("release-preview:{id}"),&json!({"project":"sample","owner":7,"session":f.r.session,"generation":f.r.generation,"config":f.c,"sha":f.b,"live":f.a,"created":crate::now(),"state":"preview","migrations":true})).unwrap();put(&j,&format!("release-latest:{}",f.r.session),&json!(id)).unwrap();id
    }
    fn request(f:&Fixture,user:i64)->crate::builtins::Request {crate::builtins::Request{id:f.r.request,session:f.r.session.clone(),sender:user,text:"/release-confirm 1".into()}}
    // A2 / R8: a merged subscription remains visible after leaving the open-task snapshot.
    #[test]
    fn closed_subscription_dev_projection_174() {
        let f=fixture("subscription");let j=f.j.lock().unwrap();
        j.0.execute("INSERT INTO tasks(number,request_id,chat,state,revision,observed,closed,project) VALUES (42,?1,7,'closed',1,10,1,'sample')",[f.r.request]).unwrap();
        assert_eq!(project_numbers(&j,"sample",&json!([{ "n":43 }])).unwrap().into_iter().collect::<Vec<_>>(),vec![42,43]);
    }
    // A2 / R8: inclusion must distinguish ancestry, not-equal descendants, missing commits and untrusted prose.
    #[test]
    fn dev_ancestry_and_trusted_merge_174() {
        let f=fixture("ancestry");assert!(ancestor(&f.c,&f.a,&f.b).unwrap());assert!(!ancestor(&f.c,&f.b,&f.a).unwrap());assert!(ancestor(&f.c,&"a".repeat(40),&f.b).is_err());
        let j=f.j.lock().unwrap();let observed=dev_read(&j,"sample",&f.c,Some(&f.a),&json!({"sha":f.b,"deployed_at":"2026-10-30T10:00:00Z"})).unwrap();assert_eq!(observed["state"],"on DEV since 2026-10-30T10:00:00Z");assert_eq!(dev_read(&j,"sample",&f.c,Some(&f.b),&json!({"sha":f.a})).unwrap()["state"],"not yet on DEV");assert!(dev_read(&j,"sample",&f.c,None,&json!({"sha":f.b})).is_err());
        let body=format!("taskq: close\nChanged: sample\n\ndelivery auto-merge r0: merged #1 as {}; merge-result CI green",f.b);
        assert_eq!(merged_sha(&json!({"comments":[{"trusted":true,"body":body}]})),Some(f.b.clone()));assert_eq!(merged_sha(&json!({"comments":[{"trusted":false,"body":body}]})),None);
    }
    // A3/A5/A7: no implicit grant; single immutable SHA, changed LIVE and repeated/stale/foreign/reset confirmations refuse.
    #[test]
    fn release_grant_stale_pinned_and_recovery_174() {
        let f=fixture("pinned");let id=save_preview(&f);let r=request(&f,7);
        assert!(confirm(&f.tq,&f.j,&r,id).unwrap_err().contains("grant"));assert!(!f.dir.join("calls").exists());
        f.j.lock().unwrap().grant(7,7,&["release:sample".into()],0).unwrap();
        assert!(validate(&f.j.lock().unwrap(),&request(&f,8),id).is_err());
        {let j=f.j.lock().unwrap();j.0.execute("DELETE FROM meta WHERE key='release-config:sample'",[]).unwrap();}
        assert!(confirm(&f.tq,&f.j,&r,id).unwrap_err().contains("configuration"));assert!(!f.dir.join("calls").exists());
        configure(&f.j.lock().unwrap(),7,"sample",f.c.clone()).unwrap();
        {let j=f.j.lock().unwrap();j.0.execute("DELETE FROM projects WHERE name='sample'",[]).unwrap();}
        assert!(confirm(&f.tq,&f.j,&r,id).unwrap_err().contains("registered"));assert!(!f.dir.join("calls").exists());
        f.j.lock().unwrap().add_project(7,"sample","invented/project",f.dir.to_str().unwrap(),0).unwrap();
        {let j=f.j.lock().unwrap();j.0.execute("DELETE FROM meta WHERE key=?1",[format!("callback:{}",r.id)]).unwrap();assert!(validate(&j,&r,id).unwrap_err().contains("button"));}save_preview(&f);
        {let j=f.j.lock().unwrap();let mut v=get(&j,&format!("release-preview:{id}")).unwrap().unwrap();v["generation"]=json!(f.r.generation+1);put(&j,&format!("release-preview:{id}"),&v).unwrap();}
        assert!(validate(&f.j.lock().unwrap(),&r,id).is_err());save_preview(&f);
        // Move main after preview; the argv must retain b. No live/network command is used.
        git(&f.c,&["update-ref","refs/heads/main",&f.a]).unwrap();
        let outcome=confirm(&f.tq,&f.j,&r,id).unwrap();assert!(outcome.contains("uncertain"));
        let calls=std::fs::read_to_string(f.dir.join("calls")).unwrap();assert_eq!(calls,format!("preflight\nbackup\nrelease\n{}\npipeline\n{}\n",f.b,f.b));
        assert!(confirm(&f.tq,&f.j,&r,id).is_err());assert_eq!(std::fs::read_to_string(f.dir.join("calls")).unwrap(),calls);
        {let j=f.j.lock().unwrap();let mut v=get(&j,&format!("release-preview:{id}")).unwrap().unwrap();v["state"]=json!("intent");v["confirmation"]=json!(r.id);put(&j,&format!("release-preview:{id}"),&v).unwrap();recover(&j).unwrap();assert_eq!(get(&j,&format!("release-preview:{id}")).unwrap().unwrap()["state"],"uncertain");}
        assert_eq!(std::fs::read_to_string(f.dir.join("calls")).unwrap(),calls);
    }
    // A7 / §4: failed prerequisites stop release; validated stage links are required for a successful receipt.
    #[test]
    fn release_stages_and_prerequisite_failure_174() {
        let mut f=fixture("stages");f.j.lock().unwrap().grant(7,7,&["release:sample".into()],0).unwrap();
        let mut bad=f.c.clone();bad["preflight"]=json!(["/bin/false"]);bad["requirements"]["/bin/false"]=json!("1.0.0");f.c=configure(&f.j.lock().unwrap(),7,"sample",bad).unwrap();let id=save_preview(&f);
        assert!(confirm(&f.tq,&f.j,&request(&f,7),id).unwrap().contains("uncertain"));assert!(!f.dir.join("calls").exists());
        let script=f.dir.join("runner");let report=json!({"sha":f.b,"live_sha":f.b,"stages":{"release":{"state":"success","url":"https://example.invalid/release"},"smoke":{"state":"success","url":"https://example.invalid/smoke"},"backmerge":{"state":"success","url":"https://example.invalid/backmerge"}}});
        std::fs::write(&script,format!("#!/bin/sh\nprintf '%s\\n' \"$@\" >> calls\nif [ \"$1\" = pipeline ]; then printf '%s' '{}'; fi\n",report)).unwrap();
        let mut good=f.c.clone();good["preflight"]=json!([script,"preflight"]);f.c=configure(&f.j.lock().unwrap(),7,"sample",good).unwrap();save_preview(&f);
        let result=confirm(&f.tq,&f.j,&request(&f,7),id).unwrap();assert!(result.contains("release verified") && result.contains(&format!("LIVE SHA: {}",f.b)),"{result}");
        assert!(result.contains("backmerge: success https://example.invalid/backmerge"));
    }
    // A6: other registered checkout of same repository must block before even preflight.
    #[test]
    fn release_running_checkout_blocker_174() {
        let f=fixture("blocker");let other=f.dir.join("other");std::fs::create_dir(&other).unwrap();
        let mut c=f.c.clone();c["root"]=json!(other);std::fs::write(other.join("taskq.json"),r#"{"repo":"invented/tight"}"#).unwrap();
        let status=std::process::Command::new("/usr/bin/git").current_dir(&other).args(["init","-q"]).status().unwrap();assert!(status.success());git(&c,&["remote","add","origin","https://example.invalid/project.git"]).unwrap();
        std::fs::write(other.join("snapshot.json"),r#"{"complete":true,"board":{"repo":"invented/tight"},"tasks":[{"n":42,"executions":[{"attempt":123,"state":"running"}]}]}"#).unwrap();
        f.j.lock().unwrap().add_project(7,"tight","invented/tight",other.to_str().unwrap(),0).unwrap();f.j.lock().unwrap().grant(7,7,&["release:sample".into()],0).unwrap();let id=save_preview(&f);
        let error=confirm(&f.tq,&f.j,&request(&f,7),id).unwrap_err();assert!(error.contains("tight task #42 attempt 123"),"{error}");assert!(!f.dir.join("calls").exists());
    }
}
