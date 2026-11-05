//! Admission persistence (pilotq.md §§2,4–9).
use super::*;

impl Journal {
    /// #106 / R13: operator controls use the same admission, grants and audit boundary.
    pub fn builtin_cli(&mut self, sender:i64, text:&str, at:i64) -> Result<Option<i64>,String> {
        let Some(cmd)=control(Some(text)) else { return Ok(None) };
        if !has(&self.0,sender,"chat")? { return Err("chat grant required".into()); }
        let tx=self.0.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
        let update:i64=tx.query_row("SELECT min(coalesce(min(update_id),0),0)-1 FROM requests",[],|r|r.get(0)).map_err(|e|e.to_string())?;
        let session=format!("cli:{sender}");
        let deferred=matches!(cmd,"/help"|"/model"|"/doctor"|"/onboard"|"/onboard-button"|"/release"|"/release-confirm"|"/restart") && (cmd!="/restart" || meta(&tx,"owner")?==Some(sender));
        let failed=matches!(cmd,"/unknown"|"/skill"|"/restart") && !deferred;
        let id=insert(&tx,update,"cli",sender,None,sender,None,0,if deferred {"builtin"} else {"control"},Some(text),if deferred {"queued"} else if failed {"failed"} else {"answered"},at,&session,None)?;
        record_reply_language(&tx,id,None,None,at,false)?;
        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,coalesce((SELECT generation FROM conversations WHERE session=?2),0))",params![format!("builtin-generation:{id}"),session]).map_err(|e|e.to_string())?;
        if !deferred {
            let (text,_)=answer_control(&tx,&session,cmd,id,sender,sender,at)?;
            let language:Option<String>=tx.query_row("SELECT language FROM requests WHERE id=?1",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
            let text=crate::builtins::localized(&text,language.as_deref(),failed);
            tx.execute("UPDATE requests SET reply=?2,error=?3 WHERE id=?1",params![id,text,if failed {Some(&text)} else {None}]).map_err(|e|e.to_string())?;
        }
        tx.commit().map_err(|e|e.to_string())?;
        self.trace(id,at);
        Ok(Some(id))
    }

    pub fn builtin_take(&mut self,id:i64) -> Result<Option<crate::builtins::Request>,String> {
        self.0.query_row("UPDATE requests SET state='waiting' WHERE id=?1 AND kind='builtin' AND state='queued' RETURNING id,session,sender,text",
            [id],|r|Ok(crate::builtins::Request{id:r.get(0)?,session:r.get(1)?,sender:r.get(2)?,text:r.get(3)?})).optional().map_err(|e|e.to_string())
    }

    pub fn builtin_next(&mut self) -> Result<Option<crate::builtins::Request>,String> {
        revalidate(&self.0)?;
        self.0.query_row("UPDATE requests SET state='waiting' WHERE id=(SELECT min(id) FROM requests WHERE kind='builtin' AND state='queued') RETURNING id,session,sender,text",
            [],|r|Ok(crate::builtins::Request{id:r.get(0)?,session:r.get(1)?,sender:r.get(2)?,text:r.get(3)?})).optional().map_err(|e|e.to_string())
    }

    pub fn builtin_allowed(&self, r:&crate::builtins::Request) -> Result<bool,String> {
        has(&self.0,r.sender,"chat")
    }

    pub fn builtin_finish(&mut self, id:i64, text:&str, ok:bool, at:i64) -> Result<(),String> {
        let tx=self.0.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
        finish_builtin(&tx,id,text,ok,at)?;
        tx.commit().map_err(|e|e.to_string())?;
        self.trace(id,at);
        Ok(())
    }

