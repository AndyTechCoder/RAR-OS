"""Independent retained signed-scenario checker; no execution or artifact writes."""
import json
from pathlib import Path
import importlib.util
CASES={"update":"update","bad-health":"update badhealth","bad-signature":"update badsig","bad-abi":"update badabi"}
def helper(name):
    path=Path(__file__).resolve().with_name(name+".py")
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m);return m
def plan(case,index,value):
    if case not in CASES or type(index) is not int or index not in (1,2,3):raise ValueError("fixed case/VM")
    if index==1:
        keys=helper("visual_oracle").plan(value)[0]
        return keys,{0:"home-1",1:"terminal-1",len(keys):"saved"}
    if index==3:
        keys=["f2","esc","f1"];scenes={0:"home-3",1:"selected-3",3:"files-3"}
        if case=="update":
            keys+=["esc","f3"];scenes[len(keys)]="terminal-3"
            keys+=list("update")+["ret","esc","f2"];scenes[len(keys)]="stale-rejected"
        return keys,scenes
    keys=["f3"]+["spc" if c==" " else c for c in CASES[case]]+["ret","esc","f2"]
    scenes={0:"home-2",1:"terminal-2",len(keys):"candidate"}
    if case=="update":
        keys+=["d"];scenes[len(keys)]="compact"
        keys+=["x","f2"];scenes[len(keys)]="fallback"
    keys+=["esc","f1"];scenes[len(keys)]="files-2"
    return keys,scenes
