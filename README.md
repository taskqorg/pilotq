# PilotQ

A server agent for Telegram conversations, short actions and TaskQ management.

PilotQ answers questions and performs bounded actions directly. It turns longer work into TaskQ tasks, relays questions, and returns verified outcomes to the originating conversation. TaskQ continues execution while PilotQ is offline.

**Status: specification plus a Rust service on [coreq](https://github.com/taskqorg/coreq) with the journal, Telegram ingress and outbox; no model, grants or deployment.** The `pilotq` binary has `service install|run|stop|status`, `update [--check]`, `selftest`, `status [--json]` and `version`. With an owner-only `<install>/config.json` (`bot_token`, `owner_id`, optional `api_url`), `service run` long-polls Telegram into `journal.sqlite`, admits only the bootstrap owner and acknowledges each request through the outbox. Model and TaskQ health read unknown until those parts exist. Checks: `cargo test --locked`. A `v*` tag publishes `linux-x86_64` with `SHA256SUMS` and `latest.json`.

- [Product contract](pilotq.md): behavior, ownership, reliability and context requirements.
- [Runtime research](docs/research.md): source findings, alternatives and adoption decisions.
- [Core options](docs/core-options.md): minimal owned service versus Pi, with Codex-only subscription access.
- [Mechanism matrix](docs/mechanism-matrix.md): Hermes, Pi, NanoClaw and Takopi mechanisms, scope decisions and acceptance coverage.
- [Acceptance cases](docs/acceptance.md): observable outcomes, failure injection and selection gates.
- [Project relationships](docs/projects.md): PilotQ in the taskqorg project portfolio.
- [Repository setup status](docs/bootstrap.md): verified publication and unresolved platform controls.

Shared rules: [specq](https://github.com/taskqorg/specq).
