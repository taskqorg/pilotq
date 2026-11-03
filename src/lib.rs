//! Portable manager launch boundary (pilotq.md R9, §6). Service policy and delivery stay in the binary.
use agentq_runtime::optimization::Intent;
use serde_json::Value;
use std::process::Command;

pub fn optimization(value: &Value) -> Result<Intent, String> {
    if value.is_null() { return Ok(Intent::Off); }
    value.as_str().and_then(Intent::parse).ok_or_else(|| "config.json: optimization must be off, inherit or on".into())
}

/// Inherited variables a runtime child keeps; the adapter's own launch variables stay as set.
const KEEP: [&str; 8] = ["PATH", "HOME", "USER", "LANG", "LC_ALL", "TMPDIR", "SSL_CERT_FILE", "SystemRoot"];

/// The trait's process setup for every runtime child (#359): only runtime necessities and the adapter's launch variables
/// reach it, so host credentials and API keys cannot become a paid fallback (R9, R16), and it leads its own process group.
pub fn configure(c: &mut Command) {
    let set: std::collections::HashSet<std::ffi::OsString> = c.get_envs().filter(|(_, v)| v.is_some()).map(|(k, _)| k.to_owned()).collect();
    for (k, _) in std::env::vars_os() {
        if !set.contains(&k) && !KEEP.iter().any(|keep| k == *keep) { c.env_remove(k); }
    }
    c.env("AGENTQ_LAUNCHER", "pilotq");
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(c, 0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // R9/R16, §6/#294/#359: optional intent parses strictly, and a runtime child keeps the adapter's launch variables but never
    // a host credential. Risk: an API key in the service environment becomes a paid fallback.
    #[test]
    fn launch_359_child_keeps_launch_variables_only() {
        assert_eq!(optimization(&Value::Null).unwrap(), Intent::Off);
        for code in ["off", "inherit", "on"] { assert_eq!(optimization(&json!(code)).unwrap().code(), code); }
        for value in [json!(false), json!(1), json!("invented")] { assert!(optimization(&value).is_err()); }
        let leaked = ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GH_TOKEN", "GITHUB_TOKEN"];
        // SAFETY: test-only variables, set before any child spawns in this test
        for k in leaked { unsafe { std::env::set_var(k, "invented-never-real") }; }
        let mut c = Command::new("invented-never-started");
        c.env("CODEX_HOME", "/invented/home").env("AGENTQ_OPTIMIZATION", "off");
        configure(&mut c);
        let env: std::collections::BTreeMap<_, _> = c.get_envs().collect();
        assert_eq!(env.get(std::ffi::OsStr::new("AGENTQ_LAUNCHER")).copied().flatten(), Some(std::ffi::OsStr::new("pilotq")));
        assert_eq!(env.get(std::ffi::OsStr::new("CODEX_HOME")).copied().flatten(), Some(std::ffi::OsStr::new("/invented/home")));
        assert_eq!(env.get(std::ffi::OsStr::new("AGENTQ_OPTIMIZATION")).copied().flatten(), Some(std::ffi::OsStr::new("off")));
        for k in leaked { assert_eq!(env.get(std::ffi::OsStr::new(k)), Some(&None), "{k} is removed from the child"); }
    }
}
