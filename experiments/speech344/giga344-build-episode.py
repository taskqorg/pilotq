#!/usr/bin/env python3
"""Prepared PM-only no-model build/feature episode. Never run without PM approval."""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import tarfile
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def unpack_source(archive, entry, dest, remaining, helper):
    """Only regular csrc and exact license/pin texts, not archive scripts or links."""
    if archive.stat().st_size != entry['bytes'] or sha(archive) != entry['sha256']:
        raise ValueError('source archive identity')
    files, seen, total = {}, set(), 0
    with tarfile.open(archive, 'r:gz') as t:
        for m in t:
            remaining()
            rel = helper.member_path(m.name, entry['prefix']).as_posix()
            if rel in seen:
                raise ValueError('duplicate source member')
            seen.add(rel)
            selected = rel in entry['select_files'] or any(rel.startswith(p) for p in entry['select_prefixes'])
            if not selected or m.isdir():
                continue
            if not m.isfile() or m.size > 32*1024**2 or m.size < 0:
                raise ValueError('selected source link/special/size')
            total += m.size
            if total > 128*1024**2:
                raise ValueError('selected source expansion')
            files[rel] = m
        for name, digest in entry['licenses'].items():
            if name not in files or hashlib.sha256(t.extractfile(files[name]).read()).hexdigest() != digest:
                raise ValueError('source license identity')
        dest.mkdir(mode=0o700)
        for rel, m in files.items():
            remaining()
            b = t.extractfile(m).read()
            if len(b) != m.size:
                raise ValueError('source member length')
            p = dest/rel; p.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            p.write_bytes(b)
    return len(files)


