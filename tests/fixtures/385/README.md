# Task explanation sources

These are selected literal fields read through the installed TaskQ CLI during #385 on 2026-11-05. They are public historical source evidence, not current board state or authority to answer a question.

- `2450.json`: `taskq status --project taskq --json`; three-choice doctor question and accepted original option 2.
- `267.json`: `taskq status --project dashq --json`; separate privacy-boundary and later end-to-end dependency-cycle question revisions.
- `2453.json`: `taskq status --closed --task 2453 --json`, `taskq issue 2453 --json` and `taskq inbox 2453 --json` from the TaskQ board on 2026-11-06, with the publish and close bodies compared to the forge. It keeps trusted unedited publish `6670835977` and close `6670877601`, the worker result and the edited summary comment, which must not supply findings. Its evidence covers four Mac runtimes on a candidate Service with a fixture board, not an installed service.
- `2451.json`: `taskq status --closed --task 2451 --json`, `taskq issue 2451 --json` and `taskq inbox 2451 --json` from the explicitly named TaskQ board. Contains verified closure identity, canonical publication/close history and literal trusted publication limitations.

Collection kept only the fields exercised by the adapter tests. Real model explanations, original-number Telegram roundtrips and screenshots require PM qualification after release (§9); these fixtures do not substitute for that proof.
