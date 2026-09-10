"""Cloud-only source tests for the integrated crypto handoff boundaries.
Mocks networking/process activation; file fixtures are fresh disposable cloud
scratch files only. No adapter/compiler/image is started by this test entry.
"""
import importlib.util
import io
from contextlib import redirect_stdout
import os
from pathlib import Path
import stat
import struct
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or not sys.dont_write_bytecode):
    raise SystemExit("isolated cloud source tests only")
path=Path(__file__).resolve().with_name("crypto_handoff.py")
spec=importlib.util.spec_from_file_location("rar_handoff_tested",path)
h=importlib.util.module_from_spec(spec);sys.modules[spec.name]=h;spec.loader.exec_module(h)

def executable():
    raw=bytearray(256);raw[:7]=b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHI",raw,16,2,62,1)
    struct.pack_into("<QQ",raw,24,0x4000b0,64)
    struct.pack_into("<HHH",raw,52,64,56,2)
    struct.pack_into("<IIQQQQQQ",raw,64,1,5,0,0x400000,0,256,256,4096)
    struct.pack_into("<IIQQQQQQ",raw,120,0x6474e551,6,0,0,0,0,0,16)
    return bytes(raw)

def parent_fixture():
    root=h.MUSL_ROOT;value=b"public musl fixture"
    return {"files":{root+"/lib/libfixture.rlib":{"size":len(value),"sha256":h.sha(value),
        "mode":0o444,"uid":0,"gid":0}},
        "directories":{root:(0o555,0,0,h.EPOCH),root+"/lib":(0o555,0,0,h.EPOCH)}}

def export(change=None,proofs=None):
    files={"rar-compile-driver":executable(),"licenses/musl/COPYRIGHT":b"public notice",
        **(h.consumed_inventory(parent_fixture()) if proofs is None else proofs)}
    directories=("licenses","licenses/musl")
    out=io.BytesIO()
    with tarfile.open(fileobj=out,mode="w",format=tarfile.USTAR_FORMAT) as archive:
        for name in directories:
            item=tarfile.TarInfo(name);item.type=tarfile.DIRTYPE;item.mode=0o555
            item.mtime=h.EPOCH;archive.addfile(item)
        for name,value in files.items():
            item=tarfile.TarInfo(name);item.size=len(value);item.mtime=h.EPOCH
            item.mode=0o555 if name=="rar-compile-driver" else 0o444
            if change is not None:change(item)
            archive.addfile(item,io.BytesIO(value))
    return out.getvalue()

