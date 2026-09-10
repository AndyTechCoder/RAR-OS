"""Cloud-only inert unavailable guest scenario and retained-evidence tests."""
import base64
import copy
import hashlib
import importlib.util
import os
from pathlib import Path
import sys
import unittest
from types import SimpleNamespace as NS
from unittest.mock import patch
if (sys.platform!="linux" or os.environ.get("CI")!="true" or os.environ.get("GITHUB_ACTIONS")!="true" or
    not sys.flags.isolated or not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")
def load(name):
    spec=importlib.util.spec_from_file_location("negative_test_"+name,Path(__file__).with_name(name+".py"))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
visual=load("visual_oracle");persistence=load("persistence")
provision=load("data_provision");scenario=load("mounted_error_scenario")

def expected_keys():
    result=["f3"]
    for command in ("write note denied","list","write note denied","list"):
        result.extend("spc" if key==" " else key for key in command)
        result.append("ret")
    return result+["esc","f1"]

class MountedErrorTests(unittest.TestCase):
    def test_same_vm_error_then_followup_and_owned_cleanup(self):
        for failure in (None,"capture","changed-data","extra-io","wrong-fault","generic-error"):
            events=[];fixtures=[];vms=[];captures=[]
            class Fixture:
                def __init__(self,root,role,data):
                    self.role=role;self.data=data;self.fd=role;self.initial=persistence.sha(data);self.closed=False
                    fixtures.append(self);events.append(("fixture",role))
                def freeze(self,owned):
                    self_check.assertTrue(all(vm.closed for vm in owned))
                    if failure=="changed-data" and self.role=="data":return b"X"+self.data[1:]
                    return self.data
                def close(self):self.closed=True
            class VM:
                def __init__(self,index,data,system,data_fault):
                    self_check.assertEqual(data_fault,dict(operation="write",ordinal=1,effect="error",prefix=0))
                    self_check.assertEqual((index,data,system),(1,"data","system"))
                    self.keys=[];self.closed=False;vms.append(self)
                def start(self):events.append(("start",))
                def key(self,key):
                    self.keys.append(key)
                    if key=="ret" and self.keys.count("ret")==1:
                        if failure=="generic-error":raise ValueError("not a planned fault")
                        raise Planned()
                def service(self):raise AssertionError("mock must signal at Enter")
                def fault_receipt(self,observation):
                    self_check.assertIs(type(observation),Planned)
                    plan=dict(scenario.FAULT)
                    if failure=="wrong-fault":plan["ordinal"]=2
                    return {"plan":plan}
                def destroy(self):self.closed=True;events.append(("destroy",))
            class Planned(RuntimeError):pass
            def joined(vm,observation):
                self_check.assertIs(type(observation),Planned);vm.destroy()
                rows=[{"type":"event"}]
                if failure=="extra-io":rows.append({"type":"request"})
                return {"cut":{"backends":[{"records":rows}]},"fault":{"event_index":0}}
            p=NS(Fixture=Fixture,sha=persistence.sha,ready=lambda vm:events.append(("ready",)),joined_fault=joined)
            modules={"persistence":p,"visual_oracle":visual,"data_oracle":load("data_oracle"),"data_provision":provision}
            def read(path,limit,exact):
                self_check.assertEqual((path,limit,exact),("/artifact/boot.img",16777216,16777216))
                return bytes(16777216)
            session=NS(cloud_guard=lambda:events.append(("guard",)),load=modules.__getitem__,VM=VM,read_regular=read,PlannedDataFault=Planned)
            def capture(vm,oracle,stage,command=None,first=False):
                if failure=="capture" and stage=="pending":raise ValueError("fixture frame failure")
                captures.append((len(vm.keys),stage,command,first))
                return {"unit_fixture":stage}
            root=NS(mkdir=lambda **kwargs:self_check.assertEqual(kwargs,dict(mode=0o700,exist_ok=False)))
            self_check=self
            with patch.object(scenario,"Path",return_value=root),patch.object(scenario.os,"urandom",return_value=bytes(range(68))),patch.object(scenario,"capture",side_effect=capture):
                if failure is None:
                    result=scenario.run(session)
                    self.assertEqual(vms[0].keys,expected_keys())
                    self.assertEqual(len(vms),1)
                    self.assertEqual(len(captures),11)
                    self.assertEqual([x[1] for x in captures],["home","terminal"]+["pending","unavailable"]*4+["files"])
                    self.assertEqual([x[3] for x in captures if x[1]=="pending"],[True,False,False,False])
                    self.assertEqual(result["initial_data_base64"],result["frozen_data_base64"])
                    self.assertFalse(result["milestone_complete"])
                else:
                    with self.assertRaises(ValueError):scenario.run(session)
            self.assertEqual(events[0],("guard",))
            self.assertTrue(vms[0].closed);self.assertTrue(all(f.closed for f in fixtures))
            self.assertEqual([f.role for f in fixtures],["data","system"])

retained=load("mounted_error_evidence");fixture=load("fault_evidence_tests")
def retained_document():
    old=fixture.document(2)
    proof=old["vm_proofs"][0]
    keys,plan=retained.input_plan()
    commands=[dict(execute="qmp_capabilities")]+[c for _,c in fixture.profile.preflight_requests()]+[dict(execute="cont")]
    positions={at for at,*_ in plan}
    for at in range(len(keys)+1):
        if at in positions:commands.append(dict(execute="screendump",arguments=dict(filename=fixture.profile.directory(1)+"/frame.ppm")))
        if at<len(keys):commands.append(dict(execute="send-key",arguments={"keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}))
    proof["commands"]=[dict(row,id=i) for i,row in enumerate(commands,1)]
    frames=[]
    for _,stage,command,first in plan:
        pixels=visual.unavailable_expected(stage,command,first)
        frames.append(dict(stage=stage,command=command,first=first,sha256=persistence.sha(pixels),actual_ppm=base64.b64encode(pixels).decode("ascii")))
    return dict(schema="rar-modern-mounted-error-candidate-v1",status="observed",
        original_data_base64=old["initial_data_base64"],initial_data_base64=old["initial_data_base64"],
        frozen_data_base64=old["frozen_data_base64"],data_sha256=old["frozen_data_sha256"],
        system_sha256=old["system_sha256"],boot_sha256=old["boot_sha256"],
        frames=frames,vm_proof=proof,milestone_complete=False)

class RetainedMountedErrorTests(unittest.TestCase):
    def check(self,doc):
        modules={"runtime_evidence":fixture.base,"fault_evidence":fixture.evidence,
            "visual_oracle":visual,"data_oracle":load("data_oracle"),
            "vm_profile":fixture.profile,"persistence":persistence,
            "fault_audit":fixture.audit,"fault_events":fixture.fault_events}
        with patch.object(retained,"helper",side_effect=modules.__getitem__),patch.object(fixture.evidence,"helper",side_effect=modules.__getitem__),patch.object(fixture.profile,"validate_preflight",return_value=dict(rtc_path="/machine/unattached/device[7]")):
            return retained.validate(retained.canonical(doc),"a"*64,(1966080,131072))
    def test_full_inert_proof(self):
        result=self.check(retained_document())
        self.assertEqual(result["post_error_requests"],0)
        self.assertEqual(result["frames"],11)
        self.assertFalse(result["milestone_complete"])
        self.assertFalse(result["provenance_validated"])
    def test_post_error_requests_and_false_lifecycle_refused(self):
        for mutation in ("read","write","flush","same-inode","changed-data","wrong-fault","alive","missing-frame","stale-input"):
            doc=retained_document();proof=doc["vm_proof"]
            if mutation in ("read","write","flush"):
                rows=proof["cut"]["backends"][0]["records"]
                rows.append(dict(type="request",operation=mutation,offset=0,length=0 if mutation=="flush" else 512))
                proof["audit"][0]=fixture.audit.scan(rows,proof["fault"]["plan"],rows[0])
            elif mutation=="same-inode":proof["cut"]["backends"][1]["records"][0]["inode"]=10
            elif mutation=="changed-data":
                raw=bytearray(base64.b64decode(doc["frozen_data_base64"]));raw[-1]^=1
                doc["frozen_data_base64"]=base64.b64encode(raw).decode("ascii");doc["data_sha256"]=persistence.sha(raw)
            elif mutation=="wrong-fault":proof["fault"]["plan"]["ordinal"]=2
            elif mutation=="alive":proof["cut"]["vm_returncode"]=0
            elif mutation=="missing-frame":doc["frames"].pop()
            else:
                rows=proof["commands"];at=next(i for i,r in enumerate(rows) if r["execute"]=="screendump");rows.pop(at)
                for i,r in enumerate(rows,1):r["id"]=i
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):self.check(doc)

if __name__=="__main__":unittest.main(argv=[sys.argv[0]],verbosity=2)
