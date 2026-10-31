# PilotQ service specification

Binding detail of [pilotq.md](../pilotq.md). Mandatory clauses retain their force; **Proposed**
clauses remain unqualified design targets, not shipped guarantees or deployment permission.

## 9. Command line, update and monitoring

### Command line

Deterministic built-in commands never enter the model backlog, including while a chat run is active.
They use the same admission and audit boundary as other requests (R10, R13, R17).

- `/help` lists available controls; `/version` reports this binary's version; `/doctor` runs
  bounded, read-only setup checks and returns check names and outcomes only. Unknown commands
  receive a usage error, not a model turn. Commands accept Telegram's `@bot` suffix.
- `/model` reads the configured Codex app-server's model catalogue and displays selection buttons;
  `/model MODEL` selects an exact catalogue name, and `/model default` removes the override. The
  catalogue is not proof of subscription entitlement. Record the requested model separately from the
  last model reported by Codex. An unavailable catalogue refuses selection without changing the
  saved choice.
- A model choice belongs to the existing isolated or shared conversation, requires current `chat`
  admission, survives restart and `/new`, and applies only to runs claimed after the change. Buttons
  bind to the displaying person, conversation generation and latest selector. Active runs keep their
  claimed choice. No automatic model or paid-provider fallback is allowed (R9).
- Unset choices inherit the isolated Codex configuration. TaskQ worker defaults, inheritance,
  overrides and host resource admission stay with TaskQ; PilotQ never passes its manager choice to
  workers or changes running tasks. Model selection does not change the four-conversation budget
  (§7).
- `/restart` is owner-only. The installed service acknowledges immediately, drains through the
  existing §9 path, and reports completion only after the restarted service writes healthy status. A
  failed drain resumes admission and reports failure. Source runs refuse service restart. Requests,
  task subscriptions and delivery receipts survive. Duplicate updates cannot restart twice.
- `/skills` lists enabled skills; `/skill NAME TEXT` is the invocation syntax. No native skills are
  currently approved for the restricted manager. Both commands explain this without starting a model
  or enabling native tools; adding an approved skill requires a contract decision under §12.
- The operator can send the same controls through `pilotq ask '/model MODEL'` (R13). Service restart
  and setup checks keep their own bounded control worker, independent of chat generation.

**Proposed:** the `pilotq` binary is the service and the operator CLI. The CLI uses the same
journal, grants and audit records as Telegram; the local operator acts as the bootstrap owner
identity from protected configuration.

| Command | Purpose |
|---|---|
| `pilotq ask TEXT [--project P]` | One request through the normal manager path; prints the reply or task link |
| `pilotq status [--json]` | Health ages (section 7), pending requests, undelivered outcomes, uncertain effects |
| `pilotq users list\|show\|grant\|revoke\|remember\|forget` | People, profiles and grants (R10, R15); `remember`/`forget` correct one profile fact |
| `pilotq routes list\|set` | Chats, topics and their session and reply policy |
| `pilotq projects list\|add` | Registered projects and the section 5 registration path |
| `pilotq keys list\|add\|grant\|revoke` | Key store; `add` reads the value from stdin |
| `pilotq pilot start\|note\|report [--since T] [--json]` | Seven-day pilot start, intervention and outage notes, gate 4 counts (section 10 step 5) |
| `pilotq audit [--user U] [--route R] [--project P] [--since T] [--request ID] [--json]` | Request audit records (section 8, R17), filtered by who, where, what and when |
| `pilotq service install\|run\|stop\|status` | coreq service unit |
| `pilotq update [--check|--pause|--resume]`, `pilotq selftest`, `pilotq doctor` | coreq update, smoke test and setup check (bot identity, Codex login, TaskQ reachability) |

Every command that writes prints the readback, supports `--json` and returns a nonzero exit code on
failure.

`pilotq doctor [--json]` performs read-only setup checks: Telegram `getMe` returns a bot identity,
the configured isolated Codex home reports a ChatGPT login, and TaskQ returns a complete snapshot
matching its configured board. It never polls updates, starts a model turn or changes grants.
Missing configuration, failed checks and unavailable dependencies return nonzero; diagnostics expose
only check names and outcomes, never dependency output or credentials. This checks setup, not the
full task flow or subscription entitlement (§11). Also check local voice readiness. Missing voice
configuration, unavailable ffmpeg or recognizer, missing GigaAM model files, or a failed one-second
silent WAV decode/recognition probe makes doctor fail with a short reason. An empty transcript is
expected for silence; the recognizer must still return valid result JSON. The probe uses protected
temporary files, a ten-second total deadline and no Telegram input or model turn. This is setup
evidence, not live voice acceptance.

TaskQ owns board-answer access and trust. Doctor checks the configured TaskQ board through its CLI;
it neither requires nor probes gh. Selftest remains offline. CLI failures retain scrubbed operator
diagnostics under §8; unreadable answers remain unknown.

