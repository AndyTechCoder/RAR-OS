"""Integrated trusted-main cloud crypto handoff. NEVER run on a Mac/SSD.
All effects are confined to a new disposable hosted-job workspace and Docker
daemon. No OS/VM boot, signing, owner data, arbitrary source paths or retries.
"""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import resource
import signal
import ssl
import stat
import sys
import tarfile
import time
import urllib.error
import urllib.parse
import urllib.request

HERE=Path(__file__).resolve().parent
REPO="AndyTechCoder/RAR-OS"
BASE="rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"
EPOCH=1785715200
MUSL_ROOT="usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl"
CONSUMED=("consumed-musl.tree","consumed-musl.sha256")
DOCKER=["/usr/bin/docker","--host=unix:///var/run/docker.sock"]
MODULES=("construction_artifacts","source_snapshot","compiler_inventory",
    "reference_inventory","compiler_driver_layer","derived_compiler_image",
    "adapter_image","compiler_runner","reference_runner","reference_comparison","crypto_failure_probes")
class Invalid(RuntimeError):pass

def sha(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()+b"\n"
def revision(value):
    if type(value) is not str or re.fullmatch("[0-9a-f]{40}",value) is None or value=="0"*40:
        raise Invalid("exact immutable revision")
    return value
def unique(raw):
    def pairs(rows):
        value={}
        for key,item in rows:
            if key in value:raise Invalid("duplicate JSON key")
            value[key]=item
        return value
    def constant(value):raise Invalid("nonfinite JSON")
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)
def module(name):
    if name not in MODULES:raise Invalid("fixed trusted helper")
    path=HERE/(name+".py")
    if path.is_symlink() or not path.is_file() or not 1<=path.stat().st_size<=131072:
        raise Invalid("bounded regular helper")
    spec=importlib.util.spec_from_file_location("rar_handoff_"+name,path)
    value=importlib.util.module_from_spec(spec);sys.modules[spec.name]=value
    spec.loader.exec_module(value);return value

def guard():
    required={"GITHUB_ACTIONS":"true","CI":"true","GITHUB_REPOSITORY":REPO,
        "GITHUB_EVENT_NAME":"workflow_dispatch","GITHUB_REF":"refs/heads/main",
        "RUNNER_OS":"Linux","RUNNER_ARCH":"X64","ImageOS":"ubuntu24"}
    if (sys.platform!="linux" or os.uname().machine!="x86_64" or
        not sys.flags.isolated or not sys.dont_write_bytecode or
        any(os.environ.get(key)!=value for key,value in required.items())):
        raise Invalid("trusted-main hosted Linux job only")
    controller=revision(os.environ.get("GITHUB_SHA"))
    target=revision(os.environ.get("RAR_SOURCE_SHA"))
    if (os.environ.get("RAR_CONTROLLER_SHA")!=controller or
        re.fullmatch("[0-9]+",os.environ.get("GITHUB_RUN_ID","")) is None or
        os.environ.get("GITHUB_RUN_ATTEMPT")!="1" or
        re.fullmatch("[A-Za-z0-9._-]{1,128}",os.environ.get("ImageVersion","")) is None):
        raise Invalid("exact first-attempt workflow identity")
    if any(key in os.environ for key in ("DOCKER_HOST","DOCKER_CONTEXT","DOCKER_CONFIG",
                                          "DOCKER_TLS_VERIFY","DOCKER_CERT_PATH")):
        raise Invalid("ambient Docker configuration")
    workspace=Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)
    if HERE!=workspace/"controller/tools/rar-lab/modern":
        raise Invalid("exact trusted controller location")
    space=os.statvfs(workspace)
    if space.f_bavail*space.f_frsize<16*1024**3:
        raise Invalid("disposable cloud disk reserve")
    return workspace,controller,target,required

