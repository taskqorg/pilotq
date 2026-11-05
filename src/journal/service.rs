//! Service persistence (pilotq.md §§2,4–9).
use super::*;

impl Journal {
    /// §9 / #77: one durable owner incident per version pair, with the ordinary send receipt.
    /// Returns true when this call opened the incident.
    pub fn rollback_notice(&mut self, owner: i64, failed: &str, restored: &str, at: i64) -> Result<bool, String> {
        let valid = |v: &str| !v.is_empty() && v.len() <= 64 && !v.contains("..") && v.bytes().all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b));
        if !valid(failed) || !valid(restored) || failed == restored { return Err("invalid rollback versions".into()); }
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        let key = format!("rollback:{failed}:{restored}");
        let added = tx.execute("INSERT OR IGNORE INTO meta(key,value) VALUES (?1,?2)", params![key, at]).map_err(|e| e.to_string())? != 0;
        if added {
            let ru=current_reply_language(&tx,"telegram",owner,owner)?.as_deref()==Some("ru");
            let text=if ru {format!("Обновление PilotQ не удалось: {failed}; восстановлена версия {restored}.")}else{format!("PilotQ update failed: {failed}; restored {restored}.")};
            service_message(&tx,owner,None,&text,at)?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(added)
    }

    /// §9 / #81 N1: opens an owner incident for `model`, `taskq` or `poll` after the hold-down and closes it on the next
    /// successful observation. The open key lives in the journal, so a restart neither reopens nor forgets an episode.
    pub fn incidents(&mut self, owner: i64, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.transaction().map_err(e)?;
        let ru=current_reply_language(&tx,"telegram",owner,owner)?.as_deref()==Some("ru");
        for (source, open_en, close_en, open_ru, close_ru) in INCIDENTS {
            let (open,close)=if ru {(open_ru,close_ru)}else{(open_en,close_en)};
            let state: Option<String> = tx.query_row("SELECT state FROM health WHERE source=?1", [source], |r| r.get(0)).optional().map_err(e)?;
            let key = format!("incident:{source}");
            let opened = meta(&tx, &key)?.is_some();
            match state.as_deref() {
                // set_health writes ok only on a fresh success, and an open incident saw this source failed, so ok here is newer
                Some("ok") if opened => {
                    if meta(&tx, &format!("notified:{source}"))?.is_some() {
                        service_message(&tx, owner, None, close, at)?;
                    }
                    tx.execute("DELETE FROM meta WHERE key IN (?1, ?2)", [key, format!("notified:{source}")]).map_err(e)?;
                }
                Some(s) if s != "ok" && !opened => {
                    let held = if source == "model" {
                        meta(&tx, "failures:model")?.unwrap_or(0) >= 2
                    } else {
                        meta(&tx, &format!("failing:{source}"))?.is_some_and(|since| at - since >= HOLD)
                    };
                    if held {
                        tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)", params![key, at]).map_err(e)?;
                        if service_message(&tx, owner, Some(&key), open, at)? {
                            tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)", params![format!("notified:{source}"), at]).map_err(e)?;
                        }
                    }
                }
                _ => {}
            }
        }
        let held:Vec<(String,String)> = {
            let mut q=tx.prepare("SELECT source,coalesce(reason,'route unavailable') FROM health WHERE source LIKE 'route-hold:%' AND state='failed' AND at<=?1").map_err(e)?;
            q.query_map([at-HOLD],|r|Ok((r.get(0)?,r.get(1)?))).and_then(Iterator::collect).map_err(e)?
        };
        for (source,reason) in held {
            let key=format!("notified:{source}");
            if meta(&tx,&key)?.is_none() && service_message(&tx,owner,Some(&source),&if ru {format!("Маршрут Telegram недоступен ({source}). Вывод остаётся в исходном чате; администратор должен повторно проверить доступ. Причина сохранена в аудите.")}else{format!("PilotQ Telegram route unavailable ({source}): {reason}. Output remains at its original destination; administrator revalidation is required.")},at)? {
                tx.execute("INSERT INTO meta(key,value) VALUES (?1,?2)",params![key,at]).map_err(e)?;
            }
        }
        tx.commit().map_err(e)
    }

    /// Records that the next stop is planned (`pilotq update` drain or `pilotq service stop`); `false` withdraws it.
    pub fn planned_stop(&self, planned: bool, at: i64) -> Result<(), String> {
        let r = if planned { self.0.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('planned',?1)", [at]) } else { self.0.execute("DELETE FROM meta WHERE key='planned'", []) };
        r.map(drop).map_err(|e| e.to_string())
    }

    /// Each service pass records that this process is alive (#81 U1).
    pub fn alive(&self, at: i64) -> Result<(), String> {
        self.0.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('alive',?1)", [at]).map(drop).map_err(|e| e.to_string())
    }

    /// Service start, before the first pass: a stop without a planned marker or update drain leaves a pending gap from the
    /// last pass to now. A crash loop keeps the earliest start of the gap.
    pub fn service_started(&mut self, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.transaction().map_err(e)?;
        let planned = meta(&tx, "planned")?.is_some() || meta(&tx, "draining")? == Some(1);
        tx.execute("DELETE FROM meta WHERE key='planned'", []).map_err(e)?;
        if let (false, Some(alive)) = (planned, meta(&tx, "alive")?) {
            tx.execute("INSERT OR IGNORE INTO meta(key,value) VALUES ('down_from',?1)", [alive]).map_err(e)?;
            tx.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('down_until',?1)", [at]).map_err(e)?;
        }
        tx.commit().map_err(e)
    }

    /// Healthy start: one owner notice for a pending unplanned gap, downtime when it exceeds 5 minutes. `silent` consumes the
    /// gap without a message (a rollback incident already explains it).
    pub fn restart_notice(&mut self, owner: i64, silent: bool, at: i64) -> Result<(), String> {
        let e = |e: rusqlite::Error| e.to_string();
        let tx = self.0.transaction().map_err(e)?;
        if let (Some(from), Some(until)) = (meta(&tx, "down_from")?, meta(&tx, "down_until")?) {
            let gap = until - from;
            let ru=current_reply_language(&tx,"telegram",owner,owner)?.as_deref()==Some("ru");
            let text = if ru && gap>DOWNTIME {format!("⚠️ PilotQ был недоступен {}, до {} · сообщения за это время обрабатываются сейчас",crate::footer::russian_units(&crate::time::age(gap)),crate::time::exact(Some(until)))} else if ru {"⚠️ PilotQ перезапущен без запланированной остановки".into()} else if gap > DOWNTIME {
                format!("⚠️ PilotQ was unavailable for {}, until {} · messages from that time are processed now", crate::time::age(gap), crate::time::exact(Some(until)))
            } else {
                "⚠️ PilotQ restarted without a planned stop".into()
            };
            if !silent {
                service_message(&tx, owner, Some("restart"), &text, at)?;
            }
            tx.execute("DELETE FROM meta WHERE key IN ('down_from','down_until')", []).map_err(e)?;
        }
        tx.commit().map_err(e)
    }

}
