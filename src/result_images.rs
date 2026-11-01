//! §5 / #273: board attachment bytes, never a checkout, become individually receipted inline photos.
use crate::{direct::Direct, journal::{Journal,Run}, taskq::Taskq};
use rusqlite::{params,OptionalExtension};
use serde_json::{Value,json};
use std::{sync::Mutex, time::Duration};
const LIMIT:usize=20*1024*1024;
const PHOTO:usize=10*1024*1024;
const COUNT:usize=32;

pub fn tool()->Value {
    json!({"type":"function","name":"task_result_images","description":crate::catalog::purpose("task_result_images"),"inputSchema":{"type":"object","properties":{"project":{"type":"string"},"issue":{"type":"string"},"key":{"type":"string"}},"required":["issue"],"additionalProperties":false}})
}

// The binding contains no URL, credential or local filename. It is revalidated at every send.
#[derive(serde::Serialize,serde::Deserialize,Clone)]
struct Binding { request:i64, user:i64, project:String, board:String, root:String, chat:i64, topic:Option<i64> }
impl Binding {
    fn active(&self,j:&Journal)->Result<(),String> {
        if j.sender(self.request)?!=self.user || !j.has(self.user,"chat")? ||
            !(j.has(self.user,"projects")? || j.has(self.user,&format!("project:{}",self.project))?) ||
            j.route_held(self.chat,self.topic)? ||
            !j.projects()?.iter().any(|p|p.0==self.project && p.1==self.board && p.2==self.root) {
            return Err("Image access or destination changed; delivery held".into());
        }
        let route_project:Option<String>=j.0.query_row("SELECT project FROM route_projects WHERE chat=?1 AND topic=?2",params![self.chat,self.topic.unwrap_or(0)],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if route_project.is_some_and(|p|p!=self.project) {return Err("Image route project changed; delivery held".into());}
        let (chat,topic):(i64,Option<i64>)=j.0.query_row("SELECT chat,topic FROM requests WHERE id=?1",[self.request],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|_|"Image route unavailable")?;
        if chat!=self.chat || topic!=self.topic { return Err("Image destination changed; delivery held".into()); }
        Ok(())
    }
}

pub fn call(d:&Direct,tq:Option<&Taskq>,j:&Mutex<Journal>,r:&Run,args:&Value)->(bool,String) {
    let result:Result<String,String>=(|| {
        let (binding,n)={
            let j=j.lock().unwrap();
            if j.fenced(r)? {return Err("Image request stopped".into());}
            let user=j.sender(r.request)?;
            let (project,board,n)=crate::direct::target(&j,r,user,args)?;
            let root=j.projects()?.into_iter().find(|p|p.0==project).ok_or("Project unavailable")?.2;
            let (chat,topic)=j.0.query_row("SELECT chat,topic FROM requests WHERE id=?1",[r.request],|q|Ok((q.get(0)?,q.get(1)?))).map_err(|_|"Image route unavailable")?;
            let binding=Binding{request:r.request,user,project,board,root,chat,topic};
            binding.active(&j)?;
            let front:String=j.0.query_row("SELECT front FROM requests WHERE id=?1",[r.request],|q|q.get(0)).map_err(|_|"Image route unavailable")?;
            if front!="telegram" {return Err("Inline images require a Telegram request; use the task's result attachments in the CLI".into());}
            (binding,n)
        };
        fetch(d,tq.ok_or("TaskQ unavailable")?,j,&binding,n,args["key"].as_str(),Some(r),false)
    })();
    match result {Ok(t)=>(true,t),Err(e)=>(false,d.keys.scrub(&e))}
}

#[allow(clippy::too_many_arguments)]
fn fetch(d:&Direct,tq:&Taskq,j:&Mutex<Journal>,binding:&Binding,n:i64,key_name:Option<&str>,run:Option<&Run>,automatic:bool)->Result<String,String> {
        let (bound,board)=tq.registered(&j.lock().unwrap(),&binding.project)?;
        if board!=binding.board {return Err("Board changed".into());}
        let snapshot=bound.snapshot_for(&board)?;
        if automatic && snapshot["board"]["kind"]!="gitlab" {return Ok("No GitLab image attachments".into());}
        let host=snapshot["board"]["host"].as_str().ok_or("Board artifact host unavailable")?;
        if snapshot["board"]["kind"]!="gitlab" {return Err("Image retrieval is unavailable for this board provider".into());}
        let base=board_base(host,&board)?;
        let issue=bound.cli_checked(&["issue".into(),n.to_string(),"--json".into()],|out| {
            let v:Value=serde_json::from_str(out).map_err(|_|"Task result unreadable")?;
            if v["number"]!=n || !v["comments"].is_array() {return Err("No matching task result".into());}
            let u=url::Url::parse(v["url"].as_str().unwrap_or("")).map_err(|_|"Task identity unavailable")?;
            if u.origin()!=base.origin() || u.path()!=format!("/{board}/-/issues/{n}") {return Err("Task result belongs to another board".into());}
            Ok(v)
        })?;
        let refs:Vec<_>=references(&issue).into_iter().filter(|(_,u)|u.contains("/uploads/")).collect();
        let (key,secret)={let j=j.lock().unwrap();binding.active(&j)?;d.keys.take(key_name,"issue_read",binding.user,&j.route(binding.request)?,&binding.project,crate::now() as i64)?};
        let active=|| {
            let j=j.lock().unwrap();
            run.is_none_or(|r|!j.fenced(r).unwrap_or(true)) && binding.active(&j).is_ok() &&
                d.keys.take(Some(&key),"issue_read",binding.user,&j.route(binding.request).unwrap_or_default(),&binding.project,crate::now() as i64).is_ok_and(|(_,s)|s==secret)
        };
        let source_revision=snapshot["tasks"].as_array().and_then(|ts|ts.iter().find(|t|t["n"]==n)).and_then(|t|t["revision"].as_i64()).ok_or("Current task revision unavailable; no images queued")?;
        let images=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|"Image reader unavailable")?.block_on(async {
            tokio::time::timeout(Duration::from_secs(30),retrieve(&base,&board,&refs,&secret,&active)).await.map_err(|_|"Image retrieval timed out".to_string())?
        });
        let current=bound.snapshot_for(&board)?;
        if current["tasks"].as_array().and_then(|ts|ts.iter().find(|t|t["n"]==n)).and_then(|t|t["revision"].as_i64())!=Some(source_revision) {return Err("Task question changed during image retrieval; read its current card".into());}
        let j=j.lock().unwrap();
        if run.is_some_and(|r|j.fenced(r).unwrap_or(true)) {return Err("Image request stopped".into());}
        binding.active(&j)?;
        {
            if j.task(&binding.project,n)?.is_some_and(|(_,r,_)|r!=source_revision) {return Err("Task question changed during image retrieval; read its current card".into());}
        }
        // One gap accompanies a partial batch, and also covers an empty/failed batch.
        let (images,gap)=match images {Ok(v)=>v,Err(e)=>(vec![],Some(e))};
        let (images,sources):(Vec<_>,Vec<_>)=images.into_iter().map(|(label,bytes,source)|((d.keys.scrub(&label),bytes),source)).unzip();
        queue(&j,&binding,n,&images,gap.as_deref(),&sources,Some(source_revision))?;
        j.direct_event(binding.request,&binding.project,&format!("task_result_images #{n}"),Some(&key),&format!("queued {} images; gap={}",images.len(),gap.is_some()),crate::now() as i64)?;
        Ok(format!("Queued {} inline task images in this route{}. Delivery is confirmed only by the outbox receipts; no new decision was inferred.",images.len(),if gap.is_some() {" with one explicit image gap"} else {""}))
}

