# Owner requirements audit

Audit of task #191 against `76994fbe28556794a53d4e931845b8b41f51ca03`, including merged #189 and #190. Sources were read through installed TaskQ `issue N --json`; the 827 original clauses come from the #177 inventory. The injected shared revision is `43a2dbe0e4b97bff31fc33a46bb8fc6881d5d80b`; the binary retains its separate specq gitlink pin.

## How to read the evidence

Each inventory sentence/table row and every marked owner note has its own row. The Source column preserves the inventory wording verbatim (HTML escaping only) and links the original task; the source register quotes that task’s text and relevant accepted decisions. A historical inventory sentence is not a new owner decision. Where #1 imports an earlier draft, the inventory is the available exact wording; the earlier owner conversation is not claimed to have been recovered.

`met` means the retained decision is consistent with the cited implementation and the named deterministic risk protection, or is explicitly documentary/proposed as stated. It does not mean a test separately asserts every word. `narrowed` identifies a before-state defect; its row records the fix and retained protection. `missing` never means healthy: it names the absent path, implementation or platform evidence and a read-back held follow-up. A proposal retained as a proposal is distinguished from its unimplemented behavior. Live checks, model quality, deadlines under real load and production cutover are not qualified by local checks (R19; §9).

Code/test references are `file:line` in this candidate. A test reference on a missing row identifies the nearest existing protection and its limit, not a passing test of the missing behavior. Documentary rows retain source history and use the policy/CLI/qualification boundary as their code/test evidence; they assert no executable date, approval or historical deployment.

## Findings and disposition

