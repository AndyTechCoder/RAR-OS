"""Trusted cloud orchestration for the fixed 60 Data fault cases.
No proposal-supplied host command or path. All VM execution remains inside the
existing inspected, unprivileged, disconnected disposable launcher container.
"""
import json
import time
from pathlib import Path
import importlib.util

def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed controller helper")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def observe(execute,launcher,inputs,boot_sha256,firmware_sizes,retain,save,report):
    evidence=helper("fault_evidence");campaign=helper("fault_campaign")
    base=helper("runtime_evidence")
    captures=[];seen=[set() for _ in range(4)];total=0
    deadline=time.monotonic()+2400
    report["fault_cases"]=[];report["phase"]="fixed-data-fault-campaign";save()
    for index in range(60):
        remaining=int(deadline-time.monotonic())
        if remaining<1:raise TimeoutError("bounded 40-minute fault execution budget")
        report["active_fault_case"]=index;save()
        raw=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/fault_launch.py",str(index)],
                    [(inputs,"/artifact")],min(300,remaining),8*1024**2)
        if type(raw) is not bytes or not 1<=len(raw)<=8*1024**2:
            raise ValueError("bounded raw fault capture")
        total+=len(raw)
        if total>384*1024**2:raise ValueError("bounded retained fault campaign")
        # Preserve the exact bounded observation even if content checking fails.
        retain("fault-"+str(index).zfill(2)+".json",raw)
        checked=evidence.validate(raw,index,boot_sha256,firmware_sizes)
        doc=json.loads(raw)
        initial=base.decoded(doc["initial_data_base64"],99328)
        values=(doc["initial_data_sha256"],initial[32:64],initial[64:96],doc["challenge"])
        if any(value in prior for value,prior in zip(values,seen)):
            raise ValueError("fresh image/key/challenge required before next case")
        for value,prior in zip(values,seen):prior.add(value)
        captures.append(raw)
        report["fault_cases"].append(dict(case=index,input_sha256=base.sha(raw),content=checked))
        save()
    report["phase"]="independent-fault-aggregate";save()
    checked=campaign.validate(captures,boot_sha256,firmware_sizes)
    report["fault_campaign"]=checked
    report["status"]="fault-campaign-observed"
    report["phase"]="complete-fixed-data-fault-campaign"
    report["milestone_complete"]=False
    report["crypto_interoperability_accepted"]=False
    save()
    print("Modern: 60 fixed cloud Data faults observed and content-checked; M4 remains incomplete.",flush=True)

if __name__=="__main__":
    helper("runtime_controller").main("fault-campaign")