def build_commands(out, root):
    """Exact direct argv, inspected without executing compilers in preparation."""
    knf, kiss, ort = out/'knf', out/'kissfft', out/'ort'
    common = ['/usr/bin/c++','-O2','-DNDEBUG','-std=c++17','-ffunction-sections','-fdata-sections',
              '-I',str(root),'-I',str(knf),'-I',str(kiss),'-I',str(ort/'include')]
    commands = []
    for unit in ('kiss_fft','kiss_fftr'):
        commands.append(('build-'+unit, ['/usr/bin/cc','-O2','-DNDEBUG','-I',str(kiss),'-c',str(kiss/(unit+'.c')),'-o',str(out/(unit+'.o'))]))
    # Syntax/API integration of real Forward, but no session construction or linking its object.
    commands.append(('compile-forward', common+['-DSHERPA_ONNX_EXPERIMENT_GIGA344','-c',
        str(root/'sherpa-onnx/csrc/offline-nemo-enc-dec-ctc-model.cc'),'-o',str(out/'forward.o')]))
    commands.append(('build-boundary', common+['-DSHERPA_ONNX_EXPERIMENT_GIGA344',
        str(HERE/'giga344-boundary.cc'),str(ort/'lib/libonnxruntime.so'),
        '-Wl,-rpath,'+str(ort/'lib'),'-pthread','-ldl','-lm','-o',str(out/'boundary')]))
    units = ['feature-fbank','feature-functions','feature-mfcc','feature-raw-audio-samples','feature-window',
             'istft','kaldi-math','mel-computations','online-feature','rfft','stft','whisper-feature']
    for variant, source, flags in [('candidate',root,['-DSHERPA_ONNX_EXPERIMENT_GIGA344']),
                                  ('disabled',root,[]),('upstream',out/'upstream',[])]:
        argv = common.copy(); argv[argv.index(str(root))] = str(source)
        argv += flags+[str(HERE/'giga344-features.cc')]
        argv += [str(source/'sherpa-onnx/csrc'/n) for n in ('features.cc','resample.cc')]
        argv += [str(knf/'kaldi-native-fbank/csrc'/(n+'.cc')) for n in units]
        argv += [str(out/'kiss_fft.o'),str(out/'kiss_fftr.o'),'-Wl,--gc-sections','-pthread','-lm','-o',str(out/('features-'+variant))]
        commands.append(('build-features-'+variant,argv))
    return commands


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('archives','reference','knf-observations','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    args = parser.parse_args()
    started = time.monotonic(); deadline = started+600
    def terminate(signum, frame):
        raise TimeoutError('original shared600s terminated; no retry')
    for sig in (signal.SIGTERM,signal.SIGINT,signal.SIGALRM):
        signal.signal(sig,terminate)
    signal.setitimer(signal.ITIMER_REAL,600)
    def remaining():
        left = deadline-time.monotonic()
        if left <= 0: raise TimeoutError('original shared600s expired')
        return left
    os.umask(0o077)
    out = args.output.resolve(); archives = args.archives.resolve(); reference = args.reference.resolve()
    knf_observations = args.knf_observations.resolve()
    target = (REPO/'target').resolve()
    if not all(p.is_relative_to(target) and p != target for p in (out,archives,reference,knf_observations)):
        raise ValueError('private project target paths required')
    if any(out == p or p.is_relative_to(out) or out.is_relative_to(p) for p in (archives,reference,knf_observations)):
        raise ValueError('new separate output required')
    out.mkdir(mode=0o700,exist_ok=False)
    evidence = {'status':'incomplete','scope':'no-model boundary/features only; no ASR/parity acceptance',
                'runs':[],'cases':[],'numeric_pass_tolerance':None,'deadline_seconds':600}
    def save():
        evidence['seconds'] = time.monotonic()-started
        evidence['peak_self_KiB'] = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        (out/'observations.json').write_text(json.dumps(evidence,indent=2,allow_nan=False))
    try:
        lock = json.loads((HERE/'giga344-build-handoff.json').read_text())
        evidence['manifest_sha256'] = sha(HERE/'giga344-build-handoff.json')
        for name,digest in lock['files'].items():
            if sha(REPO/name) != digest: raise ValueError('handoff identity: '+name)
        for exe,digest in lock['tools'].items():
            if sha(exe) != digest: raise ValueError('tool identity: '+exe)
        helper = load('feature_helper344',HERE/'knf-frontend-episode.py')
        groups = load('groups344',HERE/'probe-sherpa-english.py')
        guard = load('guard344',REPO/'scripts/probe-speech-344.py')
        evidence['isolation'] = guard.limits()
        for variable in ('OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS','NUMEXPR_NUM_THREADS'):
            os.environ[variable]='4'
        # Retained qualification-only NumPy; no setup/download/import of ONNX or a model.
        import importlib.metadata
        if importlib.metadata.version('numpy') != '2.4.2': raise ValueError('retained NumPy2.4.2 required')
        import numpy as np
        env = {'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TMPDIR':str(out),
               'OMP_NUM_THREADS':'4','OPENBLAS_NUM_THREADS':'4','MKL_NUM_THREADS':'4','NUMEXPR_NUM_THREADS':'4'}
        def run(name, argv, cwd=None):
            remaining(); row={'name':name,'argv':[str(a) for a in argv]}; evidence['runs'].append(row)
            child=None; start=time.monotonic(); timed_out=False
            with (out/(name+'.stdout')).open('xb') as stdout, (out/(name+'.stderr')).open('xb') as stderr:
                try:
                    child=subprocess.Popen(row['argv'],cwd=cwd or out,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
                    row.update(pid=child.pid,pgid=child.pid)
                    timed_out=groups.finish_group(child,remaining())
                finally:
                    if child is not None:
                        # Includes signals/errors between spawn and finish_group entry.
                        groups.finish_group(child,0)
                        row.update(exit=child.returncode,reaped=child.poll() is not None)
                        try: os.killpg(child.pid,0);row['group_observed_gone']=False
                        except ProcessLookupError: row['group_observed_gone']=True
                    row['timeout']=timed_out;row['seconds']=time.monotonic()-start;save()
            if timed_out or child.returncode or not row['group_observed_gone']:
                raise ValueError(name+': command/deadline/group failure; no retry')
            remaining()
        evidence['tools']={}
        for exe,digest in lock['tools'].items():
            name=Path(exe).name
            run('version-'+name,[exe,'--version'])
            evidence['tools'][exe]={'sha256':digest,'resolved':str(Path(exe).resolve()),'version':(out/('version-'+name+'.stdout')).read_text()}
        evidence['archives']=[]
        # All archives are staged, never network-acquired by this episode.
        for entry in lock['archives']:
            p=archives/entry['filename']
            if p.is_symlink() or p.stat().st_size != entry['bytes'] or sha(p) != entry['sha256']:
                raise ValueError('archive identity: '+entry['name'])
            if entry['name']=='sherpa':
                count=unpack_source(p,entry,out/'upstream',remaining,helper)
            else:
                count=len(helper.unpack_verified(p,entry,out/entry['name'],out/(entry['name']+'-licenses'),remaining))
            evidence['archives'].append({'name':entry['name'],'sha256':sha(p),'selected_files':count});save()
        root=out/'candidate';shutil.copytree(out/'upstream',root)
        pinned=json.loads((HERE/'giga344-candidate.json').read_text())
        for name,digest in pinned['sources'].items():
            if sha(root/'sherpa-onnx/csrc'/name)!=digest: raise ValueError('changed upstream source')
        run('patch-check',['/usr/bin/git','apply','--check','--directory='+str(root.relative_to(REPO)),str(HERE/'giga344-candidate.patch')],REPO)
        run('patch-apply',['/usr/bin/git','apply','--directory='+str(root.relative_to(REPO)),str(HERE/'giga344-candidate.patch')],REPO)
        for name,argv in build_commands(out,root): run(name,argv)
        evidence['binaries']={n:sha(out/n) for n in ('boundary','features-candidate','features-disabled','features-upstream')}
        # Static ELF readback records dependency closure; never ldd/dlopen in preparation.
        run('boundary-elf',['/usr/bin/readelf','-d',str(out/'boundary')])
        run('ort-elf',['/usr/bin/readelf','-d',str(out/'ort/lib/libonnxruntime.so')])
        run('synthetic-boundary',[str(out/'boundary')])
        if sha(reference/'reference.json')!=lock['reference_sha256'] or sha(knf_observations/'observations.json')!=lock['knf_observations_sha256']:
            raise ValueError('retained observation identities')
        ref=json.loads((reference/'reference.json').read_text());old=json.loads((knf_observations/'observations.json').read_text())
        expected=[f'{k}-{n}' for n in (0,319,320,321,479,480,481,640,1600,400000)
                  for k in (('tone',) if n==400000 else ('zero','impulse-first','impulse-middle','impulse-last','ramp','tone'))]
        if [r['name'] for r in ref['cases']]!=expected or [r['name'] for r in old['cases']]!=expected:
            raise ValueError('all55 ordered cases required')
        for row,previous in zip(ref['cases'],old['cases']):
            remaining();name=row['name'];frames=row['frames'];raw=helper.pcm_bytes(name.rsplit('-',1)[0],row['samples'],np)
            helper.verify_pcm(raw,row)  # Before accessor scaling; no reference rerun.
            pcm=out/(name+'.s16le');pcm.write_bytes(raw)
            arrays={}
            for variant in ('candidate','disabled','upstream'):
                result=out/(name+'-'+variant+'.f32le')
                run(name+'-'+variant,[str(out/('features-'+variant)),str(pcm),str(result)])
                expected_frames=frames if variant=='candidate' else (row['samples']+80)//160
                stdout=(out/(name+'-'+variant+'.stdout')).read_text()
                if stdout!=f'frames={expected_frames} bins=64 repeated/stream identical; no model\n':
                    raise ValueError('exact frame/stream readback')
                if result.stat().st_size!=expected_frames*64*4: raise ValueError('feature byte shape')
                a=np.fromfile(result,dtype='<f4').reshape(expected_frames,64)
                if not bool(np.isfinite(a).all()): raise ValueError('nonfinite feature')
                arrays[variant]=a
            if not np.array_equal(arrays['disabled'],arrays['upstream']):
                raise ValueError('compile-disabled legacy differs from pristine upstream')
            comparisons={}
            for label,path,digest in [('reference',reference/row['logmel32']['path'],row['logmel32']['sha256']),
                    ('knf',knf_observations/'features'/name/'log32.npy',previous['outputs']['log32.npy']['sha256'])]:
                if sha(path)!=digest:raise ValueError('retained array identity')
                a=np.load(path,allow_pickle=False)
                if a.shape!=(frames,64) or a.dtype!=np.float32 or not bool(np.isfinite(a).all()):raise ValueError('retained shape/finite')
                delta=np.abs(a.astype(np.float64)-arrays['candidate'].astype(np.float64))
                index=tuple(int(x) for x in np.unravel_index(int(np.argmax(delta)),delta.shape)) if delta.size else None
                comparisons[label]={'max_abs':float(delta[index]) if index else 0.0,'coordinate':list(index) if index else None}
            evidence['cases'].append({'name':name,'frames':frames,'pcm16_sha256':sha(pcm),'comparisons':comparisons,
                'outputs':{v:sha(out/(name+'-'+v+'.f32le')) for v in arrays},'disabled_equals_upstream':True});save()
        evidence['status']='no-model observations complete; numeric differences unqualified; no ASR acceptance'
        remaining()
    except BaseException as error:
        evidence['failure']=str(error);raise
    finally:
        signal.setitimer(signal.ITIMER_REAL,0);save()


if __name__=='__main__': main()
