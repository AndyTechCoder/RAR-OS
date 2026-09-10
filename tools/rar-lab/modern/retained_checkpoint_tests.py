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
from unittest import mock
from types import SimpleNamespace
import zipfile

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")

def load(name):
    spec=importlib.util.spec_from_file_location("checkpoint_test_"+name,Path(__file__).with_name(name+".py"))
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module
reader=load("retained_checkpoint")


def source_fixture():
    snapshot=load("source_snapshot")
    trees={};blobs={};root={}
    for path in sorted(snapshot.FILES):
        cursor=root;parts=path.split("/")
        for name in parts[:-1]:cursor=cursor.setdefault(name,{})
        cursor[parts[-1]]=("// inert "+path+"\n").encode("ascii")
    def encode(node):
        raw=bytearray()
        for name,value in sorted(node.items(),key=lambda row:(row[0]+("/" if type(row[1]) is dict else "")).encode()):
            if type(value) is dict:mode=b"40000";oid=encode(value)
            else:mode=b"100644";oid=snapshot._git_hash(b"blob",value);blobs[oid]=value
            raw.extend(mode+b" "+name.encode()+b"\0"+bytes.fromhex(oid))
        raw=bytes(raw);oid=snapshot._git_hash(b"tree",raw);trees[oid]=raw
        return oid
    tree=encode(root)
    commit=("tree "+tree+"\nauthor RAR <lab@example.invalid> 1 +0000\n"
            "committer RAR <lab@example.invalid> 1 +0000\n\nfixture\n").encode()
    revision=snapshot._git_hash(b"commit",commit)
    layer,report=snapshot.build_from_objects(revision,commit,trees,blobs)
    members={"source-commit-"+revision+".bin":commit,"source-layer.tar":layer,
             "source-binding.json":reader.canonical(report)}
    for kind,objects in (("tree",trees),("blob",blobs)):
        members.update({"source-"+kind+"-"+oid+".bin":raw for oid,raw in objects.items()})
    return members,report,dict(reader.FIXED[1],source=revision)

SOURCE_MEMBERS,SOURCE_REPORT,CRYPTO_FIXED=source_fixture()

def baseline_fixture():
    content={"fixture":"validator-result"}
    raw=reader.canonical({"vm_proofs":[{"cut":{"backends":[{"records":[{"fixture":i}]}]}} for i in (1,2)]})
    names=[row[0] for row in load("runtime_evidence").refusal_cases()]
    refusals=dict(schema="rar-modern-actual-refusals-v1",base_sha256=hashlib.sha256(raw).hexdigest(),
        rejected=60,cases=[dict(case=name,input_sha256="a"*64,rejection_sha256="b"*64) for name in names],
        disk_faults_tested=False,milestone_complete=False)
    tools="".join("a"*64+"  "+path+"\n" for path in
        ("/usr/bin/python3.11","/usr/bin/qemu-system-x86_64",
         "/usr/share/OVMF/OVMF_CODE.fd","/usr/share/OVMF/OVMF_VARS.fd"))+"1966080\n131072\n"
    members={"modern.efi":b"inert kernel","modern-service.efi":b"inert service",
             "boot.img":b"inert boot","tool-identities.txt":tools.encode(),
             "persistence.json":raw,"refusals.json":(json.dumps(refusals,indent=2)+"\n").encode()}
    fixed=reader.FIXED[0]
    manifest=dict(source=fixed["source"],controller=fixed["controller"],run=str(fixed["run"]),
        attempt="1",status="observed",milestone_complete=False,crypto_interoperability_accepted=False,
        boot_sha256=hashlib.sha256(members["boot.img"]).hexdigest(),
        binaries={name:hashlib.sha256(members[name]).hexdigest() for name in ("modern.efi","modern-service.efi")},
        persistence_sha256=hashlib.sha256(raw).hexdigest(),content_check=content,actual_refusals=refusals)
    members["manifest.json"]=reader.canonical(manifest)
    return members,content

