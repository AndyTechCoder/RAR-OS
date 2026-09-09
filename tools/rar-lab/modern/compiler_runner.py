"""Bounded cloud compiler-role invocation; never executes compiler output.
The trusted caller must verify the derived image and construction provenance
before using this helper and retain evidence in a new invocation-owned location.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import struct
import sys
import uuid

MAX_OUTPUT=8*1024*1024
TMPFS="rw,noexec,nosuid,nodev,size=33554432,uid=65532,gid=65532,mode=0700"
ENV=["PATH=/nonexistent","RAR_COMPILER_ROLE=modern-v0"]
EVIDENCE={"compiler-create.json","compiler-before.json","compiler-after.json",
          "compiler-stdout.bin","compiler-stderr.bin","compiler-failure.json",
          "compiler-cleanup.json","compiler-control-stdout.bin","compiler-control-stderr.bin"}

def _load(name):
    if sys.flags.isolated!=1 or not sys.dont_write_bytecode:
        raise RuntimeError("isolated no-bytecode compiler runner required")
    path=Path(__file__).resolve().with_name(name+".py")
    if path.is_symlink() or not path.is_file() or path.stat().st_size>128*1024:
        raise RuntimeError("fixed trusted helper source")
    spec=importlib.util.spec_from_file_location("modern_compiler_runner_"+name,path)
    module=importlib.util.module_from_spec(spec)
    sys.modules[spec.name]=module;spec.loader.exec_module(module)
    return module

transport=_load("reference_runner")
elf=_load("compiler_elf")
RunFailure=transport.RunFailure
cloud_guard=transport.cloud_guard
exchange=transport.exchange
control=transport.control
owned_container=transport.owned_container

def command(image,name):
    if type(image) is not str or re.fullmatch(r"sha256:[0-9a-f]{64}",image) is None:
        raise RunFailure("immutable compiler image")
    if type(name) is not str or re.fullmatch(r"rar-modern-compiler-[0-9a-f]{32}",name) is None:
        raise RunFailure("invocation-owned compiler name")
    return ["/usr/bin/docker","--host=unix:///var/run/docker.sock","create",
        "--name",name,"--label","rar.modern.owner="+name,
        "--pull=never","--interactive","--ipc=none","--network=none",
        "--read-only","--user=65532:65532","--cap-drop=ALL",
        "--security-opt=no-new-privileges","--cpus=2","--memory=1024m",
        "--memory-swap=1024m","--pids-limit=64","--ulimit=core=0:0",
        "--ulimit=nofile=64:64","--tmpfs=/build:"+TMPFS,
        "--log-driver=none","--restart=no","--stop-timeout=1",
        "--entrypoint","/rar-compile-driver",image]

def confined_container(item):
    host=item.get("HostConfig")
    if type(host) is not dict:raise RunFailure("effective compiler confinement absent")
    exact={"NetworkMode":"none","IpcMode":"none","ReadonlyRootfs":True,
        "Privileged":False,"NanoCpus":2000000000,"Memory":1073741824,
        "MemorySwap":1073741824,"PidsLimit":64,"PublishAllPorts":False,
        "AutoRemove":False,"Tmpfs":{"/build":TMPFS}}
    for key,value in exact.items():
        if type(host.get(key)) is not type(value) or host.get(key)!=value:
            raise RunFailure("compiler confinement: "+key)
    for key in ("Binds","Mounts","VolumesFrom","Devices","DeviceRequests",
                "Links","ExtraHosts","PortBindings","CapAdd"):
        if host.get(key) not in (None,[],{}):
            raise RunFailure("unexpected compiler host resource: "+key)
    mounts=item.get("Mounts")
    if mounts not in (None,[]):
        if type(mounts) is not list or len(mounts)!=1 or type(mounts[0]) is not dict:
            raise RunFailure("compiler mount inventory")
        mount=mounts[0]
        if (mount.get("Type")!="tmpfs" or mount.get("Destination")!="/build" or
            mount.get("RW") is not True or mount.get("Source") not in (None,"") or
            mount.get("Name") not in (None,"") or mount.get("Propagation") not in (None,"","rprivate")):
            raise RunFailure("only private compiler tmpfs is allowed")
    if (host.get("CapDrop")!=["ALL"] or
        host.get("SecurityOpt")!=["no-new-privileges"] or
        any(host.get(key) not in (None,"") for key in ("PidMode","UTSMode","UsernsMode")) or
        host.get("LogConfig")!={"Type":"none","Config":{}} or
        host.get("RestartPolicy")!={"Name":"no","MaximumRetryCount":0}):
        raise RunFailure("compiler isolation/log/restart policy")
    limits=host.get("Ulimits")
    required=[{"Name":"core","Soft":0,"Hard":0},{"Name":"nofile","Soft":64,"Hard":64}]
    if (type(limits) is not list or len(limits)!=2 or
        any(type(value) is not dict or set(value)!={"Name","Soft","Hard"} or
            type(value.get("Soft")) is not int or type(value.get("Hard")) is not int for value in limits) or
        any(value not in limits for value in required)):
        raise RunFailure("compiler core/descriptor limits")

def static_output(raw):
    if type(raw) is not bytes or not 176<=len(raw)<=MAX_OUTPUT:
        raise RunFailure("bounded static adapter output")
    metadata=elf.inspect(raw)
    if metadata!={"kind":2,"interpreter":None,"needed":[],"soname":None,"search":None}:
        raise RunFailure("adapter must be static ET_EXEC")
    entry,phoff=struct.unpack_from("<QQ",raw,24)
    phsize,phnum=struct.unpack_from("<HH",raw,54)
    mappings=0;total=0;virtual_ranges=[];file_ranges=[]
    for n in range(phnum):
        typ,flags,offset,address,_,filesz,memsz,align=struct.unpack_from("<IIQQQQQQ",raw,phoff+n*phsize)
        if typ in (2,3):raise RunFailure("adapter dynamic/interpreter segment")
        if typ==1:
            for start,length,ranges in ((address,memsz,virtual_ranges),(offset,filesz,file_ranges)):
                if length:
                    end=start+length
                    if any(start<old_end and old_start<end for old_start,old_end in ranges):
                        raise RunFailure("overlapping adapter load ranges")
                    ranges.append((start,end))
            total+=memsz
            if (total>64*1024*1024 or flags&~7 or
                (align not in (0,1) and (align&(align-1) or align>2*1024*1024)) or
                (align>1 and offset%align!=address%align)):
                raise RunFailure("adapter mapping/alignment budget")
            if flags&1 and address<=entry<address+filesz:mappings+=1
    if mappings!=1:raise RunFailure("unique executable file-backed adapter entry")
    return {"sha256":hashlib.sha256(raw).hexdigest(),"size":len(raw),"mapped_bytes":total}

def _stopped(item):
    state=item.get("State")
    if type(state) is not dict:raise RunFailure("compiler completion state")
    exact={"Status":"exited","Running":False,"Paused":False,"Restarting":False,
           "OOMKilled":False,"Dead":False,"ExitCode":0,"Error":"","Pid":0}
    for key,want in exact.items():
        if type(state.get(key)) is not type(want) or state.get(key)!=want:
            raise RunFailure("compiler incomplete/anomalous exit: "+key)

def execute(image,retain):
    """No retry: ambiguous create/retention/cleanup failures terminate the job.
    retain(fixed_leaf, bytes) must exclusively/durably store bounded evidence and
    acknowledge its SHA256. It must use a fresh evidence directory per call.
    Callbacks are reviewed controller code, never proposal or guest input.
    """
    cloud_guard()
    if not callable(retain):raise RunFailure("trusted evidence retainer required")
    name="rar-modern-compiler-"+uuid.uuid4().hex
    argv=command(image,name)
    def keep(leaf,raw):
        limit=MAX_OUTPUT if leaf=="compiler-stdout.bin" else 65536
        if leaf not in EVIDENCE or type(raw) is not bytes or len(raw)>limit:
            raise RunFailure("bounded fixed compiler evidence")
        digest=hashlib.sha256(raw).hexdigest()
        ack=retain(leaf,raw)
        if type(ack) is not str or ack!=digest:
            raise RunFailure("compiler evidence retention acknowledgement")
    def record(leaf,value):
        keep(leaf,json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()+b"\n")
    cid=None;owned=False;failure=None;result=None;attach_active=False
    try:
        code,raw,error=exchange(argv,b"",10,65,1024)
        record("compiler-create.json",{"exit_code":code,"stdout":raw.hex(),"stderr":error.hex()})
        if type(code) is not int or code!=0 or error or re.fullmatch(b"[0-9a-f]{64}"+bytes([10]),raw) is None:
            raise RunFailure("ambiguous compiler create; terminate disposable job")
        cid=raw[:-1].decode("ascii")
        before=control(["container","inspect",cid])
        item=owned_container(before,cid,name,image);owned=True
        keep("compiler-before.json",before)
        confined_container(item)
        config=item["Config"];state=item.get("State")
        if (config.get("User")!="65532:65532" or
            config.get("WorkingDir")!="/source" or
            config.get("Entrypoint")!=["/rar-compile-driver"] or
            config.get("Env")!=ENV or config.get("Cmd") not in (None,[]) or
            config.get("Tty") is not False or config.get("OpenStdin") is not True or
            type(state) is not dict or state.get("Status")!="created" or
            state.get("Running") is not False):
            raise RunFailure("compiler pre-start process state")
        attach_active=True
        code,output,error=exchange(
            ["/usr/bin/docker","--host=unix:///var/run/docker.sock",
             "start","--attach","--interactive",cid],b"",120,MAX_OUTPUT,1024)
        attach_active=False
        keep("compiler-stdout.bin",output);keep("compiler-stderr.bin",error)
        after=control(["container","inspect",cid])
        completed=owned_container(after,cid,name,image)
        keep("compiler-after.json",after)
        if type(code) is not int or code!=0 or error:
            raise RunFailure("compiler process/stream failure")
        _stopped(completed)
        inspected=static_output(output)
        result=(output,{"image":image,"container_id":cid,"owned_name":name,
                       "output":inspected,"output_executed":False})
    except Exception as error:
        failure=RunFailure(str(error)[:4096]+"; terminate disposable job, no retry")
        try:
            if hasattr(error,"partial_stdout") and hasattr(error,"partial_stderr"):
                prefix="compiler" if attach_active else "compiler-control"
                keep(prefix+"-stdout.bin",error.partial_stdout)
                keep(prefix+"-stderr.bin",error.partial_stderr)
            record("compiler-failure.json",{"failure":str(failure),
                "stream_truncated":bool(getattr(error,"stream_truncated",False)),
                "successful_compilation":False})
        except Exception:failure=RunFailure("compiler execution/evidence failed; terminate disposable job, no retry")
    finally:
        if owned:
            try:
                control(["container","rm","--force",cid],128)
                remaining=control(["container","ls","--all","--no-trunc",
                                   "--filter","id="+cid,"--format","{{.ID}}"],65)
                if remaining!=b"":raise RunFailure("owned compiler remains")
                record("compiler-cleanup.json",{"container_id":cid,"confirmed_absent":True})
            except Exception:
                failure=RunFailure("compiler cleanup/evidence unconfirmed; terminate disposable job, no retry")
    if failure is not None:raise failure
    if result is None:raise RunFailure("missing compiler result")
    result[1]["cleanup_confirmed"]=True
    return result

def self_test():
    import copy
    import unittest
    from unittest.mock import patch
    def executable():
        raw=bytearray(256);raw[:7]=b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HHI",raw,16,2,62,1)
        struct.pack_into("<QQ",raw,24,0x4000b0,64)
        struct.pack_into("<HHH",raw,52,64,56,2)
        struct.pack_into("<IIQQQQQQ",raw,64,1,5,0,0x400000,0,256,256,4096)
        struct.pack_into("<IIQQQQQQ",raw,120,0x6474e551,6,0,0,0,0,0,16)
        return bytes(raw)
    def fixture():
        image="sha256:"+"a"*64;cid="c"*64;name="rar-modern-compiler-"+"b"*32
        host={"NetworkMode":"none","IpcMode":"none","ReadonlyRootfs":True,
            "Privileged":False,"NanoCpus":2000000000,"Memory":1073741824,
            "MemorySwap":1073741824,"PidsLimit":64,"PublishAllPorts":False,
            "AutoRemove":False,"Tmpfs":{"/build":TMPFS},"CapDrop":["ALL"],
            "SecurityOpt":["no-new-privileges"],"LogConfig":{"Type":"none","Config":{}},
            "RestartPolicy":{"Name":"no","MaximumRetryCount":0},
            "Ulimits":[{"Name":"core","Soft":0,"Hard":0},{"Name":"nofile","Soft":64,"Hard":64}]}
        item={"Id":cid,"Image":image,"Name":"/"+name,"HostConfig":host,"Mounts":[],
            "Config":{"Labels":{"rar.modern.owner":name},"User":"65532:65532",
                "WorkingDir":"/source","Entrypoint":["/rar-compile-driver"],"Env":ENV,
                "Cmd":None,"Tty":False,"OpenStdin":True},
            "State":{"Status":"created","Running":False}}
        after=copy.deepcopy(item)
        after["State"]={"Status":"exited","Running":False,"Paused":False,
            "Restarting":False,"OOMKilled":False,"Dead":False,"ExitCode":0,"Error":"","Pid":0}
        return image,cid,item,after
    def retain(leaf,raw):return hashlib.sha256(raw).hexdigest()
    class Tests(unittest.TestCase):
        def test_exact_policy_and_identity(self):
            image,cid,item,after=fixture()
            argv=command(image,item["Name"][1:])
            for flag in ("--pull=never","--network=none","--read-only","--ipc=none",
                         "--cap-drop=ALL","--tmpfs=/build:"+TMPFS):
                self.assertIn(flag,argv)
            self.assertEqual(argv[-3:],["--entrypoint","/rar-compile-driver",image])
            for bad in ("latest",None,"--privileged","sha256:"+"A"*64):
                with self.assertRaises(RunFailure):command(bad,item["Name"][1:])
            for bad in ("existing",None,item["Name"][1:]+"x"):
                with self.assertRaises(RunFailure):command(image,bad)
        def test_effective_configuration_refusals(self):
            image,cid,item,after=fixture();confined_container(item)
            for key in item["HostConfig"]:
                bad=copy.deepcopy(item);bad["HostConfig"][key]=None
                with self.assertRaises(RunFailure):confined_container(bad)
            for key in ("Binds","Mounts","VolumesFrom","Devices","DeviceRequests",
                        "Links","ExtraHosts","PortBindings","CapAdd"):
                bad=copy.deepcopy(item);bad["HostConfig"][key]=["unexpected"]
                with self.assertRaises(RunFailure):confined_container(bad)
            for mounts in ([{"Type":"bind","Destination":"/build","Source":"/owner","RW":True}],
                           [{"Type":"tmpfs","Destination":"/elsewhere","RW":True}],
                           [{"Type":"tmpfs","Destination":"/build","RW":True,"Source":"/owner"}]):
                bad=copy.deepcopy(item);bad["Mounts"]=mounts
                with self.assertRaises(RunFailure):confined_container(bad)
            item["Mounts"]=[{"Type":"tmpfs","Destination":"/build","RW":True,"Source":""}]
            confined_container(item)
        def test_static_output_limits_and_mutations(self):
            raw=executable();self.assertEqual(static_output(raw)["size"],256)
            for offset,fmt,value in ((16,"<H",3),(18,"<H",183),(24,"<Q",0),
                                    (64,"<I",2),(68,"<I",7),(124,"<I",7)):
                bad=bytearray(raw);struct.pack_into(fmt,bad,offset,value)
                with self.assertRaises((ValueError,RunFailure)):static_output(bytes(bad))
            for bad in (b"",raw[:175],bytes(MAX_OUTPUT+1)):
                with self.assertRaises(RunFailure):static_output(bad)
        def test_adjacent_loads_allowed_and_overlaps_refused(self):
            raw=bytearray(executable())
            struct.pack_into("<H",raw,56,3)
            struct.pack_into("<QQ",raw,96,232,232)
            struct.pack_into("<IIQQQQQQ",raw,176,1,6,232,0x4000e8,0,24,24,1)
            self.assertEqual(static_output(bytes(raw))["mapped_bytes"],256)
            for offset,value in ((192,0x4000e7),(184,231)):
                bad=bytearray(raw);struct.pack_into("<Q",bad,offset,value)
                with self.assertRaises(RunFailure):static_output(bytes(bad))
        def test_complete_owned_lifecycle_retains_bytes(self):
            image,cid,before,after=fixture();saved={}
            def keep(leaf,raw):
                saved[leaf]=raw;return retain(leaf,raw)
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",side_effect=[
                        (0,(cid+"\n").encode(),b""),(0,executable(),b"")]) as ex:
                    with patch(__name__+".control",side_effect=[
                            json.dumps([before]).encode(),json.dumps([after]).encode(),b"",b""]) as ctl:
                        output,report=execute(image,keep)
                        self.assertEqual(output,executable());self.assertTrue(report["cleanup_confirmed"])
                        self.assertFalse(report["output_executed"])
                        self.assertEqual(ex.call_args_list[1].args[1],b"")
                        self.assertEqual(ex.call_args_list[1].args[2],120)
                        self.assertEqual(ctl.call_args_list[2].args[0],["container","rm","--force",cid])
            self.assertEqual(saved["compiler-stdout.bin"],executable())
            self.assertTrue(json.loads(saved["compiler-cleanup.json"])["confirmed_absent"])
        def test_rejected_config_never_starts_and_owned_id_is_cleaned(self):
            image,cid,before,after=fixture();before["Config"]["Env"]=["PATH=/bin"]
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",return_value=(0,(cid+"\n").encode(),b"")) as ex:
                    with patch(__name__+".control",side_effect=[json.dumps([before]).encode(),b"",b""]) as ctl:
                        with self.assertRaises(RunFailure):execute(image,retain)
                        self.assertEqual(ex.call_count,1)
                        self.assertEqual(ctl.call_args_list[1].args[0],["container","rm","--force",cid])
        def test_retention_failure_before_start_cleans_only_verified_id(self):
            image,cid,before,after=fixture()
            def bad(leaf,raw):
                return "0"*64 if leaf=="compiler-before.json" else retain(leaf,raw)
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",return_value=(0,(cid+"\n").encode(),b"")) as ex:
                    with patch(__name__+".control",side_effect=[json.dumps([before]).encode(),b"",b""]) as ctl:
                        with self.assertRaises(RunFailure):execute(image,bad)
                        self.assertEqual(ex.call_count,1)
                        self.assertEqual(ctl.call_args_list[1].args[0],["container","rm","--force",cid])
        def test_foreign_ownership_is_never_started_or_removed(self):
            image,cid,before,after=fixture();before["Config"]["Labels"]={"rar.modern.owner":"someone-else"}
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",return_value=(0,(cid+"\n").encode(),b"")) as ex:
                    with patch(__name__+".control",return_value=json.dumps([before]).encode()) as ctl:
                        with self.assertRaises(RunFailure):execute(image,retain)
                        self.assertEqual(ex.call_count,1);self.assertEqual(ctl.call_count,1)
        def test_ambiguous_create_never_starts_or_deletes_by_name(self):
            image,cid,before,after=fixture()
            with patch(__name__+".cloud_guard"),patch(__name__+".exchange",return_value=(1,b"",b"collision")) as ex:
                with patch(__name__+".control") as ctl:
                    with self.assertRaises(RunFailure):execute(image,retain)
                    self.assertEqual(ex.call_count,1);ctl.assert_not_called()
        def test_timeout_retains_partial_output_and_cleans_owned_id(self):
            image,cid,before,after=fixture();saved={}
            failure=transport.stream_failure(RunFailure("deadline"),
                {1:bytearray(b"partial"),2:bytearray(b"diagnostic")},MAX_OUTPUT,1024)
            def keep(leaf,raw):
                saved[leaf]=raw;return retain(leaf,raw)
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",side_effect=[(0,(cid+"\n").encode(),b""),failure]) as ex:
                    with patch(__name__+".control",side_effect=[json.dumps([before]).encode(),b"",b""]) as ctl:
                        with self.assertRaises(RunFailure):execute(image,keep)
                        self.assertEqual(ex.call_count,2)
                        self.assertEqual(ctl.call_args_list[1].args[0],["container","rm","--force",cid])
            self.assertEqual(saved["compiler-stdout.bin"],b"partial")
            self.assertEqual(saved["compiler-stderr.bin"],b"diagnostic")
            self.assertFalse(json.loads(saved["compiler-failure.json"])["successful_compilation"])
        def test_post_run_inspect_failure_keeps_compiler_and_control_bytes_separate(self):
            image,cid,before,after=fixture();saved={}
            failure=transport.stream_failure(RunFailure("inspect deadline"),
                {1:bytearray(b"partial Docker JSON"),2:bytearray(b"inspect diagnostic")},65536,1024)
            def keep(leaf,raw):
                if leaf in saved:raise AssertionError("exclusive evidence leaf reused")
                saved[leaf]=raw;return retain(leaf,raw)
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",side_effect=[(0,(cid+"\n").encode(),b""),(0,executable(),b"")]):
                    with patch(__name__+".control",side_effect=[json.dumps([before]).encode(),failure,b"",b""]):
                        with self.assertRaises(RunFailure):execute(image,keep)
            self.assertEqual(saved["compiler-stdout.bin"],executable())
            self.assertEqual(saved["compiler-stderr.bin"],b"")
            self.assertEqual(saved["compiler-control-stdout.bin"],b"partial Docker JSON")
            self.assertEqual(saved["compiler-control-stderr.bin"],b"inspect diagnostic")
            self.assertIn("compiler-cleanup.json",saved)
        def test_cleanup_failure_never_returns_success(self):
            image,cid,before,after=fixture()
            with patch(__name__+".cloud_guard"),patch(__name__+".uuid.uuid4") as uid:
                uid.return_value.hex="b"*32
                with patch(__name__+".exchange",side_effect=[(0,(cid+"\n").encode(),b""),(0,executable(),b"")]):
                    with patch(__name__+".control",side_effect=[
                            json.dumps([before]).encode(),json.dumps([after]).encode(),b"",(cid+"\n").encode()]):
                        with self.assertRaises(RunFailure):execute(image,retain)
        def test_exit_and_oom_anomalies_refused(self):
            image,cid,before,after=fixture();_stopped(after)
            for key in after["State"]:
                bad=copy.deepcopy(after);bad["State"][key]=None
                with self.assertRaises(RunFailure):_stopped(bad)
        def test_guard_prevents_host_execution(self):
            import os
            with patch.dict(os.environ,{},clear=True),patch(__name__+".exchange") as ex:
                with self.assertRaises(RunFailure):execute("sha256:"+"a"*64,retain)
                ex.assert_not_called()
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():raise SystemExit(1)

if __name__=="__main__":
    import os
    if (sys.argv!=[sys.argv[0],"--self-test"] or os.environ.get("CI")!="true" or
        os.environ.get("GITHUB_ACTIONS")!="true" or sys.platform!="linux"):
        raise SystemExit("cloud isolated source-test entrypoint only")
    self_test()
