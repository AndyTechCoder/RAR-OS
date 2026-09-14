"""Pure plan, wire and mutation checks; never creates a socket or VM."""
import importlib.util
from pathlib import Path
def load(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def self_test():
    p=load("network_fault_plan");e=load("network_fault_evidence");wire=load("expansion_wire")
    challenges=["a"*32,"b"*32];refusals=0
    for mode in ("faults","expiry","peer-stop"):
        steps=p.plan(mode,challenges)
        for peer,keys,stage,value in steps:
            assert peer in ("a","b") and type(keys) is list
            if stage in ("peer-stopped","send-only"):continue
            pixels=p.expected(stage,value);p.validate(pixels,stage,value)
            try:p.validate(pixels[:-1]+bytes([pixels[-1]^1]),stage,value)
            except ValueError:refusals+=1
            else:raise AssertionError("mutated full scene accepted")
        for index,peer in enumerate(("a","b"),1):
            profile=load("expansion_profile").Profile(load("vm_profile"),peer,37)
            rows=[{"execute":"qmp_capabilities"}]+[v for _,v in profile.preflight_requests()]+[{"execute":"cont"}]
            for owner,keys,stage,value in steps:
                if owner!=peer:continue
                for key in keys:rows.append({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":key}],"hold-time":50}})
                if stage not in ("peer-stopped","send-only"):rows.append({"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}})
            rows=[dict(row,id=i) for i,row in enumerate(rows,1)]
            p.commands(rows,index,peer,mode,challenges,profile)
            assert len(rows)<=512, "fixed journey exceeds actual Vm.request limit"
            # Three capture attempts per boundary must fit, not merely an
            # ideal one-shot receipt. Existing runtime limit stays512.
            delayed=[]
            for row in rows:
                delayed.extend([row]*(3 if row["execute"]=="screendump" else 1))
            delayed=[dict(row,id=i) for i,row in enumerate(delayed,1)]
            assert len(delayed)<=512, "ordinary capture settling lacks headroom"
            p.commands(delayed,index,peer,mode,challenges,profile)
            oversized=[dict(rows[-1],id=i) for i in range(1,514)]
            try:p.commands(oversized,index,peer,mode,challenges,profile)
            except ValueError:refusals+=1
            else:raise AssertionError("receipt above actual512 limit accepted")
            for bad in ([],rows[:-1],rows+[dict(execute="cont",id=len(rows)+1)]):
                try:p.commands(bad,index,peer,mode,challenges,profile)
                except ValueError:refusals+=1
                else:raise AssertionError("incomplete or extra command accepted")
    actual=e.expected_wire("faults",challenges)
    assert len(actual)==9 and [len(x) for x in actual]==[74,60]+[74]*7
    assert actual[0][14]==0x65
    assert actual[1]==wire.packet("b",b"a"*32,1)[:60]
    bad=bytearray(wire.packet("b",b"a"*32,2));bad[40]^=1
    assert actual[2]==bytes(bad) and actual[3]==wire.packet("b",b"a"*32,4)
    assert actual[4:]==[wire.packet("b",b"b"*32,i) for i in range(5,10)]
    assert e.expected_wire("expiry",challenges)==[] and e.expected_wire("peer-stop",challenges)==[]
    return refusals
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure self-test only")
    print("Network campaign pure checks:",self_test(),"refusals; no guest or socket")
