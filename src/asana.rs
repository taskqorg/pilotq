//! Bounded, credential-scoped Asana reads (pilotq.md §5, #159).
use crate::{direct::Direct, journal::{Journal, Run}};
use serde_json::{Value, json};
use std::{sync::Mutex, time::{Duration, Instant}};

pub fn tool() -> Value {
    json!({"type":"function","name":"asana_read","description":crate::catalog::purpose("asana_read"),"inputSchema":{"type":"object","properties":{"url":{"type":"string"},"project":{"type":"string"},"key":{"type":"string"}},"required":["url"],"additionalProperties":false}})
}

fn task_id(link: &str) -> Result<String,String> {
    let u = url::Url::parse(link).map_err(|_|"invalid Asana task link")?;
    if u.scheme() != "https" || u.host_str() != Some("app.asana.com") || u.port().is_some() || !u.username().is_empty() || u.password().is_some() {
        return Err("expected an https://app.asana.com task link".into());
    }
    let parts: Vec<_> = u.path().trim_matches('/').split('/').collect();
    let id = match parts.as_slice() {
        ["0", _, id] | ["0", _, id, "f"] => *id,
        ["1", _, "project", _, "task", id] | ["1", _, "project", _, "task", id, "f"] => *id,
        _ => return Err("unsupported Asana task link".into()),
    };
    if id.is_empty() || id.len()>32 || !id.bytes().all(|c|c.is_ascii_digit()) { return Err("invalid Asana task ID".into()); }
    Ok(id.into())
}

pub fn call(d: &Direct, j: &Mutex<Journal>, r: &Run, args: &Value, deadline: Instant) -> (bool,String) {
    call_at(d,j,r,args,deadline,"https://app.asana.com/api/1.0")
}

fn call_at(d: &Direct, j: &Mutex<Journal>, r: &Run, args: &Value, deadline: Instant, base: &str) -> (bool,String) {
    let result = (|| {
        let link=args["url"].as_str().ok_or("Asana task URL is required")?;
        let id=task_id(link)?;
        if !crate::direct::own_url(&r.text,link) { return Err("Asana link must be in the person's own message".into()); }
        let (project,key,secret) = {
            let j=j.lock().unwrap();
            let user=j.sender(r.request)?;
            if j.fenced(r)? || !j.has(user,"chat")? { return Err("Asana read refused: stopped run or missing chat grant".into()); }
            // Reuse §5 target selection without treating the external link as a board URL.
            let mut target=args.clone(); target["issue"]=json!("1");
            let (project,_,_)=crate::direct::target(&j,r,user,&target)?;
            let (key,secret)=d.keys.take(args["key"].as_str(),"asana_read",user,&j.route(r.request)?,&project,crate::now() as i64)?;
            j.direct_event(r.request,&project,&format!("asana_read {id}"),Some(&key),"start",crate::now() as i64)?;
            (project,key,secret)
        };
        let end=deadline.min(Instant::now()+Duration::from_secs(10));
        let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|"Asana runtime unavailable")?;
        let out=rt.block_on(async {
            let left=end.checked_duration_since(Instant::now()).ok_or("Asana read deadline reached")?;
            let active=|| {
                let j=j.lock().unwrap();
                let Ok(user)=j.sender(r.request) else { return false; };
                !j.fenced(r).unwrap_or(true) && j.has(user,"chat").unwrap_or(false)
                    && (j.has(user,"projects").unwrap_or(false) || j.has(user,&format!("project:{project}")).unwrap_or(false))
                    && j.projects().is_ok_and(|ps|ps.iter().any(|p|p.0==project))
                    && j.route(r.request).is_ok_and(|route|d.keys.take(Some(&key),"asana_read",user,&route,&project,crate::now() as i64).is_ok_and(|(_,s)|s==secret))
            };
            tokio::time::timeout(left,read(base,&id,&secret,&active)).await.map_err(|_|"Asana read timed out".to_string())?
        });
        // §8 / R16: scrub decoded strings before JSON escaping can hide a key.
        let text=match &out {
            Ok(v)=>{ let mut v=v.clone(); scrub_value(&d.keys,&mut v); v.to_string() },
            Err(e)=>d.keys.scrub(e),
        };
        let j=j.lock().unwrap();
        j.direct_event(r.request,&project,&format!("asana_read {id}"),Some(&key),&text,crate::now() as i64)?;
        if j.fenced(r)? { return Err("Asana read cancelled".into()); }
        out.map(|_|text)
    })();
    match result { Ok(t)=>(true,t), Err(e)=>(false,d.keys.scrub(&e)) }
}

