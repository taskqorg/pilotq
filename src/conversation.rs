//! §2 / #165: person-facing plan projection; worker detail stays in the saved task.
use serde_json::Value;

/// Internal failure prose is retained by the caller in audit, never forwarded to chat.
pub fn reply(text: &str, language: Option<&str>) -> String {
    let mut changed=false;
    let lines:Vec<String>=text.lines().filter_map(|line| {
        let diagnostic = ["op_id", "op-id", "Plan revision ", "plan revision ", "no held readback", "taskq add failed", "immutable request", "PilotQ retries", "Error:", "error:", "This request reached its budget", "operation ID", "retried with", "TaskQ effect", "{\""]
            .iter().any(|marker| line.contains(marker))
            || line.split_whitespace().any(|word| word.starts_with("pilotq-") && (word.contains("-r") || word.contains("-p")) && (word.contains("-e") || word.contains("-t")))
            || serde_json::from_str::<Value>(line).is_ok_and(|v| v.is_object() || v.is_array());
        if diagnostic {changed=true;None} else {
            // TaskQ tool observations carry internal revisions; their human status stays useful.
            Some(if let Some((status,_))=line.split_once(" (revision ") {changed=true;status} else {line}.to_string())
        }
    }).collect();
    if !changed {return text.into()}
    let clean=lines.join("\n");
    if clean.trim().is_empty() && !text.trim().is_empty() {
        if language.unwrap_or("en").starts_with("ru") { "Не удалось завершить действие; результат пока не подтверждён.".into() }
        else { "Could not finish the action; its result is not confirmed yet.".into() }
    } else { clean }
}

pub fn refusal(language: Option<&str>) -> &'static str {
    if language.unwrap_or("en").starts_with("ru") {"Ответ не сохранён; проверьте актуальный вопрос и права."}
    else {"Reply not saved; check the current question and access."}
}

/// Only successful, bounded read results become fallback findings; never project a mutation as success.
pub fn finding(name: &str, text: &str, language: Option<&str>) -> Option<String> {
    let ru=language.unwrap_or("en").starts_with("ru");
    let v:Value=serde_json::from_str(text).ok()?;
    match name {
        "project_list" => {
            let names=v["projects"].as_array()?.iter().filter_map(|p|p["name"].as_str()).collect::<Vec<_>>().join(", ");
            Some(format!("{}: {}",if ru {"Доступные проекты"} else {"Accessible projects"},if names.is_empty() {if ru {"нет"} else {"none"}} else {&names}))
        }
        "issue_read" => Some(format!("{}\n{}",v["title"].as_str()?,v["body"].as_str().unwrap_or(""))),
        _ => None,
    }.map(|s|reply(&s,language))
}

pub fn progress(phase: &str, elapsed: i64, language: Option<&str>) -> String {
    let ru=language.unwrap_or("en").starts_with("ru");
    let label=match (ru,phase) {
        (true,"creating")=>"Создаю задачи",(true,"starting")=>"Отправляю задачи в работу",(true,_)=>"Ожидаю продолжения",
        (false,"creating")=>"Creating tasks",(false,"starting")=>"Starting tasks",(false,_)=>"Waiting to continue",
    };
    format!("{label} · {} {}",elapsed.max(0),if ru {"с"} else {"s"})
}

pub fn task_state(state: &str, closed: bool, language: Option<&str>) -> &'static str {
    let ru = language.unwrap_or("en").starts_with("ru");
    if closed { return if ru { "Задача закрыта; результат по ссылке" } else { "Task closed; see the result" }; }
    match (ru, state) {
        (true, "held") => "Создана, ещё не в очереди",
        (true, "ready") => "В очереди к исполнителю",
        (true, "doing") => "Исполнитель работает",
        (true, "ask") => "Ожидает вашего ответа",
        (true, "review") => "Результат на проверке",
        (true, "waiting") => "Ожидает зависимые задачи",
        (true, "later") => "Отложена",
        (true, "failed") => "Попытка завершилась без результата",
        (true, _) => "Состояние неизвестно",
        (false, "held") => "Created; not queued yet",
        (false, "ready") => "Queued for a worker",
        (false, "doing") => "Worker is working",
        (false, "ask") => "Waiting for your answer",
        (false, "review") => "Result is being reviewed",
        (false, "waiting") => "Waiting for dependent tasks",
        (false, "later") => "Parked",
        (false, "failed") => "Attempt failed without a result",
        (false, _) => "State unknown",
    }
}

pub fn plan(tickets: &Value, language: Option<&str>) -> String {
    let ru = language.unwrap_or("en").starts_with("ru");
    tickets.as_array().into_iter().flatten().map(|ticket| {
        let title = ticket["title"].as_str().unwrap_or("");
        let goal = ticket["goal"].as_str().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
        // Existing immutable drafts predate done_when; use one acceptance line, never the list.
        let done = ticket["done_when"].as_str().or_else(|| ticket["acceptance"][0].as_str()).unwrap_or("");
        if ru { format!("**{title}**\nСделаем: {goal}\nГотово, когда: {done}") }
        else { format!("**{title}**\nWe will: {goal}\nDone when: {done}") }
    }).collect::<Vec<_>>().join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // #165 A2/A3/A6: owner before cases must lose diagnostics; task bodies keep
    // acceptance/scope while chat contains only the localized outcome summary.
    #[test]
    fn conversation_snapshots() {
        let fixture: Value = serde_json::from_str(include_str!("fixtures/conversation-165.json")).unwrap();
        let clean = |text: &str| {
            for forbidden in ["op_id", "op-id", "{\"", "revision 1", "Error: --", "Draft saved locally", "Use Run plan"] {
                assert!(!text.contains(forbidden), "{text}");
            }
        };
        for lang in ["en","ru"] {
            assert_eq!(refusal(Some(lang)),fixture["recovered_refusal"][lang].as_str().unwrap());
            clean(refusal(Some(lang)));
        }
        for case in fixture["plans"].as_array().unwrap() {
            let rendered = plan(&case["tickets"], case["language"].as_str());
            assert_eq!(rendered, case["snapshot"].as_str().unwrap());
            clean(&rendered);
        }
        for case in fixture["progress"].as_array().unwrap() {
            for lang in ["en","ru"] {let text=progress(case["phase"].as_str().unwrap(),case["seconds"].as_i64().unwrap(),Some(lang));assert_eq!(text,case[lang].as_str().unwrap());clean(&text);}
        }
        for case in fixture["stages"].as_array().unwrap() {
            for lang in ["en", "ru"] {
                let text = task_state(case["state"].as_str().unwrap(), case["closed"].as_bool().unwrap(), Some(lang));
                assert_eq!(text, case[lang].as_str().unwrap());
                clean(text);
            }
        }
        for line in fixture["owner_before"][1].as_str().unwrap().lines().skip(1) {
            for lang in ["en", "ru"] {
                let rendered = reply(line, Some(lang));
                assert_eq!(rendered, fixture["failure"][lang].as_str().unwrap());
                clean(&rendered);
            }
        }
        let before = fixture["owner_before"][0].as_str().unwrap();
        assert!(before.contains("gpt-6.1-sol") && before.contains("Draft saved locally"));
        assert!(!plan(&fixture["plans"][0]["tickets"],Some("en")).contains("gpt-"));
    }
}
