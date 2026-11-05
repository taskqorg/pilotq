//! R17 / §8: semantic metadata through CoreQ's existing writer; no content capture or storage.
use serde_json::{Value, json};
use std::cell::RefCell;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const PASS: &str = "service.pass";

/// A span created under no subscriber can cache `never` for a later collector.
fn collecting() -> bool {
    tracing::dispatcher::get_default(|d| !d.is::<tracing::subscriber::NoSubscriber>())
}

/// CoreQ's repeating root: steady calls under it fold into 15-minute aggregates, while a failed
/// call or changed result keeps its rows. Unchanged service work must not displace request records.
fn repeating() -> tracing::Span {
    if collecting() { tracing::info_span!("operation", coreq_operation="service.steady", coreq_repeating=true) }
    else { tracing::Span::none() }
}

/// Runs a repeated background read, such as a board snapshot, under the repeating root.
pub fn steady<T>(action: impl FnOnce() -> T) -> T { repeating().in_scope(action) }

pub struct Observation { value: Value, start: Instant, keep: bool }

impl Observation {
    pub fn start(kind: &'static str, operation: &'static str) -> Self {
        let link = coreq::telemetry::capture::TraceContext::current().map(|c| c.encode());
        let ids = link.as_deref().map(|s| s.split(' ').collect::<Vec<_>>());
        Self { start: Instant::now(), keep: kind != PASS, value: coreq::telemetry::shared(&json!({
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

    /// Items this pass acted on. A nonzero count keeps the pass as its own record.
    pub fn handled(&mut self, count: usize) {
        self.keep |= count > 0;
        self.value["counts"] = json!({"handled":count});
    }

    /// A polled level, such as registered boards: only a change is news, even a change to zero.
    pub fn level(&mut self, last: &mut Option<usize>, count: usize) {
        if *last != Some(count) {
            self.handled(count);
            self.keep = true;
            *last = Some(count);
        }
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        self.value["duration_us"] = json!(self.start.elapsed().as_micros().min(u64::MAX as u128) as u64);
        self.value["unknown"].as_object_mut().unwrap().remove("duration_us");
        for key in ["trace", "span", "started_ms"] {
            if !self.value[key].is_null() { self.value["unknown"].as_object_mut().unwrap().remove(key); }
        }
        if std::thread::panicking() { self.outcome("failed", "panic"); }
        // An idle pass adds to its 15-minute aggregate; failed, unfinished and working passes stay rows.
        let idle = !self.keep && self.value["outcome"] == "ok";
        let root = if idle { repeating() } else { tracing::Span::none() };
        if idle {
            self.value["kind"] = json!("method.aggregate");
            self.value.as_object_mut().unwrap().remove("counts");
        }
        // A missing collector is explicit; it never changes the operation's result.
        if let Err(error) = root.in_scope(|| coreq::telemetry::capture::observe_linked(self.value.clone())) {
            if error != "telemetry collector unavailable" { eprintln!("Telemetry: unknown ({error})"); }
        }
    }
}

/// One CLI command's terminal record. Process exit skips Drop, so every failing exit closes it here first.
pub struct Command(RefCell<Option<(Observation, tracing::span::EnteredSpan)>>);

impl Command {
    pub fn start(operation: &'static str) -> Self {
        let span = tracing::info_span!("command", coreq_operation=operation, coreq_result="unclassified", coreq_error=tracing::field::Empty).entered();
        Self(RefCell::new(Some((Observation::start("operation", operation), span))))
    }

    pub fn finish(&self, outcome: &'static str, error: &'static str) {
        if let Some((mut observation, span)) = self.0.borrow_mut().take() {
            observation.outcome(outcome, error);
            if error != "none" { span.record("coreq_error", error); }
            span.record("coreq_result", "classified");
        }
    }

    pub fn die(&self, t: &coreq::Tool, message: &str) -> ! {
        self.finish("failed", if message.contains(crate::USAGE) {"usage"} else {"command-failed"});
        t.die(message)
    }

    pub fn exit(&self, code: i32, error: &'static str) -> ! {
        self.finish("failed", error);
        coreq::telemetry::capture::exit(code)
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

    // #374 A2, A4 / R17: idle passes displaced request records from the store. They now fold with
    // their calls and duration, while a working or failed pass and a level change keep their rows.
    #[test]
    fn idle_passes_374_fold_and_exceptions_keep_rows() {
        let path = std::env::temp_dir().join(format!("pilotq-idle374-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let store = Arc::new(coreq::telemetry::Store::open(&path).unwrap());
        let capture = coreq::telemetry::capture::Capture::new(store.clone(), "pilotq", env!("CARGO_PKG_VERSION")).unwrap();
        let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(capture.clone()));
        let rows = |store: &coreq::telemetry::Store| store.export(0, 0, 1024).unwrap()["events"].as_array().unwrap().clone();
        let idle = |count: usize| for _ in 0..count {
            let mut pass = Observation::start(PASS, "service.controls");
            pass.result(&Ok::<(), ()>(()));
            pass.handled(0);
        };
        tracing::dispatcher::with_default(&dispatch, || {
            drop(Observation::start("operation", "request"));
            idle(1);
            capture.flush().unwrap();
            let before = rows(&store).len();
            idle(50);
            capture.flush().unwrap();
            assert_eq!(rows(&store).len(), before, "an idle pass adds no row");
            let mut level = None;
            for count in [2, 2, 0, 0] {
                let mut pass = Observation::start(PASS, "service.boards");
                pass.result(&Ok::<(), ()>(()));
                pass.level(&mut level, count);
            }
            let mut working = Observation::start(PASS, "service.delivery");
            working.result(&Ok::<(), ()>(()));
            working.handled(1);
            drop(working);
            let mut failed = Observation::start(PASS, "service.controls");
            failed.result(&Err::<(), _>(()));
            drop(failed);
            let mut failed_read = None;
            steady(|| failed_read = coreq::telemetry::capture::operation("taskq.read", || Err::<(), ()>(()), |_| "taskq-failed").err());
            assert!(failed_read.is_some());
            for _ in 0..3 { steady(|| coreq::telemetry::capture::operation("taskq.cli", || Ok::<(), ()>(()), |_| "taskq-failed")).unwrap(); }
        });
        capture.finish().unwrap();
        let all = rows(&store);
        let events: Vec<_> = all.iter().map(|r| &r["observation"]).collect();
        assert!(events.iter().any(|r| r["operation"] == "request" && r["kind"] == "operation"), "the request row stays");
        let folded: Vec<_> = events.iter().filter(|r| r["operation"] == "service.controls" && r["kind"] == "method.aggregate").collect();
        assert_eq!(folded.iter().map(|r| r["calls"].as_u64().unwrap()).sum::<u64>(), 51, "calls survive the fold");
        assert!(folded.iter().all(|r| r["outcome"] == "ok" && r["duration_us"].is_u64()), "duration survives the fold");
        let boards: Vec<_> = events.iter().filter(|r| r["operation"] == "service.boards" && r["kind"] == PASS).map(|r| r["counts"]["handled"].as_u64().unwrap()).collect();
        assert_eq!(boards, [2, 0], "only a changed level keeps its count");
        assert!(events.iter().any(|r| r["operation"] == "service.delivery" && r["kind"] == PASS && r["counts"]["handled"] == 1));
        assert!(events.iter().any(|r| r["operation"] == "service.controls" && r["kind"] == PASS && r["outcome"] == "failed"));
        assert!(events.iter().any(|r| r["operation"] == "pilotq.operation.taskq.read" && r["kind"] == "method" && r["outcome"] == "failed"), "{:?}", events.iter().map(|r| (&r["operation"], &r["kind"], &r["outcome"])).collect::<Vec<_>>());
        assert_eq!(events.iter().filter(|r| r["operation"] == "pilotq.operation.taskq.cli" && r["kind"] == "method.aggregate").map(|r| r["calls"].as_u64().unwrap()).sum::<u64>(), 3);
        assert!(!events.iter().any(|r| r["operation"] == "pilotq.operation.taskq.cli" && r["kind"] == "method"));
        drop(capture); drop(store);
        let _ = std::fs::remove_file(path);
    }
}
