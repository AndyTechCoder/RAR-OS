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
evidence=load("runtime_evidence");visual=load("visual_oracle");profile=load("vm_profile")
persistence=load("persistence");provision=load("data_provision");scenario=load("unavailable_scenario")
def document():
    original=provision.Provisioner().fresh(bytes(range(68)))
    data=bytearray(original);data[0]^=1;data[512]^=1;data=bytes(data)
    keys,plan=evidence.unavailable_plan()
    frames=[]
    for _,stage,command,first in plan:
        pixels=visual.unavailable_expected(stage,command,first)
        frames.append(dict(stage=stage,command=command,first=first,sha256=evidence.sha(pixels),
            actual_ppm=base64.b64encode(pixels).decode("ascii")))
    commands=[dict(execute="qmp_capabilities")]+[c for _,c in profile.preflight_requests()]+[dict(execute="cont")]
    cont=len(commands);positions={p for p,*_ in plan}
    for at in range(len(keys)+1):
        if at in positions:commands.append(dict(execute="screendump",arguments=dict(filename=profile.directory(1)+"/frame.ppm")))
        if at<len(keys):commands.append(dict(execute="send-key",arguments=dict(keys=[dict(type="qcode",data=keys[at])],**{"hold-time":50})))
    commands=[dict(row,id=i) for i,row in enumerate(commands,1)]
    reports=[];summaries=[]
    for index,(role,size) in enumerate((("data",99328),("system",8388608),("boot",16777216))):
        ready=dict(type="ready",kind=role,capacity=size,device=1,inode=index+10,
            readonly=role=="boot",export_readonly=False);rows=[ready]
        if role=="data":
            for number,sector in enumerate((0,0,1),1):
                rows.extend([dict(type="request",operation="read",offset=sector*512,length=512),
                    dict(type="event",event=dict(operation="read",ordinal=number,offset=sector*512,length=512,status="completed"))])
        rows.append(dict(type="terminal",outcome="failed",fault_hit=False,failed=False))
        reports.append(dict(returncode=21,problem="backend-failed",records=rows,joined=True))
        summaries.append(persistence.audit(rows,role,ready,False))
    proof=dict(cut=dict(vm_pid=101,vm_returncode=-9,backends=reports,joined=True),
        audit=summaries,argv=profile.argv(1,7,8,9,False),
        preflight=dict(raw={},verified=dict(rtc_path="/machine/unattached/device[7]")),
        commands=commands,events=[dict(event="RESUME",timestamp=dict(seconds=1,microseconds=1))],
        event_receipts=[dict(event_index=0,request_id=cont)],qmp_drained=True,serial="RAR-MODERN:GUI-READY\n")
    b64=lambda raw:base64.b64encode(raw).decode("ascii")
    return dict(schema="rar-modern-unavailable-candidate-v1",status="observed",
        original_data_base64=b64(original),initial_data_base64=b64(data),frozen_data_base64=b64(data),
        data_sha256=evidence.sha(data),system_sha256=evidence.sha(bytes(8388608)),boot_sha256="a"*64,
        frames=frames,vm_proof=proof,milestone_complete=False)

