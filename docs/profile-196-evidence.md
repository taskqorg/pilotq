# Scoped Telegram profile and access controls (#196)

Source: [task #196](https://github.com/taskqorg/pilotq/issues/196), including I531 “Provide a Telegram flow to inspect, correct and forget remembered facts” and I038 “Support inspecting and correcting remembered information and changing or revoking access.” R10, R13 and pilotq.md §6 own the boundary. Existing administrator grant/revoke tools retain access mutation authority.

The missing path reproduced with `cargo test --locked profile_196_telegram_trace`: without profile dispatch, the first Telegram `/profile set name Old` did not return a remembered-fact readback (assertion failed). The retained regression protects the admission-to-storage-to-reply connection rather than a parser helper. Build setup initially required a writable temporary Cargo cache and initialization of the pinned specq submodule.

The same trace exercises admitted Telegram updates and durable replies: own set/correction with incremented revision, inspect via bot suffix without the old value, own access inspection, reopen with corrected fact retained, forget with audit-retention disclosure, reset/reopen without resurrection, foreign-subject syntax refusal, group/topic DM guidance, another admitted person's forget leaving the first person's private fact intact, duplicate ingress, and grants unchanged by profile edits. The model reads the complete current profile on every turn, including resumed sessions; missing keys override history (existing manager path, §6).

These are local journal/Telegram ingress traces, not a live Bot API or model qualification. Live DM/topic presentation remains PM qualification under §9; no deployment or production cutover is authorized.

After connection: `cargo test --locked profile_196_telegram_trace` passed (1 test). The trace also checks a private fact survives reset, durable DM outbox content, all inputs staying deterministic controls, and revoked-user admission producing no request. Tests use invented preferences and IDs only.

The candidate gate exposed an old help assumption that ordinary users have no people capabilities. Under #196/R10, the help assertion now requires self-service visibility while retaining administrator-capability hiding; its filtered check failed before and passed after. `scripts/check-contract.sh` and offline selftest passed. The parallel full suite twice reported the existing command overflow fixture as `timeout` instead of `output_limit`; that unchanged check passed in isolation. This is an unresolved local parallel-run limitation, not proof that the normal gate passed.
