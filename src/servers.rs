//! Read-only server discussion through DashQ's server projection (pilotq.md §5, #375).
//! Nothing here reaches a server: the only process is the owner-configured local report.
use crate::{builtins::Request, direct::Direct, journal::{Journal, Run}};
use serde_json::{Value, json};
use std::{path::Path, sync::Mutex, time::{Duration, Instant}};

pub const PREFIX: &str = "server-";
const SOURCE_CAP: usize = 16 << 20;
const OUTPUT_CAP: usize = 128 << 10;
const REFUSAL: &str = "No access to that server, or it is not in the inventory. Ask an administrator for its server grant.";

// Fixed diagnostics only: neither host paths nor source errors belong in a reply (R16, §6).
fn error_text(error: &str, ru: bool) -> String {
    let (en, translated) = match error {
        REFUSAL => (REFUSAL, "Нет доступа к этому серверу, либо его нет в инвентаре. Попросите администратора выдать право на сервер."),
        "server inventory is not configured" => (error, "Инвентарь серверов не настроен."),
        "command configuration unavailable" | "invalid command configuration" | "server inventory executable must be absolute" | "server inventory args must be a string array" | "server inventory directory unavailable" => ("Server inventory configuration is invalid or unavailable.", "Настройки инвентаря серверов недоступны или неверны."),
        "servers.json must be owner-only (chmod 600)" => (error, "Файл настроек инвентаря должен быть доступен только владельцу (chmod 600)."),
        "server inventory unavailable (this DashQ report has no server table)" => (error, "Инвентарь недоступен: в отчёте DashQ нет таблицы серверов."),
        "server inventory unavailable (invalid report)" => (error, "Инвентарь недоступен: неверный отчёт DashQ."),
        "inventory too large; name one server" => (error, "Инвентарь слишком большой; укажите один сервер."),
        "server name is ambiguous; specify workspace and full host name" => (error, "Имя подходит нескольким серверам; укажите рабочую область и полное имя сервера."),
        "server inventory unavailable (timeout)" => (error, "Инвентарь недоступен: время ожидания истекло."),
        "server inventory unavailable (output_limit)" => (error, "Инвентарь недоступен: отчёт превышает допустимый размер."),
        _ => ("Server inventory unavailable; the read did not complete.", "Инвентарь серверов недоступен; чтение не завершено."),
    };
    if ru { translated } else { en }.into()
}

pub fn tool() -> Value {
    json!({"type":"function","name":"server_inventory","description":crate::catalog::purpose("server_inventory"),"inputSchema":{"type":"object","properties":{"server":{"type":"string"}},"additionalProperties":false}})
}

pub fn any_access(j: &Journal, user: i64) -> Result<bool, String> {
    Ok(j.has(user, "chat")? && (j.has(user, "servers")? || j.grants(user)?.iter().any(|g| g.starts_with("server:"))))
}

fn permitted(j: &Journal, user: i64, row: &Value) -> Result<bool, String> {
    Ok(j.has(user, "chat")? && (j.has(user, "servers")? || j.has(user, &grant(row))?))
}

fn grant(row: &Value) -> String {
    format!("server:{}/{}", row["workspace"].as_str().unwrap_or("-"), row["host"].as_str().unwrap_or("")).to_lowercase()
}

