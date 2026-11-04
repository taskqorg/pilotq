# PM-owned no-model build episode (#344)

This is the exact preparation requested by note6625480080. The worker fetched and inspected dependency archives, prepared commands and checked guards. No compiler, linker, harness, ONNX session or model was executed. PM must review this handoff and dispatch the episode once through its existing isolated runner. This does not select or activate an ASR engine (§9, R19).

## Pinned inputs and provenance

Full origins, sizes, archive and license hashes are in [giga344-build-handoff.json](giga344-build-handoff.json). All four actual retained archives and six license/notice files were read and verified before handoff; independent read-only review matched them too.

| Dependency | Origin and identity | License and limitation |
|---|---|---|
| Sherpa v2.0.1 | Official k2-fsa archive at commit `af2d8d81dbfa61c4cea4c568da78f3982edca005`; tag readback matches. 11932226 bytes; SHA256 `afcafe3c34bc2f15a363ad791fa1644a69bf0977a58c1b4f617468976c2faeb5` | Apache-2.0; archive has LICENSE, no root NOTICE. Select only regular csrc and exact license/dependency-pin texts. |
| ORT 1.28.2 Linux x64 glibc2_17 shared | csukuangfj/onnxruntime-libs archive pinned by this exact Sherpa source's `cmake/onnxruntime-linux-x86_64.cmake`. 8781463 bytes; SHA256 `c4f8994d56191d9d2c92a961b39fe790459f2c5d155f912b239506ea31359534` | MIT plus bundled ThirdPartyNotices. This is Sherpa's maintainer-distributed prebuilt dependency, not independent Microsoft binary attestation. |
| KNF 1.22.4 | Official csukuangfj archive pinned by Sherpa's CMake. 72131 bytes; SHA256 `257fea639585ca6ee00860493bb9d7ab32488b8a5407ffe434c1f361ea545fae` | Apache-2.0; exact previously observed feature dependency, not a repeated reference episode. |
| KissFFT | Official mborgerding archive at `febd4caeed32e33ad8b2e0bb5ea77542c40f18ec`. 74252 bytes; SHA256 `497103e664168ebe39580b757adbe616f6cf85a16572af581ca7bc42d0ab13fd` | COPYING and BSD-3-Clause text retained; pinned KNF dependency. |

The archives are retained privately as `sherpa.tar.gz`, `ort.zip`, `knf.tar.gz`, `kissfft.zip` in `target/pm344-build-closure` (directory0700/files0600). PM verifies those actual bytes in its checkout before dispatch; worker namespace visibility is not assumed. Do not replace a missing artifact or acquire an alternative during execution. No model/checkpoint/token files are inputs to this episode. Existing Russian model, recognizer, configuration and all settings remain untouched.

The ORT library SHA256 is `4b3607aebd1784b26b6f9b20e4bd974c7ab8287043e4d095cb7d2cb40b5e566e`. Static `readelf -d` inspection—not `ldd`, loading or execution—reports SONAME `libonnxruntime.so`, RPATH `$ORIGIN`, and platform dependencies `libdl.so.2`, `librt.so.1`, `libpthread.so.0`, `libstdc++.so.6`, `libm.so.6`, `libgcc_s.so.1`, `libc.so.6`, `ld-linux-x86-64.so.2`. The qualified host must already provide these standard libraries; a missing dependency stops, without installation. This shared-library harness dependency does not replace the installed static Sherpa executable or prove its future build/runtime equivalence.

## Exact commands and build boundary

[The wrapper](giga344-build-episode.py) owns the actual argument arrays. [The reviewable command expansion](giga344-build-commands.json) uses the literal placeholders `$REPO` for the task worktree and `$OUT` for the fresh output directory; it is review data, not a shell script. The wrapper resolves those paths directly, without shell expansion.

It sequentially compiles two KissFFT C objects, the patched real NeMo `Forward` translation unit as an unlinked object, one synthetic ORT tensor validator and three feature executables. The KNF source list matches its retained CMake core list. Feature binaries use the real Sherpa `features.cc`/`resample.cc`; the experimental define is present only for candidate/tensor/Forward builds. Disabled-patched and pristine-upstream binaries keep GigaAM's original25ms/nonsnipped frontend settings. Candidate uses the explicit20ms/snipped configuration. Each source uses the same fixed four-thread environment.

