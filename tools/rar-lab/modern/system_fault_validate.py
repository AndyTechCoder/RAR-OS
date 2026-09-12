"""Inactive independent retained System-fault evidence validator."""
import json
from pathlib import Path
import importlib.util

def helper(name):
    path=Path(__file__).resolve().with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed trusted helper")
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module);return module

def mutation_sequence(records,factory,candidate,mode,plan):
    expected=helper("signed_runtime_evidence").system_fault_operations(factory,candidate,mode)
    at=0;hit=False
    for row in records[1:]:
        if row.get("type")=="request" and hit and row.get("operation") in ("write","flush"):
            raise ValueError("System retried mutation after selected failure")
        if row.get("type")!="event":continue
        event=row["event"]
        if event["operation"] not in ("write","flush"):continue
        if hit or at>=len(expected):raise ValueError("extra System mutation")
        wanted=expected[at];at+=1
        if any(event.get(key)!=value for key,value in wanted.items()):
            raise ValueError("wrong System mutation geometry or complete payload hash")
        if "injection" in event:
            if helper("runtime_evidence").canonical(event["injection"])!=helper("runtime_evidence").canonical(plan):
                raise ValueError("wrong injected mutation")
            hit=True
    if not hit:raise ValueError("missing exact selected mutation")

def complete_mutations(records,operations):
    events=[row["event"] for row in records if row.get("type")=="event"
        and row["event"].get("operation") in ("write","flush")]
    if len(events)!=len(operations):raise ValueError("unexpected complete System mutation count")
    if any(any(event.get(key)!=value for key,value in wanted.items())
        for event,wanted in zip(events,operations)):
        raise ValueError("unexpected complete System mutation bytes/order")

def plan(mode,index,value):
    if mode not in ("install","repair") or type(index) is not int or index not in (1,2,3):
        raise ValueError("fixed mode/VM")
    if index==1:
        keys=helper("visual_oracle").plan(value)[0]
        scenes={0:"home-1",1:"terminal-1",len(keys):"saved"}
        if mode=="repair":
            keys+=list("update")+["ret","esc","f2"];scenes[len(keys)]="installed-1"
        return keys,scenes
    if index==2:
        if mode=="repair":return [],{}
        return ["f3"]+list("update")+["ret"],{0:"home-2",1:"terminal-2"}
    return ["f2","esc","f1"],{0:"home-3",1:"selected-3",3:"files-3"}

