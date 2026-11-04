# Prepared supported offline CLI / short-quality PM handoff

Preparation authorized by note6626590400. **No configure, compile, link, model load or inference has run.** PM must separately approve and dispatch this one episode. Existing Russian executable, assets, settings and services remain untouched (R5, R19, §9).

## Exact closure

The [manifest](giga344-cli-handoff.json) pins full origins, sizes, hashes and every license/notice digest. Twelve archives are retained privately in `target/pm344-cli-closure`; execution does no acquisition. Sherpa is official v2.0.1 commit af2d8d81dbfa61c4cea4c568da78f3982edca005. ORT is the exact Sherpa-maintainer distributed binary, not Microsoft binary attestation. CMake4.4.5 is a private build input only, never installed or added to PATH. Other dependencies follow this pinned Sherpa's CMake closure; no replacement or missing-dependency download is permitted.

| Archive | Bytes | SHA256 | License |
| --- | ---: | --- | --- |
| sherpa | 11932226 | `afcafe3c34bc2f15a363ad791fa1644a69bf0977a58c1b4f617468976c2faeb5` | Apache-2.0 |
| ort | 8781463 | `c4f8994d56191d9d2c92a961b39fe790459f2c5d155f912b239506ea31359534` | MIT plus bundled ThirdPartyNotices |
| knf | 72131 | `257fea639585ca6ee00860493bb9d7ab32488b8a5407ffe434c1f361ea545fae` | Apache-2.0 |
| kissfft | 74252 | `497103e664168ebe39580b757adbe616f6cf85a16572af581ca7bc42d0ab13fd` | BSD-3-Clause |
| utf8proc | 207976 | `f564011d38b2888d583d510b08e69ffa15aa117155db1b9b49ef1dfe1fa25111` | MIT plus Unicode data notice |
| kaldi-decoder | 51199 | `b9f34cfb4fd3b1344100eead79ef4d37aa15962274b9e3056de345021f76a1b0` | Apache-2.0 |
| simple-sentencepiece | 355335 | `1748a822060a35baa9f6609f84efc8eb54dc0e74b9ece3d82367b7119fdc75af` | Apache-2.0 |
| json | 9678593 | `4b92eb0c06d10683f7447ce9406cb97cd4b453be18d7279320f7b2f025c10187` | MIT |
| kaldifst | 172147 | `3f247b7e5a2409071202f5e2bc6200060f66728c0a3443c03923ad2723e040b3` | Apache-2.0 |
| eigen | 2705005 | `8586084f71f9bde545ee7fa6d00288b264a2b7ac3607b974e54d13e7162c1c72` | MPL-2.0 primary; bundled per-file license alternatives retained |
| openfst | 1501685 | `2ff712a32952fcb01d351121a6bc8ccf4fdc6b2aa06ce8df2b3095dedd518c0e` | Apache-2.0 |
| cmake | 64865573 | `ed85135c5a5135b45e50423cb406dea04ca3338da0b8ef3c9ede963559680be0` | BSD-3-Clause plus bundled notices |

Exact license files and25 notice digests are in the manifest. Source extraction checks archive and license hashes before writing selected regular files; selected links/path traversal fail closed. The root Sherpa CMake files and csrc/cmake trees are retained; unrelated archive platform links are never followed. CMake uses only its selected binary/data/license files. System compiler, make, Git, readelf and ffmpeg byte identities are pinned. Host standard platform libraries must already exist; missing libraries stop with evidence.

## Build and quality sequence

[The wrapper](giga344-cli-episode.py) owns argument arrays; [the command expansion](giga344-cli-commands.json) shows configure/build with literal `$OUT` review placeholders. Only the supported `sherpa-onnx-offline` target builds, Release/shared/CPU, four build threads, candidate compile define. All FetchContent sources are explicit offline overrides; downloads/updates disconnected. Optional Python, GPU, audio-device, websocket, TTS, diarization and platform backends are disabled. The verified ORT include/lib directories are supplied explicitly. No install target runs. The candidate patch applies only to the new extracted source.

One actual outer600s episode includes startup, identities, extraction, configure/build, ELF/binary readback, asset copying/checks, readiness, decoding, five short recognitions and final readback. It may exhaust memory or time during compilation; no completion promise is inferred from the earlier no-model build. Use original MemoryMax1280M/MemoryHigh1024M/MemorySwapMax256M/CPU150%/Tasks256/four threads. Every child owns a process group and is killed/reaped on exception/deadline; any surviving observed group stops. PM independently verifies inactive/MainPID0, wrapper/descendants/cgroup gone and unchanged ordinary service PIDs/restarts.

