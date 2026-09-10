"""Read one exact successful cloud crypto artifact. Never extracts or executes it."""
import importlib.util
import os
from pathlib import Path
import re
import resource
import signal
import sys

def helper(name):
    if name not in ("retained_checkpoint","crypto_handoff","crypto_failure"):
        raise ValueError("fixed crypto inspection helper")
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file() or not 1<=path.stat().st_size<=131072:
        raise ValueError("bounded regular trusted helper")
    spec=importlib.util.spec_from_file_location("crypto_inspection_"+name,path)
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module);return module

def number(value):
    if type(value) is not str or re.fullmatch("[1-9][0-9]{0,17}",value) is None:
        raise ValueError("canonical positive identifier")
    return int(value)

def receipt(metadata,run,run_id,artifact_id,source):
    handoff=helper("crypto_handoff")
    handoff.revision(source)
    if (type(run_id) is not int or not 1<=run_id<10**18 or
        type(artifact_id) is not int or not 1<=artifact_id<10**18 or
        type(run) is not dict or type(metadata) is not dict):
        raise ValueError("exact receipt inputs")
    controller=handoff.revision(run.get("head_sha"))
    check=helper("crypto_failure").fixed_fields
    repo=dict(id=1302587720,full_name="AndyTechCoder/RAR-OS")
    check(run,dict(id=run_id,head_sha=controller,run_attempt=1,event="workflow_dispatch",
        head_branch="main",status="completed",conclusion="success",
        path=".github/workflows/modern-crypto.yml",repository=repo,head_repository=repo))
    size=metadata.get("size_in_bytes");digest=metadata.get("digest")
    if (type(size) is not int or not 1<=size<=32*1024**2 or type(digest) is not str or
        re.fullmatch("sha256:[0-9a-f]{64}",digest) is None):
        raise ValueError("bounded digested crypto archive")
    check(metadata,dict(id=artifact_id,expired=False,name="modern-crypto-"+str(run_id)+"-1",
        workflow_run=dict(id=run_id,head_sha=controller,head_branch="main",
            repository_id=1302587720,head_repository_id=1302587720)))
    return dict(role="crypto",run=run_id,artifact=artifact_id,controller=controller,
        source=source,size=size,digest=digest[7:],workflow=".github/workflows/modern-crypto.yml")

def inspect(raw,binding):
    retained=helper("retained_checkpoint")
    # Full archive SHA256 precedes decompression; explicit v1 count cap. No files
    # are extracted and source Git objects remain inert in-memory bytes.
    members=retained.archive(raw,binding,challenge=True)
    report=retained.crypto(members,binding,challenge=True)
    output=retained.canonical(dict(inspection="fresh-m4-crypto-checkpoint-v1",
        artifact=binding["artifact"],archive_sha256=binding["digest"],**report))
    if len(output)>2*1024**2:raise ValueError("bounded escaped inspection report")
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
        # Never expose signed archive URLs, exception chains or credentials.
        raise ValueError("retained crypto inspection failed") from None

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
    resource.setrlimit(resource.RLIMIT_CPU,(180,180))
    def timeout(*args):raise ValueError("crypto inspection deadline")
    signal.signal(signal.SIGALRM,timeout);signal.alarm(600)
    client=handoff.GitHub(token);token=None
    try:
        print(acquire(client,run_id,artifact_id,source).decode("ascii"),end="",flush=True)
    finally:
        client.token="";signal.alarm(0)

if __name__=="__main__":
    if sys.argv!=[sys.argv[0]] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated cloud crypto inspection only")
    main()
