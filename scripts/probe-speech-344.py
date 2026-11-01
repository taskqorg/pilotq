#!/usr/bin/env python3
"""PM-only CTC1B evaluation; run through the existing host isolation runner.

No downloading, installation, prompts, reference rewriting or model selection.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import resource
import subprocess
import time
import wave


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def limits(proc=Path('/proc/self/cgroup'), mount=Path('/sys/fs/cgroup')):
    """Read effective ancestor minima, refusing missing or different real caps."""
    relative = next(line[3:] for line in proc.read_text().splitlines() if line.startswith('0::'))
    leaf = mount / relative.lstrip('/')
    if '..' in Path(relative).parts:
        raise ValueError('invalid cgroup identity')
    observed = {}
    for name, expected in [('memory.max', 1280 * 1024**2),
                           ('memory.high', 1024 * 1024**2),
                           ('memory.swap.max', 256 * 1024**2), ('pids.max', 256)]:
        values = []
        for directory in [leaf, *leaf.parents]:
            if directory == mount.parent:
                break
            path = directory / name
            if directory == mount and not path.exists():
                continue
            value = path.read_text().strip()
            if value != 'max':
                values.append(int(value))
        observed[name] = min(values) if values else None
        if observed[name] != expected:
            raise ValueError(f'effective {name}: {observed[name]}, required {expected}')
    from fractions import Fraction
    cpus = []
    for directory in [leaf, *leaf.parents]:
        if directory == mount.parent:
            break
        path = directory / 'cpu.max'
        if directory == mount and not path.exists():
            continue
        quota, period = path.read_text().split()
        if quota != 'max':
            cpus.append(Fraction(int(quota), int(period)))
    effective = min(cpus) if cpus else None
    if effective != Fraction(3, 2):
        raise ValueError(f'effective CPU quota: {effective}, required 3/2')
    return {'cgroup': relative, 'unit': next((part for part in reversed(leaf.parts) if part.endswith(('.service', '.scope'))), leaf.name), 'effective': observed, 'cpu_quota': str(effective)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--assets', type=Path, required=True)
    parser.add_argument('--engine', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    assets, engine, output = args.assets.resolve(), args.engine.resolve(), args.output.resolve()
    # All assets must be verified BEFORE an engine is started; partial downloads cannot qualify.
    for path, expected in [
        (engine, 'bf482639a18895904336da88d3a5fab1ca94bee79235bd1bd4666363887185eb'),
        (assets / 'model.int8.onnx', 'f7b74c964039162423b83e3fa950ce24810c9a635d9ff8468b5f4d142b7c1e8c'),
        (assets / 'tokens.txt', 'a7a044c52cb29cbe8b0dc1953e92cefd4ca16b0ed968177b6beab21f9a7d0b31'),
        (assets / 'LICENSE', 'a70a523bafbb595c2844104feb313d204904dac91c3d186c05f22a10a71c7a94'),
    ]:
        if digest(path) != expected:
            raise ValueError(f'asset identity mismatch: {path.name}')
    isolation = limits()  # Fail closed; never fall back to an inherited-profile claim.
    private = repo / 'target/pm344-speech-inputs'
    manifest = json.loads((private / 'manifest.json').read_text())
    for case in manifest['cases'].values():
        for name, identity in case['files'].items():
            if Path(name).name != name or digest(private / name) != identity['sha256']:
                raise ValueError('private fixture identity mismatch')
    os.umask(0o077)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    (output / 'isolation.json').write_text(json.dumps(isolation, indent=2))
    base = [str(engine), f'--omnilingual-asr-model={assets / "model.int8.onnx"}',
            f'--tokens={assets / "tokens.txt"}', '--num-threads=4', '--provider=cpu']
    observations = []

    def run(name, command, deadline, json_result=False):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('recording deadline expired before dispatch')
        start = time.monotonic()
        with (output / f'{name}.stdout').open('wb') as stdout, (output / f'{name}.stderr').open('wb') as stderr:
            child = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=True)
            timed_out = False
            try:
                child.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
        entry = {'name': name, 'seconds': time.monotonic() - start,
                 'exit': child.returncode, 'timeout': timed_out, 'argv': command,
                 'peak_child_KiB_cumulative': resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss}
        observations.append(entry)
        (output / 'observations.json').write_text(json.dumps(observations, ensure_ascii=False, indent=2))
        if timed_out or child.returncode:
            raise RuntimeError(f'{name}: timeout or process failure; stop, no retries')
        if json_result:
            data = json.loads((output / f'{name}.stdout').read_text(encoding='utf-8'))
            text = data['text']
            if not isinstance(text, str) or '\ufffd' in text:
                raise ValueError('invalid transcript')
            entry['text'] = text
            (output / 'observations.json').write_text(json.dumps(observations, ensure_ascii=False, indent=2))
        print(json.dumps(entry, ensure_ascii=False), flush=True)

    def wav(path, pcm):
        with wave.open(str(path), 'wb') as stream:
            stream.setnchannels(1)
            stream.setsampwidth(2)
            stream.setframerate(16000)
            stream.writeframes(pcm)

    silence = output / 'silence.wav'
    wav(silence, bytes(32000))
    run('readiness', base + [str(silence)], time.monotonic() + 10, True)
    for name, source in [(lang, repo / f'tests/fixtures/speech/{lang}.ogg') for lang in ['ru', 'en', 'he']] + [
        ('private-he', private / 'speech344-he.wav')]:
        deadline = time.monotonic() + 600
        decoded = output / f'{name}.wav'
        run(name + '-decode', ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(source),
                              '-t', '301', '-ac', '1', '-ar', '16000', str(decoded)], deadline)
        with wave.open(str(decoded), 'rb') as stream:
            if stream.getnframes() > 300 * 16000:
                raise ValueError('recording exceeds five-minute cap')
        run(name, base + [str(decoded)], deadline, True)
    # Identical repeated human PCM construction to prior experiments; this is stress data.
    with wave.open(str(output / 'he.wav'), 'rb') as stream:
        pcm = stream.readframes(stream.getnframes())
    pcm = (pcm * (300 * 32000 // len(pcm) + 1))[:300 * 32000]
    deadline = time.monotonic() + 600
    for n in range(12):
        part = output / f'long-{n}.wav'
        wav(part, pcm[n * 25 * 32000:(n + 1) * 25 * 32000])
        run(f'long-{n}', base + [str(part)], deadline, True)
    (output / 'completed.json').write_text(json.dumps({'long_segments': 12, 'isolation': isolation}, indent=2))


if __name__ == '__main__':
    main()
