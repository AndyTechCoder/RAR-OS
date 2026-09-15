"""Pure integrated plan/oracle checks, never guest execution."""
import ast,importlib.util
from pathlib import Path
def load(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def self_test():
    p=load("alpha_journey_plan");e=load("alpha_journey_evidence")
    for name in ("alpha_journey_controller","alpha_journey_scenario","alpha_journey_evidence"):
        ast.parse(Path(__file__).with_name(name+".py").read_text())
    challenges=[c*32 for c in "abcd"];refused=0
    from types import SimpleNamespace
    scenario=load("alpha_journey_scenario");attempted=[]
    def broken_pair():
        attempted.append("pair");raise RuntimeError("original teardown failure")
    def broken_fixture():
        attempted.append("fixture");raise OSError("fixture failure")
    pair=SimpleNamespace(closed=False,destroy=broken_pair,vms=[SimpleNamespace(serial=b"RAR-PANIC:original")])
    failures=scenario.cleanup(pair,[SimpleNamespace(close=lambda:attempted.append("last")),SimpleNamespace(close=broken_fixture)])
    assert attempted==["pair","fixture","last"] and len(failures)==2
    assert "original teardown failure" in failures[0]
    result=scenario.failure("fallback",{"phase":"event"},[1,2],pair,"initial failure")
    result["cleanup_errors"]=failures
    assert result["status"]=="failed" and result["milestone_complete"] is False
    assert result["completed_frames"]==2 and result["serial"]==["RAR-PANIC:original"]
    try:e.validate(load("runtime_evidence").canonical(result),None,None,None,None,"fallback",None,None,None)
    except ValueError:refused+=1
    else:raise AssertionError("cleanup failure accepted")
    assert scenario.cleanup(None,[])==[]
    for mode in p.MODES:
        steps=p.plan(mode,challenges)
        assert sum(1 for s in steps if s[2]=="notes-ready")==4
        for peer,keys,stage,value,compact in steps:
            if p.marker(stage):continue
            pixels=p.expected(stage,value,compact);p.validate(pixels,stage,value,compact)
            try:p.validate(pixels[:-1],stage,value,compact)
            except ValueError:refused+=1
            else:raise AssertionError("truncated pixels accepted")
        for index,peer in enumerate(("a","b"),1):
            profile=load("expansion_profile").Profile(load("vm_profile"),peer,37)
            rows=[{"execute":"qmp_capabilities"}]+[v for _,v in profile.preflight_requests()]+[{"execute":"cont"}]
            for owner,keys,stage,_,_compact in steps:
                if owner!=peer:continue
                rows.extend({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":key}],"hold-time":50}} for key in keys)
                if not p.marker(stage):rows.append({"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}})
            rows=[dict(r,id=i) for i,r in enumerate(rows,1)]
            p.commands(rows,index,peer,mode,challenges,profile)
            for bad in ([],rows[:-1],rows+[dict(execute="cont",id=len(rows)+1)]):
                try:p.commands(bad,index,peer,mode,challenges,profile)
                except ValueError:refused+=1
                else:raise AssertionError("invalid exact journey commands")
    from hashlib import sha256
    def fake(g):
        b=bytearray(896);b[:8]=b"RARMODL0";b[56:60]=(512).to_bytes(4,"little");b[72:80]=g.to_bytes(8,"little")
        b[288:320]=sha256(b[:288]).digest();return bytes(b)
    factory,candidate=fake(1),fake(2);bad=bytearray(candidate);bad[320]^=1
    old,new=e.systems(factory,candidate,bytes(bad),"install")
    assert old[0]==old[1] and new[1]==old[1] and new[0]!=old[0]
    repair_input,repair_output=e.systems(factory,candidate,bytes(bad),"repair")
    assert [i for i,(a,b) in enumerate(zip(new[0],repair_input[0])) if a!=b]==[1024,1024+load("signed_runtime_evidence").SLOT_BYTES]
    final_input,final_output=e.systems(factory,candidate,bytes(bad),"final")
    assert repair_output==final_input==final_output

    import copy
    for mode in p.MODES:
        operations=e.operations(factory,candidate,bytes(bad),mode)
        records=[]
        for op in operations:
            records.append(dict(type="request",**{k:v for k,v in op.items() if k!="payload_sha256"}))
            records.append(dict(type="event",event=dict(op,status="completed",ordinal=1)))
        e.check_operations(records,factory,candidate,bytes(bad),mode)
        variants=[records+[dict(type="request",operation="write",offset=8192,length=512),
                            dict(type="event",event=dict(operation="write",offset=8192,length=512,payload_sha256="0"*64,status="completed",ordinal=99))]]
        if records:
            variants.extend([records[:-2],records[2:]+records[:2]])
            for key,replacement in (("offset",4096),("payload_sha256","0"*64)):
                changed=copy.deepcopy(records);changed[1]["event"][key]=replacement;variants.append(changed)
        for changed in variants:
            try:e.check_operations(changed,factory,candidate,bytes(bad),mode)
            except ValueError:refused+=1
            else:raise AssertionError("extra/reordered/changed/missing System mutation accepted")
    marker=p.marker("installed-event")
    serial=b"boot\n"+marker.encode()+b"\n"
    receipt=dict(peer="a",stage="installed-event",before_bytes=5,before_sha256=sha256(b"boot\n").hexdigest(),trigger_command_id=17,after_bytes=len(serial))
    p.event_receipt(receipt,serial,"installed-event","a",17)
    bad_serial=[b"boot\nX"+marker.encode()+b"\n",b"boot\n"+marker.encode()+b"X\n",
                b"boot\nnothing\n",marker.encode()+b"\nboot\n",serial+marker.encode()+b"\n"]
    for raw in bad_serial:
        changed=dict(receipt,after_bytes=len(raw))
        try:p.event_receipt(changed,raw,"installed-event","a",17)
        except ValueError:refused+=1
        else:raise AssertionError("preexisting/prefix/suffix/missing lifecycle event accepted")
    for changed in (dict(receipt,before_bytes=6),dict(receipt,trigger_command_id=16),
                    dict(receipt,before_sha256="0"*64),dict(receipt,after_bytes=len(serial)-1)):
        try:p.event_receipt(changed,serial,"installed-event","a",17)
        except ValueError:refused+=1
        else:raise AssertionError("wrong trigger/boundary accepted")
    return refused
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure tests only")
    print("Integrated Alpha journey:",self_test(),"refusals; no guest or disk mutation")