/// Model-free delivery follows the existing subscription route, never discovers a root.
pub fn watch(tq:std::sync::Arc<Taskq>,j:crate::telegram::Shared,d:std::sync::Arc<Direct>,wake:std::sync::Arc<tokio::sync::Notify>) {
    loop {
        let rows:Vec<(Binding,i64,i64)>=(||->Result<_,String>{
            let j=j.lock().unwrap();
            let mut q=j.0.prepare("SELECT t.request_id,q.sender,t.project,t.board,t.root,t.chat,t.topic,t.number,t.revision FROM tasks t JOIN requests q ON q.id=t.request_id WHERE q.front='telegram' AND (t.closed=1 OR t.state IN ('ask','review')) AND NOT EXISTS(SELECT 1 FROM meta WHERE key='result-image-observed:' || t.project || ':' || t.number || ':' || t.revision)").map_err(|e|e.to_string())?;
            q.query_map([],|r|Ok((Binding{request:r.get(0)?,user:r.get(1)?,project:r.get(2)?,board:r.get(3)?,root:r.get(4)?,chat:r.get(5)?,topic:r.get(6)?},r.get(7)?,r.get(8)?))).and_then(Iterator::collect).map_err(|e|e.to_string())
        })().unwrap_or_default();
        for (b,n,rev) in rows {
            if b.active(&j.lock().unwrap()).is_err() {continue;}
            let result=fetch(&d,&tq,&j,&b,n,None,None,true);
            // A newer question owns the next pass; do not let an old read queue a
            // gap under that newer revision and suppress its actual images.
            if result.as_ref().is_err_and(|e|e.starts_with("Task question changed during image retrieval")) {continue;}
            let j=j.lock().unwrap();
            // Retrieval errors remain explicit and bounded, rather than spinning or claiming delivery.
            if b.active(&j).is_ok() {
                if let Err(error)=result {let _=queue(&j,&b,n,&[],Some(&d.keys.scrub(&error)),&[],Some(rev));}
                let _=j.0.execute("INSERT OR IGNORE INTO meta(key,value) VALUES (?1,?2)",params![format!("result-image-observed:{}:{n}:{rev}",b.project),crate::now() as i64]);
                wake.notify_one();
            }
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

fn board_base(host:&str,board:&str)->Result<url::Url,String> {
    let base=url::Url::parse(&format!("https://{host}/")).map_err(|_|"Invalid board host")?;
    if base.host_str()!=Some(host) || base.port().is_some() || !base.username().is_empty() || base.password().is_some() || base.path()!="/" || base.query().is_some() || base.fragment().is_some() || board.split('/').count()<2 || board.split('/').any(|s|s.is_empty() || s=="." || s==".." || !s.bytes().all(|c|c.is_ascii_alphanumeric() || b"-_.".contains(&c))) {return Err("Invalid board artifact identity".into());}
    Ok(base)
}

// Only TaskQ-authenticated outcome/question entries select attachments; arbitrary prose cannot select a download.
fn references(issue:&Value)->Vec<(String,String)> {
    let mut refs=vec![];
    if let Some(comments)=issue["comments"].as_array() {
        for c in comments.iter().rev().filter(|c|c["trusted"]==true) {
            let body=c["body"].as_str().unwrap_or("");
            if !["taskq: result","taskq: ask","taskq: publish","taskq: close"].iter().any(|p|body.starts_with(p)) {continue;}
            for part in body.split('[').skip(1) {
                if let Some((label,tail))=part.split_once("](") {
                    if let Some((link,_))=tail.split_once(')') {
                        let link=link.trim_start_matches('<').trim_end_matches('>');
                        if link.contains("/uploads/") && refs.len()<COUNT && !refs.iter().any(|(_,u)|u==link) {refs.push((label.chars().take(100).collect(),link.to_string()));}
                    }
                }
            }
            // The latest outcome owns its artifacts; do not mix superseded variants.
            break;
        }
    }
    refs
}

// Accept only this project's API uploads, /uploads/secret/name and its exact web paths.
fn upload_url(base:&url::Url,board:&str,reference:&str)->Result<url::Url,String> {
    let u=base.join(reference).map_err(|_|"Invalid image attachment reference")?;
    if u.origin()!=base.origin() || !u.username().is_empty() || u.password().is_some() || u.query().is_some() || u.fragment().is_some() {return Err("Image attachment is outside this board".into());}
    let encoded: String=url::form_urlencoded::byte_serialize(board.as_bytes()).collect();
    let prefix=format!("/api/v4/projects/{encoded}/uploads/");
    let web=format!("/{board}/uploads/");
    let api_id=u.path().strip_prefix("/api/v4/projects/").and_then(|s|s.split_once("/uploads/")).filter(|(id,_)|!id.is_empty() && id.bytes().all(|b|b.is_ascii_digit())).map(|(_,s)|s);
    let full_path=u.path().strip_prefix("/-/project/").and_then(|s|s.split_once("/uploads/")).filter(|(id,_)|!id.is_empty() && id.bytes().all(|b|b.is_ascii_digit())).map(|(_,s)|s);
    let rest=if let Some(s)=u.path().strip_prefix(&prefix) {s} else if let Some(s)=u.path().strip_prefix(&web).or_else(||u.path().strip_prefix("/uploads/")).or(full_path).or(api_id) {s} else {return Err("Unsupported board image attachment reference".into());};
    let parts:Vec<_>=rest.split('/').collect();
    if !(parts.len()==1 && !parts[0].is_empty() && parts[0].bytes().all(|b|b.is_ascii_digit()) || parts.len()==2 && parts[0].len()==32 && parts[0].bytes().all(|b|b.is_ascii_hexdigit()) && !parts[1].is_empty() && parts[1].bytes().all(|b|b.is_ascii_alphanumeric() || b"-_.%".contains(&b)) && valid_upload_name(parts[1])) {return Err("Unsupported board upload identity".into());}
    base.join(&format!("{prefix}{rest}")).map_err(|_|"Invalid board upload URL".into())
}

fn image_reference(u:&url::Url)->bool {
    let name=u.path().rsplit('/').next().unwrap_or("").to_ascii_lowercase();
    name.bytes().all(|b|b.is_ascii_digit()) || [".jpg",".jpeg",".png"].iter().any(|ext|name.ends_with(ext))
}

fn valid_upload_name(name:&str)->bool {
    let bytes=name.as_bytes();let mut i=0;
    while i<bytes.len() {
        if bytes[i]==b'%' {
            let Some(hex)=name.get(i+1..i+3) else {return false;};
            let Ok(value)=u8::from_str_radix(hex,16) else {return false;};
            if value.is_ascii_control() || b"/\\.%".contains(&value) {return false;}
            i+=3;
        } else {i+=1;}
    }
    true
}

fn check_image_headers(response:&reqwest::Response)->Result<(),String> {
    if !response.status().is_success() {return Err(format!("Image attachment inaccessible (HTTP {})",response.status().as_u16()));}
    if response.headers().get(reqwest::header::CONTENT_LENGTH).and_then(|v|v.to_str().ok()).and_then(|v|v.parse::<u64>().ok()).is_some_and(|n|n>PHOTO as u64) {return Err("Image exceeds Telegram's 10 MiB photo limit (20 MiB batch limit)".into());}
    let mime=response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v|v.to_str().ok()).unwrap_or("").split(';').next().unwrap_or("").trim();
    if !matches!(mime,"image/jpeg"|"image/png") {return Err("Image-only source unavailable; server did not identify a JPEG or PNG; no bundle body downloaded".into());}
    Ok(())
}

async fn download(client:&reqwest::Client,u:url::Url,secret:&str,active:&impl Fn()->bool)->Result<Vec<u8>,String> {
    if !active() {return Err("Image access revoked".into());}
    if !image_reference(&u) {return Err("Image-only source unavailable; ZIPs and non-image files are not downloaded".into());}
    // HEAD checks metadata without transferring bundle bodies, including opaque upload IDs.
    let metadata=client.head(u.clone()).bearer_auth(secret).send().await.map_err(|_|"Image metadata inaccessible")?;
    check_image_headers(&metadata)?;
    if !active() {return Err("Image access revoked".into());}
    let mut response=client.get(u).bearer_auth(secret).send().await.map_err(|_|"Image attachment inaccessible")?;
    check_image_headers(&response)?;
    if response.content_length().is_some_and(|n|n>LIMIT as u64) {return Err("Image attachment exceeds 20 MiB".into());}
    let mut bytes=vec![];
    while let Some(chunk)=response.chunk().await.map_err(|_|"Image attachment interrupted")? {
        if !active() {return Err("Image access revoked".into());}
        if bytes.len()+chunk.len()>LIMIT {return Err("Image attachment exceeds 20 MiB".into());}
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn retrieve(base:&url::Url,board:&str,refs:&[(String,String)],secret:&str,active:&impl Fn()->bool)->Result<(Vec<(String,Vec<u8>,Value)>,Option<String>),String> {
    let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(10)).build().map_err(|_|"Image reader unavailable")?;
    let mut images=vec![];let mut gap=None;let mut total=0;
    for (label,reference) in refs {
        let result=async {
            let u=upload_url(base,board,reference)?;
            let bytes=download(&client,u.clone(),secret,active).await?;
            let receipt=json!({"source":u.as_str(),"label":label,"downloaded_at":crate::now(),"bytes":bytes.len(),"crc32":format!("{:08x}",crc32fast::hash(&bytes))});
            Ok::<_,String>((unpack(label,&bytes)?,receipt))
        }.await;
        match result {
            Ok((batch,receipt))=>for (name,bytes) in batch {
                total+=bytes.len();
                if total>LIMIT || images.len()>=COUNT {gap=Some("Result images exceed the 20 MiB / 32 image batch limit".into());break;}
                images.push((name,bytes,receipt.clone()));
            },
            Err(e)=>{gap.get_or_insert(e);},
        }
    }
    if images.is_empty() && gap.is_none() {gap=Some("No supported images in the trusted task result".into());}
    Ok((images,gap))
}

fn image(bytes:&[u8])->bool {bytes.starts_with(b"\xff\xd8\xff") || bytes.starts_with(b"\x89PNG\r\n\x1a\n")}
fn unpack(label:&str,bytes:&[u8])->Result<Vec<(String,Vec<u8>)>,String> {
    if !image(bytes) {return Err("Image-only source unavailable; expected a direct JPEG or PNG, never a ZIP or mixed bundle".into());}
    if bytes.len()>PHOTO {return Err("Image exceeds Telegram's 10 MiB photo limit".into());}
    Ok(vec![(label.to_string(),bytes.to_vec())])
}

fn queue(j:&Journal,b:&Binding,n:i64,images:&[(String,Vec<u8>)],gap:Option<&str>,sources:&[Value],expected_revision:Option<i64>)->Result<(),String> {
    b.active(j)?;
    let tx=j.0.unchecked_transaction().map_err(|e|e.to_string())?;
    let revision:i64=expected_revision.unwrap_or(tx.query_row("SELECT coalesce((SELECT revision FROM tasks WHERE project=?1 AND number=?2),0)",params![b.project,n],|r|r.get(0)).map_err(|e|e.to_string())?);
    let tag=format!("result-images:{}:{n}:{revision}",b.request);
    if tx.query_row("SELECT EXISTS(SELECT 1 FROM meta WHERE key=?1)",[&tag],|r|r.get::<_,bool>(0)).map_err(|e|e.to_string())? {return Ok(());}
    let at=crate::now() as i64;
    for (i,(label,bytes)) in images.iter().enumerate() {
        let text=format!("{} #{n} · revision {revision} · {} / {}\n{}",b.project,i+1,images.len(),label);
        tx.execute("INSERT INTO outbox(request_id,chat,topic,text,next_at,kind,project,task,revision) VALUES (?1,?2,?3,?4,?5,'result_image',?6,?7,?8)",params![b.request,b.chat,b.topic,text,at,b.project,n,revision]).map_err(|e|e.to_string())?;
        let id=tx.last_insert_rowid();
        if let Some(source)=sources.get(i) {
            let receipt=json!({"project":b.project,"board":b.board,"task":n,"revision":revision,"variant":label,"outbox":id,"download":source});
            tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![format!("result-image-source:{id}"),receipt.to_string()]).map_err(|e|e.to_string())?;
        }
        tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![format!("result-image:{id}"),bytes]).map_err(|e|e.to_string())?;
        tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![format!("result-image-binding:{id}"),serde_json::to_string(b).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    }
    if let Some(gap)=gap {
        let language:Option<String>=tx.query_row("SELECT language FROM requests WHERE id=?1",[b.request],|r|r.get(0)).map_err(|e|e.to_string())?;
        let text=if language.as_deref().unwrap_or("en").starts_with("ru") {format!("{} #{n}: изображения доставлены не полностью. {gap}",b.project)} else {format!("{} #{n}: {gap}. Image delivery is incomplete.",b.project)};
        tx.execute("INSERT INTO outbox(request_id,chat,topic,text,next_at,kind,project,task) VALUES (?1,?2,?3,?4,?5,'result_image_gap',?6,?7)",params![b.request,b.chat,b.topic,text,at,b.project,n]).map_err(|e|e.to_string())?;
        let id=tx.last_insert_rowid();
        tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![format!("result-image-binding:{id}"),serde_json::to_string(b).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    }
    tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![tag,at]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}

pub fn authorize(j:&Journal,id:i64)->Result<(),String> {
    let binding:String=j.0.query_row("SELECT value FROM meta WHERE key=?1",[format!("result-image-binding:{id}")],|r|r.get(0)).map_err(|_|"Image delivery binding unavailable")?;
    let binding:Binding=serde_json::from_str(&binding).map_err(|_|"Image delivery binding unreadable")?;
    binding.active(j)?;
    let (project,n,revision):(String,i64,Option<i64>)=j.0.query_row("SELECT project,task,revision FROM outbox WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
    if let Some(revision)=revision {
        if j.task(&project,n)?.is_some_and(|(_,current,_)|current!=revision) {return Err("Image task revision changed; delivery held".into());}
    }
    Ok(())
}

pub fn payload(j:&Journal,id:i64)->Result<Vec<u8>,String> {
    authorize(j,id)?;
    j.0.query_row("SELECT value FROM meta WHERE key=?1",[format!("result-image:{id}")],|r|r.get(0)).map_err(|_|"Retained result image unavailable".into())
}

pub fn prune(j:&Journal,at:i64)->Result<(),String> {
    j.0.execute("DELETE FROM meta WHERE (key LIKE 'result-image:%' OR key LIKE 'result-image-binding:%') AND CAST(substr(key,instr(key,':')+1) AS INTEGER) IN (SELECT id FROM outbox WHERE kind IN ('result_image','result_image_gap') AND state IN ('delivered','failed','superseded') AND next_at<?1)",[at-7*86400]).map(drop).map_err(|e|e.to_string())
}

/// Visual option answers carry the exact selection and stable Telegram identity on the board.
/// The original argv still owns reconciliation and card projection.
pub fn answer_actor(j:&Journal,project:&str,n:i64,revision:i64,sender:i64,argv:&mut Vec<String>)->Result<(),String> {
    let visual:bool=j.0.query_row("SELECT EXISTS(SELECT 1 FROM outbox WHERE project=?1 AND task=?2 AND revision=?3 AND kind='result_image')",params![project,n,revision],|r|r.get(0)).map_err(|e|e.to_string())?;
    if !visual {return Ok(());}
    let choice=argv.get(1).and_then(|s|s.split_once('.')).map(|(_,pick)|pick.to_string());
    let text_index=argv.iter().position(|a|a=="--text").map(|i|i+1);
    let text=choice.or_else(||text_index.and_then(|i|argv.get(i).cloned())).unwrap_or_default();
    let selected=j.selected_answer(project,n,revision,&text)?;
    if selected==text && text.parse::<u64>().is_ok() {return Err("Visual option no longer belongs to this question; nothing was written".into());}
    let annotated=format!("{selected}\nTelegram user {sender}");
    if let Some(i)=text_index {argv[i]=annotated;} else {argv.extend(["--text".into(),annotated]);}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // A3/A7: reject mixed bundles rather than transferring code bytes; a newer
    // attachment-free outcome must never inherit superseded review variants.
    #[test]
    fn result_images_273_retrieval_boundaries() {
        let base=board_base("gitlab.example.test","invented/csgo").unwrap();
        for reference in ["/api/v4/projects/invented%2Fcsgo/uploads/42","/uploads/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/review.jpg"] {
            assert!(upload_url(&base,"invented/csgo",reference).is_ok());
        }
        for reference in ["https://evil.invalid/uploads/42","/api/v4/projects/other%2Fproject/uploads/42","file:///tmp/private.jpg","/uploads/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/%2e%2e"] {
            assert!(upload_url(&base,"invented/csgo",reference).is_err());
        }
        let issue=json!({"comments":[{"trusted":true,"body":"taskq: result\n![old](/uploads/old)"},{"trusted":true,"body":"taskq: ask\nChoose without attachments"}]});
        assert!(references(&issue).is_empty(),"a new outcome cannot reuse old variants");
        assert!(unpack("bundle",b"PK\x03\x04mixed bundle").is_err(),"ZIP bytes are unsupported");
        assert!(image_reference(&base.join("/uploads/42").unwrap()));
        assert!(!image_reference(&base.join("/uploads/secret/review.zip").unwrap()),"ZIP must be refused before network dispatch");
        assert_eq!(unpack("B",b"\xff\xd8\xffinvented").unwrap()[0].0,"B");
    }

    // #273 A3/A7: real HTTP dispatch injects auth only at the bound API, refuses
    // redirects, observes access revocation and size/404 errors without leaking URLs.
    #[test]
    fn result_images_273_http() {
        use std::io::{Read,Write};
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base=url::Url::parse(&format!("http://{}/",listener.local_addr().unwrap())).unwrap();
        let worker=std::thread::spawn(move || {
            for (method,response) in [("HEAD","HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: 4\r\nConnection: close\r\n\r\n"),("GET","HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: 4\r\nConnection: close\r\n\r\nDATA"),("HEAD","HTTP/1.1 302 Found\r\nLocation: https://evil.invalid/secret\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),("HEAD","HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),("HEAD","HTTP/1.1 200 OK\r\nContent-Length: 20971521\r\nConnection: close\r\n\r\n"),("HEAD","HTTP/1.1 200 OK\r\nContent-Type: application/zip\r\nContent-Length: 4\r\nConnection: close\r\n\r\n")] {
                let (mut stream,_)=listener.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();let mut buf=[0;4096];let n=stream.read(&mut buf).unwrap();let request=String::from_utf8_lossy(&buf[..n]);
                assert!(request.starts_with(&format!("{method} /api/v4/projects/invented%2Fcsgo/uploads/42 ")),"{request}");
                assert!(request.to_ascii_lowercase().contains("authorization: bearer invented-test-key"));
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();let u=upload_url(&base,"invented/csgo","/api/v4/projects/123/uploads/42").unwrap();
            assert_eq!(download(&client,u.clone(),"invented-test-key",&||true).await.unwrap(),b"DATA");
            for expected in ["302","404","20 MiB","Image-only"] {let e=download(&client,u.clone(),"invented-test-key",&||true).await.unwrap_err();assert!(e.contains(expected) && !e.contains("http") && !e.contains("invented-test-key"),"{e}");}
            let zip=base.join("/api/v4/projects/invented%2Fcsgo/uploads/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/review.zip").unwrap();
            assert!(download(&client,zip,"invented-test-key",&||true).await.unwrap_err().starts_with("Image-only"));
            assert!(download(&client,u,"invented-test-key",&||false).await.unwrap_err().contains("revoked"));
        });worker.join().unwrap();
    }

    // #273 A2/A5/A6: exact topic binding and durable per-photo receipts survive
    // reopen; duplicate queueing and revoked/rebound access cannot send a photo.
    #[test]
    fn result_images_273_outbox_and_actor() {
        let dir=std::env::temp_dir().join(format!("pilotq-result-images-273-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();let path=dir.join("journal.sqlite");
        let mut j=Journal::open(&path).unwrap();j.bootstrap(7,0).unwrap();j.add_project(7,"csgo","invented/csgo","/invented/board-only",0).unwrap();
        let r=j.ask(7,"show csgo task 444",1).unwrap();
        j.0.execute("UPDATE requests SET front='telegram',chat=-100,topic=5 WHERE id=?1",[r.request]).unwrap();
        j.0.execute("DELETE FROM outbox",[]).unwrap();
        let b=Binding{request:r.request,user:7,project:"csgo".into(),board:"invented/csgo".into(),root:"/invented/board-only".into(),chat:-100,topic:Some(5)};
        j.0.execute("INSERT INTO tasks(number,request_id,chat,topic,state,revision,observed,project,board,root) VALUES (444,?1,-100,5,'doing',1,0,'csgo','invented/csgo','/invented/board-only')",[r.request]).unwrap();
        let q=json!({"parsed":true,"choices":[{"id":1,"label":"Variant A"},{"id":2,"label":"Variant B"}]});
        j.observe("csgo",444,"ask",2,Some(&q.to_string()),"status",Some("Choose a variant"),2).unwrap();
        let images=vec![("Variant B".into(),b"\xff\xd8\xffinvented".to_vec())];queue(&j,&b,444,&images,None,&[json!({"source":"invented-source","bytes":images[0].1.len()})],None).unwrap();queue(&j,&b,444,&images,None,&[json!({"source":"invented-source","bytes":images[0].1.len()})],None).unwrap();
        let id:i64=j.0.query_row("SELECT id FROM outbox WHERE kind='result_image'",[],|r|r.get(0)).unwrap();
        assert_eq!(j.0.query_row("SELECT count(*) FROM outbox WHERE kind='result_image'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(payload(&j,id).unwrap(),images[0].1);
        let mut argv=vec!["answer".into(),"444.2".into(),"--revision".into(),"2".into()];answer_actor(&j,"csgo",444,2,7,&mut argv).unwrap();assert_eq!(argv.last().unwrap(),"2. Variant B\nTelegram user 7");
        let mut stale=vec!["answer".into(),"444.3".into()];assert!(answer_actor(&j,"csgo",444,2,7,&mut stale).is_err());
        j.0.execute("UPDATE tasks SET revision=3 WHERE number=444",[]).unwrap();assert!(payload(&j,id).is_err());
        j.0.execute("UPDATE tasks SET revision=2 WHERE number=444",[]).unwrap();
        j.0.execute("UPDATE projects SET root='/invented/rebound' WHERE name='csgo'",[]).unwrap();assert!(payload(&j,id).is_err());j.0.execute("UPDATE projects SET root='/invented/board-only' WHERE name='csgo'",[]).unwrap();
        j.0.execute("UPDATE requests SET topic=6 WHERE id=?1",[r.request]).unwrap();assert!(payload(&j,id).is_err());j.0.execute("UPDATE requests SET topic=5 WHERE id=?1",[r.request]).unwrap();
        drop(j);let mut j=Journal::open(&path).unwrap();assert_eq!(payload(&j,id).unwrap(),images[0].1);
        j.record(id,&crate::journal::Sent::Ok(1234),3).unwrap();
        let source:String=j.0.query_row("SELECT value FROM meta WHERE key=?1",[format!("result-image-source:{id}")],|r|r.get(0)).unwrap();
        let receipt:Value=serde_json::from_str(&source).unwrap();
        assert_eq!(receipt["download"]["source"],"invented-source");assert_eq!(receipt["task"],444);assert_eq!(receipt["variant"],"Variant B");assert_eq!(receipt["revision"],2);
        assert_eq!(j.0.query_row("SELECT state FROM outbox WHERE id=?1",[id],|r|r.get::<_,String>(0)).unwrap(),"delivered");
        j.revoke(7,7,&["admin".into(),"projects".into()],4).unwrap();assert!(payload(&j,id).is_err());
        prune(&j,7*86400+4).unwrap();assert!(j.0.query_row("SELECT value FROM meta WHERE key=?1",[format!("result-image:{id}")],|r|r.get::<_,Vec<u8>>(0)).optional().unwrap().is_none());
        drop(j);std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    // A2/A6: inspect the actual multipart photo send, not a link or a JSON mock
    // of the rendering helper; the original chat/topic and bytes must survive.
    #[test]
    fn result_images_273_photo_transport() {
        use std::io::{Read,Write};
        let dir=std::env::temp_dir().join(format!("pilotq-photo-273-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();
        let mut journal=Journal::open(&dir.join("journal.sqlite")).unwrap();journal.bootstrap(7,0).unwrap();journal.add_project(7,"csgo","invented/csgo","/invented/board-only",0).unwrap();let r=journal.ask(7,"csgo 444",1).unwrap();
        journal.0.execute("UPDATE requests SET front='telegram',chat=-100,topic=5 WHERE id=?1",[r.request]).unwrap();
        let b=Binding{request:r.request,user:7,project:"csgo".into(),board:"invented/csgo".into(),root:"/invented/board-only".into(),chat:-100,topic:Some(5)};
        queue(&journal,&b,444,&[("Variant B".into(),b"\xff\xd8\xffinvented-photo-bytes".to_vec())],None,&[],None).unwrap();
        let id=journal.0.query_row("SELECT id FROM outbox WHERE kind='result_image'",[],|r|r.get(0)).unwrap();
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();
        let server=std::thread::spawn(move || {
            let (mut stream,_)=listener.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();let mut bytes=vec![];let mut buf=[0;8192];
            loop {let n=stream.read(&mut buf).unwrap();assert!(n>0);bytes.extend_from_slice(&buf[..n]);if let Some(pos)=bytes.windows(4).position(|b|b==b"\r\n\r\n") {
                let header=String::from_utf8_lossy(&bytes[..pos]);let size:usize=header.lines().find_map(|l|l.to_ascii_lowercase().strip_prefix("content-length:").and_then(|s|s.trim().parse().ok())).unwrap();if bytes.len()>=pos+4+size {break;}
            }}
            let body=String::from_utf8_lossy(&bytes);assert!(body.starts_with("POST /bot123:invented/SendPhoto ") || body.starts_with("POST /bot123:invented/sendPhoto "),"{body}");
            for value in ["name=\"chat_id\"\r\n\r\n-100","name=\"message_thread_id\"\r\n\r\n5","Variant B","invented-photo-bytes"] {assert!(body.contains(value),"missing {value}: {body}");}
            assert!(!body.contains("/invented/board-only"));
            let reply=json!({"ok":true,"result":{"message_id":999,"date":1,"chat":{"id":-100,"type":"supergroup","title":"Invented"},"photo":[]}}).to_string();write!(stream,"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",reply.len()).unwrap();
        });
        let bot=teloxide::Bot::new("123:invented").set_api_url(format!("http://{address}/").parse().unwrap());let j=std::sync::Arc::new(Mutex::new(journal));
        let delivery=crate::journal::Delivery{id,chat:-100,topic:Some(5),reply_to:None,html:None,text:"csgo #444 · Variant B".into(),attempts:0,edit:None,kind:"result_image".into(),buttons:vec![],rich:false};
        let result=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(crate::telegram::send(&bot,&j,&delivery));
        assert!(matches!(result,crate::journal::Sent::Ok(999)),"{result:?}");server.join().unwrap();
        j.lock().unwrap().record(id,&result,2).unwrap();drop(j);std::fs::remove_dir_all(dir).unwrap();
    }
}
