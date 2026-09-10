"""Retained fault evidence validator; pure bytes only, no scenario activation.
Outer trusted controller must bind exact source/tool/provenance and confinement.
"""
import json
from pathlib import Path
import importlib.util

def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed immutable sibling")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode("ascii")+b"\n"

def exact(value,wanted):
    if canonical(value)!=canonical(wanted):raise ValueError("exact typed evidence value")

def fields(value,names):
    if type(value) is not dict or set(value)!=set(names.split()):
        raise ValueError("exact evidence object fields")

def selection(case):
    """Independent fixed campaign mapping; never import the scenario producer."""
    if type(case) is not int or not 0<=case<60:raise ValueError("fixed fault case")
    operation="write" if case<30 else "flush"
    ordinal=(case%30)//5+1
    effect=("before-cut","after-cut","error","torn-cut","short-error")[case%5]
    return dict(operation=operation,ordinal=ordinal,effect=effect,
        prefix=255 if effect in ("torn-cut","short-error") else 0)

def classification(plan):
    slot=(plan["ordinal"]-1)//3;stage=(plan["ordinal"]-1)%3
    committed=stage==2 and (plan["effect"] in ("torn-cut","short-error") or
        plan["operation"]=="flush" and plan["effect"]=="after-cut")
    virgin=stage==0 and (plan["effect"] in ("before-cut","error") or
        plan["operation"]=="write" and plan["effect"]=="after-cut")
    revision=slot+int(committed)
    return dict(revision=revision,committed_slots=list(range(revision)),
        burned_slots=[] if committed or virgin else [slot],
        next_slot=slot+int(committed or not virgin),readonly=False)

