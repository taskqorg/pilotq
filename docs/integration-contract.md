# PilotQ integration specification

Binding detail of [pilotq.md](../pilotq.md). Mandatory clauses retain their force; **Proposed**
clauses remain unqualified design targets, not shipped guarantees or deployment permission.

## 5. TaskQ integration

PilotQ reads and manages boards only through the installed TaskQ CLI (R13). A registered project is
the local root where TaskQ runs; TaskQ owns forge, host, authentication and board access. The
registry binds that root to the board identity read back by `taskq status --json`, without a forge
adapter or credential configuration in PilotQ. Use task URLs returned by TaskQ, never construct or
interpret forge-specific issue URLs. Read task progress with `status --task` and accepted answers
with `inbox --json` (actor, time and original question revision); writes use the supported `answer`
and operation-receipt path. An unreadable response stays unknown. Missing TaskQ capabilities become
TaskQ findings, never a PilotQ forge integration. Existing direct issue tools remain refused when
TaskQ has no equivalent; their grants do not authorize a forge bypass. `issue_read`, `issue_comment`
and `issue_label` use `taskq issue N` (TaskQ 4.6.44): reads take its `--json` issue JSON; writes run
`taskq issue N --comment T|--label L --op-id ID --json` and settle from its readback receipt. A
board registration grants no server access or worker capacity. The PM registers tight on PDS 18 and
records the A3 chat readback after release; workers verify the implementation locally and in CI and
make no installation changes.

**Proposed:** wrap the supported CLI with argument arrays and typed validation; no shell
interpolation of Telegram text. Use its JSON status and existing task log facilities. Do not create
a new public TaskQ API before evidence shows the CLI cannot support the required flow.

- Read: registered projects, fresh status, task details, current question and attempt evidence.
- Write: create a held task, start it, answer the current question, steer, requeue or park within
  the user's scope.
- Exclude from ordinary manager tools: host grants, capacity changes, arbitrary closure/publication
  and forced cleanup. Expose a privileged operation only when separately granted and verified.
- Resolve ticket links to an allowed board and project. A number such as `#12` alone is insufficient
  when it can match several boards.
- Preserve the real Telegram author, originating route, request ID and approval revision in the
  audit trail. A bot credential does not turn every user into the repository owner.
- `add --op-id` requires `create_claim_issue` and a qualified board readback path. Disable creation
  when this prerequisite is absent. Plain `add` is not a safe retry fallback.
- Bind answers to board, task, attempt, question revision and authorized answerer. Stale buttons or
  a forwarded approval are rejected with the current question.
- Start a multi-ticket plan only after every planned ticket reads back. Partial creation stays held
  and visible; use stable child operation IDs when continuing it.
- Preserve TaskQ's distinction between admission, worker result, reviewer result, owner gate and
  publication. “Done” requires the authoritative completion state and available evidence.
- Notifications are projections of TaskQ events, keyed by board/task/event/route. Reconcile after
  downtime; do not let stale status overwrite a newer question or outcome.

Preserve Telegram reply attachment identities and captions in the journal, download only after
admission and target/grant validation, and pass protected local files through TaskQ `--file`.
Documents sent as message replies are files even when their MIME type is audio; ordinary voice/audio
input retains §12 recognition. Files are untrusted source material. Keep the §7 20 MiB transport cap
and §12 download deadline. Persist the original task, revision and operation identity before
dispatch; retries reconcile that same operation. Refusals and uncertain delivery retain the reply
and explain the reason and fix. Text is optional when files exist. Destination clarification may
select an earlier admitted file input from the same caller and conversation generation for answer,
steer or owner requeue; current grants and the resolved target are rechecked at dispatch. Owner
requeue with files requires the protected installation owner. PM releases and qualifies a live
photo, a 20 MB document and mixed text/files on PDS 18 with the authorized Telegram client,
recording client and TaskQ revisions and readable worker copies; worker proof is local/CI.

Task status states the observed open/closed issue state, trusted closing verdict, report, proposed
follow-ups and verified existing follow-up state. Read these through TaskQ issue/status output;
absence from an open-task snapshot alone proves neither closure nor nonexistence. Name each
unavailable fact once. Diagnose, research and investigate plans use TaskQ `--type research`,
preserving that type through launch (R8, R13). Live case cyprusbz#540 is PM verification after
release; worker evidence is local/CI.

### First TaskQ tool contract