Readiness is one second of zero PCM, no delivered output; generation, engine/asset hash checks, recognition and transcript/evidence readback must finish within original10s, inside the same600s. Nonempty silence output stops. Then decode unchanged human RU/EN Ogg into16k monoPCM and run humanRU, humanEN, privateRU, privateEN, contiguous unpadded humanRU+EN. Every short clip is capped25s. No expected text, fixed-language hint, source stretching or added speech padding goes to recognition. CPU greedy NeMo CTC uses the same candidate model and tokens for every case, with four threads and no fallback.

The manifest pins the already retained community fussraider220M int8 model224762524bytes SHA2d94f93ffd4ef58e7899c9de885c25bbbc8c9f1073618868d118a674450ba5f7, tokens and MIT notice at revision9f5a77e8975211abe8511693accd3a63ee1e9f43. Official checkpoint/export equivalence remains unknown. Verified copies live only in the new private output; asset/engine identities are rechecked before each call. Private synthetic MacTTS fixtures are not published. Full original normalized references, strict UTF8/JSON/type checks and numeric words remain unchanged. First mismatch stops; retain ordered complete text and error count, no best-prefix acceptance, retry or reference weakening.

## Exact proposed PM inner argv

From the task worktree, through the existing PM-owned isolated runner only:

```sh
target/pm344-giga-static-tools-v2/venv/bin/python -I \
  experiments/speech344/giga344-cli-episode.py \
  --archives target/pm344-cli-closure \
  --assets target/pm344-giga-multilingual-assets \
  --output target/pm344-giga-cli-v2
```

Output must not exist. PM verifies actual checkout, retained Python/tool/archive/fixture identities first; missing inputs stop without substitution. This is an inner argv, not authorization to run outside original isolation or invent outer runner flags. PM must authorize this revised v2 episode before dispatch. Do not reuse or remove v1 output.

Script SHA256: `53ce144a91bda368321413fd20ebbb122990b2f637de8c0b5942d4894bf0a808`.

Manifest SHA256: `7df695ba7d6b5a31bf9d85b1523a29c26026b3452b1c49b7496b84cd022cef47`.

## Evidence and remaining gates

The readiness-readback regression initially failed (`TimeoutError not raised` at10.001s), then passed after elapsed-time enforcement;9.999s remains accepted. Five affected handoff guards and six existing KNF guards pass. This retains the deadline, malformed-output, PCM-cap and unsafe-extraction protections; it is not model proof. No accepted product rule or tolerance changed (specq testing, §7 step4).

The earlier181-run/55-case no-model observations remain unqualified numerical measurements. Even complete short reference equality would prove only these clips and this constructed two-speaker switch. Natural same-speaker code-switching, baseline installed Russian quality parity, official frontend/checkpoint equivalence, service cancellation/corrupt/long300, all shared intake paths, output-language coverage, CI merge_group, final-source A40, release and live Chrome acceptance remain separate open gates (§§1/3/9/12, R19). No production engine selection or activation follows this handoff.

## Failed v1 and corrected v2 preparation (note6627510790)

PM ran v1 once at1bd9bbb: build failed after111s/exit1 before model load or recognition. The retained `fst-utils.cc:69` error requires `FarReader::Open(unique_ptr<istringstream>)`; the April OpenFst override provided only filename/list overloads. Worker read the original stderr/observations; they remain untouched. PM reports peak1G plus115M swap, original caps, independent cleanup and unchanged service PIDs/restarts. No quality verdict follows this build failure.

The mistaken override followed transitive kaldifst1.8.0's April pin instead of the owning Sherpa `cmake/openfst.cmake` July9 pin. V2 uses that exact csukuangfj fork archive,1501685bytes, full digest in the table. Its COPYING digest is unchanged; actual `far.h` contains the stream declaration/implementation (SHA256 c6ad725b0819f3c8020e487dc24318996d1bb21acd7da5095485746f29f28b74). No target or validation was removed. The guard checks the override URL/digest against the extracted owning Sherpa declaration before OpenFst extraction. The old actual entry is rejected; the corrected actual entry passes. Six handoff guards and six unchanged KNF guards pass. This is static pin/guard evidence, not compiler red/green: no worker configure/build/model run is authorized. Only the separately approved PM v2 run can establish build outcome.

All other archives, model/fixtures, commands, quality references, caps, readiness10s, shared600s and cleanup remain unchanged. Preserve the old April archive and failed v1 evidence. Stop on any v2 failure without retry/fallback, new dependencies or limit increases. Official equivalence, numerical qualification and ASR acceptance remain open (R5, §9, R19).
