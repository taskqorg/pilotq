# Conversation flow

Presentation detail of the [Conversation contract](conversation-contract.md#conversation-contract), owner task #165. This table governs both front ends (R13); Telegram message edits and buttons project the same durable receipts. It changes presentation, never grants or TaskQ authority.

## Reply shape

Normal replies lead with one short essence line in the person's language, followed by at most three short bullets. When something waits, end with one “Next:” line (“Далее:” in Russian) naming who acts and what they do. Keep verified findings and material limitations; do not append unrelated release or DEV health information. Read and show release or health observations only when the person's question concerns release, deployment or health.

Task updates explain the effect and why it matters, with a specq semantic status icon, observed state and verified `repo#N — title` link. Keep title, trusted closing verdict and limitations together; closure never implies release or deployment. Every model-free task card uses one format: verified link and meaningful title, observed state, the essence (the first line of the task's filed goal, as literal source) and, when closed, the available trusted outcome and material limitations, unfiltered. Say the outcome is unconfirmed only when no qualified report exists; a failed read of an available report is retried, never shown as unknown. Waiting updates name the next authorized actor, or explicitly say that actor is unknown.

Decisions must be understandable without reading a specification. Preserve original option numbers and complete pros and cons; explain the recommendation and its grounded reason. Do not invent missing trade-offs, reasons, state or evidence. Keep literal immutable source in audit/evidence, separate from the localized explanation. Raw source alone is not an explanation. With PM review enabled (`/review on`), its existing event-woken turn explains source cards in the recipient’s language; with PM review off, deterministic cards retain source text without using a model. A PM explanation states what the trusted outcome and each limitation mean for the recipient; it never invents, drops or softens them, and it calls a fact unknown only when the source card lacks it. Ordinary model replies and PM explanations follow this rule for questions, results and errors in English and Russian, including rich Markdown, HTML and plain fallback. Model-free cards localize owned state and control labels; they do not pretend to translate arbitrary source prose. Explanations confer no authority and never rewrite revision, actor, grants, option mapping or receipts. Encode literal source once at the output sink.

### Question explanations

With PM review on, a worker question has one primary explanation: the card itself. It states the issue, its practical impact, the applicable requirement in plain language and the next actor. A criterion code alone never explains a problem. Raw English, criterion codes, hashes and attempt IDs stay in the expandable source details and the audit. Inspect the current clause before claiming a violation or exception; a missing clause reads as an unverified rule, and the card then claims no violation. The explanation binds to one task and question revision, and a clause it cites must exist in that task's text or in a Markdown file under the registered root.

The explanation preserves original option numbers, the source star, verified links and every material trade-off, and it adds a recommendation reason only when one is evidenced. A PM technical action and a real owner choice are named as different next actors; the explanation invents no product answer and grants no authority. Buttons keep their source numbers with plain labels, so an answer still binds to the original question.

| Path | What the person sees |
|---|---|
| PM review off | The complete literal source card with its buttons; no model runs |
| PM on, explanation pending, delayed or failed | A compact card with every number, label, star and link, the expandable source and a line saying an explanation replaces that line when ready |
| Explanation accepted | The same message and buttons, edited in place; a repeated identical explanation queues nothing |
| Stale, answered or superseded revision | The explanation is refused; a superseded card returns to its source body with the existing supersession line |
| Question message missing or never delivered | The explanation sends nothing, because a new message would duplicate controls that cannot bind |
| Rich, HTML or plain fallback | Rich and HTML collapse the source; plain keeps it as quoted lines. An explanation stays one message, moving a long source behind the task link |
| Restart | Queued edits and compact cards survive in the outbox; an interrupted model turn is not replayed, so its compact card remains |
| Card in another shared route or the CLI | That card stays the literal source; the PM reply carries the same parts in its own route, because a review writes only to its route and its author's own DM |

With PM review on, other task events keep state and verified link on the status card and move literal goal, outcome and limits into expandable details, and the PM reply is the primary explanation.

Fixed controls, plan states, cancellation and failure lines follow the person's language, including controls sent after a Russian message. A message without words, such as `/help`, takes the language of the sender's latest own text in the same chat, so every topic of that chat answers alike; other people's messages never decide it. Task links use `pilotq#205 — short title` (with the actual repository, number and verified title); omit an unknown title. Held tasks name the actor they await from fresh TaskQ evidence; when evidence is missing, say the waiting actor is unknown rather than inventing a reason. Separate tasks in plan and status cards with a blank line. Plans and complete source question cards retain the stage table's required content and controls.

A failed request produces one plain localized failure line, with no commands, paths, tool output or suggestion to plan its remainder. Preserve diagnostics and findings in audit. After restart, deliver safe localized completed read findings, including the available project names; summarize diagnostic-only findings and unconfirmed actions without commands or paths. Native findings and tool targets stay in audit. A recorded finding does not prove an external effect succeeded. Discard unlaunched draft revisions made by the failed turn, preserving prior successful drafts and authorized launch receipts. `/status` names a model failure reason in plain words from a bounded category; raw diagnostics remain in audit.

| Stage / trigger | Text in the person's language | Buttons | Press effect | Replacement |
|---|---|---|---|---|
| Accepted input | 👀 reaction; existing bounded receipt fallback | None | None | Final answer; one queue notice after 30 seconds |
| Discussion | Answer or one concrete question with complete numbered choices | Two to four choices when needed | Record the person's answer; no launch | Same question, no buttons, one chosen-option-and-actor line |
| Plan saved or revised | Per task: title, “We will: …”, “Done when: …” | Launch when setup is ready; otherwise project setup reason and owner Recheck step. Cancel remains available | Launch authorizes the displayed immutable scope; Cancel closes without board effect or model call | New display removes old buttons; launch/cancel edits source with one outcome line |
| Launch accepted | Verified task links; distinguish held, queued and working | None | None | Same plan receives launch outcome; one status card tracks the launch |
| Queued | Last confirmed waiting state, observation age and verified links | None | None | Edit status card on fresh working/question/done observation |
| Working | Last confirmed working state, observation age and verified links | None | None | Edit status card; never infer execution from admission |
| Worker question | PM off: complete numbered source card. PM on: [one explanation on the card](#question-explanations), no footer | Number and label, recommendation star | Answer only current question under current grants | Edit question with accepted option/text and actor; remove buttons; status card says waiting for answer |
| Done | One factual effect line followed by trusted TaskQ closing result and limitations; explicitly unconfirmed when unavailable | None | None | Edit status card; preserve unanswered question and terminal evidence |
| Cancelled plan | Original summary plus “Plan cancelled.” | None | None | Same message, no new acknowledgement |
| Repeated answered current-question callback | Exact saved answer; durable confirmation in the original route | None | No TaskQ write or continuation | Replace a delivered queue notice with the terminal answer; popup expiry does not lose confirmation |
| Stale/foreign/reset/other repeated callback | Short popup toast | None | No effect | No chat message |
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