def fixture(seed=None):
    comparison=load("reference_comparison")
    protocol,corpus=comparison.protocol,comparison.corpus
    answers={}
    for case in corpus.cases()+(() if seed is None else corpus.challenge_cases(seed)):
        value=case.value
        if value is None:
            _,length=struct.unpack_from("<HH",case.payload,44);value=bytes(length+16)
        request=protocol.request(case.operation,case.payload)
        answers[request]=(case.status,value)
        if case.operation==4:
            for derived in corpus.open_cases(case,value):
                answers[protocol.request(derived.operation,derived.payload)]=(derived.status,derived.value)
    members=dict(SOURCE_MEMBERS)
    def execute(ident,request):
        status,value=answers[request];op,_=protocol.parse_request(request)
        output=b"RARMCO00"+bytes((op,status,ident))+bytes(5)+struct.pack("<I",len(value))+bytes(4)
        output+=hashlib.sha256(request).digest()+bytes(8)+value
        return ident,0,output,b""
    def retain(name,raw):
        members[name]=raw;return hashlib.sha256(raw).hexdigest()
    report=comparison._compare(execute,retain,seed)
    fixed=CRYPTO_FIXED
    manifest=dict(schema="rar-modern-crypto-handoff-v0",controller=fixed["controller"],
        source=fixed["source"],run=str(fixed["run"]),attempt="1",status="challenge-corpus-compared" if seed is not None else "fixed-corpus-compared",
        phase="complete-challenge-corpus" if seed is not None else "complete-fixed-corpus",crypto_interoperability_accepted=False,
        milestone_complete=False,target_os_execution=False,comparison=report,source_binding=SOURCE_REPORT)
    return members,manifest

def bind(members,manifest):
    manifest=copy.deepcopy(manifest)
    manifest["evidence_files"]={name:dict(size=len(raw),sha256=hashlib.sha256(raw).hexdigest())
                               for name,raw in members.items()}
    return dict(members,**{"manifest.json":reader.canonical(manifest)})

