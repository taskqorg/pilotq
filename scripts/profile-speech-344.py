#!/usr/bin/env python3
"""PM isolated diagnostic: unchanged small model, two original stress segments.

This measures CLI model reuse, not recognition acceptance or production selection.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import wave

spec = importlib.util.spec_from_file_location('guard', Path(__file__).with_name('probe-speech-344.py'))
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


def argv(engine, model, inputs, prefixes):
    # Logging stays enabled so the pinned engine emits its native stage counters.
    command = [str(engine), '--model', str(model), '--language', 'auto', '--threads', '4',
               '--processors', '1', '--no-gpu', '--beam-size', '1', '--best-of', '1',
               '--no-fallback', '--output-json']
    for source, prefix in zip(inputs, prefixes, strict=True):
        command += ['--file', str(source), '--output-file', str(prefix)]
    return command


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine', type=Path, required=True)
    parser.add_argument('--model', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    engine, model, out = args.engine.resolve(), args.model.resolve(), args.output.resolve()
    for path, sha in [(engine, '30ab87ffca69ba2e03f6c9c110c158cc5d7e4099eaa0179793559215b99aed70'),
                      (model, 'ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb')]:
        if guard.digest(path) != sha:
            raise ValueError('verified small profiling asset required')
    isolation = guard.limits()
    os.umask(0o077)
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    (out / 'isolation.json').write_text(json.dumps(isolation, indent=2))
    deadline = time.monotonic() + 600  # One budget for decode AND the entire paired diagnostic.
    observations = []

    def run(name, command):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('shared 600-second diagnostic deadline expired')
        started = time.monotonic()
        with (out / (name + '.stdout')).open('wb') as stdout, (out / (name + '.stderr')).open('wb') as stderr:
            child = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=True)
            timed_out = False
            try:
                child.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
        log = (out / (name + '.stderr')).read_text(encoding='utf-8')
        row = {'name': name, 'seconds': time.monotonic() - started, 'argv': command,
               'exit': child.returncode, 'timeout': timed_out,
               'native_timings': [line for line in log.splitlines() if line.startswith('whisper_print_timings:')]}
        observations.append(row)
        (out / 'observations.json').write_text(json.dumps(observations, indent=2))
        print(json.dumps(row), flush=True)
        if timed_out or child.returncode:
            raise RuntimeError('process or deadline failure; stop without retry')

    repo = Path(__file__).resolve().parents[1]
    wav = out / 'human-he.wav'
    run('decode', ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(repo / 'tests/fixtures/speech/he.ogg'),
                   '-t', '301', '-ac', '1', '-ar', '16000', str(wav)])
    with wave.open(str(wav), 'rb') as stream:
        if stream.getnframes() > 300 * 16000:
            raise ValueError('five-minute cap exceeded')
        pcm = stream.readframes(stream.getnframes())
    # Same repetition construction as the retained 300-second stress source; only the
    # first two existing 25-second segments are used for this paired diagnostic.
    pcm = (pcm * (300 * 32000 // len(pcm) + 1))[:300 * 32000]
    inputs = []
    for n in range(2):
        path = out / f'segment-{n}.wav'
        with wave.open(str(path), 'wb') as stream:
            stream.setnchannels(1)
            stream.setsampwidth(2)
            stream.setframerate(16000)
            stream.writeframes(pcm[n * 25 * 32000:(n + 1) * 25 * 32000])
        inputs.append(path)
    separate = [out / f'separate-{n}' for n in range(2)]
    together = [out / f'batch-{n}' for n in range(2)]
    for n in range(2):
        run(f'separate-{n}', argv(engine, model, [inputs[n]], [separate[n]]))
    run('batch', argv(engine, model, inputs, together))
    comparison = []
    for n in range(2):
        texts = []
        for prefix in [separate[n], together[n]]:
            data = json.loads(prefix.with_suffix('.json').read_text(encoding='utf-8'))
            text = ' '.join(part['text'] for part in data['transcription'])
            if not text.strip() or '\ufffd' in text:
                raise ValueError('empty or damaged diagnostic transcript')
            texts.append(text)
        comparison.append({'segment': n, 'same_text': texts[0] == texts[1], 'texts': texts})
    (out / 'comparison.json').write_text(json.dumps(comparison, ensure_ascii=False, indent=2))
    # Differences are retained as results, never normalized into a passing comparison.
    (out / 'completed.json').write_text(json.dumps({'isolation': isolation, 'comparison': comparison}, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
