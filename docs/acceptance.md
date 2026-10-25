# PilotQ acceptance cases

These cases verify [pilotq.md](../pilotq.md), not a separate product contract. Status: proposed; first-slice results are in [First-slice qualification](#first-slice-qualification). Owner requirements R1–R17 are defined there; numerical targets are proposed in sections 7 and 12. Review cases before implementation. Test accounts and separate credentials must be isolated from the production bot and production queue. The [mechanism matrix](mechanism-matrix.md) maps 60 scope decisions to these cases and names the source evidence.

## Functional cases

| ID and requirement | Initial state and intent | Action | Observable result and exception |
|---|---|---|---|
| U01 R1 R5 | Authorized DM; new conversation | Ask a simple question, then refer to its answer | Correct follow-up in the same conversation; each request has a terminal outcome and delivery state |
| U02 R1 R6 | Two users, ordinary group and two forum topics | Interleave distinct requests | Preserve configured shared/per-user scope, real authors and reply destinations; no cross-route content |
| U03 R2 | Granted short diagnostic target | Ask for a bounded service/log check | Real tool evidence, source time and concise answer; failure says what could not be checked |
| U04 R2 R5 | Granted small note mutation | Ask for one update; crash after the external write | Readback proves one update; uncertain effects are not replayed blindly |
| U05 R3 | User asks only to plan | Discuss and revise the plan twice | No task starts; authorization of revision 2 cannot start changed revision 3 |
| U06 R3 R8 | Authorized plan and qualified TaskQ board | Create held task, read it back and start | One task URL; TaskQ worker and reviewer own execution; final reply cites authoritative evidence |
| U07 R3 R8 | Worker asks current question | Relay it; authorized user answers | Correct board/task/attempt/question revision resumes; manager never invents the owner's answer |
| U08 R3 | Old question button or different user's forwarded decision | Submit answer after question changed | No write; show current question or lack of authority |
| U09 R4 | Group usually discusses project A | Ask for B, then use an ambiguous ticket number | First action uses B's registered target; ambiguity is clarified, never guessed from stale preference |
| U10 R3 R8 | Task running independently | Restart manager, later ask using issue URL | Task continues; current status and attempts resolve without a duplicate create |
| U11 R2 R3 | Short action exceeds budget | Reach deadline or tool-call limit | Preserve partial evidence; delegate remaining work within existing scope or request the missing decision |
| U12 R1 R5 | A conversation has an active slow turn | Send follow-up, then `/stop`; use another topic | Input retained; scoped cancellation; second topic stays responsive; stale runtime cannot mutate |
| U13 R3 R8 | Worker is running and admission is open | Ask to pause queue, then to park a specific task | Report actual distinct transitions; queue pause does not claim worker termination |
| U14 R1 R6 | Edited message, voice/image/document or removed topic | Receive input or try to deliver | Revision is explicit; supported media works, unsupported media gets a clear outcome; no silent loss or broad rerouting |
| U15 R3 | Three-ticket authorized plan | Lose network after creating ticket two | Existing tickets remain held; restart reconciles same IDs; start only after complete readback |
| U16 R1 R5 | Model unavailable | Use `/status`, `/stop`, `/new` | Recovery controls work without an LLM; `/new` retains active task references and receipts |
| U17 R1 R10 | Two authorized people with different preferences, including a private DM fact | Remember facts, restart, inspect and correct one person's profile, then use a shared topic | Profiles remain distinct; correction persists; shared reply contains no private DM fact; changing a preference does not change grants |
| U18 R10 | Person has been ignored; administrator has user-management authority | Grant conversation access, inspect it, change project scope, then revoke | Exact user ID and administrator are recorded; only new authorized requests run; ignored messages are not replayed; conversation access alone cannot mutate projects; revoked input is ignored |
| U19 R13 | Operator shell on the host; Telegram DM from the same owner | Ask the same question with `pilotq ask` and in Telegram | Both use one grant check and audit path; CLI writes print readback, `--json` parses, failure exits nonzero |
| U20 R15 | Administrator holds `users` and `projects` grants | Tell PilotQ to remember a person with a grant, then to register a project | Exact user ID and grant read back; project goes through TaskQ registration and reads back; without the grant nothing is written |
| U21 R16 | Key granted to one person and one tool | Run the tool for that person, then for another person; make the tool echo the value | First call works; second is refused at dispatch; the value never appears in model context, journal, telemetry, logs or replies |
| U22 R14 | Published release with a broken selftest, then a good one | Run `pilotq update` during an active turn | Broken release is refused before the switch; good release waits for the drain, switches and reports the new version in `status.json`; three unhealthy starts return to `previous` |
| U23 R17 | Two granted people in a DM and a forum topic, one CLI operator | Each asks something; one request files a TaskQ task and one fails; then run `pilotq audit` as the owner, as each person and as a person without the `audit` grant | Every request has one record with who, when, where, text, tools, task link and outcome; its request ID finds the CoreQ trace and the task; each person sees only their own records; no key value appears; each audit read is recorded |

## Fault and authorization cases

For every injected failure, record the exact version, operation ID, fault point, restart, native events, board evidence and observed Telegram result. Repeat each crash point at least ten times; more repetitions are justified by a failure, not by a desire to report a larger count.

| ID and requirement | Fault point or hostile input | Required outcome |
|---|---|---|
| F01 R5 | SIGKILL before inbox commit, after commit, before/after cursor advancement | No durably acknowledged input disappears; redelivery uses the same request |
| F02 R5 | Duplicate update; two processes start with one token | One admitted request; one local poller owns the lock; cross-host token conflict becomes visible |
| F03 R5 | Provider stalls, disconnects before/after tools, or ends without terminal event | Bounded failure/recovery; no false success and no unverified mutation replay |
| F04 R5 | Crash before effect, after effect but before receipt, after receipt | Reconcile effect or mark uncertain; repeated task creation never follows missing local evidence |
| F05 R5 | Telegram sends but its response is lost; formatting rejection; 429; temporary outage | Durable result remains; bounded delivery retry/plain-text fallback; possible visible duplicate is tracked, execution is not repeated |
| F06 R5 | Network or auth failure in polling while outbound still works | Ingress health becomes degraded from stale successful-poll evidence; host alert path works |
| F07 R3 R5 | Board unavailable or create-claim prerequisite absent | Conversation continues; task creation stays unsent/unknown with clear reason; no plain-add fallback |
| F08 R5 | Disk full, corrupt database, truncated native transcript or newer schema | Preserve evidence, stop unsafe admission, expose unknown; rebuild only from proven inputs/receipts |
| F09 R1 R4 | Malicious quoted text, issue, tool output, ticket URL or anonymous sender | No grant expansion, arbitrary command injection, project redirection or cross-route disclosure |
| F10 R1 R5 | Revoke user or project grant while work/delivery waits | Revalidate before dispatch/delivery; pending unauthorized effects do not execute; completed effects remain recorded |
| F11 R5 | Restore journal backup on test host; old runtime survives restart | Reconcile board and effects before resuming; stale runtime fenced; no two active consumers |
| F12 R5 | Saturate one route and fill global backlog | Fair service for other routes, recorded busy outcome, bounded storage; no accepted work silently pruned |
| F13 R5 R6 | Bot offline beyond Telegram retention; same-token cutover/rollback | Report possible input gap; preserve locally accepted requests; no overlapping consumers or repeated TaskQ admission |
| F14 R3 R8 | Old TaskQ notification arrives after newer completion | Ignore stale projection; no stale question sent as current; result keeps originating route |
| F15 R1 R10 | Unknown/revoked user sends DM, group/topic message, mention, reply, command and callback | No outbound response or typing, model call, tool effect or profile update; body is absent from shared context; minimal ignored disposition allows polling to progress |
| F16 R10 | Stranger copies an allowed username; existing user changes name; group admin or model claims user-management authority | Stable ID preserves the real user's identity; impersonation and ungranted administration fail; granting an entire group never bypasses individual admission |
| F17 R10 | User tries to edit another person's profile or change rights through a remembered role; grant is revoked during a run | Profile/grant boundaries hold; effects and delivery recheck current grants; unauthorized pending work is blocked, completed receipts remain |

## Context cases

| ID and requirement | Test | Required evidence |
|---|---|---|
| C01 R4 R7 | Long conversation across at least three compactions | Project corrections, prohibitions, pending questions and task links remain correct |
| C02 R7 | agentq compresses a large tool result containing one important error | Model sees the error and a usable reference; recovered original matches its content hash |
| C03 R7 | Compression fails, times out or would overflow context | Original or bounded recovery path; no silent constraint loss and no infinite retry |
| C04 R1 R7 | New message arrives during manual/automatic compaction | One serialized run; no stale branch or mixed authorship |
| C05 R1 R7 | A private result reference is requested from another route | Access denied without revealing private original data |
| C06 R5 R7 | Native session is damaged after acknowledged direct mutation | Restore discussion and verified receipt without re-executing the mutation |
| C07 R1 R7 R10 | Correct or forget a person's fact, then compact, resume an old session and restart | Current profile revision wins; superseded fact does not return as current memory; other users and private visibility remain isolated |

Run the same scenarios with agentq disabled and enabled. Fix model/provider/settings and runtime versions for each comparison; repeat representative model-dependent cases three times. Compare output quality and real provider usage separately from transport correctness.

## Lifecycle cases

These twenty scenario families close gaps found in the source review. They supplement U/F/C cases instead of treating typing, retries or an upstream module name as proof of reliability. All are **proposed, not executed**. Root [section 12](../pilotq.md#12-proposed-lifecycle-and-feedback-protocol) owns their behavior.

Each trace records initial request/run/effect/delivery state, authorized actor and route, input event, injected fault, expected persisted state, user-visible output and permitted external effects. Use fake clocks, Telegram, Codex and TaskQ for the cheap deterministic suite. Qualify the corresponding real transport, subscription, process and storage behavior separately at release time.

| ID and requirement | Initial state and action | Fault or variation | Observable acceptance evidence |
|---|---|---|---|
| L01 R5 R11 | Send a request while a run slot is busy; then free capacity | Fast answer, queued request, failed inbox commit, waiting for a person or TaskQ | Acknowledgement follows durable commit; queued is not running; fast answer suppresses redundant acknowledgement; waiting has a reason and does not trigger a hung-run warning |
| L02 R1 R5 R11 | Start a Codex turn in a forum topic | No first event; active events; stale activity; wait; cancel; finish; typing API failure | Typing stays in the correct topic, follows section 12's freshness/rate bounds and stops on wait/stale/terminal state; its timer/failure never extends a deadline, creates work or blocks a final reply |
| L03 R5 R11 | Emit factual tool/task progress, a stall and subsequent recovery | Event burst, repeated error, service restart, recovery without a successful probe, sensitive text in errors | Coalesced status reflects evidence; incident notices deduplicate across restart; no raw secrets/reasoning; a reset timer alone cannot claim recovery; final/question/failure is not starved by edits |
| L04 R5 R11 | Spawn a Codex process that never emits an event | Startup hangs, or stops before activity state is initialized | First-event watch still fires within the configured bound; request becomes recoverable/failed with notice; slot and process resources are released or fenced |
| L05 R2 R5 R7 R11 | Run a chatty model, a quiet tool and compaction under the same request budget | Endless tokens; repeatedly ineffective compaction; no events during bounded legitimate tool work | Absolute deadline and tool-call budget apply despite activity; no unbounded retry/compaction loop; legitimate tool waiting is diagnosed accurately; only remaining work is delegated |
| L06 R1 R5 R10 R11 | Fill the normal queue, then issue authorized stop/status/reset | Cancel queued request; interrupt running tool; old runtime emits after `/new`; unauthorized control command | Control remains responsive within budgets; generation blocks stale effects/replies; old receipts remain queryable; revoked/unknown actor gets no effect or reply; reset does not recreate or lose active TaskQ work |
| L07 R2 R5 R11 | Deliver native events and a candidate tool call | Partial but parseable arguments; invalid target; unknown control event; EOF without terminal; empty answer; optional unknown diagnostic event | No partial/invalid effect; no guessed success; optional diagnostic can be ignored safely; explicit failed/uncertain outcome retains evidence; native tools cannot bypass the verified effect boundary |
| L08 R5 R9 R11 | Provider or transport fails during a request | 401, exhausted subscription, 429 with retry-after, transient 5xx; restart during wait; SDK retries also enabled | Auth/quota/transport reasons differ; no busy loop or billed-key fallback; one logical-operation budget survives restart; server wait honored; stale authorization rechecked before resumed effects |
| L09 R1 R5 R9 | Resume a thread after subscription credentials expire | Renew same account; auth revoked; login returns a different account/workspace | Explicit required-login state; refresh does not discard conversation; different account binding is not adopted silently; old effects are reconciled before continuation; live entitlement/renewal is a separate release check |
| L10 R1 R5 R9 R10 | Start with a pinned Codex and a reviewed Telegram manifest | Wrong bot token identity; missing topic/privacy permission; missing required tool; incompatible native protocol; unknown subscription deployment eligibility | Component readiness shows the precise missing prerequisite; no unsafe executable admission; remaining read-only status is honest; actual model-visible tools match the reviewed inventory; mocks do not establish production entitlement |
| L11 R5 R11 | Final answer needs several messages with Unicode and code blocks | Crash after chunk 1; chunk 2 rejected; lost acknowledgement; Markdown/HTML failure; 429 | Stable persisted text/chunk IDs; receipt-aware ordered continuation; only ambiguous chunks risk bounded duplicates; formatting fallback preserves meaning; no model/tool rerun or false delivered state |
| L12 R5 R11 | Queue slow progress edits and then a terminal result | Reorder responses; late old edit; deleted status message; exhausted edit retry | Final revision cannot be overwritten; replacement stays in the same authorized route; superseded cosmetic work drops without dropping the final result or regenerating it |
| L13 R1 R6 R10 | Serve DM, ordinary group, General and named forum topics | Topic closes/deletes; bot removed; group migrates; chat permission revoked before send | Identity/routing preserved; delivery is held with an actionable incident; no fallback to wider chat or private DM; administrative remap is explicit and does not change the original effect authority |
| L14 R1 R5 R10 | Interleave text, album parts, edits and replies from two people/topics | Missing album part; delayed part; edit after execution starts; unsupported attachment | Each admitted input remains accounted for; bounded assembly preserves author/route/revision; no cross-person merge or swallowed text; unsupported content gets an explicit outcome only for authorized senders |
| L15 R3 R5 R11 | Plan restart/update with accepted input, a running manager and pending output | Graceful drain expires; child ignores interrupt; restart mid-effect; OS kills parent | Persisted work survives; stale descendants are terminated/fenced; original pending requests reconcile; no startup backlog discard; TaskQ worker continues under TaskQ ownership; new readiness follows recovery checks |
| L16 R3 R5 R8 | Persist work, then lose its timer/event notification | Add event during reconciliation; one route repeatedly throws; TaskQ event missed during outage | Periodic scan recovers work without a user nudge; dirty key reruns once; one failing key does not block others; no duplicate create/effect/delivery obligation; external call does not hold the DB transaction |
| L17 R5 | Run deadlines/backoff, then change wall clock and restart | Clock jumps forward/backward; stale persisted timestamps | Monotonic live deadlines remain bounded; restart does not reset attempts or grant unlimited extra run time; invalid time evidence is explicit rather than healthy |
| L18 R5 R11 | Independently break ingress, dispatch, model, delivery and the service process | Outbound works while polling is dead; polls succeed while dispatch is stuck; event loop blocked; host monitor cannot query service | Correct component becomes stale/unknown; external monitor detects total service failure; no claim of successful Telegram alert when delivery is down; recovery notice requires fresh evidence; no alert/restart storm |
| L19 R1 R5 R10 | Stress one route and resource budgets during another normal conversation | Oversized text/file/tool output; storage contention/full disk; repeated invalid sender input; failed optional telemetry | Bounds and fair service hold; uncommitted input is not acknowledged; raw secrets/message bodies do not leak into diagnostics; resources/child handles are released; telemetry failure does not block the core flow |
| L20 R5 R6 | Upgrade a pinned PilotQ/Codex/config/schema set with accepted work, then roll back | Candidate fails startup; older binary sees newer schema; restore a backup taken during work | Current/previous version and compatibility are explicit; incompatible downgrade stops safely; receipts and native mappings survive compatible rollback; restored effects reconcile with TaskQ; actual Linux service and storage proof is separate from fake traces |

For each applicable L case, vary duplicate/stale/out-of-order events, permission revocation and cancellation at the same persistence boundary. Reuse a trace family where it proves the same chain; do not multiply tests solely to inflate the count. A power-loss guarantee needs storage-level evidence in addition to process-kill tests.

## Metrics

All metrics are **new/proposed** for PilotQ. Langfuse's [evaluation guidance](https://langfuse.com/academy/evaluate/choosing-what-to-evaluate) supports choosing outcome-linked measures. Local deterministic evidence is primary; an LLM judge does not decide delivery, authorization or task completion. Review this metric set before implementing evaluators.

| Priority | Metric and proposed gate | Source and measurability | Decision informed |
|---|---|---|---|
| P0 | Zero lost durably accepted inputs | Reconcile inbox records with explicit request outcome and delivery state in all crash cases | Block adoption on loss |
| P0 | Zero unauthorized/cross-route effects or disclosures | Fixed identity/route fixtures and effect receipts | Block adoption on any violation |
| P0 | Zero duplicate task creation or blind replay of uncertain effects | Stable operation IDs plus authoritative board/target readback | Block adoption on any violation |
| P0 | Zero false success claims in fault fixtures | Compare native terminal state, tool readback, task state and user response | Reject runtime/adapter behavior |
| P0 | Every required mechanism has an acceptance result on the exact candidate; exclusions and unresolved choices are explicit | M01–M60 traceability plus U/F/C/L evidence; a source review is not a result | Block sign-off on missing evidence for an in-scope mechanism |
| P1 | At least 95% correct simple requests without human nudges | Owner-reviewed outcomes over at least 200 normal simple requests; required domain clarification is not a recovery nudge | Qualify conversational usefulness |
| P1 | 100% of accepted requests remain accounted for; report achieved user outcomes separately | Includes dependency outages; completed, waiting, failed and uncertain counts remain distinct | Prevent good error accounting from masquerading as successful work |
| P1 | Receipt, acknowledgement, cancellation and notification latency meet contract budgets | Timestamp spans with p50/p95, pilot load and dependency availability stated | Set capacity and operating limits |
| P1 | Forced recovery within contract budget; no manual nudge for pure reads | Fault evidence and interventions per 100 requests | Compare candidate runtimes |
| P1 | agentq passes every fixed critical-fact/recovery case and reduces measured input use on the intended workload | C01–C07, paired provider tokens/cache data; missing usage is unknown | Enable adapter only after quality and efficiency evidence |
| P2 | Cost per successful scenario, latency and repeat-work rate | Native billed usage where available, direct provider cost where available, retries and owner interventions | Choose between passing candidates |

## Selection gates

1. **Transport and recovery:** run F01–F17 and deterministic L01–L20 traces with a fake runtime, fake clocks and board fixtures. This isolates coordinator defects without model variability. All safety invariants pass; live/platform portions remain separately unqualified.
2. **Real vertical slice:** run U01–U18 and the transport/auth/process/storage portions of L01–L20 on a separate Telegram bot and safe TaskQ test board with real selected Codex subscription access. Include actual worker ask/resume/review; mocks do not qualify this path. Validate latest authorization and task state before each mutation.
3. **Context:** run C01–C07 and paired agentq comparisons. Missing adapter is an implementation prerequisite, not a reason to reject the architecture without investigation.
4. **Pilot:** at least seven days and 500 accepted requests, including at least 200 simple requests, two users, three routes, two projects and ten TaskQ tasks. Deliberate faults remain a separate result set. Record all manual interventions and provider outages. Small samples do not establish long-term uptime.
5. **Production decision:** choose the passing Codex subscription integration with the lowest maintenance burden and manual intervention rate, following R9 and the proposals in [core options](core-options.md). A candidate that fails these gates is not qualified; changes to the product direction return to the owner before implementation.

## First-slice qualification

Results for the [section 10](../pilotq.md#10-adoption-sequence) step 2 slice, task [#14](https://github.com/taskqorg/pilotq/issues/14). Deterministic traces run in `cargo test --locked` with a fake Bot API (`tests/telegram.rs`), a fake Codex app-server script, journal calls that take explicit times (fake clock) and fixtures. Test names below are in `src/journal.rs` (`journal::`), `tests/telegram.rs` or `tests/update.rs`. Fakes do not qualify real Telegram, Codex, launchd or storage behavior; the live results are recorded separately below.

### Step 2 qualifications

| Qualification | Traces |
|---|---|
| Duplicate input | `f02_duplicate_lock_and_conflict`, `journal::duplicates_admit_once_and_strangers_leave_no_body` |
| Timeout | `f03_provider_faults_are_bounded`: no first event, silence, endless output past the absolute deadline |
| Crash | `f01_crash_points_keep_each_input_once` (SIGKILL around the inbox commit and cursor), `crash_during_turn_requeues_once` (SIGKILL during a Codex turn) |
| Update during a turn | `u12_stop_is_scoped_and_fences_the_run` (a message during a turn); `update_waits_for_the_running_turn` (`pilotq update` drains the running turn before it switches) |
| Two-route isolation | `u12_stop_is_scoped_and_fences_the_run` (a second topic answers while the first is busy), `journal::u02_route_session_policy` |

### Fault and lifecycle traces

"Partial" names what the first slice proves; the rest belongs to the named step. Live portions stay separate.

| Case | First slice | Traces or owning step |
|---|---|---|
| F01 | Yes | `f01_crash_points_keep_each_input_once`, `crash_during_turn_requeues_once` |
| F02 | Yes | `f02_duplicate_lock_and_conflict`, `journal::duplicates_admit_once_and_strangers_leave_no_body` |
| F03 | Yes | `f03_provider_faults_are_bounded`, `journal::runs_reconcile_and_fence` |
| F04 | Partial | A started native tool reads uncertain and is never replayed (`f03_provider_faults_are_bounded`, `journal::runs_reconcile_and_fence`). Effect receipts: step 4; task creation: step 3 |
| F05 | Yes | `f05_delivery_faults_are_bounded`, `journal::outbox_keeps_route_order_and_bounds_retries` |
| F06 | Partial | `f06_poll_failure_is_visible_while_delivery_works`. Host alert path: step 6 |
| F07 | No | Step 3 (TaskQ board) |
| F08 | Partial | `journal::newer_schema_and_corrupt_journal_are_refused`. Disk budget and full disk: step 6; native transcript rebuild: step 5 |
| F09 | Partial | `journal::strangers_and_claims_change_nothing` (anonymous sender, authority claims in text); Telegram text reaches Codex only as a JSON argument. Issue text, tool output, ticket URLs and project redirection: steps 3 and 4 |
| F10 | Partial | `journal::revoked_work_is_not_dispatched_or_delivered` (user grant). Project grants: step 4 |
| F11 | Partial | Stale run fenced (`u12_stop_is_scoped_and_fences_the_run`, `journal::runs_reconcile_and_fence`), one poller (`f02_duplicate_lock_and_conflict`). Backup restore: step 6 |
| F12 | Yes | `journal::backlog_is_bounded_and_fair` |
| F13 | No | Step 6 (cutover and rollback) |
| F14 | No | Step 3 (TaskQ notifications) |
| F15 | Yes | `journal::strangers_and_claims_change_nothing`, `u18_cli_grant_admits_only_new_requests`, `f02_duplicate_lock_and_conflict`. The first slice offers no inline buttons; callback queries arrive with step 3 |
| F16 | Yes | `journal::strangers_and_claims_change_nothing` (a stranger reusing the owner's display name), `journal::u18_grant_lifecycle` (grants are per user ID; no group grant exists) |
| F17 | Yes | `journal::u17_profiles_distinct_and_private`, `journal::revoked_work_is_not_dispatched_or_delivered` |
| L01 | Partial | Acknowledgement only after commit (`f01_crash_points_keep_each_input_once`), queued is not running (`journal::backlog_is_bounded_and_fair`), acknowledgement before the answer (`u01_follow_up_resumes_the_conversation`). The optional fast-answer suppression is not implemented. Waiting for a person or TaskQ: step 3 |
| L02 | No | Step 3, with progress status messages (no typing indicator in the first slice) |
| L03 | No | Step 3 |
| L04 | Yes | `f03_provider_faults_are_bounded` (no first event) |
| L05 | Partial | Absolute deadline despite endless tokens (`f03_provider_faults_are_bounded`). Tool-call budget: step 4; compaction: step 5 |
| L06 | Partial | `u12_stop_is_scoped_and_fences_the_run`, `u16_controls_work_without_the_model`, `journal::runs_reconcile_and_fence` (`/new` generation), `journal::backlog_is_bounded_and_fair` (control on a full route), `journal::strangers_and_claims_change_nothing` (stranger `/stop`). TaskQ work under reset: step 3 |
| L07 | Yes | `f03_provider_faults_are_bounded`: EOF without terminal, empty answer, unknown optional event, refused server request. The real Codex tool inventory is a live check |
| L08 | Partial | Quota failure stays failed and no API key reaches Codex (`f03_provider_faults_are_bounded`). Distinct auth, quota and 429 handling with retry-after: step 5 |
| L09 | No | Step 5; the login state is part of the live run |
| L10 | Partial | Unsafe or malformed `config.json` stops startup. Bot identity and permission checks against a manifest: step 6 |
| L11 | Yes | `journal::long_reply_is_chunked_in_order`, `f05_delivery_faults_are_bounded` |
| L12 | No | Step 3 (status message edits) |
| L13 | Partial | A blocked chat is held, never rerouted (`f05_delivery_faults_are_bounded`). Administrative remap: step 5 |
| L14 | Partial | Unsupported media gets an explicit outcome and service messages are not input (`journal::strangers_and_claims_change_nothing`, `service_messages_are_not_input`); no cross-person merge (`journal::u02_route_session_policy`). Edits and albums: step 5 with the media decision (section 11) |
| L15 | Partial | `crash_during_turn_requeues_once`, `update_waits_for_the_running_turn`, process-group kill at a deadline (`f03_provider_faults_are_bounded`). A Codex child of a SIGKILLed service ends only on stdin EOF. Linux service proof: step 6 |
| L16 | Partial | The manager rescans the journal every 0.5 s and delivery rescans the outbox, so a lost wake loses no work (every service trace above). Missed TaskQ events: step 3 |
| L17 | Partial | Model attempts persist across restarts (`journal::runs_reconcile_and_fence`); live deadlines use the monotonic clock. No trace injects a wall-clock jump |
| L18 | Partial | `f06_poll_failure_is_visible_while_delivery_works`, `f02_duplicate_lock_and_conflict`. External monitor: step 6 |
| L19 | Partial | `journal::telemetry_failure_does_not_block`, `journal::long_reply_is_chunked_in_order`, `journal::backlog_is_bounded_and_fair`. Storage full and contention: step 6 |
| L20 | Partial | `journal::newer_schema_and_corrupt_journal_are_refused`, `journal::schema_1_journal_upgrades`, `update_refuses_a_release_whose_selftest_fails`. Rollback with accepted work and restore: step 6 |

### Live test-bot run

Applicable to the first slice: U01, U02, U12, U14 (unsupported input only), U16, U17 and U18. Owned by later steps: U05–U08, U10, U13 and U15 (step 3); U03, U04, U09 and U11 (step 4).

Run on 2026-10-22 on one Mac (macOS 27.0.1), pilotq 0.1.0 at commit `56199c9` built `--release`, codex-cli 0.159.3 in PilotQ's own Codex home (subscription login, model `gpt-6.1-sol`), the separate test bot and a test forum group (General and one named topic) with the owner and a second test user. No production bot and no Hermes change.

| Case | Result | Evidence |
|---|---|---|
| A3 service | Pass | From the worker session `launchctl bootstrap gui/501` returned `Bootstrap failed: 5: Input/output error`. The owner ran `pilotq service install` in a Terminal; launchd then reported `com.pilotq.service` running, and its log reads `pilotq 0.1.0 service running`. The service runs release 0.1.0 built from `56199c9`, which predates the service-message fix |
| Real Codex turn | Pass | `pilotq ask` answered in 10.7 s; 13 Telegram turns answered in 8–11 s each and were delivered |
| U02 | Partial pass | Owner messages in General and in the topic ran as separate conversations, and each reply went to its own topic; after `routes set -1003995096980:3 shared` the topic was one conversation. **Not proven:** no update from the second user (`728914418`) reached the bot after the grant, so two people in one shared topic are untested live |
| U14 | Partial pass | A photo got the explicit unsupported reply; replies to the old group, which Telegram had migrated to a supergroup, were held with the migration error instead of rerouted. **Defect found:** Telegram service messages (group created, migration, topic created) were answered as unsupported input. Fixed; `service_messages_are_not_input` failed before the fix |
| U17 | Partial pass | Through the CLI: two people's facts stay distinct, a correction bumps the revision to 2 and leaves grants unchanged, and the administrator's readback omits the other person's private fact. Telegram profile commands are not part of the slice |
| U18 | Partial pass | Twelve messages from the ungranted second user left only ID-only dispositions and no reply. After `pilotq users grant 728914418 chat` (logged with the acting owner), none of them was replayed. **Not proven:** a new request from that person after the grant, and live revoke; revoke is covered by `journal::u18_grant_lifecycle` and `u18_cli_grant_admits_only_new_requests` |
| U19, U23 | Pass | 21 audit records (one CLI, 20 Telegram) with who, where, model and delivery; 21 CoreQ events, one per ended request |
| F05 | Live pass | A Telegram `retry-after` was honored and the reply delivered later |
| U01 | Pass | In the DM, `hi` and a follow-up ran as requests #23 and #24 in one conversation, the second resuming the stored Codex thread; both were answered and delivered after their acknowledgements. The follow-up repeated the greeting, so the run shows thread continuity, not a reply that uses the earlier answer |
| U12 | Pass | `/stop` cancelled the running request in the DM (#27) and in the shared topic (#31). Both runs ended `fenced`, and neither posted a late reply. `/status` in the topic then read `Nothing is running or queued` |
| U16 | Pass | During a running turn `/status` answered `Running: #24. Queued: none.` without the model. `/new` advanced the DM and topic conversation generations and cleared their threads; earlier requests stayed in the journal |
| U02, U18 two-user | Covered by the step 3 scripted run (synthetic users, fake Bot API; see A8 below). Earlier: open | On 2026-10-23 the owner accepted the deterministic evidence (`journal::u02_route_session_policy`, `u18_cli_grant_admits_only_new_requests`) for the first slice. A live second-user request after the grant stays unproven until a later live run |
| DM topics | Note | The test bot has topics enabled, so DM messages carry a thread ID (`1065394`); replies went to that thread. Request #22, a non-text DM update on the pre-fix build, drew the unsupported reply |

## Step 3 qualification

Results for the [section 10](../pilotq.md#10-adoption-sequence) step 3 slice, task [#18](https://github.com/taskqorg/pilotq/issues/18), against the [first TaskQ tool contract](../pilotq.md#first-taskq-tool-contract). Deterministic traces run in `cargo test --locked` with a fake TaskQ CLI (`FAKE_TASKQ` in `tests/telegram.rs`: argument log, op-id receipts, revision-checked answers, board-down and lost-response faults), a fake Codex that makes one complete dynamic tool call per turn, the fake Bot API (messages, edits, typing) and a 1 s board scan. Fakes do not qualify the real board, Codex dynamic tools or Telegram; the live run is listed separately.

| Case | Deterministic result | Traces |
|---|---|---|
| U05 | Pass | `u06_plan_revisions_then_held_create_start_and_one_status_message` (three revisions write nothing; revision 2 cannot run once revision 3 exists), `journal::plans_and_task_projections` (a revision cannot be run by the request that saved it) |
| U06 | Pass | `u06_plan_revisions_then_held_create_start_and_one_status_message`: `add --op-id` held, readback, `start --op-id <op>.start`, author, route and revision in the task, one task URL, the closing outcome links the issue |
| U07 | Pass | `u07_question_relay_and_bound_answers`: the question reaches the originating topic; the owner's reply writes `answer 19.2 --revision R --op-id` |
| U08 | Pass | `u07_question_relay_and_bound_answers`: a reply to the older question and a reply from a person without the grant write nothing and say why |
| U10 | Pass | `u10_restart_new_status_pause_and_park`: restart and `/new` keep the task; status by issue URL creates nothing; another board's URL is refused |
| U13 | Pass | `u10_restart_new_status_pause_and_park`: a queue pause reports the running worker as still running; park reports TaskQ's `later` |
| U15 | Pass | `u15_network_loss_after_ticket_two_reconciles_the_same_ids`: the board created ticket two and the response was lost; after restart the same op ID finds it, ticket three is filed, all three start only after |
| F04 | Partial | Task creation: lost create response reconciled by op ID (U15 trace). Direct-effect receipts: step 4 |
| F07 | Pass | `f07_board_unavailable_or_no_create_claim`: no `create_claim_issue` disables creation with that reason and never runs plain `add`; a down board leaves the create unsent with its reason |
| F09 | Partial | Ticket URLs of another board are refused (U10 trace); Telegram text reaches TaskQ only as argument-array items. Tool output and project redirection: step 4 |
| F14 | Pass | `u06_…` (an observation older than the stored revision changes nothing), `journal::plans_and_task_projections` |
| L01 | Pass | `u06_…`: the status message names what the task waits for (`ready: waiting for a free worker`); no hung or stall warning. Fast-answer suppression stays unimplemented |
| L02 | Partial | `u07_…`: typing goes only to the originating topic while the turn runs and stops after it. Typing is bounded by the absolute deadline, not by the last useful event |
| L03 | Partial | `journal::plans_and_task_projections`: a burst leaves one pending status update; the question and the outcome are never coalesced. Incident notices across restart: step 5 |
| L06 | Pass | `u10_…`: `/new` keeps the task and its subscription; an ungranted person's TaskQ call changes nothing. Earlier traces cover stop, status and stale runs |
| L12 | Pass | `u06_…`: one status message edited in place; the outcome is its own message and nothing edits it; `journal::…` (a superseded update is never sent). A deleted status message is replaced by a new one in the same route (`u10_…`) |
| L16 | Pass | `u10_…` (a transition made while PilotQ was down is found by the scan after restart), `f07_…` (the scan files a deferred create when the board returns), `u15_…` |
| U23 | Partial | The request that filed a task lists `add #N` and `start #N` in its audit record (`u06_…`); the task carries the request trace ID. Live check below |

### Scripted live-board run (A8)

The owner replaced the human test-bot run on 2026-10-23 (root [section 5](../pilotq.md#first-taskq-tool-contract)). `a8_live_board_run` in `tests/telegram.rs` runs the real PilotQ binary with real Codex (PilotQ's own Codex home holding only the subscription login), the real `taskqorg/pilotq` board and TaskQ CLI. Telegram is the local fake Bot API with two synthetic users, 7 (configured owner) and 8. It files and publishes a real task, so it is `#[ignore]` and runs only on request:

```sh
PILOTQ_A8_ROOT=<pilotq checkout with create_claim_issue> PILOTQ_A8_TASKQ=<taskq binary> PILOTQ_A8_CODEX_HOME=<logged-in Codex home> \
  cargo test --locked --test telegram a8_live -- --ignored --nocapture
```

It checks, in order: U18 (an ungranted message is ignored and not replayed after `users grant`), U02 (a DM per person and two people in one shared topic, each reply in its own route), U05 (a plan request saves revision 1 and files nothing), U06 (the next request files it held, reads it back and starts it; the issue carries the request trace ID, author, route and revision), then a PilotQ restart while the task is `doing`, the status message edited in place through `review`, the closing outcome as its own message, U23 (the audit record lists `add #N` and `start #N`, the CoreQ trace exists) and the issue closed as completed. A worker question gets the recommended option as a reply.

**Result, 2026-10-23, macOS 27.0.1:** pass on run 2, with one test-side assertion rechecked. Build `867fabc` (debug), codex-cli 0.159.3 (model `gpt-6.1-sol`), TaskQ CLI on the Mac host, root = the PR worktree with `create_claim_issue` #20.

| Step | Result | Evidence |
|---|---|---|
| U18, U02 | Pass | The ungranted update stayed `ignored` and was not replayed after `users grant 8 chat`; person 8's DM ran as `dm:8`; persons 7 and 8 in the shared topic ran as one `-100:5` conversation with their own authors; each reply went to its own chat and thread |
| U05 | Pass | Real Codex called `task_plan`; revision 1 saved, no plan run, no board write |
| U06 | Pass | The next request ran `task_run_plan`: [#21](https://github.com/taskqorg/pilotq/issues/21) filed held with `op-id: pilotq-196702e68ba09fa0-p1-t1`, read back, started; its scope line reads `Filed by PilotQ pilotq.request.5 for Telegram user 7 in DM 7, plan revision 1.` |
| Restart mid-run | Pass | PilotQ was killed once the status message read `doing` and restarted; the worker continued under TaskQ |
| Reviewer outcome | Pass | One status message edited in place through `review`; the closing outcome arrived as its own message and nothing edited it. The worker's PR [#22](https://github.com/taskqorg/pilotq/pull/22) merged; #21 reads `closed completed`. About 9 minutes from start to close; no worker question was asked |
| Acknowledgements | Rechecked | The run's own assertion expected a plain-text prefix, but acknowledgements are HTML (`Request <b>#N</b>`), so it failed after all checks above had passed. The journal holds one acknowledgement per request (5 of 5). The assertion is fixed |
| U23 | Pass | `a8_after_recheck` on run 2's kept install: the audit record of request 5 lists `add #21` and `start #21` with trace `pilotq.request.5`; the CoreQ store holds that trace; the issue carries it |

Run 1 failed before any board write and exposed a defect. Three requests started Codex together on a fresh Codex home, and one app-server exited with `failed to initialize sqlite state runtime`. PilotQ reported it only as `Codex exited without a terminal event`. Fix: Codex startup is serialized until `initialize` answers, and the stderr reason goes to the operator log, never to a reply (`concurrent_codex_starts_do_not_race`, which failed without the lock). The live test bot was not used. It ran the pre-step3 build and journal throughout, restored after the earlier attempt. A later owner check there is optional.

## Step 4 qualification

Results for the [section 10](../pilotq.md#10-adoption-sequence) step 4 slice, task [#23](https://github.com/taskqorg/pilotq/issues/23), against [direct actions, keys and project switching](../pilotq.md#direct-actions-keys-and-project-switching). Deterministic traces run in `cargo test --locked` with a fake `gh` (`FAKE_GH` in `tests/telegram.rs`: argument log, a token check that never records the token, issues, comments and labels in a state file, an option that makes an issue body echo the token), the fake TaskQ CLI as the registration readback, a fake Codex that makes several dynamic tool calls per turn (`multi`, or `hex` so that the person's text names nothing the calls use), the fake Bot API and the `PILOTQ_FAULT` crash points `before-effect` and `after-effect`. Fakes do not qualify GitHub, real Codex tool choice or Telegram; the live run is listed separately.

| Case | Deterministic result | Traces |
|---|---|---|
| U03 | Pass | `u03_u21_key_backed_read_and_no_value_anywhere`: the owner's read returns title, state, labels, body excerpt and `Source time: updated … on GitHub; read at …`; the audit record lists `issue_read taskqorg/pilotq#3` with the key name and the project |
| U04 | Pass | `u04_f04_owner_comment_and_label_once_and_crash_readback`: one comment carrying its operation ID and one label, each read back with a receipt; a person holding every grant but not owner gets `only the owner may comment on or label board issues`; another project is read-only |
| U09 | Pass | `u09_f09_f10_switch_ambiguity_redirect_and_revoke`: a bare `#3` with two accessible projects and no current one is refused with the list; a switch the person names makes the next bare number read that project; a project named in the message is read without switching |
| U11 | Pass | `u11_l05_budget_and_deadline_keep_evidence`: six reads in one request run five, the sixth is refused and runs nothing; a request cut off at its deadline replies with `Evidence kept: …` and offers to plan the rest |
| U21, U23 (key values) | Pass | `u03_…`: the CLI reads the value from stdin into an owner-only `keys.json`, refuses a second value for the same name and never prints it; a person without the key is refused at dispatch and `gh` never runs; the value the tool echoed reaches the model and Telegram as `[key github]`; no file under the install (journal, WAL, telemetry, Codex log), log, reply, `audit` or `keys list` output contains it. Failed with the scrub removed |
| F04 | Pass | `u04_…`: killed after the write and before the receipt, startup reads the comment back by its operation ID, records it done and tells the route that it was found once; killed before the write, the readback finds nothing and the route is told it was not written. Neither is repeated. Task creation: step 3 (U15) |
| F09 | Pass | `u09_…`: calls that name `pilotq` while the person's message does not, under a quoted message that names it, neither switch nor read it, by name or by issue URL |
| F10 | Pass | `u09_…`: after `users revoke 8 project:other` the person's next read is refused; earlier records stay. User grants: step 2 |
| U21 (key holders) | Pass | `a1_key_grants_by_route_and_project` ([#25](https://github.com/taskqorg/pilotq/issues/25)): `keys grant ro project:other` lets a person with `project:other` read that project and nothing else; `route:CHAT:TOPIC` lets the same person read from that topic only; after `keys revoke` of the project or the route the next call is refused and `gh` never runs. Person grants: `u03_…`. Failed before #25: the CLI refused `project:` |
| U11 (retries) | Pass | `a2_budget_holds_across_a_retry` (#25): four reads run, PilotQ is killed mid-turn, the restart retries the request and its four reads run one more; five reads in total, three refused, five `call` events in the audit record. Failed before #25: the retry ran all four again |
| L05 | Partial | `u11_…`: the tool-call budget and the deadline apply; the deadline despite endless tokens stays `f03_provider_faults_are_bounded`. Compaction: step 5 |

### Scripted live run (A7)

`a7_live_direct_run` in `tests/telegram.rs` runs the real PilotQ binary with real Codex in PilotQ's own Codex home, the fake Bot API with the owner 7 and a second synthetic person 8, the TaskQ CLI as registration readback, the real `taskqorg/pilotq` board and `taskqorg/specq` as the second, read-only project. The key is the host's GitHub CLI login, passed on stdin. It comments on and labels a real issue, so it is `#[ignore]` and runs only on request:

```sh
PILOTQ_A8_ROOT=<pilotq checkout> PILOTQ_A8_TASKQ=<taskq binary> PILOTQ_A8_CODEX_HOME=<logged-in Codex home> PILOTQ_A7_ISSUE=<closed pilotq issue> \
  cargo test --locked --test telegram a7_live -- --ignored --nocapture
```

It checks, in order: a key-backed read whose reply names the issue title and whose audit record holds the source time; one owner comment and the existing label `documentation`, each read back on GitHub (the comment once, by its operation ID); a non-owner's comment request that writes nothing; a named switch to `specq` and a bare `#1` read there; and no key value in any install file, log or message.

**Result, 2026-10-23, macOS 27.0.1:** pass on run 2. Debug build of this branch, codex-cli 0.159.3, TaskQ CLI on the Mac host, issue [#21](https://github.com/taskqorg/pilotq/issues/21) (closed, PilotQ's earlier A8 task).

| Step | Result | Evidence |
|---|---|---|
| U03 key-backed read | Pass | Real Codex called `issue_read`; the reply names the title of #21; the audit record holds `issue_read taskqorg/pilotq#21` with `updated 2026-…` and the key name |
| U04 owner writes | Pass | [Comment](https://github.com/taskqorg/pilotq/issues/21#issuecomment-6357737347) and label `documentation`: two effects `done` with receipts; GitHub shows the comment once by its operation ID and the label on the issue |
| A4 non-owner | Pass | Person 8, holding the key and `project:pilotq`, asked for a comment; Codex called `issue_comment`, the boundary refused it (`only the owner may comment on or label board issues`), no effect row, the issue's comment count unchanged |
| U09 switch | Pass | `project_use specq` set `dm:7` to `specq`; the next bare `#1` read `taskqorg/specq#1` |
| Key value | Pass | No install file at any depth, log or message holds the value |

Run 1 failed on the last check and exposed a defect. Every step above passed, but Codex wrote its inherited environment into `codex/shell_snapshots/*.sh` under PilotQ's Codex home, and the host's `GH_TOKEN` and `GITHUB_TOKEN` were in it. Fix: Codex starts with a cleared environment plus `PATH`, `HOME`, `USER`, `LANG`, `LC_ALL`, `TMPDIR`, `SSL_CERT_FILE` and `CODEX_HOME`. `f03_provider_faults_are_bounded` now passes `GH_TOKEN` and `GITHUB_TOKEN` to the service and failed without the fix. The run 1 install was deleted. Run 1 also left one comment ([6357708343](https://github.com/taskqorg/pilotq/issues/21#issuecomment-6357708343)) on #21; run 2 added one more, and the label was already present.

## Step 5 qualification

Results for the [section 10](../pilotq.md#10-adoption-sequence) step 5 slice, task [#26](https://github.com/taskqorg/pilotq/issues/26). The owner decided on 2026-10-23 that the pilot runs on the native compaction baseline: agentq 0.9.0 compresses only Codex shell results (`exec_command`, `shell`), PilotQ starts Codex with the shell tool disabled, and PilotQ's own tools are Codex dynamic tools whose results agentq passes unchanged. agentq qualification follows the adapter task [taskqorg/agentq#167](https://github.com/taskqorg/agentq/issues/167).

Versions: pilotq 0.1.0 (this branch), codex-cli 0.159.3 in PilotQ's own Codex home (subscription login), model `gpt-6.1-sol`, TaskQ CLI 4.6.36-dev on the Mac host, agentq 0.9.0 (installed, its Codex integration detached and its service not running), macOS 27.0.1.

### Context cases, agentq off (live)

`c_live_context_runs` in `tests/telegram.rs` runs the real PilotQ binary and Codex with the fake Bot API, synthetic people 7 (owner) to 11 and a fresh install per repetition. Native compaction is forced early with `model_auto_compact_token_limit` in PilotQ's Codex home (default 12000 tokens). C06 comments on a real closed issue, so the test is `#[ignore]` and runs only on request:

```sh
PILOTQ_A8_ROOT=<pilotq checkout> PILOTQ_A8_TASKQ=<taskq binary> PILOTQ_A8_CODEX_HOME=<logged-in Codex home> PILOTQ_A7_ISSUE=<closed pilotq issue> \
  [PILOTQ_C_REPEAT=3] [PILOTQ_C_COMPACT=<tokens>] cargo test --locked --test telegram c_live -- --ignored --nocapture
```

It prints one `c-live` JSON line per case and repetition: outcome, evidence, provider usage from Codex's session files (input, cached input, output tokens), native compaction count, the longest run and nudges (always 0: the script never retries a person's request).

**Result, 2026-10-23, macOS 27.0.1:** three repetitions of every case, all with agentq off and 12000-token compaction. C01, C04 and C05 pass 3 of 3; C06 and C07 fail 3 of 3 and expose two defects, filed as [#29](https://github.com/taskqorg/pilotq/issues/29) and [#28](https://github.com/taskqorg/pilotq/issues/28). No run needed a nudge. C06 added one comment per run to the closed issue [#21](https://github.com/taskqorg/pilotq/issues/21).

| Case | Result | Evidence |
|---|---|---|
| C01 | Pass 3/3 | 3, 4 and 3 native compactions. After them, each reply listed the corrected project `pilotq`, the forbidden word only spelled with dashes, the open question and the task link `issues/26` |
| C02 | Not applicable | It needs agentq compression. With agentq off nothing compresses a tool result; PilotQ's only large read, `issue_read`, returns a bounded excerpt |
| C03 | Not applicable | Same reason as C02: there is no compression step that can fail |
| C04 | Pass 3/3 | The shared topic's thread had compacted once. Person 10's `ALPHA` request was running when person 11's `BRAVO` arrived. Runs in the session did not overlap, and each reply named only its own author. No compaction fell exactly inside the ALPHA run, so the run shows serialization on a compacted thread, not a message during the compaction step itself |
| C05 | Pass 3/3 | With agentq off no recovery references exist; the route-isolation half was checked. Person 9 told a code word in a DM; asked in the shared topic, the owner got `I don't know` each time |
| C06 | Fail 3/3 | The owner's comment was written once with a receipt and never repeated (one effect, one new comment per run). After the conversation's rollout file was damaged, the next request failed with `Codex refused 2: failed to read thread: …`, and every later request in that conversation would fail until `/new`. PilotQ does not rebuild the conversation from its records, and the failure reply carries a local file path. Task [#29](https://github.com/taskqorg/pilotq/issues/29) |
| C07 | Fail 3/3 | Person 10 said `Call me Ana`; then `users remember 10 preferred_name Bea`. After compaction and a PilotQ restart, Codex answered `Ana` each time. Profile facts never reach the model, so a correction cannot win over conversation history. Task [#28](https://github.com/taskqorg/pilotq/issues/28) |

Provider usage for C01, from Codex's session files (all input, cached input, output tokens): 133953 / 111360 / 2249, 135139 / 96640 / 3194 and 123605 / 100736 / 3140. The longest single run in each repetition took 51, 62 and 46 s. `c-live` lines for the other cases carry cumulative usage per install.

### agentq on

Not run for C01–C07. With agentq attached, PilotQ's model-visible tool results would pass unchanged, so a paired run would measure only run-to-run noise. The P1 agentq gate therefore cannot pass, and agentq stays disabled for the pilot (task #26 A2). The paired comparison runs after [agentq#167](https://github.com/taskqorg/agentq/issues/167).

### Pilot report

`pilotq pilot start` records the pilot start once; `pilotq pilot note intervention|outage TEXT` records a manual intervention or a provider outage as the configured owner; `pilotq pilot report [--since T] [--json]` prints the gate 4 counts since the start:

| Field | Source |
|---|---|
| `accepted` | Requests received since the start, any front end |
| `simple` | Of those, answered with no plan, plan run or TaskQ task |
| `users`, `routes`, `projects` | Distinct senders; distinct Telegram chat and topic pairs; distinct request projects, plus `pilotq` for a request that planned or filed a TaskQ task |
| `tasks` | TaskQ tasks filed by those requests |
| `interventions`, `outages` | Notes recorded with `pilot note` |
| `failed` | `pilotq.request` CoreQ events with outcome `failed`; null with `failed_reason` when the telemetry store cannot be read |

`journal::pilot_report_counts_gate_4` covers the counts, the first-start rule and unknown telemetry.

### Pilot start

The seven-day FRA pilot started on 2026-10-23 at 09:12:26 UTC (`pilot_start` 1792746746) on the separate test bot. The earliest gate 4 readout is 2026-10-30 at 09:12:26 UTC, and only once the counts reach 500 accepted requests, 200 simple requests, two users, three routes, two projects and ten TaskQ tasks.

| Item | Setting |
|---|---|
| Build | FRA `hermes-fra1`, Linux 6.8.0-142-generic; pilotq 0.1.0 built `--release --locked` from `a2165805a1c13db30a24e77ca9ce14d32febe9e7`; binary SHA256 `655731731f2f085435d77cd7626a6a96203ed69949a40f2cd1188eee22e43bd1` |
| Runtime | FRA codex-cli 0.162.0 in PilotQ's own Codex home, its own owner device-auth ChatGPT login (PM confirmation); model `gpt-6.1-sol`, native compaction, agentq off |
| People | Owner `134142744` (every grant); second user `728914418` with `chat`, `taskq`, `project:pilotq`, `project:agentq` (owner, 2026-10-23) |
| Routes | The owner's DM, the second user's DM, the test group's shared topic `-1003995096980:3` (three registered routes) |
| Projects | `pilotq` (`taskqorg/pilotq`, TaskQ tools, `create_claim_issue` 20) and `agentq` (`taskqorg/agentq`, direct reads) |
| TaskQ | The host TaskQ CLI, root = the owner's `pilotq` checkout, 60 s board scan |
| Keys | None yet: `issue_read` and the owner's comments and labels need a key the owner adds with `pilotq keys add` |

The earlier Mac start at 06:17:54 UTC failed A5's host requirement and does not count toward this pilot. The PM verified the Mac poller stopped before FRA activation (§4). Before remediation, FRA had no second-user grants, routes, projects or isolated login. After the PM ran the prepared setup, read-only journal inspection confirmed the approved grants, all three routes, both projects and the stored start. Request 1 ended `answered` with `FRA pilot readiness confirmed.` before the start; no requests had been accepted since the start at readback. The installed binary hash matches the built artifact. FRA status showed fresh successful poll/storage observations and model success; TaskQ observation and Telegram delivery remained `unknown: not-observed`, not qualified by this CLI check (§7, §12). The PM confirmed the user service active. Service-bus access and administrative CLI writes remain sandbox-blocked in the worker.

The deployed artifact predates the rebase integration with #33. The candidate's `pilot report` retains gate counts and adds #33's `tools`, `since` and `until` measurement fields. `--until` bounds tool-size measurements only; gate counts remain since-start or since-argument counts. The rebased artifact has deterministic evidence below, not a claim of deployment.

A4 regression remediation (§10 step 5): before the fix, `pilot report --since 20 --until 30` printed only tool sizes and omitted every gate count. `cargo test --locked --test update pilot_report_prints_gate_counts_and_tool_window -- --nocapture` reproduced the missing `accepted` count, then passed after text output included all gate counts. The CLI check verifies both text and JSON, unavailable telemetry as unknown with its reason, and the retained tool-size window `[since, until)`. An intervention at `until` remains counted, confirming that `--until` bounds tool measurements only. This correction changes the candidate, not the deployed FRA artifact or pilot start.

Record each manual intervention and provider outage with `pilotq pilot note`. Rollback on FRA: stop and drain the test service; preserve the schema 7 journal and protected configuration. No previous FRA release exists. Do not downgrade its journal or reactivate the Mac poller automatically. Resume only a compatible verified build (§12).

### Case matrix

All 62 scenario families at the end of step 5. "Deterministic" names `cargo test --locked` traces on fakes; "Live" names runs with real Codex, the real board or the test bot; "Not run" names what neither covers and where it belongs. Live results on fakes for Telegram (A7, A8, C live) do not qualify the real Bot API.

| Case | Deterministic | Live | Not run |
|---|---|---|---|
| U01 | Pass | Pass (test bot) | — |
| U02 | Pass | Pass (A8, synthetic people); partial on the test bot | Second real person on the test bot: pilot |
| U03 | Pass | Pass (A7) | — |
| U04 | Pass | Pass (A7) | — |
| U05 | Pass | Pass (A8) | — |
| U06 | Pass | Pass (A8) | — |
| U07 | Pass | — | Live worker question: A8 asked none; pilot |
| U08 | Pass | — | Live stale answer: pilot |
| U09 | Pass | Pass (A7) | — |
| U10 | Pass | Pass (A8 restart while `doing`) | — |
| U11 | Pass | — | Live budget: pilot |
| U12 | Pass | Pass (test bot) | — |
| U13 | Pass | — | Live pause and park: pilot |
| U14 | Partial: unsupported media, service messages | Partial (test bot) | Edits and media interpretation: section 11 decision |
| U15 | Pass | — | Live network loss: deterministic only |
| U16 | Pass | Pass (test bot) | — |
| U17 | Pass | Partial (CLI) | Telegram profile flow: not built |
| U18 | Pass | Pass (A8); partial on the test bot | Live revoke: deterministic only |
| F01 | Pass | — | Power loss: storage-level proof, step 6 |
| F02 | Pass | — | Cross-host token conflict: step 6 |
| F03 | Pass | — | — |
| F04 | Pass | Pass (A7 receipts) | — |
| F05 | Pass | Pass (test bot `retry-after`) | — |
| F06 | Partial | — | Host alert path: step 6 |
| F07 | Pass | — | — |
| F08 | Partial: newer schema, corrupt journal, native-session rebuild (#29) | C06: Pass 3/3 at `3e5a3dc` (#29, PM run below) | Disk full: step 6 |
| F09 | Pass | Partial (A7 project redirection) | — |
| F10 | Pass | — | — |
| F11 | Partial: stale run fenced, one poller | — | Backup restore: step 6 |
| F12 | Pass | — | — |
| F13 | — | — | Cutover and rollback: step 6 |
| F14 | Pass | — | — |
| F15 | Pass | Pass (test bot, ignored person) | — |
| F16 | Pass | — | — |
| F17 | Pass | — | — |
| C01 | — | Pass 3/3, agentq off | agentq on: [agentq#167](https://github.com/taskqorg/agentq/issues/167) |
| C02 | — | — | Needs agentq compression: agentq#167 |
| C03 | — | — | Needs agentq compression: agentq#167 |
| C04 | Pass (`u12_stop_is_scoped_and_fences_the_run`: a message during a turn waits) | Pass 3/3, agentq off | agentq on: agentq#167 |
| C05 | Pass (`journal::u02_route_session_policy`, `journal::u17_profiles_distinct_and_private`) | Pass 3/3, route isolation, agentq off | Recovery references: agentq#167 |
| C06 | Pass: resume-failure rebuild, receipt recovery, no repeated effect (#29) | Pass 3/3 at `3e5a3dc` (#29, PM run below) | Real Telegram remains unqualified |
| C07 | Pass (`c07_current_profiles_reach_each_turn`): corrections, forget, sender and route privacy | Pass 3/3 on FRA after #28; original Mac failure retained above | Agentq comparison: agentq#167 |
| L01 | Pass | Pass (A8) | Fast-answer suppression: not implemented |
| L02 | Partial | — | Typing bounded by last useful event |
| L03 | Partial | — | Incident notices across restart |
| L04 | Pass | — | — |
| L05 | Partial: deadline, tool budget | C01 compaction below | Ineffective-compaction loop |
| L06 | Pass | Pass (test bot `/stop`, `/new`) | — |
| L07 | Pass | — | Real Codex tool inventory: A7 and A8 used only PilotQ tools |
| L08 | Partial | — | 401, 429 retry-after and 5xx from the provider |
| L09 | — | — | Credential expiry and account change |
| L10 | Partial | — | Bot identity and permission manifest: step 6 |
| L11 | Pass | — | — |
| L12 | Pass | Pass (A8) | — |
| L13 | Partial | Partial (test bot group migration) | Administrative remap |
| L14 | Partial | — | Edits and albums: section 11 media decision |
| L15 | Partial | Pass (A8 restart) | Linux service: step 6 |
| L16 | Pass | Pass (A8 scan after restart) | — |
| L17 | Partial | — | Wall-clock jump |
| L18 | Partial | — | External monitor: step 6 |
| L19 | Partial | — | Storage full and contention: step 6 |
| L20 | Partial | — | Rollback with accepted work and restore: step 6 |

## Step 5 profile correction follow-up (#28)

Task [#28](https://github.com/taskqorg/pilotq/issues/28) applies R10, R13 and [section 6](../pilotq.md#6-memory-and-agentq): every turn loads the sender's current route-visible profile, with revisions, before the person's request. An empty snapshot after forgetting supersedes earlier conversation and summary values. Profile values remain data, never grants or instructions. The contract is unchanged.

**Before:** #26's C07 live runs failed 3/3: conversation history said Ana after `users remember` set Bea. Inspection on base `6bad757` found no profile read in the manager. The new `c07_current_profiles_reach_each_turn` fake-Codex trace failed because the first turn input was request text rather than a profile snapshot.

**After, development check (Linux, 2026-10-23):** the same trace passed after the fix, covering revision 1 Ana → revision 2 Bea → forgotten, a resumed CLI thread, subject isolation and private-fact exclusion from a group topic. It was then extended to inspect Telegram DM visibility and another sender's turn. Its isolated runs were sandbox-blocked (exit 137); the extended trace subsequently passed in the frozen candidate suite. That suite passed all 15 journal tests but was later sandbox-blocked (exit 137), so the full suite is not a pass. `cargo run --locked -- selftest` passed. The trace inspects actual `turn/start` input, not fake model assertions about memory.

**Live C07: pass 3/3 on FRA, 2026-10-23.** The C07 slice of #26's `c_live_context_runs` is available as `c_live_context_runs_c07`. It uses three fresh installs, the real binary, synthetic Telegram input through a local Bot API, subscription Codex, verified native compaction, a restart after correction and another after forgetting. Run with:

```sh
PILOTQ_A8_CODEX_HOME=<subscription-authenticated-Codex-home> \
  cargo test --locked --test telegram c_live_context_runs_c07 -- --ignored --nocapture
```

The earlier worker run used codex-cli 0.162.0 and exited 137 in the sandbox during the first request; it established no live pass. The PM subsequently ran the command above outside the worker sandbox on `9dab603b70017bfa9a830fab097ea13978ae1b44`, using a separate owner-approved subscription login for tests. [PM evidence and owner answer](https://github.com/taskqorg/pilotq/issues/28#issuecomment-6362362439) authorize reuse without rerunning live calls when the tested code is unchanged. The check passed in 263.09 seconds (1 test passed, 30 filtered out).

| Repetition | Correction / forget | Input / cached input / output tokens | Compactions | Missing usage threads |
|---|---|---|---|---|
| 1 | Bea (revision 2) / UNKNOWN | 44761 / 17664 / 531 | 3 | 0 |
| 2 | Bea (revision 2) / UNKNOWN | 44556 / 17664 / 536 | 3 | 0 |
| 3 | Bea (revision 2) / UNKNOWN | 44853 / 26496 / 545 | 3 | 0 |

After rebasing on `dae24fa`, `cargo test --locked` passed all 17 journal tests, then exited 137 as Telegram integration tests began (sandbox-blocked, not a full-suite pass). `cargo run --locked -- selftest`, specification-file checks and whitespace checks passed. No service process remained after the test attempt.

All three repetitions report `pass: true`, agentq off. The rebase retains the profile implementation and both C07 tests unchanged; conflict resolution only joins the independent upstream tests and acceptance evidence. This establishes A3 for the tested implementation; it does not qualify real Telegram, production deployment or agentq. Merge-result CI remains the publication gate (TaskQ §7).

### Native-session rebuild remediation (#29)

R5 and §3–4 require reconstruction from journal records when Codex cannot read its native session. §12 keeps the original deadline and run fence. The adapter rebuilds once before submitting the current turn, using the same subscription home and workspace. It seeds the replacement with the latest 12 model-handled requests and finalized replies of the same conversation generation, plus its retained direct-effect and TaskQ receipts. Historical content is data, never authorization. Current profile snapshots still supersede old preferences. Oversized recovery evidence is refused rather than silently dropping receipts.

The journal records recovery and queues one notice per damaged native thread in the original route. The damaged ID remains until a replacement is finalized, so a crash during reconstruction does not discard the recovery source. Resume/open failure replies omit native diagnostic paths. Reconstruction invokes no historical tool or TaskQ command.

**Before:** `cargo test --locked --test telegram c06_resume_failure -- --nocapture` failed with `timed out: rebuilt answer`: the comment had completed, but the follow-up never answered after the injected resume failure. **After:** the same trace passed. It verifies one completed comment, no additional GitHub calls or effects, recovered historical request/reply and effect receipt, one audit recovery event, one notice, no native error path in replies, and a follow-up on the replacement thread. This retains mutation-once protection while adding conversation recovery; no contract rule changes.

**Earlier worker live run, Linux, 2026-10-23, Codex 0.162.0, agentq off:** `c_live_context_runs_c06`, the C06 slice of `c_live_context_runs`, completed repetition 1 successfully against closed issue [#21](https://github.com/taskqorg/pilotq/issues/21). It damaged the actual native rollout, recovered the [verified comment receipt](https://github.com/taskqorg/pilotq/issues/21#issuecomment-6362676412), answered another follow-up, and checked exactly one comment, one effect, one recovery event and one notice. The runner then exited 137 (sandbox-blocked); repetitions 2–3 have no passing result. This is **not** three-run live acceptance or real Telegram qualification. Original #26 failures above remain historical evidence.

**PM live acceptance, Linux, 2026-10-23:** the PM reran `cargo test --locked --test telegram c_live_context_runs_c06 -- --ignored --nocapture` outside the worker sandbox at exact SHA `3e5a3dcf1c5d91bc0cb1ece3290150e6ddd21e54`, using a test-only Codex subscription login and `PILOTQ_A7_ISSUE=21`. The result was `test result: ok. 1 passed; 0 failed`, exit 0 in 55.99 seconds. All three repetitions passed; each recorded exactly one new comment, one effect, one rebuild and one notice. Each rebuilt answer named issue #21 and the verified receipt; no effect replayed (§4).

| Repetition | Result | Verified comment receipt |
|---|---|---|
| 1 | Pass | [6364473140](https://github.com/taskqorg/pilotq/issues/21#issuecomment-6364473140) |
| 2 | Pass | [6364478384](https://github.com/taskqorg/pilotq/issues/21#issuecomment-6364478384) |
| 3 | Pass | [6364484070](https://github.com/taskqorg/pilotq/issues/21#issuecomment-6364484070) |

The [PM evidence and owner answer](https://github.com/taskqorg/pilotq/issues/29) supply the three-run A3 acceptance missing from the earlier worker run. The tested implementation and single-write checks are unchanged by this documentation correction. This qualifies the real PilotQ/Codex/board flow with a local fake Bot API, not real Telegram. The earlier sandbox-blocked runs and original failures remain historical evidence; merge-result (`merge_group`) CI remains a separate publication gate.

Run the explicit live slice with an authenticated subscription home and board token, selecting a closed issue that may receive three test comments:

```sh
PILOTQ_A8_ROOT=<pilotq-checkout> PILOTQ_A8_TASKQ=<taskq-binary> \
  PILOTQ_A8_CODEX_HOME=<subscription-home> PILOTQ_A7_ISSUE=<closed-issue-number> \
  cargo test --locked --test telegram c_live_context_runs_c06 -- --ignored --nocapture
```

Completion evidence is a report with exact versions, mechanism-to-case results, unresolved failures, recovery receipts, task links and aggregate metrics. Maintain 62 scenario-family entries: U01–U18, F01–F17, C01–C07 and L01–L20. Record deterministic and live/platform results separately, including not run. Runtime research, a healthy service or green repository CI cannot replace it.

### Registered-board controls (task #39, §5, R4, R13, R17)

| Case | Action | Expected result |
|---|---|---|
| U39a | Steer a doing task, then requeue with a reason | Argument arrays carry the fresh revision and stable op ID; intent and receipt are audited; reply gives readback and board URL |
| U39b | Plan and run on a second registered board; restart and relay a question | Plan keeps its board and root; status, question and outcome stay in the originating route; the same task number on another board stays independent |
| U39c | Use a bare number with several accessible projects, a foreign URL, mismatched project/URL, missing grant or stopped run | Refusal explains the target or grant problem; no board write |
| U39d | Reply to an old question on the second board after a conversation switch | Fresh question revision rejects the answer; no answer reaches either board |
| U39e | Lose a mutation response, restart, or change a registration before retry | Reconcile only the saved operation ID, revision and target; changed registration blocks retry; no duplicate effect |

**Development evidence (Linux, task #39):** `cargo test --locked --test telegram u39_registered_boards_steer_requeue_and_stale_answer -- --nocapture` failed against the original adapter at `timed out: ambiguous task`, then passed against the change. The retained fake-Codex trace exercises both controls, grant and conflicting-target refusals, two boards' #19 subscriptions with distinct status messages, a stale second-board answer after restart and a lost control response reconciled with exactly the same arguments, op ID and revision. The target revision remains unchanged during reconciliation. This proves the adapter/journal flow on fakes, not real Codex, Telegram or a live board.

The journal's `plans_and_task_projections`, `schema_1_journal_upgrades`, `u39_schema_7_preserves_task_targets` and `taskq::tests::u39_fenced_controls_write_nothing` passed in filtered checks. A combined integration development run (`cargo test --locked --test telegram -- u06 u07 u10 u39 f07 --nocapture`) passed F07 and U10 before exit 137: sandbox-blocked, not a suite pass. Merge acceptance still requires green CI on the merge result (§7, R12).

**Review remediation (task #39):** U06 reproduced the CI failure at `tests/telegram.rs`: actual audit targets were `taskqorg/pilotq add #19` and `taskqorg/pilotq start #19`, while its expectations omitted the board. Under §5 and R17, U06 and the matching live A8 assertion now retain exact board, operation and task checks. Accepted behavior and required protection are unchanged. The filtered U06 rerun exited 137 (sandbox-blocked), so an after-pass remains unproved locally; merge-result CI remains required.


## Automatic management suite (A40)

Task #40 adds one release qualification command for R3, R5, R8, R10 and R17, over the §5 management path. The real candidate binary and subscription Codex talk to a local fake Bot API and the real `taskqorg/pilotq` board through the TaskQ CLI. Synthetic owner 7 and person 8 share a forum topic; person 9 has no grants. This does not qualify the real Telegram service or production deployment. The PM corrected A4 on 2026-10-24: test stale question replies as U08 does; inline buttons are outside this task.

Prerequisites: a Linux host with Rust, Python 3, `sha256sum`, `gh`, a working board token, TaskQ, an already authenticated test-only Codex subscription home, and a TaskQ root with an existing queue focus. No worker service is required for the fixture. No login or paid API fallback occurs. The selected root must name `taskqorg/pilotq` and configure `create_claim_issue`. Run from the candidate checkout:

```sh
PILOTQ_A8_ROOT=<absolute PilotQ TaskQ root> \
PILOTQ_A8_TASKQ=<absolute TaskQ binary> \
PILOTQ_A8_CODEX_HOME=<authenticated test-only Codex home> \
PILOTQ_A40_REPORT=<absolute new report.json path> \
  cargo test --locked --test telegram automatic::a40_live_management -- --ignored --exact --nocapture
```

The command creates one real board task for a scripted navigation-check fixture, with no implementation or publication. Before forwarding any `add` to TaskQ, the test-only wrapper appends `--host pilotq-e2e-<unique-run-id>`; that host is bound into the create claim and retained on retries and cleanup. Reserve `pilotq-e2e-*` for these tests: never configure a service with such a host name. The runner verifies the issue has exactly that host label and remains unclaimed after start. It then writes trusted, hand-authored TaskQ log entries (§2 of the TaskQ contract) for synthetic worker spawn, question, result, reviewer spawn and close, reading back each transition. The resulting notifications qualify PilotQ's management path, not worker execution, code review or publication. Real Codex still chooses PilotQ tools; TaskQ still performs held creation, start, status, revision-fenced controls and answers.

The run exercises queue pause on the selected host/project and restores it with `service resume` only after the suite's own pause. Restoration checks the original focus; TaskQ retains `focus.paused` after resume, so that file alone is not an active pause. Existing paused admission is a failed prerequisite, never silently enabled. Use a host where the PM has authorized this bounded admission pause and has reserved admission controls for the run. The fixture launches no worker or reviewer process. TaskQ retains control of unrelated worker processes; pause, park and requeue never prove their termination.

| Case | Required observation |
|---|---|
| A40.01 admission | Person 9 is ignored with no request, outgoing message or typing; person 8 has chat only |
| A40.02 discussion | Codex answers a discussion without saving a plan or filing a task |
| A40.03 plan | A separate request saves one revision and files nothing |
| A40.04 create/start | Next request files exactly one ticket held, reads it back and starts it; board log and journal agree; exactly one reserved host label; no service spawn |
| A40.05 status | Explicit status request records a fresh task read and returns its URL |
| A40.06 steer | A doing task accepts a revision-fenced steer; journal receipt and board log agree |
| A40.07 pause | Queue pause reports running workers continuing; admission is restored after controls |
| A40.08 park/requeue | Park and requeue have accepted persisted receipts and board readback; each intent retains an op ID and revision |
| A40.09 restart | Kill and restart PilotQ while TaskQ is doing; subscription and task identity survive, with no duplicate creation |
| A40.10 question/foreign | A required worker question reaches the original topic; person 8's reply is refused and writes no answer |
| A40.11 answer/stale | Owner replies to the current question; board records the answer; replying again to the old message is refused without a second answer |
| A40.12 reviewer | Observe scripted reviewer state and the separate closing outcome; fixture board issue is closed completed with explicitly synthetic result evidence |
| A40.13 audit/delivery | Audit joins author, route, request trace, tools and task; CoreQ trace exists; recorded sends remain in the original route |
| A40.14 cleanup | Stop the test PilotQ, remove copied credentials, leave every created task closed or parked and restore admission if paused by this run |

The JSON report starts with every case `not_run`, changes the active case to `running`, and writes each verified result immediately. It includes candidate SHA, binary hash, versions, board/task links, recorded Bot API messages and per-case evidence. A panic marks the active case failed and still attempts cleanup. A killed test may leave `running` cases: that is incomplete, never pass. Keep the report and install evidence private (request/audit text is synthetic; native sessions and protected files still need care). No credentials enter the report. Only an exit-zero run whose report has every case `pass`, including cleanup, qualifies. No question, refusal, scripted reviewer result or cleanup may be silently skipped. The report retains the reserved host, host-label readback and every scripted event comment ID/body. Missing observations remain unknown. The scenario deadline is three hours (cleanup has separate 120-second CLI call limits); this live suite is separate from the deterministic merge gate `cargo test --locked` and runs before each release under §9.

On failure the runner stops PilotQ and parks its task with TaskQ, retaining the install for investigation and recording cleanup failures. It reconciles any ticket whose create response was interrupted by its original operation ID before parking it; it never uses plain `add`. If the runner is killed, the PM must inspect the kept journal and report, reconcile the stored ticket op ID, park any still-open task, and restore only this run's saved pause before rerunning. A sandbox-blocked run (including exit 137) supplies the command above for the PM; it supplies no live pass. Green merge-result CI is a separate publication requirement (TaskQ §7, R12).


**Worker evidence, 2026-10-24:** before A40, A8 could complete without any worker question and did not cover steer/requeue/park/pause or write a per-case release report. The new `automatic::a40_failure_report_and_cleanup` deterministic check passed: an injected missing answer stays failed, later cases stay not run, an interrupted add reuses its exact arguments and op ID, its task is parked, the copied login is removed, and the original queue focus is restored. Filtered U07, U10 and U39 checks also passed (4 checks total, 14.68 seconds), preserving reply authorization, revision fencing and existing lifecycle protection. These are fake-adapter checks, not live qualification.

The first diagnostic live A40 run used subscription Codex 0.162.0 and the real board. Admission, discussion, plan and held-create/readback/start passed on [#45](https://github.com/taskqorg/pilotq/issues/45); the worker sandbox then exited 137 during status. The remaining cases and automatic cleanup were not proved. The worker separately verified no test PilotQ remained, removed the copied login and parked #45 with an accepted `later` receipt. Queue pause had not run. This is **sandbox-blocked**, not a complete live pass or release evidence. The command above is the PM handoff for a complete run outside the worker sandbox on the final candidate. Green merge-result CI remains required separately (§7, R12).

**Review remediation, 2026-10-24 (R5, §5; TaskQ §7 step 4):** the earlier runner had no host label and let an ordinary Mac service claim #45. The existing authorization, revision fencing, failure-report and cleanup protections remain required. `python3 tests/support/automatic_host.py` reproduced the defect before the fix (`--host` absent from dispatched `add`) and passed after it: initial creation, a retry and cleanup reconciliation use identical host-bound arguments/op ID; a conflicting host or missing isolation configuration dispatches nothing. `automatic::a40_host_isolation` includes that regression in `cargo test --locked`. The lifecycle now advances through explicitly synthetic board events instead of waiting for ordinary workers. The earlier partial live results do not qualify this revised runner; use the PM command above on the candidate for live qualification.

**Task #48 remediation (R5, R17, §5, §9):** the PM run on `b0b508d` failed the pause case with `QueryReturnedNoRows`. The audit stores `queue_pause` as a `call` target and the completed native item as a `tool` with target `dynamicToolCall`; it does not store tool-result text there. The pause check now reads those existing records and verifies the fixture remains doing. `u10_restart_new_status_pause_and_park` reproduced the same missing-row failure before the query fix and passed afterward.

**Task #62 remediation (R5, R8, §5, §9):** the FRA run at `ded8266` timed out waiting for the scripted doing observation. The adapter subscribed immediately after held creation and treated absence from a complete snapshot as closure, including while the manager was still starting the task. TaskQ 4.6.39's `start_one` removes `q-held` before adding `taskq`; its open snapshot excludes an issue with neither label. A scan in that interval permanently closed the subscription. `Journal::subscriptions` now defers scanning tickets whose plan run is still creating; the existing submission recovery retains their operation IDs. `cargo test --locked a40_start_label_gap_keeps_subscription` failed before the fix with `an in-flight start must not emit a closing outcome` and passed afterward, also checking subsequent doing, stale-revision rejection and eventual closure. The 60-second live wait is unchanged. This reproduces the race at the adapter/journal boundary; the private FRA journal was not inspected and the earlier passing run's TaskQ version is unknown. The PM must rerun installed A40 on FRA after merge; this deterministic check is not live release evidence.

Cleanup now uses an exact-argument, matching-operation accepted create receipt to identify a known fixture, then reads its current board state and parks it. It replays the original claim only when no accepted receipt exists; an unavailable reconciliation without a receipt remains a cleanup failure. `automatic::a40_failure_report_and_cleanup` failed before the fix with an unavailable create replay and retained `focus.paused`; it passed afterward. The check retains lost-response reconciliation with the same arguments/op ID, failed/not-run case reporting, copied-login removal and refusal to overwrite concurrent focus changes. This changes neither the product contract nor those required protections (TaskQ §7 step 4).

The A40 report includes per-tool `calls`, `p95_result_bytes` and `max_result_bytes` from `pilot report --since 0 --json`. Its source is the install journal's `tool_sizes`, measured as UTF-8 result bytes after secret scrubbing and before Codex delivery; `audit_events` holds native tool status rather than result text. These sizes are input for agentq#167, not provider token savings. Live qualification remains pending: the PM must run the A40 command above on the merged candidate and retain an exit-zero report with every case passing, cleanup included. Deterministic checks do not establish that §9 release gate.

**A40 audit project remediation, task #51 (R17, §5, §8):** `cargo test --locked a40_topic_plan_run_audit_project -- --nocapture` failed before the fix because the authorized run was absent from the `pilotq` project audit filter (0 rows, expected 1), then passed after authorization stored the saved plan's project in the same transaction. The retained journal regression checks a forum-topic request after a conversation project switch, refusal without an audit project change, single-run retry, restart persistence and the exported `where.project`, chat and topic. The live `audit_delivery` assertion remains unchanged. This deterministic evidence does not qualify A40 live; the PM reruns A40 on FRA at the result candidate and reports to #38.

**A40 topic delivery remediation, task #54 (R1, §5, §12):** `cargo test --locked --test telegram a40_topic_state_notifications -- --nocapture` failed before the fix because recorded status edits had no `message_thread_id`, then passed after the fake Bot API retained the edited message's original topic. Telegram's `editMessageText` addresses an existing chat/message and has no topic parameter; production sends and durable outbox rows already preserved the topic. The fake now rejects edits of unknown messages and records their effective destination, rather than treating an edit payload as a new message. The retained deterministic trace covers ready, doing, parked, ask, review and closed across restart, verifies every task outbox destination and every accepted message's chat/topic, and asserts that closure is a new message that is never edited. The live A40 assertions remain unchanged. This proves the fake-backed delivery path, not real Telegram or the live management suite; the PM reruns A40 on FRA at the candidate SHA and reports to #38.


## FRA release update and setup check (#41)

Owner continuation 2026-10-24 authorizes implementing `doctor` and deployment after verification. Before this change, the CLI dispatch had no `doctor` command and the tester guide explicitly reported it unimplemented; A1 could not pass. The §9 read-only check now reports configuration, Telegram bot identity, isolated Codex ChatGPT login and complete TaskQ board reachability, with nonzero failure and no raw dependency diagnostics (§8, R9). `doctor_missing_configuration_is_not_healthy` protects against reporting absent setup as healthy and against writing state during inspection.

Worker verification: Cargo is absent (exit 127), so the filtered check and candidate checks are sandbox-blocked, not PASS. No executable before/after proof is available here. FRA activation, running SHA, owner-entered `issue_read` key, A40 and journal continuity across activation remain unverified (§4, §8, §12). No host update or credential change has been performed by this worker.

Owner readback recorded in #41 on 2026-10-24: the protected CLI key is entered and granted for `project:pilotq`; FRA owner request 2 successfully used `issue_read` on #41, and key metadata recorded last use (§5/§8). This is owner-supplied evidence, not a worker inspection of the secret or host. It supersedes the key-entry gap above.

The approved post-0.2.0 release candidate is `0.2.1`, including doctor from `602841ea2b66927195c25bdb981c076961adcee7` (§9). Before: published 0.2.0 lacks doctor; after: version metadata and tester commands select 0.2.1. Executable candidate A40, release publication, FRA activation, running SHA and installed suite/continuity remain unknown until host execution. No regression checks or acceptance protections were removed.


### FRA activation readback, 2026-10-25

The [PM host report](https://github.com/taskqorg/pilotq/issues/41#issuecomment-6391544828) supersedes the deployment gaps above. These are attributed host observations, not commands rerun by this worker. No test, acceptance rule or retained regression protection changed.

- **A1 (§9, §12):** before activation, FRA ran 0.1.0. `pilotq update` selected 0.2.1 and reported `drain: ok`. Restart initially failed with `systemctl --user daemon-reload` under a session without a user bus; the PM then restarted the unit with the user runtime directory set. Afterward PID 582086 executed `releases/0.2.1/pilotq`, whose checksum matched the published artifact. The reported running source is `ded8266cde9dc91611d3db35091635925718c17c`. `status --json` reported version 0.2.1 and healthy poll, storage and model observations; `selftest` passed; `doctor` returned exit 0 with configuration, Telegram, Codex and TaskQ checks passing. This proves recovered activation, not a successful automatic restart in the failed session.
- **Publication (§9):** PR #58 merged at that SHA; `live` fast-forwarded from `4831a0c` to it. Both [tests](https://github.com/taskqorg/pilotq/actions/runs/40146817010) and [release verification](https://github.com/taskqorg/pilotq/actions/runs/40146817014) passed on the exact `merge_group` SHA. The [release workflow](https://github.com/taskqorg/pilotq/actions/runs/40147385891) succeeded; the PM reports v0.2.1 published at 00:40:50 UTC.
- **A2 (§5, §8, R16):** the earlier owner CLI grant and successful request 2 `issue_read` remain the key-use evidence. No credential value was inspected or copied into this record.
- **A3 suite (§9):** the [PM A40 report on PR #58](https://github.com/taskqorg/pilotq/pull/58#issuecomment-6391412988) records 14/14 passing cases on FRA at `f5aa0eb7bf26577fb8a8cee43c3d3d222ab74e08`, completed at 00:27 UTC, including cleanup. Its tree equals the release tree (`0e6c69a`); this is candidate management-path evidence before activation. A separate run against the installed release is not recorded.
- **A3 continuity (§4, R5, R17):** the PM reports post-update `pilot report` counts of failed 0, interventions 0 and outages 0; `status` reports pending 0, undelivered 0 and uncertain 0. These establish no reported backlog or unresolved effects at that observation. No before/after request-ID, effect or delivery reconciliation was supplied, so absence of lost or duplicated requests across activation remains unproved. The service restart intervention above is recorded here even though the pilot counter reports zero.

Remaining acceptance evidence: an installed-release suite run and request-level journal reconciliation across activation. Production cutover remains unauthorized (§10–§11).

### Follow-up verification, attempt 6391625559

PR #60's [tests job](https://github.com/taskqorg/pilotq/actions/runs/40148004772/job/121033387904) failed at `1e009113f98f1be40b4d1ed7e9c2bc6a0c3743a8`: `u09_f09_f10_switch_ambiguity_redirect_and_revoke` reported `pilotq: audit: database is locked` (32 passed, 1 failed, 7 ignored). This is a failed gate, not passing publication evidence. The retained check protects project selection, rejection of quoted target redirects, revocation and project-filtered audit readback (§5, §8, R17).

Read-only inspection found that `Journal::audit` opens a deferred transaction, reads the request records and then records the audit access. A concurrent writer between those operations is a possible lock-upgrade conflict; the CI log does not establish that cause. No implementation or test change was made within this task's operation-and-evidence scope, and no after-fix pass is claimed. The filtered reproduction command `cargo test --locked --test telegram u09_f09_f10_switch_ambiguity_redirect_and_revoke -- --exact --nocapture` is sandbox-blocked because Cargo is absent (exit 127). `pilotq status --json` is also sandbox-blocked because the installed CLI is unavailable here (exit 127). Installed A40 and request/effect/delivery reconciliation remain unproved; A1/A2 retain the attributed host evidence above. No acceptance protection was weakened.


### 0.2.2 preparation, task #41 continuation

The owner requested `0.2.2` in PR #60 because published `0.2.1` lacks the #62 fix at `49043392fb12b80d2e6d4c8db3fee28942d916ce` (§9). Package metadata and tester commands now select `0.2.2`; no test or acceptance protection changed. The PM reports A40 passed 14/14 on FRA at `4904339` on 2026-10-25 at 01:14 UTC. This is attributed development-candidate evidence, not an installed `0.2.2` result.

After merge and the PM's `live` push, the PM updates FRA through `pilotq update` and supplies the installed source commit, A40 case results and cleanup, pilot report and journal request IDs before and after activation. Reconcile those IDs with effects and delivery receipts before claiming A3 (§4, §9, R17). The earlier installed-suite failure and continuity gap remain open until that evidence arrives.
