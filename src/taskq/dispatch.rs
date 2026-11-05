//! Scoped board reads, immutable target resolution and tool dispatch (§5).
use super::*;

fn board_status(tq: &Taskq, j: &Mutex<Journal>, r: &Run, args: &Value, sender: i64, turn_end: std::time::Instant) -> Result<Option<String>, String> {
    // #242/#370 / §5: status reads may name several boards, including release observations.
    // Historical context and model arguments cannot narrow an unbound broad read.
    {
        let projects = {
            let guard=j.lock().unwrap();
            let projects=guard.projects()?.into_iter().filter(|p|allowed(&guard,sender,&p.0,false).is_ok()).collect::<Vec<_>>();
            let named=projects.iter().filter(|p|crate::direct::named_target(&r.text,&p.0) || crate::direct::named_target(&r.text,&p.1)).cloned().collect::<Vec<_>>();
            let own=crate::direct::own_text(&r.text).to_lowercase();
            let words:Vec<_>=own.split(|c:char| !c.is_alphanumeric()).filter(|w| !w.is_empty()
                && !matches!(*w,"the"|"of"|"my"|"our"|"registered"|"accessible"|"available"|"по"|"на"
                    |"мои"|"моих"|"моим"|"моими"|"наши"|"наших"|"нашим"|"нашими")
                && !w.starts_with("доступн") && !w.starts_with("зарегистрированн")).collect();
            let all_boards=words.windows(2).enumerate().any(|(i,w)| (i==0 || !matches!(words[i-1],"not"|"не"))
                && matches!(w[0],"all"|"every"|"все"|"всех"|"всем"|"всеми"|"каждой"|"каждом"|"каждый"|"каждая"|"каждую")
                && (matches!(w[1],"board"|"boards"|"project"|"projects"|"досок") || w[1].starts_with("доск") || w[1].starts_with("проект")));
            // §5: current all-board intent overrides a model-supplied follow-up source.
            if named.is_empty() && !all_boards && !args["source_request"].is_null() {
                let source=args["source_request"].as_i64().ok_or("status source_request must be an input ID")?;
                // §2/§5: semantic follow-ups bind a completed one-board read, never ambiguous history or a model guess.
                let target:String=guard.0.query_row("SELECT min(a.target) FROM requests q JOIN runs x ON x.request_id=q.id
                    JOIN audit_events a ON a.request_id=q.id AND a.kind='task' AND a.result='read' AND a.target LIKE '% project status'
                    WHERE q.id=?1 AND q.id<?2 AND q.sender=?3 AND q.session=?4 AND x.generation=?5 AND q.state='answered'
                    GROUP BY q.id HAVING count(DISTINCT a.target)=1",
                    rusqlite::params![source,r.request,sender,r.session,r.generation],|row|row.get(0))
                    .map_err(|_|"Follow-up source must be a completed one-board status from this person in this conversation generation")?;
                let scoped:Vec<_>=projects.into_iter().filter(|p|format!("{} project status",p.1)==target).collect();
                if scoped.len()!=1 {return Err("Follow-up board is no longer uniquely registered or accessible".into());}
                if args["project"].as_str().map(crate::direct::canonical_project).is_some_and(|p|p!=scoped[0].0) {
                    return Err("project contradicts the follow-up source; nothing was done".into());
                }
                Some(scoped)
            } else if named.len()>1 {Some(named)} else if named.is_empty() && (guard.unbound_topic(r)? || guard.current_project(&r.session)?.is_none()) {Some(projects)} else {None}
        };
        if let Some(projects)=projects {
            if projects.is_empty() {return Err("No accessible registered board; status unavailable".into());}
            let mut boards=Vec::new();
            let mut seen=std::collections::HashSet::new();
            for (project,board, ..) in projects {
                if !seen.insert(board.clone()) {continue;}
                let result=(|| {
                    let bound={
                        let guard=j.lock().unwrap();
                        if guard.fenced(r)? || !guard.has(sender,"chat")? {return Err("this run was stopped or access was revoked".into());}
                        allowed(&guard,sender,&project,false)?;
                        tq.registered(&guard,&project)?.0
                    };
                    project_status_read(&bound,j,r,&project,&board,args["include_release"]==true,turn_end)
                })();
                boards.push(match result {
                    Ok(status)=>status,
                    Err(reason)=>{
                        j.lock().unwrap().task_event(r.request,&format!("{board} project status"),&format!("unavailable: {reason}"),now())?;
                        json!({"project":project,"board":board,"unavailable":reason})
                    },
                });
            }
            return Ok(Some(json!({"boards":boards}).to_string()));
        }
    }
    Ok(None)
}