    /// Atomic selection, audit reply and delivery: a crash cannot lose a committed choice receipt.
    pub fn builtin_models(&mut self,r:&crate::builtins::Request,models:&[String],at:i64) -> Result<(),String> {
        let tx=self.0.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
        let e=|e:rusqlite::Error|e.to_string();
        if !has(&tx,r.sender,"chat")? { finish_builtin(&tx,r.id,"Access revoked.",false,at)?; return tx.commit().map_err(e); }
        let generation:i64=tx.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&r.session],|x|x.get(0)).map_err(e)?;
        let saved=meta(&tx,&format!("builtin-generation:{}",r.id))?.unwrap_or(0);
        let mut args=r.text.split_whitespace();
        let command=args.next().unwrap_or("").split('@').next().unwrap_or("");
        let arg=args.next();
        let selected=if command=="/model-button" {
            let (source,index)=arg.and_then(|s|s.split_once(':')).ok_or("Invalid model button")?;
            let source=source.parse::<i64>().map_err(|_|"Invalid model button")?;
            let index=index.parse::<usize>().map_err(|_|"Invalid model button")?;
            let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM requests q JOIN outbox o ON o.request_id=q.id WHERE q.id=?1 AND q.session=?2 AND q.sender=?3 AND o.sent_message=(SELECT message FROM requests WHERE id=?4) AND o.state='delivered' AND q.id=(SELECT max(x.id) FROM requests x JOIN meta m ON m.key='models:' || x.id WHERE x.session=?2))",
                params![source,r.session,r.sender,r.id],|x|x.get(0)).map_err(e)?;
            if !valid || meta(&tx,&format!("models-epoch:{source}"))?.unwrap_or(0)!=meta(&tx,&format!("model-epoch:{}",r.session))?.unwrap_or(0) || meta(&tx,&format!("builtin-generation:{source}"))?!=Some(generation) || meta(&tx,&format!("model-used:{source}"))?.is_some() { return Err("This model selector is stale or belongs to another person; use /model.".into()); }
            let catalogue:String=tx.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[format!("models:{source}")],|x|x.get(0)).map_err(e)?;
            let names:Vec<String>=serde_json::from_str(&catalogue).map_err(|_|"Invalid model selector")?;
            let name=names.get(index).ok_or("Invalid model button")?.clone();
            tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![format!("model-used:{source}"),r.id]).map_err(e)?;
            Some(name)
        } else { arg.map(str::to_string) };
        if args.next().is_some() { return Err("Use /model, /model MODEL or /model default.".into()); }
        if generation!=saved { return Err("Conversation was reset; use /model again.".into()); }
        let ru:bool=tx.query_row("SELECT coalesce(language,'en') LIKE 'ru%' FROM requests WHERE id=?1",[r.id],|x|x.get(0)).map_err(e)?;
        let key=format!("model:{}",r.session);
        let before:Option<String>=tx.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[&key],|x|x.get(0)).optional().map_err(e)?;
        let text=if let Some(name)=selected {
            if name!="default" && !models.contains(&name) { return Err("Model is not in the current Codex catalogue; choice unchanged.".into()); }
            if name=="default" { tx.execute("DELETE FROM meta WHERE key=?1",[&key]).map_err(e)?; }
            else { tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![key,name]).map_err(e)?; }
            tx.execute("INSERT INTO meta(key,value) VALUES (?1,1) ON CONFLICT(key) DO UPDATE SET value=value+1",[format!("model-epoch:{}",r.session)]).map_err(e)?;
            tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,1)",[format!("refresh:{}",r.session)]).map_err(e)?;
            tx.execute("INSERT INTO audit_events(request_id,at,kind,item,result) VALUES (?1,?2,'model',?3,?4)",params![r.id,at,before,name]).map_err(e)?;
            if ru {format!("Модель сохранена: {name}. Применится к следующему запуску в этом диалоге; активные запуски и исполнители TaskQ сохраняют свои настройки.")}else{format!("Saved model: {name}. Applies to the next run in this conversation; active runs and TaskQ workers keep their policy.")}
        } else {
            let mut choices=vec!["default".to_string()]; choices.extend_from_slice(models);
            tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![format!("models:{}",r.id),serde_json::to_string(&choices).map_err(|e|e.to_string())?]).map_err(e)?;
            tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![format!("models-epoch:{}",r.id),meta(&tx,&format!("model-epoch:{}",r.session))?.unwrap_or(0)]).map_err(e)?;
            let actual:Option<String>=tx.query_row("SELECT u.model FROM runs u JOIN requests q ON q.id=u.request_id WHERE q.session=?1 AND u.model IS NOT NULL ORDER BY u.id DESC LIMIT 1",[&r.session],|x|x.get(0)).optional().map_err(e)?;
            if ru {format!("Сохранённый выбор: {}. Последняя наблюдаемая модель: {}.\nКаталог Codex (доступ подтверждает только завершённый запрос):\n{}\nВыберите кнопку или /model MODEL. Только для следующего запуска; настройки исполнителей TaskQ не меняются.",before.as_deref().unwrap_or("по умолчанию"),actual.as_deref().unwrap_or("нет данных"),choices.iter().map(|m|if m=="default"{"по умолчанию"}else{m.as_str()}).collect::<Vec<_>>().join("\n"))}else{format!("Saved choice: {}. Last reported model: {}.\\nCodex catalogue (access is verified only by a completed turn):\\n{}\\nUse a button or /model MODEL. Next run only; TaskQ worker policy is unchanged.",before.as_deref().unwrap_or("default"),actual.as_deref().unwrap_or("unknown"),choices.join("\\n")).replace("\\n","\n")}
        };
        finish_builtin(&tx,r.id,&text,true,at)?;
        tx.commit().map_err(e)?; self.trace(r.id,at); Ok(())
    }

    pub fn builtin_recover(&mut self,_at:i64) -> Result<(),String> {
        // Read-only controls and uncommitted local selections can retry. A dispatched restart never repeats.
        self.0.execute("UPDATE requests SET state='queued' WHERE kind='builtin' AND state='waiting' AND id<>coalesce((SELECT value FROM meta WHERE key='command-restart'),0)",[]).map_err(|e|e.to_string())?;
        Ok(())
    }

    pub fn builtin_restart(&self,id:i64) -> Result<(),String> {
        let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        tx.execute("INSERT INTO meta(key,value) VALUES ('command-restart',?1)",[id]).map_err(|e|e.to_string())?;
        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('command-restart-pid',?1)",[std::process::id()]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())
    }

    pub fn builtin_restarted(&mut self,at:i64) -> Result<(),String> {
        if meta(&self.0,"command-restart-pid")?==Some(std::process::id() as i64) {return Ok(());}
        if let Some(id)=meta(&self.0,"command-restart")? {
            let tx=self.0.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
            finish_builtin(&tx,id,"PilotQ restarted and wrote healthy status. Earlier requests and TaskQ work remain recorded.",true,at)?;
            tx.execute("DELETE FROM meta WHERE key='command-restart'",[]).map_err(|e|e.to_string())?;
            tx.commit().map_err(|e|e.to_string())?; self.trace(id,at);
        }
        Ok(())
    }

    /// Offset for the next getUpdates call; it confirms everything already committed.
    pub fn cursor(&self) -> Result<i64, String> {
        self.0.query_row("SELECT value FROM meta WHERE key='cursor'", [], |r| r.get(0)).optional().map(Option::unwrap_or_default).map_err(|e| e.to_string())
    }

    /// Commits a getUpdates batch, its dispositions and the next cursor in one transaction, before Telegram is told to forget them.
    /// A known update ID is a duplicate and changes nothing. Only a sender holding `chat` is admitted; everyone else is ignored
    /// silently, and a later grant does not replay them. Returns the number of newly admitted requests.
    pub fn admit(&mut self, batch: &[(i64, Option<Input>)], at: i64) -> Result<usize, String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        let (mut admitted, mut ended) = (0, vec![]);
        for (update, input) in batch {
            let known: bool = tx.query_row("SELECT 1 FROM updates WHERE update_id=?1", [update], |_| Ok(true)).optional().map_err(|e| e.to_string())?.is_some();
            if known {
                continue;
            }
            if let Some(i)=input.as_ref().filter(|i|i.kind=="route_incident") {
                let reason=i.text.as_deref().unwrap_or("route unavailable");
                hold_route(&tx,i.chat,i.topic,reason,at)?;
                tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![format!("route-event:{update}"),json!({"chat":i.chat,"topic":i.topic,"reason":reason,"at":at}).to_string()]).map_err(|e|e.to_string())?;
                tx.execute("INSERT INTO updates(update_id,disposition,at) VALUES (?1,'unsupported',?2)",params![update,at]).map_err(|e|e.to_string())?;
                continue;
            }
            if let Some(i)=input.as_ref().filter(|i|i.kind=="edit") {
                let mut request=None;
                if let Some(sender)=i.sender {
                    if !has(&tx,sender,"chat")? {
                        tx.execute("INSERT INTO updates(update_id,disposition,at) VALUES (?1,'ignored',?2)",params![update,at]).map_err(|e|e.to_string())?;
                        continue;
                    }
                    let original:Option<(i64,String,String,String)>=tx.query_row(
                        "SELECT q.id,q.state,q.session,q.kind FROM requests q WHERE q.front='telegram' AND q.chat=?1 AND q.topic IS ?2 AND (q.message=?3 OR EXISTS(SELECT 1 FROM audit_events a WHERE a.request_id=q.id AND a.kind='album_member' AND json_extract(a.item,'$.message')=?3)) AND q.sender=?4 ORDER BY q.id LIMIT 1",
                        params![i.chat,i.topic,i.message,sender],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(|e|e.to_string())?;
                    if let Some((id,state,session,kind))=original {
                        // §2: durable album membership identifies the source, but cannot revive a reset generation.
                        let album_generation:Option<i64>=tx.query_row("SELECT cast(item AS INTEGER) FROM audit_events WHERE request_id=?1 AND kind='album_generation' ORDER BY id LIMIT 1",[id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
                        if let Some(generation)=album_generation {
                            let current:i64=tx.query_row("SELECT coalesce((SELECT generation FROM conversations WHERE session=?1),0)",[&session],|r|r.get(0)).map_err(|e|e.to_string())?;
                            if generation!=current {
                                tx.execute("INSERT INTO updates(update_id,disposition,at) VALUES (?1,'ignored',?2)",params![update,at]).map_err(|e|e.to_string())?;
                                continue;
                            }
                        }
                        let before:Option<String>=tx.query_row("SELECT text FROM requests WHERE id=?1",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
                        let revision=json!({"update":update,"message":i.message,"author":sender,"before":before,"text":i.text,"image":i.image,"voice":i.voice,"files":i.files,"quote":i.quote,"media_group":i.media_group});
                        let previous:Option<String>=tx.query_row("SELECT item FROM audit_events WHERE request_id=?1 AND kind='input_revision' ORDER BY id DESC LIMIT 1",[id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
                        let same=if let Some(previous)=previous {
                            let mut previous:Value=serde_json::from_str(&previous).map_err(|e|e.to_string())?;
                            let mut current=revision.clone();
                            for v in [&mut previous,&mut current] {v.as_object_mut().unwrap().remove("update");v.as_object_mut().unwrap().remove("before");}
                            previous==current
                        } else {false};
                        if same {
                            tx.execute("INSERT INTO updates(update_id,disposition,request_id,at) VALUES (?1,'duplicate_revision',?2,?3)",params![update,id,at]).map_err(|e|e.to_string())?;
                            continue;
                        }
                        // Every prior request for this source is stale, including earlier change requests.
                        // Already admitted launches retain their immutable plan and operation identities.
                        tx.execute("INSERT OR IGNORE INTO meta(key,value) SELECT 'edited-request:' || q.id,?5 FROM requests q WHERE q.front='telegram' AND q.chat=?1 AND q.topic IS ?2 AND (q.message=?3 OR q.message=(SELECT message FROM requests WHERE id=?6) OR q.id=?6 OR q.message IN (SELECT json_extract(a.item,'$.message') FROM audit_events a WHERE a.request_id=?6 AND a.kind='album_member')) AND q.sender=?4",params![i.chat,i.topic,i.message,sender,at,id]).map_err(|e|e.to_string())?;
                        tx.execute("INSERT OR IGNORE INTO meta(key,value) SELECT 'edited-plan:' || p.id,?5 FROM plans p JOIN requests q ON q.id=p.request_id WHERE q.front='telegram' AND q.chat=?1 AND q.topic IS ?2 AND (q.message=?3 OR q.message=(SELECT message FROM requests WHERE id=?6) OR q.id=?6 OR q.message IN (SELECT json_extract(a.item,'$.message') FROM audit_events a WHERE a.request_id=?6 AND a.kind='album_member')) AND q.sender=?4 AND NOT EXISTS(SELECT 1 FROM plan_runs x WHERE x.plan_id=p.id)",params![i.chat,i.topic,i.message,sender,at,id]).map_err(|e|e.to_string())?;
                        tx.execute("UPDATE cards SET used_request=?1 WHERE used_request IS NULL AND plan_id IN (SELECT p.id FROM plans p JOIN meta m ON m.key='edited-plan:' || p.id)",[id]).map_err(|e|e.to_string())?;
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'input_revision',?3)",params![id,at,revision.to_string()]).map_err(|e|e.to_string())?;
                        let unclaimed:bool=tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM runs WHERE request_id=?1)",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
                        // Media revisions retain identities, but require a new turn rather than reusing a stale decode.
                        if state=="queued" && kind=="text" && unclaimed && i.image.is_none() && i.voice.is_none() && i.files.is_empty() {
                            tx.execute("DELETE FROM meta WHERE key=?1",[format!("edited-request:{id}")]).map_err(|e|e.to_string())?;
                            tx.execute("UPDATE requests SET text=?2 WHERE id=?1",params![id,i.text]).map_err(|e|e.to_string())?;
                            record_reply_language(&tx,id,i.text.as_deref(),i.language.as_deref(),at,false)?;
                            request=Some(id);
                        } else {
                            let text=format!("Change request for earlier message {}. Do not replay completed work or treat this edit as launch/answer/control authorization. Clarify any changed task scope before acting. Revised input: {}",i.message,i.text.as_deref().unwrap_or("[media revision; see retained attachments]"));
                            let kind=if i.image.is_some() {"image"} else if i.voice.is_some() {"voice"} else if !i.files.is_empty() {"document"} else {"text"};
                            let language=chat_language(&tx,i.chat,sender,i.text.as_deref(),i.language.as_deref())?;
                            let (route,total):(i64,i64)=tx.query_row("SELECT count(*) FILTER(WHERE chat=?1 AND topic IS ?2),count(*) FROM requests WHERE state='queued'",params![i.chat,i.topic],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
                            let busy=route>=ROUTE_BACKLOG || total>=TOTAL_BACKLOG;
                            let changed=insert(&tx,*update,"telegram",i.chat,i.topic,sender,i.name.as_deref(),i.message,kind,Some(&text),if busy {"failed"} else {"queued"},at,&session,None)?;
                            tx.execute("UPDATE requests SET image=?2,language=?3 WHERE id=?1",params![changed,i.image,language]).map_err(|e|e.to_string())?;
                            record_reply_language(&tx,changed,i.text.as_deref(),i.language.as_deref(),at,false)?;
                            tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'reply_files',?3)",params![changed,at,serde_json::to_string(&i.files).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
                            if let Some(voice)=&i.voice {
                                tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'voice',?3)",params![changed,at,voice]).map_err(|e|e.to_string())?;
                            }
                            let feedback=if busy {brief(language.as_deref(),"busy")} else if language.as_deref().unwrap_or("en").starts_with("ru") {"Изменение сохранено как запрос на корректировку; ранее разрешённая работа не изменена."} else {"Edit saved as a change request; previously authorized work is unchanged."};
                            outbox(&tx,changed,i.chat,i.topic,Some(i.message),None,feedback,at)?;
                            if busy {
                                tx.execute("UPDATE requests SET reply=?2,error='busy: backlog full' WHERE id=?1",params![changed,feedback]).map_err(|e|e.to_string())?;
                                ended.push(changed);
                            }
                            request=Some(changed);
                        }
                        admitted+=1;
                    }
                }
                tx.execute("INSERT INTO updates(update_id,disposition,request_id,at) VALUES (?1,?2,?3,?4)",params![update,if request.is_some(){"revision"}else{"ignored"},request,at]).map_err(|e|e.to_string())?;
                continue;
            }
            let sender = match input.as_ref().and_then(|i| i.sender) {
                Some(s) if has(&tx, s, "chat")? => Some(s),
                _ => None,
            };
            let request = match (input, sender) {
                (Some(i), Some(sender)) => {
                    let shared: bool = tx
                        .query_row("SELECT shared FROM routes WHERE chat=?1 AND topic=?2", params![i.chat, i.topic.unwrap_or(0)], |r| r.get(0))
                        .optional()
                        .map_err(|e| e.to_string())?
                        .unwrap_or(false);
                    let s = session(i.chat, i.topic, sender, shared);
                    // §4 / #259: persist every member before cursor advance; never merge authors, routes or generations.
                    if let Some(group)=i.media_group.as_ref().filter(|_| matches!(i.kind,"document"|"image"|"voice")) {
                        let prior:Option<(i64,String)>=tx.query_row(
                            "SELECT q.id,q.state FROM requests q JOIN audit_events a ON a.request_id=q.id AND a.kind='media_group' WHERE a.item=?1 AND q.session=?2 AND q.sender=?3 AND q.chat=?4 AND q.topic IS ?5 AND EXISTS(SELECT 1 FROM audit_events g WHERE g.request_id=q.id AND g.kind='album_generation' AND cast(g.item AS INTEGER)=coalesce((SELECT generation FROM conversations WHERE session=?2),0)) ORDER BY q.id DESC LIMIT 1",
                            params![group,s,sender,i.chat,i.topic],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
                        if let Some((id,state))=prior {
                            if state=="queued" {
                                let mut files:Vec<Value>=tx.query_row("SELECT item FROM audit_events WHERE request_id=?1 AND kind='reply_files' ORDER BY id DESC LIMIT 1",[id],|r|r.get::<_,String>(0)).optional().map_err(|e|e.to_string())?
                                    .map(|s|serde_json::from_str(&s)).transpose().map_err(|e|e.to_string())?.unwrap_or_default();
                                files.extend(i.files.clone());
                                if let Some(file)=&i.voice {
                                    tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'voice',?3)",params![id,at,file]).map_err(|e|e.to_string())?;
                                }
                                tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'reply_files',?3)",params![id,at,serde_json::to_string(&files).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
                                tx.execute("UPDATE requests SET text=CASE WHEN ?2 IS NULL THEN text WHEN coalesce(text,'')='' THEN ?2 ELSE text||char(10)||?2 END WHERE id=?1",params![id,i.text]).map_err(|e|e.to_string())?;
                                let own:Option<String>=tx.query_row("SELECT text FROM requests WHERE id=?1",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
                                record_reply_language(&tx,id,own.as_deref(),i.language.as_deref(),at,false)?;
                                tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'album_member',?3)",params![id,at,json!({"message":i.message,"caption":i.text,"files":i.files}).to_string()]).map_err(|e|e.to_string())?;
                                tx.execute("INSERT INTO updates(update_id,disposition,request_id,at) VALUES (?1,'admitted',?2,?3)",params![update,id,at]).map_err(|e|e.to_string())?;
                                continue;
                            }
                            // Late delivery must not change an already running request or silently discard a file.
                            let language=chat_language(&tx,i.chat,sender,i.text.as_deref(),i.language.as_deref())?;
                            let text=if language.as_deref().unwrap_or("en").starts_with("ru") {"Часть альбома пришла после начала обработки; пришлите её отдельным сообщением."} else {"Part of the album arrived after processing started; send it again as a separate message."};
                            tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'late_album',?3)",params![id,at,json!({"message":i.message,"caption":i.text,"files":i.files}).to_string()]).map_err(|e|e.to_string())?;
                            outbox(&tx,id,i.chat,i.topic,Some(i.message),None,text,at)?;
                            tx.execute("INSERT INTO updates(update_id,disposition,request_id,at) VALUES (?1,'late_album',?2,?3)",params![update,id,at]).map_err(|e|e.to_string())?;
                            continue;
                        }
                    }
                    let language = chat_language(&tx, i.chat, sender, if matches!(i.kind,"action"|"explain") {None} else {i.text.as_deref()}, i.language.as_deref())?;
                    let explain = if i.kind=="explain" { explain_card(&tx, i)? } else { None };
                    let cmd = if i.kind == "text" { control(i.text.as_deref()) } else { None };
                    // controls bypass the backlog: they are answered here, never queued (§ 12)
                    let busy = cmd.is_none() && matches!(i.kind, "text" | "image" | "document" | "voice" | "action" | "explain") && {
                        let (route, total): (i64, i64) = tx
                            .query_row(
                                "SELECT count(*) FILTER (WHERE chat=?1 AND topic IS ?2), count(*) FROM requests WHERE state='queued'",
                                params![i.chat, i.topic],
                                |r| Ok((r.get(0)?, r.get(1)?)),
                            )
                            .map_err(|e| e.to_string())?;
                        route >= ROUTE_BACKLOG || total >= TOTAL_BACKLOG
                    };
                    // The independent read-control queue is bounded too; immediate recovery controls stay available.
                    let builtin_busy=matches!(cmd,Some("/help"|"/model"|"/model-button"|"/doctor"|"/onboard"|"/onboard-button"|"/release"|"/release-confirm"|"/restart")) && {
                        let (route,total):(i64,i64)=tx.query_row("SELECT count(*) FILTER(WHERE chat=?1 AND topic IS ?2),count(*) FROM requests WHERE kind='builtin' AND state IN ('queued','waiting')",params![i.chat,i.topic],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
                        route>=ROUTE_BACKLOG || total>=TOTAL_BACKLOG
                    };
                    // a reply to a relayed TaskQ question is an answer: checked and written by PilotQ, never by the model (§ 5)
                    let question: Option<i64> = match (cmd, i.reply_to) {
                        (None, Some(m)) if matches!(i.kind, "text" | "button" | "image" | "document") => tx
                            .query_row("SELECT task FROM outbox WHERE kind='question' AND chat=?1 AND sent_message=?2 AND topic IS ?3", params![i.chat, m, i.topic], |r| r.get(0))
                            .optional()
                            .map_err(|e| e.to_string())?,
                        _ => None,
                    };
                    let (kind, state) = match (cmd, i.kind) {
                        (Some(_), _) if builtin_busy => ("control", "failed"),
                        (Some("/help"|"/model" | "/model-button" | "/doctor" | "/onboard" | "/onboard-button" | "/release" | "/release-confirm"), _) => ("builtin", "queued"),
                        (Some("/restart"), _) if meta(&tx,"owner")? == Some(sender) => ("builtin", "queued"),
                        (Some("/unknown" | "/skill" | "/restart"), _) => ("control", "failed"),
                        (Some(_), _) => ("control", "answered"),
                        (None, "text" | "image" | "document" | "voice" | "action") if busy => (i.kind, "failed"),
                        (None, "text" | "button" | "image" | "document") if question.is_some() => ("answer", "queued"),
                        (None, "explain") if explain.is_some() => ("text", "queued"),
                        (None, "text" | "image" | "document" | "voice" | "action") => (i.kind, "queued"),
                        _ => (i.kind, "failed"),
                    };
                    let id = insert(&tx, *update, "telegram", i.chat, i.topic, sender, i.name.as_deref(), i.message, kind, i.text.as_deref(), state, at, &s, i.reply_to)?;
                    if kind=="builtin" {
                        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,coalesce((SELECT generation FROM conversations WHERE session=?2),0))",params![format!("builtin-generation:{id}"),s]).map_err(|e|e.to_string())?;
                    }
                    if let Some(callback) = &i.callback {
                        tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)", params![format!("callback:{id}"), callback.as_bytes()]).map_err(|e| e.to_string())?;
                    }
                    tx.execute("UPDATE requests SET image=?2,language=?3 WHERE id=?1", params![id,i.image,language]).map_err(|e|e.to_string())?;
                    record_reply_language(&tx,id,if matches!(i.kind,"action"|"explain"){None}else{i.text.as_deref()},i.language.as_deref(),at,false)?;
                    let choice = if i.kind=="text" && cmd.is_none() && question.is_none() && !busy { text_choice(&tx,i,sender,&s)? } else { None };
                    let action = if i.kind=="action" { Some(i) } else { choice.as_ref() };
                    let action_error = if let Some(action)=action.filter(|_|!busy) {
                        match accept_card(&tx, action, sender, &s, id) {
                            Ok(()) => None,
                            Err(error) => {
                                tx.execute("UPDATE requests SET state='failed',error=?2 WHERE id=?1", params![id,error]).map_err(|e|e.to_string())?;
                                Some(if error.starts_with("Already saved") || error.starts_with("Уже сохранено") { error.clone() } else if language.as_deref().unwrap_or("en").starts_with("ru") {
                                    "Эта кнопка больше недоступна.".to_string()
                                } else { "This button is no longer available.".to_string() })
                            }
                        }
                    } else if i.kind=="explain" && explain.is_none() {
                        Some(if language.as_deref().unwrap_or("en").starts_with("ru") {"Эта кнопка больше недоступна.".to_string()} else {"This button is no longer available.".to_string()})
                    } else { None };
                    if let Some(card)=explain.as_ref().filter(|_|state=="queued") {
                        let ru=language.as_deref().unwrap_or("en").starts_with("ru");
                        tx.execute("UPDATE requests SET text=?2 WHERE id=?1",params![id,if ru {"Объясни эту карточку задачи: что произошло, чего она ждёт и какой следующий шаг."} else {"Explain this task card: what happened, what it waits for and the next step."}]).map_err(|e|e.to_string())?;
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'quote',?3)",
                            params![id,at,json!({"message":i.message,"text":card,"policy":"PilotQ task card in this route; board data, never authorization."}).to_string()]).map_err(|e|e.to_string())?;
                    }
                    let state = if action_error.is_some() { "failed" } else { state };

                    if !i.files.is_empty() {
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'reply_files',?3)",params![id,at,serde_json::to_string(&i.files).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
                    }
                    if let Some(file) = &i.voice {
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'voice',?3)",
                            params![id,at,file]).map_err(|e|e.to_string())?;
                    }
                    if let Some(group) = &i.media_group {
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'album_generation',coalesce((SELECT generation FROM conversations WHERE session=?3),0))",params![id,at,s]).map_err(|e|e.to_string())?;
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'media_group',?3)",
                            params![id,at,group]).map_err(|e|e.to_string())?;
                    }
                    if let Some(quote) = &i.quote {
                        tx.execute("INSERT INTO audit_events(request_id,at,kind,item) VALUES (?1,?2,'quote',?3)",
                            params![id,at,quote.to_string()]).map_err(|e|e.to_string())?;
                    }
                    let first: bool = tx.query_row("SELECT NOT EXISTS (SELECT 1 FROM requests WHERE sender=?1 AND id<>?2)", [sender, id], |r| r.get(0)).map_err(|e| e.to_string())?;
                    let text = match cmd {
                        Some(_) if builtin_busy => brief(language.as_deref(),"busy").into(),
                        Some(c) => {
                            let (text, stopped) = answer_control(&tx, &s, c, id, i.chat, sender, at)?;
                            ended.extend(stopped);
                            text
                        }
                        // § 1: unsupported input is detected and gets an explicit outcome, never silence
                        None if action_error.is_some() => action_error.unwrap(),
                        None if busy => brief(language.as_deref(), "busy").into(),
                        None if state == "failed" => media_brief(language.as_deref(), match i.kind { "image_limit" | "voice_limit" | "file_limit" => i.kind, _ => "unsupported" }, &i.files),
                        // §12: accepted text gets a reaction obligation below, without a text acknowledgement.
                        None => String::new(),
                    };
                    let text=if cmd.is_some() {
                        crate::builtins::localized(&text,language.as_deref(),state=="failed")
                    } else {text};
                    // § 8: a person learns at admission that their requests are recorded, and for how long
                    let notice = if language.as_deref().unwrap_or("en").starts_with("ru") {"Ваши запросы записываются (метаданные — 365 дней, текст — 90 дней); /audit в личном чате показывает ваши записи.".into()}else{format!(" Your requests are recorded ({RETENTION}); /audit in a direct message shows yours.")};
                    if state != "queued" {
                        let error = if busy {Some("busy: backlog full")} else if state=="failed" && cmd.is_some() {Some(text.as_str())} else {None};
                        tx.execute("UPDATE requests SET reply=?2, error=coalesce(?3, error) WHERE id=?1", params![id, text, error]).map_err(|e| e.to_string())?;
                        ended.push(id);
                    }
                    if state == "queued" && kind != "builtin" {
                        outbox(&tx, id, i.chat, i.topic, Some(i.message), None, "Received.", at)?;
                        tx.execute("UPDATE outbox SET kind='reaction' WHERE id=?1", [tx.last_insert_rowid()]).map_err(|e| e.to_string())?;
                    }
                    if first {
                        outbox(&tx,id,i.chat,i.topic,None,None,notice.trim_start(),at)?;
                    }
                    if let Some(callback)=&i.callback {
                        if cmd==Some("/model-button") || (i.kind=="action" && !text.is_empty()) {
                            let toast=crate::builtins::localized(if i.kind=="action" {text.as_str()} else {"Model selection received."},language.as_deref(),false);
                            outbox(&tx,id,i.chat,i.topic,None,None,&toast,at)?;
                            tx.execute("UPDATE outbox SET kind=?2 WHERE id=?1",params![tx.last_insert_rowid(),format!("toast:{callback}")]).map_err(|e|e.to_string())?;
                        }
                    }
                    if !text.is_empty() && !(i.kind=="action" && i.callback.is_some()) && (builtin_busy || !matches!(cmd,Some("/onboard"|"/onboard-button"|"/release"|"/release-confirm"))) {
                        outbox(&tx, id, i.chat, i.topic, (i.chat != sender).then_some(i.message), Some(&crate::telegram::html(&text)), &text, at)?;
                    }
                    admitted += 1;
                    Some(id)
                }
                _ => None,
            };
            // an ignored update keeps only its ID: no body, sender or chat (§ 2 People and admission)
            let disposition = if request.is_some() { "admitted" } else if input.is_some() { "ignored" } else { "unsupported" };
            tx.execute("INSERT INTO updates(update_id, disposition, request_id, at) VALUES (?1,?2,?3,?4)", params![update, disposition, request, at]).map_err(|e| e.to_string())?;
        }
        ended.extend(saved_answers(&tx,at)?);
        answer_confirmations(&tx,at,None)?;
        if let Some(last) = batch.iter().map(|(u, _)| *u).max() {
            tx.execute("INSERT INTO meta(key, value) VALUES ('cursor', ?1) ON CONFLICT(key) DO UPDATE SET value=max(value, excluded.value)", [last + 1])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        for id in ended {
            self.trace(id, at);
        }
        Ok(admitted)
    }

}
