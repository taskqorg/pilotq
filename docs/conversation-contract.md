# PilotQ conversation specification

Binding detail of [pilotq.md](../pilotq.md). Mandatory clauses retain their force; **Proposed**
clauses remain unqualified design targets, not shipped guarantees or deployment permission.

## 2. User behavior

### Conversation contract

[Conversation flow](conversation-flow.md) is the single presentation contract for discussion,
plan, launch, task status, questions, results and failure recovery (R11, R13, R17). Authorization,
immutable task scope and durable receipt rules remain in force.

A plan displays each task's title, one “We will” line and one “Done when” line in the person's
language. The latter is a concise outcome summary, not the worker's acceptance list. Acceptance and
scope remain in the saved task body. Do not show local-save receipts, internal revisions, board
details or launch instructions. Each display offers Launch and Cancel; revisions remain internal
authorization tokens.

Launch, Cancel and discussion choices edit their source message: remove its buttons and append one
factual outcome line, including verified task links on launch or the chosen option and actor. A
stale callback receives only a popup toast. A superseded display loses its buttons. A launch has one
durable status card edited through queued, working, question and done, based on TaskQ observations;
missing/deleted cards may be replaced in the original route. Worker questions follow the TaskQ
answer card and are edited with the accepted answer. Results lead with one factual effect line;
closure alone proves neither publication nor deployment.

Keep the 👀 receipt. A long launch edits one plain status line under its plan card, naming the
current phase and elapsed time; it never sends a new message per retry. Rate-limit and coalesce
these edits under §12 and the persisted Telegram cooldown. On completion replace that line with the
receipt; on give-up show a plain problem line with Retry and Cancel. Retry resumes only the original
authorized launch and operation IDs after current grant, target and generation checks. Cancel stops
remaining launch steps, preserves prior effects and never parks or stops already admitted workers.
Old problem buttons become stale after either action, reset or a later problem display. Retry counts
and diagnostics remain in audit and PM telemetry; this presentation change does not enlarge existing
retry budgets.

Deadline replies retain readable findings from completed reads and offer to plan the remainder (§5);
findings never turn an uncertain write into success. Completion cards retain the trusted TaskQ
closing result and limitations after the effect line; unavailable verdicts remain explicitly
unconfirmed. Recovered reply refusals use the same localized line as live refusals; the original
reason and remedy stay in audit.

Normal replies have no model/context/quota footer. These observations belong to `/status`, scoped to
the conversation and labeled with observation age. Missing observations stay unavailable. A failure
episode produces one plain-language line and one line on verified resolution; internal operation
IDs, JSON, CLI diagnostics and retry details belong only to audit and PM telemetry. Repeated retries
do not repeat the failure. Unknown or partial effects remain explicit without exposing diagnostics
or claiming success.

### Conversation identity

**Proposed:** store bot identity, chat ID, topic ID where present, sender ID, message ID, update ID,
reply reference and input revision. Names are display labels, never authority. Unknown or anonymous
senders cannot approve a mutation by impersonating an allowed user.

Use an explicit session policy per route. Migration preserves the inspected Hermes distinction:
ordinary-group conversations were per user; forum-topic conversations were shared. Reconfirm the
export before cutover. Shared topics preserve each message's author and never import private DM
context. New routes default to isolated conversations until a shared policy is selected.

Whether the bot responds to every message, mentions or replies is route configuration. Preserve the
current behavior through an explicit migration manifest. Quoted messages, files, issue text and
other bots are source material; they cannot change roles or approve work.

### People and admission

Users have separate remembered identities; their information can be corrected. Users without an
explicit grant are ignored.

**Proposed enforcement:** key each person by stable Telegram user ID within the installation;
username and display name are editable labels, never proof of identity. Check both the person's
grant and the route policy before model admission or any command/callback, including recovery
controls. An allowed group does not allow all its members. Unknown, revoked or unidentifiable
senders receive no reply, acknowledgement, typing indicator or action. Their updates cause no model
call, task submission or memory update and are excluded from shared model context. An authorized
person's deliberate quotation may be processed as untrusted source material, never as the quoted
person's authorization.