def commands(rows,index,mode,value,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:raise ValueError("bounded commands")
    plain=[]
    for number,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=number:
            raise ValueError("contiguous QMP IDs")
        plain.append({k:v for k,v in row.items() if k!="id"})
    start=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(start)]!=start:raise ValueError("paused certification then sole CONT")
    keys,scenes=plan(mode,index,value);groups=[0]*(len(keys)+1);at=0
    capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
    for row in plain[len(start):]:
        if row==capture:
            groups[at]+=1
            if at not in scenes or groups[at]>24:raise ValueError("unplanned capture")
        else:
            if at>=len(keys) or row!={"execute":"send-key","arguments":{
                "keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}}:
                raise ValueError("unplanned guest input or host command")
            at+=1
    if at!=len(keys) or any(groups[n]<1 for n in scenes):raise ValueError("missing input/capture")
    return list(scenes.values())

def fault_serial(serial,mode,matched):
    if type(serial) is not str or not serial.isascii() or len(serial)>65536:
        raise ValueError("bounded ASCII transcript")
    if any(x in serial for x in ("UNEXPECTED-USER-FAULT","INVALID-USER-RETURN","APP-FAULT=6")):
        raise ValueError("unrelated guest isolation fault")
    if serial.count("RAR-MODERN:GUI-READY")!=(1 if mode=="install" else 0):
        raise ValueError("unexpected fault-time GUI readiness")
    for marker in ("UPDATE-INSTALLED","UPDATE-FALLBACK","UPDATE-ACTIVE-LOST",
                   "SETTINGS-ACTIVE-FAULT","STALE-AUTHORITY-REVOKED"):
        if "RAR-MODERN:"+marker in serial:raise ValueError("uncommitted fault reported activation")
    if serial.count("RAR-MODERN:UPDATE-REQUEST")!=(1 if mode=="install" else 0):
        raise ValueError("wrong update trigger count")
    rejected=serial.count("RAR-MODERN:UPDATE-REJECTED")
    if rejected not in ((0,1) if mode=="install" else (0,)):
        raise ValueError("unexpected rejection count")
    # A deliberate cut can truncate the exact expected reconcile panic emission.
    # Never permit a different panic or any panic lacking the backend fault proof.
    helper("system_selector_fault").serial_status(serial.encode(),matched)
    if "RAR-PANIC" in serial and matched is None:raise ValueError("unproven panic")

def validate(raw,boot,firmware_sizes,mode,case,factory,candidate):
    base=helper("runtime_evidence");expected=helper("signed_runtime_evidence")
    visual=helper("visual_oracle");persist=helper("persistence");profile=helper("vm_profile")
    signed=helper("signed_runtime_validate");fault_audit=helper("fault_audit")
    plans=expected.system_fault_cases(factory,candidate,mode)
    if type(case) is not int or not 0<=case<len(plans)<=256:raise ValueError("bounded fixed case")
    wanted_plan=plans[case]
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024:raise ValueError("bounded JSON")
    def pairs(items):
        result={}
        for key,value in items:
            if key in result:raise ValueError("duplicate field")
            result[key]=value
        return result
    value=json.loads(raw,object_pairs_hook=pairs)
    fields={"schema","mode","case","plan","challenge","frames","vm_proofs","boot_sha256",
        "initial_data_sha256","frozen_data_sha256","frozen_data_base64","before_system_base64",
        "frozen_system_base64","restarted_system_base64","system_corruption","choice","status","milestone_complete"}
    if (type(value) is not dict or set(value)!=fields or base.canonical(value)!=raw or
        value["schema"]!="rar-system-fault-candidate-v0" or value["mode"]!=mode or
        type(value["case"]) is not int or value["case"]!=case or
        base.canonical(value["plan"])!=base.canonical(wanted_plan) or value["status"]!="observed" or
        value["milestone_complete"] is not False or value["boot_sha256"]!=base.digest(boot)):
        raise ValueError("exact nonaccepting envelope and source boot")
    challenge=visual.value_check(value["challenge"])
    data=base.decoded(value["frozen_data_base64"],99328);state=helper("data_oracle").inspect(data)
    if (value["frozen_data_sha256"]!=base.sha(data) or
        value["initial_data_sha256"]!=base.sha(data[:1024]+bytes(99328-1024)) or
        state["files"]!={b"note":challenge.encode()} or state["revision"]!=2 or
        state["committed_slots"]!=[0,1] or state["burned_slots"]!=[] or state["next_slot"]!=2 or state["readonly"]):
        raise ValueError("exact authenticated Data/challenge")
    before=base.decoded(value["before_system_base64"],8388608)
    installed,damaged,_=expected.repair_images(factory,candidate)
    if mode=="repair":
        receipt=dict(offsets=[1024,1024+expected.SLOT_BYTES],xor=1,
            before_sha256=base.sha(installed),after_sha256=base.sha(damaged))
        if before!=damaged or base.canonical(value["system_corruption"])!=base.canonical(receipt):
            raise ValueError("exact post-join repair corruption")
    else:
        initial=bytearray(installed);initial[512:1024]=bytes(512)
        initial[1024+expected.SLOT_BYTES:]=bytes(8388608-1024-expected.SLOT_BYTES)
        if before!=bytes(initial) or value["system_corruption"] is not None:
            raise ValueError("exact pristine install input")
    frozen=base.decoded(value["frozen_system_base64"],8388608)
    restarted=base.decoded(value["restarted_system_base64"],8388608)
    after,updated,choice=expected.system_fault_restart(factory,candidate,mode,case)
    if (frozen!=expected.system_fault_image(factory,candidate,mode,case) or
        restarted!=after or value["choice"]!=choice):
        raise ValueError("exact stable bytes and fresh restart selection")
    proofs=value["vm_proofs"]
    if type(proofs) is not list or len(proofs)!=3:raise ValueError("exact three VM lifecycles")
    names=[];bindings=[];pids=[]
    for index,proof in enumerate(proofs,1):
        fault=index==2
        fields={"cut","audit","argv","preflight","commands","events","event_receipts","qmp_drained","serial"}
        if fault:fields.add("fault")
        if type(proof) is not dict or set(proof)!=fields:raise ValueError("VM proof fields")
        cut=proof["cut"];cut_fields={"vm_pid","vm_returncode","backends","joined"}
        if fault:cut_fields.add("entry")
        if (type(cut) is not dict or set(cut)!=cut_fields or cut["joined"] is not True or
            type(cut["vm_pid"]) is not int or cut["vm_pid"]<=0 or
            not base.qemu_killed(cut["vm_returncode"]) or
            type(cut["backends"]) is not list or len(cut["backends"])!=3):
            raise ValueError("whole QEMU and all backend joins")
        pids.append(cut["vm_pid"]);names+=commands(proof["commands"],index,mode,challenge,profile)
        base.argv(proof["argv"],index,profile,readonly_data=index>1)
        preflight=proof["preflight"]
        if type(preflight) is not dict or set(preflight)!={"raw","verified"}:raise ValueError("preflight fields")
        verified=profile.validate_preflight(preflight["raw"],index>1,index,firmware_sizes)
        if verified!=preflight["verified"]:raise ValueError("paused topology mismatch")
        audits=[];current=[];matched=None
        for role,report in zip(("data","system","boot"),cut["backends"]):
            if type(report) is not dict or set(report)!={"returncode","problem","records","joined"} or report["joined"] is not True:
                raise ValueError("backend proof fields")
            rows=report["records"]
            if type(rows) is not list or not rows:raise ValueError("backend records")
            ready=rows[0]
            if fault and role=="system":
                matched=fault_audit.scan(rows,wanted_plan,ready,role="system")
                if matched is None:raise ValueError("missing exact System fault")
                mutation_sequence(rows,factory,candidate,mode,wanted_plan)
                is_cut=wanted_plan["effect"] in ("before-cut","after-cut","torn-cut")
                if is_cut:
                    if type(report["returncode"]) is not int or report["returncode"]!=20 or report["problem"]!="cut" or matched["terminal"] is not True:
                        raise ValueError("cut backend termination")
                else:
                    base.terminated(report["returncode"],report["problem"])
                    if report["returncode"]==21 and matched["terminal"] is not True:
                        raise ValueError("System error closure without terminal")
                audits.append(matched)
            else:
                base.terminated(report["returncode"],report["problem"])
                summary=persist.audit(rows,role,ready,index>1,mode=="repair" and role=="system")
                if role=="system":
                    mutations=[]
                    if mode=="repair" and index==1:
                        mutations=expected.system_fault_operations(factory,candidate,"install")
                    elif mode=="repair" and index==3 and frozen!=restarted:
                        mutations=(expected.system_fault_operations(factory,candidate,"repair") if choice=="repair"
                            else [dict(operation="write",offset=0,length=512,payload_sha256=base.sha(restarted[:512])),
                                dict(operation="flush",offset=0,length=0)])
                    complete_mutations(rows,mutations)
                if report["returncode"]==21 and summary["transport_closed"] is not True:
                    raise ValueError("peer closure without complete EOF receipt")
                audits.append(summary)
            # Independent inode identity/capacity/RO binding comes from strict audit
            # and fixed preflight, then all three lifecycles must retain it.
            current.append((ready["device"],ready["inode"]))
        if len(set(current))!=3:raise ValueError("aliased System/Data/boot")
        bindings.append(current)
        if base.canonical(proof["audit"])!=base.canonical(audits):raise ValueError("recomputed audits differ")
        if proof["qmp_drained"] is not True:raise ValueError("missing post-reap QMP EOF")
        if fault:
            receipt=proof["fault"]
            if type(receipt) is not dict or set(receipt)!={"plan","request_index","event_index","offset","length","delivery"}:
                raise ValueError("fault receipt fields")
            for key in ("plan","request_index","event_index","offset","length"):
                if base.canonical(receipt[key])!=base.canonical(matched[key]):raise ValueError("fault identity mismatch")
            is_cut=wanted_plan["effect"] in ("before-cut","after-cut","torn-cut")
            allowed=({"code":20,"problem":"cut","eof":True},) if is_cut else (
                {"code":None,"problem":None,"eof":False},{"code":21,"problem":"backend-failed","eof":True})
            if base.canonical(receipt["delivery"]) not in tuple(base.canonical(x) for x in allowed):
                raise ValueError("exact delivery state")
            entry=cut["entry"]
            if (type(entry) is not dict or set(entry)!={"vm_code","backend_codes","backend_problems","event_count"} or
                entry["vm_code"] is not None or type(entry["backend_codes"]) is not list or len(entry["backend_codes"])!=3 or
                type(entry["backend_problems"]) is not list or len(entry["backend_problems"])!=3):
                raise ValueError("live teardown entry")
            if any(entry["backend_codes"][j] is not None or entry["backend_problems"][j] is not None for j in (0,2)):
                raise ValueError("unplanned peer failure")
            code,problem=entry["backend_codes"][1],entry["backend_problems"][1]
            if is_cut:
                if type(code) is not int or code!=20 or problem!="cut":raise ValueError("cut entry")
            elif (code is not None and (type(code) is not int or code!=21)) or problem not in (None,"backend-failed"):
                raise ValueError("error entry")
            fault_serial(proof["serial"],mode,matched)
            helper("fault_events").validate(proof["events"],proof["event_receipts"],proof["commands"],
                proof["qmp_drained"],verified["rtc_path"],entry["event_count"],wanted_plan,
                fault_role="system-"+mode)
        else:
            signed.transcript(proof["serial"],index,"repair-both" if mode=="repair" and index==1 else "bad-signature")
            base.checked_event_stream(proof["events"],proof["event_receipts"],proof["commands"],
                proof["qmp_drained"],verified["rtc_path"])
    if len(set(pids))!=3 or bindings[1:]!=[bindings[0],bindings[0]]:
        raise ValueError("distinct QEMU processes and same three retained image identities")
    frames=value["frames"]
    if type(frames) is not list or len(frames)!=len(names):raise ValueError("exact retained frame count")
    for frame,name in zip(frames,names):
        if type(frame) is not dict or set(frame)!={"scene","sha256","actual_ppm"} or frame["scene"]!=name:
            raise ValueError("frame fields/order")
        ppm=base.decoded(frame["actual_ppm"],len(visual.HEADER)+640*480*3)
        if name.startswith("home-"):digest=visual.validate(ppm,0)
        elif name.startswith("terminal-"):digest=visual.validate(ppm,1)
        elif name=="saved":digest=visual.validate(ppm,2,challenge)
        elif name=="files-3":digest=visual.validate(ppm,3,challenge)
        else:digest=expected.settings_validate(ppm,visual,True if name=="installed-1" else updated)
        if frame["sha256"]!=digest:raise ValueError("actual frame hash")
    return dict(mode=mode,case=case,plan=wanted_plan,choice=choice,
        frozen_system_sha256=base.sha(frozen),restarted_system_sha256=base.sha(restarted),
        data_sha256=base.sha(data),image_id=data[32:64].hex(),
        key_sha256=base.sha(data[64:96]),challenge=challenge,
        fresh_vms=3,frames=names,content_validated=True,provenance_validated=False,milestone_complete=False)