fn scrub_value(keys: &crate::keys::Keys, value: &mut Value) {
    match value {
        Value::String(s)=>*s=keys.scrub(s),
        Value::Array(values)=>values.iter_mut().for_each(|v|scrub_value(keys,v)),
        Value::Object(values)=>values.values_mut().for_each(|v|scrub_value(keys,v)),
        _=>{},
    }
}

async fn get(client: &reqwest::Client, url: &str, secret: &str) -> Result<Value,String> {
    let mut response=client.get(url).bearer_auth(secret).send().await.map_err(|_|"Asana API unavailable")?;
    if response.status().as_u16()==404 { return Err("Asana task not found or inaccessible (404)".into()); }
    if !response.status().is_success() { return Err(format!("Asana API error HTTP {}",response.status().as_u16())); }
    let mut bytes=Vec::new();
    while let Some(chunk)=response.chunk().await.map_err(|_|"Asana response interrupted")? {
        if bytes.len()+chunk.len()>256*1024 { return Err("Asana response exceeds 256 KiB".into()); }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_|"Asana returned invalid JSON".into())
}

fn cut(v: &Value, n: usize) -> Value {
    match v.as_str() {
        Some(s) if s.chars().count()>n => json!(format!("{}…",s.chars().take(n).collect::<String>())),
        Some(s)=>json!(s), None=>Value::Null,
    }
}

