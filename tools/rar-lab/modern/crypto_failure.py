"""Read one fixed retained cloud failure; no extraction/build/adapter/VM execution."""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import resource
import signal
import stat
import sys
import zipfile

ARTIFACT=10096184040
RUN=34332066725
SIZE=387106
DIGEST="bd5ee6fe146338fa72f3a6349a26d4705b93345a2b7c48f5dc26be30a6b19451"
CONTROLLER="cb863b78ef914f9a1d9e2c2dddf704928e0a91f2"
SOURCE="bfd8647b10e1daa38934b1e12363ba919938bbf7"
class Invalid(ValueError):pass

def unique(raw):
    def pairs(rows):
        out={}
        for key,value in rows:
            if key in out:raise Invalid("duplicate key")
            out[key]=value
        return out
    def constant(value):raise Invalid("nonfinite JSON")
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)

def fixed_fields(actual,expected):
    if type(actual) is not dict:raise Invalid("typed receipt object")
    for key,want in expected.items():
        if key not in actual:raise Invalid("missing receipt field")
        value=actual[key]
        if type(want) is dict:fixed_fields(value,want)
        elif type(value) is not type(want) or value!=want:
            raise Invalid("strict receipt field type/value")

def receipt_expectations():
    repository={"id":1302587720,"full_name":"AndyTechCoder/RAR-OS"}
    metadata={"id":ARTIFACT,"size_in_bytes":SIZE,"digest":"sha256:"+DIGEST,
        "expired":False,"name":"modern-crypto-"+str(RUN)+"-1",
        "workflow_run":{"id":RUN,"head_sha":CONTROLLER,"head_branch":"main",
            "repository_id":1302587720,"head_repository_id":1302587720}}
    run={"id":RUN,"head_sha":CONTROLLER,"run_attempt":1,"event":"workflow_dispatch",
        "head_branch":"main","status":"completed","conclusion":"failure",
        "path":".github/workflows/modern-crypto.yml",
        "repository":dict(repository),"head_repository":dict(repository)}
    return metadata,run

def receipt(metadata,run):
    expected_metadata,expected_run=receipt_expectations()
    fixed_fields(metadata,expected_metadata);fixed_fields(run,expected_run)

def inspect_archive(raw,expected_sha=DIGEST,expected_size=SIZE):
    # Optional expectations are for pure synthetic tests only. The cloud
    # entrypoint always uses the fixed receipt above.
    if (type(raw) is not bytes or len(raw)!=expected_size or
        hashlib.sha256(raw).hexdigest()!=expected_sha):
        raise Invalid("whole ZIP identity before parsing")
    members={}
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        entries=archive.infolist()
        if not 1<=len(entries)<=512:raise Invalid("member count")
        total=0
        for entry in entries:
            name=entry.filename
            mode=entry.external_attr>>16
            if (re.fullmatch("[a-z0-9][a-z0-9._-]{0,127}",name) is None or
                name in members or entry.is_dir() or entry.flag_bits&1 or
                stat.S_IFMT(mode) not in (0,stat.S_IFREG) or
                entry.compress_type not in (zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED) or
                not 0<=entry.file_size<=8*1024**2):
                raise Invalid("flat regular bounded inert evidence")
            total+=entry.file_size
            if total>64*1024**2:raise Invalid("total evidence bound")
            data=archive.read(entry)
            if len(data)!=entry.file_size:raise Invalid("complete member")
            members[name]=data
    raw_manifest=members.pop("manifest.json",None)
    if raw_manifest is None or len(raw_manifest)>1024**2:raise Invalid("bounded manifest")
    manifest=unique(raw_manifest)
    if (type(manifest) is not dict or
        manifest.get("schema")!="rar-modern-crypto-handoff-v0" or
        manifest.get("controller")!=CONTROLLER or manifest.get("source")!=SOURCE or
        manifest.get("run")!=str(RUN) or manifest.get("attempt")!="1" or
        manifest.get("status")!="failed" or manifest.get("phase")!="driver-construction" or
        manifest.get("milestone_complete") is not False or
        manifest.get("target_os_execution") is not False):
        raise Invalid("exact nonaccepting failure manifest")
    inventory=manifest.get("evidence_files")
    if type(inventory) is not dict or set(inventory)!=set(members):
        raise Invalid("complete retained inventory")
    for name,data in members.items():
        record=inventory[name]
        if (type(record) is not dict or set(record)!={"sha256","size"} or
            type(record["size"]) is not int or record["size"]!=len(data) or
            record["sha256"]!=hashlib.sha256(data).hexdigest()):
            raise Invalid("member inventory binding")
    commands=sorted(int(m.group(1)) for name in members
        if (m:=re.fullmatch(r"command-([1-9][0-9]*)-argv\.json",name)))
    if not commands or commands!=list(range(1,max(commands)+1)):
        raise Invalid("complete command sequence")
    selected=[]
    for number in commands[-3:]:
        prefix="command-"+str(number)
        command={}
        for suffix in ("argv.json","result.json","failure.json","stdout.bin","stderr.bin",
                       "partial_stdout.bin","partial_stderr.bin"):
            name=prefix+"-"+suffix
            if name not in members:continue
            data=members[name]
            command[suffix]={"sha256":hashlib.sha256(data).hexdigest(),"size":len(data),
                "truncated":len(data)>32768,"text":data[:32768].decode("utf-8","backslashreplace")}
        selected.append({"ordinal":number,"evidence":command})
    summary={key:manifest.get(key) for key in
        ("schema","controller","source","run","attempt","status","phase",
         "error_type","validation_error","controller_maxrss_kib","milestone_complete","target_os_execution")}
    return {"inspection":"fixed-retained-failure-v0","artifact":ARTIFACT,
        "artifact_sha256":expected_sha,"artifact_size":expected_size,
        "manifest":summary,"last_commands":selected,"execution_attempted":False}

