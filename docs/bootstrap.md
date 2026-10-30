# Repository setup status

Current state: public repository with an implemented Rust test service and separate-bot trials; production cutover remains unapproved. See [acceptance evidence](acceptance.md) and [release timing](service-contract.md#first-release-delivery). This page records repository setup, not a security exception or product qualification.

## Historical setup, 2026-10-16

The repository was initially private, with specification-only checks. Private-plan branch protection and secret-scanning availability were unresolved, and the CLI token could not access the repository. These observations describe the initial setup, not the current runtime or repository visibility. Organization-wide two-factor enforcement and collaborator review were not verified in that setup.

## 2026-10-22 update

The owner chose a public repository so the `main` ruleset applies on the current plan. TaskQ setup adds `taskq.json` (`publish: pr`, incident reports on) and names the CI job `tests` with a `merge_group` trigger, matching the other TaskQ projects. The ruleset, token access and queue registration are verified after the visibility change.
