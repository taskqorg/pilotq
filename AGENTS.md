# PilotQ

Shared rules: https://github.com/taskqorg/specq (binding for every task). Used owner pages: [principles](https://github.com/taskqorg/specq/blob/main/engineering/principles.md), [spec first](https://github.com/taskqorg/specq/blob/main/engineering/spec-first.md), [verification](https://github.com/taskqorg/specq/blob/main/engineering/verification.md), [security](https://github.com/taskqorg/specq/blob/main/engineering/security.md), [documentation](https://github.com/taskqorg/specq/blob/main/caveman/rules/documentation.md).

Read [pilotq.md](pilotq.md) before work. It owns project decisions and the spec hierarchy; linked research and acceptance cases support it. Change a decision in the contract before changing implementation.

Current stage: implemented Rust test service; first release preparation. Follow the release timing in pilotq.md §9. Proposed thresholds and production cutover remain unqualified.

Development: run the affected Cargo test filter. Candidate gate: `cargo test --locked` and `cargo run --locked -- selftest`; release artifacts also run `scripts/verify-release.sh DIST VERSION`.
