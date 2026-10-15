# PilotQ

A server agent for Telegram conversations, short actions and TaskQ management.

PilotQ answers questions and performs bounded actions directly. It turns longer work into TaskQ tasks, relays questions, and returns verified outcomes to the originating conversation. TaskQ continues execution while PilotQ is offline.

**Status: specification, no runtime or deployment.** The owner approved the name and a private `taskqorg/pilotq` repository on 2026-10-16. Pi is the first proposed manager runtime to qualify; production selection remains open.

- [Product contract](pilotq.md): behavior, ownership, reliability and context requirements.
- [Runtime research](docs/research.md): source findings, alternatives and adoption decisions.
- [Acceptance cases](docs/acceptance.md): observable outcomes, failure injection and selection gates.
- [Project relationships](docs/projects.md): PilotQ in the taskqorg project portfolio.
- [Repository setup status](docs/bootstrap.md): verified publication and unresolved platform controls.

Shared rules: [specq](https://github.com/taskqorg/specq).
