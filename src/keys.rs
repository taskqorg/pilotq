//! Key store (pilotq.md § 8 Key store): named secrets in one owner-only file. A value is stored once, injected into a tool's
//! child process at dispatch and never returned: not to the model, the journal, telemetry, logs or replies. Keys enter only
//! through the CLI (owner decision 2026-10-23).

use serde_json::{Map, Value, json};
use std::path::PathBuf;

/// Tools that may name a key (§ 5 Direct actions).
pub const TOOLS: [&str; 3] = ["issue_read", "issue_comment", "issue_label"];

pub struct Keys(pub PathBuf);

/// A key's readback: everything but its value.
fn entry(name: &str, k: &Value) -> Value {
    json!({"name": name, "purpose": k["purpose"], "tools": k["tools"], "users": k["users"], "routes": k["routes"], "last_use": k["last_use"]})
}

impl Keys {
    fn load(&self) -> Result<Map<String, Value>, String> {
        let text = match std::fs::read_to_string(&self.0) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
            Err(e) => return Err(format!("key store: {e}")),
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&self.0).map_err(|e| format!("key store: {e}"))?.permissions().mode();
            if mode & 0o077 != 0 {
                return Err(format!("{} must be owner-only (chmod 600)", self.0.display()));
            }
        }
        // the parse error would quote the file, so it is not passed on
        match serde_json::from_str(&text) {
            Ok(Value::Object(m)) => Ok(m),
            _ => Err(format!("{} is malformed", self.0.display())),
        }
    }

    /// Read, change and write the store under an exclusive lock, so the CLI and the service never lose each other's change.
    fn change<T>(&self, f: impl FnOnce(&mut Map<String, Value>) -> Result<T, String>) -> Result<T, String> {
        let lock = std::fs::File::create(self.0.with_extension("lock")).map_err(|e| format!("key store lock: {e}"))?;
        lock.lock().map_err(|e| format!("key store lock: {e}"))?;
        let mut m = self.load()?;
        let out = f(&mut m)?;
        let tmp = self.0.with_extension("tmp");
        let _ = std::fs::remove_file(&tmp);
        let mut o = std::fs::OpenOptions::new();
        o.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
        let mut file = o.open(&tmp).map_err(|e| format!("key store: {e}"))?;
        std::io::Write::write_all(&mut file, Value::Object(m).to_string().as_bytes()).and_then(|_| file.sync_all()).map_err(|e| format!("key store: {e}"))?;
        std::fs::rename(&tmp, &self.0).map_err(|e| format!("key store: {e}"))?;
        Ok(out)
    }

    /// Stores a new key. An existing name is refused: a value is stored once, and replacing one starts with `revoke`.
    pub fn add(&self, name: &str, purpose: &str, tools: &[String], value: &str) -> Result<Value, String> {
        if name.is_empty() || name.len() > 32 || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-_".contains(c)) {
            return Err(format!("{name}: a key name is 1-32 characters a-z, 0-9, - or _"));
        }
        if purpose.trim().is_empty() || tools.is_empty() || tools.iter().any(|t| !TOOLS.contains(&t.as_str())) {
            return Err(format!("a key needs a purpose and at least one of the tools {}", TOOLS.join(", ")));
        }
        // ponytail: scrubbing replaces the whole value, so a short value would rewrite ordinary output
        if value.chars().count() < 8 || value.chars().any(char::is_whitespace) {
            return Err("the value on stdin must be at least 8 characters without whitespace".into());
        }
        self.change(|m| {
            if m.contains_key(name) {
                return Err(format!("key {name} exists; revoke it first to replace its value"));
            }
            let k = json!({"purpose": purpose, "value": value, "tools": tools, "users": [], "routes": [], "last_use": null});
            let e = entry(name, &k);
            m.insert(name.into(), k);
            Ok(e)
        })
    }

    /// Adds holders: a Telegram user ID, or `route:CHAT[:TOPIC]` / `route:cli`.
    pub fn grant(&self, name: &str, who: &[String]) -> Result<Value, String> {
        let mut parsed = vec![];
        for w in who {
            parsed.push(match w.strip_prefix("route:") {
                Some(r) if r == "cli" || r.split(':').count() <= 2 && r.split(':').all(|p| p.parse::<i64>().is_ok()) => ("routes", json!(r)),
                None => ("users", json!(w.parse::<i64>().map_err(|_| format!("{w}: expected a Telegram user ID or route:CHAT[:TOPIC]"))?)),
                _ => return Err(format!("{w}: expected route:CHAT, route:CHAT:TOPIC or route:cli")),
            });
        }
        self.change(|m| {
            let k = m.get_mut(name).ok_or(format!("no key {name}"))?;
            for (field, v) in parsed {
                let list = k[field].as_array_mut().ok_or("key store is malformed")?;
                if !list.contains(&v) {
                    list.push(v);
                }
            }
            Ok(entry(name, k))
        })
    }

    /// Removes holders, or the whole key with its value when `who` is empty.
    pub fn revoke(&self, name: &str, who: &[String]) -> Result<Value, String> {
        self.change(|m| {
            if who.is_empty() {
                m.remove(name).ok_or(format!("no key {name}"))?;
                return Ok(json!({"name": name, "removed": true}));
            }
            let k = m.get_mut(name).ok_or(format!("no key {name}"))?;
            for w in who {
                let (field, v) = match w.strip_prefix("route:") {
                    Some(r) => ("routes", json!(r)),
                    None => ("users", w.parse::<i64>().map(|n| json!(n)).unwrap_or(json!(w))),
                };
                if let Some(list) = k[field].as_array_mut() {
                    list.retain(|x| *x != v);
                }
            }
            Ok(entry(name, k))
        })
    }

    pub fn list(&self) -> Result<Value, String> {
        Ok(self.load()?.iter().map(|(n, k)| entry(n, k)).collect())
    }

    /// The value of the key a call may use, checked at dispatch (§ 8): the key allows `tool` and names `user` or `route`.
    /// `name` None picks the only such key. Records the use time; the value goes only into the tool's child process.
    pub fn take(&self, name: Option<&str>, tool: &str, user: i64, route: &str, at: i64) -> Result<(String, String), String> {
        self.change(|m| {
            let fits = |k: &Value| {
                k["tools"].as_array().is_some_and(|t| t.iter().any(|x| x == tool))
                    && (k["users"].as_array().is_some_and(|u| u.contains(&json!(user))) || k["routes"].as_array().is_some_and(|r| r.contains(&json!(route))))
            };
            let usable: Vec<String> = m.iter().filter(|(_, k)| fits(k)).map(|(n, _)| n.clone()).collect();
            let name = match name {
                Some(n) if usable.iter().any(|u| u == n) => n.to_string(),
                None if usable.len() == 1 => usable[0].clone(),
                _ => {
                    let list = if usable.is_empty() { "none".to_string() } else { usable.join(", ") };
                    return Err(format!("no key {}granted to {user} for {tool}; keys you may use for it: {list}. Nothing was done", name.map_or(String::new(), |n| format!("{n} "))));
                }
            };
            let k = m.get_mut(&name).ok_or("key store is malformed")?;
            k["last_use"] = json!(at);
            Ok((name, k["value"].as_str().ok_or("key store is malformed")?.to_string()))
        })
    }

    /// The value of a stored key, for the restart readback of an effect it already authorized.
    pub fn value(&self, name: &str) -> Result<String, String> {
        self.load()?.get(name).and_then(|k| k["value"].as_str()).map(str::to_string).ok_or(format!("key {name} is gone"))
    }

    /// Replaces every stored value in `text` with `[key NAME]`. An unreadable store withholds the text instead (§ 8: tool output
    /// is scrubbed before the model or Telegram sees it).
    pub fn scrub(&self, text: &str) -> String {
        match self.load() {
            Ok(m) => m.iter().fold(text.to_string(), |t, (n, k)| match k["value"].as_str() {
                Some(v) if !v.is_empty() => t.replace(v, &format!("[key {n}]")),
                _ => t,
            }),
            Err(e) => format!("Tool output withheld: {e}"),
        }
    }
}
