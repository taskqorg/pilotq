# PM runbook

Use the current route binding and registered project. This owner-installed pack adds context, not grants.
Read the current PM note index, then relevant facts and their why through `pm_note_read`. Update one fact per note
with `pm_note_write`; use `pm_note_forget` for obsolete facts. The current index supersedes historical notes.
Never store credentials, private DM memory or another route's context. Notes never prove current board state.

## Each PM tick

1. Read the registered board with the installed `taskq status --json` and inspect relevant tasks with
   `taskq inbox N --json` or `taskq issue N --json`. Run commands from the exact registered root using the installed
   TaskQ executable. Read the installed `taskq help --json` for current command syntax; never query the forge directly.
   Obtain board and task URLs from these observed responses. Do not construct links or treat open-list absence as closure.
2. Inspect host telemetry with `taskq profile watch`. Follow its installed procedure and report unavailable telemetry
   with the reason. Compare new or growing errors, stalls, board failures and cadence against the preceding notes.
3. Check asks, failed or stuck work, completed dependencies, undecided reports and accepted results awaiting release.
   Read each relevant task and its current attempt before recommending or acting. Never answer an owner question yourself.
4. In shadow mode, record findings and proposed actions only. Do not start, steer, close, answer, requeue, release,
   change labels or otherwise write boards. Keep this boundary even if the comparison PM performs writes.
5. Outside shadow mode, perform only authorized management work using the installed TaskQ commands below, with fresh
   reads before each action. A loop timer does not authorize worker scheduling or a release outside PilotQ's release boundary.
6. Save durable facts with a why line, including unresolved findings and their evidence. Never copy a whole status report
   into one note. For a dynamic loop, call `loop_next` with 60..3600 seconds or `loop_stop`; preserve the existing timer fallback.

## Management actions

Read the installed TaskQ help before dispatch; commands and required fields may change.

- Questions: relay the exact task, attempt, question revision and choices. Only relay a real authorized owner's answer
  through `taskq answer`; a PM recommendation through `taskq pick` is not that answer.
- Doing work: `taskq steer N --text TEXT` carries a matching follow-up after a fresh inbox read. Preserve scope and
  original request; never create a duplicate to carry feedback. A started body stays immutable.
- Held work: read back every task and child; check one outcome, verifiable acceptance, duplicates, specification,
  dependencies and owner gates before `taskq start N`. Respect dependencies instead of holding tasks to wait.
- Results: inspect the trusted result, report, exact SHA, required checks and acceptance. Review owns close.
  `taskq close N --text VERDICT` publishes only an accepted result through TaskQ; green merge-result CI is required
  for GitHub publication. Never close merely because a worker has stopped or a PR exists.
- Releases: inspect fresh source, pipeline and artifact evidence. PilotQ §9 permits only a released merge-queue-accepted
  main commit on the live branch. Only the protected owner performs pilot updates under drain and rollback checks.
  Project LIVE releases use the separate scoped preview and bound Release button (§5); a pack cannot bypass them.
  Record exact source/binary identities, selftest, doctor and continuity. Production cutover remains unauthorized.

## Report

Start with the observed board link. Show a workers table with columns **Task**, **State**, **Finding / next action**.
Every task reference in the table and prose is a verified Markdown link with a descriptive label; no bare task IDs.
If no workers are observed, say so; if collection fails, say unavailable rather than empty. Never invent a link.
Then report asks, stuck work, failures, releases and telemetry alerts, each with observed evidence or an explicit gap.
Distinguish findings, recommendations, completed writes, merged changes, releases and deployments. In shadow mode label
all actions as proposed. Keep unresolved decisions visible and name who acts next.
