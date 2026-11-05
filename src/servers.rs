//! Read-only server discussion through DashQ's server projection (pilotq.md §5, #375).
//! Nothing here reaches a server: the only process is the owner-configured local report.
use crate::{builtins::Request, direct::Direct, journal::{Journal, Run}};
use serde_json::{Value, json};
use std::{path::Path, sync::Mutex, time::{Duration, Instant}};

pub const PREFIX: &str = "server-";
const SOURCE_CAP: usize = 16 << 20;
const OUTPUT_CAP: usize = 128 << 10;
const REFUSAL: &str = "No access to that server, or it is not in the inventory. Ask an administrator for its server grant; nothing was read.";

pub fn tool() -> Value {
    json!({"type":"function","name":"server_inventory","description":crate::catalog::purpose("server_inventory"),"inputSchema":{"type":"object","properties":{"server":{"type":"string"}},"additionalProperties":false}})
}

pub fn any_access(j: &Journal, user: i64) -> Result<bool, String> {
    Ok(j.has(user, "chat")? && (j.has(user, "servers")? || j.grants(user)?.iter().any(|g| g.starts_with("server:"))))
}

fn permitted(j: &Journal, user: i64, row: &Value) -> Result<bool, String> {
    let name = format!("server:{}/{}", row["workspace"].as_str().unwrap_or("-"), row["host"].as_str().unwrap_or("")).to_lowercase();
    Ok(j.has(user, "servers")? || j.has(user, &name)?)
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
    let output = crate::commands::execute_bounded(crate::commands::command(bin, cwd, &args), end, active, SOURCE_CAP);
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
        let size = entity.to_string().len();
        if used + size > budget { break; }
        used += size;
        kept.push(json!({"inventory":entity["inventory"],"source":entity["source"],"trends":entity["trends"]}));
    }
    json!({"server":row["host"],"workspace":row["workspace"],"state":row["source"]["state"],"reason":row["source"]["reason"],"observed_at":row["source"]["at"],
        "data_age_seconds":age(&row["source"],now),"omitted_entities":total-kept.len(),"entities":kept})
}

fn named<'a>(rows: &'a [Value], wanted: &str) -> Vec<&'a Value> {
    rows.iter().filter(|r| r["pilotq_selector"] == wanted || r["host"].as_str().is_some_and(|h|
        h.eq_ignore_ascii_case(wanted) || h.rsplit_once('/').is_some_and(|(n, _)| n.eq_ignore_ascii_case(wanted)))).collect()
}

fn visible(j: &Journal, user: i64, data: &Value) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    for row in data["rows"].as_array().into_iter().flatten() { if permitted(j, user, row)? { rows.push(row.clone()); } }
    Ok(rows)
}

pub fn call(d: &Direct, j: &Mutex<Journal>, r: &Run, args: &Value, deadline: Instant) -> (bool, String) {
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
            let names: Vec<String> = chosen.iter().map(|r| format!("{}: {}", r["workspace"].as_str().unwrap_or("-"), r["host"].as_str().unwrap_or(""))).collect();
            return Err(format!("that name matches several servers; ask which one: {}", names.join(", ")));
        }
        let budget = OUTPUT_CAP / chosen.len().max(1);
        let servers: Vec<Value> = chosen.iter().map(|row| view(row, now, budget)).collect();
        j.direct_event(r.request, "", &format!("server_inventory {scope}"), None, &format!("read {} servers", servers.len()), now)?;
        Ok(json!({"read_at":now,"scope":scope,"source":{"state":data["source"]["state"],"reason":data["source"]["reason"],"observed_at":data["source"]["at"],"data_age_seconds":age(&data["source"],now)},"servers":servers}).to_string())
    })();
    match result { Ok(text) => (true, text), Err(error) => (false, error) }
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
    let refused = || if ru { "Нет доступа к этому серверу, либо его нет в инвентаре. Попросите администратора выдать право на сервер.".to_string() } else { REFUSAL.to_string() };
    if !{ let j = j.lock().unwrap(); any_access(&j, r.sender)? } { return Ok(refused()); }
    let end = Instant::now() + Duration::from_secs(20);
    let data = match read(keys, end, || true) {
        Ok(data) => data,
        Err(reason) => return Ok(if ru { format!("Сервер не выбран: инвентарь недоступен ({reason}).") } else { format!("No server selected: {reason}.") }),
    };
    let j = j.lock().unwrap();
    let rows = visible(&j, r.sender, &data)?;
    let found: Vec<&Value> = rows.iter().filter(|row| row["pilotq_selector"].as_str().is_some_and(|s| link_payload(s) == payload)).collect();
    let [row] = found[..] else { return Ok(refused()); };
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
    }
}
