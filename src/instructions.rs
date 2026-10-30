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
    format!("You are PilotQ, a conversational front end, not a TaskQ worker. Shared communication rules apply to every answer, progress message, plan, task title and body, question and result. Use Documentation for persistent task text and Change communication for change reports. Shared workflow sections apply to preparing and admitting tasks through PilotQ tools. PilotQ's contract additions take precedence. Tool grants are enforced separately.\n\n{}\n{}\n{}\n{}\n{}\n{}\n## Prepare the task{}\n## Task intake{}\n## PilotQ discussion and delegation{}\n## PilotQ conversation contract{}\n## Telegram feedback{}",
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
        include_str!("../docs/telegram-feedback.md").split_once("change them here first.\n").expect("feedback detail exists").1)
}

pub fn fingerprint() -> i64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    (policy(), crate::manager::CONVERSATION, crate::taskq::INSTRUCTIONS, crate::direct::INSTRUCTIONS,
        crate::taskq::tools().to_string(), serde_json::Value::Array(crate::direct::tools()).to_string()).hash(&mut hash);
    hash.finish() as i64
}

#[cfg(test)]
mod tests {
    #[test]
    fn relocated_product_policy_is_loaded() {
        // R13 / §2: relocation must retain both delegation authority and card presentation.
        let policy = super::policy();
        assert!(policy.contains("PilotQ does not perform queued task work or choose answers on the owner's"));
        assert!(policy.contains("Each display offers Launch and Cancel"));
        assert!(policy.contains("immutable task scope and durable receipt rules remain in force"));
        assert!(policy.contains("No need to survive as a long-running job"));
    }
}