def first_commands(rows,value,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:
        raise ValueError("bounded first-VM transcript")
    plain=[]
    for index,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=index:
            raise ValueError("contiguous first-VM commands")
        plain.append({k:v for k,v in row.items() if k!="id"})
    prefix=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
    exact(plain[:len(prefix)],prefix)
    capture={"execute":"screendump","arguments":{"filename":profile.directory(1)+"/frame.ppm"}}
    keys=helper("visual_oracle").plan(value)[0]
    at=0;captures=[0,0]
    for row in plain[len(prefix):]:
        if row==capture:
            if at not in (0,1):raise ValueError("no capture after typed faulted save")
            captures[at]+=1
            if captures[at]>24:raise ValueError("bounded capture polling")
        else:
            if at>=len(keys):raise ValueError("extra first-VM input")
            exact(row,{"execute":"send-key","arguments":{
                "keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}})
            at+=1
    if at!=len(keys) or min(captures)<1:
        raise ValueError("complete submitted save and preceding GUI captures required")

def mutation_geometry(records,plan):
    """Fixed single-sector reservation/payload/commit sequence, no extra writes.
    Reads do not advance the per-operation mutation ordinals.
    """
    mutations=[row for row in records if row.get("type")=="request" and
               row.get("operation") in ("write","flush")]
    count=2*plan["ordinal"]-(1 if plan["operation"]=="write" else 0)
    if len(mutations)!=count:raise ValueError("exact publication mutation prefix")
    for index,row in enumerate(mutations):
        operation="write" if index%2==0 else "flush"
        offset=(2+index//2)*512 if operation=="write" else 0
        exact(row,dict(type="request",operation=operation,offset=offset,
                       length=512 if operation=="write" else 0))

def vm_proof(proof,index,value,plan,firmware_sizes,command_checker=None):
    base=helper("runtime_evidence");profile=helper("vm_profile")
    persistence=helper("persistence");audit=helper("fault_audit")
    names="cut audit argv preflight commands events event_receipts qmp_drained serial"
    fields(proof,names+(" fault" if index==1 else ""))
    cut=proof["cut"]
    fields(cut,"vm_pid vm_returncode backends joined"+(" entry" if index==1 else ""))
    if (cut["joined"] is not True or type(cut["vm_pid"]) is not int or cut["vm_pid"]<=0 or
        not base.qemu_killed(cut["vm_returncode"]) or type(cut["backends"]) is not list or
        len(cut["backends"])!=3):
        raise ValueError("whole deliberate VM kill and three backend joins")
    serial=proof["serial"]
    if (type(serial) is not str or not serial.isascii() or len(serial)>65536 or
        "RAR-MODERN:GUI-READY" not in serial or any(mark in serial for mark in
        ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN"))):
        raise ValueError("actual bounded readiness without guest failure")
    fault_command_id=None
    if index==1:fault_command_id=(first_commands if command_checker is None else command_checker)(proof["commands"],value,profile)
    else:base.commands(proof["commands"],index,value,profile)
    base.argv(proof["argv"],index,profile)
    fields(proof["preflight"],"raw verified")
    verified=profile.validate_preflight(proof["preflight"]["raw"],index==2,index,firmware_sizes)
    exact(proof["preflight"]["verified"],verified)
    if index==1:
        fields(cut["entry"],"vm_code backend_codes backend_problems event_count")
        helper("fault_events").validate(proof["events"],proof["event_receipts"],
            proof["commands"],proof["qmp_drained"],verified["rtc_path"],cut["entry"].get("event_count"),plan,fault_command_id=fault_command_id)
    else:
        base.checked_event_stream(proof["events"],proof["event_receipts"],
            proof["commands"],proof["qmp_drained"],verified["rtc_path"])
    iscut=plan["effect"] in ("before-cut","after-cut","torn-cut")
    if index==1:
        fields(proof["fault"],"plan request_index event_index offset length delivery")
        exact(proof["fault"]["plan"],plan)
        exact(proof["fault"]["delivery"],dict(code=20 if iscut else None,
            problem="cut" if iscut else None,eof=iscut))
        exact(cut["entry"],dict(vm_code=None,backend_codes=[20 if iscut else None,None,None],
            backend_problems=["cut" if iscut else None,None,None],event_count=cut["entry"]["event_count"]))
    summaries=[];bindings=[]
    for number,(role,report) in enumerate(zip(("data","system","boot"),cut["backends"])):
        fields(report,"returncode problem records joined")
        if report["joined"] is not True:raise ValueError("all actual backend joins")
        records=report["records"]
        if type(records) is not list or not records:raise ValueError("actual backend records")
        if index==1 and number==0:
            summary=audit.scan(records,plan,records[0])
            if summary is None:raise ValueError("missing planned fault")
            mutation_geometry(records,plan)
            for key in ("request_index","event_index","offset","length"):
                exact(proof["fault"][key],summary[key])
            if iscut:
                exact(report["returncode"],20);exact(report["problem"],"cut")
                if summary["terminal"] is not True:raise ValueError("complete cut terminal")
            else:
                base.terminated(report["returncode"],report["problem"])
                if report["returncode"]==21 and summary["terminal"] is not True:
                    raise ValueError("complete post-kill Data EOF")
        else:
            base.terminated(report["returncode"],report["problem"])
            summary=persistence.audit(records,role,records[0],index==2)
            if report["returncode"]==21 and summary["transport_closed"] is not True:
                raise ValueError("complete post-kill peer EOF")
        summaries.append(summary)
        bindings.append(tuple(records[0][key] for key in ("device","inode","capacity")))
    exact(proof["audit"],summaries)
    if len({(device,inode) for device,inode,_ in bindings})!=3:raise ValueError("three separate disk identities")
    return cut["vm_pid"],bindings

def validate(raw,case,expected_boot_digest,firmware_sizes):
    base=helper("runtime_evidence")
    visual=helper("visual_oracle");disk=helper("data_oracle")
    plan=selection(case);reverse=False
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024 or not raw.endswith(b"\n"):
        raise ValueError("bounded canonical fault envelope")
    def pairs(rows):
        result={}
        for key,value in rows:
            if key in result:raise ValueError("duplicate evidence key")
            result[key]=value
        return result
    try:doc=json.loads(raw,object_pairs_hook=pairs)
    except (ValueError,UnicodeError,RecursionError) as error:raise ValueError("invalid JSON") from error
    if canonical(doc)!=raw:raise ValueError("canonical retained JSON bytes")
    fields(doc,"schema case plan reverse_flush challenge frames vm_proofs initial_data_sha256 "
        "frozen_data_sha256 initial_data_base64 frozen_data_base64 expected_revision "
        "expected_classification system_sha256 boot_sha256 status milestone_complete")
    exact(doc["schema"],"rar-modern-data-fault-candidate-v0")
    exact(doc["status"],"observed-not-independently-accepted")
    exact(doc["milestone_complete"],False)
    exact(doc["case"],case);exact(doc["plan"],plan);exact(doc["reverse_flush"],reverse)
    exact(doc["boot_sha256"],base.digest(expected_boot_digest))
    exact(doc["system_sha256"],base.sha(bytes(8388608)))
    value=visual.value_check(doc["challenge"])
    initial=base.decoded(doc["initial_data_base64"],99328)
    frozen=base.decoded(doc["frozen_data_base64"],99328)
    exact(doc["initial_data_sha256"],base.sha(initial))
    exact(doc["frozen_data_sha256"],base.sha(frozen))
    virgin=disk.inspect(initial)
    if (any(initial[1024:]) or initial[:1024]!=frozen[:1024] or
        virgin["files"]!={} or virgin["revision"]!=0 or virgin["next_slot"]!=0):
        raise ValueError("same untouched headers from one empty Data fixture")
    result=disk.inspect(frozen);expected=classification(plan)
    exact(doc["expected_classification"],expected)
    exact(doc["expected_revision"],expected["revision"])
    for key,wanted in expected.items():exact(result[key],wanted)
    revision=expected["revision"]
    wanted=({}, {b"note":b""}, {b"note":value.encode("ascii")})[revision]
    if result["files"]!=wanted:raise ValueError("exact old/new authenticated file state")
    frames=doc["frames"]
    if type(frames) is not list or len(frames)!=4:raise ValueError("exact four fault frames")
    for number,frame in enumerate(frames):
        fields(frame,"scene sha256 actual_ppm")
        pixels=base.decoded(frame["actual_ppm"],len(visual.HEADER)+640*480*3)
        if number<3:
            scene=(0,1,0)[number];exact(frame["scene"],visual.SCENES[scene])
            actual=visual.validate(pixels,scene)
        else:
            state=("absent","empty","written")[revision]
            exact(frame["scene"],"recovered-"+state)
            actual=visual.recovered_validate(pixels,state,value if revision==2 else None)
        exact(frame["sha256"],actual)
    proofs=doc["vm_proofs"]
    if type(proofs) is not list or len(proofs)!=2:raise ValueError("two actual fault/reboot proofs")
    first=vm_proof(proofs[0],1,value,plan,firmware_sizes)
    second=vm_proof(proofs[1],2,value,plan,firmware_sizes)
    helper("fault_replay").readonly(proofs[1]["cut"]["backends"][0]["records"])
    replay=helper("fault_replay").validate(proofs[0]["cut"]["backends"][0]["records"],
        initial,frozen,value,plan)
    if first[0]==second[0] or first[1]!=second[1]:
        raise ValueError("fresh VM reopens the same three separate retained inodes")
    return dict(schema="rar-modern-data-fault-validation-v0",case=case,
        data_sha256=base.sha(frozen),revision=revision,frames=4,fresh_vms=2,
        request_count=replay["requests"],content_validated=True,provenance_validated=False,milestone_complete=False)
