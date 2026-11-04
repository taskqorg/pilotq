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

## Prepared GigaAM community export static inspection

Note6564856780 approves only the pinned fussraider220M acquisition and preparation
of private static tooling for PM review. Assets in
`target/pm344-giga-multilingual-assets` are verified in full: model224,762,524bytes,
SHA256 `2d94f93ffd4ef58e7899c9de885c25bbbc8c9f1073618868d118a674450ba5f7`.
The private manifest pins revision `9f5a77e8975211abe8511693accd3a63ee1e9f43`
and tokens/config/exporter/MIT/README sizes, hashes and immutable source URLs.
Directory0700/files0600; no model load, exporter execution or installed write.

Private handoff `target/pm344-giga-static-handoff/manifest.json` SHA256
`16680b31cd2418a736c1a96da6c832023fa9f5b7fb664e59c98fb9f248478251`
pins the inspector, standard cap helper, asset manifest and proposed wheel locks.
PM rehashes all artifacts in its namespace before using them. Do not substitute
another derivative/revision or acquire fp32/600M/i2z1.

`onnx-inspector-wheels.json` identifies exact official PyPI filenames, URLs,
sizes, expected full SHA256 and dependency metadata; these wheels have **not**
been acquired/installed/import-tested. The requirements file pins ONNX1.20.1,
NumPy2.4.2, protobuf6.33.5, ml_dtypes0.5.4 and typing_extensions4.15.0 with hashes.
Proposed platform: standard-GIL CPython3.14 x86_64/glibc>=2.27. ONNX uses its
cp312-abi3 stable-ABI wheel (not a cp313t free-threaded wheel); protobuf cp39-abi3,
NumPy/ml_dtypes cp314-cp314. Published tags and dependency bounds fit inspected
Python3.14.4/glibc2.43; actual import compatibility remains untested.
No Torch, Transformers, ONNXRuntime, ReferenceEvaluator or reference extra.

After PM review only, proposed private setup commands from the worktree root:

```sh
python3 -m venv target/pm344-onnx-inspector
target/pm344-onnx-inspector/bin/python -m pip --isolated --disable-pip-version-check download \
  --index-url https://pypi.org/simple --only-binary=:all: --require-hashes --no-deps \
  --dest target/pm344-onnx-wheels -r experiments/speech344/onnx-inspector-requirements.txt
target/pm344-onnx-inspector/bin/python -m pip --isolated --disable-pip-version-check install \
  --no-index --find-links target/pm344-onnx-wheels --only-binary=:all: --require-hashes --no-deps \
  -r experiments/speech344/onnx-inspector-requirements.txt
```

These are a proposal, not an instruction to install globally or run now. Use new
private0700 directories/umask077 and verify downloaded filenames/full hashes
against the wheel manifest before installing. Missing venv/ensurepip, wrong
platform/tag, dependency/import failure or byte mismatch stops, without tool
upgrades, source builds, PATH changes or alternative packages. Preserve bundled
pip/version evidence; do not upgrade it. PM's existing isolation runner must
cover approved setup and inspection with one hard600s episode and actual
1280M/1024M/256M/CPU150%/Tasks256. A later inner clock cannot extend that outer
deadline. Do not run setup outside those limits to obtain a passing result.

Exact inner inspection argv, under that same episode after approved setup:

```sh
target/pm344-onnx-inspector/bin/python experiments/speech344/inspect-giga-multilingual.py \
  --assets target/pm344-giga-multilingual-assets \
  --output target/pm344-giga-static-v1
```

Output must be new. The inspector checks actual caps, exact package versions and
all asset sizes/full hashes before parser load. It limits numerical-library
thread environment to4 within its own process. Standard `onnx.load_model` uses
`load_external_data=False`; recursive standard protobuf reflection refuses any
external tensor location/data, including nested attributes/graphs/sparse tensors,
functions and training graphs, before `onnx.checker.check_model(full_check=False)`.
A prepared synthetic external-data guard runs first under the same deadline;
its runtime result is **not yet known**. AST/CLI preparation checks pass only.