No CMake, downloaded script or full recognizer/decoder build is executed. All compiler processes run from the new episode output root, outside the extracted source directories. `-ffunction-sections -fdata-sections -Wl,--gc-sections` removes uncalled command-registration code from the feature-only link; it avoids adding ParseOptions/text/utf8proc just for unused registration. These options do not remove candidate tensor checks or the executed frontend. Compile/link compatibility is unknown until PM runs it; a missing symbol/header is a failed episode, never permission to add dependencies or retry.

Compiler, Git and readelf paths/full byte hashes are pinned in the manifest. The wrapper records actual resolved paths/version output, generated binary hashes and ELF readback. It never changes global PATH, system packages or host configuration. Model execution is not part of this authorization: the NeMo object is never linked into an executable; the tensor harness creates no session; feature programs read PCM only.

## Exact PM inner argv

From the task worktree root, after verifying the exact script and manifest below:

```sh
target/pm344-giga-static-tools-v2/venv/bin/python -I \
  experiments/speech344/giga344-build-episode.py \
  --archives target/pm344-build-closure \
  --reference target/pm344-frontend-reference-v1 \
  --knf-observations target/pm344-knf-frontend-v1 \
  --output target/pm344-giga-build-v1
```

`target/pm344-giga-build-v1` must not exist. This is the exact inner argv, not permission to run it outside the PM-owned isolation runner. PM owns the existing runner's outer invocation; its flags have not been copied or invented by the worker. Require actual MemoryMax1280M, MemoryHigh1024M, MemorySwapMax256M, CPU150%, Tasks256 and **one outer600s deadline**, including runner startup, verification, extraction, all builds, calls and readback. The inner alarm starts once before checks; outer startup/import time remains inside the original outer deadline. Archives were acquired during authorized preparation; execution does no network acquisition and makes no claim about download-plus-build throughput.

Script SHA256: `9847d4e26cf096637ddcf4f46d35902e836a69f588243bc5c457e3fa01afdba9`.

Manifest SHA256: `0b59975826d5ba11cf8ce8e76afc2238a3b1edc4a775275341e8503f2c56a2f9`.

The script verifies all nine runtime file hashes, exact staged archive hashes/sizes, tool identities, current cgroup effective caps and retained NumPy2.4.2. It uses the existing process-group helper, kills/reaps descendants on every exit/exception/deadline, records PID/PGID/exit/reaping/group readback, and refuses a group still observed. There are no retries/fallbacks, timeout resets or resource increases. Partial output is retained as evidence; original reference/assets are never cleaned or overwritten.

## Observations and stop conditions

The synthetic harness calls the same candidate validation seam used by real Forward: source-derived valid lengths, changed graph-returned lengths, tensor types/shapes/batches/vocabulary, finite values and energy-clamp guards. This exercises validation only, without a graph/session. The feature phase uses all55 original ordered PCM16 cases, checks each byte hash before scaling, requires exact frame/byte shapes and finite returned arrays, and checks repeated GetFrames/fresh split-stream equality. The completed reference is never imported or rerun.

Retained reference/KNF observation JSONs and every compared array are identity-checked. Raw candidate arrays, raw disabled/upstream arrays, maxabs differences and coordinates against both retained log32 arrays are saved. Disabled-patched arrays must equal pristine-upstream arrays exactly; this is frontend non-regression evidence only, not a whole Russian engine or quality verdict. No numerical tolerance or ASR PASS is introduced. The current harness does not execute a decoder, callback, cancellation during model inference, official Torchaudio or any checkpoint.

Any identity/license/link/path/shape/finite/build/process-group/deadline failure stops immediately; no unchanged retry, alternative dependency, new model or installed change follows. `observations.json` records incomplete/failure or completed observations, never speech acceptance. PM independently checks unit inactive/MainPID0, empty/gone cgroup, wrapper and all descendants gone, plus unchanged ordinary service PIDs/restarts. Build success, if obtained, still leaves official-export equivalence, actual automatic RU/EN/mixed/Russian parity, readiness/full300, remaining output-language paths and release/live gates open (§9, R19).

## Preparation verification

The existing whole-archive source extractor refused the actual Sherpa archive's unrelated platform symlinks before extraction. The source-specific selector then retained722 regular C++/license/pin files without following those links. Two synthetic guard tests pass, preserving selected-link/path/full-digest/license/deadline refusals before source writes; the original six KNF handoff guards still pass. Independent review verified all nine runtime file identities, four archive hashes/sizes and six license/notice digests, and found no actionable source defect. These are preparation checks only. No compile, link, feature call, library load, graph inference, installation or service mutation ran here.
