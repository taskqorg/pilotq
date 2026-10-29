# Status line research for #119

The owner selected a footer on final replies for DMs, ordinary groups and forum
topics in the answer to #119 at revision `6490594957`. The implementation resumes after #106 reached main. The sanitized Hermes source was inspected and the A2 read-only reviewer
cross-check passed before code.

## Placement comparison

| Placement | Fit and limits |
| --- | --- |
| Reply footer, selected | Keeps the observation beside its answer in all three routes. Needs no additional administrator rights or separate send. Repeats on each reply and can wrap on narrow screens. |
| One edited pinned message | Offers one current display, but can be far from the answer. Group pinning requires `can_pin_messages`; channel pinning requires `can_edit_messages`. Separate ordinary-group conversations would compete for a shared display unless separately correlated. |
| Topic title | Only fits topic routes, replaces a navigational label and has a 128-character limit. In supergroups it requires `can_manage_topics`, unless the bot created the topic. It cannot serve ordinary DMs or groups uniformly. |
| Draft or ephemeral message | Temporary output cannot replace a durable status observation attached to the final answer (§4). Drafts serve private chats and expire; they do not provide a uniform group surface. |
| Rich footer block | Telegram now offers `InputRichBlockFooter`. It is a possible rendering of the selected placement, rather than another location. The existing rich-to-HTML/plain fallback must still carry the line (§4). |

Telegram's [FAQ](https://core.telegram.org/bots/faq#my-bot-is-hitting-limits-how-do-i-avoid-this)
advises at most one send per second per chat and 20 per minute in groups. These
are send guidance, not a documented universal edit allowance. Edits must obey
`retry_after`; PilotQ already specifies progress edits at most once per two
seconds (§12) and a shared persisted cooldown (§4). An immutable final footer
adds neither a periodic edit loop nor another send.

Permissions and field limits were checked against the
[Bot API](https://core.telegram.org/bots/api#pinchatmessage), including
[topic edits](https://core.telegram.org/bots/api#editforumtopic),
[drafts](https://core.telegram.org/bots/api#sendmessagedraft) and
[rich footer blocks](https://core.telegram.org/bots/api#inputrichblockfooter).

## Data and boundaries

The minimal requested data is the selected model, current conversation context
used, Codex weekly account quota remaining and its reset. The canonical Unicode
families, thresholds, unknown/stale notation and reset rounding are owned by
[the pinned specq indicators](https://github.com/taskqorg/specq/blob/12a2219b8d1bbef28e2bab4e3824e10e269bee24/caveman/rules/indicators.md).
No account identifier, credential, email or subscription detail is needed (§8).

At the inspected base `1bb85e22faca12972b87c20385a610b95c964677`, the manager
records the runtime-returned model, but has no context/quota event projection.
`/new` resets the conversation without a model call and returns a generic line.
Its model announcement must preserve that deterministic recovery path (§2).
The implementation must integrate #106's command/model choice rather than
announce the last run's model as the new selection. This dependency is not yet
qualified by this research.

Worker question cards must retain their explicit no-footer rule (§12, #97).
No policy to expose the subscription quota to additional people is inferred
from a bot credential; the requested status remains within admitted routes
(R10, §8).

## Outstanding evidence

The PM supplied a read-only, sanitized reference bundle for #119. Inspection
covered `runtime_footer.py`, `plugins.py`, the plugin implementation, manifest,
README, config excerpt and checkpoint note. The implementation and manifest
agree on version 1.3.0: native Telegram footer disabled, `post_api_request`
captures per-session/per-turn context and `transform_llm_output` appends the
single line after final text. The native formatter uses the model basename and
last-call prompt occupancy. The plugin matches the pinned face/moon thresholds.

Quota reads use a five-minute per-account cache and four-second timeout, select
the 604800-second weekly window and show remaining rather than used quota.
Missing or invalid quota with no usable retained value, or an expired reset,
is unavailable; failed or malformed refreshes retain an unexpired value marked
stale. Account changes invalidate the cache. These are reference mechanisms,
not permission to inspect or copy Hermes credentials (§8). PilotQ should obtain
measurements through its own Codex runtime boundary (R9, R12).

The supplied README and older checkpoint describe a `transform_runtime_footer`
core extension, but neither supplied core file contains that hook. The current
plugin instead registers `transform_llm_output`, consistent with the manifest
and disabled native-footer config. Therefore the README/checkpoint do not prove
the current hook or live delivery. No Hermes restart or tests were run. The
sanitized source establishes the formatting algorithm, not deployment status.

A read-only reviewer cross-checked this stage against the sanitized source and
A1/A2 acceptance. Task fit passed: approved placement, alternatives, rights,
rate-limit qualifications, minimal data and the stale-reference discrepancy
are covered. Implementation quality had no blocking findings in this research;
no executable or live tests were run. Update the owning contract before
implementation. A4 still requires
FRA screenshots covering `/new`, a model switch, DM and topic replies and a
missing quota source; local tests cannot establish that visual acceptance.

## Implementation and live evidence

#106 reached main at `f501f99bfac85deb3263ff6c9a54c81a0fa85749`.
The owner extended §9 to #119 on 2026-10-29. One worker ask will name the
exact candidate and DM/topic steps; the PM performs the authorized run and
attaches screenshots. A4 remains pending until those observations exist.

The adapter consumes `thread/tokenUsage/updated` (last input tokens and context
window) and the ordinary `codex` weekly window from `account/rateLimits/read`
or `account/rateLimits/updated`. Field shapes were checked against schemas
generated by the installed Codex app-server. Only numeric measurements enter
the footer; account identifiers, plan types and credit information are dropped.
No credentials are read. There is no cross-turn cache: this deliberately avoids
account-change invalidation and stale context after reset or model switch.
Optional quota responses are bounded by the original turn deadline and never
change a successful native terminal outcome into failure.