Prepare version `0.2.12` for #127. After merge-result CI and review acceptance, the PM publishes
#127 through the existing `live` release workflow, updates the existing FRA service through `pilotq
update`, and records source/binary identity, health, selftest, doctor and journal continuity with
§12 drain and compatible-schema rollback protections. The PM records a real authorized person's
question answered through TaskQ CLI, the original Telegram card with buttons removed and `Answered
by <actor>: <text>`, and a later tap showing the same receipt. These post-release checks remain
required evidence, not inferred from adapter tests. This grants no Hermes consumer cutover.

### Host requirements

The supported managed host is Linux x86_64. The installation owner provisions these tools; PilotQ
never runs a package manager, elevates privileges or changes a subscription login. Doctor probes the
configured binaries and PATH used by the service. Installed-release startup, install and
manual/scheduled update checks refuse missing, unparseable or older versions, reporting the exact
operator step. After remediation, rerun `pilotq doctor --json` and the original install/update
command. Offline selftest remains independent of host setup.

| Host tool | Minimum | Required by / remediation |
|---|---|---|
| Codex CLI | 0.162.0 | Subscription app-server and catalogue; install `@openai/codex@0.162.0` or newer and set the absolute `codex` path; retain the isolated ChatGPT login |
| TaskQ CLI | 4.6.46 | Task #145 file handoff, operation receipts and task #151 `taskq issue`; update the CLI to 4.6.46 or newer; doctor reads `taskq --version` |
| TaskQ service | 4.6.40 | Board commands; doctor checks the service version from `service status --json` |
| ffmpeg | 6.1.0 | Audio decode; install ffmpeg 6.1 or newer and set `voice.ffmpeg` |
| sherpa-onnx-offline | 2.0.1 | GigaAM recognition; install the static no-TTS 2.0.1 release or newer and set `voice.recognizer` |
| curl | 7.68.0 | CoreQ release download; install curl 7.68 or newer on service PATH |
| systemctl (systemd) | 245 | CoreQ Linux user service; provision systemd 245 or newer with a working user service manager |
| kill (procps-ng) | 3.3.17 | Process-group cleanup on PATH and `/bin/kill`; install procps-ng 3.3.17 or newer on service PATH |
| `commands.json` executables | Each entry declares `minimum_version` | Approved installed reads (§5); doctor runs the executable with `--version`; an absent minimum or unavailable version fails doctor |
| hostname | 3.23 | Telemetry host identity; install hostname 3.23 or newer on service PATH |

TaskQ CLI 4.6.46 or newer reports its binary version through `taskq --version`, without board or
network access. Doctor uses that command for `taskq.bin` and `--version` for approved command
executables; TaskQ needs no receipt. Receipts remain only for tools without a version command:
sherpa-onnx-offline 2.0.1 has no CLI version command. Its installed executable has an
owner-provisioned `<binary>.pilotq-version.json` receipt containing the verified release `version`
and executable `sha256`; doctor verifies the bytes before trusting that version. The tester guide
gives the provisioning step. A missing or mismatched receipt fails with an unavailable version. A
later recognizer with a valid version command can report its version directly.

There are no separately installed PilotQ shared libraries: SQLite, Rust TLS and CoreQ 0.14.6 are
linked into the artifact through Cargo.lock. The static recognizer is required; ffmpeg and system
tools retain their distribution-managed libraries. Doctor's existing silent decode/recognition probe
checks the voice runtime and model files in addition to versions. Developer/release-only tools
(`cargo`, the Rust compiler, `sha256sum`, and shell utilities in download instructions) are not
service runtime dependencies. Optional unsupported platforms are not qualified by the Linux
requirement probes.

Doctor's operator JSON adds `requirements` with tool, found version, minimum, outcome and fix; only
validated version numbers enter this report. Missing configuration still reports independently
probeable requirements and fails configured ones. `/doctor` projects the aggregate `host_tools`
outcome without paths, versions or diagnostics (§8). Failed subprocesses log command and scrubbed
errors to operator stderr; failures remain errors, never successful empty output. A new flag/API
must retain the listed minimum or raise it with its oldest-version fixture.

### Update and monitoring

Follow coreq: `releases/<version>`, `current` and `previous` in the install directory; GitHub
releases with `SHA256SUMS`; `pilotq update` verifies the checksum and runs `pilotq selftest` before
the switch; the launcher returns to `previous` after three unhealthy starts. An update waits for the
planned-restart drain in section 12.

On the restored version’s first healthy start after a launcher rollback, send one incident to the
protected `owner_id` Telegram DM, naming the failed and restored versions only. Persist the rollback
marker before launcher handoff and enqueue the incident atomically with a failed/restored-version
deduplication key in the existing journal. The existing outbox records the send receipt and applies
bounded retries across restarts (§4, §7); never reroute a failed DM. Successful updates produce no
failure notice. Both the launcher and restored release must include this integration; older releases
cannot deliver a new rollback marker. No schema change or new notification infrastructure is
required.

Use the rollback outbox path for service state; standard simple setup, no new component. `/status` adds the
version and each non-ok health source with its age. Service incidents for `model`, `taskq`, `poll`
and an unplanned restart go only to the `owner_id` DM through the outbox: one incident per episode
key across restarts, opened after the source stays failed 2 minutes (model: two consecutive failed
runs), closed only on a fresh successful observation, at most one message per key per 6 hours and 10
per day. Planned updates and drained restarts stay silent. On a healthy start after an undrained gap
over 5 minutes, send one downtime notice; detection is after the fact. Rules and examples: [status
and notifications](status-notifications.md). An ops chat (stage B) and real-time dead-service
detection through DashQ (stage C) wait for evidence.

`status.json` publishes the section 7 health fields. Incident reports follow coreq. Telemetry uses
the shared CoreQ telemetry store and DashQ connection, so the TaskQ PM sees PilotQ beside the other
tools. Release and CI follow the shared [release
contract](https://github.com/taskqorg/specq/blob/main/engineering/release-contract.md).

### First release delivery

Releases publish only through the `live` channel. After merge-queue CI accepts a `main` commit, the
PM fast-forwards `live` to it; that push builds and verifies the static `linux-x86_64` artifact,
creates the `vX.Y.Z` tag and publishes the binary, `SHA256SUMS` and `latest.json` from
`taskqorg/pilotq`. Tags alone do not publish. After publishing, the PM updates the test pilot
through `pilotq update` and verifies its running source commit, status, selftest, doctor and request
continuity. Failure drills simulate failures in PilotQ's own environment only: an empty `CODEX_HOME`
drop-in instead of the real Codex login, stopping only what PilotQ calls or waiting for zero workers
before stopping TaskQ, and killing only the PilotQ process; afterwards the service configuration is
restored and read back. A release authorizes the test pilot update, not production cutover.

The installed Linux service checks for releases every five minutes, including at startup when no
recent check is recorded. It persists the attempt time before the check so restarts do not reset the
retry interval. A separate maintenance thread keeps health reporting and conversations available
during downloads. Manual and scheduled updates share one OS-held update lock. A journal drain flag
gates both CLI and Telegram claims in their claim transactions, so a new run cannot race the drain.
Before activation, stop new manager turns and wait up to 65 seconds for running requests; an
incomplete or failed drain refuses activation and preserves the current release. Deferred-update diagnostics name
each blocking request, conversation, age and idle time. Runtime events persist the run’s last activity; typing
and progress timers never renew it. Drain retires terminal orphan slots and requests with no runtime events
for ten minutes, fences late results and preserves uncertain effects without replay. Startup also reconciles
requests left running beside terminal runs. Failed downloads,
checksum checks or selftests retain the current release and resume admission. Successful scheduled
activation exits the service with code 75; the coreq Linux unit restarts it through the launcher.
`pilotq update --pause` and `--resume` persist and read back automatic-update admission; pausing
does not stop health or conversations. Manual updates remain available.

This release retains coreq `v0.14.6` (locked in `Cargo.lock`) and journal schema 12. It opens
existing schemas 0–12 through the current migrations and refuses newer schemas; rollback to a binary
that cannot read the resulting schema is unsafe. The installation owner runs the user service and
owns its protected configuration. See the [tester guide](tester-guide.md) for installation,
checks and current limitations. A published release and successful selftest do not qualify
production cutover or the open decisions in section 11.

### Development

PilotQ is still in development. The PM installs merged, released builds on the existing live service
on PDS 18 (`pds18.ufobe.com`) and verifies them there; no separate installation or bot. This is a
scoped exception to the shared release contract's development-host separation, not authorization to
install unfinished branches or a Hermes consumer cutover. The installation owner remains the service
owner. Record the source commit, running binary identity, health, selftest and journal continuity
across restart, with the section 12 drain and compatible-schema rollback protections. Telegram
commands, model selection and answer cards need a real authorized person's run: the PM performs one
DM/topic run and attaches screenshots. The live answer-card author, button removal and
repeated-callback scenario (#117) remains unverified. Worker verification stays local/CI; workers
never install live. Communication scope: [Telegram feedback](telegram-feedback.md) under §12.

PilotQ has its own TaskQ board. Changes follow the TaskQ flow: spec edit first, held task, worker,
reviewer, release. Before each release, run the [automatic management
suite](acceptance.md#automatic-management-suite-a40) against the release candidate. Suite tasks
use a unique reserved `pilotq-e2e-*` host that no service claims; the runner scripts their
worker/reviewer board events without launching ordinary workers. This qualifies the management path,
not real implementation or review. All cases and cleanup must pass; a blocked or incomplete run is
not release evidence.

### Loop controls

`/loop`, including Telegram bot suffixes and CLI `ask`, uses the deterministic admission/audit path under [the route-loop protocol](conversation-contract.md#route-loops-and-board-wakes) (R13). The capability catalog owns help and manager-tool descriptions. The protected owner and shell grantees may use loop controls; other admitted callers receive a scoped refusal. `/new` stops the destination's loop. Restart and update drain retain armed and pending loops without replaying interrupted native effects. [PM TEST qualification](loop-220-live.md) remains separate from local/CI checks and installation authority (§9).
