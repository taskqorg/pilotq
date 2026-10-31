# PilotQ product contract

PilotQ is the conversational front end for work on a shared server: it answers in Telegram or the CLI, performs authorized actions and manages substantial work through TaskQ. It has no worker scheduler, autonomous goal engine, reviewer, separate task board or general plugin marketplace. Accepted requests must survive a lost model connection without authorizing repeated external effects.

## Rules

- **R9 Model access:** use Codex with existing subscription access only. Claude and separately billed model APIs are out of scope; never fall back automatically to paid API credentials.
- **R10 People and admission:** ignore people without explicit administrator-granted access; group membership grants nothing. Keep distinct, persistent, correctable profiles and support access inspection, change and revocation.
- **R16 Keys:** tools may use credentials on an authorized person's behalf; the model never sees their values. Keys enter only through the protected CLI.
- **R4 Flexible projects:** remember the usual project and allow switching; every external action resolves one concrete authorized target.
- **R5 Reliability:** ordinary conversations and simple tasks work without repeated nudges. Restarts, timeouts and long context have explicit recoverable outcomes; accepted requests and uncertain effects are never silently lost or replayed.
- **R17 Request audit and telemetry:** each request durably records who, when, where, request, actions and outcome, joined to CoreQ telemetry for investigations, personal history and self-improvement analysis.
- **R13 Two front ends:** the CLI and Telegram gateway share the request, grant and audit path.
- **R11 Observable lifecycle:** explain acceptance, progress, waiting, interruption, failure and completion in Telegram. Specify recovery and operational edge cases and their acceptance cases before implementation.
- **R1 Conversation:** serve multiple authorized people on a server through DMs, groups and forum topics, preserving authorship and route.
- **R2 Direct help:** discuss, research, inspect and act directly with the service user’s access; task work uses TaskQ first.
- **R3 Management:** use TaskQ for most substantial work: clarify, plan, file, follow, report status and relay worker questions and results.
- **R8 Visibility:** provide task URLs, current state, attempt evidence and outcomes through existing TaskQ facilities.
- **R15 Administration by request:** authorized administrators may manage people and rights, create or register projects and open project-bound forum topics only through §5's scoped administration tools.
- **R6 Continuity:** a later Hermes migration preserves existing people, access roles, conversation boundaries and project associations.
- **R7 Context:** support effective agentq integration, original-data recovery and a measured quality baseline.
- **R14 Operated like our tools:** monitor PilotQ, update it through releases and develop it spec-first through its own TaskQ board.
- **R12 Least effort:** build the simplest agent serving TaskQ work; reuse proven mechanisms from known projects and the shared TaskQ system.
- **R18 Specification authority:** [specq](https://github.com/taskqorg/specq) is binding. Explicit approved PilotQ exceptions take precedence and name their shared rule, scope and reason. Supporting specifications below are binding details of this contract; README, instructions, tests and code follow it.
- **R19 Qualification:** the service is implemented for testing and first-release preparation. **Proposed** clauses and thresholds remain design targets, not shipped guarantees, an SLA or deployment permission. Production consumer replacement and unqualified acceptance gates remain unauthorized. Summaries inherit their linked clauses’ proposal status.

## 1. Scope and input boundaries

**Proposed scope:** one Linux host, one Telegram bot, one manager runtime, existing TaskQ boards and explicitly registered project roots. The input choices below are mandatory within that scope.

Accept text, replies, task links, inline decisions, photos and image documents with captions retained. Recognize voice notes, audio files and audio documents locally as Russian speech. Photos and arbitrary documents may be TaskQ reply attachments through answer, steer and owner requeue.

Other documents up to 20 MiB reach the turn as PDF text, ZIP listing or plain text, else the exact reason; videos and video notes as frames plus recognized speech. Larger files get one short file-limit line; other input one unsupported-input line, both in the chat's language.

Recurring execution stays with the existing scheduler or TaskQ. PilotQ's local inbox schedules only conversation handling and delivery. Detailed media, retention and admission protections are in §§2, 5, 7 and 12.

## 2. Conversation, admission and authorization

**Mandatory:** [conversation rules](docs/conversation-contract.md) define admission, route identity, discussion, delegation and follow-ups; [conversation flow](docs/conversation-flow.md) alone owns presentation. Grants, immutable task scope and durable receipts retain authority over presentation.

Ignore unknown, revoked and unidentifiable senders without reply, reaction, typing, model calls, tasks, profile updates or retained bodies. Bootstrap the protected owner through installation configuration. Stable Telegram IDs and explicit grants establish authority; names, group badges, quotations and external content do not.

Use isolated routes by default; shared topics preserve authorship and never import private DM context. Preserve the inspected Hermes per-user ordinary-group and shared-topic policies through a reviewed migration manifest before cutover.

Edits retain the original author, current grants and an audited revision; started work receives a change request. Telegram route incidents hold output at its original destination without automatic rerouting (§4, §12).

Discussion never executes a plan. A direct request authorizes its requested work; task launch binds the saved immutable revision, target and scope. Task work uses TaskQ first; other work is done directly with a verifiable result.

Keep one open draft per conversation. Each display offers Launch and Cancel; old, foreign, repeated and superseded callbacks cannot authorize new work. Edit source cards with factual outcomes and verified links. One durable launch status card projects TaskQ state; retain trusted closing findings and limitations.

Persist follow-ups immediately and process them in order after the active turn. Relate them by meaning, without time windows or required reply syntax; steer a matching doing task after a fresh read. Ambiguity requires a target question. Supply the current generation's latest twelve handled inputs plus the current input, saved plan and scoped subscriptions, retaining image bytes and explicit gaps.

Plain-language controls are primary. `/status`, `/stop` and `/new` remain available outside the model backlog. Reset fences old runs without erasing task links, subscriptions, grants or receipts. Resolve stop scope explicitly; a manager stop, task park and queue pause differ.

## 3. Architecture and authority

**Mandatory:** one Rust binary on CoreQ; teloxide only for the Bot API; Codex `app-server` with subscription login and one small adapter; SQLite journal; installed TaskQ CLI for execution; ffmpeg and static `sherpa-onnx-offline` with GigaAM v3 RNNT-punct int8 for local CPU recognition.

[Architecture detail](docs/runtime-contract.md#3-architecture-and-ownership) assigns journal, runtime, proposal, TaskQ and agentq authority. PilotQ owns durable input and delivery; TaskQ owns admitted execution. Registered roots and host policy own capacity. Never discover roots or silently enroll projects.

Use a neutral workspace and scoped registry tools; no process-wide project directory switch or universal runtime
framework. Native shell, live web search and danger-full-access start only for a turn whose principal is the configured
owner_id or holds an explicit shell grant. Other turns disable native shell and web search and use a read-only sandbox;
PilotQ’s own granted tools remain available.
Admission and current grants still gate turns; model and effort are unchanged. Grammy, Pi and a new runtime framework
remain out of scope. Hermes gateway patterns are reference material; this work does not change Hermes.

## 4. Durable requests, effects and delivery

**Mandatory:** preserve the [durability protocol](docs/runtime-contract.md#4-durable-conversation-and-effects): atomic ingress/cursor disposition, one OS-held poller lock, stable request/run/operation identities, fencing, effect intent before dispatch and verified receipts afterward. Completion and Telegram delivery are separate.

Reconcile TaskQ mutations by the same operation ID and revision. Retry direct mutations only under their declared idempotency/readback contract; uncertain non-idempotent effects block replay. Rebuild native sessions from recorded input/output and receipts without repeating completed effects.

The proposed request states are queued, running, waiting for a person, delegated, answered, failed, cancelled and uncertain. Commit final replies and questions to an ordered outbox with immutable destinations. Retry delivery without rerunning work; ambiguous Telegram sends can visibly duplicate and must stay explicit.

Normal accepted text gets 👀 within five seconds, text on reaction failure and one queue notice after thirty seconds. Keep the retention notice separate and omit internal request IDs. Private drafts are ephemeral; shared rich/HTML/plain fallback for every user-visible message, UTF-16 limits, individual receipts and persisted per-chat cooldowns follow the linked delivery protocol.

## 5. TaskQ, tools and project administration

**Owner, task #190:** “Let it stay a full agent that can do anything, but its specialization is our tasks: for tasks it
works through TaskQ first, as its convenient mechanism.” “Access must be wide, full, like Hermes.” This decision
replaces the read-only, no-shell, restricted native-tool and five-call rules. Board state still comes only through
TaskQ; R9 subscription access, R10 admission and §9 release qualification remain binding. Native effects have
service-user authority, not typed-tool replay guarantees: persist observed tool outcomes and never replay an interrupted
turn automatically. After any restart, deliver the recorded completed findings and explicitly identify unfinished or
unconfirmed work. Shell requests without that authority are refused in one plain line.

**Mandatory:** [integration detail](docs/integration-contract.md) and the [TaskQ tool contract](docs/taskq-tool-contract.md) bind tool scope, target selection, grants, operation receipts, refusals and recovery. Use only installed TaskQ CLI board access; no forge adapter, constructed task URLs or forge bypass.

Use argument arrays and typed validation. TaskQ status/readback binds the registered root and board. Creation requires `create_claim_issue` and qualified readback; no plain-add retry fallback. Read every child ticket before starting a multi-ticket plan. Bind answers to board, task, attempt, question revision and authorized answerer.

For the owner or an explicit shell grantee, native tools may perform requested work with the service user’s access, including host and remote-host administration. Typed TaskQ and administration tools retain their grant checks and operation receipts. Revocation refuses later calls; completed effects remain recorded. Keys enter through the CLI only.

`issue_read`, `issue_comment` and `issue_label` use `taskq issue N`; writes use stable `--op-id` and TaskQ receipts. `asana_read` uses the fixed read-only Asana API origin with no redirects/downloads, a 256 KiB response cap, ten-second deadline and bounded fields. External content never authorizes work.

Task status reports observed issue state, trusted closing verdict, report, proposed follow-ups and verified existing follow-ups; missing facts stay unavailable. Diagnose, research and investigate plans retain TaskQ `--type research`. Open-snapshot absence proves neither closure nor nonexistence.

TaskQ reply files retain identity/caption, protected downloads, a 20 MiB cap, original target/revision/operation and same-operation reconciliation. Target/grants are validated before downloading and again at dispatch. Requeue with files requires the protected owner. Text is optional when files exist; refusals and uncertainty retain the reply and remedy.

Project aliases live in the code-owned alias table in `src/direct.rs`: `cs` means `cyprusbz`.
Resolve aliases in the caller’s own project references and manager instructions before target selection;
aliases never register a root or grant access. “Is cs #541 on DEV?” uses the same target as “Is cyprusbz #541 on DEV?” without an alias clarification.

Project resolution uses current grants and explicit own messages, the current project or narrowly scoped trusted subscriptions. Quoted/tool content never selects a project. Ambiguous targets require one question; numbered replies select only the latest boundary-generated complete option listing from the same person and generation.

Installed read commands use an owner-only `commands.json`, absolute executables, exact arguments and bound project/root/board/user permissions. This optional typed read adapter does not restrict native shell access. A fresh complete TaskQ board snapshot precedes execution; process-group limits are ten seconds and 16 KiB output within the turn budget.

Administration tools prepare/register only explicitly requested existing repositories under `<install>/projects`, change explicitly named grants for verified positive IDs seen in the route, or create one authorized forum topic with fresh bot permission. They cannot change the protected owner, create repositories/claim issues, install services, provision keys or alter capacity.

Persist immutable administration intent, readback and receipts. Unknown topic-creation effects never replay; a recorded topic ID permits only completing its local route binding after revalidation. Registration does not imply filing readiness or authority.

Onboard only registered roots after registration, startup gaps or changed TaskQ identity. Require fresh doctor/setup evidence, active service, usable Codex capacity, explicit effective grant, claim prerequisite and owner CLI/DM `issue_read` key scope. Gaps remain visible in one durable owner DM card and nonzero CLI findings; Recheck edits it.

Block a project's first launch until its current root/board readiness passes. Protected-owner `/onboard NAME override` bypasses only PilotQ's readiness gate, records findings and retains TaskQ checks; recheck or registration changes invalidate it. Existing workers continue. Owner-only installation, capacity, keys and login remain named steps rather than executable fixes.

Registered DEV/LIVE release is an exception to §2 deployment delegation and §5 direct writes. Require protected owner, project scope and separate `release:NAME`; only a bound Release button executes. Protected configuration binds exact board/root, HTTPS health, refs, absolute executables, exact arguments and minimum versions. No shell or model command exists.

When asked about release, deployment or health, DEV visibility names one highest proved task delivery stage: closed from board state, merged from the trusted merge commit, on DEV from the deploy job, DEV smoke from the smoke jobs of the first main pipeline containing it, or on LIVE from LIVE branch ancestry.

Read jobs through the protected project release adapter; name source failures in plain text without raw URLs or adapter
stderr. Drop reqwest URLs; retain adapter diagnostics only in audit after keys.scrub. Worker notes never override pipeline facts; only a successful containing main pipeline proves CI passed. Report failed smoke job links and first failing checks, plus whether a later containing pipeline passed. Keep DEV health timing separate.
Internal recovery progress stays in audit. Merge identity: TaskQ's trusted merge receipt, else for a GitLab hand merge the first main first-parent commit containing the latest trusted `taskq: result` SHA (that SHA when fast-forwarded); squashed heads stay unknown.

Preview pins main/LIVE SHAs, range/authors, verified task URLs, migrations and DEV inclusion; unavailable facts refuse. Expiry is fifteen minutes. Inspect every registered same-origin checkout through TaskQ before preview and dispatch; a running attempt or incomplete evidence blocks. Main advancement never retargets; LIVE advancement requires another preview.

Persist confirmation before ordered preflight, backup, pinned release and pipeline readback. Use a repository lock and fifteen-minute deadline; drain refuses during release. Revocation stops remaining stages; ambiguous/interrupted commands stay uncertain without replay. Completion needs matching pinned/LIVE SHA and successful release, smoke and backmerge evidence with validated HTTPS links.

For cs use `https://dev.cyprus.bz/__health`, its registered release root and configured npm release/preflight/backup commands; never Tight's checkout. Git ancestry and trusted
explicit merged SHAs prove DEV inclusion. Health timestamps or labeled first-positive observations bound timing; failures stay unknown. Pipeline adapter, command schema and PM live
qualification remain in the integration detail.

The #190 owner decision is a manager-only exception to specq’s
[permission authority](https://github.com/taskqorg/specq/blob/main/engineering/permission-authority.md)
rule protecting the policy store from governed workers: the trusted manager has the service user’s
filesystem and host authority so it can administer those hosts. This is trust, not isolation from
same-user policy or credential files. R16 still forbids deliberately reading secret values into the
model or replies. TaskQ workers and §9 release qualification retain their existing boundaries.

## 6. Memory and agentq

**Mandatory:** [memory detail](docs/runtime-contract.md#6-memory-and-agentq) separates conversation history, scoped remembered preferences and authoritative TaskQ facts. Profiles retain subject, visibility, source and revision across restarts and resets; private DM facts never silently enter shared routes.

Telegram `/profile` inspects own facts in DM; `/profile set KEY VALUE` corrects or remembers a private preference and `/profile forget KEY` removes it. `/access` inspects own current grants in DM; neither control accepts another subject or changes grants. Both use the CLI/Telegram admission, audit and durable reply path.

Users inspect, correct and forget non-authoritative preferences; administrators act only within scope. Persist and show changes, suppress superseded facts after rebuild/compaction, and distinguish preference deletion from audit retention. Profiles cannot change grants.

Preserve identities, grants, revocations, project references, operation IDs, task URLs, pending questions, plan revisions, prohibitions and receipts outside lossy summaries. Use fresh TaskQ reads for task state.

Native compaction is the functional baseline. Agentq tool compression and conversation summarization have separate owners; preserve call/result associations and scoped original recovery. Qualify identical scenarios off/on for completion, constraints, recovery, provider tokens where available, latency and interventions. Byte reduction alone proves no savings or long-conversation correctness.

## 7. Proposed operating budgets and storage

**Proposed:** [budget detail](docs/runtime-contract.md#7-proposed-operating-budgets) retains every acceptance target and retention constraint. These are initial targets to tune from recorded pilot evidence, not an SLA.

Receipt p95 ≤2 seconds; acknowledgement ≤5 seconds; no fixed turn duration or tool-call cap; cancel after ten minutes
without a runtime event, with the exact reason; cancellation acknowledgement ≤5 seconds plus a five-second termination
grace; recovery within thirty seconds of readiness. Runtime events renew the inactivity clock; typing and progress
timers do not. One topic progress message is updated no more often than every thirty seconds.

Start with four concurrent conversations, one run each, round-robin routes; pending caps are fifty per route and 1,000 total. Safe dependency attempts total at most three within the original deadline; delivery attempts at most five per episode across restarts. Task notifications follow a fresh observed transition within fifteen seconds when Telegram is reachable.

SQLite uses WAL, `synchronous=FULL`, one writer, local disk and bounded transactions. Raw conversations/native inactive sessions retain thirty days; audit metadata 365 days and text ninety days; reconciled receipts ninety days; diagnostics seven days; temporary attachments seven days, 20 MiB each. Preserve active/unresolved work and replay-preventing tombstones.

Disk budget is 2 GiB; at 80% prune eligible data and alert. At the hard limit preserve unresolved records, stop admission and report pressure; never advance an unstored input's cursor. Report possible Telegram retention gaps. Use SQLite snapshots and prove restoration separately.

## 8. Access, credentials and audit

**Mandatory:** [access detail](docs/runtime-contract.md#8-access-and-observability) binds current user/route/project/action grants, revalidation at effect dispatch and delayed delivery, protected key storage and scrubbed output. Reading grants no mutation right; revocation cancels pending unauthorized effects without erasing prior effects.

Store named keys once in owner-only `keys.json` (0600 or Windows equivalent), with purpose, tool/holder scopes and last use. Inject at dispatch; never expose values through model, journal, telemetry, logs, replies or task bodies. CLI list shows metadata only; CLI add/grant/revoke are the sole key-entry path.

Audit each admitted CLI/Telegram request in the input transaction, retaining actor/grants, route/project, request edits/cancellation, times, model/version/agentq, tools, effects, task references, outcome and delivery. Join CoreQ and TaskQ by request ID; tell the person the retention at admission. Audit reads are themselves recorded.

Owner and `audit` administrators read via CLI; people read their own `/audit` only in DM without the model. Group/topic audit shows nothing. Analysis exports omit message text unless specifically approved. Optional Langfuse capture needs approved scope, redaction and bounded export; failure never blocks work or owns the only outcome copy.

## 9. CLI, releases and service operations

**Owner, task #163:** one code catalog owns capability names, areas, purposes, examples and required
grants for slash controls, operator commands and manager tools. Help and manager tool descriptions
render from that catalog; schemas and dispatch checks remain at their existing boundaries (R13,
R17). Areas are conversation, projects, tasks, reads, people and operations. Slash commands are
deterministic controls; plain requests select authorized manager tools. Operator-only commands are
labelled and never imply chat authority. Tool names use `area_verb`; existing names remain
compatible.

`/help` and `pilotq help` show short grouped commands, an introduction, how to ask, three examples on separate lines and one line per
accessible area. `/help AREA` and `pilotq help AREA` show purposes, examples and grants for that
area. Hide operator-only controls from ordinary chat callers and unavailable commands from every help view. Hide capabilities outside current grants; end with one line explaining how to request access,
with the TaskQ PM catalog read live only for `/help tasks` or `pilotq help tasks` through its
machine-readable help. Missing, old or unreadable TaskQ help produces one availability line, never a copied catalog. Help follows the caller's
language (English and Russian deterministic text); external TaskQ text remains source text.

Projects cover accessible list, current selection, switch, fresh status, connect
(`project_register`) and disconnect (`project_disconnect`). Current selection uses
`project_current`. Disconnect requires `chat` and `projects`, an explicitly named registered project
in the caller's own request, the active run fence and the same bounded audit/receipt path as §5
administration. It removes only the PilotQ registration and matching conversation/route defaults,
with atomic readback and receipt. It never deletes repositories, boards, files, grants, task
subscriptions, saved plans or prior receipts. Bound pending work refuses dispatch while registration
is absent; reconnect requires the normal verified registration path. Repeated operations return
their receipt; changed arguments in one request are refused. CLI project operations can also use
`pilotq ask` through this same boundary. A5 live Telegram qualification belongs to the PM under §9;
worker proof remains local/CI.

**Mandatory:** [service detail](docs/service-contract.md) defines CLI controls, host minimums, diagnostics, incidents, update drain, schema compatibility and release qualification. Built-in controls bypass the model backlog, retain admission/audit and support Telegram `@bot` suffixes. Unknown commands return usage errors.

`/help`, `/version`, `/doctor`, `/model`, `/restart`, `/skills` and `/skill` keep their specified bounded paths. Model selection uses the configured Codex catalogue, persists per conversation across reset/restart and affects only subsequently claimed runs. Catalogue presence proves no subscription entitlement; no model/provider fallback or worker-default change exists.

Only the protected owner restarts the installed service; acknowledge, drain and report completion after healthy restart. Source runs refuse. No native manager skills are approved; adding one needs a contract decision. CLI asks use the same controls. Every writing command prints readback, supports JSON and fails nonzero on failure.

Doctor is read-only: bot identity, isolated ChatGPT login, complete matching TaskQ board, required executable versions and bounded local voice decode/recognition probe. It never polls updates, changes grants or starts a model. Offline selftest remains setup-independent; setup success proves neither subscription entitlement nor full task flow.

Supported managed host: Linux x86_64. Minimums: Codex 0.162.0, TaskQ CLI 4.6.46/service 4.6.40, ffmpeg 6.1.0, sherpa-onnx-offline 2.0.1, curl 7.68.0, systemd 245, procps-ng kill 3.3.17, hostname 3.23 and Git 2.25.0 for registered releases. Each approved command/release executable declares its own minimum.

Use executable `--version`; only tools without that command use owner-provisioned version/SHA256 receipts. Verify recognizer bytes before trusting its receipt. Missing/unparseable/old tools refuse installed startup, install and update; report an exact operator remedy. Never provision tools, elevate privileges or change logins automatically.

Retain CoreQ v0.14.6 and journal schema 12; open schemas 0–12 through migrations, refuse newer schemas and refuse unsafe downgrade. The installation owner owns the user service/configuration. Releases use CoreQ versioned directories, checksum verification, selftest and rollback after three unhealthy starts.

Publish only by fast-forwarding `live` to a merge-queue-accepted main commit. That workflow builds/verifies static linux-x86_64 and publishes tag, binary, SHA256SUMS and latest.json. Tags alone do not publish. The PM updates the test pilot and records source/binary identity, status, selftest, doctor and request/journal continuity; production cutover is not authorized.

Check for updates every five minutes, persisting attempt time; use a separate maintenance thread and one OS-held lock. Drain gates CLI/Telegram claims atomically, waits up to sixty-five seconds and refuses incomplete activation.
Report blocking request identity, conversation and age. Retire terminal orphan slots and slots without runtime events
for §7’s ten-minute window, preserving uncertain effects; active runtime events keep a slot live. Failed download/checksum/selftest resumes admission on the current release. Scheduled activation exits 75. Pause/resume affects automatic updates only.

Rollback and service incidents use the durable owner-DM outbox only. Persist markers and episode deduplication across restarts. Report failed/restored versions after healthy
rollback; successful updates stay silent. Model incidents need two failed runs; other sources need two minutes, with fresh recovery, six-hour per-key and ten/day limits. Undrained
gaps over five minutes produce one after-start notice.

**Development exception:** the shared release contract's development-host separation is waived only for merged, released builds on the existing PDS 18 (`pds18.ufobe.com`) live service, with the installation owner unchanged, drain and compatible-schema rollback. Workers verify locally/CI and never install live. Unfinished branches and Hermes cutover remain unauthorized.

PM live qualification includes real authorized DM/topic controls, model selection, answer-card actor/button removal/repeated-callback screenshots and task-specific
file/status/release checks retained in linked detail. Before release all automatic-management suite cases and cleanup pass on a unique reserved `pilotq-e2e-*` host without ordinary
workers. Simulated events qualify management, not real implementation/review.

## 10. Adoption and migration

**Mandatory:** [adoption sequence](docs/adoption.md#10-adoption-sequence) owns ordered qualification, pilot gates, reviewed migration inventory, restore proof, poller handoff and rollback. The human trial supplements scripted evidence; gate 4 retains its separate pilot start. Neither ended trials nor successful releases authorize production cutover.

## 11. Qualification boundaries

**Mandatory:** [qualification gaps](docs/adoption.md#11-decisions-still-open) retain unresolved subscription entitlement, cutover media parity, final retention/capacity and permitted telemetry content. Do not implement or deploy an unresolved proposal as an approved decision. Qualify agentq only through its dynamic-tool adapter and measured gate; the native-compaction pilot baseline remains.

## 12. Lifecycle and recovery protocol

**Proposed:** [lifecycle detail](docs/runtime-contract.md#12-proposed-lifecycle-and-feedback-protocol) translates R5/R11 into request phases, independent transport/activity/delivery clocks, deadline supervision, controls, capability validation and reconciliation. It does not add a task state machine or authorize installation, model calls, queue activation or consumer replacement.

**Mandatory:** [Telegram feedback](docs/telegram-feedback.md) binds manager instructions, questions, layout, reactions, drafts, images and voice. Startup readiness needs identity, permissions, route configuration, storage, poller lock and reconciliation. Partial tool arguments, unknown terminal semantics and empty useful output never prove success.

Use one bounded journal reconciler at startup and periodically. Keep external calls outside transactions; one failing route cannot block others. Persist retries, deadlines and account/workspace binding; auth changes never silently change identity or enable paid APIs. Planned restart fences/drains manager trees without stopping TaskQ workers; external host monitoring handles a dead PilotQ process.

## Linked specifications and evidence

- [docs/conversation-contract.md](docs/conversation-contract.md): admission, route identity, delegation and follow-up rules.
- [docs/runtime-contract.md](docs/runtime-contract.md): architecture, durable effects, memory, budgets, access and lifecycle protocols.
- [docs/integration-contract.md](docs/integration-contract.md): TaskQ, project scopes, files, reads, administration, onboarding and DEV/LIVE releases.
- [docs/service-contract.md](docs/service-contract.md): CLI, host requirements, incidents, release flow and development exception.
- [docs/conversation-flow.md](docs/conversation-flow.md): authoritative presentation stages and cards.
- [docs/taskq-tool-contract.md](docs/taskq-tool-contract.md): first TaskQ tools, fences, grants and refusals.
- [docs/telegram-feedback.md](docs/telegram-feedback.md): restricted manager instructions and Telegram media/feedback protocol.
- [docs/adoption.md](docs/adoption.md): ordered adoption, migration and unresolved qualification boundaries.
- [docs/acceptance.md](docs/acceptance.md): product cases, metrics, automatic-management suite and qualification evidence.
- [docs/tester-guide.md](docs/tester-guide.md): operator installation, provisioning, checks and current limitations.
- [docs/status-notifications.md](docs/status-notifications.md): service-incident projections and deferred notification stages.
- [docs/reply-files-145.md](docs/reply-files-145.md): PM live file-handoff qualification recipe.
- [docs/trial-42.md](docs/trial-42.md): request-linked human-trial observations.
- [docs/research.md](docs/research.md): runtime and Telegram mechanism evidence.
- [docs/mechanism-matrix.md](docs/mechanism-matrix.md): source mechanisms, scoped proposals and acceptance mapping.
- [docs/core-options.md](docs/core-options.md): architecture alternatives and subscription-runtime findings.
- [docs/bootstrap.md](docs/bootstrap.md): repository setup observations.
- [docs/projects.md](docs/projects.md): project relationships without implicit registration or grants.

<!-- contract-budget: bytes=61440 lines=300 -->
