# PilotQ

A server agent for Telegram conversations, short actions and TaskQ management.

PilotQ answers questions and performs bounded actions directly. It turns longer work into TaskQ tasks, relays questions, and returns verified outcomes to the originating conversation. TaskQ continues execution while PilotQ is offline.

**Status: specification plus a Rust service on [coreq](https://github.com/taskqorg/coreq) with the journal, Telegram ingress and outbox, grants, route session policy, profiles and a Codex app-server manager; no TaskQ tools or deployment.** The `pilotq` binary has `service install|run|stop|status`, `update [--check]`, `selftest`, `status [--json]`, `ask TEXT [--json]`, `users list|show|grant|revoke|remember|forget`, `routes list|set` and `version`. With an owner-only `<install>/config.json` (`bot_token`, `owner_id`, optional `api_url`, `codex` binary path and `codex_limits` seconds for `first_event`, `silent` and `deadline`, default 30, 30 and 60), `service run` long-polls Telegram into `journal.sqlite`, admits only people holding the `chat` grant (the configured owner holds every grant), acknowledges each request and answers it through one Codex turn per request. `/status`, `/stop` and `/new` answer without the model. `pilotq ask` takes the same grant check, request record and manager path as the configured owner.

Codex runs as `codex app-server` with its own home, `<install>/codex`, so no personal MCP server, plugin or config loads; shell, apps, plugins, browser, computer use, multi-agent, image generation and web search are off and the sandbox is read-only. Sign it in once with the subscription account: `CODEX_HOME=<install>/codex codex login`. API-key login is refused (`forced_login_method="chatgpt"`) and API-key variables are not passed. TaskQ health reads unknown until that part exists. Checks: `cargo test --locked`. A `v*` tag publishes `linux-x86_64` with `SHA256SUMS` and `latest.json`.

- [Product contract](pilotq.md): behavior, ownership, reliability and context requirements.
- [Runtime research](docs/research.md): source findings, alternatives and adoption decisions.
- [Core options](docs/core-options.md): minimal owned service versus Pi, with Codex-only subscription access.
- [Mechanism matrix](docs/mechanism-matrix.md): Hermes, Pi, NanoClaw and Takopi mechanisms, scope decisions and acceptance coverage.
- [Acceptance cases](docs/acceptance.md): observable outcomes, failure injection and selection gates.
- [Project relationships](docs/projects.md): PilotQ in the taskqorg project portfolio.
- [Repository setup status](docs/bootstrap.md): verified publication and unresolved platform controls.

Shared rules: [specq](https://github.com/taskqorg/specq).
