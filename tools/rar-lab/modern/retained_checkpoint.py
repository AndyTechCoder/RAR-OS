"""Inspect two fixed retained cloud checkpoints. No extraction or execution."""
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

FIXED=(
 dict(role="persistence",artifact=10091254514,run=34319265996,size=223800,
      digest="86d6bfe56c595408925495f39fa413dd95d9041572fb6d12ae5c42cdfd3c52e1",
      controller="e4abb7ea966b5f65199c5f724db1633ef3dfcbb3",
      source="977aa66f8b4cc3d83370c10d88ea9763e31c11e7",
      workflow=".github/workflows/modern-persistence.yml"),
 dict(role="crypto",artifact=10099863097,run=34340985436,size=8803217,
      digest="a74400671ae13808b0755391e10018c05650312749d8a4421b923c52bc77db7c",
      controller="18146218af1c6f2a3dd629235cc17be0c964347b",
      source="bfd8647b10e1daa38934b1e12363ba919938bbf7",
      workflow=".github/workflows/modern-crypto.yml"),
)
class Invalid(ValueError):pass

def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise Invalid("fixed immutable helper")
    spec=importlib.util.spec_from_file_location("retained_"+name,path)
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True,allow_nan=False).encode("ascii")+b"\n"

def unique(raw):
    def pairs(rows):
        out={}
        for key,value in rows:
            if key in out:raise Invalid("duplicate JSON key")
            out[key]=value
        return out
    def constant(value):raise Invalid("nonfinite JSON")
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)

def strict(actual,wanted):
    if canonical(actual)!=canonical(wanted):raise Invalid("typed exact retained value")

def receipt(metadata,run,fixed):
    checker=helper("crypto_failure")
    repo=dict(id=1302587720,full_name="AndyTechCoder/RAR-OS")
    checker.fixed_fields(metadata,dict(id=fixed["artifact"],size_in_bytes=fixed["size"],
        digest="sha256:"+fixed["digest"],expired=False,
        name="modern-"+("persistence" if fixed["role"]=="persistence" else "crypto")+"-"+str(fixed["run"])+"-1",
        workflow_run=dict(id=fixed["run"],head_sha=fixed["controller"],head_branch="main",
                          repository_id=1302587720,head_repository_id=1302587720)))
    checker.fixed_fields(run,dict(id=fixed["run"],head_sha=fixed["controller"],run_attempt=1,
        event="workflow_dispatch",head_branch="main",status="completed",conclusion="success",
        path=fixed["workflow"],repository=repo,head_repository=repo))

def archive(raw,fixed):
    if (type(raw) is not bytes or len(raw)!=fixed["size"] or
        hashlib.sha256(raw).hexdigest()!=fixed["digest"]):
        raise Invalid("whole fixed ZIP identity before parsing")
    members={}
    with zipfile.ZipFile(io.BytesIO(raw)) as source:
        entries=source.infolist()
        if not 1<=len(entries)<=7000:raise Invalid("bounded member count")
        total=0
        for entry in entries:
            name=entry.filename;mode=entry.external_attr>>16
            if (re.fullmatch("[a-z0-9][a-z0-9._-]{0,127}",name) is None or name in members or
                entry.is_dir() or entry.flag_bits&1 or stat.S_IFMT(mode) not in (0,stat.S_IFREG) or
                entry.compress_type not in (zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED) or
                not 0<=entry.file_size<=32*1024**2):
                raise Invalid("flat inert bounded regular members only")
            total+=entry.file_size
            if total>256*1024**2:raise Invalid("bounded decompressed archive")
            data=source.read(entry)
            if len(data)!=entry.file_size:raise Invalid("complete member required")
            members[name]=data
    if "manifest.json" not in members or len(members["manifest.json"])>2*1024**2:
        raise Invalid("bounded retained manifest")
    return members

