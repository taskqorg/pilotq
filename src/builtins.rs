//! Deterministic controls outside the model queue (#106, R10, R13, R17, §12).
use crate::{journal::Journal, telegram::Shared};
use coreq::Tool;
use serde_json::Value;
use std::time::{Duration, Instant};


pub struct Request { pub id:i64, pub session:String, pub sender:i64, pub text:String }

/// Each child has an absolute deadline and a process group; killed diagnostics cannot leave a CLI behind.
fn setup_report(exe:&std::path::Path,budget:Duration) -> Result<String,String> {
    use std::io::Read;
    use std::process::{Command,Stdio};
    let mut command=Command::new(exe);
    command.args(["doctor","--json"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::inherit());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command,0);
    let mut child=command.spawn().map_err(|e| { crate::host::log("pilotq doctor --json",&e.to_string()); "Setup checks could not start" })?;
    let stdout=child.stdout.take().unwrap();
    let reader=std::thread::spawn(move || { let mut out=Vec::new(); stdout.take(65536).read_to_end(&mut out).map(|_|out) });
    let end=Instant::now()+budget;
    let completed=loop {
        match child.try_wait() {
            Ok(Some(_))=>break true,
            Ok(None) if Instant::now()<end=>std::thread::sleep(Duration::from_millis(25)),
            _=>break false,
        }
    };
    #[cfg(unix)]
    let _=crate::host::output(Command::new("kill").args(["-KILL","--",&format!("-{}",child.id())]));
    let _=child.kill(); let _=child.wait();
    if !completed { return Err("Setup checks timed out; dependency state is unknown.".into()); }
    let output=reader.join().map_err(|_|"Setup checks failed")?.map_err(|_|"Setup checks failed")?;
    let v:Value=serde_json::from_slice(&output).map_err(|_|"Setup checks returned no report")?;
    let mut lines=vec!["Setup checks (read only):".to_string()];
    for name in ["configuration","telegram","codex","taskq","voice","host_tools"] {
        let state=match v["checks"][name].as_str() { Some("ok")=>"ok",Some("failed")=>"failed",_=>"unknown" };
        lines.push(format!("{name}: {state}"));
    }
    Ok(lines.join("\n"))
}

pub fn execute(t:&Tool,j:&Shared,r:&Request,installed:bool) -> Result<(),String> {
    if !j.lock().unwrap().builtin_allowed(r)? { return j.lock().unwrap().builtin_finish(r.id,"Access revoked.",false,crate::now() as i64); }
    let cmd=r.text.split_whitespace().next().unwrap_or("").split('@').next().unwrap_or("");
    let result=(|| {
        match cmd {
            "/release"|"/release-confirm"=>return crate::release::control(t,j,r).map(|s|Some(format!("Release: {s}"))).map_err(|s|format!("Release: {s}")),
            "/onboard"|"/onboard-button"=> {
                crate::onboarding::control(t,j,r)?;
                return Ok(None);
            }
            "/help"=> {
                let words:Vec<_>=r.text.split_whitespace().collect();
                if words.len()>2 { return Err("Invalid command arguments; use /help.".into()); }
                let config=crate::config(t)?;
                let keys=crate::keys::Keys(t.install.join("keys.json"));
                let (mut text,language)={
                    let j=j.lock().unwrap();
                    let language:Option<String>=j.0.query_row("SELECT language FROM requests WHERE id=?1",[r.id],|x|x.get(0)).map_err(|e|e.to_string())?;
                    (crate::catalog::render(&j,r.sender,words.get(1).copied(),language.as_deref(),&keys,&j.route(r.id)?)?,language)
                };
                text.push('\n');
                text.push_str(&keys.scrub(&crate::catalog::taskq_help(config.as_ref().and_then(|c|c.taskq.as_ref()).map(|t|t.bin.as_path()),language.as_deref())));
                return Ok(Some(text));
            }
            "/model"|"/model-button"=> {
                let config=crate::config(t)?.ok_or("PilotQ configuration unavailable")?;
                let models=config.codex.models()?;
                j.lock().unwrap().builtin_models(r,&models,crate::now() as i64)?;
                return Ok(None);
            }
            "/doctor" if r.text.split_whitespace().count()==1=> {
                let exe=std::env::current_exe().map_err(|_|"Setup checks unavailable")?;
                return setup_report(&exe,Duration::from_secs(20)).map(Some);
            }
            "/restart" if r.text.split_whitespace().count()==1=> {
                let config=crate::config(t)?.ok_or("PilotQ configuration unavailable")?;
                if r.sender!=config.owner { return Err("Only the configured owner can restart PilotQ.".into()); }
                if !installed { return Err("Service restart is unavailable in a source run.".into()); }
                let _lock=coreq::file_lock::try_lock(&t.install.join("update.lock")).map_err(|_|"Another update or restart is active; retry later.")?;
                if crate::drain(t,crate::DRAIN_GRACE).is_err() {
                    crate::resume_after_update(t,true)?;
                    return Err("Restart refused: active requests did not drain; admission resumed.".into());
                }
                // Recheck immediately before the effect, including a revocation during drain.
                if !j.lock().unwrap().builtin_allowed(r)? {
                    crate::resume_after_update(t,true)?;
                    return Err("Restart refused: access revoked; admission resumed.".into());
                }
                if let Err(error)=j.lock().unwrap().builtin_restart(r.id) {
                    crate::resume_after_update(t,true)?;
                    return Err(error);
                }
                std::process::exit(75);
            }
            _=>Err("Invalid command arguments; use /help.".into()),
        }
    })();
    match result {
        Ok(None)=>Ok(()),
        Ok(Some(text))=>j.lock().unwrap().builtin_finish(r.id,&text,true,crate::now() as i64),
        Err(text)=>j.lock().unwrap().builtin_finish(r.id,&text,false,crate::now() as i64),
    }
}

