# PilotQ

A server agent for Telegram conversations, short actions and TaskQ management.

PilotQ answers questions and performs bounded actions directly. It turns longer work into TaskQ tasks, relays questions, and returns verified outcomes to the originating conversation. TaskQ continues execution while PilotQ is offline.

**Status: specification plus a Rust service on [coreq](https://github.com/taskqorg/coreq) with the journal, Telegram ingress and outbox, grants, route session policy, profiles, a Codex app-server manager and TaskQ tools for registered boards; separate test-bot trials, no production cutover.** The `pilotq` binary has `service install|run|stop|status`, `update [--check|--pause|--resume]`, `selftest`, `status [--json]`, `ask TEXT [--json]`, `users list|show|grant|revoke|remember|forget`, `routes list|set`, `projects list|add`, `keys list|add|grant|revoke`, `audit [--user|--route|--project|--since|--request] [--json]`, `pilot start|note|report [--json]` and `version`. With an owner-only `<install>/config.json` (`bot_token`, `owner_id`, optional `api_url`, `codex` binary path and `codex_limits` seconds for `first_event`, `silent` and `deadline`, default 30, 30 and 60), `service run` long-polls Telegram into `journal.sqlite`, admits only people holding the `chat` grant (the configured owner holds every grant), acknowledges each request and answers it through one Codex turn per request. `/status`, `/stop`, `/new` and `/audit` (a person's own request records, in a DM only) answer without the model. Every request leaves one audit record in the journal and one metadata-only CoreQ event in `<install>/telemetry.sqlite` with trace ID `pilotq.request.<ID>`. `pilotq ask` takes the same grant check, request record and manager path as the configured owner.

With `config.json` `taskq` (`bin`: the TaskQ CLI, `root`: a pilotq checkout whose `taskq.json` names `taskqorg/pilotq`, optional `poll` seconds, default 15) the manager gets the [section 5](pilotq.md#first-taskq-tool-contract) tools: `task_plan`, `task_run_plan`, `task_status`, `task_park`, `task_steer`, `task_requeue` and `queue_pause`. Task tools resolve the explicit or current registered project; ambiguous numbers are refused. Filing needs `create_claim_issue` in the resolved root's `taskq.json`. Plans retain their saved target across project switches. Schema 8 preserves existing plans and subscriptions as `pilotq`; older binaries refuse this journal, so rollback needs a compatible binary or a pre-upgrade backup and reconciliation of later accepted work. Each filed task gets one status message in its route, edited in place; a worker question is relayed there, and a person answers by replying to it.

Direct actions ([section 5](pilotq.md#direct-actions-keys-and-project-switching)): `project_use`, `issue_read`, and, for the owner only, `issue_comment` and `issue_label` on `taskqorg/pilotq` issues, at most 5 tool calls per request, counted across its retries and restarts. `pilotq projects add NAME ROOT` registers a root after `taskq status --json` under it reads back the board its `taskq.json` names. `pilotq keys add NAME PURPOSE TOOL..` reads the value from stdin into the owner-only `<install>/keys.json`; `keys grant NAME ID|route:CHAT[:TOPIC]|project:NAME..` and `keys revoke NAME [WHO..]` change holders, and `keys list` never prints a value. Issue tools run `gh api` (`config.json` `gh`, default `gh`) with the key as `GH_TOKEN`; their output is scrubbed of every stored value before the model sees it.

Codex runs as `codex app-server` with its own home, `<install>/codex`, so no personal MCP server, plugin or config loads; shell, apps, plugins, browser, computer use, multi-agent, image generation and web search are off and the sandbox is read-only. Sign it in once with the subscription account: `CODEX_HOME=<install>/codex codex login`. API-key login is refused (`forced_login_method="chatgpt"`) and API-key variables are not passed. TaskQ health reads unknown until the first board scan. Checks: `cargo test --locked`. The owner fast-forwards `live` to a main commit verified by merge-queue CI; its workflow publishes `linux-x86_64` with `SHA256SUMS` and `latest.json` ([release contract](pilotq.md#first-release-delivery)). Tags alone do not publish.

- [Tester guide](docs/tester-guide.md): isolated installation, access, Telegram use and problem reports.
- [Product contract](pilotq.md): behavior, ownership, reliability and context requirements.
- [Runtime research](docs/research.md): source findings, alternatives and adoption decisions.
- [Core options](docs/core-options.md): minimal owned service versus Pi, with Codex-only subscription access.
- [Mechanism matrix](docs/mechanism-matrix.md): Hermes, Pi, NanoClaw and Takopi mechanisms, scope decisions and acceptance coverage.
- [Acceptance cases](docs/acceptance.md): observable outcomes, failure injection and selection gates.
- [Project relationships](docs/projects.md): PilotQ in the taskqorg project portfolio.
- [Repository setup status](docs/bootstrap.md): verified publication and unresolved platform controls.

Shared rules: [specq](https://github.com/taskqorg/specq).

Measure dynamic-tool result sizes during the step 5 pilot with
`pilotq pilot report --since UNIX_SECONDS [--until UNIX_SECONDS] [--json]`.
The half-open window defaults to ending now. Each tool reports call count,
nearest-rank p95 and maximum UTF-8 result-text bytes, after key scrubbing and
before JSON serialization. Refusals count too. The journal's `tool_sizes` table
stores request/run IDs, time, tool name, bytes, PilotQ version and the Codex
initialization user-agent (or `unknown` if absent), without arguments or result
content. An empty window has no tool rows; a missing journal is an error.
This measures bytes, not tokens or savings (§ 6, R17).
