"""Trusted-main cloud Modern persistence controller. Never execute locally.
This source candidate has no workflow dispatch until independent review.
"""
import json
import os
from pathlib import Path
import re
import hashlib

HERE=Path(__file__).resolve().parent

def load(name):
    import importlib.util
    path=HERE/(name+".py")
    if path.is_symlink() or not path.is_file(): raise ValueError("fixed trusted helper")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def digest(value): return hashlib.sha256(value).hexdigest()

def revision(value):
    if type(value) is not str or re.fullmatch("[0-9a-f]{40}",value) is None:
        raise ValueError("exact immutable revision")
    return value

def effective(info,image,name,entry,mounts,environment):
    """Inspect the created, never-started container before any tool/target code."""
    if type(info) is not dict or info.get("Image")!=image or info.get("Name")!="/"+name:
        raise ValueError("container/image identity")
    state=info.get("State",{})
    if state.get("Running") is not False or state.get("Status")!="created":
        raise ValueError("must inspect before first start")
    config,host=info.get("Config",{}),info.get("HostConfig",{})
    if (config.get("User")!="65532:65532" or config.get("Entrypoint")!=[entry[0]] or
        config.get("Cmd")!=entry[1:] or sorted(config.get("Env",[]))!=sorted(environment) or
        config.get("OpenStdin") or config.get("Tty")):
        raise ValueError("fixed user/entrypoint/environment")
    exact={"ReadonlyRootfs":True,"Privileged":False,"NetworkMode":"none",
        "Memory":1073741824,"MemorySwap":1073741824,"NanoCpus":2000000000,
        "PidsLimit":64,"PidMode":"","IpcMode":"private","PublishAllPorts":False,
        "AutoRemove":False}
    if any(type(host.get(k)) is not type(v) or host.get(k)!=v for k,v in exact.items()):
        raise ValueError("effective cloud confinement differs")
    if (host.get("CapDrop")!=["ALL"] or host.get("CapAdd") or
        host.get("SecurityOpt")!=["no-new-privileges"] or host.get("Devices") or
        host.get("DeviceRequests") or host.get("DeviceCgroupRules") or
        host.get("Binds") or host.get("VolumesFrom") or host.get("Links") or
        host.get("PortBindings") or host.get("ExtraHosts")):
        raise ValueError("extra device, capability, namespace or host access")
    tmp="/tmp:rw,exec,nosuid,nodev,size=256m,uid=65532,gid=65532,mode=700"
    if host.get("Tmpfs")!={"/tmp":tmp.split(":",1)[1]}:
        raise ValueError("exact disposable scratch mount")
    if host.get("Ulimits")!=[{"Name":"fsize","Soft":67108864,"Hard":67108864}]:
        raise ValueError("bounded file size")
    actual=info.get("Mounts")
    if type(actual) is not list or len(actual) not in (len(mounts),len(mounts)+1):
        raise ValueError("no extra mounts")
    found={}
    temporary=0
    for item in actual:
        if type(item) is dict and item.get("Type")=="tmpfs":
            if item.get("Destination")!="/tmp" or item.get("RW") is not True or item.get("Source","")!="" or temporary:
                raise ValueError("only the exact declared temporary mount")
            temporary+=1
            continue
        if (type(item) is not dict or item.get("Type")!="bind" or item.get("RW") is not False or
            item.get("Propagation")!="rprivate" or item.get("Destination") in found):
            raise ValueError("only unique read-only private binds")
        found[item["Destination"]]=item.get("Source")
    if found!={destination:str(source) for source,destination in mounts}:
        raise ValueError("exact source/artifact mount identities")
    return True

