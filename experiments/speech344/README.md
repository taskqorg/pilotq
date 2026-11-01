# First-window reuse experiment

**Stopped after the one authorized paired test.** PM note 6563635934 reports shared600s timeout, no OOM: readiness 4.514/4.376s, private Hebrew count identical/correct at 308.814/154.722s, original stress timed out in the remaining127.537s and patched stress never ran. Recipes below are retained evidence, not permission to rerun or start another corpus/model. No installed changes or ASR acceptance; the later owner note 6563703776 supersedes that quality/latency decision: retain Russian with the existing model, English only if that same model supports it, Hebrew removed. No further experiment is approved. See [terminal result and options](../../docs/research/344.md#terminal-paired-result-and-owner-decision).

This is an experimental patch for whisper.cpp 1.8.2, not a PilotQ engine selection. It addresses duplicate encoding after automatic language detection. It does not repair small's known Hebrew count error. No savings or correctness result is claimed.

## Identity and build

`manifest.json` pins upstream commit, original file hashes, patch hash and retained baseline binary. The upstream source archive is `https://codeload.github.com/ggml-org/whisper.cpp/tar.gz/4979e04f5dcaccb36057e059bbaed8a2f5288315`, SHA256 `ec8e33f4c838ed459ad0a23da255c0262f150a6f54839a17e504ddf1495305d5`. Its MIT license must accompany either build. Preserve the existing baseline binary and assets; build in a separate private directory, never installed paths.

After verifying the archive and manifest source hashes, extract it into a fresh private build workspace. From the extracted source root, apply this patch using its absolute repository path, then run:

```sh
cmake -S . -B build-344 -DCMAKE_BUILD_TYPE=Release \
  -DGGML_CUDA=OFF -DGGML_NATIVE=OFF \
  -DCMAKE_CXX_FLAGS=-DPILOTQ_344_EXPERIMENT
cmake --build build-344 --target whisper-cli -j2
sha256sum build-344/bin/whisper-cli
```

Record CMake/compiler versions, complete configure/build logs, effective cache flags, source/patch/license/output hashes and linked dependencies. The PM subsequently reproduced the uninstrumented experimental build in 101.749 seconds with no OOM under the unchanged actual limits; its SHA256 is `a3e6f4ae6303793b89adfba1edbbc25c4a6477ea1428e67f92831612d6927d9a`. That is build evidence only. CMake remains absent from the worker PATH. Do not provision it or infer a binary identity from the patch. The original binary remains SHA256 `30ab87ffca69ba2e03f6c9c110c158cc5d7e4099eaa0179793559215b99aed70`.

## Mechanism and boundaries

An explicit new parameter defaults false. Only the experimental pinned CLI opts in, with one processor and no VAD. Its existing callbacks only read their own abort flags; callers with callbacks that mutate encoder state must never opt in. Production PilotQ does not activate this field or binary.

A function-local token becomes eligible only after successful automatic detection over newly computed mel data, at offset zero, full/default context both before detection and at transcription, CPU execution and no prompt continuity. The token is consumed at the first transcription window, even on mismatch. Explicit language, changed context, absent audio, failed detection and new invocations cannot obtain a token. Later seeks run the original encoder. Encoder-begin callbacks still run before the decision; abort is checked before using cached features. Detection failures still return the original error. Cancellation returns failure without decoding cached output. The original decoder reset remains in place; no language, temperature, fallback, prompt or text-reference changes are made.

## Required proof before any larger run

PM execution must use the existing host isolation runner, actual MemoryMax1280M/High1024M/Swap256M/CPU150%/Tasks256, four recognizer threads and original deadlines. The worker cannot access that runner. No model inference was run for this patch.

First build and run an instrumented API boundary probe against both original and patched source. Record encoder counts and exact transcripts, not just exit codes. Cover default offset/context reuse, nonzero offset, reduced context followed by default context in the same state, explicit language, failed detection, empty audio, encoder-begin refusal and abort-before-reuse. Check decoder reset and a two-file batch versus separate invocations, with differing languages and silence between recordings. The original encoder path must handle every ineligible case; failures/cancellation must never emit cached transcription. Arbitrary mutating callbacks remain unsupported for opt-in. Stop on a mismatch; do not proceed to performance measurement.

Only after those boundaries pass, run one bounded paired correctness/performance proof using the unchanged baseline and new binary against identical unhinted audio/settings. Reuse `scripts/profile-speech-344.py`'s effective-cgroup guard and recording construction; its hard-coded baseline hash intentionally rejects this new binary, so it is not yet a patched-engine harness. Keep human ru/en/he, private Hebrew count, silence/readiness and long references unchanged. Record binary/model hashes, texts/languages, native encoder counts/times, wall time/RSS, actual isolation and cleanup. Readiness stays ten seconds; each recording stays at most five minutes with 25-second chunks and an actual 600-second total. Do not run a broader corpus before the boundary proof or retry unchanged failures. Insufficient benefit ends this experiment and returns the concrete quality/latency tradeoff to the owner.

Instrumented builds, API boundary execution, paired performance harness, model proof and independent review remain open. The source-only apply check cannot qualify any of them (§9, R19).

## Instrumented PM boundary handoff

The API harness and runner are now supplied. `instrumentation.patch` adds real encoder offset/context traces, decoder-reset traces and a read-only encoder counter. Its one explicit fault injector invalidates mel length between recordings to test failed detection after prior success; it is not a quality input or production change. `boundary.cpp` uses identical parameters in both variants except the reuse opt-in. It covers the default, opt-out, explicit-language, offset, reduced/default context transition, callback refusal, armed abort, failure recovery and differing-language files separated by silence. `multi` also compares the second file with a fresh independent invocation. Text and language must match exactly, not merely exit successfully. The small model's known Hebrew failure is still a failure; these boundary inputs provide no new quality verdict.

`prepare-boundaries.py` verifies the official archive, manifest source/patch/harness identities and the PM's existing CMake receipt, then builds **two new instrumented binaries** from the same source/flags. The baseline has only instrumentation; the patched variant also has the experimental patch. These hashes will differ from both retained CLI binaries and must be recorded before inference. Nothing replaces the originals. Both scripts fail closed unless their effective cgroup limits match the existing guard. Preparation has a 600-second deadline, logs and hash proof; no model loading occurs during preparation.

Use the existing host isolation runner for each command below. All uppercase arguments are explicit private-path or independently recorded SHA placeholders, not literal names. Do not change the runner's real memory/CPU/task limits or permissions. No worker access to that runner is needed or authorized.

```sh
python3 experiments/speech344/prepare-boundaries.py \
  --archive VERIFIED_UPSTREAM_ARCHIVE --cmake VERIFIED_PRIVATE_CMAKE \
  --cmake-sha256 VERIFIED_PRIVATE_CMAKE_SHA256 --output NEW_PRIVATE_BUILD_DIRECTORY

python3 experiments/speech344/run-boundaries.py \
  --baseline INSTRUMENTED_BASELINE --baseline-sha256 RECORDED_BASELINE_SHA256 \
  --patched INSTRUMENTED_PATCHED --patched-sha256 RECORDED_PATCHED_SHA256 \
  --model VERIFIED_SMALL_Q5_1 --case faildetect --output NEW_PRIVATE_CASE_DIRECTORY
```

After each successful command, run the same guarded second command with the next `--case` and a new output directory, in this order: `default`, `optout`, `offset`, `explicit`, `refuse`, `cancel`, `context`, `recovery`, `multi`. Stop the entire sequence at the first failure/mismatch or timeout; do not retry it unchanged. Each boundary sequence has its own actual 600-second deadline including decode and both variants. This does not pool or enlarge any recording's 600-second budget. `faildetect` additionally caps each initialization/empty-input invocation at ten seconds; it does not qualify the real voice-readiness probe. `cancel` checks both callback cancellation and real SIGTERM after an encoder trace, requires no completed result and reaps both children; the PM still independently verifies unit/cgroup/process cleanup. No performance or larger corpus run is authorized by a partial boundary pass.

The runner retains all stdout/stderr and build/fixture identities privately. Public corpus originals are decoded completely and must fit a single 25-second chunk; the harness refuses a larger fixture rather than truncating or padding speech. Silence is explicit three-second zero PCM between files. The reference corpus and private Hebrew count are untouched. Exact UTF-8 transcript bytes are decoded strictly; failed checks retain their evidence. Total suite wall time is not a five-minute recording-feasibility measurement.

Worker checks: both patch applications and baseline/patched C++ syntax pass. The observer check rejects deliberately stale text, a wrong encoder count, a missing callback and a missing decoder reset, then accepts the intact observation and Hebrew UTF-8. These are checks of the harness, not of the ASR engine. The existing synthetic cgroup-guard regression also passes. Instrumented binaries, actual API behavior and all later acceptance remain unknown until PM execution (§9, R19).

## One proposed turbo paired diagnostic

PM note 6563430854 reports every small-model boundary case passed, including recovery and multifile/fresh-second equivalence, with original limits/four threads, no mismatch/OOM and independently verified unit/process/cgroup cleanup. Multi took 368.390 seconds. The worker reviewed that TaskQ report; raw host files are outside its permitted filesystem scope and were not independently opened. `paired-performance.py` requires all ten original observations, their pass status, effective caps and matching instrumented-engine identities before any model process. It records each proof hash. This does not independently attest the PM's cleanup or qualify turbo behavior.

The one proposed candidate is the already verified multilingual **large-v3-turbo Q5_0**, SHA256 `394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2`, 574,041,195 bytes, MIT, from the retained ggerganov/whisper.cpp conversion of OpenAI's model. It previously got the private count right and had 8/30 human Hebrew word errors versus medium's 12/30; those remaining errors prevent acceptance. Its original long300 deadline failure remains. Choosing it for a diagnostic grants no product selection.

Run the following **once**, through the existing PM host isolation runner at the unchanged real caps/four threads. Uppercase arguments are private-path placeholders mapped by the PM. Use the retained original CLI and PM's uninstrumented experimental CLI with their exact hashes; the newly instrumented API binaries are not CLI replacements.

```sh
python3 experiments/speech344/paired-performance.py \
  --baseline VERIFIED_ORIGINAL_CLI --patched VERIFIED_EXPERIMENTAL_CLI \
  --model VERIFIED_TURBO_Q5_0 --output NEW_PRIVATE_PAIRED_DIRECTORY \
  --boundary-proof FAILDETECT_OBSERVATIONS --boundary-proof DEFAULT_OBSERVATIONS \
  --boundary-proof OPTOUT_OBSERVATIONS --boundary-proof OFFSET_OBSERVATIONS \
  --boundary-proof EXPLICIT_OBSERVATIONS --boundary-proof REFUSE_OBSERVATIONS \
  --boundary-proof CANCEL_OBSERVATIONS --boundary-proof CONTEXT_OBSERVATIONS \
  --boundary-proof RECOVERY_OBSERVATIONS --boundary-proof MULTI_OBSERVATIONS
```

One actual 600-second monotonic deadline covers decoding, both ten-second readiness calls, both unchanged private-Hebrew calls and both first-25-second stress calls. The source/canonical stress-segment/private-fixture hashes are pinned. The stress PCM is the exact original repeated human recording construction; it is budget data, not new quality evidence. Reference text never enters engine arguments. Readiness uses the earlier zero-PCM specialization (`--audio-ctx 50 --language en`); its hallucinated text is not a speech result. Normal inference remains automatic language detection. The full ru/en/he corpus, human Hebrew reference, five-minute cap and every previous failure remain unchanged.

The runner checks the retained three-blue-books semantic guard before spending time on stress data, then requires identical original-language transcripts/language and native counters proving exactly one encoder pass removed. It records stage timings, wall times, cumulative child RSS, logs, hashes and ratios; PM retains actual cgroup/OOM accounting and independently checks cleanup. Any process failure, ten-second readiness/600-second deadline failure, semantic/text/language mismatch, missing counter/reuse marker or absent measured saving stops the experiment without retries or further inference. A nonzero exit retains the failure evidence.

Even a successful diagnostic proves only these two paired inputs. One segment cannot prove twelve segments finish in 600 seconds, human all-language accuracy, installed readiness or live Telegram use. The necessary average for twelve chunks is under fifty seconds before overhead, but extrapolation is never a pass or a universal impossibility proof. If measured gain/quality is insufficient, stop this mechanism and report an explicit owner quality/latency tradeoff with the original limits and failures intact. Do not silently run a new model, full300, extra corpus or production activation. Any further qualification needs PM review of this terminal result (§9, R19).

## Prepared Sherpa small-int8 English diagnostic

Note6563991018 approves this separate candidate's asset/harness preparation only. The stopped first-window/Hebrew/CTC experiments above remain stopped. No ASR inference, product routing, English UI, production selection or installed change is authorized by this preparation.

Verified candidate assets are private in `target/pm344-sherpa-small-en-assets` (directory0700, files0600). `manifest.json` contains pinned origins, exact sizes/hashes, token Git identity, notices, upstream metadata, engine/fixture/harness/helper identities and diagnostic settings. `SHA256SUMS` covers every handed-off file including the manifest. Manifest SHA256 is `3de52d0b7791abb36e144962fcf79d01c9e7056dce93ee1567f90e9a483a8468`; harness SHA256 is `e9ada7ab8bf31f77f4915f1a8c95ce07f829d96edc7c71903b39524875df4fcf`. PM must retain and independently verify these exact bytes before worktree cleanup; namespace visibility is not assumed. The original Russian executable/model/configuration remain unchanged.

From this worktree, PM verifies the private handoff without running a model:

```sh
(cd target/pm344-sherpa-small-en-assets && sha256sum --check SHA256SUMS)
```

Only after reviewing the source, manifests and argv, PM can authorize **one** actual-host invocation through the existing isolated runner at MemoryMax1280M/High1024M/Swap256M/CPU150%/Tasks256. The worker cannot access that runner. The exact inner command, with `NEW_PRIVATE_OUTPUT_DIRECTORY` as a new private-path placeholder, is:

```sh
python3 experiments/speech344/probe-sherpa-english.py \
  --engine target/pm344-assets/sherpa-onnx-offline \
  --assets target/pm344-sherpa-small-en-assets \
  --output NEW_PRIVATE_OUTPUT_DIRECTORY
```

Run from the worktree root. No download/build/dependency install occurs in the harness. It verifies all model/token/license/provenance, engine and original fixture hashes, then refuses missing/wrong effective cgroup limits. A real one-second-silence decode and English-mode recognition must complete first within ten seconds including readback. Readiness, both full fixture decodes, English recognition and readback share one monotonic600s deadline. The source input remains capped at300s and uses no added source padding. These two short fixtures fit one25s chunk; this harness does not qualify long recordings.

Engine arguments explicitly select CPU, four threads, transcribe, `en` and300 engine tail frames. Fixed English is only the diagnostic mode, not language detection or approved routing. Reference text never enters engine argv. Silence text remains diagnostic evidence and is not delivered. Every model result must be fresh strict-UTF8 JSON with a text field. The human gate compares the **entire** original reference after case/punctuation normalization, including final Manta, and retains word-error counts plus complete reference/observed token sequences. Private output retains the entire reference/text/word-error evidence and requires the unaltered five-green-cups phrase with no contradictory number. No reference is changed to obtain a pass.

Every child is launched in its own process group. Timeout, error, SIGINT/SIGTERM and normal early parent exit kill the group and reap the direct child; the PM still independently checks unit/cgroup/process cleanup, OOM accounting and ordinary-service continuity. Failures retain private stdout/stderr and `observations.json`; stop without retry, fallback, another model or larger budget. Passing these clips/readiness proves neither installed English nor five-minute throughput, shared-media/live acceptance or broad English accuracy (§9, R19).

Local checks used **no ASR**: asset and token identities pass; the observer rejects truncated/final-phrase loss, foreign-script additions, wrong/contradictory numbers, wrong color and damaged text before accepting intact references. Tiny owned Python subprocesses prove timeout and descendant cleanup after normal parent exit. The existing synthetic cgroup-guard check passes. Actual readiness, English transcripts, resources and PM host cleanup remain unrun/unknown.

## Prepared original CPP small English diagnostic

Note6564218360 authorizes preparation only. No inference, model selection,
configuration switch or installation is authorized. Use the original retained
whisper.cpp1.8.2, never the experimental encoder-reuse binary. The exact private
handoff is `target/pm344-cpp-en-handoff/manifest.json`, SHA256
`aa8a47fa5e8875a5668c9bbc10f8a2c6964682710aaec1d99ce008cc46bfbf71`.
It pins script/helpers, original engine/model/notices and both unchanged fixtures.
The existing assets are in `target/pm344-assets`; no acquisition/build occurred.
PM must verify manifest and all bytes in its namespace before running, retaining
private synthetic audio and output privately.

After review, PM uses its existing host isolation runner with actual
MemoryMax1280M/MemoryHigh1024M/MemorySwapMax256M/CPU150%/Tasks256. The exact inner
argv, from the worktree root, is:

```sh
python3 experiments/speech344/probe-cpp-english.py \
  --engine target/pm344-assets/whisper-cli \
  --assets target/pm344-assets \
  --output target/pm344-cpp-en-v1
```

The output directory must not exist. This is an inner command, not permission to
run outside the actual host limits. Stop after one failure, without retry or
fallback. `guard.limits()` refuses absent/wrong effective limits before any child
runs. Full asset/fixture hashes and sizes are checked before dispatch. Capture
outer runner observations and independently verify unit/PID/cgroup cleanup.

Readiness generates exactly one second of zero PCM, decodes it and runs the real
model with `--audio-ctx 50` and fixed English. Its ten-second deadline includes
decode/readback. Fresh JSON, raw stdout/stderr and any silence-generated words
are retained as **nondelivered evidence**; an empty segment list is valid for
silence only. Human/private calls construct fresh argv without `--audio-ctx`, so
both use the engine's full default context. Every engine call uses CPU, threads4,
processors1, fixeden, beam1/bestof1/no-fallback and no expected prompt. The pinned
upstream CLI source confirms these flags. No audio padding or changed reference.

One600s deadline covers readiness and both clips including decode/readback;
recognition failure, malformed strict-UTF8 JSON, full-human mismatch (including
Manta), numeric/color mismatch, timeout or cap failure stops immediately. Shared
process-group fencing kills/reaps descendants on normal exit, error, timeout or
SIGTERM. Each result has a fresh output prefix. Readback retains complete text,
word-error/count evidence, argv and cumulative child RSS; actual cgroup peak/OOM
and cleanup remain outer-runner evidence. Short results do not qualify300s/25s
chunking, installed/live behavior or product selection. Russian stays unchanged.

Preparation checks, without ASR:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 experiments/speech344/check-cpp-english.py
PYTHONDONTWRITEBYTECODE=1 python3 experiments/speech344/check-sherpa-english.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_probe_speech_344.py
```

## Prepared contiguous300 CPP English stress and controls

Note6564345386 authorizes preparation only; no inference. This replaces the
unexecuted padded25s-block proposal. Private handoff:
`target/pm344-cpp-en-long-handoff/manifest.json`, SHA256
`d413a35cd1e77eac8b110a8ab83fb34b0569138b8ae1620bae3af50b33ec0a95`.
It pins original engine/model/MIT notices, human fixture and scripts/helpers.
PM must independently match all identities in its namespace before dispatch.
No new assets/build, installed writes, routing or model selection.

Exact inner argv, from the worktree root, **only after PM review** through its
existing host runner with actual1280M/1024M/256M/CPU150%/Tasks256:

```sh
python3 experiments/speech344/probe-cpp-english-long.py \
  --engine target/pm344-assets/whisper-cli \
  --assets target/pm344-assets \
  --output target/pm344-cpp-en-long-v1
```

Output must be new. Effective-cap/engine/model/license/source guards precede
child dispatch. Readiness, controls, stress construction/decode, all segments and
readback share one600s deadline. Readiness and standalone zero silence each also
have10s inclusive decode/readback. Only generated verified zero PCM uses context50;
its JSON/text stays nondelivered. Ordinary speech uses full context, CPU/threads4,
fixeden/processors1/beam1/bestof1/no-fallback/no prompt. Corrupt ffmpeg refusal and
301s decoded-duration refusal must precede ASR. Explicit cancellation kills/reaps
one ordinary full-context engine process after0.1s, records PID/group and no
readback/delivery; it proves early cancellation only, not decoder-phase or
callback cancellation. A process that exits before cancellation fails the control.

The stress is unchanged humanEnglish PCM repeated **contiguously** to exactly
4,800,000 samples, truncating only the last source copy. No silence padding,
stretching or reference-aware segment placement. Construction/decoded/full and
segment hashes, source offset/sample count and12 true25s windows are retained.
Each engine result has a unique prefix and strict-UTF8 readback. Any empty speech,
process/format/cap/deadline failure stops with partial evidence and no retries.

The complete ordered text is compared against all complete source-reference
repetitions plus every possible word prefix for the final partial source copy.
All error counts remain observations. There is no timed word alignment for the
cut final phrase: the harness explicitly reports fidelity **unknown**, never
picks the lowest error prefix as quality PASS. With no partial tail an exact
reference is available and a mismatch fails. Segment boundaries can also split
words; raw ordered transcripts and true offsets remain available for PM fidelity
review. Successful processing is budget-stress completion only. It does not
qualify full-quality300s, shared media, installed/live or product behavior.

The shared helper fences/reaps process groups on exit/error/timeout/SIGTERM.
Generated WAV/segments/control files live only under this run's `work` directory,
removed in `finally` with readback; private JSON/stdout/stderr remain evidence.
PM must verify unit inactive/MainPID0, wrapper/group/cgroup gone, actual peak/OOM
and unchanged ordinary service identities/restarts. Outer termination or missing
cleanup evidence remains a gap. The harness never claims independent host cleanup.

No-ASR preparation checks:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 experiments/speech344/check-cpp-english-long.py
PYTHONDONTWRITEBYTECODE=1 python3 experiments/speech344/check-cpp-english.py
PYTHONDONTWRITEBYTECODE=1 python3 experiments/speech344/check-sherpa-english.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_probe_speech_344.py
```