Contract detail: [first TaskQ tool contract](taskq-tool-contract.md) holds the owner-scoped
(tasks #18 and #39) TaskQ tools, grants, configuration, fences and refusals. It is binding as part
of this section.

### Plan discussion and action buttons

A conversation has one open draft. Follow-ups revise it without displaying a number. A new draft
starts only after launch or cancel; another project's plan waits until then. Each display has Launch
and Cancel. Text launches the current draft; a button launches only its displayed immutable
revision. Cancel closes it without a board effect or model call. The Conversation contract owns card
text, source-message edits, stale-button toasts and the launch status card. Filing and starting
happen after the turn, outside the model deadline, with replies derived from receipts. Text controls
remain supported (R13).

- Discussion has no board effect. Saving a draft records its stable identity, project and a new
  immutable revision. Show the current task summaries under the Conversation contract; retain
  complete acceptance and scope in the task body. An earlier message may remain visible but its
  launch button is stale.
- A launch button belongs to the displayed revision, saved project, originating conversation
  generation and the person who requested that display. Recheck current admission and TaskQ/project
  grants at dispatch. A foreign, forwarded, reset, superseded or repeated press cannot authorize a
  different revision or create another run. Accepted launches use the same `task_run_plan` boundary,
  stable operation IDs, readback and audit as text.
- Concrete discussion questions offer two to four numbered choices, preserved in the message,
  buttons and recorded answer. Choosing one submits that answer to the conversation; it never
  authorizes launch. Launch remains a separate button or explicit later text request. Old or already
  answered choice cards are refused.
- Example: “Plan reply improvements”, “also keep line breaks”, “and add buttons” produces one draft
  with three internal revisions, no board tickets. The latest display has “Run plan”; pressing it
  launches only that display. Pressing either an earlier display or the same button again starts
  nothing new. Under #110 a separate idea waits for launch or Cancel, then starts a new draft. After
  launch, “also support audio” starts a separate discussion, preserving the launched scope.

### Human trial

The human trial ran with the owner and Simon, the two administrators, on the separate test bot from
2026-10-27T15:15Z to 2026-10-29T20:46Z. Its report covers request IDs, outcomes, refusals, failures
and tester feedback; discovered defects are filed as held tasks. Naming administrators does not
itself apply grants or expand authority.

Human feedback supplements scripted-run evidence. Keep [gate 4](acceptance.md#selection-gates)
separate and preserve its existing pilot start. Hermes production cutover remains outside scope (§10
step 6). See the [trial cases](acceptance.md#human-trial-task-42).

### Direct actions, keys and project switching

Direct actions are reads. Only the owner (`owner_id` from protected configuration) may write
directly, and only a comment or a label on an issue of the `taskqorg/pilotq` board; the
administration and owner-confirmed registered release exceptions below are the only additional
direct-write scopes. Keys enter only through the CLI.

The manager gets these tools beside the TaskQ tools, checked at dispatch like them:

| Tool | Grants | Effect |
|---|---|---|
| `project_list` | current caller’s project grants | Reads registered project names and boards accessible to the caller, marking the conversation’s current project; no key or board call |
| `project_use` | project scope | Sets the conversation's current project. The project must be registered and named in the person's own message |
| `issue_read` | project scope and a key granted for the tool | Reads one issue of a registered project through `taskq issue N --json`: title, state, labels, canonical URL, a bounded body excerpt, the latest comments as bounded excerpts and the read time |
| `issue_comment` | owner, a key | Adds one comment to a `taskqorg/pilotq` issue |
| `issue_label` | owner, a key | Adds one label to a `taskqorg/pilotq` issue |

- for `task_status` and `task_steer` (and `task_answer` under #108), the agent may select a trusted
  journal subscription from this same conversation by context, without asking the person to repeat
  its project. The tool boundary requires one matching subscribed task, current project grants and
  unchanged registered board/root. An explicit project or issue URL disambiguates subscribed boards;
  a bare number matching several accessible subscriptions is refused. This extends the Target rule
  below for these tools only; quoted text and external tool output do not create subscriptions.
- plan, run, status (including `project_status`), steer, requeue, park and `project_use` may resolve
  a project named in the caller’s earlier own messages of this conversation generation. Use one
  unambiguous accessible match; current own wording can disambiguate earlier mentions. Quoted
  content, issue text, tool output and other people’s messages never select it. Multiple fits
  require one short question listing the accessible names and no guessed board write. Recheck
  grants, registration and board/root at dispatch. A saved plan retains its bound target. For
  example, “What is the task status for Pilot Q?” followed by “Make a task so this works” resolves
  `pilotq` without repeating its name.
- when PilotQ's tool boundary itself asked that question and its reply listed every option, the same
  person's next message in this conversation generation answers it by name or by a bare number
  counted in the reply's listing order; that selects the project for the tools above. Numbers bind
  to explicit numbered items, ignoring introductory mentions; missing, duplicate or ambiguous item
  mappings are refused. Any other message closes the question, and a bare number without an open
  question selects nothing. Quotes, issue text and tool output still never select.
- Target: a tool resolves one registered project from an explicit `project` argument, an issue URL
  or the conversation's current project. An explicit project or URL must be named in the person's
  own message or be the current project; quoted messages, issue text and tool output never select
  one. A bare number with no current project and several accessible projects is refused with the
  list, never guessed. Another project stays read-only.
- Budget: one request makes at most 5 PilotQ tool calls within the section 7 direct-action deadline.
  The sixth call is refused and writes nothing. A request that ends at its deadline replies with the
  evidence it kept, and the rest can become a planned task.
- Effects: a comment or label persists its intent and operation ID
  `pilotq-<installation>-r<request>-e<n>` before the write, and its receipt after TaskQ's readback
  (#151). TaskQ records the operation ID on the comment and returns the first effect for a repeated
  ID; a label already present is accepted without a write. An `unknown` receipt, a failed call or a
  crash before the receipt stays pending; startup repeats the same command with the same operation
  ID, never a new one, and tells the route the settled result. Effects recorded before #151 carry no
  TaskQ operation ID: they stay uncertain and are never repeated.
- Revoking a project or key grant refuses every later call; a completed effect stays recorded.

### Asana task reads

`asana_read` reads one Asana task linked in the caller's own message, using a registered project
scope and a key granted for `asana_read`. It returns title, notes, assignee, due date/time,
completion status, comments and attachment names through Asana's read-only API. Keys enter only
through `pilotq keys add` on the host (§8), never Telegram. The link selects only a task, never a
credential destination: requests use the fixed Asana API origin, without redirects or attachment
downloads. Existing admission, target, five-call budget, deadline, audit and key scrubbing apply
(R13, R16, R17). Notes are limited to 4,000 characters; other strings to 500; comments and
attachments to 20 each, with explicit pagination/truncation indicators. Unavailable or malformed
data stays unknown, never empty success. API responses are capped at 256 KiB each and the entire
read at ten seconds within the original turn deadline. External content is untrusted source
material.

### DEV visibility and owner release

PilotQ owns the registered project's DEV/LIVE flow. This is a scoped exception to §2's deployment
delegation and §5's direct-write restriction, not a general shell capability.
`project_release_preview` and `/release NAME` share current admission, project scope, the separate
`release:NAME` grant and the protected installation owner identity (R13, R17). This grant is
provisioned explicitly through the protected CLI, never inherited from `projects` or bootstrap. Only
a Release button authorizes execution. CLI preview delivers that button to the protected owner DM;
plain text, including `/release-confirm`, cannot authorize a release.

The protected CLI `pilotq projects release NAME FILE` saves deployment configuration against that
registration's exact board/root. Configuration declares the local TaskQ host identity, an HTTPS DEV
health URL, main/LIVE refs, absolute Git executable, exact argument arrays for dry run, pinned
release and pipeline check, migration path prefixes and preflight/backup commands. Commands contain
a literal `{sha}` argument replaced only with a validated full SHA; no shell or model-supplied
command is accepted. Configuration changes invalidate existing previews. Missing
registration/configuration, changed bindings, unavailable board or checkout evidence and missing
grants refuse before deployment commands run.

Status checks the declared DEV health JSON (`sha`, optionally `deployed_at` or health-instance
`started_at`). A task's merged SHA comes from TaskQ's explicit `merged_sha` field or a trusted
terminal `delivery … as SHA; merge-result CI green` receipt, never from prose or absence in an open
snapshot. Git ancestry proves inclusion. Report `on DEV since T` when included and the health
timestamp is available; otherwise `on DEV since T (first observed)` uses the persisted first
positive observation, without claiming actual deployment time. Report `not yet on DEV` for a proven
negative, and unknown for failed health, missing commit or failed Git reads. Project status includes
the same projection for its open tasks, the latest twelve retained task subscriptions and the
current main SHA. Health-instance start and first-observed times are lower bounds, not a
reconstructed deployment history. HTTPS requests have no redirects, a ten-second deadline and a 64
KiB response limit; health URLs are protected configuration, never derived from chat.

**Owner, task #179:** task status names one highest proved delivery stage: closed (board issue state), merged (trusted TaskQ merge identity), on DEV (successful deploy job), DEV smoke (successful smoke job), or on LIVE (Git ancestry against the configured LIVE ref). Open issues remain open. Read delivery jobs through an optional protected `delivery_status` argument array with a separate `{sha}` argument, using the same executor, root, requirements, ten-second deadline and 64 KiB limit as the release adapter. Missing configuration, failed reads or malformed evidence name the unavailable source; worker notes never replace pipeline facts. Existing health observations remain a separate observation, not job evidence.

The delivery adapter returns `{ "sha": "<requested merge SHA>", "merged_at": "<time>", "first_main": { "sha": "<pipeline SHA>", "state": "success|failed|running|pending|canceled|skipped", "url": "https://…", "deploy": { "state": "…", "url": "https://…" }, "smoke": [{ "state": "…", "url": "https://…", "first_failing_check": "<required for failed jobs>" }] }, "later_main": null }`. `first_main` is the first main pipeline containing the merge; smoke lists its attempts in order. `later_main`, when present, has `sha`, `state` and `url` for the latest later main pipeline containing it. Validate full SHAs, HTTPS links, states and Git inclusion; missing facts are unknown, never empty success. Only a successful containing main pipeline proves main CI passed. Name failed smoke attempts with their job links and first failing checks, and state whether the later pipeline passed. State each fact once; closing limitations remain historical and cannot override these observations. Internal recovery progress stays in audit, with no chat message. PM verifies a recently closed cyprusbz task in the owner topic against GitLab after release and records the request audit ID; workers verify locally and in CI (§9).

Preview runs the configured dry run, pins main and LIVE to full commit identities, and shows target
SHA, commit count/authors since LIVE, verified TaskQ task URLs with explicit merged SHAs in that
range, migrations yes/no and DEV inclusion. Missing facts refuse preview rather than showing empty
success. A preview expires after fifteen minutes, supersedes earlier previews in that conversation
and binds owner, generation, registration, configuration and source card. Foreign, stale, repeated
or reset confirmations run nothing. Main advancing does not retarget a preview; LIVE advancing
requires a fresh preview.

Before preview and again before dispatch, read every explicitly registered checkout on this host
through TaskQ. A running execution on another checkout with the same Git origin blocks release,
naming project, task and attempt; unavailable or incomplete evidence blocks rather than assuming
idle. Do not discover other directories. The protected configuration names the only release root;
never use Tight's checkout for cs.

When migrations are present, Release also authorizes the configured promotion preflight and LIVE
backup, in that order; both must succeed before the pinned release command. Persist one immutable
confirmation intent before any command, then each stage receipt. Execute without a shell in the
registered root only. Release execution runs outside the model turn, under a fifteen-minute total
deadline and a repository lock; planned update drain refuses while it is active. Revoked access
stops remaining stages. An interrupted/timeout/ambiguous command stays uncertain and is never
automatically replayed. Repeated confirmation cannot repeat prerequisites or release. Pipeline
readback must bind the pinned SHA and report `release`, `smoke`, `backmerge` stages with validated
HTTPS links and the LIVE SHA; only all successful stages and matching LIVE SHA prove completion.
Missing or pending evidence stays unknown. Startup preserves intents and reports uncertain
interrupted operations through the existing outbox.

For cs, DEV is `https://dev.cyprus.bz/__health`; commands are `npm run release -- --dry-run`, `npm
run release -- {sha}`, `npm run db:promotion:preflight` and `npm run db:live:backup`. The
installation owner supplies the pipeline read adapter and executable paths. Every executable
declares a minimum version in the registration `requirements` map and supports `--version`; Git
requires at least 2.25.0. Doctor checks these alongside §9 host requirements. That adapter returns
JSON `{ "sha": "<pinned SHA>", "live_sha": "<SHA>", "stages": { "release":
{"state":"success","url":"https://…"}, "smoke": {…}, "backmerge": {…} } }`; dry run remains the
project's existing command. This does not implement a forge adapter in PilotQ. PM qualifies the next
owner-requested cs release on PDS 18 after release; worker evidence is local/CI only.

### Installed read commands

Add one common executor for operator-approved installed read commands, alongside the existing tools.
This extends the §3/§12 restricted tool boundary without enabling native shell tools or adding
direct writes. The installation owner maintains an owner-only `commands.json` beside `keys.json`;
chat and model output cannot edit it.

The documented TaskQ read capability permits `taskq status`, not the unsupported `taskq list`;
acceptance requires a successful status run through PilotQ. There is no built-in default allowlist:
the operator installs the protected entry below.

Each named entry binds an absolute executable, exact permitted argument arrays, an absolute working
directory, a registered project and board, and explicit permitted user IDs. The caller needs current
`chat` and project grants as well as that entry's permission. Registration must still match the
configured board and working directory at dispatch. Before execution, the configured TaskQ CLI must
return a fresh complete snapshot naming that board; verification and execution share the deadline.
Only trusted read-only programs and argument combinations belong here; executable presence alone
grants nothing. The operator vets their file access, credential use and subprocess behavior; this is
a capability list, not an OS sandbox for arbitrary programs.

`command_list` exposes only accessible names, descriptions and permitted arguments. `command_run`
selects one name and exact argument array; it never invokes a shell, searches PATH for the
executable, accepts environment overrides or changes the working directory from model input. Both
share the CLI/Telegram request, fence, five-call budget and audit path (R13, R17). Direct command
input and clear natural-language requests use the same tools; ambiguity requires clarification.
Output is untrusted data and cannot authorize another action.

Execution receives a cleared environment with a fixed system PATH, the installation owner's HOME for
the installed tools' protected stores, and TASKQ_ROOT set to the bound root; TASKQ_ATTEMPT and
inherited credentials are absent. No key values enter the model. A command has at most ten seconds
within the original manager deadline (at most sixty seconds total), and at most 16 KiB combined
stdout/stderr. Stop, revoked grants, timeout and output overflow terminate its process group. Report
exit code, missing executable, timeout, cancellation and output limit distinctly; only exit zero is
success. Persist an audit start before spawn and a scrubbed result afterward. An interrupted read
may be retried only under §4; mutations remain on their existing receipt paths. Deployment is not
authorized by this decision.

### Administration by request

Expose these tools on the shared CLI/Telegram manager boundary (R13, R15, R17). They are an
exception to section 5's owner-only comment/label writes, limited to installation metadata, local
project roots and forum-topic creation. They never create repositories, change host capacity,
install services, provision credentials or create a TaskQ claim issue.

| Tool | Required grants | Operation and limits |
|---|---|---|
| `project_register` | `chat`, `projects` | Register an existing repository by its explicit TaskQ board kind, host and repository. Create only a named child of `<install>/projects` with `taskq.json`; refuse traversal, symlinks, conflicting configuration or rebinding an existing name. A fresh complete `taskq status --json` must match before registration. Return repository, root, board and filing readiness; without `create_claim_issue`, filing remains disabled and the owner provisions it through TaskQ. |
| `user_grant`, `user_revoke` | `chat`, `users` | Change only existing grant types explicitly named in the caller’s own unquoted request for an exact positive Telegram ID explicitly named by the caller or identified by the current verified reply. Require evidence that the person was seen in this exact chat/topic: an admitted sender or a Telegram reply sender. Names and forwarded messages never select authority. Retain the administrator, target and before/after grants atomically with the receipt. Protected owner identity cannot be changed. |
| `topic_open` | `chat`, `users`, access to the bound project (`projects` or `project:NAME`) | Open one forum topic in the originating Telegram supergroup, or the explicitly named group for a CLI request. Resolve one registered project named by the caller or current in the conversation; recheck its board/root. Require a fresh bot `can_manage_topics` permission and forum capability; otherwise refuse plainly. Set isolated route policy by default (shared only when explicitly requested), and bind the new route to that project as its default. A route default never grants project access. |

Each tool shares the five-call limit, original deadline, current admission, run fence and request
audit. Store one immutable operation per tool per request; a changed argument set in that request is
refused. Read back local changes before success. Duplicate calls return the saved receipt without
applying the change again. Project preparation persists intent before filesystem writes and can
reconcile only the same matching configuration. Topic creation has no Bot API idempotency key:
persist intent before dispatch and its topic ID immediately on response. An interrupted creation
without that ID is uncertain and is never automatically repeated; an ID recorded before route
binding permits completing only that local binding after grant revalidation. Startup preserves
unresolved operations and reports their partial or uncertain result. Unknown senders still follow
R10; the service does not retain ignored message bodies or build profiles to make them grant
candidates. An administrator may reply to that person's message to supply the verified sender
reference.

### Project registration

An administrator with the `projects` grant can ask PilotQ to create or register a project. PilotQ
shows the resolved repository, root and board, then applies the request through TaskQ's supported
registration path (Add project, [taskq.md](https://github.com/taskqorg/taskq/blob/main/taskq.md)
menu M10) and reads it back. Repository creation, capacity and worker grants stay owner operations
unless a separate grant names them. A failed or partial registration stays visible and is never
reported as done.

Step 4 registers through the CLI: `pilotq projects add NAME ROOT` accepts a root whose `taskq.json`
names a repository only when a fresh `taskq status --json` under that root reads back the same
board, and records who registered it. TaskQ's Add project conversation stays the owner's way to
create the TaskQ side; #160 additionally prepares a local root for an existing repository, without
claiming filing readiness.

### Project onboarding

Onboarding covers only explicitly registered roots (§3). Successful `pilotq projects add` and
`project_register` queue onboarding; service startup queues registrations without onboarding
records, and a changed TaskQ installation/service identity queues a recheck of those roots. A new
TaskQ project must first be registered explicitly. No directory discovery or silent enrollment is
added.

Registration and readiness are separate. After registration run `pilotq doctor --json` and the TaskQ
setup check; until taskq#1938 ships use `taskq service status --json`, matching the registered root
and board. Require complete evidence with service and per-root observations no older than 120
seconds (at most five seconds in the future), an active service, usable Codex capacity and an
explicit effective Codex grant without refusal, plus the existing claim prerequisite. A capacity
limit such as 4 never substitutes for a grant. Unavailable checks stay gaps. Setup evidence is not
proof of the full first-task scenario or subscription entitlement.

Check that each registered project has an `issue_read` key grant usable by the protected owner
through both CLI and DM. Project grants and existing owner-wide grants retain §8 semantics; paired
CLI/owner-DM route grants also qualify. Read key metadata only, never return values or update
last-use time. If suitable issue-tool keys exist but lack scope, show the exact `pilotq keys grant
KEY project:NAME` command for each owner-selectable candidate; do not grant automatically. Missing
or unreadable key configuration remains a gap with a named protected-CLI provisioning step. Keep the
2026-10-30 fixture: `pilotq-github` permits `issue_read` but is granted to `pilotq` alone;
`cyprusbz`, `tight` and `agentq` must each report their missing grant.

The protected owner receives one durable Telegram DM card per project. It lists each gap on one
plain line with its exact fix command or a named owner step, and offers Recheck. Rechecks edit that
card; missing cards may be replaced only in the same DM. CLI registration prints the same findings
and returns nonzero when gaps remain, preserving the registered project. `/onboard NAME` rechecks
through the bounded control path; the CLI equivalent is `pilotq ask '/onboard NAME'`. Controls share
current admission, audit and persisted Telegram cooldowns (R10, R13, R17). Buttons bind to the
owner, card, current registration and conversation generation; stale or foreign buttons receive only
a toast.

Existing §5/§9 authority remains unchanged: installation, capacity, worker grants, credentials and
subscription login are owner-only steps, shown with Recheck rather than executable Fix buttons. Any
fix within PilotQ's existing authority requires one explicit OK for its displayed scope and the
existing durable intent/readback path; this flow adds no host mutation capability. The flow ends
either with all checks green or with the named owner-only steps still outstanding. It never calls a
project ready with gaps.

Before PilotQ starts the first task of an onboarding project, require a passed report bound to its
current root and board. Pending, failed or unknown onboarding leaves the launch unsent. The
protected owner may write `/onboard NAME override` to bypass only this PilotQ readiness gate; record
actor, target and findings, keep readiness false and retain TaskQ's own admission, grant and claim
checks. Recheck or changed registration invalidates the override. Existing admitted workers
continue. Onboarding and its report survive restart without replaying mutations.

An operations board is an explicit server-care project. Project work stays on that project's board.
A server-wide overview aggregates tasks; it does not duplicate every issue onto a server board.

TaskQ execution continues when PilotQ is down. If TaskQ is unreachable, conversations continue and
proposed tasks remain unsent. Status shows the last observation time and unknown freshness.
Previously authorized, unchanged submissions can resume when dependencies recover; changed scope or
stale grants require revalidation.