class Evidence:
    """New owned directory, exclusive/no-follow files, bounded durable writes."""
    def __init__(self,parent,name,budget=256*1024**2):
        if type(name) is not str or re.fullmatch("[a-z][a-z0-9-]{0,63}",name) is None:
            raise Invalid("fixed evidence directory name")
        self.root=Path(parent)/name
        self.root.mkdir(mode=0o700,exist_ok=False)
        self.fd=os.open(self.root,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        info=os.fstat(self.fd)
        if not stat.S_ISDIR(info.st_mode) or info.st_uid!=os.geteuid() or stat.S_IMODE(info.st_mode)!=0o700:
            raise Invalid("owned private evidence directory")
        self.budget=budget;self.total=0;self.entries={};self.closed=False
    def retain(self,name,raw):
        if (self.closed or type(name) is not str or
            re.fullmatch("[a-z0-9][a-z0-9._-]{0,127}",name) is None or
            name in self.entries or type(raw) is not bytes or len(raw)>64*1024**2 or
            len(self.entries)>=10000 or self.total+len(raw)>self.budget):
            raise Invalid("exclusive bounded evidence")
        fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=self.fd)
        self.total+=len(raw)
        try:
            info=os.fstat(fd)
            if not stat.S_ISREG(info.st_mode) or info.st_nlink!=1 or info.st_uid!=os.geteuid():
                raise Invalid("evidence file ownership")
            view=memoryview(raw)
            while view:
                n=os.write(fd,view)
                if n<=0:raise Invalid("evidence short write")
                view=view[n:]
            os.fsync(fd)
        finally:os.close(fd)
        os.fsync(self.fd)
        digest=sha(raw);self.entries[name]={"sha256":digest,"size":len(raw)}
        return digest
    def scoped(self,prefix):
        if re.fullmatch("[a-z][a-z0-9-]{0,63}",prefix) is None:raise Invalid("fixed evidence scope")
        return lambda name,raw:self.retain(prefix+"-"+name,raw)
    def close(self):
        if not self.closed:os.close(self.fd);self.closed=True

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,req,fp,code,msg,headers,newurl):return None

def archive_url(url):
    try:value=urllib.parse.urlsplit(url);port=value.port
    except (TypeError,ValueError) as error:raise Invalid("artifact location") from error
    host=value.hostname or ""
    if (value.scheme!="https" or port not in (None,443) or value.username is not None or
        value.password is not None or value.fragment or
        not (host.endswith(".blob.core.windows.net") or host.endswith(".actions.githubusercontent.com")) or
        not value.path.startswith("/") or len(url)>16384):
        raise Invalid("fixed HTTPS artifact storage boundary")
    return url

class GitHub:
    """Token only to api.github.com; never forwarded to archive storage/tools."""
    def __init__(self,token):
        if type(token) is not str or not 1<=len(token)<=4096 or not token.isascii() or any(c.isspace() for c in token):
            raise Invalid("ephemeral read-only workflow token")
        self.token=token
        self.opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),NoRedirect(),
            urllib.request.HTTPSHandler(context=ssl.create_default_context()))
    def request(self,path):
        if re.fullmatch(r"/repos/AndyTechCoder/RAR-OS/actions/(artifacts/[0-9]+(?:/zip)?|runs/[0-9]+)",path) is None:
            raise Invalid("fixed repository artifact API path")
        return urllib.request.Request("https://api.github.com"+path,headers={
            "Accept":"application/vnd.github+json","Authorization":"Bearer "+self.token,
            "X-GitHub-Api-Version":"2022-11-28","User-Agent":"RAR-Modern-Crypto-Handoff"})
    def read(self,request,maximum,seconds):
        deadline=time.monotonic()+seconds;out=bytearray()
        with self.opener.open(request,timeout=15) as response:
            if response.status!=200:raise Invalid("artifact HTTP status")
            declared=response.headers.get("Content-Length")
            if declared is not None and (not declared.isdigit() or int(declared)>maximum):
                raise Invalid("artifact Content-Length")
            while True:
                if time.monotonic()>deadline:raise Invalid("artifact acquisition deadline")
                block=response.read(min(1024*1024,maximum+1-len(out)))
                if not block:break
                out.extend(block)
                if len(out)>maximum:raise Invalid("artifact acquisition size")
        return bytes(out)
    def metadata(self,path):return unique(self.read(self.request(path),4*1024**2,60))
    def zip(self,artifact,size):
        request=self.request("/repos/"+REPO+"/actions/artifacts/"+str(artifact)+"/zip")
        try:
            response=self.opener.open(request,timeout=15)
        except urllib.error.HTTPError as response:
            try:
                if response.code!=302:raise Invalid("artifact redirect status") from None
                location=archive_url(response.headers.get("Location"))
            finally:response.close()
        else:
            response.close();raise Invalid("artifact must use credential-separated redirect")
        # No Authorization, no cookies, no proxy, no second redirect, no URL logs.
        raw=self.read(urllib.request.Request(location,headers={"User-Agent":"RAR-Modern-Crypto-Handoff"}),size,300)
        if len(raw)!=size:raise Invalid("exact artifact bytes")
        return raw