def commands(rows,index,case,value,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:raise ValueError("bounded QMP")
    plain=[]
    for n,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=n:
            raise ValueError("contiguous command IDs")
        plain.append({k:v for k,v in row.items() if k!="id"})
    start=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(start)]!=start:raise ValueError("paused preflight then sole continue")
    keys,scenes=plan(case,index,value);groups=[0]*(len(keys)+1);at=0
    capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
    for row in plain[len(start):]:
        if row==capture:
            groups[at]+=1
            if at not in scenes or groups[at]>24:raise ValueError("unplanned capture boundary")
        else:
            if at>=len(keys) or row!={"execute":"send-key","arguments":{
                "keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}}:
                raise ValueError("unplanned input or QMP command")
            at+=1
    if at!=len(keys) or any(groups[n]<1 for n in scenes):raise ValueError("missing input/capture")
    return list(scenes.values())
def transcript(serial,index,case):
    if type(serial) is not str or not serial.isascii() or len(serial)>65536:
        raise ValueError("bounded ASCII transcript")
    if serial.count("RAR-MODERN:GUI-READY")!=1 or any(x in serial for x in
        ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN","APP-FAULT=6")):
        raise ValueError("guest readiness/fault failure")
    markers=["UPDATE-REQUEST","UPDATE-REJECTED","UPDATE-INSTALLED","UPDATE-FALLBACK",
             "UPDATE-ACTIVE-LOST","SETTINGS-ACTIVE-FAULT"]
    expected=[]
    if index==2:
        expected=(["UPDATE-REQUEST","UPDATE-INSTALLED","SETTINGS-ACTIVE-FAULT",
                   "UPDATE-ACTIVE-LOST","UPDATE-FALLBACK"] if case=="update" else
                  ["UPDATE-REQUEST","UPDATE-REJECTED"])
    elif index==3 and case=="update":expected=["UPDATE-REQUEST","UPDATE-REJECTED"]
    positions=[]
    for name in markers:
        marker="RAR-MODERN:"+name
        if serial.count(marker)!=expected.count(name):raise ValueError("unexpected lifecycle marker count")
    for name in expected:positions.append(serial.index("RAR-MODERN:"+name))
    if positions!=sorted(positions):raise ValueError("lifecycle causality order")
    if index==2 and case=="update" and serial.count("RAR-MODERN:PRIVATE-MEMORY-RETIRED")<2:
        raise ValueError("actual old/candidate physical retirement missing")
def validate(raw,boot,firmware_sizes,case,factory,candidate):
    base=helper("runtime_evidence");visual=helper("visual_oracle")
    expected=helper("signed_runtime_evidence");profile=helper("vm_profile");persist=helper("persistence")
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024:raise ValueError("bounded JSON")
    def pairs(items):
        result={}
        for key,value in items:
            if key in result:raise ValueError("duplicate field")
            result[key]=value
        return result
    value=json.loads(raw,object_pairs_hook=pairs)
    if base.canonical(value)!=raw:raise ValueError("canonical evidence")
    fields={"schema","case","status","challenge","frames","vm_proofs","boot_sha256",
        "initial_data_sha256","frozen_data_sha256","frozen_data_base64","system_base64",
        "system_check","milestone_complete"}
    if (type(value) is not dict or set(value)!=fields or case not in CASES or value["case"]!=case or
        value["schema"]!="rar-signed-runtime-candidate-v1" or value["status"]!="observed" or
        value["milestone_complete"] is not False or value["boot_sha256"]!=base.digest(boot)):
        raise ValueError("exact nonaccepting signed envelope and trusted boot binding")
    challenge=visual.value_check(value["challenge"])
    data=base.decoded(value["frozen_data_base64"],99328);state=helper("data_oracle").inspect(data)
    if (value["frozen_data_sha256"]!=base.sha(data) or
        value["initial_data_sha256"]!=base.sha(data[:1024]+bytes(99328-1024)) or
        state["files"]!={b"note":challenge.encode()} or state["revision"]!=2 or
        state["committed_slots"]!=[0,1] or state["burned_slots"]!=[] or
        state["next_slot"]!=2 or state["readonly"]):
        raise ValueError("actual authenticated Data/challenge mismatch")
    system=base.decoded(value["system_base64"],8388608)
    checked=expected.validate_system(system,factory,candidate,"fallback" if case=="update" else "rejected")
    if value["system_check"]!=checked:raise ValueError("independent System result")
    proofs=value["vm_proofs"]
    if type(proofs) is not list or len(proofs)!=3:raise ValueError("exact three VM proofs")
    names=[];bindings=[];pids=[]
    for index,proof in enumerate(proofs,1):
        if type(proof) is not dict or set(proof)!={"cut","audit","argv","preflight","commands",
            "events","event_receipts","qmp_drained","serial"}:raise ValueError("VM proof fields")
        cut=proof["cut"]
        if (type(cut) is not dict or set(cut)!={"vm_pid","vm_returncode","backends","joined"} or
            cut["joined"] is not True or type(cut["vm_pid"]) is not int or cut["vm_pid"]<=0 or
            not base.qemu_killed(cut["vm_returncode"]) or type(cut["backends"]) is not list or len(cut["backends"])!=3):
            raise ValueError("whole VM plus three backend joins")
        pids.append(cut["vm_pid"]);transcript(proof["serial"],index,case)
        names+=commands(proof["commands"],index,case,challenge,profile)
        base.argv(proof["argv"],index,profile,readonly_data=index>1)
        preflight=proof["preflight"]
        if type(preflight) is not dict or set(preflight)!={"raw","verified"}:raise ValueError("preflight")
        verified=profile.validate_preflight(preflight["raw"],index>1,index,firmware_sizes)
        if verified!=preflight["verified"]:raise ValueError("independent paused topology")
        base.checked_event_stream(proof["events"],proof["event_receipts"],
            proof["commands"],proof["qmp_drained"],verified["rtc_path"])
        audits=[];current=[]
        for role,report in zip(("data","system","boot"),cut["backends"]):
            if type(report) is not dict or set(report)!={"returncode","problem","records","joined"} or report["joined"] is not True:
                raise ValueError("backend proof")
            base.terminated(report["returncode"],report["problem"])
            records=report["records"]
            if type(records) is not list or not records:raise ValueError("actual backend records")
            ready=records[0]
            audits.append(persist.audit(records,role,ready,index>1,index==2 and role=="system"))
            current.append((ready["device"],ready["inode"],ready["capacity"]))
        if audits!=proof["audit"] or len({(d,i) for d,i,_ in current})!=3:raise ValueError("audit or disk separation")
        bindings.append(current)
    if len(set(pids))!=3 or bindings[1:]!=[bindings[0],bindings[0]]:
        raise ValueError("fresh processes must retain the same three separate inodes")
    frames=value["frames"]
    if type(frames) is not list or len(frames)!=len(names):raise ValueError("exact frame count")
    for row,name in zip(frames,names):
        if type(row) is not dict or set(row)!={"scene","sha256","actual_ppm"} or row["scene"]!=name:
            raise ValueError("frame identity")
        pixels=base.decoded(row["actual_ppm"],len(visual.HEADER)+640*480*3)
        if name.startswith("home-"):digest=visual.validate(pixels,0)
        elif name.startswith("terminal-"):digest=visual.validate(pixels,1)
        elif name=="saved":digest=visual.validate(pixels,2,challenge)
        elif name.startswith("files-"):digest=visual.validate(pixels,3,challenge)
        else:digest=expected.settings_validate(pixels,visual,name=="compact" or name=="candidate" and case=="update",name=="compact")
        if row["sha256"]!=digest:raise ValueError("actual full frame hash/expectation")
    return dict(case=case,frames=len(frames),fresh_vms=3,data_sha256=base.sha(data),
        system=checked,content_validated=True,provenance_validated=False,milestone_complete=False)

def self_test():
    # Inert command/serial fixtures only, never VM or target execution.
    profile=helper("vm_profile");base=helper("runtime_evidence")
    value="abcdefghijklmnop"*2;rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except ValueError:rejected+=1
        else:raise AssertionError("invalid signed evidence accepted")
    for case in CASES:
        for index in (1,2,3):
            keys,scenes=plan(case,index,value)
            plain=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
            capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
            for at in range(len(keys)+1):
                if at in scenes:plain.append(capture)
                if at<len(keys):plain.append({"execute":"send-key","arguments":{
                    "keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}})
            rows=[dict(row,id=n) for n,row in enumerate(plain,1)]
            assert commands(rows,index,case,value,profile)==list(scenes.values())
            for changed in (rows[:-1],[dict(rows[0],id=True)]+rows[1:],
                rows+[{"id":len(rows)+1,"execute":"cont"}],
                [dict(row,id=n) for n,row in enumerate(plain[1:],1)]):
                reject(lambda changed=changed:commands(changed,index,case,value,profile))
            names=[]
            if index==2:
                names=(["UPDATE-REQUEST","UPDATE-INSTALLED","SETTINGS-ACTIVE-FAULT",
                    "UPDATE-ACTIVE-LOST","UPDATE-FALLBACK"] if case=="update" else
                    ["UPDATE-REQUEST","UPDATE-REJECTED"])
            elif index==3 and case=="update":names=["UPDATE-REQUEST","UPDATE-REJECTED"]
            serial="RAR-MODERN:GUI-READY\n"+"".join("RAR-MODERN:"+n+"\n" for n in names)
            if index==2 and case=="update":serial+="RAR-MODERN:PRIVATE-MEMORY-RETIRED\n"*2
            transcript(serial,index,case)
            reject(lambda:transcript(serial+"RAR-PANIC",index,case))
            reject(lambda:transcript(serial+"RAR-MODERN:UPDATE-INSTALLED",index,case))
            reject(lambda:transcript(serial.replace("GUI-READY","NOT-READY"),index,case))
            if names:reject(lambda:transcript(serial.replace("RAR-MODERN:"+names[0],"missing",1),index,case))
    for index in (0,4,True,None):reject(lambda index=index:plan("update",index,value))
    reject(lambda:base.argv([],1,profile,readonly_data=1))
    # IDE argv deliberately stays writable; the private host descriptor and
    # backend refusal (not QEMU's IDE flag) enforce physical immutability.
    args=profile.argv(3,20,21,22,True)
    assert base.argv(args,3,profile,readonly_data=True)
    assert args==profile.argv(3,20,21,22,False)
    persist=helper("persistence")
    ready=dict(type="ready",kind="data",readonly=True,export_readonly=False,
        capacity=99328,device=1,inode=2)
    assert persist.audit([ready],"data",ready,True)["counts"]["write"]==0
    reject(lambda:persist.audit([ready],"data",ready,False))
    writable=dict(ready,readonly=False)
    reject(lambda:persist.audit([writable],"data",writable,True))
    write=dict(type="request",operation="write",offset=0,length=512)
    reject(lambda:persist.audit([ready,write],"data",ready,True))
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure signed evidence self-test only")
    print("Signed runtime evidence:",self_test(),"negative fixtures; not guest proof")
