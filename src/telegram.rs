//! Telegram long polling and outbox delivery over the journal (pilotq.md § 4). teloxide is the Bot API client only.

use crate::journal::{Delivery, Input, Journal, Sent};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::{ChatAction, MediaKind, MessageId, MessageKind, ParseMode, ReplyParameters, ThreadId, UpdateKind};
use teloxide::{ApiError, RequestError};
use tokio::sync::Notify;
use serde_json::{Value, json};

/// Long-poll wait in seconds; it must stay below teloxide's 17 s client timeout, or every idle poll fails.
const POLL: u32 = 10;

pub type Shared = Arc<Mutex<Journal>>;

fn now() -> i64 {
    crate::now() as i64
}

/// Test hook for F01: SIGKILL this process at a named point, so no destructor or flush runs.
// ponytail: env-gated fault point; a fault-injection framework only if more points appear.
pub(crate) fn fault(point: &str) {
    if std::env::var("PILOTQ_FAULT").is_ok_and(|p| p == point) {
        let _ = std::process::Command::new("kill").args(["-9", &std::process::id().to_string()]).status();
        std::thread::sleep(Duration::from_secs(10));
    }
}

/// A person's message, else None. Service messages (topic created, chat migrated, member joined, pin) are not input: they get an
/// ID-only disposition and no reply. Found on the live test bot, where each one drew an "unsupported" reply.
fn input(kind: &UpdateKind) -> Option<Input> {
    let UpdateKind::Message(m) = kind else { return None };
    let MessageKind::Common(c) = &m.kind else { return None };
    if matches!(c.media_kind, MediaKind::Migration(_)) {
        return None;
    }
    Some(Input {
        chat: m.chat.id.0,
        topic: m.thread_id.map(|t| t.0.0 as i64),
        sender: m.from.as_ref().map(|u| u.id.0 as i64),
        name: m.from.as_ref().map(|u| u.full_name()),
        message: m.id.0 as i64,
        kind: if m.text().is_some() { "text" } else { "other" },
        text: m.text().map(str::to_string),
        reply_to: m.reply_to_message().map(|r| r.id.0 as i64),
    })
}

fn health(j: &Shared, source: &str, state: &str, reason: Option<&str>) {
    if let Err(e) = j.lock().unwrap().set_health(source, now(), state, reason) {
        eprintln!("health {source}: {e}");
    }
}

