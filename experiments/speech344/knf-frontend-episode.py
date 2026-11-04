#!/usr/bin/env python3
"""PM-only combined KNF feature proof. Preparation is not execution approval."""
import argparse
import hashlib
import importlib.metadata
import importlib.util
import io
import json
import math
import os
from pathlib import Path, PurePosixPath
import resource
import signal
import stat
import subprocess
import tarfile
import time
import zipfile

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
ARCHIVE_LIMIT = 32 * 1024**2
EXPANDED_LIMIT = 128 * 1024**2


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def member_path(name, prefix):
    p = PurePosixPath(name)
    if p.is_absolute() or '\\' in name or '..' in p.parts or not p.parts or p.parts[0] != prefix:
        raise ValueError('unsafe archive path')
    return Path(*p.parts[1:])


def unpack_verified(archive, entry, target, license_dir, remaining):
    """Digest + all license checks before writing any extracted source bytes."""
    if archive.stat().st_size > ARCHIVE_LIMIT or sha(archive) != entry['sha256']:
        raise ValueError('archive identity mismatch')
    remaining()
    if entry['format'] == 'tar.gz':
        container = tarfile.open(archive, 'r:gz')
        members = container.getmembers()
        records = [(m.name, m.size, m.isdir(), m.isfile() or m.isdir(), m) for m in members]
        def read(m):
            return container.extractfile(m).read()
    elif entry['format'] == 'zip':
        container = zipfile.ZipFile(archive)
        records = []
        for m in container.infolist():
            mode = m.external_attr >> 16
            regular = stat.S_IFMT(mode) in (0, stat.S_IFREG, stat.S_IFDIR)
            records.append((m.filename, m.file_size, m.is_dir(), regular and not (m.flag_bits & 1), m))
        def read(m):
            return container.read(m)
    else:
        raise ValueError('unreviewed archive format')
    with container:
        if len(records) > 10000:
            raise ValueError('archive member cap')
        files, seen, total = {}, set(), 0
        for name, size, directory, regular, m in records:
            remaining()
            rel = member_path(name, entry['prefix'])
            if not rel.parts:
                if not directory:
                    raise ValueError('invalid archive root')
                continue
            if rel in seen or not regular or size < 0 or size > ARCHIVE_LIMIT:
                raise ValueError('duplicate/link/special/oversized member')
            seen.add(rel)
            total += size
            if total > EXPANDED_LIMIT:
                raise ValueError('archive expansion cap')
            if not directory:
                files[rel.as_posix()] = (rel, size, m)
        licenses = {}
        for name, expected in entry['licenses'].items():
            remaining()
            if name not in files:
                raise ValueError('missing license chain')
            b = read(files[name][2])
            actual = hashlib.sha256(b).hexdigest()
            if actual != expected:
                raise ValueError('license identity mismatch')
            licenses[name] = b
        # Only after complete validation; no extractall, links, mode bits or archive scripts.
        target.mkdir(mode=0o700, exist_ok=False)
        license_dir.mkdir(mode=0o700, exist_ok=False)
        for name, b in licenses.items():
            dest = license_dir / name
            dest.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            dest.write_bytes(b)
        inventory = {}
        for name, (rel, size, m) in files.items():
            remaining()
            b = read(m)
            if len(b) != size:
                raise ValueError('archive member length mismatch')
            dest = target / rel
            dest.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            with dest.open('xb') as stream:
                stream.write(b)
            inventory[name] = {'bytes': size, 'sha256': hashlib.sha256(b).hexdigest()}
    return inventory


def pcm_bytes(kind, count, np):
    samples = np.zeros(count, dtype=np.int16)
    if count and kind.startswith('impulse'):
        index = 0 if kind.endswith('first') else count // 2 if kind.endswith('middle') else count - 1
        samples[index] = 16384
    elif kind == 'ramp':
        samples = (np.arange(count, dtype=np.int64) % 32768 - 16384).astype(np.int16)
    elif kind == 'tone':
        samples = np.rint(8192.0 * np.sin(2.0 * math.pi * 1000.0 * np.arange(count) / 16000.0)).astype(np.int16)
    return samples.astype('<i2').tobytes()


