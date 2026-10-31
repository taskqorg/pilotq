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

/// TaskQ's trusted terminal delivery receipt proves a merge identity; hand merges use `merge_identity`.
pub fn merged_sha(issue:&Value)->Option<String> {
    if let Some(s)=issue["merged_sha"].as_str().filter(|s|sha(s)) {return Some(s.into());}
    issue["comments"].as_array()?.iter().rev().filter(|c|c["trusted"]==true)
        .filter_map(|c|c["body"].as_str()).filter(|b|b.starts_with("taskq: close\n"))
        .flat_map(|b|b.lines().rev()).find_map(|line| {
            if !line.starts_with("delivery ") || !line.contains("; merge-result CI green") {return None;}
            let (_,part)=line.split_once(" as ")?;let s=part.split(';').next()?;sha(s).then(||s.into())
        })
}

// #183 / §5: a GitLab hand merge writes no receipt; Git proves which main commit merged the trusted result head.
fn merge_identity(c:&Value,issue:&Value,end:Instant)->Option<String> {
    if let Some(s)=merged_sha(issue) {return Some(s);}
    let result=issue["comments"].as_array()?.iter().rev().filter(|c|c["trusted"]==true)
        .filter_map(|c|c["body"].as_str()?.strip_prefix("taskq: result ")?.split(' ').next()).find(|s|sha(s))?;
    let main=resolve_until(c,c["main_ref"].as_str()?,end).ok()?;
    let list=|args:&[&str]| {
        let out=crate::commands::execute_bounded(crate::commands::command(c["git"].as_str()?,c["root"].as_str()?,args),end,||true,1024*1024);
        (out["status"]=="success").then(||out["stdout"].as_str().unwrap_or("").lines().map(String::from).collect::<Vec<_>>())
    };
    let range=format!("{result}..{main}");
    // ponytail: lists every main commit since the merge base (1 MiB cap); bisect if histories outgrow it.
    let descendants:std::collections::HashSet<String>=list(&["rev-list","--ancestry-path",&range])?.into_iter().collect();
    let merge=list(&["rev-list","--first-parent",&range])?.into_iter().rev().find(|s|descendants.contains(s))?;
    // Fast-forward merges leave the result head itself on the main first-parent chain.
    let parent=resolve_until(c,&format!("{merge}^1"),end).ok()?;
    Some(if parent==result {result.into()} else {merge})
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
    if !c["delivery_status"].is_null() {
        if !argv(&c["delivery_status"] )?.contains(&"{sha}".into()) {return Err("delivery_status requires a separate {sha} argument".into());}
        let bin=argv(&c["delivery_status"])?[0].clone();
        if c["requirements"][&bin].as_str().and_then(crate::host::version).is_none() {return Err("Delivery executable requires a minimum version".into());}
    }
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
#[cfg(test)]
fn ancestor(c:&Value,a:&str,b:&str)->Result<bool,String> {ancestor_until(c,a,b,Instant::now()+Duration::from_secs(5))}
fn ancestor_until(c:&Value,a:&str,b:&str,end:Instant)->Result<bool,String> {
    if !sha(a)||!sha(b) {return Err("Merged/deployed SHA unavailable".into());}
    // Distinguish a negative ancestry result from missing commits and Git errors.
    resolve_until(c,a,end)?;resolve_until(c,b,end)?;
    let out=crate::commands::execute_bounded(crate::commands::command(c["git"].as_str().ok_or("Git unavailable")?,c["root"].as_str().ok_or("Root unavailable")?,&["merge-base","--is-ancestor",a,b]),end,||true,1024);
    match (out["status"].as_str(),out["exit_code"].as_i64()) {(Some("success"),Some(0))=>Ok(true),(Some("exit_nonzero"),Some(1))=>Ok(false),_=>Err("DEV ancestry unknown".into())}
}
fn health_until(c:&Value,end:Instant)->Result<Value,String> {
    let remaining=end.saturating_duration_since(Instant::now()).min(Duration::from_secs(10));
    if remaining.is_zero() {return Err("DEV health deadline exhausted".into());}
    let url=c["dev_health_url"].as_str().ok_or("DEV URL missing")?.to_string();
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|"DEV runtime unavailable")?.block_on(async {
            let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(remaining).build().map_err(|_|"DEV client unavailable")?;
            let mut response=client.get(url).send().await.map_err(|_|"DEV health read failed")?.error_for_status().map_err(|_|"DEV health status failed")?;
            let mut bytes=Vec::new();while let Some(chunk)=response.chunk().await.map_err(|_|"DEV health read failed")? {if bytes.len()+chunk.len()>65536 {return Err("DEV health too large".into());}bytes.extend_from_slice(&chunk);}
            let v:Value=serde_json::from_slice(&bytes).map_err(|_|"DEV health unreadable")?;
            if !v["sha"].as_str().is_some_and(sha) {return Err("DEV health SHA unavailable".into());} Ok::<Value,String>(v)
        })
    }).join().map_err(|_|"DEV health unavailable")?.map_err(String::from)
}
// #179 / §5: pipeline evidence is independent of historical worker closing notes.
#[cfg(test)]
fn delivery_config(c:Result<Value,String>,issue:&Value,end:Instant)->Value {
    delivery_config_audit(c,issue,end,|_|Ok(()))
}
fn delivery_config_audit(c:Result<Value,String>,issue:&Value,end:Instant,mut diagnostic:impl FnMut(&Value)->Result<(),String>)->Value {
    let merged=c.as_ref().ok().and_then(|c|merge_identity(c,issue,end)).or_else(||merged_sha(issue));
    let mut stage=if issue["state"].as_str().is_some_and(|s|s.eq_ignore_ascii_case("closed")) {"closed"} else {"open"};
    if merged.is_some() {stage="merged";}
    let result=(|| -> Result<Value,String> {
        let c=c?;
        let target=merged.as_deref().ok_or("Merge identity unavailable")?;
        let output=run(&c,"delivery_status",target,end,||true);
        if output["status"]!="success" {
            diagnostic(&output).map_err(|_|"Delivery adapter failed; diagnostic audit unavailable")?;
            return Err("Delivery adapter failed; pipeline state unknown".into());
        }
        let report:Value=serde_json::from_str(output["stdout"].as_str().unwrap_or("")).map_err(|_|"Delivery adapter JSON unreadable")?;
        delivery_report(&c,target,report,end)
    })();
    match result {
        Ok(mut v)=>{if v["live_includes"]==true {stage="on LIVE";} else if v["first_main"]["deploy"]["state"]=="success" {
            stage=if v["first_main"]["smoke"].as_array().unwrap().last().is_some_and(|s|s["state"]=="success") {"DEV smoke"} else {"on DEV"};
        } v["stage"]=json!(stage);v},
        Err(e)=>json!({"stage":stage,"pipeline":"unknown","reason":e}),
    }
}
fn delivery_report(c:&Value,target:&str,report:Value,end:Instant)->Result<Value,String> {
    fn job(v:&Value)->Result<(),String> {
        if !matches!(v["state"].as_str(),Some("success"|"failed"|"running"|"pending"|"canceled"|"skipped")) {return Err("Delivery adapter job state missing or invalid".into());}
        let u=url::Url::parse(v["url"].as_str().unwrap_or("")).map_err(|_|"Delivery adapter job link missing or invalid")?;
        if u.scheme()!="https" || u.host_str().is_none() || !u.username().is_empty() || u.password().is_some() {return Err("Delivery adapter job link invalid".into());} Ok(())
    }
    if report["sha"]!=target || !report["merged_at"].as_str().is_some_and(|s|!s.is_empty() && s.len()<100) {return Err("Delivery adapter merge identity/time missing or mismatched".into());}
    let first=&report["first_main"];job(first)?;job(&first["deploy"])?;
    if !ancestor_until(c,target,first["sha"].as_str().unwrap_or(""),end)? {return Err("First main pipeline excludes merge".into());}
    let smoke=first["smoke"].as_array().filter(|s|!s.is_empty()).ok_or("Delivery adapter smoke attempts missing")?;
    for attempt in smoke {job(attempt)?;if attempt["state"]=="failed" && !attempt["first_failing_check"].as_str().is_some_and(|s|!s.is_empty() && s.len()<=1000) {return Err("Delivery adapter first failing check missing".into());}}
    let later=&report["later_main"];
    if !report.as_object().is_some_and(|o|o.contains_key("later_main")) {return Err("Delivery adapter later main evidence missing".into());}
    if !later.is_null() {job(later)?;if !ancestor_until(c,target,later["sha"].as_str().unwrap_or(""),end)? {return Err("Later main pipeline excludes merge".into());}}
    let mut result=report;
    let live=(|| -> Result<(String,bool),String> {
        let live=resolve_until(c,c["live_ref"].as_str().ok_or("LIVE ref missing")?,end)?;
        let included=ancestor_until(c,target,&live,end)?;Ok((live,included))
    })();
    match live {Ok((live,included))=>{result["live_includes"]=json!(included);result["live_sha"]=json!(live);},Err(e)=>{result["live_includes"]=Value::Null;result["live_reason"]=json!(e);}}
    Ok(result)
}
pub fn delivery_text(v:&Value)->String {
    let mut text=format!("Delivery stage: {}.",v["stage"].as_str().unwrap_or("unknown"));
    if let Some(reason)=v["reason"].as_str() {return format!("{text} Pipeline unknown: {reason}.");}
    text+=&format!("\nMerged at {}.",v["merged_at"].as_str().unwrap_or("unknown"));
    let first=&v["first_main"];
    text+=&format!("\nFirst containing main pipeline: {} ({}).",first["state"].as_str().unwrap_or("unknown"),first["url"].as_str().unwrap_or(""));
    text+=&format!("\nDEV deploy: {} ({}).",first["deploy"]["state"].as_str().unwrap_or("unknown"),first["deploy"]["url"].as_str().unwrap_or(""));
    for (i,job) in first["smoke"].as_array().into_iter().flatten().enumerate() {
        text+=&format!("\nDEV smoke attempt {}: {} ({})",i+1,job["state"].as_str().unwrap_or("unknown"),job["url"].as_str().unwrap_or(""));
        if let Some(check)=job["first_failing_check"].as_str().filter(|_|job["state"]=="failed") {text+=&format!("; first failing check: {check}");}text.push('.');
    }
    let later=&v["later_main"];
    if later.is_null() {text+="\nNo later containing main pipeline observed.";} else {
        text+=&format!("\nLater containing main pipeline: {} ({}).",later["state"].as_str().unwrap_or("unknown"),later["url"].as_str().unwrap_or(""));
    }
    text+=match v["live_includes"].as_bool() {Some(true)=>"\nLIVE contains the merge.",Some(false)=>"\nLIVE is behind the merge.",None=>"\nLIVE inclusion unknown."};
    if let Some(reason)=v["live_reason"].as_str() {text+=&format!(" {reason}");}text
}
fn project_numbers(j:&Journal,p:&str,tasks:&Value)->Result<std::collections::BTreeSet<i64>,String> {
    let mut numbers=std::collections::BTreeSet::new();for t in tasks.as_array().ok_or("Task list unavailable")? {if let Some(n)=t["n"].as_i64(){numbers.insert(n);}}
    let mut q=j.0.prepare("SELECT number FROM tasks WHERE project=?1 ORDER BY observed DESC LIMIT 12").map_err(err)?;
    let cached=q.query_map([p],|r|r.get::<_,i64>(0)).and_then(Iterator::collect::<Result<Vec<_>,_>>).map_err(err)?;numbers.extend(cached);Ok(numbers)
}
// Snapshot protected configuration under the mutex; all network/process reads run unlocked.
pub fn delivery(j:&std::sync::Mutex<Journal>,request:i64,p:&str,issue:&Value,end:Instant)->Value {
    let c=config(&j.lock().unwrap(),p);
    delivery_config_audit(c,issue,end,|output| {
        delivery_diagnostic(&j.lock().unwrap(),request,p,output,&crate::keys::Keys(crate::tool().install.join("keys.json")))
    })
}
fn delivery_diagnostic(j:&Journal,request:i64,p:&str,output:&Value,keys:&crate::keys::Keys)->Result<(),String> {
    let scrubbed=keys.scrub(&output.to_string());
    audit(j,request,p,"delivery_adapter_error",&json!({"diagnostic":scrubbed}))
}
fn dev_observe(j:&std::sync::Mutex<Journal>,p:&str,c:&Value,merged:Option<&str>,h:&Value,end:Instant)->Result<Value,String> {
    let a=merged.ok_or("Merged SHA unavailable")?;
    let b=h["sha"].as_str().ok_or("DEV SHA unavailable")?;
    let included=ancestor_until(c,a,b,end)?;
    if !included {return Ok(json!({"state":"not yet on DEV","sha":b,"serves_target_exactly":a==b,"observed_at":crate::now()}));}
    let j=j.lock().unwrap();
    if config(&j,p)?!=*c {return Err("Release configuration changed during observation".into());}
    let key=format!("dev-first:{p}:{a}");
    let first=get(&j,&key)?.unwrap_or(json!(crate::now()));put(&j,&key,&first)?;
    let time=h["deployed_at"].as_str().or_else(||h["started_at"].as_str()).filter(|s|!s.is_empty() && s.len()<100);
    Ok(json!({"state":match time {Some(t)=>format!("on DEV since {t}"),None=>format!("on DEV since {} (first observed)",first)},"sha":b,"serves_target_exactly":a==b,"observed_at":crate::now()}))
}
pub fn task_observations(j:&std::sync::Mutex<Journal>,request:i64,p:&str,issue:&Value,end:Instant)->(Value,Value) {
    std::thread::scope(|scope| {
        let pipeline=scope.spawn(||delivery(j,request,p,issue,end));
        let dev=task_dev(j,p,issue,end);
        (dev,pipeline.join().unwrap_or_else(|_|json!({"stage":"unknown","reason":"Delivery reader failed"})))
    })
}
pub fn task_dev(j:&std::sync::Mutex<Journal>,p:&str,issue:&Value,end:Instant)->Value {
    let result=(|| {let c=config(&j.lock().unwrap(),p)?;let h=health_until(&c,end)?;
        dev_observe(j,p,&c,merge_identity(&c,issue,end).as_deref(),&h,end)})();
    result.unwrap_or_else(|e:String|json!({"state":"unknown","reason":e,"observed_at":crate::now()}))
}
pub fn project_status_dev(tq:&Taskq,j:&std::sync::Mutex<Journal>,p:&str,tasks:&Value,end:Instant)->Value {
    let result=(|| -> Result<Value,String> {
        let (c,board,numbers)={let j=j.lock().unwrap();let c=config(&j,p)?;let (board,_)=binding(&j,p)?;
            (c,board,project_numbers(&j,p,tasks)?)};
        if numbers.len()>20 {return Err("Too many tasks for a bounded DEV observation".into());}
        let h=health_until(&c,end)?;
        let main=resolve_until(&c,c["main_ref"].as_str().ok_or("Main ref missing")?,end)?;
        let main=dev_observe(j,p,&c,Some(&main),&h,end)?;
        let mut projected=vec![];
        for n in numbers {let issue=read(tq,&board,&["issue",&n.to_string(),"--json"],end);
            let merged=issue.ok().filter(|i|i["number"]==n).as_ref().and_then(|i|merge_identity(&c,i,end));
            let dev=dev_observe(j,p,&c,merged.as_deref(),&h,end).unwrap_or_else(|e|json!({"state":"unknown","reason":e}));
            projected.push(json!({"task":n,"dev":dev}));}
        Ok(json!({"main":main,"tasks":projected}))
    })();result.unwrap_or_else(|e|json!({"state":"unknown","reason":e}))
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
                None=>return Err(format!("Release blocked by {name} task #{}: running attempt evidence unavailable",task["n"])),
            };
            if attempts.is_empty() && !matches!(task["state"].as_str(),Some("held"|"ready"|"later"|"waiting"|"ask"|"failed")) {
                return Err(format!("Release blocked by {name} task #{}: running attempt evidence unavailable",task["n"]));
            }
            for attempt in attempts {
                if attempt["host"].as_str().is_some_and(|h|!h.is_empty() && h!=host) {continue;}
                match attempt["state"].as_str() {
                    Some("exited"|"released")=>{},
                    Some("running"|"doing")=>return Err(format!("Release blocked by {name} task #{} attempt {}",task["n"],attempt["attempt"])),
                    _=>return Err(format!("Release blocked by {name} task #{} attempt {}: execution state unknown",task["n"],attempt["attempt"])),
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
        if let Some(s)=merge_identity(&c,&issue,end) {
            if ancestor_until(&c,&s,&target,end)? && !ancestor_until(&c,&s,&live,end)? {
                let u=issue["url"].as_str().filter(|u|crate::taskq::valid_url(u)).ok_or("Task URL unavailable")?;links.push(u.to_string());
            }
        }
    }
    if Instant::now()>=end {return Err("Preview deadline reached".into());}
    let observed=task_dev(j,p,&json!({"merged_sha":target}),end);
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
        if field=="pipeline" {
            let report:Value=output["stdout"].as_str().and_then(|s|serde_json::from_str(s).ok()).unwrap_or(Value::Null);
            let mut verified=json!({"sha":target,"stages":{}});
            let bound=report["sha"]==target;
            for stage in ["release","smoke","backmerge"] {
                let state=report["stages"][stage]["state"].as_str().filter(|s|matches!(*s,"success"|"failed"|"pending"|"running"|"cancelled"|"skipped"));
                let link=report["stages"][stage]["url"].as_str().filter(|s|crate::taskq::valid_url(s));
                if let (true,Some(state))=(bound,state) {
                    verified["stages"][stage]=json!({"state":state});
                    if let Some(link)=link {
                        let link=crate::keys::Keys(crate::tool().install.join("keys.json")).scrub(link);
                        stages.push(format!("{stage}: {state} {link}"));verified["stages"][stage]["url"]=json!(link);
                    } else {stages.push(format!("{stage}: {state} (link unavailable)"));uncertain=true;}
                    if state!="success" {uncertain=true;}
                } else {stages.push(format!("{stage}: unconfirmed"));uncertain=true;}
            }
            if let Some(live)=report["live_sha"].as_str().filter(|s|bound && sha(s)) {
                stages.push(format!("LIVE SHA: {live}"));verified["live_sha"]=json!(live);
                if live!=target {uncertain=true;}
            } else {stages.push("LIVE SHA: unknown".into());uncertain=true;}
            audit(&j.lock().unwrap(),r.id,p,"verified",&verified)?;
        }
        if output["status"]!="success" {uncertain=true;break;}
    }
    if !stages.iter().any(|s|s.starts_with("pipeline:")) {
        if !stages.iter().any(|s|s.starts_with("release:")) {stages.push("release: not run".into());}
        stages.extend(["smoke: unconfirmed".into(),"backmerge: unconfirmed".into(),"LIVE SHA: unknown".into()]);
    }
    let message=format!("{p}: {}\n{}",if uncertain {"release outcome uncertain; no automatic retry"}else{"release verified"},stages.join("\n"));
    let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;
    let mut saved=get(&j,&format!("release-preview:{id}"))?.ok_or("Release intent unavailable")?;saved["state"]=json!(if uncertain {"uncertain"}else{"done"});saved["result"]=json!(message);
    put(&j,&format!("release-preview:{id}"),&saved)?;result_card(&j,r.id,id,&message)?;
    j.0.execute("UPDATE requests SET state=?2,reply=?3,error=?4 WHERE id=?1 AND state IN ('queued','waiting')",params![r.id,if uncertain {"uncertain"}else{"answered"},message,if uncertain {Some("Release outcome uncertain; automatic replay blocked")}else{None}]).map_err(err)?;
    tx.commit().map_err(err)?;Ok(message)
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
                let message="Release interrupted; outcome uncertain. It will not be repeated automatically.";
                result_card(j,id,preview,message)?;
                j.0.execute("UPDATE requests SET state='uncertain',reply=?2,error=?2 WHERE id=?1 AND state IN ('queued','waiting','failed')",params![id,message]).map_err(err)?;}
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
    // #202 A5 / R16–R17: transport URLs and adapter stderr stay out of replies;
    // scrubbed diagnostics remain available in audit.
    #[test]
    fn release_errors_202() {
        let f=fixture("errors202");
        let mut c=f.c.clone();c["dev_health_url"]=json!("invented://example.invalid/private?token=invented-secret");
        assert_eq!(health_until(&c,Instant::now()+Duration::from_secs(2)).unwrap_err(),"DEV health read failed");
        let script=f.dir.join("runner");
        std::fs::write(&script,"#!/bin/sh\necho 'invented-secret https://example.invalid/private' >&2\nexit 1\n").unwrap();
        c["delivery_status"]=json!([script]);
        let keys=crate::keys::Keys(f.dir.join("keys.json"));
        std::fs::write(&keys.0,r#"{"test":{"value":"invented-secret"}}"#).unwrap();
        std::fs::set_permissions(&keys.0,std::fs::Permissions::from_mode(0o600)).unwrap();
        let j=f.j.lock().unwrap();
        let got=delivery_config_audit(Ok(c),&json!({"state":"closed","merged_sha":f.b}),Instant::now()+Duration::from_secs(2),
            |out|delivery_diagnostic(&j,f.r.request,"sample",out,&keys));
        assert_eq!(got["reason"],"Delivery adapter failed; pipeline state unknown");
        let saved:String=j.0.query_row("SELECT result FROM audit_events WHERE item='delivery_adapter_error'",[],|r|r.get(0)).unwrap();
        assert!(!saved.contains("invented-secret"));
        assert!(saved.contains("[key test]") && saved.contains("example.invalid/private"));
    }
    // #179 A2–A6 / R8: adapter facts outrank close notes; retain failed attempts and later recovery.
    #[test]
    fn delivery_trace_179() {
        let f=fixture("delivery179");
        let job=|state:&str,id:&str|json!({"state":state,"url":format!("https://example.invalid/jobs/{id}")});
        let mut failed=job("failed","smoke1");failed["first_failing_check"]=json!("homepage returns 503");
        let mut retry=job("failed","smoke2");retry["first_failing_check"]=json!("login returns 500");
        let report=json!({"sha":f.b,"merged_at":"2026-10-30T16:10:00Z","first_main":{"sha":f.b,"state":"failed","url":"https://example.invalid/pipelines/1","deploy":job("success","web_dev"),"smoke":[failed,retry]},"later_main":{"sha":f.b,"state":"success","url":"https://example.invalid/pipelines/2"}});
        let script=f.dir.join("runner");
        std::fs::write(&script,format!("#!/bin/sh\ncat <<'JSON'\n{report}\nJSON\n")).unwrap();
        let mut c=f.c.clone();c["delivery_status"]=json!([script,"{sha}"]);
        let j=f.j.lock().unwrap();configure(&j,7,"sample",c).unwrap();
        let issue=json!({"state":"closed","merged_sha":f.b,"body":"deployed after-check unknown; CI passed"});
        let got=delivery_config(config(&j,"sample"),&issue,Instant::now()+Duration::from_secs(10));
        assert_eq!(got["stage"],"on DEV");
        assert_eq!(got["merged_at"],"2026-10-30T16:10:00Z");
        assert_eq!(got["first_main"],report["first_main"]);
        assert_eq!(got["later_main"],report["later_main"]);
        assert_eq!(got["live_includes"],false);
        assert!(!got.to_string().contains("after-check"));
        let reply=delivery_text(&got);
        for fact in ["smoke1","smoke2","web_dev","2026-10-30T16:10:00Z","homepage returns 503","login returns 500","LIVE is behind"] {assert_eq!(reply.matches(fact).count(),1,"{reply}");}
        assert!(reply.contains("Later containing main pipeline: success"));
        assert!(reply.contains("First containing main pipeline: failed"));
        let slow_git=f.dir.join("slow-git");
        std::fs::write(&slow_git,"#!/bin/sh\nsleep 0.15\nexec /usr/bin/git \"$@\"\n").unwrap();
        std::fs::set_permissions(&slow_git,std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut slow=f.c.clone();slow["git"]=json!(slow_git);
        let start=Instant::now();
        assert!(delivery_report(&slow,&f.b,report.clone(),start+Duration::from_millis(200)).is_err());
        assert!(start.elapsed()<Duration::from_millis(600),"Git reads renewed the shared deadline");
        let mut bad=report.clone();bad["sha"]=json!(f.a);
        assert!(delivery_report(&f.c,&f.b,bad,Instant::now()+Duration::from_secs(10)).is_err());
        let mut bad=report.clone();bad["first_main"]["smoke"][0]["first_failing_check"]=Value::Null;
        assert!(delivery_report(&f.c,&f.b,bad,Instant::now()+Duration::from_secs(10)).is_err());
        let mut missing_live=f.c.clone();missing_live["live_ref"]=json!("refs/heads/missing");
        let partial=delivery_report(&missing_live,&f.b,report.clone(),Instant::now()+Duration::from_secs(10)).unwrap();
        assert_eq!(partial["first_main"],report["first_main"]);assert!(partial["live_includes"].is_null());
        assert!(partial["live_reason"].as_str().unwrap().contains("Git read failed"));
        let mut green=report.clone();green["first_main"]["smoke"][1]["state"]=json!("success");
        std::fs::write(&script,format!("#!/bin/sh\ncat <<'JSON'\n{green}\nJSON\n")).unwrap();
        assert_eq!(delivery_config(config(&j,"sample"),&issue,Instant::now()+Duration::from_secs(10))["stage"],"DEV smoke");
        let status=std::process::Command::new("/usr/bin/git").current_dir(&f.dir).args(["update-ref","refs/heads/live",&f.b]).status().unwrap();assert!(status.success());
        assert_eq!(delivery_config(config(&j,"sample"),&issue,Instant::now()+Duration::from_secs(10))["stage"],"on LIVE");
        std::fs::write(&script,"#!/bin/sh\nexit 1\n").unwrap();
        let got=delivery_config(config(&j,"sample"),&issue,Instant::now()+Duration::from_secs(10));assert_eq!(got["pipeline"],"unknown");assert_eq!(got["reason"],"Delivery adapter failed; pipeline state unknown");
    }
    // #183 A2 / §5: a task closed without the delivery line still reports pipeline facts for its hand-merged result.
    #[test]
    fn hand_merge_identity_183() {
        let f=fixture("handmerge183");
        let cmd=|args:&[&str]| {let o=std::process::Command::new("/usr/bin/git").current_dir(&f.dir).args(args).output().unwrap();assert!(o.status.success(),"{:?}: {}",args,String::from_utf8_lossy(&o.stderr));String::from_utf8_lossy(&o.stdout).trim().to_string()};
        cmd(&["checkout","-qb","taskq-541",&f.b]);std::fs::write(f.dir.join("r"),"r").unwrap();cmd(&["add","r"]);cmd(&["commit","-qm","result"]);let result=cmd(&["rev-parse","HEAD"]);
        cmd(&["checkout","-q","main"]);std::fs::write(f.dir.join("m"),"m").unwrap();cmd(&["add","m"]);cmd(&["commit","-qm","other"]);
        cmd(&["merge","-q","--no-ff","-m","Merge branch 'taskq-541' into 'main'","taskq-541"]);let merge=cmd(&["rev-parse","HEAD"]);
        std::fs::write(f.dir.join("m"),"later").unwrap();cmd(&["commit","-qam","later"]);let later=cmd(&["rev-parse","HEAD"]);
        let end=Instant::now()+Duration::from_secs(10);
        let issue=|sha:&str,trusted:bool|json!({"state":"closed","comments":[{"trusted":trusted,"body":format!("taskq: result {sha} attempt 1\nChanged: invented")},{"trusted":true,"body":"taskq: close\nChanged: invented\ndelivery review-pr r0: pending"}]});
        assert_eq!(merge_identity(&f.c,&issue(&result,true),end),Some(merge.clone()));
        assert_eq!(merge_identity(&f.c,&issue(&result,false),end),None);
        assert_eq!(merge_identity(&f.c,&issue(&f.b,true),end),Some(f.b.clone()),"fast-forward head is its own merge");
        cmd(&["checkout","-qb","unmerged",&f.b]);std::fs::write(f.dir.join("u"),"u").unwrap();cmd(&["add","u"]);cmd(&["commit","-qm","unmerged"]);let unmerged=cmd(&["rev-parse","HEAD"]);
        assert_eq!(merge_identity(&f.c,&issue(&unmerged,true),end),None);
        let job=|state:&str,id:&str|json!({"state":state,"url":format!("https://example.invalid/jobs/{id}")});
        let mut smoke=job("failed","smoke1");smoke["first_failing_check"]=json!("homepage returns 503");
        let report=json!({"merged_at":"2026-10-30T16:10:00Z","first_main":{"sha":merge,"state":"failed","url":"https://example.invalid/pipelines/1","deploy":job("success","web_dev"),"smoke":[smoke]},"later_main":{"sha":later,"state":"success","url":"https://example.invalid/pipelines/2"}});
        let script=f.dir.join("runner");
        std::fs::write(&script,format!("#!/bin/sh\nprintf '%s' '{}' | sed \"s/SHA/$1/\"\n",report.to_string().replacen('{',r#"{"sha":"SHA","#,1))).unwrap();
        let mut c=f.c.clone();c["delivery_status"]=json!([script,"{sha}"]);
        let j=f.j.lock().unwrap();configure(&j,7,"sample",c).unwrap();
        let got=delivery_config(config(&j,"sample"),&issue(&result,true),end);
        assert_eq!(got["sha"],merge.as_str(),"{got}");assert_eq!(got["stage"],"on DEV");assert_eq!(got["live_includes"],false);
        let reply=delivery_text(&got);
        for fact in ["2026-10-30T16:10:00Z","web_dev","homepage returns 503","Later containing main pipeline: success","LIVE is behind"] {assert!(reply.contains(fact),"{reply}");}
    }
    // §5/§7/§12: a slow pipeline cannot hold controls or renew Git read budgets.
    #[test]
    fn delivery_slow_adapter_179() {
        let f=fixture("slow179");
        let script=f.dir.join("runner");
        std::fs::write(&script,"#!/bin/sh\ntouch entered\nsleep 5\n").unwrap();
        let mut c=f.c.clone();c["delivery_status"]=json!([script,"{sha}"]);
        let c=configure(&f.j.lock().unwrap(),7,"sample",c).unwrap();
        let journal=f.j.clone();let issue=json!({"state":"closed","merged_sha":f.b});
        let start=Instant::now();let end=start+Duration::from_millis(400);
        let worker=std::thread::spawn(move ||delivery(&journal,1,"sample",&issue,end));
        while !f.dir.join("entered").exists() && Instant::now()<end {std::thread::yield_now();}
        assert!(f.dir.join("entered").exists(),"adapter did not start");
        let control=f.j.try_lock().expect("slow adapter blocked recovery controls");
        assert_eq!(control.owner().unwrap(),Some(7));drop(control);
        assert_eq!(worker.join().unwrap()["pipeline"],"unknown");
        assert!(start.elapsed()<Duration::from_secs(2),"adapter renewed deadline");
        let end=Instant::now()+Duration::from_secs(1);
        let h=json!({"sha":f.b,"deployed_at":"2026-10-30T16:10:00Z"});
        assert_eq!(dev_observe(&f.j,"sample",&c,Some(&f.b),&h,end).unwrap()["state"],"on DEV since 2026-10-30T16:10:00Z");
        let h=json!({"sha":f.b});
        let first=dev_observe(&f.j,"sample",&c,Some(&f.b),&h,end).unwrap();
        assert!(first["state"].as_str().unwrap().ends_with("(first observed)"));
        assert_eq!(dev_observe(&f.j,"sample",&c,Some(&f.b),&h,end).unwrap()["state"],first["state"]);
    }
    // #190 A3 / R8, #212 A5: explicit release-status reads retain a separate delivery budget.
    #[test]
    fn slow_task_status_190() {
        let f=fixture("slow190");
        let issue=json!({"number":42,"title":"Invented","url":"https://example.invalid/issues/42","state":"closed","merged_sha":f.b});
        std::fs::write(f.dir.join("issue.json"),issue.to_string()).unwrap();
        std::fs::write(&f.tq.bin,"#!/bin/sh\ncase \"$1\" in\n status) sleep 9; cat snapshot.json ;;\n issue) cat issue.json ;;\n *) exit 99 ;;\nesac\n").unwrap();
        let script=f.dir.join("runner");
        let report=json!({"sha":f.b,"merged_at":"2026-10-31T00:00:00Z","first_main":{"sha":f.b,"state":"success","url":"https://example.invalid/pipelines/1","deploy":{"state":"success","url":"https://example.invalid/jobs/1"},"smoke":[{"state":"pending","url":"https://example.invalid/jobs/2"}]},"later_main":null});
        std::fs::write(&script,format!("#!/bin/sh\nprintf '%s' '{}'\n",report)).unwrap();
        let mut c=f.c.clone();c["delivery_status"]=json!([script,"{sha}"]);
        configure(&f.j.lock().unwrap(),7,"sample",c).unwrap();
        let (ok,text)=crate::taskq::call(&f.tq,&f.j,&f.r,"task_status",&json!({"project":"sample","task":"42","include_release":true}));
        assert!(ok,"{text}");
        assert!(text.contains("on DEV"),"slow board reads lost DEV evidence: {text}");
        assert!(!text.contains("deadline exhausted"),"board reads consumed observation budget: {text}");
    }
    fn save_preview(f:&Fixture)->i64 {
        let id=f.r.request;let j=f.j.lock().unwrap();
        j.0.execute("UPDATE requests SET front='telegram',message=99,state='waiting',text='/release-confirm 1' WHERE id=?1",[id]).unwrap();
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
        let end=Instant::now()+Duration::from_secs(10);let observed=dev_observe(&f.j,"sample",&f.c,Some(&f.a),&json!({"sha":f.b,"deployed_at":"2026-10-30T10:00:00Z"}),end).unwrap();assert_eq!(observed["state"],"on DEV since 2026-10-30T10:00:00Z");assert_eq!(dev_observe(&f.j,"sample",&f.c,Some(&f.b),&json!({"sha":f.a}),end).unwrap()["state"],"not yet on DEV");assert!(dev_observe(&f.j,"sample",&f.c,None,&json!({"sha":f.b}),end).is_err());
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
        {let mut j=f.j.lock().unwrap();j.builtin_finish(r.id,&outcome,true,crate::now() as i64).unwrap();assert_eq!(j.request(r.id).unwrap().0,"uncertain","§4/R17: the command audit must preserve unknown external effects");}
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
    // A7 / R11/R17: a failed backmerge must retain successful release/smoke and observed LIVE identity.
    #[test]
    fn partial_pipeline_evidence_174() {
        let f=fixture("partial");f.j.lock().unwrap().grant(7,7,&["release:sample".into()],0).unwrap();
        let report=json!({"sha":f.b,"live_sha":f.b,"stages":{"release":{"state":"success","url":"https://example.invalid/release"},"smoke":{"state":"success","url":"https://example.invalid/smoke"},"backmerge":{"state":"failed","url":"https://example.invalid/backmerge"}}});
        std::fs::write(f.dir.join("runner"),format!("#!/bin/sh\nif [ \"$1\" = pipeline ]; then printf '%s' '{}'; exit 1; fi\n",report)).unwrap();
        let id=save_preview(&f);let result=confirm(&f.tq,&f.j,&request(&f,7),id).unwrap();
        assert!(result.contains("release: success https://example.invalid/release"),"{result}");
        assert!(result.contains("smoke: success https://example.invalid/smoke"),"{result}");
        assert!(result.contains("backmerge: failed https://example.invalid/backmerge"),"{result}");
        assert!(result.contains(&format!("LIVE SHA: {}",f.b)),"{result}");
        assert!(result.contains("uncertain") && !result.contains("LIVE SHA: unknown"));
        assert!(confirm(&f.tq,&f.j,&request(&f,7),id).is_err());
        let j=f.j.lock().unwrap();
        let evidence:String=j.0.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND item='verified'",[f.r.request],|r|r.get(0)).unwrap();
        let evidence:Value=serde_json::from_str(&evidence).unwrap();
        assert_eq!(evidence["live_sha"],f.b);assert_eq!(evidence["stages"],report["stages"]);
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
        // A6: absence of running evidence is not evidence of an idle checkout.
        for task in [json!({"n":42,"state":"doing","executions":[]}),json!({"n":42,"state":"doing","executions":[{"attempt":123,"host":"invented-local"}]}),json!({"n":42,"executions":[{"attempt":123,"state":"unrecognized"}]})] {
            std::fs::write(other.join("snapshot.json"),json!({"complete":true,"board":{"repo":"invented/tight"},"tasks":[task]}).to_string()).unwrap();
            let error=confirm(&f.tq,&f.j,&request(&f,7),id).unwrap_err();
            assert!(error.contains("tight task #42"),"unresolved attempt must name blocker: {task}: {error}");
            assert!(!f.dir.join("calls").exists());
        }

    }
}
