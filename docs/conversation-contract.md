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

Direct work must satisfy all of these conditions:

- One clear authorized target and a bounded result.
- No need to survive as a long-running job, produce a reviewed implementation, or coordinate
  dependencies.
- A read-only operation or a small mutation with an explicit grant and a verifiable outcome.
- Execution fits the short-action budget in section 7.

Examples: inspect one service, read a bounded log interval, look up a ticket or explain an error.
Apart from the scoped administration and owner release operations in section 5, the only direct
writes are the owner's board comments and labels ([section
5](integration-contract.md#direct-actions-keys-and-project-switching)). A fast action can still be consequential:
deployment, money, permissions or deletion do not become direct-safe because they take seconds.

Multi-file implementation, long research, broad audits, repeated experiments, deployment work and
uncertain-duration work go to TaskQ, except the owner-confirmed registered release operation in §5.
On reaching a direct-action budget, preserve observations and completed effects; propose the
remaining task. Reuse existing authorization if it covers the delegation. Never describe a task as
created until board readback identifies it.

When acting as TaskQ PM, PilotQ does not perform queued task work or choose answers on the owner's
behalf. Its short-action role is separate from that PM role. This preserves the [TaskQ role
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