fn bind(j: &Journal, request: i64, rows: &[&Value], now: i64) -> Result<(), String> {
    let tx = j.0.unchecked_transaction().map_err(|e| e.to_string())?;
    for row in rows {
        tx.execute("INSERT INTO audit_events(request_id,at,kind,target) SELECT ?1,?2,'server_read',?3 WHERE NOT EXISTS(SELECT 1 FROM audit_events WHERE request_id=?1 AND kind='server_read' AND target=?3)", rusqlite::params![request,now,grant(row)]).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

/// Telegram limits `/start` payloads to 64 URL-safe characters, so the DashQ selector travels as a fixed-length digest.
/// ponytail: FNV-1a selects among rows the person may already read, so it needs no secrecy; a collision is refused as ambiguous.
pub fn link_payload(selector: &str) -> String {
    let hash = selector.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    format!("{PREFIX}{hash:016x}")
}

/// Runs the protected local report. Its stderr stays out of the result: it may name host paths or keys.
fn read(keys: &Path, end: Instant, active: impl Fn() -> bool) -> Result<Value, String> {
    let config = crate::commands::load_protected(&keys.with_file_name("servers.json"))?;
    if config.as_object().is_some_and(|o| o.is_empty()) { return Err("server inventory is not configured".into()); }
    let bin = config["executable"].as_str().filter(|p| Path::new(p).is_absolute()).ok_or("server inventory executable must be absolute")?;
    let args: Vec<&str> = config["args"].as_array().ok_or("server inventory args must be a string array")?.iter()
        .map(|a| a.as_str().ok_or("server inventory args must be a string array")).collect::<Result<_, _>>()?;
    let cwd = keys.parent().and_then(Path::to_str).ok_or("server inventory directory unavailable")?;
    let output = crate::commands::execute_bounded_private(crate::commands::command(bin, cwd, &args), end, active, SOURCE_CAP);
    if output["status"] != "success" { return Err(format!("server inventory unavailable ({})", output["status"].as_str().unwrap_or("unknown"))); }
    let report: Value = output["stdout"].as_str().and_then(|s| serde_json::from_str(s).ok()).ok_or("server inventory unavailable (invalid report)")?;
    report.get("servers").filter(|s| s["rows"].is_array()).cloned().ok_or("server inventory unavailable (this DashQ report has no server table)".into())
}

fn age(source: &Value, now: i64) -> Value {
    source["at"].as_i64().filter(|at| *at <= now).map_or(Value::Null, |at| json!(now - at))
}

/// Host and mounts first, then the largest consumers, within a per-server budget; what does not fit is counted.
fn view(row: &Value, now: i64, budget: usize) -> Value {
    let mut entities: Vec<Value> = row["entities"].as_array().cloned().unwrap_or_default();
    let rank = |e: &Value| match e["inventory"]["entity"].as_str() { Some("host") => 0, Some("mount") => 1, _ => 2 };
    entities.sort_by_key(|e| (rank(e), std::cmp::Reverse(e["inventory"]["disk_used_bytes"].as_u64().unwrap_or(0))));
    let (mut kept, mut used, total) = (Vec::new(), 0, entities.len());
    for entity in entities {
        let entity = json!({"inventory":entity["inventory"],"source":entity["source"],"trends":entity["trends"]});
        let size = entity.to_string().len() + 1;
        if used + size > budget { break; }
        used += size;
        kept.push(entity);
    }
    json!({"server":row["host"],"workspace":row["workspace"],"state":row["source"]["state"],"reason":row["source"]["reason"],"observed_at":row["source"]["at"],
        "data_age_seconds":age(&row["source"],now),"omitted_entities":total-kept.len(),"entities":kept})
}

fn named<'a>(rows: &'a [Value], wanted: &str) -> Vec<&'a Value> {
    rows.iter().filter(|r| r["pilotq_selector"] == wanted || r["host"].as_str().is_some_and(|h|
        h.eq_ignore_ascii_case(wanted) || format!("{}/{}",r["workspace"].as_str().unwrap_or("-"),h).eq_ignore_ascii_case(wanted)
        || h.rsplit_once('/').is_some_and(|(n, _)| n.eq_ignore_ascii_case(wanted)))).collect()
}

fn visible(j: &Journal, user: i64, data: &Value) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    for row in data["rows"].as_array().into_iter().flatten() { if permitted(j, user, row)? { rows.push(row.clone()); } }
    Ok(rows)
}

pub fn call(d: &Direct, j: &Mutex<Journal>, r: &Run, args: &Value, deadline: Instant) -> (bool, String) {
    let ru = j.lock().unwrap().language(r.request).ok().flatten().is_some_and(|l| l.starts_with("ru"));
    let result = (|| {
        let user = {
            let j = j.lock().unwrap();
            let user = j.sender(r.request)?;
            if j.fenced(r)? || !any_access(&j, user)? { return Err(REFUSAL.to_string()); }
            user
        };
        let end = deadline.min(Instant::now() + Duration::from_secs(30));
        let data = read(&d.keys.0, end, || { let j = j.lock().unwrap(); !j.fenced(r).unwrap_or(true) && any_access(&j, user).unwrap_or(false) })?;
        let now = crate::now() as i64;
        // Grants are current after the read, so a revocation during it still refuses.
        let j = j.lock().unwrap();
        if j.fenced(r)? || !any_access(&j, user)? { return Err(REFUSAL.to_string()); }
        let rows = visible(&j, user, &data)?;
        let selected = j.selected_server(&r.session)?;
        let wanted = args["server"].as_str().map(str::trim).filter(|s| !s.is_empty());
        let (chosen, scope): (Vec<&Value>, String) = match (wanted, &selected) {
            (Some(w), _) if w.eq_ignore_ascii_case("all") => (rows.iter().collect(), "all".into()),
            (Some(w), _) => (named(&rows, w), w.into()),
            (None, Some(s)) => (named(&rows, s), "selected".into()),
            (None, None) => (rows.iter().collect(), "all".into()),
        };
        if chosen.is_empty() && (scope != "all" || !j.has(user, "servers")?) { return Err(REFUSAL.to_string()); }
        if scope != "all" && chosen.len() > 1 {
            return Err("server name is ambiguous; specify workspace and full host name".into());
        }
        let mut output = json!({"read_at":now,"scope":scope,"source":{"state":data["source"]["state"],"reason":data["source"]["reason"],"observed_at":data["source"]["at"],"data_age_seconds":age(&data["source"],now)},"servers":chosen.iter().map(|row| view(row,now,0)).collect::<Vec<_>>()});
        let base = output.to_string().len();
        if base > OUTPUT_CAP { return Err("inventory too large; name one server".into()); }
        let budget = (OUTPUT_CAP - base) / chosen.len().max(1);
        output["servers"] = json!(chosen.iter().map(|row| view(row,now,budget)).collect::<Vec<_>>());
        let text = output.to_string();
        if text.len() > OUTPUT_CAP { return Err("inventory too large; name one server".into()); }
        bind(&j, r.request, &chosen, now)?;
        j.direct_event(r.request, "", "server_inventory", None, &format!("read {} servers", chosen.len()), now)?;
        Ok(text)
    })();
    match result { Ok(text) => (true, text), Err(error) => (false, error_text(&error,ru)) }
}

