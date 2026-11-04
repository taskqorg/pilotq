import importlib.util
from pathlib import Path
spec=importlib.util.spec_from_file_location('b',Path(__file__).with_name('run-boundaries.py'))
b=importlib.util.module_from_spec(spec); spec.loader.exec_module(b)
a=dict(label='default',rc=0,language=1,encodes=2,begins=1,aborts=2,text='שלום')
z=dict(a,encodes=1)
log_a='PILOTQ344 encode offset=0 context=0\n'*2+'PILOTQ344 decoder-reset\n'
log_z='PILOTQ344 encode offset=0 context=0\nPILOTQ344 decoder-reset\nexperimental first-window reuse offset=0 audio_ctx=0\n'
for defect, row, log in [('stale-text',dict(z,text='old'),log_z),('wrong-count',dict(z,encodes=2),log_z),('lost-callback',dict(z,begins=0),log_z),('lost-reset',z,log_z.replace('PILOTQ344 decoder-reset\n',''))]:
 try: b.compare('default',[a],[row],log_a,log)
 except ValueError: print('RED detector rejected',defect)
 else: raise AssertionError(defect)
b.compare('default',[a],[z],log_a,log_z)
assert b.rows('RESULT\tdefault\t0\t1\t1\t1\t2\t'+ 'שלום'.encode().hex())[0]['text']=='שלום'
print('GREEN intact observation and UTF-8 parser PASS; no inference')

# The paired probe must reject incomplete/stale boundary evidence before ASR dispatch.
import json
import tempfile
pair_spec=importlib.util.spec_from_file_location('pair',Path(__file__).with_name('paired-performance.py'))
pair=importlib.util.module_from_spec(pair_spec); pair_spec.loader.exec_module(pair)
with tempfile.TemporaryDirectory(prefix='344-proof-check-') as directory:
 paths=[]
 for case in sorted(pair.CASES):
  data={'argv':{'case':case,'baseline_sha256':'a'*64,'patched_sha256':'b'*64},
        'status':'boundary-pass-only; no performance or quality qualification',
        'passed_boundaries':['multi','second'] if case=='multi' else [case],
        'isolation':{'effective':{'memory.max':1342177280,'memory.high':1073741824,
                                'memory.swap.max':268435456,'pids.max':256},'cpu_quota':'3/2'}}
  if case=='cancel': data['external_cancellation']='observed'
  path=Path(directory)/(case+'.json'); path.write_text(json.dumps(data)); paths.append(path)
 for defect, selected in [('missing-case',paths[:-1]),('duplicate-case',paths+[paths[0]])]:
  try: pair.proofs(selected)
  except ValueError: print('RED paired gate rejected',defect)
  else: raise AssertionError(defect)
 original=paths[0].read_text(); data=json.loads(original); data['failure']='mismatch'
 paths[0].write_text(json.dumps(data))
 try: pair.proofs(paths)
 except ValueError: print('RED paired gate rejected failed-proof')
 else: raise AssertionError('failed-proof')
 paths[0].write_text(original)
 assert len(pair.proofs(paths)['observation_sha256'])==10
 print('GREEN complete boundary evidence gate PASS; no inference')
