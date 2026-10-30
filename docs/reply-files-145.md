# Telegram reply file qualification (#145)

PilotQ forwards admitted Telegram photos and documents through TaskQ's existing `--file` interface under [§5](../pilotq.md#5-taskq-integration). This is a PM live recipe, not a claim of live acceptance. Task note 145.1 assigns release and A2 execution to the PM; workers verify A1/A3 locally and in CI.

## Boundary and local evidence

The TaskQ CLI minimum is 4.6.43 for this handoff (§9); the service minimum remains 4.6.40. The pinned [CLI contract](https://github.com/taskqorg/taskq/blob/v4.6.43/docs/design/answer-card.md#human-replies) documents repeated `--file`, optional text and original operation reconciliation.

The journal retains the caption, sender, route, reply reference and Telegram file identity before polling confirmation. Question-card media replies enter the deterministic answer path. Other media may be selected by the manager for steer or owner requeue; ambiguous destinations require clarification. After clarification, all three tools can select the earlier file input with `source_request`, validated against the current caller and conversation generation. Current project grants, owner-only file requeue, target validation and operation reconciliation still apply. File-only inputs are supported. Each Telegram message remains a separate request, including album members (§12); a caption and file travel together. Arbitrary documents are forwarded as source files, not interpreted by the manager. Audio documents sent as message replies use this file path; ordinary voice/audio input retains §12 recognition.

Downloads use the existing 20 MiB cap, ten-second per-download limit and original sixty-second operation budget. The adapter accepts journal-owned identities, never model-supplied paths. Original safe filenames survive. Protected temporary files are removed; private copies beside the journal remain while delivery is unresolved and are removed after an accepted durable receipt (§7). A missing download, unsafe filename, provider rejection or unknown receipt retains the reply with diagnostics and a remedy. Revoked grants and a stopped request block dispatch after downloading. Recovery uses the original operation and revision, never a new command.

Before the fix, `reply_files_ingress` against the original parser failed with zero captured files (expected one). After the fix, `cargo test --locked reply_files` passed three unit/component checks and one service-chain check. The service check covers file-only answer, mixed-text steer, owner file-only requeue, owner-only refusal, lost-response restart and exact provider refusal/fix with retained bytes. The checks cover Telegram ingress, durable identity/admission/source isolation, and real HTTP photo/document bytes passed as readable files to a TaskQ-shaped CLI. The CLI fake checks task/revision and operation identity. It does not qualify a live Telegram client, board upload or worker delivery. Existing TaskQ receipt, stale question and fence checks remain part of the candidate gate.

The review found a second defect: a file followed by text clarifying its destination lost the file for steer and owner requeue. The expanded `reply_files_service` check against the old dispatch line failed with `clarification lost the earlier file for ["task_steer", "task_requeue"]`. The fix uses the existing validated source selector for both tools; the same check covers both clarified file-only forwards without a guessed write on the initial message. The fixed `cargo test --locked reply_files` passes; both tools also refuse another caller’s file source without a board command. Journal checks retain route/generation isolation; existing grant, owner, stale-revision and same-operation recovery protections remain unchanged.

The next review found that release verification still provisioned a 4.6.40 CLI receipt. On the unchanged retained 0.2.13 debug binary, `scripts/verify-release.sh /tmp/pilotq-145-dist 0.2.13` reproduced `taskq: found 4.6.40; minimum 4.6.43` and exited 1. Updating only that receipt to 4.6.43 made the same command exit 0, including checksum rejection, activation and offline selftest. The service fixture still returns 4.6.40, preserving its separate minimum (§9). This is isolated updater evidence, not fresh static-artifact qualification. Cargo is unavailable in this attempt's sandbox; fresh compilation, candidate tests and static-artifact verification remain CI checks.

## PM A2 procedure on PDS 18

1. Publish the reviewed merge-result-green candidate through the existing `live` workflow, then update the existing installation with the §9/§12 drain and rollback protections. Record PilotQ source SHA, release, binary digest, health, selftest, doctor and journal continuity. Do not install a worker branch or create another bot.
2. Use the existing authorized owner's Telegram Desktop client and bot conversation. Record the exact Telegram Desktop version from Settings/About, OS, PilotQ source/binary identity, and TaskQ release/source revision and executable checksum from its verified release receipt. Keep bot credentials and unrelated conversation content out of evidence. The client version is pending the PM observation, not inferred from this recipe.
3. Continue this real task #145 for its remaining A2 acceptance, with a worker on PDS 18. Coordinate an actual question asking for the qualification inputs. Do not create placeholder tasks. Record its board, worker attempt and question revision; the owner retains the existing `chat`, project and TaskQ grants.
4. Create these invented fixture files on the client machine:

   ```sh
   python3 - <<'PY'
   import hashlib, struct, zlib
   from pathlib import Path
   def chunk(kind, data):
       return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
   png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 32, 32, 8, 2, 0, 0, 0))
   png += chunk(b'IDAT', zlib.compress((b'\0' + b'\x20\x80\xe0' * 32) * 32)) + chunk(b'IEND', b'')
   Path('pilotq-145.png').write_bytes(png)
   Path('pilotq-145-20MB.bin').write_bytes(bytes(range(256)) * 78125)
   Path('pilotq-145-mixed.txt').write_text('Invented #145 acceptance evidence.\n', encoding='utf-8')
   for name in ['pilotq-145.png', 'pilotq-145-20MB.bin', 'pilotq-145-mixed.txt']:
       data = Path(name).read_bytes()
       print(name, len(data), hashlib.sha256(data).hexdigest())
   PY
   ```

5. Reply to the relayed worker question with `pilotq-145.png` sent as a Telegram **photo**, without a caption. Confirm TaskQ records an answer against that question revision and the intended worker receives a readable image. Telegram may recompress a photo: compare the worker bytes to the bot-downloaded bytes, not the original PNG. Record dimensions, byte length, digest and successful image decoding.
6. While that worker is doing #145, send `pilotq-145-20MB.bin` as a **file/document**, captioned “Forward this qualification document to the worker on pilotq #145.” Confirm a steer receipt for the intended task/attempt. The worker reads exactly 20,000,000 bytes and reports the fixture SHA256. This is decimal 20 MB, below PilotQ's 20 MiB transport cap.
7. From the installation owner, send `pilotq-145-mixed.txt` with the caption “Requeue pilotq #145 with this qualification evidence; verify its bytes and preserve the earlier receipts.” Confirm owner requeue, the new worker attempt, the caption and readable file with matching digest. If using several Telegram files, verify every message's separate receipt and worker copy; do not infer batch atomicity.
8. Record only the relevant Telegram message/caption, task URL, revision, operation receipt, attempt and worker copy name/length/SHA256. Worker readback must name each verified `.taskq/reply-files/...` copy and confirm actual reading; a board link or upload success alone does not pass A2.
9. Check a repeated delivery of the same operation via supported recovery: one accepted board command, unchanged operation ID and no second worker input. For a refusal, retain the original request/files, record the exact reason/fix, repair that cause, and reconcile the original operation. Never resend an uncertain reply as a new request before checking status.

If the host/client run is unavailable, A2 remains **not run**. The PM posts exact revisions and relevant readbacks on #145 after execution. No Hermes consumer cutover is authorized.
