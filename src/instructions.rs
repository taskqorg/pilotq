//! Compile the pinned shared sources into the binary; no checkout or network is needed at runtime.
//! The gitlink is the pin: updating it updates these instructions without editing Rust.
use std::hash::{Hash, Hasher};

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source.split_once(start).expect("instruction section exists").1
        .split_once(end).expect("instruction section ends").0
}

pub fn policy() -> String {
    let workflow = include_str!("../specq/engineering/agent-workflow.md");
    let conversation = include_str!("../docs/conversation-contract.md");
    format!("You are PilotQ, a conversational front end, not a TaskQ worker. Shared communication rules apply to every answer, progress message, plan, task title and body, question and result. Use Documentation for persistent task text and Change communication for change reports. Shared workflow sections apply to preparing and admitting tasks through PilotQ tools. PilotQ's contract additions take precedence. Tool grants are enforced separately.\n\n{}\n{}\n{}\n{}\n{}\n{}\n## Prepare the task{}\n## Task intake{}\n## PilotQ discussion and delegation{}\n## PilotQ conversation contract{}\n## Telegram feedback{}\n## PilotQ reply presentation{}\n\n## Read-first conversation (R5, R13, §5)\nFor near-miss read requests such as taskq list or список задач, run the nearest allowed read-only command and answer with its observed result in the same turn. For a board list use project_status (installed taskq status), not a guessed command. In an unbound route (including a Telegram topic with a remembered conversation project but no explicit route binding), when the current own message names no project, call project_status once with no project argument: it reads every accessible registered board once. Show all returned boards in one reply without a project question; report each unavailable board explicitly. Do not narrow the list using project names from older messages or a guessed tool argument. Never refuse while naming a working equivalent: run that equivalent within current grants and the resolved project. A contextual selection such as 1 of the read option follows through to its result.\nFor yes/да after an either-or question with one read-only option, perform that read and name the other option in one line without a second clarification. This does not answer a TaskQ worker question or select a mutation.\nFor an ambiguous voice transcript with one cheap read-only reading, run that reading first and note the reading taken in the reply. Do not demand exact command spelling.\nNever infer mutation authority, choose a worker answer, bypass grants or guess an ambiguous project or task target. Resolve material target ambiguity with one question; unavailable reads remain explicitly unavailable, never fabricated success.",
        include_str!("../specq/caveman/rules/core.md"),
        include_str!("../specq/caveman/rules/agents.md"),
        include_str!("../specq/caveman/rules/documentation.md"),
        include_str!("../specq/caveman/rules/changes.md"),
        include_str!("../specq/caveman/rules/indicators.md"),
        include_str!("../specq/caveman/rules/notifications.md"),
        section(workflow, "## Prepare the task", "## Task intake"),
        section(workflow, "## Task intake", "## Queue flow"),
        section(conversation, "### Discuss, act, delegate", "### Changing direction"),
        section(conversation, "### Conversation contract", "### Conversation identity"),
        include_str!("../docs/telegram-feedback.md").split_once("change them here first.\n").expect("feedback detail exists").1,
        include_str!("../docs/conversation-flow.md"))
}

pub fn fingerprint() -> i64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    (policy(), crate::direct::alias_instructions(), crate::manager::CONVERSATION, crate::taskq::INSTRUCTIONS, crate::direct::INSTRUCTIONS, crate::loops::INSTRUCTIONS,
        crate::taskq::tools().to_string(), serde_json::Value::Array(crate::direct::tools()).to_string()).hash(&mut hash);
    hash.finish() as i64
}

#[cfg(test)]
mod tests {
    // #251 A2 / R13: PM remedies must reach owner/shell manager turns, not only the setup card.
    #[test]
    fn onboarding_251_pm_remedies_loaded() {
        assert!(crate::direct::INSTRUCTIONS.contains("complete PM onboarding findings"));
        assert!(crate::direct::INSTRUCTIONS.contains("owner or explicit shell authority"));
        assert!(crate::direct::INSTRUCTIONS.contains("recheck before reporting readiness"));
    }

    // #244 A3 / R13: this is an instruction-loading check, not a live model-compliance claim.
    #[test]
    fn project_244_refusal_recovery_is_loaded() {
        let policy=super::policy();
        for requirement in ["Correct the tool\narguments from the person's own text", "Never demand a fixed format", "Never route around a refused grant or replay an uncertain effect"] {
            assert!(policy.contains(requirement),"missing project recovery instruction: {requirement}");
        }
    }

    // #218 / R5, §12: suppressing duplicate plan prose must not suppress the
    // terminal answer required to commit the draft. This is a model-input check.
    #[test]
    fn plan_218_requires_terminal_acknowledgement() {
        assert!(crate::taskq::INSTRUCTIONS.contains("After a successful task_plan, always send a nonempty final answer"));
    }

    // #235 A1/A2 / R13: protect instruction loading on both front ends and
    // rebuilt sessions; this does not prove model compliance or live rendering.
    #[test]
    fn live_235_evidence_limits_are_loaded() {
        let policy = super::policy();
        for requirement in [
            "Report a worker's narrower check as that narrower check",
            "HTTP timing and response HTML do not prove rendered visible text",
            "Check available headless browser executables",
            "Never install a browser, start a login or reuse another person's session",
            "Apply these limits to progress, drafts, final replies and task summaries",
        ] {
            assert!(policy.contains(requirement), "missing live-check instruction: {requirement}");
        }
    }

    #[test]
    fn relocated_product_policy_is_loaded() {
        // R13 / §2: relocation must retain both delegation authority and card presentation.
        let policy = super::policy();
        // #242 / R13: CLI, Telegram and rebuilt sessions load the broad-read rule.
        assert!(policy.contains("it reads every accessible registered board once"));
        assert!(policy.contains("Show all returned boards in one reply without a project question"));
        // #212 A1/A5: native sessions load the authoritative reply rules, including after rebuild.
        assert!(policy.contains("at most three short bullets"));
        assert!(policy.contains("naming who acts"));
        assert!(policy.contains("only when the person's question concerns release"));
        assert!(policy.contains("PilotQ does not perform queued task work or choose answers on the owner's"));
        // #236 A3: load the approved readiness condition while retaining Cancel.
        assert!(policy.contains("Each display offers Cancel and, when project readiness permits, Launch"));
        assert!(policy.contains("immutable task scope and durable receipt rules remain in force"));
        // #190 / §5: a recovered session must not reload the superseded direct-work restrictions.
        assert!(policy.contains("Other authorized work is done directly"));
        for obsolete in ["No need to survive as a long-running job", "only direct\nwrites", "short-action budget"] {
            assert!(!policy.contains(obsolete), "superseded instruction: {obsolete}");
        }
    }
}
