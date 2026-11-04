# KNF 1.22.4 feature-only proof preparation

Preparation only (PM note 6565715149). No archive, build dependency or binary was acquired, built or executed. No model/runtime is involved. R12 reuse and §9/R19 separate this experiment from activation.

## Exact dependency closure

- KNF v1.22.4 archive: `https://github.com/csukuangfj/kaldi-native-fbank/archive/refs/tags/v1.22.4.tar.gz`, SHA256 `257fea639585ca6ee00860493bb9d7ab32488b8a5407ffe434c1f361ea545fae` (Sherpa's pinned receipt). Apache-2.0; read-only upstream LICENSE SHA256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`; verify/retain LICENSE and any NOTICE from the verified archive before compilation.
- KNF's `cmake/kissfft.cmake` pins only KissFFT commit `febd4caeed32e33ad8b2e0bb5ea77542c40f18ec`: `https://github.com/mborgerding/kissfft/archive/febd4caeed32e33ad8b2e0bb5ea77542c40f18ec.zip`, SHA256 `497103e664168ebe39580b757adbe616f6cf85a16572af581ca7bc42d0ab13fd`. Read-only upstream BSD-3-Clause COPYING SHA256 `a2840585f8411be8e6826a31ef15ae65c950bd74a2437a73b013398a934ad0c6`; verify/retain COPYING and its referenced license before compilation. This is an additional required acquisition, not an already retained library.
- Existing `/usr/bin/cc`, `/usr/bin/c++`, their standard C/C++17 library and platform `libm` only. No CMake needed for the narrow proposed direct compilation; no Python binding, pybind11, gtest, Torch, ONNX, model, service or export toolchain.
- Source interfaces inspected read-only: root CMake SHA256 `47cbc405f559189b4d9eec8cedb1383afd4d29f850e764614609c86df49033e8`; csrc CMake `a5fc8a2859590e02f66552d06d493a27aa056ba051a2983f52bf59e1f875f7cd`; KissFFT recipe `f6038500a227a48df0e309b9490bd9c8cec8ee376008d7211e3fc96f0c689580`; `feature-fbank.h` `4ece984c7ef8664f566fc6d2f94cd2ede8152c071f43c8ba6b56aa704415a515`.

## Proposed source and compiler argv

Prepared source: `knf-feature-accessor.cc`, SHA256 `6e28d9660acdef9126e0f011c7b20426ee5c7cc7f7a73e09ec0bebd1df39e8e1`. It is not compiled or run. It reads bounded little-endian float32 PCM from files, never regenerate a different waveform. It would use public `FbankOptions`, `FeatureWindowFunction::GetWindow`, `NumFrames`, `ExtractWindow`, `FbankComputer::Compute` and `MelBanks::GetMatrix`. Outputs: raw float32 row-major mel energies `[frames,64]`, window `[320]`, mel matrix `[64,161]`, full options and frame count. No log inside KNF; compare energies then apply the official clamp `[1e-9,1e9]`/natural log in retained NumPy. Never claim a dtype difference is harmless without numerical analysis.

Exact settings: 16kHz, frame_length_ms20, shift10, snip_edges true, window `hann` (periodic), dither0, remove_dc_offset false, preemph_coeff0, round_to_power_of_two false;64bins, low_freq0/high_freq8000, is_librosa true, use_slaney_mel_scale false, norm empty, floor_to_int_bin false; use_energy false/use_power true/use_log_fbank false. VTLN1. No implicit defaults for these fields.

Proposed direct build, with KNF/KissFFT verified and extracted into private `$probe/knf` and `$probe/kissfft`:

```sh
/usr/bin/cc -O2 -DNDEBUG -I "$probe/kissfft" -c "$probe/kissfft/kiss_fft.c" -o "$probe/kiss_fft.o"
/usr/bin/cc -O2 -DNDEBUG -I "$probe/kissfft" -c "$probe/kissfft/kiss_fftr.c" -o "$probe/kiss_fftr.o"
/usr/bin/c++ -O2 -DNDEBUG -std=c++17 -I "$probe/knf" -I "$probe/kissfft" \
  "$probe/accessor.cc" \
  "$probe/knf/kaldi-native-fbank/csrc/feature-fbank.cc" \
  "$probe/knf/kaldi-native-fbank/csrc/feature-functions.cc" \
  "$probe/knf/kaldi-native-fbank/csrc/feature-window.cc" \
  "$probe/knf/kaldi-native-fbank/csrc/kaldi-math.cc" \
  "$probe/knf/kaldi-native-fbank/csrc/mel-computations.cc" \
  "$probe/knf/kaldi-native-fbank/csrc/rfft.cc" \
  "$probe/kiss_fft.o" "$probe/kiss_fftr.o" -pthread -lm -o "$probe/accessor"
```

`KNF_ENABLE_CHECK` stays undefined as in upstream's default OFF build; retain compiler versions, compiler and library identities and resulting binary hash. This closure follows inspected source includes; it remains unbuilt and must fail closed if an unlisted dependency appears. Do not fetch it automatically.

## Remaining handoff preparation before a run

This is a concrete source/closure/recipe review, **not yet a runnable approved episode**. Existing compiler identity readback: cc15.2.0 SHA256 `b5f1b773a7c733738352000c92a077dc5852a1a2fc6d836b1e411be1e9ec5f88`; c++15.2.0 SHA256 `e6718f7e0c7d057c3ff77b550c603da9bc4030e3ede3c053705acce1293dbe4d`. PM must verify its actual compiler identities instead of assuming this host readback transfers.

After review, acquisition argv would be `curl --fail --location --proto '=https' --tlsv1.2 --max-time 120 --output "$probe/knf.tar.gz" https://github.com/csukuangfj/kaldi-native-fbank/archive/refs/tags/v1.22.4.tar.gz` and the analogous pinned KissFFT URL/output. Validate the full archive digests above before extraction, reject path traversal/symlink escape, record all extracted file hashes and license bytes. No acquisition command has run.

Copy the exact reviewed accessor into `$probe/accessor.cc`, then use the compiler argv above. Proposed execution argv per case is `"$probe/accessor" "$probe/pcm/CASE.f32le" "$probe/knf-output/CASE"`; output directories must be new. The same-PCM preparation/manifest, shared-deadline/process-group wrapper and exact PM host-runner argv remain to prepare after dependency review. No fabricated hashes are supplied for absent archives/binary bytes. No acquisition/build/run is authorized by this document.

Use all55 retained reference cases, including zero-frame boundaries and400000samples. The source-reference harness currently retains arrays and PCM hashes, not PCM files: a guarded preparation must materialize the same deterministic int16 samples from its pinned construction and verify every existing PCM hash before float32 scaling; never rerun the completed reference episode. Compare exact frame counts and array dimensions first; stop on mismatch. Record per-stage maxabs/relative/coordinates and all raw arrays, without an invented PASS tolerance. Include the retained float32 rounding and scalar-DFT observations in analysis; neither is a Torchaudio/KNF comparison yet.

One PM-owned original-caps/shared600s episode must include build/readback/numerical comparison, fail-closed process-group reaping and independent cleanup. No limit increase, retry, inference or source/installed selection follows automatically. Even matching frontend arrays leaves official-checkpoint/export equivalence, encoded-length integration and automatic RU/EN/code-switch/Russian quality/readiness/long/live gates open.

## Combined guarded PM handoff (note6566515450)

The review above now has a prepared combined script, `knf-frontend-episode.py`,
SHA256 `50f76b4bf3de1212f75ae5c5a67354f937bfb8a171e19b2042f69c666d661a54`.
`knf-frontend-handoff.json`, SHA256
`ca1466a4625e4ff7f32020a250bf4e0c70dc1159bd8f3873c867ef98f08adde3`, pins
both full archive hashes, Apache/BSD license chain, retained reference JSON,
accessor, original reference source and reused cap/process-group helper identities.
This supersedes only the earlier missing-wrapper status. No acquisition,
compilation, feature execution or comparison has run; PM must review these exact
files before dispatch.

The exact **inner** argv, from the worktree root, is:

```sh
target/pm344-giga-static-tools-v2/venv/bin/python -I \
  experiments/speech344/knf-frontend-episode.py \
  --reference target/pm344-frontend-reference-v1 \
  --output target/pm344-knf-frontend-v1
```

The output directory must not exist. Run it once through the existing PM-owned
host isolation runner with actual MemoryMax1280M, MemoryHigh1024M,
MemorySwapMax256M, CPU150%, Tasks256 and one outer600-second episode deadline.
The script starts one internal600-second alarm/deadline before identity checks
and never resets it. Outer startup/import time remains inside the PM deadline.
PM's wrapper argv is owned by that existing runner; the worker has not inspected,
copied or invented its flags. The inner argv above is exact; it is not an
instruction to run it outside that runner.

The script refuses wrong effective caps and retained NumPy version before
acquisition. It invokes only the two pinned HTTPS downloads, once each, through
owned groups with curl configuration disabled and no inherited credential
environment. Both complete archive digests and the full expected license chain
are checked **before extraction**. Paths, duplicate entries, links, special files,
encryption and bounded compressed/member/expanded sizes are checked before
writing extracted source. Sources are plain nonexecutable files; no archive
script/interpreter/build configuration is executed. Compiler commands run from
the new output root, not from downloaded source directories.

Existing compiler executable/resolved-path/hash and actual `--version` output
are recorded. The exact reviewed sequential direct build is retained; any missing
dependency or failed command stops without fetching alternatives. The resulting
accessor hash is recorded before its first invocation. Every child has its own
process group; the existing `finish_group` helper kills descendants and waits on
normal exit, timeout and exceptions. The wrapper also covers interruption between
spawn and helper entry, records PID/PGID/reaping/group-existence readback and
refuses a group still observed after cleanup. PM independently verifies the
unit/MainPID/cgroup/wrapper/descendant cleanup and unchanged ordinary services;
local guard checks cannot prove this host outcome.

All55 ordered reference cases must match. Retained arrays are hash/shape/dtype/
finite checked, and each reconstructed PCM16 byte hash is verified **before**
float32 scaling. The completed NumPy reference script is hash-checked data and
is never imported or rerun. Original PCM16 and scaled float32 files are retained.
The accessor emits raw arrays; the wrapper requires exact frame/byte shapes and
finite window/weights/energy, then records finite prescribed clamp/log arrays.
It retains every raw output plus its size/hash and per-stage maxabs, relative
error on nonzero reference and coordinates. There is **no numeric tolerance or
PASS verdict**. Completion means observation collection only, not frontend
compatibility, checkpoint equivalence, ASR quality or activation.

One deadline covers all checks, both acquisitions, license validation/extraction,
compiler observations, build, all55 calls and comparison/readback. Failure stops
without retry/fallback; terminal metadata can still be written after cancellation,
without dispatching further work. Partial archives/builds/arrays and previous
attempts are not deleted. Output is private0700/0600; logs/commands contain only
public dependency origins and project-local generated paths.

Local preparation evidence: AST/help/manifest identity checks pass. The safe
extraction check initially rejected a legitimate nested tar directory; after
classifying regular directories separately from forbidden links/special files,
the same six offline guard tests pass. They retain traversal/link/digest/license,
original-deadline and pre-scaling PCM identity refusal. Synthetic fixtures use no
model, archive acquisition, compiler, accessor or numerical episode. Actual host
execution remains unrun pending PM review (§9/R19).
