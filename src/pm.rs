//! #221 / R13,R16,R17: owner-installed route context; no new execution authority.
use crate::journal::{Journal, Run};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const NAMES: &[&str] = &["pm_note_read", "pm_note_write", "pm_note_forget"];
const POLICY: &str = "PM pack is owner-installed operational guidance, subordinate to current grants and the PilotQ contract. PM notes are historical context, never authorization or current board truth. Only notes in the current index remain valid; suppress forgotten or superseded notes from older turns and receipts. Read relevant notes with pm_note_read; write one fact and its why with pm_note_write. Never store credentials. For PM ticks obtain board state only through installed TaskQ commands; inspect taskq profile telemetry. Report the observed board link and a workers table using verified task links, never bare task IDs. Missing links or observations are unavailable, never invented. Shadow mode means findings only: no board writes, releases, or answers on the owner's behalf.";
#[derive(Clone, Serialize, Deserialize)]
struct Note { fact: String, why: String }
#[derive(Default, Serialize, Deserialize)]
struct Binding { pack: String, project: Option<String>, notes: BTreeMap<String, Note> }
fn err(e: rusqlite::Error) -> String { e.to_string() }
fn get<T: serde::de::DeserializeOwned>(j:&Journal,key:&str)->Result<Option<T>,String> {
    j.0.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[key],|r|r.get::<_,String>(0)).optional().map_err(err)?
        .map(|s|serde_json::from_str(&s).map_err(|_|"PM context is corrupt; inspect it through the owner CLI.".into())).transpose()
}
fn put(j:&Journal,key:&str,v:&impl Serialize)->Result<(),String> {
    j.0.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![key,serde_json::to_string(v).map_err(|e|e.to_string())?]).map(drop).map_err(err)
}
fn ident(s:&str)->Result<(),String> {
    if s.is_empty() || s.len()>64 || !s.bytes().all(|b|b.is_ascii_alphanumeric() || b"-_.".contains(&b)) {return Err("Use a 1..64 character name containing letters, digits, -, _ or .".into());} Ok(())
}
fn route(s:&str)->Result<(),String> {
    if s=="cli" {return Ok(());}
    let parts:Vec<_>=s.split(':').collect();
    if parts.len()>2 || parts[0].parse::<i64>().ok().is_none_or(|n| n==0 || n.to_string()!=parts[0]) || (parts.len()==2 && parts[1].parse::<i64>().ok().is_none_or(|n|n<=0 || n.to_string()!=parts[1])) {return Err("Route must be cli, CHAT or CHAT:TOPIC with exact numeric IDs.".into());} Ok(())
}
fn clean(keys:&crate::keys::Keys,s:&str,max:usize)->Result<(),String> {
    if s.trim().is_empty() || s.len()>max {return Err(format!("PM text must be nonempty and at most {max} bytes."));}
    // Known stored values plus common credential syntax. No rejected text is echoed or persisted here.
    let lower=s.to_ascii_lowercase();
    if keys.scrub(s)!=s || lower.split_whitespace().any(|w|w.starts_with("sk-") && w.len()>12) || ["-----begin", "bearer ", "ghp_", "github_pat_", "password=", "password:", "token=", "token:", "secret=", "secret:", "api_key", "apikey=", "authorization:"].iter().any(|p|lower.contains(p)) {
        return Err("PM context refuses credential material; use the protected keys CLI.".into());
    }
    Ok(())
}
fn key(route:&str)->String {format!("pm-binding:{route}")}
fn binding(j:&Journal,route:&str)->Result<Option<Binding>,String> {get(j,&key(route))}
fn save(j:&Journal,route:&str,b:&Binding)->Result<(),String> {put(j,&key(route),b)}
fn authority(j:&Journal,user:i64)->Result<bool,String> {Ok(j.has(user,"chat")? && (j.owner()?==Some(user) || j.has(user,"shell")?))}
fn scoped(j:&Journal,user:i64,route:&str)->Result<Option<Binding>,String> {
    if !authority(j,user)? {return Ok(None);}
    let Some(b)=binding(j,route)? else {return Ok(None)};
    if b.pack.is_empty() {return Ok(None);}
    if let Some(p)=&b.project {
        if !j.projects()?.iter().any(|x|&x.0==p) || !(j.has(user,"projects")? || j.has(user,&format!("project:{p}"))?) {return Ok(None);}
    }
    Ok(Some(b))
}
pub fn available(j:&Journal,user:i64,route:&str)->Result<bool,String> {Ok(scoped(j,user,route)?.is_some())}
pub fn help(j:&Journal,user:i64,route:&str,ru:bool)->Result<String,String> {
    Ok(match scoped(j,user,route)? {
        Some(b)=>format!("{}: {}{}",if ru {"PM-пакет"}else{"PM pack"},b.pack,b.project.map(|p|format!(" · {p}")).unwrap_or_default()),
        None=>if ru {"PM-пакет: не привязан или недоступен."} else {"PM pack: unbound or unavailable."}.into(),
    })
}
pub fn context(j:&Journal,request:i64)->Result<Value,String> {
    let user=j.sender(request)?;let route=j.route(request)?;
    let Some(b)=scoped(j,user,&route)? else {return Ok(Value::Null)};
    let pack:String=get(j,&format!("pm-pack:{}",b.pack))?.ok_or("Bound PM pack is missing.")?;
    Ok(json!({"policy":POLICY,"pack":b.pack,"runbook":pack,"project":b.project,"notes":b.notes.iter().map(|(id,n)|json!({"id":id,"fact":n.fact})).collect::<Vec<_>>()}))
}
pub fn tools()->Vec<Value> {
    NAMES.iter().map(|name| {
        let mut properties=json!({"id":{"type":"string"}});let mut required=vec!["id"];
        if *name=="pm_note_write" {properties["fact"]=json!({"type":"string"});properties["why"]=json!({"type":"string"});required.extend(["fact","why"]);}
        json!({"type":"function","name":name,"description":crate::catalog::purpose(name),"inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false}})
    }).collect()
}
fn note(b:&mut Binding,name:&str,args:&Value,keys:&crate::keys::Keys)->Result<Value,String> {
    let id=args["id"].as_str().ok_or("Note ID required.")?;ident(id)?;clean(keys,id,64)?;
    match name {
        "pm_note_read"=>{let n=b.notes.get(id).ok_or("PM note unavailable.")?;clean(keys,&n.fact,1000)?;clean(keys,&n.why,500)?;Ok(json!(n))},
        "pm_note_forget"=>{b.notes.remove(id);Ok(json!({"forgotten":id}))},
        "pm_note_write"=>{
            let fact=args["fact"].as_str().ok_or("Fact required.")?;let why=args["why"].as_str().ok_or("Why required.")?;
            clean(keys,fact,1000)?;clean(keys,why,500)?;
            if fact.contains(['\n','\r']) || why.contains(['\n','\r']) {return Err("Use one fact and one why line per note.".into());}
            if b.notes.len()>=64 && !b.notes.contains_key(id) {return Err("PM note limit is 64; forget an obsolete note first.".into());}
            b.notes.insert(id.into(),Note{fact:fact.into(),why:why.into()});Ok(json!({"id":id,"note":b.notes[id]}))
        },_=>Err("Unknown PM note tool.".into()),
    }
}
pub fn call(j:&std::sync::Mutex<Journal>,r:&Run,name:&str,args:&Value,keys:&crate::keys::Keys,at:i64)->(bool,String) {
    let result=(||{
        let j=j.lock().unwrap();let tx=j.0.unchecked_transaction().map_err(err)?;
        if j.fenced(r)? {return Err("PM note run is fenced.".into());}
        let route=j.route(r.request)?;let mut b=scoped(&j,j.sender(r.request)?,&route)?.ok_or("No authorized PM binding in this route.")?;
        let signature=json!({"tool":name,"args":args}).to_string();
        let operation=format!("{name}:{}",args["id"].as_str().unwrap_or(""));
        // Validate before receipts/audit, so rejected credentials never enter PM storage.
        let out=note(&mut b,name,args,keys)?;
        if name=="pm_note_read" {
            j.0.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,?3,?4)",params![r.request,at,name,route]).map_err(err)?;
            tx.commit().map_err(err)?;return Ok(out.to_string());
        }
        let old:Option<(String,String)>=j.0.query_row("SELECT args,receipt FROM admin_ops WHERE request=?1 AND tool=?2",params![r.request,operation],|x|Ok((x.get(0)?,x.get(1)?))).optional().map_err(err)?;
        if let Some((before,out))=old {if before!=signature {return Err("PM note arguments changed in one request.".into());}return Ok(out);}
        save(&j,&route,&b)?;
        let out=out.to_string();
        j.0.execute("INSERT INTO admin_ops(request,tool,args,receipt) VALUES (?1,?2,?3,?4)",params![r.request,operation,signature,out]).map_err(err)?;
        j.0.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,?3,?4)",params![r.request,at,name,route]).map_err(err)?;
        tx.commit().map_err(err)?;Ok(out)
    })();match result {Ok(s)=>(true,s),Err(e)=>(false,e)}
}
/// Protected operator path, sharing the journal and scope logic with the manager tools.
pub fn cli(j:&Journal,actor:i64,args:&[&str],keys:&crate::keys::Keys,at:i64)->Result<Value,String> {
    if j.owner()?!=Some(actor) {return Err("PM administration requires the protected owner.".into());}
    let tx=j.0.unchecked_transaction().map_err(err)?;
    let out=match args {
        ["install",name,file]=>{
            ident(name)?;clean(keys,name,64)?;
            let f=std::fs::File::open(file).map_err(|_|"PM pack file unavailable.")?;
            use std::io::Read;let mut text=String::new();f.take(16385).read_to_string(&mut text).map_err(|_|"PM pack must be UTF-8." )?;
            clean(keys,&text,16384)?;put(j,&format!("pm-pack:{name}"),&text)?;json!({"installed":name,"bytes":text.len()})
        },
        ["bind",r,name,rest @ ..] if rest.len()<=1=>{
            route(r)?;ident(name)?;let _:String=get(j,&format!("pm-pack:{name}"))?.ok_or("Install the PM pack first.")?;
            let project=rest.first().map(|s|s.to_string());
            if project.as_ref().is_some_and(|p|!j.projects().unwrap_or_default().iter().any(|x|&x.0==p)) {return Err("PM binding requires a registered project.".into());}
            let mut b=binding(j,r)?.unwrap_or_default();
            if b.project!=project && !b.notes.is_empty() {return Err("Forget existing notes before changing their project scope.".into());}
            b.pack=name.to_string();b.project=project;save(j,r,&b)?;json!({"route":r,"binding":b})
        },
        ["unbind",r]=>{route(r)?;let mut b=binding(j,r)?.unwrap_or_default();b.pack.clear();save(j,r,&b)?;json!({"route":r,"unbound":true})},
        ["notes",r,"list"]=>{route(r)?;json!({"route":r,"binding":binding(j,r)?})},
        ["notes",r,"set",id,fact,why]=>{route(r)?;let mut b=binding(j,r)?.ok_or("Route has no PM binding.")?;let out=note(&mut b,"pm_note_write",&json!({"id":id,"fact":fact,"why":why}),keys)?;save(j,r,&b)?;out},
        ["notes",r,"forget",id]=>{route(r)?;let mut b=binding(j,r)?.ok_or("Route has no PM binding.")?;let out=note(&mut b,"pm_note_forget",&json!({"id":id}),keys)?;save(j,r,&b)?;out},
        _=>return Err("usage: pm install NAME FILE | bind ROUTE NAME [PROJECT] | unbind ROUTE | notes ROUTE list|set ID FACT WHY|forget ID".into()),
    };
    // Audit only metadata; pack/note text belongs in scoped storage, not global audit context.
    let update:i64=j.0.query_row("SELECT min(coalesce(min(update_id),0),0)-1 FROM requests",[],|r|r.get(0)).map_err(err)?;
    let action=format!("pm {}",args.first().unwrap_or(&""));
    let request=crate::journal::insert(&j.0,update,"cli",actor,None,actor,None,0,"pm",Some(&action),"answered",at,&format!("cli:{actor}"),None)?;
    let target=if matches!(args.first(),Some(&"bind"|&"unbind"|&"notes")) {args.get(1).copied()} else {None};
    j.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'pm_cli',?3,'completed')",params![request,at,target]).map_err(err)?;
    tx.commit().map_err(err)?;
    serde_json::from_str(&keys.scrub(&out.to_string())).map_err(|_|"PM readback withheld: key store unavailable.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::Input;
    fn setup()->(Journal,std::path::PathBuf,crate::keys::Keys) {
        let p=std::env::temp_dir().join(format!("pilotq-pm-221-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&p).unwrap();let mut j=Journal::open(&p.join("journal.sqlite")).unwrap();j.bootstrap(7,0).unwrap();
        let keys=crate::keys::Keys(p.join("keys.json"));
        std::fs::write(p.join("pack.txt"),include_str!("../docs/pm-pack.md")).unwrap();
        cli(&j,7,&["install","pm",p.join("pack.txt").to_str().unwrap()],&keys,0).unwrap();(j,p,keys)
    }
    fn admit(j:&mut Journal,update:i64,topic:i64,user:i64,text:&str)->i64 {
        let i=Input{callback:None,media_group:None,quote:None,image:None,voice:None,language:Some("en".into()),chat:-100,topic:Some(topic),sender:Some(user),name:None,message:update,kind:"text",text:Some(text.into()),reply_to:None,files:vec![]};
        j.admit(&[(update,Some(i))],1).unwrap();j.0.query_row("SELECT id FROM requests WHERE update_id=?1",[update],|r|r.get(0)).unwrap()
    }
    // A2-A4/A6 / R13: real ingress, note tools, restart and /new must preserve scope and authority.
    #[test]
    fn pm_221_binding_scope_restart_reset_and_revocation() {
        let (mut j,p,keys)=setup();
        cli(&j,7,&["bind","-100:1","pm"],&keys,0).unwrap();
        cli(&j,7,&["bind","-100:2","pm"],&keys,0).unwrap();
        let id=admit(&mut j,1,1,7,"inspect");let other=admit(&mut j,2,2,7,"inspect");
        assert_eq!(context(&j,id).unwrap()["pack"],"pm");
        for language in ["en","ru"] {
            let help=crate::catalog::render(&j,7,Some("operations"),Some(language),&keys,"-100:1").unwrap();
            assert!(help.contains(if language=="en" {"PM pack: pm"}else{"PM-пакет: pm"}));
        }
        assert!(crate::catalog::visible(crate::catalog::CATALOG.iter().find(|c|c.name=="pm_note_read").unwrap(),&j,7,&keys,"7").unwrap()==false);
        let r=j.claim_next(4,2).unwrap().into_iter().find(|r|r.request==id).unwrap();
        let mutex=std::sync::Mutex::new(j);
        let args=json!({"id":"release","fact":"Release needs verified CI.","why":"Protect the pilot."});
        assert!(call(&mutex,&r,"pm_note_write",&args,&keys,3).0);
        assert!(call(&mutex,&r,"pm_note_write",&args,&keys,3).0,"repeat returns receipt");
        assert!(call(&mutex,&r,"pm_note_read",&json!({"id":"release"}),&keys,3).0);
        let unsafe_note=json!({"id":"unsafe","fact":"token=INVENTED","why":"test refusal"});
        assert!(!call(&mutex,&r,"pm_note_write",&unsafe_note,&keys,3).0);
        assert!(binding(&mutex.lock().unwrap(),"-100:1").unwrap().unwrap().notes.get("unsafe").is_none());
        let mut altered=args.clone();altered["fact"]=json!("Changed fact.");
        assert!(!call(&mutex,&r,"pm_note_write",&altered,&keys,3).0,"changed retry must refuse");
        assert_eq!(context(&mutex.lock().unwrap(),other).unwrap()["notes"],json!([]));
        mutex.lock().unwrap().finish(&r,None,None,&crate::manager::Outcome::Answered("checked".into()),4).unwrap();
        drop(mutex);
        let mut j=Journal::open(&p.join("journal.sqlite")).unwrap();
        admit(&mut j,3,1,7,"/new");let fresh=admit(&mut j,4,1,7,"continue");
        assert_eq!(context(&j,fresh).unwrap()["notes"][0]["id"],"release");
        assert_eq!(context(&j,other).unwrap()["notes"],json!([]));
        assert!(help(&j,7,"7",false).unwrap().contains("unbound"),"no DM leak");
        j.grant(7,8,&["chat".into(),"shell".into()],5).unwrap();let grantee=admit(&mut j,5,1,8,"inspect");
        assert!(!context(&j,grantee).unwrap().is_null());j.revoke(7,8,&["shell".into()],6).unwrap();
        assert!(context(&j,grantee).unwrap().is_null());
        cli(&j,7,&["notes","-100:1","forget","release"],&keys,7).unwrap();
        assert_eq!(context(&j,fresh).unwrap()["notes"],json!([]));
        cli(&j,7,&["unbind","-100:1"],&keys,8).unwrap();assert!(context(&j,fresh).unwrap().is_null());
        std::fs::remove_dir_all(p).unwrap();
    }
    // A4/A6 / R16: rejected known keys and recognizable credential syntax never persist.
    #[test]
    fn pm_221_secret_refusal_and_owner_cli() {
        let (j,p,keys)=setup();cli(&j,7,&["bind","cli","pm"],&keys,0).unwrap();
        keys.add("invented","test fixture",&["issue_read".into()],"invented-protected-value-221").unwrap();
        for text in ["invented-protected-value-221","token=INVENTED","Bearer INVENTED","-----BEGIN PRIVATE KEY-----"] {
            assert!(cli(&j,7,&["notes","cli","set","unsafe",text,"reason"],&keys,1).is_err());
            std::fs::write(p.join("unsafe.txt"),text).unwrap();
            assert!(cli(&j,7,&["install","unsafe",p.join("unsafe.txt").to_str().unwrap()],&keys,1).is_err());
        }
        assert!(binding(&j,"cli").unwrap().unwrap().notes.is_empty());
        assert!(cli(&j,8,&["notes","cli","list"],&keys,1).is_err());
        cli(&j,7,&["notes","cli","set","safe","One reviewed fact.","One reason."],&keys,2).unwrap();
        assert_eq!(cli(&j,7,&["notes","cli","list"],&keys,3).unwrap()["binding"]["notes"]["safe"]["why"],"One reason.");
        std::fs::remove_dir_all(p).unwrap();
    }
    // A4 / R4,R10: optional project narrowing cannot share route notes or outlive project grants.
    #[test]
    fn pm_221_project_narrowing_and_tick_context() {
        let (mut j,p,keys)=setup();
        j.add_project(7,"example","invented/example",p.to_str().unwrap(),0).unwrap();
        for r in ["-100:1","-100:2"] {cli(&j,7,&["bind",r,"pm","example"],&keys,0).unwrap();}
        cli(&j,7,&["notes","-100:1","set","release","One project fact.","One project reason."],&keys,0).unwrap();
        j.grant(7,8,&["chat".into(),"shell".into()],0).unwrap();
        let id=admit(&mut j,1,1,8,"inspect");assert!(context(&j,id).unwrap().is_null());
        j.grant(7,8,&["project:example".into()],1).unwrap();
        assert_eq!(context(&j,id).unwrap()["notes"][0]["id"],"release");
        let other=admit(&mut j,2,2,8,"inspect");assert_eq!(context(&j,other).unwrap()["notes"],json!([]));
        admit(&mut j,3,1,7,"/loop 1m shadow findings");crate::loops::fire(&mut j,61).unwrap();
        let tick:i64=j.0.query_row("SELECT max(request_id) FROM audit_events WHERE kind='loop_tick'",[],|r|r.get(0)).unwrap();
        assert_eq!(context(&j,tick).unwrap()["notes"][0]["id"],"release");
        j.revoke(7,8,&["project:example".into()],62).unwrap();assert!(context(&j,id).unwrap().is_null());
        j.0.execute("DELETE FROM projects WHERE name='example'",[]).unwrap();assert!(context(&j,tick).unwrap().is_null());
        std::fs::remove_dir_all(p).unwrap();
    }

    // A3/A4 / R5,R13: a read is an observation, not a mutation receipt; CLI corrections must be visible immediately.
    #[test]
    fn pm_221_note_reads_follow_cli_correction() {
        let (mut j,p,keys)=setup();cli(&j,7,&["bind","cli","pm"],&keys,0).unwrap();
        let r=j.ask(7,"inspect notes",1).unwrap();let mutex=std::sync::Mutex::new(j);
        assert!(call(&mutex,&r,"pm_note_write",&json!({"id":"fact","fact":"Old fact.","why":"Original reason."}),&keys,2).0);
        assert!(call(&mutex,&r,"pm_note_read",&json!({"id":"fact"}),&keys,3).1.contains("Old fact."));
        cli(&mutex.lock().unwrap(),7,&["notes","cli","set","fact","Corrected fact.","Corrected reason."],&keys,4).unwrap();
        let read=call(&mutex,&r,"pm_note_read",&json!({"id":"fact"}),&keys,5);
        assert!(read.0 && read.1.contains("Corrected fact."),"note reads must observe the CLI correction: {read:?}");
        std::fs::remove_dir_all(p).unwrap();
    }

}