def main():
    import sys
    if (sys.argv!=[sys.argv[0]] or sys.platform!="linux" or not sys.flags.isolated or not sys.dont_write_bytecode or
        os.environ.get("GITHUB_ACTIONS")!="true" or os.environ.get("GITHUB_REPOSITORY")!="AndyTechCoder/RAR-OS" or
        os.environ.get("GITHUB_EVENT_NAME")!="workflow_dispatch" or os.environ.get("GITHUB_REF")!="refs/heads/main" or
        os.environ.get("RUNNER_OS")!="Linux" or os.environ.get("RUNNER_ARCH")!="X64" or os.environ.get("ImageOS")!="ubuntu24"):
        raise ValueError("trusted-main GitHub cloud workflow only")
    if any(key in os.environ for key in ("DOCKER_HOST","DOCKER_CONTEXT","DOCKER_CONFIG","DOCKER_TLS_VERIFY","DOCKER_CERT_PATH")):
        raise ValueError("ambient Docker endpoint/configuration prohibited")
    workspace=Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)
    controller,source=workspace/"controller",workspace/"source"
    if HERE!=controller/"tools/rar-lab/modern": raise ValueError("trusted controller checkout path")
    build=load("build_controller")
    run,sandbox,policy=build.run,build.sandbox,build.POLICY
    source_sha=revision(os.environ["RAR_SOURCE_SHA"])
    controller_sha=revision(os.environ["RAR_CONTROLLER_SHA"])
    if controller_sha!=revision(os.environ["GITHUB_SHA"]): raise ValueError("trusted-main controller revision")
    for root,expected in ((controller,controller_sha),(source,source_sha)):
        _,got=run(["git","-C",str(root),"rev-parse","HEAD"],10,1024)
        if got.decode().strip()!=expected: raise ValueError("checkout revision mismatch")
        _,dirty=run(["git","-C",str(root),"status","--porcelain=v1","--untracked-files=all","--ignored=matching"],15,1048576)
        if dirty: raise ValueError("source/controller must be clean exact Git snapshots")
    build.source_tree(build.helpers["git_tree"](source))
    run_id,attempt=os.environ["GITHUB_RUN_ID"],os.environ["GITHUB_RUN_ATTEMPT"]
    if any(re.fullmatch("[0-9]+",x) is None for x in (run_id,attempt)):
        raise ValueError("fixed run identity")
    evidence=workspace/"modern-runtime-evidence";evidence.mkdir(exist_ok=False)
    work=workspace/"modern-runtime-work";work.mkdir(mode=0o700,exist_ok=False)
    config=work/"docker-config";config.mkdir(mode=0o700,exist_ok=False)
    docker=["docker","--config",str(config),"--host","unix:///var/run/docker.sock"]
    bootcheck=load("boot_image")
    content=load("runtime_evidence")
    containers=[]
    report={"source":source_sha,"controller":controller_sha,"run":run_id,"attempt":attempt,
        "runner_image":os.environ["ImageVersion"],"status":"started","milestone_complete":False,
        "crypto_interoperability_accepted":False,"container_proofs":[]}
    def save(): (evidence/"manifest.json").write_text(json.dumps(report,indent=2)+"\n")
    save()
    def command(args,seconds,limit,allowed=(0,)):
        return run(docker+args,seconds,limit,allowed)
    def image(recipe,role):
        tag="rar-modern-"+role+":"+run_id+"-"+attempt
        command(["build","--pull=false","--tag",tag,"--file",str(HERE/recipe),str(HERE)],900,2097152)
        _,raw=command(["image","inspect",tag],10,65536)
        objects=json.loads(raw)
        if type(objects) is not list or len(objects)!=1: raise ValueError("exact built image")
        info=objects[0];identifier=info.get("Id")
        if type(identifier) is not str or re.fullmatch("sha256:[0-9a-f]{64}",identifier) is None:
            raise ValueError("digest-bound image")
        report[role+"_image"]=identifier
        return identifier,info["Config"].get("Env",[])
    sequence=0
    def execute(tool,entry,mounts,seconds,limit):
        nonlocal sequence
        sequence+=1
        name="rar-modern-"+run_id+"-"+attempt+"-"+str(sequence)
        identifier,defaults=tool
        env={"CI":"true","GITHUB_ACTIONS":"true","RAR_CI_RUNNER_OS":"Linux"}
        merged={}
        for item in defaults:
            key,value=item.split("=",1)
            if key in merged: raise ValueError("duplicate image environment")
            merged[key]=value
        merged.update(env)
        expected=[k+"="+v for k,v in merged.items()]
        args=["create","--name",name]+sandbox(policy)
        for path,destination in mounts:
            if path.is_symlink() or path.resolve(strict=True)!=path:
                raise ValueError("literal non-symlink controlled mount")
            args+=["--mount","type=bind,src="+str(path)+",dst="+destination+",readonly"]
        for key,value in env.items(): args+=["--env",key+"="+value]
        args+=["--entrypoint",entry[0],identifier]+entry[1:]
        _,created=command(args,30,1024)
        containers.append(name)  # Only after Docker confirmed successful creation.
        if re.fullmatch(b"[0-9a-f]{64}\n",created) is None: raise ValueError("exact created container ID")
        _,raw=command(["inspect",name],10,131072)
        inspection=json.loads(raw)
        if type(inspection) is not list or len(inspection)!=1: raise ValueError("one owned container")
        effective(inspection[0],identifier,name,entry,mounts,expected)
        proof={"name":name,"id":created.decode().strip(),"image":identifier,"created":inspection[0]}
        report["container_proofs"].append(proof);save()
        _,output=command(["start","--attach",name],seconds,limit)
        _,post=command(["inspect",name],10,131072)
        ended=json.loads(post)[0]
        if ended["State"]["Running"] is not False or ended["State"]["Status"]!="exited" or ended["State"]["ExitCode"]!=0:
            raise ValueError("owned cloud process did not exit successfully")
        proof["exited"]=ended;save()
        return output
    try:
        compiler=image("build.Containerfile","build")
        builds=[]
        for number in (1,2):
            raw=execute(compiler,["/bin/sh","-c",
                '/bin/sh /opt/rar-build.sh 2>/tmp/build.log; result=$?; if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi'],
                [(source,"/source")],300,6*1024*1024)
            binary=build.binary["unpack"](raw)
            for name,data in binary.items(): build.binary["inspect"](data,name=="modern-service.efi")
            builds.append(binary)
        if builds[0]!=builds[1]: raise ValueError("two independent actual UEFI builds differ")
        inputs=work/"inputs";inputs.mkdir(mode=0o755,exist_ok=False)
        (inputs/"modern.efi").write_bytes(builds[0]["modern.efi"])
        for name,data in builds[0].items(): (evidence/name).write_bytes(data)
        report["binaries"]={name:digest(data) for name,data in builds[0].items()}
        packaged=execute(compiler,["/bin/sh","/pack.sh"],
            [(HERE/"pack.sh","/pack.sh"),(controller/"nucleus/foundation/image.rs","/packager.rs"),(inputs,"/artifact")],
            120,24*1024*1024)
        boot=bootcheck.unpack(packaged,builds[0]["modern.efi"])
        (inputs/"boot.img").write_bytes(boot)
        (evidence/"boot.img").write_bytes(boot)
        report["boot_sha256"]=digest(boot);save()
        launcher=image("launch.Containerfile","runtime")
        identities=execute(launcher,["/bin/sh","-c",
            "sha256sum /usr/bin/python3.11 /usr/bin/qemu-system-x86_64 /usr/share/OVMF/OVMF_CODE.fd /usr/share/OVMF/OVMF_VARS.fd; stat -c %s /usr/share/OVMF/OVMF_CODE.fd /usr/share/OVMF/OVMF_VARS.fd"],
            [],20,8192)
        lines=identities.decode("ascii").splitlines()
        if len(lines)!=6 or any(re.fullmatch("[0-9]+",line) is None for line in lines[4:]):
            raise ValueError("fixed tool inventory and firmware sizes")
        paths=("/usr/bin/python3.11","/usr/bin/qemu-system-x86_64",
               "/usr/share/OVMF/OVMF_CODE.fd","/usr/share/OVMF/OVMF_VARS.fd")
        for line,path in zip(lines[:4],paths):
            if re.fullmatch("[0-9a-f]{64}  "+re.escape(path),line) is None:
                raise ValueError("exact tool hash inventory")
        sizes=tuple(int(line) for line in lines[4:])
        (evidence/"tool-identities.txt").write_bytes(identities)
        output=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/launch.py"],
            [(inputs,"/artifact")],300,64*1024*1024)
        (evidence/"persistence.json").write_bytes(output)
        report["content_check"]=content.validate(output,digest(boot),sizes)
        report["status"]="observed"
        report["persistence_sha256"]=digest(output)
        save()
        print("Modern: actual two-VM persistence envelope independently checked; M4 and crypto acceptance remain incomplete.",flush=True)
    except BaseException as error:
        report["status"]="failed";report["failure"]=str(error);save();raise
    finally:
        # Only exact disposable cloud containers created by this invocation.
        # No host files, source trees, volumes, images, branches or owner data.
        failures=[]
        for name in reversed(containers):
            try: command(["rm","--force",name],15,4096)
            except BaseException: failures.append(name)
        if failures:
            report["status"]="cleanup-failed";report["cleanup_failures"]=failures;save()
            raise RuntimeError("owned cloud container cleanup failed")