def consumed_inventory(parent):
    """Expected bytes come from the independent pinned parent inspector."""
    files={name:entry for name,entry in parent["files"].items() if name.startswith(MUSL_ROOT+"/")}
    dirs={name:entry for name,entry in parent["directories"].items()
          if name==MUSL_ROOT or name.startswith(MUSL_ROOT+"/")}
    if not files or MUSL_ROOT not in dirs or len(files)+len(dirs)>4096:
        raise Invalid("complete bounded consumed sysroot")
    lines=[]
    for name,entry in dirs.items():
        if tuple(entry[:3])!=(0o555,0,0):raise Invalid("parent sysroot directory metadata")
        lines.append("d 555 0 0 /"+name+"\n")
    for name,entry in files.items():
        if (entry["mode"]!=0o444 or entry["uid"]!=0 or entry["gid"]!=0 or
            re.fullmatch("[0-9a-f]{64}",entry["sha256"]) is None):
            raise Invalid("parent sysroot file metadata")
        lines.append("f 444 0 0 /"+name+"\n")
    tree="".join(sorted(lines)).encode("ascii")
    hashes="".join(files[name]["sha256"]+"  /"+name+"\n" for name in sorted(files)).encode("ascii")
    if max(len(tree),len(hashes))>2*1024**2:raise Invalid("consumption evidence bound")
    return {CONSUMED[0]:tree,CONSUMED[1]:hashes}

def verify_parent_tag(raw,parent_image):
    if type(parent_image) is not str or re.fullmatch("sha256:[0-9a-f]{64}",parent_image) is None:
        raise Invalid("immutable compiler parent image")
    rows=unique(raw)
    if (type(rows) is not list or len(rows)!=1 or type(rows[0]) is not dict or
        rows[0].get("Id")!=parent_image or rows[0].get("Architecture")!="amd64" or
        rows[0].get("Os")!="linux"):
        raise Invalid("compiler parent tag changed")
    return True

def driver_export(raw,notices,driver_gate,parent):
    if type(raw) is not bytes or not 10240<=len(raw)<=64*1024**2:
        raise Invalid("bounded driver export")
    consumed=consumed_inventory(parent)
    expected_files={**notices,**consumed}
    directories={"licenses"}
    for name in notices:
        parts=name.split("/")
        directories.update("/".join(parts[:i]) for i in range(1,len(parts)))
    seen=set();driver=None
    with tarfile.open(fileobj=io.BytesIO(raw),mode="r:") as archive:
        for item in archive:
            name=item.name.removeprefix("./").rstrip("/")
            if name in ("","."):
                if not item.isdir():raise Invalid("driver export root")
                continue
            if (name in seen or item.uid!=0 or item.gid!=0 or item.pax_headers or
                item.sparse is not None or item.linkname or item.mode&0o7022):
                raise Invalid("driver export metadata")
            seen.add(name)
            if len(seen)>len(expected_files)+len(directories)+1:raise Invalid("driver export member budget")
            if name in directories:
                if not item.isdir() or item.mode not in (0o555,0o755):raise Invalid("driver export directory")
                continue
            expected=expected_files.get(name)
            if name!="rar-compile-driver" and expected is None:raise Invalid("unexpected driver export file")
            maximum=2*1024**2 if name=="rar-compile-driver" else len(expected)
            if (not item.isfile() or not 1<=item.size<=maximum or item.mtime!=EPOCH or
                item.mode!=(0o555 if name=="rar-compile-driver" else 0o444)):
                raise Invalid("driver export file metadata")
            stream=archive.extractfile(item)
            if stream is None:raise Invalid("driver export bytes")
            value=stream.read(maximum+1)
            if len(value)!=item.size:raise Invalid("driver export exact bytes")
            if name=="rar-compile-driver":driver_gate.static_driver(value);driver=value
            elif value!=expected:raise Invalid("upstream notice or consumed sysroot identity")
    if seen!=set(expected_files)|directories|{"rar-compile-driver"} or driver is None:
        raise Invalid("complete driver export")
    return driver,{name:{"sha256":sha(value),"size":len(value)} for name,value in consumed.items()}

