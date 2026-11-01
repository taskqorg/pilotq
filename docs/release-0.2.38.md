# PilotQ 0.2.38 qualification handoff

This candidate updates only the existing Cargo package version and lockfile to 0.2.38. It prepares merged behavior for PM released-build qualification; it does not authorize activation or production cutover (§9, R19).

## Source and release boundary

Fresh origin and GitHub latest-release reads on 2026-11-01 confirmed v0.2.37, published at 04:49:32Z, source/live `3a581fff3e3336fa1bc60bf7178629598c52052e`. Main was `4dbfc348a45c6d71450283fe210a453b67b598c9`; live is its ancestor. No v0.2.38 tag existed. The complete live-to-main range before this version bump, oldest first:

```text
42fbd708bc448cf59499ff62d4758dbc0fbdb3d6 feat(tasks): carry requested host, runtime, priority and deps to taskq add (#269)
8aded5ba466d0378572f0356c55898d287250d80 fix(tests): publish helper scripts without writable descriptor races (#271)
a86bff1697a6b4d293e5b7f9c2f9a67bd01ba481 feat: deliver board events without a model turn; one PM review per person (#272)
2a5b629dc9526c630eff8e81a92da37ca01409b0 fix: show Russian onboard usage when project is missing (#276)
9a9e6439105e288e4347d917b242173f86cf7bbe fix: resolve project aliases before task-tool lookup (#277)
4dbfc348a45c6d71450283fe210a453b67b598c9 feat(access): admin, project owner and member roles replace per-project grants (#278)

```

TaskQ trusted closing receipts identify #270 → PR #271 → `8aded5ba466d0378572f0356c55898d287250d80`, #274 → PR #276 → `2a5b629dc9526c630eff8e81a92da37ca01409b0`, and #275 → PR #277 → `9a9e6439105e288e4347d917b242173f86cf7bbe`, each with green merge-result CI. All are ancestors of the candidate base. The other included behavior is task-launch host/runtime/priority/dependency forwarding, model-free board cards and event-woken PM reviews, and admin/project-owner/member roles. No implementation is added here.

## Merge and PM checklist

1. Review the two version edits and this evidence. Require both tests and release workflows green on the merge_group result; TaskQ ordinary review owns publication. No admin bypass or auto-merge. Before PM activation, reread main, live and latest release; stop if another release or version conflicts. Record the exact accepted merge SHA and complete live-to-that-SHA range, including the version commit. Only the PM may fast-forward live to that accepted main commit (§9).
2. After ordinary release publication, verify latest.json source_commit equals that merge SHA, binary version is 0.2.38, and downloaded bytes match SHA256SUMS. Record the installed executable identity, source, status, selftest and doctor on pds18. Use the existing drain and compatible-schema rollback path; workers never install or restart live (§9).
3. In the authorized Russian DM, send `/onboard` without a project. Retain request identity and visible actionable Russian usage for `/onboard NAME` and `/onboard NAME override`; it must not be the generic status/retry response. Compare the recorded pre-fix failure and passing local regression in docs/acceptance.md (#274). Check English and bot-suffix paths where relevant (R13).
4. In a clean authorized context, ask `Is cs #541 on DEV?`, then compare `Is cyprusbz #541 on DEV?`. Retain exact inputs, selected canonical project/board, tool arguments/results, verified task link and registered release evidence. No alias registration error is acceptable. Report only the highest proved delivery stage; missing release facts remain unknown with the real reason (§5).
5. Separately replay the exact original req359 wording and its relevant route/context from the PM-retained original. The available TaskQ evidence does not retain that exact text or failed tool arguments: it says the text also named PilotQ. Do not substitute the clean query for this regression or infer that alias normalization resolves context ambiguity. Compare the original three task_status failures, project_use failure and unrelated PilotQ status read. PM evidence: #257 comment 6553765177 and #275 answer 6553877232; local red/green alias proof: docs/research/275.md.
6. Record request/journal continuity across the drained update: same people, roles/grants, routes, subscriptions, saved work and receipts; schema stays 13. Prior shipped 0.2.37 source `3a581fff3e3336fa1bc60bf7178629598c52052e` and candidate base `1a87c6fc86ab8b979435d1c6b5cf9f9b37129144` both support schema 13; this static compatibility evidence does not prove rollback restoration. Accepted requests retain an explicit outcome, completed effects are not replayed, and unfinished/uncertain work is identified (R5, R13, R17, §4). Retain request evidence for both retests and compare with the prior 0.2.37 source.

Live retests, exact req359 replay, host continuity and automatic-management reserved-host qualification remain unknown until PM evidence. Local tests and isolated updater fixtures do not prove them (§9, R19).

## Candidate verification

`scripts/check-contract.sh` passed with zero errors after initializing the existing pinned specq submodule (the initial run lacked its linter). `cargo test --locked` and `cargo run --locked -- selftest` passed using a temporary writable Cargo cache; the default cache attempt was sandbox-blocked by a read-only filesystem. No tests or protecting assertions changed. No service process remained after the suite.

`scripts/verify-release.sh /tmp/pilotq-279-dist 0.2.38` passed checksum, manifest, selftest, corrupt-payload rejection and isolated updater checks against the local GNU debug binary. This is updater evidence only: the worker host has neither the musl target nor musl-gcc. Static optimized linux-x86_64 artifact verification remains a required release-workflow CI check on the merge result; do not treat the local debug fixture as the release artifact (§9).

The scope is version preparation, not a new defect fix. Prior failure/pass evidence for the included fixes stays in their trusted TaskQ results and linked acceptance/research records. PR and merge-result CI remain pending at worker handoff; TaskQ review must enforce green merge_group before publication.

## Schema declaration correction (#301)

Before correction, the active root/service contracts, rollback guide and this checklist declared schema 12 despite both cited sources declaring 13. The correction changes documentation only; journal, migrations, rollback and runtime behavior remain unchanged (§9, R19). Historical candidate verification above remains evidence for its original revision.

After both #299 and #301 are accepted with green merge-result CI, the #281 PM must record their exact final accepted source and run one fresh full 14-case A40 suite plus independent cleanup on that source before release. Any source advancement requires renewed exact-source gates; earlier runs cannot qualify the new candidate. No host changes, installation, release or new migration are authorized by this correction.
