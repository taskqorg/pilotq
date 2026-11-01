# First-window reuse experiment

This is an unbuilt, experimental patch for whisper.cpp 1.8.2, not a PilotQ engine selection. It addresses duplicate encoding after automatic language detection. It does not repair small's known Hebrew count error. No savings or correctness result is claimed.

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

Record CMake/compiler versions, complete configure/build logs, effective cache flags, source/patch/license/output hashes and linked dependencies. These commands are a proposed recipe, not a reproduced build: CMake is absent from the worker PATH. Do not provision it or infer a binary identity from the patch. The original binary remains SHA256 `30ab87ffca69ba2e03f6c9c110c158cc5d7e4099eaa0179793559215b99aed70`.

## Mechanism and boundaries

An explicit new parameter defaults false. Only the experimental pinned CLI opts in, with one processor and no VAD. Its existing callbacks only read their own abort flags; callers with callbacks that mutate encoder state must never opt in. Production PilotQ does not activate this field or binary.

A function-local token becomes eligible only after successful automatic detection over newly computed mel data, at offset zero, full/default context both before detection and at transcription, CPU execution and no prompt continuity. The token is consumed at the first transcription window, even on mismatch. Explicit language, changed context, absent audio, failed detection and new invocations cannot obtain a token. Later seeks run the original encoder. Encoder-begin callbacks still run before the decision; abort is checked before using cached features. Detection failures still return the original error. Cancellation returns failure without decoding cached output. The original decoder reset remains in place; no language, temperature, fallback, prompt or text-reference changes are made.

## Required proof before any larger run

PM execution must use the existing host isolation runner, actual MemoryMax1280M/High1024M/Swap256M/CPU150%/Tasks256, four recognizer threads and original deadlines. The worker cannot access that runner. No model inference was run for this patch.

First build and run an instrumented API boundary probe against both original and patched source. Record encoder counts and exact transcripts, not just exit codes. Cover default offset/context reuse, nonzero offset, reduced context followed by default context in the same state, explicit language, failed detection, empty audio, encoder-begin refusal and abort-before-reuse. Check decoder reset and a two-file batch versus separate invocations, with differing languages and silence between recordings. The original encoder path must handle every ineligible case; failures/cancellation must never emit cached transcription. Arbitrary mutating callbacks remain unsupported for opt-in. Stop on a mismatch; do not proceed to performance measurement.

Only after those boundaries pass, run one bounded paired correctness/performance proof using the unchanged baseline and new binary against identical unhinted audio/settings. Reuse `scripts/profile-speech-344.py`'s effective-cgroup guard and recording construction; its hard-coded baseline hash intentionally rejects this new binary, so it is not yet a patched-engine harness. Keep human ru/en/he, private Hebrew count, silence/readiness and long references unchanged. Record binary/model hashes, texts/languages, native encoder counts/times, wall time/RSS, actual isolation and cleanup. Readiness stays ten seconds; each recording stays at most five minutes with 25-second chunks and an actual 600-second total. Do not run a broader corpus before the boundary proof or retry unchanged failures. Insufficient benefit ends this experiment and returns the concrete quality/latency tradeoff to the owner.

Build, API boundary harness, paired performance harness, model proof and independent review remain open. The source-only apply check cannot qualify any of them (§9, R19).
