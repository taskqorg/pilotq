#!/usr/bin/env python3
"""No compiler/linker/model calls: protect selective extraction before PM dispatch."""
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[1]
def module(name, path):
    spec=importlib.util.spec_from_file_location(name,path)
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
helper=module('knf344guard',REPO/'experiments/speech344/knf-frontend-episode.py')
episode=module('giga344guard',REPO/'experiments/speech344/giga344-build-episode.py')

class SourceGuards(unittest.TestCase):
    def test_selected_source_never_extracts_archive_scripts_or_links(self):
        with tempfile.TemporaryDirectory(prefix='giga344-guard-') as temp:
            root=Path(temp);archive=root/'source.tar.gz'
            with tarfile.open(archive,'w:gz') as t:
                for name,body in [('p/LICENSE',b'invented license'),('p/sherpa-onnx/csrc/features.cc',b'invented source')]:
                    m=tarfile.TarInfo(name);m.size=len(body);t.addfile(m,io.BytesIO(body))
                m=tarfile.TarInfo('p/unused-script');m.type=tarfile.SYMTYPE;m.linkname='/invented/outside';t.addfile(m)
            entry={'bytes':archive.stat().st_size,'sha256':episode.sha(archive),'prefix':'p',
                   'select_files':['LICENSE'],'select_prefixes':['sherpa-onnx/csrc/'],
                   'licenses':{'LICENSE':hashlib.sha256(b'invented license').hexdigest()}}
            self.assertEqual(episode.unpack_source(archive,entry,root/'ok',lambda:None,helper),2)
            self.assertFalse((root/'ok/unused-script').exists())
            entry['licenses']['LICENSE']='0'*64
            with self.assertRaisesRegex(ValueError,'license'):
                episode.unpack_source(archive,entry,root/'refused',lambda:None,helper)
            self.assertFalse((root/'refused').exists(),'no source before license checks')
    def test_selected_links_paths_deadline_and_digest_fail_closed(self):
        for name in ['p/sherpa-onnx/csrc/evil','p/sherpa-onnx/csrc/../../escape','/p/LICENSE']:
            with self.subTest(name=name), tempfile.TemporaryDirectory(prefix='giga344-guard-') as temp:
                root=Path(temp);archive=root/'source.tar.gz'
                with tarfile.open(archive,'w:gz') as t:
                    m=tarfile.TarInfo(name);m.type=tarfile.SYMTYPE;m.linkname='/invented/outside';t.addfile(m)
                entry={'bytes':archive.stat().st_size,'sha256':episode.sha(archive),'prefix':'p',
                       'select_files':['LICENSE'],'select_prefixes':['sherpa-onnx/csrc/'],'licenses':{}}
                with self.assertRaises(ValueError):episode.unpack_source(archive,entry,root/'no',lambda:None,helper)
                self.assertFalse((root/'no').exists())
                entry['sha256']='0'*64
                with self.assertRaisesRegex(ValueError,'identity'):episode.unpack_source(archive,entry,root/'no',lambda:None,helper)
                entry['sha256']=episode.sha(archive)
                def expired():raise TimeoutError('original deadline')
                with self.assertRaises(TimeoutError):episode.unpack_source(archive,entry,root/'no',expired,helper)
                self.assertFalse((root/'no').exists())

# §1/§12: a future model result cannot become a fabricated transcript when its
# UTF8/JSON/type is damaged; short mixed PCM must preserve bytes and the25s cap.
class QualityPlanGuards(unittest.TestCase):
    def test_openfst_override_follows_sherpa_not_transitive_pin(self):
        cli=module('clipin344guard',REPO/'experiments/speech344/giga344-cli-episode.py')
        owner='set(openfst_URL "https://example.invalid/owner.tar.gz")\nset(openfst_HASH "SHA256='+('a'*64)+'")'
        with self.assertRaisesRegex(ValueError,'owning Sherpa'):
            cli.validate_openfst_pin(owner,{'url':'https://example.invalid/transitive.tar.gz','sha256':'b'*64})
        cli.validate_openfst_pin(owner,{'url':'https://example.invalid/owner.tar.gz','sha256':'a'*64})

    def test_cli_transcript_damage_is_refused(self):
        cli=module('cli344guard',REPO/'experiments/speech344/giga344-cli-episode.py')
        with tempfile.TemporaryDirectory(prefix='giga344-text-') as temp:
            p=Path(temp)/'stdout'
            for body in [b'{"text":"\xff"}',b'{"text":5}',b'{"text":"\xef\xbf\xbd"}',b'{"text":"one"}\n{"text":"two"}']:
                p.write_bytes(body)
                with self.assertRaises((ValueError,UnicodeError)):cli.read_text(p)
            p.write_text('{"text":"Проверь задачи. Please show the table."}')
            self.assertEqual(cli.read_text(p),'Проверь задачи. Please show the table.')
    def test_readiness_readback_keeps_original_ten_seconds(self):
        from unittest.mock import patch
        cli=module('clideadline344guard',REPO/'experiments/speech344/giga344-cli-episode.py')
        with tempfile.TemporaryDirectory(prefix='giga344-ready-') as temp:
            p=Path(temp)/'stdout';p.write_text('{"text":""}')
            with patch.object(cli.time,'monotonic',return_value=10.001):
                with self.assertRaises(TimeoutError):cli.readiness_result(p,0,str.split)
            with patch.object(cli.time,'monotonic',return_value=9.999):
                self.assertEqual(cli.readiness_result(p,0,str.split),('',9.999))

    def test_short_pcm_bytes_and_limit(self):
        cli=module('clipcm344guard',REPO/'experiments/speech344/giga344-cli-episode.py')
        with tempfile.TemporaryDirectory(prefix='giga344-pcm-') as temp:
            p=Path(temp)/'mixed.wav';raw=b'\x01\x00'*320+b'\x02\x00'*480
            cli.write_pcm(p,raw);self.assertEqual(cli.pcm(p),raw)
            for bad in [b'',b'\x01',b'\x00\x00'*400001]:
                with self.assertRaises(ValueError):cli.write_pcm(p,bad)
            self.assertEqual(cli.pcm(p),raw,'refusal must not overwrite earlier PCM')

if __name__=='__main__':unittest.main()
