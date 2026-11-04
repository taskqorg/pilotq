#!/usr/bin/env python3
"""PM-reviewed candidate only: contiguous English stress/controls, no installed writes."""
import argparse
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import time
import wave

spec = importlib.util.spec_from_file_location('cpp344', Path(__file__).with_name('probe-cpp-english.py'))
cpp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cpp)
RATE = 16000


def pcm(path):
    with wave.open(str(path), 'rb') as audio:
        if (audio.getnchannels(), audio.getsampwidth(), audio.getframerate()) != (1, 2, RATE):
            raise ValueError('PCM format mismatch')
        return audio.readframes(audio.getnframes())


def write_pcm(path, data):
    with wave.open(str(path), 'wb') as audio:
        audio.setnchannels(1); audio.setsampwidth(2); audio.setframerate(RATE)
        audio.writeframes(data)


def repeat_pcm(source, seconds=300):
    if not source or len(source) % 2:
        raise ValueError('invalid source PCM')
    size = seconds * RATE * 2
    cycles, tail = divmod(size, len(source))
    return source * cycles + source[:tail], cycles, tail // 2


def fidelity(text, cycles, tail_samples):
    reference = cpp.shared.words(cpp.shared.HUMAN)
    actual = cpp.shared.words(text)
    complete = reference * cycles
    # No forced alignment exists for the terminal partial phrase. Keep every possible
    # prefix error count, never select a convenient prefix as an accepted reference.
    errors = [{'tail_reference_words': n,
               'word_errors': cpp.shared.distance(complete + reference[:n], actual)}
              for n in (range(len(reference) + 1) if tail_samples else [0])]
    return {'text': text, 'observed_words': actual, 'complete_source_repetitions': cycles,
            'complete_reference_words': complete, 'tail_source_samples': tail_samples,
            'tail_prefix_error_observations': errors,
            'quality_status': 'unknown: terminal partial phrase lacks timed word alignment' if tail_samples else 'exact reference available',
            'quality_pass': None if tail_samples else actual == complete}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('engine', 'assets', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    engine, assets, out = args.engine.resolve(), args.assets.resolve(), args.output.resolve()
    for name, (size, sha) in cpp.ASSETS.items():
        path = assets / name
        if path.stat().st_size != size or cpp.guard.digest(path) != sha:
            raise ValueError('asset mismatch: ' + name)
    if engine != (assets / 'whisper-cli').resolve():
        raise ValueError('original retained engine required')
    human = cpp.REPO / 'tests/fixtures/speech/en.ogg'
    if cpp.guard.digest(human) != '23860dba7d1d072f39b9244e31ee151c89f1192537bd587bb37baeb371f6430f':
        raise ValueError('human fixture mismatch')
    isolation = cpp.guard.limits()
    os.umask(0o077)
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    work = out / 'work'; work.mkdir(mode=0o700)
    deadline = time.monotonic() + 600
    evidence = {'status': 'incomplete', 'isolation': isolation, 'runs': [], 'delivered': False,
                'engine_sha256': cpp.guard.digest(engine), 'fixture_sha256': cpp.guard.digest(human),
                'assets': cpp.ASSETS, 'script_sha256': cpp.guard.digest(Path(__file__)),
                'cleanup': 'pending', 'host_cleanup': 'unknown: independent PM readback required'}

    def save():
        (out / 'observations.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2))

    def check_time():
        if time.monotonic() >= deadline:
            raise TimeoutError('shared600s including decode/readback/construction')

    def run(name, argv, until=None, expected_failure=False, cancel=False):
        check_time()
        remaining = min(deadline, until if until is not None else deadline) - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(name + ': deadline before dispatch')
        started = time.monotonic()
        with (out / (name + '.stdout')).open('xb') as stdout, (out / (name + '.stderr')).open('xb') as stderr:
            child = subprocess.Popen(argv, stdout=stdout, stderr=stderr, start_new_session=True)
            # Cancellation is deliberate, bounded and never a delivered/quality result.
            timed_out = cpp.finish_group(child, min(0.1, remaining) if cancel else remaining)
        row = {'name': name, 'argv': argv, 'pid': child.pid, 'pgid': child.pid,
               'reaped': child.poll() is not None, 'seconds': time.monotonic() - started,
               'exit': child.returncode, 'timeout': timed_out, 'delivered': False,
               'peak_child_KiB_cumulative': resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss}
        evidence['runs'].append(row); save()
        if cancel:
            if not timed_out or child.returncode != -signal.SIGKILL:
                raise ValueError('cancel control did not observe a killed/reaped engine')
            row['outcome'] = 'cancelled; no transcript read or delivery'
        elif timed_out or (expected_failure and child.returncode == 0) or (not expected_failure and child.returncode != 0):
            raise ValueError(name + ': unexpected process/deadline outcome')
        check_time()
        return row

    def decode(name, source, target, until=None, corrupt=False):
        run(name, ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(source), '-t', '301',
                   '-ac', '1', '-ar', '16000', str(target)], until, expected_failure=corrupt)
        if corrupt:
            return None  # The only dispatch here was ffmpeg, never ASR.
        data = pcm(target)
        if not data or len(data) > 300 * RATE * 2:
            raise ValueError('five-minute recording cap')
        check_time()
        return data

    def recognize(name, wav, silence=False, until=None):
        prefix = out / name
        row = run(name, cpp.command(engine, assets, wav, prefix, readiness=silence), until)
        text = cpp.read_result(prefix.with_suffix('.json'), readiness=silence)
        row['text'] = text; row['context'] = 50 if silence else 'full default'
        if silence:
            row['delivered_text'] = ''  # Retain hallucination only as private nondelivered evidence.
        elif not cpp.shared.words(text):
            raise ValueError('empty speech result')
        save(); check_time()
        return text

    try:
        for name in ('readiness', 'silence'):
            until = min(deadline, time.monotonic() + 10)
            source = work / (name + '-source.wav'); write_pcm(source, bytes(RATE * 2))
            wav = work / (name + '.wav')
            data = decode(name + '-decode', source, wav, until)
            if any(data):
                raise ValueError('silence control not zero PCM')
            recognize(name, wav, silence=True, until=until)
            if time.monotonic() > until:
                raise TimeoutError(name + '10s includes decode/readback')
        corrupt = work / 'corrupt'; corrupt.write_bytes(b'not an audio container: invented fixture')
        decode('corrupt-decode', corrupt, work / 'corrupt.wav', min(deadline, time.monotonic() + 10), corrupt=True)
        evidence['corrupt'] = 'expected decoder refusal before ASR'
        too_long = work / '301.wav'; write_pcm(too_long, bytes(301 * RATE * 2))
        try:
            decode('301-decode', too_long, work / '301-decoded.wav', min(deadline, time.monotonic() + 10))
        except ValueError as error:
            if str(error) != 'five-minute recording cap':
                raise
        else:
            raise ValueError('301s accepted')
        evidence['301'] = 'expected duration refusal before ASR'; save()
        source_wav = work / 'human.wav'
        source = decode('human-decode', human, source_wav)
        run('cancel', cpp.command(engine, assets, source_wav, out / 'cancel'), cancel=True)
        # Plain contiguous repetition and a final source-prefix cut: no silence padding,
        # stretching, block alignment or expected words passed to the recognizer.
        repeated, cycles, tail = repeat_pcm(source)
        stress = work / 'contiguous300.wav'; write_pcm(stress, repeated)
        evidence['construction'] = {'method': '(unchanged decoded PCM repeated)[:300*16000*2]',
                                    'source_samples': len(source) // 2, 'source_pcm_sha256': hashlib.sha256(source).hexdigest(),
                                    'complete_repetitions': cycles, 'final_partial_source_samples': tail,
                                    'stress_sha256': cpp.guard.digest(stress), 'source_padding': False}
        decoded = decode('stress-decode', stress, work / 'stress-decoded.wav')
        if decoded != repeated or len(decoded) != 300 * RATE * 2:
            raise ValueError('stress PCM changed or not exactly300s')
        texts = []
        evidence['segments'] = []
        for n, start in enumerate(range(0, len(decoded), 25 * RATE * 2)):
            check_time()
            segment = work / ('segment-' + str(n) + '.wav')
            part = decoded[start:start + 25 * RATE * 2]; write_pcm(segment, part)
            evidence['segments'].append({'index': n, 'start_sample': start // 2,
                                         'samples': len(part) // 2, 'sha256': cpp.guard.digest(segment),
                                         'source_offset_sample': (start // 2) % (len(source) // 2)})
            save()
            texts.append(recognize('segment-' + str(n), segment))
            evidence['ordered_partial_text'] = ' '.join(texts); save()
        evidence['fidelity'] = fidelity(' '.join(texts), cycles, tail)
        check_time()
        if evidence['fidelity']['quality_pass'] is False:
            raise ValueError('unambiguous full reference mismatch')
        evidence['status'] = 'budget stress completed; fidelity unknown if partial-tail oracle; no installed/live qualification'
    except BaseException as error:
        evidence['failure'] = str(error)
        raise
    finally:
        # Only this run's generated work is removed; evidence, assets and source stay.
        shutil.rmtree(work)
        evidence['cleanup'] = {'generated_work_removed': not work.exists(), 'elapsed_seconds': 600 - (deadline - time.monotonic())}
        save()


if __name__ == '__main__':
    def terminate(signum, frame):
        raise SystemExit('terminated; child group fenced; PM must verify host cleanup')
    signal.signal(signal.SIGTERM, terminate)
    main()
