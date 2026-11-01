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
import unicodedata
import wave

REPO = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('guard344', REPO / 'scripts/probe-speech-344.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
ASSETS = {
    'small-encoder.int8.onnx': (112442483, '4cbe7b22fa9026b843b60a68640c747de05bafb1a11b57edc0e66c232d9f33a9'),
    'small-decoder.int8.onnx': (262226114, 'acad50b5c782696e91b55914cc5ab4f756f1532f76e22aa6fc615f39fb69a8ee'),
    'small-tokens.txt': (816730, 'b34b360dbb493e781e479794586d661700670d65564001f23024971d1f2fa126'),
    'LICENSE.openai-whisper': (1063, 'b5d65a59060e68c4ff940e1eddfa6f94b2d68fdf58ed7f4dd57721c997e35e9d'),
    'LICENSE.sherpa-onnx': (11358, 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30'),
    'upstream-metadata.json': (1954, 'df6c7dfd905232336f1ddd010481ae77c1380c3a7bd9617be45b6162efb7d6ce'),
}
HUMAN = 'The main local beer is Number One, it is not a complex beer, but pleasant and refreshing. The other local beer is called Manta.'
PRIVATE = 'This is an English speech recognition test. There are five green cups beside the window.'


def words(text):
    # Preserve every script; never erase a non-English hallucination into a match.
    return ''.join(ch.casefold() if unicodedata.category(ch)[0] in 'LMN' else ' '
                   for ch in text).split()


def distance(expected, actual):
    row = list(range(len(actual) + 1))
    for i, left in enumerate(expected, 1):
        next_row = [i]
        for j, right in enumerate(actual, 1):
            next_row.append(min(row[j] + 1, next_row[-1] + 1, row[j - 1] + (left != right)))
        row = next_row
    return row[-1]


def assess(case, text):
    if not isinstance(text, str) or '\ufffd' in text:
        raise ValueError('damaged/non-string transcript')
    expected = words(HUMAN if case == 'human-en' else PRIVATE)
    actual = words(text)
    result = {'text': text, 'reference': HUMAN if case == 'human-en' else PRIVATE,
              'reference_words': expected, 'observed_words': actual,
              'word_errors': distance(expected, actual), 'reference_word_count': len(expected)}
    if case == 'human-en':
        result['pass'] = actual == expected  # Entire original reference, including Manta.
    else:
        numeric = [word for word in actual if word in {'zero', 'one', 'two', 'three', 'four', 'five',
                   'six', 'seven', 'eight', 'nine', 'ten', 'eleven', 'twelve', 'thirteen', 'fourteen',
                   'fifteen', 'sixteen', 'seventeen', 'eighteen', 'nineteen', 'twenty', 'thirty', 'forty',
                   'fifty', 'sixty', 'seventy', 'eighty', 'ninety', 'hundred', 'thousand', 'million'} or any(ch.isdigit() for ch in word)]
        result['numeric_words'] = numeric
        result['pass'] = numeric == ['five'] and any(actual[i:i + 3] == ['five', 'green', 'cups']
                                                   for i in range(len(actual) - 2))
    return result


def command(engine, assets, wav):
    return [str(engine), '--model-type=whisper',
            '--whisper-encoder=' + str(assets / 'small-encoder.int8.onnx'),
            '--whisper-decoder=' + str(assets / 'small-decoder.int8.onnx'),
            '--tokens=' + str(assets / 'small-tokens.txt'), '--whisper-task=transcribe',
            '--whisper-language=en', '--whisper-tail-paddings=300',
            '--num-threads=4', '--provider=cpu', str(wav)]


def finish_group(child, seconds):
    timed_out = False
    try:
        child.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        timed_out = True
    finally:
        # Every exit/error/signal path also fences descendants after parent exit.
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()
    return timed_out


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('engine', 'assets', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    args = p.parse_args()
    engine, assets, out = args.engine.resolve(), args.assets.resolve(), args.output.resolve()
    if guard.digest(engine) != 'bf482639a18895904336da88d3a5fab1ca94bee79235bd1bd4666363887185eb':
        raise ValueError('retained verified Sherpa2.0.1 binary required')
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
            raw = (out / (name + '.stdout')).read_text(encoding='utf-8', errors='strict')
            data = json.loads(raw)
            text = data.get('text')
            if not isinstance(text, str) or '\ufffd' in text:
                raise ValueError('invalid result text')
            row['text'] = text
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
        run('readiness', command(engine, assets, silence), max(0, ready_deadline - time.monotonic()), True)
        if time.monotonic() > ready_deadline:
            raise TimeoutError('readiness10s includes decode/readback')
        # Silence-generated words are retained as evidence but never treated as speech or delivered.
        for name, original in [('human-en', human), ('private-en', private)]:
            wav = out / (name + '.wav')
            decode(name + '-decode', original, wav)
            evidence.setdefault('decoded_sha256', {})[name] = guard.digest(wav)
            run(name, command(engine, assets, wav), json_result=True)
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