- **F1 — fixed narrowing:** #82’s repository paths were restricted to fifteen extensions and ASCII names by `tidy`. The initial regression failed on `ui/App.vue`, but review found the first fix incomplete: `src/Makefile` stayed raw and `docs/my file.md` was split. Both initial review cases failed separately before correction. The next review exposed the directory allowlist (`crates/Makefile`) and uppercase guard (`docs/My file.md`); each also failed separately before their removal. All four cases now pass through rich Markdown and HTML alongside unfamiliar extensions, Unicode paths and line references. Written links, URLs, code, versions and abbreviations retain their existing protection.
- **F2 — fixed narrowing:** the loaded discussion policy still prohibited long, non-read-only direct work after #190. The policy regression failed because “Other authorized work is done directly” was absent. The linked spec now follows §5; the guard retains immutable launch/card scope, TaskQ-first queued work, owner answer authority and no replay. Instruction fingerprinting rebuilds changed sessions.
- **F3 — fixed specification drift:** #72 still said “final replies” while §4/#189 requires every user-visible message. Feedback, typing/progress, adoption notes and the installed-command guide now agree with #189/#190. Historical test observations stay historical; obsolete duration/call assertions are not reintroduced.
- **F4 — missing:** synchronous document parsing has no enforceable time/cancel bound. Data/page/output caps do not establish one. The misleading `ponytail:` assurance is removed; [#195](https://github.com/taskqorg/pilotq/issues/195) quotes the owner requirement and retains supported formats.
- **F5 — missing:** scoped Telegram profile correction/forget and access inspection are not exposed by the catalog. Journal methods and the owner CLI do not prove a user self-service flow: [#196](https://github.com/taskqorg/pilotq/issues/196).
- **F6 — missing:** edited-message revisions and Telegram route lifecycle events do not reach durable ingress. Forbidden-send handling is narrower: [#197](https://github.com/taskqorg/pilotq/issues/197).
- **F7 — missing platform evidence:** current live scenario proof belongs to the PM after release, including the next separately requested registered release: [#198](https://github.com/taskqorg/pilotq/issues/198). Historical owner-accepted runs remain in their source tasks and are not silently reused for this candidate.
- **F8 — missing adoption/proposal proof:** storage retention/pressure and restore, migration inventory, native account/workspace binding, startup permissions and paired agentq qualification remain incomplete. [#199](https://github.com/taskqorg/pilotq/issues/199) preserves decision/enablement gates; it does not authorize proposed thresholds or cutover.

The five follow-ups were filed held and read back through TaskQ. The fresh open-board snapshot contained only #191 before filing; no parallel writer was started. These are A3 dispositions, not completed implementations.

## Source task text

### Source #1

[#1](https://github.com/taskqorg/pilotq/issues/1) — Spec: Rust service on coreq, CLI, keys, projects, first slice; TaskQ setup

> Owner decision 2026-10-22: option A. PilotQ is one small Rust service on coreq with teloxide for the Bot API, Hermes gateway patterns as reference and Codex app-server under the subscription login.
>
> - Commits the 2026-10-16 draft (R9-R11, admission, lifecycle protocol, mechanism matrix).
> - Adds R12-R16, the `pilotq` command line, key store, project registration, coreq update and monitoring, the first slice and acceptance cases U19-U22.
> - TaskQ setup: `taskq.json` (`publish: pr`), CI job `tests` with `merge_group`, public repository note.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

### Source #3

[#3](https://github.com/taskqorg/pilotq/issues/3) — Rust skeleton on coreq

> ## Goal
> Create the pilotq Rust binary on coreq per pilotq.md sections 9 and 10 step 2: coreq Tool, service install&#124;run&#124;stop&#124;status, update [--check], selftest, status --json; release workflow publishing linux-x86_64 with SHA256SUMS.
>
> ## Acceptance
> A1. CI job tests is green on the PR and in the merge queue.
> A2. pilotq selftest passes on the built binary.
> A3. pilotq update refuses a release whose selftest fails and keeps the current binary (docs/acceptance.md U22, first half).

### Source #4

[#4](https://github.com/taskqorg/pilotq/issues/4) — Journal, ingress and outbox

> ## Goal
> Implement the SQLite journal (WAL, synchronous=FULL), teloxide long polling whose cursor commits in the same transaction as admitted inputs, an OS poll lock, and an outbox with per-route order and bounded retry, per pilotq.md section 4.
>
> ## Acceptance
> A1. docs/acceptance.md F01, F02 and F05 pass against a test bot.
>
> Depends on #3

### Source #5

[#5](https://github.com/taskqorg/pilotq/issues/5) — People, grants and routes

> ## Goal
> Bootstrap the owner from protected config; add pilotq users list&#124;show&#124;grant&#124;revoke and pilotq routes list&#124;set; ignore unknown senders silently; apply DM, group and forum-topic session policy; keep per-person profiles.
>
> ## Acceptance
> A1. docs/acceptance.md U02, U17 and U18 pass.
>
> Depends on #4

### Source #6

[#6](https://github.com/taskqorg/pilotq/issues/6) — Codex manager and pilotq ask

> ## Goal
> Add one Codex app-server adapter under the subscription login with request and run identity, fencing, first-event and absolute deadlines; pilotq ask and Telegram share one path.
>
> ## Acceptance
> A1. docs/acceptance.md U01, U12, U16, U19 and F03 pass.
>
> Depends on #4
> Depends on #5

### Source #8

[#8](https://github.com/taskqorg/pilotq/issues/8) — Request audit records and CoreQ telemetry

> ## Goal
> Implement pilotq.md R17 and section 8 Request audit: one audit record per CLI or Telegram request, written in the same transaction as the admitted input, joined by request ID to the CoreQ trace and TaskQ task text.
> Add `pilotq audit` with the section 9 filters and reader grants (owner and audit grant see all, a person sees only their own, every read recorded).
>
> ## Acceptance
> A1. docs/acceptance.md U23 passes.
> A2. No key value appears in any audit record, export or CoreQ event.
>
> Depends on #4
> Depends on #5
> Depends on #6

### Source #18

[#18](https://github.com/taskqorg/pilotq/issues/18) — Drive one TaskQ project from PilotQ: plan, held create, start, answer, outcome

> ## Goal
> An authorized person plans work in Telegram; PilotQ files it held on the pilotq board, reads it back, starts it, relays the worker question and answer, and reports the reviewer outcome (pilotq.md section 10 step 3).
>
> ## Acceptance
> A1. pilotq.md sets the TaskQ tool contract before code (R13): board taskqorg/pilotq, grants per tool, create claim with op id, no plain add.
> A2. U05, U06, U15 pass: a plan-only request starts nothing; an authorized revision is created held, read back and started; a network loss after ticket two reconciles the same IDs.
> A3. U07, U08 pass: the worker question reaches the originating route; only the authorized current answer is written; a stale or foreign answer writes nothing and shows why.
> A4. U10, U13, L06 pass: a PilotQ restart or /new keeps the running task; status by issue URL never creates a duplicate; pause and park report distinct transitions.
> A5. F07, F14, L16 pass: an unavailable board leaves the create unsent with a reason; a stale notification is ignored; a missed TaskQ event is recovered by the periodic scan.
> A6. L01, L02, L03, L12 pass: waiting for TaskQ shows its reason and no hung warning; progress lives in one status message edited in place; the final result is never overwritten.
> A7. Deterministic traces with a fake TaskQ run in cargo test --locked; docs/acceptance.md maps every step 3 case to its test or live evidence.
> A8. Live on the test bot: one real pilotq task goes from discussion to reviewer outcome with a PilotQ restart mid-run; U23 task-link checks and the U02/U18 two-user rerun pass.

Accepted source/update [comment 6351452960](https://github.com/taskqorg/pilotq/issues/18#issuecomment-6351452960):

> taskq: answer 1
> Set create_claim_issue and join a live test-bot run; I then run A8 on the PR build

Accepted source/update [comment 6355493919](https://github.com/taskqorg/pilotq/issues/18#issuecomment-6355493919):

> taskq: answer 1
> Allow me to file the claim issue, edit taskq.json and the install config, install the PR build; then you two message the bot

Accepted source/update [comment 6356068324](https://github.com/taskqorg/pilotq/issues/18#issuecomment-6356068324):

> taskq: answer
> Owner answer (2026-10-23): no human live run. Automate A8: real PilotQ binary, pilotq board and TaskQ CLI; Telegram replaced by injected updates from two synthetic users (U02/U18) and a local fake Bot API recording all sends.
> One small real pilotq docs task goes from discussion to reviewer outcome with a PilotQ restart mid-run; check journal, audit, board and recorded messages against expected texts, U23 links included.
> Update pilotq.md and docs/acceptance.md so A8 names this run as evidence. Stop the live test-bot run; restore the pre-step3 build and journal.
> A later owner check on the test bot is optional and not part of this task.

> A6. L01, L02, L03, L12 pass: waiting for TaskQ shows its reason and no hung warning; progress lives in one status message edited in place; the final result is never overwritten.

> A7. Deterministic traces with a fake TaskQ run in cargo test --locked; docs/acceptance.md maps every step 3 case to its test or live evidence.

> A8. Live on the test bot: one real pilotq task goes from discussion to reviewer outcome with a PilotQ restart mid-run; U23 task-link checks and the U02/U18 two-user rerun pass.

### Source #23

[#23](https://github.com/taskqorg/pilotq/issues/23) — Key store, owner board comments and labels, project switching (step 4)

> ## Goal
> Owner decision 2026-10-23 (option B): direct actions are reads; only the owner may add a board comment or label; keys enter only through the CLI.
> Add the key store, these direct actions and project registration with switching (pilotq.md section 10 step 4).
>
> ## Acceptance
> A1. pilotq.md records the decision before code (R13): section 11 closes short-action grants and key entry; reads per grant, comment and label owner only, no other writes; keys CLI only.
> A2. Key store per section 8: pilotq keys list&#124;add&#124;grant&#124;revoke, value from stdin, owner-only file; the value never reaches model context, journal, telemetry, logs, replies or task text; tool output is scrubbed. U23 key-value checks pass.
> A3. U03, U11, L05 pass: a granted read returns real evidence with its source time; a short action at its 60 s or 5-call budget keeps partial evidence and delegates the rest or asks.
> A4. U04, F04 pass: an owner comment or label is written once with a receipt and read back; a crash after the write and before the receipt reconciles and never replays blindly; a non-owner gets no write and the reason.
> A5. U09, F09, F10 pass: a switch uses the named registered project; an ambiguous ticket number is clarified; quoted text or tool output cannot redirect a project or widen a grant; a revoked project grant stops pending effects.
> A6. Deterministic traces run in cargo test --locked; docs/acceptance.md maps every step 4 case to its test or live evidence.
> A7. Scripted live run like a8_live_board_run with real Codex and the real board: a key-backed read, one owner comment and label on a pilotq issue with readback, a refused non-owner write and a project switch.

> A6. Deterministic traces run in cargo test --locked; docs/acceptance.md maps every step 4 case to its test or live evidence.

> A7. Scripted live run like a8_live_board_run with real Codex and the real board: a key-backed read, one owner comment and label on a pilotq issue with readback, a refused non-owner write and a project switch.

### Source #26

[#26](https://github.com/taskqorg/pilotq/issues/26) — Step 5: agentq qualification, full case matrix, seven-day pilot start

> ## Goal
> pilotq.md section 10 step 5 (owner go 2026-10-23): qualify agentq against the native-compaction baseline, record the complete case matrix, then start the pilot on the test bot.
> Gates per docs/acceptance.md Selection gates 3 and 4; the pilot itself runs seven days, so this attempt ends at a started, measurable pilot.
>
> ## Acceptance
> A1. C01-C07 run with agentq off and on at fixed model, provider and runtime versions; model-dependent cases three times; report quality, exact recovery, provider tokens/cache, latency, interventions.
> A2. agentq is enabled for the pilot only if the P1 metric gate passes; otherwise the pilot runs on the native baseline and the result says why.
> A3. docs/acceptance.md records all 62 scenario families (U01-U18, F01-F17, C01-C07, L01-L20) with exact versions; deterministic, live and not-run results kept separate.
> A4. pilotq pilot report (or an equivalent CLI) prints gate 4 counts from journal and telemetry: accepted and simple requests, users, routes, projects, TaskQ tasks, interventions, outages.
> A5. Pilot running on the test bot with two users, three routes and two projects; the result gives the start time and the earliest gate readout date.

Accepted source/update [comment 6358518303](https://github.com/taskqorg/pilotq/issues/26#issuecomment-6358518303):

> taskq: answer 1
> User 728914418, grants chat,taskq, project:pilotq plus the second project

Accepted source/update [comment 6358560070](https://github.com/taskqorg/pilotq/issues/26#issuecomment-6358560070):

> taskq: answer 1
> Run C01-C07 off live; record on as not-run with reason; pilot on native baseline; file agentq adapter task

Accepted source/update [comment 6358562164](https://github.com/taskqorg/pilotq/issues/26#issuecomment-6358562164):

> taskq: steer
> Owner 2026-10-23: option 1. Edit pilotq.md §10 step 5 first (R13): pilot on native baseline; agentq qualification moves after the agentq dynamic-tool adapter.
> File the agentq adapter task with --hold; PM reads it back and starts it. Record agentq-on C01-C07 as not-run with the reason.

Accepted source/update [comment 6361271160](https://github.com/taskqorg/pilotq/issues/26#issuecomment-6361271160):

> taskq: answer 1
> requeue: retry with a fresh attempt

Accepted source/update [comment 6361417797](https://github.com/taskqorg/pilotq/issues/26#issuecomment-6361417797):

> taskq: answer
> Done by PM 09:05Z: fra-install.sh ran; protected config created (600); pilotq.service user unit installed, enabled, active running on FRA.
> Mac has no pilotq poller (plist disabled, no process). Unit logs: journalctl --user -u pilotq with XDG_RUNTIME_DIR=/run/user/$(id -u). Resume A5.

Accepted source/update [comment 6361538252](https://github.com/taskqorg/pilotq/issues/26#issuecomment-6361538252):

> taskq: answer
> Done by PM 09:12Z: pilot Codex home has its own device-auth ChatGPT login (owner, not a copy of the host login); fra-ready.sh ran, exit 0.
> Grants 728914418 chat,taskq,project:pilotq,project:agentq; 3 routes; projects pilotq, agentq; readiness ask answered; pilotq.service active; pilot_start 1792746746. Resume.

> A3. docs/acceptance.md records all 62 scenario families (U01-U18, F01-F17, C01-C07, L01-L20) with exact versions; deterministic, live and not-run results kept separate.

### Source #39

[#39](https://github.com/taskqorg/pilotq/issues/39) — Steer, requeue and every registered board in the TaskQ tools

> ## Goal
> pilotq.md section 5 lists steer and requeue among the write tools; the first tool contract has neither and serves only taskqorg/pilotq. Close that gap.
> Owner 2026-10-24 (PM plan A): first full PilotQ version for TaskQ work; automatic tests first, then a human trial.
>
> ## Acceptance
> A1. Spec first (R13): section 5 First TaskQ tool contract names the new tools, grants and effects and covers every registered project; acceptance.md gets cases.
> A2. task_steer and task_requeue run the TaskQ CLI with argument arrays, check grants and the run fence at dispatch, audit, and reply with the readback.
> A3. A refused call writes nothing and says why.
> A4. Plan, run, status, park, steer, requeue and question relay resolve one registered project per the section 5 Target rules; ambiguous numbers are refused with the list.
> A5. Fake-Codex journal traces cover each new tool, a cross-project refusal and a stale answer on a second board; cargo test --locked and selftest pass.

### Source #42

[#42](https://github.com/taskqorg/pilotq/issues/42) — Human trial of PilotQ with invited testers

> ## Goal
> After the automatic suite passes on FRA, invited people use PilotQ on the test bot for real TaskQ work; their feedback closes the first version.
> Owner 2026-10-24 (PM plan A): first full PilotQ version for TaskQ work; automatic tests first, then a human trial.
>
> ## Acceptance
> A1. Spec first: pilotq.md and docs/acceptance.md describe the trial (people, length, measures) beside the owner 2026-10-23 scripted-run decision.
> A2. The owner approves the trial plan and tester list through an ask before any tester gets access.
> A3. Testers get access by admin grant and the tester guide; each request leaves an audit record.
> A4. A trial report: requests, outcomes, refusals, failures with request IDs and tester feedback; each defect becomes a held task.

Accepted source/update [comment 6399202264](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6399202264):

> taskq: answer 1
> Seven days, owner plus two invitees; real TaskQ work; report request IDs, outcomes/refusals/failures, feedback; file defects held. Gate 4 stays separate.
> op-id: qac-mvnydqrjqwovf2z3fr
> revision: 6392242124

Accepted source/update [comment 6400716819](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6400716819):

> taskq: answer
> Owner 2026-10-26: keep it simple, no new gates. No invitation or private ID step.
> Testers: the two users already on the FRA server, the owner and Simon; they test on the fly with test tasks.
> When the system is ready for testing, end with an ask that says so; the owner then names admin one and admin two.
> A40 proof: PM run 14/14 at 4904339, see pilotq#62.

Accepted source/update [comment 6507868500](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6507868500):

> taskq: answer 1
> requeue: retry with a fresh attempt
> op-id: qac-mvu0fhmnwxpw0dm9ho
> revision: 6507692082

Accepted source/update [comment 6507931415](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6507931415):

> taskq: steer
> PM: codex-code-mode-host now installed on pds18, tools work. Owner 2026-10-29 ends the trial now: A2 approval exists (owner 10-27, admins owner+Simon). Close on current audit and feedback; no waiting window.

Accepted source/update [comment 6507962947](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6507962947):

> taskq: answer
> PM export on pds18 (pilotq 0.2.11, read-only `pilot report --json` and `audit --since 1793114100 --json`), window 2026-10-27T15:15Z to 2026-10-29T20:46Z. Audit rows span 10-28T18:19Z..10-29T19:03Z.
> Report: accepted 98, simple 71, users 1, routes 3, projects 2, tasks 11, interventions 0, outages 0, failed 8. Tools: task_plan 15, task_status 5, project_list 4, issue_read 3, issue_comment 3, task_run_plan 3, project_use 2.
> Outcomes: answered 88, failed 9, uncertain 1. Deliveries: 318 delivered, 51 superseded, 0 undelivered.
> Failures by request ID:
> - 12, 14, 28: unsupported input type (kind other), refusal text sent.
> - 19, 20: Codex 401 Missing bearer on api.openai.com (drill #84), host login gap before pds18 device login.
> - 35: uncertain, Codex did not finish within 60 s on a multi-plan run request; not repeated.
> - 43/45: PM manual end, duplicate answer after answer write landed but end_answer never ran.
> - 60: /bogus, unknown command refusal (expected).
> - 68: voice recognition not configured.
> - 97: stale question answer refused.
> Feedback: only the owner (Alex K) used the bot; Simon sent no requests, so no tester feedback from Simon. Owner feedback is the tasks he filed through the bot (#81, #82, #89-#93, #106, #108). Trial ended by owner 2026-10-29 on this data.
> Write the A4 report from this; file held tasks for defects (60 s uncertain timeout, end_answer gap, voice off) if not already tracked.

Accepted source/update [comment 6508172165](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6508172165):

> taskq: answer 1
> Accept bounded report - + ends trial now; - A3 grant/audit proof and A4 success IDs remain unverified; explicitly waive those proof items only
> op-id: qac-mvu0z9viknofphofa9
> revision: 6508041507

### Source #68

[#68](https://github.com/taskqorg/pilotq/issues/68) — Make Telegram replies clear: no ack spam, streaming, rich format, project list

> ## Goal
> Owner tried the bot: each message gets a separate 'Request #N is queued' reply, answers re-quote it, long answers are plain 4096-char text, and 'which projects do you see' fails (no project list tool).
> Make the chat feel like a normal assistant (reaction ack, live draft streaming, rich final message) and keep the durable outbox guarantees; spec first in pilotq.md.
>
> ## Acceptance
> A1. pilotq.md is updated first (section 4 Delivery, 5 direct tools, 7 ack budget, 12 Telegram feedback) to these decisions; the code matches it
> A2. Accepted text input gets an eyes reaction within 5 s, no text ack; text ack only if the reaction fails or work waits over 30 s ('queued behind N'); busy/unsupported/control stay text
> A3. Request numbers leave normal replies; they stay in /status, /audit, failure and stop texts; the first-use retention notice still shows once
> A4. Private chats stream the answer as a draft (sendRichMessageDraft, else sendMessageDraft), max 1 update/s, draft errors never fail; final answer still goes through the outbox
> A5. Final answer uses sendRichMessage (up to 32768 chars) when accepted; else HTML sendMessage split at 4096 in stable ordered chunks; else plain text; output escaped safely
> A6. Private chats: the answer does not quote the user's latest message; groups and topics keep reply_to, typing and the edited status message
> A7. New read tool project_list returns the registered projects the person may access, current one marked; 'which projects do you see' answers from it
> A8. Tests cover reaction fail, draft fail, rich rejected and entity parse error; A40 live suite updated and passing on FRA; release on FRA, owner DM shows the new flow

Accepted source/update [comment 6477224354](https://github.com/taskqorg/pilotq/issues/68#issuecomment-6477224354):

> taskq: answer
> A: owner asked for this flow live; add 0.2.3 to section 9 in the same PR, release through merge-queue/live.
> PM updates FRA pilot and runs A40 after merge; owner checks the DM flow. Proceed with spec, code and tests now.

> A1. pilotq.md is updated first (section 4 Delivery, 5 direct tools, 7 ack budget, 12 Telegram feedback) to these decisions; the code matches it

> A8. Tests cover reaction fail, draft fail, rich rejected and entity parse error; A40 live suite updated and passing on FRA; release on FRA, owner DM shows the new flow

### Source #72

[#72](https://github.com/taskqorg/pilotq/issues/72) — Telegram: clean rich layout, accept screenshots, clear action status

> ## Goal
> Owner checked 0.2.3 on FRA: rich replies lose all line breaks (lists render as one paragraph), a screenshot with caption got 'Request #12: this input type is not supported yet'.
> Owner decision 2026-10-28: images with captions must work fluently in Telegram; replies must be crisp, minimal and clear about what was done. Spec first: close the image part of the §11 media decision.
>
> ## Acceptance
> A1. Rich replies keep paragraphs, lists, headings and bold exactly as the model wrote them (sent HTML newlines collapse today; use rich Markdown mode or block tags); fixture covers a plan reply with a list.
> A2. Photos and image documents, with or without caption, reach the model as images plus caption text for the same session; unsupported types (voice, other files) get one short plain-language line.
> A3. No user-facing text shows internal request numbers ('Request #12:'); failure and limit lines are one short sentence in the user's language.
> A4. Action replies state the effect in one line first: saved locally only, created on the board (task link), started, or nothing done; then the next step. Plan reply stays short.
> A5. pilotq.md updated first: image input decided by the owner (§11 and §1 line on media), rich layout and status rule in §12; tester guide notes the change.
> A6. Version 0.2.4; tests and fixtures green; live check stays the PM FRA update plus A40 after merge, then the owner checks the chat.

> A6. Version 0.2.4; tests and fixtures green; live check stays the PM FRA update plus A40 after merge, then the owner checks the chat.

### Source #77

[#77](https://github.com/taskqorg/pilotq/issues/77) — Notify the owner when an update rolls back

> ## Goal
> Make a failed PilotQ update and launcher rollback visible to the owner without reading service logs (R11, #76 A5).
>
> ## Acceptance
> A1. A failed candidate startup followed by rollback produces one owner-visible incident naming failed and restored versions, with no credentials or request content.
> A2. A deterministic lifecycle check proves the notification survives restart and does not repeat indefinitely; successful updates produce no failure notice.

Accepted source/update [comment 6479734720](https://github.com/taskqorg/pilotq/issues/77#issuecomment-6479734720):

> taskq: steer
> Keep it simple: the restored version sends one Telegram message to the owner chat on its first healthy start after a rollback (failed and restored versions), marked sent in the journal. No new infrastructure.

Accepted source/update [comment 6479788661](https://github.com/taskqorg/pilotq/issues/77#issuecomment-6479788661):

> taskq: answer 1
> A: Owner Telegram DM from protected owner_id; durable outbox deduplicated by failed/restored versions, existing bounded retries.

### Source #81

[#81](https://github.com/taskqorg/pilotq/issues/81) — Исследовать отображение состояния PilotQ и уведомления

> ## Goal
> Предложить понятный способ показывать состояние сервиса и уведомлять о перезапусках, обновлениях, сбоях и восстановлении при множестве досок и чатов.
>
> ## Acceptance
> A1. Сравнены варианты отображения состояния и маршрутизации уведомлений по доскам и чатам.
> A2. Предложены правила против шума и дублей и способ обнаружить недоступность самого PilotQ.
> A3. Даны примеры сообщений и рекомендован минимальный вариант с этапами внедрения.

Accepted source/update [comment 6480230211](https://github.com/taskqorg/pilotq/issues/81#issuecomment-6480230211):

> taskq: answer
> 1. A: acceptance A3 asks for the minimal variant; owner prefers standard simple setups.
> Build on #77 (rollback DM to owner); keep B and C as later stages in the doc.

### Source #82

[#82](https://github.com/taskqorg/pilotq/issues/82) — Сделать форматирование ответов PilotQ понятным и компактным

> ## Goal
> Обеспечить чёткое, минималистичное и наглядное оформление ответов в чате.
>
> ## Acceptance
> A1. Проверен текущий путь формирования и отображения сообщений и определены причины потери форматирования.
> A2. Исправлено оформление абзацев, списков, ссылок и выделений с учётом возможностей чата.
> A3. На примерах плана, статуса и ошибки подтверждено, что сообщения читаются без слипания текста и лишнего оформления.

Accepted source/update [comment 6480258198](https://github.com/taskqorg/pilotq/issues/82#issuecomment-6480258198):

> taskq: steer
> Owner review of the #81 ask relay in Telegram: chaotic. Ask options must be inline keyboard buttons (recommended marked); a press answers; drop the 'To answer: taskq answer N.K' line.
> Every link clickable with a readable label: no raw [link](url), no bare long URLs; at most one link preview per message, or none.
> Repo paths like docs/status-notifications.md get auto-linked by Telegram as a .md domain: render them as GitHub links at the SHA, or as code.
> Prove with Telegram renders of an ask with options, a plan, a status and an error; every link opens the right page.

### Source #89

[#89](https://github.com/taskqorg/pilotq/issues/89) — Исследовать бесплатные модели распознавания голосовых сообщений Telegram

> ## Goal
> Выбрать бесплатную модель для качественного распознавания голосовых сообщений PilotQ, прежде всего на русском языке, с учётом доступного сервера.
>
> ## Acceptance
> A1. Проверены ресурсы доступного сервера: CPU, RAM, GPU и возможность локального запуска.
> A2. Parakeet сравнён с подходящими альтернативами по поддержке русского языка, качеству, лицензии, памяти и скорости на доступном оборудовании.
> A3. На репрезентативных голосовых записях оценены качество и задержка; выбран вариант и описан способ интеграции.

Accepted source/update [comment 6488061328](https://github.com/taskqorg/pilotq/issues/89#issuecomment-6488061328):

> taskq: answer
> 1. GigaAM v3 RNNT int8. Non-Russian notes get one short line for now; Parakeet fallback only if real use shows the need. Timing on the FRA CPU goes into the acceptance evidence.

### Source #91

[#91](https://github.com/taskqorg/pilotq/issues/91) — Восстановить наглядность обработки и компактное оформление PilotQ

> ## Goal
> Пользователь видит обработку сообщения и получает короткие понятные статусы без повторов.
>
> ## Acceptance
> A1. Исправлен Telegram typing: работает в течение обработки, включая длительные вызовы, и завершается при ответе, ошибке или отмене.
> A2. Создание, очередь и выполнение имеют точные разные статусы.
> A3. Ссылки содержат обозначение площадки, номер и короткое название; убраны служебные повторы и дубли уведомлений.
> A4. Правила закреплены в спеке и проверены в Telegram.

Accepted source/update [comment 6486780232](https://github.com/taskqorg/pilotq/issues/91#issuecomment-6486780232):

> taskq: answer
> Option 1: the owner's plan is newer than §12; update §12 in this PR. Typing until the active call deadline; links GitHub · pilotq#91 · title. PM updates the test bot after merge; owner checks A4.

### Source #92

[#92](https://github.com/taskqorg/pilotq/issues/92) — Обеспечить общий контекст связанных сообщений Telegram

> ## Goal
> Картинка и последующий текст, в том числе пришедший во время обработки, дополняют один запрос без потери контекста.
>
> ## Acceptance
> A1. Проверены и исправлены передача изображений, подписей, reply и последовательных сообщений в контекст модели.
> A2. Связанные дополнения обновляют текущий замысел без дублирования задач; независимые запросы различаются.
> A3. Проверены картинка с последующим длинным текстом, несколько изображений и текст во время активной обработки.
> A4. Правила закреплены в спеке; изображение само по себе не считается авторизацией действий.

Accepted source/update [comment 6486904526](https://github.com/taskqorg/pilotq/issues/92#issuecomment-6486904526):

> taskq: answer
> Owner: none of A/B/C. No time windows, no user duties. The agent links messages by context: clear ones steer the matching task. If unclear, ask: 'to task X or new?' with options plus free-form answer.

### Source #93

[#93](https://github.com/taskqorg/pilotq/issues/93) — Переработать UX обсуждения плана и добавить кнопки действий

> ## Goal
> Обсуждение развивает один понятный черновик плана; пользователь запускает согласованную версию кнопкой без необходимости печатать служебную команду.
>
> ## Acceptance
> A1. Исследован текущий сценарий и в спеке описаны состояния обсуждения, черновика, запуска и последующих изменений с конкретными примерами.
> A2. Дополнения обновляют один пользовательский план с устойчивой идентичностью; внутренние ревизии не подаются как новые планы, новый самостоятельный замысел отделяется осмысленно.
> A3. Реализованы Telegram-кнопки запуска плана и выбора ответа там, где есть конкретные варианты; текстовые команды сохранены.
> A4. Кнопка привязана к показанной версии, проекту и авторизованному пользователю; устаревшие и повторные нажатия не запускают изменённый план и не создают дубли.
> A5. После изменения черновика актуализируется сообщение с кнопками либо явно показывается актуальный вариант; обсуждение само по себе ничего не запускает.
> A6. Проверен весь сценарий из текущего примера: последовательные дополнения, один черновик, запуск кнопкой, повторное и устаревшее нажатие.

Accepted source/update [comment 6486786054](https://github.com/taskqorg/pilotq/issues/93#issuecomment-6486786054):

> taskq: answer 1
> Один черновик до запуска; новый — по явной просьбе. После запуска изменения обсуждаются отдельно. Выбор отвечает, запуск — отдельной кнопкой.

### Source #94

[#94](https://github.com/taskqorg/pilotq/issues/94) — Telegram: a question answered elsewhere closes in place

> ## Goal
> When a TaskQ question is answered anywhere (tray, DashQ, CLI, PM), PilotQ edits its question message in place, so nobody answers a closed question.
>
> ## Acceptance
> A1. Within one poll after the answer event the message loses its buttons and gains 'Answered by &lt;actor&gt;: &lt;text&gt;'; an old button tap shows that text as a toast and writes nothing.
> A2. Follows the taskq.md rule for late answers (taskq#1785); a test covers answer-elsewhere and the stale tap.

Accepted source/update [comment 6486906697](https://github.com/taskqorg/pilotq/issues/94#issuecomment-6486906697):

> taskq: answer
> Option 1: owner wants answered questions closed in place; edit §5 accordingly. Use taskq#1785's snapshot fields once defined; until then read the answer entries from the board.

### Source #95

[#95](https://github.com/taskqorg/pilotq/issues/95) — PilotQ follows specq rules for replies, tasks and questions, proved by tests

> ## Goal
> PilotQ's Codex instructions come from the pinned specq rules (replies, icons, task intake, decision questions) instead of one hand-written string, and tests prove the bot follows them.
>
> ## Acceptance
> A1. Instructions load the pinned specq caveman and agent-workflow rules plus pilotq.md additions; a specq pin bump changes them without a code edit; the hand-written string keeps only PilotQ specifics.
> A2. Scripted cases (plan, task creation, relayed ask, status) checked: language, length, no bare URL or internal id, task goal ≤2 lines, ask options with plus, minus and reasoned recommendation.
> A3. Relayed TaskQ asks reach Telegram in the specq decision-question form; the case runs in CI with a fake model and once live on FRA.
>
> Depends on specq#90

Accepted source/update [comment 6487074488](https://github.com/taskqorg/pilotq/issues/95#issuecomment-6487074488):

> taskq: answer
> 1: relay unchanged, test A3 with complete cards. Completeness is enforced at the source: taskq#1787 makes taskq ask reject options without label, plus and minus. Do not invent reasons in PilotQ.

> A3. Relayed TaskQ asks reach Telegram in the specq decision-question form; the case runs in CI with a fake model and once live on FRA.

### Source #97

[#97](https://github.com/taskqorg/pilotq/issues/97) — Telegram ask message follows the TaskQ ask layout

> ## Goal
> A relayed ask in Telegram is short and structured: task title link, icon sections, short option buttons, footer in the chat language; nothing is said twice.
>
> ## Acceptance
> A1. src/taskq.rs ask message uses the taskq#1787 render: header with task title link, Done and Problem lines with icons, numbered options with label and detail, one footer line in the chat language.
> A2. Buttons show N and the option label only; no truncated detail text; recommended option marked.
> A3. Tests: Russian and English ask fixtures render to the expected Telegram text and buttons; one live relay on FRA shows the screenshot in the result.
>
>
> Depends on taskq#1787

Accepted source/update [comment 6487859617](https://github.com/taskqorg/pilotq/issues/97#issuecomment-6487859617):

> taskq: answer
> 1. Follow taskq#1787 answer-card.md without footer; it overrides the A1 footer wording (owner contract: one question contract on all surfaces). Options '&lt;label&gt; - + &lt;gain&gt;; - &lt;cost&gt;', visual choice captions images by number.

Accepted source/update [comment 6487992011](https://github.com/taskqorg/pilotq/issues/97#issuecomment-6487992011):

> taskq: answer
> Neither gate: review the code now. A3 becomes a post-merge PM check: PM releases, sends one live ask through FRA and attaches the Telegram screenshot to the release note. No owner check.

> A3. Tests: Russian and English ask fixtures render to the expected Telegram text and buttons; one live relay on FRA shows the screenshot in the result.

### Source #104

[#104](https://github.com/taskqorg/pilotq/issues/104) — Recognize Russian Telegram voice messages with GigaAM v3

> ## Goal
> Owner decision on #89 (option 1): voice notes and audio files become text input through local GigaAM v3 RNNT int8, so people can talk to PilotQ instead of typing.
>
> ## Acceptance
> A1. pilotq.md first: §1 accepts voice/audio as recognized input; §11 records the #89 decision; non-Russian notes get one short line; Parakeet fallback only if real use shows the need.
> A2. Voice, audio and audio documents use the image path (§12): admission, grants, 20 MiB cap, journal file identity and transcript, audit; ffmpeg decode to 16 kHz; failures are explicit outcomes.
> A3. Recognition runs locally on CPU via sherpa-onnx with GigaAM v3 RNNT-punct int8; the transcript reaches Codex as the sender's text marked as a voice transcript; no download URL or token reaches Codex.
> A4. Tests cover OGG/Opus voice, an audio file, undecodable and over-cap input; FRA CPU timing (RTF, p50/p95, peak RSS) on Russian samples is recorded as acceptance evidence.

> A4. Tests cover OGG/Opus voice, an audio file, undecodable and over-cap input; FRA CPU timing (RTF, p50/p95, peak RSS) on Russian samples is recorded as acceptance evidence.

### Source #106

[#106](https://github.com/taskqorg/pilotq/issues/106) — Добавить встроенные команды и управление моделями PilotQ и TaskQ

> ## Goal
> Управлять командами Telegram и выбором моделей менеджера и воркеров по явной политике.
>
> ## Acceptance
> A1. Проверены встроенные команды и skills; описана поддержка /version, /model, /doctor, /help, /restart, списка и вызова skills с точным синтаксисом.
> A2. Команды вызывают встроенные обработчики; версия, модель и диагностика отражают фактическое состояние; справка перечисляет доступные действия.
> A3. /model показывает доступные модели и выбор кнопкой; определены область настройки, сохранение выбора и момент применения.
> A4. Изучен запуск менеджера и воркеров; предложена политика моделей: значения по умолчанию, наследование, переопределение, ресурсы и недоступность. Материальные решения вынесены владельцу до реализации.
> A5. Согласованная политика реализована в PilotQ и интеграции TaskQ; выбор модели соблюдает права и не меняет запущенные задачи молча.
> A6. /doctor выполняет ограниченную диагностику без секретов; /restart учитывает права, активные запросы и проверку результата.
> A7. Правила и примеры закреплены в спеке; в Telegram проверены команды, выбор моделей, skills, ошибки, права и перезапуск с активным запросом.

Accepted source/update [comment 6488607801](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6488607801):

> taskq: steer
> Evidence: FRA requests 37-41 on 2026-10-29 sent /version /model /doctor /help /restart to Codex (answers: no tool); /restart waited 40 min. Replay them as tests; commands never queue behind a chat run.

Accepted source/update [comment 6488642907](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6488642907):

> taskq: answer 1
> По диалогу - + Изолированный выбор менеджера; - воркеры остаются на политике TaskQ.
> op-id: pilotq-3cb9674929ca8680-a48
> revision: 6488615593

Accepted source/update [comment 6488958287](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6488958287):

> taskq: answer
> Owner (Telegram): install c63069f on the live PilotQ service and test A7 live with the real Telegram bot.
> No separate test install; dev mode, everything goes live right away. §12 candidate rule does not block this.

Accepted source/update [comment 6489840359](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6489840359):

> taskq: answer
> PM installed fd2a983 (c63069f + #115 answer fix) live on FRA at 06:45Z: service install, user restart, running exe sha equals build, selftest ok. Continue A7 live verification on the live service; no install needed. Rebase PR #114 on main.

Accepted source/update [comment 6490429093](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6490429093):

> taskq: answer
> Owner ran the A7 Telegram checklist on FRA (version, help, doctor, skills, bogus, model switch and default, refusal, restart mid-answer): commands work. A7 accepted; continue.

Accepted source/update [comment 6491877459](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6491877459):

> taskq: answer
> close: PR #114 merged at f501f99 after the PM rebase (4958c4d); released as 0.2.10 and live on FRA.

Accepted source/update [comment 6492223249](https://github.com/taskqorg/pilotq/issues/106#issuecomment-6492223249):

> taskq: answer 3
> close: superseded by merged #114 at 4958c4d; no merge

### Source #108

[#108](https://github.com/taskqorg/pilotq/issues/108) — Сделать фиксацию ответов на вопросы понятной и надёжной

> ## Goal
> Пользователь видит, какой ответ сохранён, к какому вопросу он относится и что произошло дальше.
>
> ## Acceptance
> A1. Исследован путь от кнопки или текстового ответа до сохранения и передачи воркеру; установлено, где ответ теряется либо отсутствует видимое подтверждение.
> A2. После подтверждённого сохранения карточка показывает выбранный ответ и статус; получение нажатия не выдаётся за сохранение или возобновление работы.
> A3. Повторные нажатия не дублируют действия; устаревшие кнопки и изменённые вопросы дают понятный результат без применения к другому вопросу.
> A4. При ошибке или неопределённом результате видны проблема и доступное следующее действие; перезапуск и повторная доставка не теряют сохранённый ответ.
> A5. Проверены ответы кнопкой и текстом, несколько открытых вопросов, задержка, ошибка, повтор и перезапуск; связь определяется смыслом и адресацией, а не временем.
> A6. В Telegram показан полный сценарий: вопрос, ответ, сохранённый выбор и следующий фактический статус; оформление компактное, понятное и на языке пользователя.
> A7. Правила закреплены в спеке; изучены результаты #93 и проверены открытые задачи, чтобы исправление не дублировало текущую работу.

Accepted source/update [comment 6488655806](https://github.com/taskqorg/pilotq/issues/108#issuecomment-6488655806):

> taskq: answer 2
> Добавить смысловую адресацию - + Свободный текст выбирает вопрос после уточнения; - нужен новый инструмент и изменение §5. A6 после выпуска PM.
> op-id: pilotq-3cb9674929ca8680-a49
> revision: 6488629506

### Source #110

[#110](https://github.com/taskqorg/pilotq/issues/110) — One plan draft per chat, launched by button with true task states

> ## Goal
> Revising a plan edits one draft with Launch and Cancel buttons; launch reports each task's real board state.
>
> ## Acceptance
> A1. Follow-up messages revise the current draft in place; no new numbered plan per message (FRA requests 29-34 made plans 3, 4, 5, 6 for one ask); older drafts close.
> A2. The Launch button starts that draft; a text 'launch' maps to the current draft only.
> A3. Launch never ends 'Codex did not finish within 60 s' (FRA request 35): filing and starting run in the service, and the reply lists each task with held, ready or doing.
> A4. Replies never claim a task is in work when it is only queued or held (FRA request 31); tests replay requests 29-35.

> A1. Follow-up messages revise the current draft in place; no new numbered plan per message (FRA requests 29-34 made plans 3, 4, 5, 6 for one ask); older drafts close.

> A3. Launch never ends 'Codex did not finish within 60 s' (FRA request 35): filing and starting run in the service, and the reply lists each task with held, ready or doing.

> A4. Replies never claim a task is in work when it is only queued or held (FRA request 31); tests replay requests 29-35.

### Source #117

[#117](https://github.com/taskqorg/pilotq/issues/117) — Установить обсуждаемые изменения PilotQ на live и проверить их

> ## Goal
> Обновить действующий PilotQ готовыми изменениями по командам и ответам и проверить пользовательские сценарии на live.
>
> ## Acceptance
> A1. Проверены результаты #106 и #108, уточнения владельца и текущая версия live; определено, какие изменения готовы и какие требования ещё не реализованы.
> A2. Недостающие требования переданы в соответствующую работу без параллельных писателей; незавершённое не объявляется установленным.
> A3. Готовые изменения собраны и установлены на действующий live-сервис PilotQ; отдельные тестовые установки и боты не создаются.
> A4. После выпуска проверены здоровье сервиса, версия и обработка Telegram; сохранённые запросы и ответы не теряются при перезапуске.
> A5. На live проверены встроенные команды и выбор модели, а также сохранение ответа с автором, удаление кнопок и блокировка повторного выбора.
> A6. Проверена заявленная возможная путаница вариантов 1 и 2 по #108; фактическая запись отделена от предположений.
> A7. Результат содержит установленную версию, доказательства live-проверки и оставшиеся ограничения; правила live-выпуска закреплены в спеке.

Accepted source/update [comment 6490140502](https://github.com/taskqorg/pilotq/issues/117#issuecomment-6490140502):

> taskq: answer
> PM host session: installed PR #114 head a141ab2 (has #115) on FRA live; active, exe sha = build sha, selftest ok, journal states same across restart. A5 user Telegram checks: one owner run with #106 A7. Record A3/A4, no reinstall.

Accepted source/update [comment 6490481807](https://github.com/taskqorg/pilotq/issues/117#issuecomment-6490481807):

> taskq: answer
> Done: owner ran the combined live run with #106 A7 on FRA 2026-10-29 ~07:06-07:12Z; commands mostly work. Use FRA journal requests 63-70 as evidence (model button pick, /new, restart). Note gaps you find; continue.

Accepted source/update [comment 6490651208](https://github.com/taskqorg/pilotq/issues/117#issuecomment-6490651208):

> taskq: answer
> Owner: check everything you can check automatically.
> If all is fine, continue the work.
> If something is not fine, record it as a question in the work; the owner answers later.

Accepted source/update [comment 6490731447](https://github.com/taskqorg/pilotq/issues/117#issuecomment-6490731447):

> taskq: answer
> давай примем задачу
> op-id: qac-mvt84cd2ghpe2fwldb
> revision: 6490686196

> A1. Проверены результаты #106 и #108, уточнения владельца и текущая версия live; определено, какие изменения готовы и какие требования ещё не реализованы.

> A3. Готовые изменения собраны и установлены на действующий live-сервис PilotQ; отдельные тестовые установки и боты не создаются.

> A5. На live проверены встроенные команды и выбор модели, а также сохранение ответа с автором, удаление кнопок и блокировка повторного выбора.

> A7. Результат содержит установленную версию, доказательства live-проверки и оставшиеся ограничения; правила live-выпуска закреплены в спеке.

### Source #119

[#119](https://github.com/taskqorg/pilotq/issues/119) — Status line after /new and on replies, Hermes format

> ## Goal
> After /new PilotQ shows the chosen model briefly; a compact Unicode status line (model, context used, Codex account quota and reset) in the Hermes spec format is shown in the most convenient Telegram place.
>
> ## Acceptance
> A1. Stage 1 research: read how Hermes builds its status line (config, script) and the specq format; pick minimal useful data; compare placements: footer on each reply, one edited pinned message, topic title, newer Telegram means.
> A2. Stage 1 picks a placement for DMs, groups and topics with reasons and Telegram limits (edit rate, pin rights, topic rename rights); the reviewer cross-checks before code.
> A3. Stage 2: /new answers with the chosen model in one short line; the status line uses the Hermes Unicode format and thresholds; unknown or stale data is marked, never shown as zero; no account secrets appear.
> A4. Stage 3 evidence on FRA: /new, a model switch, normal replies in a DM and a topic, a missing quota source; Telegram screenshots plus tests; respects #106 commands and #91 compact style.

Accepted source/update [comment 6490594957](https://github.com/taskqorg/pilotq/issues/119#issuecomment-6490594957):

> taskq: answer 1
> Reply footer - + beside each answer in DM/group/topic, no admin rights; - repeats on every final reply
> op-id: qac-mvt7u66s04m0zy0zyush
> revision: 6490537062

Accepted source/update [comment 6492310982](https://github.com/taskqorg/pilotq/issues/119#issuecomment-6492310982):

> taskq: answer
> Option 1. Owner 2026-10-29 on #119: workers get every right their task needs on FRA and in pilotq. Extend section 9 development-live grant to #119; scoped FRA install is approved.
> For the DM/topic run and Telegram screenshots, raise one ask naming the exact build and steps; the PM runs it and attaches the screenshots.

Accepted source/update [comment 6493228042](https://github.com/taskqorg/pilotq/issues/119#issuecomment-6493228042):

> taskq: answer
> Done: A4 live on FRA with exact de82e17 build: DM/topic /new, /model, missing quota all pass; release 0.2.10 restored. Evidence: https://github.com/taskqorg/pilotq/pull/122#issuecomment-6493225953

Accepted source/update [comment 6493460304](https://github.com/taskqorg/pilotq/issues/119#issuecomment-6493460304):

> taskq: answer
> Genuine DM: journal requests 72-76,79 have chat=sender=134142744; bot DM uses private topics (topic 1067093, reply_to 37). The quote is PilotQ treating a private topic as a group, against pilotq.md: fix here, PM reruns.

Accepted source/update [comment 6493752300](https://github.com/taskqorg/pilotq/issues/119#issuecomment-6493752300):

> taskq: answer
> 6bb2e55 built (sha 74d30951), live DM topic: /new, /model, reply, missing quota ? unavailable, all reply_to NULL, no quote. Release 0.2.10 restored, updates resumed, continuity ok. PR #122 comment 6493750343.

> A4. Stage 3 evidence on FRA: /new, a model switch, normal replies in a DM and a topic, a missing quota source; Telegram screenshots plus tests; respects #106 commands and #91 compact style.

### Source #120

[#120](https://github.com/taskqorg/pilotq/issues/120) — Voice recognition live on FRA

> ## Goal
> Voice notes on FRA fail with 'voice recognition is not configured': #104 shipped code but the host has no voice config, recognizer, GigaAM model or ffmpeg on PATH. Make voice work live and stop this gap recurring.
>
> ## Acceptance
> A1. FRA has the sherpa-onnx-offline v2.0.1 static binary, the GigaAM v3 RNNT-punct int8 model and ffmpeg per docs/tester-guide.md; config.json voice names them by absolute path; the service restarts cleanly.
> A2. pilotq doctor reports voice readiness: missing config, missing binary or model, or a failed one-second probe recognition each fail the check with a short reason.
> A3. Evidence: a real Russian voice note in Telegram on FRA gets a transcript-based answer; journal shows the transcript event; a failure case still gives the short line.

Accepted source/update [comment 6490611812](https://github.com/taskqorg/pilotq/issues/120#issuecomment-6490611812):

> taskq: answer
> Ты попроси ты сделай заметку, что Project менеджер должен запустить эту задачу с такими правами, чтобы задача могла быть выполнена. Средства у воркера должны быть права, чтобы что он там должен настраивать.
> op-id: qac-mvt7vg1m8nv0yh6vlst
> revision: 6490543254

Accepted source/update [comment 6490797239](https://github.com/taskqorg/pilotq/issues/120#issuecomment-6490797239):

> taskq: answer
> Granted per owner: codex workers on FRA pilotq now run full (no sandbox), policy rev 6; your next attempt has it. Install ffmpeg, sherpa-onnx and the GigaAM model, set voice config, restart the test service, check Telegram and journal.
> Keep §9: live release only through CI and the live tag. Never print secrets. Continue A1-A3.

Accepted source/update [comment 6491110267](https://github.com/taskqorg/pilotq/issues/120#issuecomment-6491110267):

> taskq: answer
> Я отослал только сообщение, и пилот кью его успешно распознал.
> op-id: qac-mvt8w4i7njjfx4m2io
> revision: 6491000536

> A1. FRA has the sherpa-onnx-offline v2.0.1 static binary, the GigaAM v3 RNNT-punct int8 model and ffmpeg per docs/tester-guide.md; config.json voice names them by absolute path; the service restarts cleanly.

> A3. Evidence: a real Russian voice note in Telegram on FRA gets a transcript-based answer; journal shows the transcript event; a failure case still gives the short line.

### Source #124

[#124](https://github.com/taskqorg/pilotq/issues/124) — Project from conversation context, board status and footer spacing

> ## Goal
> PilotQ takes the project from the conversation when it is clear, asks only when it is not, shows a project's task status on request, and puts one empty line before the status footer.
> Owner (Telegram, 2026-10-29): "you must take it from context; I named the project. If it is clear, act; if not, ask."
>
> ## Acceptance
> A1. Project from context: plan, run, status, steer, requeue, park and project_use accept a project the person named in an earlier own message of this conversation generation when exactly one accessible project fits.
> A2. Example that must pass: "What is the task status for Pilot Q?" then "Make a task so this works" files the draft on pilotq without asking to repeat the project name.
> A3. Ambiguity asks: when two or more accessible projects fit, the reply asks one short question listing them; nothing is written to a guessed board.
> A4. Boundaries stay: quoted messages, issue text, tool output and other people's messages never select a project; grants, registration and board/root are rechecked at dispatch.
> A5. Board status: a read-only tool returns one accessible project's open tasks (task, state, runtime, session, as TaskQ status shows) with the read time; a status question answers from it, not by asking for numbers.
> A6. Footer spacing: every reply with the status footer has exactly one empty line between the text and the footer in DMs, groups and topics, on rich, HTML and plain delivery.
> A7. pilotq.md records these as an owner decision of this task: the Target rule, the manager tool table and the footer paragraph (R13).
> A8. Tests cover earlier-named project, two fits, quoted name refused, board status read and each footer delivery path. A live FRA check after release shows context, status and spacing in the owner DM topic.

Accepted source/update [comment 6495176090](https://github.com/taskqorg/pilotq/issues/124#issuecomment-6495176090):

> taskq: answer
> 1. Accept code, defer A8 live proof. Merge-group CI gates publication; PM releases 0.2.12 after merge and runs the FRA DM check (context project, board status, blank line before footer), result noted on #124.

> A6. Footer spacing: every reply with the status footer has exactly one empty line between the text and the footer in DMs, groups and topics, on rich, HTML and plain delivery.

> A8. Tests cover earlier-named project, two fits, quoted name refused, board status read and each footer delivery path. A live FRA check after release shows context, status and spacing in the owner DM topic.

### Source #126

[#126](https://github.com/taskqorg/pilotq/issues/126) — Добавить универсальное выполнение установленных команд из чата

> ## Goal
> Пользователь просит выполнить установленную команду обычным текстом или вводит её напрямую и получает фактический результат через общий механизм PilotQ.
>
> ## Acceptance
> A1. В разрешённом окружении taskq list выполняется как установленная команда; ответ содержит её фактический результат без необходимости отдельного инструмента списка.
> A2. Через тот же механизм выполняется другая установленная команда чтения, например pwd, без добавления специального обработчика для неё.
> A3. Поддерживаются как прямой ввод команды, так и однозначная просьба обычным языком; неизвестная или неоднозначная команда приводит к понятному сообщению или уточнению.
> A4. Вывод, отсутствие команды, ненулевой код завершения и тайм-аут различаются; успех сообщается только по результату выполнения.
> A5. Права, рабочий каталог, ограничения времени и объёма вывода применяются на стороне выполнения; запрос не расширяет полномочия сам по себе.
> A6. Проверки покрывают общий путь выполнения, ошибки и соблюдение полномочий; пользовательский сценарий подтверждён через PilotQ.

Accepted source/update [comment 6496818913](https://github.com/taskqorg/pilotq/issues/126#issuecomment-6496818913):

> taskq: answer 1
> Исполнитель по списку - + taskq list и pwd через один путь; - защищённый список команд и аргументов настраивает оператор

Accepted source/update [comment 6497254373](https://github.com/taskqorg/pilotq/issues/126#issuecomment-6497254373):

> taskq: answer
> Option 1. My error: TaskQ has no list command. Replace 'taskq list' with 'taskq status' in the default read-only allowlist and contract; prove A1 with a successful 'taskq status' run.

### Source #130

[#130](https://github.com/taskqorg/pilotq/issues/130) — Declare host tool requirements and check them in pilotq doctor

> ## Goal
> Apply specq Host requirements (taskqorg/specq#98, owner 2026-10-29): each external tool pilotq runs on a host has a minimum version, checked by pilotq doctor.
> Install/update brings hosts up to it. Trigger: pilotq silently broke on FRA's old gh (pilotq#127).
>
> ## Acceptance
> A1. The contract lists each external tool/library pilotq runs on a host: name, minimum version, what needs it; or states none.
> A2. pilotq doctor checks each listed requirement: missing or below minimum fails naming the tool, found version, minimum and fix.
> A3. Install, onboarding and update bring managed hosts up to the minimum, or report the exact step and keep pilotq doctor failed.
> A4. Every external tool call logs command and error on failure; no failure reads as empty. A test runs against an old or missing tool fake.
> A5. Live check on one real host: pilotq doctor output with the found versions attached to the result.

Accepted source/update [comment 6497527863](https://github.com/taskqorg/pilotq/issues/130#issuecomment-6497527863):

> taskq: answer 1
> Allow worker host doctor - + worker proves A5 on the candidate; - extends scope to reading existing installation configuration

> A5. Live check on one real host: pilotq doctor output with the found versions attached to the result.

### Source #139

[#139](https://github.com/taskqorg/pilotq/issues/139) — Обеспечить применение specq ко всему общению агентов PilotQ

> ## Goal
> Выяснить, почему уже подключённые правила specq не обеспечивают краткие и понятные сообщения PilotQ, и исправить спецификацию и её применение к ответам, планам и задачам.
>
> ## Acceptance
> A1. Причины расхождения между правилами и реальными сообщениями подтверждены ссылками на код, спецификации и действующие инструкции; приведено сравнение с TaskQ.
> A2. Определено, каких правил не хватает и какие уже есть, но не применяются; устранены подтверждённые причины в спецификации и интеграции PilotQ.
> A3. Контракт охватывает ответы в чате, прогресс, планы, заголовки и описания задач, вопросы и результаты агентов; краткость не удаляет существенные условия и неопределённость.
> A4. На примерах из этого чата показаны проверяемые варианты до/после: краткий результат команды, статус задач, план и постановка задачи; исключены лишние вступления и повторные итоги.
> A5. Проверены переносы строк, плоские списки, выделение, ссылки и пустая строка перед статусом в rich Markdown, HTML и plain доставке; вопросы работников сохраняют исходный текст.
> A6. Применимые проверки проходят; подтверждено, какие инструкции получает новый и восстановленный сеанс после обновления, выполнено необходимое пересоздание из журнала.
> A7. Результат содержит состояние внедрения и живые примеры из Telegram; невыбранные существенные решения вынесены вопросами до зависимых изменений.

Accepted source/update [comment 6508376060](https://github.com/taskqorg/pilotq/issues/139#issuecomment-6508376060):

> taskq: answer
> 1. PM after merge. FRA is retired; the live host is pds18.ufobe.com. PM merges, releases, updates pds18 and posts chat examples and A6-A7 live evidence on this issue. Finish the PR; park with later on it if nothing else is left.

Accepted source/update [comment 6508382255](https://github.com/taskqorg/pilotq/issues/139#issuecomment-6508382255):

> taskq: steer
> Еще раз повторяю: у нас фра сервера уже нет. Мы сейчас все делаем на новом сервере PDS 18.
> op-id: pilotq-3cb9674929ca8680-r118-tq1
> revision: 6508352705

### Source #144

[#144](https://github.com/taskqorg/pilotq/issues/144) — Подключить GitLab-проект tight и проверить доступ к доске

> ## Goal
> Подключить gradus/tight на gitlab.ufobe.com к PilotQ для инфраструктурных задач PDS 18 и подтвердить чтение его доски из чата.
>
> ## Acceptance
> A1. Проверены поддержка GitLab, доступ к проекту и его доске; отсутствие интерфейса, отсутствие прав и сетевые ошибки различены.
> A2. При поддерживаемом подключении tight зарегистрирован и виден в project_list; существующие проекты остаются доступны.
> A3. Из чата получены фактические колонки и задачи доски либо подтверждено, что она пуста; приведены ссылки на источник.
> A4. Указано, пригодна ли доска для инфраструктурных задач PDS 18 и что требуется для выполнения через TaskQ; регистрация доски не выдаётся за доступ к серверу.

Accepted source/update [comment 6510147446](https://github.com/taskqorg/pilotq/issues/144#issuecomment-6510147446):

> taskq: answer
> PM filed https://github.com/taskqorg/taskq/issues/1885 (answers with actor, time and question revision; open-task progress read) and started it. Link it as the blocker and park with later --blocker if the PilotQ cutover needs it; the spec
> edit and gh removal that do not depend on it can go in now. create_claim_issue is not configured; the PM files findings by hand.

Accepted source/update [comment 6511288336](https://github.com/taskqorg/pilotq/issues/144#issuecomment-6511288336):

> taskq: answer
> PM releases taskq with #1885 and #1886 after #1886 merges, updates pds18, registers tight there and runs A2-A3 readback. You: no install changes. Park with later 3h, then verify inbox 144 --json on pds18 and hand in.

Accepted source/update [comment 6511752122](https://github.com/taskqorg/pilotq/issues/144#issuecomment-6511752122):

> taskq: steer
> PM done: taskq 4.6.42 on pds18; tight registered in PilotQ (projects list: agentq, pilotq, tight); taskq status on gradus/tight reads board, 0 tasks, complete. Verify inbox 144 --json and project_list, then hand in.

### Source #145

[#145](https://github.com/taskqorg/pilotq/issues/145) — Forward Telegram reply photos and documents to TaskQ

> ## Goal
> PilotQ delivers Telegram reply photos and documents through the TaskQ attachment CLI (taskq --file, from taskqorg/taskq#1817). Replaces #133, whose lost spawn on the retired hermes-fra1 can never be released.
>
> ## Acceptance
> A1. PilotQ passes downloaded photo/document files to TaskQ answer, steer and owner requeue with text optional, preserving task, revision and operation identity.
> A2. A live Telegram photo, a 20 MB document and mixed text plus files reach the intended worker as verified readable files; record exact client and TaskQ revisions.
> A3. Refusal or uncertain delivery preserves the reply and reports the exact reason and fix; same-operation retry never duplicates the board command.

Accepted source/update [comment 6512304808](https://github.com/taskqorg/pilotq/issues/145#issuecomment-6512304808):

> taskq: answer
> 145.1. Build A1 and A3 with local and CI proof, open the PR, and write the exact A2 live recipe (client, files, expected worker readback). PM releases and runs A2 on pds18 with the authorized client, then records revisions.

Accepted source/update [comment 6514740962](https://github.com/taskqorg/pilotq/issues/145#issuecomment-6514740962):

> taskq: answer 1
> Run live recipe - + proves A2 with readable worker copies; - requires authorized Telegram client and staged replies
> op-id: qac-mvugd3k915qutrm6y9s
> revision: 6513424806

Accepted source/update [comment 6539308095](https://github.com/taskqorg/pilotq/issues/145#issuecomment-6539308095):

> taskq: answer
> PM: Telegram cannot relay this question. #145 was filed outside PilotQ, so the PilotQ journal holds no subscription for it and task_answer refuses it. So the photo comes as a Telegram steer instead. Keep doing: for up to 20 minutes, check
> for steers and new .taskq/reply-files copies about every 30 seconds. Expect two steers: first pilotq-145.png as a Telegram photo (Telegram may recompress it), then pilotq-145-20MB.bin as a document (20,000,000 bytes, SHA256
> 42e65a57483f7993819c73125f9f266dad8788c191a5df93455270767466f36c). For each copy report path, name, length and SHA256; for the photo also report dimensions and whether it decodes. Then finish with that readback. An owner requeue with
> pilotq-145-mixed.txt follows later.

Accepted source/update [comment 6539336854](https://github.com/taskqorg/pilotq/issues/145#issuecomment-6539336854):

> taskq: steer
> A2 qualification: the current Telegram message attaches the photo. Please read the forwarded file and report its path, name, byte length, SHA256, dimensions, and whether it decodes.
>
> ![photo.jpg](https://github.com/taskqorg/pilotq/raw/08148379c6ee3178c2ff655e0e4514f1e0c3ea46/145/dac8e34070b4-photo.jpg)
> &lt;!-- taskq-files [{"id":"dac8e34070b41870e6fa1de7a6c813dd7b11655a445f63d80fccb1767995a415","name":"photo.jpg","mime":"image/jpeg","hash":"dac8e34070b41870e6fa1de7a6c813dd7b11655a445f63d80fccb1767995a415","bytes":2328,"url":"https://github.com/taskqorg/pilotq/raw/08148379c6ee3178c2ff655e0e4514f1e0c3ea46/145/dac8e34070b4-photo.jpg"}] --&gt;
> op-id: pilotq-3cb9674929ca8680-r188-tq1
> revision: 6539285880

Accepted source/update [comment 6539349409](https://github.com/taskqorg/pilotq/issues/145#issuecomment-6539349409):

> taskq: steer
> A2 qualification: the current Telegram message attaches pilotq-145-20MB.bin. Please verify the forwarded document copy and report its path, name, byte length, and SHA256. PilotQ could not preview this binary file.
>
> [pilotq-145-20MB.bin](https://github.com/taskqorg/pilotq/raw/0d8e75d68e33d136fa15624564d48e0841cbfba5/145/42e65a57483f-pilotq-145-20MB.bin)
> &lt;!-- taskq-files [{"id":"42e65a57483f7993819c73125f9f266dad8788c191a5df93455270767466f36c","name":"pilotq-145-20MB.bin","mime":"application/octet-stream","hash":"42e65a57483f7993819c73125f9f266dad8788c191a5df93455270767466f36c","bytes":20000000,"url":"https://github.com/taskqorg/pilotq/raw/0d8e75d68e33d136fa15624564d48e0841cbfba5/145/42e65a57483f-pilotq-145-20MB.bin"}] --&gt;
> op-id: pilotq-3cb9674929ca8680-r189-tq1
> revision: 6539285880

Accepted source/update [comment 6539397197](https://github.com/taskqorg/pilotq/issues/145#issuecomment-6539397197):

> taskq: answer
> Provenance recorded in comment 6539395990: client Telegram for macOS 13.1.0 (build 283357) on macOS 27.0.1; PilotQ 0.2.23 (v0.2.23 = 4316162623b3c45142d461c3fa335d2d6d929149) and TaskQ 4.6.52 (v4.6.52 =
> 73df356b19e841e3900c06878206bfdd73b7c41e) on pds18, binary SHA256s in that comment. The client is native Telegram for macOS, not Telegram Desktop. Step 5 deviation: the photo came as a steer because the question could not be relayed (no
> PilotQ subscription for #145); record that and finish A2 with the existing readbacks. Step 9 was not exercised live.

> A2. A live Telegram photo, a 20 MB document and mixed text plus files reach the intended worker as verified readable files; record exact client and TaskQ revisions.

> A3. Refusal or uncertain delivery preserves the reply and reports the exact reason and fix; same-operation retry never duplicates the board command.

### Source #151

[#151](https://github.com/taskqorg/pilotq/issues/151) — Issue tools through taskq issue

> ## Goal
> PilotQ issue_read, issue_comment and issue_label work through taskq issue N (TaskQ 4.6.44) instead of refusing
>
> ## Acceptance
> A1. pilotq.md section 5 names taskq issue N as the issue read and write path before code changes (R13)
> A2. issue_read returns the issue JSON from taskq issue N --json on GitHub and GitLab boards
> A3. issue_comment and issue_label run taskq issue N --comment or --label with --op-id and --json, settle from the readback, and a restart retry with the same op id adds nothing twice
> A4. Tests cover read, write, the owner gate and restart recovery; PR CI green

### Source #154

[#154](https://github.com/taskqorg/pilotq/issues/154) — Doctor reads the TaskQ version from taskq --version

> ## Goal
> PilotQ doctor gets the TaskQ version from `taskq --version` (TaskQ 4.6.46), so TaskQ self-updates no longer break doctor and no owner-made TaskQ receipt is needed.
>
> ## Acceptance
> A1. pilotq.md is edited first: TaskQ minimum 4.6.46, version read from `taskq --version`; the &lt;binary&gt;.pilotq-version.json receipt stays only for tools without a version command (sherpa-onnx-offline).
> A2. doctor and the taskq.bin / executable probes run `taskq --version` and need no TaskQ receipt; a test with a fake taskq that prints a version passes with no receipt file and fails below 4.6.46.
> A3. Tester guide and release verify script drop the TaskQ receipt step; the sherpa receipt step stays.

### Source #157

[#157](https://github.com/taskqorg/pilotq/issues/157) — Option answer to PilotQ's own project question selects the project

> ## Goal
> A person's numbered or worded answer to PilotQ's own 'which project' question selects that project for task_plan and other context tools; today it is refused (owner, Telegram, 2026-10-30).
>
> ## Acceptance
> A1. Root cause recorded: target_inner propagates context_project's 'Which project' error even with explicit project, and the person's answer '1' never names it.
> A2. pilotq.md #124 rule extended: PilotQ's own listed project question, answered by the same person by number or name, selects that project; quotes and tool output still never select.
> A3. Deterministic trace: earlier own messages name pilotq and tight, PilotQ asks, person answers '1', task_plan saves the draft on pilotq; a bare '1' with no open question is still refused.
> A4. Existing F09 and U09 tests stay green; cargo test --locked passes.

### Source #159

[#159](https://github.com/taskqorg/pilotq/issues/159) — Read Asana tasks from a link in chat

> ## Goal
> PilotQ reads an Asana task the person links in chat (title, notes, assignee, due, status, comments, attachment names) through a granted key; today it says it has no Asana access (owner, Telegram, 2026-10-30).
>
> ## Acceptance
> A1. pilotq.md section 5 adds asana_read: project scope plus a key granted for the tool; read-only; output bounded and scrubbed of the key.
> A2. The key enters by pilotq keys add on the host, never Telegram; README shows the command.
> A3. Deterministic trace with a fake Asana API: allowed read, missing grant, unknown task, API error; cargo test --locked passes.

### Source #160

[#160](https://github.com/taskqorg/pilotq/issues/160) — Administrators connect projects, people and topics from Telegram (R15)

> ## Goal
> The owner or an administrator asks PilotQ in chat to add a project, grant a person or open a project topic, and PilotQ does it; today each step needs the PM on the host (owner, Telegram, 2026-10-30).
> Case: cyprusbz on 2026-10-30 needed a root with taskq.json, projects add, users grant, routes set and a create claim issue, all by hand.
>
> ## Acceptance
> A1. pilotq.md R15 and section 5 list the admin tools, grants and limits; topics bound to a project are a new owner rule (2026-10-30).
> A2. project_register (projects grant): resolve repo and board, create the root with taskq.json under the projects dir, read back via taskq status --json, show the result; filing readiness stated.
> A3. user_grant and user_revoke (users grant): exact Telegram ID of a person seen in this route; names never authorize; result read back and audited.
> A4. topic_open (admin): create a forum topic via Bot API, set route policy, bind it to one registered project as its default; missing can_manage_topics is a plain refusal.
> A5. Deterministic traces with fake TaskQ and Bot API cover allow, refuse without grant, duplicate request and crash between effect and readback; cargo test --locked passes.

Accepted source/update [comment 6521032853](https://github.com/taskqorg/pilotq/issues/160#issuecomment-6521032853):

> taskq: answer 1
> users + project access - + reuses route authority and scopes the target; - needs both grants

### Source #163

[#163](https://github.com/taskqorg/pilotq/issues/163) — PilotQ command model: one catalog, project commands, structured help

> ## Goal
> PilotQ gets one command model: one catalog of every capability (slash commands and manager tools) by area, the single source for /help, CLI help and the manager's own knowledge of its tools.
> Projects get a complete command set; help serves a first-time user and a daily user; the taskq catalog shows live from taskq (taskqorg/taskq#1910).
>
> ## Acceptance
> A1. pilotq.md first states the command model (areas, naming, slash command vs plain request, grants, help layout); code follows it (spec first, R13).
> A2. One catalog in code lists every command and tool with area, purpose, example and grant; /help, `pilotq help` and manager tool descriptions render from it; a test fails when a capability is missing from it.
> A3. Projects area covers list, current and switch, status, connect (register) and disconnect; disconnect removes only PilotQ's registration, never the repository, board or files, with readback and audit.
> A4. /help shows a short start block (what PilotQ does, how to ask, 3 examples), then one line per area; `/help AREA` shows that area; entries outside the caller's grants are hidden, with one line on getting access.
> A5. Test-bot Telegram run: /help, /help projects and the plain request "which projects can I use" each answer from the catalog, in the caller's language; reply texts read back in the result.
> A6. Help ends with the taskq catalog read live from taskq's machine-readable help, no copy; a stub test proves a catalog change shows with no PilotQ change; a missing or older taskq gives one line.

Accepted source/update [comment 6521765531](https://github.com/taskqorg/pilotq/issues/163#issuecomment-6521765531):

> taskq: answer 1
> PM live run - + preserves §9; - needs PM evidence after worker implementation

Accepted source/update [comment 6522321848](https://github.com/taskqorg/pilotq/issues/163#issuecomment-6522321848):

> taskq: answer
> A5/A6 are post-release PM evidence (as #127). Accept the source review if merge-result CI passes. PM merges PR #164 through the merge queue, releases, updates pds18, then posts the three reply texts and the live catalog readback here
> before close.

> A5. Test-bot Telegram run: /help, /help projects and the plain request "which projects can I use" each answer from the catalog, in the caller's language; reply texts read back in the result.

> A6. Help ends with the taskq catalog read live from taskq's machine-readable help, no copy; a stub test proves a catalog change shows with no PilotQ change; a missing or older taskq gives one line.

### Source #165

[#165](https://github.com/taskqorg/pilotq/issues/165) — PilotQ conversation contract: clear plans, no technical messages

> ## Goal
> One spec defines what PilotQ shows a person at each stage, which buttons it offers and how they go away. Plans show only the task and its result; technical detail never reaches chat.
>
> ## Acceptance
> A1. Spec first: pilotq.md gets a Conversation contract section and docs/conversation-flow.md a stage table (trigger, text, buttons, press effect, replacement) before code; conflicting #72, #91, #93, #110 text is rewritten.
> A2. Plan card per task: title, one 'we will' line, one 'done when' line, in the person's language. Acceptance and scope stay in the task body. No 'Draft saved locally', revision, board or 'Use Run plan' lines.
> A3. Owner choices: no model/quota footer in replies, it moves to /status. A failure shows one plain line once and one on resolution; op-ids, JSON, CLI errors and retries go to audit and PM telemetry only.
> A4. Launch, cancel or a choice edits the same message: buttons removed, one outcome line added (launched with task links, cancelled, or chosen option and who). A stale press gets a popup toast, no chat message.
> A5. Each launch has one status card edited in place through queued, working, question and done. Questions follow the answer card; results lead with one effect line.
> A6. Snapshot tests render every card and stage from fixtures, with the owner's two screenshots as before cases. No rendered text holds an op-id, JSON, revision number or English CLI error.
> A7. PM live Telegram check on pds18: plan, launch, cancel, question and failure replies match the snapshots.

Accepted source/update [comment 6522366126](https://github.com/taskqorg/pilotq/issues/165#issuecomment-6522366126):

> taskq: steer
> Owner: keep liveness. A long step shows one plain status line under its card, edited in place (creating task, retry 2 of 5, waiting for a host) with elapsed time. The last edit shows the result, or a problem card with Retry/Cancel. Never a
> new message per retry.

Accepted source/update [comment 6522504624](https://github.com/taskqorg/pilotq/issues/165#issuecomment-6522504624):

> taskq: steer
> Owner confirmed liveness: in-place status line under the card (rate-limited edits) plus the 👀 reaction; on give-up it becomes a problem card with Retry/Cancel. Retries, op-ids and JSON go to audit and PM telemetry only. Write it into the
> spec.

Accepted source/update [comment 6523813686](https://github.com/taskqorg/pilotq/issues/165#issuecomment-6523813686):

> taskq: answer 1
> PM check after release - + uses §9 release path; - A7 remains unverified at publication; approve this timing exception.

> A7. PM live Telegram check on pds18: plan, launch, cancel, question and failure replies match the snapshots.

### Source #168

[#168](https://github.com/taskqorg/pilotq/issues/168) — Project onboarding: setup check and guided fixes in Telegram

> ## Goal
> When the owner adds a project (pilotq projects add or a new taskq install), pilotq runs the taskq setup check and pilotq doctor, then walks the owner through each gap in Telegram in plain steps until the project can run its first task.
>
> ## Acceptance
> A1. Spec first: pilotq.md names the onboarding flow (trigger, checks run, message shape, done state) before code lands.
> A2. projects add ends with pilotq doctor plus the taskq setup check (taskqorg/taskq#1938; until it ships, taskq service status) and reports each failed item as one plain line with the exact fix or owner step.
> A3. Telegram flow: one message lists the gaps; the owner taps a fix or Recheck; items pilotq may fix itself are fixed after one OK; the flow ends with all green or a named owner-only step.
> A4. A project with a gap is never marked ready; its first task is not started until the check passes or the owner overrides in writing.
> A5. Test reproduces the 2026-10-30 pds18 case: cyprusbz-board with codex limit 4 and no codex grant is reported as a gap with the grant command, not as ready.

Accepted source/update [comment 6524652683](https://github.com/taskqorg/pilotq/issues/168#issuecomment-6524652683):

> taskq: answer 1
> Registered roots only - + preserves §3; - new TaskQ projects need explicit PilotQ registration first

Accepted source/update [comment 6525104391](https://github.com/taskqorg/pilotq/issues/168#issuecomment-6525104391):

> taskq: steer
> Add one setup check from a live gap on 2026-10-30: issue_read refused for cyprusbz because the only issue-tool key (pilotq-github) was granted to project pilotq alone, so the owner's 'статус' reply said the result was unreadable. The check
> flags every registered project with no key granted for issue_read and prints the exact 'pilotq keys grant &lt;key&gt; project:&lt;name&gt;' fix. PM granted cyprusbz, tight and agentq by hand; keep the fixture for the gap.

### Source #170

[#170](https://github.com/taskqorg/pilotq/issues/170) — Status answers state board facts; research tasks filed as research

> ## Goal
> Asked about a task, PilotQ reads its board state and close result and says plainly: open or closed, verdict, report, proposed follow-up tasks and whether they exist.
> Diagnose and research tasks PilotQ files carry --type research, so their reports reach Reports.
>
> ## Acceptance
> A1. Owner case 2026-10-30: asked 'статус' about cyprusbz#540, PilotQ said it was not open and its result unconfirmed. The answer must say: closed, diagnosis accepted, report docs/research/540.md, fix task #541 in work.
> A2. Status answers come from taskq issue/status output (state, close comment, linked tasks), never from guesses; when a fact is missing PilotQ names the missing fact once, without hedging chains.
> A3. Diagnose, research and investigate requests are filed with --type research; a fixture test covers the mapping.
> A4. Telegram check on pds18 with the #540 question; reply text in the task result.

Accepted source/update [comment 6526009490](https://github.com/taskqorg/pilotq/issues/170#issuecomment-6526009490):

> taskq: answer
> 1

Accepted source/update [comment 6526080283](https://github.com/taskqorg/pilotq/issues/170#issuecomment-6526080283):

> taskq: answer
> 1. The PM runs the A4 Telegram check on pds18 after release and posts the reply text on this task. The worker delivers A1-A3 with local/CI evidence and names A4 as PM-pending in the result.

### Source #174

[#174](https://github.com/taskqorg/pilotq/issues/174) — Operator sees DEV deploy state and releases a project to LIVE with one button

> ## Goal
> Owner chose 2026-10-30: PilotQ owns the cs DEV/LIVE flow. Status must show if a merged task is on DEV; an owner-only tool previews a LIVE release and runs it only after a Release button press.
> Spec first (R13). Each project declares DEV health URL, release, dry-run and pipeline check in its PilotQ registration; cs reference values are in the first comment.
>
> ## Acceptance
> A1. pilotq.md and docs/taskq-tool-contract.md define the DEV state read and the release tool (grants, preview, confirmation, pinned SHA, refusals, audit) in the same PR as the code
> A2. task_status and project_status say 'on DEV since T' when the merged SHA is an ancestor of the SHA from the declared DEV health URL, 'not yet on DEV' otherwise, unknown when the read fails
> A3. the release tool needs a separate owner grant; another caller, a missing release config or an unregistered root is refused plainly with no command run
> A4. preview runs the dry run and shows target SHA, commit count since LIVE, linked task URLs, commit authors, migrations yes/no and whether DEV serves that SHA, then a Release button
> A5. the button binds to previewed SHA, owner and conversation generation; stale, foreign or repeated presses run nothing; release runs pinned to the previewed SHA, never a newer main
> A6. release is refused, naming the blocker, while another registered checkout of the same repository on that host has a running TaskQ attempt (cs: tight board on pds18)
> A7. release runs only in the registered root with durable intent before spawn; an interrupted run is uncertain, never auto-repeated; after it PilotQ reports release, smoke and backmerge per stage with links and the LIVE SHA
> A8. tests cover DEV ancestor check, grant refusal, stale button, pinned SHA and blocker refusal (local and CI only); PM records live acceptance on the next owner-requested cs release on PDS 18

Accepted source/update [comment 6527711591](https://github.com/taskqorg/pilotq/issues/174#issuecomment-6527711591):

> taskq: answer 3
> Button runs prerequisites - + One-button migration release; - Grants PilotQ LIVE database and backup commands.

> A4. preview runs the dry run and shows target SHA, commit count since LIVE, linked task URLs, commit authors, migrations yes/no and whether DEV serves that SHA, then a Release button

> A7. release runs only in the registered root with durable intent before spawn; an interrupted run is uncertain, never auto-repeated; after it PilotQ reports release, smoke and backmerge per stage with links and the LIVE SHA

> A8. tests cover DEV ancestor check, grant refusal, stale button, pinned SHA and blocker refusal (local and CI only); PM records live acceptance on the next owner-requested cs release on PDS 18

### Source #179

[#179](https://github.com/taskqorg/pilotq/issues/179) — Task status answers name the delivery stage from board and pipeline facts

> ## Goal
> Asked whether a task is done, PilotQ states one delivery stage (closed, merged, on DEV, DEV smoke, on LIVE) read from the board and the project pipeline, never a guess from worker notes.
>
> ## Acceptance
> A1. Spec first: pilotq.md DEV visibility defines the task delivery stage and its fact sources: board state, merge commit, DEV deploy and smoke jobs of the first main pipeline containing it, LIVE branch containing it.
> A2. A task status answer reads DEV deploy and smoke state for the merge commit through the project release adapter; it says unknown only when that source fails, and names the failure.
> A3. Worker close notes (e.g. deployed after-check unknown) never override pipeline facts; CI passed is claimed only when the main pipeline after the merge passed.
> A4. A failed DEV smoke is named with its job link and the first failing check, plus whether a later pipeline containing the commit passed.
> A5. Internal progress such as Restoring the conversation is not sent to the person as a chat message.
> A6. Deterministic test with a fake adapter replays the cs#541 trace: merge 16:10Z, web_dev success, smoke failed twice, next pipeline green, LIVE behind; reply states each fact once.
> A7. Live check on pds18 after release: ask the status of a recently closed cyprusbz task in the owner topic; the reply matches GitLab; audit id recorded.

Accepted source/update [comment 6530383167](https://github.com/taskqorg/pilotq/issues/179#issuecomment-6530383167):

> taskq: answer 1
> requeue: retry with a fresh attempt

> A1. Spec first: pilotq.md DEV visibility defines the task delivery stage and its fact sources: board state, merge commit, DEV deploy and smoke jobs of the first main pipeline containing it, LIVE branch containing it.

> A6. Deterministic test with a fake adapter replays the cs#541 trace: merge 16:10Z, web_dev success, smoke failed twice, next pipeline green, LIVE behind; reply states each fact once.

> A7. Live check on pds18 after release: ask the status of a recently closed cyprusbz task in the owner topic; the reply matches GitLab; audit id recorded.

### Source #186

[#186](https://github.com/taskqorg/pilotq/issues/186) — Chat files: read documents, no stale image, video, refusals in chat language

> ## Goal
> Owner live test 2026-10-31 04:25-04:27 local, direct chat: a 310 KB PDF and a 15 MB zip got a description of an earlier header screenshot ("4, Simon") plus "content unavailable"; a 20.4 MB rar got "The image is too large".
> A 1:02 video got "format is not supported yet"; all refusals were English while the owner wrote Russian (brief() keys on the Telegram language label). The agent must read what was actually sent, and replies fit the chat.
>
> ## Acceptance
> A1. Root cause of the stale image is named from the audit of these requests; a turn never describes an earlier image as the current attachment; the agent prompt states exactly which files this message carries.
> A2. Documents up to 20 MiB are downloaded and given to the agent as readable files (PDF text, archive listing, plain text); the reply describes the real content or names the exact reason it cannot.
> A3. Video and video notes up to 20 MiB are accepted (frames or audio as the runtime allows) or refused with a clear reason; an over-limit document refusal says file, not image.
> A4. Refusal text follows the language of the chat or the message text, not only the client language label; a Russian chat gets Russian refusals.
> A5. Live check on pds18 after release with the same four types (PDF, zip, file over 20 MiB, video) sent by an authorized tester; record request audit ids.

> A3. Video and video notes up to 20 MiB are accepted (frames or audio as the runtime allows) or refused with a clear reason; an over-limit document refusal says file, not image.

> A5. Live check on pds18 after release with the same four types (PDF, zip, file over 20 MiB, video) sent by an authorized tester; record request audit ids.

### Source #189

[#189](https://github.com/taskqorg/pilotq/issues/189) — Rich delivery for every PilotQ message and a short /help

> ## Goal
> Owner asked for rich Telegram replies by default and a readable /help; only final answers go rich (final_outbox has 2 callers, 25 outbox() callers send split replies plain).
> pilotq.md §9 itself makes /help append the full TaskQ catalog; fix the root cause spec first and route every user-visible message through one send path.
>
> ## Acceptance
> A1. Spec first: docs/runtime-contract.md states the delivery rule for every user-visible message on every path (builtins, notices, errors, final answers)
> A2. Spec first: pilotq.md §9 keeps the TaskQ catalog behind `/help tasks` instead of appending it inline to /help
> A3. Every user-visible message uses sendRichMessage up to 32768 UTF-16 units, then HTML chunks, then plain text; the split-goes-plain shortcut is gone
> A4. One test enumerates every outbox kind and fails if any user-visible kind bypasses the shared send path
> A5. /help in English and Russian fits one rich message and reads as a short command list
> A6. Live pds18 check: /help, /help tasks, an error notice and a long final answer arrive as rich messages; evidence in the result

Accepted source/update [comment 6539441816](https://github.com/taskqorg/pilotq/issues/189#issuecomment-6539441816):

> taskq: steer
> A6 live pds18 check: PM performs it after the merged release (same as #190 A8, answer 1). Hand in with a ready-to-run check list (messages and expected results); do not install live yourself.

Accepted source/update [comment 6539444968](https://github.com/taskqorg/pilotq/issues/189#issuecomment-6539444968):

> taskq: answer 1
> PM qualifies after release - + Preserves §9; - A6 evidence follows merge and release

Accepted source/update [comment 6539512055](https://github.com/taskqorg/pilotq/issues/189#issuecomment-6539512055):

> taskq: answer 1
> Allow fixture update - + Tests observe real rich edits; - Adds one test-file change beyond scope

> A1. Spec first: docs/runtime-contract.md states the delivery rule for every user-visible message on every path (builtins, notices, errors, final answers)

> A6. Live pds18 check: /help, /help tasks, an error notice and a long final answer arrive as rich messages; evidence in the result

### Source #190

[#190](https://github.com/taskqorg/pilotq/issues/190) — Full-capability PilotQ agent with TaskQ-first task work and stable long turns

> ## Goal
> Owner 2026-10-31: PilotQ is a full, smart agent with wide access like the hermes user (own and other hosts, system settings); its specialty is our tasks, via TaskQ first.
> Priority one is full, stable function; security is by trust for now. Today the spec makes Codex read-only, no shell/web, 60 s turns, 5 tool calls. Model and effort unchanged.
>
> ## Acceptance
> A1. Spec first: pilotq.md and docs/integration-contract.md describe a full Codex agent with the service user's access; TaskQ first for tasks, board state only via TaskQ (R1)
> A2. Spec first: the read-only, no-shell, restricted-tool and 5-call rules are removed, and the owner decision is quoted
> A3. task_status starts the DEV and delivery budget after board reads, names the exact failure reason, and a slow fake TaskQ test (9 s reads) still reports DEV state
> A4. Codex threads run with shell, network and web search and full filesystem access as the service user; tool turns stay limited to granted users
> A5. No fixed 60 s turn or tool-call cap; one topic progress message updates every 30 s at most; 10 min with no event cancels with a reason; /stop cancels
> A6. A service restart during a turn, including one the agent causes, ends with a delivered message saying what finished and what did not
> A7. Instructions say task work uses TaskQ tools first and other work is done directly; the "name unavailable sources" wording is gone
> A8. Live pds18 test topic: "is cs #541 on DEV?" is correct; disk usage plus last 20 pilotq journal lines runs via shell; a 5+ minute request shows progress and delivers

Accepted source/update [comment 6539439394](https://github.com/taskqorg/pilotq/issues/190#issuecomment-6539439394):

> taskq: answer 1
> PM qualification - + Keeps §9 release boundary; - A8 waits for PM after release

> A3. task_status starts the DEV and delivery budget after board reads, names the exact failure reason, and a slow fake TaskQ test (9 s reads) still reports DEV state

> A6. A service restart during a turn, including one the agent causes, ends with a delivered message saying what finished and what did not

> A8. Live pds18 test topic: "is cs #541 on DEV?" is correct; disk usage plus last 20 pilotq journal lines runs via shell; a 5+ minute request shows progress and delivers

## Marked owner notes

This table covers marked notes in the current tracked specifications, including dated owner markers with a task number. Notes embedded in the archived inventory are covered again by their I-row below. Task references inside a note’s body are not reclassified as independent owner attributions.

| Note and retained text | Source task text | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|
| `docs/adoption.md:11` — **Owner, 2026-10-23 (task #26):** the original agentq 0.9.0 comparison could not exercise PilotQ dynamic-tool results. Native shell access is now enabled under §5; that change does not qualify agentq or replace the measured dynamic-tool gate. C01–C07 run live with agentq off; agentq-on runs are recorded as not run with that reason. agentq qualification follows the agentq dynamic-tool adapter ([taskqorg/agentq#167](https://github.com/taskqorg/agentq/issues/167)) and enables agentq only if the P1 metric gate passes. `pilotq pilot start` records the pilot start, `pilotq pilot note intervention&#124;outage TEXT` records a manual intervention or a provider outage, and `pilotq pilot report` prints the [gate 4](acceptance.md#selection-gates) counts from the journal and telemetry.<br>6. Prepare migration inventory, restore proof and rollback. Switch production only after the acceptance gates pass and deployment is authorized. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Local protection only; absent behavior/platform evidence → [#199](https://github.com/taskqorg/pilotq/issues/199). |
| `docs/integration-contract.md:214` — **Owner, task #179:** task status names one highest proved delivery stage: closed (board issue state), merged (trusted TaskQ merge identity), on DEV (successful deploy job), DEV smoke (successful smoke job), or on LIVE (Git ancestry against the configured LIVE ref). Open issues remain open. Read delivery jobs through an optional protected `delivery_status` argument array with a separate `{sha}` argument, using the same executor, root, requirements, ten-second deadline and 64 KiB limit as the release adapter. Missing configuration, failed reads or malformed evidence name the unavailable source; worker notes never replace pipeline facts. Existing health observations remain a separate observation, not job evidence. | [#179](https://github.com/taskqorg/pilotq/issues/179) / Source #179 | `src/release.rs:217` | `src/release.rs:522` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/status-notifications.md:3` — **Owner decision (task #81, option A):** the minimal variant below, built on the #77 rollback DM; [pilotq.md §9](service-contract.md#update-and-monitoring) records it. Options outside it stay later stages. Notification grammar follows [specq notifications](https://github.com/taskqorg/specq/blob/main/caveman/rules/notifications.md). | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained current decision; local evidence only. |
| `docs/taskq-tool-contract.md:5` — **Owner-scoped (tasks #18 and #39):** every explicitly registered project, through the TaskQ CLI on the PilotQ host. TaskQ owns forge access, authentication, task URLs and trusted answers (#144, root contract §5). Registered entries bind local project roots, not a forge adapter. Direct issue tools refuse when TaskQ has no equivalent. | [#18](https://github.com/taskqorg/pilotq/issues/18) / Source #18; [#39](https://github.com/taskqorg/pilotq/issues/39) / Source #39 | `src/taskq.rs:366` | `tests/telegram.rs:2415` | met | Retained current decision; local evidence only. |
| `docs/taskq-tool-contract.md:32` — **Owner, task #165:** the [Conversation contract](conversation-contract.md#conversation-contract) owns presentation: each launch has one status card for all its tasks, edited through queue, work, question and closure. Worker questions remain separate answer cards; closure updates the status card instead of adding a closing message. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `src/journal.rs:273` | `src/journal.rs:3060` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/taskq-tool-contract.md:32` — **Owner, task #94:** within one board poll after an accepted answer from any surface, edit the originating question in place, remove its buttons and show `Answered by &lt;actor&gt;: &lt;text&gt;`. A late button tap shows that accepted answer as a toast and writes nothing. Read TaskQ's accepted answers through its CLI (§5). Bind each receipt to its original question revision; never attach an earlier answer to a newer question. Closing outcomes remain immutable. A projection older than the stored task revision is ignored.<br>- A periodic scan of `taskq status --json` recovers missed transitions; restart and `/new` keep subscriptions. An incomplete snapshot reads unknown and closes nothing.<br>- | [#94](https://github.com/taskqorg/pilotq/issues/94) / Source #94 | `src/journal.rs:2161` | `tests/telegram.rs:2752` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/taskq-tool-contract.md:36` — **Owner, task #108 (option 2):** a person may answer in free text without replying to a message. Resolve the intended question by conversational meaning, never elapsed time or the most recent question alone. If the question or answer is ambiguous, clarify before writing. `task_answer` forwards only the person's answer, bound to a same-conversation subscription, project, board and fresh question revision; it never chooses an answer. The source input must belong to that person in the current conversation generation. Recheck grants, registration and revision at dispatch, persist the intent before writing, and reconcile the same operation ID after interruption. Buttons and message replies keep the same boundary. | [#108](https://github.com/taskqorg/pilotq/issues/108) / Source #108 | `src/journal.rs:2131` | `tests/telegram.rs:3113` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/taskq-tool-contract.md:40` — **Owner, task #145:** Telegram photos and documents accompany `task_answer`, `task_steer` and owner `task_requeue` through TaskQ `--file`, with text optional. File identities come from admitted inputs by this caller in the current generation; model-supplied local paths are never accepted. A question-card media reply uses the deterministic answer boundary. Preserve task, revision and stable operation identity through recovery, retain refused/uncertain replies and explain the exact reason and fix. The [qualification recipe](reply-files-145.md) defines the PM's post-release A2 run; local CLI-shaped fixtures do not qualify live worker delivery. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `src/taskq.rs:610` | `tests/telegram.rs:3443` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:5` — **Owner, task #95 (option 1):** compile the pinned specq reply, indicator and task preparation/intake sources into the manager instructions, followed by PilotQ-specific tool and feedback rules. The specq submodule gitlink owns the pin; bumping it requires no Rust policy edit. Rebuild native conversations explicitly from the journal when instruction content changes. Relay worker question content unchanged, without the CLI answer line; never invent missing trade-offs or recommendation reasons. Completeness is enforced at the TaskQ source; qualification uses complete source cards. | [#95](https://github.com/taskqorg/pilotq/issues/95) / Source #95 | `src/instructions.rs:10` | `tests/telegram.rs:2643` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:7` — **Owner, task #139:** the pinned specq [Core](../specq/caveman/rules/core.md), [Documentation](../specq/caveman/rules/documentation.md) and [Change communication](../specq/caveman/rules/changes.md) apply across manager answers, factual progress, saved plans, task titles/goals/acceptance/scope, questions and results. Compile these sources into manager instructions; apply the documentation profile to persistent task text and the change profile to change reports. TaskQ remains responsible for worker/reviewer instruction delivery. PilotQ relays their questions unchanged under #95 and renders observed results without claiming publication or deployment from task closure alone. | [#139](https://github.com/taskqorg/pilotq/issues/139) / Source #139 | `src/instructions.rs:10` | `tests/telegram.rs:2743` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:13` — **Owner, task #97 (option 1):** Telegram worker questions follow TaskQ’s [answer-card layout](https://github.com/taskqorg/taskq/blob/8bea3bbcb35fba8664a9d42c46b53d0ed9607556/docs/design/answer-card.md#one-render), without a footer. Preserve source content and numbered options; buttons show only the number, label and recommendation star. Option details remain complete in the body. Visual-choice evidence keeps its source option numbers. | [#97](https://github.com/taskqorg/pilotq/issues/97) / Source #97 | `src/taskq.rs:278` | `src/taskq.rs:1016` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:15` — **Owner, task #97 continuation:** review the code before live visual evidence. A3's live relay screenshot is a post-merge PM check: the PM releases, sends one live ask through FRA and attaches the Telegram screenshot to the release note. No owner check is required. | [#97](https://github.com/taskqorg/pilotq/issues/97) / Source #97 | `src/taskq.rs:278` | `src/taskq.rs:1016` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:17` — **Owner, task #72:** send every user-visible message and private draft as rich Markdown, preserving model-authored paragraphs, lists, headings and bold. Keep safe escaped HTML and plain-text fallback from §4. Action replies lead with one short factual effect line under the Conversation contract. Plans use its three-line task summaries. Never claim that saving a plan created or started board work. Failures, limits and unsupported-input replies use one short sentence in the person’s language and omit internal request numbers; detailed diagnostics remain in the audit. | [#72](https://github.com/taskqorg/pilotq/issues/72) / Source #72 | `src/telegram.rs:517` | `src/journal.rs:3840` | narrowed | Before: final replies only; after: every visible message (§4/#189); existing all-kind delivery guard retained. |
| `docs/telegram-feedback.md:19` — **Owner, task #82:** replies stay compact and readable in every delivery path. A single line break the model wrote stays visible in rich Markdown and drafts; it never merges two lines into one. The model writes short paragraphs, flat lists, bold only for key words and inline links; no tables, nested lists, rules or raw HTML. The HTML fallback and fallback drafts render bullets, bold, italic, links, code and Markdown escapes without leaving raw Markdown marks. Every link is clickable with a short readable label: a bare URL becomes a labeled link (the host for a bare URL; TaskQ task links use `TaskQ · repo#number · title` with the exact CLI readback URL under #144), and task and service messages render through the same HTML subset. Repository file paths, including extensionless files such as src/Makefile or crates/Makefile, names with spaces such as docs/my file.md or docs/My file.md, unfamiliar extensions, non-ASCII names and line references, render as code, so Telegram does not auto-link them as domains. Replies, task messages and questions disable link previews. | [#82](https://github.com/taskqorg/pilotq/issues/82) / Source #82 | `src/telegram.rs:265` | `src/telegram.rs:808` | narrowed | F1: prior fixes incomplete; crates/Makefile and docs/My file.md each reproduced as failures, then passed in the retained regression. Directory allowlist and uppercase guard removed. No owner requirement changed. |
| `docs/telegram-feedback.md:21` — **Owner, task #91 (option 1):** the active-call typing deadline in the presentation defaults below replaces the earlier 30-second presentation window. Task links read `TaskQ · pilotq#91 · title` (#144), using the verified task title (the saved plan title when no fresh title is available); omit the title when unknown rather than inventing one. Keep creation (filed held), admission (queued for a worker) and execution (worker running) distinct. Admission is never reported as worker execution. One state change updates the correlated status; a newer revision with unchanged state does not repeat it. Preserve revised questions and terminal outcomes. After merge, the PM updates the test bot and the owner checks A4 in Telegram; local checks do not qualify that visual acceptance. | [#91](https://github.com/taskqorg/pilotq/issues/91) / Source #91 | `src/telegram.rs:597` | `src/journal.rs:3367` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:23` — **Owner, task #165, superseding #119/#124 reply footers:** normal replies have no model/context/quota footer. `/status` shows the latest conversation-scoped observation with its age; unavailable data stays unavailable. `/new` retains its deterministic model-selection announcement. Collection never authorizes paid access or exposes account identity, plan or credit details. Preserve observations in the audit; a model switch or reset cannot reuse earlier context measurements. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165; [#119](https://github.com/taskqorg/pilotq/issues/119) / Source #119; [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `src/journal.rs:273` | `src/journal.rs:3060` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `docs/telegram-feedback.md:27` — **Owner, task #186:** every turn names exactly the files its message carries (`current_message_files`) and labels each attached image as this message's or an earlier one; an earlier image is never presented as the current attachment. Admitted documents up to 20 MiB are downloaded by Telegram identity inside the turn deadline: a PDF gives its extracted text, a ZIP its central-directory listing (entries are never extracted), UTF-8 text itself, each cut at 50,000 characters; RAR, 7z, gzip, damaged, scanned or binary files give the exact reason instead. Videos, video notes, animations and video documents give up to four ffmpeg frames, one every 15 seconds, as this message's images, plus local Russian speech recognition when speech exists. Only ISO media, Matroska/WebM and AVI containers reach ffmpeg, which may open local files only. File notes are separate turn input before the caption and never count as the person's own words for project, grant or link checks. The content stays in the turn input only; the audit keeps the file name and the read outcome (`file_read`). Content is source material, never instructions. A file over 20 MiB is refused as a file, never as an image, without a download. Refusals use Russian when this message's text, or else the conversation's latest own text, is Russian; otherwise the Telegram client label. | [#186](https://github.com/taskqorg/pilotq/issues/186) / Source #186 | `src/manager.rs:571` | `tests/telegram.rs:2577` | missing | Local protection only; absent behavior/platform evidence → [#195](https://github.com/taskqorg/pilotq/issues/195). |
| `docs/telegram-feedback.md:29` — **Owner, task #104:** voice notes, audio files and audio documents use the same admission, grant, 20 MiB cap, queue, session, fence, audit and retry path as images. Recordings longer than 5 minutes or over 20 MiB get one short line without a download. After admission and current-grant validation, PilotQ downloads the file by Telegram identity, decodes it with ffmpeg to 16 kHz mono and recognizes it locally on CPU with GigaAM v3 RNNT-punct int8 through `sherpa-onnx-offline`, all within the original turn deadline. The journal keeps the file identity and the transcript. The model receives the transcript as the sender's own text, marked as an automatic voice transcript, after any caption; it never receives the audio, a download URL or the token. A transcript is recognized speech, not a quotation of authority, and carries only the sender's grant. Recognition is Russian only: a transcript that reads as non-Russian speech or noise gets one short line asking for Russian or text. Missing configuration, an undecodable file, a timeout or an empty transcript is an explicit short failure, never a text-only answer. A recovered run reuses a stored transcript instead of recognizing again. Installation config `voice` names `recognizer` and `model`, optionally `ffmpeg` and `threads`; without it voice requests fail with the short line. | [#104](https://github.com/taskqorg/pilotq/issues/104) / Source #104 | `src/voice.rs:71` | `tests/telegram.rs:3043` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `pilotq.md:76` — **Owner, task #190:** “Let it stay a full agent that can do anything, but its specialization is our tasks: for tasks it<br>works through TaskQ first, as its convenient mechanism.” “Access must be wide, full, like Hermes.” This decision<br>replaces the read-only, no-shell, restricted native-tool and five-call rules. Board state still comes only through<br>TaskQ; R9 subscription access, R10 admission and §9 release qualification remain binding. Native effects have<br>service-user authority, not typed-tool replay guarantees: persist observed tool outcomes and never replay an interrupted<br>turn automatically. After any restart, deliver the recorded completed findings and explicitly identify unfinished or<br>unconfirmed work. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: loaded linked policy corrected; failed before/passed after. Live proof → [#198](https://github.com/taskqorg/pilotq/issues/198). |
| `pilotq.md:167` — **Owner, task #163:** one code catalog owns capability names, areas, purposes, examples and required<br>grants for slash controls, operator commands and manager tools. Help and manager tool descriptions<br>render from that catalog; schemas and dispatch checks remain at their existing boundaries (R13,<br>R17). Areas are conversation, projects, tasks, reads, people and operations. Slash commands are<br>deterministic controls; plain requests select authorized manager tools. Operator-only commands are<br>labelled and never imply chat authority. Tool names use `area_verb`; existing names remain<br>compatible. | [#163](https://github.com/taskqorg/pilotq/issues/163) / Source #163 | `src/catalog.rs:99` | `src/catalog.rs:180` | missing | Local protection only; absent behavior/platform evidence → [#198](https://github.com/taskqorg/pilotq/issues/198). |

## Inventory: every source line

The section labels preserve the original inventory grouping. Each source link points to the original task text in the source register above; the exact I-row wording is retained here. Where a later owner decision supersedes a clause, that decision is named explicitly. The old wording remains evidence, not an instruction to restore removed restrictions.

### PilotQ product contract

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I001 (`docs/contract-177-inventory.md:6`) — PilotQ is the conversational front end for work on a shared server. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I002 (`docs/contract-177-inventory.md:7`) — People discuss work in Telegram; PilotQ answers, performs bounded direct actions, or manages execution through TaskQ. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I003 (`docs/contract-177-inventory.md:8`) — A lost model connection must never silently lose an accepted request or authorize a repeated external action. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I004 (`docs/contract-177-inventory.md:9`) — Date: 2026-10-24. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I005 (`docs/contract-177-inventory.md:10`) — Stage: implemented test service; first release preparation. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I006 (`docs/contract-177-inventory.md:11`) — The name and repository are owner-approved; the owner made the repository public on 2026-10-22 so `main` rules apply (docs/bootstrap.md). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I007 (`docs/contract-177-inventory.md:12`) — The owner chose the base on 2026-10-22 (section 3). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I008 (`docs/contract-177-inventory.md:13`) — Requirements marked **Owner** come from the requested workflow. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I009 (`docs/contract-177-inventory.md:14`) — **Proposed** sections define a design to review and qualify, not shipped guarantees or permission to deploy. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I010 (`docs/contract-177-inventory.md:15`) — Shared engineering, storage, security and release rules belong to [specq](https://github.com/taskqorg/specq/tree/main/engineering). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I011 (`docs/contract-177-inventory.md:16`) — Evidence: [research](docs/research.md), [mechanism matrix](docs/mechanism-matrix.md). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I012 (`docs/contract-177-inventory.md:17`) — Requirement checks: [acceptance](docs/acceptance.md). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
### Specification hierarchy

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I013 (`docs/contract-177-inventory.md:18`) — The owner requires spec-driven work. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I014 (`docs/contract-177-inventory.md:19`) — `specq` is the shared base; this root contract contains only PilotQ-specific decisions and additions. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I015 (`docs/contract-177-inventory.md:20`) — For PilotQ, an explicit owner-approved decision or exception here takes priority over a conflicting shared default. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I016 (`docs/contract-177-inventory.md:21`) — Unmentioned areas inherit `specq`. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I017 (`docs/contract-177-inventory.md:22`) — An exception names the shared rule, scope and reason; an accidental conflict or an unapproved proposal is not an exception. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I018 (`docs/contract-177-inventory.md:23`) — Supporting documents, README, instructions, tests and code follow this contract. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I019 (`docs/contract-177-inventory.md:24`) — They cannot introduce independent product decisions. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I020 (`docs/contract-177-inventory.md:25`) — Update the owning specification before implementing a changed decision. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I021 (`docs/contract-177-inventory.md:26`) — The development-live exception in section 9 is owner-approved for tasks #106, #108, #117 and #119. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Documentary/provenance clause retained; no executable history or live claim. |
### 1. Outcomes and boundaries

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I022 (`docs/contract-177-inventory.md:27`) — **Owner requirements** | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I023 (`docs/contract-177-inventory.md:28`) — - **R1 Conversation:** run on a server, serve multiple authorized people through DMs, groups and forum topics, and preserve who said what and where. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `pilotq.md:15` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I024 (`docs/contract-177-inventory.md:29`) — - **R2 Direct help:** discuss, research briefly, answer, inspect a bounded resource and make a small authorized update without filing every request as a task. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `pilotq.md:16` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I025 (`docs/contract-177-inventory.md:30`) — - **R3 Management:** use TaskQ for most substantial work: clarify, plan, file, follow, answer status questions and relay worker questions and results. | [#18](https://github.com/taskqorg/pilotq/issues/18) / Source #18 | `pilotq.md:17` | `src/taskq.rs:366` | `tests/telegram.rs:1145` | met | Retained; named deterministic protection covers the domain risk. |
| I026 (`docs/contract-177-inventory.md:31`) — - **R4 Flexible projects:** remember the usual project but allow a conversation to switch projects. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `pilotq.md:10` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I027 (`docs/contract-177-inventory.md:32`) — Every external action must resolve a concrete authorized target. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `pilotq.md:10` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I028 (`docs/contract-177-inventory.md:33`) — - **R5 Reliability:** ordinary conversations and simple tasks work without repeated nudges. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `pilotq.md:11` | `src/journal.rs:1920` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I029 (`docs/contract-177-inventory.md:34`) — Restarts, timeouts and long context have explicit recoverable outcomes. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `pilotq.md:11` | `src/journal.rs:1920` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I030 (`docs/contract-177-inventory.md:35`) — - **R6 Continuity:** preserve existing people, access roles, conversation boundaries and project associations during a later Hermes migration. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:20` | `src/journal.rs:1016` | `tests/telegram.rs:677` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I031 (`docs/contract-177-inventory.md:36`) — - **R7 Context:** support effective agentq integration, including original-data recovery and a measured quality baseline. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:21` | `src/journal.rs:1016` | `tests/telegram.rs:677` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I032 (`docs/contract-177-inventory.md:37`) — - **R8 Visibility:** provide task URLs, current state, attempt evidence and outcomes through existing TaskQ facilities. | [#18](https://github.com/taskqorg/pilotq/issues/18) / Source #18 | `pilotq.md:18` | `src/taskq.rs:366` | `tests/telegram.rs:1145` | met | Retained; named deterministic protection covers the domain risk. |
| I033 (`docs/contract-177-inventory.md:38`) — - **R9 Model access:** use Codex with existing subscription access only. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `pilotq.md:7` | `src/manager.rs:99` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I034 (`docs/contract-177-inventory.md:39`) — Claude and separately billed model API access are out of scope. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `pilotq.md:7` | `src/manager.rs:99` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I035 (`docs/contract-177-inventory.md:40`) — Do not fall back automatically to paid API credentials. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `pilotq.md:7` | `src/manager.rs:99` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I036 (`docs/contract-177-inventory.md:41`) — - **R10 People and admission:** remember distinct users in persistent, correctable profiles. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `pilotq.md:5` | `src/journal.rs:1016` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I037 (`docs/contract-177-inventory.md:42`) — Ignore a person until an authorized administrator explicitly grants access; group membership alone grants nothing. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `pilotq.md:5` | `src/journal.rs:1016` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I038 (`docs/contract-177-inventory.md:43`) — Support inspecting and correcting remembered information and changing or revoking access. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:5` | `src/journal.rs:1016` | `tests/telegram.rs:677` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I039 (`docs/contract-177-inventory.md:44`) — - **R11 Observable lifecycle:** make acceptance, progress, waiting, interruption, failure and completion understandable in Telegram. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `pilotq.md:14` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I040 (`docs/contract-177-inventory.md:45`) — Cover recovery and operational edge cases in the specification and acceptance plan before implementation, rather than adding them only after incidents. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `pilotq.md:5` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I041 (`docs/contract-177-inventory.md:46`) — - **R12 Least effort:** build the simplest agent that serves TaskQ work. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:23` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I042 (`docs/contract-177-inventory.md:47`) — Reuse proven mechanisms from known projects and the shared TaskQ system instead of inventing new ones. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:23` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I043 (`docs/contract-177-inventory.md:48`) — - **R13 Two front ends:** offer a command-line interface and a Telegram gateway over the same request, grant and audit path. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `pilotq.md:13` | `src/main.rs:400` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I044 (`docs/contract-177-inventory.md:49`) — - **R14 Operated like our tools:** PilotQ is monitored, updates itself through releases and is developed spec-first through its own TaskQ board. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `pilotq.md:22` | `src/main.rs:82` | `tests/update.rs:70` | met | Retained; named deterministic protection covers the domain risk. |
| I045 (`docs/contract-177-inventory.md:50`) — - **R15 Administration by request:** an authorized administrator can tell PilotQ to remember a person with given rights and to create or register projects and open project-bound forum topics through the scoped administration tools in section 5. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `pilotq.md:19` | `src/admin.rs:119` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
| I046 (`docs/contract-177-inventory.md:51`) — - **R16 Keys:** tools can use credentials on an authorized person's behalf; the model never sees a credential value. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `pilotq.md:9` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I047 (`docs/contract-177-inventory.md:52`) — - **R17 Request audit and telemetry:** every request leaves one durable record of who asked, when, where, what was asked, what PilotQ did and the outcome, joined to CoreQ telemetry. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `pilotq.md:5` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I048 (`docs/contract-177-inventory.md:53`) — The record answers later service investigations, a person's questions about their own history and PilotQ self-improvement analysis. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `pilotq.md:5` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I049 (`docs/contract-177-inventory.md:54`) — **Proposed scope for the first release** | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:27` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I050 (`docs/contract-177-inventory.md:55`) — One Linux host, one Telegram bot, one manager runtime, existing TaskQ boards and explicitly registered project roots. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:29` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I051 (`docs/contract-177-inventory.md:56`) — Text, replies, task links and inline decisions are core inputs. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:27` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I052 (`docs/contract-177-inventory.md:57`) — Accept Telegram photos and image documents, with or without captions, as image input in the same Codex conversation. | [#72](https://github.com/taskqorg/pilotq/issues/72) / Source #72 | `pilotq.md:27` | `src/telegram.rs:32` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I053 (`docs/contract-177-inventory.md:58`) — Preserve their file identity and caption in the request journal. | [#72](https://github.com/taskqorg/pilotq/issues/72) / Source #72 | `pilotq.md:27` | `src/telegram.rs:32` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I054 (`docs/contract-177-inventory.md:59`) — **Owner, task #89 (option 1):** accept Telegram voice notes, audio files and audio documents as Russian speech, recognized locally into text input (§12). | [#104](https://github.com/taskqorg/pilotq/issues/104) / Source #104; [#89](https://github.com/taskqorg/pilotq/issues/89) / Source #89 | `pilotq.md:27` | `src/voice.rs:71` | `tests/telegram.rs:3043` | met | Retained; named deterministic protection covers the domain risk. |
| I055 (`docs/contract-177-inventory.md:60`) — **Owner, task #145:** photos and arbitrary documents may be forwarded as TaskQ reply attachments with optional text through `answer`, `steer` and owner `requeue` (§5); other documents outside that flow and video remain unsupported and receive one short plain-language line. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `pilotq.md:27` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Explicitly superseded/expanded by [#186](https://github.com/taskqorg/pilotq/issues/186); current typed-tool and qualification protections remain. |
| I056 (`docs/contract-177-inventory.md:61`) — Recurring work stays with an existing scheduler or TaskQ capability. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:27` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I057 (`docs/contract-177-inventory.md:62`) — PilotQ has no worker scheduler, autonomous goal engine, reviewer, new task board or general plugin marketplace. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:27` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I058 (`docs/contract-177-inventory.md:63`) — Its local inbox schedules conversation handling and delivery only; TaskQ remains authoritative for execution. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `pilotq.md:27` | `src/journal.rs:1016` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
### Conversation contract

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I059 (`docs/contract-177-inventory.md:64`) — **Owner, task #165:** [Conversation flow](docs/conversation-flow.md) is the single presentation contract for discussion, plan, launch, task status, questions, results and failure recovery (R11, R13, R17). | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:10` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I060 (`docs/contract-177-inventory.md:65`) — It supersedes presentation wording from #72, #91, #93, #110, #119 and #124; authorization, immutable task scope and durable receipt rules remain in force. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:12` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Explicitly superseded/expanded by [#165](https://github.com/taskqorg/pilotq/issues/165); current typed-tool and qualification protections remain. |
| I061 (`docs/contract-177-inventory.md:66`) — A plan displays each task's title, one “We will” line and one “Done when” line in the person's language. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:14` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I062 (`docs/contract-177-inventory.md:67`) — The latter is a concise outcome summary, not the worker's acceptance list. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:15` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I063 (`docs/contract-177-inventory.md:68`) — Acceptance and scope remain in the saved task body. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:16` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I064 (`docs/contract-177-inventory.md:69`) — Do not show local-save receipts, internal revisions, board details or launch instructions. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:16` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I065 (`docs/contract-177-inventory.md:70`) — Each display offers Launch and Cancel; revisions remain internal authorization tokens. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:17` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I066 (`docs/contract-177-inventory.md:71`) — Launch, Cancel and discussion choices edit their source message: remove its buttons and append one factual outcome line, including verified task links on launch or the chosen option and actor. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:20` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I067 (`docs/contract-177-inventory.md:72`) — A stale callback receives only a popup toast. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:22` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I068 (`docs/contract-177-inventory.md:73`) — A superseded display loses its buttons. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:22` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I069 (`docs/contract-177-inventory.md:74`) — A launch has one durable status card edited through queued, working, question and done, based on TaskQ observations; missing/deleted cards may be replaced in the original route. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:23` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I070 (`docs/contract-177-inventory.md:75`) — Worker questions follow the TaskQ answer card and are edited with the accepted answer. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:25` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I071 (`docs/contract-177-inventory.md:76`) — Results lead with one factual effect line; closure alone proves neither publication nor deployment. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:26` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I072 (`docs/contract-177-inventory.md:77`) — **Owner, #165 liveness continuation:** keep the 👀 receipt. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:8` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I073 (`docs/contract-177-inventory.md:78`) — A long launch edits one plain status line under its plan card, naming the current phase and elapsed time; it never sends a new message per retry. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:28` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I074 (`docs/contract-177-inventory.md:79`) — Rate-limit and coalesce these edits under §12 and the persisted Telegram cooldown. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:30` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I075 (`docs/contract-177-inventory.md:80`) — On completion replace that line with the receipt; on give-up show a plain problem line with Retry and Cancel. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:31` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I076 (`docs/contract-177-inventory.md:81`) — Retry resumes only the original authorized launch and operation IDs after current grant, target and generation checks. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:32` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I077 (`docs/contract-177-inventory.md:82`) — Cancel stops remaining launch steps, preserves prior effects and never parks or stops already admitted workers. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:33` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I078 (`docs/contract-177-inventory.md:83`) — Old problem buttons become stale after either action, reset or a later problem display. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:34` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I079 (`docs/contract-177-inventory.md:84`) — Retry counts and diagnostics remain in audit and PM telemetry; this presentation change does not enlarge existing retry budgets. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:35` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I080 (`docs/contract-177-inventory.md:85`) — Deadline replies retain readable findings from completed reads and offer to plan the remainder (§5); findings never turn an uncertain write into success. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:38` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I081 (`docs/contract-177-inventory.md:86`) — Completion cards retain the trusted TaskQ closing result and limitations after the effect line; unavailable verdicts remain explicitly unconfirmed. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:40` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I082 (`docs/contract-177-inventory.md:87`) — Recovered reply refusals use the same localized line as live refusals; the original reason and remedy stay in audit. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:41` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I083 (`docs/contract-177-inventory.md:88`) — Normal replies have no model/context/quota footer. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:44` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Explicitly superseded/expanded by [#165](https://github.com/taskqorg/pilotq/issues/165); current typed-tool and qualification protections remain. |
| I084 (`docs/contract-177-inventory.md:89`) — These observations belong to `/status`, scoped to the conversation and labeled with observation age. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:45` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Explicitly superseded/expanded by [#165](https://github.com/taskqorg/pilotq/issues/165); current typed-tool and qualification protections remain. |
| I085 (`docs/contract-177-inventory.md:90`) — Missing observations stay unavailable. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:45` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Explicitly superseded/expanded by [#165](https://github.com/taskqorg/pilotq/issues/165); current typed-tool and qualification protections remain. |
| I086 (`docs/contract-177-inventory.md:91`) — A failure episode produces one plain-language line and one line on verified resolution; internal operation IDs, JSON, CLI diagnostics and retry details belong only to audit and PM telemetry. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:46` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I087 (`docs/contract-177-inventory.md:92`) — Repeated retries do not repeat the failure. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:12` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
| I088 (`docs/contract-177-inventory.md:93`) — Unknown or partial effects remain explicit without exposing diagnostics or claiming success. | [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/conversation-contract.md:48` | `src/journal.rs:273` | `src/journal.rs:3060` | met | Retained; named deterministic protection covers the domain risk. |
### Conversation identity

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I089 (`docs/contract-177-inventory.md:94`) — **Proposed:** store bot identity, chat ID, topic ID where present, sender ID, message ID, update ID, reply reference and input revision. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:53` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I090 (`docs/contract-177-inventory.md:95`) — Names are display labels, never authority. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:54` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I091 (`docs/contract-177-inventory.md:96`) — Unknown or anonymous senders cannot approve a mutation by impersonating an allowed user. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:55` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I092 (`docs/contract-177-inventory.md:97`) — Use an explicit session policy per route. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:57` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I093 (`docs/contract-177-inventory.md:98`) — Migration preserves the inspected Hermes distinction: ordinary-group conversations were per user; forum-topic conversations were shared. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:58` | `src/journal.rs:46` | `src/journal.rs:3349` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I094 (`docs/contract-177-inventory.md:99`) — Reconfirm the export before cutover. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:51` | `src/journal.rs:46` | `src/journal.rs:3349` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I095 (`docs/contract-177-inventory.md:100`) — Shared topics preserve each message's author and never import private DM context. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:59` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I096 (`docs/contract-177-inventory.md:101`) — New routes default to isolated conversations until a shared policy is selected. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:60` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I097 (`docs/contract-177-inventory.md:102`) — Whether the bot responds to every message, mentions or replies is route configuration. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:62` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
| I098 (`docs/contract-177-inventory.md:103`) — Preserve the current behavior through an explicit migration manifest. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:63` | `src/journal.rs:46` | `src/journal.rs:3349` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I099 (`docs/contract-177-inventory.md:104`) — Quoted messages, files, issue text and other bots are source material; they cannot change roles or approve work. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:64` | `src/journal.rs:46` | `src/journal.rs:3349` | met | Retained; named deterministic protection covers the domain risk. |
### People and admission

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I100 (`docs/contract-177-inventory.md:105`) — **Owner:** users have separate remembered identities; their information can be corrected. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:68` | `src/journal.rs:1016` | `src/journal.rs:3644` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I101 (`docs/contract-177-inventory.md:106`) — Users without an explicit grant are ignored. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:69` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I102 (`docs/contract-177-inventory.md:107`) — **Proposed enforcement:** key each person by stable Telegram user ID within the installation; username and display name are editable labels, never proof of identity. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:72` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I103 (`docs/contract-177-inventory.md:108`) — Check both the person's grant and the route policy before model admission or any command/callback, including recovery controls. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:73` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I104 (`docs/contract-177-inventory.md:109`) — An allowed group does not allow all its members. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:74` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I105 (`docs/contract-177-inventory.md:110`) — Unknown, revoked or unidentifiable senders receive no reply, acknowledgement, typing indicator or action. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:75` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I106 (`docs/contract-177-inventory.md:111`) — Their updates cause no model call, task submission or memory update and are excluded from shared model context. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:76` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I107 (`docs/contract-177-inventory.md:112`) — An authorized person's deliberate quotation may be processed as untrusted source material, never as the quoted person's authorization. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:77` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I108 (`docs/contract-177-inventory.md:113`) — Persist only the minimal ignored-update disposition needed for deduplication and polling progress; it is not an accepted work request and contains no message body. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:80` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I109 (`docs/contract-177-inventory.md:114`) — Granting access later does not replay ignored messages. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:69` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I110 (`docs/contract-177-inventory.md:115`) — The newly allowed person can send a new request. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:82` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I111 (`docs/contract-177-inventory.md:116`) — An authorized administrator can inspect a user's access and permitted profile fields, grant scoped access, change it or revoke it through Telegram. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:84` | `src/journal.rs:1016` | `src/journal.rs:3644` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I112 (`docs/contract-177-inventory.md:117`) — Accept an exact Telegram user ID or a verified sender reference; resolve ambiguous names before writing. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:86` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I113 (`docs/contract-177-inventory.md:118`) — Record the acting administrator, target ID and before/after grant. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:86` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I114 (`docs/contract-177-inventory.md:119`) — Admission to conversation does not automatically grant project mutations, TaskQ management or user administration. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:87` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I115 (`docs/contract-177-inventory.md:120`) — A Telegram group-admin badge does not confer PilotQ administration. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:88` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I116 (`docs/contract-177-inventory.md:121`) — Bootstrap the initial owner ID through protected installation configuration, never through the first incoming message. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:89` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
| I117 (`docs/contract-177-inventory.md:122`) — Model output, remembered job titles and self-declarations cannot grant rights. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/conversation-contract.md:90` | `src/journal.rs:1016` | `src/journal.rs:3644` | met | Retained; named deterministic protection covers the domain risk. |
### Discuss, act, delegate

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I118 (`docs/contract-177-inventory.md:123`) — **Proposed:** discussion never implies execution. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:107` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Retained; named deterministic protection covers the domain risk. |
| I119 (`docs/contract-177-inventory.md:124`) — A direct request such as “check the disk space” already authorizes that bounded check. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:95` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I120 (`docs/contract-177-inventory.md:125`) — “Let's plan” produces a revisable proposal. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:96` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Retained; named deterministic protection covers the domain risk. |
| I121 (`docs/contract-177-inventory.md:126`) — “Run this plan” binds authorization to the displayed plan revision, target and scope; it does not require another ritual confirmation. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:97` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Retained; named deterministic protection covers the domain risk. |
| I122 (`docs/contract-177-inventory.md:127`) — Direct work must satisfy all of these conditions: | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:107` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I123 (`docs/contract-177-inventory.md:128`) — - One clear authorized target and a bounded result. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:102` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I124 (`docs/contract-177-inventory.md:129`) — - No need to survive as a long-running job, produce a reviewed implementation, or coordinate dependencies. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I125 (`docs/contract-177-inventory.md:130`) — - A read-only operation or a small mutation with an explicit grant and a verifiable outcome. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I126 (`docs/contract-177-inventory.md:131`) — - Execution fits the short-action budget in section 7. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:103` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I127 (`docs/contract-177-inventory.md:132`) — Examples: inspect one service, read a bounded log interval, look up a ticket or explain an error. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I128 (`docs/contract-177-inventory.md:133`) — Apart from the scoped administration and owner release operations in section 5, the only direct writes are the owner's board comments and labels ([section 5](#direct-actions-keys-and-project-switching)). | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I129 (`docs/contract-177-inventory.md:134`) — A fast action can still be consequential: deployment, money, permissions or deletion do not become direct-safe because they take seconds. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I130 (`docs/contract-177-inventory.md:135`) — Multi-file implementation, long research, broad audits, repeated experiments, deployment work and uncertain-duration work go to TaskQ, except the owner-confirmed registered release operation in §5. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I131 (`docs/contract-177-inventory.md:136`) — On reaching a direct-action budget, preserve observations and completed effects; propose the remaining task. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:93` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I132 (`docs/contract-177-inventory.md:137`) — Reuse existing authorization if it covers the delegation. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:109` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Retained; named deterministic protection covers the domain risk. |
| I133 (`docs/contract-177-inventory.md:138`) — Never describe a task as created until board readback identifies it. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:109` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Retained; named deterministic protection covers the domain risk. |
| I134 (`docs/contract-177-inventory.md:139`) — When acting as TaskQ PM, PilotQ does not perform queued task work or choose answers on the owner's behalf. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:112` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I135 (`docs/contract-177-inventory.md:140`) — Its short-action role is separate from that PM role. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:113` | `src/instructions.rs:10` | `src/instructions.rs:37` | narrowed | F2: old direct-work restriction reached loaded instructions; fixed against §5/#190, failed-before/passed-after policy check. |
| I136 (`docs/contract-177-inventory.md:141`) — This preserves the [TaskQ role contract](https://github.com/taskqorg/taskq/blob/93b499cf562167e380793c0c8fb15562407c6900/taskq.md). | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/conversation-contract.md:114` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Retained; named deterministic protection covers the domain risk. |
### Changing direction

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I137 (`docs/contract-177-inventory.md:142`) — Persist follow-up messages immediately. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:118` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I138 (`docs/contract-177-inventory.md:143`) — Serialize model turns and compaction within a conversation; other conversations continue independently. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:118` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I139 (`docs/contract-177-inventory.md:144`) — New input can wait, steer at a supported safe point, or stop the current short action. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:119` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I140 (`docs/contract-177-inventory.md:145`) — Do not silently discard an earlier request when a new message arrives. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:120` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I141 (`docs/contract-177-inventory.md:146`) — **Owner, task #92:** relate messages by conversational meaning, without time windows or requiring replies or special syntax from the person. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:123` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I142 (`docs/contract-177-inventory.md:147`) — A clear addition updates the existing intent; an independent request stays separate. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:124` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I143 (`docs/contract-177-inventory.md:148`) — For a matching doing TaskQ task, use `task_steer` after a fresh read instead of creating another task. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:125` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I144 (`docs/contract-177-inventory.md:149`) — If the destination is unclear, ask “to task X or new?” in the person's language, with explicit options and an invitation to answer in their own words; do not mutate a guessed target. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:127` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I145 (`docs/contract-177-inventory.md:150`) — A reply is contextual evidence, never authorization by itself. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:128` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I146 (`docs/contract-177-inventory.md:151`) — Follow-ups received during generation are committed immediately and processed in order after the active turn. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:130` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I147 (`docs/contract-177-inventory.md:152`) — Their next turn receives the preceding result and task receipts, so it can refine the plan or steer the task already created. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:131` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I148 (`docs/contract-177-inventory.md:153`) — Preserve each input's author, caption, message and reply reference; do not merge audit records or grants. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:132` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I149 (`docs/contract-177-inventory.md:154`) — On every turn supply a bounded journal projection of the current conversation generation (the latest 12 handled inputs plus the current one), the latest saved plan and accessible task subscriptions with their observation times. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:134` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I150 (`docs/contract-177-inventory.md:155`) — It is historical source material, not renewed authorization or fresh board state. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:136` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I151 (`docs/contract-177-inventory.md:156`) — Attach available image bytes from that projection again, including multiple images before a text follow-up, so native-session recovery does not replace images with file IDs alone. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:137` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I152 (`docs/contract-177-inventory.md:157`) — Apply §12 download protections and the original turn deadline to all attachments. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:138` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I153 (`docs/contract-177-inventory.md:158`) — An unavailable historical image is an explicit context gap; an unavailable current image fails the request. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:139` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I154 (`docs/contract-177-inventory.md:159`) — Context outside this projection may require clarification; never pretend to have seen missing content. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:141` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I155 (`docs/contract-177-inventory.md:160`) — `/new`, route isolation and current grants bound the projection. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:142` | `src/journal.rs:1674` | `tests/telegram.rs:2494` | met | Retained; named deterministic protection covers the domain risk. |
| I156 (`docs/contract-177-inventory.md:161`) — Plain-language controls are primary. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:144` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I157 (`docs/contract-177-inventory.md:162`) — Deterministic `/status`, `/stop` and `/new` are recovery controls that remain available when the model is unavailable. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:145` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I158 (`docs/contract-177-inventory.md:163`) — `/new` resets the model conversation, not task links, authorization records or active TaskQ work. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:146` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I159 (`docs/contract-177-inventory.md:164`) — “Stop” must resolve its scope. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:148` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I160 (`docs/contract-177-inventory.md:165`) — Stopping the manager turn, parking a task and pausing new queue admission are different actions. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:148` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I161 (`docs/contract-177-inventory.md:166`) — A queue pause must not be reported as stopping an active worker. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:149` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
| I162 (`docs/contract-177-inventory.md:167`) — Use supported TaskQ transitions and verify their result; explain partial or pending termination. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/conversation-contract.md:150` | `src/journal.rs:1674` | `src/journal.rs:3695` | met | Retained; named deterministic protection covers the domain risk. |
### 3. Architecture and ownership

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I163 (`docs/contract-177-inventory.md:168`) — **Owner decision 2026-10-22 (option A):** PilotQ is one small Rust service in the TaskQ tool family. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:8` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I164 (`docs/contract-177-inventory.md:169`) — &#124; Part &#124; Choice &#124; Reason &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:10` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I165 (`docs/contract-177-inventory.md:170`) — &#124; Language and packaging &#124; Rust single binary on [coreq](https://github.com/taskqorg/coreq) &#124; Same release, update, service, status, selftest and incident path as taskq, agentq and dashq (R14) &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:12` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I166 (`docs/contract-177-inventory.md:171`) — &#124; Telegram &#124; [teloxide](https://github.com/teloxide/teloxide) for the Bot API only &#124; Maintained Rust library; PilotQ keeps its own durable ingress and outbox (section 4) &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:13` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I167 (`docs/contract-177-inventory.md:172`) — &#124; Telegram behavior &#124; Patterns adapted from the Hermes gateway: update intake, delivery retry, stall watch, topic routing &#124; Already exercised on our server (R12); read as reference, no change to Hermes &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:14` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I168 (`docs/contract-177-inventory.md:173`) — &#124; Manager runtime &#124; Codex `app-server` under the subscription login, one adapter &#124; R9; [core options](docs/core-options.md) &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:15` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I169 (`docs/contract-177-inventory.md:174`) — &#124; Journal &#124; SQLite (section 7 settings) &#124; One local file, supported snapshot backup &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:16` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I170 (`docs/contract-177-inventory.md:175`) — &#124; Work execution &#124; Existing TaskQ CLI &#124; Section 5 &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:17` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I171 (`docs/contract-177-inventory.md:176`) — &#124; Voice recognition &#124; ffmpeg and `sherpa-onnx-offline` with GigaAM v3 RNNT-punct int8, local CPU subprocesses &#124; Best Russian accuracy per CPU in the #89 benchmark; keeps the static PilotQ build free of onnxruntime &#124; | [#104](https://github.com/taskqorg/pilotq/issues/104) / Source #104 | `docs/runtime-contract.md:18` | `src/voice.rs:71` | `tests/telegram.rs:3043` | met | Retained; named deterministic protection covers the domain risk. |
| I172 (`docs/contract-177-inventory.md:177`) — Grammy, Pi and a new runtime framework are out of scope. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:20` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I173 (`docs/contract-177-inventory.md:178`) — Earlier source findings remain in [research](docs/research.md). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:21` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I174 (`docs/contract-177-inventory.md:179`) — ```mermaid flowchart LR   T[Telegram] &lt;--&gt; P[PilotQ ingress and delivery]   P &lt;--&gt; D[(Conversation journal)]   P &lt;--&gt; A[Manager runtime]   A --&gt; B[Authorized tool boundary]   B --&gt; Q[TaskQ]   B --&gt; S[Short actions]   Q --&gt; W[Workers and reviewers]   Q --&gt; P   C[agentq] --&gt; A ``` | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:6` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I175 (`docs/contract-177-inventory.md:180`) — &#124; State &#124; Authority &#124; PilotQ responsibility &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:36` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I176 (`docs/contract-177-inventory.md:181`) — &#124; Received inputs, response delivery, route policy, grants and request correlation &#124; PilotQ journal &#124; Commit, deduplicate, restore and expose failures &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:38` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I177 (`docs/contract-177-inventory.md:182`) — &#124; Model conversation and summaries &#124; Runtime session, backed by recoverable PilotQ input and finalized-output records &#124; Preserve session identity; rebuild explicitly if native state is unusable &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:39` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I178 (`docs/contract-177-inventory.md:183`) — &#124; Plans being discussed &#124; Versioned PilotQ proposal &#124; Track target, author and authorization scope &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:40` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I179 (`docs/contract-177-inventory.md:184`) — &#124; Admitted task state, attempts, questions, dependencies and completion &#124; TaskQ trusted board log &#124; Read and invoke supported commands; cache only with source time &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:41` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I180 (`docs/contract-177-inventory.md:185`) — &#124; Project roots and worker capacity &#124; TaskQ registration and host policy &#124; Use registered entries; register a new project only on an administrator request (section 5); no directory discovery or silent enrollment &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:42` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I181 (`docs/contract-177-inventory.md:186`) — &#124; Compressed tool output and recovery reference &#124; agentq adapter plus retained original evidence &#124; Preserve access scope and validate recovered content &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:43` | `src/main.rs:295` | `tests/telegram.rs:677` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I182 (`docs/contract-177-inventory.md:187`) — Keep the runtime interface small: open/resume by ID, submit, stream events, interrupt, observe terminal result and close. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:45` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I183 (`docs/contract-177-inventory.md:188`) — Implement one adapter first. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:46` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I184 (`docs/contract-177-inventory.md:189`) — Do not build a universal runtime framework or expose native runtime tools that bypass the authorization boundary. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:46` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I185 (`docs/contract-177-inventory.md:190`) — The manager uses a neutral explicit workspace. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:49` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I186 (`docs/contract-177-inventory.md:191`) — Project context is resolved through the approved registry and scoped tools, avoiding a process-wide current-directory switch shared by conversations. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:50` | `src/main.rs:295` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I187 (`docs/contract-177-inventory.md:192`) — Separate runtime processes are fault boundaries; OS permissions and mounts enforce actual resource isolation. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:51` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
### 4. Durable conversation and effects

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I188 (`docs/contract-177-inventory.md:193`) — **Proposed invariant:** every durably accepted request is queryable as queued, running, waiting for a person, delegated, answered, failed, cancelled or uncertain. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:57` | `src/journal.rs:1833` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I189 (`docs/contract-177-inventory.md:194`) — Completion and Telegram delivery are separate fields. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:96` | `src/journal.rs:1833` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I190 (`docs/contract-177-inventory.md:195`) — No persistent state is inferred from a typing indicator or model prose. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:59` | `src/journal.rs:1833` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
### Ingress

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I191 (`docs/contract-177-inventory.md:196`) — Use long polling initially. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:63` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I192 (`docs/contract-177-inventory.md:197`) — Commit admitted inputs, minimal ignored-update dispositions, deduplication keys and the next polling cursor in one SQLite transaction before making a Telegram request that confirms those updates. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:64` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I193 (`docs/contract-177-inventory.md:198`) — A handler cannot advance the cursor past an uncommitted disposition. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:65` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I194 (`docs/contract-177-inventory.md:199`) — A duplicate admitted update produces the existing request, not a second turn or task; ignored updates remain ignored. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:66` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I195 (`docs/contract-177-inventory.md:200`) — Use one OS-held polling lock per bot installation; PID text alone is insufficient. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:69` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I196 (`docs/contract-177-inventory.md:201`) — The supported deployment is one host per token. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:70` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I197 (`docs/contract-177-inventory.md:202`) — A Telegram conflict from a second host is an incident, not an endless retry loop. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:70` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I198 (`docs/contract-177-inventory.md:203`) — Deployment explicitly stops the old consumer before activating a new one. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:71` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I199 (`docs/contract-177-inventory.md:204`) — Treat update IDs as protocol identifiers, not proof of uninterrupted human history. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:73` | `src/journal.rs:1016` | `tests/telegram.rs:370` | met | Retained; named deterministic protection covers the domain risk. |
| I200 (`docs/contract-177-inventory.md:205`) — Telegram retains undelivered updates for at most 24 hours; after a longer outage report the possible gap. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:74` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I201 (`docs/contract-177-inventory.md:206`) — Do not claim that a new bot can fetch all old group history. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:75` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I202 (`docs/contract-177-inventory.md:207`) — Edited messages create a revision. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:77` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
| I203 (`docs/contract-177-inventory.md:208`) — They can revise an unstarted proposal; they cannot silently alter already authorized work. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:77` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
| I204 (`docs/contract-177-inventory.md:209`) — If work started, present the edit as a change request. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:78` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
| I205 (`docs/contract-177-inventory.md:210`) — A deleted input does not revoke an already executed side effect. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:79` | `src/journal.rs:1016` | `tests/telegram.rs:370` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
### Runs and tools

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I206 (`docs/contract-177-inventory.md:211`) — Persist request and run identity before invoking a runtime. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:83` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I207 (`docs/contract-177-inventory.md:212`) — Require a native terminal outcome; a closed stream or an empty response is not success. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:84` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I208 (`docs/contract-177-inventory.md:213`) — A generation/fencing token prevents a replaced or aborted runtime from emitting new authorized tool calls or final replies. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:85` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I209 (`docs/contract-177-inventory.md:214`) — Persist the intent and stable operation ID before each external effect; persist its outcome and readback after it. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:87` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I210 (`docs/contract-177-inventory.md:215`) — A crash can occur between these steps. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:88` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I211 (`docs/contract-177-inventory.md:216`) — Recovery rules are: | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:81` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I212 (`docs/contract-177-inventory.md:217`) — &#124; Interrupted operation &#124; Recovery &#124; | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:90` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I213 (`docs/contract-177-inventory.md:218`) — &#124; Pure model generation or bounded read &#124; Retry within budget; distinguish regenerated text from the original &#124; | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:81` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I214 (`docs/contract-177-inventory.md:219`) — &#124; TaskQ mutation &#124; Reconcile the same operation ID and revision against TaskQ &#124; | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:93` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I215 (`docs/contract-177-inventory.md:220`) — &#124; Direct idempotent mutation &#124; Read back the target, then retry only under its declared idempotency contract &#124; | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:94` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I216 (`docs/contract-177-inventory.md:221`) — &#124; Non-idempotent or unobservable external effect &#124; Mark uncertain, block automatic replay and request a decision &#124; | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:95` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I217 (`docs/contract-177-inventory.md:222`) — &#124; Completed result with pending Telegram delivery &#124; Retry delivery; do not rerun the agent or tools &#124; | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:96` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I218 (`docs/contract-177-inventory.md:223`) — SQLite records are the durable acceptance boundary. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:98` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I219 (`docs/contract-177-inventory.md:224`) — Pi's native JSONL session is not sufficient evidence of power-loss durability or atomic external effects. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:99` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I220 (`docs/contract-177-inventory.md:225`) — Preserve a recoverable transcript projection and tool receipts before acknowledging completion. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:100` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
| I221 (`docs/contract-177-inventory.md:226`) — Rebuilding a damaged native session must not replay previous mutations. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:101` | `src/journal.rs:1920` | `src/journal.rs:3504` | met | Retained; named deterministic protection covers the domain risk. |
### Delivery

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I222 (`docs/contract-177-inventory.md:227`) — Commit final replies, worker questions and results to an outbox with immutable destination and correlation ID. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:105` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I223 (`docs/contract-177-inventory.md:228`) — Preserve per-conversation order; coalesce superseded progress updates only. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:106` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I224 (`docs/contract-177-inventory.md:229`) — Keep terminal outcomes and unanswered questions. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:107` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I225 (`docs/contract-177-inventory.md:230`) — Record Telegram message IDs after successful sends; update a known status message when useful. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:109` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I226 (`docs/contract-177-inventory.md:231`) — Telegram `sendMessage` has no general client idempotency key: after a lost send response, a duplicate visible reply is possible. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:110` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I227 (`docs/contract-177-inventory.md:232`) — Report an ambiguous send as such and use bounded at-least-once delivery for results. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:111` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I228 (`docs/contract-177-inventory.md:233`) — Never repeat task execution to solve a delivery problem. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:112` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I229 (`docs/contract-177-inventory.md:234`) — **Owner, task #68:** normal accepted text uses an 👀 reaction within five seconds, without a text acknowledgement. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:114` | `src/journal.rs:1016` | `src/journal.rs:3801` | met | Retained; named deterministic protection covers the domain risk. |
| I230 (`docs/contract-177-inventory.md:235`) — If the reaction fails, send a short text receipt. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:115` | `src/journal.rs:1016` | `src/journal.rs:3801` | met | Retained; named deterministic protection covers the domain risk. |
| I231 (`docs/contract-177-inventory.md:236`) — If an unstarted request waits over 30 seconds, send one “Queued behind N” notice with the observed number of earlier requests in its conversation. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:116` | `src/journal.rs:1016` | `src/journal.rs:3801` | met | Retained; named deterministic protection covers the domain risk. |
| I232 (`docs/contract-177-inventory.md:237`) — Busy, unsupported-input and recovery-control replies remain text. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:117` | `src/journal.rs:1016` | `src/journal.rs:3801` | met | Retained; named deterministic protection covers the domain risk. |
| I233 (`docs/contract-177-inventory.md:238`) — Keep the first-use retention notice as a separate message once per person. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:117` | `src/journal.rs:1016` | `src/journal.rs:3801` | met | Retained; named deterministic protection covers the domain risk. |
| I234 (`docs/contract-177-inventory.md:239`) — No Telegram reply exposes internal request numbers, including controls, failures and limits; IDs remain available through the operator CLI and audit records. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:119` | `src/journal.rs:1016` | `src/journal.rs:3801` | met | Retained; named deterministic protection covers the domain risk. |
| I235 (`docs/contract-177-inventory.md:240`) — Private-chat answers do not quote the latest user message. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:121` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I236 (`docs/contract-177-inventory.md:241`) — Groups and topics retain reply correlation, typing and edited task status messages. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:122` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I237 (`docs/contract-177-inventory.md:242`) — While generating in a private chat, stream user-visible answer text through sendRichMessageDraft, falling back to sendMessageDraft. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:123` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I238 (`docs/contract-177-inventory.md:243`) — Coalesce updates to at most one per second; drafts are ephemeral, fenced to the active run, and failures never change the request outcome or replace final delivery. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:124` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I239 (`docs/contract-177-inventory.md:244`) — Final answers first use sendRichMessage for content within 32768 UTF-16 units. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:128` | `src/telegram.rs:517` | `src/journal.rs:3840` | narrowed | F3: final-only wording widened by #189; current feedback aligned here; all-kind guard retained. |
| I240 (`docs/contract-177-inventory.md:245`) — On an explicit unsupported-method or content rejection, fall back to HTML sendMessage in stable ordered chunks of at most 4096 UTF-16 units, then plain text on entity errors. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:131` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I241 (`docs/contract-177-inventory.md:246`) — Preserve literal source content, escape generated HTML at the output boundary and never allow raw model HTML to become markup. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:133` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I242 (`docs/contract-177-inventory.md:247`) — Persist fallback chunks and individual receipts before sending; retries after an ambiguous rich send stay on that same delivery method. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:134` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I243 (`docs/contract-177-inventory.md:248`) — A draft is never a final receipt. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:135` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I244 (`docs/contract-177-inventory.md:249`) — On upgrade, native sessions with the older fixed tool catalogue rebuild explicitly from the recorded conversation and receipts before receiving the new tools; completed effects are never replayed. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:136` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
| I245 (`docs/contract-177-inventory.md:250`) — Split oversized replies safely; do not rerun generation. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:137` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I246 (`docs/contract-177-inventory.md:251`) — Reaction, draft, typing and final sends share a persisted per-chat cooldown after Telegram rate limits; no fallback bypasses retry_after. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:138` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I247 (`docs/contract-177-inventory.md:252`) — Respect server retry delays, bound retry counts and retain exhausted deliveries for operator recovery. | [#189](https://github.com/taskqorg/pilotq/issues/189) / Source #189 | `docs/runtime-contract.md:140` | `src/telegram.rs:517` | `src/journal.rs:3840` | met | Retained; named deterministic protection covers the domain risk. |
| I248 (`docs/contract-177-inventory.md:253`) — Forbidden or missing-chat errors must not reroute private results to a broader group or silently mark them delivered. | [#68](https://github.com/taskqorg/pilotq/issues/68) / Source #68 | `docs/runtime-contract.md:141` | `src/telegram.rs:517` | `src/journal.rs:3859` | met | Retained; named deterministic protection covers the domain risk. |
### 5. TaskQ integration

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I249 (`docs/contract-177-inventory.md:254`) — **Owner, task #144:** PilotQ reads and manages boards only through the installed TaskQ CLI (R13). | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:8` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I250 (`docs/contract-177-inventory.md:255`) — A registered project is the local root where TaskQ runs; TaskQ owns forge, host, authentication and board access. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:9` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I251 (`docs/contract-177-inventory.md:256`) — The registry binds that root to the board identity read back by `taskq status --json`, without a forge adapter or credential configuration in PilotQ. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:10` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I252 (`docs/contract-177-inventory.md:257`) — Use task URLs returned by TaskQ, never construct or interpret forge-specific issue URLs. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:11` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I253 (`docs/contract-177-inventory.md:258`) — Read task progress with `status --task` and accepted answers with `inbox --json` (actor, time and original question revision); writes use the supported `answer` and operation-receipt path. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:13` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I254 (`docs/contract-177-inventory.md:259`) — An unreadable response stays unknown. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:14` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I255 (`docs/contract-177-inventory.md:260`) — Missing TaskQ capabilities become TaskQ findings, never a PilotQ forge integration. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:15` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I256 (`docs/contract-177-inventory.md:261`) — Existing direct issue tools remain refused when TaskQ has no equivalent; their grants do not authorize a forge bypass. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:16` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I257 (`docs/contract-177-inventory.md:262`) — **Owner, task #151:** `issue_read`, `issue_comment` and `issue_label` use `taskq issue N` (TaskQ 4.6.44): reads take its `--json` issue JSON; writes run `taskq issue N --comment T&#124;--label L --op-id ID --json` and settle from its readback receipt. | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:17` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I258 (`docs/contract-177-inventory.md:263`) — A board registration grants no server access or worker capacity. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:19` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I259 (`docs/contract-177-inventory.md:264`) — The PM registers tight on PDS 18 and records the A3 chat readback after release; workers verify the implementation locally and in CI and make no installation changes. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:20` | `src/taskq.rs:144` | `src/taskq.rs:1362` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
| I260 (`docs/contract-177-inventory.md:265`) — **Proposed:** wrap the supported CLI with argument arrays and typed validation; no shell interpolation of Telegram text. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:23` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I261 (`docs/contract-177-inventory.md:266`) — Use its JSON status and existing task log facilities. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:24` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I262 (`docs/contract-177-inventory.md:267`) — Do not create a new public TaskQ API before evidence shows the CLI cannot support the required flow. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:25` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I263 (`docs/contract-177-inventory.md:268`) — - Read: registered projects, fresh status, task details, current question and attempt evidence. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:27` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I264 (`docs/contract-177-inventory.md:269`) — - Write: create a held task, start it, answer the current question, steer, requeue or park within the user's scope. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:28` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I265 (`docs/contract-177-inventory.md:270`) — - Exclude from ordinary manager tools: host grants, capacity changes, arbitrary closure/publication and forced cleanup. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:30` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I266 (`docs/contract-177-inventory.md:271`) — Expose a privileged operation only when separately granted and verified. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:31` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I267 (`docs/contract-177-inventory.md:272`) — - Resolve ticket links to an allowed board and project. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:32` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I268 (`docs/contract-177-inventory.md:273`) — A number such as `#12` alone is insufficient when it can match several boards. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:33` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I269 (`docs/contract-177-inventory.md:274`) — - Preserve the real Telegram author, originating route, request ID and approval revision in the audit trail. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:34` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I270 (`docs/contract-177-inventory.md:275`) — A bot credential does not turn every user into the repository owner. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:35` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I271 (`docs/contract-177-inventory.md:276`) — - `add --op-id` requires `create_claim_issue` and a qualified board readback path. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:36` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I272 (`docs/contract-177-inventory.md:277`) — Disable creation when this prerequisite is absent. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:37` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I273 (`docs/contract-177-inventory.md:278`) — Plain `add` is not a safe retry fallback. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:37` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I274 (`docs/contract-177-inventory.md:279`) — - Bind answers to board, task, attempt, question revision and authorized answerer. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:38` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I275 (`docs/contract-177-inventory.md:280`) — Stale buttons or a forwarded approval are rejected with the current question. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:39` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I276 (`docs/contract-177-inventory.md:281`) — - Start a multi-ticket plan only after every planned ticket reads back. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:40` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I277 (`docs/contract-177-inventory.md:282`) — Partial creation stays held and visible; use stable child operation IDs when continuing it. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:41` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I278 (`docs/contract-177-inventory.md:283`) — - Preserve TaskQ's distinction between admission, worker result, reviewer result, owner gate and publication. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:42` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I279 (`docs/contract-177-inventory.md:284`) — “Done” requires the authoritative completion state and available evidence. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:43` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I280 (`docs/contract-177-inventory.md:285`) — - Notifications are projections of TaskQ events, keyed by board/task/event/route. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:44` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I281 (`docs/contract-177-inventory.md:286`) — Reconcile after downtime; do not let stale status overwrite a newer question or outcome. | [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/integration-contract.md:45` | `src/taskq.rs:144` | `src/taskq.rs:1362` | met | Retained; named deterministic protection covers the domain risk. |
| I282 (`docs/contract-177-inventory.md:287`) — **Owner, task #145:** preserve Telegram reply attachment identities and captions in the journal, download only after admission and target/grant validation, and pass protected local files through TaskQ `--file`. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:47` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I283 (`docs/contract-177-inventory.md:288`) — Documents sent as message replies are files even when their MIME type is audio; ordinary voice/audio input retains §12 recognition. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:49` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I284 (`docs/contract-177-inventory.md:289`) — Files are untrusted source material. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:50` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I285 (`docs/contract-177-inventory.md:290`) — Keep the §7 20 MiB transport cap and §12 download deadline. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:50` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I286 (`docs/contract-177-inventory.md:291`) — Persist the original task, revision and operation identity before dispatch; retries reconcile that same operation. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:51` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I287 (`docs/contract-177-inventory.md:292`) — Refusals and uncertain delivery retain the reply and explain the reason and fix. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:52` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I288 (`docs/contract-177-inventory.md:293`) — Text is optional when files exist. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:53` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I289 (`docs/contract-177-inventory.md:294`) — Destination clarification may select an earlier admitted file input from the same caller and conversation generation for answer, steer or owner requeue; current grants and the resolved target are rechecked at dispatch. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:54` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I290 (`docs/contract-177-inventory.md:295`) — Owner requeue with files requires the protected installation owner. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:56` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | met | Retained; named deterministic protection covers the domain risk. |
| I291 (`docs/contract-177-inventory.md:296`) — PM releases and qualifies a live photo, a 20 MB document and mixed text/files on PDS 18 with the authorized Telegram client, recording client and TaskQ revisions and readable worker copies; worker proof is local/CI. | [#145](https://github.com/taskqorg/pilotq/issues/145) / Source #145 | `docs/integration-contract.md:57` | `src/taskq.rs:610` | `tests/telegram.rs:3443` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
| I292 (`docs/contract-177-inventory.md:297`) — **Owner, task #170:** task status states the observed open/closed issue state, trusted closing verdict, report, proposed follow-ups and verified existing follow-up state. | [#170](https://github.com/taskqorg/pilotq/issues/170) / Source #170 | `docs/integration-contract.md:60` | `src/taskq.rs:212` | `src/taskq.rs:1033` | met | Retained; named deterministic protection covers the domain risk. |
| I293 (`docs/contract-177-inventory.md:298`) — Read these through TaskQ issue/status output; absence from an open-task snapshot alone proves neither closure nor nonexistence. | [#170](https://github.com/taskqorg/pilotq/issues/170) / Source #170 | `docs/integration-contract.md:62` | `src/taskq.rs:212` | `src/taskq.rs:1033` | met | Retained; named deterministic protection covers the domain risk. |
| I294 (`docs/contract-177-inventory.md:299`) — Name each unavailable fact once. | [#170](https://github.com/taskqorg/pilotq/issues/170) / Source #170 | `docs/integration-contract.md:6` | `src/taskq.rs:212` | `src/taskq.rs:1033` | met | Retained; named deterministic protection covers the domain risk. |
| I295 (`docs/contract-177-inventory.md:300`) — Diagnose, research and investigate plans use TaskQ `--type research`, preserving that type through launch (R8, R13). | [#170](https://github.com/taskqorg/pilotq/issues/170) / Source #170 | `docs/integration-contract.md:63` | `src/taskq.rs:212` | `src/taskq.rs:1033` | met | Retained; named deterministic protection covers the domain risk. |
| I296 (`docs/contract-177-inventory.md:301`) — Live case cyprusbz#540 is PM verification after release; worker evidence is local/CI. | [#170](https://github.com/taskqorg/pilotq/issues/170) / Source #170 | `docs/integration-contract.md:252` | `src/taskq.rs:212` | `src/taskq.rs:1033` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
### First TaskQ tool contract

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I297 (`docs/contract-177-inventory.md:302`) — Contract detail: [first TaskQ tool contract](docs/taskq-tool-contract.md) holds the owner-scoped (tasks #18 and #39) TaskQ tools, grants, configuration, fences and refusals. | [#18](https://github.com/taskqorg/pilotq/issues/18) / Source #18 | `docs/integration-contract.md:69` | `src/taskq.rs:366` | `tests/telegram.rs:2415` | met | Retained; named deterministic protection covers the domain risk. |
| I298 (`docs/contract-177-inventory.md:303`) — It is binding as part of this section. | [#18](https://github.com/taskqorg/pilotq/issues/18) / Source #18 | `docs/integration-contract.md:71` | `src/taskq.rs:366` | `tests/telegram.rs:2415` | met | Retained; named deterministic protection covers the domain risk. |
### Plan discussion and action buttons

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I299 (`docs/contract-177-inventory.md:304`) — **Owner, tasks #93, #110 and #165:** a conversation has one open draft. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110; [#93](https://github.com/taskqorg/pilotq/issues/93) / Source #93; [#165](https://github.com/taskqorg/pilotq/issues/165) / Source #165 | `docs/integration-contract.md:75` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I300 (`docs/contract-177-inventory.md:305`) — Follow-ups revise it without displaying a number. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:75` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I301 (`docs/contract-177-inventory.md:306`) — A new draft starts only after launch or cancel; another project's plan waits until then. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:76` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I302 (`docs/contract-177-inventory.md:307`) — Each display has Launch and Cancel. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:73` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I303 (`docs/contract-177-inventory.md:308`) — Text launches the current draft; a button launches only its displayed immutable revision. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:77` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I304 (`docs/contract-177-inventory.md:309`) — Cancel closes it without a board effect or model call. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:78` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I305 (`docs/contract-177-inventory.md:310`) — The Conversation contract owns card text, source-message edits, stale-button toasts and the launch status card. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:79` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I306 (`docs/contract-177-inventory.md:311`) — Filing and starting happen after the turn, outside the model deadline, with replies derived from receipts. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:80` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I307 (`docs/contract-177-inventory.md:312`) — Text controls remain supported (R13). | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:81` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I308 (`docs/contract-177-inventory.md:313`) — - Discussion has no board effect. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:73` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I309 (`docs/contract-177-inventory.md:314`) — Saving a draft records its stable identity, project and a new immutable revision. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:83` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I310 (`docs/contract-177-inventory.md:315`) — Show the current task summaries under the Conversation contract; retain complete acceptance and scope in the task body. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:84` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I311 (`docs/contract-177-inventory.md:316`) — An earlier message may remain visible but its launch button is stale. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:85` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I312 (`docs/contract-177-inventory.md:317`) — - A launch button belongs to the displayed revision, saved project, originating conversation generation and the person who requested that display. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:87` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I313 (`docs/contract-177-inventory.md:318`) — Recheck current admission and TaskQ/project grants at dispatch. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:88` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I314 (`docs/contract-177-inventory.md:319`) — A foreign, forwarded, reset, superseded or repeated press cannot authorize a different revision or create another run. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:89` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I315 (`docs/contract-177-inventory.md:320`) — Accepted launches use the same `task_run_plan` boundary, stable operation IDs, readback and audit as text. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:91` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I316 (`docs/contract-177-inventory.md:321`) — - Concrete discussion questions offer two to four numbered choices, preserved in the message, buttons and recorded answer. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:92` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I317 (`docs/contract-177-inventory.md:322`) — Choosing one submits that answer to the conversation; it never authorizes launch. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:93` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I318 (`docs/contract-177-inventory.md:323`) — Launch remains a separate button or explicit later text request. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:94` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I319 (`docs/contract-177-inventory.md:324`) — Old or already answered choice cards are refused. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:95` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I320 (`docs/contract-177-inventory.md:325`) — - Example: “Plan reply improvements”, “also keep line breaks”, “and add buttons” produces one draft with three internal revisions, no board tickets. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:96` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I321 (`docs/contract-177-inventory.md:326`) — The latest display has “Run plan”; pressing it launches only that display. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:97` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Explicitly superseded/expanded by [#165](https://github.com/taskqorg/pilotq/issues/165); current typed-tool and qualification protections remain. |
| I322 (`docs/contract-177-inventory.md:327`) — Pressing either an earlier display or the same button again starts nothing new. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:98` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I323 (`docs/contract-177-inventory.md:328`) — Under #110 a separate idea waits for launch or Cancel, then starts a new draft. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:99` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
| I324 (`docs/contract-177-inventory.md:329`) — After launch, “also support audio” starts a separate discussion, preserving the launched scope. | [#110](https://github.com/taskqorg/pilotq/issues/110) / Source #110 | `docs/integration-contract.md:100` | `src/journal.rs:2077` | `tests/telegram.rs:2897` | met | Retained; named deterministic protection covers the domain risk. |
### Human trial

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I325 (`docs/contract-177-inventory.md:330`) — **Owner, task #42:** the human trial ran with the owner and Simon, the two administrators, on the separate test bot from 2026-10-27T15:15Z to 2026-10-29T20:46Z. | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:104` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I326 (`docs/contract-177-inventory.md:331`) — Its report covers request IDs, outcomes, refusals, failures and tester feedback; discovered defects are filed as held tasks. | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:106` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I327 (`docs/contract-177-inventory.md:332`) — Naming administrators does not itself apply grants or expand authority. | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:107` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I328 (`docs/contract-177-inventory.md:333`) — This human feedback supplements the 2026-10-23 scripted-run evidence above. | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:109` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I329 (`docs/contract-177-inventory.md:334`) — Keep [gate 4](docs/acceptance.md#selection-gates) separate and preserve its existing pilot start. | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:110` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I330 (`docs/contract-177-inventory.md:335`) — Hermes production cutover remains outside scope (§10 step 6). | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:110` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I331 (`docs/contract-177-inventory.md:336`) — See the [trial cases](docs/acceptance.md#human-trial-task-42). | [#42](https://github.com/taskqorg/pilotq/issues/42) / Source #42 | `docs/integration-contract.md:111` | `src/journal.rs:2796` | `src/journal.rs:3455` | met | Documentary/provenance clause retained; no executable history or live claim. |
### Direct actions, keys and project switching

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I332 (`docs/contract-177-inventory.md:337`) — **Owner decision 2026-10-23 (option B, task #23, section 10 step 4):** direct actions are reads. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190; [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:113` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I333 (`docs/contract-177-inventory.md:338`) — Only the owner (`owner_id` from protected configuration) may write directly, and only a comment or a label on an issue of the `taskqorg/pilotq` board; the administration and owner-confirmed registered release exceptions below are the only additional direct-write scopes. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:117` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I334 (`docs/contract-177-inventory.md:339`) — Keys enter only through the CLI. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:118` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I335 (`docs/contract-177-inventory.md:340`) — The manager gets these tools beside the TaskQ tools, checked at dispatch like them: | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:120` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I336 (`docs/contract-177-inventory.md:341`) — &#124; Tool &#124; Grants &#124; Effect &#124; | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:122` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I337 (`docs/contract-177-inventory.md:342`) — &#124; `project_list` &#124; current caller’s project grants &#124; Reads registered project names and boards accessible to the caller, marking the conversation’s current project; no key or board call &#124; | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:124` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I338 (`docs/contract-177-inventory.md:343`) — &#124; `project_use` &#124; project scope &#124; Sets the conversation's current project. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:125` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I339 (`docs/contract-177-inventory.md:344`) — The project must be registered and named in the person's own message &#124; | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:125` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I340 (`docs/contract-177-inventory.md:345`) — &#124; `issue_read` &#124; project scope and a key granted for the tool &#124; Reads one issue of a registered project through `taskq issue N --json`: title, state, labels, canonical URL, a bounded body excerpt, the latest comments as bounded excerpts and the read time &#124; | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:126` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I341 (`docs/contract-177-inventory.md:346`) — &#124; `issue_comment` &#124; owner, a key &#124; Adds one comment to a `taskqorg/pilotq` issue &#124; | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:127` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I342 (`docs/contract-177-inventory.md:347`) — &#124; `issue_label` &#124; owner, a key &#124; Adds one label to a `taskqorg/pilotq` issue &#124; | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:128` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I343 (`docs/contract-177-inventory.md:348`) — - **Owner, task #92:** for `task_status` and `task_steer` (and `task_answer` under #108), the agent may select a trusted journal subscription from this same conversation by context, without asking the person to repeat its project. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/integration-contract.md:130` | `src/direct.rs:175` | `tests/telegram.rs:3113` | met | Retained; named deterministic protection covers the domain risk. |
| I344 (`docs/contract-177-inventory.md:349`) — The tool boundary requires one matching subscribed task, current project grants and unchanged registered board/root. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/integration-contract.md:132` | `src/direct.rs:175` | `tests/telegram.rs:3113` | met | Retained; named deterministic protection covers the domain risk. |
| I345 (`docs/contract-177-inventory.md:350`) — An explicit project or issue URL disambiguates subscribed boards; a bare number matching several accessible subscriptions is refused. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/integration-contract.md:134` | `src/direct.rs:175` | `tests/telegram.rs:3113` | met | Retained; named deterministic protection covers the domain risk. |
| I346 (`docs/contract-177-inventory.md:351`) — This extends the Target rule below for these tools only; quoted text and external tool output do not create subscriptions. | [#92](https://github.com/taskqorg/pilotq/issues/92) / Source #92 | `docs/integration-contract.md:135` | `src/direct.rs:175` | `tests/telegram.rs:3113` | met | Retained; named deterministic protection covers the domain risk. |
| I347 (`docs/contract-177-inventory.md:352`) — - **Owner, task #124:** plan, run, status (including `project_status`), steer, requeue, park and `project_use` may resolve a project named in the caller’s earlier own messages of this conversation generation. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:136` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I348 (`docs/contract-177-inventory.md:353`) — Use one unambiguous accessible match; current own wording can disambiguate earlier mentions. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:138` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I349 (`docs/contract-177-inventory.md:354`) — Quoted content, issue text, tool output and other people’s messages never select it. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:139` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I350 (`docs/contract-177-inventory.md:355`) — Multiple fits require one short question listing the accessible names and no guessed board write. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:140` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I351 (`docs/contract-177-inventory.md:356`) — Recheck grants, registration and board/root at dispatch. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:141` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I352 (`docs/contract-177-inventory.md:357`) — A saved plan retains its bound target. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:141` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I353 (`docs/contract-177-inventory.md:358`) — For example, “What is the task status for Pilot Q?” followed by “Make a task so this works” resolves `pilotq` without repeating its name. | [#124](https://github.com/taskqorg/pilotq/issues/124) / Source #124 | `docs/integration-contract.md:142` | `src/direct.rs:135` | `src/taskq.rs:1178` | met | Retained; named deterministic protection covers the domain risk. |
| I354 (`docs/contract-177-inventory.md:359`) — - **Owner, task #157:** when PilotQ's tool boundary itself asked that question and its reply listed every option, the same person's next message in this conversation generation answers it by name or by a bare number counted in the reply's listing order; that selects the project for the tools above. | [#157](https://github.com/taskqorg/pilotq/issues/157) / Source #157 | `docs/integration-contract.md:144` | `src/direct.rs:99` | `src/taskq.rs:1224` | met | Retained; named deterministic protection covers the domain risk. |
| I355 (`docs/contract-177-inventory.md:360`) — Numbers bind to explicit numbered items, ignoring introductory mentions; missing, duplicate or ambiguous item mappings are refused. | [#157](https://github.com/taskqorg/pilotq/issues/157) / Source #157 | `docs/integration-contract.md:147` | `src/direct.rs:99` | `src/taskq.rs:1224` | met | Retained; named deterministic protection covers the domain risk. |
| I356 (`docs/contract-177-inventory.md:361`) — Any other message closes the question, and a bare number without an open question selects nothing. | [#157](https://github.com/taskqorg/pilotq/issues/157) / Source #157 | `docs/integration-contract.md:148` | `src/direct.rs:99` | `src/taskq.rs:1224` | met | Retained; named deterministic protection covers the domain risk. |
| I357 (`docs/contract-177-inventory.md:362`) — Quotes, issue text and tool output still never select. | [#157](https://github.com/taskqorg/pilotq/issues/157) / Source #157 | `docs/integration-contract.md:149` | `src/direct.rs:99` | `src/taskq.rs:1224` | met | Retained; named deterministic protection covers the domain risk. |
| I358 (`docs/contract-177-inventory.md:363`) — - Target: a tool resolves one registered project from an explicit `project` argument, an issue URL or the conversation's current project. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:150` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I359 (`docs/contract-177-inventory.md:364`) — An explicit project or URL must be named in the person's own message or be the current project; quoted messages, issue text and tool output never select one. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:152` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I360 (`docs/contract-177-inventory.md:365`) — A bare number with no current project and several accessible projects is refused with the list, never guessed. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:153` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
| I361 (`docs/contract-177-inventory.md:366`) — Another project stays read-only. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:113` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I362 (`docs/contract-177-inventory.md:367`) — - Budget: one request makes at most 5 PilotQ tool calls within the section 7 direct-action deadline. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:113` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I363 (`docs/contract-177-inventory.md:368`) — The sixth call is refused and writes nothing. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:153` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I364 (`docs/contract-177-inventory.md:369`) — A request that ends at its deadline replies with the evidence it kept, and the rest can become a planned task. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:113` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I365 (`docs/contract-177-inventory.md:370`) — - Effects: a comment or label persists its intent and operation ID `pilotq-&lt;installation&gt;-r&lt;request&gt;-e&lt;n&gt;` before the write, and its receipt after TaskQ's readback (#151). | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:157` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I366 (`docs/contract-177-inventory.md:371`) — TaskQ records the operation ID on the comment and returns the first effect for a repeated ID; a label already present is accepted without a write. | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:158` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I367 (`docs/contract-177-inventory.md:372`) — An `unknown` receipt, a failed call or a crash before the receipt stays pending; startup repeats the same command with the same operation ID, never a new one, and tells the route the settled result. | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:160` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I368 (`docs/contract-177-inventory.md:373`) — Effects recorded before #151 carry no TaskQ operation ID: they stay uncertain and are never repeated. | [#151](https://github.com/taskqorg/pilotq/issues/151) / Source #151 | `docs/integration-contract.md:162` | `src/direct.rs:312` | `src/direct.rs:438` | met | Retained; named deterministic protection covers the domain risk. |
| I369 (`docs/contract-177-inventory.md:374`) — - Revoking a project or key grant refuses every later call; a completed effect stays recorded. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/integration-contract.md:163` | `src/direct.rs:175` | `src/direct.rs:374` | met | Retained; named deterministic protection covers the domain risk. |
### Asana task reads

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I370 (`docs/contract-177-inventory.md:375`) — **Owner, task #159:** `asana_read` reads one Asana task linked in the caller's own message, using a registered project scope and a key granted for `asana_read`. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:167` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I371 (`docs/contract-177-inventory.md:376`) — It returns title, notes, assignee, due date/time, completion status, comments and attachment names through Asana's read-only API. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:169` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I372 (`docs/contract-177-inventory.md:377`) — Keys enter only through `pilotq keys add` on the host (§8), never Telegram. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:170` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I373 (`docs/contract-177-inventory.md:378`) — The link selects only a task, never a credential destination: requests use the fixed Asana API origin, without redirects or attachment downloads. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:171` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I374 (`docs/contract-177-inventory.md:379`) — Existing admission, target, five-call budget, deadline, audit and key scrubbing apply (R13, R16, R17). | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:172` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I375 (`docs/contract-177-inventory.md:380`) — Notes are limited to 4,000 characters; other strings to 500; comments and attachments to 20 each, with explicit pagination/truncation indicators. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:173` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I376 (`docs/contract-177-inventory.md:381`) — Unavailable or malformed data stays unknown, never empty success. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:175` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I377 (`docs/contract-177-inventory.md:382`) — API responses are capped at 256 KiB each and the entire read at ten seconds within the original turn deadline. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:176` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
| I378 (`docs/contract-177-inventory.md:383`) — External content is untrusted source material. | [#159](https://github.com/taskqorg/pilotq/issues/159) / Source #159 | `docs/integration-contract.md:176` | `src/asana.rs:25` | `src/asana.rs:128` | met | Retained; named deterministic protection covers the domain risk. |
### DEV visibility and owner release (task #174)

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I379 (`docs/contract-177-inventory.md:384`) — **Owner, 2026-10-30:** PilotQ owns the registered project's DEV/LIVE flow. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:181` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I380 (`docs/contract-177-inventory.md:385`) — This is a scoped exception to §2's deployment delegation and §5's direct-write restriction, not a general shell capability. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:182` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I381 (`docs/contract-177-inventory.md:386`) — `project_release_preview` and `/release NAME` share current admission, project scope, the separate `release:NAME` grant and the protected installation owner identity (R13, R17). | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:183` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I382 (`docs/contract-177-inventory.md:387`) — This grant is provisioned explicitly through the protected CLI, never inherited from `projects` or bootstrap. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:185` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I383 (`docs/contract-177-inventory.md:388`) — Only a Release button authorizes execution. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:186` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I384 (`docs/contract-177-inventory.md:389`) — CLI preview delivers that button to the protected owner DM; plain text, including `/release-confirm`, cannot authorize a release. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:187` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I385 (`docs/contract-177-inventory.md:390`) — The protected CLI `pilotq projects release NAME FILE` saves deployment configuration against that registration's exact board/root. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:189` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I386 (`docs/contract-177-inventory.md:391`) — Configuration declares the local TaskQ host identity, an HTTPS DEV health URL, main/LIVE refs, absolute Git executable, exact argument arrays for dry run, pinned release and pipeline check, migration path prefixes and preflight/backup commands. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:191` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I387 (`docs/contract-177-inventory.md:392`) — Commands contain a literal `{sha}` argument replaced only with a validated full SHA; no shell or model-supplied command is accepted. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:193` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I388 (`docs/contract-177-inventory.md:393`) — Configuration changes invalidate existing previews. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:194` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I389 (`docs/contract-177-inventory.md:394`) — Missing registration/configuration, changed bindings, unavailable board or checkout evidence and missing grants refuse before deployment commands run. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:195` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I390 (`docs/contract-177-inventory.md:395`) — Status checks the declared DEV health JSON (`sha`, optionally `deployed_at` or health-instance `started_at`). | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:198` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I391 (`docs/contract-177-inventory.md:396`) — A task's merged SHA comes from TaskQ's explicit `merged_sha` field or a trusted terminal `delivery … as SHA; merge-result CI green` receipt, never from prose or absence in an open snapshot. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:200` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I392 (`docs/contract-177-inventory.md:397`) — Git ancestry proves inclusion. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:179` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I393 (`docs/contract-177-inventory.md:398`) — Report `on DEV since T` when included and the health timestamp is available; otherwise `on DEV since T (first observed)` uses the persisted first positive observation, without claiming actual deployment time. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:206` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I394 (`docs/contract-177-inventory.md:399`) — Report `not yet on DEV` for a proven negative, and unknown for failed health, missing commit or failed Git reads. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:208` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I395 (`docs/contract-177-inventory.md:400`) — Project status includes the same projection for its open tasks, the latest twelve retained task subscriptions and the current main SHA. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:209` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I396 (`docs/contract-177-inventory.md:401`) — Health-instance start and first-observed times are lower bounds, not a reconstructed deployment history. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:210` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I397 (`docs/contract-177-inventory.md:402`) — HTTPS requests have no redirects, a ten-second deadline and a 64 KiB response limit; health URLs are protected configuration, never derived from chat. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:212` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I398 (`docs/contract-177-inventory.md:403`) — Preview runs the configured dry run, pins main and LIVE to full commit identities, and shows target SHA, commit count/authors since LIVE, verified TaskQ task URLs with explicit merged SHAs in that range, migrations yes/no and DEV inclusion. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:218` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I399 (`docs/contract-177-inventory.md:404`) — Missing facts refuse preview rather than showing empty success. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:220` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I400 (`docs/contract-177-inventory.md:405`) — A preview expires after fifteen minutes, supersedes earlier previews in that conversation and binds owner, generation, registration, configuration and source card. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:221` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I401 (`docs/contract-177-inventory.md:406`) — Foreign, stale, repeated or reset confirmations run nothing. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:223` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I402 (`docs/contract-177-inventory.md:407`) — Main advancing does not retarget a preview; LIVE advancing requires a fresh preview. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:223` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I403 (`docs/contract-177-inventory.md:408`) — Before preview and again before dispatch, read every explicitly registered checkout on this host through TaskQ. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:226` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I404 (`docs/contract-177-inventory.md:409`) — A running execution on another checkout with the same Git origin blocks release, naming project, task and attempt; unavailable or incomplete evidence blocks rather than assuming idle. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:228` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I405 (`docs/contract-177-inventory.md:410`) — Do not discover other directories. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:229` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I406 (`docs/contract-177-inventory.md:411`) — The protected configuration names the only release root; never use Tight's checkout for cs. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:229` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I407 (`docs/contract-177-inventory.md:412`) — **Owner answer #174:** when migrations are present, Release also authorizes the configured promotion preflight and LIVE backup, in that order; both must succeed before the pinned release command. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:232` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I408 (`docs/contract-177-inventory.md:413`) — Persist one immutable confirmation intent before any command, then each stage receipt. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:234` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I409 (`docs/contract-177-inventory.md:414`) — Execute without a shell in the registered root only. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:234` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I410 (`docs/contract-177-inventory.md:415`) — Release execution runs outside the model turn, under a fifteen-minute total deadline and a repository lock; planned update drain refuses while it is active. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:236` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I411 (`docs/contract-177-inventory.md:416`) — Revoked access stops remaining stages. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:179` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I412 (`docs/contract-177-inventory.md:417`) — An interrupted/timeout/ambiguous command stays uncertain and is never automatically replayed. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:237` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I413 (`docs/contract-177-inventory.md:418`) — Repeated confirmation cannot repeat prerequisites or release. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:238` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I414 (`docs/contract-177-inventory.md:419`) — Pipeline readback must bind the pinned SHA and report `release`, `smoke`, `backmerge` stages with validated HTTPS links and the LIVE SHA; only all successful stages and matching LIVE SHA prove completion. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:239` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I415 (`docs/contract-177-inventory.md:420`) — Missing or pending evidence stays unknown. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:241` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I416 (`docs/contract-177-inventory.md:421`) — Startup preserves intents and reports uncertain interrupted operations through the existing outbox. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:242` | `src/release.rs:401` | `src/release.rs:687` | met | Retained; named deterministic protection covers the domain risk. |
| I417 (`docs/contract-177-inventory.md:422`) — For cs, DEV is `https://dev.cyprus.bz/__health`; commands are `npm run release -- --dry-run`, `npm run release -- {sha}`, `npm run db:promotion:preflight` and `npm run db:live:backup`. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:244` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I418 (`docs/contract-177-inventory.md:423`) — The installation owner supplies the pipeline read adapter and executable paths. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:246` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I419 (`docs/contract-177-inventory.md:424`) — Every executable declares a minimum version in the registration `requirements` map and supports `--version`; Git requires at least 2.25.0. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:247` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I420 (`docs/contract-177-inventory.md:425`) — Doctor checks these alongside §9 host requirements. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:248` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I421 (`docs/contract-177-inventory.md:426`) — That adapter returns JSON `{ "sha": "&lt;pinned SHA&gt;", "live_sha": "&lt;SHA&gt;", "stages": { "release": {"state":"success","url":"https://…"}, "smoke": {…}, "backmerge": {…} } }`; dry run remains the project's existing command. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:250` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I422 (`docs/contract-177-inventory.md:427`) — This does not implement a forge adapter in PilotQ. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:251` | `src/release.rs:312` | `src/release.rs:662` | met | Retained; named deterministic protection covers the domain risk. |
| I423 (`docs/contract-177-inventory.md:428`) — PM qualifies the next owner-requested cs release on PDS 18 after release; worker evidence is local/CI only. | [#174](https://github.com/taskqorg/pilotq/issues/174) / Source #174 | `docs/integration-contract.md:252` | `src/release.rs:312` | `src/release.rs:662` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
### Installed read commands

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I424 (`docs/contract-177-inventory.md:429`) — **Owner, task #126 (option 1):** add one common executor for operator-approved installed read commands, alongside the existing tools. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:256` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I425 (`docs/contract-177-inventory.md:430`) — This extends the §3/§12 restricted tool boundary without enabling native shell tools or adding direct writes. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:257` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I426 (`docs/contract-177-inventory.md:431`) — The installation owner maintains an owner-only `commands.json` beside `keys.json`; chat and model output cannot edit it. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:258` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I427 (`docs/contract-177-inventory.md:432`) — **Owner, task #126 continuation:** the documented TaskQ read capability permits `taskq status`, not the unsupported `taskq list`; acceptance requires a successful status run through PilotQ. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:261` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I428 (`docs/contract-177-inventory.md:433`) — There is no built-in default allowlist: the operator installs the protected entry below. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:263` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I429 (`docs/contract-177-inventory.md:434`) — Each named entry binds an absolute executable, exact permitted argument arrays, an absolute working directory, a registered project and board, and explicit permitted user IDs. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:265` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I430 (`docs/contract-177-inventory.md:435`) — The caller needs current `chat` and project grants as well as that entry's permission. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:267` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I431 (`docs/contract-177-inventory.md:436`) — Registration must still match the configured board and working directory at dispatch. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:268` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I432 (`docs/contract-177-inventory.md:437`) — Before execution, the configured TaskQ CLI must return a fresh complete snapshot naming that board; verification and execution share the deadline. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:269` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I433 (`docs/contract-177-inventory.md:438`) — Only trusted read-only programs and argument combinations belong here; executable presence alone grants nothing. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:270` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I434 (`docs/contract-177-inventory.md:439`) — The operator vets their file access, credential use and subprocess behavior; this is a capability list, not an OS sandbox for arbitrary programs. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:271` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I435 (`docs/contract-177-inventory.md:440`) — `command_list` exposes only accessible names, descriptions and permitted arguments. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:274` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I436 (`docs/contract-177-inventory.md:441`) — `command_run` selects one name and exact argument array; it never invokes a shell, searches PATH for the executable, accepts environment overrides or changes the working directory from model input. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:275` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I437 (`docs/contract-177-inventory.md:442`) — Both share the CLI/Telegram request, fence, five-call budget and audit path (R13, R17). | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:277` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I438 (`docs/contract-177-inventory.md:443`) — Direct command input and clear natural-language requests use the same tools; ambiguity requires clarification. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:278` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I439 (`docs/contract-177-inventory.md:444`) — Output is untrusted data and cannot authorize another action. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:279` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I440 (`docs/contract-177-inventory.md:445`) — Execution receives a cleared environment with a fixed system PATH, the installation owner's HOME for the installed tools' protected stores, and TASKQ_ROOT set to the bound root; TASKQ_ATTEMPT and inherited credentials are absent. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:281` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I441 (`docs/contract-177-inventory.md:446`) — No key values enter the model. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:283` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I442 (`docs/contract-177-inventory.md:447`) — A command has at most ten seconds within the original manager deadline (at most sixty seconds total), and at most 16 KiB combined stdout/stderr. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:284` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I443 (`docs/contract-177-inventory.md:448`) — Stop, revoked grants, timeout and output overflow terminate its process group. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:285` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I444 (`docs/contract-177-inventory.md:449`) — Report exit code, missing executable, timeout, cancellation and output limit distinctly; only exit zero is success. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:286` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I445 (`docs/contract-177-inventory.md:450`) — Persist an audit start before spawn and a scrubbed result afterward. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:287` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I446 (`docs/contract-177-inventory.md:451`) — An interrupted read may be retried only under §4; mutations remain on their existing receipt paths. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:288` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
| I447 (`docs/contract-177-inventory.md:452`) — Deployment is not authorized by this decision. | [#126](https://github.com/taskqorg/pilotq/issues/126) / Source #126 | `docs/integration-contract.md:289` | `src/commands.rs:52` | `src/commands.rs:193` | met | Retained; named deterministic protection covers the domain risk. |
### Administration by request (task #160)

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I448 (`docs/contract-177-inventory.md:453`) — **Owner, 2026-10-30:** expose these tools on the shared CLI/Telegram manager boundary (R13, R15, R17). | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:293` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I449 (`docs/contract-177-inventory.md:454`) — They are an exception to section 5's owner-only comment/label writes, limited to installation metadata, local project roots and forum-topic creation. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:294` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I450 (`docs/contract-177-inventory.md:455`) — They never create repositories, change host capacity, install services, provision credentials or create a TaskQ claim issue. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:296` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I451 (`docs/contract-177-inventory.md:456`) — &#124; Tool &#124; Required grants &#124; Operation and limits &#124; | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:298` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I452 (`docs/contract-177-inventory.md:457`) — &#124; `project_register` &#124; `chat`, `projects` &#124; Register an existing repository by its explicit TaskQ board kind, host and repository. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:300` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I453 (`docs/contract-177-inventory.md:458`) — Create only a named child of `&lt;install&gt;/projects` with `taskq.json`; refuse traversal, symlinks, conflicting configuration or rebinding an existing name. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:300` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I454 (`docs/contract-177-inventory.md:459`) — A fresh complete `taskq status --json` must match before registration. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I455 (`docs/contract-177-inventory.md:460`) — Return repository, root, board and filing readiness; without `create_claim_issue`, filing remains disabled and the owner provisions it through TaskQ. &#124; | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I456 (`docs/contract-177-inventory.md:461`) — &#124; `user_grant`, `user_revoke` &#124; `chat`, `users` &#124; Change only existing grant types explicitly named in the caller’s own unquoted request for an exact positive Telegram ID explicitly named by the caller or identified by the current verified reply. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:301` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I457 (`docs/contract-177-inventory.md:462`) — Require evidence that the person was seen in this exact chat/topic: an admitted sender or a Telegram reply sender. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I458 (`docs/contract-177-inventory.md:463`) — Names and forwarded messages never select authority. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:314` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I459 (`docs/contract-177-inventory.md:464`) — Retain the administrator, target and before/after grants atomically with the receipt. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:306` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I460 (`docs/contract-177-inventory.md:465`) — Protected owner identity cannot be changed. &#124; | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I461 (`docs/contract-177-inventory.md:466`) — &#124; `topic_open` &#124; `chat`, `users`, access to the bound project (`projects` or `project:NAME`) &#124; Open one forum topic in the originating Telegram supergroup, or the explicitly named group for a CLI request. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:302` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I462 (`docs/contract-177-inventory.md:467`) — Resolve one registered project named by the caller or current in the conversation; recheck its board/root. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I463 (`docs/contract-177-inventory.md:468`) — Require a fresh bot `can_manage_topics` permission and forum capability; otherwise refuse plainly. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I464 (`docs/contract-177-inventory.md:469`) — Set isolated route policy by default (shared only when explicitly requested), and bind the new route to that project as its default. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:291` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I465 (`docs/contract-177-inventory.md:470`) — A route default never grants project access. &#124; | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:298` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I466 (`docs/contract-177-inventory.md:471`) — Each tool shares the five-call limit, original deadline, current admission, run fence and request audit. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/integration-contract.md:304` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I467 (`docs/contract-177-inventory.md:472`) — Store one immutable operation per tool per request; a changed argument set in that request is refused. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:305` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I468 (`docs/contract-177-inventory.md:473`) — Read back local changes before success. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:306` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I469 (`docs/contract-177-inventory.md:474`) — Duplicate calls return the saved receipt without applying the change again. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:306` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I470 (`docs/contract-177-inventory.md:475`) — Project preparation persists intent before filesystem writes and can reconcile only the same matching configuration. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:307` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I471 (`docs/contract-177-inventory.md:476`) — Topic creation has no Bot API idempotency key: persist intent before dispatch and its topic ID immediately on response. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:309` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I472 (`docs/contract-177-inventory.md:477`) — An interrupted creation without that ID is uncertain and is never automatically repeated; an ID recorded before route binding permits completing only that local binding after grant revalidation. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:310` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I473 (`docs/contract-177-inventory.md:478`) — Startup preserves unresolved operations and reports their partial or uncertain result. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:312` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I474 (`docs/contract-177-inventory.md:479`) — Unknown senders still follow R10; the service does not retain ignored message bodies or build profiles to make them grant candidates. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:313` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
| I475 (`docs/contract-177-inventory.md:480`) — An administrator may reply to that person's message to supply the verified sender reference. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:314` | `src/admin.rs:26` | `src/admin.rs:433` | met | Retained; named deterministic protection covers the domain risk. |
### Project registration

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I476 (`docs/contract-177-inventory.md:481`) — An administrator with the `projects` grant can ask PilotQ to create or register a project. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:319` | `src/admin.rs:184` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
| I477 (`docs/contract-177-inventory.md:482`) — PilotQ shows the resolved repository, root and board, then applies the request through TaskQ's supported registration path (Add project, [taskq.md](https://github.com/taskqorg/taskq/blob/main/taskq.md) menu M10) and reads it back. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:320` | `src/admin.rs:184` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
| I478 (`docs/contract-177-inventory.md:483`) — Repository creation, capacity and worker grants stay owner operations unless a separate grant names them. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:322` | `src/admin.rs:184` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
| I479 (`docs/contract-177-inventory.md:484`) — A failed or partial registration stays visible and is never reported as done. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:323` | `src/admin.rs:184` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
| I480 (`docs/contract-177-inventory.md:485`) — Step 4 registers through the CLI: `pilotq projects add NAME ROOT` accepts a root whose `taskq.json` names a repository only when a fresh `taskq status --json` under that root reads back the same board, and records who registered it. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:326` | `src/admin.rs:184` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
| I481 (`docs/contract-177-inventory.md:486`) — TaskQ's Add project conversation stays the owner's way to create the TaskQ side; #160 additionally prepares a local root for an existing repository, without claiming filing readiness. | [#160](https://github.com/taskqorg/pilotq/issues/160) / Source #160 | `docs/integration-contract.md:329` | `src/admin.rs:184` | `src/admin.rs:544` | met | Retained; named deterministic protection covers the domain risk. |
### Project onboarding (task #168)

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I482 (`docs/contract-177-inventory.md:487`) — **Owner, task #168, registered-roots answer:** onboarding covers only explicitly registered roots (§3). | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:334` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I483 (`docs/contract-177-inventory.md:488`) — Successful `pilotq projects add` and `project_register` queue onboarding; service startup queues registrations without onboarding records, and a changed TaskQ installation/service identity queues a recheck of those roots. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:335` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I484 (`docs/contract-177-inventory.md:489`) — A new TaskQ project must first be registered explicitly. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:337` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I485 (`docs/contract-177-inventory.md:490`) — No directory discovery or silent enrollment is added. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:337` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I486 (`docs/contract-177-inventory.md:491`) — Registration and readiness are separate. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:340` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I487 (`docs/contract-177-inventory.md:492`) — After registration run `pilotq doctor --json` and the TaskQ setup check; until taskq#1938 ships use `taskq service status --json`, matching the registered root and board. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:341` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I488 (`docs/contract-177-inventory.md:493`) — Require complete evidence with service and per-root observations no older than 120 seconds (at most five seconds in the future), an active service, usable Codex capacity and an explicit effective Codex grant without refusal, plus the existing claim prerequisite. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:343` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I489 (`docs/contract-177-inventory.md:494`) — A capacity limit such as 4 never substitutes for a grant. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:345` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I490 (`docs/contract-177-inventory.md:495`) — Unavailable checks stay gaps. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:345` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I491 (`docs/contract-177-inventory.md:496`) — Setup evidence is not proof of the full first-task scenario or subscription entitlement. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:346` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I492 (`docs/contract-177-inventory.md:497`) — **Owner, #168 continuation:** check that each registered project has an `issue_read` key grant usable by the protected owner through both CLI and DM. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:348` | `src/onboarding.rs:109` | `src/onboarding.rs:351` | met | Retained; named deterministic protection covers the domain risk. |
| I493 (`docs/contract-177-inventory.md:498`) — Project grants and existing owner-wide grants retain §8 semantics; paired CLI/owner-DM route grants also qualify. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:349` | `src/onboarding.rs:109` | `src/onboarding.rs:351` | met | Retained; named deterministic protection covers the domain risk. |
| I494 (`docs/contract-177-inventory.md:499`) — Read key metadata only, never return values or update last-use time. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:350` | `src/onboarding.rs:109` | `src/onboarding.rs:351` | met | Retained; named deterministic protection covers the domain risk. |
| I495 (`docs/contract-177-inventory.md:500`) — If suitable issue-tool keys exist but lack scope, show the exact `pilotq keys grant KEY project:NAME` command for each owner-selectable candidate; do not grant automatically. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:352` | `src/onboarding.rs:109` | `src/onboarding.rs:351` | met | Retained; named deterministic protection covers the domain risk. |
| I496 (`docs/contract-177-inventory.md:501`) — Missing or unreadable key configuration remains a gap with a named protected-CLI provisioning step. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:353` | `src/onboarding.rs:109` | `src/onboarding.rs:351` | met | Retained; named deterministic protection covers the domain risk. |
| I497 (`docs/contract-177-inventory.md:502`) — Keep the 2026-10-30 fixture: `pilotq-github` permits `issue_read` but is granted to `pilotq` alone; `cyprusbz`, `tight` and `agentq` must each report their missing grant. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:354` | `src/onboarding.rs:109` | `src/onboarding.rs:351` | met | Retained; named deterministic protection covers the domain risk. |
| I498 (`docs/contract-177-inventory.md:503`) — The protected owner receives one durable Telegram DM card per project. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:357` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I499 (`docs/contract-177-inventory.md:504`) — It lists each gap on one plain line with its exact fix command or a named owner step, and offers Recheck. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:358` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I500 (`docs/contract-177-inventory.md:505`) — Rechecks edit that card; missing cards may be replaced only in the same DM. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:359` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I501 (`docs/contract-177-inventory.md:506`) — CLI registration prints the same findings and returns nonzero when gaps remain, preserving the registered project. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:360` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I502 (`docs/contract-177-inventory.md:507`) — `/onboard NAME` rechecks through the bounded control path; the CLI equivalent is `pilotq ask '/onboard NAME'`. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:361` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I503 (`docs/contract-177-inventory.md:508`) — Controls share current admission, audit and persisted Telegram cooldowns (R10, R13, R17). | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:362` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I504 (`docs/contract-177-inventory.md:509`) — Buttons bind to the owner, card, current registration and conversation generation; stale or foreign buttons receive only a toast. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:363` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I505 (`docs/contract-177-inventory.md:510`) — Existing §5/§9 authority remains unchanged: installation, capacity, worker grants, credentials and subscription login are owner-only steps, shown with Recheck rather than executable Fix buttons. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:366` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I506 (`docs/contract-177-inventory.md:511`) — Any fix within PilotQ's existing authority requires one explicit OK for its displayed scope and the existing durable intent/readback path; this flow adds no host mutation capability. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:368` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I507 (`docs/contract-177-inventory.md:512`) — The flow ends either with all checks green or with the named owner-only steps still outstanding. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:370` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I508 (`docs/contract-177-inventory.md:513`) — It never calls a project ready with gaps. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:371` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I509 (`docs/contract-177-inventory.md:514`) — Before PilotQ starts the first task of an onboarding project, require a passed report bound to its current root and board. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:373` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I510 (`docs/contract-177-inventory.md:515`) — Pending, failed or unknown onboarding leaves the launch unsent. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:374` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I511 (`docs/contract-177-inventory.md:516`) — The protected owner may write `/onboard NAME override` to bypass only this PilotQ readiness gate; record actor, target and findings, keep readiness false and retain TaskQ's own admission, grant and claim checks. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:375` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I512 (`docs/contract-177-inventory.md:517`) — Recheck or changed registration invalidates the override. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:377` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I513 (`docs/contract-177-inventory.md:518`) — Existing admitted workers continue. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:366` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I514 (`docs/contract-177-inventory.md:519`) — Onboarding and its report survive restart without replaying mutations. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:378` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I515 (`docs/contract-177-inventory.md:520`) — An operations board is an explicit server-care project. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:380` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I516 (`docs/contract-177-inventory.md:521`) — Project work stays on that project's board. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:380` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I517 (`docs/contract-177-inventory.md:522`) — A server-wide overview aggregates tasks; it does not duplicate every issue onto a server board. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:381` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I518 (`docs/contract-177-inventory.md:523`) — TaskQ execution continues when PilotQ is down. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:383` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I519 (`docs/contract-177-inventory.md:524`) — If TaskQ is unreachable, conversations continue and proposed tasks remain unsent. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:383` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I520 (`docs/contract-177-inventory.md:525`) — Status shows the last observation time and unknown freshness. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:384` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
| I521 (`docs/contract-177-inventory.md:526`) — Previously authorized, unchanged submissions can resume when dependencies recover; changed scope or stale grants require revalidation. | [#168](https://github.com/taskqorg/pilotq/issues/168) / Source #168 | `docs/integration-contract.md:385` | `src/onboarding.rs:132` | `src/onboarding.rs:387` | met | Retained; named deterministic protection covers the domain risk. |
### 6. Memory and agentq

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I522 (`docs/contract-177-inventory.md:527`) — **Proposed:** conversation history, remembered preferences and authoritative task facts are distinct. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:146` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I523 (`docs/contract-177-inventory.md:528`) — Store route/project preferences with author, scope, source and update time. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:147` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I524 (`docs/contract-177-inventory.md:529`) — “Usually this project” is a hint; tool invocation still resolves the target. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:148` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I525 (`docs/contract-177-inventory.md:530`) — Explicit corrections supersede old preferences. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:148` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I526 (`docs/contract-177-inventory.md:531`) — Shared memory never includes private conversation content without authorization. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:149` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I527 (`docs/contract-177-inventory.md:532`) — Maintain durable per-person profiles for preferred name, relevant working preferences and explicitly remembered project associations. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:151` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I528 (`docs/contract-177-inventory.md:533`) — Each fact carries its subject user ID, visibility scope, source and revision. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:152` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I529 (`docs/contract-177-inventory.md:534`) — Load only facts permitted in the current route; remembering one person across chats does not copy private DM history into a group. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:153` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I530 (`docs/contract-177-inventory.md:535`) — Profiles survive restart and conversation reset. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:154` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I531 (`docs/contract-177-inventory.md:536`) — Provide a Telegram flow to inspect, correct and forget remembered facts. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:156` | `src/journal.rs:1444` | `src/journal.rs:3386` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I532 (`docs/contract-177-inventory.md:537`) — Users can manage their own non-authoritative preferences; administrators can correct shared profile information within their grant. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:157` | `src/journal.rs:1444` | `src/journal.rs:3386` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I533 (`docs/contract-177-inventory.md:538`) — Administration does not implicitly reveal another person's private DM memory. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:158` | `src/journal.rs:1444` | `src/journal.rs:3386` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I534 (`docs/contract-177-inventory.md:539`) — Show the applied change and persist it before claiming success. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:159` | `src/journal.rs:1444` | `src/journal.rs:3386` | missing | Journal/CLI protection exists; scoped Telegram self-service or access inspection is missing. Follow-up [#196](https://github.com/taskqorg/pilotq/issues/196). |
| I535 (`docs/contract-177-inventory.md:540`) — Corrected or forgotten facts must not reappear as current facts from an old summary, restored session or automatic extraction. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:160` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I536 (`docs/contract-177-inventory.md:541`) — Profile deletion and correction are distinct from retention of conversation/audit records; do not claim that forgetting a preference erases every historical record. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:161` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I537 (`docs/contract-177-inventory.md:542`) — Access rights live in the grant registry and cannot be changed by editing a profile. | [#5](https://github.com/taskqorg/pilotq/issues/5) / Source #5 | `docs/runtime-contract.md:163` | `src/journal.rs:1444` | `src/journal.rs:3386` | met | Retained; named deterministic protection covers the domain risk. |
| I538 (`docs/contract-177-inventory.md:543`) — Keep these outside lossy summaries: identities and roles, grants and revocations, active project references, request/operation IDs, task URLs, pending questions, plan revisions, explicit prohibitions and verified effect receipts. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:165` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I539 (`docs/contract-177-inventory.md:544`) — Fresh TaskQ reads supply task state. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:167` | `src/manager.rs:183` | `tests/telegram.rs:2333` | met | Retained; named deterministic protection covers the domain risk. |
| I540 (`docs/contract-177-inventory.md:545`) — Start with native runtime compaction as the functional baseline. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:169` | `src/manager.rs:183` | `tests/telegram.rs:2333` | met | Retained; named deterministic protection covers the domain risk. |
| I541 (`docs/contract-177-inventory.md:546`) — Integrate agentq at the selected runtime's supported tool-result/context boundary. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:170` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I542 (`docs/contract-177-inventory.md:547`) — Tool-result compression and conversation summarization are separate stages with one owner each; never let two compactors race or compress the same result repeatedly. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:171` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I543 (`docs/contract-177-inventory.md:548`) — Compression preserves tool-call/result associations, errors, evidence locations and original-data recovery references. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:174` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I544 (`docs/contract-177-inventory.md:549`) — Recovery authorizes the requesting route before returning content. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:175` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I545 (`docs/contract-177-inventory.md:550`) — On compression failure, use original data only if it fits the context budget; otherwise fetch a bounded relevant portion or surface a context limit. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:176` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I546 (`docs/contract-177-inventory.md:551`) — Never silently truncate authorization or replay work to reconstruct a missing result. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:177` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I547 (`docs/contract-177-inventory.md:552`) — Qualification compares identical scenario sets with agentq off and on. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:180` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I548 (`docs/contract-177-inventory.md:553`) — Measure correct completion, preserved constraints, exact original recovery, provider input/output/cache tokens where available, latency and interventions. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:181` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I549 (`docs/contract-177-inventory.md:554`) — Byte reduction alone is not a savings claim. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:182` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I550 (`docs/contract-177-inventory.md:555`) — A session that survives one compression is not evidence of a long conversation remaining correct. | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/runtime-contract.md:183` | `src/manager.rs:183` | `tests/telegram.rs:2333` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
### 7. Proposed operating budgets

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I551 (`docs/contract-177-inventory.md:556`) — These are initial acceptance targets, not observed performance or a production SLA. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:188` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I552 (`docs/contract-177-inventory.md:557`) — Tune only from recorded pilot evidence. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:189` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I553 (`docs/contract-177-inventory.md:558`) — &#124; Concern &#124; Initial target &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:191` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I554 (`docs/contract-177-inventory.md:559`) — &#124; Durable receipt &#124; p95 &lt;= 2 seconds from update receipt at PilotQ, under the defined pilot load &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:193` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I555 (`docs/contract-177-inventory.md:560`) — &#124; Telegram acknowledgement &#124; 👀 reaction within 5 seconds when Telegram is reachable; text fallback on reaction failure, one queue notice after 30 seconds &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:194` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I556 (`docs/contract-177-inventory.md:561`) — &#124; Direct action &#124; &lt;= 60 seconds and &lt;= 5 tool calls; longer work is delegated or returned with a bounded result &#124; | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:195` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I557 (`docs/contract-177-inventory.md:562`) — &#124; Silent runtime &#124; No useful event for 30 seconds triggers diagnosis; runtime deadline still applies even if it emits tokens indefinitely &#124; | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:198` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I558 (`docs/contract-177-inventory.md:563`) — &#124; Cancellation &#124; Acknowledge within 5 seconds when transport is healthy; abort then terminate the isolated process after a 5-second grace period if necessary &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:197` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I559 (`docs/contract-177-inventory.md:564`) — &#124; Runtime recovery &#124; Reconcile interrupted requests within 30 seconds after service readiness; uncertain effects stay blocked &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:198` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I560 (`docs/contract-177-inventory.md:565`) — &#124; Model concurrency &#124; 4 conversations initially, one active run per conversation; round-robin waiting routes &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:199` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I561 (`docs/contract-177-inventory.md:566`) — &#124; Backlog &#124; 50 pending requests per route, 1,000 total; excess gets a recorded busy response, never an untracked promise &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:200` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I562 (`docs/contract-177-inventory.md:567`) — &#124; Dependency retry &#124; At most 3 safe model/read attempts within the original deadline; no retry multiplication across SDK and coordinator &#124; | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:201` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I563 (`docs/contract-177-inventory.md:568`) — &#124; Delivery retry &#124; At most 5 attempts per episode, persisted across restart; obey Telegram retry delays, then surface pending delivery &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:202` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I564 (`docs/contract-177-inventory.md:569`) — &#124; Task notifications &#124; Within 15 seconds of observing a fresh TaskQ transition, while Telegram is reachable &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:203` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I565 (`docs/contract-177-inventory.md:570`) — &#124; Health &#124; Separate last successful poll, storage commit, model event, TaskQ observation and delivery; expose age and unknown state &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:204` | `src/journal.rs:1592` | `src/journal.rs:3782` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I566 (`docs/contract-177-inventory.md:571`) — Storage: SQLite WAL with `synchronous=FULL`, one writer, local disk and bounded transactions. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:206` | `src/journal.rs:732` | `src/journal.rs:3624` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I567 (`docs/contract-177-inventory.md:572`) — A healthy process is not evidence of a healthy ingress path. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:207` | `src/journal.rs:732` | `src/journal.rs:3624` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I568 (`docs/contract-177-inventory.md:573`) — Run liveness checks outside the model process; use the existing host monitoring path for an outage that prevents Telegram alerts. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:208` | `src/journal.rs:732` | `src/journal.rs:3624` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I569 (`docs/contract-177-inventory.md:574`) — Proposed retention: raw conversations 30 days; request audit records (section 8) 365 days for metadata and 90 days for message text; inactive native session files 30 days; request/effect receipts 90 days after reconciliation; diagnostic logs 7 days; temporary attachments 7 days with a 20 MiB per-file cap. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:212` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I570 (`docs/contract-177-inventory.md:575`) — Keep active plans, unresolved effects, unanswered questions and undelivered outcomes until resolved. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:213` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I571 (`docs/contract-177-inventory.md:576`) — Keep minimal deduplication/receipt tombstones beyond content pruning where replay remains possible. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:214` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I572 (`docs/contract-177-inventory.md:577`) — Initial PilotQ disk budget: 2 GiB including its journals, sessions and temporary files. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:217` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I573 (`docs/contract-177-inventory.md:578`) — At 80%, prune only eligible data and alert. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:218` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I574 (`docs/contract-177-inventory.md:579`) — At the hard limit, preserve unresolved records, stop new admission and report pressure through the host channel. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:218` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I575 (`docs/contract-177-inventory.md:580`) — Do not advance the Telegram cursor for an input that could not be stored. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:219` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I576 (`docs/contract-177-inventory.md:581`) — This limit can cause a Telegram retention gap during a prolonged outage; report it. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:220` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I577 (`docs/contract-177-inventory.md:582`) — Back up with SQLite's supported snapshot mechanism; prove restoration separately. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:221` | `src/journal.rs:732` | `src/journal.rs:3624` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
### 8. Access and observability

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I578 (`docs/contract-177-inventory.md:583`) — **Proposed:** configure explicit Telegram users/routes and action grants. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:226` | `src/journal.rs:382` | `src/journal.rs:3757` | met | Retained; named deterministic protection covers the domain risk. |
| I579 (`docs/contract-177-inventory.md:584`) — Reading a group or a task does not grant permission to mutate a project. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:227` | `src/journal.rs:382` | `src/journal.rs:3757` | met | Retained; named deterministic protection covers the domain risk. |
| I580 (`docs/contract-177-inventory.md:585`) — Revalidate authorization at effect dispatch and before delayed delivery. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:227` | `src/journal.rs:382` | `src/journal.rs:3757` | met | Retained; named deterministic protection covers the domain risk. |
| I581 (`docs/contract-177-inventory.md:586`) — Revocation cancels pending unauthorized effects but cannot erase an already completed operation. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:228` | `src/journal.rs:382` | `src/journal.rs:3757` | met | Retained; named deterministic protection covers the domain risk. |
| I582 (`docs/contract-177-inventory.md:587`) — Keep credentials in protected host configuration, outside model context and diagnostics. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:231` | `src/journal.rs:382` | `src/journal.rs:3757` | met | Retained; named deterministic protection covers the domain risk. |
| I583 (`docs/contract-177-inventory.md:588`) — The manager receives only tools granted to its caller and route. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:232` | `src/journal.rs:382` | `src/journal.rs:3757` | met | Retained; named deterministic protection covers the domain risk. |
### Key store

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I584 (`docs/contract-177-inventory.md:589`) — **Proposed:** a key is a named secret in one owner-only file under the install directory (mode 0600 or the Windows equivalent). | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:236` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I585 (`docs/contract-177-inventory.md:590`) — Each key records its name, purpose, allowed tools, allowed users or routes and last use; the value is stored once and never returned. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:238` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I586 (`docs/contract-177-inventory.md:591`) — - A tool call names a key; the tool boundary injects the value into the child process environment or request header at dispatch. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:240` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I587 (`docs/contract-177-inventory.md:592`) — - The value never enters model context, the journal, telemetry, logs, replies or task text. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:242` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I588 (`docs/contract-177-inventory.md:593`) — Tool output is scrubbed for known values before the model or Telegram sees it. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:243` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I589 (`docs/contract-177-inventory.md:594`) — - A grant on a key names the person, route or project. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:244` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I590 (`docs/contract-177-inventory.md:595`) — Use is checked at dispatch, like any effect (section 8). | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:244` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I591 (`docs/contract-177-inventory.md:596`) — - Keys are added, granted and revoked only through the CLI (owner decision 2026-10-23). | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:246` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I592 (`docs/contract-177-inventory.md:597`) — Telegram key entry is not built. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:247` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I593 (`docs/contract-177-inventory.md:598`) — - `pilotq keys list` shows names, grants and last use, never values. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:248` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I594 (`docs/contract-177-inventory.md:599`) — A model's tool choice is a request, not enforcement. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:234` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
| I595 (`docs/contract-177-inventory.md:600`) — Downloaded content cannot expand grants, switch repositories or redirect private answers. | [#23](https://github.com/taskqorg/pilotq/issues/23) / Source #23 | `docs/runtime-contract.md:249` | `src/keys.rs:136` | `tests/telegram.rs:1591` | met | Retained; named deterministic protection covers the domain risk. |
### Request audit

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I596 (`docs/contract-177-inventory.md:601`) — **Proposed:** the journal keeps one audit record per request from the CLI or Telegram (R17). | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:254` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I597 (`docs/contract-177-inventory.md:602`) — It is written in the same transaction as the admitted input and updated as the request moves. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:255` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I598 (`docs/contract-177-inventory.md:603`) — - Who: person ID, display name at that time and the grant set used. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:257` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I599 (`docs/contract-177-inventory.md:604`) — When: receive, start, each effect and delivery times. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:258` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I600 (`docs/contract-177-inventory.md:605`) — Where: front end, chat, topic and project. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:258` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I601 (`docs/contract-177-inventory.md:606`) — - What: the request text, its edits and cancellations, and the reply sent. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:259` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I602 (`docs/contract-177-inventory.md:607`) — Key values never appear (section 8 Key store). | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:260` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I603 (`docs/contract-177-inventory.md:608`) — - Done: model, version and agentq mode; each tool call and effect with its target and result; TaskQ tasks filed, answered or followed; the final outcome or failure. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:261` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I604 (`docs/contract-177-inventory.md:609`) — - One request ID joins the audit record, the CoreQ telemetry trace and TaskQ task text, so an investigation can move from a Telegram message to a task attempt and back. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:263` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I605 (`docs/contract-177-inventory.md:610`) — - Readers: the owner and administrators with an `audit` grant read all records through `pilotq audit`; every person can read their own with `/audit` in a direct message, answered without the model like the section 2 controls. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:266` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I606 (`docs/contract-177-inventory.md:611`) — In a group or topic `/audit` shows nothing, since it would expose one person's history to the others. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:267` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I607 (`docs/contract-177-inventory.md:612`) — Reading is itself recorded. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:268` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I608 (`docs/contract-177-inventory.md:613`) — - Use: service investigation, answers to a person about their history, and self-improvement analysis. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:269` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I609 (`docs/contract-177-inventory.md:614`) — Analysis exports leave out message text unless the owner approves a specific export. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:270` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I610 (`docs/contract-177-inventory.md:615`) — - The person learns at admission that their requests are recorded and for how long. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:271` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I611 (`docs/contract-177-inventory.md:616`) — Local correlated events cover update → request → model turn → tool operation → TaskQ task/attempt → Telegram delivery. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:273` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I612 (`docs/contract-177-inventory.md:617`) — The runtime adapter reports actual completion/error events. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:274` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I613 (`docs/contract-177-inventory.md:618`) — Model/provider/version and agentq mode accompany evaluation evidence. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:275` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I614 (`docs/contract-177-inventory.md:619`) — Use the existing Langfuse installation for optional, bounded telemetry export after capture scope is approved. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:277` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I615 (`docs/contract-177-inventory.md:620`) — Content capture is opt-in and redacted; operational metadata is enough for delivery and intervention metrics. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:278` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I616 (`docs/contract-177-inventory.md:621`) — Langfuse failure must not block conversations or become the only copy of a task outcome. | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:279` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
| I617 (`docs/contract-177-inventory.md:622`) — The proposed metric set lives in [acceptance](docs/acceptance.md#metrics). | [#8](https://github.com/taskqorg/pilotq/issues/8) / Source #8 | `docs/runtime-contract.md:280` | `src/journal.rs:595` | `src/journal.rs:3564` | met | Retained; named deterministic protection covers the domain risk. |
### Command line

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I618 (`docs/contract-177-inventory.md:623`) — **Owner, task #106 (option 1):** deterministic built-in commands never enter the model backlog, including while a chat run is active. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:10` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I619 (`docs/contract-177-inventory.md:624`) — They use the same admission and audit boundary as other requests (R10, R13, R17). | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:11` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I620 (`docs/contract-177-inventory.md:625`) — - `/help` lists available controls; `/version` reports this binary's version; `/doctor` runs bounded, read-only setup checks and returns check names and outcomes only. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:13` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I621 (`docs/contract-177-inventory.md:626`) — Unknown commands receive a usage error, not a model turn. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:15` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I622 (`docs/contract-177-inventory.md:627`) — Commands accept Telegram's `@bot` suffix. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:15` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I623 (`docs/contract-177-inventory.md:628`) — - `/model` reads the configured Codex app-server's model catalogue and displays selection buttons; `/model MODEL` selects an exact catalogue name, and `/model default` removes the override. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:16` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I624 (`docs/contract-177-inventory.md:629`) — The catalogue is not proof of subscription entitlement. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:18` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I625 (`docs/contract-177-inventory.md:630`) — Record the requested model separately from the last model reported by Codex. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:18` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I626 (`docs/contract-177-inventory.md:631`) — An unavailable catalogue refuses selection without changing the saved choice. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:19` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I627 (`docs/contract-177-inventory.md:632`) — - A model choice belongs to the existing isolated or shared conversation, requires current `chat` admission, survives restart and `/new`, and applies only to runs claimed after the change. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:21` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I628 (`docs/contract-177-inventory.md:633`) — Buttons bind to the displaying person, conversation generation and latest selector. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:23` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I629 (`docs/contract-177-inventory.md:634`) — Active runs keep their claimed choice. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:20` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I630 (`docs/contract-177-inventory.md:635`) — No automatic model or paid-provider fallback is allowed (R9). | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:24` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I631 (`docs/contract-177-inventory.md:636`) — - Unset choices inherit the isolated Codex configuration. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:25` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I632 (`docs/contract-177-inventory.md:637`) — TaskQ worker defaults, inheritance, overrides and host resource admission stay with TaskQ; PilotQ never passes its manager choice to workers or changes running tasks. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:26` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I633 (`docs/contract-177-inventory.md:638`) — Model selection does not change the four-conversation budget (§7). | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:27` | `src/journal.rs:939` | `src/journal.rs:4531` | met | Retained; named deterministic protection covers the domain risk. |
| I634 (`docs/contract-177-inventory.md:639`) — - `/restart` is owner-only. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:29` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I635 (`docs/contract-177-inventory.md:640`) — The installed service acknowledges immediately, drains through the existing §9 path, and reports completion only after the restarted service writes healthy status. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:30` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I636 (`docs/contract-177-inventory.md:641`) — A failed drain resumes admission and reports failure. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:31` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I637 (`docs/contract-177-inventory.md:642`) — Source runs refuse service restart. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:31` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I638 (`docs/contract-177-inventory.md:643`) — Requests, task subscriptions and delivery receipts survive. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:32` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I639 (`docs/contract-177-inventory.md:644`) — Duplicate updates cannot restart twice. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:32` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I640 (`docs/contract-177-inventory.md:645`) — - `/skills` lists enabled skills; `/skill NAME TEXT` is the invocation syntax. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:33` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I641 (`docs/contract-177-inventory.md:646`) — No native skills are currently approved for the restricted manager. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/service-contract.md:34` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I642 (`docs/contract-177-inventory.md:647`) — Both commands explain this without starting a model or enabling native tools; adding an approved skill requires a contract decision under §12. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:35` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I643 (`docs/contract-177-inventory.md:648`) — - The operator can send the same controls through `pilotq ask '/model MODEL'` (R13). | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:36` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I644 (`docs/contract-177-inventory.md:649`) — Service restart and setup checks keep their own bounded control worker, independent of chat generation. | [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106 | `docs/service-contract.md:37` | `src/builtins.rs:109` | `src/journal.rs:4503` | met | Retained; named deterministic protection covers the domain risk. |
| I645 (`docs/contract-177-inventory.md:650`) — **Proposed:** the `pilotq` binary is the service and the operator CLI. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:39` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I646 (`docs/contract-177-inventory.md:651`) — The CLI uses the same journal, grants and audit records as Telegram; the local operator acts as the bootstrap owner identity from protected configuration. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:40` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I647 (`docs/contract-177-inventory.md:652`) — &#124; Command &#124; Purpose &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:43` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I648 (`docs/contract-177-inventory.md:653`) — &#124; `pilotq ask TEXT [--project P]` &#124; One request through the normal manager path; prints the reply or task link &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:45` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I649 (`docs/contract-177-inventory.md:654`) — &#124; `pilotq status [--json]` &#124; Health ages (section 7), pending requests, undelivered outcomes, uncertain effects &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:46` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I650 (`docs/contract-177-inventory.md:655`) — &#124; `pilotq users list\&#124;show\&#124;grant\&#124;revoke\&#124;remember\&#124;forget` &#124; People, profiles and grants (R10, R15); `remember`/`forget` correct one profile fact &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:47` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I651 (`docs/contract-177-inventory.md:656`) — &#124; `pilotq routes list\&#124;set` &#124; Chats, topics and their session and reply policy &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:48` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I652 (`docs/contract-177-inventory.md:657`) — &#124; `pilotq projects list\&#124;add` &#124; Registered projects and the section 5 registration path &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:49` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I653 (`docs/contract-177-inventory.md:658`) — &#124; `pilotq keys list\&#124;add\&#124;grant\&#124;revoke` &#124; Key store; `add` reads the value from stdin &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:50` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I654 (`docs/contract-177-inventory.md:659`) — &#124; `pilotq pilot start\&#124;note\&#124;report [--since T] [--json]` &#124; Seven-day pilot start, intervention and outage notes, gate 4 counts (section 10 step 5) &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:51` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I655 (`docs/contract-177-inventory.md:660`) — &#124; `pilotq audit [--user U] [--route R] [--project P] [--since T] [--request ID] [--json]` &#124; Request audit records (section 8, R17), filtered by who, where, what and when &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:52` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I656 (`docs/contract-177-inventory.md:661`) — &#124; `pilotq service install\&#124;run\&#124;stop\&#124;status` &#124; coreq service unit &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:53` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I657 (`docs/contract-177-inventory.md:662`) — &#124; `pilotq update [--check&#124;--pause&#124;--resume]`, `pilotq selftest`, `pilotq doctor` &#124; coreq update, smoke test and setup check (bot identity, Codex login, TaskQ reachability) &#124; | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:54` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I658 (`docs/contract-177-inventory.md:663`) — Every command that writes prints the readback, supports `--json` and returns a nonzero exit code on failure. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/service-contract.md:56` | `src/main.rs:639` | `tests/telegram.rs:677` | met | Retained; named deterministic protection covers the domain risk. |
| I659 (`docs/contract-177-inventory.md:664`) — `pilotq doctor [--json]` performs read-only setup checks: Telegram `getMe` returns a bot identity, the configured isolated Codex home reports a ChatGPT login, and TaskQ returns a complete snapshot matching its configured board. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:59` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I660 (`docs/contract-177-inventory.md:665`) — It never polls updates, starts a model turn or changes grants. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:61` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I661 (`docs/contract-177-inventory.md:666`) — Missing configuration, failed checks and unavailable dependencies return nonzero; diagnostics expose only check names and outcomes, never dependency output or credentials. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:62` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I662 (`docs/contract-177-inventory.md:667`) — This checks setup, not the full task flow or subscription entitlement (§11). | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:64` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I663 (`docs/contract-177-inventory.md:668`) — **Owner, task #120:** also check local voice readiness. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:64` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I664 (`docs/contract-177-inventory.md:669`) — Missing voice configuration, unavailable ffmpeg or recognizer, missing GigaAM model files, or a failed one-second silent WAV decode/recognition probe makes doctor fail with a short reason. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:65` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I665 (`docs/contract-177-inventory.md:670`) — An empty transcript is expected for silence; the recognizer must still return valid result JSON. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:67` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I666 (`docs/contract-177-inventory.md:671`) — The probe uses protected temporary files, a ten-second total deadline and no Telegram input or model turn. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:68` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I667 (`docs/contract-177-inventory.md:672`) — This is setup evidence, not live voice acceptance. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:69` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I668 (`docs/contract-177-inventory.md:673`) — **Owner, task #144:** this supersedes #127's GitHub CLI answer reader and doctor probe. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120; [#144](https://github.com/taskqorg/pilotq/issues/144) / Source #144 | `docs/service-contract.md:73` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Explicitly superseded/expanded by [#144](https://github.com/taskqorg/pilotq/issues/144); current typed-tool and qualification protections remain. |
| I669 (`docs/contract-177-inventory.md:674`) — TaskQ owns board-answer access and trust. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:71` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I670 (`docs/contract-177-inventory.md:675`) — Doctor checks the configured TaskQ board through its CLI; it neither requires nor probes gh. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:71` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I671 (`docs/contract-177-inventory.md:676`) — Selftest remains offline. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:8` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I672 (`docs/contract-177-inventory.md:677`) — CLI failures retain scrubbed operator diagnostics under §8; unreadable answers remain unknown. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:73` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
| I673 (`docs/contract-177-inventory.md:678`) — Prepare version `0.2.12` for #127. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:75` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Explicitly superseded/expanded by [#144](https://github.com/taskqorg/pilotq/issues/144); current typed-tool and qualification protections remain. |
| I674 (`docs/contract-177-inventory.md:679`) — After merge-result CI and review acceptance, the PM publishes #127 through the existing `live` release workflow, updates the existing FRA service through `pilotq update`, and records source/binary identity, health, selftest, doctor and journal continuity with §12 drain and compatible-schema rollback protections. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:8` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Explicitly superseded/expanded by [#144](https://github.com/taskqorg/pilotq/issues/144); current typed-tool and qualification protections remain. |
| I675 (`docs/contract-177-inventory.md:680`) — The PM records a real authorized person's question answered through TaskQ CLI, the original Telegram card with buttons removed and `Answered by &lt;actor&gt;: &lt;text&gt;`, and a later tap showing the same receipt. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:8` | `src/main.rs:259` | `tests/telegram.rs:3223` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
| I676 (`docs/contract-177-inventory.md:681`) — These post-release checks remain required evidence, not inferred from adapter tests. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:8` | `src/main.rs:259` | `tests/telegram.rs:3223` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
| I677 (`docs/contract-177-inventory.md:682`) — This grants no Hermes consumer cutover. | [#120](https://github.com/taskqorg/pilotq/issues/120) / Source #120 | `docs/service-contract.md:41` | `src/main.rs:259` | `tests/telegram.rs:3223` | met | Retained; named deterministic protection covers the domain risk. |
### Host requirements (task #130)

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I678 (`docs/contract-177-inventory.md:683`) — The supported managed host is Linux x86_64. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:85` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I679 (`docs/contract-177-inventory.md:684`) — The installation owner provisions these tools; PilotQ never runs a package manager, elevates privileges or changes a subscription login. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:86` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I680 (`docs/contract-177-inventory.md:685`) — Doctor probes the configured binaries and PATH used by the service. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:87` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I681 (`docs/contract-177-inventory.md:686`) — Installed-release startup, install and manual/scheduled update checks refuse missing, unparseable or older versions, reporting the exact operator step. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:88` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I682 (`docs/contract-177-inventory.md:687`) — After remediation, rerun `pilotq doctor --json` and the original install/update command. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:89` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I683 (`docs/contract-177-inventory.md:688`) — Offline selftest remains independent of host setup. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:90` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I684 (`docs/contract-177-inventory.md:689`) — &#124; Host tool &#124; Minimum &#124; Required by / remediation &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:92` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I685 (`docs/contract-177-inventory.md:690`) — &#124; Codex CLI &#124; 0.162.0 &#124; Subscription app-server and catalogue; install `@openai/codex@0.162.0` or newer and set the absolute `codex` path; retain the isolated ChatGPT login &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:94` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I686 (`docs/contract-177-inventory.md:691`) — &#124; TaskQ CLI &#124; 4.6.46 &#124; Task #145 file handoff, operation receipts and task #151 `taskq issue`; update the CLI to 4.6.46 or newer; doctor reads `taskq --version` &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:95` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I687 (`docs/contract-177-inventory.md:692`) — &#124; TaskQ service &#124; 4.6.40 &#124; Board commands; doctor checks the service version from `service status --json` &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:96` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I688 (`docs/contract-177-inventory.md:693`) — &#124; ffmpeg &#124; 6.1.0 &#124; Audio decode; install ffmpeg 6.1 or newer and set `voice.ffmpeg` &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:97` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I689 (`docs/contract-177-inventory.md:694`) — &#124; sherpa-onnx-offline &#124; 2.0.1 &#124; GigaAM recognition; install the static no-TTS 2.0.1 release or newer and set `voice.recognizer` &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:98` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I690 (`docs/contract-177-inventory.md:695`) — &#124; curl &#124; 7.68.0 &#124; CoreQ release download; install curl 7.68 or newer on service PATH &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:99` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I691 (`docs/contract-177-inventory.md:696`) — &#124; systemctl (systemd) &#124; 245 &#124; CoreQ Linux user service; provision systemd 245 or newer with a working user service manager &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:100` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I692 (`docs/contract-177-inventory.md:697`) — &#124; kill (procps-ng) &#124; 3.3.17 &#124; Process-group cleanup on PATH and `/bin/kill`; install procps-ng 3.3.17 or newer on service PATH &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:101` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I693 (`docs/contract-177-inventory.md:698`) — &#124; `commands.json` executables &#124; Each entry declares `minimum_version` &#124; Approved installed reads (§5); doctor runs the executable with `--version`; an absent minimum or unavailable version fails doctor &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:102` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I694 (`docs/contract-177-inventory.md:699`) — &#124; hostname &#124; 3.23 &#124; Telemetry host identity; install hostname 3.23 or newer on service PATH &#124; | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:103` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I695 (`docs/contract-177-inventory.md:700`) — **Owner, task #154:** TaskQ CLI 4.6.46 or newer reports its binary version through `taskq --version`, without board or network access. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:105` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I696 (`docs/contract-177-inventory.md:701`) — Doctor uses that command for `taskq.bin` and `--version` for approved command executables; TaskQ needs no receipt. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:106` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I697 (`docs/contract-177-inventory.md:702`) — Receipts remain only for tools without a version command: sherpa-onnx-offline 2.0.1 has no CLI version command. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:107` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I698 (`docs/contract-177-inventory.md:703`) — Its installed executable has an owner-provisioned `&lt;binary&gt;.pilotq-version.json` receipt containing the verified release `version` and executable `sha256`; doctor verifies the bytes before trusting that version. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:109` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I699 (`docs/contract-177-inventory.md:704`) — The tester guide gives the provisioning step. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:83` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I700 (`docs/contract-177-inventory.md:705`) — A missing or mismatched receipt fails with an unavailable version. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:111` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I701 (`docs/contract-177-inventory.md:706`) — A later recognizer with a valid version command can report its version directly. | [#154](https://github.com/taskqorg/pilotq/issues/154) / Source #154 | `docs/service-contract.md:112` | `src/host.rs:35` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I702 (`docs/contract-177-inventory.md:707`) — There are no separately installed PilotQ shared libraries: SQLite, Rust TLS and CoreQ 0.14.6 are linked into the artifact through Cargo.lock. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:114` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I703 (`docs/contract-177-inventory.md:708`) — The static recognizer is required; ffmpeg and system tools retain their distribution-managed libraries. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:115` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I704 (`docs/contract-177-inventory.md:709`) — Doctor's existing silent decode/recognition probe checks the voice runtime and model files in addition to versions. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:117` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I705 (`docs/contract-177-inventory.md:710`) — Developer/release-only tools (`cargo`, the Rust compiler, `sha256sum`, and shell utilities in download instructions) are not service runtime dependencies. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:118` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I706 (`docs/contract-177-inventory.md:711`) — Optional unsupported platforms are not qualified by the Linux requirement probes. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:119` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I707 (`docs/contract-177-inventory.md:712`) — Doctor's operator JSON adds `requirements` with tool, found version, minimum, outcome and fix; only validated version numbers enter this report. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:122` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I708 (`docs/contract-177-inventory.md:713`) — Missing configuration still reports independently probeable requirements and fails configured ones. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:123` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I709 (`docs/contract-177-inventory.md:714`) — `/doctor` projects the aggregate `host_tools` outcome without paths, versions or diagnostics (§8). | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:125` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I710 (`docs/contract-177-inventory.md:715`) — Failed subprocesses log command and scrubbed errors to operator stderr; failures remain errors, never successful empty output. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:126` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
| I711 (`docs/contract-177-inventory.md:716`) — A new flag/API must retain the listed minimum or raise it with its oldest-version fixture. | [#130](https://github.com/taskqorg/pilotq/issues/130) / Source #130 | `docs/service-contract.md:127` | `src/host.rs:68` | `tests/update.rs:252` | met | Retained; named deterministic protection covers the domain risk. |
### Update and monitoring

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I712 (`docs/contract-177-inventory.md:717`) — Follow coreq: `releases/&lt;version&gt;`, `current` and `previous` in the install directory; GitHub releases with `SHA256SUMS`; `pilotq update` verifies the checksum and runs `pilotq selftest` before the switch; the launcher returns to `previous` after three unhealthy starts. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:132` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I713 (`docs/contract-177-inventory.md:718`) — An update waits for the planned-restart drain in section 12. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:134` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I714 (`docs/contract-177-inventory.md:719`) — **Owner, task #77 (option A):** on the restored version’s first healthy start after a launcher rollback, send one incident to the protected `owner_id` Telegram DM, naming the failed and restored versions only. | [#77](https://github.com/taskqorg/pilotq/issues/77) / Source #77 | `docs/service-contract.md:136` | `src/main.rs:140` | `src/main.rs:829` | met | Retained; named deterministic protection covers the domain risk. |
| I715 (`docs/contract-177-inventory.md:720`) — Persist the rollback marker before launcher handoff and enqueue the incident atomically with a failed/restored-version deduplication key in the existing journal. | [#77](https://github.com/taskqorg/pilotq/issues/77) / Source #77 | `docs/service-contract.md:138` | `src/main.rs:140` | `src/main.rs:829` | met | Retained; named deterministic protection covers the domain risk. |
| I716 (`docs/contract-177-inventory.md:721`) — The existing outbox records the send receipt and applies bounded retries across restarts (§4, §7); never reroute a failed DM. | [#77](https://github.com/taskqorg/pilotq/issues/77) / Source #77 | `docs/service-contract.md:140` | `src/main.rs:140` | `src/main.rs:829` | met | Retained; named deterministic protection covers the domain risk. |
| I717 (`docs/contract-177-inventory.md:722`) — Successful updates produce no failure notice. | [#77](https://github.com/taskqorg/pilotq/issues/77) / Source #77 | `docs/service-contract.md:140` | `src/main.rs:140` | `src/main.rs:829` | met | Retained; named deterministic protection covers the domain risk. |
| I718 (`docs/contract-177-inventory.md:723`) — Both the launcher and restored release must include this integration; older releases cannot deliver a new rollback marker. | [#77](https://github.com/taskqorg/pilotq/issues/77) / Source #77 | `docs/service-contract.md:141` | `src/main.rs:140` | `src/main.rs:829` | met | Retained; named deterministic protection covers the domain risk. |
| I719 (`docs/contract-177-inventory.md:724`) — No schema change or new notification infrastructure is required. | [#77](https://github.com/taskqorg/pilotq/issues/77) / Source #77 | `docs/service-contract.md:142` | `src/main.rs:140` | `src/main.rs:829` | met | Retained; named deterministic protection covers the domain risk. |
| I720 (`docs/contract-177-inventory.md:725`) — **Owner, task #81 (option A):** extend the #77 path to service state; standard simple setup, no new component. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:145` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I721 (`docs/contract-177-inventory.md:726`) — `/status` adds the version and each non-ok health source with its age. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:146` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I722 (`docs/contract-177-inventory.md:727`) — Service incidents for `model`, `taskq`, `poll` and an unplanned restart go only to the `owner_id` DM through the outbox: one incident per episode key across restarts, opened after the source stays failed 2 minutes (model: two consecutive failed runs), closed only on a fresh successful observation, at most one message per key per 6 hours and 10 per day. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:149` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I723 (`docs/contract-177-inventory.md:728`) — Planned updates and drained restarts stay silent. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:150` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I724 (`docs/contract-177-inventory.md:729`) — On a healthy start after an undrained gap over 5 minutes, send one downtime notice; detection is after the fact. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:151` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I725 (`docs/contract-177-inventory.md:730`) — Rules and examples: [status and notifications](docs/status-notifications.md). | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:152` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I726 (`docs/contract-177-inventory.md:731`) — An ops chat (stage B) and real-time dead-service detection through DashQ (stage C) wait for evidence. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:153` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I727 (`docs/contract-177-inventory.md:732`) — `status.json` publishes the section 7 health fields. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:155` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I728 (`docs/contract-177-inventory.md:733`) — Incident reports follow coreq. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:155` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I729 (`docs/contract-177-inventory.md:734`) — Telemetry uses the shared CoreQ telemetry store and DashQ connection, so the TaskQ PM sees PilotQ beside the other tools. | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:156` | `src/journal.rs:1495` | `src/journal.rs:4249` | met | Retained; named deterministic protection covers the domain risk. |
| I730 (`docs/contract-177-inventory.md:735`) — Release and CI follow the shared [release contract](https://github.com/taskqorg/specq/blob/main/engineering/release-contract.md). | [#81](https://github.com/taskqorg/pilotq/issues/81) / Source #81 | `docs/service-contract.md:158` | `src/journal.rs:1495`; `.github/workflows/release.yml:38` | `src/journal.rs:4249`; `scripts/verify-release.sh:6` (artifact recipe; not run for this task) | met | Configured release/merge-result gate and artifact recipe retained; this task publishes no release. |
### First release delivery

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I731 (`docs/contract-177-inventory.md:736`) — Releases publish only through the `live` channel. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:162` | `src/main.rs:102`; `.github/workflows/release.yml:38` | `tests/update.rs:180`; `scripts/verify-release.sh:6` (artifact recipe; not run for this task) | met | Configured release/merge-result gate and artifact recipe retained; this task publishes no release. |
| I732 (`docs/contract-177-inventory.md:737`) — After merge-queue CI accepts a `main` commit, the PM fast-forwards `live` to it; that push builds and verifies the static `linux-x86_64` artifact, creates the `vX.Y.Z` tag and publishes the binary, `SHA256SUMS` and `latest.json` from `taskqorg/pilotq`. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:163` | `src/main.rs:102`; `.github/workflows/release.yml:38` | `tests/update.rs:180`; `scripts/verify-release.sh:6` (artifact recipe; not run for this task) | met | Configured release/merge-result gate and artifact recipe retained; this task publishes no release. |
| I733 (`docs/contract-177-inventory.md:738`) — Tags alone do not publish. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:165` | `src/main.rs:102`; `.github/workflows/release.yml:38` | `tests/update.rs:180`; `scripts/verify-release.sh:6` (artifact recipe; not run for this task) | met | Configured release/merge-result gate and artifact recipe retained; this task publishes no release. |
| I734 (`docs/contract-177-inventory.md:739`) — After publishing, the PM updates the test pilot through `pilotq update` and verifies its running source commit, status, selftest, doctor and request continuity. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:166` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I735 (`docs/contract-177-inventory.md:740`) — Failure drills simulate failures in PilotQ's own environment only: an empty `CODEX_HOME` drop-in instead of the real Codex login, stopping only what PilotQ calls or waiting for zero workers before stopping TaskQ, and killing only the PilotQ process; afterwards the service configuration is restored and read back. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:168` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I736 (`docs/contract-177-inventory.md:741`) — A release authorizes the test pilot update, not production cutover. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:170` | `src/main.rs:102`; `.github/workflows/release.yml:38` | `tests/update.rs:180`; `scripts/verify-release.sh:6` (artifact recipe; not run for this task) | met | Configured release/merge-result gate and artifact recipe retained; this task publishes no release. |
| I737 (`docs/contract-177-inventory.md:742`) — The installed Linux service checks for releases every five minutes, including at startup when no recent check is recorded. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:172` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I738 (`docs/contract-177-inventory.md:743`) — It persists the attempt time before the check so restarts do not reset the retry interval. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:173` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I739 (`docs/contract-177-inventory.md:744`) — A separate maintenance thread keeps health reporting and conversations available during downloads. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:174` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I740 (`docs/contract-177-inventory.md:745`) — Manual and scheduled updates share one OS-held update lock. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:175` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I741 (`docs/contract-177-inventory.md:746`) — A journal drain flag gates both CLI and Telegram claims in their claim transactions, so a new run cannot race the drain. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:176` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I742 (`docs/contract-177-inventory.md:747`) — Before activation, stop new manager turns and wait up to 65 seconds for running requests; an incomplete or failed drain refuses activation and preserves the current release. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:177` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I743 (`docs/contract-177-inventory.md:748`) — Failed downloads, checksum checks or selftests retain the current release and resume admission. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:179` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I744 (`docs/contract-177-inventory.md:749`) — Successful scheduled activation exits the service with code 75; the coreq Linux unit restarts it through the launcher. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:180` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I745 (`docs/contract-177-inventory.md:750`) — `pilotq update --pause` and `--resume` persist and read back automatic-update admission; pausing does not stop health or conversations. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:181` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I746 (`docs/contract-177-inventory.md:751`) — Manual updates remain available. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:182` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I747 (`docs/contract-177-inventory.md:752`) — This release retains coreq `v0.14.6` (locked in `Cargo.lock`) and journal schema 12. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:184` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I748 (`docs/contract-177-inventory.md:753`) — It opens existing schemas 0–12 through the current migrations and refuses newer schemas; rollback to a binary that cannot read the resulting schema is unsafe. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:185` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I749 (`docs/contract-177-inventory.md:754`) — The installation owner runs the user service and owns its protected configuration. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:186` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I750 (`docs/contract-177-inventory.md:755`) — See the [tester guide](docs/tester-guide.md) for installation, checks and current limitations. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:187` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
| I751 (`docs/contract-177-inventory.md:756`) — A published release and successful selftest do not qualify production cutover or the open decisions in section 11. | [#3](https://github.com/taskqorg/pilotq/issues/3) / Source #3 | `docs/service-contract.md:189` | `src/main.rs:102` | `tests/update.rs:180` | met | Retained; named deterministic protection covers the domain risk. |
### Development

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I752 (`docs/contract-177-inventory.md:757`) — **Owner, tasks #106, #117, #119 and #139:** PilotQ is still in development. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117; [#106](https://github.com/taskqorg/pilotq/issues/106) / Source #106; [#119](https://github.com/taskqorg/pilotq/issues/119) / Source #119; [#139](https://github.com/taskqorg/pilotq/issues/139) / Source #139 | `docs/service-contract.md:191` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I753 (`docs/contract-177-inventory.md:758`) — The PM installs merged, released builds on the existing live service on PDS 18 (`pds18.ufobe.com`) and verifies them there; no separate installation or bot. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:194` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I754 (`docs/contract-177-inventory.md:759`) — This is a scoped exception to the shared release contract's development-host separation, not authorization to install unfinished branches or a Hermes consumer cutover. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:195` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I755 (`docs/contract-177-inventory.md:760`) — The installation owner remains the service owner. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:196` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I756 (`docs/contract-177-inventory.md:761`) — Record the source commit, running binary identity, health, selftest and journal continuity across restart, with the section 12 drain and compatible-schema rollback protections. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:197` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I757 (`docs/contract-177-inventory.md:762`) — Telegram commands, model selection and answer cards need a real authorized person's run: the PM performs one DM/topic run and attaches screenshots. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:199` | `src/main.rs:45` | `src/main.rs:873` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
| I758 (`docs/contract-177-inventory.md:763`) — The live answer-card author, button removal and repeated-callback scenario (#117) remains unverified. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:201` | `src/main.rs:45` | `src/main.rs:873` | missing | Exact required live route/artifact evidence is absent from this candidate; deterministic tests are not live proof (§9). Follow-up [#198](https://github.com/taskqorg/pilotq/issues/198). |
| I759 (`docs/contract-177-inventory.md:764`) — Worker verification stays local/CI; workers never install live. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:201` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I760 (`docs/contract-177-inventory.md:765`) — Communication scope: [Telegram feedback](docs/telegram-feedback.md) under §12. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:202` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I761 (`docs/contract-177-inventory.md:766`) — PilotQ has its own TaskQ board. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:204` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I762 (`docs/contract-177-inventory.md:767`) — Changes follow the TaskQ flow: spec edit first, held task, worker, reviewer, release. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:204` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I763 (`docs/contract-177-inventory.md:768`) — Before each release, run the [automatic management suite](docs/acceptance.md#automatic-management-suite-a40) against the release candidate. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:206` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I764 (`docs/contract-177-inventory.md:769`) — Suite tasks use a unique reserved `pilotq-e2e-*` host that no service claims; the runner scripts their worker/reviewer board events without launching ordinary workers. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:207` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I765 (`docs/contract-177-inventory.md:770`) — This qualifies the management path, not real implementation or review. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:195` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
| I766 (`docs/contract-177-inventory.md:771`) — All cases and cleanup must pass; a blocked or incomplete run is not release evidence. | [#117](https://github.com/taskqorg/pilotq/issues/117) / Source #117 | `docs/service-contract.md:209` | `src/main.rs:45` | `src/main.rs:873` | met | Documentary/provenance clause retained; no executable history or live claim. |
### 10. Adoption sequence

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I767 (`docs/contract-177-inventory.md:772`) — See [adoption sequence](docs/adoption.md#10-adoption-sequence). | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/adoption.md:5` | `src/journal.rs:2796` | `src/journal.rs:3455` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
### 11. Decisions still open

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I768 (`docs/contract-177-inventory.md:773`) — See [open decisions](docs/adoption.md#11-decisions-still-open). | [#26](https://github.com/taskqorg/pilotq/issues/26) / Source #26 | `docs/adoption.md:18` | `src/journal.rs:2796` | `src/journal.rs:3455` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
### 12. Proposed lifecycle and feedback protocol

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I769 (`docs/contract-177-inventory.md:774`) — This section translates R5 and R11 into a proposed PilotQ protocol. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:285` | `src/manager.rs:183` | `src/manager.rs:712` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I770 (`docs/contract-177-inventory.md:775`) — The [mechanism matrix](docs/mechanism-matrix.md) records source evidence and scope choices; [acceptance](docs/acceptance.md#lifecycle-cases) defines the additional checks. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:286` | `src/manager.rs:183` | `src/manager.rs:712` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I771 (`docs/contract-177-inventory.md:776`) — These are design targets, not implemented guarantees. | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:288` | `src/manager.rs:183` | `src/manager.rs:712` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
| I772 (`docs/contract-177-inventory.md:777`) — Notification presentation follows [specq](https://github.com/taskqorg/specq/blob/main/caveman/rules/notifications.md). | [#1](https://github.com/taskqorg/pilotq/issues/1) / Source #1 | `docs/runtime-contract.md:289` | `src/manager.rs:183` | `src/manager.rs:712` | met | Proposal/target retained under R19; test is boundary evidence, not a measured target. |
### One request, separate observations

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I773 (`docs/contract-177-inventory.md:778`) — Keep the request state from section 4; use a reason/phase field for queued capacity, model startup, tool execution, compaction, provider backoff, quota exhaustion or required login. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:293` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I774 (`docs/contract-177-inventory.md:779`) — Do not introduce a second task state machine. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:291` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I775 (`docs/contract-177-inventory.md:780`) — Waiting for a person or for TaskQ is not a hung manager run. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:295` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I776 (`docs/contract-177-inventory.md:781`) — A delegated request remains linked to the authoritative TaskQ task and its notification subscription. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:296` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I777 (`docs/contract-177-inventory.md:782`) — Track transport health, request activity and delivery separately. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:298` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I778 (`docs/contract-177-inventory.md:783`) — A successful empty poll proves transport contact; it does not prove received inputs reached admission. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:299` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I779 (`docs/contract-177-inventory.md:784`) — A live process or typing indicator proves neither useful progress nor task completion. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:300` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I780 (`docs/contract-177-inventory.md:785`) — Count accepted/dispatched inputs and pending age as well as successful polls. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:301` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I781 (`docs/contract-177-inventory.md:786`) — Unknown observation remains unknown. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:301` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I782 (`docs/contract-177-inventory.md:787`) — The runtime supervisor observes startup/first-event latency, last useful runtime/tool event and the absolute run deadline. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:291` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I783 (`docs/contract-177-inventory.md:788`) — The first-event watch starts before invoking Codex; it cannot depend on Codex having emitted its first event. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:291` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I784 (`docs/contract-177-inventory.md:789`) — Typing, polling and periodic status timers never renew useful-progress time. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:291` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I785 (`docs/contract-177-inventory.md:790`) — Token activity cannot extend the absolute short-action budget. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:291` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I786 (`docs/contract-177-inventory.md:791`) — Waiting states retain their reason and wake condition without busy retries. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:296` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
| I787 (`docs/contract-177-inventory.md:792`) — Use monotonic time within a process; persist deadlines/attempts across restart and handle wall-clock jumps without granting a fresh unlimited budget. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:291` | `src/manager.rs:183` | `src/manager.rs:712` | met | Retained; named deterministic protection covers the domain risk. |
### Telegram feedback

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I788 (`docs/contract-177-inventory.md:793`) — Contract detail: [Telegram feedback](docs/telegram-feedback.md) holds the owner decisions on manager instructions, worker questions, reply layout, reactions, drafts, images and voice. | [#139](https://github.com/taskqorg/pilotq/issues/139) / Source #139 | `docs/runtime-contract.md:307` | `src/instructions.rs:10` | `tests/telegram.rs:2643` | met | Retained; named deterministic protection covers the domain risk. |
| I789 (`docs/contract-177-inventory.md:794`) — It is binding as part of this section. | [#139](https://github.com/taskqorg/pilotq/issues/139) / Source #139 | `docs/runtime-contract.md:309` | `src/instructions.rs:10` | `tests/telegram.rs:2643` | met | Retained; named deterministic protection covers the domain risk. |
### Input and control edges

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I790 (`docs/contract-177-inventory.md:795`) — During startup, check bot identity, required Telegram permissions/privacy configuration and forum-topic support against the migration manifest. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:313` | `src/telegram.rs:32` | `tests/telegram.rs:857` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I791 (`docs/contract-177-inventory.md:796`) — Do not report readiness solely from login success. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:314` | `src/telegram.rs:32` | `tests/telegram.rs:857` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I792 (`docs/contract-177-inventory.md:797`) — Received but unsupported inputs get the explicit outcome in section 1; unknown senders still follow R10's silent policy. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:315` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I793 (`docs/contract-177-inventory.md:798`) — Preserve message/reply IDs, edits and media-group identities before optional batching. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:318` | `src/telegram.rs:32` | `tests/telegram.rs:857` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
| I794 (`docs/contract-177-inventory.md:799`) — Never coalesce messages across users, routes or authorization revisions. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:319` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I795 (`docs/contract-177-inventory.md:800`) — A bounded media assembly window cannot swallow a text follow-up or change a running plan. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:320` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I796 (`docs/contract-177-inventory.md:801`) — Recognizing attachments and reporting limits are required; interpreting every media type remains an open cutover decision. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:321` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I797 (`docs/contract-177-inventory.md:802`) — Authorized `/status`, `/stop` and `/new` use a bounded control path outside the normal model backlog. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:323` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I798 (`docs/contract-177-inventory.md:803`) — They do not bypass identity or scope checks. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:324` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I799 (`docs/contract-177-inventory.md:804`) — A reset creates a new conversation generation; old requests and receipts remain queryable, and late old-run events cannot attach to the new conversation. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:325` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I800 (`docs/contract-177-inventory.md:805`) — Reset does not revoke the originating subscriptions to already admitted TaskQ work. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:326` | `src/telegram.rs:32` | `tests/telegram.rs:857` | met | Retained; named deterministic protection covers the domain risk. |
| I801 (`docs/contract-177-inventory.md:806`) — Topic deletion/closure, group migration, bot removal and permission loss become explicit delivery/configuration incidents. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:328` | `src/telegram.rs:32` | `tests/telegram.rs:857` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
| I802 (`docs/contract-177-inventory.md:807`) — Preserve the original destination and hold affected output. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:329` | `src/telegram.rs:32` | `tests/telegram.rs:857` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
| I803 (`docs/contract-177-inventory.md:808`) — An administrator must authorize a route change; do not fall back to General, a private DM or another group automatically. | [#4](https://github.com/taskqorg/pilotq/issues/4) / Source #4 | `docs/runtime-contract.md:330` | `src/telegram.rs:32` | `tests/telegram.rs:857` | missing | Message edits and route lifecycle events have no durable ingress projection; send failures alone do not cover them. Follow-up [#197](https://github.com/taskqorg/pilotq/issues/197). |
### Runtime and recovery boundaries

| ID and exact source text | Source task | Specification | Code evidence | Test evidence | Status | Disposition / limit |
|---|---|---|---|---|---|---|
| I804 (`docs/contract-177-inventory.md:809`) — Validate the pinned Codex protocol/capabilities before accepting executable work. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:335` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I805 (`docs/contract-177-inventory.md:810`) — Unknown optional diagnostic events can be retained as metadata; unknown control/terminal semantics cannot be guessed as success. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:336` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I806 (`docs/contract-177-inventory.md:811`) — Parse tools only after a complete validated request; a parseable partial argument stream is not permission to act. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:337` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I807 (`docs/contract-177-inventory.md:812`) — Disable or contain native tools that bypass PilotQ's effect boundary. | [#190](https://github.com/taskqorg/pilotq/issues/190) / Source #190 | `docs/runtime-contract.md:333` | `src/instructions.rs:10` | `src/instructions.rs:37` | met | Explicitly superseded/expanded by [#190](https://github.com/taskqorg/pilotq/issues/190); current typed-tool and qualification protections remain. |
| I808 (`docs/contract-177-inventory.md:813`) — An empty useful answer with no verified delegation/action is an explicit outcome to recover or report, not silent success. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:339` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I809 (`docs/contract-177-inventory.md:814`) — Classify auth, quota, transport, protocol and tool failures. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:342` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I810 (`docs/contract-177-inventory.md:815`) — Keep one bounded retry budget per logical operation across layers/restarts; honor server retry times without holding conversation locks or spinning. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:343` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I811 (`docs/contract-177-inventory.md:816`) — Persist account/workspace binding with the native session. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:344` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I812 (`docs/contract-177-inventory.md:817`) — Reauthentication must not silently change that binding, discard the session or fall back to billed API access. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:345` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I813 (`docs/contract-177-inventory.md:818`) — The shared deployment's subscription entitlement remains a separate pre-production qualification. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:346` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I814 (`docs/contract-177-inventory.md:819`) — Use one journal reconciler at startup and on a bounded periodic scan to discover pending inputs, stale runs, uncertain effects, missed TaskQ events and undelivered outcomes. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:348` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I815 (`docs/contract-177-inventory.md:820`) — Timers are wakeups, never the sole copy of work. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:333` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I816 (`docs/contract-177-inventory.md:821`) — Reconcile one request at a time; a failing route cannot stop recovery for the others. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:350` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I817 (`docs/contract-177-inventory.md:822`) — Keep external calls outside database transactions. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:351` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I818 (`docs/contract-177-inventory.md:823`) — On planned restart/update, stop new execution, checkpoint/drain within a bounded grace period, fence and terminate remaining manager process trees, and leave unresolved work in the journal. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:353` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I819 (`docs/contract-177-inventory.md:824`) — TaskQ workers retain their own lifecycle. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:355` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I820 (`docs/contract-177-inventory.md:825`) — Startup validates config and storage, obtains the poller lock, reconciles state and only then reports readiness. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:355` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I821 (`docs/contract-177-inventory.md:826`) — A crash-loop backoff prevents repeated startup from consuming retries or producing alert storms. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:357` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I822 (`docs/contract-177-inventory.md:827`) — An external host monitor handles a dead or blocked PilotQ process; PilotQ cannot promise a Telegram notice while both it and Telegram are unavailable. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:358` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I823 (`docs/contract-177-inventory.md:828`) — Apply the shared [release/service contract](https://github.com/taskqorg/specq/blob/main/engineering/release-contract.md) to the selected Linux service and pinned Codex artifact. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:361` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |
| I824 (`docs/contract-177-inventory.md:829`) — Before implementation, name the service owner, artifact/update source and compatible schema range. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:363` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I825 (`docs/contract-177-inventory.md:830`) — A rollback must preserve accepted work; an incompatible database prevents unsafe downgrade. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:364` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I826 (`docs/contract-177-inventory.md:831`) — No live runtime replacement occurs mid-turn without the drain/reconcile path. | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:364` | `src/manager.rs:183` | `tests/telegram.rs:811` | missing | Missing implementation or current qualification; retained adoption/proposal gate, not a shipped guarantee (R19). Follow-up [#199](https://github.com/taskqorg/pilotq/issues/199). |
| I827 (`docs/contract-177-inventory.md:832`) — This specification does not authorize installation, model calls, queue activation or replacement of the production Telegram consumer.  | [#6](https://github.com/taskqorg/pilotq/issues/6) / Source #6 | `docs/runtime-contract.md:367` | `src/manager.rs:183` | `tests/telegram.rs:811` | met | Retained; named deterministic protection covers the domain risk. |

## Every original ponytail comment in src/

The baseline contains seventeen markers. Each is reviewed below; removing a narrowing comment does not conceal the underlying open behavior.

| Original marker / location | Requirement and code/test evidence | Status | Review outcome |
|---|---|---|---|
| src/files.rs:29:// ponytail: in-process parse bounded by the 20 MiB input, INFLATED and PAGES; a child process with rlimits if a PDF ever hangs a turn. | §1/#186; src/files.rs:30; src/files.rs:67 | missing | Removed the assurance: size/page limits are not time or cancellation bounds. F4 → [#195](https://github.com/taskqorg/pilotq/issues/195). |
| src/files.rs:43:// ponytail: listing only; read entries (docx/xlsx text) when a person needs them. | §1/#186; src/files.rs:44; src/files.rs:67 | met | ZIP listing is the approved extraction scope. No promise to read docx/xlsx entries is silently dropped. |
| src/journal.rs:63:// ponytail: Cyrillic is the only detected script; a language detector only if a third chat language appears. | §9/#186; src/journal.rs:64; src/telegram.rs:714 | met | The deterministic language scope is English/Russian. A third language is not claimed supported by this detector. |
| src/journal.rs:1583:        // ponytail: CLI requests take negative synthetic update IDs, so Telegram's positive IDs never collide | R13/§4; src/journal.rs:1575; tests/telegram.rs:677 | met | CLI and Telegram update namespaces remain separate while using the same journal/grants. |
| src/journal.rs:1755:        // ponytail: refuse oversized recovery evidence instead of silently dropping receipts; bounded retrieval if needed. | §4/§6; src/journal.rs:1733; tests/telegram.rs:2333 | met | Explicit refusal preserves receipts and names the limit; it does not silently truncate or replay. |
| src/journal.rs:2939:            // ponytail: CoreQ codes allow only [A-Za-z0-9._:-]; a model name outside that is left out, not rewritten | R17/§8; src/journal.rs:2922; src/journal.rs:4210 | met | Unsafe telemetry code names are omitted from that field, not rewritten into false identities; audit retains actual model. |
| src/keys.rs:66:        // ponytail: scrubbing replaces the whole value, so a short value would rewrite ordinary output | R16/§8; src/keys.rs:59; tests/telegram.rs:1591 | met | Refuses short/whitespace key values explicitly instead of silently rewriting ordinary text. Supported board/Asana credential types retain protected entry and scrubbing; arbitrary secret formats are not claimed. |
| src/main.rs:253:    // ponytail: no config or board read yet; the online part arrives with the journal and TaskQ in later slices. | §9; src/main.rs:246; tests/update.rs:82 | met | Replaced obsolete later-slices comment with the enduring offline-selftest/online-doctor split. |
| src/manager.rs:57:// ponytail: process-wide startup lock; a hung startup delays other conversations by at most the first-event bound | §3/R5; src/manager.rs:99; tests/telegram.rs:833 | met | Serializes initialization only; independent turns retain their own processes and inactivity bounds. |
| src/manager.rs:90:        // ponytail: `kill` on the process group; a pidfd/cgroup supervisor if children escape the group | §5/#190; src/manager.rs:84; src/manager.rs:700 | met | Implements the specified process-group stop. Escaped process trees are not claimed to be contained; broader planned-restart qualification remains in #199. |
| src/manager.rs:137:                // ponytail: a non-JSON line is diagnostic output, never a control event | §12; src/manager.rs:99; tests/telegram.rs:703 | met | Diagnostic lines cannot authorize calls or terminal success; the inactivity/EOF path still fails explicitly. |
| src/manager.rs:692:        // ponytail: 0.5 s journal poll; a wake channel from ingress if admission latency matters | §7; src/manager.rs:571; tests/telegram.rs:556 | met | Polling cadence is an implementation choice; no accepted request or route is dropped. |
| src/release.rs:36:    // ponytail: lists every main commit since the merge base (1 MiB cap); bisect if histories outgrow it. | §5/#174; src/release.rs:121; src/release.rs:662 | met | Bounded history enumeration refuses overflow; it never substitutes an empty range or guessed commit. |
| src/release.rs:405:    // ponytail: one installation-wide release lock; per-repository locks only if simultaneous releases are needed. | §5/#174; src/release.rs:401; src/release.rs:687 | met | A stronger shared lock serializes releases; simultaneous independent releases were not an owner requirement. |
| src/telegram.rs:22:// ponytail: env-gated fault point; a fault-injection framework only if more points appear. | §4/R5; src/telegram.rs:23; tests/telegram.rs:370 | met | Fault-injection seam does not define product behavior or reduce durability checks. |
| src/telegram.rs:264:// ponytail: word-level scan with a fixed extension list; a Markdown tokenizer only if labels or paths need more cases. | #82/§12; src/telegram.rs:265; src/telegram.rs:808 | narrowed | F1: fixed-extension/ASCII ceiling narrowed repository-path formatting. Marker removed and path detection generalized; failed before/passed after. |
| src/telegram.rs:328:/// ponytail: headings, bullets, bold, italic, escapes, code and HTTP(S) links; a full Markdown parser only when the subset needs to grow. | #72/#82/§4; src/telegram.rs:353; src/telegram.rs:792 | met | This is the approved safe HTML fallback subset, not the rich Markdown transport ceiling. Raw model HTML stays escaped. |

## Verification and limits

- Development: `cargo test --locked --bin pilotq links_are_labeled_and_paths_are_code` failed on the old code (`ui/App.vue` remained raw), then passed after the fix. The same test retains link/code/URL/version cases and now checks both rich Markdown and HTML for ten path variants, including the two review failures; surrounding prose and command/repository guards remain.
- Development: `cargo test --locked --bin pilotq relocated_product_policy_is_loaded` failed against the old loaded policy, then passed after spec correction. Launch/card/answer authority protection remains; only the owner-superseded direct-work restriction changes.
- `cargo test --locked --bin pilotq telegram::tests` exercises the affected formatting and input boundaries. Candidate gate results are recorded in the companion [task report](191.md).
- Initial Cargo attempts were environment-blocked by the read-only default cache and absent submodule files, not passing behavior checks. A writable temporary Cargo home and an archive of the exact existing specq gitlink were used without changing its pin.
- A one-time coverage/readback check verifies I001–I827 exactly once, every baseline marker, owner-note locations, statuses, source links, code/test references and held follow-up readbacks. Textual completeness is not semantic or live proof.
- Not deployed. CI on the merge result is required for publication; PM live checks and all unresolved proposal/cutover gates remain explicit (§9, R19).
