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
