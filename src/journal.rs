//! SQLite journal: admitted inputs, ignored-update dispositions, the polling cursor, the outbox (pilotq.md § 4, § 7),
//! and the grant registry, route policy and per-person profiles (§ 2, § 6).

use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// Schema this binary writes; a newer journal is refused rather than guessed (F08).
const SCHEMA: i64 = 2;
/// Delivery attempts per episode, persisted across restart (§ 7).
pub const ATTEMPTS: i64 = 5;
/// Named grants (§ 2, § 8). `chat` admits conversation only; every other grant is separate. `project:NAME` scopes a project.
pub const GRANTS: [&str; 5] = ["chat", "users", "projects", "taskq", "audit"];

fn valid_grant(g: &str) -> bool {
    GRANTS.contains(&g) || g.strip_prefix("project:").is_some_and(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)))
}

fn has(c: &Connection, user: i64, grant: &str) -> Result<bool, String> {
    c.query_row("SELECT 1 FROM grants WHERE user=?1 AND name=?2", params![user, grant], |_| Ok(())).optional().map(|r| r.is_some()).map_err(|e| e.to_string())
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

/// One Telegram message as the ingress sees it. Names are labels, never authority, so none are stored.
pub struct Input {
    pub chat: i64,
    pub topic: Option<i64>,
    pub sender: Option<i64>,
    pub message: i64,
    pub kind: &'static str,
    pub text: Option<String>,
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
}

/// What one send attempt established.
#[derive(Debug)]
pub enum Sent {
    Ok(i64),
    /// Telegram rejected the formatting: resend the same content as plain text.
    Plain,
    /// Retry after this many seconds; `ambiguous` when Telegram may already show the message.
    Retry { after: i64, ambiguous: bool, error: String },
    /// Destination unusable (forbidden, chat or topic gone): hold for an administrator, never reroute.
    Held(String),
    Failed(String),
}

pub struct Journal(Connection);

impl Journal {
    pub fn open(path: &Path) -> Result<Journal, String> {
        let c = Connection::open(path).map_err(|e| format!("journal {}: {e}", path.display()))?;
        let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if version > SCHEMA {
            return Err(format!("journal schema {version} is newer than {SCHEMA}; refusing to downgrade"));
        }
        c.execute_batch(&format!(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA busy_timeout=5000;
            CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS updates(update_id INTEGER PRIMARY KEY, disposition TEXT NOT NULL, request_id INTEGER, at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS requests(id INTEGER PRIMARY KEY, update_id INTEGER NOT NULL UNIQUE, chat INTEGER NOT NULL, topic INTEGER,
                sender INTEGER NOT NULL, message INTEGER NOT NULL, kind TEXT NOT NULL, text TEXT, state TEXT NOT NULL, received_at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS outbox(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, chat INTEGER NOT NULL, topic INTEGER, reply_to INTEGER,
                html TEXT, text TEXT NOT NULL, state TEXT NOT NULL DEFAULT 'pending', attempts INTEGER NOT NULL DEFAULT 0, next_at INTEGER NOT NULL,
                possible_duplicate INTEGER NOT NULL DEFAULT 0, sent_message INTEGER, error TEXT);
            CREATE TABLE IF NOT EXISTS health(source TEXT PRIMARY KEY, at INTEGER NOT NULL, state TEXT NOT NULL, reason TEXT);
            CREATE TABLE IF NOT EXISTS grants(user INTEGER NOT NULL, name TEXT NOT NULL, PRIMARY KEY(user, name));
            CREATE TABLE IF NOT EXISTS grant_log(id INTEGER PRIMARY KEY, at INTEGER NOT NULL, actor INTEGER, target INTEGER NOT NULL, before TEXT NOT NULL, after TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS routes(chat INTEGER NOT NULL, topic INTEGER NOT NULL, shared INTEGER NOT NULL, by INTEGER NOT NULL, at INTEGER NOT NULL,
                PRIMARY KEY(chat, topic));
            CREATE TABLE IF NOT EXISTS facts(subject INTEGER NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, private INTEGER NOT NULL, source TEXT NOT NULL,
                revision INTEGER NOT NULL, at INTEGER NOT NULL, PRIMARY KEY(subject, key));"
        ))
        .map_err(|e| format!("journal {}: {e}", path.display()))?;
        if version < 2 {
            c.execute_batch(&format!("ALTER TABLE requests ADD COLUMN session TEXT; PRAGMA user_version={SCHEMA};")).map_err(|e| format!("journal {}: {e}", path.display()))?;
        }
        Ok(Journal(c))
    }

    /// Offset for the next getUpdates call; it confirms everything already committed.
    pub fn cursor(&self) -> Result<i64, String> {
        self.0.query_row("SELECT value FROM meta WHERE key='cursor'", [], |r| r.get(0)).optional().map(Option::unwrap_or_default).map_err(|e| e.to_string())
    }

    /// Commits a getUpdates batch, its dispositions and the next cursor in one transaction, before Telegram is told to forget them.
    /// A known update ID is a duplicate and changes nothing. Only a sender holding `chat` is admitted; everyone else is ignored
    /// silently, and a later grant does not replay them. Returns the number of newly admitted requests.
    pub fn admit(&mut self, batch: &[(i64, Option<Input>)], at: i64) -> Result<usize, String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        let mut admitted = 0;
        for (update, input) in batch {
            let known: bool = tx.query_row("SELECT 1 FROM updates WHERE update_id=?1", [update], |_| Ok(true)).optional().map_err(|e| e.to_string())?.is_some();
            if known {
                continue;
            }
            let sender = match input.as_ref().and_then(|i| i.sender) {
                Some(s) if has(&tx, s, "chat")? => Some(s),
                _ => None,
            };
            let request = match (input, sender) {
                (Some(i), Some(sender)) => {
                    let shared: bool = tx
                        .query_row("SELECT shared FROM routes WHERE chat=?1 AND topic=?2", params![i.chat, i.topic.unwrap_or(0)], |r| r.get(0))
                        .optional()
                        .map_err(|e| e.to_string())?
                        .unwrap_or(false);
                    tx.execute(
                        "INSERT INTO requests(update_id, chat, topic, sender, message, kind, text, state, received_at, session) VALUES (?1,?2,?3,?4,?5,?6,?7,'queued',?8,?9)",
                        params![update, i.chat, i.topic, sender, i.message, i.kind, i.text, at, session(i.chat, i.topic, sender, shared)],
                    )
                    .map_err(|e| e.to_string())?;
                    let id = tx.last_insert_rowid();
                    // § 12: acknowledge the committed request and say it is queued
                    tx.execute(
                        "INSERT INTO outbox(request_id, chat, topic, reply_to, html, text, next_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                        params![id, i.chat, i.topic, i.message, format!("Received. Request <b>#{id}</b> is queued."), format!("Received. Request #{id} is queued."), at],
                    )
                    .map_err(|e| e.to_string())?;
                    admitted += 1;
                    Some(id)
                }
                _ => None,
            };
            // an ignored update keeps only its ID: no body, sender or chat (§ 2 People and admission)
            let disposition = if request.is_some() { "admitted" } else if input.is_some() { "ignored" } else { "unsupported" };
            tx.execute("INSERT INTO updates(update_id, disposition, request_id, at) VALUES (?1,?2,?3,?4)", params![update, disposition, request, at]).map_err(|e| e.to_string())?;
        }
        if let Some(last) = batch.iter().map(|(u, _)| *u).max() {
            tx.execute("INSERT INTO meta(key, value) VALUES ('cursor', ?1) ON CONFLICT(key) DO UPDATE SET value=max(value, excluded.value)", [last + 1])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(admitted)
    }

    /// The next delivery due at `now`, else the earliest time one becomes due. Only the oldest pending row of a route
    /// (chat and topic) is eligible, so a retrying head keeps later replies in that route behind it.
    pub fn due(&self, now: i64) -> Result<(Option<Delivery>, Option<i64>), String> {
        let heads = "FROM outbox o WHERE state='pending' AND NOT EXISTS
            (SELECT 1 FROM outbox p WHERE p.state='pending' AND p.chat=o.chat AND p.topic IS o.topic AND p.id<o.id)";
        let row = self
            .0
            .query_row(&format!("SELECT id, chat, topic, reply_to, html, text, attempts {heads} AND next_at<=?1 ORDER BY id LIMIT 1"), [now], |r| {
                Ok(Delivery { id: r.get(0)?, chat: r.get(1)?, topic: r.get(2)?, reply_to: r.get(3)?, html: r.get(4)?, text: r.get(5)?, attempts: r.get(6)? })
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let next = self.0.query_row(&format!("SELECT min(next_at) {heads}"), [], |r| r.get(0)).map_err(|e| e.to_string())?;
        Ok((row, next))
    }

    /// Records one send attempt. Retries stop at `ATTEMPTS`; the row stays for operator recovery, never deleted.
    pub fn record(&mut self, id: i64, sent: &Sent, now: i64) -> Result<(), String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        let attempts: i64 = tx.query_row("UPDATE outbox SET attempts=attempts+1 WHERE id=?1 RETURNING attempts", [id], |r| r.get(0)).map_err(|e| e.to_string())?;
        let (state, after, error, ambiguous, message) = match sent {
            Sent::Ok(m) => ("delivered", 0, None, false, Some(*m)),
            Sent::Plain => ("pending", 0, Some("formatting rejected; plain text".to_string()), false, None),
            Sent::Retry { after, ambiguous, error } => ("pending", *after, Some(error.clone()), *ambiguous, None),
            Sent::Held(e) => ("held", 0, Some(e.clone()), false, None),
            Sent::Failed(e) => ("failed", 0, Some(e.clone()), false, None),
        };
        let state = if state == "pending" && attempts >= ATTEMPTS { "exhausted" } else { state };
        tx.execute(
            "UPDATE outbox SET state=?2, next_at=?3, error=coalesce(?4, error), possible_duplicate=possible_duplicate OR ?5, sent_message=?6,
                html=CASE WHEN ?7 THEN NULL ELSE html END WHERE id=?1",
            params![id, state, now + after, error, ambiguous, message, matches!(sent, Sent::Plain)],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }

    /// The owner from protected configuration holds every named grant; a message can never make someone the owner (§ 2).
    pub fn bootstrap(&mut self, owner: i64, at: i64) -> Result<(), String> {
        self.change(None, owner, at, |g| g.extend(GRANTS.map(String::from))).map(drop)
    }

    pub fn grants(&self, user: i64) -> Result<Vec<String>, String> {
        grants_of(&self.0, user)
    }

    /// Adds grants to `target`; the actor needs `users`. Returns the target's grants after the change.
    pub fn grant(&mut self, actor: i64, target: i64, names: &[String], at: i64) -> Result<Vec<String>, String> {
        if let Some(g) = names.iter().find(|g| !valid_grant(g)) {
            return Err(format!("unknown grant {g}; use {} or project:NAME", GRANTS.join(", ")));
        }
        self.change(Some(actor), target, at, |g| g.extend(names.iter().cloned()))
    }

    /// Removes the named grants, or every grant when `names` is empty. Ignored from then on unless `chat` remains.
    pub fn revoke(&mut self, actor: i64, target: i64, names: &[String], at: i64) -> Result<Vec<String>, String> {
        self.change(Some(actor), target, at, |g| if names.is_empty() { g.clear() } else { g.retain(|x| !names.contains(x)) })
    }

    /// Applies one grant change and logs actor, target and before/after in the same transaction. `actor` None is protected config.
    fn change(&mut self, actor: Option<i64>, target: i64, at: i64, f: impl FnOnce(&mut std::collections::BTreeSet<String>)) -> Result<Vec<String>, String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        if let Some(a) = actor
            && !has(&tx, a, "users")?
        {
            return Err(format!("{a} has no users grant"));
        }
        let before = grants_of(&tx, target)?;
        let mut after: std::collections::BTreeSet<String> = before.iter().cloned().collect();
        f(&mut after);
        let after: Vec<String> = after.into_iter().collect();
        if after != before {
            tx.execute("DELETE FROM grants WHERE user=?1", [target]).map_err(|e| e.to_string())?;
            for g in &after {
                tx.execute("INSERT INTO grants(user, name) VALUES (?1,?2)", params![target, g]).map_err(|e| e.to_string())?;
            }
            tx.execute(
                "INSERT INTO grant_log(at, actor, target, before, after) VALUES (?1,?2,?3,?4,?5)",
                params![at, actor, target, before.join(","), after.join(",")],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(after)
    }

    /// Grant changes for `target`, oldest first: (at, actor, before, after); actor None is protected config.
    pub fn grant_log(&self, target: i64) -> Result<Vec<(i64, Option<i64>, String, String)>, String> {
        let mut q = self.0.prepare("SELECT at, actor, before, after FROM grant_log WHERE target=?1 ORDER BY id").map_err(|e| e.to_string())?;
        q.query_map([target], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).and_then(Iterator::collect).map_err(|e| e.to_string())
    }

    /// Every person with a grant or a remembered fact, with their grants.
    pub fn users(&self) -> Result<Vec<(i64, Vec<String>)>, String> {
        let mut q = self.0.prepare("SELECT user FROM grants UNION SELECT subject FROM facts ORDER BY 1").map_err(|e| e.to_string())?;
        let ids: Vec<i64> = q.query_map([], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())?;
        ids.into_iter().map(|u| Ok((u, grants_of(&self.0, u)?))).collect()
    }

    /// Remembers or corrects one fact; a correction bumps the revision. The subject manages their own facts; others need `users`.
    /// Facts never touch grants (§ 6).
    pub fn remember(&mut self, actor: i64, subject: i64, key: &str, value: &str, private: bool, at: i64) -> Result<Fact, String> {
        if actor != subject && !has(&self.0, actor, "users")? {
            return Err(format!("{actor} cannot change {subject}'s profile"));
        }
        self.0
            .query_row(
                "INSERT INTO facts(subject, key, value, private, source, revision, at) VALUES (?1,?2,?3,?4,?5,1,?6)
                ON CONFLICT(subject, key) DO UPDATE SET value=excluded.value, private=excluded.private, source=excluded.source, revision=revision+1, at=excluded.at
                RETURNING key, value, private, source, revision",
                params![subject, key, value, private, format!("user:{actor}"), at],
                |r| Ok(Fact { key: r.get(0)?, value: r.get(1)?, private: r.get(2)?, source: r.get(3)?, revision: r.get(4)? }),
            )
            .map_err(|e| e.to_string())
    }

    /// Deletes one fact. Returns false when there was none.
    pub fn forget(&mut self, actor: i64, subject: i64, key: &str) -> Result<bool, String> {
        if actor != subject && !has(&self.0, actor, "users")? {
            return Err(format!("{actor} cannot change {subject}'s profile"));
        }
        self.0.execute("DELETE FROM facts WHERE subject=?1 AND key=?2", params![subject, key]).map(|n| n > 0).map_err(|e| e.to_string())
    }

    /// The subject's facts visible in `viewer`, a chat ID or the CLI actor: private facts only in the subject's own DM or to the subject.
    /// Administration never reveals another person's private facts (§ 6).
    pub fn facts(&self, subject: i64, viewer: i64) -> Result<Vec<Fact>, String> {
        let mut q = self
            .0
            .prepare("SELECT key, value, private, source, revision FROM facts WHERE subject=?1 AND (private=0 OR ?1=?2) ORDER BY key")
            .map_err(|e| e.to_string())?;
        q.query_map([subject, viewer], |r| Ok(Fact { key: r.get(0)?, value: r.get(1)?, private: r.get(2)?, source: r.get(3)?, revision: r.get(4)? }))
            .and_then(Iterator::collect)
            .map_err(|e| e.to_string())
    }

    pub fn routes(&self) -> Result<Vec<Route>, String> {
        let mut q = self.0.prepare("SELECT chat, topic, shared, by, at FROM routes ORDER BY chat, topic").map_err(|e| e.to_string())?;
        q.query_map([], |r| Ok(Route { chat: r.get(0)?, topic: r.get(1)?, shared: r.get(2)?, by: r.get(3)?, at: r.get(4)? }))
            .and_then(Iterator::collect)
            .map_err(|e| e.to_string())
    }

    /// Sets a route's session policy; applies to requests admitted afterwards. The actor needs `users`.
    pub fn set_route(&mut self, actor: i64, chat: i64, topic: i64, shared: bool, at: i64) -> Result<Route, String> {
        if !has(&self.0, actor, "users")? {
            return Err(format!("{actor} has no users grant"));
        }
        self.0
            .query_row(
                "INSERT OR REPLACE INTO routes(chat, topic, shared, by, at) VALUES (?1,?2,?3,?4,?5) RETURNING chat, topic, shared, by, at",
                params![chat, topic, shared, actor, at],
                |r| Ok(Route { chat: r.get(0)?, topic: r.get(1)?, shared: r.get(2)?, by: r.get(3)?, at: r.get(4)? }),
            )
            .map_err(|e| e.to_string())
    }

    pub fn set_health(&self, source: &str, at: i64, state: &str, reason: Option<&str>) -> Result<(), String> {
        self.0
            .execute("INSERT OR REPLACE INTO health(source, at, state, reason) VALUES (?1,?2,?3,?4)", params![source, at, state, reason])
            .map(drop)
            .map_err(|e| e.to_string())
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
        s["undelivered"] = serde_json::json!({"count": count("SELECT count(*) FROM outbox WHERE state<>'delivered'")?});
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journal(tag: &str) -> (Journal, std::path::PathBuf) {
        let d = std::env::temp_dir().join(format!("pilotq-journal-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let mut j = Journal::open(&d.join("journal.sqlite")).unwrap();
        j.bootstrap(7, 0).unwrap();
        (j, d)
    }

    fn msg(chat: i64, sender: i64, message: i64) -> Option<Input> {
        Some(Input { chat, topic: None, sender: Some(sender), message, kind: "text", text: Some("hi".into()) })
    }

    fn count(j: &Journal, sql: &str) -> i64 {
        j.0.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    // F02, F15: a repeated update ID admits nothing new; a stranger leaves only an ID-only disposition; the cursor follows the batch.
    #[test]
    fn duplicates_admit_once_and_strangers_leave_no_body() {
        let (mut j, p) = journal("admit");
        assert_eq!(j.admit(&[(10, msg(1, 7, 100)), (10, msg(1, 7, 100)), (11, msg(1, 8, 101)), (12, None)], 0).unwrap(), 1);
        assert_eq!(j.admit(&[(10, msg(1, 7, 100))], 0).unwrap(), 0);
        assert_eq!(j.cursor().unwrap(), 13);
        assert_eq!(count(&j, "SELECT count(*) FROM requests"), 1);
        assert_eq!(count(&j, "SELECT count(*) FROM outbox"), 1);
        assert_eq!(count(&j, "SELECT count(*) FROM updates WHERE update_id=11 AND disposition='ignored' AND request_id IS NULL"), 1);
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    // F05: a route's later reply waits behind its retrying head; another route is not blocked; retries stop at ATTEMPTS.
    #[test]
    fn outbox_keeps_route_order_and_bounds_retries() {
        let (mut j, p) = journal("outbox");
        j.admit(&[(1, msg(1, 7, 100)), (2, msg(1, 7, 101)), (3, msg(2, 7, 102))], 0).unwrap();
        let retry = Sent::Retry { after: 10, ambiguous: true, error: "lost".into() };
        let (head, _) = j.due(0).unwrap();
        assert_eq!(head.as_ref().map(|d| d.id), Some(1));
        j.record(1, &retry, 0).unwrap();
        let (other, next) = j.due(0).unwrap();
        assert_eq!(other.map(|d| d.chat), Some(2), "route 2 proceeds; route 1's second reply waits");
        j.record(3, &Sent::Ok(55), 0).unwrap();
        assert_eq!(j.due(0).unwrap().0.map(|d| d.id), None);
        assert_eq!(next, Some(0));
        for _ in 1..ATTEMPTS {
            j.record(1, &retry, 0).unwrap();
        }
        assert_eq!(count(&j, "SELECT count(*) FROM outbox WHERE id=1 AND state='exhausted' AND possible_duplicate=1"), 1);
        assert_eq!(j.due(100).unwrap().0.map(|d| d.id), Some(2), "an exhausted head no longer blocks its route");
        j.record(2, &Sent::Plain, 100).unwrap();
        let plain = j.due(100).unwrap().0.unwrap();
        assert!(plain.html.is_none() && plain.id == 2, "formatting rejection resends plain text");
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    fn at(chat: i64, topic: Option<i64>, sender: i64, message: i64) -> Option<Input> {
        Some(Input { chat, topic, sender: Some(sender), message, kind: "text", text: Some("hi".into()) })
    }

    fn sessions(j: &Journal) -> Vec<(i64, String)> {
        let mut q = j.0.prepare("SELECT sender, session FROM requests ORDER BY id").unwrap();
        q.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect()
    }

    // U02: DM, an ordinary group left at the isolated default and a topic set to shared keep real authors and separate conversations.
    #[test]
    fn u02_route_session_policy() {
        let (mut j, p) = journal("u02");
        j.grant(7, 8, &["chat".into()], 0).unwrap();
        j.set_route(7, -100, 5, true, 0).unwrap();
        let batch = [(1, at(7, None, 7, 1)), (2, at(-50, None, 7, 2)), (3, at(-50, None, 8, 3)), (4, at(-100, Some(5), 7, 4)), (5, at(-100, Some(5), 8, 5)), (6, at(-100, Some(6), 8, 6))];
        assert_eq!(j.admit(&batch, 0).unwrap(), 6);
        let s = sessions(&j);
        assert_eq!(s[0], (7, "dm:7".into()));
        assert_eq!((&s[1].1, &s[2].1), (&"-50:0:7".into(), &"-50:0:8".into()), "ordinary group defaults to per person");
        assert_eq!((s[3].0, &s[3].1, s[4].0, &s[4].1), (7, &"-100:5".into(), 8, &"-100:5".into()), "shared topic keeps each author");
        assert_eq!(s[5].1, "-100:6:8", "another topic of the same forum is its own isolated route");
        assert!(j.set_route(8, -50, 0, true, 0).is_err(), "route policy needs the users grant");
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    // U17: two people's profiles stay distinct across reopen; a correction persists; a private DM fact is visible only in that
    // person's DM; a preference change leaves grants alone; administration does not reveal someone else's private fact.
    #[test]
    fn u17_profiles_distinct_and_private() {
        let (mut j, p) = journal("u17");
        j.grant(7, 8, &["chat".into()], 0).unwrap();
        j.remember(7, 7, "name", "Alex", false, 0).unwrap();
        j.remember(8, 8, "name", "Bo", false, 0).unwrap();
        j.remember(8, 8, "health", "private note", true, 0).unwrap();
        assert!(j.remember(8, 7, "name", "X", false, 0).is_err(), "a person without users cannot edit someone else");
        drop(j);
        let mut j = Journal::open(&p.join("journal.sqlite")).unwrap();
        let fixed = j.remember(8, 8, "name", "Bob", false, 1).unwrap();
        assert_eq!((fixed.value.as_str(), fixed.revision), ("Bob", 2));
        assert_eq!(j.grants(8).unwrap(), ["chat"], "a preference change does not change grants");
        let keys = |f: Vec<Fact>| f.into_iter().map(|f| format!("{}={}", f.key, f.value)).collect::<Vec<_>>();
        assert_eq!(keys(j.facts(8, 8).unwrap()), ["health=private note", "name=Bob"], "own DM sees the private fact");
        assert_eq!(keys(j.facts(8, -100).unwrap()), ["name=Bob"], "shared topic gets no private DM fact");
        assert_eq!(keys(j.facts(8, 7).unwrap()), ["name=Bob"], "an administrator does not see another person's private fact");
        assert_eq!(keys(j.facts(7, -100).unwrap()), ["name=Alex"]);
        assert!(j.forget(8, 8, "health").unwrap());
        assert!(j.facts(8, 8).unwrap().iter().all(|f| f.key != "health"));
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    // U18: an ignored person is not replayed after a grant; the grant is logged with exact IDs; chat alone holds no project grant;
    // only an administrator with users can grant; revoked input is ignored again.
    #[test]
    fn u18_grant_lifecycle() {
        let (mut j, p) = journal("u18");
        assert_eq!(j.admit(&[(1, msg(9, 9, 1))], 0).unwrap(), 0);
        assert!(j.grant(9, 9, &["chat".into()], 0).is_err(), "self-grant without users is refused");
        assert!(j.grant(7, 9, &["root".into()], 0).is_err(), "unknown grant names are refused");
        assert_eq!(j.grant(7, 9, &["chat".into()], 1).unwrap(), ["chat"]);
        assert_eq!(j.admit(&[(1, msg(9, 9, 1))], 2).unwrap(), 0, "the ignored update stays ignored");
        assert_eq!(j.admit(&[(2, msg(9, 9, 2))], 2).unwrap(), 1, "a new request is admitted");
        assert!(!has(&j.0, 9, "projects").unwrap(), "conversation access is not project access");
        assert_eq!(j.grant(7, 9, &["project:pilotq".into()], 3).unwrap(), ["chat", "project:pilotq"]);
        assert_eq!(j.revoke(7, 9, &["project:pilotq".into()], 4).unwrap(), ["chat"]);
        assert_eq!(j.revoke(7, 9, &[], 5).unwrap(), Vec::<String>::new());
        assert_eq!(j.admit(&[(3, msg(9, 9, 3))], 6).unwrap(), 0, "revoked input is ignored");
        let log = j.grant_log(9).unwrap();
        assert_eq!(log.len(), 4);
        assert_eq!(log[0], (1, Some(7), String::new(), "chat".into()));
        assert_eq!(log[3], (5, Some(7), "chat".into(), String::new()));
        assert_eq!(j.grant_log(7).unwrap()[0].1, None, "the owner comes from protected config");
        j.bootstrap(7, 9).unwrap();
        assert_eq!(j.grant_log(7).unwrap().len(), 1, "an unchanged bootstrap logs nothing");
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    // A schema 1 journal from the previous release gains the session column and keeps working.
    #[test]
    fn schema_1_journal_upgrades() {
        let (j, p) = journal("upgrade");
        j.0.execute_batch("ALTER TABLE requests DROP COLUMN session; PRAGMA user_version=1").unwrap();
        drop(j);
        let mut j = Journal::open(&p.join("journal.sqlite")).unwrap();
        assert_eq!(j.admit(&[(1, msg(7, 7, 1))], 0).unwrap(), 1);
        assert_eq!(sessions(&j), [(7, "dm:7".to_string())]);
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    // F08: a journal from a newer release is refused, not migrated down.
    #[test]
    fn newer_schema_is_refused() {
        let (j, p) = journal("schema");
        j.0.execute_batch("PRAGMA user_version=99").unwrap();
        drop(j);
        assert!(Journal::open(&p.join("journal.sqlite")).err().unwrap().contains("newer"));
        let _ = std::fs::remove_dir_all(p);
    }
}
