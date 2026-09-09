"""Pure in-memory diagnostic tests; no network, files, subprocess, or VM."""
import copy
import hashlib
import importlib.util
import io
from pathlib import Path
import stat
import traceback
import unittest
import zipfile
from unittest.mock import patch

spec=importlib.util.spec_from_file_location("diagnostic",Path(__file__).with_name("fault_diagnostic.py"))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
r=d.helper("retained_checkpoint")
SOURCE="a"*40;CONTROLLER="b"*40
def fixture():
    repo=dict(id=1302587720,full_name="AndyTechCoder/RAR-OS")
    run=dict(id=1,head_sha=CONTROLLER,run_attempt=1,event="workflow_dispatch",head_branch="main",
        status="completed",conclusion="failure",path=".github/workflows/modern-data-faults.yml",
        repository=repo,head_repository=repo)
    manifest=dict(source=SOURCE,controller=CONTROLLER,run="1",attempt="1",milestone_complete=False,
        active_fault_case=0,status="failed",failure="unexpected event")
    capture=dict(schema="rar-modern-data-fault-candidate-v0",case=0,milestone_complete=False,
        plan={"effect":"before-cut"},initial_data_base64="MUST_NOT_PRINT",
        vm_proofs=[dict(events=[dict(event="BLOCK_IO_ERROR")],event_receipts=[],
            cut=dict(entry={},vm_returncode=-9)) for _ in range(2)])
    return run,manifest,capture

def packed(manifest,capture,extra=None):
    buf=io.BytesIO()
    with zipfile.ZipFile(buf,"w",compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("manifest.json",r.canonical(manifest))
        if capture is not None:archive.writestr("fault-00.json",r.canonical(capture))
        if extra is not None:archive.writestr(*extra)
    raw=buf.getvalue()
    binding=dict(run=1,artifact=2,source=SOURCE,controller=CONTROLLER,
        size=len(raw),digest=hashlib.sha256(raw).hexdigest())
    return raw,binding

class Tests(unittest.TestCase):
    def test_inert_projection_and_no_payload(self):
        run,manifest,capture=fixture();raw,binding=packed(manifest,capture)
        out=d.inspect(raw,binding);report=r.unique(out)
        self.assertFalse(report["content_accepted"])
        self.assertFalse(report["execution_attempted"])
        self.assertNotIn(b"MUST_NOT_PRINT",out)
        self.assertEqual(report["capture"]["vm_proofs"][0]["events"][0]["event"],"BLOCK_IO_ERROR")
    def test_missing_capture_is_diagnostic_not_acceptance(self):
        _,manifest,_=fixture();manifest.pop("active_fault_case")
        raw,binding=packed(manifest,None)
        self.assertIsNone(r.unique(d.inspect(raw,binding))["capture"])
    def test_manifest_and_capture_identity(self):
        _,manifest,capture=fixture()
        for key,value in (("source","c"*40),("run",1),("attempt",1),("milestone_complete",True),
                          ("active_fault_case",True),("active_fault_case",60)):
            bad=copy.deepcopy(manifest);bad[key]=value
            raw,binding=packed(bad,capture)
            with self.assertRaises(ValueError):d.inspect(raw,binding)
        capture["case"]=True
        raw,binding=packed(manifest,capture)
        with self.assertRaises(ValueError):d.inspect(raw,binding)
    def test_archive_boundaries_and_json(self):
        _,manifest,capture=fixture()
        link=zipfile.ZipInfo("tool-identities.txt")
        link.external_attr=(stat.S_IFLNK|0o777)<<16
        for extra in (("../outside",b"x"),("unknown",b"x"),(link,b"x"),
                      ("tool-identities.txt",b"x"*8193)):
            raw,binding=packed(manifest,capture,extra)
            with self.assertRaises(ValueError):d.inspect(raw,binding)
        raw,binding=packed(manifest,capture)
        with self.assertRaises(ValueError):d.inspect(raw+b"x",binding)
        for raw_json in (b'{"a":1,"a":2}',b'{"a":NaN}'):
            with self.assertRaises(ValueError):r.unique(raw_json)
    def test_identifiers(self):
        self.assertEqual(d.number("123"),123)
        for value in ("0","01","-1","1/2","1 "*2,"1"*19,1,True,None):
            with self.assertRaises(ValueError):d.number(value)
    def test_metadata_binding(self):
        run,manifest,capture=fixture();raw,binding=packed(manifest,capture)
        metadata=dict(id=2,size_in_bytes=len(raw),digest="sha256:"+binding["digest"],expired=False,
            name="modern-data-faults-1-1",workflow_run=dict(id=1,head_sha=CONTROLLER,head_branch="main",
                repository_id=1302587720,head_repository_id=1302587720))
        self.assertEqual(d.receipt(metadata,run,1,2,SOURCE),binding)
        for key,value in (("run_attempt",2),("event","pull_request"),("path","wrong"),
                          ("head_branch","other"),("id",True),("status","in_progress")):
            bad=copy.deepcopy(run);bad[key]=value
            with self.assertRaises(ValueError):d.receipt(metadata,bad,1,2,SOURCE)
        for key,value in (("expired",True),("size_in_bytes",True),("size_in_bytes",32*1024**2+1),
                          ("digest","x"),("name","other")):
            bad=copy.deepcopy(metadata);bad[key]=value
            with self.assertRaises(ValueError):d.receipt(bad,run,1,2,SOURCE)
        class Client:
            token="secret"
            def metadata(self,path):return run if "/runs/" in path else metadata
            def zip(self,artifact,size):return raw
        client=Client()
        real=d.inspect
        def inspect(data,bound):
            self.assertEqual(client.token,"")
            return real(data,bound)
        with patch.object(d,"inspect",inspect):d.acquire(client,1,2,SOURCE)
        client.token="secret"
        with patch.object(client,"zip",side_effect=ValueError("https://signed.invalid/SENTINEL?secret=URL")):
            try:d.acquire(client,1,2,SOURCE)
            except ValueError as error:
                displayed="".join(traceback.format_exception(error))
                self.assertNotIn("SENTINEL",displayed)
                self.assertNotIn("signed.invalid",displayed)
                self.assertTrue(error.__suppress_context__)
            else:self.fail("download failure accepted")
        self.assertEqual(client.token,"")

if __name__=="__main__":unittest.main(verbosity=2)
