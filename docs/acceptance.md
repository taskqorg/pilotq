# PilotQ acceptance cases

These cases verify [pilotq.md](../pilotq.md), not a separate product contract. Status: proposed, not executed. Owner requirements R1–R17 are defined there; numerical targets are proposed in sections 7 and 12. Review cases before implementation. Test accounts and separate credentials must be isolated from the production bot and production queue. The [mechanism matrix](mechanism-matrix.md) maps 60 scope decisions to these cases and names the source evidence.

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

Completion evidence is a report with exact versions, mechanism-to-case results, unresolved failures, recovery receipts, task links and aggregate metrics. Maintain 62 scenario-family entries: U01–U18, F01–F17, C01–C07 and L01–L20. Record deterministic and live/platform results separately, including not run. Runtime research, a healthy service or green repository CI cannot replace it.
