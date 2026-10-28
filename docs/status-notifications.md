# Service state and notifications

Research for [task #81](https://github.com/taskqorg/pilotq/issues/81). It proposes how PilotQ shows its own state and notifies about restarts, updates, failures and recovery across several boards and chats. It is a proposal: nothing here changes [pilotq.md](../pilotq.md) until the owner chooses an option. Notification grammar follows [specq notifications](https://github.com/taskqorg/specq/blob/main/caveman/rules/notifications.md).

## Current state (origin/main 93d15dc)

| Area | What exists | Gap |
|---|---|---|
| Health | `status.json` and `pilotq status` report `poll`, `storage`, `model`, `taskq`, `delivery` with age; unobserved reads unknown (§7) | Operator-only; no Telegram view of service health |
| `/status` in Telegram | Counts running and queued requests of the current conversation | Says nothing about the service, its version or degraded dependencies |
| Task notices | One edited status message per task in its originating route; question and outcome are separate messages (§5) | Per-task only; correct by design, keep as is |
| Rollback | One owner DM per failed/restored version pair through the outbox (#77) | The only service notice; no restart, outage or recovery notice |
| Stall/failure per request | One short failure line to the request's route (§12) | Covers a request, not the service |
| PilotQ dead | §7 points to "the existing host monitor" | ServerQ `failed-services` reads system units only; PilotQ is a user unit. DashQ does not read PilotQ. **Nobody detects a dead PilotQ today** |

Many boards and chats matter because a service event (Codex login lost, TaskQ unreachable, restart) affects every route at once. Broadcasting to each route multiplies one incident by the number of chats.

## A1. Options

### State display

| Option | What a person sees | Cost | Risk |
|---|---|---|---|
| **S1 Extended `/status`** (pull) | `/status` adds one service line: version and every non-ok health source with its age, plus the caller's queue. Same path as today, no model | Small: reads the existing health table | None new; only seen when asked |
| S2 Pinned live card per chat | One edited "service state" message per route | An edit per change per route; pin rights in every group | Edit storms; stale card when PilotQ is dead looks healthy |
| S3 DashQ panel | PilotQ appears in DashQ Services/Hosts through coreq enrollment | Enrollment client, DashQ projection and owner authorization | Largest; DashQ is operator-facing, not chat users |

### Notification routing

| Option | Who receives service events | Fits many boards/chats |
|---|---|---|
| **N1 Owner DM** | Service incidents go to `owner_id` only, as #77 already does. Routes hear only about their own requests | Yes: one message per incident regardless of route count |
| N2 Owner DM + affected routes | Owner gets the incident; a route that has a request waiting on the failed dependency also gets one line about that request | Yes, bounded by affected requests, not by chats |
| N3 Configured ops route | A route marked `ops` in `pilotq routes set` (a chat or forum topic) receives service incidents; owner DM is the fallback when none is set | Yes; lets the team see incidents without a second bot |
| N4 Broadcast to every route | Every active chat | No: one incident becomes N messages; rejected |

Board events stay where §5 puts them: per task, in the originating route. A board-wide or server-wide feed is TaskQ/DashQ territory (§1 "no new task board").

## A2. Noise and duplicate rules

1. **One incident per episode.** Key = `kind:source[:detail]` (for example `dependency:model:login`). Opening inserts the key in the journal with the outbox row in one transaction, as `rollback_notice` does. A restart reopens nothing that is already open.
2. **Hold-down before opening.** A source must stay failed for 2 minutes (poll, TaskQ, delivery) or fail on two consecutive model runs before an incident opens. Transient retries stay silent.
3. **Recovery needs fresh evidence.** Close only after a fresh successful observation of the same source (a poll, a model terminal event, a complete TaskQ snapshot). Text: "recovered", never "work completed". A cleared timer alone says nothing.
4. **Planned events are silent.** Successful update, drained restart and `pilotq service stop` send nothing. A failed update, a rollback or an unplanned restart (no drain marker) sends one incident.
5. **Delivery failure is not reported through delivery.** If `delivery` itself fails, the incident waits in the outbox and is retried under §4/§7 limits; it is never rerouted to another chat.
6. **Rate cap.** At most one service message per incident key per 6 hours while it stays open, and at most 10 service messages per day overall; excess incidents stay in `pilotq status` and the audit.
7. **No internal IDs or raw errors** in Telegram (§4, task #68). Diagnostics stay in `status.json` and audit.

### Detecting that PilotQ itself is unavailable

PilotQ cannot report its own total outage while it is down (§12). Options, cheapest first:

- **U1 Downtime note on next start.** Each service pass already writes `status.json`; it would also store `alive_at` in the journal meta table. On the next healthy start, a gap above 5 minutes without a drain marker sends one owner incident: "PilotQ was unavailable from … to …; Telegram keeps undelivered updates for 24 hours." Detects after the fact; no new component.
- **U2 Host monitor reads `status.json` age.** Extend ServerQ to read user units or `status.json` freshness. Real-time, but it is a ServerQ contract change and its alert path is the operator's monitor, not Telegram.
- **U3 DashQ enrollment heartbeat (S3).** DashQ marks PilotQ stale after its freshness limit and shows an Attention item. Real-time, largest cost.
- systemd `OnFailure=` with a curl sender: rejected, a second notifier holding the bot token outside the outbox.

## A3. Example messages

Plain text, one line, in the recipient's language; icons per the specq indicator catalog.

```text
⚠️ PilotQ: Codex login lost · new requests wait · run pilotq doctor on the host
✅ PilotQ: Codex login restored · waiting requests resume
⚠️ PilotQ: TaskQ board unavailable for 3 min · plans stay unsent · retrying
✅ PilotQ: TaskQ board reachable again · held plans resume
⚠️ PilotQ restarted without a planned stop · 2 requests resumed, 0 uncertain
⚠️ PilotQ was unavailable 14:02–14:31 · messages from that time are processed now
❌ PilotQ update failed: 0.2.6; restored 0.2.5.
```

Extended `/status` (S1):

```text
PilotQ 0.2.5 · Codex login lost 12 min ago · TaskQ ok · Telegram ok
This conversation: 1 running, 0 queued.
```

A route affected under N2 gets one line about its own request, for example "Your request waits: Codex login lost; it resumes automatically."

## Recommendation: minimal variant

**S1 + N1 + U1**, with the A2 rules. It reuses the health table, the outbox and the #77 incident path; it adds no new component, chat configuration or monitor. N3 and U2/U3 wait for evidence that the owner DM is not enough (for example a second operator).

Stages:

1. Contract edit (§4, §9, §12) for the chosen options, then extended `/status` (S1). Check: unit test of the reply for ok, failed and unknown sources.
2. Service incidents to the owner DM (N1): open/close with hold-down, episode key and rate cap for `model`, `taskq`, `poll`, unplanned restart. Check: journal tests for dedup across restart, hold-down and recovery only on fresh evidence.
3. Downtime note (U1). Check: test with a stale `alive_at` and with a drain marker (silent).
4. A40 on FRA and a fault drill: revoke the Codex login, stop TaskQ, kill the service.
5. Later, on evidence: N3 ops route, U2 or U3 real-time dead-service detection.
