#!/usr/bin/env python3
"""Reviewed private no-ensurepip setup + static inspector; never graph execution."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import sysconfig
import time
import urllib.parse
import urllib.request

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[1]
LOCKS = {
    'onnx-inspector-bootstrap.json': '961beeba32d38d5188d9dd1e95afb9c94c4b5f9bf401a61a3f106b7bc4b746f6',
    'onnx-inspector-wheels.json': 'f38e9bdb8fae81c77c18568a13a30f53d8d5934257e0f773b1f8e04bdb03f0c9',
    'onnx-inspector-requirements.txt': 'bebe1372c5eb1e2539e5b25b0e5d471922cda68e67f45cbfc1b62c927a58424c',
}
BOOTSTRAP = 'import runpy,sys; sys.path.insert(0,sys.argv.pop(1)); runpy.run_module("pip",run_name="__main__")'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--assets', type=Path, required=True)
    args = parser.parse_args()
    if sys.version_info[:2] != (3, 14) or sysconfig.get_config_var('Py_GIL_DISABLED'):
        raise ValueError('standard-GIL CPython3.14 required for exact wheels')
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise ValueError('Linux x86_64 required')
    spec = importlib.util.spec_from_file_location('cpp344', BASE / 'probe-cpp-english.py')
    cpp = importlib.util.module_from_spec(spec); spec.loader.exec_module(cpp)
    isolation = cpp.guard.limits()
    started = time.monotonic(); deadline = started + 600
    root = args.root.resolve(); assets = args.assets.resolve()
    if not root.is_relative_to((REPO / 'target').resolve()):
        raise ValueError('new private project target root required')
    for name, digest in LOCKS.items():
        if cpp.guard.digest(BASE / name) != digest:
            raise ValueError('tool lock mismatch: ' + name)
    # Verify candidate bytes without importing ONNX, before private setup starts.
    inspect_spec = importlib.util.spec_from_file_location('static344', BASE / 'inspect-giga-multilingual.py')
    inspector = importlib.util.module_from_spec(inspect_spec); inspect_spec.loader.exec_module(inspector)
    for name, (size, digest) in inspector.EXPECTED.items():
        path = assets / name
        if path.stat().st_size != size or cpp.guard.digest(path) != digest:
            raise ValueError('candidate identity mismatch: ' + name)
    os.umask(0o077)
    root.mkdir(mode=0o700, parents=True, exist_ok=False)  # Never repair/reuse failed v1.
    wheelhouse = root / 'wheels'; wheelhouse.mkdir(mode=0o700)
    evidence = {'status': 'incomplete', 'isolation': isolation, 'runs': [], 'wheels': [],
                'base_python': sys.version, 'base_executable': sys.executable,
                'model_executed': False, 'script_sha256': cpp.guard.digest(Path(__file__)),
                'inspector_sha256': cpp.guard.digest(BASE / 'inspect-giga-multilingual.py'),
                'cleanup': 'private setup retained for PM readback; independent host cleanup required'}

    def save():
        evidence['seconds'] = time.monotonic() - started
        (root / 'setup-result.json').write_text(json.dumps(evidence, indent=2))

    def remaining():
        value = deadline - time.monotonic()
        if value <= 0:
            raise TimeoutError('combined setup/static600s')
        return value

    def acquire(entry):
        url = entry['url']; parts = urllib.parse.urlsplit(url)
        if parts.scheme != 'https' or parts.hostname != 'files.pythonhosted.org':
            raise ValueError('official pinned PyPI file origin required')
        name = entry['filename']
        if Path(name).name != name:
            raise ValueError('invalid wheel basename')
        part = wheelhouse / (name + '.part'); count = 0
        with urllib.request.urlopen(url, timeout=min(30, remaining())) as response, part.open('xb') as stream:
            if response.geturl() != url:
                raise ValueError('unexpected wheel redirect')
            while True:
                remaining()
                data = response.read(1024 * 1024)
                if not data:
                    break
                count += len(data)
                if count > entry['size']:
                    raise ValueError('wheel exceeds pinned size')
                stream.write(data)
        if count != entry['size'] or cpp.guard.digest(part) != entry['sha256']:
            raise ValueError('wheel identity mismatch: ' + name)
        target = wheelhouse / name; part.rename(target)
        evidence['wheels'].append({'filename': name, 'size': count, 'sha256': cpp.guard.digest(target)})
        save(); remaining()
        return target

    def run(name, argv):
        remaining()  # Refuse before creating a child; never raise a budget error while leaving it unfenced.
        start = time.monotonic()
        with (root / (name + '.stdout')).open('xb') as stdout, (root / (name + '.stderr')).open('xb') as stderr:
            child = subprocess.Popen(argv, stdout=stdout, stderr=stderr, start_new_session=True)
            timeout = cpp.finish_group(child, max(0, deadline - time.monotonic()))
        evidence['runs'].append({'name': name, 'argv': argv, 'pid': child.pid, 'pgid': child.pid,
                                 'reaped': child.poll() is not None, 'exit': child.returncode,
                                 'timeout': timeout, 'seconds': time.monotonic() - start})
        save()
        if timeout or child.returncode:
            raise ValueError(name + ': failed; no retry')
        remaining()

    try:
        # No ensurepip/get-pip/system pip/global install. Execute only verified pip zip code.
        bootstrap = json.loads((BASE / 'onnx-inspector-bootstrap.json').read_text())
        pip_wheel = acquire(bootstrap)
        for wheel in json.loads((BASE / 'onnx-inspector-wheels.json').read_text())['packages']:
            acquire(wheel)
        venv = root / 'venv'
        run('venv', [sys.executable, '-I', '-m', 'venv', '--without-pip', str(venv)])
        python = str(venv / 'bin/python')
        pip = [python, '-I', '-c', BOOTSTRAP, str(pip_wheel), '--isolated',
               '--disable-pip-version-check', '--no-cache-dir']
        run('pip-version', pip + ['--version'])
        run('install', pip + ['install', '--no-index', '--find-links', str(wheelhouse),
                             '--only-binary=:all:', '--require-hashes', '--no-deps',
                             '-r', str(BASE / 'onnx-inspector-requirements.txt')])
        run('dependency-check', pip + ['check'])
        # Same absolute monotonic deadline, not a fresh600 seconds after setup.
        run('static-inspect', [python, str(BASE / 'inspect-giga-multilingual.py'),
                              '--assets', str(assets), '--output', str(root / 'inspection'),
                              '--deadline-monotonic', str(deadline)])
        evidence['status'] = 'private setup/static completed only; no compatibility/quality acceptance'
    except BaseException as error:
        evidence['failure'] = str(error)
        raise
    finally:
        save()  # Retain exact failure/partial venv; no generic cleanup of prior evidence.


if __name__ == '__main__':
    def terminate(signum, frame):
        raise SystemExit('setup terminated; child group fenced; PM verifies host cleanup')
    signal.signal(signal.SIGTERM, terminate)
    main()
