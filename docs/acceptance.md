# PilotQ acceptance cases

These cases verify [pilotq.md](../pilotq.md), not a separate product contract. Status: proposed, not executed. Owner requirements R1–R8 are defined there; numerical targets are proposed in section 7. Review cases before implementation. Test accounts and separate credentials must be isolated from the production bot and production queue.

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

## Context cases

| ID and requirement | Test | Required evidence |
|---|---|---|
| C01 R4 R7 | Long conversation across at least three compactions | Project corrections, prohibitions, pending questions and task links remain correct |
| C02 R7 | agentq compresses a large tool result containing one important error | Model sees the error and a usable reference; recovered original matches its content hash |
| C03 R7 | Compression fails, times out or would overflow context | Original or bounded recovery path; no silent constraint loss and no infinite retry |
| C04 R1 R7 | New message arrives during manual/automatic compaction | One serialized run; no stale branch or mixed authorship |
| C05 R1 R7 | A private result reference is requested from another route | Access denied without revealing private original data |
| C06 R5 R7 | Native session is damaged after acknowledged direct mutation | Restore discussion and verified receipt without re-executing the mutation |

Run the same scenarios with agentq disabled and enabled. Fix model/provider/settings and runtime versions for each comparison; repeat representative model-dependent cases three times. Compare output quality and real provider usage separately from transport correctness.

## Metrics

All metrics are **new/proposed** for PilotQ. Langfuse's [evaluation guidance](https://langfuse.com/academy/evaluate/choosing-what-to-evaluate) supports choosing outcome-linked measures. Local deterministic evidence is primary; an LLM judge does not decide delivery, authorization or task completion. Review this metric set before implementing evaluators.

| Priority | Metric and proposed gate | Source and measurability | Decision informed |
|---|---|---|---|
| P0 | Zero lost durably accepted inputs | Reconcile inbox records with explicit request outcome and delivery state in all crash cases | Block adoption on loss |
| P0 | Zero unauthorized/cross-route effects or disclosures | Fixed identity/route fixtures and effect receipts | Block adoption on any violation |
| P0 | Zero duplicate task creation or blind replay of uncertain effects | Stable operation IDs plus authoritative board/target readback | Block adoption on any violation |
| P0 | Zero false success claims in fault fixtures | Compare native terminal state, tool readback, task state and user response | Reject runtime/adapter behavior |
| P1 | At least 95% correct simple requests without human nudges | Owner-reviewed outcomes over at least 200 normal simple requests; required domain clarification is not a recovery nudge | Qualify conversational usefulness |
| P1 | 100% of accepted requests remain accounted for; report achieved user outcomes separately | Includes dependency outages; completed, waiting, failed and uncertain counts remain distinct | Prevent good error accounting from masquerading as successful work |
| P1 | Receipt, acknowledgement, cancellation and notification latency meet contract budgets | Timestamp spans with p50/p95, pilot load and dependency availability stated | Set capacity and operating limits |
| P1 | Forced recovery within contract budget; no manual nudge for pure reads | Fault evidence and interventions per 100 requests | Compare candidate runtimes |
| P1 | agentq passes every fixed critical-fact/recovery case and reduces measured input use on the intended workload | C01–C06, paired provider tokens/cache data; missing usage is unknown | Enable adapter only after quality and efficiency evidence |
| P2 | Cost per successful scenario, latency and repeat-work rate | Native billed usage where available, direct provider cost where available, retries and owner interventions | Choose between passing candidates |

## Selection gates

1. **Transport and recovery:** run F01–F14 with a deterministic fake runtime and board fixtures. This isolates coordinator defects without model variability. All safety invariants pass.
2. **Real vertical slice:** run U01–U16 on a separate Telegram bot and safe TaskQ test board with real selected runtime/provider. Include actual worker ask/resume/review; mocks do not qualify this path. Validate latest authorization and task state before each mutation.
3. **Context:** run C01–C06 and paired agentq comparisons. Missing adapter is an implementation prerequisite, not a reason to reject the architecture without investigation.
4. **Pilot:** at least seven days and 500 accepted requests, including at least 200 simple requests, two users, three routes, two projects and ten TaskQ tasks. Deliberate faults remain a separate result set. Record all manual interventions and provider outages. Small samples do not establish long-term uptime.
5. **Production decision:** choose the passing runtime with the lowest maintenance burden and manual intervention rate. Pi is first; compare Codex app-server if Pi fails or needs an upstream fork. Select model/provider separately from the harness. If neither passes economically, evaluate the full OpenClaw gateway against the same cases.

Completion evidence is a report with exact versions, case results, unresolved failures, recovery receipts, task links and aggregate metrics. Runtime research, a healthy service or green repository CI cannot replace it.
