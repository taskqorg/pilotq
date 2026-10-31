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
| U11 R2 R3 | Authorized direct work has active runtime events | Exceed the former total-duration and call caps, then stop or become inactive | Active work continues without a fixed cap; inactivity or stop ends it with retained findings and explicit unfinished or uncertain effects (§5, §7) |
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

These twenty scenario families close gaps found in the source review. They supplement U/F/C cases instead of treating typing, retries or an upstream module name as proof of reliability. All are **proposed, not executed**. Root [section 12](runtime-contract.md#12-proposed-lifecycle-and-feedback-protocol) owns their behavior.

Each trace records initial request/run/effect/delivery state, authorized actor and route, input event, injected fault, expected persisted state, user-visible output and permitted external effects. Use fake clocks, Telegram, Codex and TaskQ for the cheap deterministic suite. Qualify the corresponding real transport, subscription, process and storage behavior separately at release time.

| ID and requirement | Initial state and action | Fault or variation | Observable acceptance evidence |
|---|---|---|---|
| L01 R5 R11 | Send a request while a run slot is busy; then free capacity | Fast answer, queued request, failed inbox commit, waiting for a person or TaskQ | Acknowledgement follows durable commit; queued is not running; reaction receipt avoids a redundant text acknowledgement; reaction failure falls back to text; a queue wait over 30 seconds gets one observed queue-depth notice; waiting has a reason and does not trigger a hung-run warning |
| L02 R1 R5 R11 | Start a Codex turn in a forum topic | No first event; active events; stale activity; wait; cancel; finish; typing API failure | Typing stays in the correct topic, continues through a long active call within its inactivity bound, follows section 12's rate bound and stops on wait/cancel/terminal state; its timer/failure never extends a deadline, creates work or blocks a final reply |
| L03 R5 R11 | Emit factual tool/task progress, a stall and subsequent recovery | Event burst, repeated error, service restart, recovery without a successful probe, sensitive text in errors | Coalesced status reflects evidence; incident notices deduplicate across restart; no raw secrets/reasoning; a reset timer alone cannot claim recovery; final/question/failure is not starved by edits |
| L04 R5 R11 | Spawn a Codex process that never emits an event | Startup hangs, or stops before activity state is initialized | First-event watch still fires within the configured bound; request becomes recoverable/failed with notice; slot and process resources are released or fenced |
| L05 R2 R5 R7 R11 | Run a chatty model, a quiet tool and compaction | Active events beyond the former total budget; ten-minute inactivity; stop | Runtime events renew inactivity; timers do not. No fixed turn-duration or call cap; stop and inactivity preserve completed findings and report unconfirmed effects without replay (§5, §7) |
| L06 R1 R5 R10 R11 | Fill the normal queue, then issue authorized stop/status/reset | Cancel queued request; interrupt running tool; old runtime emits after `/new`; unauthorized control command | Control remains responsive within budgets; generation blocks stale effects/replies; old receipts remain queryable; revoked/unknown actor gets no effect or reply; reset does not recreate or lose active TaskQ work |
| L07 R2 R5 R11 | Deliver native events and a candidate tool call | Partial but parseable arguments; invalid target; unknown control event; EOF without terminal; empty answer; optional unknown diagnostic event | No partial/invalid effect; no guessed success; optional diagnostic can be ignored safely; explicit failed/uncertain outcome retains evidence; typed tools retain the verified effect boundary; native tools use §5 service-user authority and no automatic replay |
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

The following revision-specific evidence records the earlier slices. Historical absolute-duration,
five-call and read-only-manager checks are superseded by §5’s full-agent decision; they are not
current acceptance requirements. Current inactivity, unlimited-call and recovery traces are
`native_and_long_turn_190`, `unrestricted_calls_keep_admission_190` and `runs_reconcile_and_fence`.

Results for the [section 10](adoption.md#10-adoption-sequence) step 2 slice, task [#14](https://github.com/taskqorg/pilotq/issues/14). Deterministic traces run in `cargo test --locked` with a fake Bot API (`tests/telegram.rs`), a fake Codex app-server script, journal calls that take explicit times (fake clock) and fixtures. Test names below are in `src/journal.rs` (`journal::`), `tests/telegram.rs` or `tests/update.rs`. Fakes do not qualify real Telegram, Codex, launchd or storage behavior; the live results are recorded separately below.

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
| L01 | Partial | Acknowledgement only after commit (`f01_crash_points_keep_each_input_once`), queued is not running (`journal::backlog_is_bounded_and_fair`), reaction receipts without numbered text and one retention notice (`journal::telegram_receipt_is_reaction_not_numbered_text`, `u01_follow_up_resumes_the_conversation`); slow-queue visibility (`journal::queue_notice_and_draft_fence`). Waiting for a person or TaskQ: step 3 |
| L02 | No | Step 3, with progress status messages (no typing indicator in the first slice) |
| L03 | No | Step 3 |
| L04 | Yes | `f03_provider_faults_are_bounded` (no first event) |
| L05 | Partial | Absolute deadline despite endless tokens (`f03_provider_faults_are_bounded`). Tool-call budget: step 4; compaction: step 5 |
| L06 | Partial | `u12_stop_is_scoped_and_fences_the_run`, `u16_controls_work_without_the_model`, `journal::runs_reconcile_and_fence` (`/new` generation), `journal::backlog_is_bounded_and_fair` (control on a full route), `journal::strangers_and_claims_change_nothing` (stranger `/stop`). TaskQ work under reset: step 3 |
| L07 | Yes | `f03_provider_faults_are_bounded`: EOF without terminal, empty answer, unknown optional event, refused server request. The real Codex tool inventory is a live check |
| L08 | Partial | Quota failure stays failed and no API key reaches Codex (`f03_provider_faults_are_bounded`). Distinct auth, quota and 429 handling with retry-after: step 5 |
| L09 | No | Step 5; the login state is part of the live run |
| L10 | Partial | Unsafe or malformed `config.json` stops startup. Bot identity and permission checks against a manifest: step 6 |
| L11 | Yes | `journal::long_reply_is_chunked_in_order`, `f05_delivery_faults_are_bounded`, `telegram_reply_delivery_fallbacks` (rich → HTML → plain, draft errors and escaping) |
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

Results for the [section 10](adoption.md#10-adoption-sequence) step 3 slice, task [#18](https://github.com/taskqorg/pilotq/issues/18), against the [first TaskQ tool contract](integration-contract.md#first-taskq-tool-contract). Deterministic traces run in `cargo test --locked` with a fake TaskQ CLI (`FAKE_TASKQ` in `tests/telegram.rs`: argument log, op-id receipts, revision-checked answers, board-down and lost-response faults), a fake Codex that makes one complete dynamic tool call per turn, the fake Bot API (messages, edits, typing) and a 1 s board scan. Fakes do not qualify the real board, Codex dynamic tools or Telegram; the live run is listed separately.

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

The owner replaced the human test-bot run on 2026-10-23 (root [section 5](integration-contract.md#first-taskq-tool-contract)). `a8_live_board_run` in `tests/telegram.rs` runs the real PilotQ binary with real Codex (PilotQ's own Codex home holding only the subscription login), the real `taskqorg/pilotq` board and TaskQ CLI. Telegram is the local fake Bot API with two synthetic users, 7 (configured owner) and 8. It files and publishes a real task, so it is `#[ignore]` and runs only on request:

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

## Human trial (task #42)

[Contract §5](integration-contract.md#human-trial) records the owner's 2026-10-26 simplification: the two existing FRA users, the owner and Simon, test on the fly with test tasks. No invitation, private-ID exchange or additional readiness gate is required. This supersedes the earlier three-person plan. The owner ended the human trial on 2026-10-29 using available evidence, shortening the planned seven-day observation period; the request-linked report remains and gate 4 stays separate.

### Testing and feedback

Use existing authorized identities and grants with the [tester guide](tester-guide.md#plan-authorize-and-follow-work). Discuss and revise a plan, authorize its saved revision in a later message, follow status and answer actual worker questions by replying to them. Keep existing R10 authorization checks; do not infer administrator privileges from participation. The owner named the owner and Simon as administrators on 2026-10-27 after the readiness ask. No grant change is claimed here.

Keep request IDs and task links for outcomes, refusals, failures and feedback about usefulness, clarity and required nudges. Each accepted request retains its R17 audit record. Record interventions and outages with `pilot note`; investigate missing replies through effect and delivery receipts rather than repeating execution (§4). File each discovered defect held, linking an existing open issue where it already covers the defect. No completed human trial or absence of defects is claimed before observation.

### Report

Record the actual UTC observation window, release revisions used, participating users by approved labels, request IDs with outcomes/refusals/failures and tester feedback, and held defect links. Keep waiting, uncertain and missing observations visible. Preserve the existing gate 4 pilot start; `pilot report --since T --json` supplies since-boundary counts, while `--until` bounds tool sizes only. Use audit receive times and request IDs to identify the final trial cohort. Keep private message content and credentials out of public reports (§8).

**Readiness evidence:** owner 2026-10-26 reports PM A40 14/14 at `49043392fb12b80d2e6d4c8db3fee28942d916ce`, referencing [#62](https://github.com/taskqorg/pilotq/issues/62). The worker read #62's merged fix and green merge-result CI record; the 14/14 result is supplied by the owner, not independently rerun here. It does not prove an installed 0.2.2 run. No further A40 or continuity submission is added as a gate for this human trial.

A1 records the current plan in the contract and these cases. The owner's continuation supplies the tester choice for A2 and replaces the invitation procedure. A3 uses existing grants and the guide; the PM export records one active user, while Simon sent no requests. A4 evidence and existing defect references are recorded in the [trial report](trial-42.md). The report preserves missing observations and does not qualify two-user behavior, gate 4 or production acceptance.

## Step 4 qualification

Results for the [section 10](adoption.md#10-adoption-sequence) step 4 slice, task [#23](https://github.com/taskqorg/pilotq/issues/23), against [direct actions, keys and project switching](integration-contract.md#direct-actions-keys-and-project-switching). Deterministic traces run in `cargo test --locked` with a fake `gh` (`FAKE_GH` in `tests/telegram.rs`: argument log, a token check that never records the token, issues, comments and labels in a state file, an option that makes an issue body echo the token), the fake TaskQ CLI as the registration readback, a fake Codex that makes several dynamic tool calls per turn (`multi`, or `hex` so that the person's text names nothing the calls use), the fake Bot API and the `PILOTQ_FAULT` crash points `before-effect` and `after-effect`. Fakes do not qualify GitHub, real Codex tool choice or Telegram; the live run is listed separately.

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

Results for the [section 10](adoption.md#10-adoption-sequence) step 5 slice, task [#26](https://github.com/taskqorg/pilotq/issues/26). The owner decided on 2026-10-23 that the pilot runs on the native compaction baseline: agentq 0.9.0 compresses only Codex shell results (`exec_command`, `shell`), PilotQ starts Codex with the shell tool disabled, and PilotQ's own tools are Codex dynamic tools whose results agentq passes unchanged. agentq qualification follows the adapter task [taskqorg/agentq#167](https://github.com/taskqorg/agentq/issues/167).

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
| L02 | Partial | — | Owner Telegram check after #91 merge and PM test-bot update |
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

Task [#28](https://github.com/taskqorg/pilotq/issues/28) applies R10, R13 and [section 6](runtime-contract.md#6-memory-and-agentq): every turn loads the sender's current route-visible profile, with revisions, before the person's request. An empty snapshot after forgetting supersedes earlier conversation and summary values. Profile values remain data, never grants or instructions. The contract is unchanged.

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
| A40.12 reviewer | Observe review and closure on the same launch status card (#165); fixture board issue is closed completed with explicitly synthetic result evidence |
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

### Task #68 reply-flow checks

The changed presentation follows [§4](runtime-contract.md#delivery), [§5](integration-contract.md#direct-actions-keys-and-project-switching) and [§12](runtime-contract.md#telegram-feedback). `telegram_reply_delivery_fallbacks` checks private drafts, one-second coalescing, reaction failure, draft failure, rich rejection, HTML entity rejection, final delivery and absence of DM reply quotes. `project_list_is_scoped_and_marks_current` protects registry/grant filtering and revocation. The journal cases protect duplicate receipts, queue notices across restart and ordered fallback receipts. A40’s discussion case now requires a real `project_list` call and reaction receipts without queued-text spam. Per the approved §9 handoff, the PM runs the updated suite after merge on FRA; the owner checks the DM flow. These remain unverified until their run and observation are recorded.

### Telegram layout and image input, task #72

The owner’s image decision is in [§1 and §11](adoption.md#11-decisions-still-open); presentation follows [§12](runtime-contract.md#telegram-feedback). The original rich-delivery regression failed because `rich_message.markdown` was absent. With the former text-only classification temporarily restored, `image_messages_are_input` failed with `other` instead of `image`; restoring the fix passes the same case. The initial image fixture used `Update` deserialization incorrectly; that fixture error is not product failure evidence.

- `telegram_reply_delivery_fallbacks` and `telegram::rich_markdown_keeps_layout_and_literal_source` protect paragraphs, list/heading/bold Markdown, literal HTML/code, and the retained durable HTML/plain fallback. The plan sample is `tests/fixtures/telegram-plan.md`.
- `telegram_images_reach_same_session` checks photo/image-document inputs with and without captions, actual local image bytes at the fake model boundary, session resume across restart, duplicate suppression and temporary-file removal.
- `telegram_image_failures_are_bounded_and_private` and `journal::image_identity_is_durable_and_scoped` protect ignored senders, grant revocation, durable file identity/caption, size/corrupt-file failure, localized short replies and credential exclusion. No failed image becomes a text-only successful answer.
- Existing journal migration, route isolation, queue/fence, receipt, retry and management checks remain required. Upgrade from schemas 8 and 9 explicitly refreshes native instructions without replaying completed effects. Schema 10 refuses unsafe downgrade to 0.2.3.

These are deterministic model/adapter observations, not Telegram rendering or real-model qualification. After merge, the PM publishes 0.2.4, updates FRA and runs A40; the owner checks the chat, including the one-line action effect and next step. No production-cutover claim follows from this task.

### Task #76: schema-9 journal repair (0.2.5)

R5 and §9 require existing journals to open without losing accepted work. A read-only FRA inspection on 2026-10-28 found `user_version=9` with both `requests.image` and `requests.language`. The original migration failed in `cargo test --locked schema_9_existing_image_columns_preserve_rows` with `duplicate column name: image`.

The version marker is not reliable evidence that these columns are absent: `v0.2.3` `Journal::open` checks the version only at entry and unconditionally writes `PRAGMA user_version=9` at exit. An old opener overlapping a v10 migration can therefore leave both columns stamped as version 9. The regression test reproduces that interleaving with two SQLite connections, as well as all four column-presence combinations, preservation of request text and existing column values, and repeated opens. This establishes a code path to FRA's observed state; the exact historical process interleaving is unconfirmed. Service-log access in the worker environment did not establish it.

The repair checks each column under the migration's writer transaction, adds only missing columns and commits version 10 with the migration. It removes the redundant unconditional version write. The existing schema tests retain older-journal upgrades, native-session refresh and refusal of newer or corrupt journals. No supported schema range or protection changes.

Rollback notification is tracked by [#77](https://github.com/taskqorg/pilotq/issues/77) (A5). After merge, the PM owns `live` publication, FRA activation and A40; the owner's screenshot answer supplies A4. These live checks remain unproved by the worker's SQLite tests.

### Rollback owner incident (#77, R11, §9)

`cargo test --locked rollback_notice` exercises the coreq launcher through two failed candidate starts and the rollback selection, then the restored service’s healthy-start notification boundary. The marker survives reopening the journal; the owner DM names only the failed and restored versions. Reopening after a transport failure preserves the same delivery and attempt count; reopening after a receipt produces no second incident. Healthy candidates produce no notice. Existing outbox checks retain the five-attempt ceiling, cooldown and held-destination protection.

This deterministic boundary check does not qualify a systemd restart or live Telegram delivery. A rollback to a release predating the integration cannot consume the marker; install a compatible baseline before relying on this notification.

### Compact reply formatting, task #82

Presentation follows [§12](runtime-contract.md#telegram-feedback). Inspection of the delivery path found three causes of lost layout:

- Rich Markdown and rich drafts passed a single model line break through unchanged. Markdown treats it as a soft break and renders a space, so status and error lines merged. `telegram::replies_keep_lines_and_compact_fallback` failed before the fix with `Task #12 is doing.\nA worker is on it.` instead of the hard-break form, and passes after it.
- The HTML fallback rendered headings, bold, code and links only. Bullets, `*italic*`, `__bold__`, Markdown escapes and horizontal rules reached the chat as raw marks. The same test now checks them.
- The plain draft fallback sent raw Markdown. It now uses the same escaped HTML subset.

The owner's review of the relayed #81 ask found three more defects, fixed in the same task:

- The question showed the CLI line `To answer: taskq answer N.K` and offered no buttons. It now drops that line and sends one inline button per option, the recommended one keeping its ★. A press is admitted like a reply with the option number, so grants, the fresh question revision and the stale-answer refusal apply unchanged.
- Task messages and model replies showed bare long URLs and link previews. Task messages now use labeled HTML links; a bare URL in a reply or a service message (for example the deferred plan-run report) becomes a labeled link; every send disables link previews.
- Repository paths such as `docs/status-notifications.md` were auto-linked by Telegram as `.md` domains. They now render as code.

`telegram::links_are_labeled_and_paths_are_code` and `telegram::button_press_is_input` cover the conversions. `u82_ask_buttons_and_labeled_links` renders a plan, a status, an ask with options and a stale-press error through the fake Bot API: it checks the buttons, the press answer `19.2`, no `To answer` line, no bare URL, disabled previews and each link's target. It timed out on the bare-URL status before the fix and passes after it.

The model instruction now asks for short paragraphs, flat lists, labeled links, file paths as code and no tables, nested lists, rules or HTML. The fake Bot API renders a hard break as a line break, so existing reply assertions keep their text. These are deterministic adapter checks with plan, status and error samples, not Telegram rendering or real-model qualification; the owner's chat check remains open.

### Service state and owner incidents (#84, R11, §9)

Task [#84](https://github.com/taskqorg/pilotq/issues/84) implements option A from #81 ([status and notifications](status-notifications.md)). Its SQLite checks are:

- `cargo test --locked status_shows_version_and_non_ok_sources` (A1). `/status` starts with `PilotQ <version>`. It then lists each non-ok source with its state and the age of its last observation, or `all sources ok`. A source that was never observed reads `unknown`, never ok. Raw failure reasons stay out of Telegram.
- `cargo test --locked service_incidents_hold_dedup_recover_and_cap` (A2). `taskq` and `poll` open an owner incident only after 2 minutes of failure. `model` opens one only after two consecutive failed runs; a success resets the count. Reopening the journal does not reopen an open episode. Only a later successful observation closes it. A recovery line is sent only for an episode whose opening was sent. Each incident key sends at most one opening per 6 hours. A capped episode still opens and closes in the journal, without messages. All service messages, the #77 rollback included, share the cap of 10 per day.
- `cargo test --locked downtime_and_unplanned_restart_notices` (A3). Each service pass records `alive`. On start, a stop without a planned marker records the gap from the last pass. `pilotq service stop` and the update drain both set the planned marker; an update that does not restart withdraws it. A crash loop keeps the earliest gap start. The next healthy start sends one notice. A gap over 5 minutes gets the downtime line; a shorter gap gets the unplanned-restart line. The `restart` key caps both at one per 6 hours. A rollback incident consumes the gap silently, so a rollback produces one message.

These checks do not qualify A4. A4 needs A40 and fault drills on FRA after merge: revoke the Codex login, stop TaskQ and kill the service. Detection is after the fact: a dead service is reported only on its next healthy start.

**FRA release and drills, 2026-10-28 (#84 A4).** `0.2.6` was published from `067dec1` and installed on FRA through `pilotq update` at 21:42:54 UTC. Two minutes later `current` was 0.2.6, the release was marked healthy, the binary matched `SHA256SUMS`, and `status`, `selftest` and `doctor` passed; the planned update sent no message. A40 passed 14/14 with cleanup at `067dec1`. The drills changed only PilotQ's own `config.json`, restored it byte for byte afterwards, and used planned restarts:

- Codex: a wrapper with an empty `CODEX_HOME` made two `pilotq ask` runs fail with 401 at 21:50:00 and 21:50:14. The incident DM was queued at 21:50:16 and delivered. After the restore, a successful ask at 21:53:18 produced the recovery DM at 21:53:44.
- Kill: `kill -9` of the PilotQ process at 22:08:30. systemd restarted it, and the healthy start sent `⚠️ PilotQ restarted without a planned stop` at 22:09:40.
- TaskQ: a failing TaskQ wrapper produced no incident in 7 minutes. With no subscribed task, `scan` returned before observing the board, so `taskq` health kept its last `ok`. `cargo test --locked scan_without_subscriptions_observes_taskq` failed before the fix (`an unreachable board is observed without subscriptions`) and passes after it: an idle scan now reads the configured root's snapshot. The TaskQ drill must be repeated on a release that contains this fix.

### Task #91 acceptance handoff

Owner option 1 updates §12: typing follows the active-call deadline; task links identify GitHub, repository/number and verified title. Creation held, admission waiting for a worker and execution are separate observations. Unchanged-state revisions do not repeat status messages; revised questions and closing outcomes remain durable.

Local regression checks cover the deadline boundary, unchanged-state revisions, escaped link titles and existing topic/edit/question/closure flows. The PM updates the test bot after merge. The owner then checks typing during a long call and its end after answer, error and cancellation, distinct creation/queue/execution text, readable links and absence of duplicate notifications. A4 remains unverified until that Telegram check is recorded on #91; fake Bot API checks do not establish visual acceptance.


## Pinned manager rules (#95)

Requirements: R13, §4, §5 and §12; task #95 A1–A3. The owner selected unchanged question relay: TaskQ owns source completeness; PilotQ does not invent missing reasons.

Before the fix, `cargo test --locked --test telegram u95_pinned_instructions_reach_codex -- --nocapture` failed with `pinned shared source absent from app-server instructions` against the previous manager policy. The same check passes with the pinned sources. It inspects the actual app-server `developerInstructions`, including shared reply and indicator content, task intake and PilotQ feedback additions. Updating the specq gitlink updates embedded source content without a Rust edit. Missing or renamed required sections fail explicitly rather than using a stale policy.

`cargo test --locked --test telegram u95 -- --nocapture` passes the instruction-boundary and scripted reply/task/question/status cases; the live case is ignored by default. The deterministic model proves wiring and rendering, not language-model compliance. The script checks a local plan with no board submission, one operation-ID creation before start, a goal of at most two lines, status without creation, short replies with labeled links and no internal request IDs, Russian input/reply, and complete question content preserved through HTML delivery with two buttons and previews disabled. Relaying the question causes no model turn. Existing stale-answer and grant checks remain in the suite.

`cargo test --locked instruction_change_rebuilds_once` passes: a changed fingerprint marks the native session for explicit journal reconstruction, preserves prior request/answer content and resumes the replacement session on later opens. The existing schema range remains 0–10; the policy marker uses existing metadata.

On FRA on 2026-10-29, `PILOTQ_A8_CODEX_HOME=<subscription home> cargo test --locked --test telegram u95_live -- --ignored --nocapture` passed all four cases in 29.25 seconds. This used the candidate PilotQ binary and real OpenAI ChatGPT subscription model `gpt-6.1-sol`, agentq off, with fake Telegram and TaskQ. The initialization user-agent was `pilotq/0.162.0 (Ubuntu 24.4.0; x86_64) unknown (pilotq; 0.2.7)`. No live board or installed service was changed.

Observed replies included “Saved locally as **plan revision 1**; nothing was filed or run”, “Created and queued” with a labeled task link and unconfirmed execution, status “**ready**, waiting for a free worker”, and “Привет, я на связи!”. The question preserved both named options, their plus/minus text and `Recommended: short wording, because the linked guide already covers setup.` Only the CLI answer line was removed. These finite cases qualify the model/adapter flow and relay, not every possible reply or visual acceptance in the Telegram client. Production cutover remains unauthorized.
### Answered Telegram questions (#94)

Under [§5](integration-contract.md#first-taskq-tool-contract), U94 starts with a delivered question in a forum topic. A trusted, unedited board answer from another surface closes that message in place with the author and answer, removes its buttons and preserves its topic. After restart and a newer question, tapping the old button shows the original accepted answer as a toast and issues no TaskQ answer command. Untrusted and edited entries do not supply receipts; duplicate scans do not enqueue another edit. The parser cases retain literal free text, option answers, question boundaries and attempt fencing.

Before the fix, `cargo test --locked --test telegram u94_answer_elsewhere_closes_question -- --nocapture` failed with `timed out: answered question edited`. After the fix the same check passed. This uses a fake TaskQ, board comments and Bot API; it does not qualify real Telegram delivery or a deployment. TaskQ #1785 has not yet published its snapshot fields; the approved transitional path reads paginated GitHub board comments using TaskQ's trust and edit rules.

Review regression (#94): the post-answer snapshot returns to the older attempt revision; U94 must still close the question and reject a stale tap. Before the correction this trace timed out waiting for the edit, and the duplicate-operation parser case reported the ignored answer instead of the accepted answer. Both cases pass after removing the revision ordering filter and deduplicating trusted, unedited log entries across the question boundary. Wrong-attempt answers and edited stale writes remain excluded. TaskQ §2 treats the trailing revision as a writer readback fence, not a fold predicate; the writer edits a rejected entry, and edited entries do not consume operation IDs.

### U93: one draft and explicit action buttons

Historical evidence below describes #93. Task #165 supersedes its verbose plan rendering; current checks retain the same authorization and recovery boundaries with three-line task summaries.

Owner decision: [§5 Plan discussion and action buttons](integration-contract.md#plan-discussion-and-action-buttons), task #93 option 1. The previous adapter instructed the manager to present every saved revision as a new plan revision and offered no launch button. `u93_draft_display` reproduces the display defect on the unchanged implementation: a saved scope could end with only `Saved.`. The same check passes with the saved title, goal, acceptance, scope and launch affordance rendered from the journal.

The executable checks retain these protections:

- `u93_draft_buttons_through_service`: three sequential additions keep one draft, discussion writes no board state, the latest displayed button survives restart and performs one held create/readback/start, and old/repeated presses add no mutation. Discussion choice buttons submit the numbered answer without launching. The real service runs against fake Codex, TaskQ and Bot API adapters in rich, HTML and plain fallback modes; plain fallback is explicitly observed.
- `u93_choice_and_authority_boundaries`: reject another person in the same shared topic, another topic/chat, an unknown message receipt, a revoked grant and a reset generation, including reset between admission and dispatch. A numbered text answer consumes the same question and preserves the exact input in the audit; a discussion choice cannot call `task_run_plan`. Explicit separate plans and post-launch changes retain separate identities and leave admitted work intact.
- `u93_schema_10_keeps_work_and_refreshes_tools`: upgrade an actual schema-10 shape, preserve saved work, refresh the native tool catalogue and reopen without duplicate plans. Existing older-schema and newer/corrupt-journal tests retain their protections.

Development evidence: the display check failed before the fix and passed after it. `cargo test --locked -- u93 schema_ plans_and_task_projections --nocapture` passed (9 journal checks and the three-mode service scenario). The broader parallel affected-check run ended with exit 137 after partial results; it is sandbox-blocked, not PASS. Candidate checks and their exact revision are reported on the task/PR. Adapter evidence does not establish real Codex intent interpretation or Telegram client appearance; no live bot, installation, release or production cutover is part of this task.

Review correction (#93, §12/#91): the button launch reply previously said `Tasks created and started` for a `ready` task and omitted its saved title. The strengthened `u93_draft_buttons_through_service` failed on that reply before the correction. Launch receipts now report admission to the worker queue and use the shared task link formatter with the saved title. `u93_draft_display` also checks English and Russian admission replies at the journal boundary; the callback authority checks remain unchanged. The affected service rerun was sandbox-blocked (exit 137), so it is not passing adapter evidence. Candidate checks are recorded on #93 and PR #99.

Task #93 integration with #95: the rebase retains pinned instruction loading, fingerprint-based session reconstruction and the schema-11 draft/card migration. The filtered `u93`, `u95` and `instruction_change_rebuilds_once` checks pass with fake adapters. The image-session check failed before correction because its caption/image assertions still used the positions from before draft context was added; the corrected assertions pass and retain caption/image pairing, session continuity, deduplication and temporary-file cleanup protection. The earlier CI U06 audit failure did not reproduce in the isolated check; its CLI exit status and stderr are now asserted before JSON parsing so any recurrence identifies the failed operation. Candidate checks are recorded on the task and PR; live acceptance remains unverified.

### Related Telegram messages (#92)

Under §2 and §12, check an image with a caption followed by long text, several images followed by text, and an addition received during generation. The next turn retains image bytes, authors, captions, reply references, the previous outcome and task identity across restart. No time window or mandatory reply selects a task. Clear additions steer a matching doing task after fresh status; independent work stays separate. Ambiguity produces “to task X or new?” with options and a free-form answer invitation before a mutation. Images alone grant no action rights. Check isolated routes, `/new`, revoked senders and historical download failure; absent context remains explicitly unknown. Adapter fixtures establish input delivery, not semantic model quality.

The deterministic checks are `cargo test --locked --test telegram telegram_images_reach_same_session` and `cargo test --locked --bin pilotq related_context_is_durable_and_scoped`. They cover byte delivery, durable follow-ups during generation, captions, album identity, quoted reply text, explicit historical image gaps, isolation, revocation and ambiguous or changed task registrations.

For semantic qualification, set `PILOTQ_A8_CODEX_HOME` to a subscription-authenticated Codex home and run `cargo test --locked --test telegram related_live_context -- --ignored --nocapture`. Repeat with `PILOTQ_RELATED_CLARIFY=1` for a free-form answer to a journaled clarification fixture and `PILOTQ_RELATED_INDEPENDENT=1` for a separate saved plan without duplicating the existing task. The default run checks clear steering and ambiguity without a write. All modes use the real Codex adapter and local fake Telegram/TaskQ services; none writes to a real board. These are bounded scenarios, not a guarantee that every natural-language ambiguity is classified correctly.

### Voice input, task #104

The owner's voice decision is in [§1 and §11](adoption.md#11-decisions-still-open); the path is in [§12](runtime-contract.md#telegram-feedback). With voice temporarily classified as before (`other`), `telegram_voice_becomes_sender_text` failed waiting for answered voice requests; restoring the classification passes it.

- `cargo test --locked --test telegram telegram_voice_becomes_sender_text` sends an OGG/Opus voice note, an MP3 audio file with a caption and an M4A audio document through the fake Bot API, fake ffmpeg and fake recognizer. It checks the marked transcript after the caption at the Codex boundary, journal file identity and transcript, recognizer arguments, temporary-file removal and that no audio, download path or token reaches Codex.
- `cargo test --locked --test telegram telegram_voice_failures_are_short` covers an undecodable file, a file over 20 MiB, a recording over 5 minutes, missing configuration and an ignored sender: one short localized line, no model turn, no download where admission already decides.
- The fakes check wiring, not recognition quality or the real binaries.

FRA CPU evidence, 2026-10-29: AMD EPYC 9575F, 8 vCPU, no GPU; `sherpa-onnx-offline` v2.0.1 static, GigaAM v3 RNNT-punct int8, 4 threads. Input: the first 100 FLEURS `ru_ru` test recordings (18.9 minutes, read speech, up to 33.8 seconds), re-encoded to Telegram-like OGG/Opus mono 48 kHz 24 kbit/s. Each recording ran the production steps as separate processes: ffmpeg decode to 16 kHz mono WAV, then one recognizer process including model load.

| Measure | Result |
|---|---|
| WER / CER (lowercase, no punctuation, ё→е) | 4.92% / 1.40% |
| RTF, end to end | 0.065 |
| Latency p50 / p95 / max | 0.72 s / 0.87 s / 1.11 s |
| Recognizer peak RSS | 378 MiB |

The same binaries with the exact production arguments confirmed: a garbage file makes ffmpeg exit nonzero; a 400-second recording decodes past the 5-minute size limit and is refused; a 5-minute recording peaks at 611 MiB recognizer RSS. At RTF 0.065 a 5-minute voice note needs about 20 seconds of the 60-second turn deadline.

Retained risk: FLEURS is read speech; spontaneous, noisy Telegram voice notes from real users were not measured. The non-Russian reply depends on the model following the §12 instruction; it is not a language detector.

### U108: saved answers and semantic addressing

Owner decision: §5, task #108 option 2. A person may answer without a Telegram reply. The manager identifies a subscribed question by meaning, clarifies an ambiguous destination or answer, and forwards the person's own text through `task_answer` at a freshly read revision. It does not choose for the person. A source input from another person, route or reset generation cannot authorize the answer. `discussion_answer` saves a free-text discussion answer locally; #93's superseded-card and separate-launch rules still apply.

The regression boundary is the journal plus the real service with fake Codex, TaskQ and Bot API adapters. `u108_semantic_answer_receipt` covers two open questions, a free-text answer to one exact revision, the original card edit and a clarification card selecting the destination, forwarding the earlier source text, and a late button press after restart without another write. Journal checks cover durable discussion receipts, source ownership, stale cards and refusal of launch from a choice. The uncertainty case covers button, reply and semantic answers after a lost CLI response and restart, checking stable operation IDs. Existing U07/U82/U94 retain revision, grant, link and external-answer protections; their success assertions now require the saved choice and factual state, and a late press shows the saved answer under #94. Before the fix, the semantic service check failed waiting for the saved choice in the original card; it passed after the fix. These checks qualify adapter wiring; they cannot establish language-model interpretation or Telegram appearance.

A6 is a post-merge PM check after release: record a question, its button and text answer paths, the saved choice with removed buttons, the next observed TaskQ state, a stale/repeated press, and an ambiguous free-text answer that asks which question. Attach the Telegram evidence to the release note. Do not mark A6 passed from the local adapters. Production cutover is not authorized.


## Voice installation readiness (#120)

Task #120 A2 extends §9 doctor to the local §12 voice path. `cargo test --locked doctor_voice_readiness` failed before the change because `checks.voice` was absent. The regression checks missing configuration, missing recognizer, missing model files, recognition failure, malformed output, timeout, missing ffmpeg and a working silent probe. With other setup checks passing, a failed voice check must make the command exit nonzero; a working probe makes it succeed. Dependency output stays private, and doctor neither creates a journal nor polls updates. Existing `telegram_voice_*` checks retain grant, limit, transcript, audit and short-failure protection.

On FRA, the initial live request 68 failed with `voice recognition is not configured`. The operator installed ffmpeg and the documented sherpa-onnx v2.0.1 static recognizer and GigaAM v3 model, saved absolute dependency paths in protected configuration, gated new claims, waited for zero running requests and restarted only the test PilotQ service. A protected configuration backup allows restoring the previous setup. No installed PilotQ release was replaced.

Downloaded archive SHA256 values matched GitHub release metadata:

- sherpa-onnx static archive: `ea25c18660047a8ec8aa0cda291ccf806ae9c18a9b3b76372399cc4a7f4d7663`.
- GigaAM model archive: `f9620a0099019c6afcee26525ef9ed3297fa50dd5691c1902af0c948fc1a470b`.

The candidate doctor on FRA returned all checks `ok`, including `voice`. The model's Russian release sample returned a nonempty Cyrillic transcript with exit code 0. These prove local setup, not A3: a new real Telegram Russian voice note, transcript audit event and delivered answer remain required. Request 68 preserves the prior short failure and is not replayed. The doctor code still requires PR acceptance and the §9 release path before installed use; production cutover is not authorized.

## Built-in commands and model selection (#106)

The owner selected per-conversation manager models; worker policy remains with
TaskQ ([§9](service-contract.md#command-line), R9, R13). FRA requests 37–41 on
2026-10-29 sent `/version`, `/model`, `/doctor`, `/help` and `/restart` to Codex;
the owner reported `/restart` waiting 40 minutes. The deterministic reproduction
`cargo test --locked u106_commands_bypass_active_chat` failed before the fix
with `/version forwarded as text` and passed after dispatch was corrected.

Run `cargo test --locked u106` for these retained protections:

- The journal replay keeps all five commands out of Codex during an active run.
- The local Bot API checks actual command replies, model buttons, duplicate
  selection refusal, selected `turn/start` arguments, `/new`, service restart,
  unsupported skills, CLI parity and command errors. It uses invented models and a
  fake runtime; no real inference is performed.
- Journal authority checks cover another person's button, revocation, unknown
  models, stale generations and a default change during an active run. A late
  old-model result cannot clear the pending conversation rebuild.
- The setup subprocess check covers deadline termination and output redaction.
- The restart receipt check requires a replacement process and the existing
  healthy-start boundary. The existing `update_drain_timeout_preserves_running_work`
  check retains refusal of activation while an active request has not drained.

These are model/adapter and process checks, not a live Telegram rendering or
installed-service restart observation. A7's live Telegram checks, including an
installed restart with an active request, remain unqualified until performed on
an authorized test installation; no deployment or production cutover is implied.

## Status footer (#119)

Historical placement and evidence: #165 moves these observations to `/status` and removes reply footers. The current delivery check is `u165_status_observations_without_reply_footer`; indicator validation remains under `u119`.

The owner selected the reply footer for DMs, groups and topics; see
[research and placement review](status-line-research.md) and contract §12.
`cargo test --locked u119` protects indicator thresholds, reset rounding,
invalid/stale readings and the full fake-app-server → journal → rich/HTML/plain
Telegram path. It also checks that missing quota cannot fail an answer, late
snapshot data cannot overwrite a newer notification, and another thread cannot
supply context. The existing `u106_telegram_controls_and_models` chain checks
that `/new` names the saved model after restart without starting inference.
These tests do not qualify Telegram appearance or live subscription readings.

A4 was performed by the PM on FRA on 2026-10-29 with code candidate `de82e17`:
[report and Telegram screenshots](https://github.com/taskqorg/pilotq/pull/122#issuecomment-6493225953).
The run covers `/new`, a model switch, DM/topic replies and an unavailable quota
source. The report records journal continuity and restoration of release 0.2.10;
it also retains a nonblocking recovery-notice wording observation. See the
[exact build and evidence](status-line-research.md#implementation-and-live-evidence).
Worker question cards retain their no-footer rule under contract §12.


## Conversation project and board status (#124)

Contract §5 and §12 own this behavior. `project_context_boundary` and
`u119_status_footer_delivery` failed before the fix: the earlier “Pilot Q”
mention resolved to no project, and a final answer with trailing newlines kept
extra empty lines before the footer. Both checks pass after the fix.

Retained checks:

- `project_context_boundary`: earlier own naming, spaced product spelling,
  multiple fits, clarification, quoted-name refusal and revoked project grants.
- `project_status_context_read`: fresh registered-board CLI read, unchanged
  task/runtime/session fields and read time, draft target from earlier wording,
  incomplete/mismatched/missing task lists, and revoked grants. The fake CLI
  refuses every command except `status --json`.
- `related_context_is_durable_and_scoped`: earlier own inputs survive restart;
  other authors, private routes and reset generations cannot select a project.
- `u165_status_observations_without_reply_footer`: no footer through rich, HTML
  and plain delivery in each route; measured or unavailable observations appear only in `/status`.

A8 remains a post-release live FRA check. In the owner DM/topic, ask “What is
the task status for Pilot Q?”, then “Make a task so this works”; verify fresh
board status and a local `pilotq` draft without another project question. Check
an ambiguous two-project request asks for the project and writes nothing.

## Answering the project question (#157)

Contract §5 owns this behavior. Root cause: `target_inner` propagated
`context_project`'s “Which project” error even when the call named an explicit
project, and the person's answer “1” named no project, so the history stayed
ambiguous and `task_plan` was refused again.

The boundary now records its own question options for the request. The
person's next message may answer with a bare number, counted in the order the
reply listed them, or by name.

Retained check: `numbered_answer_selects_project` failed before the fix at the
answer step. It covers a bare “1” with no open question (refused), earlier own
mentions of `pilotq` and `tight` (question, no draft), the answer “1” (draft on
`pilotq`) and a later bare number after the question closed (refused).
Record the exact running build and Telegram screenshots showing project selection and the current #165 cards.
Local adapters do not qualify this live appearance or model interpretation.

## Approved installed commands (#126)

Owner-approved capability list: [contract §5](integration-contract.md#installed-read-commands).
Before this change, `shell_tool` is disabled and no dynamic tool executes an
installed read command; `taskq list` and `pwd` have no common dispatch path.

- A126.1: with a protected operator entry, direct `taskq status` and an unambiguous
  natural-language request return the installed program's actual result through
  the same mechanism as `pwd`. Ambiguous requests require clarification.
- A126.2: unknown commands, unapproved argument arrays, absent executables,
  nonzero exits, timeout and output overflow remain distinguishable failures.
- A126.3: current entry permissions, project grants, board/root registration,
  request fencing and the original deadline constrain execution. Output cannot
  grant rights or select another command. No inherited credential environment.
- A126.4: CLI and Telegram retain request audit correlation through discovery,
  dispatch and final delivery. Existing direct-write and TaskQ guards remain.

Retained checks: `approved_command_boundary` protects capability enforcement and
process limits; `u126_installed_commands_share_frontends` runs real PilotQ with
fixture Codex/TaskQ and installed fixture commands through CLI and fake Telegram.
The latter proves wiring and factual subprocess output. A development CLI run
with real Codex under the subscription login also executed `pwd` and a Russian
request for `taskq list`: the journal recorded the actual cwd with exit zero for
`pwd`, and exit one with ``Error: unknown command `list`; use `taskq help` `` from
installed TaskQ. The answer reported that error rather than inventing a list or
substituting another tool. Thus command dispatch and failure reporting are
observed; successful listing with that absent TaskQ subcommand is not claimed.
The owner then corrected A1 to `taskq status` (answer 6497254373). The
operator example and retained CLI/Telegram fixture now permit `status`, preserving
the argv refusal, permission and audit protections (R13, R17, §5, §7).
On 2026-10-29, real PilotQ with subscription Codex and installed TaskQ 4.6.40
completed the Russian status request in isolated journal request 8:
`command_run taskq` recorded `start args=["status"]`, then
`{"status":"success","exit_code":0}` with empty stderr and actual board output
(`In work 2 · Waiting for answer 0 · Ready 0`, tasks #127 and #126).
The final answer reported those observations. Before: request 6 recorded
`exit_nonzero`, exit 1 for `list`. After: request 8 recorded success for `status`.
An initial retry (request 7) failed with Codex 401 because the test login had been
removed; the successful run used the existing host subscription login, then
removed its temporary copy. No model/API fallback or live service change occurred.
This run used the executor code from `7da47b9`; the continuation changes only the
contract, operator example and fixture. Live Telegram remains unqualified; no
deployment is part of #126.

### Telegram reply files (#145)

Under [§5](integration-contract.md#5-taskq-integration), photos and documents with optional captions reach TaskQ answer, steer and owner requeue as readable files. Local `reply_files` checks protect identity/caption admission, duplicate and ignored updates, source isolation, bounded HTTP downloads, readable CLI files, original revision and same-operation retry. Existing question/grant/fence checks continue to apply. A refusal or uncertain write retains the reply and explains its reason and fix. The PM executes the [A2 live recipe](reply-files-145.md) after release on PDS 18, recording exact client/TaskQ revisions, a photo, a 20 MB document, mixed text/files and verified worker copies. A2 is not qualified by fake transport or CI.

### Administration by request (#160)

R15 and §5 own the grants and limits. Deterministic component traces use invented identities, a fake TaskQ executable and a local Bot API HTTP server; they do not qualify a live installation.

| Case | Initial conditions and input | Expected observation |
|---|---|---|
| Admin160-01 | `projects` administrator requests registration of an existing repository | Safe local root and matching `taskq.json`; complete matching TaskQ readback precedes registry commit. Result names board/root and missing filing prerequisites. |
| Admin160-02 | `users` administrator grants/revokes an exact route-seen ID, including a verified reply sender | Atomic before/after grant audit and receipt; unseen IDs and names refused. |
| Admin160-03 | Caller lacks required grant or one of `users` and project access | No external call or mutation. Group-admin badge is insufficient. |
| Admin160-04 | Duplicate update or repeated tool call, followed by a later grant correction | One request/operation; original receipt returned without reapplying corrected grants or creating another topic. Changed arguments refused. |
| Admin160-05 | Crash after local-root write, topic creation response loss, or known topic ID before route binding | Partial root remains visible; unknown topic creation is never replayed. A saved ID completes only local binding after current grant/registry checks. Grant receipt failure rolls back the entire change. |
| Admin160-06 | Forum bot has or lacks `can_manage_topics` | Permitted topic gets the selected route policy and registered project default; new conversation resolves that project without gaining access. Missing permission receives a plain refusal. |
| Admin160-07 | Model substitutes unrequested grant types or shared mode; quoted/negated shared wording; recovery holds a saved topic ID | Refuse before mutation/API; no grant intent is stored. Recovery cannot bind shared visibility without caller consent. Explicit own grant names and English/Russian shared-topic requests remain accepted. |

### Conversation contract (#165)

The [Conversation contract](conversation-contract.md#conversation-contract) and [stage table](conversation-flow.md) own presentation. These cases amend the #72/#91/#93/#110 plan/status assertions and #119/#124 footer assertions; they retain authorization, immutable scope, grant, duplicate, receipt, route and recovery protection.

| Case | Input / condition | Expected observation |
|---|---|---|
| Chat165-01 | Save or revise an English/Russian draft with several acceptance/scope items | Each task shows title, one action line and one result line; full worker body stays saved. No local-save, board, revision or launch-instruction prose. |
| Chat165-02 | Runtime reports model/context/quota; later data is unavailable, model changes or conversation resets | Answer has no footer; `/status` shows only the current conversation generation/model epoch's observation with its age or unavailable. |
| Chat165-03 | Launch, Cancel or choose; repeat, foreign or reset callback | Edit the source display, remove buttons and append one factual outcome; stale controls receive a toast and write nothing. |
| Chat165-04 | Multiple tasks in one launch transition through queue, work, question and closure | One launch status card is edited; questions retain complete answer cards and accepted-actor edits. Closure never invents publication or deployment evidence. |
| Chat165-05 | Slow launch, dependency outage, terminal failure and restart | Phase/elapsed line edits the source card; coalesce and rate-limit edits. One failure and one verified resolution; details remain in audit. Give-up shows Retry/Cancel. Retry preserves operations and prior effects; Cancel stops only remaining launch steps. |
| Chat165-06 | Render all cards, stages and the owner's two before-case transcriptions | Fixture snapshots match; no operation ID, JSON, internal revision or English CLI error reaches the rendered cards. |

`conversation` component filters cover snapshots, durable edits, grouped task states, progress and problem controls. The Telegram `u165_` filters cover the service/adapters, `/status`, elapsed-time edits and Retry/Cancel; updated existing route/recovery filters keep their authority and receipt assertions. These are local/CI checks, not live appearance qualification.

A7 remains a PM check after merge/release on PDS 18: record exact source/binary identity, then compare a plan, launch, cancel, complete worker question and failure/recovery to the fixture cards using an existing authorized test route/project. Verify same-message edits, button removal, stale toast, elapsed-time liveness and Retry/Cancel. Attach screenshots and receipt evidence under §9/§12. Workers do not install or replace the running service.

### Project onboarding (#168)

Owner-required cases under [§5](integration-contract.md#project-onboarding):

- **O168-1:** register an explicitly supplied root with a matching board and claim,
  Codex limit 4 and no operator grant. Registration remains saved; the CLI exits
  nonzero, readiness is false and the owner DM lists the exact project grant command.
- **O168-2:** resolve the grant outside PilotQ and tap Recheck. Re-run doctor and the
  TaskQ fallback, edit the same DM card and remove buttons when all checks pass.
  Neither check runs the manager, polls Telegram updates or changes host policy.
- **O168-3:** missing, incomplete, stale, mismatched or failed checks remain gaps.
  Block add/start before board effects; preserve this gate and its report on restart.
  The protected owner's written override bypasses only PilotQ readiness, keeps
  failed findings, and is invalidated by recheck or registration changes.
- **O168-4:** foreign, duplicate, old-generation and superseded buttons authorize
  no check or mutation and receive only a toast. Lost/deleted cards can be replaced
  only in the original owner DM. Failed delivery stays in the existing outbox.

- **O168-5:** with `pilotq-github` permitting `issue_read` but granted to `pilotq`
  alone, report a key gap for each of `cyprusbz`, `tight` and `agentq`, with the
  exact `pilotq keys grant pilotq-github project:NAME` fix. Grant one externally
  and Recheck; only that project clears. Preserve existing owner-wide and route
  grant semantics, never expose values or update last use, and keep unreadable
  stores unknown. Existing onboarding reports without this check are rechecked
  at service startup.

Local regression: `cargo test --locked onboarding_168`. This covers the CLI and a
simulated Bot API; a live owner Telegram run remains separate target evidence.

### Plan setup refusals (#236)

Under R11, R13 and §5, `cargo test --locked onboarding_236` protects A1–A3:
English/Russian setup refusals survive tool-to-final-reply rendering; closed-gate plan cards
name the project and owner Recheck step and retain Cancel without Run plan. Saved admission
and service gaps refresh through read-only TaskQ service status. Stale or unavailable service
evidence, missing grants, mismatched roots and unrelated key findings still block launch.
Before the fix the card check failed with `closed setup offered Run plan`; after correcting
that path, the reply check exposed `The request failed; try again or check /status.` instead
of the setup refusal. Both now pass. Disabling automatic refresh makes the recovery case fail
at `admission/open`; restoring it passes. Existing `onboarding_168` checks retain owner-only
overrides, registration fencing and CLI/simulated Telegram recovery coverage.

A4 remains PM target qualification after release (§9): in the PilotQ TEST cyprus.bz topic,
request a plan and press Run plan; record the release identity, both audit IDs, displayed
readiness/refusal and launch outcome here. The worker neither installs live nor claims this replay.

### Task status facts and research routing (#170)

Under R8, R13 and §5, `src/fixtures/status-170.json` is a synthetic fixture for the owner’s cyprusbz#540 case: closed, diagnosis accepted, report `docs/research/540.md`, fix #541 doing. `status_research_170` retains the report and trusted verdict, separates proposed references from observed task existence, and checks English/Russian diagnosis and investigation mapping through saved plans and `add --type research` arguments. Untrusted close comments and missing snapshots remain unknown. `gitlab_registry_status_and_urls` checks the CLI dispatch boundary, including an absent open-board task, issue readback and mismatched issue refusal.

These deterministic checks do not qualify live model wording or Telegram. Per the owner’s #170 answer, the PM runs the #540 status question on PDS 18 after release and posts the actual reply text on #170 (A4).

### Chat files (#186)

Owner live test 2026-10-31 in a direct chat: a 310 KB PDF and a 15 MB ZIP were answered with a description of an earlier header screenshot plus "content unavailable"; a 20.4 MiB RAR got "The image is too large"; a 1:02 video got "format is not supported yet"; every refusal was English although the owner wrote Russian.

Root cause, from the code path these requests took: a document message carried no readable content into its turn, while every turn attached all images of the recent window as unlabeled `localImage` items. The only picture in the PDF and ZIP turns was the earlier screenshot, so the model described it as the attachment. Ingress classified any oversized document as `image_limit` and every video as `other`. Refusals followed only the Telegram client label. The worker had no access to the PDS 18 audit, so the request audit IDs are not recorded here; A5 records them.

| Case | Input | Expected observation | Check |
|---|---|---|---|
| Files186-01 | Russian text, a photo, then a text document and a RAR document from an English-labelled client | The document turn carries `[File NAME, attached to this message …]` with the content or the exact reason, `current_message_files` lists only that document, and the earlier photo is labeled as an earlier message; `file_read` audit rows record the outcome | `telegram_documents_are_read_in_their_turn` |
| Files186-02 | Same chat, an uncaptioned 21.4 MB RAR | One Russian file-limit line, no download, no word "image" | `telegram_documents_are_read_in_their_turn` |
| Files186-03 | PDF, stored ZIP, UTF-8 text, RAR, binary, damaged PDF | PDF text with page count, ZIP listing, text, or the exact reason | `documents_become_text_or_a_reason` |
| Files186-04 | Video, video note, video document, oversized video and document | Video kinds read as video; files over 20 MiB are `file_limit` | `videos_and_large_files_ingress` |
| Files186-05 | 62 s generated video through real ffmpeg | Four JPEG frames; garbage input names the decode failure | `video_frames` (ignored in CI: needs ffmpeg) |
| Files186-06 | Every image turn of #72 | The current image is labeled as this message's, earlier images as earlier | `telegram_images_reach_same_session` |

On the base commit `telegram_documents_are_read_in_their_turn` failed at its first document assertion: the text document's turn input was empty text. These checks qualify the adapters, not model reading quality. A5 is a post-release PM check on PDS 18: an authorized tester sends a PDF, a ZIP, a file over 20 MiB and a video in a Russian chat; record the replies and request audit IDs. Production cutover is not authorized.

### Unified capability catalog (#163)

[§9](../pilotq.md#9-cli-releases-and-service-operations) owns the command model (R13, R17). Component checks use invented identities and local TaskQ stubs; they do not qualify Telegram delivery or an upstream TaskQ help schema.

| Case | Input and initial conditions | Expected observation |
|---|---|---|
| Command163-01 | Add or remove a slash control or manager tool | Catalog completeness check detects an unmatched tool; slash parsing and descriptions use catalog entries. Each entry has area, purpose, example and grant. |
| Command163-02 | Authorized caller asks `/help`, `/help projects` or CLI `help projects`; another manager turn is active | Independent control path records and delivers the catalog answer without starting Codex or cancelling the active turn. Top-level help has introduction, three examples and one line per accessible area. |
| Command163-03 | Compare chat-only, project-scoped and administrator callers, then revoke a project or key grant | Current inaccessible capabilities disappear; help includes the access-request line. English/Russian help follows the caller language. |
| Command163-04 | Administrator disconnects an explicitly named project; repeats the call, changes arguments, or supplies only quoted wording | Atomic registration/default removal and original receipt; changed arguments, quotes, missing grants and stale fences refuse. Files, board, grants and task history remain; legacy startup cannot re-enroll the removed name. Reconnect uses normal verified registration. |
| Command163-05 | Stub changes its machine-readable TaskQ help between calls; old CLI, missing CLI, invalid JSON or output overflow | Latest JSON appears without a PilotQ change; unavailable cases give one short line. The live catalog is external source JSON, never a vendored command list. |
| Command163-06 | PM runs `/help`, `/help projects`, then “which projects can I use” with an authorized test-bot caller | Read back all three Telegram reply texts, caller language and exact PilotQ source/binary identity. Confirm the natural-language answer uses `project_list` and actual accessible registrations. A5 remains unverified until this PM run. |

### Rich delivery and concise help (#189)

Under §4 and §9, all user-visible content kinds use shared rich delivery, including builtins,
notices, errors and final answers. Native reactions, callback toasts and button-only edits keep
native methods. Content up to 32768 UTF-16 units is one rich message; explicit rejection activates
persisted HTML chunks of at most 4096 units, with plain fallback on entity errors. Each received
chunk survives restart without replay; ambiguous rich sends retain their method and destination.
The outbox-kind regression includes native exceptions and a future content kind to protect the
default path. English and Russian overview help remains one short command list; TaskQ's live
catalog is requested only through `/help tasks` or `pilotq help tasks`.

PM checklist for A6, after the merged release on pds18 (§9; no worker installation):

1. Record the installed release version and source SHA. In an authorized DM, send `/help`, first
   with English and then Russian configured for the person. Expect one rich message with a short
   command list and `/help tasks`, without the inline TaskQ catalog. Retain screenshots and the
   matching successful `sendRichMessage` delivery evidence.
2. Send `/help tasks`. Expect task help and the fresh machine-readable TaskQ catalog as rich
   content when within 32768 units; oversized content must use ordered HTML chunks. Record the
   actual size and method. An unavailable catalog must produce its availability line honestly.
3. Send `/help invented`. Expect a short localized unknown-area error notice through
   `sendRichMessage`, with no model invocation or task creation.
4. Ask: `Write an answer containing 5000 to 6000 characters about planning work. Do not use tools
   or create tasks.` Expect one final rich message when the observed answer is between 4097 and
   32768 UTF-16 units. If the model returns less, record that the long-answer case is unqualified
   and request the required length again. Retain actual length, screenshot and delivery receipt.
5. Repeat the control and error checks in an authorized topic. Verify the same topic and original
   reply destination, and confirm existing cards retain edits and remove used buttons.

Local adapter/model tests do not qualify Telegram rendering or replace this PM evidence (R19).

### Scoped profile and access self-service (#196)

R10/R13 and §6: an admitted person uses `/profile`, `/profile set KEY VALUE`,
`/profile forget KEY` and `/access` in their own DM without a model turn. Inspect
returns current visibility/source/revision; correction and deletion return durable
readback. Foreign subject arguments refuse; group/topic replies contain only DM
guidance. Restart/reset retain current facts and grants; forgotten/superseded facts
stay absent from the fresh model profile, even when history contains them. Profile
edits never grant access, and revoked people remain ignored. The local executable
trace and its before/after evidence are in [profile-196-evidence.md](profile-196-evidence.md).
Real Telegram presentation remains PM qualification under §9.

## Input revisions and route incidents (#197)

R5/R11 and runtime §§4, 12 retain I202–I204 and I801–I803 from the
[contract inventory](contract-177-inventory.md). Source request:
[task #197](https://github.com/taskqorg/pilotq/issues/197).

- `edit_and_route_events_reach_ingress` checks edited text, topic closure, migration,
  bot removal and permission loss at the Bot API adapter boundary.
- `edit_revisions_and_route_holds` checks queued/started/duplicate revisions, original
  author and current grants, audit retention, restart persistence, unaffected routes
  and deduplicated owner incidents. A held route retains pending output without replay.
- `edit_preserves_launch_scope` rejects launch of an edited proposal source and retains
  immutable launch identities after later edits.
- `route_delivery_failure_holds_later_output` holds future output after a missing-topic
  delivery failure. Telegram has no topic-deletion update; that case starts at delivery.
- Existing `service_messages_are_not_input` and `strangers_and_claims_change_nothing`
  retain no-model service events and ID-only ignored senders.

These are local component/process checks. Real Telegram lifecycle qualification remains
with the PM under §9; no installation or production cutover is authorized.

Before/after evidence: with the three `edit_` regressions applied to the unchanged
`bf82a9671fb7af000135af9c638962cdd0cfda10` implementation,
`cargo test --locked --bin pilotq edit_` failed all three: the adapter dropped an
`EditedMessage`, queued edits created extra request rows, and a started edit produced
no claimable change request. The same command passes all three after the fix.
`cargo test --locked --bin pilotq route_delivery_failure_holds_later_output` and
`cargo test --locked service_messages_are_not_input` also pass. The retained tests
protect route identity and service-event admission rather than requiring new model calls.

## Route loops and board wakes (#220)

The [owner decision and protocol](conversation-contract.md#route-loops-and-board-wakes) bind these cases (R5, R10, R13, R17). `loop_220_cli_route_exclusion` protects both directions of CLI/tick mutual exclusion. `loop_220_board_adapter_shared_cursor` protects bounded installed-CLI readback, confirmed closure and shared board cursors. `loop_220_controls_and_dynamic` protects fixed/dynamic bounds, bot suffixes, CLI/Telegram common admission, refusal and stale tick decisions. `loop_220_restart_drain_coalesce_revoke` protects durable armed/pending timers, restart/drain, queued coalescing, revocation and reset. `loop_220_running_reset_and_revocation_fence` protects immediate fencing of running ticks after reset or shell revocation. `loop_220_three_routes_boards_and_human` protects independent boards, baseline/deduplicated wakes, review options, failure/closure, drain-held events, human priority and route context/reply isolation. Adapter collection and target-platform timing require their own evidence; [the PM checklist](loop-220-live.md) covers real Codex and Telegram TEST qualification without authorizing worker installation.

## PM packs and scoped notes (#221)

The [root contract §9](../pilotq.md#9-cli-releases-and-service-operations) owns the approved pack boundary.
The worker stage supplies [the owner-installable runbook](pm-pack.md) and [the PM checklist](pm-221-live.md).
`cargo test --locked pm_221 -- --nocapture` covers pack binding, fresh context on resumed turns, note tools,
route/DM isolation, credential refusal, restart, `/new`, CLI correction/forget and revoked access.
These are journal/process/adapter checks, not real model parity or live qualification (R13, R16, R19).

The owner confirmed a staged handoff in task #221: worker implementation/checklist first, then the PM's one-hour
findings-only shadow run. The PM must replace the pending cells below with sanitized same-hour evidence;
no shadow run, Claude comparison, live installation or parity pass is claimed by the worker.

Window (UTC): **not run**. Released source/binary: **not recorded**. Shadow route/project: **not selected**.

| Category | PilotQ shadow evidence | Claude PM evidence, same hour | Match / gap |
|---|---|---|---|
| Asks | Pending PM run | Pending capture | Unknown |
| Stuck work | Pending PM run | Pending capture | Unknown |
| Failed work | Pending PM run | Pending capture | Unknown |
| Releases | Pending PM run | Pending capture | Unknown |
| Telemetry alerts | Pending PM run | Pending capture | Unknown |

A8 remains open. Record collection failures and tick offsets explicitly; absence of evidence is never a match.