pub(super) fn dispatch_until(tq: &Taskq, j: &Mutex<Journal>, r: &Run, name: &str, args: &Value,turn_end:std::time::Instant) -> Result<String, String> {
    let sender = {
        let j = j.lock().unwrap();
        // a stopped or replaced run emits no authorized tool call (§ 4 Runs)
        if j.fenced(r)? {
            return Err("this run was stopped; nothing was done".into());
        }
        j.sender(r.request)?
    };
    if !j.lock().unwrap().has(sender,"chat")? { return Err("conversation access was revoked; nothing was done".into()); }
    if name == "discussion_answer" {
        return j.lock().unwrap().discussion_answer(r,args["card"].as_i64().ok_or("card is required")?,args["text"].as_str().ok_or("text is required")?,args["source_request"].as_i64());
    }
    if name == "discussion_choices" {
        let question=args["question"].as_str().ok_or("question is required")?;
        let options:Vec<String>=serde_json::from_value(args["options"].clone()).map_err(|_|"options must be strings")?;
        j.lock().unwrap().save_choices(r,question,&options)?;
        return Ok("Discussion choices saved locally; the final reply appends them. Choosing is an answer, not launch authorization.".into());
    }
    if name == "task_run_plan" && j.lock().unwrap().is_choice(r)? {
        return Err("a discussion choice never authorizes launch; ask for the separate launch button or a later explicit request".into());
    }
    // queue pause is host-scoped; other tools resolve the same target rules as issue tools.
    if name == "queue_pause" {
        let guard = j.lock().unwrap();
        if !guard.has(sender, "queue")? { return Err(format!("{sender} has no queue grant; nothing was paused")); }
        drop(guard);
        tq.cli(&["service".into(), "pause".into()])?;
        j.lock().unwrap().task_event(r.request, "service pause", "accepted", now())?;
        let running = tq.snapshot().map(|s| s["tasks"].as_array().into_iter().flatten().filter(|t| t["state"] == "doing").map(|t| format!("#{}", t["n"])).collect::<Vec<_>>().join(", ")).unwrap_or("unknown".into());
        return Ok(format!("TaskQ admission is paused on this host; no new task starts. Running workers continue. Still running: {running}."));
    }
    if name == "project_status" && let Some(status) = board_status(tq, j, r, args, sender, turn_end)? { return Ok(status); }
    let saved = if name == "task_run_plan" {
        Some(j.lock().unwrap().plan_target(&r.session, args["revision"].as_i64())?)
    } else { None };
    // Resolve new task URLs by exact fresh CLI readback, never by forge-specific path parsing.
    let link_target = if let Some(url)=args["task"].as_str().filter(|s|s.starts_with("https://")) {
        let (cached,projects,own)={
            let guard=j.lock().unwrap();
            let projects=guard.projects()?.into_iter().filter(|p| allowed(&guard,sender,&p.0,false).is_ok()
                && args["project"].as_str().map(crate::direct::canonical_project).is_none_or(|name|name==p.0)).collect::<Vec<_>>();
            let mut own=guard.own_project_inputs(r)?;own.push(r.text.clone());
            (guard.url_target(url)?,projects,own)
        };
        if cached.is_some() { None } else {
            if !own.iter().any(|text|crate::direct::own_url(text,url)) {return Err("task URL was not named by this person; nothing was done".into());}
            let mut matches=Vec::new();
            let mut unavailable=false;
            for (project,board,root, ..) in projects {
                let bound=Taskq {bot:tq.bot.clone(),bin:tq.bin.clone(),root:root.into(),poll:tq.poll};
                match bound.snapshot_for(&board) {
                    Ok(snapshot)=>for task in snapshot["tasks"].as_array().into_iter().flatten() {
                        if task["url"]==url {
                            if let Some(n)=task["n"].as_i64().filter(|n|*n>0) { matches.push((project.clone(),board.clone(),n)); }
                        }
                    },
                    Err(_)=>unavailable=true,
                }
            }
            if matches.len()!=1 {return Err(if unavailable {"task URL could not be verified: a registered board is unavailable"} else {"task URL does not identify one accessible TaskQ task"}.into());}
            matches.pop()
        }
    } else { None };
    let (project, board, n) = {
        let j = j.lock().unwrap();
        let mut target = args.clone();
        target["issue"] = args.get("task").cloned().unwrap_or(if matches!(name,"task_plan" | "task_run_plan" | "project_status" | "project_release_preview") { json!("1") } else { Value::Null });
        if let Some(target) = &link_target {
            target.clone()
        } else if let Some((_, project, _, _)) = &saved {
            if args["project"].as_str().map(crate::direct::canonical_project).is_some_and(|p| p != project) { return Err("saved plan cannot be retargeted; nothing was filed".into()); }
            allowed(&j, sender, project, true)?;
            (project.clone(), saved.as_ref().unwrap().2.clone(), 0)
        } else if matches!(name,"task_status" | "task_steer" | "task_answer") {
            let inferred = if name == "task_answer" || args["project"].is_string() { None } else { crate::direct::context_project(&j,r,sender)? };
            match j.related_target(r,args["task"].as_str().ok_or("task is required")?,args["project"].as_str().map(crate::direct::canonical_project).or(inferred.as_deref()))? {
                Some(target) => target,
                None => if name == "task_answer" { crate::direct::target(&j,r,sender,&target)? } else { crate::direct::target_context(&j,r,sender,&target)? },
            }
        } else { crate::direct::target_context(&j, r, sender, &target)? }
    };
    let (bound, registered_board) = tq.registered(&j.lock().unwrap(), &project)?;
    let tq = &bound;
    if saved.as_ref().is_some_and(|(_, _, _, root)| registered_board != board || (!root.is_empty() && PathBuf::from(root) != tq.root)) {
        return Err("saved plan registration changed; nothing was filed".into());
    }
    dispatch_target(tq, j, r, name, args, sender, &project, &board, n, saved.as_ref().map(|s|s.0), turn_end)
}

