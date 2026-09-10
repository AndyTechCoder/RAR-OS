"""Inert cloud source tests for exact crypto artifact acquisition and replay."""
import copy
import hashlib
import importlib.util
import io
import os
from pathlib import Path
import sys
import unittest
from unittest import mock
from types import SimpleNamespace
import zipfile

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")

def load(name):
    spec=importlib.util.spec_from_file_location("inspection_test_"+name,Path(__file__).with_name(name+".py"))
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module

check=load("crypto_inspection")
fixtures=load("retained_checkpoint_tests")

def metadata():
    fixed=fixtures.CRYPTO_FIXED;run_id=fixed["run"];artifact=fixed["artifact"]
    repo=dict(id=1302587720,full_name="AndyTechCoder/RAR-OS")
    run=dict(id=run_id,head_sha=fixed["controller"],run_attempt=1,event="workflow_dispatch",
        head_branch="main",status="completed",conclusion="success",
        path=".github/workflows/modern-crypto.yml",repository=repo,head_repository=repo)
    meta=dict(id=artifact,size_in_bytes=123,digest="sha256:"+"a"*64,expired=False,
        name="modern-crypto-"+str(run_id)+"-1",workflow_run=dict(id=run_id,
        head_sha=fixed["controller"],head_branch="main",repository_id=1302587720,
        head_repository_id=1302587720))
    return meta,run,fixed

class Tests(unittest.TestCase):
    def test_exact_success_receipt_and_rejections(self):
        meta,run,fixed=metadata()
        def accept(m=meta,r=run):
            return check.receipt(m,r,fixed["run"],fixed["artifact"],fixed["source"])
        self.assertEqual(accept()["controller"],fixed["controller"])
        for key,value in (("run_attempt",2),("status","in_progress"),("conclusion","failure"),
            ("event","push"),("head_branch","other"),("path",".github/workflows/desktop.yml"),
            ("id",True),("head_sha","main")):
            with self.subTest(key=key),self.assertRaises(ValueError):accept(r=dict(run,**{key:value}))
        for key,value in (("id",True),("expired",True),("size_in_bytes",True),
            ("size_in_bytes",32*1024**2+1),("digest","sha256:"+"A"*64),("name","other")):
            with self.subTest(key=key),self.assertRaises(ValueError):accept(m=dict(meta,**{key:value}))
        bad=copy.deepcopy(meta);bad["workflow_run"]["id"]+=1
        with self.assertRaises(ValueError):accept(m=bad)
        bad=copy.deepcopy(run);bad["repository"]["full_name"]="other/repo"
        with self.assertRaises(ValueError):accept(r=bad)
        for value in (None,True,"0","01","-1","1/zip","1"*19):
            with self.assertRaises(ValueError):check.number(value)
        self.assertEqual(check.number("123"),123)

    def test_actual_inert_archive_regenerates_v1_and_rejects_v0(self):
        for seed in (bytes(range(32)),None):
            members,manifest=fixtures.fixture(seed)
            members=fixtures.bind(members,manifest)
            stream=io.BytesIO()
            with zipfile.ZipFile(stream,"w",compression=zipfile.ZIP_DEFLATED) as archive:
                for name,raw in members.items():archive.writestr(name,raw)
            raw=stream.getvalue()
            binding=dict(fixtures.CRYPTO_FIXED,size=len(raw),digest=hashlib.sha256(raw).hexdigest())
            if seed is None:
                with self.assertRaises(ValueError):check.inspect(raw,binding)
            else:
                report=fixtures.reader.unique(check.inspect(raw,binding))
                self.assertEqual(report["comparison"]["compared"],324)
                self.assertEqual(report["comparison"]["adapter_invocations"],972)
                self.assertFalse(report["execution_attempted"])
                self.assertFalse(report["crypto_interoperability_accepted"])
                with self.assertRaises(ValueError):check.inspect(raw+b"x",binding)
                bad=dict(binding,source="a"*40)
                with self.assertRaises(ValueError):check.inspect(raw,bad)

    def test_acquisition_clears_token_before_parse_and_redacts_failures(self):
        meta,run,fixed=metadata()
        for failure in (None,"metadata","download","parse"):
            events=[]
            class Client:
                token="fixture-secret"
                def metadata(self,path):
                    self_check.assertEqual(self.token,"fixture-secret");events.append(path)
                    if failure=="metadata":raise RuntimeError("signed-private-url")
                    return run if "/runs/" in path else meta
                def zip(self,artifact,size):
                    self_check.assertEqual(self.token,"fixture-secret")
                    self_check.assertEqual((artifact,size),(fixed["artifact"],123));events.append("download")
                    if failure=="download":raise RuntimeError("signed-private-url")
                    return b"inert"
            self_check=self;client=Client()
            def inspect(raw,binding):
                self.assertEqual(client.token,"");self.assertEqual(raw,b"inert");events.append("parse")
                if failure=="parse":raise RuntimeError("signed-private-url")
                return b"report"
            with mock.patch.object(check,"inspect",side_effect=inspect):
                if failure:
                    with self.assertRaisesRegex(ValueError,"^retained crypto inspection failed$"):
                        check.acquire(client,fixed["run"],fixed["artifact"],fixed["source"])
                else:
                    self.assertEqual(check.acquire(client,fixed["run"],fixed["artifact"],fixed["source"]),b"report")
                    self.assertEqual(events[-2:],["download","parse"])
            self.assertEqual(client.token,"")
            if failure in ("metadata","download"):self.assertNotIn("parse",events)

    def test_host_guard_precedes_client_or_limits(self):
        with mock.patch.dict(os.environ,{},clear=True),mock.patch.object(check,"acquire") as acquire, \
             mock.patch.object(check.resource,"setrlimit") as limit:
            with self.assertRaises(Exception):check.main()
            acquire.assert_not_called();limit.assert_not_called()

    def test_v1_archive_count_is_explicit_and_still_bounded(self):
        reader=fixtures.reader
        raw=b"inert";fixed=dict(size=len(raw),digest=hashlib.sha256(raw).hexdigest())
        fake=mock.MagicMock();fake.__enter__.return_value=fake
        fake.infolist.return_value=[object()]*10001
        with mock.patch.object(reader.zipfile,"ZipFile",return_value=fake):
            with self.assertRaisesRegex(ValueError,"bounded member count"):reader.archive(raw,fixed,challenge=True)
            fake.read.assert_not_called()
        with self.assertRaises(ValueError):reader.archive(raw,fixed,challenge=1)

if __name__=="__main__":
    unittest.main(argv=[sys.argv[0]],verbosity=2)
