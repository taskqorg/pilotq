# PilotQ adoption sequence and open decisions

Detail of [pilotq.md](../pilotq.md) §§ 10–11.

## 10. Adoption sequence

1. Review this contract and cases. Decide the direct-action grants, media parity, runtime/provider credentials and deployment budgets before implementing affected behavior.
2. **First slice:** coreq skeleton with `service`, `update`, `selftest`, `status`; separate test bot; journal with durable ingress and outbox; `pilotq ask` and Telegram DM, ordinary group and forum topic; admission and grants with `users`; per-person profile memory; Codex answer; request audit records and CoreQ telemetry from the first request. Qualify duplicate input, timeout, crash, an update during a turn and two-route isolation before adding features.
3. Add one TaskQ project: discuss → authorize → held create/readback → start → status → worker question → answer → reviewer outcome. Restart PilotQ during execution.
4. Add the key store, small direct actions, project registration and flexible project switching, with the same effect receipts and authorization checks.
5. Run the complete route/role matrix and start the seven-day pilot on the native compaction baseline. **Owner, 2026-10-23 (task #26):** agentq 0.9.0 compresses only Codex shell results and PilotQ disables the shell tool, so agentq cannot change PilotQ context yet. C01–C07 run live with agentq off; agentq-on runs are recorded as not run with that reason. agentq qualification follows the agentq dynamic-tool adapter ([taskqorg/agentq#167](https://github.com/taskqorg/agentq/issues/167)) and enables agentq only if the P1 metric gate passes. `pilotq pilot start` records the pilot start, `pilotq pilot note intervention|outage TEXT` records a manual intervention or a provider outage, and `pilotq pilot report` prints the [gate 4](acceptance.md#selection-gates) counts from the journal and telemetry.
6. Prepare migration inventory, restore proof and rollback. Switch production only after the acceptance gates pass and deployment is authorized.

Migration imports a reviewed manifest of route policies, roles, projects and user-approved memories. Credentials transfer through protected configuration. Inspect existing Hermes goals and scheduled jobs one by one; resume, retire or migrate explicitly to avoid two systems performing the same work. Link old task evidence rather than copying execution claims into a model summary.

For a later same-token cutover, stop and verify the old poller before enabling PilotQ. Keep a handoff cursor and identify any old in-flight inputs. Native Hermes history may need a bounded reviewed summary plus archived references, not blind transcript conversion. Rollback drains or preserves PilotQ's accepted inputs and outbox, stops its poller and restores Hermes's consumer. TaskQ tasks already admitted remain authoritative and are not resubmitted.

## 11. Decisions still open

- Codex account and subscription entitlement for a shared server deployment.
- Required behavior for documents that are neither images nor audio, and for video, at Hermes cutover. Image input is decided by the owner in task #72. Voice and audio input are decided by the owner in task #89 (option 1): local GigaAM v3 recognition of Russian speech; a Parakeet fallback for other languages is added only if real use shows the need.
- Final retention/capacity thresholds and permitted telemetry content, including the proposed audit retention (365 days metadata, 90 days message text).

Closed 2026-10-23 (owner, option B): short-action grants and Telegram key entry. Direct actions are reads per grant, the owner alone comments on and labels board issues, nothing else is written directly, and keys enter only through the CLI ([section 5](integration-contract.md#direct-actions-keys-and-project-switching)).
