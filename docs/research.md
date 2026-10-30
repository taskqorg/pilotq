# Manager runtime and Telegram reliability research

Decision supported: the proposed composition in [pilotq.md](runtime-contract.md#3-architecture-and-ownership). Source review: 2026-10-16. Source code and issue reports establish mechanisms and risks; they do not prove a reliability ranking. No new live model or Telegram scenario was run for this specification.

Follow-up: the owner now requires Codex-only subscription access and asks whether Pi is needed at all. [Core options](core-options.md) reassesses the initial recommendation below. Neither runtime recommendation is an adopted implementation decision.

[Mechanism matrix](mechanism-matrix.md) extends the source review to lifecycle, progress, supervision, delivery and recovery. It records 60 scope decisions and the acceptance gaps added to the root contract and case set.

## Recommendation

Qualify **Pi coding-agent as the first manager runtime**, behind a small durable Telegram coordinator. Reuse TaskQ for all substantial execution. Use grammY for Telegram protocol handling, with a commit-before-ack ingress path owned by PilotQ. Do not fork a complete assistant or build another goal scheduler.

Pi is preferred for controllable tools, context and compaction, which suit agentq. This is an integration-fit recommendation. Its reported transport hangs and compaction race make a bounded runtime process and explicit qualification necessary. If the first vertical slice requires maintaining a Pi fork, compare Codex app-server before expanding the implementation.

OpenClaw remains the full-product fallback if operating a custom coordinator costs more than adopting its existing gateway. A larger product may reduce initial integration work but adds components unrelated to the requested manager role. Neither size nor ownership proves stability.

## Candidates

| Candidate | Useful existing behavior | Material limitation for PilotQ | Proposed role |
|---|---|---|---|
| Pi coding-agent 1.3.0 | SDK/RPC, explicit sessions, event stream, custom tools, context hooks and compaction | Session persistence is not an external-effect transaction; reported hangs and compaction overlap need tests | First runtime candidate |
| Codex app-server | Native threads, resume, turn events, steer, interrupt and approvals | Telegram durability still belongs to PilotQ; dynamic tools are experimental; exact agentq boundary needs qualification | Runtime comparison if Pi fails |
| Claude Agent SDK | Explicit session resume, structured result events and tool permissions | Telegram durability still missing; terminal-result handling and context adapter need verification; inspected TaskQ matrix does not qualify non-WSL Linux Claude workers | Secondary runtime option |
| OpenClaw 2026.10.3 | Existing groups/topics, ingress spool, delivery recovery and access configuration | Broad product surface; overlapping runs have an open lost-reply report from an older release | Full gateway alternative and reference implementation |
| NanoClaw 2026.10.0 | SQLite mailboxes, independent session delivery and isolated agent containers | More operating components; an older adapter has a silent-ingress incident report | Borrow mechanisms; do not adopt wholesale initially |
| Pi Durable | Persistent turns/tool state with recovery policies | Explicitly experimental, API changes without notice; coding-agent hook parity is not established | Research only |
| Takopi and existing Pi Telegram bridges | Existing routing and remote session interaction | Poller/restart reports or narrower multi-user/session contracts; exact current compatibility unqualified | Reference behavior, not assumed durable base |
| Hermes and nanobot | Broad assistant features and sustained-goal behavior | Hermes conflicts with the owner's observed reliability needs; nanobot has reported partial-tool recovery gaps | No default replacement choice |

Official runtime references: [Pi SDK](https://github.com/earendil-works/pi/blob/71023aec1546d6d677a65669c5665c0927ce0dd0/packages/coding-agent/docs/sdk.md), [Codex app-server](https://learn.chatgpt.com/docs/app-server), [Claude sessions](https://code.claude.com/docs/en/agent-sdk/sessions). Runtime documentation describes APIs, not completed PilotQ acceptance.

## Mechanisms worth reusing

### Pi session and context control

Pi 1.3.0 provides `createAgentSession`, explicit `SessionManager`, steer/follow-up/abort and custom tools/resources. `SessionManager` owns finalized model context; directly assigning agent messages does not replace persisted history. `message_end` is a completed message, while `agent_settled` indicates that automatic recovery and queued work have stopped. A host still checks error/abort outcomes before reporting success.

The reviewed [session-manager source](https://github.com/earendil-works/pi/blob/71023aec1546d6d677a65669c5665c0927ce0dd0/packages/coding-agent/src/core/session-manager.ts) uses synchronous JSONL writes/appends; the inspected rewrite/persist paths do not establish the fsync/atomic-rename transaction required for PilotQ acknowledgements. Keep critical receipts and recovery inputs in PilotQ's durable store. Native session storage remains useful for routine resume.

The [extension API](https://github.com/earendil-works/pi/blob/71023aec1546d6d677a65669c5665c0927ce0dd0/packages/coding-agent/docs/extensions.md) provides tool-result, context and compaction integration points. Verify how an agentq replacement reaches both model context and persisted history; a hook firing alone is not sufficient evidence.

### OpenClaw Telegram ingress and delivery

In the pinned [polling implementation](https://github.com/openclaw/openclaw/blob/59cb44796b0f1f0daa0e37af8b1495abb45d9bc1/extensions/telegram/src/polling-session.ts), the isolated worker waits for admission acknowledgement; committed spool admission is the acknowledgement boundary. The code also has polling-liveness checks and restart handling. [Offset persistence](https://github.com/openclaw/openclaw/blob/59cb44796b0f1f0daa0e37af8b1495abb45d9bc1/extensions/telegram/src/update-offset-persistence.ts) distinguishes accepted and committed positions and retries storage failures.

The [delivery recovery policy](https://github.com/openclaw/openclaw/blob/59cb44796b0f1f0daa0e37af8b1495abb45d9bc1/src/infra/outbound/delivery-queue-recovery-policy.ts) avoids preempting a live delivery owner, persists attempt accounting and classifies permanent recipient errors. Adopt these invariants. Copying a large queue subsystem is not necessary for a one-host manager.

### NanoClaw mailboxes

The reviewed [delivery code](https://github.com/nanocoai/nanoclaw/blob/7203e00dc271cc2ea9ea84bb130731b8ca00319e/src/delivery.ts) separates pending output from channel sends, retains retry attempts across restarts and guards simultaneous drains of one session. A code comment documents the exact double-send race: two drains can both send before an idempotent database receipt is recorded.

Its [mailbox schema](https://github.com/nanocoai/nanoclaw/blob/7203e00dc271cc2ea9ea84bb130731b8ca00319e/src/mailbox/sqlite/schema.ts) and [router](https://github.com/nanocoai/nanoclaw/blob/7203e00dc271cc2ea9ea84bb130731b8ca00319e/src/router.ts) are useful references for persisted conversation delivery. PilotQ does not need to inherit the container/mailbox topology to preserve those properties.

### Telegram framework boundary

[grammY reliability documentation](https://grammy.dev/advanced/reliability) explicitly describes loss/duplication limits of concurrent polling and suggests queueing received updates before processing. [Runner](https://grammy.dev/plugins/runner) provides concurrency control, not a durable application inbox. Use the framework's API client and update dispatcher; qualify the acknowledgement boundary rather than assuming `bot.start()` or `run()` solves it.

Telegram [update delivery](https://core.telegram.org/bots/api#getting-updates) retains pending updates for at most 24 hours. [getUpdates](https://core.telegram.org/bots/api#getupdates) advances acknowledgement through its offset. [sendMessage](https://core.telegram.org/bots/api#sendmessage) does not offer a general client-supplied idempotency key. These protocol limits shape PilotQ's gap reporting and duplicate-delivery policy.

### TaskQ and agentq reuse

The reviewed [TaskQ contract](https://github.com/taskqorg/taskq/blob/93b499cf562167e380793c0c8fb15562407c6900/taskq.md) already defines durable task state, trusted comment logs, independent workers/reviewers, revisions, operation IDs, questions and held creation. The [operator implementation](https://github.com/taskqorg/taskq/blob/93b499cf562167e380793c0c8fb15562407c6900/src/operator.rs) contains native operator flows but is not a ready Telegram API.

Two adoption prerequisites remain material: safe creation needs configured create claims and board readback qualification; [runtime qualification](https://github.com/taskqorg/taskq/blob/93b499cf562167e380793c0c8fb15562407c6900/docs/runtime-qualification.md) is bounded evidence, with recovery cases still unqualified. An earlier read-only host inspection found TaskQ 4.5.0 installed with four project services and admission off. This specification neither activates them nor treats that health observation as task-completion evidence.

[agentq](https://github.com/taskqorg/agentq) is a first-party project, so a missing Pi adapter is normal integration work. Its native-client compression checks already distinguish a hook/config signal from real model-visible compression and original recovery. Reuse that evidence discipline. The adapter must preserve native runtime authentication and permissions.

## User reports converted into acceptance cases

All reports below were read from their issue bodies and their open status refreshed on 2026-10-16. They are reports, not reproduced defects in the latest releases. Older version numbers remain attached to the claim.

| Report | Evidence boundary | PilotQ consequence |
|---|---|---|
| [Pi #4945](https://github.com/earendil-works/pi/issues/4945) | Original report on 0.75.5; earlier research also read October comments on 1.0.4/1.1.0 | Deadline and independently observed model progress; verify chosen provider on 1.3.0 |
| [Pi #10982](https://github.com/earendil-works/pi/issues/10982) | Run starts during manual compaction, reported on 1.2.0/1.1.0 | One serialized owner for input, compaction and continuation |
| [Pi Durable #10386](https://github.com/earendil-works/pi/issues/10386) | Hook incompatibility report; author reports 45 successful forced-crash recoveries for extension-free roles | Useful positive evidence for that author's setup, not proof for agentq or PilotQ; package remains [experimental](https://github.com/earendil-works/pi/blob/71023aec1546d6d677a65669c5665c0927ce0dd0/packages/durable/README.md) |
| [OpenClaw #148707](https://github.com/openclaw/openclaw/issues/148707) | Lost reply during overlapping runs, initially 2026.9.4 | Persist request outcomes and reject stale runtime authority |
| [NanoClaw #3728](https://github.com/nanocoai/nanoclaw/issues/3728) | Silent ingress failure on 2.3.0; applicability to calendar release unverified | Track successful polls, not merely process uptime or outbound success |
| [nanobot #5747](https://github.com/HKUDS/nanobot/issues/5747) | Partial tool batch lacks complete recovery checkpoint | Persist effect outcomes individually; never assume all calls are pending |
| [Takopi #254](https://github.com/banteg/takopi/issues/254) | Duplicate pollers after stale PID ownership | OS-held token lock plus deployment ownership checks |
| [Claude gateway #241](https://github.com/overwirehq/claude-code-telegram/issues/241) | Missing final reply; stream disconnect mistaken for success | Require terminal result; retry delivery independently from execution |

These reports provide a failure catalogue. Issue counts, popularity and labels are not a ranking of successful task completion. The inspected sources contain substantial reliability mechanisms alongside reported failures; that is why the same end-to-end cases must decide adoption.

## Reuse and implementation boundary

The reviewed Pi, OpenClaw, NanoClaw and grammY root licenses are MIT at the pinned revisions. Preserve applicable notices if copying code; check the license of any separately packaged dependency before incorporating it. Prefer supported APIs and small established patterns over vendoring internals.

New PilotQ work is limited to conversation admission/delivery, session ownership, validated short-action and TaskQ tools, and recovery-visible status. It owns no competing execution queue. It should not require Redis, a workflow server or a new web dashboard for the first host.

Select the production runtime only after the [functional, fault and context gates](acceptance.md#selection-gates) pass. A test failure should first be assigned to Telegram transport, PilotQ state, runtime/provider, agentq or TaskQ; replacing the model will not fix a lost polling offset.
