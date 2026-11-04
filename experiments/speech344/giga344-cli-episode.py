#!/usr/bin/env python3
"""PREPARED ONLY. PM approval required for one supported CLI build/short-quality episode."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import re
import signal
import shutil
import subprocess
import time
import wave

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def module(name,p):
    spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def build_commands(out):
    cmake=out/'cmake/bin/cmake';source=out/'sherpa';build=out/'build'
    argv=[str(cmake),'-S',str(source),'-B',str(build),'-G','Unix Makefiles',
          '-DCMAKE_MAKE_PROGRAM=/usr/bin/make','-DCMAKE_C_COMPILER=/usr/bin/cc','-DCMAKE_CXX_COMPILER=/usr/bin/c++',
          '-DCMAKE_BUILD_TYPE=Release','-DCMAKE_CXX_FLAGS=-DSHERPA_ONNX_EXPERIMENT_GIGA344',
          '-DCMAKE_INSTALL_PREFIX='+str(out/'never-installed'),'-DBUILD_SHARED_LIBS=ON',
          '-DFETCHCONTENT_FULLY_DISCONNECTED=ON','-DFETCHCONTENT_UPDATES_DISCONNECTED=ON',
          '-DBUILD_TESTING=OFF','-DEIGEN_BUILD_DOC=OFF','-DJSON_BuildTests=OFF',
          '-DSHERPA_ONNX_ENABLE_BINARY=ON','-DSHERPA_ONNX_USE_PRE_INSTALLED_ONNXRUNTIME_IF_AVAILABLE=ON']
    for flag in ['PYTHON','TESTS','CHECK','PORTAUDIO','JNI','C_API','WEBSOCKET','GPU','DIRECTML','TTS',
                 'SPEAKER_DIARIZATION','WASM','RKNN','AXERA','AXCL','ASCEND_NPU','QNN','SPACEMIT']:
        argv+=['-DSHERPA_ONNX_ENABLE_'+flag+'=OFF']
    argv+=['-DSHERPA_ONNX_BUILD_C_API_EXAMPLES=OFF','-DKALDI_DECODER_BUILD_PYTHON=OFF',
           '-DKALDI_DECODER_ENABLE_TESTS=OFF','-DKALDIFST_BUILD_PYTHON=OFF','-DKALDIFST_BUILD_TESTS=OFF',
           '-DSBPE_BUILD_PYTHON=OFF','-DSBPE_ENABLE_TESTS=OFF']
    names={'KALDI_NATIVE_FBANK':'knf','KISSFFT':'kissfft','UTF8PROC':'utf8proc','KALDI_DECODER':'kaldi-decoder',
           'KALDIFST':'kaldifst','EIGEN':'eigen','OPENFST':'openfst','SIMPLE-SENTENCEPIECE':'simple-sentencepiece','JSON':'json'}
    argv+=['-DFETCHCONTENT_SOURCE_DIR_'+key+'='+str(out/value) for key,value in names.items()]
    return [('configure',argv),('build',[str(cmake),'--build',str(build),'--target','sherpa-onnx-offline','--parallel','4'])]

def validate_openfst_pin(source,entry):
    url=re.search(r'set\(openfst_URL\s+"([^"\n]+)"\)',source)
    digest=re.search(r'set\(openfst_HASH\s+"SHA256=([0-9a-f]{64})"\)',source)
    if not url or not digest or entry['url']!=url[1] or entry['sha256']!=digest[1]:
        raise ValueError('OpenFst override must match owning Sherpa declaration, not transitive kaldifst pin')


def recognition_command(engine,assets,wav):
    return [str(engine),'--model-type=nemo_ctc','--nemo-ctc-model='+str(assets/'model.int8.onnx'),
            '--tokens='+str(assets/'tokens.txt'),'--provider=cpu','--num-threads=4','--decoding-method=greedy_search',str(wav)]

def pcm(path):
    with wave.open(str(path),'rb') as w:
        if (w.getnchannels(),w.getsampwidth(),w.getframerate(),w.getcomptype())!=(1,2,16000,'NONE'):
            raise ValueError('exact mono PCM16/16kHz required')
        if not 0<w.getnframes()<=400000:raise ValueError('short episode 25s frame cap')
        b=w.readframes(w.getnframes())
        if len(b)!=w.getnframes()*2:raise ValueError('partial PCM')
        return b

def write_pcm(path,data):
    if not data or len(data)%2 or len(data)>800000:raise ValueError('short episode PCM cap')
    with wave.open(str(path),'wb') as w:
        w.setnchannels(1);w.setsampwidth(2);w.setframerate(16000);w.writeframes(data)

def read_text(path):
    b=path.read_bytes()
    if len(b)>1024*1024:raise ValueError('transcript output cap')
    v=json.loads(b.decode('utf-8',errors='strict'));text=v.get('text')
    if not isinstance(text,str) or '\ufffd' in text:raise ValueError('invalid transcript UTF8/type')
    return text

def readiness_result(path,started,words):
    text=read_text(path)
    if words(text):raise ValueError('silence hallucination; stop')
    elapsed=time.monotonic()-started
    if elapsed>10:raise TimeoutError("readiness generation/checks/readback exceeded original10s")
    return text,elapsed


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('archives','assets','output'):parser.add_argument('--'+name,type=Path,required=True)
    args=parser.parse_args();started=time.monotonic();deadline=started+600
    def terminate(signum,frame):raise TimeoutError('one original600s episode terminated; no retry')
    for sig in (signal.SIGTERM,signal.SIGINT,signal.SIGALRM):signal.signal(sig,terminate)
    signal.setitimer(signal.ITIMER_REAL,600)
    def remaining():
        left=deadline-time.monotonic()
        if left<=0:raise TimeoutError('original600s expired')
        return left
    os.umask(0o077);out=args.output.resolve();archives=args.archives.resolve();assets=args.assets.resolve();target=(REPO/'target').resolve()
    if not all(p.is_relative_to(target) and p!=target for p in (out,archives,assets)) or any(out==p or out.is_relative_to(p) or p.is_relative_to(out) for p in (archives,assets)):
        raise ValueError('new distinct project-private target paths required')
    out.mkdir(mode=0o700,exist_ok=False)
    evidence={'status':'incomplete','scope':'candidate-only build/short quality; no install/full300/live/official equivalence',
              'runs':[],'cases':[],'deadline_seconds':600,'delivered':False}
    def save():
        evidence['seconds']=time.monotonic()-started;evidence['peak_self_KiB']=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        (out/'observations.json').write_text(json.dumps(evidence,ensure_ascii=False,indent=2,allow_nan=False))
    try:
        lock=json.loads((HERE/'giga344-cli-handoff.json').read_text());evidence['manifest_sha256']=sha(HERE/'giga344-cli-handoff.json')
        for name,h in lock['files'].items():
            if sha(REPO/name)!=h:raise ValueError('handoff identity: '+name)
        for name,h in lock['tools'].items():
            if sha(name)!=h:raise ValueError('tool identity: '+name)
        for name,row in lock['model_files'].items():
            p=assets/name
            if p.is_symlink() or p.stat().st_size!=row['size'] or sha(p)!=row['sha256']:raise ValueError('candidate identity: '+name)
        (assets/'tokens.txt').read_text(encoding='utf-8',errors='strict')
        for name,row in lock['fixtures'].items():
            p=REPO/name
            if p.is_symlink() or p.stat().st_size!=row['bytes'] or sha(p)!=row['sha256']:raise ValueError('fixture identity')
        helper=module('knf344',HERE/'knf-frontend-episode.py');source=module('source344',HERE/'giga344-build-episode.py')
        groups=module('groups344',HERE/'probe-sherpa-english.py');guard=module('guard344',REPO/'scripts/probe-speech-344.py')
        evidence['isolation']=guard.limits()
        env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TMPDIR':str(out),'OMP_NUM_THREADS':'4',
             'OPENBLAS_NUM_THREADS':'4','MKL_NUM_THREADS':'4','NUMEXPR_NUM_THREADS':'4',
             'SHERPA_ONNXRUNTIME_INCLUDE_DIR':str(out/'ort/include'),'SHERPA_ONNXRUNTIME_LIB_DIR':str(out/'ort/lib')}
        def run(name,argv,limit=None,cwd=None):
            remaining();row={'name':name,'argv':[str(a) for a in argv]};evidence['runs'].append(row)
            child=None;start=time.monotonic();timed=False
            with (out/(name+'.stdout')).open('xb') as stdout,(out/(name+'.stderr')).open('xb') as stderr:
                try:
                    child=subprocess.Popen(row['argv'],cwd=cwd or out,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
                    row.update(pid=child.pid,pgid=child.pid)
                    timed=groups.finish_group(child,min(remaining(),limit) if limit is not None else remaining())
                finally:
                    if child is not None:
                        groups.finish_group(child,0);row.update(exit=child.returncode,reaped=child.poll() is not None)
                        try:os.killpg(child.pid,0);row['group_observed_gone']=False
                        except ProcessLookupError:row['group_observed_gone']=True
                    row.update(timeout=timed,seconds=time.monotonic()-start);save()
            if timed or child.returncode or not row['group_observed_gone']:raise ValueError(name+': process/deadline/group failure; no retry')
            remaining()
        evidence['archives']=[]
        for entry in lock['archives']:
            if entry['name']=='openfst':
                validate_openfst_pin((out/'sherpa/cmake/openfst.cmake').read_text(),entry)
            p=archives/entry['filename']
            if p.is_symlink() or p.stat().st_size!=entry['bytes'] or sha(p)!=entry['sha256']:raise ValueError('archive identity')
            if 'select_prefixes' in entry:
                count=source.unpack_source(p,entry,out/entry['name'],remaining,helper)
            else:count=len(helper.unpack_verified(p,entry,out/entry['name'],out/(entry['name']+'-licenses'),remaining))
            evidence['archives'].append({'name':entry['name'],'sha256':sha(p),'selected_files':count});save()
        cmake=out/'cmake/bin/cmake'
        if sha(cmake)!=lock['cmake_binary_sha256']:raise ValueError('CMake binary identity')
        cmake.chmod(0o700) # Only this new private verified tool; never PATH/global installation.
        for name,exe in [('cmake',cmake),('make','/usr/bin/make'),('cc','/usr/bin/cc'),('cxx','/usr/bin/c++'),('ffmpeg','/usr/bin/ffmpeg')]:
            run('version-'+name,[str(exe),'-version' if name=='ffmpeg' else '--version'])
        root=out/'sherpa';pinned=json.loads((HERE/'giga344-candidate.json').read_text())
        for name,h in pinned['sources'].items():
            if sha(root/'sherpa-onnx/csrc'/name)!=h:raise ValueError('pinned Sherpa source')
        for flag in ['--check','']:
            argv=['/usr/bin/git','apply']+([flag] if flag else [])+['--directory='+str(root.relative_to(REPO)),str(HERE/'giga344-candidate.patch')]
            run('patch-check' if flag else 'patch-apply',argv,cwd=REPO)
        for name,argv in build_commands(out):run(name,argv)
        engine=out/'build/bin/sherpa-onnx-offline'
        evidence['engine_sha256']=sha(engine)
        evidence['built_libraries']={str(p.relative_to(out)):sha(p) for p in (out/'build/lib').glob('*.so*') if p.is_file()}
        run('engine-elf',['/usr/bin/readelf','-d',str(engine)])
        episode_assets=out/'candidate-assets';episode_assets.mkdir(mode=0o700)
        for name in lock['model_files']:shutil.copyfile(assets/name,episode_assets/name)
        def checked_command(wav):
            remaining()
            for name,row in lock['model_files'].items():
                p=episode_assets/name
                if p.stat().st_size!=row['size'] or sha(p)!=row['sha256']:raise ValueError('episode candidate identity')
            if sha(engine)!=evidence['engine_sha256']:raise ValueError('episode executable identity')
            return recognition_command(engine,episode_assets,wav)
        # No transcript/reference/prompt is ever passed to the recognizer.
        ready_started=time.monotonic();ready=out/'readiness.wav';write_pcm(ready,b'\0\0'*16000)
        run('readiness',checked_command(ready),limit=max(0,10-(time.monotonic()-ready_started)))
        ready_text,ready_elapsed=readiness_result(out/'readiness.stdout',ready_started,groups.words)
        evidence.update(readiness_text=ready_text,readiness_seconds=ready_elapsed);save()
        if time.monotonic()-ready_started>10:raise TimeoutError('readiness evidence readback exceeded original10s')
        decoded={}
        for lang in ['ru','en']:
            wav=out/('human-'+lang+'.wav')
            run('decode-'+lang,['/usr/bin/ffmpeg','-nostdin','-v','error','-i',str(REPO/('tests/fixtures/speech/'+lang+'.ogg')),
                '-vn','-ac','1','-ar','16000','-c:a','pcm_s16le','-y',str(wav)])
            decoded[lang]=pcm(wav)
            evidence['cases'].append({'name':'human-'+lang,'decoded_pcm_sha256':hashlib.sha256(decoded[lang]).hexdigest(),'samples':len(decoded[lang])//2})
        mixed=out/'mixed-ru-en.wav';write_pcm(mixed,decoded['ru']+decoded['en'])
        evidence['mixed']={'construction':'contiguous unchanged decoded RU then EN PCM; no inserted silence/padding/stretch',
                          'pcm_sha256':hashlib.sha256(decoded['ru']+decoded['en']).hexdigest(),'samples':(len(decoded['ru'])+len(decoded['en']))//2}
        for name,path in [('human-ru',out/'human-ru.wav'),('human-en',out/'human-en.wav'),
                          ('private-ru',REPO/'target/pm344-speech-inputs/speech344-ru.wav'),
                          ('private-en',REPO/'target/pm344-speech-inputs/speech344-en.wav'),('mixed-ru-en',mixed)]:
            pcm(path);run(name,checked_command(path));text=read_text(out/(name+'.stdout'))
            expected=lock['references'][name];actual=groups.words(text);reference=groups.words(expected)
            row={'name':name,'text':text,'reference':expected,'word_errors':groups.distance(reference,actual),
                 'reference_words':len(reference),'full_reference_equal':actual==reference,'delivered':False}
            evidence['cases'].append(row);save()
            if actual!=reference:raise ValueError(name+': full-reference failure; spelling/meaning require separate review, no retry')
        evidence['status']='short candidate references matched; not Russian baseline/full300/official/live acceptance';remaining()
    except BaseException as e:evidence['failure']=str(e);raise
    finally:signal.setitimer(signal.ITIMER_REAL,0);save()

if __name__=='__main__':main()