fn dispatch_target(tq: &Taskq, j: &Mutex<Journal>, r: &Run, name: &str, args: &Value, sender: i64, project: &str, board: &str, n: i64, revision: Option<i64>, turn_end: std::time::Instant) -> Result<String, String> {
    // #212 A5: unrelated status reads do not collect release or health projections.
    let release_question = args["include_release"].as_bool().unwrap_or(false);
    let create = tq.check(board)?;
    match name {
        "project_release_preview" => crate::release::preview(tq,j,r,project),
        "task_plan" => {
            allowed(&j.lock().unwrap(), sender, project, false)?;
            let language=j.lock().unwrap().language(r.request)?;
            let _=crate::onboarding::launch_gate(tq,j,project,language.as_deref(),turn_end);
            let placed = args["tickets"].as_array().is_some_and(|t| t.iter().any(|x| !x["host"].is_null()));
            let hosts: Vec<String> = if placed {
                tq.snapshot_for(board)?["hosts"].as_array().into_iter().flatten().filter_map(|h| h["name"].as_str().map(String::from)).collect()
            } else { vec![] };
            let t = tickets(args, &hosts)?;
            let rev = j.lock().unwrap().save_draft(r, &t, project, board, &tq.root.to_string_lossy(), now())?;
            let list: Vec<String> = t.as_array().unwrap().iter().enumerate().map(|(i, x)| format!("{}. {}", i + 1, x["title"].as_str().unwrap_or(""))).collect();
            Ok(format!("Draft updated locally; nothing is filed. Internal launch revision: {rev}. Tickets:\n{}\nNow send a nonempty final answer: one short acknowledgement in the person’s language, such as «План подготовлен.» or «Plan prepared.». PilotQ replaces it with this updated draft and Launch/Cancel buttons. An empty final answer fails the request and discards its draft. Do not repeat the draft or internal revision. A later explicit text request can also run it.", list.join("\n")))
        }
        "task_run_plan" => {
            let rev = revision.unwrap_or(0);
            allowed(&j.lock().unwrap(), sender, project, true)?;
            let language=j.lock().unwrap().language(r.request)?;
            // §5 / #245: preserve the owner's established refusal path; members get escalation.
            if !create && j.lock().unwrap().owner()?==Some(sender) {
                return Err("task creation is disabled: create_claim_issue is not configured for the resolved board, and plain add is never used. Nothing was filed.".into());
            }
            let gate_error=if !create {Some("task creation is disabled: create_claim_issue is not configured for the resolved board, and plain add is never used. Nothing was filed.".to_string())}
                else {crate::onboarding::launch_gate(tq,j,project,language.as_deref(),turn_end).err()};
            if let Some(reason)=gate_error {
                let reason=crate::onboarding::setup_refusal(&j.lock().unwrap(),r.request,project,language.as_deref(),&reason)?;
                j.lock().unwrap().0.execute("INSERT INTO audit_events(request_id,run_id,at,kind,result) VALUES (?1,?2,?3,'plan_gate_refusal',?4)",
                    rusqlite::params![r.request,r.id,now(),reason]).map_err(|e|e.to_string())?;
                return Err(reason);
            }
            // #110: filing and starting run in the service after this turn (`launch`), never inside the model's deadline
            let run = j.lock().unwrap().authorize(r, rev, sender, LEASE, now())?;
            if j.lock().unwrap().launch_run(r.request)? != Some(run) {
                return Ok("This draft was already launched by an earlier request; nothing new was filed. Use task_status for its tasks.".into());
            }
            Ok("Launch of the current draft accepted. PilotQ files and starts its tickets after this turn and replies with each task's board state; nothing is filed yet. Do not describe task states.".into())
        }
        "task_answer" => {
            let text=args["text"].as_str().unwrap_or("");
            let files=j.lock().unwrap().reply_file_source(r,args["source_request"].as_i64())?;
            let revision=args["revision"].as_i64().ok_or("question revision is required")?;
            {
                let guard=j.lock().unwrap();
                allowed(&guard,sender,project,true)?;
                if !text.is_empty() || files.is_empty() {guard.answer_source(r,args["source_request"].as_i64(),text)?;}
                if guard.related_target(r,args["task"].as_str().ok_or("task is required")?,Some(project))?.is_none()
                    || !guard.task_target_matches(project,n,board,&tq.root.to_string_lossy())? {
                    return Err("question must belong to this conversation's unchanged subscription; nothing was saved".into());
                }
                if let Some(receipt)=guard.question_answer(project,n,revision)? { return Ok(receipt); }
            }
            let snapshot=tq.snapshot_for(board)?;
            let task=find(&snapshot,n).ok_or("task is not open; nothing was saved; check task status")?;
            if task["state"]!="ask" || task["revision"]!=revision {
                return Err(format!("Question changed; nothing was saved. Read task_status and answer its current question. {}",task_link(board,n,task["title"].as_str(),task["url"].as_str())));
            }
            if !j.lock().unwrap().open_questions(project,n)?.contains(&revision) {
                return Err("question revision has not been relayed here; read the current task status before answering".into());
            }
            let mut argv=vec!["answer".into(),n.to_string(),"--revision".into(),revision.to_string()];
            if !text.is_empty() { argv.extend(["--text".into(),text.into()]); }
            attachment_args(&mut argv,&files);
            let op={
                let guard=j.lock().unwrap();
                allowed(&guard,sender,project,true)?;
                guard.task_intent(r,project,board,&tq.root.to_string_lossy(),&argv,now())?
            };
            reconcile_effect(tq,j,&op,board,&argv,false)
        }
        "project_status" => {
            allowed(&j.lock().unwrap(), sender, project, false)?;
            Ok(project_status_read(tq,j,r,project,board,release_question,turn_end)?.to_string())
        }
        "task_status" => {
            allowed(&j.lock().unwrap(), sender, project, false)?;
            let s = tq.task_snapshot(board,n)?;
            let text = match find(&s, n) {
                Some(t) => {
                    j.lock().unwrap().remember_task_url(project,n,t["url"].as_str(),now())?;
                    let mut text = format!("Task #{n} {} (revision {}, observed {}).\n{}", describe(t["state"].as_str().unwrap_or("")), t["revision"], crate::time::moment(s["observed"].as_i64(),now()), task_link(board, n, t["title"].as_str(), t["url"].as_str()));
                    for e in t["executions"].as_array().into_iter().flatten() {
                        text += &format!("\nAttempt {} ({}, {}): {}", e["attempt"], e["role"].as_str().unwrap_or(""), e["state"].as_str().unwrap_or(""), e["reason"].as_str().unwrap_or(""));
                    }
                    if let Some(f) = t["failed"].as_str() {
                        text += &format!("\nFailed: {f}");
                    }
                    if t["question"].is_object() {
                        text += &format!("\nOpen question:\n{}", question_body(&t["question"]));
                    }
                    if let Some(v) = verdict(&t["verdict"]) { text += &format!("\nHanded-in result:\n{v}"); }
                    if let Ok(inbox)=tq.inbox(n) {
                        text += &format!("\nTaskQ progress and accepted answers (source data): {inbox}");
                    }
                    let issue=tq.issue(n);
                    match &issue {
                        Ok(issue) => {
                            let open = tq.snapshot_for(board);
                            text += &format!("\n{}",task_issue_status(&issue,open.as_ref().ok(),n));
                        }
                        Err(e) => text += &format!("\nIssue state and closing evidence unavailable: {e}"),
                    }
                    if let (true,Ok(issue))=(release_question,&issue) {let end=std::time::Instant::now()+Duration::from_secs(10);let (dev,delivery)=crate::release::task_observations(j,r.request,project,issue,end);let ru=j.lock().unwrap().language(r.request)?.as_deref().unwrap_or("en").starts_with("ru");text += &format!("\n{}: {dev}\n{}: {}",if ru {"Проверка DEV (исходные данные)"}else{"DEV health"},if ru {"Доставка"}else{"Delivery"},crate::release::delivery_text_localized(&delivery,ru));}
                    text
                }
                None => {
                    let issue = tq.issue(n)?;
                    j.lock().unwrap().remember_task_url(project,n,issue["url"].as_str(),now())?;
                    let open = tq.snapshot_for(board);
                    let mut result:Value=serde_json::from_str(&task_issue_status(&issue, open.as_ref().ok(), n)).map_err(|_|"Task state unreadable")?;
                    if release_question {
                        let end=std::time::Instant::now()+Duration::from_secs(10);
                        let (dev,delivery)=crate::release::task_observations(j,r.request,project,&issue,end);
                        result["dev"]=dev;
                        result["delivery"]=json!(crate::release::delivery_text_localized(&delivery,j.lock().unwrap().language(r.request)?.as_deref().unwrap_or("en").starts_with("ru")));
                    }
                    result.to_string()
                },
            };
            j.lock().unwrap().task_event(r.request, &format!("{board} status #{n}"), "read", now())?;
            Ok(text)
        }
        "task_park" | "task_steer" | "task_requeue" => {
            allowed(&j.lock().unwrap(), sender, project, true)?;
            let (cmd, field) = match name { "task_steer" => ("steer", "text"), "task_requeue" => ("requeue", "reason"), _ => ("later", "reason") };
            let files=if cmd=="later" {vec![]} else {j.lock().unwrap().reply_file_source(r,args["source_request"].as_i64())?};
            if cmd=="requeue" && !files.is_empty() && j.lock().unwrap().owner()?!=Some(sender) { return Err("Only the installation owner may requeue with files; ask the owner to send this reply.".into()); }
            let value = args[field].as_str().unwrap_or("");
            if value.trim().is_empty() && files.is_empty() {return Err(format!("{field} or files are required"));}
            let text = if cmd == "steer" || value.is_empty() { value.to_string() } else { format!("Reason: {value}") };
            let limit = if cmd == "steer" { 4 } else { 1 };
            if text.lines().count() > limit || text.lines().any(|s| s.trim().is_empty() || s.chars().count() > 240) {
                return Err(format!("{field} must be 1-{limit} nonempty lines of at most 240 characters including the Reason prefix; nothing was written"));
            }
            let snapshot = tq.snapshot_for(board)?;
            let task = find(&snapshot, n).ok_or("task is not open; nothing was written")?;
            if cmd == "steer" && task["state"] != "doing" { return Err("only a doing task can be steered; nothing was written".into()); }
            let revision = task["revision"].as_i64().ok_or("task revision is unknown; nothing was written")?;
            let mut argv = vec![cmd.into(), n.to_string(), "--revision".into(), revision.to_string()];
            if !text.is_empty() {argv.extend(["--text".into(),text]);}
            attachment_args(&mut argv,&files);
            let op = {
                let j = j.lock().unwrap();
                if j.fenced(r)? { return Err("this run was stopped; nothing was written".into()); }
                allowed(&j, sender, project, true)?;
                j.task_intent(r, project, board, &tq.root.to_string_lossy(), &argv, now())?
            };
            reconcile_effect(tq, j, &op, board, &argv, false)
        }
        _ => Err(format!("unknown tool {name}")),
    }
}
