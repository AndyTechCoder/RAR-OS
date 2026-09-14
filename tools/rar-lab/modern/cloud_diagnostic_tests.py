"""Pure failed-receipt refusal/log bounds, no cloud execution."""
import importlib.util,json
from pathlib import Path
def load(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def self_test():
    # Only the Alpha/network failure envelope is examined before unrelated
    # acceptance fields. An oversized escaped diagnostic must still refuse.
    count=0
    for module,schema,args in (
        ("alpha_evidence","rar-native-alpha-failure-v1",({}, "0"*64,(1,1),bytes([1])*32,"first")),
        ("network_fault_evidence","rar-network-campaign-failure-v1",({},"0"*64,(1,1),"faults"))):
        raw=load("runtime_evidence").canonical(dict(schema=schema,status="failed",reason="bounded",serial=["x"*30000]))
        try:load(module).validate(raw,*args)
        except ValueError as error:
            assert str(error).startswith("actual cloud scenario failed: ") and len(str(error))<=24610
            count+=1
        else:raise AssertionError("failed runtime receipt accepted")
    return count
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure tests only")
    print("Cloud diagnostic receipts:",self_test(),"bounded unconditional refusals")
