//! SQLite journal: admitted inputs, ignored-update dispositions, the polling cursor, the outbox (pilotq.md § 4, § 7),
//! and the grant registry, route policy and per-person profiles (§ 2, § 6).

use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// Schema this binary writes; a newer journal is refused rather than guessed (F08).
const SCHEMA: i64 = 3;
/// Model attempts per request across restarts (§ 7 dependency retry).
const RUNS: i64 = 3;
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

/// Recovery control in a message (§ 2 Changing direction), optionally addressed as `/stop@bot`.
fn control(text: Option<&str>) -> Option<&'static str> {
    let cmd = text?.split_whitespace().next()?.split('@').next()?;
    ["/stop", "/status", "/new"].into_iter().find(|c| *c == cmd)
}

#[allow(clippy::too_many_arguments)]
fn insert(tx: &Connection, update: i64, front: &str, chat: i64, topic: Option<i64>, sender: i64, message: i64, kind: &str, text: Option<&str>, state: &str, at: i64, session: &str) -> Result<i64, String> {
    tx.execute(
        "INSERT INTO requests(update_id, front, chat, topic, sender, message, kind, text, state, received_at, session) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![update, front, chat, topic, sender, message, kind, text, state, at, session],
    )
    .map_err(|e| e.to_string())?;
    Ok(tx.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
fn outbox(tx: &Connection, request: i64, chat: i64, topic: Option<i64>, reply_to: Option<i64>, html: Option<&str>, text: &str, at: i64) -> Result<(), String> {
    tx.execute("INSERT INTO outbox(request_id, chat, topic, reply_to, html, text, next_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![request, chat, topic, reply_to, html, text, at])
        .map(drop)
        .map_err(|e| e.to_string())
}

fn ids(tx: &Connection, session: &str, state: &str) -> Result<Vec<String>, String> {
    let mut q = tx.prepare("SELECT '#' || id FROM requests WHERE session=?1 AND state=?2 ORDER BY id").map_err(|e| e.to_string())?;
    q.query_map(params![session, state], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())
}

/// Answers `/stop`, `/status` or `/new` inside the admission transaction, without the model and scoped to one conversation (§ 12).
fn answer_control(tx: &Connection, session: &str, cmd: &str) -> Result<String, String> {
    let e = |e: rusqlite::Error| e.to_string();
    Ok(match cmd {
        "/stop" => {
            let running: Option<i64> = tx.query_row("SELECT id FROM requests WHERE session=?1 AND state='running'", [session], |r| r.get(0)).optional().map_err(e)?;
            let Some(id) = running else { return Ok("Nothing is running in this conversation.".into()) };
            // the fence bump is the verified stop: the old run can no longer record a reply or effect
            tx.execute("UPDATE requests SET state='cancelled', error='stopped by /stop' WHERE id=?1", [id]).map_err(e)?;
            tx.execute("UPDATE conversations SET fence=fence+1 WHERE session=?1", [session]).map_err(e)?;
            format!("Stopped request #{id}. Queued follow-ups stay queued.")
        }
        "/status" => {
            let (running, queued) = (ids(tx, session, "running")?, ids(tx, session, "queued")?);
            if running.is_empty() && queued.is_empty() {
                "Nothing is running or queued in this conversation.".into()
            } else {
                let list = |v: &[String]| if v.is_empty() { "none".to_string() } else { v.join(", ") };
                format!("Running: {}. Queued: {}.", list(&running), list(&queued))
            }
        }
        _ => {
            tx.execute("INSERT INTO conversations(session, generation) VALUES (?1, 1) ON CONFLICT(session) DO UPDATE SET generation=generation+1, thread=NULL", [session])
                .map_err(e)?;
            "New conversation started. Earlier requests, task links and receipts stay recorded.".into()
        }
    })
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
    pub text: String,
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
    let (generation, thread, fence) = tx
        .query_row("UPDATE conversations SET fence=fence+1 WHERE session=?1 RETURNING generation, thread, fence", [&session], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(e)?;
    tx.execute("INSERT INTO runs(request_id, session, generation, fence, state, started_at) VALUES (?1,?2,?3,?4,'running',?5)", params![request, session, generation, fence, at])
        .map_err(e)?;
    Ok(Some(Run { id: tx.last_insert_rowid(), request, session, generation, fence, thread, text }))
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
        let up = |sql: &str| c.execute_batch(sql).map_err(|e| format!("journal {}: {e}", path.display()));
        if version < 2 {
            up("ALTER TABLE requests ADD COLUMN session TEXT;")?;
        }
        if version < 3 {
            // § 4 Runs: request and run identity, the conversation's native thread and the fence that retires a stopped run
            up("ALTER TABLE requests ADD COLUMN front TEXT NOT NULL DEFAULT 'telegram';
                ALTER TABLE requests ADD COLUMN reply TEXT;
                ALTER TABLE requests ADD COLUMN error TEXT;
                CREATE TABLE conversations(session TEXT PRIMARY KEY, generation INTEGER NOT NULL DEFAULT 0, thread TEXT, fence INTEGER NOT NULL DEFAULT 0);
                CREATE TABLE runs(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, session TEXT NOT NULL, generation INTEGER NOT NULL, fence INTEGER NOT NULL,
                    state TEXT NOT NULL, tool INTEGER NOT NULL DEFAULT 0, started_at INTEGER NOT NULL, ended_at INTEGER, outcome TEXT);")?;
        }
        up(&format!("PRAGMA user_version={SCHEMA};"))?;
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
                    let s = session(i.chat, i.topic, sender, shared);
                    let cmd = control(i.text.as_deref());
                    let (kind, state) = match (cmd, i.kind) {
                        (Some(_), _) => ("control", "answered"),
                        (None, "text") => ("text", "queued"),
                        _ => (i.kind, "failed"),
                    };
                    let id = insert(&tx, *update, "telegram", i.chat, i.topic, sender, i.message, kind, i.text.as_deref(), state, at, &s)?;
                    let (html, text) = match cmd {
                        Some(c) => (None, answer_control(&tx, &s, c)?),
                        // § 1: unsupported input is detected and gets an explicit outcome, never silence
                        None if state == "failed" => (None, format!("Request #{id}: this input type is not supported yet; send text.")),
                        // § 12: acknowledge the committed request and say it is queued
                        None => (Some(format!("Received. Request <b>#{id}</b> is queued.")), format!("Received. Request #{id} is queued.")),
                    };
                    if state != "queued" {
                        tx.execute("UPDATE requests SET reply=?2 WHERE id=?1", params![id, text]).map_err(|e| e.to_string())?;
                    }
                    outbox(&tx, id, i.chat, i.topic, Some(i.message), html.as_deref(), &text, at)?;
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

    /// Admits and claims one `pilotq ask` request through the same grant check and request record as Telegram (R13, U19).
    pub fn ask(&mut self, sender: i64, text: &str, at: i64) -> Result<Run, String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        if !has(&tx, sender, "chat")? {
            return Err(format!("{sender} has no chat grant"));
        }
        // ponytail: CLI requests take negative synthetic update IDs, so Telegram's positive IDs never collide
        let update: i64 = tx.query_row("SELECT min(coalesce(min(update_id), 0), 0) - 1 FROM requests", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        let id = insert(&tx, update, "cli", sender, None, sender, 0, "text", Some(text), "queued", at, &format!("cli:{sender}"))?;
        let run = claim(&tx, id, at)?.ok_or("request was claimed twice")?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(run)
    }

    /// Claims the oldest queued Telegram request of each conversation without a running one, up to `limit` running in total.
    pub fn claim_next(&mut self, limit: usize, at: i64) -> Result<Vec<Run>, String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        let running: i64 = tx.query_row("SELECT count(*) FROM requests WHERE state='running' AND front='telegram'", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        let free = (limit as i64 - running).max(0);
        let ids: Vec<i64> = {
            let mut q = tx
                .prepare(
                    "SELECT min(id) FROM requests r WHERE state='queued' AND front='telegram'
                    AND NOT EXISTS (SELECT 1 FROM requests p WHERE p.session=r.session AND p.state='running') GROUP BY session ORDER BY 1 LIMIT ?1",
                )
                .map_err(|e| e.to_string())?;
            q.query_map([free], |r| r.get(0)).and_then(Iterator::collect).map_err(|e| e.to_string())?
        };
        let mut runs = vec![];
        for id in ids {
            runs.extend(claim(&tx, id, at)?);
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(runs)
    }

    /// True when a stop or a later claim has retired this run's fence.
    pub fn fenced(&self, r: &Run) -> Result<bool, String> {
        self.0.query_row("SELECT fence<>?2 FROM conversations WHERE session=?1", params![r.session, r.fence], |x| x.get(0)).map_err(|e| e.to_string())
    }

    /// Persists that the run started a tool item; a lost terminal outcome then reads uncertain, never failed-and-retryable.
    pub fn tool_started(&self, run: i64) -> Result<(), String> {
        self.0.execute("UPDATE runs SET tool=1 WHERE id=?1", [run]).map(drop).map_err(|e| e.to_string())
    }

    /// Records a run's outcome, reply and native thread in one transaction, only while its fence holds. Returns false for a
    /// fenced run, which changes nothing but its own run row.
    pub fn finish(&self, r: &Run, thread: Option<&str>, outcome: &crate::manager::Outcome, at: i64) -> Result<bool, String> {
        use crate::manager::Outcome;
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.unchecked_transaction().map_err(e)?;
        let (state, text, error) = match outcome {
            _ if self.fenced(r)? => ("fenced", String::new(), None),
            Outcome::Fenced => ("fenced", String::new(), None),
            Outcome::Answered(a) => ("answered", a.clone(), None),
            Outcome::Failed(x) => ("failed", format!("Request #{} failed: {x}", r.request), Some(x)),
            Outcome::Uncertain(x) => ("uncertain", format!("Request #{} has an uncertain outcome and will not be repeated automatically: {x}", r.request), Some(x)),
        };
        tx.execute("UPDATE runs SET state=?2, ended_at=?3, outcome=?4 WHERE id=?1", params![r.id, state, at, error]).map_err(e)?;
        if state == "fenced" {
            tx.commit().map_err(e)?;
            return Ok(false);
        }
        tx.execute("UPDATE requests SET state=?2, reply=?3, error=?4 WHERE id=?1", params![r.request, state, text, error]).map_err(e)?;
        // a thread from before /new never attaches to the new conversation generation
        tx.execute("UPDATE conversations SET thread=coalesce(?3, thread) WHERE session=?1 AND generation=?2", params![r.session, r.generation, thread]).map_err(e)?;
        let dest: (String, i64, Option<i64>, i64) =
            tx.query_row("SELECT front, chat, topic, message FROM requests WHERE id=?1", [r.request], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?))).map_err(e)?;
        if dest.0 == "telegram" {
            outbox(&tx, r.request, dest.1, dest.2, Some(dest.3), None, &text, at)?;
        }
        tx.commit().map_err(e)?;
        Ok(true)
    }

    /// Startup recovery (§ 4, § 12): a run left running by a dead process is interrupted. Its request is requeued while model
    /// attempts remain, uncertain once a tool started, and failed for a CLI caller that is gone. Returns the requests changed.
    pub fn reconcile(&mut self, at: i64) -> Result<usize, String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.transaction().map_err(e)?;
        let stale: Vec<(i64, bool, String, i64)> = {
            let mut q = tx
                .prepare(
                    "SELECT r.request_id, r.tool, q.front, (SELECT count(*) FROM runs x WHERE x.request_id=r.request_id)
                    FROM runs r JOIN requests q ON q.id=r.request_id WHERE r.state='running' AND q.state='running'",
                )
                .map_err(e)?;
            q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).and_then(Iterator::collect).map_err(e)?
        };
        for (request, tool, front, attempts) in &stale {
            let (state, why) = match (tool, front.as_str()) {
                (true, _) => ("uncertain", "PilotQ restarted after a tool started"),
                (false, "telegram") if *attempts < RUNS => ("queued", ""),
                _ => ("failed", "PilotQ restarted during the run"),
            };
            if state == "queued" {
                tx.execute("UPDATE requests SET state='queued' WHERE id=?1", [request]).map_err(e)?;
                continue;
            }
            let text = format!("Request #{request} {}: {why}.", if state == "failed" { "failed" } else { "has an uncertain outcome and will not be repeated automatically" });
            tx.execute("UPDATE requests SET state=?2, error=?3, reply=?4 WHERE id=?1", params![request, state, why, text]).map_err(e)?;
            let dest: (i64, Option<i64>, i64) = tx.query_row("SELECT chat, topic, message FROM requests WHERE id=?1", [request], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?))).map_err(e)?;
            if front == "telegram" {
                outbox(&tx, *request, dest.0, dest.1, Some(dest.2), None, &text, at)?;
            }
        }
        tx.execute("UPDATE runs SET state='interrupted', ended_at=?1 WHERE state='running'", [at]).map_err(e)?;
        tx.commit().map_err(e)?;
        Ok(stale.len())
    }

    /// A request's state, reply and error, for the CLI readback.
    pub fn request(&self, id: i64) -> Result<(String, Option<String>, Option<String>), String> {
        self.0.query_row("SELECT state, reply, error FROM requests WHERE id=?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(|e| e.to_string())
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
        s["uncertain"] = serde_json::json!({"count": count("SELECT count(*) FROM requests WHERE state='uncertain'")?});
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

    // A schema 1 journal from an earlier release gains the session column and the schema 3 run tables and keeps working.
    #[test]
    fn schema_1_journal_upgrades() {
        let (j, p) = journal("upgrade");
        j.0.execute_batch(
            "ALTER TABLE requests DROP COLUMN session; ALTER TABLE requests DROP COLUMN front; ALTER TABLE requests DROP COLUMN reply;
            ALTER TABLE requests DROP COLUMN error; DROP TABLE conversations; DROP TABLE runs; PRAGMA user_version=1",
        )
        .unwrap();
        drop(j);
        let mut j = Journal::open(&p.join("journal.sqlite")).unwrap();
        assert_eq!(j.admit(&[(1, msg(7, 7, 1))], 0).unwrap(), 1);
        assert_eq!(sessions(&j), [(7, "dm:7".to_string())]);
        drop(j);
        let _ = std::fs::remove_dir_all(p);
    }

    // F03, F04, U12: a restart requeues a pure model run within its attempt budget, never one that started a tool; a fenced
    // run records nothing on its request; /new keeps the old thread off the new conversation generation.
    #[test]
    fn runs_reconcile_and_fence() {
        use crate::manager::Outcome;
        let (mut j, p) = journal("runs");
        j.admit(&[(1, msg(7, 7, 1)), (2, msg(-5, 7, 2))], 0).unwrap();
        let runs = j.claim_next(4, 0).unwrap();
        assert_eq!(runs.len(), 2);
        j.tool_started(runs[1].id).unwrap();
        assert_eq!(j.reconcile(1).unwrap(), 2);
        assert_eq!(j.request(1).unwrap().0, "queued", "a pure model run is retried");
        assert_eq!(j.request(2).unwrap().0, "uncertain", "a run that started a tool is never replayed");
        for _ in 1..RUNS {
            j.claim_next(4, 2).unwrap();
            j.reconcile(3).unwrap();
        }
        assert_eq!(j.request(1).unwrap().0, "failed", "model attempts are bounded across restarts");

        j.admit(&[(3, msg(7, 7, 3))], 4).unwrap();
        let r = j.claim_next(4, 4).unwrap().remove(0);
        j.admit(&[(4, Some(Input { chat: 7, topic: None, sender: Some(7), message: 4, kind: "text", text: Some("/stop".into()) }))], 5).unwrap();
        assert!(j.fenced(&r).unwrap());
        assert!(!j.finish(&r, Some("t-old"), &Outcome::Answered("late".into()), 6).unwrap(), "a stopped run cannot reply");
        assert_eq!(j.request(r.request).unwrap(), ("cancelled".into(), None, Some("stopped by /stop".into())));

        j.admit(&[(5, msg(7, 7, 5))], 7).unwrap();
        let r = j.claim_next(4, 7).unwrap().remove(0);
        j.admit(&[(6, Some(Input { chat: 7, topic: None, sender: Some(7), message: 6, kind: "text", text: Some("/new".into()) }))], 8).unwrap();
        assert!(j.finish(&r, Some("t-old"), &Outcome::Answered("ok".into()), 9).unwrap());
        assert_eq!(count(&j, "SELECT count(*) FROM conversations WHERE session='dm:7' AND thread IS NULL AND generation=1"), 1);
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
