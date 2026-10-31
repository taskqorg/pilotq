# PM pack and shadow parity qualification

Qualify task #221 after worker implementation and CI. The PM owns this stage under PilotQ §9; workers do not install live.
A one-hour shadow loop proves observed parity for that hour, not an SLA, unrestricted automation or production cutover.

## Preparation

- Record the released source SHA, binary checksum, version and healthy TEST status on pds18. Follow the normal PM update,
  drain and rollback procedure. Do not install an unfinished branch. Record selftest, doctor and journal continuity.
- Choose a dedicated authorized topic and one registered board. Keep the comparison Claude PM session's scope identical.
  Record exact route, project, participants and UTC start/end in the evidence; omit secrets.
- Review Claude PM memory manually. Seed only relevant facts and why lines; exclude credentials, private DM facts and
  other routes' facts. A project-qualified binding remains private to its exact route.

The following examples use invented route and project identifiers; replace them with the chosen authorized target.
Run through the installation owner's protected CLI. Quoting keeps each fact and why one argument.

```sh
pilotq pm install pm docs/pm-pack.md --json
pilotq pm bind -1000000000001:10 pm example --json
pilotq pm notes -1000000000001:10 set release 'Release needs verified CI.' 'Protect the pilot.' --json
pilotq pm notes -1000000000001:10 list --json
```

## Checks and one-hour run

1. On the bound route, `/help operations` must show `pm`; an unbound topic and DM must show no binding or notes.
   Ask the model to read the seeded note through `pm_note_read`, update it with one fact and why, then read back via CLI.
   Confirm another topic cannot read it, including one bound to the same project. Verify an ordinary admitted user
   without shell cannot load the pack or use its tools.
2. Use `/new`, then confirm the note remains while the route loop has stopped. Restart TEST through the authorized PM
   path and repeat the read. Revoke the test shell grantee's access and confirm subsequent context/tools refuse it.
   Restore only the explicitly intended grant. Record screenshots and durable request evidence.
3. Test refusal using invented credential syntax only: `token=INVENTED`. Verify no note or pack was stored and no rejected
   content appeared in the PM readback. Never submit a real secret to a model as a test. Use CLI set/list/forget and verify
   forgotten facts are absent from the next index, including after restart. Restore the reviewed seed facts.
4. Start `/loop 5m SHADOW PM: findings only, no board writes or releases. Use the bound PM runbook and registered project;
   inspect board status and taskq profile telemetry; report board and worker links and proposed actions.` Keep the loop
   running for at least one hour. Record every actual tick, UTC time, request evidence, notes read/written, installed
   TaskQ reads and report. Confirm each report has an observed board link and workers table with links rather than bare IDs.
5. Capture the Claude PM ticks for exactly the same UTC window. Compare asks, stuck work, failed work, pending releases
   and telemetry alerts against the same observations. Account for source changes and tick offsets explicitly; missing
   observations are unknown, never a match. Confirm the shadow performed no board writes.
6. Stop with `/loop stop`; verify no pending tick runs after the active turn settles. Record the five-category match table
   in `docs/acceptance.md`, linking each side's evidence and explaining every mismatch. Retain sanitized traces/screenshots
   in the task's evidence attachment. A failure remains open for correction inside this task.

## Cleanup and rollback

Stop the loop before unbinding with `pilotq pm unbind ROUTE --json`. Unbinding hides notes and preserves them for
reconnection; forget individual notes explicitly with `pilotq pm notes ROUTE forget ID --json` when appropriate.
Remove only qualification-created grants. Roll back TEST only through the compatible-schema §9 procedure; rollback
cannot undo external effects. No board writes should need reversal in a findings-only shadow run.
