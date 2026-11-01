"""No ASR: protect full-reference/numeric evidence and actual subprocess-group cleanup."""
import importlib.util
import os
from pathlib import Path
import signal
import subprocess
import sys

spec = importlib.util.spec_from_file_location('english344', Path(__file__).with_name('probe-sherpa-english.py'))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
for name, text in [('human-en', probe.HUMAN.rsplit('Manta', 1)[0]),
                   ('human-en', probe.HUMAN + ' неправильно'),
                   ('private-en', probe.PRIVATE.replace('five', 'six')),
                   ('private-en', probe.PRIVATE + ' There are six more.'),
                   ('private-en', probe.PRIVATE.replace('green', 'yellow'))]:
    assert not probe.assess(name, text)['pass'], (name, text)
    print('RED rejected incomplete/contradictory reference observation')
try:
    probe.assess('human-en', probe.HUMAN + '\ufffd')
except ValueError:
    print('RED rejected damaged transcript')
else:
    raise AssertionError('damaged UTF-8 observation accepted')
assert probe.assess('human-en', probe.HUMAN.upper())['pass']
assert probe.assess('private-en', probe.PRIVATE)['pass']
assert probe.assess('human-en', probe.HUMAN)['word_errors'] == 0
argv = probe.command(Path('/invented/engine'), Path('/invented/assets'), Path('/invented/input.wav'))
assert not any(reference in ' '.join(argv) for reference in (probe.HUMAN, probe.PRIVATE))
assert '--whisper-tail-paddings=300' in argv and '--whisper-language=en' in argv
print('GREEN complete human reference and numeric meaning PASS; no prompt payload')

# This runs only tiny owned Python processes, never an ASR executable/model.
# Prove both timeout cleanup and descendants surviving a normally exited parent.
for mode in ('timeout', 'parent-exit'):
    code = "import subprocess,sys,time; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); print(p.pid,flush=True); "
    code += 'time.sleep(30)' if mode == 'timeout' else 'sys.exit(0)'
    child = subprocess.Popen([sys.executable, '-c', code], stdout=subprocess.PIPE, start_new_session=True, text=True)
    try:
        # Bounded readiness for the synthetic lifecycle fixture, not ASR readiness.
        import select
        if not select.select([child.stdout], [], [], 5)[0]:
            raise RuntimeError('synthetic child readiness blocked')
        descendant = int(child.stdout.readline())
        timed_out = probe.finish_group(child, 0.1 if mode == 'timeout' else 5)
        assert timed_out == (mode == 'timeout')
        stat = Path('/proc') / str(descendant) / 'stat'
        # A killed orphan may briefly remain a zombie pending its subreaper; it cannot execute.
        def still_running(path):
            try:
                return path.read_text().split(') ', 1)[1][0] != 'Z'
            except (FileNotFoundError, ProcessLookupError):
                # The process can vanish between locating /proc/PID and reading stat.
                return False
        import time
        until = time.monotonic() + 2
        while still_running(stat) and time.monotonic() < until:
            time.sleep(0.01)
        assert not still_running(stat)
        print('GREEN owned process group terminated:', mode)
    finally:
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()
        child.stdout.close()