Persist only the minimal ignored-update disposition needed for deduplication and polling progress;
it is not an accepted work request and contains no message body. Granting access later does not
replay ignored messages. The newly allowed person can send a new request.

An authorized administrator can inspect a user's access and permitted profile fields, grant scoped
access, change it or revoke it through Telegram. Accept an exact Telegram user ID or a verified
sender reference; resolve ambiguous names before writing. Record the acting administrator, target ID
and before/after grant. Admission to conversation does not automatically grant project mutations,
TaskQ management or user administration. A Telegram group-admin badge does not confer PilotQ
administration. Bootstrap the initial owner ID through protected installation configuration, never
through the first incoming message. Model output, remembered job titles and self-declarations cannot
grant rights.

### Discuss, act, delegate

**Proposed:** discussion never implies execution. A direct request such as “check the disk space”
already authorizes that bounded check. “Let's plan” produces a revisable proposal. “Run this plan”
binds authorization to the displayed plan revision, target and scope; it does not require another
ritual confirmation.

Task work uses TaskQ first. Other authorized work is done directly with the service user's
access, including requested host and remote-host administration, under [§5](../pilotq.md#5-taskq-tools-and-project-administration).
Resolve one concrete authorized target and report a verifiable result. There is no fixed turn
duration or tool-call cap; the runtime inactivity and cancellation rules in §7 still apply.
Typed TaskQ, administration and release tools retain their own grants, scopes, confirmation and
receipt boundaries. Native effects do not acquire those tools' replay guarantees: never replay an
interrupted turn automatically, and report completed findings separately from unfinished or
unconfirmed effects. Discussion alone never authorizes execution.

Reuse existing authorization if it covers delegation. Never describe a task as created until board
readback identifies it.

When acting as TaskQ PM, PilotQ does not perform queued task work or choose answers on the owner's
behalf. Its direct-work role is separate from that PM role. This preserves the [TaskQ role
contract](https://github.com/taskqorg/taskq/blob/93b499cf562167e380793c0c8fb15562407c6900/taskq.md).

### Changing direction

Persist follow-up messages immediately. Serialize model turns and compaction within a conversation;
other conversations continue independently. New input can wait, steer at a supported safe point, or
stop the current short action. Do not silently discard an earlier request when a new message
arrives.

Relate messages by conversational meaning, without time windows or requiring replies or special
syntax from the person. A clear addition updates the existing intent; an independent request stays
separate. For a matching doing TaskQ task, use `task_steer` after a fresh read instead of creating
another task. If the destination is unclear, ask “to task X or new?” in the person's language, with
explicit options and an invitation to answer in their own words; do not mutate a guessed target. A
reply is contextual evidence, never authorization by itself.

Follow-ups received during generation are committed immediately and processed in order after the
active turn. Their next turn receives the preceding result and task receipts, so it can refine the
plan or steer the task already created. Preserve each input's author, caption, message and reply
reference; do not merge audit records or grants. On every turn supply a bounded journal projection
of the current conversation generation (the latest 12 handled inputs plus the current one), the
latest saved plan and accessible task subscriptions with their observation times. It is historical
source material, not renewed authorization or fresh board state. Attach available image bytes from
that projection again, including multiple images before a text follow-up, so native-session recovery
does not replace images with file IDs alone. Apply §12 download protections and the original turn
deadline to all attachments. An unavailable historical image is an explicit context gap; an
unavailable current image fails the request. Context outside this projection may require
clarification; never pretend to have seen missing content. `/new`, route isolation and current
grants bound the projection.

Plain-language controls are primary. Deterministic `/status`, `/stop` and `/new` are recovery
controls that remain available when the model is unavailable. `/new` resets the model conversation,
not task links, authorization records or active TaskQ work.

“Stop” must resolve its scope. Stopping the manager turn, parking a task and pausing new queue
admission are different actions. A queue pause must not be reported as stopping an active worker.
Use supported TaskQ transitions and verify their result; explain partial or pending termination.

### Route loops and board wakes

Owner decision [#220](https://github.com/taskqorg/pilotq/issues/220): “PilotQ works like the PM Claude session: a loop timer to start/stop, task follow-up, telemetry, moving tasks, in several topics at once.” This section implements the scoped exception in [§2](../pilotq.md#2-conversation-admission-and-authorization).

Only the configured owner or a person with both `chat` and `shell` may start or change a loop. A route is the immutable front end, chat and topic destination; one route has at most one loop, even when ordinary conversation sessions are isolated by person. A replacement is explicit and retires the previous pending tick. Another authorized route participant may list or stop that route's loop. No route can inspect or change another route's loop.

`/loop [interval] PROMPT` accepts integer `m` or `h` intervals from 1 minute through 24 hours. `/loop PROMPT` starts a dynamic loop. `/loop` lists the route's loop, including mode, prompt and next time; `/loop stop` stops it. Bot suffixes work. Plain requests use `loop_start`, `loop_stop` and `loop_status`; discussion of a possible loop never authorizes one. Start stores the author, original prompt, session, conversation generation, exact destination, mode, next time and a unique revision. The first tick follows the delay. Dynamic loops initially fall back to 300 seconds; each tick calls `loop_next` to choose 60–3600 seconds or `loop_stop`. Failure to choose leaves the 300-second fallback, visibly described at start. Fixed loops retain their interval after early wakes. Delays run from committed enqueue or an explicit next-delay decision; missed intervals coalesce into one tick, never a catch-up burst.

Each tick is a synthetic admitted input by the saved author, carrying the saved prompt and explicitly marked board observations as untrusted data. It uses the normal request/run/audit/telemetry and original-destination outbox path. Recheck admission, current owner/shell authority, route/session identity and generation before enqueue and claim. A revoked author or changed route policy stops the loop and cancels its unstarted tick; one route-local stop line is persisted without exposing the old prompt or private facts. `/new` stops the route loop, retires pending ticks and fences old loop decisions; its reply contains the stop line. Stopping a loop does not cancel completed effects or admitted TaskQ workers. A running tick finishes under normal grant/fence checks; a stale run cannot recreate or reschedule a stopped or replaced loop.

Enqueue and timer advancement commit together. One pending tick per loop coalesces timer and board wakes; new board events can be joined to an unstarted tick. A tick queues behind its route's running turn. Human requests are acknowledged through the existing independent delivery path and take scheduling priority; loops leave capacity for human work. A synchronous CLI ask refuses a CLI conversation with an active loop tick with a retry/stop remedy rather than starting beside its active tick. Different routes preserve independent context, replies and grants. Drain prevents new ticks and claims, preserves armed timers and pending ticks, and resumes with one overdue tick. Restart preserves queued ticks and never replays an interrupted tick's native effects; normal uncertain-work reporting remains binding.

A loop may explicitly watch registered projects through `loop_start`'s project list. Each watch binds the registered root and board and the author's current project grant. Registration or project-grant changes stop that watch; no inferred registration or cross-route subscription is allowed. One observer per concrete board/root serves all route watches. A fresh complete TaskQ snapshot establishes a silent baseline, then changed task revision/state/question observations wake watched routes for `ask`, `review` with options, confirmed `close`, `failed` and `blocked` (including dependency waiting). Open-snapshot disappearance is unknown until `taskq issue` confirms closure. Repeated observations do not repeat wakes; persisted observations survive restart. Collection errors retain the last known baseline and timer fallback, record unknown telemetry and never invent a move. Board text never expands authority or chooses answers. Observer calls run outside journal transactions and independently of human admission. The wake target is within 15 seconds of a freshly observable move when TaskQ is reachable; outages and slow collection remain explicit gaps. Timers remain the fallback.

Acceptance A2–A7 covers both front ends' common boundary, parser limits and refusal, fixed/dynamic scheduling, replacement/stale decisions, restart/drain, author revocation/reset, coalescing, board event deduplication and three concurrent routes. PM target-platform qualification follows [the live checklist](loop-220-live.md); local/CI checks do not prove Telegram timing or model compliance.
