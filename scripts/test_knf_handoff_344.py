#!/usr/bin/env python3
"""Offline guard checks only: no acquisition, compiler, accessor or numeric episode."""
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile

REPO = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('knf344', REPO/'experiments/speech344/knf-frontend-episode.py')
h = importlib.util.module_from_spec(spec); spec.loader.exec_module(h)


class HandoffGuards(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='knf344-guards-')
        self.root = Path(self.temp.name)
    def tearDown(self):
        self.temp.cleanup()
    def archive(self, names, kind='tar.gz'):
        p = self.root/('archive.tar.gz' if kind=='tar.gz' else 'archive.zip')
        if kind=='tar.gz':
            with tarfile.open(p,'w:gz') as f:
                d=tarfile.TarInfo('p/');d.type=tarfile.DIRTYPE;f.addfile(d)
                d=tarfile.TarInfo('p/src/');d.type=tarfile.DIRTYPE;f.addfile(d)
                for name,data,link in names:
                    m=tarfile.TarInfo(name);m.size=len(data)
                    if link:m.type=tarfile.SYMTYPE;m.linkname='/tmp/escape';m.size=0
                    f.addfile(m,io.BytesIO(data) if not link else None)
        else:
            with zipfile.ZipFile(p,'w') as f:
                for name,data,link in names:
                    m=zipfile.ZipInfo(name)
                    if link:m.external_attr=0o120777<<16
                    f.writestr(m,data)
        return p,{'sha256':h.sha(p),'format':kind,'prefix':'p',
                  'licenses':{'LICENSE':hashlib.sha256(b'license').hexdigest()}}
    def unpack(self,p,e,remaining=lambda:1):
        return h.unpack_verified(p,e,self.root/'source',self.root/'licenses',remaining)
    def test_valid_directory_and_license_chain(self):
        p,e=self.archive([('p/LICENSE',b'license',False),('p/src/a.cc',b'source',False)])
        result=self.unpack(p,e)
        self.assertEqual((self.root/'source/src/a.cc').read_bytes(),b'source')
        self.assertEqual((self.root/'licenses/LICENSE').read_bytes(),b'license')
        self.assertEqual(result['src/a.cc']['sha256'],hashlib.sha256(b'source').hexdigest())
    def test_tar_link_rejected_before_extraction(self):
        p,e=self.archive([('p/LICENSE',b'license',False),('p/escape',b'',True)])
        with self.assertRaises(ValueError):self.unpack(p,e)
        self.assertFalse((self.root/'source').exists())
    def test_zip_traversal_and_links_rejected(self):
        for name,link in [('p/../escape',False),('p/link',True)]:
            with self.subTest(name=name):
                p,e=self.archive([('p/LICENSE',b'license',False),(name,b'bad',link)],'zip')
                with self.assertRaises(ValueError):self.unpack(p,e)
                self.assertFalse((self.root/'source').exists())
    def test_full_digest_and_license_guard(self):
        p,e=self.archive([('p/LICENSE',b'wrong',False)])
        with self.assertRaisesRegex(ValueError,'license identity'):self.unpack(p,e)
        self.assertFalse((self.root/'source').exists())
        e['sha256']='0'*64
        with self.assertRaisesRegex(ValueError,'archive identity'):self.unpack(p,e)
    def test_no_deadline_reset_during_extraction(self):
        p,e=self.archive([('p/LICENSE',b'license',False)])
        def expired():raise TimeoutError('original shared deadline')
        with self.assertRaises(TimeoutError):self.unpack(p,e,expired)
        self.assertFalse((self.root/'source').exists())
    def test_pcm16_identity_precedes_scaling(self):
        raw=b'\x00\x00\x00\x40'
        row={'samples':2,'pcm16le_sha256':hashlib.sha256(raw).hexdigest()}
        h.verify_pcm(raw,row)
        with self.assertRaises(ValueError):h.verify_pcm(b'\x00'*4,row)
        with self.assertRaises(ValueError):h.verify_pcm(raw[:2],row)


if __name__=='__main__':unittest.main()