async fn read(base: &str, id: &str, secret: &str, active: &impl Fn() -> bool) -> Result<Value,String> {
    let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().map_err(|_|"Asana client unavailable")?;
    if !active() { return Err("Asana read authorization revoked or run stopped".into()); }
    let task=get(&client,&format!("{base}/tasks/{id}?opt_fields=gid,name,notes,assignee.name,due_on,due_at,completed"),secret).await?;
    let t=&task["data"];
    if t["gid"]!=id || !t["name"].is_string() || !t["completed"].is_boolean() { return Err("Asana returned no matching complete task".into()); }
    if !active() { return Err("Asana read authorization revoked or run stopped".into()); }
    let comments=get(&client,&format!("{base}/tasks/{id}/stories?limit=20&resource_subtype=comment_added&sort_ascending=false&opt_fields=text,created_by.name,created_at,resource_subtype"),secret).await?;
    if !active() { return Err("Asana read authorization revoked or run stopped".into()); }
    let attachments=get(&client,&format!("{base}/attachments?parent={id}&limit=20&opt_fields=name"),secret).await?;
    if !active() { return Err("Asana read authorization revoked or run stopped".into()); }
    let cs=comments["data"].as_array().ok_or("Asana comments unavailable")?;
    let files=attachments["data"].as_array().ok_or("Asana attachments unavailable")?;
    Ok(json!({"task_id":id,"title":cut(&t["name"],500),"notes":cut(&t["notes"],4000),"assignee":cut(&t["assignee"]["name"],500),"due_on":cut(&t["due_on"],500),"due_at":cut(&t["due_at"],500),"completed":t["completed"],
        "comments":cs.iter().take(20).map(|c|json!({"text":cut(&c["text"],500),"author":cut(&c["created_by"]["name"],500),"created_at":cut(&c["created_at"],500)})).collect::<Vec<_>>(),
        "attachment_names":files.iter().take(20).map(|f|cut(&f["name"],500)).collect::<Vec<_>>(),
        "comments_more":!comments["next_page"].is_null() || cs.len()>20,"attachments_more":!attachments["next_page"].is_null() || files.len()>20,
        "excerpts":true,"read_at":crate::now()}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read,Write};

    // §5 / R16: trace the actual dispatcher and HTTP boundary, retaining grant,
    // fixed destination, bounded excerpts and unknown/error protections (#159 A3).
    #[test]
    fn asana_read_trace() {
        let dir=std::env::temp_dir().join(format!("pilotq-asana-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
        let mut j=Journal::open(&dir.join("journal.sqlite")).unwrap();
        j.bootstrap(7,0).unwrap();
        assert!(crate::direct::tools().iter().any(|t|t["name"]=="asana_read"));
        j.grant(7,8,&["chat".into()],0).unwrap();
        j.add_project(7,"pilotq","taskqorg/pilotq",dir.to_str().unwrap(),0).unwrap();
        let r=j.ask(7,"Read pilotq https://app.asana.com/0/123/456",1).unwrap();
        let j=Mutex::new(j);
        let d=Direct{keys:crate::keys::Keys(dir.join("keys.json"))};
        let args=json!({"url":"https://app.asana.com/0/123/456","project":"pilotq"});
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base=format!("http://{}/api/1.0",listener.local_addr().unwrap());
        let denied=call_at(&d,&j,&r,&args,Instant::now()+Duration::from_secs(2),&base);
        assert!(!denied.0 && denied.1.contains("no key"),"{denied:?}");
        let secret="invented-asana-\"secret\\value";
        d.keys.add("asana","invented",&["asana_read".into()],secret).unwrap();
        d.keys.grant("asana",&["7".into()]).unwrap();
        let other=j.lock().unwrap().ask(8,"Read pilotq https://app.asana.com/0/123/456",2).unwrap();
        let denied=call_at(&d,&j,&other,&args,Instant::now()+Duration::from_secs(2),&base);
        assert!(!denied.0 && denied.1.contains("no project:pilotq grant"),"{denied:?}");
        let server=std::thread::spawn(move || {
            let responses=[
                (200,json!({"data":{"gid":"456","name":"Invented task","notes":format!("{secret}{}","x".repeat(5000)),"assignee":{"name":"Tester"},"due_on":"2026-10-31","due_at":null,"completed":false}})),
                (200,json!({"data":[{"text":secret,"created_by":{"name":"Tester"},"created_at":"2026-10-30"}],"next_page":{"offset":"untrusted","uri":"https://evil.invalid"}})),
                (200,json!({"data":[{"name":"design.pdf"}],"next_page":null})),
                (404,json!({"errors":[{"message":secret}]})),
                (500,json!({"errors":[{"message":secret}]})),
            ];
            for (index,(status,body)) in responses.into_iter().enumerate() {
                let (mut stream,_)=listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut request=Vec::new();
                loop { let mut buf=[0;1024]; let n=stream.read(&mut buf).unwrap(); assert!(n>0);request.extend_from_slice(&buf[..n]);if request.windows(4).any(|w|w==b"\r\n\r\n"){break;} }
                let request=String::from_utf8(request).unwrap();
                assert!(request.starts_with("GET /api/1.0/"));
                assert!(request.to_lowercase().contains(&format!("authorization: bearer {secret}")));
                match index { 1=>assert!(request.contains("/tasks/456/stories?")),2=>assert!(request.contains("/attachments?parent=456")),_=>assert!(request.contains("/tasks/456?")) }
                let body=body.to_string();
                write!(stream,"HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            }
        });
        let (ok,text)=call_at(&d,&j,&r,&args,Instant::now()+Duration::from_secs(5),&base);
        assert!(ok,"{text}");
        let v:Value=serde_json::from_str(&text).unwrap();
        assert_eq!(v["title"],"Invented task"); assert_eq!(v["assignee"],"Tester");
        assert_eq!(v["due_on"],"2026-10-31"); assert_eq!(v["completed"],false);
        assert_eq!(v["attachment_names"],json!(["design.pdf"])); assert_eq!(v["comments_more"],true);
        assert!(v["notes"].as_str().unwrap().chars().count()<=4001);
        assert!(!v["notes"].as_str().unwrap().contains(secret));
        assert_eq!(v["comments"][0]["text"], "[key asana]");
        assert!(!text.contains(secret) && text.contains("[key asana]"));
        for expected in ["not found or inaccessible (404)","API error HTTP 500"] {
            let (ok,text)=call_at(&d,&j,&r,&args,Instant::now()+Duration::from_secs(5),&base);
            assert!(!ok && text.contains(expected) && !text.contains(secret),"{text}");
        }
        server.join().unwrap();
        for link in ["https://evil.invalid/0/123/456","https://app.asana.com@evil.invalid/0/123/456","http://app.asana.com/0/123/456","https://app.asana.com/0/123/abc"] { assert!(task_id(link).is_err()); }
        let audit=json!(j.lock().unwrap().audit(7,&crate::journal::Filter::default(),2).unwrap()).to_string();
        assert!(!audit.contains(secret) && audit.contains("asana_read"));
        let decoded_audit: Value = serde_json::from_str(&audit).unwrap();
        let receipt=decoded_audit.as_array().unwrap().iter()
            .flat_map(|record|record["direct"].as_array().unwrap())
            .filter_map(|event|event["result"].as_str())
            .find_map(|result|serde_json::from_str::<Value>(result).ok()).unwrap();
        assert_eq!(receipt["comments"][0]["text"], "[key asana]");
        assert!(!receipt["notes"].as_str().unwrap().contains(secret));
        drop(j); std::fs::remove_dir_all(dir).unwrap();
    }
}
