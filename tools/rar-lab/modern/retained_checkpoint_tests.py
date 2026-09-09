"""Pure cloud reader tests; no downloads, files, adapters or VM execution."""
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import struct
import sys
import unittest
import zipfile

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")

def load(name):
    spec=importlib.util.spec_from_file_location("checkpoint_test_"+name,Path(__file__).with_name(name+".py"))
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module
reader=load("retained_checkpoint")

def fixture():
    comparison=load("reference_comparison")
    protocol,corpus=comparison.protocol,comparison.corpus
    answers={}
    for case in corpus.cases():
        value=case.value
        if value is None:
            _,length=struct.unpack_from("<HH",case.payload,44);value=bytes(length+16)
        request=protocol.request(case.operation,case.payload)
        answers[request]=(case.status,value)
        if case.operation==4:
            for derived in corpus.open_cases(case,value):
                answers[protocol.request(derived.operation,derived.payload)]=(derived.status,derived.value)
    members={}
    def execute(ident,request):
        status,value=answers[request];op,_=protocol.parse_request(request)
        output=b"RARMCO00"+bytes((op,status,ident))+bytes(5)+struct.pack("<I",len(value))+bytes(4)
        output+=hashlib.sha256(request).digest()+bytes(8)+value
        return ident,0,output,b""
    def retain(name,raw):
        members[name]=raw;return hashlib.sha256(raw).hexdigest()
    report=comparison._compare(execute,retain)
    fixed=reader.FIXED[1]
    manifest=dict(schema="rar-modern-crypto-handoff-v0",controller=fixed["controller"],
        source=fixed["source"],run=str(fixed["run"]),attempt="1",status="fixed-corpus-compared",
        phase="complete-fixed-corpus",crypto_interoperability_accepted=False,
        milestone_complete=False,target_os_execution=False,comparison=report,source_binding={})
    return members,manifest

def bind(members,manifest):
    manifest=copy.deepcopy(manifest)
    manifest["evidence_files"]={name:dict(size=len(raw),sha256=hashlib.sha256(raw).hexdigest())
                               for name,raw in members.items()}
    return dict(members,**{"manifest.json":reader.canonical(manifest)})

class Tests(unittest.TestCase):
    def test_recorded_corpus_replays_without_adapter_calls(self):
        members,manifest=fixture()
        report=reader.crypto(bind(members,manifest),reader.FIXED[1])
        self.assertEqual(report["comparison"]["compared"],288)
        self.assertEqual(report["comparison"]["adapter_invocations"],864)
        self.assertFalse(report["execution_attempted"])
        self.assertFalse(report["crypto_interoperability_accepted"])

    def test_rehashed_semantic_corruption_is_not_accepted(self):
        original,manifest=fixture()
        for field,value in (("implementation",3),("exit_code",True),("stdout","00"),("stderr","00")):
            members=dict(original);results=json.loads(members["three-way-results.json"])
            results["cases"][0]["runs"][1][field]=value
            members["three-way-results.json"]=reader.canonical(results)
            with self.assertRaises(ValueError):reader.crypto(bind(members,manifest),reader.FIXED[1])
        members=dict(original);frozen=json.loads(members["frozen-rar-results.json"])
        frozen["cases"][0]["request"]="00"
        members["frozen-rar-results.json"]=reader.canonical(frozen)
        with self.assertRaises(ValueError):reader.crypto(bind(members,manifest),reader.FIXED[1])
        for field,value in (("source","a"*40),("status","failed"),("milestone_complete",True)):
            bad=dict(manifest);bad[field]=value
            with self.assertRaises(ValueError):reader.crypto(bind(original,bad),reader.FIXED[1])
        bad=copy.deepcopy(manifest);bad["comparison"]["compared"]=287
        with self.assertRaises(ValueError):reader.crypto(bind(original,bad),reader.FIXED[1])
        bad=bind(original,manifest);bad["unlisted.txt"]=b"extra"
        with self.assertRaises(ValueError):reader.crypto(bad,reader.FIXED[1])

    def test_fixed_receipts_require_exact_success_identity(self):
        for fixed in reader.FIXED:
            repo=dict(id=1302587720,full_name="AndyTechCoder/RAR-OS")
            metadata=dict(id=fixed["artifact"],size_in_bytes=fixed["size"],
                digest="sha256:"+fixed["digest"],expired=False,
                name="modern-"+fixed["role"]+"-"+str(fixed["run"])+"-1",
                workflow_run=dict(id=fixed["run"],head_sha=fixed["controller"],head_branch="main",
                    repository_id=1302587720,head_repository_id=1302587720))
            run=dict(id=fixed["run"],head_sha=fixed["controller"],run_attempt=1,
                event="workflow_dispatch",head_branch="main",status="completed",conclusion="success",
                path=fixed["workflow"],repository=repo,head_repository=repo)
            reader.receipt(metadata,run,fixed)
            for field,value in (("id",True),("expired",True),("digest","sha256:"+"a"*64),
                                ("size_in_bytes",fixed["size"]+1),("name","wrong")):
                with self.assertRaises(ValueError):reader.receipt(dict(metadata,**{field:value}),run,fixed)
            for field,value in (("run_attempt",2),("conclusion","failure"),("status","in_progress"),
                                ("path","other"),("head_sha","a"*40)):
                with self.assertRaises(ValueError):reader.receipt(metadata,dict(run,**{field:value}),fixed)

    def test_archive_is_inert_bounded_and_identity_checked_first(self):
        def archive(name="manifest.json",mode=stat.S_IFREG|0o600):
            stream=io.BytesIO()
            with zipfile.ZipFile(stream,"w") as out:
                entry=zipfile.ZipInfo(name);entry.external_attr=mode<<16
                out.writestr(entry,b"{}")
            return stream.getvalue()
        def inspect(raw):
            fixed=dict(size=len(raw),digest=hashlib.sha256(raw).hexdigest())
            return reader.archive(raw,fixed)
        self.assertEqual(inspect(archive()),{"manifest.json":b"{}"})
        for raw in (archive("../outside"),archive(mode=stat.S_IFLNK|0o777),archive("other")):
            with self.assertRaises(ValueError):inspect(raw)
        with self.assertRaises(ValueError):reader.archive(b"not ZIP",reader.FIXED[0])
        for raw in (b'{"a":1,"a":2}',b'{"a":NaN}',b'{"a":Infinity}'):
            with self.assertRaises(ValueError):reader.unique(raw)
        self.assertEqual(len(reader.canonical({"text":"::error::inert\ntext"}).splitlines()),1)

if __name__=="__main__":unittest.main(verbosity=2)
