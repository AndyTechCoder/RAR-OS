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
    return refused
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure tests only")
    print("Integrated Alpha journey:",self_test(),"refusals; no guest or disk mutation")
