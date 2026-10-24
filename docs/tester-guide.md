# Test PilotQ

Use PilotQ to discuss work, approve a saved plan and follow TaskQ in Telegram. This guide describes the first release candidate, `0.2.0`, not production-cutover acceptance ([contract §9](../pilotq.md#first-release-delivery)). Release publication waits for #36 to close after 2026-10-30 09:47 UTC. Until then the download commands below will not find this release.

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

Each accepted input gets a request number and acknowledgement unless the answer arrives first. Typing is only feedback, not completion. Send text for this trial; voice, images and documents are detected and reported unsupported.

## Plan, authorize and follow work

Use a small, real, authorized change on `taskqorg/pilotq`; do not file placeholder work on its live board.

1. Say “Plan this change for taskqorg/pilotq: … Do not run it yet.” Supply the goal, observable acceptance and scope. Expect one to three proposed tickets and a saved plan revision; discussion creates no board issue.
2. Inspect that revision. Ask for corrections if needed; approve only the latest displayed revision.
3. In a **later message in the same conversation**, say “Run plan revision N for taskqorg/pilotq,” replacing `N`. Expect issue URLs after held creation, readback and start. A plan saved in the same request cannot run.
4. Ask “What is the current state of https://github.com/taskqorg/pilotq/issues/N?” using the real URL. Each filed task has a status message updated in its original route. A verified closing outcome is a separate message; worker completion, reviewer completion and publication are distinct.
5. When a worker question arrives, use Telegram **Reply on that question message** and supply your answer. You need `taskq` and project scope. A forwarded question, another route or an old question revision cannot authorize the answer. PilotQ itself does not answer for you.

If creation is partial or TaskQ is unavailable, keep the request number and observed URLs. The saved plan can reconcile using the same operation IDs; do not submit a duplicate plan to repair delivery. Missing `create_claim_issue` disables creation with a reason (§5).

## Steer, park, pause and recover

- **Steer a task:** this release has no PilotQ `task_steer` tool. Give the task URL and requested change to the board PM/operator, who uses TaskQ's supported steering command and reads it back. A chat follow-up to PilotQ does not prove the worker received it.
- **Park a task:** ask “Park https://github.com/taskqorg/pilotq/issues/N because …”. Expect the supported TaskQ transition and its readback; this is separate from terminating a running worker.
- **Pause admission:** a person holding `queue` may ask “Pause new TaskQ queue admission on this host.” This affects the host, not just this chat. Running workers continue. Queue resume is an operator action through TaskQ; PilotQ has no resume tool.
- **Stop the manager:** `/stop` cancels this conversation's manager work. Check its outcome; it does not park TaskQ work or pause the host queue.
- **Reset context:** `/new` starts a new conversation generation. It keeps audit records and subscriptions to admitted tasks.
- **Inspect:** `/status` uses the deterministic control path even if the model is unavailable. Unknown health or an uncertain effect needs investigation, not another execution request.

Missing grants, a foreign board, stale answers, unregistered targets and the sixth tool call are refusals, not successful actions. Only the configured owner can directly comment or label a PilotQ issue, with a suitable key. Other projects are read-only. Keys enter through `pilotq keys add` on the CLI, never Telegram (§5, §8).

## Install a separate Linux test instance

This procedure is for an authorized operator on a separate Linux x86_64 account or host, with `curl`, `sha256sum`, Codex and TaskQ already installed. Use a **separate test bot token** and a verified subscription login. Never reuse the production token or change the protected FRA pilot. Do not run two pollers with one token (§4). Shared-server subscription entitlement remains an open production decision (§11).

After publication, download and verify the release before executing it:

```sh
mkdir pilotq-download
cd pilotq-download
curl -fSLO https://github.com/taskqorg/pilotq/releases/download/v0.2.0/pilotq-0.2.0-linux-x86_64
curl -fSLO https://github.com/taskqorg/pilotq/releases/download/v0.2.0/SHA256SUMS
curl -fSLO https://github.com/taskqorg/pilotq/releases/download/v0.2.0/latest.json
sha256sum -c SHA256SUMS
chmod 755 pilotq-0.2.0-linux-x86_64
./pilotq-0.2.0-linux-x86_64 selftest
./pilotq-0.2.0-linux-x86_64 update --check
```

All checksum entries must pass. A fresh installation running this same version prints `update: up to date` after fetching the release manifest. An older installed selection sees `update: 0.2.0 available`; `pilotq update` verifies the download and runs offline selftest before activation. A checksum or selftest failure must leave the old selection intact. Selftest checks the local fixture, not Telegram, Codex login or the full task flow.

The default install directory is `$HOME/.local/share/pilotq`. Use the dedicated account's default for service installation; the generated user unit does not preserve an arbitrary `PILOTQ_INSTALL` override. Before starting a poller:

1. Create the install directory with mode 0700. Write its `config.json` with mode 0600 using protected local editing, with `bot_token`, numeric `owner_id`, an absolute `codex` binary path, and `taskq` containing absolute `bin` and `root` paths. The registered root's `taskq.json` must name `taskqorg/pilotq` and have the create-claim prerequisite. Never paste the config into chat or a report.
2. Sign in to the approved subscription account with `CODEX_HOME="$HOME/.local/share/pilotq/codex" codex login`. API-key login is refused; no billed API fallback exists (R9). This is Codex login, not GitHub device login. Have the owner provision TaskQ's existing host GitHub credential through its protected store.
3. Register the project with `pilotq projects add pilotq ROOT --json`, substituting the real registered root. When running the downloaded binary before installation, substitute `./pilotq-0.2.0-linux-x86_64` for `pilotq`. Read back `projects list`, grants and route policies.
4. Run `./pilotq-0.2.0-linux-x86_64 service install` only after explicitly authorizing this test instance. It installs and starts the coreq user service. Inspect `pilotq service status` and `pilotq status --json` using `$HOME/.local/share/pilotq/pilotq` if it is not on PATH.
5. Verify bot identity and route delivery with an admitted person's DM, group and topic inputs. Then perform the authorized TaskQ flow above. A CLI-only answer or fresh health file does not prove Telegram acceptance (R13).

No `doctor` command is implemented yet. Automatic scheduled self-update is also not wired into the current service loop; publication alone is not proof of rollout. That existing gap must be resolved before claiming the shared automatic-rollout contract is qualified. This task changes release preparation and docs, not runtime behavior.

## Report a problem and preserve work

Report to the test administrator or [PilotQ board](https://github.com/taskqorg/pilotq/issues), with version, request number, task URL if any, route type, expected and observed result, UTC time and whether an external effect might have occurred. Share message content or route/user identifiers only through an approved channel. Never include a token, key, login/session file, full configuration or raw journal.

The operator can use `pilotq status --json`, `pilotq audit --request ID --json` and service logs locally. Distinguish unavailable observations from healthy ones. Do not resend execution because a Telegram reply is missing: delivery may be pending or ambiguous while the task already exists. Preserve uncertain effects and original destinations; never redirect private output to another chat.

For a failed test installation, stop it with `pilotq service stop`, verify the poller stopped and retain accepted requests, the outbox and protected configuration. Preserve a supported SQLite snapshot before any restore; copying a live database file alone is not restore proof. The first fresh install has no previous release. An existing install keeps `previous`, and the coreq launcher can fall back after three unhealthy starts; only use a previous binary compatible with journal schema 7. There is no PilotQ rollback command. Do not delete the journal, resubmit admitted tasks or restart another consumer as a rollback shortcut (§4, §12).
