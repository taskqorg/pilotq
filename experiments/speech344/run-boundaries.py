#!/usr/bin/env python3
"""PM-only boundary investigation. Requires independently recorded instrumented build hashes."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time

REPO = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('guard', REPO / 'scripts/probe-speech-344.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


def rows(text):
    result = []
    for line in text.splitlines():
        if line.startswith('RESULT\t'):
            _, label, rc, language, encodes, begins, aborts, encoded = line.split('\t')
            transcript = bytes.fromhex(encoded).decode('utf-8', errors='strict')
            if '\ufffd' in transcript:
                raise ValueError('replacement character in transcript')
            result.append(dict(label=label, rc=int(rc), language=int(language),
                               encodes=int(encodes), begins=int(begins), aborts=int(aborts), text=transcript))
    if not result:
        raise ValueError('missing API observation')
    return result


def compare(case, baseline, patched, baseline_log, patched_log):
    expected = {'recovery': [1, 0, 1], 'optout': [0], 'faildetect': [0], 'default': [1], 'offset': [0], 'explicit': [0],
                'refuse': [0], 'cancel': [1], 'context': [0, 0, 1], 'multi': [1, 1, 1], 'second': [1]}[case]
    if len(baseline) != len(expected) or len(patched) != len(expected):
        raise ValueError('wrong observation count')
    for a, b, saved in zip(baseline, patched, expected, strict=True):
        for field in ['label', 'rc', 'language', 'begins', 'text']:
            if a[field] != b[field]:
                raise ValueError(f'{case}: {field} mismatch')
        if a['encodes'] - b['encodes'] != saved:
            raise ValueError(f'{case}: encoder count mismatch')
        if a['label'] in ('cancel', 'faildetect') and (a['rc'] >= 0 or a['text']):
            raise ValueError('failure/cancellation returned success or text')
        if case == 'refuse' and (a['text'] or a['begins'] != 1):
            raise ValueError('refused encoder emitted text or missed callback')
        if a['label'] not in ('cancel', 'faildetect', 'refuse') and a['rc'] != 0:
            raise ValueError('ordinary boundary returned failure')
        if case == 'cancel' and (a['begins'] != 1 or not a['aborts'] or not b['aborts']):
            raise ValueError('abort callback not exercised')
    resets_a = baseline_log.count('PILOTQ344 decoder-reset')
    resets_b = patched_log.count('PILOTQ344 decoder-reset')
    if resets_a != resets_b:
        raise ValueError('decoder reset mismatch')
    if case not in ('cancel', 'faildetect', 'refuse') and not resets_a:
        raise ValueError('decoder reset not observed')
    if baseline_log.count('PILOTQ344 encode offset=') - patched_log.count('PILOTQ344 encode offset=') != sum(expected):
        raise ValueError('encoder trace/count mismatch')
    # Every reuse must be observed at the exact guarded offset/context.
    if patched_log.count('experimental first-window reuse offset=0 audio_ctx=0') != sum(expected) - (case == 'cancel'):
        raise ValueError('reuse trace mismatch')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('baseline', 'patched', 'model', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    for name in ('baseline-sha256', 'patched-sha256'):
        p.add_argument('--' + name, required=True)
    p.add_argument('--case', required=True, choices=['faildetect', 'default', 'optout', 'offset', 'explicit', 'refuse', 'cancel', 'context', 'recovery', 'multi'])
    args = p.parse_args()
    isolation = guard.limits()
    for path, expected in [(args.baseline, args.baseline_sha256), (args.patched, args.patched_sha256),
                           (args.model, 'ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb')]:
        if len(expected) != 64 or guard.digest(path) != expected:
            raise ValueError('independently recorded build/model hash mismatch')
    os.umask(0o077)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    deadline = time.monotonic() + 600
    evidence = {'isolation': isolation, 'argv': vars(args).copy(), 'runs': [], 'status': 'incomplete'}
    evidence['argv'] = {k: str(v) for k, v in evidence['argv'].items()}

    def save():
        (out / 'observations.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2))

    def run(name, command, cap=None, cancel_at_encoder=False):
        remaining = deadline - time.monotonic()
        if cap is not None:
            remaining = min(remaining, cap)
        if remaining <= 0:
            raise TimeoutError('shared 600s boundary deadline')
        start = time.monotonic()
        with (out / (name + '.stdout')).open('wb') as stdout, (out / (name + '.stderr')).open('wb') as stderr:
            child = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=True)
            timeout = False
            try:
                if cancel_at_encoder:
                    until = time.monotonic() + remaining
                    while 'PILOTQ344 encode offset=' not in (out / (name + '.stderr')).read_text():
                        if child.poll() is not None:
                            raise RuntimeError('child exited before external cancellation')
                        if time.monotonic() >= until:
                            raise subprocess.TimeoutExpired(command, remaining)
                        time.sleep(0.02)
                    os.killpg(child.pid, signal.SIGTERM)
                    child.wait(timeout=min(5, max(0.001, until - time.monotonic())))
                else:
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
        evidence['runs'].append({'name': name, 'argv': command, 'seconds': time.monotonic() - start,
                                 'exit': child.returncode, 'timeout': timeout})
        save()
        if timeout or (child.returncode != -signal.SIGTERM if cancel_at_encoder else child.returncode != 0):
            raise RuntimeError('boundary process/deadline failure; stop without retry')
        return (out / (name + '.stdout')).read_text(encoding='utf-8'), (out / (name + '.stderr')).read_text(encoding='utf-8')

    try:
        pcms = []
        for language in ('ru', 'en'):
            source = REPO / f'tests/fixtures/speech/{language}.ogg'
            pcm = out / (language + '.f32')
            run('decode-' + language, ['ffmpeg', '-nostdin', '-v', 'error', '-i', str(source),
                                      '-t', '301', '-ac', '1', '-ar', '16000', '-f', 'f32le', str(pcm)])
            if not 0 < pcm.stat().st_size <= 25 * 16000 * 4:
                raise ValueError('boundary fixture must fit one full 25s chunk; no truncation')
            evidence.setdefault('fixtures', {})[language] = {'source_sha256': guard.digest(source), 'pcm_sha256': guard.digest(pcm)}
            pcms.append(str(pcm))
        # Real process cancellation during encoder work, before normal API results.
        for variant in (('baseline', 'patched') if args.case == 'cancel' else ()):
            command = [str(getattr(args, variant).resolve()), str(args.model.resolve()), *pcms, 'default']
            stdout, _ = run('external-cancel-' + variant, command, 10, True)
            if 'RESULT\t' in stdout:
                raise ValueError('cancelled process emitted completed API result')
        if args.case == 'cancel':
            evidence['external_cancellation'] = 'SIGTERM during observed encoder; both children reaped'
        results = {}
        for case in (['multi', 'second'] if args.case == 'multi' else [args.case]):
            pair, logs = [], []
            for variant in ('baseline', 'patched'):
                command = [str(getattr(args, variant).resolve()), str(args.model.resolve()), *pcms, case]
                stdout, stderr = run(case + '-' + variant, command, 10 if case == 'faildetect' else None)
                pair.append(rows(stdout)); logs.append(stderr)
            compare(case, *pair, *logs)
            results[case] = pair
            evidence.setdefault('passed_boundaries', []).append(case)
            save()
        # Same-state second file must equal an independent invocation, each variant.
        for index in ((0, 1) if args.case == 'multi' else ()):
            multi, single = results['multi'][index][2], results['second'][index][0]
            for field in ('rc', 'language', 'text', 'encodes', 'begins'):
                if multi[field] != single[field]:
                    raise ValueError('multifile state leak: ' + field)
        evidence['status'] = 'boundary-pass-only; no performance or quality qualification'
    except Exception as error:
        evidence['failure'] = str(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
