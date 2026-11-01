"""No ASR: guard readiness context isolation and strict CPP result readback."""
import importlib.util
import json
from pathlib import Path
import tempfile

spec = importlib.util.spec_from_file_location('cpp344', Path(__file__).with_name('probe-cpp-english.py'))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
paths = [Path('/invented/' + name) for name in ('engine', 'assets', 'input.wav', 'output')]
ready = probe.command(*paths, readiness=True)
speech = probe.command(*paths)
assert ready[-2:] == ['--audio-ctx', '50']
assert '--audio-ctx' not in speech  # Critical: reduced silence context never reaches speech.
for argv in (ready, speech):
    for flag, value in [('--language', 'en'), ('--threads', '4'), ('--processors', '1')]:
        assert argv[argv.index(flag) + 1] == value
    assert '--no-gpu' in argv and '--no-fallback' in argv
    assert not any(text in ' '.join(argv) for text in (probe.shared.HUMAN, probe.shared.PRIVATE))
with tempfile.TemporaryDirectory(prefix='speech344-readback-') as temp:
    path = Path(temp) / 'result.json'
    for payload in (b'{"transcription":[{"text":"\xff"}]}', b'{"text":"stale"}',
                    b'{"transcription":[]}', b'{"transcription":[{"text":null}]}',
                    json.dumps({'transcription': [{'text': '\ufffd'}]}).encode()):
        path.write_bytes(payload)
        try:
            probe.read_result(path)
        except (ValueError, TypeError, KeyError):
            pass
        else:
            raise AssertionError('invalid transcript accepted')
    path.write_text('{"transcription":[]}')
    assert probe.read_result(path, readiness=True) == ''
    print('RED invalid UTF8/missing/empty/damaged result rejected')
    path.write_text(json.dumps({'transcription': [{'text': probe.shared.HUMAN}]}))
    assert probe.assess('human-en', probe.read_result(path))['pass']
    assert not probe.assess('human-en', probe.read_result(path).replace('Manta', 'Monta'))['pass']
print('GREEN fresh full-reference CPP readback and speech-context isolation; no inference')
