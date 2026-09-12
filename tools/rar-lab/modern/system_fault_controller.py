"""Bounded trusted-main campaign; all fixtures confined by runtime_controller."""
import gzip,time
from hashlib import sha256

def observe(load,execute,launcher,inputs,boot,sizes,evidence,report,save,bank,mode):
    if mode not in ("install","repair"):raise ValueError("fixed System transaction")
    factory=bank["modern-settings-factory.layer"];candidate=bank["modern-settings-update.layer"]
    plans=load("signed_runtime_evidence").system_fault_cases(factory,candidate,mode)
    if not 1<=len(plans)<=256:raise ValueError("System campaign case budget")
    validator=load("system_fault_validate");public=load("signed_runtime_controller").public_file
    start=time.monotonic();seen={key:set() for key in ("image_id","key_sha256","challenge")}
    report["system_faults"]={"mode":mode,"planned":len(plans),"completed":0,"cases":[]}
    save()
    for case,plan in enumerate(plans):
        # Leave a complete per-case deadline inside the two-hour campaign cap.
        if time.monotonic()-start>6600:raise TimeoutError("bounded System campaign")
        print("System fault",mode,case+1,"/",len(plans),plan,flush=True)
        raw=execute(launcher,["/usr/bin/python3","-I","-B",
            "/opt/rar-modern/system_fault_launch.py",mode,str(case)],
            [(inputs,"/artifact")],600,64*1024*1024)
        if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024:
            raise ValueError("bounded actual capture")
        packed=gzip.compress(raw,compresslevel=6,mtime=0)
        name="system-"+mode+"-"+str(case).zfill(3)+".json.gz"
        public(evidence/name,packed)
        # Retain failure captures too; actual content validation precedes acceptance.
        record={"case":case,"plan":plan,"capture":name,"raw_sha256":sha256(raw).hexdigest(),
            "raw_bytes":len(raw),"gzip_sha256":sha256(packed).hexdigest(),"gzip_bytes":len(packed)}
        report["system_faults"]["cases"].append(record);save()
        result=validator.validate(raw,boot,sizes,mode,case,factory,candidate)
        for key,values in seen.items():
            value=result[key]
            if value in values:raise ValueError("reused independent fixture "+key)
            values.add(value)
        record["check"]=result
        report["system_faults"]["completed"]=case+1;save()
    report["status"]="observed";save()
    print("System fault campaign passed:",mode,len(plans),"cases",3*len(plans),
        "fresh VM lifecycles; final milestone gate still required.",flush=True)
