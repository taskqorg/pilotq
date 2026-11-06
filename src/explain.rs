//! #401 / §2: a PM explanation of one question revision. The model supplies plain-language fields; the service checks them,
//! keeps option numbers, stars and links from the source, and renders the card. An explanation confers no authority.
use crate::telegram::details;
use serde_json::{Value, json};
use std::path::{Component, Path};

/// Conciseness and delivery bound: a card edit must stay one Telegram message, or its buttons would leave the card.
const FIELD: usize = 400;
const LABEL: usize = 48;
const TOTAL: usize = 2000;
const EXCERPT: usize = 12;
const CLAUSE_FILE: u64 = 4 << 20;

#[derive(Debug, PartialEq)]
pub struct Source { pub kind: String, pub path: Option<String>, pub excerpt: String }

#[derive(Debug)]
pub struct Explanation {
    issue: String,
    impact: String,
    requirement: Option<(String, Vec<Source>)>,
    actor: String,
    action: String,
    options: Vec<(u64, String, String, String, String)>,
    reason: Option<String>,
    unknowns: Vec<String>,
}

fn text<'a>(v: &'a Value, field: &str) -> Result<&'a str, String> {
    v[field].as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or_else(|| format!("{field} is required; nothing was changed"))
}

fn digits(s: &str) -> usize { s.chars().rev().take_while(char::is_ascii_digit).count() }

/// A criterion code, hash, attempt ID or code location used as prose; the source details keep them for whoever needs them.
fn technical(word: &str) -> bool {
    let w = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '§');
    let mut chars = w.chars();
    let code = chars.next().is_some_and(char::is_uppercase) && chars.clone().count() > 0 && chars.clone().count() <= 3 && chars.all(|c| c.is_ascii_digit());
    let hash = w.len() >= 7 && w.chars().all(|c| c.is_ascii_hexdigit()) && w.chars().any(|c| c.is_ascii_digit());
    let location = w.rsplit_once(':').is_some_and(|(file, line)| file.contains('.') && !line.is_empty() && line.chars().all(|c| c.is_ascii_digit()));
    code || hash || digits(w) >= 7 || location || w.contains('§')
}

