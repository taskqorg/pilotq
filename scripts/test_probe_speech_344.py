"""#344 / §9: refuse ASR when real effective isolation cannot be established.

Temporary cgroup fixtures check guard logic only; they never qualify host limits.
"""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('probe', Path(__file__).with_name('probe-speech-344.py'))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class IsolationGuard(unittest.TestCase):
    def test_effective_ancestor_caps_and_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            mount = root / 'cgroup'
            parent = mount / 'bounded.service'
            leaf = parent / 'child'
            leaf.mkdir(parents=True)
            proc = root / 'membership'
            proc.write_text('0::/bounded.service/child\n')
            expected = {'memory.max': '1342177280', 'memory.high': '1073741824',
                        'memory.swap.max': '268435456', 'pids.max': '256', 'cpu.max': '150000 100000'}
            for name, value in expected.items():
                (parent / name).write_text(value)
                (leaf / name).write_text('max 100000' if name == 'cpu.max' else 'max')
            self.assertEqual(probe.limits(proc, mount)['cpu_quota'], '3/2')
            self.assertEqual(probe.limits(proc, mount)['unit'], 'bounded.service')
            for name, value in [('memory.max', '2684354560'), ('cpu.max', '200000 100000')]:
                with self.subTest(unbounded=name):
                    (parent / name).write_text(value)
                    with self.assertRaises(ValueError):
                        probe.limits(proc, mount)
                    (parent / name).write_text(expected[name])
            (leaf / 'memory.max').unlink()
            with self.assertRaises(FileNotFoundError):
                probe.limits(proc, mount)


if __name__ == '__main__':
    unittest.main()
