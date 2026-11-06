//! SQLite journal: admitted inputs, ignored-update dispositions, the polling cursor, the outbox (pilotq.md § 4, § 7),
//! the grant registry, route policy and per-person profiles (§ 2, § 6), and the request audit record (§ 8, R17).

use coreq::telemetry::{Event, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Known rebuild trigger, supplied by the caller rather than inferred from native state (§6, R17).
#[derive(Clone, Copy)]
pub enum RebuildReason { PolicyRefresh, ForeignRuntime, NativeUnavailable }

impl RebuildReason {
    fn audit(self) -> &'static str {
        match self {
            Self::PolicyRefresh => "planned policy refresh; journal context prepared",
            Self::ForeignRuntime => "foreign runtime; journal context prepared",
            Self::NativeUnavailable => "native session unavailable after failed resume; journal context prepared",
        }
    }
}

/// Schema this binary writes; a newer journal is refused rather than guessed (F08).
const SCHEMA: i64 = 14;

mod schema;
mod admission;
mod delivery;
mod people;
mod service;
mod requests;
mod tasks;
#[cfg(test)]
mod tests;
/// Delivery attempts per episode, persisted across restart (§ 7).
pub const ATTEMPTS: i64 = 5;
/// Named grants (§ 2, § 5, § 8). `chat` admits conversation only; every other grant is separate. `project:NAME` scopes a project;
/// `projects` covers every project; `queue` pauses host admission. Roles (#267): `admin` implies every named grant and every
/// project; `owner:NAME` implies `project:NAME` and TaskQ writes there only. Neither implies `release:NAME`.
pub const GRANTS: [&str; 8] = ["chat", "users", "projects", "taskq", "audit", "queue", "shell", "admin"];
/// Section 7 health sources, each reported separately.
pub const SOURCES: [&str; 5] = ["poll", "storage", "model", "taskq", "delivery"];
/// §9 / #81 owner incidents: source, opening and recovery text. Plain lines, no internal IDs or raw errors (§4).
const INCIDENTS: [(&str, &str, &str, &str, &str); 3] = [
    ("model", "⚠️ PilotQ: Codex runs are failing · new requests may fail · run pilotq doctor on the host", "✅ PilotQ: Codex answers again · waiting requests resume", "⚠️ PilotQ: запросы к Codex завершаются с ошибкой · новые запросы могут не пройти · выполните pilotq doctor на сервере", "✅ PilotQ: Codex снова отвечает · ожидающие запросы продолжаются"),
    ("taskq", "⚠️ PilotQ: TaskQ board unavailable · plans stay unsent · retrying", "✅ PilotQ: TaskQ board reachable again · held plans resume", "⚠️ PilotQ: доска TaskQ недоступна · планы не отправляются · проверка повторяется", "✅ PilotQ: доска TaskQ снова доступна · отложенные планы продолжаются"),
    ("poll", "⚠️ PilotQ: Telegram polling is failing · new messages wait", "✅ PilotQ: Telegram polling works again", "⚠️ PilotQ: приём сообщений Telegram не работает · новые сообщения ожидают", "✅ PilotQ: приём сообщений Telegram восстановлен"),
];
/// Hold-down before an incident opens; `model` instead needs two consecutive failed runs.
const HOLD: i64 = 120;
/// At most one service message per incident key per 6 hours and 10 per day.
const KEY_CAP: i64 = 6 * 3600;
const DAY_CAP: i64 = 10;
/// An undrained gap above this is reported as downtime, a shorter one as an unplanned restart.
const DOWNTIME: i64 = 300;
/// §7: one route progress message changes at most every thirty seconds.
const PROGRESS_EDIT: i64 = 30;

pub(crate) fn valid_grant(g: &str) -> bool {
    let name = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c));
    GRANTS.contains(&g) || ["release:", "project:", "owner:"].iter().any(|k| g.strip_prefix(k).is_some_and(name))
}

/// R17: rows are pruned oldest first and their times are gone, so a window is complete only if the
/// oldest row above the pruned range predates it. Any other pruned window can hide a failure.
fn complete_window(r: &coreq::telemetry::Reader, page: &Value, since_ms: i64) -> Result<(), String> {
    let through = page["pruned_through"].as_i64().ok_or("telemetry export has no pruned range")?;
    if page["pruned_count"].as_i64().unwrap_or(1) == 0 { return Ok(()); }
    let oldest = r.export(0, through, 1)?["events"][0]["received_ms"].as_i64();
    if oldest.is_some_and(|at| at < since_ms) { return Ok(()); }
    Err("telemetry pruned rows inside the reported window".into())
}

fn held(c: &Connection, user: i64, grant: &str) -> Result<bool, String> {
    c.query_row("SELECT 1 FROM grants WHERE user=?1 AND name=?2", params![user, grant], |_| Ok(())).optional().map(|r| r.is_some()).map_err(|e| e.to_string())
}

/// A stored grant or one its role implies (#267). `chat` stays the admission switch, so revoking it silences an admin too;
/// `release:NAME` stays a separate owner grant (§ 5).
pub(crate) fn has(c: &Connection, user: i64, grant: &str) -> Result<bool, String> {
    if held(c, user, grant)? { return Ok(true); }
    if let Some(p) = grant.strip_prefix("project:") && held(c, user, &format!("owner:{p}"))? { return Ok(true); }
    Ok(grant != "chat" && !grant.starts_with("release:") && !grant.starts_with("owner:") && held(c, user, "admin")?)
}

/// TaskQ decisions in `project` (answer, start, steer, park, requeue, close, file): admin, its owner, or legacy `taskq` with project scope.
pub(crate) fn decides(c: &Connection, user: i64, project: &str) -> Result<bool, String> {
    Ok(has(c, user, &format!("owner:{project}"))? || has(c, user, "taskq")? && (has(c, user, "projects")? || has(c, user, &format!("project:{project}"))?))
}

/// #267 A4: the one refusal line for a person without decision rights in `project`.
pub fn decide_refusal(project: &str) -> String {
    format!("Only an admin or the owner of project {project} can decide or start its tasks; ask one of them. Nothing was written.")
}

/// #267 A6: legacy grants map onto roles once; the old grants stay, so no one's access narrows.
/// `users` already allowed granting anything except release, so it maps to `admin`; `taskq` with project scope maps to project owner.
fn map_roles(c: &Connection, at: i64) -> Result<(), String> {
    let done: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM meta WHERE key='roles')", [], |r| r.get(0)).map_err(|e| e.to_string())?;
    if done { return Ok(()); }
    let mut q = c.prepare("SELECT DISTINCT user FROM grants").map_err(|e| e.to_string())?;
    let users: Vec<i64> = q.query_map([], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())?;
    let mut q = c.prepare("SELECT name FROM projects").map_err(|e| e.to_string())?;
    let registered: Vec<String> = q.query_map([], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())?;
    for u in users {
        let g = grants_of(c, u)?;
        let mut add: Vec<String> = vec![];
        if g.iter().any(|x| x == "users") { add.push("admin".into()); }
        else if g.iter().any(|x| x == "taskq") {
            if g.iter().any(|x| x == "projects") { add.extend(registered.iter().map(|p| format!("owner:{p}"))); }
            add.extend(g.iter().filter_map(|x| x.strip_prefix("project:")).map(|p| format!("owner:{p}")));
        }
        Journal::change_in(c, None, u, at, |set| set.extend(add))?;
    }
    c.execute("INSERT INTO meta(key, value) VALUES ('roles', ?1)", [at]).map(drop).map_err(|e| e.to_string())
}

fn grants_of(c: &Connection, user: i64) -> Result<Vec<String>, String> {
    let mut q = c.prepare("SELECT name FROM grants WHERE user=?1 ORDER BY name").map_err(|e| e.to_string())?;
    q.query_map([user], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())
}

/// Conversation key (§ 2): a DM is its own conversation; a group or topic is shared only when its route says so, else per person.
fn session(chat: i64, topic: Option<i64>, sender: i64, shared: bool) -> String {
    let topic = topic.unwrap_or(0);
    if chat == sender { format!("dm:{chat}") } else if shared { format!("{chat}:{topic}") } else { format!("{chat}:{topic}:{sender}") }
}

/// Recovery control in a message (§ 2 Changing direction), optionally addressed as `/stop@bot`.
fn control(text: Option<&str>) -> Option<&'static str> {
    let cmd = text?.split_whitespace().next()?.split('@').next()?;
    if !cmd.starts_with('/') { return None; }
    // #360: Telegram sends /start (optionally with a deep-link payload) as a new person's first message; it gets top-level help.
    if cmd=="/start" { return Some("/help"); }
    if matches!(cmd,"/model-button"|"/release-confirm"|"/onboard-button") { return Some(match cmd {"/model-button"=>"/model-button","/release-confirm"=>"/release-confirm",_=>"/onboard-button"}); }
    Some(crate::catalog::CATALOG.iter().find(|c|c.name==cmd && c.name.starts_with('/')).map(|c|c.name).unwrap_or("/unknown"))
}


