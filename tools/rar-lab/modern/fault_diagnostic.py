"""Cloud-only inert inspection of one retained Data-fault capture.
Never extracts files, executes archive contents, or grants runtime acceptance.
"""
import hashlib
import importlib.util
import io
import os
from pathlib import Path
import re
import resource
import signal
import stat
import sys
import zipfile

def helper(name):
    if name not in ("retained_checkpoint","crypto_handoff","crypto_failure"):
        raise ValueError("fixed diagnostic helper")
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("regular trusted helper")
    spec=importlib.util.spec_from_file_location("diagnostic_"+name,path)
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module

def number(value):
    if type(value) is not str or re.fullmatch("[1-9][0-9]{0,17}",value) is None:
        raise ValueError("canonical positive identifier")
    return int(value)

def receipt(metadata,run,run_id,artifact_id,source):
    helper("crypto_handoff").revision(source)
    if type(run) is not dict or type(metadata) is not dict:raise ValueError("metadata objects")
    controller=helper("crypto_handoff").revision(run.get("head_sha"))
    check=helper("crypto_failure").fixed_fields
    repo=dict(id=1302587720,full_name="AndyTechCoder/RAR-OS")
    check(run,dict(id=run_id,head_sha=controller,run_attempt=1,event="workflow_dispatch",
        head_branch="main",status="completed",path=".github/workflows/modern-data-faults.yml",
        repository=repo,head_repository=repo))
    if run.get("conclusion") not in ("success","failure","cancelled","timed_out"):
        raise ValueError("completed diagnostic run")
    size=metadata.get("size_in_bytes");digest=metadata.get("digest")
    if (type(size) is not int or not 1<=size<=32*1024**2 or type(digest) is not str or
        re.fullmatch("sha256:[0-9a-f]{64}",digest) is None):
        raise ValueError("bounded digested archive")
    check(metadata,dict(id=artifact_id,expired=False,
        name="modern-data-faults-"+str(run_id)+"-1",
        workflow_run=dict(id=run_id,head_sha=controller,head_branch="main",
            repository_id=1302587720,head_repository_id=1302587720)))
    return dict(run=run_id,artifact=artifact_id,controller=controller,source=source,
                size=size,digest=digest[7:])

def inspect(raw,binding):
    retained=helper("retained_checkpoint")
    if (type(raw) is not bytes or len(raw)!=binding["size"] or
        hashlib.sha256(raw).hexdigest()!=binding["digest"]):
        raise ValueError("archive identity before parsing")
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        entries=archive.infolist();members={};total=0
        if not 1<=len(entries)<=65:raise ValueError("bounded archive member count")
        fixed={"manifest.json":2*1024**2,"modern.efi":32*1024**2,
            "modern-service.efi":32*1024**2,"boot.img":32*1024**2,"tool-identities.txt":8192}
        for entry in entries:
            name=entry.filename
            limit=fixed.get(name,8*1024**2 if re.fullmatch(r"fault-[0-5][0-9]\.json",name) else None)
            if (limit is None or name in members or entry.is_dir() or
                entry.flag_bits&1 or stat.S_IFMT(entry.external_attr>>16) not in (0,stat.S_IFREG) or
                entry.compress_type not in (zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED) or
                not 0<=entry.file_size<=limit):
                raise ValueError("flat unique bounded regular archive members")
            total+=entry.file_size
            if total>416*1024**2:raise ValueError("bounded declared archive expansion")
            members[name]=entry
        if "manifest.json" not in members:raise ValueError("required manifest")
        manifest=retained.unique(archive.read(members["manifest.json"]))
        helper("crypto_failure").fixed_fields(manifest,dict(source=binding["source"],
            controller=binding["controller"],run=str(binding["run"]),attempt="1",milestone_complete=False))
        case=manifest.get("active_fault_case")
        if case is not None and (type(case) is not int or not 0<=case<60):
            raise ValueError("bounded active case")
        report=dict(schema="rar-modern-fault-diagnostic-v0",binding=binding,
            members=sorted(members),reported={key:manifest.get(key) for key in
                ("status","phase","failure","active_fault_case")},
            content_accepted=False,execution_attempted=False,milestone_complete=False,capture=None)
        name=None if case is None else "fault-"+str(case).zfill(2)+".json"
        if name in members:
            doc=retained.unique(archive.read(members[name]))
            helper("crypto_failure").fixed_fields(doc,dict(
                schema="rar-modern-data-fault-candidate-v0",case=case,milestone_complete=False))
            proofs=doc.get("vm_proofs")
            if type(proofs) is not list or len(proofs)!=2:raise ValueError("two recorded VM proofs")
            rows=[]
            for proof in proofs:
                if type(proof) is not dict or type(proof.get("cut")) is not dict:
                    raise ValueError("recorded proof object")
                # Explicit field selection: never print image payloads or Data keys.
                cut=proof["cut"]
                rows.append(dict(events=proof.get("events"),receipts=proof.get("event_receipts"),
                    entry=cut.get("entry"),fault=proof.get("fault"),
                    vm_returncode=cut.get("vm_returncode")))
            report["capture"]=dict(case=case,plan=doc.get("plan"),vm_proofs=rows)
        output=retained.canonical(report)
        if len(output)>128*1024:raise ValueError("bounded escaped diagnostic output")
        return output

def acquire(client,run_id,artifact_id,source):
    try:
        try:
            run=client.metadata("/repos/AndyTechCoder/RAR-OS/actions/runs/"+str(run_id))
            metadata=client.metadata("/repos/AndyTechCoder/RAR-OS/actions/artifacts/"+str(artifact_id))
            binding=receipt(metadata,run,run_id,artifact_id,source)
            raw=client.zip(artifact_id,binding["size"])
        finally:
            client.token=""
        return inspect(raw,binding)
    except BaseException:
        # Transport exceptions can contain a signed artifact URL. Do not print
        # exception messages, chained contexts, or payload-derived diagnostics.
        raise ValueError("retained fault diagnostic failed") from None

def main():
    handoff=helper("crypto_handoff")
    _,_,source,required=handoff.guard()
    run_id=number(os.environ.get("RAR_INSPECT_RUN"))
    artifact_id=number(os.environ.get("RAR_INSPECT_ARTIFACT"))
    token=os.environ.get("RAR_ARTIFACT_TOKEN")
    os.environ.clear();os.environ.update({**required,"PATH":"/usr/bin:/bin","LC_ALL":"C"})
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    resource.setrlimit(resource.RLIMIT_FSIZE,(0,0))
    resource.setrlimit(resource.RLIMIT_AS,(1536*1024**2,1536*1024**2))
    resource.setrlimit(resource.RLIMIT_CPU,(120,120))
    def timeout(*args):raise ValueError("diagnostic deadline")
    signal.signal(signal.SIGALRM,timeout);signal.alarm(600)
    client=handoff.GitHub(token);token=None
    try:
        output=acquire(client,run_id,artifact_id,source)
        print(output.decode("ascii"),end="",flush=True)
    finally:
        client.token="";signal.alarm(0)

if __name__=="__main__":
    if sys.argv!=[sys.argv[0]] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated cloud diagnostic only")
    main()
