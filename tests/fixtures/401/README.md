# Explanation replay sources (#401)

Each file replays one question the owner saw raw in Telegram. They are public historical evidence, not current board state or authority to answer.

- `question`: the literal question fields, parsed from the `taskq: ask` comment named in `source`. Read through `gh` on 2026-11-06. The tests read these fields; they do not call a board.
- `task`: the task text that question cites. Only the cited acceptance and steering lines are kept.
- `clauses`: the literal clause, stored under the project-relative path it has in that project's contract. The replay places it in a temporary root, so the source check reads a real file.
- `authored`: an explanation written by the #401 worker.
- `model`: an explanation from `claude -p` on the subscription CLI with no tools and API keys removed, on 2026-11-06. `model_id` names the model. Its prompt held the shipped review instructions, explanation rules, `task_explain` schema, this card, task text and clauses. It did not hold `authored`.
- `model_refused_first` (taskq#2482 only): that model's first call, whose unknowns named a bare `R13`. The boundary refuses it. `model` is the second call, made after the model received that exact refusal.

This is an offline replay of the rules with a real model. It is not a full compiled session, a live PilotQ turn or Telegram rendering; those remain the PM's check after release (§9). The R13 code in agentq#260 was not resolved to a clause, so its explanation cites the task text and the contract's owner-decision rule instead, and reports what it did not read.