def read_owned(path,maximum):
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW)
    try:
        before=os.fstat(fd)
        if (not stat.S_ISREG(before.st_mode) or before.st_nlink!=1 or
            before.st_uid!=os.geteuid() or not 1<=before.st_size<=maximum):
            raise Invalid("bounded owned cloud output")
        out=bytearray()
        while len(out)<=maximum:
            part=os.read(fd,min(1024*1024,maximum+1-len(out)))
            if not part:break
            out.extend(part)
        after=os.fstat(fd)
        fields=("st_dev","st_ino","st_mode","st_nlink","st_uid","st_size","st_mtime_ns","st_ctime_ns")
        if any(getattr(before,key)!=getattr(after,key) for key in fields) or len(out)!=before.st_size:
            raise Invalid("stable exact cloud output")
        return bytes(out)
    finally:os.close(fd)

def tool_identities():
    tools={}
    for path in (Path(sys.executable),Path("/usr/bin/git"),Path("/usr/bin/docker")):
        resolved=path.resolve(strict=True)
        if not resolved.is_file() or not 1<=resolved.stat().st_size<=128*1024**2:
            raise Invalid("bounded hosted tool identity")
        digest=hashlib.sha256();total=0
        with resolved.open("rb") as stream:
            while True:
                part=stream.read(65536)
                if not part:break
                total+=len(part)
                if total>128*1024**2:raise Invalid("hosted tool growth")
                digest.update(part)
        tools[str(path)]={"resolved":str(resolved),"sha256":digest.hexdigest(),"size":total}
    return tools

class DockerClient:
    """Fresh cloud-only CLI state; never import existing credentials or context."""
    def __init__(self,work):
        self.paths=[Path(work)/"docker-config",Path(work)/"buildx-config"]
        self.identities=[]
        for path in self.paths:
            path.mkdir(mode=0o700,exist_ok=False)
            info=path.lstat()
            if (not stat.S_ISDIR(info.st_mode) or info.st_uid!=os.geteuid() or
                stat.S_IMODE(info.st_mode)!=0o700):
                raise Invalid("owned private Docker client state")
            self.identities.append((info.st_dev,info.st_ino))
    def argv(self,args):
        for path,identity in zip(self.paths,self.identities):
            info=path.lstat()
            if (not stat.S_ISDIR(info.st_mode) or info.st_uid!=os.geteuid() or
                stat.S_IMODE(info.st_mode)!=0o700 or (info.st_dev,info.st_ino)!=identity):
                raise Invalid("Docker client directory changed")
        config,buildx=map(str,self.paths)
        return ["/usr/bin/env","-i","PATH=/usr/bin:/bin","LANG=C","LC_ALL=C",
            "DOCKER_CONFIG="+config,"BUILDX_CONFIG="+buildx]+DOCKER+["--config",config]+args

