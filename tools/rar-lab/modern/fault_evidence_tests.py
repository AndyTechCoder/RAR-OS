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

def document(case=0,data_exit=None,peer_exit=21):
    # Fixed public synthetic in-memory bytes, not a writable replay fixture.
    data=provision.Provisioner().fresh(bytes(range(68)))
    value="a"*32;plan,_=scenario.selected(case)
    frozen,requests,write_hashes=replay.expected(data,value,plan)
    state=evidence.classification(plan);revision=state["revision"]
    iscut=plan["effect"] in ("before-cut","after-cut","torn-cut")
    if data_exit is None:data_exit=20 if iscut else -9
    frames=[]
    for scene in (0,1,0):
        pixels=visual.expected(scene,None)
        frames.append(dict(scene=visual.SCENES[scene],sha256=base.sha(pixels),
            actual_ppm=base64.b64encode(pixels).decode("ascii")))
    recovered=("absent","empty","written")[revision]
    pixels=visual.recovered_expected(recovered,value if revision==2 else None)
    frames.append(dict(scene="recovered-"+recovered,sha256=base.sha(pixels),
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
                    if position==len(requests)-1:
                        event.update(status="cut-no-reply" if iscut else "failed-no-success",injection=plan)
                        if plan["effect"] in ("short-error","torn-cut"):event["persisted_prefix_bytes"]=255
                    rows.extend([request,dict(type="event",event=event)])
                if iscut or data_exit==21:
                    rows.append(dict(type="terminal",outcome="cut" if iscut else "failed",fault_hit=True,failed=not iscut))
                summary=audit.scan(rows,plan,ready);code=data_exit;problem="cut" if iscut else "backend-failed"
            else:
                if number==2 and index==0:
                    for sector in range(194):
                        rows.extend([dict(type="request",operation="read",offset=sector*512,length=512),
                            dict(type="event",event=dict(operation="read",ordinal=sector+1,
                                offset=sector*512,length=512,status="completed"))])
                if peer_exit==21:rows.append(dict(type="terminal",outcome="failed",fault_hit=False,failed=False))
                summary=persistence.audit(rows,role,ready,number==2);code=peer_exit;problem="backend-failed"
            reports.append(dict(returncode=code,problem=problem,records=rows,joined=True))
            summaries.append(summary)
        cut=dict(vm_pid=number+100,vm_returncode=-9,backends=reports,joined=True)
        proof=dict(cut=cut,audit=summaries,argv=profile.argv(number,7,8,9,number==2),
            preflight=dict(raw={},verified=dict(rtc_path="/machine/unattached/device[7]")),
            commands=commands,events=[dict(event="RESUME",timestamp=dict(seconds=1,microseconds=1))],
            event_receipts=[dict(event_index=0,request_id=cont)],qmp_drained=True,
            serial="RAR-MODERN:GUI-READY\n")
        if number==1:
            cut["entry"]=dict(vm_code=None,backend_codes=[20 if iscut else None,None,None],
                backend_problems=["cut" if iscut else None,None,None],event_count=1)
            hit=summaries[0]
            proof["fault"]={key:hit[key] for key in ("request_index","event_index","offset","length")}
            proof["fault"].update(plan=plan,delivery=dict(code=20 if iscut else None,
                problem="cut" if iscut else None,eof=iscut))
        proofs.append(proof)
    return dict(schema="rar-modern-data-fault-candidate-v0",case=case,plan=plan,reverse_flush=False,
        challenge=value,frames=frames,vm_proofs=proofs,initial_data_sha256=base.sha(data),
        frozen_data_sha256=base.sha(frozen),initial_data_base64=base64.b64encode(data).decode("ascii"),
        frozen_data_base64=base64.b64encode(frozen).decode("ascii"),expected_revision=revision,
        expected_classification=state,system_sha256=base.sha(bytes(8388608)),
        boot_sha256="a"*64,status="observed-not-independently-accepted",milestone_complete=False)

class Tests(unittest.TestCase):
    def check(self,doc):
        return self.check_raw(evidence.canonical(doc),doc["case"])

    def check_raw(self,raw,case=0):
        # Topology parser has its own real fixtures; this isolates retained
        # envelope/commands/audit/lifecycle validation with an inert topology.
        modules={"runtime_evidence":base,"fault_scenarios":scenario,"visual_oracle":visual,
            "data_oracle":load("data_oracle"),"vm_profile":profile,
            "persistence":persistence,"fault_audit":audit,"fault_events":fault_events,"fault_replay":replay}
        with patch.object(evidence,"helper",side_effect=modules.__getitem__), \
             patch.object(profile,"validate_preflight",return_value=dict(rtc_path="/machine/unattached/device[7]")):
            return evidence.validate(raw,case,"a"*64,(1966080,131072))

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

    def test_representative_complete_envelopes_and_termination_mutations(self):
        for case,data_exit,peer_exit in ((0,20,-9),(2,-9,21),(4,21,-9),
                (17,-9,-9),(24,21,21),(28,20,21),(41,20,-9),(56,20,21)):
            doc=document(case,data_exit,peer_exit)
            self.assertTrue(self.check(doc)["content_validated"])
            for path,value in (
                (("vm_proofs",0,"fault","delivery","code"),99),
                (("vm_proofs",0,"cut","entry","backend_codes",0),99),
                (("vm_proofs",0,"cut","backends",0,"returncode"),99),
                (("vm_proofs",0,"cut","backends",0,"problem"),"unknown"),
                (("vm_proofs",0,"cut","backends",1,"returncode"),0)):
                changed=copy.deepcopy(doc);at=changed
                for key in path[:-1]:at=at[key]
                at[path[-1]]=value
                with self.subTest(case=case,path=path):
                    with self.assertRaises(ValueError):self.check(changed)
            changed=copy.deepcopy(doc);report=changed["vm_proofs"][0]["cut"]["backends"][0]
            event=next(row["event"] for row in report["records"]
                if row.get("type")=="event" and "injection" in row["event"])
            event["status"]="completed"
            with self.assertRaises(ValueError):self.check(changed)
            if report["records"][-1].get("type")=="terminal":
                changed=copy.deepcopy(doc)
                changed["vm_proofs"][0]["cut"]["backends"][0]["records"].pop()
                with self.assertRaises(ValueError):self.check(changed)

    def test_raw_parser_boundary(self):
        raw=evidence.canonical(document())
        values=(b"",raw[:-1],raw+b"x",b'{"a":1,"a":2}\n',
            b'{"a":{"b":1,"b":2}}\n',b'{"a":NaN}\n',b'{"a":Infinity}\n',
            b'[]\n',b'null\n',b'"x"\n',b'{}\n',b' {"a":1}\n',
            b'{"b":1,"a":2}\n',b'\xff\n',b'{"x":"\xc3\xa9"}\n',
            b"x"*(64*1024*1024+1))
        for malformed in values:
            with self.assertRaises(ValueError):self.check_raw(malformed)
        spaced=raw.replace(b'":',b'": ',1)
        with self.assertRaises(ValueError):self.check_raw(spaced)

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
