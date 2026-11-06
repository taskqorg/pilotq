//! Tasks persistence (pilotq.md §§2,4–9).
use super::*;

impl Journal {
    /// Immutable plan target; a conversation switch cannot retarget approved work (§5). Without a revision, the current draft's
    /// latest one (#110: a text launch maps to the current draft only).
    pub fn plan_target(&self, session: &str, revision: Option<i64>) -> Result<(i64, String, String, String), String> {
        self.0.query_row("SELECT revision, project, board, root FROM plans WHERE session=?1 AND revision=coalesce(?2,(SELECT max(revision) FROM plans WHERE session=?1))",
            params![session, revision], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?))).map_err(|_| "this conversation has no saved plan at that revision; nothing was filed".into())
    }

    pub fn task_target_matches(&self, project: &str, task: i64, board: &str, root: &str) -> Result<bool, String> {
        self.0.query_row("SELECT board=?3 AND (root='' OR root=?4) FROM tasks WHERE project=?1 AND number=?2", params![project, task, board, root], |x| x.get(0)).map_err(|e| e.to_string())
    }

    /// Durable exact TaskQ mutation intent, before any board write (§4, §5).
    pub fn task_intent(&self, r: &Run, project: &str, board: &str, root: &str, args: &[String], at: i64) -> Result<String, String> {
        if self.fenced(r)? { return Err("this run was stopped; nothing was written".into()); }
        let count: i64 = self.0.query_row("SELECT count(*) FROM task_effects WHERE request_id=?1", [r.request], |x| x.get(0)).map_err(|e| e.to_string())?;
        let op = format!("pilotq-{}-r{}-tq{}", self.instance()?, r.request, count + 1);
        self.0.execute("INSERT INTO task_effects(op,request_id,project,board,root,args,at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![op,r.request,project,board,root,serde_json::to_string(args).map_err(|e| e.to_string())?,at]).map_err(|e| e.to_string())?;
        Ok(op)
    }

    pub fn task_receipt(&self, op: &str, receipt: &Value, notification: Option<&str>, at: i64) -> Result<(), String> {
        let tx = self.0.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute("UPDATE task_effects SET receipt=?2 WHERE op=?1", params![op,receipt.to_string()]).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) SELECT request_id,?2,'task',board || ' ' || args,op,?3 FROM task_effects WHERE op=?1", params![op,at,receipt.to_string()]).map_err(|e| e.to_string())?;
        tx.execute("UPDATE requests SET project=(SELECT project FROM task_effects WHERE op=?1) WHERE id=(SELECT request_id FROM task_effects WHERE op=?1)", [op]).map_err(|e| e.to_string())?;
        if let Some(text) = notification {
            // Recovery shares the live refusal projection; receipt diagnostics remain above in audit.
            let language:Option<String>=tx.query_row("SELECT q.language FROM requests q JOIN task_effects e ON e.request_id=q.id WHERE e.op=?1",[op],|r|r.get(0)).map_err(|e|e.to_string())?;
            let localized=crate::conversation::refusal(language.as_deref());
            let text=if receipt["outcome"]=="accepted" {text} else {localized};
            tx.execute("UPDATE requests SET state=CASE WHEN ?2='accepted' THEN 'answered' ELSE 'failed' END,reply=?3,error=NULL WHERE kind='answer' AND id=(SELECT request_id FROM task_effects WHERE op=?1)",params![op,receipt["outcome"].as_str(),text]).map_err(|e|e.to_string())?;
            let (request, chat, topic, front): (i64, i64, Option<i64>, String) = tx.query_row("SELECT q.id,q.chat,q.topic,q.front FROM requests q JOIN task_effects e ON e.request_id=q.id WHERE e.op=?1", [op], |x| Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?))).map_err(|e| e.to_string())?;
            if front == "telegram" { outbox(&tx, request, chat, topic, None, None, text, at)?; }
        }
        tx.commit().map_err(|e| e.to_string())
    }

    /// Buttons/replies use the same durable effect store and reconciliation as semantic answers.
    pub fn direct_answer_intent(&self, a: &Answer, board: &str, root: &str, args: &[String], op: &str, at: i64) -> Result<(),String> {
        self.0.execute("INSERT INTO task_effects(op,request_id,project,board,root,args,at) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(op) DO NOTHING",
            params![op,a.request,a.project,board,root,serde_json::to_string(args).map_err(|e|e.to_string())?,at]).map(drop).map_err(|e|e.to_string())
    }

    pub fn task_effect_receipt(&self, op: &str) -> Result<Option<Value>,String> {
        let raw:Option<String>=self.0.query_row("SELECT receipt FROM task_effects WHERE op=?1",[op],|r|r.get(0)).map_err(|e|e.to_string())?;
        raw.map(|s|serde_json::from_str(&s).map_err(|e|e.to_string())).transpose()
    }

    pub fn effect_dispatchable(&self, op: &str) -> Result<bool,String> {
        self.0.query_row("SELECT q.state<>'cancelled' AND NOT EXISTS(SELECT 1 FROM audit_events WHERE request_id=q.id AND kind='drain_retired') AND NOT EXISTS(SELECT 1 FROM runs r JOIN conversations c ON c.session=r.session WHERE r.request_id=q.id AND r.state='running' AND (r.generation<>c.generation OR r.fence<>c.fence)) FROM task_effects t JOIN requests q ON q.id=t.request_id WHERE t.op=?1",[op],|r|r.get(0)).map_err(|e|e.to_string())
    }

    pub fn task_effect_pending(&self, op: &str) -> Result<bool,String> {
        self.0.query_row("SELECT receipt IS NULL FROM task_effects WHERE op=?1",[op],|x|x.get(0)).map_err(|e|e.to_string())
    }

    pub fn language(&self, request: i64) -> Result<Option<String>,String> {
        self.0.query_row("SELECT language FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(|e|e.to_string())
    }

    pub fn answer_effect_owner(&self, op: &str) -> Result<(i64,String),String> {
        self.0.query_row("SELECT request_id,project FROM task_effects WHERE op=?1",[op],|x|Ok((x.get(0)?,x.get(1)?))).map_err(|e|e.to_string())
    }

    pub fn pending_task_effects(&self) -> Result<Vec<(String, i64, String, String, String, Vec<String>)>, String> {
        let mut q = self.0.prepare("SELECT op,request_id,project,board,root,args FROM task_effects e WHERE receipt IS NULL AND NOT EXISTS (SELECT 1 FROM runs r WHERE r.request_id=e.request_id AND r.state='running')").map_err(|e| e.to_string())?;
        let rows = q.query_map([], |x| Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?,x.get(4)?,x.get::<_,String>(5)?))).map_err(|e| e.to_string())?;
        rows.map(|row| { let (op,r,p,b,root,args) = row.map_err(|e| e.to_string())?; Ok((op,r,p,b,root,serde_json::from_str(&args).map_err(|e| e.to_string())?)) }).collect()
    }

    /// Saves the next plan revision of the run's conversation while its fence holds (§ 5 `task_plan`). No board write.
    #[cfg(test)]
    pub fn save_plan(&self, r: &Run, tickets: &Value, project: &str, board: &str, root: &str, at: i64) -> Result<i64, String> {
        self.save_draft(r, tickets, project, board, root, at)
    }

    /// #110: one open draft per conversation. Every save revises it until it is launched or cancelled; only then does a new draft start.
    pub fn save_draft(&self, r: &Run, tickets: &Value, project: &str, board: &str, root: &str, at: i64) -> Result<i64, String> {
        if self.fenced(r)? { return Err("this run was stopped; nothing was saved".into()); }
        let tx = self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        if meta(&tx,&format!("edited-request:{}",r.request))?.is_some() {
            return Err("the source input was edited; only the current revision can save a new proposal".into());
        }
        let previous: Option<(i64,String,String,String,bool)> = tx.query_row(
            "SELECT draft,project,board,root,EXISTS(SELECT 1 FROM plan_runs x JOIN plans p ON p.id=x.plan_id WHERE p.draft=plans.draft) FROM plans WHERE session=?1 ORDER BY revision DESC LIMIT 1",
            [&r.session], |x| Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?,x.get(4)?))).optional().map_err(|e|e.to_string())?;
        if let Some((_,p,..)) = previous.as_ref().filter(|(_,p,b,rt,closed)| !closed && (p!=project || b!=board || rt!=root)) {
            return Err(format!("the open draft targets {p}; launch or cancel it before planning another project. Nothing was saved."));
        }
        let draft = previous.filter(|(_,_,_,_,closed)| !closed).map(|p|p.0);
        let revision = tx.query_row(
            "INSERT INTO plans(session,revision,request_id,tickets,project,board,root,at,draft) SELECT ?1,coalesce(max(revision),0)+1,?2,?3,?4,?5,?6,?7,?8 FROM plans WHERE session=?1 RETURNING revision",
            params![r.session,r.request,tickets.to_string(),project,board,root,at,draft], |x|x.get::<_,i64>(0)).map_err(|e|e.to_string())?;
        tx.execute("UPDATE plans SET draft=id WHERE draft IS NULL",[]).map_err(|e|e.to_string())?;
        // #257: admission rows (reaction, retention notice) predate the plan; they belong to its project too.
        tx.execute("UPDATE outbox SET project=?2 WHERE request_id=?1 AND task IS NULL",params![r.request,project]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())?;
        Ok(revision)
    }

    /// Pending attachment copies live beside the journal; unresolved effects are retained under §7.
    pub fn reply_cache(&self, op: &str) -> Result<PathBuf,String> {
        if !op.chars().all(|c|c.is_ascii_alphanumeric() || "-_.".contains(c)) || op=="." || op==".." {return Err("invalid attachment operation".into());}
        Ok(self.1.parent().ok_or("journal directory unavailable")?.join("reply-files").join(op))
    }

    pub fn reply_budget(&self, op: &str, at: i64) -> Result<std::time::Duration,String> {
        let start:i64=self.0.query_row("SELECT coalesce((SELECT min(started_at) FROM runs WHERE request_id=t.request_id),t.at) FROM task_effects t WHERE op=?1",[op],|r|r.get(0)).map_err(|e|e.to_string())?;
        Ok(std::time::Duration::from_secs((60-at.saturating_sub(start).max(0)).max(0) as u64))
    }

    pub fn reply_files(&self, request: i64) -> Result<Vec<Value>,String> {
        let raw:Option<String>=self.0.query_row("SELECT item FROM audit_events WHERE request_id=?1 AND kind='reply_files' ORDER BY id DESC LIMIT 1",[request],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        raw.map(|s|serde_json::from_str(&s).map_err(|e|e.to_string())).unwrap_or(Ok(vec![]))
    }

    /// #186: files a claimed document request carries for its model turn; an image request's file is read as an image.
    pub fn documents(&self, request: i64) -> Result<Vec<Value>,String> {
        let eligible:bool=self.0.query_row("SELECT kind IN ('document','image') FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(|e|e.to_string())?;
        if eligible { Ok(self.reply_files(request)?.into_iter().filter(|f| matches!(f["media"].as_str(),Some("document"|"video"))).collect()) } else { Ok(vec![]) }
    }

    /// #186: what reading one attached file gave the turn; the content itself is never retained.
    pub fn file_read(&self, r: &Run, name: &str, result: &str, at: i64) -> Result<(),String> {
        self.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'file_read',?3,?4)",params![r.request,at,name,result]).map(drop).map_err(|e|e.to_string())
    }

    /// Only this caller's admitted input from this generation can supply files.
    pub fn reply_file_source(&self, r: &Run, source: Option<i64>) -> Result<Vec<Value>,String> {
        let source=source.unwrap_or(r.request);
        let own:bool=self.0.query_row("SELECT EXISTS(SELECT 1 FROM requests q JOIN runs x ON x.request_id=q.id WHERE q.id=?1 AND q.id<=?2 AND q.sender=(SELECT sender FROM requests WHERE id=?2) AND q.session=?3 AND x.generation=?4)",params![source,r.request,r.session,r.generation],|x|x.get(0)).map_err(|e|e.to_string())?;
        if !own { return Err("files must come from your own input in this conversation generation".into()); }
        self.reply_files(source)
    }

    /// #108: only literal text supplied by this person in this conversation generation can be forwarded.
    pub fn answer_source(&self, r: &Run, source: Option<i64>, text: &str) -> Result<(),String> {
        if text.trim().is_empty() || text.lines().count()>4 || text.lines().any(|s|s.trim().is_empty() || s.chars().count()>240) {
            return Err("answer must be 1-4 nonempty lines of at most 240 characters".into());
        }
        let source=source.unwrap_or(r.request);
        let own:bool=self.0.query_row("SELECT EXISTS(SELECT 1 FROM requests q JOIN runs x ON x.request_id=q.id
            WHERE q.id=?1 AND q.id<=?2 AND q.sender=(SELECT sender FROM requests WHERE id=?2) AND q.session=?3 AND x.generation=?4
            AND instr(coalesce(q.text,''),?5)>0)",params![source,r.request,r.session,r.generation,text],|x|x.get(0)).map_err(|e|e.to_string())?;
        if !own { return Err("answer must quote the person's own input in this conversation generation; clarify before writing".into()); }
        Ok(())
    }

    pub fn discussion_answer(&self, r: &Run, card: i64, text: &str, source: Option<i64>) -> Result<String,String> {
        if self.fenced(r)? { return Err("this run was stopped".into()); }
        self.answer_source(r,source,text)?;
        let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        let current:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM cards c JOIN requests q ON q.id=c.request_id
            WHERE c.id=?1 AND q.session=?2 AND q.sender=(SELECT sender FROM requests WHERE id=?3) AND c.generation=?4
            AND c.options IS NOT NULL AND c.used_request IS NULL AND q.state='answered'
            AND NOT EXISTS(SELECT 1 FROM cards newer JOIN requests n ON n.id=newer.request_id WHERE n.session=q.session AND newer.id>c.id))",
            params![card,r.session,r.request,r.generation],|x|x.get(0)).map_err(|e|e.to_string())?;
        if !current { return Err("question is superseded, already answered or belongs to another person; use the current question".into()); }
        tx.execute("UPDATE cards SET used_request=?2 WHERE id=?1",params![card,r.request]).map_err(|e|e.to_string())?;
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'decision',?3,?4)",params![r.request,self.0.query_row("SELECT received_at FROM requests WHERE id=?1",[r.request],|x|x.get::<_,i64>(0)).map_err(|e|e.to_string())?,format!("card:{card}"),text]).map_err(|e|e.to_string())?;
        tx.execute("UPDATE requests SET kind='choice' WHERE id=?1",[r.request]).map_err(|e|e.to_string())?;
        choice_receipt(&tx,card,r.request,text)?;
        tx.commit().map_err(|e|e.to_string())?;
        Ok(format!("Answer saved locally: {text}. Continue discussion; no launch authorized."))
    }

    /// The typed TaskQ question relayed in outbox row `id`, when one was stored (#2010).
    pub(super) fn question(&self, id: i64) -> Result<Option<serde_json::Value>,String> {
        let q:Option<String>=self.0.query_row("SELECT question FROM outbox WHERE id=?1",[id],|x|x.get(0)).optional().map_err(|e|e.to_string())?.flatten();
        Ok(q.and_then(|q|serde_json::from_str(&q).ok()))
    }

    /// TaskQ #2010: a pick is the id of a choice in the question relayed at `revision`; anything else is free text.
    pub fn choice(&self, project: &str, task: i64, revision: i64, text: &str) -> Result<Option<(u64,String)>,String> {
        let id:Option<i64>=self.0.query_row("SELECT id FROM outbox WHERE project=?1 AND task=?2 AND revision=?3 AND kind='question' AND question IS NOT NULL ORDER BY id DESC LIMIT 1",
            params![project,task,revision],|x|x.get(0)).optional().map_err(|e|e.to_string())?;
        let Some(q)=(match id { Some(id)=>self.question(id)?, None=>None }) else { return Ok(None) };
        let Ok(pick)=text.trim().parse::<u64>() else { return Ok(None) };
        Ok(crate::taskq::choices(&q).iter().find(|c|c["id"].as_u64()==Some(pick)).map(|c|(pick,c["label"].as_str().unwrap_or("").to_owned())))
    }

    pub fn selected_answer(&self, project: &str, task: i64, revision: i64, text: &str) -> Result<String,String> {
        Ok(self.choice(project,task,revision,text)?.map_or_else(||text.to_owned(),|(id,label)|format!("{id}. {label}")))
    }

    pub fn answer_state(&self, request: i64, state: &str) -> Result<String,String> {
        let ru:bool=self.0.query_row("SELECT coalesce(language,'en') LIKE 'ru%' FROM requests WHERE id=?1",[request],|x|x.get(0)).map_err(|e|e.to_string())?;
        Ok(if ru { crate::conversation::task_state(state,false,Some("ru")) } else { describe(state) }.into())
    }

    pub fn answer_actor(&self, request: i64, text: &str) -> Result<String,String> {
        self.0.query_row("SELECT sender,name,language FROM requests WHERE id=?1",[request],|x| {
            let sender:i64=x.get(0)?; let name:Option<String>=x.get(1)?; let lang:Option<String>=x.get(2)?;
            let actor=name.unwrap_or_else(||sender.to_string());
            Ok(if lang.as_deref().unwrap_or("en").starts_with("ru") { format!("Ответил {actor}: {text}") } else { format!("Answered by {actor}: {text}") })
        }).map_err(|e|e.to_string())
    }

    /// §5/#93: structured choices are discussion input, never launch authorization.
    pub fn save_choices(&self, r: &Run, question: &str, options: &[String]) -> Result<(), String> {
        if self.fenced(r)? { return Err("this run was stopped".into()); }
        if question.trim().is_empty() || question.chars().count()>500 || !(2..=4).contains(&options.len()) || options.iter().any(|s|s.trim().is_empty() || s.contains('\n') || s.chars().count()>80) {
            return Err("use a question up to 500 characters and 2-4 single-line choices up to 80 characters".into());
        }
        self.0.execute("INSERT INTO cards(request_id,generation,question,options) VALUES (?1,?2,?3,?4) ON CONFLICT(request_id) DO UPDATE SET question=excluded.question,options=excluded.options",
            params![r.request,r.generation,question,serde_json::to_string(options).map_err(|e|e.to_string())?]).map(drop).map_err(|e|e.to_string())
    }

    /// Durable source of buttons, shared by rich delivery and the last HTML/plain fallback part.
    pub fn card_buttons(&self, delivery: i64) -> Result<Vec<(String,String)>,String> {
        let release=crate::release::buttons(self,delivery)?;
        if !release.is_empty() {return Ok(release);}
        let onboarding=crate::onboarding::buttons(self,delivery)?;
        if !onboarding.is_empty() {return Ok(onboarding); }
        let card: Option<(i64,Option<i64>,Option<String>,Option<String>)> = self.0.query_row(
            "SELECT c.id,c.plan_id,c.options,q.language FROM cards c JOIN outbox o ON o.request_id=c.request_id JOIN requests q ON q.id=c.request_id
            WHERE o.id=?1 AND c.used_request IS NULL AND q.state='answered' AND (o.kind='rich' OR (o.kind='rich_part' AND o.id=(SELECT max(id) FROM outbox WHERE request_id=o.request_id AND kind='rich_part')))",
            [delivery],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?))).optional().map_err(|e|e.to_string())?;
        let problem:Option<(i64,Option<String>)>=self.0.query_row("SELECT o.id,q.language FROM outbox o JOIN requests q ON q.id=o.request_id WHERE o.id=?1 AND o.kind='plan_problem'",[delivery],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        if let Some((id,language))=problem {
            let ru=language.as_deref().unwrap_or("en").starts_with("ru");
            return Ok(vec![(format!("retry:{id}"),if ru {"Повторить"} else {"Retry"}.into()),(format!("cancel_launch:{id}"),if ru {"Отменить"} else {"Cancel"}.into())]);
        }
        let board:Option<Option<String>>=self.0.query_row("SELECT coalesce((SELECT a.result FROM audit_events a WHERE a.request_id=q.id AND a.kind='delivery_reply_language' AND a.target=CAST(o.id AS TEXT) ORDER BY a.id DESC LIMIT 1),q.language) FROM outbox o JOIN requests q ON q.id=o.request_id WHERE o.id=?1 AND o.kind='status' AND q.kind='board_event'",[delivery],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if let Some(language)=board {
            return Ok(vec![(format!("explain:{delivery}"),if language.as_deref().unwrap_or("en").starts_with("ru") {"Объяснить"} else {"Explain"}.into())]);
        }
        let mut buttons=vec![];
        let source:i64=self.0.query_row("SELECT request_id FROM outbox WHERE id=?1",[delivery],|r|r.get(0)).map_err(|e|e.to_string())?;
        let models:Option<String>=self.0.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[format!("models:{source}")],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if let Some(models)=models {
            for (i,model) in serde_json::from_str::<Vec<String>>(&models).map_err(|e|e.to_string())?.iter().enumerate() {
                let ru=self.language(source)?.as_deref()==Some("ru");
                buttons.push((format!("model:{source}:{i}"),if ru && model=="default" {"По умолчанию".into()}else{model.chars().take(64).collect()}));
            }
        }
        if let Some((id,plan,options,language))=card {
            if let Some(plan)=plan {
                let ru=language.as_deref().unwrap_or("en").starts_with("ru");
                let project:String=self.0.query_row("SELECT project FROM plans WHERE id=?1",[plan],|r|r.get(0)).map_err(|e|e.to_string())?;
                if crate::onboarding::gate(self,&project).is_ok() {
                    buttons.push((format!("plan:{id}"),if ru {"Запустить план"} else {"Run plan"}.into()));
                }
                buttons.push((format!("cancel:{id}"),if ru {"Отменить"} else {"Cancel"}.into()));
            }
            if let Some(options)=options {
                for (i,option) in serde_json::from_str::<Vec<String>>(&options).map_err(|e|e.to_string())?.into_iter().enumerate() {
                    buttons.push((format!("choice:{id}:{}",i+1),format!("{}. {option}",i+1)));
                }
            }
        }
        Ok(buttons)
    }

    /// #165: record factual phase changes; the delivery scan owns elapsed-time edits.
    pub fn launch_phase(&self, id: i64, phase: &str, at: i64) -> Result<(),String> {
        self.0.execute("INSERT INTO audit_events(request_id,at,kind,target,result) SELECT request_id,?2,'launch_phase',CAST(id AS TEXT),?3 FROM plan_runs WHERE id=?1 AND state='creating'",params![id,at,phase]).map_err(|e|e.to_string())?;
        self.launch_progress(at)
    }

    pub(super) fn launch_progress(&self, at: i64) -> Result<(),String> {
        let e=|e:rusqlite::Error|e.to_string();
        let mut q=self.0.prepare("SELECT x.id,x.request_id,x.actor,p.project,x.at,q.language,(SELECT result FROM audit_events a WHERE a.kind='launch_phase' AND a.target=CAST(x.id AS TEXT) ORDER BY a.id DESC LIMIT 1) FROM plan_runs x JOIN plans p ON p.id=x.plan_id JOIN requests q ON q.id=x.request_id WHERE x.state='creating' AND q.front='telegram'").map_err(e)?;
        let rows:Vec<(i64,i64,i64,String,i64,Option<String>,Option<String>)>=q.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).and_then(Iterator::collect).map_err(e)?;
        for (_id,request,actor,project,start,language,phase) in rows {
            let Some(phase)=phase else {continue};
            if at.saturating_sub(start)<2 {continue}
            if !self.has(actor,"chat")? || !(self.has(actor,"projects")? || self.has(actor,&format!("project:{project}"))?) {continue}
            let last:Option<i64>=self.0.query_row("SELECT max(next_at) FROM outbox WHERE request_id=?1 AND kind='plan_progress'",[request],|r|r.get(0)).map_err(e)?;
            // #256 A5 / §7: one coalesced elapsed-time edit per thirty seconds; a 2 s cadence hit Telegram retry-after.
            if last.is_some_and(|last|at.saturating_sub(last)<PROGRESS_EDIT) {continue}
            let line=crate::conversation::progress(&phase,at.saturating_sub(start).max(0),language.as_deref());
            let tx=self.0.unchecked_transaction().map_err(e)?;
            if plan_receipt(&tx,request,&line,at)? {
                tx.execute("UPDATE outbox SET kind='plan_progress' WHERE id=?1",[tx.last_insert_rowid()]).map_err(e)?;
            }
            tx.commit().map_err(e)?;
        }
        Ok(())
    }

    pub(super) fn task_progress(&self, at: i64) -> Result<(),String> {
        let e=|e:rusqlite::Error|e.to_string();
        let mut q=self.0.prepare("SELECT DISTINCT t.request_id,t.chat,t.topic,t.project,q.sender FROM tasks t JOIN requests q ON q.id=t.request_id WHERE t.closed=0 AND t.state IN ('ready','doing','review','waiting') AND q.front='telegram' AND q.state IN ('answered','failed','uncertain') AND NOT EXISTS(SELECT 1 FROM plan_runs x WHERE x.request_id=q.id AND x.state='creating')").map_err(e)?;
        let rows:Vec<(i64,i64,Option<i64>,String,i64)>=q.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).and_then(Iterator::collect).map_err(e)?;
        for (request,chat,topic,project,actor) in rows {
            if !self.has(actor,"chat")? || !(self.has(actor,"projects")? || self.has(actor,&format!("project:{project}"))?) {continue}
            let last:Option<i64>=self.0.query_row("SELECT max(next_at) FROM outbox WHERE request_id=?1 AND kind='status'",[request],|r|r.get(0)).map_err(e)?;
            // #256 A5 / §7: one coalesced elapsed-time edit per thirty seconds; a 2 s cadence hit Telegram retry-after.
            if last.is_some_and(|last|at.saturating_sub(last)<PROGRESS_EDIT) {continue}
            let text=self.launch_status(request,at)?;
            let tx=self.0.unchecked_transaction().map_err(e)?;
            tx.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND kind='status' AND state='pending'",[request]).map_err(e)?;
            notice(&tx,request,chat,topic,"status","",0,0,&text,at)?;
            tx.commit().map_err(e)?;
        }
        Ok(())
    }

    pub fn launch_control(&self, r: &Run) -> Result<Option<(i64,bool)>,String> {
        self.0.query_row("SELECT CAST(result AS INTEGER),item='cancel' FROM audit_events WHERE request_id=?1 AND kind='launch_control' LIMIT 1",[r.request],|x|Ok((x.get(0)?,x.get(1)?))).optional().map_err(|e|e.to_string())
    }

    pub fn launch_control_reply(&self, id: i64, cancel: bool) -> Result<String,String> {
        let request:i64=self.0.query_row("SELECT request_id FROM plan_runs WHERE id=?1",[id],|r|r.get(0)).map_err(|e|e.to_string())?;
        if cancel {
            let ru:bool=self.0.query_row("SELECT coalesce(language,'en') LIKE 'ru%' FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(|e|e.to_string())?;
            return Ok(if ru {"Дальнейший запуск отменён; выполненные действия сохранены."} else {"Further launch steps cancelled; earlier effects are preserved."}.into());
        }
        self.launch_receipt(request)?.ok_or("launch receipt unavailable".into())
    }

    pub fn button_plan(&self, r: &Run) -> Result<Option<Value>,String> {
        let launch: bool = self.0.query_row("SELECT kind='launch' FROM requests WHERE id=?1",[r.request],|x|x.get(0)).map_err(|e|e.to_string())?;
        if !launch { return Ok(None); }
        self.0.query_row("SELECT p.revision,p.project FROM cards c JOIN plans p ON p.id=c.plan_id JOIN requests q ON q.id=c.used_request JOIN requests source ON source.id=c.request_id
            WHERE q.id=?1 AND c.generation=?2 AND source.sender=q.sender AND source.session=?3",
            params![r.request,r.generation,r.session],|x|Ok(json!({"revision":x.get::<_,i64>(0)?,"project":x.get::<_,String>(1)?}))).optional().map_err(|e|e.to_string())?
            .map(Some).ok_or_else(||"the displayed plan belongs to an earlier conversation; nothing was launched".into())
    }

    /// §12 / #110: factual localized launch outcome from receipts, for the button and the text launch alike: each filed task with
    /// its board state read back at filing or start (held, ready or doing). No model prose or internal revision. None when this
    /// request authorized no plan run.
    pub fn launch_reply(&self, r: &Run) -> Result<Option<String>,String> {
        self.launch_receipt(r.request)
    }

    pub(super) fn launch_receipt(&self, request: i64) -> Result<Option<String>,String> {
        let row:Option<(i64,String,Option<String>)>=self.0.query_row(
            "SELECT x.id,x.state,q.language FROM plan_runs x JOIN requests q ON q.id=x.request_id WHERE q.id=?1 AND x.state<>'cancelled'",[request],
            |x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).optional().map_err(|e|e.to_string())?;
        let Some((id,state,language))=row else { return Ok(None) };
        let ru=language.as_deref().unwrap_or("en").starts_with("ru");
        let mut text=match (ru,state.as_str()) {
            (true,"started")=>"Задачи отправлены:",
            (false,"started")=>"Tasks submitted:",
            (true,"creating")=>"Не удалось отправить весь план.",
            (false,"creating")=>"Could not submit the whole plan.",
            (true,_)=>"Не удалось запустить план.",
            (false,_)=>"Could not launch the plan.",
        }.to_string();
        // #236 / R11: submission can encounter a closed gate after authorization.
        // Only the typed setup refusal replaces the summary; task receipts remain visible.
        if state=="failed" {
            let refusal:Option<String>=self.0.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='plan_gate_refusal' AND target=?2 ORDER BY id DESC LIMIT 1",params![request,id.to_string()],|x|x.get(0)).optional().map_err(|e|e.to_string())?;
            if let Some(refusal)=refusal {text=refusal;}
        }
        let run=self.plan_run(id)?;
        for ticket in run.tickets {
            let Some(n)=ticket.task else { continue };
            let board_state=if ticket.started {
                self.0.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='task' AND target=?2 ORDER BY id DESC LIMIT 1",
                    params![run.request,format!("{} start #{n}",run.board)],|x|x.get::<_,String>(0)).optional().map_err(|e|e.to_string())?.unwrap_or_default()
            } else { "held".into() };
            let status=crate::conversation::task_state(&board_state,false,language.as_deref()).to_lowercase();
            // #263: the placement TaskQ accepted with the held filing
            let status=[("host",&ticket.plan["host"]),("runtime",&ticket.plan["runtime"])].iter()
                .filter_map(|(k,v)|v.as_str().map(|v|format!(" · {k} {v}"))).fold(status,|s,x|s+&x);
            text+=&format!("\n- {}: {status}",crate::taskq::task_link(&run.board,n,ticket.plan["title"].as_str(),self.task_url(&run.project,n)?.as_deref()));
        }
        Ok(Some(text))
    }

    /// #110: the plan run `request` authorized itself, if any; a repeated launch of an admitted draft authorizes none.
    pub fn launch_run(&self, request: i64) -> Result<Option<i64>,String> {
        self.0.query_row("SELECT id FROM plan_runs WHERE request_id=?1 AND state<>'cancelled'",[request],|x|x.get(0)).optional().map_err(|e|e.to_string())
    }

    /// #110: the Cancel button closes the open draft it was shown with. A cancelled run row marks the draft closed, files nothing
    /// and makes any later launch of it a refusal. None when this request is not a cancel.
    pub fn cancel_draft(&self, r: &Run, at: i64) -> Result<Option<String>,String> {
        let cancel:bool=self.0.query_row("SELECT kind='cancel' FROM requests WHERE id=?1",[r.request],|x|x.get(0)).map_err(|e|e.to_string())?;
        if !cancel { return Ok(None); }
        let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        let plan:Option<(i64,Option<String>)>=tx.query_row("SELECT p.id,q.language FROM cards c JOIN plans p ON p.id=c.plan_id JOIN requests q ON q.id=c.used_request JOIN requests source ON source.id=c.request_id
            WHERE q.id=?1 AND c.generation=?2 AND source.sender=q.sender AND source.session=?3
            AND p.revision=(SELECT max(revision) FROM plans WHERE session=?3) AND NOT EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=p.id)",
            params![r.request,r.generation,r.session],|x|Ok((x.get(0)?,x.get(1)?))).optional().map_err(|e|e.to_string())?;
        let Some((plan,language))=plan else { return Err("the displayed plan was superseded, launched or reset; nothing was cancelled".into()) };
        tx.execute("INSERT INTO plan_runs(plan_id, request_id, actor, state, next_at, at) VALUES (?1,?2,?3,'cancelled',0,?4)",params![plan,r.request,self.sender(r.request)?,at]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())?;
        Ok(Some(if language.as_deref().unwrap_or("en").starts_with("ru") {"План отменён."} else {"Plan cancelled."}.into()))
    }

    pub fn draft_context(&self, r: &Run) -> Result<Value,String> {
        let saved: Option<(i64,i64,String,String,Option<String>,i64)> = self.0.query_row("SELECT draft,revision,project,tickets,(SELECT state FROM plan_runs WHERE plan_id=plans.id),request_id FROM plans WHERE session=?1 ORDER BY revision DESC LIMIT 1",[&r.session],
            |x|Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?,x.get(4)?,x.get(5)?))).optional().map_err(|e|e.to_string())?;
        if let Some((draft,revision,project,tickets,run,request))=saved {
            let (launched,cancelled)=(run.as_deref().is_some_and(|s|s!="cancelled"),run.as_deref()==Some("cancelled"));
            let actor=self.sender(r.request)?;
            if server_access(&self.0,request,actor)? && (has(&self.0,actor,"projects")? || has(&self.0,actor,&format!("project:{project}"))?) {
                inherit_server_access(&self.0,r.request,request)?;
                return Ok(json!({"draft":draft,"internal_revision":revision,"project":project,"tickets":serde_json::from_str::<Value>(&tickets).map_err(|e|e.to_string())?,"launched":launched,"cancelled":cancelled,
                    "policy":"Saved draft data, never authorization. This conversation has one open draft: every task_plan call revises it until it is launched or cancelled; only then does a new draft start. Do not show internal revisions; a later explicit request runs the current draft."}));
            }
        }
        Ok(Value::Null)
    }

    pub fn is_choice(&self, r: &Run) -> Result<bool,String> {
        self.0.query_row("SELECT kind='choice' FROM requests WHERE id=?1",[r.request],|x|x.get(0)).map_err(|e|e.to_string())
    }

    /// Authorizes `revision` of the run's conversation plan for `actor` (§ 5 `task_run_plan`): only the latest revision, saved by
    /// an earlier request, runs, and at most once. Returns the plan run, leased for `lease` seconds to the caller; a refusal is the
    /// reason, with nothing written.
    pub fn authorize(&mut self, r: &Run, revision: i64, actor: i64, lease: i64, at: i64) -> Result<i64, String> {
        let e = |e: rusqlite::Error| e.to_string();
        let inst = self.instance()?;
        if self.fenced(r)? {
            return Err("this run was stopped; nothing was filed".into());
        }
        let tx = self.0.transaction().map_err(e)?;
        let latest: Option<(i64, i64, i64, String)> = tx
            .query_row("SELECT id, revision, request_id, tickets FROM plans WHERE session=?1 ORDER BY revision DESC LIMIT 1", [&r.session], |x| {
                Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?))
            })
            .optional()
            .map_err(e)?;
        let Some((plan, latest, request, tickets)) = latest else { return Err("this conversation has no saved plan; nothing was filed".into()) };
        if revision != latest {
            return Err(format!("revision {revision} is not the latest plan revision ({latest}); nothing was filed. Show revision {latest} and ask again."));
        }
        if request == r.request {
            return Err(format!("revision {revision} was saved by this same request; the person authorizes it in a later message after seeing it. Nothing was filed."));
        }
        if let Some((run, state)) = tx.query_row("SELECT id, state FROM plan_runs WHERE plan_id=?1", [plan], |x| Ok((x.get(0)?, x.get::<_, String>(1)?))).optional().map_err(e)? {
            if state == "cancelled" { return Err("this draft was cancelled; nothing was filed. Plan again to open a new draft.".into()); }
            return Ok(run);
        }
        if meta(&tx,&format!("edited-plan:{plan}"))?.is_some() {
            return Err("the source input was edited; review and save a new plan revision before launching".into());
        }
        tx.execute("INSERT INTO plan_runs(plan_id, request_id, actor, state, next_at, at) VALUES (?1,?2,?3,'creating',?4,?5)", params![plan, r.request, actor, at + lease, at]).map_err(e)?;
        tx.execute("UPDATE outbox SET project=(SELECT project FROM plans WHERE id=?2) WHERE request_id=?1 AND task IS NULL", params![r.request, plan]).map_err(e)?;
        let run = tx.last_insert_rowid();
        let n = serde_json::from_str::<Value>(&tickets).map_err(|x| x.to_string())?.as_array().map_or(0, Vec::len);
        for idx in 1..=n as i64 {
            tx.execute("INSERT INTO tickets(run_id, idx, op_id) VALUES (?1,?2,?3)", params![run, idx, format!("pilotq-{inst}-p{plan}-t{idx}")]).map_err(e)?;
        }
        tx.execute("UPDATE requests SET project=(SELECT project FROM plans WHERE id=?2) WHERE id=?1", params![r.request, plan]).map_err(e)?;
        tx.commit().map_err(e)?;
        Ok(run)
    }

    /// Plan runs still filing or starting whose retry time has come, each leased for `lease` seconds (§ 5, L16).
    pub fn due_runs(&self, now: i64, lease: i64) -> Result<Vec<i64>, String> {
        let mut q = self.0.prepare("UPDATE plan_runs SET next_at=?1 + ?2 WHERE state='creating' AND next_at<=?1 RETURNING id").map_err(|e| e.to_string())?;
        q.query_map([now, lease], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())
    }

    pub fn plan_run(&self, id: i64) -> Result<PlanRun, String> {
        let e = |e: rusqlite::Error| e.to_string();
        let (revision, request, actor, chat, topic, plan): (i64, i64, i64, i64, Option<i64>, String) = self
            .0
            .query_row(
                "SELECT p.revision, r.request_id, r.actor, q.chat, q.topic, p.tickets FROM plan_runs r JOIN plans p ON p.id=r.plan_id JOIN requests q ON q.id=r.request_id WHERE r.id=?1",
                [id],
                |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?, x.get(4)?, x.get(5)?)),
            )
            .map_err(e)?;
        let plan: Value = serde_json::from_str(&plan).map_err(|x| x.to_string())?;
        let mut q = self.0.prepare("SELECT idx, op_id, task, started FROM tickets WHERE run_id=?1 ORDER BY idx").map_err(e)?;
        let tickets = q
            .query_map([id], |x| Ok(Ticket { idx: x.get(0)?, op_id: x.get(1)?, task: x.get(2)?, started: x.get(3)?, plan: Value::Null }))
            .and_then(Iterator::collect::<Result<Vec<Ticket>, _>>)
            .map_err(e)?
            .into_iter()
            .map(|t| Ticket { plan: plan[(t.idx - 1) as usize].clone(), ..t })
            .collect();
        let (project, board, root) = self.0.query_row("SELECT p.project, p.board, p.root FROM plans p JOIN plan_runs r ON r.plan_id=p.id WHERE r.id=?1", [id], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?))).map_err(e)?;
        Ok(PlanRun { id, revision, request, actor, chat, topic, tickets, project, board, root })
    }

    /// Records a ticket's held readback: the task joins its route's subscriptions and the request's audit record.
    pub fn filed(&self, run: &PlanRun, idx: i64, task: i64, revision: i64, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        tx.execute("UPDATE tickets SET task=?3 WHERE run_id=?1 AND idx=?2", params![run.id, idx, task]).map_err(e)?;
        tx.execute(
            "INSERT INTO tasks(number, request_id, chat, topic, state, revision, observed, project, board, root) VALUES (?1,?2,?3,?4,'held',?5,?6,?7,?8,?9) ON CONFLICT(project,number) DO NOTHING",
            params![task, run.request, run.chat, run.topic, revision, at, run.project, run.board, run.root],
        )
        .map_err(e)?;
        let op = &run.tickets.iter().find(|t| t.idx == idx).ok_or("no such ticket")?.op_id;
        tx.execute("INSERT INTO audit_events(request_id, at, kind, target, item, result) VALUES (?1,?2,'task',?3,?4,'held')", params![run.request, at, format!("{} add #{task}", run.board), op])
            .map_err(e)?;
        tx.commit().map_err(e)
    }

    /// #144: retain the exact TaskQ URL in the existing audit store; no schema upgrade or forge synthesis.
    pub fn remember_task_url(&self, project: &str, task: i64, url: Option<&str>, at: i64) -> Result<(), String> {
        let Some(url)=url.filter(|u|crate::taskq::valid_url(u)) else { return Ok(()) };
        let row: Option<(i64,String)>=self.0.query_row("SELECT request_id,root FROM tasks WHERE project=?1 AND number=?2",params![project,task],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        let Some((request,root))=row else { return Ok(()) };
        if self.task_url(project,task)?.as_deref()==Some(url) { return Ok(()) }
        self.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'task_url',?3,?4,?5)",params![request,at,format!("{project}#{task}"),root,url]).map_err(|e|e.to_string())?;
        Ok(())
    }

    pub fn task_url(&self, project: &str, task: i64) -> Result<Option<String>, String> {
        self.0.query_row("SELECT a.result FROM audit_events a JOIN tasks t ON a.request_id=t.request_id AND a.item=t.root
            JOIN projects p ON p.name=t.project AND p.root=t.root AND p.repo=t.board
            WHERE a.kind='task_url' AND a.target=?1 AND t.project=?2 AND t.number=?3 ORDER BY a.id DESC LIMIT 1",
            params![format!("{project}#{task}"),project,task],|r|r.get(0)).optional().map_err(|e|e.to_string())
    }

    /// Exact readback identity only; never infer a forge or board from a URL's spelling.
    pub fn url_target(&self, url: &str) -> Result<Option<(String,String,i64)>,String> {
        let mut q=self.0.prepare("SELECT project,board,number FROM tasks").map_err(|e|e.to_string())?;
        let rows:Vec<(String,String,i64)>=q.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).and_then(Iterator::collect).map_err(|e|e.to_string())?;
        let mut matches=Vec::new();
        for (p,b,n) in rows { if self.task_url(&p,n)?.as_deref()==Some(url) { matches.push((p,b,n)); } }
        match matches.len() {0=>Ok(None),1=>Ok(matches.pop()),_=>Err("task URL matches multiple registrations; clarify the project".into())}
    }

    /// Verified readback titles survive disappearance from the open board (#385 / R8).
    pub fn remember_task_title(&self, project:&str, task:i64, title:Option<&str>, at:i64)->Result<(),String> {
        let Some(title)=title.filter(|t|!t.is_empty()) else {return Ok(())};
        if self.task_title(project,task)?.as_deref()==Some(title) {return Ok(())}
        self.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result)
            SELECT request_id,?3,'task_title',?4,root,?5 FROM tasks WHERE project=?1 AND number=?2",
            params![project,task,at,format!("{project}#{task}"),title]).map(drop).map_err(|e|e.to_string())
    }

    /// The task's filed essence for its cards (§2 / #387), kept like its title.
    pub fn remember_task_goal(&self, project:&str, task:i64, goal:Option<&str>, at:i64)->Result<(),String> {
        let Some(goal)=goal.filter(|g|!g.is_empty()) else {return Ok(())};
        if self.task_goal(project,task)?.as_deref()==Some(goal) {return Ok(())}
        self.0.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result)
            SELECT request_id,?3,'task_goal',?4,root,?5 FROM tasks WHERE project=?1 AND number=?2",
            params![project,task,at,format!("{project}#{task}"),goal]).map(drop).map_err(|e|e.to_string())
    }

    pub fn task_goal(&self, project:&str, task:i64)->Result<Option<String>,String> {
        let observed:Option<String>=self.0.query_row("SELECT a.result FROM audit_events a JOIN tasks t ON a.request_id=t.request_id AND a.item=t.root
            WHERE a.kind='task_goal' AND a.target=?1 AND t.project=?2 AND t.number=?3 ORDER BY a.id DESC LIMIT 1",
            params![format!("{project}#{task}"),project,task],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if observed.is_some() {return Ok(observed)}
        // Before the first scan, use the immutable plan that filed this exact subscription (§2 / #387).
        let filed:Option<(String,i64)>=self.0.query_row("SELECT p.tickets,k.idx FROM tickets k
            JOIN plan_runs r ON r.id=k.run_id JOIN plans p ON p.id=r.plan_id
            JOIN tasks t ON t.request_id=r.request_id AND t.number=k.task AND t.project=p.project AND t.board=p.board AND t.root=p.root
            WHERE t.project=?1 AND t.number=?2 ORDER BY r.id DESC LIMIT 1",
            params![project,task],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        let Some((text,idx))=filed else {return Ok(None)};
        let tickets:Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
        Ok(tickets[idx.saturating_sub(1) as usize]["goal"].as_str()
            .and_then(|goal|crate::taskq::task_goal(Some(&format!("## Goal\n{goal}")))))
    }

    /// Saved title for a subscribed task, including after it leaves the open snapshot (§12).
    pub fn task_title(&self, project: &str, task: i64) -> Result<Option<String>, String> {
        let observed:Option<String>=self.0.query_row("SELECT a.result FROM audit_events a JOIN tasks t ON a.request_id=t.request_id AND a.item=t.root
            JOIN projects p ON p.name=t.project AND p.root=t.root AND p.repo=t.board
            WHERE a.kind='task_title' AND a.target=?1 AND t.project=?2 AND t.number=?3 ORDER BY a.id DESC LIMIT 1",
            params![format!("{project}#{task}"),project,task],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if observed.is_some() {return Ok(observed)}
        let row: Option<(String, i64)> = self.0.query_row(
            "SELECT p.tickets, k.idx FROM tickets k JOIN plan_runs r ON r.id=k.run_id JOIN plans p ON p.id=r.plan_id WHERE p.project=?1 AND k.task=?2 ORDER BY r.id DESC LIMIT 1",
            params![project, task], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(|e| e.to_string())?;
        row.map(|(text, idx)| serde_json::from_str::<Value>(&text).map(|v| v[idx.saturating_sub(1) as usize]["title"].as_str().map(str::to_owned)).map_err(|e| e.to_string())).transpose().map(Option::flatten)
    }

    /// Records a ticket's start receipt.
    pub fn started(&self, run: &PlanRun, idx: i64, task: i64, state: &str, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        tx.execute("UPDATE tickets SET started=1 WHERE run_id=?1 AND idx=?2", params![run.id, idx]).map_err(e)?;
        tx.execute("UPDATE tasks SET state=?3,observed=?4 WHERE project=?1 AND number=?2",params![run.project,task,state,at]).map_err(e)?;
        tx.execute("INSERT INTO audit_events(request_id, at, kind, target, item, result) VALUES (?1,?2,'task',?3,NULL,?4)", params![run.request, at, format!("{} start #{task}", run.board), state])
            .map_err(e)?;
        tx.commit().map_err(e)
    }

    /// Ends a plan run (`started` or `failed`) or defers it (`creating`, retried after `next_at`) with its reason. `notify` goes
    /// to the run's route: a scan, not the authorizing turn, finished it.
    pub fn run_outcome(&self, run: &PlanRun, state: &str, reason: Option<&str>, notify: Option<&str>, next_at: i64, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        tx.execute("UPDATE plan_runs SET state=?2, reason=?3, next_at=?4 WHERE id=?1", params![run.id, state, reason, next_at]).map_err(e)?;
        if state=="creating" {
            tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'launch_phase',?3,'waiting')",params![run.request,at,run.id.to_string()]).map_err(e)?;
        } else {
            tx.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND kind='plan_progress' AND state='pending'",[run.request]).map_err(e)?;
        }
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'launch_diagnostic',?3,?4,?5)",params![run.request,at,run.id.to_string(),reason,notify]).map_err(e)?;
        let episode=match state {"started"=>"resolved","creating"=>"waiting",_=>"failed"};
        let previous:Option<String>=tx.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='launch_notice' ORDER BY id DESC LIMIT 1",[run.request],|r|r.get(0)).optional().map_err(e)?;
        let changed=previous.as_deref()!=Some(episode);
        if changed {
            tx.execute("INSERT INTO audit_events(request_id,at,kind,result) VALUES (?1,?2,'launch_notice',?3)",params![run.request,at,episode]).map_err(e)?;
        }
        tx.commit().map_err(e)?;
        if notify.is_some() {
            let status=self.launch_status(run.request,at)?;
            let last:Option<String>=self.0.query_row("SELECT text FROM outbox WHERE request_id=?1 AND kind='status' ORDER BY id DESC LIMIT 1",[run.request],|r|r.get(0)).optional().map_err(e)?;
            if last.as_deref()!=Some(&status) {
                let tx=self.0.unchecked_transaction().map_err(e)?;
                tx.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND kind='status' AND state='pending'",[run.request]).map_err(e)?;
                notice(&tx,run.request,run.chat,run.topic,"status","",0,0,&status,at)?;
                tx.commit().map_err(e)?;
            }
        }
        if notify.is_some() && changed {
            if let Some(text)=self.launch_receipt(run.request)? {
                let tx=self.0.unchecked_transaction().map_err(e)?;
                if !plan_receipt(&tx,run.request,&text,at)? {outbox(&tx,run.request,run.chat,run.topic,None,Some(&crate::telegram::html(&text)),&text,at)?;}
                tx.commit().map_err(e)?;
            }
        }
        Ok(())
    }

    /// #92 / §5: only trusted same-conversation subscriptions may select a contextual status/steer target.
    pub fn related_target(&self, r: &Run, task: &str, project: Option<&str>) -> Result<Option<(String,String,i64)>,String> {
        let linked = if task.trim().starts_with("https://") { self.url_target(task.trim())? } else { None };
        let board=linked.as_ref().map(|t|t.1.as_str());
        let n=match linked.as_ref().map(|t|t.2).or_else(||task.trim().trim_start_matches('#').parse::<i64>().ok()) {
            Some(n) if n>0 => n,
            _ => return Ok(None),
        };
        let mut q=self.0.prepare("SELECT t.project,t.board,t.root FROM tasks t JOIN requests q ON q.id=t.request_id WHERE q.session=?1 AND t.number=?2
            AND (?3 IS NULL OR t.project=?3) AND (?4 IS NULL OR t.board=?4)
            AND (EXISTS(SELECT 1 FROM grants WHERE user=?5 AND name='projects')
            OR EXISTS(SELECT 1 FROM grants WHERE user=?5 AND name='project:' || t.project))").map_err(|e|e.to_string())?;
        let candidates:Vec<(String,String,String)>=q.query_map(params![r.session,n,project,board,self.sender(r.request)?],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?)))
            .and_then(Iterator::collect).map_err(|e|e.to_string())?;
        match candidates.as_slice() {
            [] => Ok(None),
            [(project,board,root)] => {
                let registered:bool=self.0.query_row("SELECT EXISTS(SELECT 1 FROM projects WHERE name=?1 AND repo=?2 AND root=?3)",
                    params![project,board,root],|x|x.get(0)).map_err(|e|e.to_string())?;
                if !registered { return Err("subscribed task registration changed; nothing was done".into()); }
                Ok(Some((project.clone(),board.clone(),n)))
            }
            _ => Err("task number matches several subscribed boards; clarify the intended task before acting".into()),
        }
    }

    /// Open task subscriptions and their stored revisions, after submission settles.
    /// TaskQ start changes eligibility labels in separate writes; a scan during that
    /// gap can omit a filed task without closing it (§5). The creating run owns recovery.
    pub fn subscriptions(&self) -> Result<Vec<(String, i64, i64)>, String> {
        let mut q = self.0.prepare("SELECT project, number, revision FROM tasks t WHERE closed=0 AND NOT EXISTS (
            SELECT 1 FROM tickets k JOIN plan_runs r ON r.id=k.run_id
            WHERE k.task=t.number AND r.request_id=t.request_id AND r.state='creating'
        ) ORDER BY number").map_err(|e| e.to_string())?;
        q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).and_then(Iterator::collect).map_err(|e| e.to_string())
    }

    /// The stored projection of a subscribed task: state, revision and card.
    pub fn task(&self, project: &str, task: i64) -> Result<Option<(String, i64, Option<String>)>, String> {
        self.0.query_row("SELECT state, revision, card FROM tasks WHERE number=?1 AND project=?2", params![task, project], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).optional().map_err(|e| e.to_string())
    }

    pub fn task_language(&self, project: &str, task: i64) -> Result<Option<String>,String> {
        let request:i64=self.0.query_row("SELECT request_id FROM tasks WHERE project=?1 AND number=?2",params![project,task],|r|r.get(0)).map_err(|e|e.to_string())?;
        event_reply_language(&self.0,request)
    }

    /// #165 A5: all tasks in a launch project into one status card, never one message per transition.
    pub(crate) fn launch_status(&self, request: i64, at: i64) -> Result<String,String> {
        let language:Option<String>=event_reply_language(&self.0,request)?;
        let mut q=self.0.prepare("SELECT project,board,number,state,closed,observed,chat,topic FROM tasks WHERE request_id=?1 ORDER BY project,number").map_err(|e|e.to_string())?;
        let rows:Vec<(String,String,i64,String,bool,i64,i64,Option<i64>)>=q.query_map([request],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).and_then(Iterator::collect).map_err(|e|e.to_string())?;
        let ru=language.as_deref().unwrap_or("en").starts_with("ru");
        let mut lines=Vec::new();
        for (project,board,n,state,closed,observed,chat,topic) in rows {
            // #401 A3: with PM review on in this route its reply is the primary explanation; literal source stays one tap away.
            let pm=crate::loops::pm_on(&self.0,&project,chat,topic)?;
            let mut source=vec![];
            let status=crate::conversation::task_state(&state,closed,language.as_deref());
            let age=if closed {String::new()} else {if ru {format!(" · подтверждено {}",crate::footer::russian_units(&crate::time::moment(Some(observed),at)).replace(" ago", " назад"))} else {format!(" · confirmed {}",crate::time::moment(Some(observed),at))}};
            lines.push(format!("{} {status}{age} · {}",crate::taskq::state_icon(&state,closed),crate::taskq::task_link(&board,n,self.task_title(&project,n)?.as_deref(),self.task_url(&project,n)?.as_deref())));
            if let Some(goal)=self.task_goal(&project,n)? {
                if pm {source.push(crate::taskq::goal_line(&goal,language.as_deref()));}
                else if let Some(card)=lines.last_mut() {*card=format!("{card}\n{}",crate::taskq::goal_line(&goal,language.as_deref()));}
            }
            if closed {
                let result:Option<String>=self.0.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='task_result' AND target=?2 ORDER BY id DESC LIMIT 1",params![request,format!("{project}#{n}")],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
                if let Some(result)=result {
                    let detail=result.lines().skip(1).collect::<Vec<_>>().join("\n");
                    // Trusted source, not diagnostics: every limit stays literal (§2 / #387).
                    if !detail.is_empty() {if pm {source.push(detail);} else {lines.push(detail);}}
                }
            }
            if !source.is_empty() {lines.push(crate::telegram::details(if ru {"Исходный текст задачи"} else {"Task source text"},&source.join("\n")));}
        }
        Ok(if lines.is_empty() {if ru {"Ожидает отправки.".into()} else {"Waiting to submit.".into()}} else {lines.join("\n\n")})
    }

    /// Applies one fresh board observation of a subscribed task (§ 5). An observation older than the stored revision is stale and
    /// ignored (F14); an unchanged one changes nothing. A change supersedes the task's pending status update with `status`, and
    /// an `ask` also queues `question` as its own message. Returns true when something was queued.
    #[allow(clippy::too_many_arguments)]
    pub fn observe(&self, project: &str, task: i64, state: &str, revision: i64, card: Option<&str>, _status: &str, question: Option<&str>, at: i64) -> Result<bool, String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        let row: Option<(i64, i64, Option<i64>, String, i64)> = tx
            .query_row("SELECT request_id, chat, topic, state, revision FROM tasks WHERE number=?1 AND project=?2 AND closed=0", params![task, project], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
            .optional()
            .map_err(e)?;
        let Some((request, chat, topic, old, rev)) = row else { return Ok(false) };
        if revision < rev || (revision == rev && state == old) {
            return Ok(false);
        }
        tx.execute("UPDATE tasks SET state=?2, revision=?3, card=?4, observed=?5 WHERE number=?1 AND project=?6", params![task, state, revision, card, at, project]).map_err(e)?;
        if state == old && state != "ask" {
            tx.commit().map_err(e)?;
            return Ok(false);
        }
        if state != old {
            tx.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND kind='status' AND state='pending'", [request]).map_err(e)?;
            let text=self.launch_status(request,at)?;
            notice(&tx, request, chat, topic, "status", project, task, revision, &text, at)?;
        }
        if let (Some(q), "ask") = (question, state) {
            // §2/§4: retirement is presentation, never an accepted-answer receipt.
            // Queue it before the replacement so route ordering also covers unsent questions.
            let ru = event_reply_language(&tx,request)?.as_deref().unwrap_or("en").starts_with("ru");
            let text = if ru { "Вопрос заменён новым; используйте текущий вопрос." } else { "Superseded by a newer question; use the current question." };
            tx.execute("INSERT INTO outbox(request_id,chat,topic,text,html,next_at,kind,project,task,revision)
                SELECT request_id,chat,topic,text || char(10) || char(10) || ?7,NULL,?8,'question_superseded',project,task,revision FROM outbox o
                WHERE request_id=?1 AND project=?2 AND task=?3 AND chat=?4 AND topic IS ?5 AND kind='question' AND revision<?6
                AND NOT EXISTS (SELECT 1 FROM outbox a WHERE a.request_id=o.request_id AND a.project=o.project AND a.task=o.task
                    AND a.chat=o.chat AND a.topic IS o.topic AND a.revision=o.revision AND a.kind IN ('question_answered','question_superseded'))",
                params![request,project,task,chat,topic,revision,text,at]).map_err(e)?;
            let delivery=notice(&tx, request, chat, topic, "question", project, task, revision, q, at)?;
            tx.execute("UPDATE outbox SET question=?1 WHERE id=?2", params![card,delivery]).map_err(e)?;
        }
        tx.commit().map_err(e)?;
        Ok(true)
    }

    /// #401 A3: PM review explains this subscribed task's cards in the route that shows them.
    pub fn question_review_on(&self, project: &str, task: i64) -> Result<bool,String> {
        let route:Option<(i64,Option<i64>)>=self.0.query_row("SELECT chat,topic FROM tasks WHERE project=?1 AND number=?2 AND closed=0",params![project,task],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        route.map_or(Ok(false),|(chat,topic)|crate::loops::pm_on(&self.0,project,chat,topic))
    }

    /// #401 A2/A5: explanation of one current question card in this turn's route or its author's own DM, never another shared route.
    /// Presentation only: the original question row, its message, numbers and answer binding stay. Answers and supersession queue
    /// later, so a stale explanation cannot be shown last.
    #[allow(clippy::too_many_arguments)]
    pub fn explain_question(&self, request: i64, project: &str, task: i64, revision: i64, text: &str, question: &Value, audit: &str, at: i64) -> Result<String,String> {
        let e=|e:rusqlite::Error|e.to_string();
        let tx=self.0.unchecked_transaction().map_err(e)?;
        let (chat,topic,sender):(i64,Option<i64>,i64)=tx.query_row("SELECT chat,topic,sender FROM requests WHERE id=?1",[request],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).map_err(e)?;
        let card:Option<(i64,i64,Option<i64>)>=tx.query_row("SELECT request_id,chat,topic FROM outbox WHERE project=?1 AND task=?2 AND revision=?3 AND ((chat=?4 AND topic IS ?5) OR (chat=?6 AND topic IS NULL)) AND kind='question' ORDER BY id DESC LIMIT 1",
            params![project,task,revision,chat,topic,sender],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).optional().map_err(e)?;
        let Some((source,chat,topic))=card else {return Err("this route has no card for this question revision; explain it in your reply instead; nothing was changed".into())};
        let settled:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM outbox WHERE project=?1 AND task=?2 AND revision=?3 AND chat=?4 AND topic IS ?5 AND kind IN ('question_answered','question_superseded'))",
            params![project,task,revision,chat,topic],|x|x.get(0)).map_err(e)?;
        if settled {return Err("this question was already answered or superseded; read the current state; nothing was changed".into());}
        let same:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM outbox WHERE request_id=?1 AND project=?2 AND task=?3 AND revision=?4 AND kind='question_explained' AND text=?5 AND state<>'superseded')",
            params![source,project,task,revision,text],|x|x.get(0)).map_err(e)?;
        if !same {
            tx.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND project=?2 AND task=?3 AND revision=?4 AND kind='question_explained' AND state='pending'",params![source,project,task,revision]).map_err(e)?;
            let delivery=notice(&tx,source,chat,topic,"question_explained",project,task,revision,text,at)?;
            tx.execute("UPDATE outbox SET question=?1 WHERE id=?2",params![question.to_string(),delivery]).map_err(e)?;
            tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'task_explanation',?3,?4)",params![request,at,format!("{project}#{task}"),audit]).map_err(e)?;
        }
        tx.commit().map_err(e)?;
        Ok("The question card now carries this explanation and its source details; do not repeat them in your reply. Explaining answered nothing.".into())
    }

    /// Questions awaiting an accepted-answer receipt; their original revision remains the answer fence (§5, #94).
    pub fn open_questions(&self, project: &str, task: i64) -> Result<Vec<i64>, String> {
        let mut q = self.0.prepare("SELECT DISTINCT revision FROM outbox o WHERE project=?1 AND task=?2 AND kind='question'
            AND NOT EXISTS (SELECT 1 FROM outbox a WHERE a.project=o.project AND a.task=o.task AND a.revision=o.revision AND a.kind='question_answered')").map_err(|e|e.to_string())?;
        q.query_map(params![project,task],|r|r.get(0)).and_then(Iterator::collect).map_err(|e|e.to_string())
    }

    pub fn question_answer(&self, project: &str, task: i64, revision: i64) -> Result<Option<String>, String> {
        self.0.query_row("SELECT text FROM outbox WHERE project=?1 AND task=?2 AND revision=?3 AND kind='question_answered' LIMIT 1",
            params![project,task,revision],|r|r.get(0)).optional().map_err(|e|e.to_string())
    }

    /// Persist the receipt and its edit obligation together; duplicate observations never enqueue another edit.
    pub fn answer_question(&self, project: &str, task: i64, revision: i64, answer: &str, at: i64) -> Result<(), String> {
        let question: Option<String> = self.0.query_row("SELECT text FROM outbox WHERE project=?1 AND task=?2 AND revision=?3 AND kind='question' LIMIT 1",
            params![project,task,revision],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        let Some(question) = question else { return Ok(()) };
        let html = crate::telegram::html(&format!("{question}\n\n{answer}"));
        self.0.execute("INSERT INTO outbox(request_id,chat,topic,text,html,next_at,kind,project,task,revision)
            SELECT request_id,chat,topic,?4,?5,?6,'question_answered',project,task,revision FROM outbox o
            WHERE project=?1 AND task=?2 AND revision=?3 AND kind='question'
            AND NOT EXISTS (SELECT 1 FROM outbox a WHERE a.project=o.project AND a.task=o.task AND a.revision=o.revision AND a.kind='question_answered') LIMIT 1",
            params![project,task,revision,answer,html,at]).map(drop).map_err(|e|e.to_string())
    }

    /// #165: a closing observation updates the launch status card once; its evidence stays in audit.
    pub fn close_task(&self, project: &str, task: i64, text: &str, at: i64) -> Result<bool, String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        let row: Option<(i64, i64, Option<i64>, i64)> = tx
            .query_row("UPDATE tasks SET closed=1, observed=?2 WHERE number=?1 AND project=?3 AND closed=0 RETURNING request_id, chat, topic, revision", params![task, at, project], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .optional()
            .map_err(e)?;
        let Some((request, chat, topic, revision)) = row else { return Ok(false) };
        tx.execute("UPDATE outbox SET state='superseded' WHERE request_id=?1 AND kind='status' AND state='pending'", [request]).map_err(e)?;
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'task_result',?3,?4)",params![request,at,format!("{project}#{task}"),text]).map_err(e)?;
        let status=self.launch_status(request,at)?;
        notice(&tx, request, chat, topic, "status", project, task, revision, &status, at)?;
        tx.commit().map_err(e)?;
        Ok(true)
    }

    /// #338: called again by a worker holding an earlier answer-list snapshot.
    pub fn saved_answer(&self, request: i64, at: i64) -> Result<bool,String> {
        let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        let completed=saved_answers(&tx,at)?;
        answer_confirmations(&tx,at,None)?;
        let answered=tx.query_row("SELECT state='answered' FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())?;
        for id in completed {self.trace(id,at);}
        Ok(answered)
    }

    /// Answers waiting to be checked and written, oldest first; `in flight` ones were taken before a restart and resume their
    /// write with the same op ID.
    pub fn answers(&self) -> Result<Vec<(Answer, bool)>, String> {
        let mut q = self
            .0
            .prepare(
                "SELECT q.id, q.sender, coalesce(q.text, ''), o.task, o.revision, q.state='running', o.project FROM requests q JOIN outbox o ON o.kind='question' AND o.chat=q.chat
                AND o.sent_message=q.reply_to AND o.topic IS q.topic WHERE q.kind='answer' AND q.state IN ('queued', 'running') AND NOT EXISTS(SELECT 1 FROM audit_events a WHERE a.request_id=q.id AND a.kind IN ('media_group','album_member') AND a.at>?1-2) ORDER BY q.id",
            )
            .map_err(|e| e.to_string())?;
        q.query_map([crate::now() as i64], |r| Ok((Answer { request: r.get(0)?, sender: r.get(1)?, text: r.get(2)?, task: r.get(3)?, revision: r.get(4)?, project: r.get(6)? }, r.get(5)?)))
            .and_then(Iterator::collect)
            .map_err(|e| e.to_string())
    }

    /// Marks an answer in flight before its board write.
    pub fn take_answer(&self, request: i64) -> Result<(), String> {
        let tx=self.0.unchecked_transaction().map_err(|e|e.to_string())?;
        tx.execute("UPDATE requests SET state='running' WHERE id=?1 AND kind='answer'", [request]).map_err(|e|e.to_string())?;
        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![format!("request-activity:{request}"),crate::now() as i64]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())
    }

    /// Ends an answer request with its reply in its route, and records the board write in its audit record.
    pub fn end_answer(&self, a: &Answer, state: &str, reply: &str, error: Option<&str>, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        let ru:bool=tx.query_row("SELECT coalesce(language,'en') LIKE 'ru%' FROM requests WHERE id=?1",[a.request],|x|x.get(0)).map_err(e)?;
        let diagnostic=reply;
        let localized=match (ru,state) {
            (true,"failed")=>Some("Ответ не сохранён; проверьте актуальный вопрос и права.".to_string()),
            (true,"uncertain")=>Some("Сохранение ответа не подтверждено; проверьте статус задачи перед повторным ответом.".to_string()),
            (false,"failed")=>Some("Answer was not saved; check the current question and your access.".to_string()),
            (false,"uncertain")=>Some("Answer delivery is not confirmed; check the task before answering again.".to_string()),
            _=>None,
        };
        let reply=localized.as_deref().unwrap_or(reply);
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'answer_receipt',?3,?4)",params![a.request,at,format!("{}#{} revision {}",a.project,a.task,a.revision),diagnostic]).map_err(e)?;
        tx.execute("UPDATE requests SET state=?2, reply=?3, error=?4 WHERE id=?1", params![a.request, state, reply, error]).map_err(e)?;
        tx.execute(
            "INSERT INTO audit_events(request_id, at, kind, target, item, result) VALUES (?1,?2,'task',?3,?4,?5)",
            params![a.request, at, format!("{} answer #{}", a.project, a.task), format!("revision {}", a.revision), state],
        )
        .map_err(e)?;
        let (chat, topic, message): (i64, Option<i64>, i64) = tx.query_row("SELECT chat, topic, message FROM requests WHERE id=?1", [a.request], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(e)?;
        // a numeric callback ID in the INTEGER `value` column reads back as text; new ones are stored as bytes, unconverted
        let callback: Option<String> = tx.query_row("SELECT value FROM meta WHERE key=?1", [format!("callback:{}", a.request)], |r|r.get::<_, rusqlite::types::Value>(0)).optional().map_err(e)?.and_then(|v| match v {
            rusqlite::types::Value::Blob(b) => String::from_utf8(b).ok(),
            rusqlite::types::Value::Text(t) => Some(t),
            rusqlite::types::Value::Integer(i) => Some(i.to_string()),
            _ => None,
        });
        if let Some(callback) = callback {
            outbox(&tx, a.request, chat, topic, None, None, reply, at)?;
            tx.execute("UPDATE outbox SET kind=?2 WHERE id=?1", params![tx.last_insert_rowid(),format!("toast:{callback}")]).map_err(e)?;
        } else {
            final_outbox(&tx, a.request, chat, topic, (chat != a.sender).then_some(message), reply, at)?;
        }
        answer_confirmations(&tx,at,Some(a.request))?;
        tx.commit().map_err(e)?;
        self.trace(a.request, at);
        Ok(())
    }

    /// Records a direct TaskQ write of a run (status read, park, queue pause) in its request's audit record.
    pub fn task_event(&self, request: i64, target: &str, result: &str, at: i64) -> Result<(), String> {
        self.0
            .execute("INSERT INTO audit_events(request_id, at, kind, target, result) VALUES (?1,?2,'task',?3,?4)", params![request, at, target, result])
            .map(drop)
            .map_err(|e| e.to_string())
    }

    /// The route of a request as key grants and audit filters name it: `cli`, `CHAT` or `CHAT:TOPIC`.
    pub fn route(&self, request: i64) -> Result<String, String> {
        self.0
            .query_row("SELECT CASE WHEN front='cli' THEN 'cli' ELSE chat || coalesce(':' || topic, '') END FROM requests WHERE id=?1", [request], |r| r.get(0))
            .map_err(|e| e.to_string())
    }

}