def self_test():
    # Inert Docker-inspect fixtures only; no subprocess or filesystem mutation.
    image="sha256:"+"a"*64
    entry=["/usr/bin/python3","-I","-B","/opt/rar-modern/launch.py"]
    env=["CI=true"]
    info={"Image":image,"Name":"/owned","State":{"Running":False,"Status":"created"},
        "Config":{"User":"65532:65532","Entrypoint":entry[:1],"Cmd":entry[1:],"Env":env},
        "HostConfig":{"ReadonlyRootfs":True,"Privileged":False,"NetworkMode":"none",
        "Memory":1073741824,"MemorySwap":1073741824,"NanoCpus":2000000000,"PidsLimit":64,
        "PidMode":"","IpcMode":"private","PublishAllPorts":False,"AutoRemove":False,
        "CapDrop":["ALL"],"SecurityOpt":["no-new-privileges"],
        "Tmpfs":{"/tmp":"rw,exec,nosuid,nodev,size=256m,uid=65532,gid=65532,mode=700"},
        "Ulimits":[{"Name":"fsize","Soft":67108864,"Hard":67108864}]},"Mounts":[]}
    assert effective(info,image,"owned",entry,[],env)
    tmp=json.loads(json.dumps(info));tmp["Mounts"]=[{"Type":"tmpfs","Destination":"/tmp","RW":True,"Source":""}]
    assert effective(tmp,image,"owned",entry,[],env)
    rejected=0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected+=1
        else: raise AssertionError("unsafe effective container accepted")
    for key,value in (("NetworkMode","host"),("Privileged",True),("ReadonlyRootfs",False),
        ("Memory",0),("PidsLimit",0),("PidMode","host"),("IpcMode","host"),
        ("CapAdd",["SYS_ADMIN"]),("SecurityOpt",[]),("Devices",[{}]),
        ("Tmpfs",{}),("Ulimits",[]),("VolumesFrom",["other"])):
        changed=json.loads(json.dumps(info));changed["HostConfig"][key]=value
        reject(lambda changed=changed:effective(changed,image,"owned",entry,[],env))
    for key,value in (("User","0:0"),("Env",["GITHUB_TOKEN=secret"]),("Cmd",["/outside"]),("Tty",True)):
        changed=json.loads(json.dumps(info));changed["Config"][key]=value
        reject(lambda changed=changed:effective(changed,image,"owned",entry,[],env))
    changed=json.loads(json.dumps(info));changed["Mounts"]=[{"Type":"bind","RW":True,"Destination":"/outside"}]
    reject(lambda:effective(changed,image,"owned",entry,[],env))
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv==[sys.argv[0],"--self-test"] and sys.flags.isolated and sys.dont_write_bytecode:
        print("Modern controller:",self_test(),"effective confinement refusals; no runtime activation")
    else:
        main()
