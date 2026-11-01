"""R5/§5: every real-board add is isolated before TaskQ can claim/create it."""
import os
import json
import pathlib
import re
import subprocess
import tempfile
import unittest


class HostIsolation(unittest.TestCase):
    def test_create_retry_and_cleanup_keep_unclaimed_host(self):
        source = pathlib.Path(__file__).with_name('automatic.rs').read_text()
        wrapper = re.search(r'const RECORD_TASKQ: &str = r#"(.*?)"#;', source, re.S)[1]
        with tempfile.TemporaryDirectory(prefix='pilotq-a40-host-') as tmp:
            base = pathlib.Path(tmp)
            record = base / 'record-taskq'
            record.write_text(wrapper)
            record.chmod(0o700)
            fake = base / 'taskq'
            fake.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
p = pathlib.Path(__file__).parent
with open(p / 'dispatched.jsonl', 'a') as f: f.write(json.dumps(sys.argv[1:]) + '\\n')
(p / 'resolved.json').write_text(json.dumps({'install': os.environ.get('TASKQ_INSTALL'), 'root': os.environ.get('TASKQ_ROOT')}))
print(json.dumps({'task':19,'op_id':'pilotq-fixture','outcome':'accepted','state':'held'}))
''')
            fake.chmod(0o700)
            (base / 'taskq-path').write_text(str(fake))
            host = 'pilotq-e2e-invented-fixture'
            (base / 'taskq-host').write_text(host)
            install = base / 'install'; install.mkdir()
            root = base / 'root'; root.mkdir()
            (base / 'taskq-scope.json').write_text(json.dumps({'install': str(install), 'root': str(root)}))
            native_bin = base / 'bin'; native_bin.mkdir()
            (native_bin / 'taskq').symlink_to(record)
            native = subprocess.run(['/bin/sh', '-c', 'taskq status'], env={'PATH': str(native_bin) + ':' + os.environ['PATH']}, cwd=base, capture_output=True)
            self.assertEqual(native.returncode, 0, native.stderr)
            self.assertEqual(json.loads((base / 'resolved.json').read_text()), {'install': str(install), 'root': str(root)})
            (base / 'dispatched.jsonl').unlink()
            original = ['add', 'A40 scripted lifecycle', '--op-id', 'pilotq-fixture', '--json']
            def run(args):
                return subprocess.run([str(record), *args], capture_output=True, env={k:v for k,v in os.environ.items() if not k.startswith('TASKQ_')}, cwd=root)
            self.assertEqual(run(original).returncode, 0)
            dispatched = json.loads((base / 'dispatched.jsonl').read_text().splitlines()[0])
            self.assertIn('--host', dispatched, 'unlabelled add can launch ordinary workers')
            self.assertEqual(dispatched[dispatched.index('--host') + 1], host)
            # Lost create response and cleanup both reconcile the same effective claim arguments.
            intents = [json.loads(s)['args'] for s in (base / 'taskq-calls.jsonl').read_text().splitlines()]
            self.assertEqual(intents[0], dispatched)
            self.assertEqual(run(original).returncode, 0)
            self.assertEqual(run(dispatched).returncode, 0)
            calls = [json.loads(s) for s in (base / 'dispatched.jsonl').read_text().splitlines()]
            self.assertEqual(calls, [dispatched] * 3)
            self.assertEqual(dispatched.count('--host'), 1)
            self.assertNotEqual(run(original + ['--host', 'ordinary-host']).returncode, 0)
            foreign = subprocess.run([str(record), 'service', 'pause'], env={**os.environ, 'TASKQ_INSTALL': '/invented/foreign'}, cwd=root, capture_output=True)
            self.assertNotEqual(foreign.returncode, 0)
            foreign = subprocess.run([str(record), 'service', 'pause'], env={**os.environ, 'TASKQ_ROOT': '/invented/foreign'}, cwd=root, capture_output=True)
            self.assertNotEqual(foreign.returncode, 0)
            other = base / 'foreign'; other.mkdir(); (other / 'taskq.json').write_text('{}')
            foreign = subprocess.run([str(record), 'service', 'pause'], env={'PATH': os.environ['PATH']}, cwd=other, capture_output=True)
            self.assertNotEqual(foreign.returncode, 0)
            probe = subprocess.run([str(record), '--a40-path-probe'], env={'PATH': os.environ['PATH']}, cwd=root, capture_output=True)
            self.assertEqual(json.loads(probe.stdout), {'install': str(install), 'root': str(root)})
            (base / 'taskq-host').unlink()
            self.assertNotEqual(run(original).returncode, 0)
            self.assertEqual(len((base / 'dispatched.jsonl').read_text().splitlines()), 3)


if __name__ == '__main__':
    unittest.main()