No graph operator executes. The report retains input/output dtype/shape/dynamic
axes, actual metadata/opsets/operator counts, convolution/normalization nodes,
output producer chains and bounded symbolic integer expressions for length
outputs. Unsupported/capped expressions remain explicitly unknown; no guessed
floor(input/4) or shape-inference result is declared equivalent to actual valid
lengths. Tensor payloads are not printed; only small integer structural constants
are read via standard `numpy_helper`. Official-checkpoint equivalence and
numerical frontend parity remain gaps. Do not declare a patch necessary or a
compatible engine until actual graph findings are reviewed.

The inspector spawns no subprocess, creates no converted model or temporary
weight files, and leaves only its private output JSON. PM records package/import
outcomes, runtime/peak/OOM and independently verifies unit inactive/MainPID0,
wrapper/cgroup gone and unchanged ordinary services; it removes its private venv
and wheel staging after preserving approved provenance/evidence as appropriate.
Never remove retained ASR assets or previous failures. External-data refusal,
parse/check/schema/import failure, resource/deadline failure and missing host
cleanup remain failures/gaps, not model-quality results. No inference or
production activation follows automatically (R19).

## Revised no-ensurepip combined static setup (v2)

Note6564967418 supersedes the unexecuted setup commands above. PM's actual v1
attempt failed `python3 -m venv` because ensurepip was absent:0.488s/exit1,
no OOM/high0, before wheel acquisition/import/model parse. Unit997ae32c4ccf
inactive/PID0, wrapper428931 and cgroup gone. Preserve that partial private venv
as failure evidence; do not repair/reuse/delete it. This is tooling failure,
not a verdict on candidate compatibility or speech quality.

Prepared v2 uses only standard `venv --without-pip`, zipimport/runpy and the
pinned official PyPI pip25.3 wheel. Bootstrap metadata is committed in
`onnx-inspector-bootstrap.json`: `pip-25.3-py3-none-any.whl`,1,778,622bytes,
SHA256 `9655943313a94722b7774661c21049070f6bbb0a1516bf02f7c8d5d9201514cd`,
Python>=3.9, no external dependencies (pip bundles its vendored code). The five
inspector packages/wheel versions/digests and complete non-extra dependency
closure stay unchanged. No ensurepip/get-pip, apt, global/system pip installation,
upgrade, PATH change, source build or extra runtime. Pip executes directly from
its verified wheel in the new private interpreter; pip itself is not installed
or used to upgrade any preexisting tool.

Private prepared handoff `target/pm344-giga-static-handoff-v2/manifest.json`
SHA256 `589b33d8fc035ccbe4c749f6082c61ca4ba4ff1110005c1754f3d033f4ff3cad`
pins wrapper/inspector/bootstrap/locks/helpers and retained asset manifest. The
old v1 handoff remains history and does not identify this revised inspector.
Verify v2 hashes before execution. No tooling wheels have been acquired and
no v2 setup/import/checker/model parse has run in the worker.

Exact **combined** PM inner argv from the worktree root, after review and only
inside the existing host isolation runner's original actual caps/hard600s:

```sh
python3 experiments/speech344/setup-giga-static.py \
  --root target/pm344-giga-static-tools-v2 \
  --assets target/pm344-giga-multilingual-assets
```

The v2 root must not exist. The wrapper requires standard-GIL CPython3.14 Linux
x86_64, validates effective1280M/1024M/256M/CPU150%/Tasks256 and pinned tooling
locks/candidate identities before setup. It acquires **only** six listed wheels
from exact official files.pythonhosted.org URLs, refuses redirects/oversize/hash
mismatch and never executes a `.part` file. It creates0700 directories/files0600
under its new root. The argument arrays use this sequence:

1. `python -I -m venv --without-pip ROOT/venv`.
2. `ROOT/venv/bin/python -I -c` with standard `runpy.run_module('pip')` and the
   verified pip wheel inserted in that subprocess's `sys.path`; record version.
3. That zip-loaded pip installs only the five locked wheels offline using
   `--isolated --disable-pip-version-check --no-cache-dir install --no-index
   --find-links ROOT/wheels --only-binary=:all: --require-hashes --no-deps -r LOCK`.
4. The same pip runs `check`; the inspector additionally verifies exact installed
   distribution versions before importing ONNX. Missing/wrong closure refuses.
5. The private interpreter runs `inspect-giga-multilingual.py --assets ASSETS
   --output ROOT/inspection --deadline-monotonic DEADLINE` with the **same
   absolute monotonic deadline** started before asset verification/wheel/setup
   work. The inspector takes the earlier deadline, never a fresh600s extension.