/// Polls until a Telegram conflict shows another consumer of this token; that is an incident, not a retry loop (§ 4 Ingress).
pub async fn poll(bot: Bot, j: Shared, wake: Arc<Notify>) {
    loop {
        let cursor = j.lock().unwrap().cursor();
        let offset = match cursor {
            Ok(c) => c,
            Err(e) => {
                health(&j, "storage", "failed", Some(&e));
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        match bot.get_updates().offset(offset as i32).timeout(POLL).await {
            Ok(updates) => {
                fault("after-fetch");
                let batch: Vec<(i64, Option<Input>)> = updates.iter().map(|u| (u.id.0 as i64, input(&u.kind))).collect();
                // the cursor commits with the dispositions; on failure the next call repeats the same offset
                let r = j.lock().unwrap().admit(&batch, now());
                match r {
                    Ok(n) => {
                        health(&j, "poll", "ok", None);
                        health(&j, "storage", "ok", None);
                        fault("after-commit");
                        if n > 0 {
                            wake.notify_one();
                        }
                    }
                    Err(e) => {
                        health(&j, "storage", "failed", Some(&e));
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            Err(RequestError::Api(ApiError::TerminatedByOtherGetUpdates)) => {
                eprintln!("poll: another consumer uses this bot token; polling stopped");
                health(&j, "poll", "conflict", Some("another consumer uses this bot token; polling stopped"));
                return;
            }
            Err(e) => {
                health(&j, "poll", "failed", Some(&e.to_string()));
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Maps a Bot API failure to what the outbox does next.
fn failed(e: RequestError, backoff: i64) -> Sent {
    match e {
        RequestError::Api(ApiError::CantParseEntities(_)) => Sent::Plain,
        RequestError::Api(e) => {
            let s = e.to_string();
            if s.starts_with("Forbidden") || s.contains("chat not found") || s.contains("thread not found") { Sent::Held(s) } else { Sent::Failed(s) }
        }
        e @ RequestError::MigrateToChatId(_) => Sent::Held(e.to_string()),
        RequestError::RetryAfter(s) => Sent::Retry { after: s.seconds() as i64, ambiguous: false, error: "Telegram retry-after".into() },
        // a refused connection never reached Telegram; any later loss may have produced a visible message
        RequestError::Network(e) => Sent::Retry { after: backoff, ambiguous: !e.is_connect(), error: e.to_string() },
        e => Sent::Retry { after: backoff, ambiguous: true, error: e.to_string() },
    }
}

/// A small safe Markdown subset for Telegram HTML. Raw HTML always stays literal; unsupported Markdown stays readable.
/// ponytail: headings, bold, code and HTTP(S) links; a full Markdown parser only when the supported subset needs to grow.
pub(crate) fn html(text: &str) -> String {
    fn escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;") }
    fn inline(mut s: &str) -> String {
        let mut out = String::new();
        while !s.is_empty() {
            let pair = if s.starts_with("**") { Some(("**", "b")) } else if s.starts_with('`') { Some(("`", "code")) } else { None };
            if let Some((mark, tag)) = pair {
                if let Some(end) = s[mark.len()..].find(mark) {
                    let end = end + mark.len();
                    out += &format!("<{tag}>{}</{tag}>", escape(&s[mark.len()..end]));
                    s = &s[end+mark.len()..];
                    continue;
                }
            }
            if s.starts_with('[') {
                if let Some(mid) = s.find("](") {
                    if let Some(end) = s[mid+2..].find(')') {
                        let end = mid+2+end;
                        let url = &s[mid+2..end];
                        if url::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https")) {
                            out += &format!("<a href=\"{}\">{}</a>", escape(url), escape(&s[1..mid]));
                            s = &s[end+1..];
                            continue;
                        }
                    }
                }
            }
            let c = s.chars().next().unwrap();
            out += &escape(&c.to_string());
            s = &s[c.len_utf8()..];
        }
        out
    }
    let mut out = String::new();
    let mut code = false;
    for line in text.split_inclusive('\n') {
        if line.starts_with("```") {
            if code { out += "</pre>"; } else { out += "<pre>"; }
            code = !code;
            continue;
        }
        if code { out += &escape(line); continue; }
        let heading = line.trim_start_matches('#');
        if heading.len() < line.len() && heading.starts_with(' ') {
            out += &format!("<b>{}</b>", inline(heading.trim_start_matches(' ')));
        } else { out += &inline(line); }
    }
    if code { out += "</pre>"; }
    out
}

/// Raw JSON for Bot API methods newer than teloxide (§3). Never expose a token-bearing request URL in diagnostics.
async fn raw(bot: &Bot, j: &Shared, method: &str, body: Value) -> Result<Value, Sent> {
    let chat = body["chat_id"].as_i64().unwrap_or(0);
    let until = j.lock().unwrap().telegram_cooldown(chat)
        .map_err(|_|Sent::Held("cannot read Telegram cooldown".into()))?;
    if until > now() { return Err(Sent::Retry {after:until.saturating_sub(now()),ambiguous:false,error:"Telegram retry-after".into()}); }
    let url = bot.api_url().join(&format!("./bot{}/{method}", bot.token())).map_err(|_| Sent::Failed("invalid Bot API base URL".into()))?;
    let response = bot.client().post(url).json(&body).timeout(Duration::from_secs(4)).send().await
        .map_err(|e| Sent::Retry {after:1,ambiguous:!e.is_connect(),error:"Bot API connection failed".into()})?;
    let v: Value = response.json().await.map_err(|_| Sent::Retry {after:1,ambiguous:true,error:"Bot API response unreadable".into()})?;
    if v["ok"] == true { return Ok(v["result"].clone()); }
    let code = v["error_code"].as_i64().unwrap_or(0);
    let error = v["description"].as_str().unwrap_or("Bot API failed");
    if code == 403 || error.contains("chat not found") || error.contains("thread not found") || v["parameters"]["migrate_to_chat_id"].is_number() {
        return Err(Sent::Held("Telegram destination unavailable; administrator action required".into()));
    }
    if code == 429 {
        let after = v["parameters"]["retry_after"].as_i64().unwrap_or(1).max(1);
        if j.lock().unwrap().cool_telegram(chat, now().saturating_add(after).saturating_add(1)).is_err() {
            return Err(Sent::Held("cannot persist Telegram cooldown".into()));
        }
        return Err(Sent::Retry {after,ambiguous:false,error:"Telegram retry-after".into()});
    }
    if matches!(code, 400 | 404) && method == "sendRichMessage" { return Err(Sent::RichFallback); }
    if code >= 500 || code == 0 { return Err(Sent::Retry {after:1,ambiguous:true,error:"Bot API temporarily unavailable".into()}); }
    Err(Sent::Failed("Bot API rejected request".into()))
}

async fn send(bot: &Bot, j: &Shared, d: &Delivery) -> Sent {
    let backoff = d.attempts + 1;
    if d.kind == "reaction" {
        return match raw(bot, j, "setMessageReaction", json!({"chat_id":d.chat,"message_id":d.reply_to,"reaction":[{"type":"emoji","emoji":"👀"}]})).await {
            Ok(_) => Sent::Ok(d.reply_to.unwrap_or(0)),
            Err(e @ Sent::Retry { .. }) | Err(e @ Sent::Held(_)) => e,
            Err(_) => Sent::Receipt,
        };
    }
    if d.kind == "rich" {
        let mut body = json!({"chat_id":d.chat,"rich_message":{"html":html(&d.text)}});
        if let Some(t) = d.topic { body["message_thread_id"] = json!(t); }
        if let Some(m) = d.reply_to { body["reply_parameters"] = json!({"message_id":m,"allow_sending_without_reply":true}); }
        return match raw(bot, j, "sendRichMessage", body).await {
            Ok(v) => v["message_id"].as_i64().map(Sent::Ok).unwrap_or(Sent::Retry {after:backoff,ambiguous:true,error:"rich message receipt missing".into()}),
            Err(e) => e,
        };
    }
    // a task status update edits its status message in place; a deleted one is replaced by a new message in the same route (L12)
    if let Some(m) = d.edit {
        match bot.edit_message_text(ChatId(d.chat), MessageId(m as i32), &d.text).await {
            Ok(m) => return Sent::Ok(m.id.0 as i64),
            Err(RequestError::Api(ApiError::MessageNotModified)) => return Sent::Ok(m),
            Err(RequestError::Api(ApiError::MessageToEditNotFound)) => {}
            Err(e) => return failed(e, backoff),
        }
    }
    let mut r = bot.send_message(ChatId(d.chat), d.html.as_deref().unwrap_or(&d.text));
    if d.html.is_some() {
        r = r.parse_mode(ParseMode::Html);
    }
    if let Some(t) = d.topic {
        r = r.message_thread_id(ThreadId(MessageId(t as i32)));
    }
    if let Some(m) = d.reply_to {
        r = r.reply_parameters(ReplyParameters::new(MessageId(m as i32)).allow_sending_without_reply());
    }
    match r.await {
        Ok(m) => Sent::Ok(m.id.0 as i64),
        Err(e) => failed(e, backoff),
    }
}

/// Best-effort typing in the exact originating chat and topic while a turn runs, refreshed every 4 s (§ 12, L02). It stops
/// when the run ends or its deadline passes; a failure is ignored and never touches the run.
// ponytail: bounded by the run's absolute deadline, not by its last useful event; an event clock in the journal if stale turns need it
pub async fn typing(bot: Bot, j: Shared, deadline: Duration) {
    loop {
        let routes = j.lock().unwrap().typing(now(), deadline.as_secs() as i64).unwrap_or_default();
        for (chat, topic) in routes {
            if j.lock().unwrap().telegram_cooldown(chat).unwrap_or(i64::MAX) > now() { continue; }
            let mut r = bot.send_chat_action(ChatId(chat), ChatAction::Typing);
            if let Some(t) = topic {
                r = r.message_thread_id(ThreadId(MessageId(t as i32)));
            }
            if let Err(RequestError::RetryAfter(delay)) = r.await {
                let _ = j.lock().unwrap().cool_telegram(chat, now() + delay.seconds() as i64 + 1);
            }
        }
        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

/// Drafts are replaceable previews. The supervisor never waits for this loop, and final content always uses the outbox.
pub async fn drafts(bot: Bot, j: Shared) {
    let mut shown = std::collections::HashMap::<i64, (String, bool)>::new();
    loop {
        let previews = j.lock().unwrap().previews().unwrap_or_default();
        shown.retain(|run, _| previews.iter().any(|p| p.0 == *run));
        for (run, chat, topic, text) in previews {
            let entry = shown.entry(run).or_insert_with(|| (String::new(), true));
            if entry.0 == text || text.is_empty() { continue; }
            // Recheck immediately before dispatch; a stale preview never starts a new request or changes its result.
            if !j.lock().unwrap().previews().unwrap_or_default().iter().any(|p|p.0==run) { continue; }
            let mut body = json!({"chat_id":chat,"draft_id":run});
            if let Some(t) = topic { body["message_thread_id"] = json!(t); }
            let result = if entry.1 {
                let text: String = text.chars().scan(0, |n,c| { *n += c.len_utf16(); (*n<=32768).then_some(c) }).collect();
                body["rich_message"] = json!({"html":html(&text)});
                raw(&bot,&j,"sendRichMessageDraft",body.clone()).await
            } else {
                body["text"] = json!(crate::journal::chunks(&text)[0]);
                raw(&bot,&j,"sendMessageDraft",body).await
            };
            // Fallback starts on the next tick, so even rejected rich drafts never exceed one update per second.
            match result {
                Err(Sent::Retry { .. }) => {} // Rate limits and temporary losses never trigger another method.
                Err(_) if entry.1 => entry.1=false,
                _ => entry.0=text,
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Sends due outbox rows one at a time. A lost response is retried and marked as a possible duplicate; execution is never rerun.
pub async fn deliver(bot: Bot, j: Shared, wake: Arc<Notify>) {
    loop {
        let due = j.lock().unwrap().due(now());
        match due {
            Ok((Some(d), _)) => {
                let sent = send(&bot, &j, &d).await;
                match &sent {
                    Sent::Ok(_) => health(&j, "delivery", "ok", None),
                    Sent::Plain | Sent::RichFallback | Sent::Receipt => {}
                    Sent::Retry { error, .. } | Sent::Held(error) | Sent::Failed(error) => health(&j, "delivery", "failed", Some(error)),
                }
                let r = j.lock().unwrap().record(d.id, &sent, now());
                if let Err(e) = r {
                    health(&j, "storage", "failed", Some(&e));
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
            Ok((None, _)) => {
                tokio::select! {
                    _ = wake.notified() => {}
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }
            }
            Err(e) => {
                health(&j, "storage", "failed", Some(&e));
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::html;

    // §4/§8: model text and link attributes cannot inject Telegram HTML or an unsafe link scheme.
    #[test]
    fn formatting_escapes_untrusted_text_and_links() {
        assert_eq!(html("**<script>& user's**"), "<b>&lt;script&gt;&amp; user's</b>");
        assert_eq!(html("[click](javascript:alert)"), "[click](javascript:alert)");
        assert_eq!(html("[<label>](https://example.invalid/?q=\"&x=1)"),
            "<a href=\"https://example.invalid/?q=&quot;&amp;x=1\">&lt;label&gt;</a>");
        assert_eq!(html("```rust\n<raw> & text\n```"), "<pre>&lt;raw&gt; &amp; text\n</pre>");
        assert_eq!(html("# Heading\n`<raw>`"), "<b>Heading\n</b><code>&lt;raw&gt;</code>");
    }
}