/// Primary text must read without a specification and in the recipient's language; links come only from the verified source.
fn plain(field: &str, value: &str, ru: bool, max: usize) -> Result<(), String> {
    if value.encode_utf16().count() > max { return Err(format!("{field} is longer than {max} characters; shorten it; nothing was changed")); }
    if value.contains("://") { return Err(format!("{field} must not contain a link; the card shows verified source links; nothing was changed")); }
    if let Some(word) = value.split_whitespace().find(|w| technical(w)) {
        return Err(format!("{field} uses the code or identifier {word}; say what it means in plain words, the source details keep it; nothing was changed"));
    }
    if ru && !value.chars().any(|c| matches!(c, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё')) { return Err(format!("{field} must be in Russian, the recipient's language; nothing was changed")); }
    Ok(())
}

/// Checks the shape against this exact question: every source option once, none added, trade-offs only where the source has them.
pub fn parse(args: &Value, q: &Value, ru: bool) -> Result<Explanation, String> {
    let issue = text(args, "issue")?;
    let impact = text(args, "impact")?;
    plain("issue", issue, ru, FIELD)?;
    plain("impact", impact, ru, FIELD)?;
    let requirement = match &args["requirement"] {
        Value::Null => None,
        r => {
            let what = text(r, "plain").map_err(|e| format!("requirement {e}"))?;
            plain("requirement", what, ru, FIELD)?;
            let sources = r["sources"].as_array().filter(|s| !s.is_empty()).ok_or("requirement needs a source you read; omit requirement when the clause is unverified; nothing was changed")?
                .iter().map(|s| {
                    let kind = s["kind"].as_str().filter(|k| matches!(*k, "task" | "file")).ok_or("source kind must be task or file; nothing was changed")?;
                    let path = s["path"].as_str().map(str::to_string);
                    if (kind == "file") != path.is_some() { return Err("a file source names its path and a task source none; nothing was changed".to_string()); }
                    Ok(Source { kind: kind.into(), path, excerpt: s["excerpt"].as_str().unwrap_or("").to_string() })
                }).collect::<Result<Vec<_>, String>>()?;
            Some((what.to_string(), sources))
        }
    };
    let actor = args["next"]["actor"].as_str().filter(|a| matches!(*a, "pm" | "owner" | "worker" | "unknown")).ok_or("next.actor must be pm, owner, worker or unknown; nothing was changed")?;
    let action = args["next"]["action"].as_str().unwrap_or("").trim();
    if actor != "unknown" && action.is_empty() { return Err("next.action is required when the actor is known; nothing was changed".into()); }
    if !action.is_empty() { plain("next.action", action, ru, FIELD)?; }
    let source = crate::taskq::choices(q);
    let given = args["options"].as_array().map_or(&[][..], Vec::as_slice);
    if given.len() != source.len() { return Err(format!("this question has {} options; explain each exactly once by its original number; nothing was changed", source.len())); }
    let mut options = vec![];
    for c in source {
        let id = c["id"].as_u64().ok_or("source option has no number; nothing was changed")?;
        let o = given.iter().find(|o| o["id"].as_u64() == Some(id)).ok_or_else(|| format!("option {id} is missing; keep the original numbers; nothing was changed"))?;
        let (label, meaning) = (text(o, "label")?, text(o, "meaning")?);
        plain("option label", label, ru, LABEL)?;
        plain("option meaning", meaning, ru, FIELD)?;
        let (gain, cost) = (o["gain"].as_str().unwrap_or("").trim(), o["cost"].as_str().unwrap_or("").trim());
        let tradeoffs = c["detail"].as_str().is_some_and(|d| !d.trim().is_empty());
        if tradeoffs && (gain.is_empty() || cost.is_empty()) { return Err(format!("option {id} has source trade-offs; give its gain and cost; nothing was changed")); }
        if !tradeoffs && !(gain.is_empty() && cost.is_empty()) { return Err(format!("option {id} has no source trade-offs; do not invent them; nothing was changed")); }
        for (field, value) in [("option gain", gain), ("option cost", cost)] { if !value.is_empty() { plain(field, value, ru, FIELD)?; } }
        options.push((id, label.to_string(), meaning.to_string(), gain.to_string(), cost.to_string()));
    }
    let recommended = source.iter().any(|c| c["recommended"] == true);
    let reason = args["recommendation_reason"].as_str().map(str::trim).filter(|s| !s.is_empty());
    if reason.is_some() && !recommended { return Err("the source recommends no option; do not invent a recommendation; nothing was changed".into()); }
    if let Some(reason) = reason { plain("recommendation_reason", reason, ru, FIELD)?; }
    let mut unknowns = vec![];
    for u in args["unknowns"].as_array().map_or(&[][..], Vec::as_slice) {
        let u = u.as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or("unknowns must be non-empty text; nothing was changed")?;
        plain("unknowns", u, ru, FIELD)?;
        unknowns.push(u.to_string());
    }
    let e = Explanation { issue: issue.into(), impact: impact.into(), requirement, actor: actor.into(), action: action.into(), options, reason: reason.map(str::to_string), unknowns };
    let total: usize = [&e.issue, &e.impact, &e.action].iter().map(|s| s.encode_utf16().count()).sum::<usize>()
        + e.requirement.as_ref().map_or(0, |r| r.0.encode_utf16().count())
        + e.options.iter().map(|o| [&o.1, &o.2, &o.3, &o.4].iter().map(|s| s.encode_utf16().count()).sum::<usize>()).sum::<usize>()
        + e.reason.as_ref().map_or(0, |s| s.encode_utf16().count())
        + e.unknowns.iter().map(|s| s.encode_utf16().count()).sum::<usize>();
    if total > TOTAL { return Err(format!("the explanation is longer than {TOTAL} characters; keep only what the reader needs; nothing was changed")); }
    Ok(e)
}

fn normalized(s: &str) -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Every string of a TaskQ read, so a clause is matched as text rather than through JSON escaping.
pub fn strings(values: &[&Value]) -> String {
    fn walk(v: &Value, out: &mut String) {
        match v {
            Value::String(s) => { out.push_str(s); out.push('\n'); }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            Value::Object(o) => o.values().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = String::new();
    values.iter().for_each(|v| walk(v, &mut out));
    out
}

/// A2: a claimed requirement must quote text that really exists in this task or in a Markdown clause of the registered root.
pub fn verify(e: &Explanation, task_text: &str, root: &Path) -> Result<(), String> {
    let Some((_, sources)) = &e.requirement else { return Ok(()) };
    let task = normalized(task_text);
    for s in sources {
        let excerpt = normalized(&s.excerpt);
        if excerpt.chars().count() < EXCERPT { return Err(format!("source excerpt must quote at least {EXCERPT} characters of the clause; nothing was changed")); }
        let found = match &s.path {
            None => task.contains(&excerpt),
            Some(path) => normalized(&clause_file(root, path)?).contains(&excerpt),
        };
        if !found { return Err(format!("the excerpt was not found in {}; read the clause again, or omit requirement and state it as unknown; nothing was changed", s.path.as_deref().unwrap_or("the task"))); }
    }
    Ok(())
}

/// Read-only, Markdown only and inside the registered root, so this check cannot probe credentials or other projects.
fn clause_file(root: &Path, path: &str) -> Result<String, String> {
    let relative = Path::new(path);
    let refused = || format!("{path} is not a Markdown clause inside this project's registered root; omit requirement and state it as unknown; nothing was changed");
    if relative.extension().is_none_or(|x| x != "md") || !relative.components().all(|c| matches!(c, Component::Normal(_))) { return Err(refused()); }
    let base = root.canonicalize().map_err(|_| refused())?;
    let file = base.join(relative).canonicalize().map_err(|_| refused())?;
    if !file.starts_with(&base) || std::fs::metadata(&file).map_err(|_| refused())?.len() > CLAUSE_FILE { return Err(refused()); }
    std::fs::read_to_string(&file).map_err(|_| refused())
}

fn evidence_line(q: &Value, ru: bool) -> Option<String> {
    let links: Vec<String> = q["evidence"].as_array().into_iter().flatten()
        .filter_map(|e| Some(format!("[{}]({})", e["label"].as_str()?, e["url"].as_str()?))).collect();
    (!links.is_empty()).then(|| format!("{}: {}", if ru { "Подтверждение" } else { "Evidence" }, links.join(", ")))
}

fn source_block(q: &Value, revision: i64, ru: bool) -> String {
    let title = if ru { format!("Исходный вопрос исполнителя, ревизия {revision}") } else { format!("Worker's source question, revision {revision}") };
    details(&title, &crate::taskq::question_body(q))
}

/// PM-on source card before an explanation exists: every number, label and link stays actionable, and the source stays one tap away.
pub fn compact_question(head: &str, goal: Option<&str>, q: &Value, revision: i64, ru: bool) -> String {
    let mut lines = vec![head.to_string()];
    if let Some(goal) = goal { lines.push(goal.to_string()); }
    lines.push(if ru { "Объяснение PM заменит эту строку, когда будет готово; исходный текст ниже." } else { "A PM explanation replaces this line when it is ready; the source text is below." }.into());
    lines.extend(evidence_line(q, ru));
    for c in crate::taskq::choices(q) {
        lines.push(format!("{}. {}{}", c["id"], c["label"].as_str().unwrap_or(""), if c["recommended"] == true { " ★" } else { "" }));
    }
    format!("{}\n\n{}", lines.join("\n"), source_block(q, revision, ru))
}

fn actor_name(actor: &str, ru: bool) -> &'static str {
    match (actor, ru) {
        ("pm", true) => "PM (техническое действие, выбор владельца не нужен)",
        ("pm", false) => "PM (a technical action; no owner choice is needed)",
        ("owner", true) => "владелец (его решение)",
        ("owner", false) => "owner (their decision)",
        ("worker", true) => "исполнитель",
        ("worker", false) => "worker",
        (_, true) => "неизвестно",
        (_, false) => "unknown",
    }
}

/// The edited card and the question copy whose relabelled buttons keep the source numbers: answers still bind to the source row.
pub fn render(e: &Explanation, q: &Value, head: &str, revision: i64, ru: bool) -> (String, Value) {
    let word = |en: &'static str, ruw: &'static str| if ru { ruw } else { en };
    let mut lines = vec![head.to_string()];
    lines.push(format!("**{}:** {}", word("Issue", "Суть"), e.issue));
    lines.push(format!("**{}:** {}", word("Why it matters", "Почему важно"), e.impact));
    match &e.requirement {
        Some((what, sources)) => {
            lines.push(format!("**{}:** {what}", word("Rule", "Правило")));
            let cited: Vec<String> = sources.iter().map(|s| s.path.as_ref().map_or_else(|| word("task text", "текст задачи").to_string(), |p| format!("`{p}`"))).collect();
            lines.push(format!("{}: {}", word("Source", "Источник"), cited.join(", ")));
        }
        None => lines.push(format!("**{}:** {}", word("Rule", "Правило"), word("not verified; this card claims no violation.", "не подтверждено; карточка не утверждает нарушение."))),
    }
    if !e.unknowns.is_empty() { lines.push(format!("**{}:** {}", word("Unknown", "Неизвестно"), e.unknowns.join("; "))); }
    let actor = actor_name(&e.actor, ru);
    lines.push(if e.action.is_empty() { format!("**{}:** {actor}", word("Next", "Далее")) } else { format!("**{}:** {actor}: {}", word("Next", "Далее"), e.action) });
    lines.extend(evidence_line(q, ru));
    let mut labelled = q.clone();
    let mut options = vec![];
    for (id, label, meaning, gain, cost) in &e.options {
        let source = crate::taskq::choices(q).iter().find(|c| c["id"].as_u64() == Some(*id));
        let star = source.is_some_and(|c| c["recommended"] == true);
        options.push(format!("{id}. {label}{}", if star { " ★" } else { "" }));
        options.push(meaning.clone());
        if !gain.is_empty() { options.push(format!("{}: {gain}. {}: {cost}.", word("Gain", "Плюс"), word("Cost", "Минус"))); }
    }
    if let Some(choices) = labelled["choices"].as_array_mut() {
        for c in choices {
            if let Some((_, label, ..)) = e.options.iter().find(|o| c["id"].as_u64() == Some(o.0)) { c["label"] = json!(label); }
        }
    }
    if crate::taskq::choices(q).iter().any(|c| c["recommended"] == true) {
        options.push(match &e.reason {
            Some(reason) => format!("{}: {reason}", word("Why recommended", "Почему рекомендовано")),
            None => word("Recommendation reason: not confirmed.", "Основание рекомендации не подтверждено.").to_string(),
        });
    }
    let mut body = lines.join("\n");
    if !options.is_empty() { body = format!("{body}\n\n{}", options.join("\n")); }
    // A longer source moves behind the verified task link, so the edit stays one message and keeps its buttons.
    let full = format!("{body}\n\n{}", source_block(q, revision, ru));
    let text = if crate::journal::chunks(&full).len() > 1 { format!("{body}\n\n{}", word("The full source question is in the task link.", "Полный исходный вопрос доступен по ссылке на задачу.")) } else { full };
    (text, labelled)
}

/// What the audit keeps next to the card: which clause text was checked for which revision.
pub fn audit(e: &Explanation, revision: i64) -> String {
    json!({"revision": revision, "next": e.actor, "requirement": e.requirement.as_ref().map(|(_, s)| s.iter().map(|x| json!({"kind": x.kind, "path": x.path, "excerpt": x.excerpt})).collect::<Vec<_>>()), "unknowns": e.unknowns}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Value {
        let text = match name {
            "agentq260" => include_str!("../tests/fixtures/401/agentq260.json"),
            _ => include_str!("../tests/fixtures/401/taskq2482.json"),
        };
        serde_json::from_str(text).unwrap()
    }

    fn root(label: &str, clause: &Value) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("pilotq-explain-{label}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (path, text) in clause.as_object().unwrap() {
            let file = dir.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text.as_str().unwrap()).unwrap();
        }
        dir
    }

    // #401 A7 / §2: replay the two observed cards. Defect: raw English with bare A2/R13, hashes and attempt IDs was the primary
    // text. Risk kept: lost option numbers or stars, invented trade-offs, claims without a source and hidden source text.
    // This proves the boundary and rendering of recorded explanations, not live Telegram delivery.
    #[test]
    fn explanation_401_replays_observed_questions() {
        for (name, topic) in [("agentq260", "дочерн"), ("taskq2482", "исследован")] {
            let f = fixture(name);
            let q = &f["question"];
            let dir = root(name, &f["clauses"]);
            let task = strings(&[&f["task"]]);
            for explanation in ["authored", "model"] {
                let args = &f[explanation];
                let e = parse(args, q, true).unwrap_or_else(|why| panic!("{name} {explanation}: {why}"));
                verify(&e, &task, &dir).unwrap_or_else(|why| panic!("{name} {explanation}: {why}"));
                let (card, labelled) = render(&e, q, "🛑 [source link](https://forge.invalid/1)", 42, true);
                let primary = card.split("> 🔎 ").next().unwrap();
                assert!(primary.to_lowercase().contains(topic), "{name} {explanation} misses the issue: {primary}");
                for word in primary.split_whitespace().filter(|w| !w.contains('(') && !w.contains('`')) { assert!(!technical(word), "{name} {explanation}: {word} in {primary}"); }
                for c in crate::taskq::choices(q) {
                    let id = c["id"].as_u64().unwrap();
                    let star = if c["recommended"] == true { " ★" } else { "" };
                    let shown = crate::taskq::choices(&labelled).iter().find(|x| x["id"].as_u64() == Some(id)).unwrap();
                    assert!(primary.contains(&format!("{id}. {}{star}", shown["label"].as_str().unwrap())), "{name} option {id} fidelity: {primary}");
                    assert_eq!(crate::telegram::options(&labelled).iter().filter(|o| o.0 == format!("answer:{id}")).count(), 1, "{name} button {id}");
                }
                for e in q["evidence"].as_array().unwrap() { assert!(primary.contains(e["url"].as_str().unwrap()), "{name} evidence link"); }
                assert!(card.contains("> 🔎 ") && card.contains(q["problem"].as_str().unwrap()), "{name} keeps literal source in details");
                let html = crate::telegram::html(&card);
                assert!(html.contains("<blockquote expandable>🔎") && !html.contains("&gt; "), "{name} html: {html}");
                assert!(crate::telegram::markdown(&card).contains("<blockquote expandable>🔎"), "{name} rich");
                assert_eq!(crate::journal::chunks(&card).len(), 1, "{name} stays one editable message");
            }
        }
    }

    // #401 A7: a real model's first call on taskq#2482 named a bare code; the boundary refused it, and the corrected call above passes.
    #[test]
    fn explanation_401_refuses_recorded_model_code() {
        let f = fixture("taskq2482");
        let why = parse(&f["model_refused_first"], &f["question"], true).unwrap_err();
        assert!(why.starts_with("unknowns uses the code or identifier R13"), "{why}");
    }

    // #401 A2/A7: honest unknowns. A clause the worker never saw, a made-up quote and a path outside the root all stay unverified.
    #[test]
    fn explanation_401_refuses_unverified_requirements() {
        let f = fixture("agentq260");
        let q = &f["question"];
        let dir = root("sources", &f["clauses"]);
        let task = strings(&[&f["task"]]);
        let mut args = f["authored"].clone();
        args["requirement"]["sources"] = json!([{"kind":"task","excerpt":"this sentence was never in the task text"}]);
        assert!(verify(&parse(&args, q, true).unwrap(), &task, &dir).unwrap_err().contains("not found"));
        for path in ["../outside.md", "keys.json", "/etc/passwd.md", "missing.md"] {
            args["requirement"]["sources"] = json!([{"kind":"file","path":path,"excerpt":"long enough excerpt here"}]);
            assert!(verify(&parse(&args, q, true).unwrap(), &task, &dir).unwrap_err().contains("not a Markdown clause"), "{path}");
        }
        args["requirement"]["sources"] = json!([]);
        assert!(parse(&args, q, true).unwrap_err().contains("source you read"));
        args["requirement"] = Value::Null;
        args["unknowns"] = json!(["Какое именно правило нарушено, не подтверждено."]);
        let e = parse(&args, q, true).unwrap();
        verify(&e, &task, &dir).unwrap();
        let (card, _) = render(&e, q, "🛑 link", 42, true);
        assert!(card.contains("**Правило:** не подтверждено; карточка не утверждает нарушение.") && card.contains("**Неизвестно:**"), "{card}");
    }

    // #401 A1/A3/A4: bare codes, identifiers, foreign links, wrong language, renumbered or invented options never reach the card.
    #[test]
    fn explanation_401_refuses_bare_codes_and_changed_options() {
        let f = fixture("agentq260");
        let q = &f["question"];
        let base = f["authored"].clone();
        let refused = |change: &dyn Fn(&mut Value), expected: &str| {
            let mut args = base.clone();
            change(&mut args);
            let why = parse(&args, q, true).unwrap_err();
            assert!(why.contains(expected) && why.contains("nothing was changed"), "{expected}: {why}");
        };
        for code in ["A2", "R13", "(A3)", "eac48ee2", "6681763630", "entry6680789876", "host.rs:447", "§7"] {
            refused(&|a| a["issue"] = json!(format!("Нарушен пункт {code} задачи")), code);
        }
        refused(&|a| a["impact"] = json!("См. https://forge.invalid/invented"), "must not contain a link");
        refused(&|a| a["issue"] = json!("Child context is lost"), "must be in Russian");
        refused(&|a| { a["options"].as_array_mut().unwrap().pop(); }, "has 2 options");
        refused(&|a| a["options"][1]["id"] = json!(3), "option 2 is missing");
        refused(&|a| a["options"][0]["cost"] = json!(""), "give its gain and cost");
        refused(&|a| a["next"]["actor"] = json!("invented"), "next.actor");
        refused(&|a| a["issue"] = json!("д".repeat(FIELD + 1)), "longer than");
        let plain_q = json!({"parsed":true,"problem":"p","evidence":[],"choices":[{"id":1,"label":"x","recommended":false}]});
        let mut args = base.clone();
        args["options"] = json!([{"id":1,"label":"Вариант","meaning":"Смысл","gain":"выдуман","cost":"выдуман"}]);
        args["recommendation_reason"] = Value::Null;
        assert!(parse(&args, &plain_q, true).unwrap_err().contains("do not invent"));
        args["options"] = json!([{"id":1,"label":"Вариант","meaning":"Смысл"}]);
        args["recommendation_reason"] = json!("Выдуманная причина");
        assert!(parse(&args, &plain_q, true).unwrap_err().contains("recommends no option"));
        // A recommendation without an evidenced reason says so instead of implying one.
        let mut args = base.clone();
        args["recommendation_reason"] = Value::Null;
        let (card, _) = render(&parse(&args, q, true).unwrap(), q, "🛑 link", 42, true);
        assert!(card.contains("Основание рекомендации не подтверждено."), "{card}");
    }

    // #401 A5: before or without an explanation every option number, label, star and link stays visible next to expandable source.
    #[test]
    fn explanation_401_compact_card_keeps_controls_and_source() {
        for name in ["agentq260", "taskq2482"] {
            let f = fixture(name);
            let q = &f["question"];
            for ru in [true, false] {
                let card = compact_question("🛑 [link](https://forge.invalid/1)", Some("Goal: literal"), q, 7, ru);
                let (primary, source) = card.split_once("> 🔎 ").unwrap();
                for c in crate::taskq::choices(q) { assert!(primary.contains(&format!("{}. {}", c["id"], c["label"].as_str().unwrap()))); }
                for e in q["evidence"].as_array().unwrap() { assert!(primary.contains(e["url"].as_str().unwrap())); }
                assert!(!primary.contains(q["problem"].as_str().unwrap()) && source.contains(q["problem"].as_str().unwrap()));
                assert!(source.contains('7'), "source is bound to its revision");
                assert!(primary.contains(if ru { "Объяснение PM" } else { "A PM explanation" }));
            }
        }
    }
}