class Tests(unittest.TestCase):
    def test_explicit_challenge_reader_preserves_historical_default(self):
        members,manifest=fixture(bytes(range(32)))
        bound=bind(members,manifest)
        result=reader.crypto(bound,CRYPTO_FIXED,challenge=True)
        self.assertEqual(result["comparison"]["compared"],324)
        self.assertEqual(result["comparison"]["adapter_invocations"],972)
        self.assertFalse(result["execution_attempted"])
        with self.assertRaises(ValueError):reader.crypto(bound,CRYPTO_FIXED)
        with self.assertRaises(ValueError):reader.crypto(bound,CRYPTO_FIXED,challenge=1)
        old,old_manifest=fixture()
        with self.assertRaises(ValueError):reader.crypto(bind(old,old_manifest),CRYPTO_FIXED,challenge=True)


    def test_source_binding_requires_actual_git_object_closure(self):
        members,manifest=fixture()
        report=reader.crypto(bind(members,manifest),CRYPTO_FIXED)
        self.assertEqual(report["source_binding"],SOURCE_REPORT)
        for prefix in ("source-commit-","source-tree-","source-blob-"):
            name=next(name for name in members if name.startswith(prefix))
            for replacement in (None,b"invalid"):
                bad=dict(members)
                if replacement is None:bad.pop(name)
                else:bad[name]=replacement
                with self.assertRaises((ValueError,KeyError)):
                    reader.crypto(bind(bad,manifest),CRYPTO_FIXED)
        for name in ("source-layer.tar","source-binding.json"):
            bad=dict(members);bad[name]+=b"\n"
            with self.assertRaises(ValueError):reader.crypto(bind(bad,manifest),CRYPTO_FIXED)
        bad=dict(members);bad["source-tree-"+("a"*40)+".bin"]=b"extra"
        with self.assertRaises(ValueError):reader.crypto(bind(bad,manifest),CRYPTO_FIXED)
        bad=copy.deepcopy(manifest);bad["source_binding"]["target_git_sha"]="b"*40
        with self.assertRaises(ValueError):reader.crypto(bind(members,bad),CRYPTO_FIXED)

    def test_baseline_wires_validator_hashes_tools_refusals_and_inventory(self):
        original,content=baseline_fixture()
        validator=SimpleNamespace(validate=mock.Mock(return_value=content),
            refusal_cases=load("runtime_evidence").refusal_cases)
        original_helper=reader.helper
        def helper(name):return validator if name=="runtime_evidence" else original_helper(name)
        with mock.patch.object(reader,"helper",side_effect=helper):
            result=reader.baseline(original,reader.FIXED[0])
            self.assertEqual(result["data_records"],[[{"fixture":1}],[{"fixture":2}]])
            validator.validate.assert_called_once_with(original["persistence.json"],
                hashlib.sha256(original["boot.img"]).hexdigest(),(1966080,131072))
            tools=original["tool-identities.txt"]
            lines=tools.splitlines()
            bad_tools=[tools.rstrip(b"\n"),tools.replace(b"\n",b"\r\n"),
                tools.replace(b"  /",b" /",1),tools.replace(b"/usr/bin/python3.11",b"/tmp/python3.11"),
                tools.replace(b"1966080",b"01966080"),tools.replace(b"131072",b"0"),
                b"\n".join([lines[1],lines[0]]+lines[2:])+b"\n",
                b"A"+tools[1:],tools+b"extra\n"]
            for value in bad_tools:
                with self.subTest(tool=value[:80]),self.assertRaises(ValueError):
                    reader.baseline(dict(original,**{"tool-identities.txt":value}),reader.FIXED[0])
            for name in ("boot.img","modern.efi","modern-service.efi","persistence.json"):
                bad=dict(original);bad[name]+=b"bad"
                with self.assertRaises(ValueError):reader.baseline(bad,reader.FIXED[0])
            bad=dict(original);bad["extra"]=b""
            with self.assertRaises(ValueError):reader.baseline(bad,reader.FIXED[0])
            for change in ("row-extra","row-hash","row-case","extra","order","compact","row-order"):
                bad=dict(original);record=json.loads(bad["refusals.json"])
                if change=="row-extra":record["cases"][0]["extra"]=1
                if change=="row-hash":record["cases"][0]["input_sha256"]="A"*64
                if change=="row-case":record["cases"][0]["case"]="wrong"
                if change=="extra":record["extra"]=1
                if change=="order":record=dict(reversed(list(record.items())))
                if change=="row-order":record["cases"][0]=dict(reversed(list(record["cases"][0].items())))
                bad["refusals.json"]=(reader.canonical(record) if change=="compact" else
                    (json.dumps(record,indent=2)+"\n").encode())
                manifest=json.loads(bad["manifest.json"]);manifest["actual_refusals"]=record
                bad["manifest.json"]=reader.canonical(manifest)
                with self.subTest(refusal=change),self.assertRaises(ValueError):
                    reader.baseline(bad,reader.FIXED[0])

    def test_main_downloads_both_then_clears_token_before_parsing(self):
        # All environment, signals, limits, clients and parsers are inert mocks.
        for failure in (None,"download","receipt","archive","parse"):
            events=[];clients=[]
            class Client:
                def __init__(self,token):self.token=token;clients.append(self)
                def metadata(self,path):
                    self_check.assertEqual(self.token,"test-token")
                    events.append("metadata");return {}
                def zip(self,artifact,size):
                    self_check.assertEqual(self.token,"test-token")
                    events.append("download")
                    if failure=="download" and events.count("download")==2:raise RuntimeError("secret")
                    return b"inert"
            self_check=self
            def receipt(*args):
                events.append("receipt")
                if failure=="receipt":raise RuntimeError("secret")
            def archive(*args):
                self.assertEqual(events.count("download"),2)
                self.assertEqual(clients[0].token,"")
                self.assertNotIn("RAR_ARTIFACT_TOKEN",reader.os.environ)
                events.append("archive")
                if failure=="archive":raise RuntimeError("secret")
                return {}
            def parse(*args):
                self.assertEqual(clients[0].token,"");events.append("parse")
                if failure=="parse":raise RuntimeError("secret")
                return {"execution_attempted":False}
            handoff=SimpleNamespace(REPO="AndyTechCoder/RAR-OS",GitHub=Client,
                guard=lambda:(None,None,reader.FIXED[1]["source"],{"CI":"true"}))
            with mock.patch.object(reader,"helper",return_value=handoff), \
                 mock.patch.dict(reader.os.environ,{"RAR_ARTIFACT_TOKEN":"test-token"},clear=True), \
                 mock.patch.object(reader.resource,"setrlimit") as limits, \
                 mock.patch.object(reader.signal,"signal"),mock.patch.object(reader.signal,"alarm") as alarm, \
                 mock.patch.object(reader,"receipt",side_effect=receipt), \
                 mock.patch.object(reader,"archive",side_effect=archive), \
                 mock.patch.object(reader,"baseline",side_effect=parse),mock.patch.object(reader,"crypto",side_effect=parse), \
                 mock.patch("builtins.print") as printed:
                if failure:
                    with self.assertRaises(reader.Invalid) as result:reader.main()
                    self.assertNotIn("secret",str(result.exception));printed.assert_not_called()
                else:
                    reader.main()
                    self.assertEqual(printed.call_count,2)
                    self.assertEqual(events[-4:],["archive","parse","archive","parse"])
                self.assertEqual(clients[0].token,"")
                self.assertEqual(limits.call_count,4)
                self.assertEqual(alarm.call_args,mock.call(0))
                if failure in ("download","receipt"):
                    self.assertNotIn("archive",events);self.assertNotIn("parse",events)

    def test_archive_rejects_duplicate_encryption_compression_and_size_headers(self):
        stream=io.BytesIO()
        with zipfile.ZipFile(stream,"w") as out:out.writestr("manifest.json",b"{}")
        original=stream.getvalue()
        def inspect(raw):
            return reader.archive(raw,dict(size=len(raw),digest=hashlib.sha256(raw).hexdigest()))
        central=original.index(b"PK\x01\x02")
        for offset,fmt,value in ((8,"<H",1),(10,"<H",99),(24,"<I",32*1024**2+1)):
            raw=bytearray(original);struct.pack_into(fmt,raw,central+offset,value)
            with self.assertRaises(ValueError):inspect(bytes(raw))
        # Clone the member's central directory entry to test duplicate names
        # without a duplicate-producing write or decompression allocation.
        end=original.index(b"PK\x05\x06")
        entry=original[central:end]
        raw=bytearray(original[:end]+entry+original[end:])
        new_end=end+len(entry)
        struct.pack_into("<HH",raw,new_end+8,2,2)
        struct.pack_into("<I",raw,new_end+12,len(entry)*2)
        with self.assertRaises(ValueError):inspect(bytes(raw))
        # Mock only the ZIP framing to exercise aggregate/count bounds before reads.
        entries=[]
        for n in range(9):
            entry=zipfile.ZipInfo("member"+str(n));entry.file_size=32*1024**2
            entries.append(entry)
        fake=mock.MagicMock();fake.__enter__.return_value=fake
        fake.infolist.return_value=entries
        payload=mock.MagicMock();payload.__len__.return_value=32*1024**2
        fake.read.return_value=payload
        with mock.patch.object(reader.zipfile,"ZipFile",return_value=fake):
            with self.assertRaisesRegex(ValueError,"bounded decompressed"):inspect(original)
            self.assertEqual(fake.read.call_count,8)
            fake.infolist.return_value=[entries[0]]*7001
            with self.assertRaisesRegex(ValueError,"bounded member count"):inspect(original)

    def test_recorded_corpus_replays_without_adapter_calls(self):
        members,manifest=fixture()
        report=reader.crypto(bind(members,manifest),CRYPTO_FIXED)
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
            with self.assertRaises(ValueError):reader.crypto(bind(members,manifest),CRYPTO_FIXED)
        members=dict(original);frozen=json.loads(members["frozen-rar-results.json"])
        frozen["cases"][0]["request"]="00"
        members["frozen-rar-results.json"]=reader.canonical(frozen)
        with self.assertRaises(ValueError):reader.crypto(bind(members,manifest),CRYPTO_FIXED)
        for field,value in (("source","a"*40),("status","failed"),("milestone_complete",True)):
            bad=dict(manifest);bad[field]=value
            with self.assertRaises(ValueError):reader.crypto(bind(original,bad),CRYPTO_FIXED)
        bad=copy.deepcopy(manifest);bad["comparison"]["compared"]=287
        with self.assertRaises(ValueError):reader.crypto(bind(original,bad),CRYPTO_FIXED)
        bad=bind(original,manifest);bad["unlisted.txt"]=b"extra"
        with self.assertRaises(ValueError):reader.crypto(bad,CRYPTO_FIXED)

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