ROOT/LOCK/ASSETS/DEADLINE in this numbered explanation are placeholders resolved
by the combined script, not additional shell commands to run manually. PM's
outer hard600s remains authoritative during blocking network/parser calls.
Every child uses the shared tested process-group fencing/reaping helper and
remaining budget; no retries/fallback. Failures retain setup-result.json, raw
logs, partial downloads/venv and original v1 evidence. The script never installs
outside its new root or deletes generic temp directories. PM independently
verifies unit inactive/MainPID0, wrapper/descendants/cgroup gone and unchanged
ordinary service PIDs/restarts; retained private files are not live activation.

Readback evidence is `ROOT/setup-result.json`, child stdout/stderr and, only if
static inspection reaches completion, `ROOT/inspection/inspection.json`.
AST/help and existing tiny owned-process cleanup checks pass; v2 download,
bootstrap, import, external-data guard, static checker and cap measurements
remain **not run** pending PM review. No model inference/export/build or product
activation is included. Record the eventual failure/pass without erasing v1.

### Targeted static supplement after actual v2 parse

PM note6565033244 reports v2 setup/static parse PASS21.537s/noOOM under
original caps; `memory.high`357 is an event count, not megabytes. The retained
v2 reports match `giga-static-supplement-manifest.json`; neither is rewritten.
The old inspector omitted FLOAT/DOUBLE constants and filtered `Conv` rather
than actual `ConvInteger`. New `supplement-giga-static.py` fills only those
static evidence gaps: complete capped `encoded_lengths` ancestry, finite
scalar/small numeric constants (at most16 elements), actual pre_encode
ConvInteger structural attributes/weight dimensions, functions/training counts.
It never evaluates operators, performs shape inference, repeats setup/schema
checking, or loads external data. Missing/nonfinite/oversize length constants,
truncated ancestry or missing subsampling nodes refuse. Unresolved weight
producers are reported explicitly, never guessed. No inference/build/install.

Script SHA256:
`a455804450c1ce2d3a34a725fd90fe9f28b46ee12ee2384fce0025ce32975e15`.
The committed manifest pins script/base/guard and both unchanged v2 reports.
PM first verifies every `files` entry using standard `hashlib.file_digest`;
any mismatch stops before the following exact inner argv. From worktree root,
inside PM's existing isolated runner with **hard600s including verification**,
actual MemoryMax1280M/High1024M/Swap256M/CPU150%/Tasks256:

```sh
target/pm344-giga-static-tools-v2/venv/bin/python -I \
  experiments/speech344/supplement-giga-static.py \
  --assets target/pm344-giga-multilingual-assets \
  --output target/pm344-giga-static-supplement-v1
```

Exact standard-library identity preflight (inside that same outer600s, before
the inner argv above; no ONNX import):

```sh
python3 -I - <<'PYVERIFY'
import hashlib, json
from pathlib import Path
manifest = json.loads(Path('experiments/speech344/giga-static-supplement-manifest.json').read_text())
for name, expected in manifest['files'].items():
    with Path(name).open('rb') as stream:
        actual = hashlib.file_digest(stream, 'sha256').hexdigest()
    if actual != expected:
        raise SystemExit('identity mismatch: ' + name)
print('All supplement and retained-v2 identities match')
PYVERIFY
```

Do not dispatch the inner argv if this preflight fails. PM verifies the
committed manifest against the reviewed source revision before trusting it.

Output must not exist. This reuses exact private versions and verifies all six
candidate identities before ONNX import/parse. Effective-cap guard runs first;
process-local math threads remain4. An in-process600s alarm bounds parsing;
PM's earlier outer deadline remains authoritative, with no reset/retry. No
subprocess is created, so termination has no child process group to leak.
PM still independently checks wrapper/unit/MainPID/cgroup/descendants and
ordinary service identities. Preserve `supplement.json`, raw stderr and
incomplete failure evidence. AST/help pass only; supplement runtime, rejection
paths and actual graph facts are **not run**, pending required PM review.
Numerical formula derivation waits for returned actual constants; source
predictions below are not substituted for that readback (§9/R19).

### Supplement result and next reference-only boundary

