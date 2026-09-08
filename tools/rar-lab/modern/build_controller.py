"""Trusted-main compile-only Modern controller. No boot, mounting, or VM path."""
import json
import os
from pathlib import Path
import re

HERE=Path(__file__).resolve().parent
helpers={"__name__":"trusted_foundation_helpers"}
exec(compile((HERE.parent/"foundation/controller.py").read_text(),
             "trusted-foundation-controller.py","exec"),helpers)
binary={"__name__":"trusted_modern_binary"}
exec(compile((HERE/"build_binary.py").read_text(),"trusted-modern-binary.py","exec"),binary)
run,digest,sandbox=(helpers[n] for n in ("run","digest","sandbox"))
POLICY=helpers["POLICY"]
REQUIRED=("nucleus/foundation/main.rs","nucleus/modern/main.rs","nucleus/modern/native_pio.rs",
          "core/modern/main.rs","services/modern/runtime.rs","apps/modern/runtime.rs")

def source_tree(tree):
    if any(mode not in ("100644","100755") for mode,_ in tree.values()):
        raise ValueError("source tree contains non-regular Git entries")
    if any(name not in tree for name in REQUIRED):
        raise ValueError("Modern runtime source is incomplete")

def identity(value):
    if not isinstance(value,str) or re.fullmatch("[0-9a-f]{40}",value) is None:
        raise ValueError("revision must be an exact lowercase Git SHA")
    return value

def self_test():
    rejected=binary["self_test"]()+helpers["negative_tests"]()
    source_tree(dict.fromkeys(REQUIRED,("100644","0"*40)))
    for tree in ({},dict.fromkeys(REQUIRED,("120000","0"*40)),
                 dict(dict.fromkeys(REQUIRED,("100644","0"*40)),extra=("160000","0"*40))):
        try: source_tree(tree)
        except ValueError: rejected+=1
        else: raise AssertionError("invalid source tree accepted")
    assert identity("a"*40)=="a"*40
    for value in ("main","A"*40,"a"*39,"a"*41,"a"*40+"\n",None,"../source"):
        try: identity(value)
        except ValueError: rejected+=1
        else: raise AssertionError("invalid revision accepted")
    assert rejected==69, "Modern build negative coverage changed"
    return rejected

def main():
    if not (os.environ.get("GITHUB_ACTIONS")=="true" and
            os.environ.get("GITHUB_REPOSITORY")=="AndyTechCoder/RAR-OS" and
            os.environ.get("GITHUB_EVENT_NAME")=="workflow_dispatch" and
            os.environ.get("GITHUB_REF")=="refs/heads/main" and
            os.environ.get("RUNNER_OS")=="Linux" and
            os.environ.get("RUNNER_ARCH")=="X64" and os.environ.get("ImageOS")=="ubuntu24"):
        raise SystemExit("Modern build requires the trusted-main cloud workflow")
    workspace=Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)
    controller,source=workspace/"controller",workspace/"source"
    if HERE!=controller/"tools/rar-lab/modern":
        raise SystemExit("trusted controller path mismatch")
    for root,key in ((controller,"RAR_CONTROLLER_SHA"),(source,"RAR_SOURCE_SHA")):
        expected=identity(os.environ[key])
        _,actual=run(["git","-C",str(root),"rev-parse","HEAD"],10,1024)
        if actual.decode().strip()!=expected:
            raise ValueError("checkout SHA mismatch")
        _,dirty=run(["git","-C",str(root),"status","--porcelain=v1","--untracked-files=all","--ignored=matching"],15,1048576)
        if dirty:
            raise ValueError("checkout must be exact and clean")
    if os.environ["RAR_CONTROLLER_SHA"]!=identity(os.environ["GITHUB_SHA"]):
        raise ValueError("controller is not this trusted-main workflow revision")
    run_id,attempt=os.environ["GITHUB_RUN_ID"],os.environ["GITHUB_RUN_ATTEMPT"]
    if any(re.fullmatch("[0-9]+",v) is None for v in (run_id,attempt)):
        raise ValueError("invalid run identity")
    source_tree(helpers["git_tree"](source))
    evidence=workspace/"modern-build-evidence"
    evidence.mkdir(exist_ok=False)
    summary=dict(source=os.environ["RAR_SOURCE_SHA"],controller=os.environ["RAR_CONTROLLER_SHA"],
                 run=run_id,attempt=attempt,job=os.environ["GITHUB_JOB"],
                 runner_image=os.environ["ImageVersion"],policy=POLICY,
                 negative_tests=self_test(),status="started",target_execution=False,
                 vm_activation=False,stack_usage_proven=False)
    def save():
        (evidence/"manifest.json").write_text(json.dumps(summary,indent=2)+"\n")
    save()
    containers=[]
    try:
        tag="rar-modern-build:"+run_id+"-"+attempt
        run(["docker","build","--pull=false","--tag",tag,
             "--file",str(HERE/"build.Containerfile"),str(HERE)],
            900,2097152,log_path=evidence/"build-image.log")
        _,raw=run(["docker","image","inspect","--format","{{.Id}}",tag],10,1024)
        image=raw.decode().strip()
        if re.fullmatch("sha256:[0-9a-f]{64}",image) is None:
            raise ValueError("tool image is not digest bound")
        summary["build_image"]=image
        name="rar-modern-identities-"+run_id+"-"+attempt
        containers.append(name)
        _,identities=run(["docker","run","--name",name]+sandbox(POLICY)+[
            "--entrypoint","/bin/sh",image,"-c",
            "sha256sum /opt/rar-toolchain/bin/rustc /opt/rar-toolchain/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"],
            20,8192)
        (evidence/"build-identities.txt").write_bytes(identities)
        builds=[]
        for index in (1,2):
            name="rar-modern-build-"+run_id+"-"+attempt+"-"+str(index)
            containers.append(name)
            argv=["docker","run","--name",name]+sandbox(POLICY)+[
                "--mount","type=bind,src="+str(source)+",dst=/source,readonly",
                "--entrypoint","/bin/sh",image,"-c",
                '/bin/sh /opt/rar-build.sh 2>/tmp/build.log; result=$?; '
                'if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi']
            print("Independent Modern UEFI build",index,flush=True)
            _,encoded=run(argv,300,6*1024*1024)
            built=binary["unpack"](encoded)
            builds.append(built)
            summary["build_"+str(index)]={n:digest(b) for n,b in built.items()}
            summary["layout_"+str(index)]={n:binary["inspect"](b,n=="modern-service.efi") for n,b in built.items()}
            save()
        if summary["build_1"]!=summary["build_2"]:
            raise ValueError("independent target builds differ")
        for name,data in builds[0].items():
            (evidence/name).write_bytes(data)
        summary.update(status="passed",reproducible=True)
        save()
        print("Modern: two identical UEFI builds and bounded PE/W^X layouts passed; no boot or M4 acceptance claim.",flush=True)
    except BaseException as error:
        summary.update(status="failed",failure=str(error))
        save()
        raise
    finally:
        # Only disposable cloud containers named by this invocation; never
        # source files, host volumes, images, tags, branches, or owner data.
        for name in containers:
            run(["docker","rm","--force",name],15,4096,(0,1))

if __name__=="__main__":
    import sys
    if sys.argv==[sys.argv[0],"--self-test"]:
        print("Modern build controller:",self_test(),"negative tests; no target execution")
    elif len(sys.argv)==1:
        main()
    else:
        raise SystemExit("unexpected arguments")
