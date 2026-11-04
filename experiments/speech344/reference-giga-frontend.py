#!/usr/bin/env python3
"""PM-only source-derived reference, not KNF/Torchaudio parity. No model access."""
import argparse
import hashlib
import importlib.metadata
import importlib.util
import json
import math
import os
from pathlib import Path
import resource
import signal
import time

HERE = Path(__file__).resolve().parent


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    start = time.monotonic()
    def terminate(signum, frame):
        raise TimeoutError('reference shared600s or termination')
    signal.signal(signal.SIGALRM, terminate)
    signal.signal(signal.SIGTERM, terminate)
    signal.setitimer(signal.ITIMER_REAL, 600)
    spec = importlib.util.spec_from_file_location('guard344', HERE.parents[1] / 'scripts/probe-speech-344.py')
    guard = importlib.util.module_from_spec(spec); spec.loader.exec_module(guard)
    isolation = guard.limits()
    if importlib.metadata.version('numpy') != '2.4.2':
        raise ValueError('retained NumPy version mismatch')
    manifest_path = HERE / 'frontend-reference-sources.json'
    sources = json.loads(manifest_path.read_text())
    for name, identity in sources.items():
        if Path(name).name != name:
            raise ValueError('invalid source name')
        p = args.sources / name
        if p.stat().st_size != identity['bytes'] or sha(p) != identity['sha256']:
            raise ValueError('source identity mismatch: ' + name)
    for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'NUMEXPR_NUM_THREADS'):
        os.environ[name] = '4'
    os.umask(0o077)
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    evidence = {'status': 'incomplete', 'scope': 'source-derived reference only; no actual KNF/Torchaudio/export parity',
                'model_accessed': False, 'isolation': isolation, 'sources': sources,
                'script_sha256': sha(Path(__file__)), 'manifest_sha256': sha(manifest_path),
                'numpy': '2.4.2', 'numeric_pass_tolerance': None, 'cases': []}
    try:
        import numpy as np
        # Explicit versioned Torchaudio mathematical convention, not an imported official implementation.
        mel_max = 2595.0 * math.log10(1.0 + 8000.0 / 700.0)
        endpoints = 700.0 * (10.0 ** (np.linspace(0.0, mel_max, 66) / 2595.0) - 1.0)
        frequencies = np.linspace(0.0, 8000.0, 161)
        weights = np.maximum(0.0, np.minimum(
            (frequencies[:, None] - endpoints[:-2]) / (endpoints[1:-1] - endpoints[:-2]),
            (endpoints[2:] - frequencies[:, None]) / (endpoints[2:] - endpoints[1:-1])))
        # KNF default is a DIFFERENT mel-space triangle convention, included as source comparison only.
        mel_frequency = 1127.0 * np.log1p(frequencies / 700.0)
        mel_points = np.linspace(0.0, 1127.0 * math.log1p(8000.0 / 700.0), 66)
        kaldi_weights = np.maximum(0.0, np.minimum(
            (mel_frequency[:, None] - mel_points[:-2]) / (mel_points[1:-1] - mel_points[:-2]),
            (mel_points[2:] - mel_frequency[:, None]) / (mel_points[2:] - mel_points[1:-1])))
        window = 0.5 - 0.5 * np.cos(2.0 * math.pi * np.arange(320) / 320.0)
        def save(name, array):
            if not bool(np.isfinite(array).all()):
                raise ValueError('nonfinite array: ' + name)
            p = args.output / (name + '.npy'); np.save(p, array, allow_pickle=False)
            return {'path': p.name, 'shape': list(array.shape), 'dtype': str(array.dtype), 'sha256': sha(p)}
        def difference(a, b):
            if a.shape != b.shape:
                raise ValueError('shape mismatch')
            if not bool(np.isfinite(a).all() and np.isfinite(b).all()):
                raise ValueError('nonfinite comparison')
            delta = np.abs(a - b)
            if not delta.size:
                return {'max_abs': 0.0, 'max_relative_on_nonzero_reference': None, 'coordinate': None}
            index = tuple(int(x) for x in np.unravel_index(int(np.argmax(delta)), delta.shape))
            nonzero = a != 0
            return {'max_abs': float(delta[index]), 'coordinate': list(index),
                    'max_relative_on_nonzero_reference': float(np.max(delta[nonzero] / np.abs(a[nonzero]))) if bool(nonzero.any()) else None}
        evidence['window'] = save('window-periodic-hann', window)
        evidence['mel_weights'] = save('weights-htk-hz', weights)
        evidence['knf_default_mel_weights'] = save('weights-default-mel', kaldi_weights)
        evidence['mel_convention_difference'] = difference(weights, kaldi_weights)
        for count in (0, 319, 320, 321, 479, 480, 481, 640, 1600, 400000):
            for kind in (('tone',) if count == 400000 else ('zero','impulse-first','impulse-middle','impulse-last','ramp','tone')):
                if time.monotonic() - start >= 600:
                    raise TimeoutError('shared600s')
                samples = np.zeros(count, dtype=np.int16)
                if count and kind.startswith('impulse'):
                    index = 0 if kind.endswith('first') else count // 2 if kind.endswith('middle') else count - 1
                    samples[index] = 16384
                elif kind == 'ramp':
                    samples = (np.arange(count, dtype=np.int64) % 32768 - 16384).astype(np.int16)
                elif kind == 'tone':
                    samples = np.rint(8192.0 * np.sin(2.0 * math.pi * 1000.0 * np.arange(count) / 16000.0)).astype(np.int16)
                expected = max(0, (count - 320) // 160 + 1)
                frames = np.empty((0,320), dtype=np.float64) if count < 320 else np.lib.stride_tricks.sliding_window_view(samples.astype(np.float64)/32768.0,320)[::160]
                if frames.shape != (expected,320):
                    raise ValueError('exact framing mismatch')
                spectrum = np.fft.rfft(frames * window, n=320, axis=1)
                power = np.abs(spectrum)**2
                mel = power @ weights
                logmel = np.log(np.clip(mel,1e-9,1e9))
                # Float32 stage rounding observation, NOT a Torch-backend emulator.
                spectrum32 = np.fft.rfft(frames.astype(np.float32) * window.astype(np.float32), n=320, axis=1).astype(np.complex64)
                mel32 = (np.abs(spectrum32)**2) @ weights.astype(np.float32)
                log32 = np.log(np.clip(mel32,np.float32(1e-9),np.float32(1e9)))
                name = f'{kind}-{count}'
                row = {'name': name, 'samples': count, 'pcm16le_sha256': hashlib.sha256(samples.astype('<i2').tobytes()).hexdigest(),
                       'frames': expected, 'mel64': save(name+'-mel64',mel), 'logmel64': save(name+'-log64',logmel),
                       'logmel32': save(name+'-log32',log32), 'float_stage_difference': difference(logmel,log32.astype(np.float64))}
                if expected and count <= 640:
                    # Independent scalar DFT of the first frame; no FFT or matrix multiplication here.
                    original = samples[:320].tolist()
                    scalar_frame = [(float(v)/32768.0)*(0.5-0.5*math.cos(2*math.pi*j/320)) for j,v in enumerate(original)]
                    scalar_power=[]
                    for k in range(161):
                        re=math.fsum(v*math.cos(2*math.pi*k*j/320) for j,v in enumerate(scalar_frame))
                        im=-math.fsum(v*math.sin(2*math.pi*k*j/320) for j,v in enumerate(scalar_frame))
                        scalar_power.append(re*re+im*im)
                    scalar_mel=[]
                    for m in range(64):
                        left,center,right=(float(v) for v in endpoints[m:m+3])
                        scalar_mel.append(math.fsum(p*max(0.0,min((50.0*k-left)/(center-left),(right-50.0*k)/(right-center))) for k,p in enumerate(scalar_power)))
                    row['scalar_dft_power_difference']=difference(power[0],np.asarray(scalar_power))
                    row['scalar_mel_difference']=difference(mel[0],np.asarray(scalar_mel))
                evidence['cases'].append(row)
        evidence['status']='reference observations complete; numeric differences unqualified, no engine parity'
    except BaseException as error:
        evidence['failure']=str(error)
        raise
    finally:
        evidence['seconds']=time.monotonic()-start
        evidence['peak_self_KiB']=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        (args.output/'reference.json').write_text(json.dumps(evidence,indent=2,allow_nan=False))


if __name__ == '__main__':
    main()