PM note6565196828 reports actual supplement PASS1.775s/noOOM/high0; retained
JSON SHA256 `3e34742ea05fddb049997dc8d794692b739d2e9d1c7a502638950f84c27c92c1`.
Do not repeat the supplement or setup. The [integer derivation](giga-length-derivation.json)
uses its actual-1/2/1 scalars and two stride2/kernel5/pad2 ConvInteger stages.
For1–12,99–101 and actual25s2499 feature frames, encoded length is
`(L+3)//4`, including625 rather than Sherpa's624 at2499. No model operators
were executed. This closes the omitted static-constant/ConvInteger evidence,
not frontend compatibility or transcription qualification (§9/R19).

The [current report](../../docs/research/344.md#smallest-numerical-frontend-reference-plan-after-supplement)
defines the next proposed NumPy-only reference episode, fixtures, exact
framing/mel/log conventions and limits. No runnable numerical harness or
approval to build an engine/accessor is implied. Retained tools cannot emit
actual KNF features or official Torch arrays: reference cross-checks must not
be reported as engine parity. No new model/runtime/tools are needed for the
proposed reference-only preparation; actual engine parity remains a separate
reviewed boundary. Installed Russian assets/settings remain unchanged.

### Prepared NumPy reference-only PM handoff

Note6565322396 authorizes preparation; **do not execute numerics before PM
review**. `reference-giga-frontend.py` SHA256 is
`4949dd17b8508be45d214e6b836cfc0a626544e58c899b78c57fe109700e265c`.
`frontend-reference-handoff.json` pins script/source-manifest/cap-helper hashes;
`frontend-reference-sources.json` pins eight exact upstream source URLs, lengths
and full hashes. These texts were retained privately under
`target/pm344-frontend-reference-sources-v1`,0700 directory/0600 files. They
are hash-checked data, never imported/executed. No weights/model file is opened.

Before dispatch, PM verifies each handoff `files` entry with standard hashlib
against the reviewed commit. The exact inner argv from the worktree root,
inside the existing PM isolation runner with original effective caps and one
**hard600s including identity preflight/import/computation/readback**, is:

```sh
target/pm344-giga-static-tools-v2/venv/bin/python -I \
  experiments/speech344/reference-giga-frontend.py \
  --sources target/pm344-frontend-reference-sources-v1 \
  --output target/pm344-frontend-reference-v1
```

Output must not exist. The effective1280M/1024M/256M/CPU150%/Tasks256 guard and
all source identities are checked before NumPy2.4.2 import. Only the existing
private interpreter/NumPy are used; four process-local math threads, no PATH or
global changes, no dependencies/setup/build/network/subprocess/ONNX import.
An internal600s signal alarm complements the earlier outer deadline without
extending it. No child process groups exist; PM independently confirms unit,
wrapper/descendant/cgroup cleanup and unchanged ordinary services.

The script generates deterministic PCM16 zero/first-middle-last impulses,
bounded ramp and1kHz tone at sample counts0/319/320/321/479/480/481/640/1600,
plus one400000-sample tone. It records exact source-sample construction/hash,
complete snipped320/160 framing, periodic Hann, unnormalized rFFT power,
161x64 HTK-Hz triangles and log clamp[1e-9,1e9]. Shorter-than-window inputs
produce zero complete reference frames and never enter a model. A scalar
math.fsum direct DFT and scalar triangle sum independently cross-check the
first frame of tiny cases. Explicit float32-stage versus float64 observations
are **not** emulation of Torch/KNF backend arithmetic. A source-derived default
KNF mel-space weight matrix is retained to expose the different convention.

Finite arrays, exact frame/shape identities and source guards fail closed.
Every numeric max-absolute/relative difference and coordinate is recorded,
without invented PASS epsilon. `.npy` arrays, SHA256s, `reference.json`, elapsed
time and peak self RSS remain private. An empty comparison has zero absolute
difference by definition; it is not healthy ASR evidence. Incomplete failures
remain in JSON/raw runner logs. AST/help/source hashes pass; numerical paths,
scalar DFT, runtime caps/time/RSS and cleanup remain **not run** in the worker.

#### Smallest real feature-parity accessor proposal (not built)

Use one disposable C++ program linked only to pinned KNFv1.22.4, with no Sherpa
recognizer, ONNX runtime or model. Reuse PM's retained matching static library,
headers and ordinary compiler if available. The worker found `/usr/bin/c++`,
no cmake on PATH and no KNF tree in the checked project's target depth4; this
is not evidence about PM's private retained build tree. No library/compiler
receipt, exact link argv or ready accessor binary is claimed. If matching
headers/library are absent, report that gap and obtain review of the pinned
source/build dependency closure before any acquisition/build; do not assume
Whisper build artifacts include KNF.

The concrete feature-only interface needs no new framework:

1. Read only the generated PCM16LE fixture and divide by32768 into float samples.
   Construct `knf::FbankOptions` with16kHz, frame_length_ms20, frame_shift_ms10,
   snip_edges=true, window_type="hann", dither0, remove_dc_offset=false,
   preemph_coeff0, round_to_power_of_two=false;64bins, low0/high8000,
   is_librosa=true, use_slaney_mel_scale=false, norm="", floor_to_int_bin=false;
   use_energy=false, use_power=true, use_log_fbank=false.
2. Use `knf::NumFrames(...,flush=true)`, `knf::FeatureWindowFunction`,
   `knf::ExtractWindow` and `knf::FbankComputer::Compute` for each exact frame.
   Dump raw64-bin mel energies, periodic window and mel matrix using the
   existing `GetWindow`/`MelBanks::GetMatrix` APIs. Shape/count/finite validation
   and explicit serialization are the whole accessor; no graph/audio decoder,
   daemon, automatic routing, model fallback or native-agent changes.
3. Compare actual dumped arrays with the SAME retained PCM/reference stage
   arrays, then apply the specified clamp/log for both. This measures the real
   KNF feature boundary; it still does not prove official Torch backend parity,
   checkpoint equivalence or speech quality. Document precision/accumulation
   order and actual differences before proposing an acceptance error bound.
   No tolerance is selected merely to turn mismatches green.

Any later build and paired feature run need their own reviewed source/link
identities, license/dependency closure, original effective caps/shared600s,
process-group cleanup and retained observations. This proposal authorizes
neither a build nor a production frontend patch (§9/R19).


## Actual reference readback and KNF preparation (current continuation)

PM note6565687464 reports the exact reference-only run completed55cases in1.763s/exit0/noOOM/high0 under originalcaps; peakself83104KiB. Worker read the retained JSON/arrays, checked hashes/shapes/finite values, and did not rerun the reference. Numeric observations and limits are in [the report](../../docs/research/344.md#current-continuation-output-coverage-catalog-correction-and-actual-reference-readback). No numerical tolerance, KNF/Torchaudio parity or ASR qualification is claimed.

PM note6565715149 requests [the pinned KNF feature-only accessor source/build-closure review](knf-accessor-plan.md). It contains source/hash, archive/dependency/license identities and exact proposed acquisition/compiler/per-case argv. No acquisition/build/run is authorized by this preparation. Same-PCM guarded orchestration and actual engine parity remain separate; no production change.


## Combined KNF feature-only PM episode preparation

Note6566515450 authorizes the [guarded combined handoff](knf-accessor-plan.md#combined-guarded-pm-handoff-note6566515450), not worker acquisition/build/execution. That section gives exact script/manifest hashes and inner PM argv, two full archive digests/license chain, original actual caps/shared600s, same55 PCM hashes, comparison/readback and process-group/independent cleanup boundaries. The new `knf-frontend-episode.py` is unexecuted; local synthetic guard/AST/help checks prove only preparation guards. No numerical tolerance or ASR/parity/activation claim.


## Prepared GigaAM no-model build handoff

[The build-only plan](giga344-build-plan.md) is the current exact dependency/command/PM-wrapper recipe under note6625480080. It supersedes missing-recipe status only; no compiler, linker, feature or model execution has occurred. Keep all original Russian assets/settings and prior failed evidence. PM reviews the final identities before one original-caps/shared600s episode (§9, R19).

## Supported offline CLI preparation (note6626590400)

[The exact CLI build/short-quality handoff](giga344-cli-plan.md) pins the full offline closure and original-caps/shared600s PM argv. Preparation only: no configure/build/model episode, selection or installed change. It supersedes missing supported-build recipe status; earlier numerical differences remain unqualified.
