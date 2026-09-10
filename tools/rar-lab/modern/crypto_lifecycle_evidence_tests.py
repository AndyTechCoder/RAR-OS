"""Cloud-only pure fixtures for retained adapter lifecycle validation."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import sys
import unittest
from unittest import mock
if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")

def load(name):
    spec=importlib.util.spec_from_file_location("lifecycle_test_"+name,Path(__file__).with_name(name+".py"))
    value=importlib.util.module_from_spec(spec);sys.modules[spec.name]=value
    spec.loader.exec_module(value);return value

gate=load("crypto_lifecycle_evidence")
fixtures=load("retained_checkpoint_tests")

def fixture():
    members,manifest=fixtures.fixture(bytes(range(32)))
    frozen=json.loads(members["frozen-rar-results.json"]);results=json.loads(members["three-way-results.json"])
    manifest.update(target_image="sha256:"+"a"*64,reference_image="sha256:"+"b"*64)
    host=dict(NetworkMode="none",IpcMode="none",ReadonlyRootfs=True,Privileged=False,
        NanoCpus=1000000000,Memory=134217728,MemorySwap=134217728,PidsLimit=16,
        PublishAllPorts=False,AutoRemove=False,CapDrop=["ALL"],SecurityOpt=["no-new-privileges"],
        LogConfig=dict(Type="none",Config={}),RestartPolicy=dict(Name="no",MaximumRetryCount=0),
        Ulimits=[dict(Name="core",Soft=0,Hard=0),dict(Name="nofile",Soft=64,Hard=64)])
    for index in range(972):
        if index<324:case=index;ident=3;wire=frozen["cases"][case]["rar"]
        else:
            case,which=divmod(index-324,2);ident=which+1;wire=results["cases"][case]["runs"][which+1]
        cid=f"{index+1:064x}";name="rar-modern-ref-"+f"{index+1:032x}"
        entry={1:"/reference-sodium",2:"/reference-openssl",3:"/target-reference"}[ident]
        item=dict(Id=cid,Image=manifest["target_image" if ident==3 else "reference_image"],
            Name="/"+name,Config=dict(Labels={"rar.modern.owner":name},User="65532:65532",
            Entrypoint=[entry],Cmd=None,Env=["PATH=/nonexistent"],WorkingDir="/"),
            State=dict(Status="created",Running=False),HostConfig=host,Mounts=[])
        after=copy.deepcopy(item);after["State"]=dict(Status="exited",Running=False,
            Paused=False,Restarting=False,OOMKilled=False,Dead=False,Error="",ExitCode=0,Pid=0)
        prefix=f"adapter-{index+1}-"
        rows={"request.bin":bytes.fromhex(frozen["cases"][case]["request"]),
            "stdout.bin":bytes.fromhex(wire["stdout"]),"stderr.bin":bytes.fromhex(wire["stderr"]),
            "create.json":gate.canonical(dict(exit_code=0,stdout=(cid+"\n").encode().hex(),stderr="")),
            "before.json":gate.canonical([item]),"after.json":gate.canonical([after]),
            "cleanup.json":gate.canonical(dict(container_id=cid,confirmed_absent=True))}
        members.update({prefix+leaf:raw for leaf,raw in rows.items()})
    members=fixtures.bind(members,manifest)
    manifest=json.loads(members["manifest.json"])
    return members,manifest,frozen,results

class Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.data=fixture()
    def test_all_972_recorded_lifecycles_without_process_calls(self):
        # The helper exposes process methods but validation must never call them.
        runner=gate.helper()
        with mock.patch.object(runner,"execute") as execute,mock.patch.object(runner,"control") as control, \
             mock.patch.object(runner,"exchange") as exchange,mock.patch.object(gate,"helper",return_value=runner):
            report=gate.validate(*self.data)
            self.assertEqual(report["invocations"],972);self.assertEqual(report["unique_containers"],972)
            self.assertFalse(report["execution_attempted"]);self.assertFalse(report["injected_failures_tested"])
            execute.assert_not_called();control.assert_not_called();exchange.assert_not_called()
    def test_missing_extra_wrong_wire_and_unconfirmed_cleanup(self):
        original,manifest,frozen,results=self.data
        for mode in ("missing","extra","request","stdout","cleanup"):
            members=dict(original)
            if mode=="missing":members.pop("adapter-1-before.json")
            elif mode=="extra":members["adapter-973-before.json"]=b"[]"
            elif mode=="request":members["adapter-1-request.bin"]=b"wrong"
            elif mode=="stdout":members["adapter-1-stdout.bin"]=b"wrong"
            else:
                value=json.loads(members["adapter-1-cleanup.json"]);value["confirmed_absent"]=1
                members["adapter-1-cleanup.json"]=gate.canonical(value)
            with self.subTest(mode=mode),self.assertRaises(ValueError):gate.validate(members,manifest,frozen,results)
    def test_pre_post_effective_confinement_and_stopped_state(self):
        original,manifest,frozen,results=self.data
        for leaf,path,value in (
            ("before.json",("HostConfig","NetworkMode"),"host"),
            ("after.json",("HostConfig","Privileged"),True),
            ("after.json",("State","Running"),True),
            ("after.json",("State","Pid"),True),
            ("after.json",("State","ExitCode"),True),
            ("before.json",("Config","User"),"0:0"),
            ("after.json",("Config","Env"),["PATH=/bin"]),
            ("before.json",("Config","Labels"),{"rar.modern.owner":"wrong"})):
            members=dict(original);name="adapter-1-"+leaf;doc=json.loads(members[name])
            doc[0][path[0]][path[1]]=value;members[name]=gate.canonical(doc)
            with self.subTest(leaf=leaf,path=path),self.assertRaises(ValueError):gate.validate(members,manifest,frozen,results)
    def test_member_association_and_record_framing(self):
        members,manifest,frozen,results=self.data
        for slot in (1,2,3):
            args=[members,manifest,frozen,results];args[slot]=dict(args[slot],unexpected=True)
            with self.assertRaises(ValueError):gate.validate(*args)
        for leaf in ("create.json","cleanup.json"):
            changed=dict(members);name="adapter-1-"+leaf;changed[name]+=b" "
            with self.assertRaises(ValueError):gate.validate(changed,manifest,frozen,results)
        # Docker inspect output is retained verbatim, not canonical record().
        # Whitespace is valid there, while duplicate/nonfinite JSON is not.
        changed=dict(members)
        for leaf in ("before.json","after.json"):
            name="adapter-1-"+leaf
            changed[name]=json.dumps(json.loads(changed[name]),indent=2).encode()
        self.assertEqual(gate.validate(changed,manifest,frozen,results)["invocations"],972)

    def test_duplicate_json_and_container_identity_rejected(self):
        original,manifest,frozen,results=self.data
        with self.assertRaises(ValueError):gate.document(b'{"id":1,"id":2}')
        members=dict(original)
        members["adapter-2-before.json"]=members["adapter-1-before.json"]
        with self.assertRaises(ValueError):gate.validate(members,manifest,frozen,results)
        with self.assertRaises(ValueError):gate.validate(original,dict(manifest,reference_image=manifest["target_image"]),frozen,results)

if __name__=="__main__":unittest.main(argv=[sys.argv[0]],verbosity=2)
