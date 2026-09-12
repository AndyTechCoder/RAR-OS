"""Inert System fault tests. No files, processes, devices or target execution."""
import base64,copy,importlib.util,os,sys,unittest
from pathlib import Path
from types import SimpleNamespace as NS
from unittest.mock import patch
if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud tests only")
def load(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
base=load("runtime_evidence");oracle=load("signed_runtime_evidence")
validator=load("system_fault_validate");visual=load("visual_oracle")
profile=load("vm_profile");persist=load("persistence");audit=load("fault_audit")
events=load("fault_events");launch=load("system_fault_launch")
campaign=load("system_fault_controller")
def fake(generation,byte):
    data=bytearray(896);data[:8]=b"RARMODL0";data[56:60]=(512).to_bytes(4,"little")
    data[72:80]=generation.to_bytes(8,"little");data[384:]=bytes([byte])*512
    data[288:320]=bytes.fromhex(base.sha(data[:288]));return bytes(data)
FACTORY,CANDIDATE=fake(1,1),fake(2,2)
def rows_for(ready,operations,plan=None):
    rows=[ready];counts={"read":0,"write":0,"flush":0}
    for operation in operations:
        op=operation["operation"];counts[op]+=1
        request={k:operation[k] for k in ("operation","offset","length")}
        event=dict(operation,ordinal=counts[op],status="completed")
        hit=plan is not None and op==plan["operation"] and counts[op]==plan["ordinal"]
        if hit:
            cut=plan["effect"].endswith("cut")
            event.update(injection=plan,status="cut-no-reply" if cut else "failed-no-success")
            if plan["prefix"]:event["persisted_prefix_bytes"]=plan["prefix"]
        rows.extend([dict(type="request",**request),dict(type="event",event=event)])
        if hit:
            if cut:rows.append(dict(type="terminal",outcome="cut",fault_hit=True,failed=False))
            break
    return rows
def document(mode="repair",case=0):
    empty=load("data_provision").Provisioner().fresh(bytes(range(68)));value="a"*32
    data,requests,hashes=load("fault_replay").expected(empty,value,
        dict(operation="flush",ordinal=6,effect="after-cut",prefix=0))
    hashes=iter(hashes);data_operations=[]
    for r in requests:
        op={k:r[k] for k in ("operation","offset","length")}
        if op["operation"]=="write":op["payload_sha256"]=next(hashes)
        data_operations.append(op)
    installed,damaged,_=oracle.repair_images(FACTORY,CANDIDATE)
    before=damaged
    if mode=="install":
        before=bytearray(installed);before[512:1024]=bytes(512)
        before[1024+oracle.SLOT_BYTES:]=bytes(8388608-1024-oracle.SLOT_BYTES);before=bytes(before)
    frozen=oracle.system_fault_image(FACTORY,CANDIDATE,mode,case)
    restarted,updated,choice=oracle.system_fault_restart(FACTORY,CANDIDATE,mode,case)
    plan=oracle.system_fault_cases(FACTORY,CANDIDATE,mode)[case];cut=plan["effect"].endswith("cut")
    frames=[];proofs=[]
    for number in (1,2,3):
        keys,scenes=validator.plan(mode,number,value)
        commands=[dict(execute="qmp_capabilities")]+[c for _,c in profile.preflight_requests()]+[dict(execute="cont")]
        cont=len(commands)
        for at in range(len(keys)+1):
            if at in scenes:
                commands.append(dict(execute="screendump",arguments=dict(filename=profile.directory(number)+"/frame.ppm")))
                name=scenes[at]
                if name.startswith("home-"):ppm=visual.expected(0,None)
                elif name.startswith("terminal-"):ppm=visual.expected(1,None)
                elif name=="saved":ppm=visual.expected(2,value)
                elif name=="files-3":ppm=visual.expected(3,value)
                else:ppm=oracle.settings_expected(visual,name=="installed-1" or updated)
                frames.append(dict(scene=name,sha256=base.sha(ppm),actual_ppm=base64.b64encode(ppm).decode()))
            if at<len(keys):
                commands.append(dict(execute="send-key",arguments={"keys":[dict(type="qcode",data=keys[at])],"hold-time":50}))
        commands=[dict(row,id=i) for i,row in enumerate(commands,1)]
        reports=[];summaries=[]
        for index,(role,size) in enumerate((("data",99328),("system",8388608),("boot",16777216))):
            ready=dict(type="ready",kind=role,capacity=size,device=1,inode=index+10,
                readonly=role=="boot" or role=="data" and number>1,export_readonly=False)
            operations=[]
            if role=="data":
                operations=data_operations if number==1 else [
                    dict(operation="read",offset=i*512,length=512) for i in range(194)]
            if role=="system":
                if number==1 and mode=="repair":operations=oracle.system_fault_operations(FACTORY,CANDIDATE,"install")
                if number==2:operations=oracle.system_fault_operations(FACTORY,CANDIDATE,mode)
                if number==3 and mode=="repair" and frozen!=restarted:
                    operations=(oracle.system_fault_operations(FACTORY,CANDIDATE,"repair") if choice=="repair"
                        else [dict(operation="write",offset=0,length=512,payload_sha256=base.sha(restarted[:512])),
                              dict(operation="flush",offset=0,length=0)])
            fault=number==2 and role=="system"
            rows=rows_for(ready,operations,plan if fault else None)
            summary=(audit.scan(rows,plan,ready,role="system") if fault
                else persist.audit(rows,role,ready,number>1,mode=="repair" and role=="system"))
            reports.append(dict(returncode=20 if fault and cut else -9,
                problem="cut" if fault and cut else "backend-failed",records=rows,joined=True))
            summaries.append(summary)
        serial="RAR-MODERN:GUI-READY\n"
        if number==1 and mode=="repair":
            serial+="RAR-MODERN:UPDATE-REQUEST\nRAR-MODERN:STALE-AUTHORITY-REVOKED\nRAR-MODERN:UPDATE-INSTALLED\nRAR-MODERN:PRIVATE-MEMORY-RETIRED\n"
        if number==2:serial=serial+"RAR-MODERN:UPDATE-REQUEST\n" if mode=="install" else ""
        stopped=dict(vm_pid=100+number,vm_returncode=-9,backends=reports,joined=True)
        proof=dict(cut=stopped,audit=summaries,argv=profile.argv(number,7,8,9,number>1),
            preflight=dict(raw={},verified=dict(rtc_path="/machine/unattached/device[7]")),
            commands=commands,events=[dict(event="RESUME",timestamp=dict(seconds=1,microseconds=1))],
            event_receipts=[dict(event_index=0,request_id=cont)],qmp_drained=True,serial=serial)
        if number==2:
            stopped["entry"]=dict(vm_code=None,backend_codes=[None,20 if cut else None,None],
                backend_problems=[None,"cut" if cut else None,None],event_count=1)
            proof["fault"]={key:summaries[1][key] for key in ("plan","request_index","event_index","offset","length")}
            proof["fault"]["delivery"]=dict(code=20 if cut else None,problem="cut" if cut else None,eof=cut)
        proofs.append(proof)
    return dict(schema="rar-system-fault-candidate-v0",mode=mode,case=case,plan=plan,challenge=value,
        frames=frames,vm_proofs=proofs,boot_sha256="a"*64,initial_data_sha256=base.sha(empty),
        frozen_data_sha256=base.sha(data),frozen_data_base64=base64.b64encode(data).decode(),
        before_system_base64=base64.b64encode(before).decode(),
        frozen_system_base64=base64.b64encode(frozen).decode(),
        restarted_system_base64=base64.b64encode(restarted).decode(),
        system_corruption=(dict(offsets=[1024,1024+oracle.SLOT_BYTES],xor=1,
            before_sha256=base.sha(installed),after_sha256=base.sha(damaged)) if mode=="repair" else None),
        choice=choice,status="observed",milestone_complete=False)
class Tests(unittest.TestCase):
    def check(self,doc):
        original=validator.helper
        with patch.object(validator,"helper",side_effect=lambda name:profile if name=="vm_profile" else original(name)), \
             patch.object(profile,"validate_preflight",return_value=dict(rtc_path="/machine/unattached/device[7]")):
            return validator.validate(base.canonical(doc),"a"*64,(1966080,131072),
                doc["mode"],doc["case"],FACTORY,CANDIDATE)
    def test_all_boundary_oracles_and_independent_operations(self):
        self.assertTrue(oracle.self_test())
        for mode in ("install","repair"):
            ops=oracle.system_fault_operations(FACTORY,CANDIDATE,mode)
            self.assertEqual([o["operation"] for o in ops],["write","write","flush","write","flush"])
            for plan in oracle.system_fault_cases(FACTORY,CANDIDATE,mode):
                ready=dict(type="ready",kind="system",capacity=8388608,device=1,inode=11,readonly=False,export_readonly=False)
                rows=rows_for(ready,ops,plan)
                self.assertIsNotNone(audit.scan(rows,plan,ready,role="system"))
                validator.mutation_sequence(rows,FACTORY,CANDIDATE,mode,plan)
                bad=copy.deepcopy(rows);bad[2]["event"]["payload_sha256"]="f"*64
                with self.assertRaises(ValueError):validator.mutation_sequence(bad,FACTORY,CANDIDATE,mode,plan)
                bad=rows+[dict(type="request",operation="flush",offset=0,length=0)]
                with self.assertRaises(ValueError):validator.mutation_sequence(bad,FACTORY,CANDIDATE,mode,plan)
    def test_complete_retained_envelopes(self):
        for mode in ("install","repair"):
            for case in (0,1,2,3,4,10,18,24):
                with self.subTest(mode=mode,case=case):
                    result=self.check(document(mode,case))
                    self.assertTrue(result["content_validated"])
                    self.assertFalse(result["provenance_validated"])
                    self.assertFalse(result["milestone_complete"])
                    self.assertEqual(result["fresh_vms"],3)
    def test_retained_claim_mutations_fail(self):
        doc=document()
        changes=[
            (("milestone_complete",),True),(("case",),True),(("choice",),"installed"),
            (("boot_sha256",),"b"*64),(("frozen_data_sha256",),"b"*64),
            (("vm_proofs",1,"cut","vm_returncode"),0),
            (("vm_proofs",1,"cut","entry","vm_code"),0),
            (("vm_proofs",1,"cut","entry","event_count"),True),
            (("vm_proofs",1,"cut","entry","backend_codes",0),21),
            (("vm_proofs",1,"fault","delivery","eof"),1),
            (("vm_proofs",1,"fault","request_index"),999),
            (("vm_proofs",1,"cut","backends",0,"records",0,"readonly"),False),
            (("vm_proofs",1,"qmp_drained"),False),
            (("vm_proofs",1,"events",0,"event"),"RESET"),
            (("vm_proofs",1,"serial"),"RAR-PANIC:OTHER\n"),
            (("vm_proofs",2,"cut","vm_pid"),101),
            (("frames",0,"sha256"),"b"*64)]
        for path,value in changes:
            changed=copy.deepcopy(doc);at=changed
            for part in path[:-1]:at=at[part]
            at[path[-1]]=value
            with self.subTest(path=path):
                with self.assertRaises(ValueError):self.check(changed)
        changed=copy.deepcopy(doc)
        for proof in changed["vm_proofs"]:
            proof["cut"]["backends"][1]["records"][0]["inode"]=10
        with self.assertRaises(ValueError):self.check(changed)
    def test_system_events_cannot_accept_data_or_unrelated_events(self):
        resume=dict(event="RESUME",timestamp=dict(seconds=1,microseconds=1))
        io=dict(event="BLOCK_IO_ERROR",timestamp=dict(seconds=1,microseconds=2),
            data={"device":"","node-name":"rar-system","operation":"write","action":"report","reason":"Input/output error"})
        plan=dict(operation="write",ordinal=1,effect="error",prefix=0)
        for mode in ("install","repair"):
            commands=[dict(execute="qmp_capabilities",id=1),dict(execute="cont",id=2)]
            if mode=="install":commands.append(dict(execute="send-key",id=3,
                arguments={"keys":[dict(type="qcode",data="ret")],"hold-time":50}))
            receipts=[dict(event_index=0,request_id=2),dict(event_index=1,request_id=None)]
            def check(rows,**kw):
                return events.validate(rows,receipts,commands,True,"/machine/unattached/device[7]",1,plan,
                    fault_role="system-"+mode,**kw)
            self.assertEqual(check([resume,io]),0)
            for key,value in (("node-name","rar-data"),("node-name","rar-boot"),("operation","read"),
                              ("action","stop"),("reason","bad\nreason")):
                bad=copy.deepcopy(io);bad["data"][key]=value
                with self.assertRaises(ValueError):check([resume,bad])
            for name in ("RESET","STOP","SHUTDOWN","GUEST_PANICKED"):
                with self.assertRaises(ValueError):check([resume,dict(io,event=name)])
            with self.assertRaises(ValueError):
                events.validate([resume,io],receipts,commands,True,"/machine/unattached/device[7]",1,plan)
    def test_boot_fault_join_and_cleanup_order(self):
        scenario=load("system_fault_scenario")
        for mode,failure in (("repair",None),("install",None),("repair","unexpected"),("install","disk")):
            trace=[];vms=[];fixtures=[];case=0
            installed,damaged,_=oracle.repair_images(FACTORY,CANDIDATE)
            pristine=bytearray(installed);pristine[512:1024]=bytes(512)
            pristine[1024+oracle.SLOT_BYTES:]=bytes(8388608-1024-oracle.SLOT_BYTES);pristine=bytes(pristine)
            frozen=oracle.system_fault_image(FACTORY,CANDIDATE,mode,case)
            restarted=oracle.system_fault_restart(FACTORY,CANDIDATE,mode,case)[0]
            empty=bytes(99328);data=empty[:1024]+b"x"+empty[1025:];value="a"*32
            class Signal(RuntimeError):pass
            class VM:
                def __init__(self,index,*args,**kwargs):
                    self.index=index;self.closed=False;vms.append(self);trace.append(("vm",index,args,kwargs))
                def start(self):
                    trace.append(("start",self.index))
                    if self.index==2 and mode=="repair":
                        if failure=="unexpected":raise RuntimeError("unexpected")
                        raise Signal()
                def key(self,key):
                    trace.append(("key",self.index,key))
                    if self.index==2 and mode=="install" and key=="ret":raise Signal()
                def service(self):raise AssertionError("fault was not delivered")
                def destroy(self):self.closed=True;trace.append(("join",self.index))
            class Fixture:
                def __init__(self,root,role,raw):
                    self.role=role;self.fd=10 if role=="data" else 11;self.observer=20;self.damaged=False
                    fixtures.append(self)
                def freeze(self,owned):
                    if not all(vm.closed for vm in owned):raise AssertionError("freeze before join")
                    trace.append(("freeze",self.role,len(owned)))
                    if self.role=="data":return data
                    if len(owned)==3:return restarted
                    if len(owned)==2:return bytes(8388608) if failure=="disk" else frozen
                    return (damaged if self.damaged else installed) if mode=="repair" else pristine
                def damage_system_headers(self,owned,before,after):
                    self.asserted=all(vm.closed for vm in owned)
                    if not self.asserted:raise AssertionError("mutation before join")
                    self.damaged=True;trace.append(("damage",))
                    return {}
                def close(self):trace.append(("close",self.role))
            def joined(vm,**kw):vm.destroy();return {}
            def fault_join(vm,error,unused):
                self.assertIs(type(error),Signal);vm.destroy()
                return {"fault":{"plan":oracle.system_fault_cases(FACTORY,CANDIDATE,mode)[case]}}
            def inspect(raw):
                return (dict(revision=0) if raw==empty else
                    dict(files={b"note":value.encode()},revision=2,committed_slots=[0,1],burned_slots=[]))
            modules={"persistence":NS(Fixture=Fixture,sha=base.sha,ready=lambda vm:None,
                challenge=lambda entropy:value,joined=joined),
                "visual_oracle":NS(plan=lambda v:(["f3","ret"],[])),
                "signed_runtime_evidence":oracle,"data_oracle":NS(inspect=inspect),
                "data_provision":NS(Provisioner=lambda:NS(fresh=lambda entropy:empty)),
                "signed_runtime_scenario":NS(capture=lambda vm,name,check:dict(scene=name),
                    marker=lambda vm,name:None)}
            def read(path,*args,**kw):
                return {"modern-settings-factory.layer":FACTORY,"modern-settings-update.layer":CANDIDATE,
                    "modern-system.img":pristine,"boot.img":b"boot"}[path.rsplit("/",1)[-1]]
            session=NS(cloud_guard=lambda:None,load=modules.__getitem__,read_regular=read,
                VM=VM,PlannedSystemFault=Signal)
            with patch.object(scenario.Path,"mkdir",return_value=None), \
                 patch.object(scenario.os,"urandom",side_effect=lambda n:bytes(n)), \
                 patch.object(scenario,"joined_fault",side_effect=fault_join):
                if failure:
                    with self.assertRaises((RuntimeError,ValueError)):scenario.run(session,mode,case)
                else:self.assertFalse(scenario.run(session,mode,case)["milestone_complete"])
            self.assertTrue(all(vm.closed for vm in vms))
            self.assertEqual([row for row in trace if row[0]=="close"],[("close","system"),("close","data")])
            self.assertEqual(len(vms),2 if failure else 3)
            second=next(row for row in trace if row[:2]==("vm",2))
            self.assertEqual(second[2],(20,11));self.assertIs(second[3]["readonly_data"],True)
            if mode=="repair":
                self.assertLess(trace.index(("join",1)),trace.index(("damage",)))
                self.assertFalse(any(row[:2]==("key",2) for row in trace))
            if not failure:
                self.assertLess(trace.index(("join",2)),trace.index(("freeze","system",2)))

    def test_literal_launch_selection(self):
        self.assertEqual(launch.selection(["repair","255"]),("repair",255))
        for args in ([],["repair"],["install","256"],["repair","01"],["repair","-1"],
                     ["other","1"],["repair","1","--device"],["repair","١"]):
            with self.assertRaises(ValueError):launch.selection(args)
    def test_campaign_stops_once_and_retains_failures(self):
        plans=oracle.system_fault_cases(FACTORY,CANDIDATE,"install")[:2]
        for failure in (None,"validation","repeat","execute"):
            calls=[];saved=[];report={};results=iter([
                dict(image_id="1",key_sha256="k1",challenge="v1"),
                dict(image_id="1" if failure=="repeat" else "2",key_sha256="k2",challenge="v2")])
            def execute(*args):
                calls.append(args)
                if failure=="execute":raise RuntimeError("failed")
                return b"actual\n"
            def validate(*args):
                if failure=="validation":raise ValueError("bad evidence")
                return next(results)
            modules={"signed_runtime_evidence":NS(system_fault_cases=lambda *a:plans),
                "signed_runtime_controller":NS(public_file=lambda path,raw:saved.append((path,raw))),
                "system_fault_validate":NS(validate=validate)}
            with patch.object(campaign.time,"monotonic",return_value=0):
                def run():campaign.observe(modules.__getitem__,execute,"launcher",Path("/in"),"boot",(1,2),
                    Path("/evidence"),report,lambda:None,
                    {"modern-settings-factory.layer":FACTORY,"modern-settings-update.layer":CANDIDATE},"install")
                if failure:
                    with self.assertRaises((ValueError,RuntimeError)):run()
                else:run()
            self.assertEqual(len(calls),1 if failure in ("validation","execute") else 2)
            self.assertEqual(len(saved),0 if failure=="execute" else len(calls))
            if not failure:self.assertEqual(report["system_faults"]["completed"],2)
            for _,raw in saved:self.assertEqual(__import__("gzip").decompress(raw),b"actual\n")
if __name__=="__main__":unittest.main(verbosity=2)
