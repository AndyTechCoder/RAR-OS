"""Pure inspection of retained successful crypto adapter lifecycles.
No process launch, container access, filesystem writes or acceptance of fault cases.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

class Invalid(ValueError):pass

def helper():
    path=Path(__file__).with_name("reference_runner.py")
    if path.is_symlink() or not path.is_file() or not 1<=path.stat().st_size<=131072:
        raise Invalid("fixed bounded trusted lifecycle helper")
    spec=importlib.util.spec_from_file_location("lifecycle_reference_runner",path)
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()+b"\n"

def document(raw,*,record=False):
    if type(raw) is not bytes or not 1<=len(raw)<=65536:raise Invalid("bounded lifecycle document")
    def pairs(items):
        out={}
        for k,v in items:
            if k in out:raise Invalid("duplicate lifecycle key")
            out[k]=v
        return out
    def constant(value):raise Invalid("nonfinite lifecycle JSON")
    try:
        value=json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)
        if record and canonical(value)!=raw:raise Invalid("canonical recorded lifecycle JSON")
        return value
    except (ValueError,UnicodeError,RecursionError) as error:
        raise Invalid("invalid lifecycle JSON") from error

def exact(actual,wanted):
    if canonical(actual)!=canonical(wanted):raise Invalid("typed exact lifecycle value")

def validate(members,manifest,frozen,results):
    """Caller first verifies receipt/inventory, source binding and full v1 replay."""
    if (type(members) is not dict or type(manifest) is not dict or
        type(frozen) is not dict or type(results) is not dict):
        raise Invalid("lifecycle evidence objects")
    for name,value in (("manifest.json",manifest),("frozen-rar-results.json",frozen),
                       ("three-way-results.json",results)):
        if members.get(name)!=canonical(value):raise Invalid("lifecycle member association")
    for obj,version in ((frozen,"rar-modern-frozen-comparison-v1"),
                        (results,"rar-modern-three-way-comparison-v1")):
        if obj.get("schema")!=version or type(obj.get("cases")) is not list or len(obj["cases"])!=324:
            raise Invalid("exact v1 lifecycle corpus")
    target=manifest.get("target_image");reference=manifest.get("reference_image")
    if any(type(x) is not str or re.fullmatch("sha256:[0-9a-f]{64}",x) is None for x in (target,reference)):
        raise Invalid("immutable lifecycle images")
    if target==reference:raise Invalid("distinct target and reference images")
    leaves=("request.bin","create.json","before.json","stdout.bin","stderr.bin","after.json","cleanup.json")
    expected={f"adapter-{i}-{leaf}" for i in range(1,973) for leaf in leaves}
    if {name for name in members if name.startswith("adapter-") and not name.startswith("adapter-image.")}!=expected:
        raise Invalid("complete exact lifecycle inventory")
    runner=helper();seen_ids=set();seen_names=set()
    digest=hashlib.sha256()
    for index in range(972):
        if index<324:
            case=index;ident=3;wire=frozen["cases"][case]["rar"]
        else:
            case,which=divmod(index-324,2);ident=which+1
            wire=results["cases"][case]["runs"][which+1]
        prefix=f"adapter-{index+1}-"
        def raw(leaf):
            value=members[prefix+leaf]
            if type(value) is not bytes or len(value)>65536:raise Invalid("bounded lifecycle member")
            return value
        request=raw("request.bin")
        if request.hex()!=frozen["cases"][case]["request"]:raise Invalid("lifecycle request binding")
        exact(wire,dict(implementation=ident,exit_code=0,
            stdout=raw("stdout.bin").hex(),stderr=raw("stderr.bin").hex()))
        before=document(raw("before.json"));after=document(raw("after.json"))
        if type(before) is not list or len(before)!=1 or type(before[0]) is not dict:
            raise Invalid("one before lifecycle object")
        cid=before[0].get("Id");name=before[0].get("Name")
        if (type(cid) is not str or re.fullmatch("[0-9a-f]{64}",cid) is None or
            type(name) is not str or re.fullmatch("/rar-modern-ref-[0-9a-f]{32}",name) is None or
            cid in seen_ids or name in seen_names):
            raise Invalid("unique owned lifecycle identities")
        seen_ids.add(cid);seen_names.add(name);name=name[1:]
        image=target if ident==3 else reference
        try:
            first=runner.owned_container(canonical(before),cid,name,image)
            last=runner.owned_container(canonical(after),cid,name,image)
            runner.confined_container(first);runner.confined_container(last)
        except runner.RunFailure as error:raise Invalid("retained lifecycle confinement") from error
        config=first["Config"];state=first.get("State")
        entry={1:"/reference-sodium",2:"/reference-openssl",3:"/target-reference"}[ident]
        if (type(state) is not dict or state.get("Status")!="created" or state.get("Running") is not False or
            config.get("User")!="65532:65532" or config.get("Entrypoint")!=[entry] or
            config.get("Cmd") not in (None,[]) or config.get("Env")!=["PATH=/nonexistent"] or
            config.get("WorkingDir")!="/"):
            raise Invalid("retained pre-start configuration")
        exact(last["Config"],config)
        state=last.get("State")
        wanted=dict(Status="exited",Running=False,Paused=False,Restarting=False,
                    OOMKilled=False,Dead=False,Error="",ExitCode=0,Pid=0)
        if type(state) is not dict:raise Invalid("retained completion state")
        exact({key:state.get(key) for key in wanted},wanted)
        exact(document(raw("create.json"),record=True),dict(exit_code=0,stdout=(cid+"\n").encode().hex(),stderr=""))
        exact(document(raw("cleanup.json"),record=True),dict(container_id=cid,confirmed_absent=True))
        for leaf in leaves:
            digest.update((prefix+leaf).encode()+b"\0"+hashlib.sha256(raw(leaf)).digest())
    return dict(schema="rar-modern-successful-crypto-lifecycles-v1",invocations=972,
        unique_containers=len(seen_ids),confinement_records_checked=True,
        stopped_and_absent_records_checked=True,evidence_sha256=digest.hexdigest(),
        execution_attempted=False,injected_failures_tested=False,
        crypto_interoperability_accepted=False,milestone_complete=False)