/// §6/#344: actor+chat state is independent of topics, model sessions and lossy history.
fn chat_language(tx: &Connection, chat:i64, sender:i64, text:Option<&str>, label:Option<&str>) -> Result<Option<String>,String> {
    if let Some((lang,_))=text.and_then(crate::reply_language::resolve) {return Ok(Some(lang.into()));}
    let prior:Option<String>=tx.query_row("SELECT value FROM meta WHERE key=?1",[format!("reply-language:telegram:{chat}:{sender}")],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    if let Some(prior)=prior {return serde_json::from_str::<Value>(&prior).map(|v|v["language"].as_str().map(str::to_string)).map_err(|e|e.to_string());}
    // Upgrade fallback: reuse the last durable own resolution, never another actor/chat.
    let prior:Option<Option<String>>=tx.query_row("SELECT language FROM requests WHERE front='telegram' AND chat=?1 AND sender=?2 AND kind IN ('text','image','document','voice','answer','control','builtin','choice') AND language IN ('ru','en') ORDER BY id DESC LIMIT 1",params![chat,sender],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    Ok(prior.flatten().or_else(||label.filter(|x|x.starts_with("ru") || x.starts_with("en")).map(|x|if x.starts_with("ru"){"ru".into()}else{"en".into()})))
}

/// Language for a newly generated delayed output, scoped to its actor and destination.
/// Existing outbox payloads remain immutable across retries.
pub(crate) fn current_reply_language(c:&Connection,front:&str,chat:i64,sender:i64)->Result<Option<String>,String> {
    if front=="telegram" {return chat_language(c,chat,sender,None,None);}
    let raw:Option<String>=c.query_row("SELECT value FROM meta WHERE key=?1",[format!("reply-language:{front}:{chat}:{sender}")],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    raw.map(|s|serde_json::from_str::<Value>(&s).map(|v|v["language"].as_str().map(str::to_string)).map_err(|e|e.to_string())).transpose().map(Option::flatten)
}

// New subscription observations follow the latest own address; the originating
// request and previously queued payloads retain their admitted language.
fn event_reply_language(c:&Connection,request:i64)->Result<Option<String>,String> {
    let (front,chat,sender,saved):(String,i64,i64,Option<String>)=c.query_row(
        "SELECT front,chat,sender,language FROM requests WHERE id=?1",[request],
        |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|e|e.to_string())?;
    Ok(current_reply_language(c,if front=="service" {"telegram"}else{&front},chat,sender)?.or(saved))
}

pub(crate) fn inherit_reply_language(c:&Connection,id:i64,at:i64)->Result<(),String> {
    let (front,chat,sender):(String,i64,i64)=c.query_row("SELECT front,chat,sender FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
    let route=if front=="service" {"telegram"}else{&front};
    let lang=current_reply_language(c,route,chat,sender)?.unwrap_or_else(||"en".into());
    let key=format!("reply-language:{route}:{chat}:{sender}");
    let basis:Option<String>=c.query_row("SELECT value FROM meta WHERE key=?1",[key],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    c.execute("UPDATE requests SET language=?2 WHERE id=?1",params![id,lang]).map_err(|e|e.to_string())?;
    c.execute("INSERT INTO audit_events(request_id,at,kind,result) VALUES (?1,?2,'reply_language_inherited',?3)",params![id,at,json!({"language":lang,"reason":"scoped_delayed_output","basis":basis.map(|s|serde_json::from_str::<Value>(&s)).transpose().map_err(|e|e.to_string())?,"actor":sender,"chat":chat,"front":route}).to_string()]).map_err(|e|e.to_string())?;
    Ok(())
}

/// Freeze one admitted revision's language and reason atomically with its input.
/// The meta state outlives retained raw input. A late transcript cannot reverse newer admission.
fn record_reply_language(tx:&Connection,id:i64,text:Option<&str>,label:Option<&str>,at:i64,transcript:bool)->Result<(),String> {
    let (front,chat,sender,update):(String,i64,i64,i64)=tx.query_row("SELECT front,chat,sender,update_id FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|e|e.to_string())?;
    let key=format!("reply-language:{front}:{chat}:{sender}");
    let prior:Option<String>=tx.query_row("SELECT value FROM meta WHERE key=?1",[&key],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    let prior:Value=prior.map(|x|serde_json::from_str(&x)).transpose().map_err(|e|e.to_string())?.unwrap_or(Value::Null);
    let old:Option<String>=tx.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='reply_language' ORDER BY id DESC LIMIT 1",[id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    let old:Value=old.map(|x|serde_json::from_str(&x)).transpose().map_err(|e|e.to_string())?.unwrap_or(Value::Null);
    let own=text.and_then(crate::reply_language::resolve);
    let inherited=if transcript {old["language"].as_str()} else {prior["language"].as_str()};
    let lang=own.map(|x|x.0.to_string()).or_else(||inherited.map(str::to_string)).or(if front=="telegram"{chat_language(tx,chat,sender,None,label)?}else{None}).unwrap_or_else(||"en".into());
    let reason=own.map(|x|x.1).unwrap_or(if inherited.is_some(){"retained"}else{"initial_default"});
    tx.execute("INSERT INTO audit_events(request_id,at,kind) VALUES (?1,?2,'reply_language')",params![id,at]).map_err(|e|e.to_string())?;
    let event=tx.last_insert_rowid();
    // Audit rowids can be reused after retention pruning; admission order must not.
    let order=if transcript {old["order"].as_i64().unwrap_or(0)} else {
        let next=meta(tx,"reply-language-order")?.unwrap_or(0).checked_add(1).ok_or("reply language order exhausted")?;
        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('reply-language-order',?1)",[next]).map_err(|e|e.to_string())?;
        next
    };
    let revision=old["revision"].as_i64().unwrap_or(0)+1;
    let value=json!({"language":lang,"reason":reason,"order":order,"revision":revision,"origin_update":update,"request":id,"source":if transcript{"own_transcript"}else{"admitted_input"}});
    tx.execute("UPDATE audit_events SET result=?2 WHERE id=?1",params![event,value.to_string()]).map_err(|e|e.to_string())?;
    tx.execute("UPDATE requests SET language=?2 WHERE id=?1",params![id,lang]).map_err(|e|e.to_string())?;
    if order>=prior["order"].as_i64().unwrap_or(0) {
        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)",params![key,value.to_string()]).map_err(|e|e.to_string())?;
    }
    Ok(())
}

/// Short transport outcomes; detailed reasons stay in the audit.
fn brief(language: Option<&str>, reason: &str) -> &'static str {
    match (language.unwrap_or("en").split('-').next().unwrap_or("en"), reason) {
        ("ru", "busy") => "Сейчас очередь заполнена; отправьте запрос позже.",
        ("ru", "unsupported") => "Отправьте текст, изображение, файл, видео или голосовое сообщение; этот формат пока не поддерживается.",
        ("ru", "image_limit") => "Изображение слишком большое; отправьте файл до 20 МиБ.",
        ("ru", "file_limit") => "Файл больше 20 МиБ: Telegram не отдаёт боту такие файлы. Сожмите его или пришлите частями.",
        ("ru", "image_failed") => "Не удалось прочитать изображение; отправьте его ещё раз.",
        ("ru", "voice_limit") => "Голосовое сообщение слишком большое; отправьте запись до 5 минут и 20 МиБ.",
        ("ru", "voice_failed") => "Не удалось распознать голосовое сообщение; отправьте его ещё раз или текстом.",
        ("ru", "document_timeout") => "Время разбора документа истекло.",
        ("ru", "limit") => "Лимит запроса исчерпан; попросите продолжить с сохранённого результата.",
        ("ru", "uncertain") => "Результат действия неизвестен; автоматического повтора не будет.",
        ("ru", "handshake") => "Не удалось открыть разговор, поэтому запрос не отправлен; разговор сохранён. Отправьте запрос ещё раз; если ошибка повторится, попросите владельца проверить журнал сервиса PilotQ.",
        ("ru", _) => "Не удалось завершить запрос; попробуйте ещё раз или проверьте /status.",
        ("he", "file_limit") => "הקובץ גדול מ־20 MiB; דחסו אותו או שלחו אותו בחלקים.",
        ("he", "voice_limit") => "ההקלטה גדולה מדי; שלחו הקלטה באורך של עד 5 דקות ובגודל של עד 20 MiB.",
        ("he", "voice_failed") => "לא ניתן היה לתמלל את ההקלטה; שלחו אותה שוב או כתבו את הדברים.",
        (_, "busy") => "PilotQ is busy; send it again later.",
        (_, "unsupported") => "Send text, an image, a file, a video or a voice message; this format is not supported yet.",
        (_, "image_limit") => "The image is too large; send a file up to 20 MiB.",
        (_, "file_limit") => "The file is larger than 20 MiB, which Telegram does not hand to bots; compress it or send it in parts.",
        (_, "image_failed") => "The image could not be read; send it again.",
        (_, "voice_limit") => "The voice message is too large; send a recording up to 5 minutes and 20 MiB.",
        (_, "voice_failed") => "The voice message could not be recognized; send it again or as text.",
        (_, "document_timeout") => "Document parsing timed out.",
        (_, "limit") => "The request reached its limit; ask to continue from the saved result.",
        (_, "uncertain") => "The action’s outcome is uncertain; it will not be repeated automatically.",
        (_, "handshake") => "The conversation could not be opened, so this request was not submitted; the conversation is kept. Send the request again; if this repeats, ask the owner to check the PilotQ service log.",
        _ => "The request failed; try again or check /status.",
    }
}

/// §1 / #234: use retained attachment identity at admission and after a late download/decoder refusal.
fn media_brief(language: Option<&str>, reason: &str, files: &[Value]) -> String {
    if language.unwrap_or("en").split('-').next() == Some("he") && reason == "file_limit" {
        return brief(language, reason).into();
    }
    let ru=language.unwrap_or("en").starts_with("ru");
    let file=files.first();
    let media=file.and_then(|f|f["media"].as_str()).unwrap_or("");
    let name=file.and_then(|f|f["name"].as_str()).unwrap_or("").to_ascii_lowercase();
    let archive=name.rsplit('.').next().is_some_and(|e|matches!(e,"zip"|"rar"|"7z"|"gz"|"tgz"|"tar"|"bz2"|"xz"));
    let noun=match (ru,media,archive) {
        (true,"audio",_)=>"аудио", (false,"audio",_)=>"audio",
        (true,"video",_)=>"видео", (false,"video",_)=>"video",
        (true,_,true)=>"архив", (false,_,true)=>"archive",
        (true,_,_)=>"документ", (false,_,_)=>"document",
    };
    match reason {
        "file_limit" => if ru {format!("Файл больше 20 МиБ ({noun}); сожмите его или пришлите частями.")}
            else {format!("The file is larger than 20 MiB ({noun}); compress it or send it in parts.")},
        "voice_limit" | "voice_failed" if media=="audio" => brief(language,reason)
            .replace("Голосовое сообщение", "Аудио").replace("голосовое сообщение", "аудио").replace("voice message", "audio file"),
        "unsupported" if file.is_some() => if ru {format!("Этот формат не поддерживается ({noun}); отправьте PDF, ZIP или текст.")}
            else {format!("This format is not supported ({noun}); send PDF, ZIP or text.")},
        _ => brief(language,reason).into(),
    }
}

/// Admits one request; its row is the audit record (§ 8 Request audit), so who, where, the display name at that time and the
/// grant set used commit with the input.
#[allow(clippy::too_many_arguments)]
pub(crate) fn insert(tx: &Connection, update: i64, front: &str, chat: i64, topic: Option<i64>, sender: i64, name: Option<&str>, message: i64, kind: &str, text: Option<&str>, state: &str, at: i64, session: &str, reply_to: Option<i64>) -> Result<i64, String> {
    let grants = grants_of(tx, sender)?.join(",");
    tx.execute(
        "INSERT INTO requests(update_id, front, chat, topic, sender, name, grants, message, kind, text, state, received_at, session, reply_to) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![update, front, chat, topic, sender, name, grants, message, kind, text, state, at, session, reply_to],
    )
    .map_err(|e| e.to_string())?;
    Ok(tx.last_insert_rowid())
}

/// #264 A3: a board card goes to the route that launched the task (its existing subscription), else the first route bound
/// to the project, else the owner's DM.
pub(crate) fn board_route(tx: &Connection, project: &str, owner: i64) -> Result<(i64, Option<i64>), String> {
    let bound: Option<(i64, i64)> = tx.query_row("SELECT chat,topic FROM route_projects WHERE project=?1 ORDER BY chat,topic LIMIT 1", [project], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(|e| e.to_string())?;
    Ok(bound.map_or((owner, None), |(chat, topic)| (chat, (topic != 0).then_some(topic))))
}

/// #264 A2: subscribe an unlaunched task once, in the observer's transaction; the subscription scan then renders the existing
/// status, question and closing cards with no model turn. Returns whether a subscription was created.
pub(crate) fn board_card(tx: &Connection, project: &str, board: &str, root: &str, task: i64, at: i64) -> Result<bool, String> {
    let e = |e: rusqlite::Error| e.to_string();
    let known: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE project=?1 AND number=?2)", params![project, task], |r| r.get(0)).map_err(e)?;
    let Some(owner) = meta(tx, "owner")?.filter(|_| !known) else { return Ok(false) };
    let (chat, topic) = board_route(tx, project, owner)?;
    let shared: bool = tx.query_row("SELECT coalesce((SELECT shared FROM routes WHERE chat=?1 AND topic=?2),0)", params![chat, topic.unwrap_or(0)], |r| r.get(0)).map_err(e)?;
    let update: i64 = tx.query_row("SELECT min(coalesce(min(update_id),0),0)-1 FROM requests", [], |r| r.get(0)).map_err(e)?;
    let id = insert(tx, update, "telegram", chat, topic, owner, None, 0, "board_event", Some(&format!("{project}#{task}")), "answered", at, &session(chat, topic, owner, shared), None)?;
    inherit_reply_language(tx,id,at)?;
    tx.execute("INSERT INTO tasks(number,request_id,chat,topic,state,revision,observed,project,board,root) VALUES (?1,?2,?3,?4,'',-1,?5,?6,?7,?8)",
        params![task, id, chat, topic, at, project, board, root]).map_err(e)?;
    tx.execute("INSERT INTO audit_events(request_id,at,kind,target) VALUES (?1,?2,'board_event',?3)", params![id, at, format!("{project}#{task}")]).map_err(e)?;
    Ok(true)
}

/// #264 A5: an Explain press on a delivered board card in this route returns that card's latest text.
fn explain_card(tx: &Connection, i: &Input) -> Result<Option<String>, String> {
    let Some(id) = i.text.as_deref().and_then(|t| t.strip_prefix("explain:")).and_then(|t| t.parse::<i64>().ok()) else { return Ok(None) };
    tx.query_row("SELECT (SELECT s.text FROM outbox s WHERE s.request_id=o.request_id AND s.kind='status' AND s.state='delivered' ORDER BY s.id DESC LIMIT 1)
        FROM outbox o JOIN requests q ON q.id=o.request_id WHERE o.id=?1 AND o.kind='status' AND q.kind='board_event' AND o.chat=?2 AND o.topic IS ?3
        AND EXISTS(SELECT 1 FROM outbox s WHERE s.request_id=o.request_id AND s.kind='status' AND s.sent_message=?4)",
        params![id, i.chat, i.topic, i.message], |r| r.get(0)).optional().map(Option::flatten).map_err(|e| e.to_string())
}

/// #165: a problem control is a receipt-bound continuation of the original launch.
fn accept_launch_control(tx: &Connection, i: &Input, sender: i64, session: &str, request: i64, delivery: i64, retry: bool) -> Result<(),String> {
    let e=|e:rusqlite::Error|e.to_string();
    let row:Option<(Option<i64>,String,i64)>=tx.query_row("SELECT x.id,p.project,card.id FROM outbox o JOIN cards card ON card.id=o.revision LEFT JOIN plan_runs x ON x.plan_id=card.plan_id JOIN plans p ON p.id=card.plan_id JOIN requests q ON q.id=card.request_id JOIN requests attempt ON attempt.id=o.request_id JOIN conversations c ON c.session=q.session
        WHERE o.id=?1 AND o.kind='plan_problem' AND o.state='delivered' AND o.sent_message=?2 AND o.chat=?3 AND o.topic IS ?4 AND q.session=?5 AND coalesce(x.actor,attempt.sender)=?6 AND card.generation=c.generation
        AND ((x.state='failed') OR (x.id IS NULL AND card.used_request=o.request_id AND p.revision=(SELECT max(revision) FROM plans WHERE session=q.session)))
        AND o.id=(SELECT max(z.id) FROM outbox z WHERE z.revision=o.revision AND z.kind='plan_problem')",
        params![delivery,i.message,i.chat,i.topic,session,sender],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(e)?;
    let (run,project,card)=row.ok_or("stale launch problem")?;
    if retry && !decides(tx,sender,&project)? {return Err(decide_refusal(&project));}
    let at:i64=tx.query_row("SELECT received_at FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
    let Some(run)=run else {
        tx.execute("UPDATE cards SET used_request=?2 WHERE id=?1",params![card,request]).map_err(e)?;
        tx.execute("UPDATE requests SET kind=?2,project=?3,text=?4 WHERE id=?1",params![request,if retry {"launch"} else {"cancel"},project,if retry {"Retry the displayed plan"} else {"Cancel displayed plan"}]).map_err(e)?;
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'decision',?3,?4)",params![request,at,format!("card:{card}"),if retry {"Retry"} else {"Cancel"}]).map_err(e)?;
        return Ok(());
    };
    tx.execute("UPDATE plan_runs SET state=?2,next_at=?3 WHERE id=?1",params![run,if retry {"creating"} else {"cancelled"},at+120]).map_err(e)?;
    tx.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'launch_control',?3,?4,?5)",params![request,at,delivery.to_string(),if retry {"retry"} else {"cancel"},run.to_string()]).map_err(e)?;
    tx.execute("UPDATE requests SET kind='launch_control',project=?2,text=?3 WHERE id=?1",params![request,project,if retry {"Retry the original launch"} else {"Cancel remaining launch steps"}]).map(drop).map_err(e)
}

/// Accept only a button actually delivered to this person/route/generation. Callback text is untrusted.
fn accept_card(tx: &Connection, i: &Input, sender: i64, session: &str, request: i64) -> Result<(),String> {
    let e=|e:rusqlite::Error|e.to_string();
    let data=i.text.as_deref().unwrap_or("");
    let parts:Vec<_>=data.split(':').collect();
    let id=parts.get(1).and_then(|s|s.parse::<i64>().ok()).ok_or("invalid button")?;
    if matches!(parts.as_slice(),["retry",_]|["cancel_launch",_]) {
        return accept_launch_control(tx,i,sender,session,request,id,parts[0]=="retry");
    }
    let card:Option<(Option<i64>,Option<String>,Option<i64>)>=tx.query_row(
        "SELECT c.plan_id,c.options,c.used_request FROM cards c JOIN requests q ON q.id=c.request_id JOIN conversations v ON v.session=q.session
        WHERE c.id=?1 AND q.state='answered' AND q.session=?2 AND q.sender=?3 AND c.generation=v.generation
        AND EXISTS(SELECT 1 FROM outbox o WHERE o.request_id=q.id AND o.chat=?4 AND o.topic IS ?5 AND o.sent_message=?6 AND o.kind IN ('rich','rich_part'))",
        params![id,session,sender,i.chat,i.topic,i.message],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).optional().map_err(e)?;
    let (plan,options,used)=card.ok_or("foreign or reset button")?;
    let pending: bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM requests WHERE session=?1 AND id<?2 AND state IN ('queued','running') AND kind IN ('text','image','document','voice','choice'))",params![session,request],|x|x.get(0)).map_err(e)?;
    if pending { return Err("discussion is still updating; wait for its current display".into()); }

    if let Some(used)=used {
        let saved: Option<String>=tx.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='decision' ORDER BY id LIMIT 1",[used],|x|x.get(0)).optional().map_err(e)?;
        return Err(if parts[0]=="choice" {
            if i.language.as_deref().unwrap_or("en").starts_with("ru") {format!("Уже сохранено: {}",saved.unwrap_or_default())}
            else {format!("Already saved: {}",saved.unwrap_or_default())}
        } else {"button already answered".into()});
    }
    let (kind,text)=match parts.as_slice() {
        ["plan",_] => {
            let plan=plan.ok_or("not a plan button")?;
            let project:Option<String>=tx.query_row("SELECT project FROM plans p WHERE id=?1 AND revision=(SELECT max(revision) FROM plans WHERE session=?2) AND NOT EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=p.id)",params![plan,session],|x|x.get(0)).optional().map_err(e)?;
            let project=project.ok_or("superseded or launched plan")?;
            if !decides(tx,sender,&project)? { return Err(decide_refusal(&project)); }
            ("launch",format!("Run displayed plan ({project})"))
        }
        // #110: cancelling closes the open draft; no board effect, so no TaskQ grant is needed.
        ["cancel",_] => {
            let plan=plan.ok_or("not a plan button")?;
            let open:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM plans p WHERE id=?1 AND revision=(SELECT max(revision) FROM plans WHERE session=?2) AND NOT EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=p.id))",params![plan,session],|x|x.get(0)).map_err(e)?;
            if !open { return Err("superseded or launched plan".into()); }
            ("cancel","Cancel displayed plan".to_string())
        }
        ["choice",_,k] => {
            let current:bool=tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM cards c JOIN requests q ON q.id=c.request_id WHERE q.session=?1 AND c.id>?2)",params![session,id],|x|x.get(0)).map_err(e)?;
            if !current { return Err("superseded question".into()); }
            let k=k.parse::<usize>().map_err(|_|"invalid choice")?;
            let options:Vec<String>=serde_json::from_str(&options.ok_or("no choices")?).map_err(|e|e.to_string())?;
            let option=options.get(k.checked_sub(1).ok_or("invalid choice")?).ok_or("invalid choice")?;
            ("choice",format!("{k}. {option}"))
        }
        _ => return Err("invalid button".into()),
    };
    tx.execute("UPDATE cards SET used_request=?2 WHERE id=?1",params![id,request]).map_err(e)?;
    tx.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) SELECT id,received_at,'decision',?2,text,?3 FROM requests WHERE id=?1",params![request,format!("card:{id}"),text]).map_err(e)?;
    tx.execute("UPDATE requests SET kind=?2,text=CASE WHEN kind='action' THEN ?3 ELSE text END WHERE id=?1",params![request,kind,text]).map_err(e)?;
    if kind=="choice" { choice_receipt(tx,id,request,&text)?; }
    Ok(())
}

