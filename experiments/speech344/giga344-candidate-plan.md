# Candidate-only GigaAM multilingual patch

This is an unbuilt draft authorized by TaskQ note6604186692. It changes no PilotQ runtime or installed asset. It is not production selection, a build recipe or permission to execute a model (§9, R19). The approved automatic Russian/English goal, no manual selector/retry UI and latest-direct-address reply language remain unchanged.

## Scope and identity boundary

`giga344-candidate.json` pins Sherpa v2.0.1 commit, every changed source file, patch SHA256 and exact retained community220M int8 model/token identities. The patch is against upstream source, not the installed executable. Upstream Sherpa is Apache-2.0; keep its LICENSE/NOTICE and KNF Apache/KissFFT BSD notices in any subsequently approved build. Community model MIT/provenance remains retained; official-checkpoint equivalence is unknown.

The draft is enabled only by `SHERPA_ONNX_EXPERIMENT_GIGA344` at compile time. There is no CLI language selector or fallback. A build without the define keeps the old feature configuration and floor-length calculation. The experimental build must never be installed or used as a general recognizer. Its NeMo loader refuses other vocabulary/subsampling/normalization/interfaces. Structural metadata checks are **not cryptographic identity**. A future reviewed outer harness must verify the exact model/token/engine/patch hashes immediately before loading and own the immutable private assets for the episode. That wrapper is not supplied or approved by this patch-only step. No model bytes or metadata are modified.

The existing Russian RNNT path is not edited. The conditional feature flag is set only inside the GigaAM NeMo CTC branch; other model frontends remain unchanged. This does not prove Russian non-regression for a new multilingual model.

## Prepared changes

- Explicit16kHz normalized PCM,20ms window,10ms shift, snipped framing, periodic Hann,64bins,0–8000Hz, no dither/DC/preemphasis/FFT rounding. KNF uses HTK-Hz triangles without Slaney normalization or integer-bin rounding.
- Raw power mel energies are checked finite/nonnegative, then clamped to `[1e-9,1e9]` and natural-logged when copied to returned frames. This avoids KNF's generic epsilon/additive-log defaults. No gain, padding, normalization or expected words are inserted.
- Candidate input tensors require FLOAT `[B,T,64]` and INT64 `[B]` with positive valid lengths, maximum2499 feature frames (the original25s segment). Checks precede candidate output allocation. Empty/short input fails rather than inventing padded speech.
- After inference, require FLOAT `[B,Tout,71]` and INT32 `[B]`. Copy each graph-returned length numerically into INT64 only when positive by equality to the source-derived `(L+3)/4` and no greater than `Tout`. Padded output time must equal the source-derived ceil of padded input time, and every log probability must be finite. Wrong count/type/rank/batch/length or nonfinite output throws; no floor fallback or reinterpret cast.

The measured KNF/reference discrepancies remain unqualified, including near-floor log differences and the terminal mel weight. This patch neither fixes nor declares those differences harmless. It prepares an explicit frontend for a later reviewed experiment; it does not prove actual Torchaudio or weight parity.

## Required boundary proof before any speech acceptance

Prepare a separately reviewed no-model boundary harness before authorizing a build. It must use the actual patched FeatureExtractor and loader validation, rather than duplicating their arithmetic as the oracle:

1. Retained55 PCM cases: exact hashes/frame counts/shapes, raw returned feature arrays versus retained accessor/reference arrays. Include repeated GetFrames/stream isolation, zero/319 samples,320/479/480/481, and400000samples. Keep raw numeric differences, no invented tolerance or repeated reference episode.
2. Synthetic ONNX outputs or an extracted validation seam: correct INT32 lengths for1–12/99–101/2499; reject wrong dtype/rank/batch/vocabulary, zero/negative/oversized/wrong-ceil lengths, wrong padded output time, nonfinite log probabilities and nonfinite/negative feature energies. A valid input with changed output length must fail, demonstrating the guard observes returned data.
3. Compile-disabled legacy frontend/length observations versus exact upstream, plus unchanged Russian RNNT caller/configuration. Compile applicability is not functional equivalence.
4. Any later real-engine episode must separately verify exact immutable identities, original actual caps/threads4, readiness10/shared600s/300s/25s, cancellation/fencing/process groups and cleanup. No language hints, source padding, fallback, new dependencies or automatic probe follows this document. Official provenance gaps must be reviewed before loading.

## Current verification and limits

Source text was read at the pinned commit; `features.cc` matches the earlier inspected hash. `git apply --check --directory=target/pm344-sherpa-patch-check experiments/speech344/giga344-candidate.patch` passes against copies of those exact source files. The first applicability command used an incorrect relative patch path and failed to open the patch; correcting the path passed. That was a command-path error, not an engine defect or red/green recognition proof.

No compile, preprocessing, linking, feature execution, ONNX/model inference, dependency acquisition, installed write or service restart occurred. Header/API/build compatibility, real FeatureExtractor parity, loader rejection tests, Russian quality, mixed speech, long/readiness/live and merge-result CI remain open. Patch preparation alone is not A2/A4 completion (R19).

Independent read-only review found missing finite-log-probability validation in the first draft; the draft now rejects nonfinite values before decoder dispatch. This is source correction only: the retained risk protection requires future synthetic rejection tests, and no executed red/green loader test is claimed.