class Tests(unittest.TestCase):
    def check(self,doc):
        modules={"visual_oracle":visual,"data_oracle":load("data_oracle"),"vm_profile":profile,"persistence":persistence}
        with patch.object(evidence,"helper",side_effect=modules.__getitem__),patch.object(profile,"validate_preflight",
            return_value=dict(rtc_path="/machine/unattached/device[7]")) as topology:
            result=evidence.validate_unavailable(evidence.canonical(doc),"a"*64,(1966080,131072))
            topology.assert_called_once_with({},False,1,(1966080,131072))
            return result
    def test_complete_inert_negative_envelope(self):
        result=self.check(document())
        self.assertEqual(result["frames"],9);self.assertEqual(result["header_reads"],3)
        self.assertEqual(result["mutation_attempts"],0);self.assertFalse(result["milestone_complete"])
    def test_changed_identity_state_and_pixels_fail(self):
        doc=document()
        mutations=[(("milestone_complete",),True),(("boot_sha256",),"b"*64),(("system_sha256",),"b"*64),
            (("vm_proof","cut","joined"),False),(("vm_proof","cut","vm_returncode"),0),
            (("vm_proof","qmp_drained"),False),(("vm_proof","serial"),"RAR-MODERN:GUI-READY\nRAR-PANIC"),
            (("vm_proof","cut","backends",0,"records",0,"capacity"),99329),
            (("vm_proof","cut","backends",0,"records",0,"readonly"),True),
            (("vm_proof","cut","backends",1,"records",0,"inode"),10),
            (("frames",2,"first"),1),(("frames",2,"command"),"write note denied")]
        for path,value in mutations:
            bad=copy.deepcopy(doc);at=bad
            for part in path[:-1]:at=at[part]
            at[path[-1]]=value
            with self.subTest(path=path),self.assertRaises((ValueError,TypeError)):self.check(bad)
        for index in (2,3,8):
            bad=copy.deepcopy(doc);frame=bad["frames"][index]
            pixels=bytearray(base64.b64decode(frame["actual_ppm"]));pixels[-1]^=1
            frame["actual_ppm"]=base64.b64encode(pixels).decode();frame["sha256"]=evidence.sha(pixels)
            with self.assertRaises(ValueError):self.check(bad)
        for field in ("initial_data_base64","frozen_data_base64"):
            bad=copy.deepcopy(doc);data=bytearray(base64.b64decode(bad[field]));data[1024]^=1
            bad[field]=base64.b64encode(data).decode();bad["data_sha256"]=evidence.sha(data)
            with self.assertRaises(ValueError):self.check(bad)
    def test_no_mutation_or_repeated_mount_with_valid_backend_framing(self):
        for op in ("write","flush","read"):
            doc=document();rows=doc["vm_proof"]["cut"]["backends"][0]["records"]
            request=dict(type="request",operation=op,offset=0,length=0 if op=="flush" else 512)
            event=dict(operation=op,ordinal=4 if op=="read" else 1,offset=0,length=request["length"],status="completed")
            if op=="write":event["payload_sha256"]="b"*64
            rows[-1:-1]=[request,dict(type="event",event=event)]
            # A trailing WRITE is rejected by the base audit; FLUSH and extra
            # reads are rejected by the negative-specific request/count rule.
            with self.assertRaises(ValueError):self.check(doc)
    def test_capture_and_input_causality(self):
        doc=document()
        for operation in ("missing","extra","wrong-key"):
            bad=copy.deepcopy(doc);rows=bad["vm_proof"]["commands"]
            at=next(i for i,row in enumerate(rows) if row["execute"]=="screendump")
            if operation=="missing":rows.pop(at)
            elif operation=="extra":rows.insert(at,dict(execute="stop"))
            else:
                at=next(i for i,row in enumerate(rows) if row["execute"]=="send-key")
                rows[at]["arguments"]["keys"][0]["data"]="f2"
            for i,row in enumerate(rows,1):row["id"]=i
            with self.assertRaises(ValueError):self.check(bad)
    def test_visual_pending_and_completed_states_are_distinct(self):
        for first in (False,True):
            for command in scenario.COMMANDS:
                pending=visual.unavailable_expected("pending",command,first)
                self.assertNotEqual(pending,visual.unavailable_expected("unavailable"))
                self.assertEqual(len(visual.unavailable_validate(pending,"pending",command,first)),64)
        for args in (("pending","erase",False),("home","list",False),("files",None,True)):
            with self.assertRaises(ValueError):visual.unavailable_expected(*args)

    def test_scenario_fixed_input_order_and_owned_cleanup(self):
        for failure in (None,"capture","changed-data"):
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
                def __init__(self,index,data,system):
                    self_check.assertEqual((index,data,system),(1,"data","system"))
                    self.keys=[];self.closed=False;vms.append(self)
                def start(self):events.append(("start",))
                def key(self,key):self.keys.append(key)
                def destroy(self):self.closed=True;events.append(("destroy",))
            def joined(vm):vm.destroy();return {"unit_fixture":True}
            p=NS(Fixture=Fixture,sha=persistence.sha,ready=lambda vm:events.append(("ready",)),joined=joined)
            modules={"persistence":p,"visual_oracle":visual,"data_oracle":load("data_oracle"),"data_provision":provision}
            def read(path,limit,exact):
                self_check.assertEqual((path,limit,exact),("/artifact/boot.img",16777216,16777216))
                return bytes(16777216)
            session=NS(cloud_guard=lambda:events.append(("guard",)),load=modules.__getitem__,VM=VM,read_regular=read)
            def capture(vm,oracle,stage,command=None,first=False):
                if failure=="capture" and stage=="pending":raise ValueError("fixture frame failure")
                captures.append((len(vm.keys),stage,command,first))
                return {"unit_fixture":stage}
            root=NS(mkdir=lambda **kwargs:self_check.assertEqual(kwargs,dict(mode=0o700,exist_ok=False)))
            self_check=self
            with patch.object(scenario,"Path",return_value=root),patch.object(scenario.os,"urandom",return_value=bytes(range(68))),patch.object(scenario,"capture",side_effect=capture):
                if failure is None:
                    result=scenario.run(session)
                    self.assertEqual(vms[0].keys,evidence.unavailable_plan()[0])
                    self.assertEqual(captures,evidence.unavailable_plan()[1])
                    self.assertEqual(result["initial_data_base64"],result["frozen_data_base64"])
                    self.assertFalse(result["milestone_complete"])
                else:
                    with self.assertRaises(ValueError):scenario.run(session)
            self.assertEqual(events[0],("guard",))
            self.assertTrue(vms[0].closed);self.assertTrue(all(f.closed for f in fixtures))
            self.assertEqual([f.role for f in fixtures],["data","system"])

if __name__=="__main__":unittest.main(argv=[sys.argv[0]],verbosity=2)