fn age_text(seconds: i64, ru: bool) -> String {
    match (seconds / 3600, seconds / 60, ru) {
        (0, m, false) => format!("{m} min"), (0, m, true) => format!("{m} мин"),
        (h, _, false) => format!("{h} h"), (h, _, true) => format!("{h} ч"),
    }
}

/// `/start server-DIGEST` from a DashQ row: the link grants nothing, so it resolves only among rows this person may read.
pub fn select(keys: &Path, j: &Mutex<Journal>, r: &Request) -> Result<String, String> {
    let payload = r.text.split_whitespace().nth(1).unwrap_or("");
    let ru = j.lock().unwrap().language(r.id)?.is_some_and(|l| l.starts_with("ru"));
    let refused = || error_text(REFUSAL,ru);
    if !{ let j = j.lock().unwrap(); any_access(&j, r.sender)? } { return Ok(refused()); }
    let end = Instant::now() + Duration::from_secs(20);
    let data = match read(keys, end, || { let j=j.lock().unwrap(); j.builtin_allowed(r).unwrap_or(false) && any_access(&j,r.sender).unwrap_or(false) }) {
        Ok(data) => data,
        Err(reason) => return Ok(if ru { format!("Сервер не выбран: {}",error_text(&reason,ru)) } else { format!("No server selected: {}",error_text(&reason,ru)) }),
    };
    let j = j.lock().unwrap();
    if !j.builtin_allowed(r)? || !any_access(&j,r.sender)? { return Ok(refused()); }
    let rows = visible(&j, r.sender, &data)?;
    let found: Vec<&Value> = rows.iter().filter(|row| row["pilotq_selector"].as_str().is_some_and(|s| link_payload(s) == payload)).collect();
    let [row] = found[..] else { return Ok(refused()); };
    bind(&j,r.id,&[row],crate::now() as i64)?;
    j.select_server(&r.session, row["pilotq_selector"].as_str().unwrap())?;
    let host = row["host"].as_str().unwrap_or("");
    let at = age(&row["source"], crate::now() as i64).as_i64();
    Ok(match (ru, at) {
        (false, Some(s)) => format!("Server {host} selected; its data is {} old. Ask what takes space, how to organize it or when resources run out. Changes become TaskQ tasks after you confirm.", age_text(s, false)),
        (false, None) => format!("Server {host} selected; its data age is unknown. Ask what takes space, how to organize it or when resources run out. Changes become TaskQ tasks after you confirm."),
        (true, Some(s)) => format!("Выбран сервер {host}; возраст данных {}. Спросите, что занимает место, как его организовать или когда закончатся ресурсы. Изменения станут задачами TaskQ после вашего подтверждения.", age_text(s, true)),
        (true, None) => format!("Выбран сервер {host}; возраст данных неизвестен. Спросите, что занимает место, как его организовать или когда закончатся ресурсы. Изменения станут задачами TaskQ после вашего подтверждения."),
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    const OWN: &str = "[\"example\",\"invented-fresh/native\"]";
    const FOREIGN: &str = "[\"example\",\"invented-foreign/native\"]";

    struct Fixture { dir: std::path::PathBuf, d: Direct, j: Mutex<Journal>, log: std::path::PathBuf }

    fn row(selector: &str, host: &str, at: i64) -> Value {
        json!({"id":selector,"host":host,"workspace":"example","pilotq_selector":selector,"source":{"state":"ok","at":at},"entities":[
            {"inventory":{"entity":"folder","identity":"/srv/invented-small","disk_used_bytes":10},"source":{"state":"ok","at":at},"trends":null},
            {"inventory":{"entity":"folder","identity":"/srv/invented-big","disk_used_bytes":900},"source":{"state":"ok","at":at},"trends":null},
            {"inventory":{"entity":"mount","identity":"/","disk_total_bytes":1000,"disk_used_bytes":950,"disk_free_bytes":50},"source":{"state":"ok","at":at},"trends":{"disk":{"state":"unknown","reason":"short history"}}},
            {"inventory":{"entity":"host","identity":host,"status":"partial","ram_used_bytes":null,"unknown":{"ram_used_bytes":"collection-failed"}},"source":{"state":"incomplete","at":at},"trends":null},
        ]})
    }

    fn fixture(name: &str) -> Fixture {
        let dir = std::env::temp_dir().join(format!("pilotq-servers-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
        let now = crate::now() as i64;
        let report = json!({"schema":1,"servers":{"source":{"state":"incomplete","at":now-600},"rows":[row(OWN,"invented-fresh/native",now-600),row(FOREIGN,"invented-foreign/native",now-600)]}});
        std::fs::write(dir.join("report.json"), report.to_string()).unwrap();
        let log = dir.join("calls.log");
        let script = dir.join("report.sh");
        std::fs::write(&script, format!("#!/bin/sh\necho \"$@\" >> '{}'\necho invented-secret-stderr >&2\ncat '{}'\n", log.display(), dir.join("report.json").display())).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let config = dir.join("servers.json");
        std::fs::write(&config, json!({"executable":script,"args":["report"]}).to_string()).unwrap();
        std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
        let mut j = Journal::open(&dir.join("journal.sqlite")).unwrap();
        j.bootstrap(7, 0).unwrap();
        j.grant(7, 8, &["chat".into(), "server:example/invented-fresh/native".into()], 0).unwrap();
        j.grant(7, 9, &["chat".into()], 0).unwrap();
        Fixture { d: Direct { keys: crate::keys::Keys(dir.join("keys.json")) }, j: Mutex::new(j), log, dir }
    }

    fn ask(f: &Fixture, user: i64, args: Value) -> (bool, String) {
        let r = f.j.lock().unwrap().ask(user, "what takes space", crate::now() as i64).unwrap();
        call(&f.d, &f.j, &r, &args, Instant::now() + Duration::from_secs(10))
    }

    fn start(f: &Fixture, sender: i64, session: &str, payload: &str) -> String {
        let text = format!("/start {payload}");
        let id = f.j.lock().unwrap().ask(sender, &text, crate::now() as i64).unwrap().request;
        select(&f.d.keys.0, &f.j, &Request { id, session: session.into(), sender, text }).unwrap()
    }

    // #375 A2/A3: answers cite observed values with data age; a partial field stays null with its reason, and a missing forecast stays unknown.
    #[test]
    fn inventory_375_reports_age_consumers_and_unknowns() {
        let f = fixture("read");
        let (ok, text) = ask(&f, 7, json!({"server":"invented-fresh"}));
        assert!(ok, "{text}");
        let v: Value = serde_json::from_str(&text).unwrap();
        let server = &v["servers"][0];
        assert_eq!(server["server"], "invented-fresh/native");
        assert!(server["data_age_seconds"].as_i64().unwrap() >= 600);
        let order: Vec<_> = server["entities"].as_array().unwrap().iter().map(|e| e["inventory"]["identity"].as_str().unwrap()).collect();
        assert_eq!(order, ["invented-fresh/native", "/", "/srv/invented-big", "/srv/invented-small"]);
        assert_eq!(server["entities"][0]["inventory"]["unknown"]["ram_used_bytes"], "collection-failed");
        assert!(server["entities"][0]["inventory"]["ram_used_bytes"].is_null());
        assert_eq!(server["entities"][1]["trends"]["disk"]["state"], "unknown");
        assert_eq!(v["source"]["state"], "incomplete");
        let future = json!({"at": crate::now() as i64 + 60});
        assert!(age(&future, crate::now() as i64).is_null(), "a future producer time is unknown, never a negative age");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // #375 A6: no grant refuses before the source runs; a narrow grant never reads another server, by name or by "all".
    #[test]
    fn inventory_375_refuses_people_without_server_access() {
        let f = fixture("refuse");
        let (ok, text) = ask(&f, 9, json!({"server":"invented-fresh"}));
        assert!(!ok && text == REFUSAL, "{text}");
        assert!(!f.log.exists(), "a person without a server grant must not start the source");
        let (ok, text) = ask(&f, 8, json!({"server":"invented-foreign"}));
        assert!(!ok && text == REFUSAL, "{text}");
        let (ok, text) = ask(&f, 8, json!({"server":"all"}));
        assert!(ok && text.contains("invented-fresh") && !text.contains("invented-foreign"), "{text}");
        let (ok, text) = ask(&f, 8, json!({}));
        assert!(ok && !text.contains("invented-foreign"), "{text}");
        let (ok, text) = ask(&f, 7, json!({"server":"all"}));
        assert!(ok && text.contains("invented-foreign"), "admin implies every server: {text}");
        f.j.lock().unwrap().revoke(7, 8, &["server:example/invented-fresh/native".into()], 1).unwrap();
        assert!(!ask(&f, 8, json!({"server":"invented-fresh"})).0, "revocation refuses the next read");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // #375 A6 / R16: source diagnostics, writable configuration and broken output never reach the person.
    #[test]
    fn inventory_375_source_failures_stay_private_and_unknown() {
        // R16: even failed-source stderr and argv must stay out of service logs.
        if std::env::var_os("PILOTQ_375_PRIVATE_CHILD").is_some() {
            let f = fixture("private-child");
            std::fs::write(f.dir.join("servers.json"),json!({"executable":f.dir.join("report.sh"),"args":["invented-secret-argument"]}).to_string()).unwrap();
            std::fs::write(f.dir.join("report.sh"), "#!/bin/sh\necho invented-secret-stderr >&2\nexit 1\n").unwrap();
            assert!(!ask(&f, 7, json!({})).0);
            std::fs::remove_dir_all(f.dir).unwrap();
            return;
        }
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "servers::tests::inventory_375_source_failures_stay_private_and_unknown", "--nocapture"])
            .env("PILOTQ_375_PRIVATE_CHILD", "1").output().unwrap();
        assert!(child.status.success());
        for secret in ["invented-secret-stderr", "invented-secret-argument"] {
            assert!(!String::from_utf8_lossy(&child.stderr).contains(secret), "source diagnostics leaked to service logs");
        }
        let f = fixture("source");
        let (ok, text) = ask(&f, 7, json!({}));
        assert!(ok && !text.contains("invented-secret-stderr"), "{text}");
        std::fs::write(f.dir.join("report.json"), "{\"schema\":1}").unwrap();
        let (ok, text) = ask(&f, 7, json!({}));
        assert!(!ok && text.contains("no server table"), "older DashQ reads unavailable, never an empty fleet: {text}");
        std::fs::set_permissions(f.dir.join("servers.json"), std::fs::Permissions::from_mode(0o644)).unwrap();
        let (ok, text) = ask(&f, 7, json!({}));
        assert!(!ok && text.contains("owner-only"), "{text}");
        std::fs::remove_file(f.dir.join("servers.json")).unwrap();
        let (ok, text) = ask(&f, 7, json!({}));
        assert!(!ok && text.contains("not configured"), "{text}");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // #375 A4/A6: the DashQ link selects only among readable rows; a forged or foreign digest selects nothing and names nothing.
    #[test]
    fn link_375_selects_only_a_readable_server() {
        // DashQ computes the same value; this vector is the cross-repository contract.
        assert_eq!(link_payload(OWN), "server-0ad79b58009edbd1");
        assert!(link_payload(OWN).len() <= 64 && link_payload(OWN).bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'));
        let f = fixture("link");
        let session = f.j.lock().unwrap().ask(8, "first", crate::now() as i64).unwrap().session;
        let session = session.as_str();
        let reply = start(&f, 8, session, &link_payload(FOREIGN));
        assert_eq!(reply, REFUSAL);
        assert!(f.j.lock().unwrap().selected_server(session).unwrap().is_none());
        let reply = start(&f, 9, "dm:9", &link_payload(OWN));
        assert_eq!(reply, REFUSAL);
        assert!(f.j.lock().unwrap().selected_server("dm:9").unwrap().is_none());
        let reply = start(&f, 8, session, &link_payload(OWN));
        assert!(reply.contains("invented-fresh/native") && reply.contains("10 min") && reply.contains("after you confirm"), "{reply}");
        assert_eq!(f.j.lock().unwrap().selected_server(session).unwrap().as_deref(), Some(OWN));
        // The selection scopes a later no-argument read in that conversation, and still rechecks grants.
        let run = f.j.lock().unwrap().ask(8, "what takes space", crate::now() as i64).unwrap();
        assert_eq!(run.session, session);
        let (ok, text) = call(&f.d, &f.j, &run, &json!({}), Instant::now() + Duration::from_secs(10));
        assert!(ok && text.contains("\"scope\":\"selected\"") && text.contains("invented-fresh"), "{text}");
        f.j.lock().unwrap().select_server(&run.session, FOREIGN).unwrap();
        let (ok, text) = call(&f.d, &f.j, &run, &json!({}), Instant::now() + Duration::from_secs(10));
        assert!(!ok && text == REFUSAL, "{text}");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // #375 A5: the only process is the protected local report with its exact arguments; changes are instructed through task_plan.
    #[test]
    fn inventory_375_runs_only_the_local_report() {
        let f = fixture("process");
        assert!(ask(&f, 7, json!({"server":"invented-fresh"})).0);
        assert_eq!(std::fs::read_to_string(&f.log).unwrap(), "report\n");
        for rule in ["runs no command on any server", "becomes a task_plan draft", "launched only after the person confirms", "Never route around a server refusal"] {
            assert!(crate::direct::INSTRUCTIONS.contains(rule), "missing instruction: {rule}");
        }
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // #375 §5: the budget keeps a large inventory bounded and states what it left out.
    #[test]
    fn inventory_375_bounds_output_and_counts_omissions() {
        let view = view(&row(OWN, "invented-fresh/native", 1), 2, 400);
        assert!(view["omitted_entities"].as_u64().unwrap() > 0, "{view}");
        assert!(view.to_string().len() < 1200, "{view}");
        let f = fixture("final-cap");
        let rows: Vec<_> = (0..300).map(|i| row(&format!("selector-{i}"), &format!("invented-{i}"), 1)).collect();
        std::fs::write(f.dir.join("report.json"), json!({"servers":{"source":{"state":"ok","at":1},"rows":rows}}).to_string()).unwrap();
        let (ok, text) = ask(&f, 7, json!({"server":"all"}));
        assert!(ok, "{text}");
        assert!(text.len() <= OUTPUT_CAP, "final JSON exceeded its cap: {} bytes", text.len());
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["servers"].as_array().unwrap().len(),300,"the cap must not silently drop servers");
        assert!(v["servers"].as_array().unwrap().iter().any(|s|s["omitted_entities"].as_u64().unwrap()>0));
        std::fs::write(f.dir.join("report.json"),json!({"servers":{"rows":[row("invented-large",&"x".repeat(OUTPUT_CAP),1)]}}).to_string()).unwrap();
        let (ok,text)=ask(&f,7,json!({"server":"all"}));
        assert!(!ok && text.contains("name one server") && text.len()<OUTPUT_CAP,"oversized identities must request a narrower read");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // §6: both tool and deep-link failures follow the person's resolved language.
    #[test]
    fn inventory_375_localizes_errors() {
        let f = fixture("language");
        let run = f.j.lock().unwrap().ask(9, "Что занимает место на сервере?", crate::now() as i64).unwrap();
        let (ok, text) = call(&f.d, &f.j, &run, &json!({}), Instant::now() + Duration::from_secs(10));
        assert!(!ok && text.starts_with("Нет доступа"), "{text}");
        f.j.lock().unwrap().ask(7, "Отвечай по-русски", crate::now() as i64).unwrap();
        std::fs::remove_file(f.dir.join("servers.json")).unwrap();
        let reply = start(&f, 7, "cli:7", &link_payload(OWN));
        assert!(reply.contains("не настроен") && !reply.contains("not configured"), "{reply}");
        let run=f.j.lock().unwrap().ask(7,"Что занимает место?",crate::now() as i64).unwrap();
        let (ok,text)=call(&f.d,&f.j,&run,&json!({}),Instant::now()+Duration::from_secs(10));
        assert!(!ok && text.contains("не настроен") && !text.contains("not configured"),"{text}");
        std::fs::write(f.dir.join("servers.json"),json!({"executable":f.dir.join("report.sh"),"args":["report"]}).to_string()).unwrap();
        std::fs::set_permissions(f.dir.join("servers.json"),std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(f.dir.join("report.json"),"{}").unwrap();
        let reply=start(&f,7,"cli:7",&link_payload(OWN));
        assert!(reply.contains("нет таблицы") && !reply.contains("no server table"),"{reply}");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // §5/A3: the clarification must accept the workspace and full host it requests.
    #[test]
    fn inventory_375_resolves_workspace_disambiguation() {
        let f=fixture("workspace");
        let own=row(OWN,"invented-fresh/native",1);
        let mut other=row("invented-other-selector","invented-fresh/native",1);
        other["workspace"]=json!("other");
        std::fs::write(f.dir.join("report.json"),json!({"servers":{"rows":[own,other]}}).to_string()).unwrap();
        let (ok,text)=ask(&f,7,json!({"server":"invented-fresh/native"}));
        assert!(!ok && text.contains("workspace"),"{text}");
        let (ok,text)=ask(&f,7,json!({"server":"example/invented-fresh/native"}));
        assert!(ok,"workspace clarification must resolve: {text}");
        let v:Value=serde_json::from_str(&text).unwrap();
        assert_eq!(v["servers"].as_array().unwrap().len(),1);
        assert_eq!(v["servers"][0]["workspace"],"example");
        assert!(!ask(&f,8,json!({"server":"other/invented-fresh/native"})).0,"a qualified name grants no access");
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // §8/A6: revoking one server blocks its already queued answer, across restart,
    // even if another server remains readable or a later call reads that other server.
    #[test]
    fn inventory_375_revalidates_delayed_delivery() {
        let f = fixture("delivery");
        let mut j = f.j.lock().unwrap();
        j.grant(7, 8, &["server:example/invented-foreign/native".into()], 0).unwrap();
        let input = crate::journal::Input { files: vec![], callback: None, media_group: None, quote: None, image: None, voice: None, language: None, chat: 8, topic: None, sender: Some(8), name: None, message: 1, kind: "text", text: Some("what takes space".into()), reply_to: None };
        j.admit(&[(1, Some(input))], 0).unwrap();
        let run = j.claim_next(1, 1).unwrap().remove(0);
        j.0.execute("UPDATE outbox SET state='delivered'", []).unwrap();
        drop(j);
        for server in ["invented-fresh", "invented-foreign"] {
            assert!(call(&f.d, &f.j, &run, &json!({"server":server}), Instant::now() + Duration::from_secs(10)).0);
        }
        let mut j = f.j.lock().unwrap();
        j.preview(&run,"inventory preview").unwrap();
        assert_eq!(j.previews().unwrap().len(),1);
        j.revoke(7, 8, &["server:example/invented-fresh/native".into()], 3).unwrap();
        assert!(j.previews().unwrap().is_empty(),"revoked inventory escaped through a DM preview");
        j.finish(&run, None, None, &crate::manager::Outcome::Answered("inventory findings".into()), 4).unwrap();
        let reopened = Journal::open(&f.dir.join("journal.sqlite")).unwrap();
        assert!(reopened.due(3).unwrap().0.is_none(), "revoked inventory escaped through the outbox");
        assert_eq!(reopened.0.query_row("SELECT count(*) FROM outbox WHERE request_id=?1 AND state='blocked'", [run.request], |r| r.get::<_, i64>(0)).unwrap(), 2);
        drop(reopened); drop(j);
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // §8: a blocked answer must not return through later context or an old native session.
    #[test]
    fn inventory_375_revalidates_context_and_native_session() {
        let f=fixture("context");
        let run=f.j.lock().unwrap().ask(8,"what takes space",0).unwrap();
        assert!(call(&f.d,&f.j,&run,&json!({}),Instant::now()+Duration::from_secs(10)).0);
        let mut j=f.j.lock().unwrap();
        let tools=json!([tool()]);
        j.catalogue(&run,&tools).unwrap();
        j.finish(&run,Some(("codex","invented-native-session")),None,&crate::manager::Outcome::Answered("invented-private-inventory-findings".into()),1).unwrap();
        j.revoke(7,8,&["server:example/invented-fresh/native".into()],2).unwrap();
        j.grant(7,8,&["server:example/invented-foreign/native".into()],2).unwrap();
        let next=j.ask(8,"repeat your answer",3).unwrap();
        assert!(next.thread.is_some());
        assert!(!j.conversation_context(&next).unwrap().to_string().contains("invented-private-inventory-findings"));
        assert!(!j.rebuild(&next,3,crate::journal::RebuildReason::PolicyRefresh).unwrap().to_string().contains("invented-private-inventory-findings"));
        j.catalogue(&next,&tools).unwrap();
        assert!(j.refresh_tools(&next).unwrap(),"server grant changes must refresh an unchanged native catalogue");
        j.finish(&next,None,None,&crate::manager::Outcome::Answered("fresh authorized answer".into()),4).unwrap();
        let third=j.ask(8,"what about the other server",5).unwrap();
        drop(j);
        assert!(call(&f.d,&f.j,&third,&json!({"server":"invented-foreign"}),Instant::now()+Duration::from_secs(10)).0);
        let mut j=f.j.lock().unwrap();
        j.finish(&third,None,None,&crate::manager::Outcome::Answered("invented-foreign-findings".into()),6).unwrap();
        let derived=j.ask(8,"explain that answer",7).unwrap();
        assert!(j.conversation_context(&derived).unwrap().to_string().contains("invented-foreign-findings"));
        j.0.execute("UPDATE requests SET front='telegram' WHERE id=?1",[derived.request]).unwrap();
        j.preview(&derived,"derived inventory preview").unwrap();
        assert_eq!(j.previews().unwrap().len(),1);
        j.revoke(7,8,&["server:example/invented-foreign/native".into()],8).unwrap();
        assert!(j.previews().unwrap().is_empty(),"context-only turns must retain the source's access binding");
        drop(j);
        std::fs::remove_dir_all(f.dir).unwrap();
    }

    // §8: revocation after input loading must not mark a contaminated native session reusable.
    #[test]
    fn inventory_375_preparation_revocation_refreshes_next_turn() {
        let f=fixture("prepare-race");
        let run=f.j.lock().unwrap().ask(8,"what takes space",0).unwrap();
        assert!(call(&f.d,&f.j,&run,&json!({}),Instant::now()+Duration::from_secs(10)).0);
        let mut j=f.j.lock().unwrap();
        let tools=json!([tool()]);
        j.catalogue(&run,&tools).unwrap();
        j.finish(&run,Some(("codex","invented-old-session")),None,&crate::manager::Outcome::Answered("invented-private-findings".into()),1).unwrap();
        let next=j.ask(8,"explain",2).unwrap();
        assert!(j.conversation_context(&next).unwrap().to_string().contains("invented-private-findings"));
        j.revoke(7,8,&["server:example/invented-fresh/native".into()],3).unwrap();
        j.grant(7,8,&["server:example/invented-foreign/native".into()],3).unwrap();
        j.catalogue(&next,&tools).unwrap();
        j.finish(&next,Some(("codex","invented-contaminated-session")),None,&crate::manager::Outcome::Answered("invented-private-derived-findings".into()),4).unwrap();
        let third=j.ask(8,"repeat",5).unwrap();
        assert!(!j.conversation_context(&third).unwrap().to_string().contains("invented-private-derived-findings"));
        j.catalogue(&third,&tools).unwrap();
        assert!(j.refresh_tools(&third).unwrap(),"post-revocation native input must force a fresh session on the next turn");
        drop(j);std::fs::remove_dir_all(f.dir).unwrap();
    }

    // §8: /new preserves drafts; loading one still carries its inventory scope.
    #[test]
    fn inventory_375_old_draft_retains_access_binding() {
        for mode in ["plan","draft"] {
            let f=fixture(&format!("old-{mode}"));
            let mut j=f.j.lock().unwrap();
            j.grant(7,8,&["project:invented".into()],0).unwrap();
            j.add_project(7,"invented","invented/repo",f.dir.to_str().unwrap(),0).unwrap();
            let run=j.ask(8,"propose a change",0).unwrap();
            drop(j);
            assert!(call(&f.d,&f.j,&run,&json!({}),Instant::now()+Duration::from_secs(10)).0);
            let mut j=f.j.lock().unwrap();
            j.save_plan(&run,&json!([{"title":"invented-private-draft"}]),"invented","invented/repo",f.dir.to_str().unwrap(),1).unwrap();
            j.finish(&run,None,None,&crate::manager::Outcome::Answered("saved".into()),2).unwrap();
            j.builtin_cli(8,"/new",3).unwrap();
            let next=j.ask(8,"explain the saved draft",4).unwrap();
            assert!(next.generation>run.generation);
            let context=if mode=="plan" {j.conversation_context(&next).unwrap()} else {j.draft_context(&next).unwrap()};
            assert!(context.to_string().contains("invented-private-draft"));
            j.0.execute("UPDATE requests SET front='telegram' WHERE id=?1",[next.request]).unwrap();
            j.preview(&next,"draft-derived inventory preview").unwrap();
            assert_eq!(j.previews().unwrap().len(),1);
            j.revoke(7,8,&["server:example/invented-fresh/native".into()],5).unwrap();
            assert!(j.previews().unwrap().is_empty(),"{mode} lost its prior-generation source binding");
            drop(j);std::fs::remove_dir_all(f.dir).unwrap();
        }
    }


    // §8/A4: selecting a server also discloses its identity in a deterministic reply.
    #[test]
    fn link_375_revalidates_delayed_delivery() {
        let f=fixture("link-delivery");
        let mut j=f.j.lock().unwrap();
        let input=crate::journal::Input { files: vec![], callback: None, media_group: None, quote: None, image: None, voice: None, language: None, chat: 8, topic: None, sender: Some(8), name: None, message: 1, kind: "text", text: Some(format!("/start {}",link_payload(OWN))), reply_to: None };
        j.admit(&[(1,Some(input))],0).unwrap();
        let r=j.builtin_next().unwrap().unwrap();
        j.0.execute("UPDATE outbox SET state='delivered'",[]).unwrap();
        drop(j);
        let reply=select(&f.d.keys.0,&f.j,&r).unwrap();
        assert!(reply.contains("invented-fresh/native"));
        let mut j=f.j.lock().unwrap();
        j.builtin_finish(r.id,&reply,true,1).unwrap();
        j.revoke(7,8,&["server:example/invented-fresh/native".into()],2).unwrap();
        assert!(j.due(2).unwrap().0.is_none(),"revoked selection escaped through the outbox");
        assert!(j.0.query_row("SELECT count(*) FROM outbox WHERE request_id=?1 AND state='blocked'",[r.id],|r|r.get::<_,i64>(0)).unwrap()>0);
        drop(j);
        std::fs::remove_dir_all(f.dir).unwrap();
    }
}
