"""Cloud-only pure retained-fault fixtures; no VM or filesystem mutation."""
import base64
import copy
import importlib.util
import os
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud tests only")

def load(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module
evidence=load("fault_evidence")
base=load("runtime_evidence")
scenario=load("fault_scenarios")
visual=load("visual_oracle")
profile=load("vm_profile")
persistence=load("persistence")
audit=load("fault_audit")
provision=load("data_provision")
replay=load("fault_replay")
fault_events=load("fault_events")

def document():
    # Fixed public synthetic in-memory bytes, not a writable replay fixture.
    data=provision.Provisioner().fresh(bytes(range(68)))
    value="a"*32;plan,_=scenario.selected(0)
    _,requests,write_hashes=replay.expected(data,value,plan)
    frames=[]
    for scene in (0,1,0):
        pixels=visual.expected(scene,None)
        frames.append(dict(scene=visual.SCENES[scene],sha256=base.sha(pixels),
            actual_ppm=base64.b64encode(pixels).decode("ascii")))
    pixels=visual.recovered_expected("absent")
    frames.append(dict(scene="recovered-absent",sha256=base.sha(pixels),
        actual_ppm=base64.b64encode(pixels).decode("ascii")))
    proofs=[]
    for number in (1,2):
        commands=[dict(execute="qmp_capabilities")]+[c for _,c in profile.preflight_requests()]+[dict(execute="cont")]
        cont=len(commands)
        capture=dict(execute="screendump",arguments=dict(filename=profile.directory(number)+"/frame.ppm"))
        commands.append(capture)
        for at,key in enumerate(visual.plan(value)[number-1]):
            commands.append(dict(execute="send-key",arguments=dict(
                keys=[dict(type="qcode",data=key)],**{"hold-time":50})))
            if number==1 and at==0:commands.append(capture)
        if number==2:commands.append(capture)
        commands=[dict(row,id=i) for i,row in enumerate(commands,1)]
        reports=[];summaries=[]
        for index,(role,size) in enumerate((("data",99328),("system",8388608),("boot",16777216))):
            ready=dict(type="ready",kind=role,capacity=size,device=1,inode=index+10,
                readonly=role=="boot" or role=="data" and number==2,export_readonly=False)
            rows=[ready]
            if number==1 and index==0:
                counts=dict(read=0,write=0,flush=0);written=iter(write_hashes)
                for position,request in enumerate(requests):
                    op=request["operation"];counts[op]+=1
                    event=dict(operation=op,ordinal=counts[op],offset=request["offset"],
                        length=request["length"],status="completed")
                    if op=="write":event["payload_sha256"]=next(written)
                    if position==len(requests)-1:event.update(status="cut-no-reply",injection=plan)
                    rows.extend([request,dict(type="event",event=event)])
                rows.append(dict(type="terminal",outcome="cut",fault_hit=True,failed=False))
                summary=audit.scan(rows,plan,ready);code=20;problem="cut"
            else:
                if number==2 and index==0:
                    for sector in range(194):
                        rows.extend([dict(type="request",operation="read",offset=sector*512,length=512),
                            dict(type="event",event=dict(operation="read",ordinal=sector+1,
                                offset=sector*512,length=512,status="completed"))])
                rows.append(dict(type="terminal",outcome="failed",fault_hit=False,failed=False))
                summary=persistence.audit(rows,role,ready,number==2);code=21;problem="backend-failed"
            reports.append(dict(returncode=code,problem=problem,records=rows,joined=True))
            summaries.append(summary)
        cut=dict(vm_pid=number+100,vm_returncode=-9,backends=reports,joined=True)
        proof=dict(cut=cut,audit=summaries,argv=profile.argv(number,7,8,9,number==2),
            preflight=dict(raw={},verified=dict(rtc_path="/machine/unattached/device[7]")),
            commands=commands,events=[dict(event="RESUME",timestamp=dict(seconds=1,microseconds=1))],
            event_receipts=[dict(event_index=0,request_id=cont)],qmp_drained=True,
            serial="RAR-MODERN:GUI-READY\n")
        if number==1:
            cut["entry"]=dict(vm_code=None,backend_codes=[20,None,None],
                backend_problems=["cut",None,None],event_count=1)
            hit=summaries[0]
            proof["fault"]={key:hit[key] for key in ("request_index","event_index","offset","length")}
            proof["fault"].update(plan=plan,delivery=dict(code=20,problem="cut",eof=True))
        proofs.append(proof)
    return dict(schema="rar-modern-data-fault-candidate-v0",case=0,plan=plan,reverse_flush=False,
        challenge=value,frames=frames,vm_proofs=proofs,initial_data_sha256=base.sha(data),
        frozen_data_sha256=base.sha(data),initial_data_base64=base64.b64encode(data).decode("ascii"),
        frozen_data_base64=base64.b64encode(data).decode("ascii"),expected_revision=0,
        expected_classification=scenario.expected_state(plan),system_sha256=base.sha(bytes(8388608)),
        boot_sha256="a"*64,status="observed-not-independently-accepted",milestone_complete=False)

class Tests(unittest.TestCase):
    def check(self,doc):
        # Topology parser has its own real fixtures; this isolates retained
        # envelope/commands/audit/lifecycle validation with an inert topology.
        modules={"runtime_evidence":base,"fault_scenarios":scenario,"visual_oracle":visual,
            "data_oracle":load("data_oracle"),"vm_profile":profile,
            "persistence":persistence,"fault_audit":audit,"fault_events":fault_events,"fault_replay":replay}
        with patch.object(evidence,"helper",side_effect=modules.__getitem__), \
             patch.object(profile,"validate_preflight",return_value=dict(rtc_path="/machine/unattached/device[7]")):
            return evidence.validate(evidence.canonical(doc),0,"a"*64,(1966080,131072))

    def test_positive_inert_retained_capture(self):
        result=self.check(document())
        self.assertTrue(result["content_validated"])
        self.assertFalse(result["provenance_validated"])
        self.assertFalse(result["milestone_complete"])

    def test_altered_retained_claims_and_boundaries_rejected(self):
        doc=document()
        cases=[
            (("case",),True),(("reverse_flush",),True),(("milestone_complete",),True),
            (("expected_revision",),True),(("expected_classification","burned_slots"),[0]),
            (("boot_sha256",),"b"*64),(("system_sha256",),"b"*64),
            (("initial_data_sha256",),"b"*64),(("frames",3,"scene"),"recovered-empty"),
            (("frames",3,"sha256"),"b"*64),
            (("vm_proofs",0,"cut","vm_returncode"),0),
            (("vm_proofs",0,"cut","vm_returncode"),-9.0),
            (("vm_proofs",0,"cut","entry","vm_code"),0),
            (("vm_proofs",0,"cut","entry","event_count"),True),
            (("vm_proofs",0,"cut","entry","event_count"),0),
            (("vm_proofs",0,"cut","entry","event_count"),2),
            (("vm_proofs",0,"fault","delivery","eof"),False),
            (("vm_proofs",0,"fault","event_index"),999),
            (("vm_proofs",0,"cut","backends",0,"returncode"),21),
            (("vm_proofs",0,"cut","backends",0,"records",1,"offset"),1536),
            (("vm_proofs",0,"cut","backends",1,"records"),[doc["vm_proofs"][0]["cut"]["backends"][1]["records"][0]]),
            (("vm_proofs",0,"qmp_drained"),False),
            (("vm_proofs",0,"serial"),"RAR-MODERN:GUI-READY RAR-PANIC"),
            (("vm_proofs",0,"events",0,"event"),"RESET"),
            (("vm_proofs",1,"cut","vm_pid"),101),
            (("vm_proofs",1,"cut","backends",0,"records",0,"inode"),99),
            (("vm_proofs",1,"cut","backends",0,"records",0,"readonly"),False),
        ]
        for path,value in cases:
            changed=copy.deepcopy(doc);at=changed
            for part in path[:-1]:at=at[part]
            at[path[-1]]=value
            with self.subTest(path=path):
                with self.assertRaises(ValueError):self.check(changed)
        for number in (0,1):
            changed=copy.deepcopy(doc)
            changed["vm_proofs"][number]["commands"].append(dict(execute="cont",id=999))
            with self.assertRaises(ValueError):self.check(changed)
        changed=copy.deepcopy(doc);changed["extra"]=0
        with self.assertRaises(ValueError):self.check(changed)

    def test_live_fault_clock_receipt_is_not_post_reap(self):
        doc=document();proof=doc["vm_proofs"][0]
        proof["events"].append(dict(event="RTC_CHANGE",timestamp=dict(seconds=1,microseconds=2),
            data={"offset":0,"qom-path":"/machine/unattached/device[7]"}))
        proof["event_receipts"].append(dict(event_index=1,request_id=None))
        proof["cut"]["entry"]["event_count"]=2
        self.assertTrue(self.check(doc)["content_validated"])
        proof["cut"]["entry"]["event_count"]=1
        with self.assertRaises(ValueError):self.check(doc)

    def test_rehashed_tampering_does_not_hide_changed_bytes(self):
        doc=document()
        raw=bytearray(base.decoded(doc["frozen_data_base64"],99328));raw[1536]^=1
        doc["frozen_data_base64"]=base64.b64encode(raw).decode("ascii")
        doc["frozen_data_sha256"]=base.sha(raw)
        with self.assertRaises(ValueError):self.check(doc)
        doc=document();frame=doc["frames"][3]
        raw=bytearray(base64.b64decode(frame["actual_ppm"]));raw[-1]^=1
        frame["actual_ppm"]=base64.b64encode(raw).decode("ascii");frame["sha256"]=base.sha(raw)
        with self.assertRaises(ValueError):self.check(doc)

    def test_independent_replay_all_sixty_cases(self):
        data=provision.Provisioner().fresh(bytes(range(68)))
        oracle=load("data_oracle");value="a"*32
        for case in range(60):
            plan=evidence.selection(case)
            self.assertEqual(plan,scenario.selected(case)[0])
            frozen,requests,hashes=replay.expected(data,value,plan)
            actual=oracle.inspect(frozen);wanted=evidence.classification(plan)
            self.assertEqual(wanted,scenario.expected_state(plan))
            for key,item in wanted.items():self.assertEqual(actual[key],item)
            self.assertEqual(actual["files"],({}, {b"note":b""}, {b"note":value.encode()})[wanted["revision"]])
            self.assertEqual(len(hashes),plan["ordinal"])
            self.assertEqual(requests[:194],[dict(type="request",operation="read",
                offset=sector*512,length=512) for sector in range(194)])
        for case in (True,False,-1,60,None,"0"):
            with self.assertRaises(ValueError):evidence.selection(case)

    def test_exact_mutation_prefix_all_sixty_cases(self):
        for case in range(60):
            plan,_=scenario.selected(case)
            count=2*plan["ordinal"]-(plan["operation"]=="write")
            rows=[]
            for index in range(count):
                write=index%2==0
                rows.append(dict(type="request",operation="write" if write else "flush",
                    offset=(2+index//2)*512 if write else 0,length=512 if write else 0))
            evidence.mutation_geometry(rows,plan)
            for changed in (rows[:-1],rows+rows[-1:]):
                with self.assertRaises(ValueError):evidence.mutation_geometry(changed,plan)
            changed=copy.deepcopy(rows);changed[0]["offset"]=0
            with self.assertRaises(ValueError):evidence.mutation_geometry(changed,plan)

if __name__=="__main__":unittest.main(verbosity=2)