def verify_pcm(raw, row):
    if len(raw) != row['samples'] * 2 or hashlib.sha256(raw).hexdigest() != row['pcm16le_sha256']:
        raise ValueError('existing PCM16 identity mismatch before scaling')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    started = time.monotonic()
    deadline = started + 600  # Never reset: guards/acquisition/build/comparison/readback all share it.
    def terminate(signum, frame):
        raise TimeoutError('combined KNF shared600s or termination; no retry')
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGALRM):
        signal.signal(sig, terminate)
    signal.setitimer(signal.ITIMER_REAL, 600)
    def remaining():
        seconds = deadline - time.monotonic()
        if seconds <= 0:
            raise TimeoutError('combined KNF shared600s')
        return seconds
    os.umask(0o077)
    out = args.output.resolve()
    reference = args.reference.resolve()
    if not out.is_relative_to((REPO / 'target').resolve()) or out == reference or reference.is_relative_to(out) or out.is_relative_to(reference):
        raise ValueError('new private target output separate from retained reference required')
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    evidence = {'status': 'incomplete', 'scope': 'feature comparison only; no model, ASR or export equivalence',
                'numeric_pass_tolerance': None, 'runs': [], 'archives': [], 'cases': [],
                'script_sha256': sha(Path(__file__)), 'deadline_seconds': 600}
    def save():
        evidence['seconds'] = time.monotonic() - started
        evidence['peak_self_KiB'] = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        (out / 'observations.json').write_text(json.dumps(evidence, indent=2, allow_nan=False))
    try:
        manifest_path = HERE / 'knf-frontend-handoff.json'
        lock = json.loads(manifest_path.read_text())
        evidence['manifest_sha256'] = sha(manifest_path)
        for name, expected in lock['files'].items():
            if sha(REPO / name) != expected:
                raise ValueError('handoff file identity mismatch: ' + name)
        for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'NUMEXPR_NUM_THREADS'):
            os.environ[name] = '4'
        def load_module(name, path):
            spec = importlib.util.spec_from_file_location(name, path)
            module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
            return module
        guard = load_module('guard344', REPO / 'scripts/probe-speech-344.py')
        groups = load_module('groups344', HERE / 'probe-sherpa-english.py')
        evidence['isolation'] = guard.limits()
        if importlib.metadata.version('numpy') != '2.4.2':
            raise ValueError('retained NumPy2.4.2 required')
        import numpy as np
        if sha(reference / 'reference.json') != lock['reference_sha256']:
            raise ValueError('retained reference identity mismatch')
        ref = json.loads((reference / 'reference.json').read_text())
        if ref['numeric_pass_tolerance'] is not None or ref['model_accessed'] or len(ref['cases']) != 55:
            raise ValueError('reference scope mismatch')
        expected_names = [f'{kind}-{n}' for n in (0,319,320,321,479,480,481,640,1600,400000)
                          for kind in (('tone',) if n == 400000 else ('zero','impulse-first','impulse-middle','impulse-last','ramp','tone'))]
        if [r['name'] for r in ref['cases']] != expected_names:
            raise ValueError('all55 ordered cases required')
        def retained(row):
            remaining()
            p = reference / row['path']
            if Path(row['path']).name != row['path'] or sha(p) != row['sha256']:
                raise ValueError('retained array identity mismatch')
            a = np.load(p, allow_pickle=False)
            if list(a.shape) != row['shape'] or str(a.dtype) != row['dtype'] or not bool(np.isfinite(a).all()):
                raise ValueError('retained array shape/dtype/finite guard')
            return a
        for row in [ref['window'], ref['mel_weights'], ref['knf_default_mel_weights']]:
            retained(row)
        for row in ref['cases']:
            for field in ('mel64','logmel64','logmel32'):
                retained(row[field])
        env = {k: os.environ[k] for k in ('PATH','LANG','LC_ALL') if k in os.environ}
        env.update({'PATH': '/usr/bin:/bin', 'TMPDIR': str(out),
                    'OMP_NUM_THREADS': '4', 'OPENBLAS_NUM_THREADS': '4', 'MKL_NUM_THREADS': '4', 'NUMEXPR_NUM_THREADS': '4'})
        def run(name, argv):
            remaining()
            start = time.monotonic()
            row = {'name': name, 'argv': argv, 'reaped': False}
            evidence['runs'].append(row)
            child = None
            timeout = None
            with (out / (name+'.stdout')).open('xb') as stdout, (out / (name+'.stderr')).open('xb') as stderr:
                try:
                    child = subprocess.Popen(argv, cwd=out, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
                    row.update({'pid': child.pid, 'pgid': child.pid})
                    timeout = groups.finish_group(child, max(0, deadline - time.monotonic()))
                    row['timeout'] = timeout
                finally:
                    if child is not None:
                        # Also cover interruption after Popen but before finish_group entry.
                        # This cleanup never dispatches another process or gives work a new budget.
                        if child.poll() is None:
                            groups.finish_group(child, 0)
                        else:
                            try:
                                os.killpg(child.pid, signal.SIGKILL)
                            except ProcessLookupError:
                                pass
                            child.wait()
                        row.update({'exit': child.returncode, 'reaped': child.poll() is not None})
                        try:
                            os.killpg(child.pid, 0); row['group_observed_gone'] = False
                        except ProcessLookupError:
                            row['group_observed_gone'] = True
                    row['seconds'] = time.monotonic()-start
                    save()
            if timeout or child.returncode or not row['group_observed_gone']:
                raise ValueError(name+': process/deadline/group failure; stop without retry')
            remaining()
        evidence['compilers'] = {}
        for name, exe in [('cc','/usr/bin/cc'), ('cxx','/usr/bin/c++')]:
            evidence['compilers'][name] = {'executable': exe, 'resolved': str(Path(exe).resolve()), 'sha256': sha(Path(exe))}
            run(name+'-version', [exe, '--version'])
            evidence['compilers'][name]['version_output'] = (out/(name+'-version.stdout')).read_text(encoding='utf-8',errors='strict')
        for entry in lock['archives']:
            archive = out / entry['filename']
            run('acquire-'+entry['name'], ['/usr/bin/curl', '-q', '--fail', '--location', '--proto', '=https',
                 '--proto-redir', '=https', '--tlsv1.2', '--max-time', str(remaining()),
                 '--max-filesize', str(ARCHIVE_LIMIT), '--output', str(archive), entry['url']])
            inventory = unpack_verified(archive, entry, out/entry['name'], out/(entry['name']+'-licenses'), remaining)
            evidence['archives'].append({'name': entry['name'], 'bytes': archive.stat().st_size,
                                        'sha256': sha(archive), 'files': inventory})
            save()
        source = out/'accessor.cc'; source.write_bytes((HERE/'knf-feature-accessor.cc').read_bytes())
        knf, kiss = out/'knf', out/'kissfft'
        for base in ('kiss_fft','kiss_fftr'):
            run('build-'+base, ['/usr/bin/cc','-O2','-DNDEBUG','-I',str(kiss),'-c',str(kiss/(base+'.c')),'-o',str(out/(base+'.o'))])
        units = ['feature-fbank','feature-functions','feature-window','kaldi-math','mel-computations','rfft']
        binary = out/'accessor'
        run('build-accessor', ['/usr/bin/c++','-O2','-DNDEBUG','-std=c++17','-I',str(knf),'-I',str(kiss),str(source)] +
            [str(knf/'kaldi-native-fbank/csrc'/(u+'.cc')) for u in units] +
            [str(out/'kiss_fft.o'),str(out/'kiss_fftr.o'),'-pthread','-lm','-o',str(binary)])
        evidence['binary_sha256'] = sha(binary)
        pcm_dir = out/'pcm'; pcm_dir.mkdir(mode=0o700)
        results = out/'features'; results.mkdir(mode=0o700)
        def raw_array(path, shape):
            remaining()
            if path.stat().st_size != math.prod(shape)*4:
                raise ValueError('KNF exact byte shape mismatch')
            a = np.fromfile(path, dtype='<f4').reshape(shape)
            if not bool(np.isfinite(a).all()):
                raise ValueError('KNF nonfinite array')
            return a
        def difference(a,b):
            if a.shape != b.shape or not bool(np.isfinite(a).all() and np.isfinite(b).all()):
                raise ValueError('exact shape/finite comparison guard')
            d = np.abs(a.astype(np.float64)-b.astype(np.float64))
            if not d.size:
                return {'max_abs':0.0, 'coordinate':None, 'max_relative_on_nonzero_reference':None}
            index = tuple(int(x) for x in np.unravel_index(int(np.argmax(d)),d.shape))
            nz = a != 0
            return {'max_abs':float(d[index]), 'coordinate':list(index),
                    'max_relative_on_nonzero_reference':float(np.max(d[nz]/np.abs(a[nz]))) if bool(nz.any()) else None}
        for row in ref['cases']:
            remaining()
            name, count = row['name'], row['samples']
            frames = max(0,(count-320)//160+1)
            if row['frames'] != frames:
                raise ValueError('retained exact framing mismatch')
            kind = name.rsplit('-',1)[0]
            raw = pcm_bytes(kind,count,np)
            verify_pcm(raw,row)  # BEFORE conversion/scaling; original reference never executed.
            pcm16 = pcm_dir/(name+'.s16le'); pcm16.write_bytes(raw)
            f32 = (np.frombuffer(raw,dtype='<i2').astype(np.float32)/np.float32(32768)).astype('<f4')
            pcm = pcm_dir/(name+'.f32le'); f32.tofile(pcm)
            dest = results/name
            run('feature-'+name,[str(binary),str(pcm),str(dest)])
            expected = f'samples={count} frames={frames} bins=64\n'
            if (out/('feature-'+name+'.stdout')).read_text() != expected:
                raise ValueError('accessor frame readback mismatch')
            window = raw_array(dest/'window.f32le',(320,))
            weights = raw_array(dest/'weights-64x161.f32le',(64,161)).T
            mel = raw_array(dest/'mel.f32le',(frames,64))
            log64 = np.log(np.clip(mel.astype(np.float64),1e-9,1e9))
            log32 = np.log(np.clip(mel,np.float32(1e-9),np.float32(1e9)))
            if not bool(np.isfinite(log64).all() and np.isfinite(log32).all()):
                raise ValueError('nonfinite derived log arrays')
            np.save(dest/'log64.npy',log64,allow_pickle=False)
            np.save(dest/'log32.npy',log32,allow_pickle=False)
            comparisons = {k:difference(retained(ref[r]),a) for k,r,a in
                           [('window','window',window),('weights_htk_hz','mel_weights',weights)]}
            comparisons.update({k:difference(retained(row[r]),a) for k,r,a in
                                [('energy64','mel64',mel),('log64','logmel64',log64),('log32','logmel32',log32)]})
            evidence['cases'].append({'name':name,'samples':count,'frames':frames,'pcm16_sha256':sha(pcm16),
                                     'pcm_float32_sha256':sha(pcm),'comparisons':comparisons,
                                     'outputs':{p.name:{'bytes':p.stat().st_size,'sha256':sha(p)} for p in dest.iterdir()}})
            save(); remaining()
        evidence['status'] = 'feature observations complete; differences unqualified, not ASR acceptance'
        remaining()
    except BaseException as error:
        evidence['failure'] = str(error)
        raise
    finally:
        # No deletion of partial evidence, archives, arrays or prior attempts.
        # Allow terminal metadata write on deadline, with no further acquisitions/children/comparison.
        signal.setitimer(signal.ITIMER_REAL, 0)
        save()


if __name__ == '__main__':
    main()