/// #108: commit the saved discussion answer and its card edit in the admission transaction.
fn choice_receipt(tx: &Connection, card: i64, request: i64, answer: &str) -> Result<(),String> {
    let (source,chat,topic,message,text):(i64,i64,Option<i64>,i64,String)=tx.query_row(
        "SELECT q.id,o.chat,o.topic,o.sent_message,o.text FROM cards c JOIN requests q ON q.id=c.request_id JOIN outbox o ON o.request_id=q.id
         WHERE c.id=?1 AND o.sent_message IS NOT NULL AND o.kind IN ('rich','rich_part') ORDER BY o.id DESC LIMIT 1",[card],
        |x|Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?,x.get(4)?))).map_err(|e|e.to_string())?;
    let (actor,language):(String,Option<String>)=tx.query_row("SELECT coalesce(name,CAST(sender AS TEXT)),language FROM requests WHERE id=?1",[request],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    let receipt=if language.as_deref().unwrap_or("en").starts_with("ru") {
        format!("Выбрал {actor}: {answer}")
    } else { format!("Chosen by {actor}: {answer}") };
    outbox(tx,source,chat,topic,Some(message),Some(&crate::telegram::html(&format!("{text}\n\n{receipt}"))),&receipt,0)?;
    tx.execute("UPDATE outbox SET kind='choice_answered',revision=?2 WHERE id=?1",params![tx.last_insert_rowid(),card]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result) SELECT id,received_at,'choice_saved',?2,?3 FROM requests WHERE id=?1",params![request,format!("card:{card}"),receipt]).map(drop).map_err(|e|e.to_string())
}

/// Numbered/text option answers share the callback boundary; plain text remains the primary interface (R13).
fn text_choice(tx: &Connection, i: &Input, sender: i64, session: &str) -> Result<Option<Input>,String> {
    if i.reply_to.is_none() {
        let worker_question:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks t JOIN requests q ON q.id=t.request_id WHERE q.session=?1 AND t.state='ask' AND t.closed=0)",[session],|x|x.get(0)).map_err(|e|e.to_string())?;
        if worker_question { return Ok(None); }
    }
    let card:Option<(i64,String,i64)>=tx.query_row("SELECT c.id,c.options,o.sent_message FROM cards c JOIN requests q ON q.id=c.request_id JOIN outbox o ON o.request_id=q.id
        WHERE q.state='answered' AND q.session=?1 AND q.sender=?2 AND c.options IS NOT NULL AND o.chat=?3 AND o.topic IS ?4 AND o.sent_message IS NOT NULL
        AND (?5 IS NULL OR o.sent_message=?5) ORDER BY c.id DESC,o.id DESC LIMIT 1",
        params![session,sender,i.chat,i.topic,i.reply_to],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).optional().map_err(|e|e.to_string())?;
    let Some((card,options,message))=card else {return Ok(None)};
    let text=i.text.as_deref().unwrap_or("").trim();
    let options:Vec<String>=serde_json::from_str(&options).map_err(|e|e.to_string())?;
    let choice=options.iter().enumerate().find(|(n,option)|text==(n+1).to_string() || text==option.as_str()).map(|(n,_)|n+1);
    Ok(choice.map(|choice|Input{files:vec![],callback:None,image:None,voice:None,media_group:None,quote:None,language:i.language.clone(),chat:i.chat,topic:i.topic,sender:Some(sender),name:i.name.clone(),message,kind:"action",text:Some(format!("choice:{card}:{choice}")),reply_to:Some(message)}))
}

fn discard_failed_drafts(tx:&Connection,request:i64,at:i64)->Result<(),String> {
    let e=|e:rusqlite::Error|e.to_string();
    // #212 A3: keep failed proposals in audit, never as an open draft. Authorized runs retain their receipts.
    tx.execute("INSERT INTO audit_events(request_id,run_id,at,kind,result) SELECT ?1,NULL,?2,'failed_draft',tickets FROM plans WHERE request_id=?1 AND NOT EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=plans.id)",params![request,at]).map_err(e)?;
    tx.execute("DELETE FROM cards WHERE request_id=?1 AND plan_id IN (SELECT id FROM plans WHERE request_id=?1 AND NOT EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=plans.id))",[request]).map_err(e)?;
    tx.execute("DELETE FROM plans WHERE request_id=?1 AND NOT EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=plans.id)",[request]).map_err(e)?;
    Ok(())
}

/// Telegram's message limit in UTF-16 code units.
const CHUNK: usize = 4096;
/// Section 7 backlog: pending requests per route and in total; excess gets a recorded busy outcome.
const ROUTE_BACKLOG: i64 = 50;
const TOTAL_BACKLOG: i64 = 1000;

// #222: quote speech literally; captions are deliberately outside this display.
fn voice_quote(transcript: &str, text: &str) -> String {
    format!("> 🎤 {}\n\n{text}", transcript.replace('\n', "\n> "))
}

fn stored_voice(tx: &Connection, request: i64) -> Result<Option<String>, String> {
    tx.query_row("SELECT item FROM audit_events WHERE request_id=?1 AND kind='transcript' ORDER BY id LIMIT 1", [request], |r|r.get(0))
        .optional().map_err(|e|e.to_string())
}

// Select speech only for its original authorized Telegram route, once per delivery obligation.
fn reply_voice(tx: &Connection, request: i64, chat: i64, topic: Option<i64>) -> Result<Option<(String,i64)>,String> {
    tx.query_row("SELECT a.item,q.message FROM requests q JOIN audit_events a ON a.request_id=q.id AND a.kind='transcript'
        WHERE q.id=?1 AND q.front='telegram' AND q.chat=?2 AND q.topic IS ?3
        AND NOT EXISTS(SELECT 1 FROM outbox WHERE request_id=q.id AND text LIKE '> 🎤 %') ORDER BY a.id LIMIT 1",
        params![request,chat,topic],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())
}

