"""Pure candidate Alpha helper checks. No subprocess, socket, VM or target."""
import ast
import base64
import importlib.util
from pathlib import Path
import sys
def load(name):
    path=Path(__file__).with_name(name+".py")
    spec=importlib.util.spec_from_file_location(name,path)
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def self_test():
    # Parse fixed launch/controller sources without importing their entrypoints.
    for name in ("alpha_controller","alpha_scenario","alpha_first_launch","alpha_fresh_launch",
                 "alpha_dispatch","alpha_evidence","native_packages","runtime_controller"):
        ast.parse(Path(__file__).with_name(name+".py").read_text())
    plan=load("alpha_plan");profile=load("expansion_profile");vm=load("vm_profile")
    challenges=[c*32 for c in "abcd"];rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except (ValueError,KeyError,TypeError):rejected+=1
        else:raise AssertionError("invalid Alpha fixture accepted")
    for mode in ("first","fresh"):
        steps=plan.plan(mode,challenges)
        assert len(steps)==(24 if mode=="first" else 7)
        for index,peer in enumerate(("a","b"),1):
            p=profile.Profile(vm,peer,19)
            rows=[{"execute":"qmp_capabilities"}]+[v for _,v in p.preflight_requests()]+[{"execute":"cont"}]
            capture={"execute":"screendump","arguments":{"filename":p.directory(index)+"/frame.ppm"}}
            for owner,keys,*_ in steps:
                if owner!=peer:continue
                for key in keys:rows.append({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":key}],"hold-time":50}})
                rows.append(capture)
            rows=[dict(row,id=n) for n,row in enumerate(rows,1)]
            plan.commands(rows,index,peer,mode,challenges,p)
            reject(lambda:plan.commands(rows[:-1],index,peer,mode,challenges,p))
            changed=[dict(r) for r in rows];changed[-1]["id"]=1
            reject(lambda:plan.commands(changed,index,peer,mode,challenges,p))
    reject(lambda:plan.plan("bad",challenges));reject(lambda:plan.plan("first",["a"*32]*4))
    packages=load("native_packages");signer=load("lab_signer")
    spec=importlib.util.spec_from_file_location("c_app_pe",Path(__file__).resolve().parent.parent/"expansion/c_app_pe.py")
    converter=importlib.util.module_from_spec(spec);spec.loader.exec_module(converter)
    pe=converter.convert(converter.fixture())
    raw=b"".join(b"RAR-APP-FILE:"+n.encode()+b"\n"+base64.b64encode(pe)+b"\n" for n in packages.NAMES)+b"RAR-APP-BUILD:END\n"
    def inspect(payload,app):assert payload==pe and app is True
    assert list(packages.unpack(raw,inspect))==list(packages.NAMES)
    for bad in (b"",raw[:-1]+b"X",raw+b"x",raw.replace(b"rar-notes.efi",b"../notes.efi")):
        reject(lambda bad=bad:packages.unpack(bad,inspect))
    for i in (0,1):
        package=packages.package(pe,i,vars(signer))
        assert len(package)==512+len(pe) and package[16:32]==packages.IDENTITIES[i]
        assert int.from_bytes(package[116:120],"little")== (3 if i==0 else 1)
        assert package[512:]==pe
    reject(lambda:packages.package(pe,True,vars(signer)))
    reject(lambda:packages.package(pe,2,vars(signer)))
    return rejected
if __name__=="__main__":
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("isolated pure test only")
    print("Alpha helpers:",self_test(),"refusals; no target execution")