pub fn serve(t:Tool,j:Shared,installed:bool) {
    loop {
        let request=j.lock().unwrap().builtin_next();
        match request {
            Ok(Some(r)) if matches!(r.text.split_whitespace().next().unwrap_or("").split('@').next(),Some("/restart"|"/release-confirm"))=> {
                // The drain may wait for an active chat; other controls must remain available.
                let j=j.clone();
                std::thread::spawn(move || { if let Err(error)=execute(&crate::tool(),&j,&r,installed) { eprintln!("builtin: {error}"); } });
            }
            Ok(Some(r))=>if let Err(error)=execute(&t,&j,&r,installed) { eprintln!("builtin: {error}"); },
            Ok(None)=>std::thread::sleep(Duration::from_millis(100)),
            Err(error)=> { eprintln!("builtin: {error}"); std::thread::sleep(Duration::from_secs(1)); }
        }
    }
}

/// Operator controls complete without constructing a manager run. Service restarts stay owned by the running service.
pub fn cli(t:&Tool,j:Journal,id:i64,text:&str)->Result<Value,String> {
    let j=std::sync::Arc::new(std::sync::Mutex::new(j));
    let restart=text.split_whitespace().next().unwrap_or("")=="/restart";
    if !restart {
        let request=j.lock().unwrap().builtin_take(id)?;
        if let Some(r)=request { execute(t,&j,&r,false)?; }
    }
    let end=Instant::now()+Duration::from_secs(if restart {140} else if text.starts_with("/release-confirm") {910} else if text.starts_with("/release") {65} else if text.starts_with("/onboard") {65} else {25});
    loop {
        let (state,reply,error)=j.lock().unwrap().request(id)?;
        if !matches!(state.as_str(),"queued"|"waiting") || Instant::now()>=end {
            return Ok(serde_json::json!({"request":id,"state":state,"reply":reply,"error":error}));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(all(test,unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    // A2/A4 / R13/R17: CLI help uses the same admitted control, catalog and durable reply, without Codex.
    #[test]
    fn command_163_help_uses_control_audit() {
        let mut t=crate::tool();
        t.install=std::env::temp_dir().join(format!("pilotq-help-control-{}",std::process::id()));
        std::fs::create_dir_all(&t.install).unwrap();
        let mut journal=Journal::open(&t.install.join("journal.sqlite")).unwrap();journal.bootstrap(7,0).unwrap();
        let active=journal.ask(7,"ordinary conversation",1).unwrap();
        let id=journal.builtin_cli(7,"/help projects",2).unwrap().unwrap();
        let j=std::sync::Arc::new(std::sync::Mutex::new(journal));
        let request=j.lock().unwrap().builtin_take(id).unwrap().unwrap();
        execute(&t,&j,&request,false).unwrap();
        let mut guard=j.lock().unwrap();let (state,reply,error)=guard.request(id).unwrap();
        assert_eq!(state,"answered");assert!(error.is_none());
        let reply=reply.unwrap();assert!(reply.contains("project_disconnect") && reply.contains("TaskQ catalog unavailable"));
        assert_eq!(guard.request(active.request).unwrap().0,"running");
        assert!(guard.builtin_take(id).unwrap().is_none(),"duplicate control must not replay");
        drop(guard);drop(j);std::fs::remove_dir_all(t.install).unwrap();
    }

    // #106 / R16, §7: a broken setup dependency must terminate, and arbitrary diagnostics never become replies.
    #[test]
    fn u106_doctor_deadline_and_redaction() {
        let dir=std::env::temp_dir().join(format!("pilotq-command-doctor-{}",std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script=dir.join("doctor");
        std::fs::write(&script,"#!/bin/sh\necho '{\"checks\":{\"configuration\":\"ok\",\"codex\":\"invented-secret\"},\"secret\":\"invented-secret\"}'\necho invented-secret >&2\n").unwrap();
        std::fs::set_permissions(&script,std::fs::Permissions::from_mode(0o700)).unwrap();
        let report=setup_report(&script,Duration::from_secs(1)).unwrap();
        assert!(report.contains("configuration: ok") && report.contains("codex: unknown"));
        assert!(!report.contains("invented-secret"));
        std::fs::write(&script,"#!/bin/sh\nsleep 30\n").unwrap();
        let start=Instant::now();
        assert!(setup_report(&script,Duration::from_millis(50)).unwrap_err().contains("timed out"));
        assert!(start.elapsed()<Duration::from_secs(2));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

/// Command failures follow the person's language; detailed dependency errors stay in the request audit (§12).
pub fn localized(text:&str,language:Option<&str>,failed:bool)->String {
    if !language.unwrap_or("en").starts_with("ru") || text.chars().any(|c| matches!(c,'А'..='я'|'ё'|'Ё')) { return text.into(); }
    if let Some(reason)=text.strip_prefix("Release: ") {
        return format!("Выпуск: {}",reason.replace("Release blocked by ","мешает выполняемая работа: ").replace("task #","задача #").replace("attempt ","попытка ").replace("release verified","выпуск проверен").replace("release outcome uncertain; no automatic retry","результат выпуска неизвестен; автоматического повтора не будет"));
    }
    match text {
        "Unknown command; use /help."=>"Неизвестная команда; используйте /help.".into(),
        "Invalid command arguments; use /help."=>"Неверные аргументы команды; используйте /help.".into(),
        "Only the configured owner can restart PilotQ."=>"Перезапуск PilotQ доступен только владельцу установки.".into(),
        "Service restart is unavailable in a source run."=>"Перезапуск сервиса недоступен при запуске из исходников.".into(),
        "Another update or restart is active; retry later."=>"Уже идёт обновление или перезапуск; повторите позже.".into(),
        "Restart requested; waiting for active requests to drain."=>"Перезапуск запрошен; жду завершения активных запросов.".into(),
        "Restart refused: active requests did not drain; admission resumed."=>"Перезапуск отменён: активные запросы не завершились; приём работы возобновлён.".into(),
        "Restart refused: access revoked; admission resumed."=>"Перезапуск отменён: доступ отозван; приём работы возобновлён.".into(),
        "PilotQ restarted and wrote healthy status. Earlier requests and TaskQ work remain recorded."=>"PilotQ перезапущен и записал healthy status; прежние запросы и работа TaskQ сохранены.".into(),
        "No skills are enabled for PilotQ's restricted manager. /skill NAME TEXT is unavailable until a skill is approved."=>"Для менеджера PilotQ нет разрешённых skills; /skill NAME TEXT недоступна до их согласования.".into(),
        "Checking setup."=>"Проверяю настройки.".into(),
        "Checking model catalogue."=>"Проверяю каталог моделей.".into(),
        "This model selector is stale or belongs to another person; use /model."=>"Эта кнопка устарела или принадлежит другому человеку; используйте /model.".into(),
        "Conversation was reset; use /model again."=>"Диалог сброшен; используйте /model заново.".into(),
        "Model is not in the current Codex catalogue; choice unchanged."=>"Модели нет в текущем каталоге Codex; выбор не изменён.".into(),
        "Use /model, /model MODEL or /model default."=>"Используйте /model, /model MODEL или /model default.".into(),
        _ if failed=>"Команда не выполнена; проверьте /status и повторите позже.".into(),
        _=>text.into(),
    }
}
