#!/usr/bin/env python3
"""ONE PM-only turbo A/B diagnostic; no selection, installation or full-recording acceptance."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import re
import signal
import subprocess
import time
import wave

REPO = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('profile344', REPO / 'scripts/profile-speech-344.py')
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)
guard = profile.guard
CASES = {'faildetect', 'default', 'optout', 'offset', 'explicit', 'refuse', 'cancel', 'context', 'recovery', 'multi'}


def proofs(paths):
    seen, identities, hashes = set(), set(), {}
    for path in paths:
        proof = json.loads(path.read_text())
        case = proof['argv']['case']
        expected = ['multi', 'second'] if case == 'multi' else [case]
        if (case not in CASES or case in seen or
                proof.get('status') != 'boundary-pass-only; no performance or quality qualification' or
                proof.get('passed_boundaries') != expected or 'failure' in proof):
            raise ValueError('missing, duplicate or failed boundary proof')
        isolation = proof['isolation']
        if isolation['effective'] != {'memory.max': 1342177280, 'memory.high': 1073741824,
                                     'memory.swap.max': 268435456, 'pids.max': 256} or isolation['cpu_quota'] != '3/2':
            raise ValueError('boundary proof isolation mismatch')
        if case == 'cancel' and 'external_cancellation' not in proof:
            raise ValueError('missing external cancellation proof')
        identities.add((proof['argv']['baseline_sha256'], proof['argv']['patched_sha256']))
        seen.add(case)
        hashes[case] = guard.digest(path)
    if seen != CASES or len(identities) != 1:
        raise ValueError('incomplete boundary set or changed instrumented engines')
    return {'observation_sha256': hashes, 'instrumented_engine_sha256': list(next(iter(identities)))}


def transcript(prefix):
    data = json.loads(prefix.with_suffix('.json').read_text(encoding='utf-8', errors='strict'))
    text = ' '.join(part['text'] for part in data['transcription'])
    if not text.strip() or '\ufffd' in text:
        raise ValueError('empty or damaged transcript')
    return text, data['result']['language']


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('baseline', 'patched', 'model', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--boundary-proof', type=Path, action='append', required=True)
    args = p.parse_args()
    boundary = proofs(args.boundary_proof)
    for path, sha in [(args.baseline, '30ab87ffca69ba2e03f6c9c110c158cc5d7e4099eaa0179793559215b99aed70'),
                      (args.patched, 'a3e6f4ae6303793b89adfba1edbbc25c4a6477ea1428e67f92831612d6927d9a'),
                      (args.model, '394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2')]:
        if guard.digest(path) != sha:
            raise ValueError('verified original/patched CLI or turbo model required')
    private = REPO / 'target/pm344-speech-inputs/speech344-he.wav'
    if guard.digest(private) != '92f21b17e0abf706d826ce12646f20316dc4809be2816b5efb3b63b642d11643':
        raise ValueError('unchanged private Hebrew fixture required')
    isolation = guard.limits()
    os.umask(0o077)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    evidence = {'boundary': boundary, 'isolation': isolation, 'runs': [], 'status': 'incomplete'}
    deadline = time.monotonic() + 600  # decode, readiness and ALL A/B calls share it

    def save():
        (out / 'observations.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2))

    def run(name, command, cap=None):
        remaining = deadline - time.monotonic()
        if cap is not None:
            remaining = min(remaining, cap)
        if remaining <= 0:
            raise TimeoutError('shared 600s paired deadline expired')
        start = time.monotonic()
        with (out / (name + '.stdout')).open('wb') as stdout, (out / (name + '.stderr')).open('wb') as stderr:
            child = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=True)
            timeout = False
            try:
                child.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                timeout = True
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            except BaseException:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                raise
        log = (out / (name + '.stderr')).read_text(encoding='utf-8', errors='strict')
        row = {'name': name, 'argv': command, 'seconds': time.monotonic() - start,
               'exit': child.returncode, 'timeout': timeout,
               'peak_child_KiB_cumulative': resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,
               'native_timings': [line for line in log.splitlines() if line.startswith('whisper_print_timings:')],
               'reuse_count': log.count('experimental first-window reuse offset=0 audio_ctx=0')}
        encoded = re.search(r'encode time =\s*([0-9.]+) ms /\s*(\d+) runs', log)
        if encoded:
            row['encode_ms'] = float(encoded[1]); row['encoder_runs'] = int(encoded[2])
        evidence['runs'].append(row)
        save()
        if timeout or child.returncode:
            raise RuntimeError('process/deadline failure: stop without retry')
        return row

    def wav(path, pcm):
        with wave.open(str(path), 'wb') as stream:
            stream.setnchannels(1); stream.setsampwidth(2); stream.setframerate(16000)
            stream.writeframes(pcm)

    try:
        source = REPO / 'tests/fixtures/speech/he.ogg'
        if guard.digest(source) != '9341f1880cc0fc4b13e42bc724e0f5050f59a7f213d381df6cc2d9e533314ac0':
            raise ValueError('unchanged human Hebrew source required')
        human = out / 'human-he.wav'
        run('decode', ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(source), '-t', '301',
                       '-ac', '1', '-ar', '16000', str(human)])
        with wave.open(str(human), 'rb') as stream:
            if not 0 < stream.getnframes() <= 300 * 16000:
                raise ValueError('five-minute cap')
            pcm = stream.readframes(stream.getnframes())
        # Exact earlier long-recording construction, not a new quality fixture.
        repeated = (pcm * (300 * 32000 // len(pcm) + 1))[:300 * 32000]
        stress = out / 'stress-first25.wav'
        wav(stress, repeated[:25 * 32000])
        if guard.digest(stress) != '8ad655aece13cd8c53849e611579f825d4dfdd74ee2610bc8ed65d0d7a997f76':
            raise ValueError('stress segment differs from retained original')
        silence = out / 'silence.wav'
        wav(silence, bytes(32000))
        evidence['fixtures'] = {name: guard.digest(path) for name, path in
                                [('human-source', source), ('stress-first25', stress), ('private-he', private)]}
        for variant in ('baseline', 'patched'):
            prefix = out / ('readiness-' + variant)
            command = profile.argv(getattr(args, variant).resolve(), args.model.resolve(), [silence], [prefix])
            run(prefix.name, command + ['--audio-ctx', '50', '--language', 'en'], 10)
            json.loads(prefix.with_suffix('.json').read_text(encoding='utf-8', errors='strict'))
        comparisons = []
        # Preserve the known semantic guard before spending time on stress data.
        for case, audio in [('private-he', private), ('stress-first25', stress)]:
            pair, times, counts = [], [], []
            for variant in ('baseline', 'patched'):
                prefix = out / (case + '-' + variant)
                row = run(prefix.name, profile.argv(getattr(args, variant).resolve(), args.model.resolve(), [audio], [prefix]))
                text, language = transcript(prefix)
                row.update(text=text, language=language)
                if language != 'he' or 'encoder_runs' not in row:
                    raise ValueError('wrong observed language or missing native encoder count')
                pair.append((text, language)); times.append(row['seconds']); counts.append(row['encoder_runs'])
                save()
                if row['reuse_count'] != (variant == 'patched'):
                    raise ValueError('missing or unexpected reuse observation')
                if case == 'private-he' and (language != 'he' or any(word not in text for word in ['שלושה', 'ספרים', 'כחולים'])):
                    raise ValueError('retained three-blue-books semantic guard failed')
            if counts[0] - counts[1] != 1:
                raise ValueError('native counters do not prove one encoder pass removed')
            comparison = {'case': case, 'same_text_and_language': pair[0] == pair[1],
                          'baseline_seconds': times[0], 'patched_seconds': times[1],
                          'saved_seconds': times[0] - times[1], 'ratio': times[1] / times[0]}
            comparisons.append(comparison)
            evidence['comparison'] = comparisons
            save()
            if pair[0] != pair[1]:
                raise ValueError('text/language mismatch; stop before more inference')
            if times[1] >= times[0]:
                raise ValueError('no measured saving; stop experiment without retry')
        evidence['status'] = 'paired-diagnostic-only; full300/all-language/quality acceptance OPEN'
    except Exception as error:
        evidence['failure'] = str(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
