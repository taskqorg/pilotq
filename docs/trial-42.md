# Human trial report: task #42

The owner ended the trial on 2026-10-29 using the evidence available that day. The trial exposed useful work and defects, but does not qualify two-user behavior, the seven-day gate 4 baseline or Hermes production cutover (§10).

## Scope and evidence

The planned window was 2026-10-27T15:15Z to 2026-11-03T15:15Z. The owner's early-stop decision changes the actual collection window to 2026-10-27T15:15Z through 2026-10-29T20:46Z. The owner approved the simplified tester list through the readiness ask and named the owner and Simon as administrators on 2026-10-27. Participation does not itself prove grant application (R10).

Sources are the [#42 task log](https://github.com/taskqorg/pilotq/issues/42), especially the [PM export](https://github.com/taskqorg/pilotq/issues/42#issuecomment-6507962947), and the linked defect acceptance below. The PM supplied `pilot report --json` and `audit --since 1793114100 --json` from pds18, PilotQ 0.2.11. Audit rows span 2026-10-28T18:19Z through 2026-10-29T19:03Z. No exact source SHA or binary hash accompanies this export; it does not identify every earlier release used in the window.

The worker could read service status, but audit and pilot report collection both failed with `attempt to write a readonly database` under the worker sandbox. The PM export supplies the report evidence; the worker did not independently reproduce its counts or inspect private message text (§8, R17). No grants, service changes or model calls were performed for this report.

The PM's 2026-10-28 readiness note records installed 0.2.2 (`fd1737f`) A40 14/14 in 210 seconds and successful setup checks. This is supplied automatic evidence, separate from human use and from the later 0.2.11 export (§9).

## Observations

| Measure | PM export |
|---|---:|
| Accepted requests | 98 |
| Simple requests | 71 |
| Active users / routes / projects | 1 / 3 / 2 |
| Tasks | 11 |
| Recorded interventions / outages | 0 / 0 |
| Pilot-report failures | 8 |
| Audit answered / failed / uncertain | 88 / 9 / 1 |
| Delivered / superseded / undelivered delivery records | 318 / 51 / 0 |

Audit outcomes sum to 98. The pilot report's 8 failures differ from the audit's 9; the supplied aggregate does not explain the difference. Neither count is substituted for the other. Delivery records are separate from requests, so 318 deliveries are not 318 successful requests (§4). Zero recorded intervention/outage notes do not establish that no interventions or outages occurred: the PM explicitly records manual endings and a login drill below.

Reported tool counts: `task_plan` 15, `task_status` 5, `project_list` 4, `issue_read` 3, `issue_comment` 3, `task_run_plan` 3 and `project_use` 2. These counts are the supplied tool summary, not a claim that every accepted request invoked a tool.

## Request-linked exceptions and defect disposition

Request IDs refer to the installation journal retained through the host move. The export does not provide a per-request host or build mapping.

| Request IDs | Observed result from PM | Disposition |
|---|---|---|
| 12, 14, 28 | Unsupported input (`kind other`); refusal sent | Input refusal; exact media types absent, so compliance with current image/audio support is unknown |
| 19, 20 | Codex 401 `Missing bearer` on `api.openai.com` during #84 drill / host login gap | Recorded operational failure; [#84](https://github.com/taskqorg/pilotq/issues/84) tracks the drill. This does not establish billed API use or fallback (R9) |
| 35 | Codex did not finish within 60 seconds on a multi-plan run; uncertain, not repeated | [#110 A3](https://github.com/taskqorg/pilotq/issues/110) explicitly tracks request 35 and service-side launch; already closed, no duplicate filed |
| 43, 45 | PM manually ended requests after answer write landed but `end_answer` did not run; duplicate answer | [#109 A1–A3](https://github.com/taskqorg/pilotq/issues/109) tracks request 43, confirmation and stale-run recovery; already closed. [#108](https://github.com/taskqorg/pilotq/issues/108) tracks answer reliability |
| 60 | `/bogus` refused as unknown command | Expected refusal under §9 |
| 68 | Voice recognition not configured | [#120](https://github.com/taskqorg/pilotq/issues/120) explicitly tracks the host setup gap and doctor probe; already closed |
| 97 | Stale question answer refused | Expected protection under §5; available export does not include the question revision |

The exception list mixes refusals, operational failures and uncertainty; its row count is not the failed-request count. Existing tasks cover all three actionable defects requested by the PM: timeout, answer completion and voice setup. No new held defect is needed for an already tracked symptom. Closed board state alone is not a post-fix replay of these trial requests; this report claims no before/after live pass.

## Tester feedback and limits

Only the owner, Alex K, used the bot. Simon sent no requests and supplied no feedback. The owner expressed feedback through tasks [#81](https://github.com/taskqorg/pilotq/issues/81), [#82](https://github.com/taskqorg/pilotq/issues/82), [#89](https://github.com/taskqorg/pilotq/issues/89), [#90](https://github.com/taskqorg/pilotq/issues/90), [#91](https://github.com/taskqorg/pilotq/issues/91), [#92](https://github.com/taskqorg/pilotq/issues/92), [#93](https://github.com/taskqorg/pilotq/issues/93), [#106](https://github.com/taskqorg/pilotq/issues/106) and [#108](https://github.com/taskqorg/pilotq/issues/108): service visibility, reply layout, voice, conversation continuity, planning and commands/answer reliability. No separate satisfaction score or Simon acceptance is inferred.

A1 is documented beside the scripted-run decision in the contract and acceptance cases. A2 has owner approval in the task log. For A3 the export supplies accepted request/audit counts and the existing tester guide; it does not include historical grant receipts or independently prove an audit for each individual request. A4 is this bounded report with request-linked exceptions, available feedback and existing defect links. The complete list of 88 answered request IDs and their individual outcomes was not supplied and remains unavailable here. The owner explicitly ended collection on this evidence; no longer observation window or additional tester run is imposed.
