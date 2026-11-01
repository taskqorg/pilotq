#!/usr/bin/env python3
"""Candidate-only PM probe. No downloads, routing, model fallback or installed writes."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import time
import wave

REPO = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('guard344', REPO / 'scripts/probe-speech-344.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
shared_spec = importlib.util.spec_from_file_location('english344', Path(__file__).with_name('probe-sherpa-english.py'))
shared = importlib.util.module_from_spec(shared_spec)
shared_spec.loader.exec_module(shared)
assess = shared.assess
finish_group = shared.finish_group
ASSETS = {
    'whisper-cli': (2633512, '30ab87ffca69ba2e03f6c9c110c158cc5d7e4099eaa0179793559215b99aed70'),
    'ggml-small-q5_1.bin': (190085487, 'ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb'),
    'LICENSE.whisper.cpp': (1078, 'e562a2ddfaf8280537795ac5ecd34e3012b6582a147ef69ba6a6a5c08c84757d'),
    'LICENSE.openai-whisper': (1063, 'b5d65a59060e68c4ff940e1eddfa6f94b2d68fdf58ed7f4dd57721c997e35e9d'),
}


def command(engine, assets, wav, prefix, readiness=False):
    argv = [str(engine), '--model', str(assets / 'ggml-small-q5_1.bin'),
            '--language', 'en', '--threads', '4', '--processors', '1', '--no-gpu',
            '--beam-size', '1', '--best-of', '1', '--no-fallback', '--output-json',
            '--output-file', str(prefix), '--file', str(wav)]
    # Only verified generated digital silence receives reduced encoder context.
    # Every ordinary speech call constructs a fresh argv with default full context.
    if readiness:
        argv += ['--audio-ctx', '50']
    return argv


def read_result(path, readiness=False):
    data = json.loads(path.read_text(encoding='utf-8', errors='strict'))
    parts = data.get('transcription')
    if not isinstance(parts, list) or (not parts and not readiness):
        raise ValueError('missing transcription segments')
    texts = [part['text'] for part in parts]
    if any(not isinstance(text, str) or '\ufffd' in text for text in texts):
        raise ValueError('damaged/non-string transcript')
    return ' '.join(texts)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('engine', 'assets', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    args = p.parse_args()
    engine, assets, out = args.engine.resolve(), args.assets.resolve(), args.output.resolve()
    if guard.digest(engine) != '30ab87ffca69ba2e03f6c9c110c158cc5d7e4099eaa0179793559215b99aed70':
        raise ValueError('retained ORIGINAL whisper.cpp1.8.2 binary required')
    for name, (size, sha) in ASSETS.items():
        path = assets / name
        if path.stat().st_size != size or guard.digest(path) != sha:
            raise ValueError('asset identity mismatch: ' + name)
    human = REPO / 'tests/fixtures/speech/en.ogg'
    private = REPO / 'target/pm344-speech-inputs/speech344-en.wav'
    for path, sha in [(human, '23860dba7d1d072f39b9244e31ee151c89f1192537bd587bb37baeb371f6430f'),
                      (private, 'b09402db6e291ac05bc452e72b59be0f4450a4a2795076f5a53838e5a250d930')]:
        if guard.digest(path) != sha:
            raise ValueError('unchanged English fixture required')
    if engine != (assets / 'whisper-cli').resolve():
        raise ValueError('engine must be the verified retained asset')
    isolation = guard.limits()  # Refuse missing/wrong actual effective caps before dispatch.
    os.umask(0o077)
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    deadline = time.monotonic() + 600  # ALL readiness/decode/two clips/readback share it.
    evidence = {'status': 'incomplete', 'isolation': isolation, 'runs': [],
                'assets': {name: {'bytes': size, 'sha256': sha} for name, (size, sha) in ASSETS.items()},
                'script_sha256': guard.digest(Path(__file__)), 'manifest_sha256': guard.digest(assets / 'manifest.json'),
                'engine_sha256': guard.digest(engine), 'fixture_sha256': {'human-en': guard.digest(human), 'private-en': guard.digest(private)}}

    def save():
        (out / 'observations.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2))

    def run(name, argv, limit=None, json_result=False):
        remaining = deadline - time.monotonic()
        if limit is not None:
            remaining = min(remaining, limit)
        if remaining <= 0:
            raise TimeoutError('shared600s deadline before dispatch')
        started = time.monotonic()
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            child = subprocess.Popen(argv, stdout=stdout, stderr=stderr, start_new_session=True)
            timed_out = finish_group(child, remaining)
        row = {'name': name, 'argv': argv, 'seconds': time.monotonic() - started,
               'exit': child.returncode, 'timeout': timed_out,
               'peak_child_KiB_cumulative': resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss}
        evidence['runs'].append(row)
        save()
        if timed_out or child.returncode:
            raise RuntimeError(name + ': process/deadline failure; stop without retry')
        if json_result:
            text = read_result(out / (name + '.json'), readiness=name == 'readiness')
            row['text'] = text
            row['delivered'] = False
            save()
            # A fixed-language probe observes that mode; a language label is not quality proof.
            if name != 'readiness':
                row['quality'] = assess(name, text)
                save()
                if not row['quality']['pass']:
                    raise ValueError(name + ': full reference/numeric guard failed; stop')
        if time.monotonic() > deadline:
            raise TimeoutError('shared600s deadline includes readback')
        return row

    def decode(name, source, target):
        run(name, ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(source), '-t', '301',
                   '-ac', '1', '-ar', '16000', str(target)])
        with wave.open(str(target), 'rb') as audio:
            if audio.getnchannels() != 1 or audio.getsampwidth() != 2 or audio.getframerate() != 16000:
                raise ValueError('decode format mismatch')
            if not 0 < audio.getnframes() <= 300 * 16000:
                raise ValueError('five-minute recording cap')

    try:
        # Real one-second PCM silence through decode and engine, never a fake successful transcript.
        ready_deadline = time.monotonic() + 10
        source = out / 'silence-source.wav'
        with wave.open(str(source), 'wb') as audio:
            audio.setnchannels(1); audio.setsampwidth(2); audio.setframerate(16000)
            audio.writeframes(bytes(32000))
        silence = out / 'silence.wav'
        run('readiness-decode', ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(source),
                                '-ac', '1', '-ar', '16000', str(silence)], max(0, ready_deadline - time.monotonic()))
        run('readiness', command(engine, assets, silence, out / 'readiness', readiness=True), max(0, ready_deadline - time.monotonic()), True)
        if time.monotonic() > ready_deadline:
            raise TimeoutError('readiness10s includes decode/readback')
        # Silence-generated words are retained as evidence but never treated as speech or delivered.
        for name, original in [('human-en', human), ('private-en', private)]:
            wav = out / (name + '.wav')
            decode(name + '-decode', original, wav)
            evidence.setdefault('decoded_sha256', {})[name] = guard.digest(wav)
            run(name, command(engine, assets, wav, out / name), json_result=True)
        evidence['status'] = 'short English diagnostic pass only; no long/installed/live qualification'
    except BaseException as error:
        evidence['failure'] = str(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    def terminate(signum, frame):
        raise SystemExit('probe terminated; child process group fenced')
    signal.signal(signal.SIGTERM, terminate)
    main()