/// Splits ordered delivery parts within the UTF-16 cap (§4, L11), preserving quoted speech.
pub(crate) fn chunks(text: &str) -> Vec<String> {
    if let Some((quote, answer)) = text.strip_prefix("> 🎤 ").and_then(|s|s.split_once("\n\n")) {
        let speech=quote.replace("\n> ","\n");
        // Reserve prefix space on every part, including a split through a long spoken line.
        let mut parts=Vec::new();
        let mut part=String::from("> 🎤 ");
        let mut units=part.encode_utf16().count();
        for c in speech.chars() {
            let cost=if c=='\n' {3} else {c.len_utf16()};
            if units+cost>CHUNK {
                parts.push(part);
                part=String::from("> "); units=2;
            }
            part.push(c);
            if c=='\n' {part.push_str("> ");}
            units+=cost;
        }
        let suffix=format!("\n\n{answer}");
        if units+suffix.encode_utf16().count()<=CHUNK {part+=&suffix;parts.push(part);} else {
            parts.push(part);
            if !answer.is_empty() {parts.extend(chunks(answer));}
        }
        return parts;
    }
    let mut out = vec![];
    let mut rest = text;
    while !rest.is_empty() {
        let (mut units, mut end) = (0, rest.len());
        for (i, c) in rest.char_indices() {
            units += c.len_utf16();
            if units > CHUNK {
                end = i;
                break;
            }
        }
        if end < rest.len()
            && let Some(nl) = rest[..end].rfind('\n').filter(|&n| n > end / 2)
        {
            end = nl + 1;
        }
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// Queue complete source content; the shared delivery path persists fallback parts and receipts.
#[allow(clippy::too_many_arguments)]
pub(crate) fn outbox(tx: &Connection, request: i64, chat: i64, topic: Option<i64>, reply_to: Option<i64>, html: Option<&str>, text: &str, at: i64) -> Result<(), String> {
    let speech=reply_voice(tx,request,chat,topic)?;
    let decorated=speech.as_ref().filter(|_| !text.is_empty() && !text.starts_with("> 🎤 ")).map(|(speech,_)|voice_quote(speech,text));
    let reply_to=if decorated.is_some() {speech.map(|(_,message)|message)} else {reply_to};
    let html=if let Some(text)=&decorated {crate::telegram::html(text)} else {html.map(str::to_string).unwrap_or_else(|| crate::telegram::html(text))};
    let text=decorated.as_deref().unwrap_or(text);
    tx.execute("INSERT INTO outbox(request_id, chat, topic, reply_to, html, text, next_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![request,chat,topic,reply_to,html,text,at]).map_err(|e|e.to_string())?;
    tx.execute(STAMP_PROJECT,[tx.last_insert_rowid()]).map(drop).map_err(|e|e.to_string())
}

/// #257: a row without its own task project takes its request's plan, request or conversation project; the column
/// otherwise keeps its legacy 'pilotq' default.
const STAMP_PROJECT: &str = "UPDATE outbox SET project=coalesce(
    (SELECT p.project FROM plans p WHERE p.request_id=outbox.request_id OR p.id IN (SELECT plan_id FROM plan_runs WHERE request_id=outbox.request_id) ORDER BY p.id DESC LIMIT 1),
    (SELECT project FROM requests WHERE id=outbox.request_id),
    (SELECT c.project FROM conversations c JOIN requests q ON q.session=c.session WHERE q.id=outbox.request_id),project) WHERE id=?1";

/// #165: a launch/cancel outcome replaces its plan display, with a durable edit obligation.
fn plan_receipt(tx: &Connection, request: i64, receipt: &str, at: i64) -> Result<bool,String> {
    let source:Option<(i64,i64,i64,Option<i64>,String)>=tx.query_row(
        "SELECT c.id,q.id,q.chat,q.topic,p.tickets FROM cards c JOIN plans p ON p.id=c.plan_id JOIN requests q ON q.id=c.request_id
        WHERE (c.used_request=?1 OR p.id=(SELECT plan_id FROM plan_runs WHERE request_id=?1 LIMIT 1) OR p.id=(SELECT plan_id FROM plan_runs WHERE id=(SELECT CAST(result AS INTEGER) FROM audit_events WHERE request_id=?1 AND kind='launch_control' LIMIT 1))) AND (c.generation=(SELECT generation FROM conversations WHERE session=q.session) OR EXISTS(SELECT 1 FROM plan_runs WHERE plan_id=p.id)) ORDER BY c.id DESC LIMIT 1",
        [request],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(|e|e.to_string())?;
    let Some((card,source,chat,topic,tickets))=source else {return Ok(false)};
    let (front,lang):(String,Option<String>)=tx.query_row("SELECT front,language FROM requests WHERE id=?1",[source],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    if front!="telegram" {return Ok(false)}
    let tickets:Value=serde_json::from_str(&tickets).map_err(|e|e.to_string())?;
    let text=format!("{}\n\n{}",crate::conversation::plan(&tickets,lang.as_deref()),receipt.replace('\n'," "));
    tx.execute("UPDATE outbox SET state='superseded' WHERE revision=?1 AND kind='plan_progress' AND state='pending'",[card]).map_err(|e|e.to_string())?;
    outbox(tx,request,chat,topic,None,Some(&crate::telegram::html(&text)),&text,at)?;
    let failed:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM plan_runs x JOIN plans p ON p.id=x.plan_id JOIN cards c ON c.plan_id=p.id WHERE c.id=?1 AND x.state='failed') OR (EXISTS(SELECT 1 FROM requests WHERE id=?2 AND state='failed') AND NOT EXISTS(SELECT 1 FROM plan_runs x JOIN cards c ON c.plan_id=x.plan_id WHERE c.id=?1))",params![card,request],|r|r.get(0)).map_err(|e|e.to_string())?;
    tx.execute("UPDATE outbox SET kind=?3,revision=?2 WHERE id=?1",params![tx.last_insert_rowid(),card,if failed {"plan_problem"} else {"plan_answered"}]).map_err(|e|e.to_string())?;
    Ok(true)
}

/// Final content and fallback parts commit together (§4), reserving their delivery order.
fn final_outbox(tx: &Connection, request: i64, chat: i64, topic: Option<i64>, reply_to: Option<i64>, text: &str, at: i64) -> Result<(), String> {
    let speech=reply_voice(tx,request,chat,topic)?;
    let decorated=speech.as_ref().map(|(speech,_)|voice_quote(speech,text));
    let text=decorated.as_deref().unwrap_or(text);
    let reply_to=if decorated.is_some() {speech.map(|(_,message)|message)} else {reply_to};
    let rich = text.encode_utf16().count() <= 32768;
    if rich {
        outbox(tx, request, chat, topic, reply_to, None, "", at)?;
        tx.execute("UPDATE outbox SET text=?2,kind='rich' WHERE id=?1", params![tx.last_insert_rowid(), text]).map_err(|e| e.to_string())?;
    }
    for part in chunks(text) {
        outbox(tx, request, chat, topic, reply_to, Some(&crate::telegram::html(&part)), &part, at)?;
        tx.execute("UPDATE outbox SET kind='rich_part',state=?2 WHERE id=?1", params![tx.last_insert_rowid(), if rich {"standby"} else {"pending"}]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Queues one task message (§ 5): a `status` row, edited in place on delivery when the task already has a delivered status
/// message; a `question`; or the `final` outcome. Answer receipts edit their original question (#94).
#[allow(clippy::too_many_arguments)]
fn notice(tx: &Connection, request: i64, chat: i64, topic: Option<i64>, kind: &str, project: &str, task: i64, revision: i64, text: &str, at: i64) -> Result<i64, String> {
    let at=if kind=="status" {
        let last:Option<i64>=tx.query_row("SELECT max(next_at) FROM outbox WHERE request_id=?1 AND kind='status' AND state='delivered'",[request],|r|r.get(0)).map_err(|e|e.to_string())?;
        at.max(last.map(|at|at.saturating_add(2)).unwrap_or(at))
    } else {at};
    tx.execute(
        "INSERT INTO outbox(request_id, chat, topic, text, next_at, kind, task, revision, project, html) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![request, chat, topic, text, at, kind, task, revision, project, crate::telegram::html(text)],
    )
    .map_err(|e| e.to_string())?;
    let delivery=tx.last_insert_rowid();
    if project.is_empty() { tx.execute(STAMP_PROJECT,[delivery]).map_err(|e|e.to_string())?; }
    let language=event_reply_language(tx,request)?;
    // Freeze the event's language with the payload so retries/buttons cannot
    // import a later address. Existing audit retains actor/chat and input order.
    tx.execute("INSERT INTO audit_events(request_id,at,kind,target,result,item)
        SELECT ?1,?2,'delivery_reply_language',?3,?4,(SELECT CAST(value AS TEXT) FROM meta WHERE key='reply-language:' || CASE WHEN q.front='service' THEN 'telegram' ELSE q.front END || ':' || q.chat || ':' || q.sender) FROM requests q WHERE q.id=?1",
        params![request,at,delivery.to_string(),language]).map_err(|e|e.to_string())?;
    Ok(delivery)
}

/// #110: an earlier draft display, or one whose draft was launched or cancelled, loses its buttons; only the current one stays
/// actionable. The edit target resolves at delivery, after the display itself was sent.
fn close_drafts(tx: &Connection, session: &str, request: i64, at: i64) -> Result<(), String> {
    tx.execute("INSERT INTO outbox(request_id,chat,topic,text,next_at,kind,revision,project)
        SELECT ?2,q.chat,q.topic,'',?3,'draft_closed',c.id,p.project FROM cards c JOIN requests q ON q.id=c.request_id JOIN plans p ON p.id=c.plan_id
        WHERE q.session=?1 AND q.front='telegram' AND (p.revision<(SELECT max(revision) FROM plans WHERE session=?1) OR EXISTS(SELECT 1 FROM plan_runs x JOIN plans y ON y.id=x.plan_id WHERE y.draft=p.draft))
        AND NOT EXISTS(SELECT 1 FROM outbox o WHERE o.kind='draft_closed' AND o.revision=c.id)", params![session, request, at]).map(drop).map_err(|e| e.to_string())
}

/// What a task in `state` waits for, in its status message (L01: waiting has a reason, never a hung warning).
pub fn describe(state: &str) -> &'static str {
    match state {
        "held" => "held: filed, not started",
        "ready" => "ready: waiting for a free worker",
        "waiting" => "waiting for the tasks it depends on",
        "doing" => "doing: a worker is on it",
        "ask" => "ask: waiting for an answer to its question",
        "review" => "review: the worker handed in a result; a reviewer checks it",
        "later" => "parked (later)",
        "failed" => "failed: the attempt ended without a result",
        _ => "in a state PilotQ does not know",
    }
}

/// A person's reply to a relayed TaskQ question, waiting to be checked and written (§ 5).
#[derive(Debug)]
pub struct Answer {
    pub project: String,
    pub request: i64,
    pub sender: i64,
    pub text: String,
    pub task: i64,
    pub revision: i64,
}

/// One planned ticket of an authorized plan run and what the board holds for it so far.
#[derive(Debug, Clone)]
pub struct Ticket {
    pub idx: i64,
    pub op_id: String,
    pub task: Option<i64>,
    pub started: bool,
    pub plan: Value,
}

/// An authorized plan run: who authorized it, from which request and route, and its tickets in order.
#[derive(Debug)]
pub struct PlanRun {
    pub project: String,
    pub board: String,
    pub root: String,
    pub id: i64,
    pub revision: i64,
    pub request: i64,
    pub actor: i64,
    pub chat: i64,
    pub topic: Option<i64>,
    pub tickets: Vec<Ticket>,
}

/// #338 / §§4,5: local saved receipts never dispatch TaskQ. Exact source and registration
/// provenance plus the presser's current grants are required before bypassing the worker.
fn saved_answers(c: &Connection, at: i64) -> Result<Vec<i64>,String> {
    let e=|e:rusqlite::Error|e.to_string();
    let mut q=c.prepare("SELECT r.id,r.sender,o.project,a.text,o.task,o.revision FROM requests r
        JOIN outbox o ON o.kind='question' AND o.state='delivered' AND o.sent_message=r.reply_to AND o.chat=r.chat AND o.topic IS r.topic
        JOIN requests source ON source.id=o.request_id AND source.chat=o.chat AND source.topic IS o.topic AND source.front='telegram'
        JOIN outbox a ON a.kind='question_answered' AND a.request_id=o.request_id AND a.project=o.project AND a.task=o.task AND a.revision=o.revision AND a.chat=o.chat AND a.topic IS o.topic
        JOIN tasks t ON t.project=o.project AND t.number=o.task AND t.request_id=o.request_id AND t.chat=o.chat AND t.topic IS o.topic
        JOIN projects p ON p.name=t.project AND p.repo=t.board AND p.root=t.root
        WHERE t.board<>'' AND t.root<>'' AND r.kind='answer' AND r.state='queued' AND r.front='telegram'
        AND EXISTS(SELECT 1 FROM meta WHERE key='callback:' || r.id)
        AND NOT EXISTS(SELECT 1 FROM task_effects WHERE request_id=r.id)
        AND NOT EXISTS(SELECT 1 FROM outbox n WHERE n.kind='question' AND n.project=o.project AND n.task=o.task AND n.revision>o.revision)
        ORDER BY r.id LIMIT 64").map_err(e)?;
    let rows:Vec<(i64,i64,String,String,i64,i64)>=q.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).and_then(Iterator::collect).map_err(e)?;
    let mut completed=vec![];
    for (id,sender,project,receipt,task,revision) in rows {
        if !has(c,sender,"chat")? || !decides(c,sender,&project)? {continue;}
        c.execute("UPDATE requests SET state='answered',reply=?2,project=?3 WHERE id=?1",params![id,receipt,project]).map_err(e)?;
        c.execute("INSERT INTO audit_events(request_id,at,kind,target,result) VALUES (?1,?2,'answer_receipt',?3,?4)",params![id,at,format!("{project}#{task} revision {revision}"),receipt]).map_err(e)?;
        let (chat,topic,callback):(i64,Option<i64>,String)=c.query_row("SELECT r.chat,r.topic,CAST(m.value AS TEXT) FROM requests r JOIN meta m ON m.key='callback:' || r.id WHERE r.id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(e)?;
        outbox(c,id,chat,topic,None,None,&receipt,at)?;
        c.execute("UPDATE outbox SET kind=?2,project=?3 WHERE id=?1",params![c.last_insert_rowid(),format!("toast:{callback}"),project]).map_err(e)?;
        answer_confirmations(c,at,Some(id))?;
        completed.push(id);
    }
    Ok(completed)
}

/// Terminal callback receipts survive popup expiry and restart. The queue message is edited,
/// never left claiming pending work; an unsent notice is superseded by the normal due scan.
fn answer_confirmations(c: &Connection, at: i64, request: Option<i64>) -> Result<(),String> {
    let e=|e:rusqlite::Error|e.to_string();
    // Revalidate delayed output, including registration changes and project-right revocation.
    let mut q=c.prepare("SELECT a.id,r.sender,a.project,EXISTS(SELECT 1 FROM tasks t JOIN projects p
        ON p.name=t.project AND p.repo=t.board AND p.root=t.root
        JOIN outbox o ON o.request_id=t.request_id AND o.project=t.project AND o.task=t.number AND o.kind='question'
        AND o.chat=t.chat AND o.topic IS t.topic
        JOIN requests source ON source.id=o.request_id AND source.chat=o.chat AND source.topic IS o.topic AND source.front='telegram'
        WHERE t.project=a.project AND o.chat=a.chat AND o.topic IS a.topic AND o.sent_message=r.reply_to)
        FROM outbox a JOIN requests r ON r.id=a.request_id WHERE (a.kind='answer_confirmation' OR (a.kind LIKE 'toast:%' AND r.kind='answer' AND a.project IS NOT NULL)) AND a.state='pending'").map_err(e)?;
    let rows:Vec<(i64,i64,String,bool)>=q.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).and_then(Iterator::collect).map_err(e)?;
    for (id,sender,project,provenance) in rows {
        if !provenance || !has(c,sender,"chat")? || !decides(c,sender,&project)? {
            c.execute("UPDATE outbox SET state='blocked',error='answer confirmation provenance or access unavailable' WHERE id=?1",[id]).map_err(e)?;
        }
    }
    c.execute("INSERT INTO outbox(request_id,chat,topic,reply_to,text,next_at,kind,project)
        SELECT r.id,r.chat,r.topic,CASE WHEN r.chat<>r.sender THEN r.message END,r.reply,?1,'answer_confirmation',o.project
        FROM requests r JOIN outbox o ON o.kind='question' AND o.chat=r.chat AND o.topic IS r.topic AND o.sent_message=r.reply_to
        JOIN requests source ON source.id=o.request_id AND source.chat=o.chat AND source.topic IS o.topic AND source.front='telegram'
        JOIN tasks t ON t.request_id=o.request_id AND t.project=o.project AND t.number=o.task AND t.chat=o.chat AND t.topic IS o.topic
        JOIN projects p ON p.name=t.project AND p.repo=t.board AND p.root=t.root
        WHERE r.kind='answer' AND r.state IN ('answered','failed','uncertain') AND r.reply IS NOT NULL
        AND EXISTS(SELECT 1 FROM meta WHERE key='callback:' || r.id)
        AND (r.id=?2 OR EXISTS(SELECT 1 FROM outbox old WHERE old.request_id=r.id AND
            ((old.kind='queued' AND old.state='delivered') OR (old.kind LIKE 'toast:%' AND old.state IN ('failed','exhausted')))))
        AND EXISTS(SELECT 1 FROM grants g WHERE g.user=r.sender AND g.name='chat')
        AND (EXISTS(SELECT 1 FROM grants g WHERE g.user=r.sender AND g.name IN ('admin','owner:' || o.project))
            OR (EXISTS(SELECT 1 FROM grants g WHERE g.user=r.sender AND g.name='taskq')
                AND EXISTS(SELECT 1 FROM grants g WHERE g.user=r.sender AND g.name IN ('projects','project:' || o.project))))
        AND NOT EXISTS(SELECT 1 FROM outbox a WHERE a.request_id=r.id AND a.kind='answer_confirmation')
        ORDER BY r.id LIMIT 64",params![at,request]).map(drop).map_err(e)
}

/// Revalidates `chat` at dispatch and before delivery (§ 8, F10, F17): queued Telegram work of a person who lost it is cancelled
/// without a reply, and their pending deliveries are blocked. Completed records stay. Returns the cancelled requests.
fn revalidate(c: &Connection) -> Result<Vec<i64>, String> {
    let e = |e: rusqlite::Error| e.to_string();
    let granted = "EXISTS (SELECT 1 FROM grants g WHERE g.user=requests.sender AND g.name='chat')";
    let mut q = c.prepare(&format!("UPDATE requests SET state='cancelled', error='access revoked before dispatch' WHERE state='queued' AND front='telegram' AND NOT {granted} RETURNING id")).map_err(e)?;
    let ids: Vec<i64> = q.query_map([], |r| r.get(0)).and_then(Iterator::collect).map_err(e)?;
    c.execute(
        &format!("UPDATE outbox SET state='blocked', error='access revoked before delivery' WHERE state IN ('pending','standby') AND request_id IN (SELECT id FROM requests WHERE NOT {granted})"),
        [],
    )
    .map_err(e)?;
    Ok(ids)
}

fn ids(tx: &Connection, session: &str, state: &str) -> Result<Vec<String>, String> {
    let mut q = tx.prepare("SELECT '#' || id FROM requests WHERE session=?1 AND state=?2 ORDER BY id").map_err(|e| e.to_string())?;
    q.query_map(params![session, state], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())
}

/// #106: one durable terminal record; revocation suppresses pending control replies too (R10).
fn finish_builtin(tx:&Connection,id:i64,text:&str,ok:bool,at:i64)->Result<(),String> {
    let e=|e:rusqlite::Error|e.to_string();
    let (sender,front,chat,topic,message):(i64,String,i64,Option<i64>,i64)=tx.query_row("SELECT sender,front,chat,topic,message FROM requests WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(e)?;
    if !has(tx,sender,"chat")? { tx.execute("UPDATE requests SET state='cancelled',error='access revoked' WHERE id=?1",[id]).map_err(e)?; return Ok(()); }
    let language:Option<String>=tx.query_row("SELECT language FROM requests WHERE id=?1",[id],|r|r.get(0)).map_err(e)?;
    let reply=crate::builtins::localized(text,language.as_deref(),!ok);
    let changed=tx.execute("UPDATE requests SET state=?2,reply=?3,error=?4 WHERE id=?1 AND state IN ('queued','waiting')",params![id,if ok {"answered"} else {"failed"},reply,if ok {None} else {Some(text)}]).map_err(e)?;
    if changed>0 && front=="telegram" {
        let input:String=tx.query_row("SELECT coalesce(text,'') FROM requests WHERE id=?1",[id],|r|r.get(0)).map_err(e)?;
        let onboarding=input.split_whitespace().next().is_some_and(|c|matches!(c.split('@').next(),Some("/onboard"|"/onboard-button"|"/release"|"/release-confirm")));
        let callback:Option<Vec<u8>>=tx.query_row("SELECT value FROM meta WHERE key=?1",[format!("callback:{id}")],|r|r.get(0)).optional().map_err(e)?;
        if onboarding && callback.is_some() {
            let callback=String::from_utf8(callback.unwrap()).map_err(|_|"Invalid callback identity")?;
            let toast=crate::builtins::localized(if ok && input.starts_with("/release") {"Release request processed."} else if ok {"Setup updated."} else {&reply},language.as_deref(),!ok);
            let acknowledged:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM outbox WHERE request_id=?1 AND kind LIKE 'toast:%')",[id],|r|r.get(0)).map_err(e)?;
            if !acknowledged {
                outbox(tx,id,chat,topic,None,None,&toast,at)?;
                tx.execute("UPDATE outbox SET kind=?2 WHERE id=?1",params![tx.last_insert_rowid(),format!("toast:{callback}")]).map_err(e)?;
            }
        } else if !onboarding || !ok || input.ends_with(" override") || input.split_whitespace().next().and_then(|c|c.split('@').next())==Some("/onboard") {
            // #256: a typed /onboard always answers at its own route; the owner DM card is edited separately.
            outbox(tx,id,chat,topic,(chat!=sender).then_some(message),None,&reply,at)?;
        }
    }
    Ok(())
}

/// R10/R13, §6: own DM self-service shares atomic admission and never changes grants.
fn profile_control(tx:&Connection, request:i64, chat:i64, sender:i64, cmd:&str, at:i64)->Result<String,String> {
    let e=|e:rusqlite::Error|e.to_string();
    let ru:bool=tx.query_row("SELECT coalesce(language,'en') LIKE 'ru%' FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
    let topic:Option<i64>=tx.query_row("SELECT topic FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
    if chat!=sender || topic.is_some() { return Ok("Use this control in a direct message with PilotQ; it shows only your own data.".into()); }
    let text:String=tx.query_row("SELECT text FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
    let mut words=text.splitn(2,char::is_whitespace);
    words.next();
    let args=words.next().unwrap_or("").trim();
    let usage="Usage: /profile | /profile set KEY VALUE | /profile forget KEY; /access takes no arguments.";
    if cmd=="/access" {
        if !args.is_empty() {return Ok(usage.into());}
        return Ok(if ru {format!("Ваши текущие права: {}. Для изменения или отзыва обратитесь к администратору. Профиль не меняет права.",grants_of(tx,sender)?.join(", "))} else {format!("Your current grants: {}. Ask an administrator to change or revoke access. Profiles cannot change grants.",grants_of(tx,sender)?.join(", "))});
    }
    if args.is_empty() {
        let mut q=tx.prepare("SELECT key,value,private,source,revision FROM facts WHERE subject=?1 ORDER BY key").map_err(e)?;
        let rows:Vec<String>=q.query_map([sender],|r|Ok(format!("{} = {} ({}; {}; {} {})",r.get::<_,String>(0)?,r.get::<_,String>(1)?,if r.get::<_,bool>(2)? {if ru {"личное"} else {"private"}} else {if ru {"общее"} else {"shared"}},r.get::<_,String>(3)?,if ru {"редакция"}else{"revision"},r.get::<_,i64>(4)?))).and_then(Iterator::collect).map_err(e)?;
        return Ok(format!("{}\n{}",if ru {"Ваши сохранённые сведения:"} else {"Your current remembered facts:"},if rows.is_empty() {if ru {"нет"}else{"none"}.into()} else {rows.join("\n")}));
    }
    let mut parts=args.splitn(3,char::is_whitespace);
    let verb=parts.next().unwrap_or(""); let key=parts.next().unwrap_or(""); let value=parts.next().unwrap_or("").trim();
    if key.is_empty() || key.len()>64 || !key.bytes().all(|c|c.is_ascii_alphanumeric() || b"_.-".contains(&c)) {return Ok(usage.into());}
    match verb {
        "set" if !value.is_empty() && value.len()<=1024 => {
            let revision:i64=tx.query_row("INSERT INTO facts(subject,key,value,private,source,revision,at) VALUES (?1,?2,?3,1,?4,1,?5) ON CONFLICT(subject,key) DO UPDATE SET value=excluded.value,private=1,source=excluded.source,revision=revision+1,at=excluded.at RETURNING revision",params![sender,key,value,format!("user:{sender}"),at],|r|r.get(0)).map_err(e)?;
            Ok(if ru {format!("Сохранено {key} = {value} (личное; редакция {revision}). Прежние аудит и история сохранены.")} else {format!("Remembered {key} = {value} (private; revision {revision}). Previous audit/history remain.")})
        },
        "forget" if value.is_empty()=> {
            let removed=tx.execute("DELETE FROM facts WHERE subject=?1 AND key=?2",params![sender,key]).map_err(e)?;
            Ok(if ru {format!("{key}: {}. Удаление предпочтения не стирает аудит и историю.",if removed>0 {"удалено"} else {"нет текущего значения"})} else {format!("{key}: {}. Preference deletion does not erase audit/history.",if removed>0 {"forgotten"} else {"no current value"})})
        },
        _=>Ok(usage.into())
    }
}

/// Answers `/stop`, `/status`, `/new` or `/audit` inside the admission transaction, without the model and scoped to one
/// conversation (§ 12). Also returns a request that `/stop` cancelled.
fn answer_control(tx: &Connection, session: &str, cmd: &str, request: i64, chat: i64, sender: i64, at: i64) -> Result<(String, Option<i64>), String> {
    let e = |e: rusqlite::Error| e.to_string();
    let ru:bool=tx.query_row("SELECT coalesce(language,'en') LIKE 'ru%' FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
    Ok((
        match cmd {
            "/stop" => {
                let running: Option<i64> = tx.query_row("SELECT id FROM requests WHERE session=?1 AND state='running'", [session], |r| r.get(0)).optional().map_err(e)?;
                let Some(id) = running else { return Ok((if ru {"В этом диалоге ничего не выполняется."}else{"Nothing is running in this conversation."}.into(), None)) };
                // the fence bump is the verified stop: the old run can no longer record a reply or effect
                tx.execute("UPDATE requests SET state='cancelled', error='stopped by /stop' WHERE id=?1", [id]).map_err(e)?;
                tx.execute("UPDATE conversations SET fence=fence+1 WHERE session=?1", [session]).map_err(e)?;
                if stored_voice(tx,id)?.is_some() {
                    let (chat,topic,message)=tx.query_row("SELECT chat,topic,message FROM requests WHERE id=?1",[id],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Option<i64>>(1)?,r.get::<_,i64>(2)?))).map_err(e)?;
                    final_outbox(tx,id,chat,topic,Some(message),if ru {"Остановлено; последующие сообщения остаются в очереди."}else{"Stopped; queued follow-ups stay queued."},at)?;
                    return Ok((String::new(),Some(id)));
                }
                return Ok((if ru {"Остановлено; последующие сообщения остаются в очереди."}else{"Stopped; queued follow-ups stay queued."}.into(), Some(id)));
            }
            "/loop" => {
                let text:String=tx.query_row("SELECT text FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
                match crate::loops::command(tx,request,&text,at) {Ok(s)=>s,Err(s)=>{tx.execute("UPDATE requests SET state='failed',error=?2 WHERE id=?1",params![request,s]).map_err(e)?;s}}
            },
            "/review" => {
                let text:String=tx.query_row("SELECT text FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
                match crate::loops::review_command(tx,request,&text,at) {Ok(s)=>s,Err(s)=>{tx.execute("UPDATE requests SET state='failed',error=?2 WHERE id=?1",params![request,s]).map_err(e)?;s}}
            },
            "/profile" | "/access" => profile_control(tx,request,chat,sender,cmd,at)?,
            "/version" => format!("PilotQ {}", env!("CARGO_PKG_VERSION")),
            "/help" => String::new(),
            "/skills" | "/skill" => "No callable native skills are enabled. Owner-installed PM packs load automatically on bound routes; /help operations shows the binding. /skill NAME TEXT is unavailable.".into(),
            "/unknown" => "Unknown command; use /help.".into(),
            "/restart" if meta(tx,"owner")? != Some(sender) => "Only the configured owner can restart PilotQ.".into(),
            "/restart" => "Restart requested; waiting for active requests to drain.".into(),
            "/doctor" | "/onboard" | "/onboard-button" | "/release" | "/release-confirm" => "Checking setup.".into(),
            "/model" | "/model-button" => "Checking model catalogue.".into(),
            // a group or topic would show one person's history to everyone in it
            "/audit" if chat != sender => if ru {"Используйте /audit в личном чате с PilotQ; команда показывает только ваши запросы."}else{"Use /audit in a direct message with PilotQ; it shows only your own requests."}.into(),
            "/audit" => {
                // Telegram shows a person their own records only, whatever their grants; `pilotq audit` serves the audit grant
                let own = Filter { user: Some(sender), ..Filter::default() };
                let rows: Vec<Value> = records(tx, sender, "telegram", &own, at)?.into_iter().filter(|r| r["request"] != request).collect();
                let mut lines = vec![if ru {"Ваши запросы записываются (метаданные — 365 дней, текст — 90 дней). Последние:".into()}else{format!("Your requests are recorded ({RETENTION}). Latest:")}];
                for r in rows.iter().rev().take(5) {
                    let text: String = r["text"].as_str().unwrap_or("").chars().take(60).collect();
                    lines.push(format!("{}: {text}", crate::conversation::request_state(r["outcome"].as_str().unwrap_or(""),if ru {Some("ru")}else{Some("en")})));
                }
                if rows.is_empty() {
                    lines.push(if ru {"нет"}else{"none"}.into());
                }
                lines.join("\n")
            }
            "/status" => {
                let (running, queued) = (ids(tx, session, "running")?, ids(tx, session, "queued")?);
                let language:Option<String>=tx.query_row("SELECT language FROM requests WHERE id=?1",[request],|r|r.get(0)).map_err(e)?;
                let ru=language.as_deref().unwrap_or("en").starts_with("ru");
                let own = if running.is_empty() && queued.is_empty() {
                    if ru {"В этом диалоге нет активных запросов и очереди."} else {"Nothing is running or queued in this conversation."}.into()
                } else {
                    if ru {format!("В работе: {}. В очереди: {}.",running.len(),queued.len())} else {format!("Running: {}. Queued: {}.", running.len(), queued.len())}
                };
                {
                    let observation:Option<(String,i64)>=tx.query_row("SELECT a.result,a.at FROM audit_events a JOIN requests q ON q.id=a.request_id JOIN runs r ON r.id=a.run_id JOIN conversations c ON c.session=q.session WHERE q.session=?1 AND a.kind='runtime_status' AND r.generation=c.generation AND CAST(a.item AS INTEGER)=coalesce((SELECT value FROM meta WHERE key='model-epoch:' || q.session),0) ORDER BY a.id DESC LIMIT 1",[session],|x|Ok((x.get(0)?,x.get(1)?))).optional().map_err(e)?;
                    let measured=observation.map(|(text,seen)| {
                        let text=if ru {crate::footer::russian_units(&text).replace("requested","запрошена").replace("model unavailable","модель недоступна").replace("context","контекст").replace("quota","квота").replace("unavailable","недоступно").replace("stale","устарело").replace("left","осталось")} else {text};
                        if ru {format!("{text} (наблюдалось {} с назад)",at.saturating_sub(seen).max(0))} else {format!("{text} (observed {} s ago)",at.saturating_sub(seen).max(0))}
                    }).unwrap_or_else(||if ru {"Модель/контекст/квота: нет данных."} else {"Model/context/quota: unavailable."}.into());
                    let model_failed:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM health WHERE source='model' AND state='failed')",[],|x|x.get(0)).map_err(e)?;
                    let next=if model_failed {
                        if ru {"\nДалее: владелец установки проверяет Codex; вы повторяете запрос после восстановления."} else {"\nNext: the installation owner checks Codex; you retry after recovery."}
                    } else if !running.is_empty() || !queued.is_empty() {
                        if ru {"\nДалее: PilotQ завершает активные запросы и обрабатывает очередь."} else {"\nNext: PilotQ finishes active requests and handles the queue."}
                    } else {""};
                    format!("{}\n{own}\n{measured}{next}", service_line(tx, at,language.as_deref())?)
                }
            }
            _ => {
                tx.execute("INSERT INTO conversations(session, generation) VALUES (?1, 1) ON CONFLICT(session) DO UPDATE SET generation=generation+1, thread=NULL, runtime=NULL", [session])
                    .map_err(e)?;
                let loop_stopped=crate::loops::reset(tx,request,at)?;
                let selected: Option<String> = tx.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1", [format!("model:{session}")], |r|r.get(0)).optional().map_err(e)?;
                if ru {format!("Начат новый диалог. Модель: {}.{}",selected.as_deref().unwrap_or("по умолчанию (настройки Codex)"),if loop_stopped {" Цикл остановлен."}else{""})} else {format!("New conversation started. Model: {}.{}", selected.as_deref().unwrap_or("default (Codex configuration)"),if loop_stopped {" Loop stopped."} else {""})}
            }
        },
        None,
    ))
}

/// §9 / #81 S1: the version and each non-ok health source with the age of its last observation; never observed reads unknown.
fn service_line(tx: &Connection, at: i64, language: Option<&str>) -> Result<String, String> {
    let ru=language.unwrap_or("en").starts_with("ru");
    let mut parts = vec![format!("PilotQ {}", env!("CARGO_PKG_VERSION"))];
    for source in SOURCES {
        let row: Option<(String, i64, Option<String>)> =
            tx.query_row("SELECT state, at, reason FROM health WHERE source=?1", [source], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).optional().map_err(|e| e.to_string())?;
        match row {
            Some((state, _, _)) if state == "ok" => {}
            Some((state, seen, reason)) => {
                let name=if ru {match source {"poll"=>"приём","storage"=>"хранилище","model"=>"модель","delivery"=>"доставка",_=>"TaskQ"}} else {source};
                let status=if ru {match state.as_str() {"failed"=>"сбой","conflict"=>"конфликт",_=>"неизвестно"}} else {&state};
                let reason=if source=="model" {format!(": {}",crate::conversation::model_failure(reason.as_deref(),language))} else {String::new()};
                let elapsed=crate::time::moment(Some(seen),at);
                parts.push(if ru {format!("{name}: {status}{reason} · {}",crate::footer::russian_units(&elapsed).replace(" ago", " назад"))} else {format!("{name} {status} {elapsed}{reason}")});
            },
            None => parts.push(if ru {format!("{}: нет данных",match source {"poll"=>"приём","storage"=>"хранилище","model"=>"модель","delivery"=>"доставка",_=>"TaskQ"})} else {format!("{source} unknown")}),
        }
    }
    if parts.len() == 1 {
        parts.push(if ru {"все источники доступны"} else {"all sources ok"}.into());
    }
    Ok(parts.join(" · "))
}

fn meta(tx: &Connection, key: &str) -> Result<Option<i64>, String> {
    tx.query_row("SELECT value FROM meta WHERE key=?1", [key], |r| r.get(0)).optional().map_err(|e| e.to_string())
}

/// §9 / #81: one owner DM through the outbox. Every service message counts toward 10 per day; `key` also caps it at one per
/// 6 hours. Returns false when a cap held it back; the incident then stays in status and the audit only.
fn service_message(tx: &Connection, owner: i64, key: Option<&str>, text: &str, at: i64) -> Result<bool, String> {
    let e = |e: rusqlite::Error| e.to_string();
    let day: i64 = tx.query_row("SELECT count(*) FROM requests WHERE front='service' AND received_at>?1", [at - 86400], |r| r.get(0)).map_err(e)?;
    let last = match key { Some(k) => meta(tx, &format!("sent:{k}"))?, None => None };
    if day >= DAY_CAP || last.is_some_and(|l| at - l < KEY_CAP) {
        return Ok(false);
    }
    let update: i64 = tx.query_row("SELECT min(coalesce(min(update_id), 0), 0) - 1 FROM requests", [], |r| r.get(0)).map_err(e)?;
    let id = insert(tx, update, "service", owner, None, owner, None, 0, "incident", None, "answered", at, &format!("service:{owner}"), None)?;
    inherit_reply_language(tx,id,at)?;
    tx.execute("UPDATE requests SET reply=?2 WHERE id=?1", params![id, text]).map_err(e)?;
    outbox(tx, id, owner, None, None, None, text, at)?;
    if let Some(k) = key {
        tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES (?1,?2)", params![format!("sent:{k}"), at]).map_err(e)?;
    }
    Ok(true)
}

/// One claimed model run. `fence` must still match the conversation's fence for the run to record anything.
#[derive(Debug, Clone)]
pub struct Run {
    pub id: i64,
    pub request: i64,
    pub session: String,
    pub generation: i64,
    pub fence: i64,
    pub thread: Option<String>,
    /// The runtime that owns `thread` (#359 A2).
    pub runtime: Option<String>,
    pub text: String,
    pub model: Option<String>,
    pub model_epoch: i64,
    pub images: Vec<(i64, PathBuf)>,
    pub image_gaps: Vec<i64>,
    /// #186: readable notes of this message's files, a separate model input never checked as the person's own words.
    pub files: Vec<String>,
    pub document_paths: Vec<std::path::PathBuf>,
}

/// Claims one queued request: the request turns running, the conversation's fence advances, and the run row records both.
fn claim(tx: &Connection, request: i64, at: i64) -> Result<Option<Run>, String> {
    let e = |e: rusqlite::Error| e.to_string();
    let row: Option<(String, String)> = tx
        .query_row("UPDATE requests SET state='running' WHERE id=?1 AND state='queued' RETURNING session, coalesce(text, '')", [request], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()
        .map_err(e)?;
    let Some((session, text)) = row else { return Ok(None) };
    tx.execute("INSERT INTO conversations(session) VALUES (?1) ON CONFLICT(session) DO NOTHING", [&session]).map_err(e)?;
    let (generation, thread, runtime, fence) = tx
        .query_row("UPDATE conversations SET fence=fence+1 WHERE session=?1 RETURNING generation, thread, runtime, fence", [&session], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .map_err(e)?;
    tx.execute("INSERT INTO runs(request_id, session, generation, fence, state, started_at) VALUES (?1,?2,?3,?4,'running',?5)", params![request, session, generation, fence, at])
        .map_err(e)?;
    let model = tx.query_row("SELECT CAST(value AS TEXT) FROM meta WHERE key=?1",[format!("model:{session}")],|r|r.get(0)).optional().map_err(e)?;
    let model_epoch=meta(tx,&format!("model-epoch:{session}"))?.unwrap_or(0);
    Ok(Some(Run { id: tx.last_insert_rowid(), request, session, generation, fence, thread, runtime, text, model, model_epoch, images: vec![], image_gaps: vec![], files: vec![], document_paths: vec![] }))
}

/// Marks recognized speech in the request text the model and history see (#104).
pub const VOICE: &str = "[Voice message, automatic speech recognition; may contain recognition errors]";

/// Proposed audit retention, told to each person at their first admitted request (§ 7, § 8; an open decision in § 11).
const RETENTION: &str = "metadata 365 days, message text 90 days";

/// The CoreQ trace ID of a request: the one ID that joins its audit record, telemetry and TaskQ task text (§ 8).
pub fn trace_id(request: i64) -> String {
    format!("pilotq.request.{request}")
}

/// `pilotq audit` filters (§ 9); `route` is `cli`, `CHAT` or `CHAT:TOPIC`.
#[derive(Default, Debug)]
pub struct Filter {
    pub user: Option<i64>,
    pub route: Option<String>,
    pub project: Option<String>,
    pub since: Option<i64>,
    pub request: Option<i64>,
}

/// Audit records `reader` may see under `f`, oldest first, and the read itself recorded (§ 8 Request audit). The owner and
/// holders of `audit` see everyone's; anyone else sees only their own, whatever the filter asks.
fn records(c: &Connection, reader: i64, front: &str, f: &Filter, at: i64) -> Result<Vec<Value>, String> {
    let e = |e: rusqlite::Error| e.to_string();
    let own = (!has(c, reader, "audit")?).then_some(reader);
    let mut q = c
        .prepare(
            "SELECT id, front, chat, topic, sender, name, grants, project, kind, text, state, reply, error, received_at, image FROM requests
            WHERE (?1 IS NULL OR sender=?1) AND (?2 IS NULL OR sender=?2) AND (?3 IS NULL OR ?3=CASE WHEN front='cli' THEN 'cli' ELSE chat || coalesce(':' || topic, '') END)
            AND (?4 IS NULL OR project=?4) AND (?5 IS NULL OR received_at>=?5) AND (?6 IS NULL OR id=?6) ORDER BY id",
        )
        .map_err(e)?;
    let mut rows: Vec<Value> = q
        .query_map(params![own, f.user, f.route, f.project, f.since, f.request], |r| {
            let (id, grants): (i64, Option<String>) = (r.get(0)?, r.get(6)?);
            Ok(json!({
                "request": id, "trace": trace_id(id), "kind": r.get::<_, String>(8)?, "image": r.get::<_,Option<String>>(14)?,
                "who": {"id": r.get::<_, i64>(4)?, "name": r.get::<_, Option<String>>(5)?, "grants": grants.unwrap_or_default().split(',').filter(|g| !g.is_empty()).collect::<Vec<_>>()},
                "where": {"front": r.get::<_, String>(1)?, "chat": r.get::<_, i64>(2)?, "topic": r.get::<_, Option<i64>>(3)?, "project": r.get::<_, Option<String>>(7)?},
                "received": r.get::<_, i64>(13)?, "text": r.get::<_, Option<String>>(9)?,
                "outcome": r.get::<_, String>(10)?, "reply": r.get::<_, Option<String>>(11)?, "error": r.get::<_, Option<String>>(12)?,
            }))
        })
        .and_then(Iterator::collect)
        .map_err(e)?;
    for v in &mut rows {
        let id = v["request"].as_i64().unwrap_or_default();
        let list = |sql: &str, f: &dyn Fn(&rusqlite::Row) -> rusqlite::Result<Value>| -> Result<Vec<Value>, String> {
            let mut q = c.prepare(sql).map_err(e)?;
            q.query_map([id], |r| f(r)).and_then(Iterator::collect).map_err(e)
        };
        v["runs"] = list("SELECT started_at, ended_at, state, model, agentq, outcome FROM runs WHERE request_id=?1 ORDER BY id", &|r| {
            Ok(json!({"started": r.get::<_, i64>(0)?, "ended": r.get::<_, Option<i64>>(1)?, "state": r.get::<_, String>(2)?, "model": r.get::<_, Option<String>>(3)?,
                "agentq": r.get::<_, String>(4)?, "error": r.get::<_, Option<String>>(5)?}))
        })?
        .into();
        let event = |r: &rusqlite::Row| Ok(json!({"at": r.get::<_, i64>(0)?, "target": r.get::<_, Option<String>>(1)?, "item": r.get::<_, Option<String>>(2)?, "result": r.get::<_, Option<String>>(3)?}));
        v["telemetry_links"] = list("SELECT at,NULL,NULL,result FROM audit_events WHERE request_id=?1 AND kind='telemetry_link' ORDER BY id", &event)?.into();
        v["revisions"] = list("SELECT at,NULL,item,NULL FROM audit_events WHERE request_id=?1 AND kind='input_revision' ORDER BY id", &event)?.into();
        v["tools"] = list("SELECT at, target, item, result FROM audit_events WHERE request_id=?1 AND kind='tool' ORDER BY id", &event)?.into();
        v["tasks"] = list("SELECT at, target, item, result FROM audit_events WHERE request_id=?1 AND kind='task' ORDER BY id", &event)?.into();
        v["direct"] = list("SELECT at, target, item, result FROM audit_events WHERE request_id=?1 AND kind='direct' ORDER BY id", &event)?.into();
        v["diagnostics"] = list("SELECT at,target,item,result FROM audit_events WHERE request_id=?1 AND kind IN ('launch_diagnostic','reply_diagnostic','runtime_status','task_result','launch_phase','launch_control','direct_notice') ORDER BY id", &event)?.into();
        v["deliveries"] = list("SELECT state, sent_message, attempts, possible_duplicate, error FROM outbox WHERE request_id=?1 ORDER BY id", &|r| {
            Ok(json!({"state": r.get::<_, String>(0)?, "message": r.get::<_, Option<i64>>(1)?, "attempts": r.get::<_, i64>(2)?, "possible_duplicate": r.get::<_, bool>(3)?,
                "error": r.get::<_, Option<String>>(4)?}))
        })?
        .into();
    }
    let filter = json!({"user": f.user, "route": f.route, "project": f.project, "since": f.since, "request": f.request}).to_string();
    c.execute("INSERT INTO audit_reads(at, reader, front, filter, count) VALUES (?1,?2,?3,?4,?5)", params![at, reader, front, filter, rows.len() as i64]).map_err(e)?;
    Ok(rows)
}

/// One remembered profile fact. A `private` fact is visible only to its subject (in their DM or as the CLI actor).
#[derive(Debug, PartialEq)]
pub struct Fact {
    pub key: String,
    pub value: String,
    pub private: bool,
    pub source: String,
    pub revision: i64,
}

/// One route's session policy; `topic` 0 is a chat without topics.
#[derive(Debug)]
pub struct Route {
    pub chat: i64,
    pub topic: i64,
    pub shared: bool,
    pub by: i64,
    pub at: i64,
}

/// One Telegram message as the ingress sees it. Names are labels, never authority.
pub struct Input {
    /// Telegram photo/document identities and original names; untrusted source material (#145).
    pub files: Vec<Value>,
    pub callback: Option<String>,
    pub media_group: Option<String>,
    /// Deliberately quoted Telegram source, never the quoted author’s authorization.
    pub quote: Option<Value>,
    pub image: Option<String>,
    /// Telegram file identity of a voice note, audio file or audio document (#104).
    pub voice: Option<String>,
    pub language: Option<String>,
    pub chat: i64,
    pub topic: Option<i64>,
    pub sender: Option<i64>,
    /// Display name at that time, kept in the audit record as a label only.
    pub name: Option<String>,
    pub message: i64,
    pub kind: &'static str,
    pub text: Option<String>,
    /// The message this one replies to; a reply to a relayed TaskQ question is an answer (§ 5).
    pub reply_to: Option<i64>,
}

/// §12: holds survive restart and never rewrite a destination or replay work.
fn hold_route(tx: &Connection, chat:i64, topic:Option<i64>, reason:&str, at:i64)->Result<(),String> {
    let source=format!("route-hold:{chat}:{}",topic.unwrap_or(0));
    tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![source,at]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO health(source,at,state,reason) VALUES (?1,?2,'failed',?3) ON CONFLICT(source) DO UPDATE SET state=excluded.state,reason=excluded.reason",params![source,at,reason]).map_err(|e|e.to_string())?;
    Ok(())
}

/// An outbox row due for a send.
#[derive(Debug)]
pub struct Delivery {
    pub id: i64,
    pub chat: i64,
    pub topic: Option<i64>,
    pub reply_to: Option<i64>,
    pub html: Option<String>,
    pub text: String,
    pub attempts: i64,
    /// A task status update edits this delivered status message instead of sending a new one (§ 12, L12).
    pub edit: Option<i64>,
    pub kind: String,
    pub buttons: Vec<(String, String)>,
    pub rich: bool,
}

/// §4: content kinds share rich/HTML/plain delivery; native feedback has no message body.
pub(crate) fn shared_message(kind:&str)->bool {
    !matches!(kind,"reaction"|"draft_closed"|"turn_progress_closed") && !kind.starts_with("toast:")
}

/// What one send attempt established.
#[derive(Debug)]
pub enum Sent {
    Ok(i64),
    /// Telegram rejected the formatting: resend the same content as plain text.
    Plain,
    /// Explicit rejection only: activate the already stored fallback chunks.
    RichFallback,
    /// A failed reaction becomes a text receipt without failing the request.
    Receipt,
    /// Retry after this many seconds; `ambiguous` when Telegram may already show the message.
    Retry { after: i64, ambiguous: bool, error: String },
    /// Destination unusable (forbidden, chat or topic gone): hold for an administrator, never reroute.
    Held(String),
    Failed(String),
}

/// The journal connection, the CoreQ telemetry store path beside it and that store, opened on first use.
pub struct Journal(pub(crate) Connection, PathBuf, std::sync::OnceLock<Result<Store, String>>, std::collections::HashMap<(i64, Option<i64>), std::sync::Weak<tokio::sync::Mutex<()>>>);

impl Journal {
    /// §2 / #229: drafts and stop receipts share a route barrier across network awaits.
    /// Weak entries retain no route history once sends finish; other routes use independent locks.
    pub(crate) fn delivery_barrier(&mut self, chat: i64, topic: Option<i64>) -> std::sync::Arc<tokio::sync::Mutex<()>> {
        self.3.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock)=self.3.get(&(chat,topic)).and_then(std::sync::Weak::upgrade) { return lock; }
        let lock=std::sync::Arc::new(tokio::sync::Mutex::new(()));
        self.3.insert((chat,topic),std::sync::Arc::downgrade(&lock));
        lock
    }


    /// R17, § 6: metadata only, measured after scrubbing; no arguments or result content enter this record.
    pub fn tool_size(&self, r: &Run, tool: &str, bytes: usize, codex: &str, at: i64) -> Result<(), String> {
        self.0.execute("INSERT INTO tool_sizes(request_id, run_id, at, tool, result_bytes, codex_version, pilotq_version) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![r.request, r.id, at, tool, bytes as i64, codex, env!("CARGO_PKG_VERSION")]).map(drop).map_err(|e| e.to_string())
    }

    /// Nearest-rank p95 of UTF-8 result text bytes, in the half-open window [since, until).
    pub fn tool_size_report(&self, since: i64, until: i64) -> Result<Value, String> {
        let mut q = self.0.prepare("SELECT tool, result_bytes FROM tool_sizes WHERE at>=?1 AND at<?2 ORDER BY tool, result_bytes").map_err(|e| e.to_string())?;
        let mut tools = std::collections::BTreeMap::<String, Vec<i64>>::new();
        let rows = q.query_map(params![since, until], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))).map_err(|e| e.to_string())?;
        for row in rows {
            let (tool, bytes) = row.map_err(|e| e.to_string())?;
            tools.entry(tool).or_default().push(bytes);
        }
        let tools: Vec<Value> = tools.into_iter().map(|(tool, bytes)| json!({"tool": tool, "calls": bytes.len(),
            "p95_result_bytes": bytes[(95 * bytes.len()).div_ceil(100) - 1], "max_result_bytes": bytes.last()})).collect();
        Ok(json!({"since": since, "until": until, "tools": tools}))
    }

    /// TaskQ op ID prefix of this installation (§ 5).
    pub fn instance(&self) -> Result<String, String> {
        self.0.query_row("SELECT value FROM meta WHERE key='instance'", [], |r| r.get::<_, i64>(0)).map(|v| format!("{v:x}")).map_err(|e| e.to_string())
    }

    pub fn has(&self, user: i64, grant: &str) -> Result<bool, String> {
        has(&self.0, user, grant)
    }

    pub fn decides(&self, user: i64, project: &str) -> Result<bool, String> {
        decides(&self.0, user, project)
    }

    /// The sender of `request`.
    pub fn sender(&self, request: i64) -> Result<i64, String> {
        self.0.query_row("SELECT sender FROM requests WHERE id=?1", [request], |r| r.get(0)).map_err(|e| e.to_string())
    }


    /// Registers a project root, or moves a registered name to another root; the actor needs `projects` (§ 5 Project registration).
    pub fn add_project(&self, actor: i64, name: &str, repo: &str, root: &str, at: i64) -> Result<(), String> {
        if !valid_grant(&format!("project:{name}")) {
            return Err(format!("{name}: a project name is letters, digits, -, _ or ."));
        }
        if !has(&self.0, actor, "projects")? {
            return Err(format!("{actor} has no projects grant"));
        }
        self.0
            .execute("INSERT OR REPLACE INTO projects(name, repo, root, by, at) VALUES (?1,?2,?3,?4,?5)", params![name, repo, root, actor, at]).map_err(|e|e.to_string())?;
        self.0.execute("DELETE FROM meta WHERE key=?1",[format!("project-disconnected:{name}")]).map(drop).map_err(|e|e.to_string())
    }

    /// Prevent legacy configuration bootstrap from silently reconnecting a deliberately removed project.
    pub fn project_disconnected(&self,name:&str)->Result<bool,String> {
        self.0.query_row("SELECT EXISTS(SELECT 1 FROM meta WHERE key=?1)",[format!("project-disconnected:{name}")],|r|r.get(0)).map_err(|e|e.to_string())
    }

    /// Registered projects by name: (name, repo, root, by, at).
    pub fn projects(&self) -> Result<Vec<(String, String, String, i64, i64)>, String> {
        let mut q = self.0.prepare("SELECT name, repo, root, by, at FROM projects ORDER BY name").map_err(|e| e.to_string())?;
        q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))).and_then(Iterator::collect).map_err(|e| e.to_string())
    }

    /// Records the pilot start once (section 10 step 5); a later start keeps the first. Returns the stored start.
    pub fn pilot_start(&self, at: i64) -> Result<i64, String> {
        self.0.execute("INSERT INTO meta(key, value) VALUES ('pilot_start', ?1) ON CONFLICT(key) DO NOTHING", [at]).map_err(|e| e.to_string())?;
        self.0.query_row("SELECT value FROM meta WHERE key='pilot_start'", [], |r| r.get(0)).map_err(|e| e.to_string())
    }

    /// Records one manual intervention or provider outage seen during the pilot (gate 4).
    pub fn pilot_note(&self, by: i64, kind: &str, text: &str, at: i64) -> Result<(), String> {
        if !["intervention", "outage"].contains(&kind) {
            return Err(format!("{kind}: expected intervention or outage"));
        }
        self.0.execute("INSERT INTO pilot_notes(at, by, kind, text) VALUES (?1,?2,?3,?4)", params![at, by, kind, text]).map(drop).map_err(|e| e.to_string())
    }

    /// Gate 4 counts since `since`, or since the pilot start: accepted and simple requests, users, routes, projects, TaskQ tasks,
    /// interventions and outages, and failed requests read from CoreQ telemetry. A simple request was answered with no plan or task.
    pub fn pilot_report(&self, since: Option<i64>) -> Result<Value, String> {
        let e = |e: rusqlite::Error| e.to_string();
        let start: Option<i64> = self.0.query_row("SELECT value FROM meta WHERE key='pilot_start'", [], |r| r.get(0)).optional().map_err(e)?;
        let Some(since) = since.or(start) else { return Err("no pilot start: run pilotq pilot start or pass --since".into()) };
        let n = |sql: &str| self.0.query_row(sql, [since], |r| r.get::<_, i64>(0)).map_err(e);
        let mine = "SELECT id FROM requests WHERE received_at >= ?1";
        let notes = |kind: &str| n(&format!("SELECT count(*) FROM pilot_notes WHERE kind='{kind}' AND at >= ?1"));
        // unknown is never zero: a telemetry store that cannot be read gives null with its reason
        let failed = (|| -> Result<i64, String> {
            let r = coreq::telemetry::Reader::open(&self.1)?;
            let (mut after, mut count) = (0, 0);
            loop {
                let page = r.export_kinds(&["pilotq.request"], since * 1000, after, 1024)?;
                count += page["events"].as_array().into_iter().flatten().filter(|v| v.to_string().contains("\"outcome\":\"failed\"")).count() as i64;
                after = page["next"].as_i64().unwrap_or(after);
                if page["more"] != true {
                    complete_window(&r, &page, since * 1000)?;
                    return Ok(count);
                }
            }
        })();
        Ok(json!({
            "since": since,
            "pilot_start": start,
            "accepted": n("SELECT count(*) FROM requests WHERE received_at >= ?1")?,
            "simple": n("SELECT count(*) FROM requests WHERE received_at >= ?1 AND state='answered'
                AND id NOT IN (SELECT request_id FROM tasks) AND id NOT IN (SELECT request_id FROM plans) AND id NOT IN (SELECT request_id FROM plan_runs)")?,
            "users": n("SELECT count(DISTINCT sender) FROM requests WHERE received_at >= ?1")?,
            "routes": n("SELECT count(*) FROM (SELECT DISTINCT chat, topic FROM requests WHERE received_at >= ?1 AND front='telegram')")?,
            "projects": n(&format!("SELECT count(*) FROM (SELECT project FROM requests WHERE received_at >= ?1 AND project IS NOT NULL
                UNION SELECT project FROM tasks WHERE request_id IN ({mine}) UNION SELECT project FROM plans WHERE request_id IN ({mine}))"))?,
            "tasks": n(&format!("SELECT count(*) FROM tasks WHERE request_id IN ({mine})"))?,
            "interventions": notes("intervention")?,
            "outages": notes("outage")?,
            "failed": failed.as_ref().ok(),
            "failed_reason": failed.err(),
        }))
    }

    /// The conversation's current project, set only by an explicit switch.
    pub fn current_project(&self, session: &str) -> Result<Option<String>, String> {
        self.0.query_row("SELECT coalesce(project,(SELECT p.project FROM route_projects p JOIN requests q ON q.chat=p.chat AND coalesce(q.topic,0)=p.topic WHERE q.session=?1 ORDER BY q.id DESC LIMIT 1)) FROM conversations WHERE session=?1", [session], |r| r.get(0)).optional().map(Option::flatten).map_err(|e| e.to_string())
    }

    /// §5 / #250: a topic's remembered selection is not an explicit route binding.
    pub(crate) fn unbound_topic(&self, r: &Run) -> Result<bool, String> {
        self.0.query_row("SELECT front='telegram' AND topic IS NOT NULL AND NOT EXISTS
            (SELECT 1 FROM route_projects p WHERE p.chat=q.chat AND p.topic=q.topic)
            FROM requests q WHERE q.id=?1", [r.request], |row| row.get(0)).map_err(|e|e.to_string())
    }

    /// Switches the run's conversation to `project` while its fence holds, with who switched and when (§ 6). `/new` keeps it.
    pub fn use_project(&self, r: &Run, project: &str, actor: i64, at: i64) -> Result<(), String> {
        if self.fenced(r)? {
            return Err("this run was stopped; nothing changed".into());
        }
        self.0
            .execute("UPDATE conversations SET project=?2, project_by=?3, project_at=?4 WHERE session=?1", params![r.session, project, actor, at])
            .map(drop)
            .map_err(|e| e.to_string())
    }

    /// Records a direct read or effect in the request's audit record and names the project there.
    pub fn direct_event(&self, request: i64, project: &str, target: &str, item: Option<&str>, result: &str, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        tx.execute("INSERT INTO audit_events(request_id, at, kind, target, item, result) VALUES (?1,?2,'direct',?3,?4,?5)", params![request, at, target, item, result])
            .map_err(e)?;
        tx.execute("UPDATE requests SET project=?2 WHERE id=?1", params![request, project]).map_err(e)?;
        tx.commit().map_err(e)
    }

    /// Persists a direct effect before its write (§ 5 Effects). `pending` effects go through `taskq issue --op-id` and are
    /// repeated with the same ID (#151); `intent` is the pre-#151 state, never repeated.
    #[allow(clippy::too_many_arguments)]
    pub fn intent(&self, r: &Run, project: &str, tool: &str, repo: &str, issue: i64, value: &str, key: &str, state: &str, at: i64) -> Result<String, String> {
        let inst = self.instance()?;
        if self.fenced(r)? {
            return Err("this run was stopped; nothing was written".into());
        }
        let n: i64 = self.0.query_row("SELECT count(*) + 1 FROM effects WHERE request_id=?1", [r.request], |x| x.get(0)).map_err(|e| e.to_string())?;
        let op = format!("pilotq-{inst}-r{}-e{n}", r.request);
        self.0
            .execute(
                "INSERT INTO effects(request_id, op_id, tool, repo, issue, value, key, state, at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![r.request, op, tool, repo, issue, value, key, state, at],
            )
            .map_err(|e| e.to_string())?;
        self.direct_event(r.request, project, &format!("{tool} {repo}#{issue}"), Some(&op), state, at)?;
        Ok(op)
    }

    /// Settles an effect from its readback: `done` with a receipt, `absent`, `failed` or `uncertain`. The audit record gets the
    /// result; `notice` goes to the request's Telegram route (restart reconciliation).
    pub fn settle(&self, op: &str, state: &str, receipt: Option<&str>, notice: Option<&str>, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        let request: i64 = tx.query_row("UPDATE effects SET state=?2, receipt=?3 WHERE op_id=?1 RETURNING request_id", params![op, state, receipt], |r| r.get(0)).map_err(e)?;
        let result = receipt.map_or(state.to_string(), |r| format!("{state}: {r}"));
        tx.execute("UPDATE audit_events SET result=?2, at=?3 WHERE kind='direct' AND item=?1", params![op, result, at]).map_err(e)?;
        let dest: (String, i64, Option<i64>) = tx.query_row("SELECT front, chat, topic FROM requests WHERE id=?1", [request], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(e)?;
        if let (Some(diagnostic), "telegram") = (notice, dest.0.as_str()) {
            let last:Option<String>=tx.query_row("SELECT result FROM audit_events WHERE request_id=?1 AND kind='direct_notice' AND item=?2 ORDER BY id DESC LIMIT 1",params![request,op],|r|r.get(0)).optional().map_err(e)?;
            if last.as_deref()!=Some(state) {
                let ru=self.language(request)?.as_deref().unwrap_or("en").starts_with("ru");
                let text=match (ru,state) {
                    (true,"done")=>"Ранее запрошенное изменение подтверждено.",(true,"failed")=>"Ранее запрошенное изменение отклонено.",(true,_)=>"Результат изменения не подтверждён; автоматического повтора не будет.",
                    (false,"done")=>"The earlier change is confirmed.",(false,"failed")=>"The earlier change was refused.",(false,_)=>"The earlier change is not confirmed; it will not be repeated automatically.",
                };
                tx.execute("INSERT INTO audit_events(request_id,at,kind,target,item,result) VALUES (?1,?2,'direct_notice',?3,?4,?5)",params![request,at,diagnostic,op,state]).map_err(e)?;
                outbox(&tx, request, dest.1, dest.2, None, None, text, at)?;
            }
        }
        tx.commit().map_err(e)
    }

    /// Effects whose write may have happened without a receipt: (op, tool, repo, issue, value, state, key, request).
    #[allow(clippy::type_complexity)]
    pub fn intents(&self) -> Result<Vec<(String, String, String, i64, String, String, String, i64)>, String> {
        let mut q = self.0.prepare("SELECT op_id, tool, repo, issue, value, state, key, request_id FROM effects WHERE state IN ('intent','pending') ORDER BY id").map_err(|e| e.to_string())?;
        q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))).and_then(Iterator::collect).map_err(|e| e.to_string())
    }

    /// Telegram routes with a running turn started within `window` seconds: they show typing (§ 12, L02).
    pub fn typing(&self, now: i64, window: i64) -> Result<Vec<(i64, Option<i64>)>, String> {
        let mut q = self
            .0
            .prepare("SELECT DISTINCT q.chat, q.topic FROM runs r JOIN requests q ON q.id=r.request_id WHERE r.state='running' AND q.state='running' AND q.front='telegram' AND r.started_at>?1")
            .map_err(|e| e.to_string())?;
        q.query_map([now - window], |r| Ok((r.get(0)?, r.get(1)?))).and_then(Iterator::collect).map_err(|e| e.to_string())
    }

    /// Appends the ended request's CoreQ event, keyed by request ID. Metadata only: no request or reply text, so no key value
    /// can reach telemetry (§ 8). A telemetry failure is reported and never blocks the request.
    pub(crate) fn trace(&self, request: i64, at: i64) {
        let event = || -> Result<Option<Vec<Event>>, String> {
            let (front, state, received, runs, model, runtime): (String, String, i64, i64, Option<String>, Option<String>) = self
                .0
                .query_row(
                    "SELECT front, state, received_at, (SELECT count(*) FROM runs WHERE request_id=?1),
                        (SELECT model FROM runs WHERE request_id=?1 AND model IS NOT NULL ORDER BY id DESC LIMIT 1),
                        (SELECT runtime FROM runs WHERE request_id=?1 AND runtime IS NOT NULL ORDER BY id DESC LIMIT 1)
                        FROM requests WHERE id=?1",
                    [request],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
                )
                .map_err(|e| e.to_string())?;
            // #257: the event ID is immutable; a queued built-in traced now would conflict with its final outcome.
            if matches!(state.as_str(), "queued" | "running" | "waiting") { return Ok(None); }
            let outcome = match state.as_str() {
                "answered" => "ok",
                "failed" => "failed",
                "cancelled" => "cancelled",
                _ => "unknown",
            };
            // ponytail: CoreQ codes allow only [A-Za-z0-9._:-]; a model name outside that is left out, not rewritten
            let model = model.filter(|m| !m.is_empty() && m.len() <= 128 && m.bytes().all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c)));
            let value = json!({"product": "pilotq", "version": env!("CARGO_PKG_VERSION"), "operation": "request", "kind": "pilotq.request",
                "trace": trace_id(request), "span": "request", "started_ms": received.max(0) * 1000, "duration_us": (at - received).max(0) * 1_000_000,
                "outcome": outcome, "error": if outcome == "ok" { "none" } else { state.as_str() }, "source": front, "attempt": runs,
                "runtime": runtime.filter(|_| runs > 0), "model": model});
            let legacy = Event::identified("pilotq.request", &request.to_string(), value.clone())?;
            let mut observation = value;
            observation["kind"] = json!("operation");
            // A request's run count is not a TaskQ attempt identity.
            observation["attempt"] = Value::Null;
            observation["unknown"] = json!({"attempt":"no-task"});
            let instance = meta(&self.0,"instance")?.ok_or("journal instance unavailable")?;
            let observation = Event::identified("pilotq.observation", &format!("{instance}:{request}"), coreq::telemetry::shared(&observation))?;
            Ok(Some(vec![legacy, observation]))
        };
        let r = event().and_then(|ev| match ev {
            Some(ev) => self.2.get_or_init(|| Store::open(&self.1)).as_ref().map_err(String::clone)?.append(&ev).map(drop),
            None => Ok(()),
        });
        if let Err(e) = r {
            eprintln!("telemetry request #{request}: {e}");
        }
    }

    /// Also tracks when a source began failing and how many failures in a row it had, for the incident hold-down (#81).
    pub fn set_health(&self, source: &str, at: i64, state: &str, reason: Option<&str>) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        tx.execute("INSERT OR REPLACE INTO health(source, at, state, reason) VALUES (?1,?2,?3,?4)", params![source, at, state, reason]).map_err(e)?;
        let (since, failures) = (format!("failing:{source}"), format!("failures:{source}"));
        if state == "ok" {
            tx.execute("DELETE FROM meta WHERE key IN (?1, ?2)", [since, failures]).map_err(e)?;
        } else {
            tx.execute("INSERT OR IGNORE INTO meta(key,value) VALUES (?1,?2)", params![since, at]).map_err(e)?;
            tx.execute("INSERT INTO meta(key,value) VALUES (?1,1) ON CONFLICT(key) DO UPDATE SET value=value+1", [failures]).map_err(e)?;
        }
        tx.commit().map_err(e)
    }

    /// Fills status.json health and counts from the journal; sources it has never observed stay unknown.
    pub fn status(&self, s: &mut serde_json::Value, now: i64) -> Result<(), String> {
        let mut q = self.0.prepare("SELECT source, at, state, reason FROM health").map_err(|e| e.to_string())?;
        let rows = q.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?))).map_err(|e| e.to_string())?;
        for row in rows {
            let (source, at, state, reason) = row.map_err(|e| e.to_string())?;
            s["health"][source] = serde_json::json!({"at": at, "age": now - at, "state": state, "reason": reason});
        }
        let count = |sql: &str| self.0.query_row(sql, [], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string());
        s["pending"] = serde_json::json!({"count": count("SELECT count(*) FROM requests WHERE state='queued'")?});
        s["undelivered"] = serde_json::json!({"count": count("SELECT count(*) FROM outbox WHERE state NOT IN ('delivered', 'superseded')")?});
        s["uncertain"] = serde_json::json!({"count": count("SELECT count(*) FROM requests WHERE state='uncertain'")?});
        Ok(())
    }
}
