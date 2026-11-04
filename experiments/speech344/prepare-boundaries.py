#!/usr/bin/env python3
"""PM-only instrumented baseline/patched builds; no model loading or installation."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import tarfile
import time

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('guard', HERE.parents[1] / 'scripts/probe-speech-344.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('archive', 'cmake', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--cmake-sha256', required=True, help='previous independently verified private CMake receipt')
    args = p.parse_args()
    identity = json.loads((HERE / 'manifest.json').read_text())
    if guard.digest(args.archive) != identity['archive_sha256'] or guard.digest(args.cmake) != args.cmake_sha256:
        raise ValueError('archive or CMake identity mismatch')
    for name in ('first-window.patch', 'instrumentation.patch', 'boundary.cpp'):
        if guard.digest(HERE / name) != identity['boundary_inputs_sha256'][name]:
            raise ValueError('boundary source identity mismatch')
    isolation = guard.limits()
    os.umask(0o077)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    proof = {'isolation': isolation, 'identity': identity, 'cmake_sha256': args.cmake_sha256, 'commands': [], 'builds': {}}
    deadline = time.monotonic() + 600

    def run(name, command, cwd):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('600s build deadline')
        with (out / (name + '.log')).open('wb') as log:
            start = time.monotonic()
            child = subprocess.Popen(command, cwd=cwd, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                child.wait(timeout=remaining)
            except BaseException:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                raise
        proof['commands'].append({'argv': command, 'seconds': time.monotonic() - start, 'exit': child.returncode})
        if child.returncode:
            raise RuntimeError('build failed; stop without retry')

    try:
        run('cmake-version', [str(args.cmake.resolve()), '--version'], out)
        run('compiler-version', ['c++', '--version'], out)
        for variant in ('baseline', 'patched'):
            parent = out / variant
            parent.mkdir()
            with tarfile.open(args.archive) as archive:
                archive.extractall(parent, filter='data')
            source = parent / ('whisper.cpp-' + identity['commit'])
            for name, sha in identity['source_sha256'].items():
                if guard.digest(source / name) != sha:
                    raise ValueError('extracted upstream source mismatch')
            patches = ['first-window.patch', 'instrumentation.patch'] if variant == 'patched' else ['instrumentation.patch']
            for patch in patches:
                run(variant + '-' + patch, ['git', 'apply', str(HERE / patch)], source)
            # Copy only the reviewed harness; both variants use identical source.
            (source / 'boundary-344.cpp').write_bytes((HERE / 'boundary.cpp').read_bytes())
            with (source / 'CMakeLists.txt').open('a') as cmake:
                cmake.write('\nadd_executable(boundary-344 boundary-344.cpp)\ntarget_link_libraries(boundary-344 PRIVATE whisper)\n')
            build = source / 'build-344-boundary'
            flags = '-DPILOTQ_344_PATCHED' if variant == 'patched' else ''
            run(variant + '-configure', [str(args.cmake.resolve()), '-S', str(source), '-B', str(build),
                '-DCMAKE_BUILD_TYPE=Release', '-DBUILD_SHARED_LIBS=OFF', '-DGGML_CUDA=OFF',
                '-DGGML_NATIVE=OFF', '-DCMAKE_CXX_FLAGS=' + flags], out)
            run(variant + '-build', [str(args.cmake.resolve()), '--build', str(build), '--target', 'boundary-344', '-j2'], out)
            binary = build / 'bin/boundary-344'
            proof['builds'][variant] = {'path': str(binary), 'sha256': guard.digest(binary),
                'cache_sha256': guard.digest(build / 'CMakeCache.txt'), 'license_sha256': guard.digest(source / 'LICENSE')}
        proof['status'] = 'build-only; no inference'
    except Exception as error:
        proof['failure'] = str(error)
        raise
    finally:
        (out / 'build-proof.json').write_text(json.dumps(proof, indent=2))


if __name__ == '__main__':
    main()
