# Project relationships

PilotQ joins taskqorg as a private, specification-stage project. Its scope is the Telegram conversation and manager service.

| Project | Responsibility | Relationship to PilotQ |
|---|---|---|
| [pilotq](https://github.com/taskqorg/pilotq) | Conversation, short actions and TaskQ management | This project; no runtime yet |
| [taskq](https://github.com/taskqorg/taskq) | Durable task lifecycle, workers, reviewers and boards | Execution authority; reuse its commands |
| [agentq](https://github.com/taskqorg/agentq) | Context optimization and recovery | Qualify an adapter for the chosen manager runtime |
| [serverq](https://github.com/taskqorg/serverq) | Server-care rules and compliance checks | Source of bounded diagnostics and operations tasks |
| [dashq](https://github.com/taskqorg/dashq) | Cross-product operational dashboard | Existing broader overview; Telegram is PilotQ's user interface |
| [coreq](https://github.com/taskqorg/coreq) | Shared Rust release and service facilities | Reuse only where the selected runtime can consume existing facilities economically |
| [specq](https://github.com/taskqorg/specq) | Shared engineering, design and writing rules | Binding shared contract |

Repository membership does not register an execution root, grant host access, install a service or enable TaskQ admission. Those belong to later setup work.

PilotQ files its own tasks on the [pilotq board](https://github.com/taskqorg/pilotq/issues), as section 9 of [pilotq.md](../pilotq.md#development) requires.
