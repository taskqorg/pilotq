"""No inference: protect contiguous construction, offsets and honest partial-tail evidence."""
import importlib.util
from pathlib import Path
spec = importlib.util.spec_from_file_location('long344', Path(__file__).with_name('probe-cpp-english-long.py'))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
source = b'\x01\x02\x03\x04\x05\x06'
result, cycles, tail = probe.repeat_pcm(source, seconds=1)
assert result == (source * (32000 // len(source) + 1))[:32000]
assert cycles == 5333 and tail == 1 and len(result) == 32000
assert b'\x00' not in result  # No invented silence padding.
for bad in (b'', b'\x01'):
    try:
        probe.repeat_pcm(bad)
    except ValueError:
        pass
    else:
        raise AssertionError('invalid source accepted')
print('RED empty/unaligned PCM refused; GREEN contiguous repetition and exact tail')
reference = probe.cpp.shared.HUMAN
partial = probe.fidelity(reference, 1, 100)
assert partial['quality_pass'] is None and partial['quality_status'].startswith('unknown')
assert len(partial['tail_prefix_error_observations']) == 25
assert min(x['word_errors'] for x in partial['tail_prefix_error_observations']) == 0
assert probe.fidelity(reference, 1, 0)['quality_pass'] is True
assert probe.fidelity(reference.replace('Manta', 'Monta'), 1, 0)['quality_pass'] is False
assert probe.fidelity('', 1, 0)['quality_pass'] is False
print('RED name/empty mismatch refused; GREEN partial tail never promoted to quality PASS')
# True25s windows are independent of source repetition boundaries.
size = 25 * probe.RATE * 2
assert [(start // 2, min(size, 300 * probe.RATE * 2 - start) // 2)
        for start in range(0, 300 * probe.RATE * 2, size)] == [(i * 400000, 400000) for i in range(12)]
print('GREEN twelve contiguous25s windows cover exactly300s; no ASR')
