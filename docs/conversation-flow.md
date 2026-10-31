# Conversation flow

Presentation detail of the [Conversation contract](conversation-contract.md#conversation-contract), owner task #165. This table governs both front ends (R13); Telegram message edits and buttons project the same durable receipts. It changes presentation, never grants or TaskQ authority.

## Reply shape

Normal replies lead with one short essence line in the person's language, followed by at most three short bullets. When something waits, end with one “Next:” line (“Далее:” in Russian) naming who acts and what they do. Keep verified findings and material limitations; do not append unrelated release or DEV health information. Read and show release or health observations only when the person's question concerns release, deployment or health.

Fixed controls, plan states, cancellation and failure lines follow the person's language, including controls sent after a Russian message. Task links use `pilotq#205 — short title` (with the actual repository, number and verified title); omit an unknown title. Held tasks name the actor they await from fresh TaskQ evidence; when evidence is missing, say the waiting actor is unknown rather than inventing a reason. Separate tasks in plan and status cards with a blank line. Plans and complete source question cards retain the stage table's required content and controls.

A failed request produces one plain localized failure line, with no commands, paths, tool output or suggestion to plan its remainder. Preserve diagnostics and findings in audit. After restart, summarize recorded findings and unconfirmed actions by count in the person’s language; raw finding bodies and tool targets stay in audit. A recorded finding does not prove an external effect succeeded. Discard unlaunched draft revisions made by the failed turn, preserving prior successful drafts and authorized launch receipts. `/status` names a model failure reason in plain words from a bounded category; raw diagnostics remain in audit.

| Stage / trigger | Text in the person's language | Buttons | Press effect | Replacement |
|---|---|---|---|---|
| Accepted input | 👀 reaction; existing bounded receipt fallback | None | None | Final answer; one queue notice after 30 seconds |
| Discussion | Answer or one concrete question with complete numbered choices | Two to four choices when needed | Record the person's answer; no launch | Same question, no buttons, one chosen-option-and-actor line |
| Plan saved or revised | Per task: title, “We will: …”, “Done when: …” | Launch, Cancel | Launch authorizes the displayed immutable scope; Cancel closes without board effect or model call | New display removes old buttons; launch/cancel edits source with one outcome line |
| Launch accepted | Verified task links; distinguish held, queued and working | None | None | Same plan receives launch outcome; one status card tracks the launch |
| Queued | Last confirmed waiting state, observation age and verified links | None | None | Edit status card on fresh working/question/done observation |
| Working | Last confirmed working state, observation age and verified links | None | None | Edit status card; never infer execution from admission |
| Worker question | Complete TaskQ answer card, no footer | Number and label, recommendation star | Answer only current question under current grants | Edit question with accepted option/text and actor; remove buttons; status card says waiting for answer |
| Done | One factual effect line followed by trusted TaskQ closing result and limitations; explicitly unconfirmed when unavailable | None | None | Edit status card; preserve unanswered question and terminal evidence |
| Cancelled plan | Original summary plus “Plan cancelled.” | None | None | Same message, no new acknowledgement |
| Stale/foreign/reset/repeated callback | Short popup toast | None | No effect | No chat message |
| Long launch | Original plan plus one phase and elapsed-time line | None | None | Coalesced, rate-limited edit of that source message; completion replaces the line |
| Give-up | Original plan plus one plain problem line | Retry, Cancel | Retry resumes the same authorized operations; Cancel stops only remaining submission | Same message; remove buttons; old controls get a toast |
| Deadline | One plain limit line, readable completed read findings and an offer to plan the remainder | None | None | Preserve confirmed observations without asserting uncertain effects |
| Failure episode | One plain line describing what could not finish | None | None | No repeated retry notices; one resolution line only after verified recovery |
| Partial or uncertain launch | Plain line; preserve verified links to filed tasks | None | None | Same status card; reconcile stable operations, never claim missing work exists |
| `/status` | Health, conversation activity, latest model/context/quota observation and age | None | Deterministic read | Separate control reply; unavailable/stale observations labeled |

Task acceptance and scope remain in the task body. A plan summary must not enumerate these lists. Existing drafts without a dedicated result summary use their first acceptance line as a compatibility summary. New drafts supply a single result summary. Internal revisions, operation IDs, JSON, CLI errors and retries remain in audit and PM telemetry, including after restart. Button removal and outcome edits persist through the existing outbox; recovery never reruns a completed effect to repair delivery (§4).

## Before cases and qualification

The owner's two screenshot transcriptions are recorded on #165 and retained as fixtures without credentials: the verbose plan/local-save/footer card and repeated technical launch errors. They reproduce the observed presentation failure, not a new product decision. Fixture checks protect every card and stage from recurrence, with English and Russian plan summaries and failure messages. Existing journal traces retain duplicate, stale, reset, grant, receipt and delivery protection.

Local rendering and adapter checks do not qualify Telegram appearance. After merge and release, the PM checks plan, launch, cancel, question and failure on PDS 18 against the snapshots (A7), recording exact binary/source identity and screenshots under §9/§12. Workers make no installation changes.