def render(report):
    # One JSON object, not raw command output or GitHub workflow-command lines.
    raw=json.dumps(report,sort_keys=True,ensure_ascii=True,allow_nan=False)
    if len(raw.encode())>2*1024**2:raise Invalid("bounded diagnostic output")
    return raw

def main():
    path=Path(__file__).with_name("crypto_handoff.py")
    if path.is_symlink() or not path.is_file():raise Invalid("trusted helper")
    spec=importlib.util.spec_from_file_location("rar_failure_handoff",path)
    handoff=importlib.util.module_from_spec(spec);spec.loader.exec_module(handoff)
    _,_,target,required=handoff.guard()
    if target!=SOURCE:raise Invalid("fixed diagnostic source")
    token=os.environ.get("RAR_ARTIFACT_TOKEN")
    os.environ.clear();os.environ.update({**required,"PATH":"/usr/bin:/bin","LC_ALL":"C"})
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    resource.setrlimit(resource.RLIMIT_AS,(256*1024**2,256*1024**2))
    resource.setrlimit(resource.RLIMIT_FSIZE,(0,0))
    resource.setrlimit(resource.RLIMIT_CPU,(30,30))
    def deadline(*args):raise Invalid("bounded inspection deadline")
    signal.signal(signal.SIGALRM,deadline);signal.alarm(120)
    client=handoff.GitHub(token);token=None
    try:
        metadata=client.metadata("/repos/"+handoff.REPO+"/actions/artifacts/"+str(ARTIFACT))
        run=client.metadata("/repos/"+handoff.REPO+"/actions/runs/"+str(RUN))
        receipt(metadata,run)
        raw=client.zip(ARTIFACT,SIZE)
        client.token=""
        print(render(inspect_archive(raw)),flush=True)
    except BaseException as error:
        # Only this module's fixed parser errors are safe to explain; HTTP
        # exceptions can contain credential-bearing redirect URLs.
        detail=str(error)[:512] if type(error) is Invalid else type(error).__name__
        raise Invalid("fixed cloud failure inspection failed ("+detail+")") from None
    finally:
        client.token="";signal.alarm(0)