def main():
    workspace,controller,target,required=guard()
    token=os.environ.get("RAR_ARTIFACT_TOKEN")
    run_id=os.environ["GITHUB_RUN_ID"];runner=os.environ["ImageVersion"]
    # Never inherit credentials, Git/Docker/proxy settings into child tools.
    clean={**required,"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C","TZ":"UTC"}
    os.environ.clear();os.environ.update(clean)
    github=GitHub(token)
    token=None
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    resource.setrlimit(resource.RLIMIT_AS,(5*1024**3,5*1024**3))
    resource.setrlimit(resource.RLIMIT_FSIZE,(2*1024**3,2*1024**3))
    def deadline(signum,frame):raise Invalid("whole handoff deadline")
    signal.signal(signal.SIGALRM,deadline);signal.alarm(3000)
    evidence=Evidence(workspace,"modern-crypto-evidence")
    work=workspace/"modern-crypto-work";work.mkdir(mode=0o700,exist_ok=False)
    client=DockerClient(work)
    helpers={name:module(name) for name in MODULES}
    transport=helpers["reference_runner"]
    phase="preflight";summary={"schema":"rar-modern-crypto-handoff-v0","controller":controller,
        "source":target,"run":run_id,"attempt":"1","runner_image":runner,
        "status":"started","milestone_complete":False,"target_os_execution":False}
    sequence=0
    def command(argv,seconds=30,maximum=65536,request=b""):
        nonlocal sequence
        sequence+=1;prefix="command-"+str(sequence)
        evidence.retain(prefix+"-argv.json",canonical(argv))
        if request:evidence.retain(prefix+"-input.json",canonical({"sha256":sha(request),"size":len(request)}))
        try:code,out,error=transport.exchange(argv,request,seconds,maximum,2*1024**2)
        except Exception as failure:
            evidence.retain(prefix+"-failure.json",canonical({"type":type(failure).__name__}))
            for field in ("partial_stdout","partial_stderr"):
                value=getattr(failure,field,None)
                if type(value) is bytes and len(value)<=64*1024**2:
                    evidence.retain(prefix+"-"+field+".bin",value)
            raise
        evidence.retain(prefix+"-result.json",canonical({"exit":code,"stdout_sha256":sha(out),
            "stdout_size":len(out),"stderr_sha256":sha(error),"stderr_size":len(error)}))
        evidence.retain(prefix+"-stderr.bin",error)
        if len(out)<=2*1024**2:evidence.retain(prefix+"-stdout.bin",out)
        if type(code) is not int or code!=0:
            # CLI environments are scrubbed and inputs are public fixed data.
            # Canonical JSON escapes newlines and workflow-command prefixes.
            print(canonical({"diagnostic":"rar-modern-command-failure-v0",
                "sequence":sequence,"argv":argv,"exit":code,
                "stderr_tail":error[-8192:].decode("utf-8","backslashreplace"),
                "stdout_tail":out[-2048:].decode("utf-8","backslashreplace")
                }).decode("ascii"),end="",flush=True)
            raise Invalid("cloud command failed")
        return out
    def git(root,args,maximum=131072):
        return command(["/usr/bin/env","-i","PATH=/usr/bin:/bin","LANG=C","LC_ALL=C",
            "GIT_CONFIG_NOSYSTEM=1","GIT_CONFIG_GLOBAL=/nonexistent","GIT_ATTR_NOSYSTEM=1",
            "GIT_NO_REPLACE_OBJECTS=1","GIT_TERMINAL_PROMPT=0","/usr/bin/git",
            "-c","core.hooksPath=/nonexistent","-c","core.fsmonitor=false",
            "-c","core.attributesFile=/nonexistent","-c","protocol.allow=never",
            "-c","protocol.https.allow=always","-C",str(root)]+args,120,maximum)
    def docker(args,seconds=30,maximum=65536,request=b""):
        return command(client.argv(args),seconds,maximum,request)
    def image_identity(image):
        raw=docker(["image","inspect",image])
        rows=unique(raw)
        if type(rows) is not list or len(rows)!=1 or rows[0].get("Id")!=image:
            raise Invalid("loaded immutable image identity")
        item=rows[0]
        if item.get("Architecture")!="amd64" or item.get("Os")!="linux":
            raise Invalid("loaded platform")
        return item
    def load_image(raw,report,config=None):
        image=report["image"]
        if re.fullmatch("sha256:[0-9a-f]{64}",image) is None:raise Invalid("inventoried image")
        docker(["image","load"],180,1048576,raw)
        item=image_identity(image)
        if item.get("RootFS")!={"Type":"layers","Layers":report["diff_ids"]}:
            raise Invalid("loaded image layer identity")
        if config is not None:
            actual=item.get("Config")
            if type(actual) is not dict:raise Invalid("loaded process")
            for key,want in config.items():
                if actual.get(key)!=want:raise Invalid("loaded process differs")
            for key in ("Cmd","Volumes","ExposedPorts","OnBuild","Healthcheck","Labels","Shell"):
                if actual.get(key) not in (None,[],{}):raise Invalid("loaded extra authority")
        evidence.retain("image-"+image[7:]+"-inspect.json",canonical(item))
        return image
    def obtain(role):
        gate=helpers["construction_artifacts"];receipt=gate.RECEIPTS[role]
        metadata=github.metadata("/repos/"+REPO+"/actions/artifacts/"+str(receipt["id"]))
        run=github.metadata("/repos/"+REPO+"/actions/runs/"+str(receipt["run"]))
        gate.receipt(role,metadata,run)
        evidence.retain(role+"-artifact-metadata.json",canonical(metadata))
        evidence.retain(role+"-artifact-run.json",canonical(run))
        archive=github.zip(receipt["id"],receipt["size"])
        manifest=unique(gate.member(role,archive,metadata,run,"manifest.json"))
        evidence.retain(role+"-construction-manifest.json",canonical(manifest))
        gate_image=helpers[role+"_inventory"]
        first=None;first_report=None
        for number in (1,2):
            raw=gate.member(role,archive,metadata,run,role+"-"+str(number)+".tar")
            report=gate_image.inspect(raw,manifest["builds"][number-1]["image"])
            if canonical(report)!=canonical(manifest["builds"][number-1]):
                raise Invalid("independent retained image inventory differs")
            evidence.retain(role+"-independent-inventory-"+str(number)+".json",canonical(report))
            if first is None:first=raw;first_report=report
            elif canonical(report)!=canonical(first_report):raise Invalid("two retained provisions differ")
        return first,first_report
    def source_objects():
        root=work/"proposal.git";root.mkdir(mode=0o700,exist_ok=False)
        git(root,["init","--bare","--template="])
        git(root,["fetch","--no-tags","--depth=1","--no-write-fetch-head",
            "https://github.com/"+REPO+".git",target],1048576)
        def obj(kind,oid,maximum):
            revision(oid)
            size=git(root,["cat-file","-s",oid],32)
            if re.fullmatch(b"[0-9]+\n",size) is None or not 1<=int(size)<=maximum:
                raise Invalid("Git object acquisition bound")
            raw=git(root,["cat-file",kind,oid],maximum)
            evidence.retain("source-"+kind+"-"+oid+".bin",raw);return raw
        snapshot=helpers["source_snapshot"]
        commit=obj("commit",target,65536)
        if snapshot._git_hash(b"commit",commit)!=target:raise Invalid("proposal commit bytes")
        first=commit.split(b"\n",1)[0]
        if re.fullmatch(b"tree [0-9a-f]{40}",first) is None:raise Invalid("proposal root tree")
        root_oid=first[5:].decode();trees={};blobs={}
        for path in sorted(snapshot.FILES):
            current=root_oid
            parts=path.encode().split(b"/")
            for index,part in enumerate(parts):
                if current not in trees:trees[current]=obj("tree",current,131072)
                mode,oid=snapshot._tree_entries(trees[current]).get(part,(None,None))
                if index<len(parts)-1:
                    if mode!=b"40000":raise Invalid("proposal directory")
                    current=oid
                else:
                    if mode!=b"100644":raise Invalid("proposal regular source")
                    if oid not in blobs:blobs[oid]=obj("blob",oid,262144)
        layer,report=snapshot.build_from_objects(target,commit,trees,blobs)
        evidence.retain("source-layer.tar",layer);evidence.retain("source-binding.json",canonical(report))
        return layer,report
    try:
        # Bind every imported controller source, recipe and driver to exact main.
        actual=git(workspace/"controller",["rev-parse","HEAD"],128).strip().decode()
        if actual!=controller:raise Invalid("controller checkout revision")
        dirty=git(workspace/"controller",["status","--porcelain=v1","--untracked-files=all"],1048576)
        if dirty:raise Invalid("clean trusted controller required")
        measured={}
        for path in sorted(HERE.iterdir()):
            if path.suffix not in (".py",".rs",".Containerfile") and path.name!="compiler-driver.Containerfile":continue
            if path.is_symlink() or not path.is_file() or not 1<=path.stat().st_size<=131072:
                raise Invalid("trusted source type/bound")
            raw=path.read_bytes();relative="tools/rar-lab/modern/"+path.name
            bound=git(workspace/"controller",["show",controller+":"+relative],131072)
            if bound!=raw:raise Invalid("controller helper differs from main blob")
            measured[relative]={"sha256":sha(raw),"size":len(raw)}
        evidence.retain("controller-sources.json",canonical(measured))
        evidence.retain("host-tools.json",canonical(tool_identities()))
        evidence.retain("docker-version.json",docker(["version","--format","{{json .}}"]))
        phase="artifact-intake"
        parent,parent_report=obtain("compiler")
        refs,refs_report=obtain("reference")
        github.token="";github=None
        phase="source-binding"
        source_layer,source_report=source_objects()
        phase="compiler-parent-load"
        parent_image=load_image(parent,parent_report)
        notices=helpers["adapter_image"].notices_from_parent(parent)
        phase="driver-construction"
        docker(["pull","--platform=linux/amd64",BASE],180,1048576)
        pulled=unique(docker(["image","inspect",BASE]))
        if (type(pulled) is not list or len(pulled)!=1 or
            pulled[0].get("Architecture")!="amd64" or pulled[0].get("Os")!="linux" or
            not any(type(x) is str and x.endswith("@"+BASE.split("@")[1])
                for x in (pulled[0].get("RepoDigests") or []))):
            raise Invalid("pinned driver bootstrap")
        evidence.retain("bootstrap-inspect.json",canonical(pulled))
        builder=docker(["buildx","inspect","default"])
        if [b"Driver:",b"docker"] not in [line.split() for line in builder.splitlines()] or b"BuildKit version:" not in builder:
            raise Invalid("private default Docker BuildKit")
        evidence.retain("builder.txt",builder)
        local_tag="rar-modern-parent:"+run_id+"-1"
        if docker(["image","ls","--no-trunc","--filter","reference="+local_tag,"--format","{{.ID}}"])!=b"":
            raise Invalid("parent tag already exists; no overwrite")
        docker(["image","tag",parent_image,local_tag])
        verify_parent_tag(docker(["image","inspect",local_tag]),parent_image)
        driver_source=(HERE/"compiler_driver.rs").read_bytes()
        recipe=(HERE/"compiler-driver.Containerfile").read_bytes()
        drivers=[]
        for number in (1,2):
            context=work/("driver-context-"+str(number));context.mkdir(mode=0o700,exist_ok=False)
            for name,raw in (("compiler_driver.rs",driver_source),("compiler-driver.Containerfile",recipe)):
                with (context/name).open("xb") as stream:stream.write(raw)
                (context/name).chmod(0o444);os.utime(context/name,(EPOCH,EPOCH))
            output=work/("driver-export-"+str(number)+".tar")
            if output.exists() or output.is_symlink():raise Invalid("new owned driver output")
            before=docker(["image","inspect",local_tag])
            evidence.retain("driver-"+str(number)+"-parent-before.json",before)
            verify_parent_tag(before,parent_image)
            docker(["build","--platform=linux/amd64","--network=none","--no-cache",
                "--pull=false","--progress=plain","--build-arg","COMPILER_PARENT="+local_tag,
                "--output","type=tar,dest="+str(output),"--file",str(context/"compiler-driver.Containerfile"),
                str(context)],900,2*1024**2)
            after=docker(["image","inspect",local_tag])
            evidence.retain("driver-"+str(number)+"-parent-after.json",after)
            verify_parent_tag(after,parent_image)
            export=read_owned(output,64*1024**2)
            evidence.retain("driver-export-"+str(number)+".tar",export)
            driver,consumed=driver_export(export,notices,helpers["compiler_driver_layer"],parent_report)
            evidence.retain("driver-"+str(number)+".bin",driver)
            evidence.retain("driver-export-"+str(number)+".json",canonical({"sha256":sha(export),
                "size":len(export),"driver_sha256":sha(driver),"consumed_sysroot":consumed,
                "notice_sha256":{k:sha(v) for k,v in notices.items()}}))
            drivers.append(driver)
        if drivers[0]!=drivers[1]:raise Invalid("two driver constructions differ")
        driver_layer,driver_report=helpers["compiler_driver_layer"].build(drivers[0],controller,sha(driver_source),sha(recipe))
        evidence.retain("driver-layer.tar",driver_layer);evidence.retain("driver-layer.json",canonical(driver_report))
        phase="derived-compiler"
        derived=helpers["derived_compiler_image"]
        image_raw,image_report=derived.build(parent,driver_layer,sha(drivers[0]),source_layer,target,source_report["files"])
        derived.inspect(image_raw,parent,driver_layer,sha(drivers[0]),source_layer,target,source_report["files"])
        evidence.retain("derived-compiler.json",canonical(image_report))
        # The derived helper reports the exact ordered rootfs as layer digests.
        config_raw=derived._parts(image_raw)[image_report["image"][7:]+".json"]
        # _parts intentionally returns immutable views over large archives.
        # Copy only this bounded JSON member, never the multi-gigabyte layers.
        if len(config_raw)>65536:raise Invalid("derived config JSON budget")
        parsed=unique(bytes(config_raw))
        load_report={"image":image_report["image"],"diff_ids":parsed["rootfs"]["diff_ids"]}
        compiler_image=load_image(image_raw,load_report,parsed["config"])
        del image_raw
        phase="adapter-compilation"
        adapters=[]
        for number in (1,2):
            value,report=helpers["compiler_runner"].execute(compiler_image,evidence.scoped("compile-"+str(number)))
            if not report.get("cleanup_confirmed"):raise Invalid("compiler still live")
            evidence.retain("compile-"+str(number)+"-report.json",canonical(report))
            adapters.append(value)
        if adapters[0]!=adapters[1]:raise Invalid("two adapter compilations differ")
        phase="adapter-image"
        adapter_image=helpers["adapter_image"]
        raw,report=adapter_image.build(parent,adapters[0],target)
        again,other=adapter_image.build(parent,adapters[1],target)
        if raw!=again or report!=other:raise Invalid("two adapter images differ")
        del again
        adapter_image.inspect(raw,parent,adapters[0],target)
        evidence.retain("adapter-image.json",canonical({**report,"archive_sha256":sha(raw),"archive_size":len(raw)}))
        target_image=load_image(raw,{"image":report["image"],"diff_ids":[report["diff_id"]]},adapter_image.PROCESS)
        del raw,parent,drivers,adapters,notices
        reference_image=load_image(refs,refs_report)
        del refs
        phase="fresh-independent-comparisons"
        invocation=0
        def execute(implementation,request):
            nonlocal invocation
            invocation+=1
            if invocation>972:raise Invalid("challenge comparison invocation count")
            selected=target_image if implementation==3 else reference_image
            return transport.execute(selected,implementation,request,evidence.scoped("adapter-"+str(invocation)))
        result=helpers["reference_comparison"].compare_all(execute,evidence.retain,challenge=True)
        if invocation!=972:raise Invalid("complete challenge comparison count")
        phase="crypto-failure-probes"
        summary.update(comparison=result,target_image=target_image,reference_image=reference_image,
            source_binding=source_report)
        summary["failure_probes"]=helpers["crypto_failure_probes"].run(
            transport,target_image,reference_image,evidence.retain)

        # Fixed corpus, fresh challenges and bounded process-failure probes.
        # Retained replay and milestone acceptance remain separate; passing
        # these probes is not a general sandbox or cryptographic-security proof.
        summary.update(status="challenge-corpus-compared",comparison=result,
            target_image=target_image,reference_image=reference_image,
            source_binding=source_report,crypto_interoperability_accepted=False)
        phase="complete-challenge-corpus"
    except BaseException as error:
        summary.update(status="failed",error_type=type(error).__name__)
        if phase!="artifact-intake":
            summary["validation_error"]=str(error)[:2048]
        print(canonical({"diagnostic":"rar-modern-crypto-failure-v0","phase":phase,
            "error_type":summary["error_type"],
            **({"validation_error":summary["validation_error"]} if "validation_error" in summary else {})
            }).decode("ascii"),end="",flush=True)
        # Do not leak a signed archive URL or credential via an exception repr.
        raise Invalid("handoff failed during "+phase+" ("+type(error).__name__+")") from None
    finally:
        signal.alarm(0)
        summary.update(phase=phase,controller_maxrss_kib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
            evidence_files=dict(evidence.entries))
        try:evidence.retain("manifest.json",canonical(summary))
        finally:evidence.close()

if __name__=="__main__":
    if sys.argv!=[sys.argv[0]]:raise SystemExit("fixed cloud workflow entrypoint only")
    main()
