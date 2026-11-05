//! Journal creation and the supported schema-13 upgrade (§9).
use super::*;

impl Journal {
    pub fn open(path: &Path) -> Result<Journal, String> {
        let c = Connection::open(path).map_err(|e| format!("journal {}: {e}", path.display()))?;
        c.busy_timeout(std::time::Duration::from_secs(30)).map_err(|e| e.to_string())?;
        let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if version > SCHEMA {
            return Err(format!("journal schema {version} is newer than {SCHEMA}; refusing to downgrade"));
        }
        let empty: bool = c.query_row("SELECT NOT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table')", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if version < 13 && !(version == 0 && empty) {
            return Err(format!("journal schema {version} is older than 13; use a compatible earlier build to migrate it first"));
        }
        let up = |sql: &str| c.execute_batch(sql).map_err(|e| format!("journal {}: {e}", path.display()));
        up("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; BEGIN IMMEDIATE;")?;
        // Re-read under the writer lock: another opener may have completed creation or the upgrade.
        let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if version == 0 { up(include_str!("schema.sql"))?; }
        if version == 13 {
            // #359 A2: the runtime owning a conversation's native session and the runtime of each run are columns. Legacy
            // sessions were `RUNTIME:ID` (#265) or a bare ID of the default runtime of that time.
            // legacy fixtures built from the current schema already hold the columns
            let has = |t: &str| c.query_row(&format!("SELECT EXISTS(SELECT 1 FROM pragma_table_info('{t}') WHERE name='runtime')"),[],|r|r.get::<_,bool>(0)).map_err(|e|e.to_string());
            if !has("conversations")? { up("ALTER TABLE conversations ADD COLUMN runtime TEXT;")?; }
            if !has("runs")? { up("ALTER TABLE runs ADD COLUMN runtime TEXT;")?; }
            up("UPDATE conversations SET runtime=CASE WHEN instr(thread,':')>0 THEN substr(thread,1,instr(thread,':')-1) ELSE 'codex' END,
                    thread=substr(thread,instr(thread,':')+1) WHERE thread IS NOT NULL;
                PRAGMA user_version=14;")?;
        }
        // §4 / #95: native sessions keep their original policy. A changed shared pin or
        // PilotQ addition rebuilds from journal history and receipts, never by replaying effects.
        let policy = crate::instructions::fingerprint();
        let previous: Option<i64> = c.query_row("SELECT value FROM meta WHERE key='instructions'", [], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
        if previous != Some(policy) {
            up("INSERT OR IGNORE INTO meta(key,value) SELECT 'refresh:' || session,1 FROM conversations WHERE thread IS NOT NULL;")?;
            c.execute("INSERT INTO meta(key,value) VALUES ('instructions',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [policy]).map_err(|e| e.to_string())?;
        }
        up("COMMIT;")?;
        // TaskQ op IDs are permanent board claims; a random installation ID keeps a fresh journal from reusing another's
        let salt = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as i64) ^ ((std::process::id() as i64) << 40);
        c.execute("INSERT INTO meta(key, value) VALUES ('instance', ?1) ON CONFLICT(key) DO NOTHING", [salt & i64::MAX]).map_err(|e| e.to_string())?;
        up("CREATE TEMP TABLE previews(run INTEGER PRIMARY KEY, text TEXT NOT NULL);")?;
        // In-memory test journals have no parent directory; keep their telemetry store in temp_dir instead of the worktree root (#320)
        let tel = if path == Path::new(":memory:") { std::env::temp_dir().join(format!("pilotq-mem-{}-telemetry.sqlite", std::process::id())) } else { path.with_file_name("telemetry.sqlite") };
        Ok(Journal(c, tel, std::sync::OnceLock::new(), std::collections::HashMap::new()))
    }

}
