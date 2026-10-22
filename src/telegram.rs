//! Telegram long polling and outbox delivery over the journal (pilotq.md § 4). teloxide is the Bot API client only.

use crate::journal::{Delivery, Input, Journal, Sent};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::{MediaKind, MessageId, MessageKind, ParseMode, ReplyParameters, ThreadId, UpdateKind};
use teloxide::{ApiError, RequestError};
use tokio::sync::Notify;

/// Long-poll wait in seconds; it must stay below teloxide's 17 s client timeout, or every idle poll fails.
const POLL: u32 = 10;

pub type Shared = Arc<Mutex<Journal>>;

fn now() -> i64 {
    crate::now() as i64
}

/// Test hook for F01: SIGKILL this process at a named point, so no destructor or flush runs.
// ponytail: env-gated fault point; a fault-injection framework only if more points appear.
fn fault(point: &str) {
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

async fn send(bot: &Bot, d: &Delivery) -> Sent {
    let backoff = d.attempts + 1;
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
        Err(RequestError::Api(ApiError::CantParseEntities(_))) => Sent::Plain,
        Err(RequestError::Api(e)) => {
            let s = e.to_string();
            if s.starts_with("Forbidden") || s.contains("chat not found") || s.contains("thread not found") { Sent::Held(s) } else { Sent::Failed(s) }
        }
        Err(e @ RequestError::MigrateToChatId(_)) => Sent::Held(e.to_string()),
        Err(RequestError::RetryAfter(s)) => Sent::Retry { after: s.seconds() as i64, ambiguous: false, error: "Telegram retry-after".into() },
        // a refused connection never reached Telegram; any later loss may have produced a visible message
        Err(RequestError::Network(e)) => Sent::Retry { after: backoff, ambiguous: !e.is_connect(), error: e.to_string() },
        Err(e) => Sent::Retry { after: backoff, ambiguous: true, error: e.to_string() },
    }
}

/// Sends due outbox rows one at a time. A lost response is retried and marked as a possible duplicate; execution is never rerun.
pub async fn deliver(bot: Bot, j: Shared, wake: Arc<Notify>) {
    loop {
        let due = j.lock().unwrap().due(now());
        match due {
            Ok((Some(d), _)) => {
                let sent = send(&bot, &d).await;
                match &sent {
                    Sent::Ok(_) => health(&j, "delivery", "ok", None),
                    Sent::Plain => {}
                    Sent::Retry { error, .. } | Sent::Held(error) | Sent::Failed(error) => health(&j, "delivery", "failed", Some(error)),
                }
                let r = j.lock().unwrap().record(d.id, &sent, now());
                if let Err(e) = r {
                    health(&j, "storage", "failed", Some(&e));
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
            Ok((None, next)) => {
                let wait = next.map_or(60, |t| (t - now()).clamp(1, 60)) as u64;
                tokio::select! {
                    _ = wake.notified() => {}
                    _ = tokio::time::sleep(Duration::from_secs(wait)) => {}
                }
            }
            Err(e) => {
                health(&j, "storage", "failed", Some(&e));
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