def baseline(members,fixed):
    manifest=unique(members["manifest.json"])
    helper("crypto_failure").fixed_fields(manifest,dict(source=fixed["source"],
        controller=fixed["controller"],run=str(fixed["run"]),attempt="1",
        status="observed",milestone_complete=False,crypto_interoperability_accepted=False))
    expected={"manifest.json","modern.efi","modern-service.efi","boot.img",
              "tool-identities.txt","persistence.json","refusals.json"}
    if set(members)!=expected:raise Invalid("exact baseline artifact inventory")
    lines=members["tool-identities.txt"].decode("ascii").splitlines()
    paths=("/usr/bin/python3.11","/usr/bin/qemu-system-x86_64",
           "/usr/share/OVMF/OVMF_CODE.fd","/usr/share/OVMF/OVMF_VARS.fd")
    if (len(lines)!=6 or members["tool-identities.txt"]!=("\n".join(lines)+"\n").encode("ascii") or
        any(re.fullmatch("[0-9a-f]{64}  "+re.escape(path),line) is None
            for line,path in zip(lines[:4],paths)) or
        any(re.fullmatch("[1-9][0-9]{0,9}",line) is None for line in lines[4:])):
        raise Invalid("canonical fixed tool identities and sizes")
    sizes=tuple(int(line) for line in lines[4:])
    boot=hashlib.sha256(members["boot.img"]).hexdigest()
    strict(manifest["boot_sha256"],boot)
    for name in ("modern.efi","modern-service.efi"):
        strict(manifest["binaries"][name],hashlib.sha256(members[name]).hexdigest())
    raw=members["persistence.json"]
    strict(manifest["persistence_sha256"],hashlib.sha256(raw).hexdigest())
    validator=helper("runtime_evidence")
    content=validator.validate(raw,boot,sizes)
    strict(manifest["content_check"],content)
    proof=unique(raw)
    records=[vm["cut"]["backends"][0]["records"] for vm in proof["vm_proofs"]]
    refusals=unique(members["refusals.json"])
    strict(manifest["actual_refusals"],refusals)
    helper("crypto_failure").fixed_fields(refusals,dict(schema="rar-modern-actual-refusals-v1",
        base_sha256=hashlib.sha256(raw).hexdigest(),rejected=60,disk_faults_tested=False,milestone_complete=False))
    fields=("schema","base_sha256","rejected","cases","disk_faults_tested","milestone_complete")
    if type(refusals) is not dict or set(refusals)!=set(fields):
        raise Invalid("exact retained refusal fields")
    rows=refusals["cases"]
    if type(rows) is not list or len(rows)!=60:raise Invalid("exact retained refusal count")
    ordered=[]
    for item in rows:
        if (type(item) is not dict or set(item)!={"case","input_sha256","rejection_sha256"} or
            any(type(item[key]) is not str or re.fullmatch("[0-9a-f]{64}",item[key]) is None
                for key in ("input_sha256","rejection_sha256"))):
            raise Invalid("canonical retained refusal row")
        ordered.append({key:item[key] for key in ("case","input_sha256","rejection_sha256")})
    strict([item["case"] for item in ordered],[item[0] for item in validator.refusal_cases()])
    rebuilt={key:(ordered if key=="cases" else refusals[key]) for key in fields}
    # This historical producer used ordered indented JSON, not compact JSON.
    encoded=(json.dumps(rebuilt,indent=2,ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")
    if members["refusals.json"]!=encoded:raise Invalid("canonical historical refusal bytes")
    return dict(role="persistence",content=content,data_records=records,
        firmware_sizes=list(sizes),source=fixed["source"],controller=fixed["controller"],
        retained_refusals=refusals,execution_attempted=False)

def source_binding(members,fixed,claimed):
    snapshot=helper("source_snapshot")
    commit_name="source-commit-"+fixed["source"]+".bin"
    if {name for name in members if name.startswith("source-commit-")}!={commit_name}:
        raise Invalid("exact retained source commit")
    objects={}
    for kind in ("tree","blob"):
        prefix="source-"+kind+"-"
        rows={}
        for name,raw in members.items():
            if not name.startswith(prefix):continue
            if re.fullmatch(prefix+"[0-9a-f]{40}\\.bin",name) is None:
                raise Invalid("canonical source object name")
            rows[name[len(prefix):-4]]=raw
        objects[kind]=rows
    layer,report=snapshot.build_from_objects(fixed["source"],members[commit_name],
                                            objects["tree"],objects["blob"])
    if members["source-layer.tar"]!=layer:raise Invalid("Git-bound source layer bytes")
    if members["source-binding.json"]!=canonical(report):
        raise Invalid("canonical recomputed source binding")
    strict(claimed,report)
    snapshot.inspect(layer,fixed["source"],report["files"])
    return report

def crypto(members,fixed):
    manifest=unique(members["manifest.json"])
    helper("crypto_failure").fixed_fields(manifest,dict(
        schema="rar-modern-crypto-handoff-v0",controller=fixed["controller"],source=fixed["source"],
        run=str(fixed["run"]),attempt="1",status="fixed-corpus-compared",
        phase="complete-fixed-corpus",crypto_interoperability_accepted=False,
        milestone_complete=False,target_os_execution=False))
    inventory=manifest["evidence_files"]
    if type(inventory) is not dict or set(inventory)!=set(members)-{"manifest.json"}:
        raise Invalid("complete crypto member inventory")
    for name,record in inventory.items():
        strict(record,dict(size=len(members[name]),sha256=hashlib.sha256(members[name]).hexdigest()))
    bound_source=source_binding(members,fixed,manifest["source_binding"])
    frozen=unique(members["frozen-rar-results.json"])
    results=unique(members["three-way-results.json"])
    if (type(frozen.get("cases")) is not list or len(frozen["cases"])!=288 or
        type(results.get("cases")) is not list or len(results["cases"])!=288):
        raise Invalid("exact recorded comparison count")
    comparison=helper("reference_comparison")
    count=0
    def recorded(implementation,request):
        nonlocal count
        if count<288:
            wire=frozen["cases"][count]["rar"]
            strict(frozen["cases"][count]["request"],request.hex())
        else:
            index,which=divmod(count-288,2)
            wire=results["cases"][index]["runs"][which+1]
        count+=1
        if type(wire) is not dict or set(wire)!={"implementation","exit_code","stdout","stderr"}:
            raise Invalid("exact recorded adapter tuple")
        strict(wire["implementation"],implementation)
        return wire["implementation"],wire["exit_code"],bytes.fromhex(wire["stdout"]),bytes.fromhex(wire["stderr"])
    def retain(name,raw):
        if name not in ("frozen-rar-results.json","three-way-results.json") or members[name]!=raw:
            raise Invalid("recomputed comparison differs from retained bytes")
        return hashlib.sha256(raw).hexdigest()
    # Pure protocol/corpus replay with recorded bytes, not adapter execution.
    report=comparison._compare(recorded,retain)
    if count!=864:raise Invalid("exact recorded invocation count")
    strict(manifest["comparison"],report)
    return dict(role="crypto",comparison=report,source_binding=bound_source,
        source=fixed["source"],controller=fixed["controller"],inventory_members=len(inventory),
        execution_attempted=False,crypto_interoperability_accepted=False,milestone_complete=False)

def main():
    handoff=helper("crypto_handoff")
    _,_,target,required=handoff.guard()
    if target!=FIXED[1]["source"]:raise Invalid("fixed inspection source")
    token=os.environ.get("RAR_ARTIFACT_TOKEN")
    os.environ.clear();os.environ.update({**required,"PATH":"/usr/bin:/bin","LC_ALL":"C"})
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    resource.setrlimit(resource.RLIMIT_AS,(1024*1024**2,1024*1024**2))
    resource.setrlimit(resource.RLIMIT_FSIZE,(0,0))
    resource.setrlimit(resource.RLIMIT_CPU,(180,180))
    def timeout(*args):raise Invalid("bounded inspection deadline")
    signal.signal(signal.SIGALRM,timeout);signal.alarm(600)
    client=handoff.GitHub(token);token=None
    try:
        acquired=[]
        for fixed in FIXED:
            metadata=client.metadata("/repos/"+handoff.REPO+"/actions/artifacts/"+str(fixed["artifact"]))
            run=client.metadata("/repos/"+handoff.REPO+"/actions/runs/"+str(fixed["run"]))
            receipt(metadata,run,fixed)
            acquired.append((fixed,client.zip(fixed["artifact"],fixed["size"])))
        client.token=""
        for fixed,raw in acquired:
            members=archive(raw,fixed)
            report=(baseline if fixed["role"]=="persistence" else crypto)(members,fixed)
            output=canonical(dict(inspection="fixed-m4-checkpoint-v0",artifact=fixed["artifact"],
                archive_sha256=fixed["digest"],**report))
            if len(output)>2*1024**2:raise Invalid("bounded inert JSON output")
            print(output.decode("ascii"),end="",flush=True)
            del members,raw
    except BaseException as error:
        detail=str(error)[:512] if type(error) is Invalid else type(error).__name__
        raise Invalid("fixed retained inspection failed ("+detail+")") from None
    finally:
        client.token="";signal.alarm(0)

if __name__=="__main__":
    if sys.argv!=[sys.argv[0]] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("fixed isolated cloud inspection only")
    main()
