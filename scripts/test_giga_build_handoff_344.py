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

if __name__=='__main__':unittest.main()
