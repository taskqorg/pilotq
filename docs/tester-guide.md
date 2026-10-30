# Test PilotQ

Use PilotQ to discuss work, approve a saved plan and follow TaskQ in Telegram. This guide describes the first release candidate, `0.2.4`, not production-cutover acceptance ([contract §9](../pilotq.md#first-release-delivery)). The owner releases by fast-forwarding `live` after merge-queue acceptance; the download commands work after that workflow publishes the assets.

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

The required versions and their consumers are owned by [contract §9](../pilotq.md#host-requirements-task-130). Use the installation owner's account and the same PATH as the service. Before installation or an update, run `pilotq doctor --json`; inspect every `requirements` entry, including `found`, `minimum`, `outcome` and `fix`. A failed entry blocks service installation and manual/scheduled update checks. The current release keeps running; doctor stays failed until the named step is completed. PilotQ does not elevate privileges or install packages on the owner's behalf.

For an Ubuntu 24.04 managed host, the operator provisions `ffmpeg`, `curl`, `procps`, `hostname` and `systemd` with the distribution package manager, then verifies the actual versions with doctor. If the distribution offers an older version, use the tool's upstream package repository or upgrade the host; do not bypass the check. Install Codex with `npm install --global @openai/codex@0.162.0` (or a newer approved version), set its absolute path in configuration and preserve the existing isolated ChatGPT login. Update the existing TaskQ installation with `taskq update`; verify its service version and preserve its existing host credentials. Do not run GitHub device login.

Install the static no-TTS [sherpa-onnx v2.0.1 Linux x64 release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1), or a newer approved release, with the GigaAM files described below. Set the absolute `voice.recognizer` and `voice.ffmpeg` paths. Doctor additionally decodes and recognizes silence; version checks alone do not establish voice readiness.

TaskQ 4.6.44 and sherpa-onnx-offline 2.0.1 do not expose their binary version. After verifying the release source and unpacked executable, record its release version and SHA-256 beside the executable. This receipt is operator-provisioned release identity, not independent upstream attestation. Do not infer a version from a directory name or copy a receipt from another executable. Example for the verified sherpa release; replace the invented absolute path with the installed executable:

```sh
TOOL_BINARY=/absolute/path/to/sherpa-onnx-offline
TOOL_VERSION=2.0.1
TOOL_SHA=$(sha256sum "$TOOL_BINARY" | cut -d ' ' -f 1)
printf '{"version":"%s","sha256":"%s"}\n' "$TOOL_VERSION" "$TOOL_SHA" > "$TOOL_BINARY.pilotq-version.json"
chmod 600 "$TOOL_BINARY.pilotq-version.json"
```

Repeat for the resolved TaskQ CLI release executable with `TOOL_VERSION=4.6.44` or its verified newer release. If `taskq.bin` is a launcher, use the resolved release executable as `taskq.bin` so its receipt identifies the CLI being executed. Regenerate receipts after replacing either executable. Doctor refuses missing receipts, changed bytes and versions below the minimum; the TaskQ service is checked separately. A newer recognizer with a working version flag needs no receipt. Rerun `pilotq doctor --json`, then repeat installation or update. Never alter a receipt merely to silence an unsupported-version failure.

Each protected `commands.json` capability also declares `"minimum_version":"<verified minimum>"`. Doctor checks that executable with the same checksum-bound receipt procedure; it does not run arbitrary version arguments supplied by a capability. For the documented TaskQ status capability, use `"minimum_version":"4.6.40"` and its resolved CLI receipt. No capability file means no additional requirements; an unreadable or malformed file fails doctor. Adding a capability does not install its dependencies.

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

For a failed test installation, stop it with `pilotq service stop`, verify the poller stopped and retain accepted requests, the outbox and protected configuration. Preserve a supported SQLite snapshot before any restore; copying a live database file alone is not restore proof. The first fresh install has no previous release. An existing install keeps `previous`, and the coreq launcher can fall back after three unhealthy starts; only use a previous binary compatible with journal schema 11. There is no PilotQ rollback command. Do not delete the journal, resubmit admitted tasks or restart another consumer as a rollback shortcut (§4, §12).

Task #68 reply checks: an accepted text gets an 👀 reaction, with a text receipt only on reaction failure or a queue wait over 30 seconds. The retention notice appears once. Private answers stream a draft and arrive without quoting the input; groups and topics keep their reply and status correlation. Ask “which projects do you see” to inspect accessible registered projects. Rich-message rejection falls back to ordered HTML chunks, then plain text on entity errors. Draft previews do not establish final delivery. After merge, the PM updates FRA and runs A40; the owner verifies the DM flow (§9).

For 0.2.4 acceptance, the PM updates FRA and runs A40 after merge and publication. The owner then checks rich layout, screenshot interpretation and action wording in chat. These checks remain separate from the deterministic fake-adapter suite.

## Built-in controls (task #106)

The command contract and permissions are in [§9](../pilotq.md#command-line).
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
explicit refusal. Native skills cannot bypass the restricted tool boundary.

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
    "minimum_version": "4.6.40",
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
The request retains its five-call budget; execution stops at ten seconds or the
original manager deadline, and retains at most 16 KiB of output bytes. The result
separates stdout, stderr, exit code, missing executable, timeout, cancellation and
output overflow. A nonzero exit is a failure. Output remains untrusted material.

Remove an entry or user from the protected file to revoke the capability; current
project grants are also checked during execution. Keep a copy of the prior file
for configuration rollback. No database migration is required. These instructions
configure an accepted build; they do not authorize installation on the live service.