def self_test():
    import unittest
    def archive(change=None,entry_change=None):
        data={"command-1-argv.json":b'["/usr/bin/docker","buildx","inspect","default"]',
              "command-1-stdout.bin":b"public diagnostic\n::error::not a workflow command"}
        manifest={"schema":"rar-modern-crypto-handoff-v0","controller":CONTROLLER,
            "source":SOURCE,"run":str(RUN),"attempt":"1","status":"failed",
            "phase":"driver-construction","milestone_complete":False,
            "target_os_execution":False,"validation_error":"public failure",
            "evidence_files":{name:{"sha256":hashlib.sha256(raw).hexdigest(),"size":len(raw)}
                              for name,raw in data.items()}}
        if change:change(manifest,data)
        data["manifest.json"]=json.dumps(manifest).encode()
        stream=io.BytesIO()
        with zipfile.ZipFile(stream,"w",compression=zipfile.ZIP_DEFLATED) as output:
            for name,raw in data.items():
                info=zipfile.ZipInfo(name);info.external_attr=(stat.S_IFREG|0o600)<<16
                if entry_change:entry_change(info)
                output.writestr(info,raw)
        return stream.getvalue()
    def inspect(raw):return inspect_archive(raw,hashlib.sha256(raw).hexdigest(),len(raw))
    class Tests(unittest.TestCase):
        def test_exact_inert_inspection_and_json_output(self):
            report=inspect(archive())
            self.assertFalse(report["execution_attempted"])
            self.assertEqual(report["manifest"]["status"],"failed")
            rendered=render(report)
            self.assertEqual(len(rendered.splitlines()),1)
            self.assertFalse(rendered.startswith("::"))
            self.assertIn("public diagnostic",rendered)
        def test_zip_identity_precedes_parse(self):
            with self.assertRaises(Invalid):inspect_archive(b"not a ZIP")
        def test_metadata_and_payload_mutations(self):
            for field,value in (("controller","a"*40),("source","b"*40),
                ("run","1"),("status","success"),("phase","complete"),
                ("milestone_complete",True),("target_os_execution",True)):
                with self.assertRaises(Invalid):
                    inspect(archive(lambda m,d:m.update({field:value})))
            with self.assertRaises(Invalid):
                inspect(archive(lambda m,d:d.update({"extra.bin":b"extra"})))
            with self.assertRaises(Invalid):
                inspect(archive(lambda m,d:d.update({"command-1-stdout.bin":b"changed"})))
        def test_member_path_link_and_size_refusal(self):
            for mutate in (lambda i:setattr(i,"filename","../outside"),
                           lambda i:setattr(i,"external_attr",(stat.S_IFLNK|0o777)<<16)):
                with self.assertRaises(Invalid):inspect(archive(entry_change=mutate))
        def test_receipt_exact_failure(self):
            import copy
            metadata,run=receipt_expectations()
            receipt(metadata,run)
            def leaves(value,path=()):
                for key,item in value.items():
                    if type(item) is dict:yield from leaves(item,path+(key,))
                    else:yield path+(key,),item
            for position,expected in enumerate((metadata,run)):
                for path,value in leaves(expected):
                    bads=((True,str(value),float(value),value+1) if type(value) is int else
                          (0,1,"false",None) if type(value) is bool else
                          (None,1,value+"x"))
                    for bad in bads:
                        pair=[copy.deepcopy(metadata),copy.deepcopy(run)]
                        item=pair[position]
                        for key in path[:-1]:item=item[key]
                        item[path[-1]]=bad
                        with self.subTest(position=position,path=path,bad=bad):
                            with self.assertRaises(Invalid):receipt(*pair)
                    pair=[copy.deepcopy(metadata),copy.deepcopy(run)]
                    item=pair[position]
                    for key in path[:-1]:item=item[key]
                    del item[path[-1]]
                    with self.assertRaises(Invalid):receipt(*pair)
            with self.assertRaises(Invalid):receipt([],run)
            with self.assertRaises(Invalid):receipt(metadata,{"repository":[]})
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():raise SystemExit(1)

if __name__=="__main__":
    if not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("isolated no-bytecode invocation")
    if sys.argv==[sys.argv[0],"--self-test"]:self_test()
    elif sys.argv==[sys.argv[0]]:main()
    else:raise SystemExit("fixed failure inspection only")