class Tests(unittest.TestCase):
    def test_integrated_pipeline_and_compiler_mismatch_stop(self):
        from types import SimpleNamespace as NS
        snapshot=h.module("source_snapshot")
        receipts=h.module("construction_artifacts").RECEIPTS
        for failure in (None,"compiler","tag","sysroot","client","config-budget","probes"):
            root=Path(tempfile.mkdtemp(prefix="rar-handoff-pipeline-"))
            here=root/"controller/tools/rar-lab/modern";here.mkdir(parents=True)
            for name,raw in (("compiler_driver.rs",b"fn main() {}\n"),
                             ("compiler-driver.Containerfile",b"FROM scratch\n")):
                (here/name).write_bytes(raw)
            tree_node={};objects={}
            for path in snapshot.FILES:
                node=tree_node;parts=path.split("/")
                for name in parts[:-1]:node=node.setdefault(name,{})
                node[parts[-1]]=("// public fixture "+path+"\n").encode()
            def tree(node):
                raw=bytearray()
                for name,value in sorted(node.items()):
                    if isinstance(value,dict):mode=b"40000";oid=tree(value)
                    else:
                        mode=b"100644";oid=snapshot._git_hash(b"blob",value);objects[oid]=value
                    raw.extend(mode+b" "+name.encode()+b"\0"+bytes.fromhex(oid))
                raw=bytes(raw);oid=snapshot._git_hash(b"tree",raw);objects[oid]=raw;return oid
            commit=b"tree "+tree(tree_node).encode()+b"\nauthor Fixture <x@example.invalid> 1 +0000\n\nfixture\n"
            target=snapshot._git_hash(b"commit",commit);objects[target]=commit
            controller="1"*40
            images={key:"sha256:"+letter*64 for key,letter in
                    (("compiler","a"),("reference","b"),("derived","c"),("adapter","d"))}
            diffs={key:["sha256:"+letter*64] for key,letter in
                   (("compiler","e"),("reference","f"),("derived","1"),("adapter","2"))}
            process={"User":"65532:65532","WorkingDir":"/",
                     "Entrypoint":["/target-reference"],"Env":["PATH=/nonexistent"]}
            compiler_process={**process,"WorkingDir":"/source",
                              "Entrypoint":["/rar-compile-driver"],
                              "Env":["PATH=/nonexistent","RAR_COMPILER_ROLE=modern-v0"]}
            reports={role:{"image":images[role],"diff_ids":diffs[role]} for role in ("compiler","reference")}
            def member(role,raw,metadata,run,name):
                if name=="manifest.json":return h.canonical({"builds":[reports[role],reports[role]]})
                return role.encode()
            gate=NS(RECEIPTS=receipts,receipt=lambda *a:None,member=member)
            api_calls=[]
            class API:
                def __init__(self,token):self.token=token
                def metadata(self,path):api_calls.append(path);return {}
                def zip(self,artifact,size):return b"symbolic-archive"
            compile_count=0;adapter_count=0;build_count=0;tagged=False;loaded=[]
            def compile_adapter(image,retain):
                nonlocal compile_count
                compile_count+=1
                value=executable()[:-1]+b"\x01" if failure=="compiler" and compile_count==2 else executable()
                retain("compiler-stdout.bin",value)
                return value,{"cleanup_confirmed":True}
            def adapter_run(image,implementation,request,retain):
                nonlocal adapter_count
                self.assertEqual(compile_count,2)
                adapter_count+=1
                if adapter_count in (1,324,325,972):retain("wire.bin",b"fixture")
                return implementation,0,b"fixture",b""
            def comparison(execute,retain,*,challenge):
                self.assertIs(challenge,True)
                # The real comparison component has separate full ordering and
                # protocol fixtures. This tests the outer pipeline's invocation
                # count, lifecycle sequencing, selected images and evidence path.
                for _ in range(324):execute(3,b"fixed fixture")
                retain("frozen-rar-results.json",b"{}\n")
                for _ in range(324):
                    execute(1,b"fixed fixture");execute(2,b"fixed fixture")
                return {"three_way_agreement":True,"milestone_complete":False}
            probe_calls=[]
            probe_report={"schema":"rar-modern-crypto-failure-probes-v1",
                "expected_failures":9,"cleanup_confirmed":True,"milestone_complete":False}
            def probe_run(transport,target_image,reference_image,retain):
                self.assertEqual(compile_count,2)
                self.assertEqual(adapter_count,972)
                self.assertIs(transport,helpers["reference_runner"])
                self.assertEqual(target_image,images["adapter"])
                self.assertEqual(reference_image,images["reference"])
                probe_calls.append(True)
                raw=b"public probe fixture"
                self.assertEqual(retain("probe-fixture.bin",raw),h.sha(raw))
                if failure=="probes":raise RuntimeError("probe failure fixture")
                return probe_report
            def inspect_image(key):
                return {"Id":images[key],"Architecture":"amd64","Os":"linux",
                    "RootFS":{"Type":"layers","Layers":diffs[key]},
                    "Config":compiler_process if key=="derived" else process}
            def exchange(argv,request,seconds,maximum,error_max):
                nonlocal build_count,tagged
                if "/usr/bin/git" in argv:
                    at=argv.index("-C");args=argv[at+2:]
                    if args[:2]==["rev-parse","HEAD"]:out=controller.encode()+b"\n"
                    elif args[0]=="status":out=b""
                    elif args[0]=="show":out=(here/args[1].rsplit("/",1)[-1]).read_bytes()
                    elif args[0] in ("init","fetch"):out=b""
                    elif args[:2]==["cat-file","-s"]:out=str(len(objects[args[2]])).encode()+b"\n"
                    elif args[0]=="cat-file":out=objects[args[2]]
                    else:raise AssertionError(args)
                    return 0,out,b""
                config=root/"modern-crypto-work/docker-config"
                buildx=root/"modern-crypto-work/buildx-config"
                prefix=["/usr/bin/env","-i","PATH=/usr/bin:/bin","LANG=C","LC_ALL=C",
                    "DOCKER_CONFIG="+str(config),"BUILDX_CONFIG="+str(buildx)]+h.DOCKER+["--config",str(config)]
                self.assertEqual(argv[:len(prefix)],prefix)
                self.assertTrue(config.is_dir() and buildx.is_dir())
                args=argv[len(prefix):]
                if args[0]=="version":out=b"{}\n"
                elif args[:2]==["image","load"]:
                    loaded.append(request);out=b"loaded\n"
                elif args[:2]==["image","inspect"]:
                    name=args[2]
                    if name==h.BASE:
                        out=h.canonical([{"Architecture":"amd64","Os":"linux","RepoDigests":[h.BASE]}])
                    else:
                        key="compiler" if name.startswith("rar-modern-parent:") else next(k for k,v in images.items() if v==name)
                        item=inspect_image(key)
                        if failure=="tag" and name.startswith("rar-modern-parent:") and build_count:
                            item["Id"]=images["reference"]
                        out=h.canonical([item])
                elif args[0]=="pull":out=b"pulled\n"
                elif args[:2]==["buildx","inspect"]:
                    if failure=="client":
                        return 1,b"x"*3000,b"y"*9000+b"\n::error::inert public fixture\n"
                    out=b"Driver: docker\nBuildKit version: fixture\n"
                elif args[:2]==["image","ls"]:out=b"existing\n" if tagged else b""
                elif args[:2]==["image","tag"]:tagged=True;out=b""
                elif args[0]=="build":
                    build_count+=1
                    output=args[args.index("--output")+1].removeprefix("type=tar,dest=")
                    Path(output).write_bytes(b"symbolic driver export")
                    out=b"built\n"
                else:raise AssertionError(args)
                return 0,out,b""
            adapter_report={"image":images["adapter"],"diff_id":diffs["adapter"][0]}
            helpers={
                "construction_artifacts":gate,"source_snapshot":snapshot,
                "compiler_inventory":NS(inspect=lambda *a:reports["compiler"]),
                "reference_inventory":NS(inspect=lambda *a:reports["reference"]),
                "compiler_driver_layer":NS(build=lambda *a:(b"driver layer",{})),
                "derived_compiler_image":NS(
                    build=lambda *a:(b"derived",{"image":images["derived"]}),inspect=lambda *a:None,
                    _parts=lambda raw:{images["derived"][7:]+".json":memoryview(b" "*65537 if failure=="config-budget" else h.canonical(
                        {"rootfs":{"diff_ids":diffs["derived"]},"config":compiler_process}))}),
                "adapter_image":NS(notices_from_parent=lambda *a:{"licenses/public":b"notice"},
                    build=lambda *a:(b"adapter",adapter_report),inspect=lambda *a:None,PROCESS=process),
                "compiler_runner":NS(execute=compile_adapter),
                "reference_runner":NS(exchange=exchange,execute=adapter_run),
                "reference_comparison":NS(compare_all=comparison),
                "crypto_failure_probes":NS(run=probe_run)}
            def checked_export(*args):
                if failure=="sysroot":raise h.Invalid("consumed sysroot mismatch fixture")
                return executable(),{}
            diagnostics=io.StringIO()
            env={"GITHUB_RUN_ID":"1","ImageVersion":"fixture","RAR_ARTIFACT_TOKEN":"fixture"}
            with redirect_stdout(diagnostics),patch.dict(os.environ,env,clear=True),patch.object(h,"HERE",here),\
                patch.object(h,"guard",return_value=(root,controller,target,{})),\
                patch.object(h,"GitHub",API),patch.object(h,"module",side_effect=lambda n:helpers[n]),\
                patch.object(h,"driver_export",side_effect=checked_export),\
                patch.object(h,"tool_identities",return_value={"fixture":{}}),\
                patch.object(h.resource,"setrlimit"),patch.object(h.signal,"signal"),patch.object(h.signal,"alarm"):
                if failure is not None:
                    with self.assertRaises(h.Invalid):h.main()
                else:h.main()
            manifest=h.unique((root/"modern-crypto-evidence/manifest.json").read_bytes())
            self.assertEqual(build_count,0 if failure=="client" else 1 if failure in ("tag","sysroot") else 2)
            self.assertEqual(compile_count,0 if failure in ("tag","sysroot","client","config-budget") else 2)
            self.assertEqual(len(api_calls),4)
            self.assertFalse(manifest["milestone_complete"])
            if failure is not None:
                self.assertEqual(adapter_count,972 if failure=="probes" else 0)
                self.assertEqual(probe_calls,[True] if failure=="probes" else [])
                self.assertEqual(loaded,[b"compiler"] if failure in ("tag","sysroot","client","config-budget") else [b"compiler",b"derived",b"adapter",b"reference"] if failure=="probes" else [b"compiler",b"derived"])
                self.assertEqual(manifest["phase"],"crypto-failure-probes" if failure=="probes" else "derived-compiler" if failure=="config-budget" else "driver-construction" if failure in ("tag","sysroot","client") else "adapter-compilation")
                self.assertEqual(manifest["status"],"failed")
                if failure=="client":
                    lines=diagnostics.getvalue().splitlines()
                    self.assertTrue(lines)
                    self.assertTrue(all(line.startswith("{") for line in lines))
                    records=[h.unique(line.encode("ascii")) for line in lines]
                    command=[row for row in records if row.get("diagnostic")=="rar-modern-command-failure-v0"]
                    self.assertEqual(len(command),1)
                    self.assertEqual(command[0]["exit"],1)
                    self.assertEqual(command[0]["stdout_tail"],"x"*2048)
                    self.assertEqual(len(command[0]["stderr_tail"]),8192)
                    self.assertTrue(command[0]["stderr_tail"].endswith("\n::error::inert public fixture\n"))
                    self.assertNotIn("RAR_ARTIFACT_TOKEN"," ".join(command[0]["argv"]))
            else:
                self.assertEqual(adapter_count,972)
                self.assertEqual(probe_calls,[True])
                self.assertEqual(manifest["failure_probes"],probe_report)
                self.assertEqual(loaded,[b"compiler",b"derived",b"adapter",b"reference"])
                self.assertEqual(manifest["status"],"challenge-corpus-compared")
                self.assertEqual(manifest["phase"],"complete-challenge-corpus")
                self.assertFalse(manifest["crypto_interoperability_accepted"])

    def test_private_docker_configuration_and_substitution_refusal(self):
        root=Path(tempfile.mkdtemp(prefix="rar-docker-client-"))
        client=h.DockerClient(root)
        for path in client.paths:
            self.assertEqual(list(path.iterdir()),[])
            self.assertEqual(stat.S_IMODE(path.stat().st_mode),0o700)
        argv=client.argv(["buildx","inspect","default"])
        self.assertEqual(argv[:2],["/usr/bin/env","-i"])
        self.assertIn("DOCKER_CONFIG="+str(root/"docker-config"),argv)
        self.assertIn("BUILDX_CONFIG="+str(root/"buildx-config"),argv)
        self.assertEqual(argv[-3:],["buildx","inspect","default"])
        self.assertNotIn("/nonexistent",argv)
        with self.assertRaises(FileExistsError):h.DockerClient(root)
        # Read-only stat adapters simulate substitution without deleting or
        # replacing any fixture or changing real permissions.
        from types import SimpleNamespace as NS
        info=client.paths[0].lstat()
        for field,value in (("st_ino",info.st_ino+1),("st_uid",info.st_uid+1),
                            ("st_mode",stat.S_IFLNK|0o700),("st_mode",stat.S_IFDIR|0o755)):
            changed=NS(st_dev=info.st_dev,st_ino=info.st_ino,st_uid=info.st_uid,st_mode=info.st_mode)
            setattr(changed,field,value)
            with patch.object(Path,"lstat",return_value=changed):
                with self.assertRaises(h.Invalid):client.argv(["buildx","inspect","default"])

    def test_host_default_denied_before_mutation_or_loading(self):
        with patch.dict(os.environ,{},clear=True),patch.object(h,"Evidence") as writer,patch.object(h,"module") as loader:
            with self.assertRaises(h.Invalid):h.main()
            writer.assert_not_called();loader.assert_not_called()
    def test_archive_location_and_api_path_scope(self):
        good="https://fixture.blob.core.windows.net/actions/bytes?sig=public-fixture"
        self.assertEqual(h.archive_url(good),good)
        for value in (good.replace("https:","http:"),good+"#fragment",
            good.replace("fixture.blob.core.windows.net","evil.example"),
            good.replace("fixture.blob.core.windows.net","blob.core.windows.net.evil.example"),
            good.replace("fixture.blob.core.windows.net","user:pass@fixture.blob.core.windows.net"),
            good.replace("fixture.blob.core.windows.net","fixture.blob.core.windows.net:8443"),
            "https://127.0.0.1/","file:///owner",None):
            with self.assertRaises(h.Invalid):h.archive_url(value)
        client=h.GitHub("ephemeral-fixture")
        path="/repos/AndyTechCoder/RAR-OS/actions/artifacts/10004371629/zip"
        request=client.request(path)
        self.assertEqual(request.full_url,"https://api.github.com"+path)
        self.assertEqual(request.get_header("Authorization"),"Bearer ephemeral-fixture")
        for path in ("/repos/other/repo/actions/artifacts/1/zip",
            "/repos/AndyTechCoder/RAR-OS/contents/private",
            "/repos/AndyTechCoder/RAR-OS/actions/artifacts/1/zip?token=x"):
            with self.assertRaises(h.Invalid):client.request(path)
    def test_archive_redirect_drops_token_and_refuses_wrong_status(self):
        from urllib.error import HTTPError
        from email.message import Message
        client=h.GitHub("ephemeral-fixture")
        headers=Message();headers["Location"]="https://fixture.blob.core.windows.net/a?sig=fixture"
        redirect=HTTPError("https://api.github.com/fixed",302,"redirect",headers,io.BytesIO())
        with patch.object(client.opener,"open",side_effect=redirect),patch.object(client,"read",return_value=b"abc") as read:
            self.assertEqual(client.zip(1,3),b"abc")
            sent=read.call_args.args[0]
            self.assertIsNone(sent.get_header("Authorization"))
            self.assertEqual(sent.full_url,headers["Location"])
        for status in (301,307,403):
            response=HTTPError("https://api.github.com/fixed",status,"failure",headers,io.BytesIO())
            with patch.object(client.opener,"open",side_effect=response),patch.object(client,"read") as read:
                with self.assertRaises(h.Invalid):client.zip(1,3)
                read.assert_not_called()
    def test_http_body_bounds(self):
        class Response:
            status=200
            def __init__(self,raw,headers=None):self.stream=io.BytesIO(raw);self.headers=headers or {}
            def __enter__(self):return self
            def __exit__(self,*args):return False
            def read(self,n):return self.stream.read(n)
        client=h.GitHub("fixture")
        for raw,headers,maximum in ((b"1234",{},3),(b"x",{"Content-Length":"4"},3),
                                     (b"x",{"Content-Length":"bad"},3)):
            with patch.object(client.opener,"open",return_value=Response(raw,headers)):
                with self.assertRaises(h.Invalid):client.read(object(),maximum,60)
        with patch.object(client.opener,"open",return_value=Response(b"123",{"Content-Length":"3"})):
            self.assertEqual(client.read(object(),3,60),b"123")
    def test_evidence_positive_exclusive_bounds_and_no_follow(self):
        # Fixtures remain inside fresh cloud scratch; no pre-existing path is
        # overwritten or deleted by these tests.
        root=Path(tempfile.mkdtemp(prefix="rar-handoff-source-"))
        evidence=h.Evidence(root,"evidence",budget=8)
        try:
            self.assertEqual(evidence.retain("first.bin",b"abc"),h.sha(b"abc"))
            self.assertEqual((evidence.root/"first.bin").read_bytes(),b"abc")
            self.assertEqual(stat.S_IMODE((evidence.root/"first.bin").stat().st_mode),0o600)
            for name,raw in (("first.bin",b"x"),("../owner",b"x"),("bad/path",b"x"),
                             ("second.bin",b"123456"),("third.bin",bytearray(b"x"))):
                with self.assertRaises(h.Invalid):evidence.retain(name,raw)
            (evidence.root/"link.bin").symlink_to(evidence.root/"first.bin")
            # Stay below the quota so this exercises exclusive/no-follow open.
            with self.assertRaises(OSError):evidence.retain("link.bin",b"x")
            self.assertNotIn("link.bin",evidence.entries)
            self.assertEqual(evidence.total,3)
            self.assertEqual((evidence.root/"first.bin").read_bytes(),b"abc")
        finally:evidence.close()
        with self.assertRaises(h.Invalid):evidence.retain("closed",b"")
        with self.assertRaises(FileExistsError):h.Evidence(root,"evidence")
    def test_evidence_never_acknowledges_failed_fsync(self):
        root=Path(tempfile.mkdtemp(prefix="rar-handoff-sync-"))
        evidence=h.Evidence(root,"evidence")
        try:
            with patch.object(h.os,"fsync",side_effect=OSError("fixture fsync")):
                with self.assertRaises(OSError):evidence.retain("unsynced.bin",b"public fixture")
            self.assertNotIn("unsynced.bin",evidence.entries)
        finally:evidence.close()
    def test_driver_export_positive_real_static_gate_and_mutations(self):
        gate=h.module("compiler_driver_layer")
        notices={"licenses/musl/COPYRIGHT":b"public notice"}
        self.assertEqual(h.driver_export(export(),notices,gate,parent_fixture())[0],executable())
        for field,value in (("mode",0o777),("uid",1),("mtime",h.EPOCH+1),
                            ("name","../outside"),("type",tarfile.SYMTYPE)):
            def change(item,field=field,value=value):
                if item.name=="rar-compile-driver":setattr(item,field,value)
            with self.assertRaises((h.Invalid,ValueError)):
                h.driver_export(export(change),notices,gate,parent_fixture())
        with self.assertRaises(h.Invalid):
            h.driver_export(export(),{"licenses/musl/COPYRIGHT":b"altered"},gate,parent_fixture())
        with self.assertRaises(h.Invalid):h.driver_export(b"bad",notices,gate,parent_fixture())
    def test_consumed_sysroot_and_parent_tag_substitution_refusals(self):
        import copy
        parent=parent_fixture();proofs=h.consumed_inventory(parent)
        notices={"licenses/musl/COPYRIGHT":b"public notice"};gate=h.module("compiler_driver_layer")
        driver,report=h.driver_export(export(),notices,gate,parent)
        self.assertEqual(driver,executable())
        self.assertEqual(set(report),set(h.CONSUMED))
        for name in proofs:
            changed=dict(proofs);changed[name]=b"wrong consumed bytes\n"
            with self.assertRaises(h.Invalid):h.driver_export(export(proofs=changed),notices,gate,parent)
        for field,value in (("mode",0o555),("uid",1)):
            def change(item,field=field,value=value):
                if item.name==h.CONSUMED[0]:setattr(item,field,value)
            with self.assertRaises(h.Invalid):h.driver_export(export(change),notices,gate,parent)
        altered=copy.deepcopy(parent)
        next(iter(altered["files"].values()))["sha256"]="0"*64
        with self.assertRaises(h.Invalid):h.driver_export(export(),notices,gate,altered)
        good=[{"Id":"sha256:"+"a"*64,"Architecture":"amd64","Os":"linux"}]
        self.assertTrue(h.verify_parent_tag(h.canonical(good),good[0]["Id"]))
        for field,value in (("Id","sha256:"+"b"*64),("Architecture","arm64"),("Os","other")):
            changed=copy.deepcopy(good);changed[0][field]=value
            with self.assertRaises(h.Invalid):h.verify_parent_tag(h.canonical(changed),good[0]["Id"])

    def test_owned_output_read_and_symlink_refusal(self):
        root=Path(tempfile.mkdtemp(prefix="rar-handoff-read-"))
        path=root/"output";path.write_bytes(b"public fixture")
        self.assertEqual(h.read_owned(path,64),b"public fixture")
        with self.assertRaises(h.Invalid):h.read_owned(path,4)
        link=root/"link";link.symlink_to(path)
        with self.assertRaises(OSError):h.read_owned(link,64)
    def test_revisions_and_json_canonical_boundaries(self):
        self.assertEqual(h.revision("1"*40),"1"*40)
        for value in ("main","0"*40,True,None,"A"*40):
            with self.assertRaises(h.Invalid):h.revision(value)
        for raw in (b'{"a":1,"a":2}',b'{"a":NaN}',b'{"a":Infinity}'):
            with self.assertRaises(h.Invalid):h.unique(raw)

if __name__=="__main__":
    unittest.main(argv=[sys.argv[0]],verbosity=2)
