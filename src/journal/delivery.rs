//! Delivery persistence (pilotq.md §§2,4–9).
use super::*;

impl Journal {
    pub fn route_held(&self, chat:i64, topic:Option<i64>) -> Result<bool,String> {
        self.0.query_row("SELECT EXISTS(SELECT 1 FROM meta WHERE key=?1 OR key=?2)",params![format!("route-hold:{chat}:0"),format!("route-hold:{chat}:{}",topic.unwrap_or(0))],|r|r.get(0)).map_err(|e|e.to_string())
    }

    /// §4: Telegram retry_after applies across methods and topics in the same chat, including restart.
    pub fn telegram_cooldown(&self, chat: i64) -> Result<i64, String> {
        self.0.query_row("SELECT value FROM meta WHERE key=?1", [format!("telegram-cooldown:{chat}")], |r| r.get(0))
            .optional().map(|v|v.unwrap_or(0)).map_err(|e|e.to_string())
    }

    pub fn cool_telegram(&self, chat: i64, until: i64) -> Result<(), String> {
        self.0.execute("INSERT INTO meta(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=max(value,excluded.value)",
            params![format!("telegram-cooldown:{chat}"),until]).map(drop).map_err(|e|e.to_string())
    }

    /// R11 / #190: one durable, editable progress message per active request, at most every 30 s.
    pub(super) fn turn_progress(&self, at: i64) -> Result<(), String> {
        self.0.execute("UPDATE outbox SET state='superseded' WHERE kind='turn_progress' AND state='pending'
            AND request_id IN (SELECT id FROM requests WHERE state<>'running')",[]).map_err(|e|e.to_string())?;
        // #256 A6: a delivered progress message is removed once the run ends, queued behind the final reply of its route.
        self.0.execute("INSERT INTO outbox(request_id,chat,topic,text,next_at,kind)
            SELECT DISTINCT o.request_id,o.chat,o.topic,'',?1,'turn_progress_closed' FROM outbox o JOIN requests q ON q.id=o.request_id
            WHERE o.kind='turn_progress' AND o.state='delivered' AND q.state<>'running'
            AND NOT EXISTS(SELECT 1 FROM outbox c WHERE c.request_id=o.request_id AND c.kind='turn_progress_closed')",[at]).map_err(|e|e.to_string())?;
        let mut q=self.0.prepare("SELECT q.id,q.chat,q.topic,nullif(q.message,0),q.language,r.started_at
            FROM requests q JOIN runs r ON r.request_id=q.id
            WHERE q.front='telegram' AND q.state='running' AND r.state='running' AND r.started_at<=?1-30
            AND NOT EXISTS(SELECT 1 FROM audit_events a WHERE a.request_id=q.id AND a.kind='loop_tick')
            AND NOT EXISTS(SELECT 1 FROM outbox o WHERE o.request_id=q.id AND o.kind='turn_progress'
                AND (o.state='pending' OR o.next_at>?1-30))").map_err(|e|e.to_string())?;
        let rows:Vec<(i64,i64,Option<i64>,Option<i64>,Option<String>,i64)>=q.query_map([at],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).and_then(Iterator::collect).map_err(|e|e.to_string())?;
        for (request,chat,topic,reply,language,start) in rows {
            let age=crate::time::age(at.saturating_sub(start));
            let text=if language.as_deref()==Some("ru") {format!("Работаю · {}",crate::footer::russian_units(&age))} else {format!("Working · {age}")};
            self.0.execute("INSERT INTO outbox(request_id,chat,topic,reply_to,text,next_at,kind) VALUES (?1,?2,?3,?4,?5,?6,'turn_progress')",params![request,chat,topic,reply,text,at]).map_err(|e|e.to_string())?;
        }
        Ok(())
    }

    /// §§2,4 / #334: recover presentation from exact schema-13 receipts, never board mutations.
    pub(super) fn retire_question_controls(&self, at: i64) -> Result<(),String> {
        let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        // A delayed accepted answer wins over an unsent supersession, including during retries.
        tx.execute("UPDATE outbox AS o SET state='superseded' WHERE kind='question_superseded' AND state='pending'
            AND EXISTS(SELECT 1 FROM outbox a WHERE a.request_id=o.request_id AND a.project=o.project AND a.task=o.task
                AND a.chat=o.chat AND a.topic IS o.topic AND a.revision=o.revision AND a.kind='question_answered')",[]).map_err(|e|e.to_string())?;
        // Recheck registration, original route and current access both when queuing and before dispatch.
        let authorized="EXISTS(SELECT 1 FROM tasks t JOIN requests q ON q.id=t.request_id
            JOIN projects p ON p.name=t.project AND p.repo=t.board AND p.root=t.root
            WHERE t.request_id=o.request_id AND t.project=o.project AND t.number=o.task AND t.board<>'' AND t.root<>''
            AND t.chat=o.chat AND t.topic IS o.topic AND q.chat=o.chat AND q.topic IS o.topic AND q.front='telegram'
            AND EXISTS(SELECT 1 FROM grants g WHERE g.user=q.sender AND g.name='chat')
            AND EXISTS(SELECT 1 FROM grants g WHERE g.user=q.sender AND g.name IN ('admin','projects','project:' || o.project,'owner:' || o.project)))";
        tx.execute(&format!("UPDATE outbox AS o SET state='blocked',error='question recovery provenance or access unavailable'
            WHERE kind='question_superseded' AND state='pending' AND reply_to IS NOT NULL AND NOT {authorized}"),[]).map_err(|e|e.to_string())?;
        // ponytail: at most 64 repair obligations per scan; the existing delivery loop continues the backlog.
        tx.execute(&format!("INSERT INTO outbox(request_id,chat,topic,reply_to,text,next_at,kind,project,task,revision)
            SELECT o.request_id,o.chat,o.topic,o.sent_message,o.text || char(10) || char(10) ||
                CASE WHEN (SELECT coalesce(language,'en') FROM requests WHERE id=o.request_id) LIKE 'ru%'
                THEN 'Вопрос заменён новым; используйте текущий вопрос.' ELSE 'Superseded by a newer question; use the current question.' END,
                ?1,'question_superseded',o.project,o.task,o.revision FROM outbox o
            WHERE o.kind='question' AND o.state='delivered' AND o.sent_message>0 AND o.revision>0
            AND json_valid(o.question) AND json_extract(o.question,'$.parsed')=1 AND {authorized}
            AND (SELECT count(*) FROM outbox s WHERE s.request_id=o.request_id AND s.project=o.project AND s.task=o.task
                AND s.chat=o.chat AND s.topic IS o.topic AND s.revision=o.revision AND s.kind='question')=1
            AND EXISTS(SELECT 1 FROM outbox n WHERE n.request_id=o.request_id AND n.project=o.project AND n.task=o.task
                AND n.chat=o.chat AND n.topic IS o.topic AND n.kind='question' AND n.id>o.id AND n.revision>o.revision
                AND n.state IN ('pending','delivered') AND json_valid(n.question) AND json_extract(n.question,'$.parsed')=1)
            AND NOT EXISTS(SELECT 1 FROM outbox a WHERE a.request_id=o.request_id AND a.project=o.project AND a.task=o.task
                AND a.chat=o.chat AND a.topic IS o.topic AND a.revision=o.revision AND a.kind IN ('question_answered','question_superseded'))
            ORDER BY o.id LIMIT 64"),[at]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())
    }

    /// The next delivery due at `now`, else the earliest time one becomes due. Only the oldest pending row of a route
    /// (chat and topic) is eligible, so a retrying head keeps later replies in that route behind it.
    pub fn due(&self, now: i64) -> Result<(Option<Delivery>, Option<i64>), String> {
        let completed={
            let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
            let completed=saved_answers(&tx,now)?;
            answer_confirmations(&tx,now,None)?;
            tx.commit().map_err(|e|e.to_string())?;
            completed
        };
        for id in completed {self.trace(id,now);}
        self.retire_question_controls(now)?;
        crate::result_images::prune(self,now)?;
        self.turn_progress(now)?;
        self.launch_progress(now)?;
        self.task_progress(now)?;
        for id in revalidate(&self.0)? {
            self.trace(id, now);
        }
        // Timers only wake this scan; the request and one-notice disposition are durable across restart.
        self.0.execute("INSERT INTO outbox(request_id,chat,topic,reply_to,text,next_at,kind)
            SELECT r.id,r.chat,r.topic,CASE WHEN r.chat<>r.sender THEN r.message END,'Queued behind ' || (SELECT count(*) FROM requests p WHERE p.session=r.session AND p.id<r.id AND p.state IN ('queued','running')) || '.',?1,'queued'
            FROM requests r WHERE r.front='telegram' AND r.state='queued' AND r.kind<>'builtin' AND r.received_at<?1-30 AND NOT EXISTS(SELECT 1 FROM audit_events WHERE request_id=r.id AND kind='loop_tick')
            AND NOT EXISTS (SELECT 1 FROM outbox o WHERE o.request_id=r.id AND o.kind='queued')", [now]).map_err(|e|e.to_string())?;
        self.0.execute("UPDATE outbox SET state='superseded' WHERE kind='queued' AND state='pending' AND request_id IN (SELECT id FROM requests WHERE state<>'queued')", []).map_err(|e|e.to_string())?;
        let heads = "FROM outbox o WHERE state='pending'
            AND NOT EXISTS(SELECT 1 FROM meta h WHERE h.key='route-hold:' || o.chat || ':0' OR h.key='route-hold:' || o.chat || ':' || coalesce(o.topic,0)) AND (o.kind='reaction' OR NOT EXISTS
            (SELECT 1 FROM outbox p WHERE p.state='pending' AND p.chat=o.chat AND p.topic IS o.topic AND p.id<o.id))";
        let mut row = self
            .0
            .query_row(
                // a status row edits the task's latest delivered status message, resolved now, after every earlier row of its route
                &format!(
                    "SELECT id, chat, topic, reply_to, html, text, attempts, CASE WHEN kind='status' THEN (SELECT s.sent_message FROM outbox s WHERE s.request_id=o.request_id AND s.chat=o.chat AND s.topic IS o.topic
                        AND s.kind='status' AND s.state='delivered' ORDER BY s.id DESC LIMIT 1)
                        WHEN kind IN ('turn_progress','turn_progress_closed') THEN (SELECT s.sent_message FROM outbox s WHERE s.request_id=o.request_id AND s.kind='turn_progress' AND s.state='delivered' ORDER BY s.id DESC LIMIT 1)
                        WHEN kind='release_result' THEN (SELECT s.sent_message FROM outbox s WHERE s.request_id=o.revision AND s.kind='release_preview' AND s.state='delivered' ORDER BY s.id DESC LIMIT 1)
                        WHEN kind='onboarding' THEN (SELECT s.sent_message FROM outbox s WHERE s.request_id=o.request_id AND s.kind='onboarding' AND s.state='delivered' ORDER BY s.id DESC LIMIT 1)
                        WHEN kind='answer_confirmation' THEN (SELECT s.sent_message FROM outbox s WHERE s.request_id=o.request_id AND s.kind='queued' AND s.state='delivered' AND s.chat=o.chat AND s.topic IS o.topic ORDER BY s.id DESC LIMIT 1)
                        WHEN kind='choice_answered' THEN o.reply_to
                        WHEN kind='question_superseded' AND o.reply_to IS NOT NULL THEN o.reply_to
                        WHEN kind IN ('question_answered','question_superseded','question_explained') THEN (SELECT s.sent_message FROM outbox s WHERE s.request_id=o.request_id AND s.task=o.task AND s.project=o.project AND s.chat=o.chat AND s.topic IS o.topic
                        AND s.kind='question' AND s.revision=o.revision AND s.state='delivered' ORDER BY s.id DESC LIMIT 1)
                        WHEN kind IN ('draft_closed','plan_answered','plan_progress','plan_problem') THEN (SELECT s.sent_message FROM outbox s JOIN cards c ON c.id=o.revision WHERE ((s.request_id=c.request_id AND s.kind IN ('rich','rich_part')) OR (s.revision=c.id AND s.kind IN ('plan_answered','plan_progress','plan_problem'))) AND s.state='delivered' ORDER BY s.id DESC LIMIT 1) END, kind {heads} AND next_at<=?1 AND coalesce((SELECT value FROM meta WHERE key='telegram-cooldown:' || o.chat),0)<=?1 ORDER BY (kind='reaction') DESC,id LIMIT 1"
                ),
                [now],
                |r| Ok(Delivery { id: r.get(0)?, chat: r.get(1)?, topic: r.get(2)?, reply_to: r.get(3)?, html: r.get(4)?, text: r.get(5)?, attempts: r.get(6)?, edit: r.get(7)?, kind: r.get(8)?, buttons: vec![], rich: false }),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(d) = &mut row {
            d.buttons = if matches!(d.kind.as_str(),"question"|"question_explained") {self.question(d.id)?.map_or_else(Vec::new,|q|crate::telegram::options(&q))} else {self.card_buttons(d.id)?};
            self.prepare_delivery(d)?;
        }
        let next = self.0.query_row(&format!("SELECT min(next_at) {heads}"), [], |r| r.get(0)).map_err(|e| e.to_string())?;
        Ok((row, next))
    }

    /// Prepare content independently of card authority; callers resolve buttons before sending.
    pub(super) fn prepare_delivery(&self,d:&mut Delivery)->Result<(),String> {
        if d.kind=="result_image" { return Ok(()); }
        // Editing an answer/plan card must not erase the speech retained on that card.
        if let Some(message)=d.edit {
            let source:Option<String>=self.0.query_row("SELECT text FROM outbox WHERE chat=?1 AND topic IS ?2 AND sent_message=?3 AND text LIKE '> %' AND EXISTS(SELECT 1 FROM audit_events WHERE request_id=outbox.request_id AND kind='transcript') ORDER BY id LIMIT 1",params![d.chat,d.topic,message],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
            if let Some(source)=source {
                let quote=source.split_once("\n\n").map_or(source.as_str(),|s|s.0);
                if !d.text.starts_with(quote) {
                    d.text=format!("{quote}\n\n{}",d.text);
                    d.html=Some(crate::telegram::html(&d.text));
                }
            }
        }
        d.rich = d.kind=="rich";
        if shared_message(&d.kind) && !matches!(d.kind.as_str(),"rich"|"rich_part") {
            let key=format!("delivery:{}",d.id);
            let stored:Option<String>=self.0.query_row("SELECT value FROM meta WHERE key=?1",[&key],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
            let progress:Value=if let Some(stored)=stored {serde_json::from_str(&stored).map_err(|e|format!("delivery progress: {e}"))?} else {
                let progress=json!({"mode":if d.html.is_none() && d.attempts>0 {"plain"} else if d.text.encode_utf16().count()<=32768 {"rich"} else {"html"},"parts":chunks(&d.text),"index":0,"receipts":[]});
                self.0.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![key,progress.to_string()]).map_err(|e|e.to_string())?;
                progress
            };
            if !matches!(progress["mode"].as_str(),Some("rich"|"html"|"plain")) {return Err("invalid delivery mode".into());}
            d.rich=progress["mode"]=="rich";
            if !d.rich {
                let index=progress["index"].as_u64().ok_or("invalid delivery index")? as usize;
                d.text=progress["parts"][index].as_str().ok_or("missing delivery chunk")?.into();
                d.html=if progress["mode"]=="plain" {None} else {Some(crate::telegram::html(&d.text))};
                if index>0 {d.edit=None;}
                if index+1<progress["parts"].as_array().ok_or("invalid delivery parts")?.len() {d.buttons.clear();}
            }
        }
        Ok(())
    }

    /// Records one send attempt. Retries stop at `ATTEMPTS`; the row stays for operator recovery, never deleted.
    pub fn record(&mut self, id: i64, sent: &Sent, now: i64) -> Result<(), String> {
        let uncertain: bool = self.0.query_row("SELECT possible_duplicate FROM outbox WHERE id=?1", [id], |r|r.get(0)).map_err(|e|e.to_string())?;
        let retry = Sent::Retry { after: 1, ambiguous: true, error: "rich rejection after ambiguous send; fallback blocked".into() };
        let sent = if uncertain && matches!(sent, Sent::RichFallback) { &retry } else { sent };
        if let Sent::Retry { after, error, .. } = sent {
            if error == "Telegram retry-after" {
                let chat = self.0.query_row("SELECT chat FROM outbox WHERE id=?1", [id], |r|r.get(0)).map_err(|e|e.to_string())?;
                self.cool_telegram(chat, now.saturating_add(*after).saturating_add(1))?;
            }
        }
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        // #229 A3 / R5: a late send result must retain its evidence without reviving stopped tick output.
        let retired_tick:bool=tx.query_row("SELECT state='superseded' AND EXISTS(SELECT 1 FROM audit_events a WHERE a.request_id=o.request_id AND a.kind='loop_tick') FROM outbox o WHERE id=?1",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
        if retired_tick {
            tx.execute("INSERT INTO audit_events(request_id,at,kind,item,result) SELECT request_id,?2,'loop_delivery_retired',?1,?3 FROM outbox WHERE id=?1",params![id,now,format!("{sent:?}")]).map_err(|e|e.to_string())?;
            tx.execute("UPDATE outbox SET attempts=attempts+1,sent_message=coalesce(?2,sent_message),possible_duplicate=possible_duplicate OR ?3 WHERE id=?1",params![id,if let Sent::Ok(m)=sent {Some(*m)} else {None},matches!(sent,Sent::Retry{ambiguous:true,..})]).map_err(|e|e.to_string())?;
            return tx.commit().map_err(|e|e.to_string());
        }
        if matches!(sent,Sent::Held(_)) {
            let (chat,topic)=tx.query_row("SELECT chat,topic FROM outbox WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
            hold_route(&tx,chat,topic,"Telegram destination unavailable; administrator revalidation required",now)?;
        }
        let key=format!("delivery:{id}");
        let progress:Option<String>=tx.query_row("SELECT value FROM meta WHERE key=?1",[&key],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if let Some(progress)=progress {
            let mut progress:Value=serde_json::from_str(&progress).map_err(|e|e.to_string())?;
            if matches!(sent,Sent::RichFallback|Sent::Plain) {
                progress["mode"]=json!(if matches!(sent,Sent::Plain) {"plain"} else {"html"});
                tx.execute("UPDATE meta SET value=?2 WHERE key=?1",params![key,progress.to_string()]).map_err(|e|e.to_string())?;
                tx.execute("UPDATE outbox SET next_at=?2,error='format rejected; fallback' WHERE id=?1",params![id,now]).map_err(|e|e.to_string())?;
                if matches!(sent,Sent::RichFallback) {return tx.commit().map_err(|e|e.to_string());}
            }
            if let Sent::Ok(message)=sent {
                if progress["mode"]!="rich" {
                    let index=progress["index"].as_u64().ok_or("invalid delivery index")? as usize;
                    progress["receipts"].as_array_mut().ok_or("invalid delivery receipts")?.push(json!(message));
                    progress["index"]=json!(index+1);
                    tx.execute("UPDATE meta SET value=?2 WHERE key=?1",params![key,progress.to_string()]).map_err(|e|e.to_string())?;
                    if index+1<progress["parts"].as_array().ok_or("invalid delivery parts")?.len() {
                        tx.execute("UPDATE outbox SET next_at=?2,attempts=0,possible_duplicate=0 WHERE id=?1",params![id,now]).map_err(|e|e.to_string())?;
                        return tx.commit().map_err(|e|e.to_string());
                    }
                }
            }
        }
        if matches!(sent, Sent::RichFallback | Sent::Receipt) {
            if matches!(sent, Sent::RichFallback) {
                tx.execute("UPDATE outbox SET state='pending' WHERE kind='rich_part' AND state='standby' AND request_id=(SELECT request_id FROM outbox WHERE id=?1)", [id]).map_err(|e|e.to_string())?;
                tx.execute("UPDATE outbox SET state='superseded',error='rich message rejected; HTML fallback' WHERE id=?1", [id]).map_err(|e|e.to_string())?;
            } else {
                tx.execute("UPDATE outbox SET kind='receipt',next_at=?2,reply_to=CASE WHEN chat=(SELECT sender FROM requests WHERE id=request_id) THEN NULL ELSE reply_to END,error='reaction failed; text receipt' WHERE id=?1", params![id, now]).map_err(|e|e.to_string())?;
            }
            return tx.commit().map_err(|e|e.to_string());
        }
        if matches!(sent, Sent::Ok(_)) {
            tx.execute("UPDATE outbox SET state='superseded' WHERE kind='rich_part' AND state='standby' AND request_id=(SELECT request_id FROM outbox WHERE id=?1 AND kind='rich')", [id]).map_err(|e|e.to_string())?;
        }
        let attempts: i64 = tx.query_row("UPDATE outbox SET attempts=attempts+1 WHERE id=?1 RETURNING attempts", [id], |r| r.get(0)).map_err(|e| e.to_string())?;
        let (state, after, error, ambiguous, message) = match sent {
            Sent::RichFallback | Sent::Receipt => unreachable!(),
            Sent::Ok(m) => ("delivered", 0, None, false, Some(*m)),
            Sent::Plain => ("pending", 0, Some("formatting rejected; plain text".to_string()), false, None),
            Sent::Retry { after, ambiguous, error } => ("pending", *after, Some(error.clone()), *ambiguous, None),
            Sent::Held(e) => ("held", 0, Some(e.clone()), false, None),
            Sent::Failed(e) => ("failed", 0, Some(e.clone()), false, None),
        };
        let state = if state == "pending" && attempts >= ATTEMPTS { "exhausted" } else { state };
        tx.execute(
            "UPDATE outbox SET state=?2, next_at=?3, error=coalesce(?4, error), possible_duplicate=possible_duplicate OR ?5, sent_message=?6,
                html=CASE WHEN ?7 THEN NULL ELSE html END WHERE id=?1",
            params![id, state, now + after, error, ambiguous, message, matches!(sent, Sent::Plain)],
        )
        .map_err(|e| e.to_string())?;
        if matches!(state,"failed"|"exhausted") {
            tx.execute("INSERT INTO outbox(request_id,chat,topic,text,next_at,kind,project,task) SELECT request_id,chat,topic,project || ' #' || task || ': image could not be delivered; result images are incomplete.',?2,'result_image_gap',project,task FROM outbox o WHERE id=?1 AND kind='result_image' AND NOT EXISTS(SELECT 1 FROM outbox g WHERE g.request_id=o.request_id AND g.kind='result_image_gap')",params![id,now]).map_err(|e|e.to_string())?;
            tx.execute("INSERT OR IGNORE INTO meta(key,value) SELECT 'result-image-binding:' || g.id,m.value FROM outbox o JOIN outbox g ON g.request_id=o.request_id AND g.kind='result_image_gap' JOIN meta m ON m.key='result-image-binding:' || o.id WHERE o.id=?1",[id]).map_err(|e|e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

}
