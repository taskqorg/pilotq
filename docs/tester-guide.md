# Test PilotQ

Use PilotQ to discuss work, approve a saved plan and follow TaskQ in Telegram. This guide describes the first release candidate, `0.2.4`, not production-cutover acceptance ([contract §9](service-contract.md#first-release-delivery)). The owner releases by fast-forwarding `live` after merge-queue acceptance; the download commands work after that workflow publishes the assets.

## Join an existing test bot

Ask its administrator for the bot username, your approved project and the approved group or forum topic. Give the administrator your exact Telegram user ID through the agreed private channel. Group membership, a username and a Telegram admin badge do not grant access (R10).

The administrator grants `chat` for conversation, `project:pilotq` for this board and `taskq` for task mutations. `queue` is a separate host-wide privilege. Send a new message after admission; previously ignored messages are not replayed. Do not send secrets: requests are audited. The configured notice describes retention; `/audit` in a DM shows your own records. A group `/audit` reveals no history (R17).

For an operator, these commands use the installation's configured owner identity. Replace the invented ID `123456789` with the verified tester ID:

```sh
pilotq users grant 123456789 chat project:pilotq taskq --json
pilotq users show 123456789 --json
```

Read back the grants before telling the tester access is ready. Revocation uses `pilotq users revoke 123456789`; completed effects remain recorded. People and profile administration currently use the CLI; a conversational administration request is not evidence that a grant was applied.

## Use a DM, group or topic

- **DM:** open the approved bot and send text, for example “What can you do for taskqorg/pilotq?”
- **Ordinary group:** send text where the administrator has installed the test bot and enabled delivery of the relevant messages. Conversations default to isolated per person, although everyone in the group can see replies.
- **Forum topic:** send text inside the approved topic. Replies and worker questions belong there. A configured shared topic shares that topic's conversation while preserving each author's identity; it does not import DM history.

The operator inspects `pilotq routes list --json`. To select a shared topic explicitly, use `pilotq routes set CHAT:TOPIC shared --json`; replace `CHAT:TOPIC` with verified numeric IDs. `isolated` selects per-person context. New routes default to isolated. Do not expect Telegram privacy mode to deliver every ordinary group message; the operator must verify the chosen bot permissions and message delivery with an allowed tester.

Accepted text and images get an 👀 reaction, with a short text receipt if the reaction fails. Typing and drafts are feedback, not completion. Replies omit internal request numbers. Send a photo or PNG, JPEG, WebP or GIF image document, with or without a caption. Send a Russian voice note, audio file or audio document (OGG, Opus, MP3, M4A, WAV or FLAC) up to 5 minutes; PilotQ answers its transcript like typed text. Other documents and video receive a short unsupported-format reply. Images and recordings are limited to 20 MiB. Rich Markdown keeps paragraphs, lists, headings and bold. Action replies lead with the verified effect and next step; saving a plan is local until explicitly run.

## Plan, authorize and follow work

Use a small, real, authorized change on `taskqorg/pilotq`; do not file placeholder work on its live board.

1. Say “Plan this change for taskqorg/pilotq: … Do not run it yet.” Supply the goal, observable acceptance and scope. Expect one to three proposed tickets and a saved plan revision; discussion creates no board issue.
2. Inspect that revision. Ask for corrections if needed; approve only the latest displayed revision.
3. In a **later message in the same conversation**, say “Run plan revision N for taskqorg/pilotq,” replacing `N`. Expect issue URLs after held creation, readback and start. A plan saved in the same request cannot run.
4. Ask “What is the current state of https://github.com/taskqorg/pilotq/issues/N?” using the real URL. Each filed task has a status message updated in its original route. A verified closing outcome is a separate message; worker completion, reviewer completion and publication are distinct.
5. When a worker question arrives, use Telegram **Reply on that question message** and supply your answer. You need `taskq` and project scope. A forwarded question, another route or an old question revision cannot authorize the answer. PilotQ itself does not answer for you.

If creation is partial or TaskQ is unavailable, keep the request number and observed URLs. The saved plan can reconcile using the same operation IDs; do not submit a duplicate plan to repair delivery. Missing `create_claim_issue` disables creation with a reason (§5).

## Steer, park, pause and recover

- **Steer a task:** ask “Steer https://github.com/taskqorg/pilotq/issues/N: …” with one to four instruction lines. You need `taskq` and project scope, and the task must currently be doing. Expect a receipt and task URL; a stale revision is refused. To restart open work, ask to requeue that URL with a reason; the receipt does not claim the old worker terminated.
- **Park a task:** ask “Park https://github.com/taskqorg/pilotq/issues/N because …”. Expect the supported TaskQ transition and its readback; this is separate from terminating a running worker.
- **Pause admission:** a person holding `queue` may ask “Pause new TaskQ queue admission on this host.” This affects the host, not just this chat. Running workers continue. Queue resume is an operator action through TaskQ; PilotQ has no resume tool.
- **Stop the manager:** `/stop` cancels this conversation's manager work. Check its outcome; it does not park TaskQ work or pause the host queue.
- **Reset context:** `/new` starts a new conversation generation. It keeps audit records and subscriptions to admitted tasks.
- **Inspect:** `/status` uses the deterministic control path even if the model is unavailable. Unknown health or an uncertain effect needs investigation, not another execution request.

Missing grants, a foreign board, stale answers, unregistered targets and the sixth tool call are refusals, not successful actions. Direct issue tools are currently unavailable without a TaskQ CLI equivalent (#144); keys and owner rights never authorize a forge bypass. Direct writes to other projects are refused; TaskQ mutations remain available on explicitly registered projects within the caller's grants. Keys enter through `pilotq keys add` on the CLI, never Telegram (§5, §8).

## Host tools and readiness

The required versions and their consumers are owned by [contract §9](service-contract.md#host-requirements). Use the installation owner's account and the same PATH as the service. Before installation or an update, run `pilotq doctor --json`; inspect every `requirements` entry, including `found`, `minimum`, `outcome` and `fix`. A failed entry blocks service installation and manual/scheduled update checks. The current release keeps running; doctor stays failed until the named step is completed. PilotQ does not elevate privileges or install packages on the owner's behalf.

For an Ubuntu 24.04 managed host, the operator provisions `ffmpeg`, `curl`, `procps`, `hostname` and `systemd` with the distribution package manager, then verifies the actual versions with doctor. If the distribution offers an older version, use the tool's upstream package repository or upgrade the host; do not bypass the check. Install Codex with `npm install --global @openai/codex@0.162.0` (or a newer approved version), set its absolute path in configuration and preserve the existing isolated ChatGPT login. Update the existing TaskQ installation with `taskq update`; verify its service version and preserve its existing host credentials. Do not run GitHub device login.

Install the static no-TTS [sherpa-onnx v2.0.1 Linux x64 release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1), or a newer approved release, with the GigaAM files described below. Set the absolute `voice.recognizer` and `voice.ffmpeg` paths. Doctor additionally decodes and recognizes silence; version checks alone do not establish voice readiness.

TaskQ 4.6.46 or newer exposes its binary version through `taskq --version`; doctor runs it without a receipt, board access or network access. Sherpa-onnx-offline 2.0.1 does not expose its binary version. After verifying the release source and unpacked executable, record its release version and SHA-256 beside the executable. This receipt is operator-provisioned release identity, not independent upstream attestation. Do not infer a version from a directory name or copy a receipt from another executable. Example for the verified sherpa release; replace the invented absolute path with the installed executable:

```sh
TOOL_BINARY=/absolute/path/to/sherpa-onnx-offline
TOOL_VERSION=2.0.1
TOOL_SHA=$(sha256sum "$TOOL_BINARY" | cut -d ' ' -f 1)
printf '{"version":"%s","sha256":"%s"}\n' "$TOOL_VERSION" "$TOOL_SHA" > "$TOOL_BINARY.pilotq-version.json"
chmod 600 "$TOOL_BINARY.pilotq-version.json"
```

Regenerate the sherpa receipt after replacing its executable. Doctor refuses missing receipts, changed bytes and versions below the minimum; the TaskQ service is checked separately. A newer recognizer with a working version flag needs no receipt. Rerun `pilotq doctor --json`, then repeat installation or update. Never alter a receipt merely to silence an unsupported-version failure.

Each protected `commands.json` capability also declares `"minimum_version":"<verified minimum>"`. Doctor runs that executable with the fixed `--version` argument; capabilities cannot supply arbitrary version arguments. For the documented TaskQ status capability, use `"minimum_version":"4.6.46"`. No TaskQ receipt is required. No capability file means no additional requirements; an unreadable or malformed file fails doctor. Adding a capability does not install its dependencies.

## Install a separate Linux test instance

This procedure is for an authorized operator on a separate Linux x86_64 account or host, with `curl`, `sha256sum`, Codex and TaskQ already installed. Use a **separate test bot token** and a verified subscription login. Never reuse the production token. Do not run two pollers with one token (§4). Shared-server subscription entitlement remains an open production decision (§11).

After publication, download and verify the release before executing it:

```sh
mkdir pilotq-download
cd pilotq-download
curl -fSLO https://github.com/taskqorg/pilotq/releases/download/v0.2.4/pilotq-0.2.4-linux-x86_64
curl -fSLO https://github.com/taskqorg/pilotq/releases/download/v0.2.4/SHA256SUMS
curl -fSLO https://github.com/taskqorg/pilotq/releases/download/v0.2.4/latest.json
sha256sum -c SHA256SUMS
chmod 755 pilotq-0.2.4-linux-x86_64
./pilotq-0.2.4-linux-x86_64 selftest
```

All checksum entries must pass. After configuring the host requirements below, `pilotq update --check` on a fresh installation running this same version prints `update: up to date` after fetching the release manifest. An older installed selection sees `update: 0.2.4 available`; `pilotq update` verifies the download and runs offline selftest before activation. A checksum or selftest failure must leave the old selection intact. Selftest checks the local fixture, not Telegram, Codex login or the full task flow.

The default install directory is `$HOME/.local/share/pilotq`. Use the dedicated account's default for service installation; the generated user unit does not preserve an arbitrary `PILOTQ_INSTALL` override. Before starting a poller:

1. Create the install directory with mode 0700. Write its `config.json` with mode 0600 using protected local editing, with `bot_token`, numeric `owner_id`, an absolute `codex` binary path, and `taskq` containing absolute `bin` and `root` paths. The registered root's `taskq.json` must name `taskqorg/pilotq` and have the create-claim prerequisite. Never paste the config into chat or a report.
1. For voice input, install ffmpeg, unpack the `sherpa-onnx-v2.0.1-linux-x64-static-no-tts` release and the `sherpa-onnx-nemo-transducer-punct-giga-am-v3-russian-2025-12-16` model from the [sherpa-onnx releases](https://github.com/k2-fsa/sherpa-onnx/releases), and add `"voice": {"recognizer": "<absolute path to bin/sherpa-onnx-offline>", "model": "<absolute model directory>"}` to `config.json`; optional `ffmpeg` (default `ffmpeg` on `PATH`) and `threads` (default 4). Use absolute paths for all three dependencies, including `ffmpeg`. Run `pilotq doctor --json` after configuring voice; its `voice` check must read `ok`. Without `voice`, doctor fails and voice messages get a short failure reply. A passing silent probe is setup evidence; also send a real Russian Telegram voice note and check the transcript event and delivered answer.
2. Sign in to the approved subscription account with `CODEX_HOME="$HOME/.local/share/pilotq/codex" codex login`. API-key login is refused; no billed API fallback exists (R9). This is Codex login, not GitHub device login. Have the owner provision TaskQ's existing host GitHub credential through its protected store.
3. Register the project with `pilotq projects add pilotq ROOT --json`, substituting the real registered root. When running the downloaded binary before installation, substitute `./pilotq-0.2.4-linux-x86_64` for `pilotq`. Read back `projects list`, grants and route policies.
4. Run `./pilotq-0.2.4-linux-x86_64 service install` only after explicitly authorizing this test instance. It installs and starts the coreq user service. Inspect `pilotq service status` and `pilotq status --json` using `$HOME/.local/share/pilotq/pilotq` if it is not on PATH.
5. Verify bot identity and route delivery with an admitted person's DM, group and topic inputs. Then perform the authorized TaskQ flow above. A CLI-only answer or fresh health file does not prove Telegram acceptance (R13).

`pilotq doctor --json` checks bot identity, the isolated ChatGPT login and a complete TaskQ board snapshot and local voice readiness without polling or starting a model turn. Voice readiness requires configured ffmpeg and recognizer executables, the four GigaAM model files and a successful decode/recognition of one second of silence within ten seconds; an empty transcript is normal for this probe. A failed or unavailable check returns nonzero; this is setup evidence, not full-flow acceptance. The installed Linux service checks releases every five minutes; failures retain the old release. Updates stop new manager turns and wait up to 65 seconds for running requests before activation. A successful scheduled update exits for the native unit to restart through the launcher. `pilotq update --pause` pauses future automatic attempts; `pilotq update --resume` enables them again. Health and conversations continue while paused. Manual updates still work. Publication alone does not prove a host updated; the operator checks its reported version.

## Report a problem and preserve work

Report to the test administrator or [PilotQ board](https://github.com/taskqorg/pilotq/issues), with version, request number, task URL if any, route type, expected and observed result, UTC time and whether an external effect might have occurred. Share message content or route/user identifiers only through an approved channel. Never include a token, key, login/session file, full configuration or raw journal.

The operator can use `pilotq status --json`, `pilotq audit --request ID --json` and service logs locally. Distinguish unavailable observations from healthy ones. Do not resend execution because a Telegram reply is missing: delivery may be pending or ambiguous while the task already exists. Preserve uncertain effects and original destinations; never redirect private output to another chat.

For a failed test installation, stop it with `pilotq service stop`, verify the poller stopped and retain accepted requests, the outbox and protected configuration. Preserve a supported SQLite snapshot before any restore; copying a live database file alone is not restore proof. The first fresh install has no previous release. An existing install keeps `previous`, and the coreq launcher can fall back after three unhealthy starts; only use a previous binary compatible with journal schema 12. There is no PilotQ rollback command. Do not delete the journal, resubmit admitted tasks or restart another consumer as a rollback shortcut (§4, §12).

Task #68 reply checks: an accepted text gets an 👀 reaction, with a text receipt only on reaction failure or a queue wait over 30 seconds. The retention notice appears once. Private answers stream a draft and arrive without quoting the input; groups and topics keep their reply and status correlation. Ask “which projects do you see” to inspect accessible registered projects. Rich-message rejection falls back to ordered HTML chunks, then plain text on entity errors. Draft previews do not establish final delivery. After merge, the PM updates FRA and runs A40; the owner verifies the DM flow (§9).

For 0.2.4 acceptance, the PM updates FRA and runs A40 after merge and publication. The owner then checks rich layout, screenshot interpretation and action wording in chat. These checks remain separate from the deterministic fake-adapter suite.

## Built-in controls (task #106)

The command contract and permissions are in [§9](service-contract.md#command-line).
Send `/help` for the current list. `/version`, `/model`, `/doctor` and `/restart`
are service controls; they do not ask Codex to explain or execute a command.
`/model` offers catalogue buttons. `/model MODEL` accepts an exact catalogue
name; `/model default` restores inheritance. The saved choice applies to the
next manager run in this conversation, survives `/new` and service restart, and
never changes TaskQ workers. A catalogue entry is not proof of account access.

`/restart` requires the protected owner identity and an installed service. Keep
its acknowledgement separate from its later healthy-start receipt. During the
drain, active requests finish and other controls remain available; queued work
and TaskQ subscriptions stay recorded. A failed drain resumes admission.
`/skills` currently reports no enabled skills; `/skill NAME TEXT` returns an
explicit refusal. Adding a native manager skill requires a contract decision (§9).

The operator uses the same path with, for example,
`pilotq ask '/model default' --json`. Setup diagnostics and model catalogue reads
have absolute deadlines; dependency failures leave observations unknown and do
not change the saved choice. Use the operator audit to investigate details.

### TaskQ-only board access

Under #144, the configured TaskQ executable and registered project root own board access. PilotQ does not require gh, configure a forge or parse forge-specific issue paths. Doctor verifies a complete TaskQ snapshot; `status --task N --json` supplies task URLs/progress and `inbox N --json` supplies accepted answers with their original question revision, author and time. Failed reads stay unknown and preserve existing cards. Under #151, `issue_read` runs `taskq issue N --json`; the owner's `issue_comment` and `issue_label` run `taskq issue N --comment|--label --op-id ID --json` on `taskqorg/pilotq` and settle from the receipt. An unknown receipt stays pending and is repeated with the same operation ID at the next start. Effects recorded before #151 stay uncertain and are never replayed.

The PM releases the accepted change on PDS 18, checks `project_list` for `agentq`, `pilotq` and `tight`, then asks an authorized person to request tight's board status in Telegram. Record the request and reply, source link, exact TaskQ/PilotQ revisions and complete snapshot. Zero tasks qualifies an empty board only after that successful read. Registration grants neither server access nor worker capacity. Task execution additionally needs TaskQ project setup, an authorized worker on PDS 18, applicable runtime/resource grants and the board's read-after-write qualification (§5). Do not change the installation from a worker attempt.


### Approved installed read commands (#126)

The installation owner may create `commands.json` beside `keys.json`, mode `0600`.
There is no chat command for editing this file. Each entry is a read capability:
review the program and every complete argument array for mutations, secret output,
file access and subprocess behavior before granting it. Never put secrets in arguments. This list does not sandbox
an arbitrary executable. A program that changes behavior on upgrade needs review.

Invented configuration (replace executable paths, root and user ID):

```json
{
  "taskq": {
    "description": "List tasks on the registered PilotQ board",
    "minimum_version": "4.6.46",
    "executable": "/opt/taskq/taskq",
    "args": [["status"]],
    "cwd": "/srv/pilotq",
    "project": "pilotq",
    "board": "taskqorg/pilotq",
    "users": [123456789]
  },
  "pwd": {
    "description": "Show the approved working directory",
    "minimum_version": "8.30",
    "executable": "/bin/pwd",
    "args": [[]],
    "cwd": "/srv/pilotq",
    "project": "pilotq",
    "board": "taskqorg/pilotq",
    "users": [123456789]
  }
}
```

The project must already be registered with that exact root and board, and its
`taskq.json` must still name the board. Configured TaskQ must return a fresh complete
board snapshot within the same deadline before execution. Users need admission, `chat` and the project
grant as well as an explicit entry in `users`. `command_list` discovers their
available commands; `command_run` uses a name and one exact `args` array. Both
`pilotq ask 'taskq status'` and a clear chat request use this common manager path.
The manager clarifies ambiguity rather than constructing an unapproved command.

The child inherits no credential environment, uses `/usr/bin:/bin` as PATH and
uses the installation owner's HOME for installed tools' protected stores. Only
approved programs may read those stores. No keys are injected by this executor.
Each typed installed read stops at ten seconds within the remaining operation budget
and retains at most 16 KiB of output bytes; manager turns have no fixed duration or call cap (§7). The result
separates stdout, stderr, exit code, missing executable, timeout, cancellation and
output overflow. A nonzero exit is a failure. Output remains untrusted material.

Remove an entry or user from the protected file to revoke the capability; current
project grants are also checked during execution. Keep a copy of the prior file
for configuration rollback. No database migration is required. These instructions
configure an accepted build; they do not authorize installation on the live service.

### Chat administration (#160)

An administrator can ask to register an existing repository by link, for example
`add project demo - https://gitlab.example.test/invented/demo, add it to taskq`
(invented repository). HTTPS, SSH and `git@HOST:OWNER/REPO` links can include `.git`
and a trailing slash. `project_register` needs `projects`, prepares
`<install>/projects/NAME/taskq.json` and verifies the board through TaskQ before
registration. It attaches to an already installed healthy TaskQ service and runs
onboarding itself. Registration creates neither a repository nor a claim issue;
`filing_ready: false` names the remaining prerequisites in the onboarding report.

To admit a person, an administrator with `users` replies to that person's message and asks for the named grants, or supplies their exact Telegram ID after they were seen in that chat/topic. Names never select an identity. `user_grant` and `user_revoke` return the acting administrator, target and before/after grants; the same receipt appears in the request audit. The protected installation owner remains a configuration operation.

To open a project topic, ask from the forum supergroup and name the registered project. The caller needs `users` and project access; the bot needs `can_manage_topics`. Topics default to isolated conversations; ask explicitly for a “shared topic” or “shared conversation” (Russian: “общую тему” or “общий разговор”) to select it. An ambiguous or negated request is refused for clarification. Grant changes require each exact grant name in the caller’s own unquoted message; naming the person alone cannot authorize additional rights. The route's project default does not grant project access. The operator may use `pilotq ask` with an explicit group ID (or `CHAT:TOPIC` for grant operations) through the same tools.

An uncertain topic creation is never repeated automatically. Inspect the forum and request audit before asking for another topic. A partial registration retains its root without overwriting an existing configuration; inspect the TaskQ readback failure before submitting a new registration request. Schema 12 preserves these operations; a schema-11 binary refuses downgrade. Use the §12 drain and compatible-schema rollback path for a later release; this task does not authorize installation.

### Project onboarding

`pilotq projects add NAME ROOT` registers the explicitly supplied TaskQ root, runs
PilotQ doctor and the TaskQ service-status fallback, and prints the setup findings.
A nonzero exit with an onboarding report preserves registration but means the
project is not ready. Registration does not grant worker access or capacity
([contract §5](integration-contract.md#project-onboarding)).

The installation owner receives a Telegram DM card with one line per gap and a
Recheck button. PilotQ performs the root/configuration/service-entry steps; it
never invokes `service install`, changes the host unit or restarts workers. If
TaskQ has not yet observed a newly attached root, PilotQ retries its checks and
updates the same card. Follow the displayed owner steps outside PilotQ, then tap Recheck;
the same card shows the new findings. Host installation, credentials, capacity and
worker grants remain owner operations. For example, four Codex slots without a
Codex grant must show the project-scoped grant command, not a ready result.
Onboarding also checks `issue_read` key metadata: a key granted only to `pilotq`
does not cover another project. Run the displayed
`pilotq keys grant KEY project:NAME` command with the approved key name, then
Recheck. Key values stay in the protected store; this check never updates last use.

Use `/onboard NAME` in Telegram, or `pilotq ask '/onboard NAME'`, to recheck a
registered project. Only the protected owner may write `/onboard NAME override` to
bypass PilotQ's readiness gate; the gaps remain visible and TaskQ still enforces
its own grants, admission and claim prerequisites. Rechecking clears the override.
A pending or failed setup does not start the first task. Existing workers continue.
New TaskQ projects require explicit PilotQ registration before onboarding; there
is no directory discovery.

### Registered DEV/LIVE releases (#174)

Only the installation owner provisions a release capability. Register the actual release checkout first; a second checkout (for example Tight's cs checkout) is a blocker source, never a fallback release directory. The contract in [§5](integration-contract.md#dev-visibility-and-owner-release) owns authority and recovery.

Save an owner-vetted JSON configuration to a protected file, then run `pilotq projects release NAME FILE --json` and `pilotq users grant OWNER_ID release:NAME --json` (replace uppercase placeholders). Configuration is stored against the registered root/board; chat cannot edit commands or provision this grant. An invented configuration shape follows; replace paths, refs, URL, migration prefixes and minimum versions with verified project values:

```json
{
  "host": "invented-local-host",
  "git": "/usr/bin/git",
  "dev_health_url": "https://dev.example.invalid/__health",
  "main_ref": "refs/remotes/origin/main",
  "live_ref": "refs/remotes/origin/live",
  "dry_run": ["/usr/bin/npm", "run", "release", "--", "--dry-run"],
  "release": ["/usr/bin/npm", "run", "release", "--", "{sha}"],
  "pipeline": ["/opt/example/pipeline-read", "{sha}"],
  "migration_paths": ["migrations/"],
  "preflight": ["/usr/bin/npm", "run", "db:promotion:preflight"],
  "backup": ["/usr/bin/npm", "run", "db:live:backup"],
  "requirements": {
    "/usr/bin/git": "2.25.0",
    "/usr/bin/npm": "10.0.0",
    "/opt/example/pipeline-read": "1.0.0"
  }
}
```

The pipeline read command must return the §5 JSON report bound to its SHA argument, with release, smoke and backmerge states, HTTPS detail links and the actual LIVE SHA. It must support `--version`. Commands run with cleared inherited credentials, fixed system PATH and the installation owner's HOME; use the tools' protected stores. Never put tokens in configuration arguments. Doctor checks every registered executable's declared minimum version. Refresh the declared refs through the project's approved workflow before previewing; PilotQ never fetches or changes a checkout to guess a target.

For task delivery status (#179), add an owner-vetted `delivery_status` command array with a separate `{sha}` argument and declare its executable in `requirements`. Its read-only JSON schema is defined in [§5 delivery adapter](integration-contract.md#dev-visibility-and-owner-release); it is separate from the promotion `pipeline` report. The adapter must query the first containing main pipeline, all smoke retries and the latest later containing main pipeline, rather than worker comments. Refresh the configured LIVE ref through the approved project workflow. Missing configuration reports pipeline evidence unavailable.

After release, the PM asks for a recently closed cyprusbz task in the owner topic and records its request audit ID, binary/source identity, merge time, deploy job, failed smoke jobs and first failing checks, later main outcome and LIVE inclusion against GitLab. This live check is pending until recorded; the fake cs#541 trace proves local projection only.

Use `/release NAME` or ask for a project release preview. CLI `pilotq ask '/release NAME'` delivers the button to the owner's Telegram DM. Review the pinned SHA, authors, tasks, migrations and DEV observation, then press Release. Text does not confirm. Migration releases run preflight and backup first. A prerequisite failure stops the remaining stages. The final card names verified stage links and LIVE SHA, or explicitly reports unknown/uncertain evidence. Do not retry an uncertain release; inspect the project's pipeline and actual LIVE state before deciding on new work.

Local tests use temporary Git repositories and invented scripts only. PM acceptance remains the next owner-requested cs release on PDS 18; local success does not prove that release or deployment.

### Capability help and projects (#163)

Send `/help` for the short introduction and area index; send `/help projects` for project capabilities. `pilotq help` and `pilotq help projects` use the configured owner's same admission and audit path. Entries reflect current grants; ask a PilotQ administrator for additional access. Help is available in English and Russian. Its final TaskQ catalog is live machine-readable source JSON from the configured CLI; an older or missing CLI produces one availability line.

Ask “which projects can I use”, “which project am I using”, “switch to project NAME” or “show tasks for NAME”. Administrators can connect an existing repository through §5 registration and disconnect a named PilotQ registration. Disconnect clears matching current/default project selections, preserves files, repositories, boards, grants and task history, and returns its recorded readback. Existing work loses dispatch access until registration is restored through the normal verified path. The operator can use `pilotq projects current|switch|status|connect|disconnect` through the shared request and tool dispatch path without a model turn, or `pilotq ask` with the same wording. `connect` takes `NAME BOARD HOST REPO`; `projects add NAME ROOT` still connects an already prepared root.

For #163 A5, the PM records exact source and binary identity, uses an already authorized test-bot caller, sends `/help`, `/help projects` and “which projects can I use”, then reads back the three delivered texts and the `project_list` tool receipt. Worker component tests do not substitute for this run (§9).
