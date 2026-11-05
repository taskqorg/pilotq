//! R17 / §8: semantic metadata through CoreQ's existing writer; no content capture or storage.
use serde_json::{Value, json};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct Observation { value: Value, start: Instant }

impl Observation {
    pub fn start(kind: &'static str, operation: &'static str) -> Self {
        let link = coreq::telemetry::capture::TraceContext::current().map(|c| c.encode());
        let ids = link.as_deref().map(|s| s.split(' ').collect::<Vec<_>>());
        Self { start: Instant::now(), value: coreq::telemetry::shared(&json!({
            "product":"pilotq", "version":env!("CARGO_PKG_VERSION"), "kind":kind, "operation":operation,
            "trace":ids.as_ref().map(|v|v[1]), "span":ids.as_ref().map(|v|v[2]),
            "started_ms":SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|d|d.as_millis() as u64),
            "outcome":"unknown", "error":"unfinished",
            "unknown":{"trace":"no-caller", "span":"no-caller", "started_ms":"clock-unavailable"}
        })) }
    }

    pub fn outcome(&mut self, outcome: &'static str, error: &'static str) {
        self.value["outcome"] = json!(outcome);
        self.value["error"] = json!(error);
    }

    pub fn result<T, E>(&mut self, result: &Result<T, E>) {
        if self.value["outcome"] == "failed" { return; }
        self.outcome(if result.is_ok() {"ok"} else {"failed"}, if result.is_ok() {"none"} else {"operation-failed"});
    }

    pub fn handled(&mut self, count: usize) { self.value["counts"] = json!({"handled":count}); }
}

impl Drop for Observation {
    fn drop(&mut self) {
        self.value["duration_us"] = json!(self.start.elapsed().as_micros().min(u64::MAX as u128) as u64);
        self.value["unknown"].as_object_mut().unwrap().remove("duration_us");
        for key in ["trace", "span", "started_ms"] {
            if !self.value[key].is_null() { self.value["unknown"].as_object_mut().unwrap().remove(key); }
        }
        if std::thread::panicking() { self.outcome("failed", "panic"); }
        // A missing collector is explicit; it never changes the operation's result.
        if let Err(error) = coreq::telemetry::capture::observe_linked(self.value.clone()) {
            if error != "telemetry collector unavailable" { eprintln!("Telemetry: unknown ({error})"); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tracing_subscriber::prelude::*;

    // #374 A2 / R16,R17: failed and unfinished passes stay visible, nested work joins by trace,
    // and neither an error's text nor arbitrary user content enters the observation.
    #[test]
    fn observations_374_metadata_and_failures() {
        let path = std::env::temp_dir().join(format!("pilotq-observation374-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let store = Arc::new(coreq::telemetry::Store::open(&path).unwrap());
        let capture = coreq::telemetry::capture::Capture::new(store.clone(), "pilotq", env!("CARGO_PKG_VERSION")).unwrap();
        let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(capture.clone()));
        tracing::dispatcher::with_default(&dispatch, || {
            let _root = tracing::info_span!("request", coreq_operation="request").entered();
            let mut pass = Observation::start("service.pass", "service.status");
            pass.handled(2);
            pass.result(&Err::<(), _>("invented-private-error /private/path"));
            let mut tool = Observation::start("operation", "manager.tool");
            tool.outcome("refused", "grant");
            let _unfinished = Observation::start("service.pass", "service.update");
        });
        capture.finish().unwrap();
        let rows = store.export(0, 0, 100).unwrap();
        let events: Vec<_> = rows["events"].as_array().unwrap().iter().map(|r| &r["observation"]).collect();
        let pass = events.iter().find(|r|r["operation"]=="service.status").unwrap();
        assert_eq!(pass["schema"], 1);
        assert_eq!(pass["outcome"], "failed");
        assert_eq!(pass["error"], "operation-failed");
        assert_eq!(pass["counts"]["handled"], 2);
        assert!(pass["task"].is_null() && pass["unknown"]["task"].is_string());
        assert!(events.iter().any(|r|r["operation"]=="manager.tool" && r["outcome"]=="refused" && r["trace"]==pass["trace"]));
        assert!(events.iter().any(|r|r["operation"]=="service.update" && r["outcome"]=="unknown"));
        let text = rows.to_string();
        assert!(!text.contains("invented-private-error") && !text.contains("/private/path"));
        drop(capture); drop(store);
        let _ = std::fs::remove_file(path);
    }
}
