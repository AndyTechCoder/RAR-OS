"""Independent retained-byte validation for the Modern persistence scenario.
No VM, file mutation, subprocess, network or target execution. The trusted outer
controller separately binds provenance, source/tool digests and confinement.
"""
import base64
import hashlib
import importlib.util
import json
from pathlib import Path
import re

def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():
        raise ValueError("fixed immutable tool sibling")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def sha(data): return hashlib.sha256(data).hexdigest()

def digest(value):
    if type(value) is not str or re.fullmatch("[0-9a-f]{64}",value) is None:
        raise ValueError("canonical SHA256")
    return value

def decoded(value,size):
    if type(value) is not str or len(value)!=4*((size+2)//3):
        raise ValueError("bounded exact base64 length")
    try: raw=base64.b64decode(value,validate=True)
    except (ValueError,UnicodeError) as error: raise ValueError("invalid base64") from error
    if len(raw)!=size or base64.b64encode(raw).decode("ascii")!=value:
        raise ValueError("canonical exact base64 bytes")
    return raw

def commands(rows,index,value,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:
        raise ValueError("bounded actual QMP transcript")
    plain=[]
    for number,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=number:
            raise ValueError("contiguous QMP command identity")
        plain.append({k:v for k,v in row.items() if k!="id"})
    start=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(start)]!=start:
        raise ValueError("paused fixed preflight must precede sole continue")
    suffix=plain[len(start):]
    capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
    keys=helper("visual_oracle").plan(value)[index-1]
    actual=[]
    groups=[0]*(len(keys)+1)
    for row in suffix:
        if row==capture:
            groups[len(actual)]+=1
            if groups[len(actual)]>24:
                raise ValueError("bounded scene capture polling")
        else:
            if len(actual)>=len(keys):
                raise ValueError("extra command after fixed key plan")
            expected={"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":keys[len(actual)]}],"hold-time":50}}
            if row!=expected:
                raise ValueError("unplanned QMP command or challenge sent to second VM")
            actual.append(keys[len(actual)])
    if actual!=keys or groups[0]<1 or groups[-1]<1:
        raise ValueError("complete input and surrounding actual captures required")
    if index==1:
        if groups[1]<1 or any(groups[2:-1]):
            raise ValueError("Terminal capture must precede typed write; SAVED follows it")
    return len(rows)

def argv(rows,index,profile,readonly_data=None):
    if readonly_data is not None and type(readonly_data) is not bool:
        raise ValueError("explicit physical Data authority")
    readonly_data=index==2 if readonly_data is None else readonly_data
    if type(rows) is not list or not 1<=len(rows)<=128 or any(type(v) is not str or len(v)>2048 for v in rows):
        raise ValueError("bounded actual QEMU argument vector")
    sockets={}
    for at,value in enumerate(rows[:-1]):
        if value!="-blockdev": continue
        try: node=json.loads(rows[at+1])
        except ValueError as error: raise ValueError("malformed retained block argument") from error
        if type(node) is not dict: raise ValueError("block argument object")
        if node.get("driver")=="nbd":
            role=node.get("node-name")
            endpoint=node.get("server")
            if role in sockets or type(endpoint) is not dict or endpoint.get("type")!="fd":
                raise ValueError("unique private inherited NBD endpoint")
            number=endpoint.get("str")
            if type(number) is not str or re.fullmatch("[1-9][0-9]{0,3}",number) is None:
                raise ValueError("canonical inherited descriptor")
            sockets[role]=int(number)
    if set(sockets)!={"rar-data-nbd","rar-system-nbd","rar-boot-nbd"}:
        raise ValueError("exact three assigned socket roles")
    expected=profile.argv(index,sockets["rar-data-nbd"],sockets["rar-system-nbd"],sockets["rar-boot-nbd"],readonly_data)
    if rows!=expected: raise ValueError("actual QEMU arguments differ from the fixed reviewed profile")
    return True

def qemu_killed(code):
    return type(code) is int and code == -9

def terminated(code,problem):
    if type(code) is not int or (code,problem) not in ((-9,"backend-failed"),(21,"backend-failed")):
        raise ValueError("exact observed backend termination pairing")
    return True


def event_summary(events,phases=None):
    """Exact bounded public QMP names, not arbitrary event values in logs."""
    if type(events) is not list:return {"shape":"not-list"}
    result=[]
    for index,event in enumerate(events[:32]):
        item={"name":"INVALID"}
        if type(event) is dict:
            name=event.get("event")
            if type(name) is str and re.fullmatch("[A-Z][A-Z0-9_]{0,63}",name):
                item["name"]=name
            item["keys"]=[k if type(k) is str and re.fullmatch("[a-zA-Z_][a-zA-Z0-9_-]{0,63}",k)
                          else "INVALID" for k in list(event)[:16]]
            try:
                encoded=json.dumps(event,sort_keys=True,ensure_ascii=True,allow_nan=False).encode("ascii")
                if len(encoded)<=2048:item["sha256"]=sha(encoded)
                else:item["oversized"]=True
            except (ValueError,TypeError,RecursionError):
                item["malformed"]=True
            data=event.get("data")
            if type(data) is dict:
                item["data_types"]={
                    (key if type(key) is str and re.fullmatch("[a-zA-Z_][a-zA-Z0-9_-]{0,63}",key) else "INVALID"):
                    type(value).__name__ for key,value in list(data.items())[:16]}
        if type(phases) is list and index<len(phases):item["receipt"]=phases[index]
        result.append(item)
    return {"count":len(events),"events":result,"truncated":len(events)>32}

def receipt_phases(receipts,events,rows,drained):
    if (drained is not True or type(events) is not list or len(events)>32 or
        type(receipts) is not list or len(receipts)!=len(events) or
        type(rows) is not list or not 1<=len(rows)<=512):
        raise ValueError("complete bounded post-reap event receipt proof")
    for ordinal,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=ordinal:
            raise ValueError("receipt command identities")
    continuations=[r["id"] for r in rows if r.get("execute")=="cont"]
    if len(continuations)!=1:raise ValueError("sole continuation receipt boundary")
    cont=continuations[0]
    previous=0;postcut=False;phases=[]
    for index,receipt in enumerate(receipts):
        if (type(receipt) is not dict or set(receipt)!={"event_index","request_id"} or
            type(receipt["event_index"]) is not int or receipt["event_index"]!=index):
            raise ValueError("one contiguous receipt per retained event")
        identity=receipt["request_id"]
        if identity is None:
            postcut=True;phases.append("post-reap-stream")
        else:
            if (postcut or type(identity) is not int or not 1<=identity<=len(rows) or
                identity<previous):
                raise ValueError("ordered exact receipt request identity")
            previous=identity
            phases.append("preflight-reply" if identity<cont else
                          "continue-reply" if identity==cont else "running-reply")
    return phases

def checked_event_stream(events,receipts,rows,drained,rtc_path):
    """One lifecycle RESUME plus bounded advisory RTC changes, never ignored events.
    Receipt positions are stream observations, not event occurrence timestamps.
    QOM paths are validated object identifiers, never host filesystem paths.
    """
    phases=receipt_phases(receipts,events,rows,drained)
    if not 1<=len(events)<=5:
        raise ValueError("one RESUME and at most four RTC events")
    if type(rtc_path) is not str or not rtc_path.startswith("/machine/unattached/"):
        raise ValueError("independently verified RTC identity required")
    for index,event in enumerate(events):
        if (type(event) is not dict or
            phases[index] not in (("continue-reply","running-reply") if index==0 else ("running-reply",))):
            raise ValueError("event before sole continue or malformed object")
        name="RESUME" if index==0 else "RTC_CHANGE"
        fields={"event","timestamp"} if index==0 else {"event","timestamp","data"}
        if set(event)!=fields or event["event"]!=name:
            raise ValueError("unexpected lifecycle/device event: "+
                             json.dumps(event_summary(events,phases),sort_keys=True))
        stamp=event["timestamp"]
        if (type(stamp) is not dict or set(stamp)!={"seconds","microseconds"} or
            type(stamp["seconds"]) is not int or not 0<=stamp["seconds"]<1<<63 or
            type(stamp["microseconds"]) is not int or not 0<=stamp["microseconds"]<1000000):
            raise ValueError("canonical bounded event timestamp")
        if index:
            data=event["data"]
            if (type(data) is not dict or set(data)!={"offset","qom-path"} or
                type(data["offset"]) is not int or not -(1<<63)<=data["offset"]<1<<63):
                raise ValueError("exact RTC_CHANGE payload")
            path=data["qom-path"]
            if type(path) is not str or path!=rtc_path:
                raise ValueError("RTC event differs from paused chipset identity")
    return len(events)-1

def validate(raw,expected_boot_digest,firmware_sizes):
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024 or not raw.endswith(b"\n"):
        raise ValueError("bounded retained JSON line")
    def pairs(items):
        result={}
        for key,value in items:
            if key in result: raise ValueError("duplicate evidence property")
            result[key]=value
        return result
    try: evidence=json.loads(raw,object_pairs_hook=pairs)
    except (ValueError,UnicodeError,RecursionError) as error: raise ValueError("malformed evidence JSON") from error
    if json.dumps(evidence,separators=(",",":"),sort_keys=True,allow_nan=False).encode("ascii")+b"\n"!=raw:
        raise ValueError("canonical retained JSON encoding")
    fields={"schema","status","challenge","frames","vm_proofs","initial_data_sha256",
            "frozen_data_sha256","frozen_data_base64","system_sha256","boot_sha256",
            "crypto_interoperability_accepted","milestone_complete"}
    if (type(evidence) is not dict or set(evidence)!=fields or
        evidence["schema"]!="rar-modern-persistence-candidate-v1" or evidence["status"]!="observed" or
        evidence["crypto_interoperability_accepted"] is not False or evidence["milestone_complete"] is not False):
        raise ValueError("exact nonaccepting persistence envelope")
    if evidence["boot_sha256"]!=digest(expected_boot_digest):
        raise ValueError("actual package hash is not trusted-build bound")
    oracle=helper("visual_oracle")
    value=oracle.value_check(evidence["challenge"])
    data=decoded(evidence["frozen_data_base64"],99328)
    if digest(evidence["frozen_data_sha256"])!=sha(data):
        raise ValueError("frozen Data hash mismatch")
    inspected=helper("data_oracle").inspect(data)
    if (inspected["revision"]!=2 or inspected["files"]!={b"note":value.encode("ascii")} or
        inspected["committed_slots"]!=[0,1] or inspected["burned_slots"]!=[] or
        inspected["next_slot"]!=2 or inspected["readonly"]):
        raise ValueError("retained authenticated disk disagrees with challenge")
    if digest(evidence["initial_data_sha256"])!=sha(data[:1024]+bytes(99328-1024)):
        raise ValueError("virgin retained header/image binding")
    if digest(evidence["system_sha256"])!=sha(bytes(8388608)):
        raise ValueError("System changed during Data-only scenario")
    frames=evidence["frames"]
    if type(frames) is not list or len(frames)!=5:
        raise ValueError("exact five retained actual frames")
    for frame,index in zip(frames,(0,1,2,0,3)):
        if type(frame) is not dict or set(frame)!={"scene","sha256","actual_ppm"} or frame["scene"]!=oracle.SCENES[index]:
            raise ValueError("scene identity/order")
        pixels=decoded(frame["actual_ppm"],len(oracle.HEADER)+640*480*3)
        if digest(frame["sha256"])!=oracle.validate(pixels,index,value if index>=2 else None):
            raise ValueError("retained actual pixels differ from independent scene")
    proofs=evidence["vm_proofs"]
    if type(proofs) is not list or len(proofs)!=2:
        raise ValueError("exact two whole-VM proofs")
    profile=helper("vm_profile");persistence=helper("persistence")
    bindings=[]
    pids=[]
    for index,proof in enumerate(proofs,1):
        if type(proof) is not dict or set(proof)!={"cut","audit","argv","preflight","commands","events","event_receipts","qmp_drained","serial"}:
            raise ValueError("exact retained VM proof")
        cut=proof["cut"]
        if (type(cut) is not dict or set(cut)!={"vm_pid","vm_returncode","backends","joined"} or
            cut["joined"] is not True or type(cut["vm_pid"]) is not int or cut["vm_pid"]<=0 or
            not qemu_killed(cut["vm_returncode"]) or type(cut["backends"]) is not list or len(cut["backends"])!=3):
            raise ValueError("whole QEMU killed before three backend joins")
        pids.append(cut["vm_pid"])
        if type(proof["serial"]) is not str or not proof["serial"].isascii() or len(proof["serial"])>65536 or "RAR-MODERN:GUI-READY" not in proof["serial"]:
            raise ValueError("bounded actual Modern readiness transcript")
        if any(marker in proof["serial"] for marker in ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN")):
            raise ValueError("guest failure in retained transcript")
        commands(proof["commands"],index,value,profile)
        argv(proof["argv"],index,profile)
        # Source-specific preflight geometry is already recorded/checked in VM;
        # here recheck against independently tool-image-bound firmware sizes.
        preflight=proof["preflight"]
        if type(preflight) is not dict or set(preflight)!={"raw","verified"}:
            raise ValueError("actual paused preflight results")
        verified=profile.validate_preflight(preflight["raw"],index==2,index,firmware_sizes)
        if verified!=preflight["verified"]:
            raise ValueError("retained topology does not revalidate")
        checked_event_stream(proof["events"],proof["event_receipts"],
                             proof["commands"],proof["qmp_drained"],verified["rtc_path"])
        summaries=[];current=[]
        for role,report in zip(("data","system","boot"),cut["backends"]):
            if (type(report) is not dict or set(report)!={"returncode","problem","records","joined"} or
                report["joined"] is not True):
                raise ValueError("bounded baseline backend termination")
            terminated(report["returncode"],report["problem"])
            records=report["records"]
            if type(records) is not list or not records: raise ValueError("actual backend records")
            ready=records[0]
            summaries.append(persistence.audit(records,role,ready,index==2))
            current.append((ready["device"],ready["inode"],ready["capacity"]))
        if summaries!=proof["audit"] or len({(device,inode) for device,inode,_ in current})!=3:
            raise ValueError("independent role audit or disk separation mismatch")
        bindings.append(current)
    if pids[0]==pids[1] or bindings[0]!=bindings[1]:
        raise ValueError("fresh QEMU must reopen the same independent retained disk inodes")
    return {"frames":5,"fresh_vms":2,"data_sha256":sha(data),"boot_sha256":expected_boot_digest,
            "content_validated":True,"provenance_validated":False,"milestone_complete":False}


def canonical(value):
    return json.dumps(value,separators=(",",":"),sort_keys=True,allow_nan=False).encode("ascii")+b"\n"

def refusal_cases():
    """Fixed paths/operations, never instructions supplied by a captured guest."""
    cases=[]
    def add(name,path,operation,value=None):
        cases.append((name,tuple(path),operation,value))
    for field in ("schema","status","boot_sha256","initial_data_sha256",
                  "frozen_data_sha256","system_sha256","challenge"):
        add("envelope-"+field,(field,),"flip")
    for field in ("milestone_complete","crypto_interoperability_accepted"):
        add("claim-"+field,(field,),"set",True)
    for index in range(5):
        add("frame-%d-hash"%index,("frames",index,"sha256"),"flip")
    add("frame-pixels-rehashed",("frames",4),"pixels")
    add("data-bytes-rehashed",(),"data")
    for vm in range(2):
        root=("vm_proofs",vm)
        for label,path,operation,value in (
            ("not-joined",("cut","joined"),"set",False),
            ("return-code",("cut","vm_returncode"),"set",0),
            ("float-return-code",("cut","vm_returncode"),"set",-9.0),
            ("no-qmp-eof",("qmp_drained",),"set",False),
            ("invalid-command-id",("commands",0,"id"),"set",0),
            ("unexpected-event",("events",0,"event"),"set","RESET"),
            ("invalid-receipt",("event_receipts",0,"event_index"),"set",1),
            ("post-reap-resume",("event_receipts",0,"request_id"),"set",None),
            ("rtc-inventory",("preflight","raw","rtc-children"),"set",[]),
            ("verified-rtc",("preflight","verified","rtc_path"),"set","/machine/other"),
            ("guest-panic",("serial",),"set","RAR-MODERN:GUI-READY\nRAR-PANIC"),
            ("extra-argv",("argv",),"append","-snapshot")):
            add("vm%d-%s"%(vm+1,label),root+path,operation,value)
        for role in range(3):
            backend=root+("cut","backends",role)
            add("vm%d-role%d-unjoined"%(vm+1,role),backend+("joined",),"set",False)
            add("vm%d-role%d-capacity"%(vm+1,role),backend+("records",0,"capacity"),"increment")
            add("vm%d-role%d-authority"%(vm+1,role),backend+("records",0,"readonly"),"invert")
    add("reused-vm-pid",("vm_proofs",1,"cut","vm_pid"),"copy",("vm_proofs",0,"cut","vm_pid"))
    add("changed-data-inode",("vm_proofs",1,"cut","backends",0,"records",0,"inode"),"increment")
    return cases

def apply_refusal(document,case):
    _,path,operation,value=case
    def at(parts):
        result=document
        for part in parts:result=result[part]
        return result
    if operation=="data":
        data=bytearray(decoded(document["frozen_data_base64"],99328))
        # Alter the retained header and recompute its outer hash. Authentication
        # and content binding, not just the outer SHA, must still reject it.
        data[0]^=1
        document["frozen_data_base64"]=base64.b64encode(data).decode("ascii")
        document["frozen_data_sha256"]=sha(data)
        return
    target=at(path[:-1]);key=path[-1];old=target[key]
    if operation=="set":target[key]=value
    elif operation=="flip":
        if type(old) is not str or not old:raise AssertionError("nonempty captured string")
        target[key]=("b" if old[0]=="a" else "a")+old[1:]
    elif operation=="increment":
        if type(old) is not int:raise AssertionError("captured integer")
        target[key]=old+1
    elif operation=="invert":
        if type(old) is not bool:raise AssertionError("captured boolean")
        target[key]=not old
    elif operation=="copy":target[key]=at(value)
    elif operation=="append":
        if type(old) is not list:raise AssertionError("captured list")
        old.append(value)
    elif operation=="pixels":
        pixels=bytearray(base64.b64decode(old["actual_ppm"],validate=True))
        pixels[-1]^=1
        old["actual_ppm"]=base64.b64encode(pixels).decode("ascii")
        old["sha256"]=sha(pixels)
    else:raise AssertionError("unknown fixed refusal operation")

def actual_refusals(raw,expected_boot_digest,firmware_sizes):
    """Recheck altered copies of one real positive capture; no VM/disk mutation."""
    import time
    positive=validate(raw,expected_boot_digest,firmware_sizes)
    cases=refusal_cases()
    names=[case[0] for case in cases]
    if len(cases)!=60 or len(set(names))!=len(names):
        raise AssertionError("fixed unique actual-evidence refusal matrix")
    deadline=time.monotonic()+300
    results=[]
    for case in cases:
        if time.monotonic()>deadline:raise TimeoutError("bounded evidence refusal matrix")
        document=json.loads(raw)
        apply_refusal(document,case)
        changed=canonical(document)
        if changed==raw:raise AssertionError("refusal did not change retained bytes")
        try:validate(changed,expected_boot_digest,firmware_sizes)
        except ValueError as error:
            results.append({"case":case[0],"input_sha256":sha(changed),
                            "rejection_sha256":sha(str(error).encode("utf-8"))})
        except Exception as error:
            raise RuntimeError("refusal checker crashed: "+case[0]) from error
        else:raise AssertionError("altered actual evidence accepted: "+case[0])
        # TypeError/KeyError/assertions/timeouts are test failures, not refusals.
    if time.monotonic()>deadline:raise TimeoutError("bounded evidence refusal matrix")
    if validate(raw,expected_boot_digest,firmware_sizes)!=positive:
        raise AssertionError("positive capture changed during refusal validation")
    return {"schema":"rar-modern-actual-refusals-v1","base_sha256":sha(raw),
            "rejected":len(results),"cases":results,"disk_faults_tested":False,
            "milestone_complete":False}


def unavailable_plan():
    keys=["f3"];captures=[(0,"home",None,False),(1,"terminal",None,False)]
    for index,command in enumerate(("list","write note denied","list")):
        keys.extend("spc" if ch==" " else ch for ch in command)
        captures.append((len(keys),"pending",command,index==0))
        keys.append("ret");captures.append((len(keys),"unavailable",None,False))
    keys.extend(("esc","f1"));captures.append((len(keys),"files",None,False))
    return keys,captures

def unavailable_commands(rows,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:raise ValueError("bounded negative QMP")
    plain=[]
    for index,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=index:
            raise ValueError("negative QMP identity")
        plain.append({key:value for key,value in row.items() if key!="id"})
    start=[{"execute":"qmp_capabilities"}]+[command for _,command in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(start)]!=start:raise ValueError("negative paused preflight first")
    keys,captures=unavailable_plan();positions={item[0] for item in captures}
    groups={position:0 for position in positions};at=0
    frame={"execute":"screendump","arguments":{"filename":profile.directory(1)+"/frame.ppm"}}
    for row in plain[len(start):]:
        if row==frame:
            if at not in groups:raise ValueError("negative capture at wrong input boundary")
            groups[at]+=1
            if groups[at]>24:raise ValueError("negative capture polling budget")
        else:
            if at>=len(keys):raise ValueError("extra negative input")
            wanted={"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}}
            if row!=wanted:raise ValueError("unplanned negative command")
            # Observe the previous scene before advancing past its boundary.
            if at in groups and groups[at]==0:raise ValueError("missing causal prior frame")
            at+=1
    if at!=len(keys) or any(count==0 for count in groups.values()):
        raise ValueError("incomplete negative input/captures")

def validate_unavailable(raw,expected_boot_digest,firmware_sizes):
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024:raise ValueError("bounded negative envelope")
    def pairs(items):
        value={}
        for key,item in items:
            if key in value:raise ValueError("duplicate negative key")
            value[key]=item
        return value
    evidence=json.loads(raw,object_pairs_hook=pairs)
    if canonical(evidence)!=raw:raise ValueError("canonical negative envelope")
    fields={"schema","status","original_data_base64","initial_data_base64","frozen_data_base64",
        "data_sha256","system_sha256","boot_sha256","frames","vm_proof","milestone_complete"}
    if (type(evidence) is not dict or set(evidence)!=fields or
        evidence["schema"]!="rar-modern-unavailable-candidate-v1" or evidence["status"]!="observed" or
        evidence["milestone_complete"] is not False):raise ValueError("exact negative envelope")
    if evidence["boot_sha256"]!=digest(expected_boot_digest):raise ValueError("negative build binding")
    original=decoded(evidence["original_data_base64"],99328)
    initial=decoded(evidence["initial_data_base64"],99328)
    frozen=decoded(evidence["frozen_data_base64"],99328)
    state=helper("data_oracle").inspect(original)
    if state["revision"]!=0 or state["files"]!={} or any(original[1024:]):
        raise ValueError("negative original must be virgin valid Data")
    expected=bytes([original[0]^1])+original[1:512]+bytes([original[512]^1])+original[513:]
    if initial!=expected or frozen!=initial or digest(evidence["data_sha256"])!=sha(frozen):
        raise ValueError("negative exact corruption and unchanged Data")
    try:helper("data_oracle").inspect(initial)
    except ValueError:pass
    else:raise ValueError("negative headers must be invalid")
    if digest(evidence["system_sha256"])!=sha(bytes(8388608)):raise ValueError("negative System changed")
    frames=evidence["frames"];oracle=helper("visual_oracle");_,capture_plan=unavailable_plan()
    if type(frames) is not list or len(frames)!=len(capture_plan):raise ValueError("nine actual negative scenes")
    for frame,(_,stage,command,first) in zip(frames,capture_plan):
        if (type(frame) is not dict or set(frame)!={"stage","command","first","sha256","actual_ppm"} or
            frame["stage"]!=stage or frame["command"]!=command or frame["first"] is not first):
            raise ValueError("negative scene order and identity")
        pixels=decoded(frame["actual_ppm"],len(oracle.HEADER)+640*480*3)
        if digest(frame["sha256"])!=oracle.unavailable_validate(pixels,stage,command,first):
            raise ValueError("negative pixel binding")
    proof=evidence["vm_proof"]
    if type(proof) is not dict or set(proof)!={"cut","audit","argv","preflight","commands","events","event_receipts","qmp_drained","serial"}:
        raise ValueError("exact negative VM proof")
    cut=proof["cut"]
    if (type(cut) is not dict or set(cut)!={"vm_pid","vm_returncode","backends","joined"} or
        cut["joined"] is not True or type(cut["vm_pid"]) is not int or cut["vm_pid"]<=0 or
        not qemu_killed(cut["vm_returncode"]) or type(cut["backends"]) is not list or len(cut["backends"])!=3):
        raise ValueError("negative whole VM termination")
    serial=proof["serial"]
    if (type(serial) is not str or not serial.isascii() or len(serial)>65536 or
        "RAR-MODERN:GUI-READY" not in serial or any(x in serial for x in
        ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN"))):
        raise ValueError("negative actual guest readiness")
    profile=helper("vm_profile");persistence=helper("persistence")
    unavailable_commands(proof["commands"],profile);argv(proof["argv"],1,profile)
    preflight=proof["preflight"]
    if type(preflight) is not dict or set(preflight)!={"raw","verified"}:raise ValueError("negative paused preflight")
    verified=profile.validate_preflight(preflight["raw"],False,1,firmware_sizes)
    if canonical(verified)!=canonical(preflight["verified"]):raise ValueError("negative topology")
    checked_event_stream(proof["events"],proof["event_receipts"],proof["commands"],proof["qmp_drained"],verified["rtc_path"])
    summaries=[];identities=[]
    for role,backend in zip(("data","system","boot"),cut["backends"]):
        if type(backend) is not dict or set(backend)!={"returncode","problem","records","joined"} or backend["joined"] is not True:
            raise ValueError("negative joined backend")
        terminated(backend["returncode"],backend["problem"])
        records=backend["records"]
        if type(records) is not list or not records:raise ValueError("negative backend records")
        summary=persistence.audit(records,role,records[0],False);summaries.append(summary)
        identities.append((records[0]["device"],records[0]["inode"],records[0]["capacity"]))
        requests=[r for r in records if r["type"]=="request"]
        if any(r["operation"]!="read" for r in requests):raise ValueError("negative mutation attempt")
        if role=="data":
            offsets=[r["offset"] for r in requests]
            # Two guest header reads plus at most the known firmware sector0
            # probe. Repeated mounts or a slot scan cannot hide in this bound.
            if (summary["trailing_read"] or len(offsets) not in (2,3) or
                offsets.count(512)!=1 or offsets.count(0)!=len(offsets)-1 or
                any(r["length"]!=512 for r in requests) or
                summary["counts"]!={"read":len(offsets),"write":0,"flush":0}):
                raise ValueError("negative single bounded header mount, no retry")
    if len({(device,inode) for device,inode,_ in identities})!=3 or canonical(summaries)!=canonical(proof["audit"]):
        raise ValueError("negative independent storage domains")
    return dict(schema="rar-modern-unavailable-result-v1",frames=9,fresh_vms=1,
        unchanged_data_sha256=sha(frozen),header_reads=summaries[0]["counts"]["read"],
        mutation_attempts=0,content_validated=True,provenance_validated=False,milestone_complete=False)

def self_test():
    cases=refusal_cases()
    assert len(cases)==60 and len({row[0] for row in cases})==60
    sample={"rows":[{"number":3,"text":"a","flag":False,"items":[]}],"source":9}
    for op,key,value,expected in (("increment","number",None,4),("flip","text",None,"b"),
                                  ("invert","flag",None,True),("append","items","x",["x"]),
                                  ("set","number",0,0),("copy","number",("source",),9)):
        changed=json.loads(json.dumps(sample))
        apply_refusal(changed,("unit",("rows",0,key),op,value))
        assert changed["rows"][0][key]==expected
        assert sample=={"rows":[{"number":3,"text":"a","flag":False,"items":[]}],"source":9}
    assert canonical({"value":-9})!=canonical({"value":-9.0})
    assert qemu_killed(-9)
    for code in (-9.0,True,False,0,9,"-9",None):
        assert not qemu_killed(code)

    rejected=0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except (ValueError,TypeError): rejected+=1
        else: raise AssertionError("malformed retained evidence accepted")
    for raw in (b"",b"{}\n",b"[]\n",b'{"x":1,"x":2}\n',b"["*2000+b"\n"):
        reject(lambda raw=raw:validate(raw,"a"*64,(1966080,131072)))
    for value in ("a"*63,"A"*64,None):
        reject(lambda value=value:digest(value))
    for value in ("", "AAAA", "!!!!"):
        reject(lambda value=value:decoded(value,1))
    assert decoded("YQ==",1)==b"a"
    for code in (-9,21):
        assert terminated(code,"backend-failed")
        reject(lambda code=code:terminated(code,None))
    reject(lambda:terminated(0,"backend-failed"))
    assert event_summary(None)=={"shape":"not-list"}
    assert event_summary([])=={"count":0,"events":[],"truncated":False}
    names=event_summary([{"event":"RESUME"},{"event":"RTC_CHANGE","data":{"offset":123}},
                         {"event":"PRIVATE\nTEXT","data":{"message":"secret"}}])
    assert [x["name"] for x in names["events"]]==["RESUME","RTC_CHANGE","INVALID"]
    assert "secret" not in json.dumps(names) and "123" not in json.dumps(names["events"][1]["data_types"])
    assert event_summary([{"event":"RESUME"}]*33)["truncated"] is True
    assert len(event_summary([{"event":"RESUME"}]*33)["events"])==32
    rows=[{"execute":name,"id":i+1} for i,name in enumerate(
        ("qmp_capabilities","query-status","cont","screendump"))]
    events=[{"event":"RESUME"}]*4
    receipts=[{"event_index":i,"request_id":n} for i,n in enumerate((1,3,4,None))]
    assert receipt_phases(receipts,events,rows,True)==[
        "preflight-reply","continue-reply","running-reply","post-reap-stream"]
    for flag in (False,1,None):reject(lambda flag=flag:receipt_phases(receipts,events,rows,flag))
    reject(lambda:receipt_phases(receipts[:-1],events,rows,True))
    for field,value in (("event_index",True),("event_index",2),("request_id",0),
                        ("request_id",5),("request_id",True)):
        changed=json.loads(json.dumps(receipts));changed[0][field]=value
        reject(lambda changed=changed:receipt_phases(changed,events,rows,True))
    for requests in ((3,1,4,None),(1,None,4,None)):
        changed=[{"event_index":i,"request_id":n} for i,n in enumerate(requests)]
        reject(lambda changed=changed:receipt_phases(changed,events,rows,True))
    reject(lambda:receipt_phases(receipts,events,rows+[{"execute":"cont","id":5}],True))

    # QMP RTC_CHANGE is advisory clock metadata, not a reset or another boot.
    stamp={"seconds":1,"microseconds":2}
    resume={"event":"RESUME","timestamp":stamp}
    rtc={"event":"RTC_CHANGE","timestamp":stamp,
         "data":{"offset":-1,"qom-path":"/machine/unattached/device[7]"}}
    def stream(sequence,identities=None,drained=True):
        if identities is None:identities=[3]+[4]*(len(sequence)-1)
        rs=[{"event_index":i,"request_id":n} for i,n in enumerate(identities)]
        return checked_event_stream(sequence,rs,rows,drained,"/machine/unattached/device[7]")
    assert stream([resume])==0
    assert stream([resume,rtc,rtc])==2
    assert stream([resume]+[rtc]*4)==4
    reject(lambda:stream([resume,rtc],(3,None)))
    reject(lambda:stream([resume],(None,)))
    for offset in (-(1<<63),0,(1<<63)-1):
        changed=json.loads(json.dumps(rtc));changed["data"]["offset"]=offset
        assert stream([resume,changed])==1
    for sequence in ([],[rtc],[rtc,resume],[resume,resume],[resume]+[rtc]*5):
        reject(lambda sequence=sequence:stream(sequence))
    for name in ("RESET","STOP","SHUTDOWN","POWERDOWN","SUSPEND","SUSPEND_DISK","WAKEUP",
                 "WATCHDOG","GUEST_PANICKED","BLOCK_IO_ERROR","DEVICE_DELETED","UNKNOWN"):
        changed=dict(rtc,event=name)
        reject(lambda changed=changed:stream([resume,changed]))
    reject(lambda:stream([resume,rtc],(1,4)))
    reject(lambda:stream([resume,rtc],(3,2)))
    reject(lambda:stream([resume,rtc],(3,3)))
    assert stream([resume,rtc],(4,4))==1
    reject(lambda:stream([resume,rtc],(None,4)))
    for flag in (False,1,None):reject(lambda flag=flag:stream([resume],drained=flag))
    for value in (True,1.0,"0",-(1<<63)-1,1<<63):
        changed=json.loads(json.dumps(rtc));changed["data"]["offset"]=value
        reject(lambda changed=changed:stream([resume,changed]))
    for path in ("","/machine","/host/rtc","/machine/../rtc","/machine/./rtc",
                 "/machine//rtc","/machine/rtc/","/machine/rtc\nprivate",
                 "/machine/"+("a"*65),"/machine/"+"/".join(["r"]*9),
                 "/machine/device[x]","/machine/rtc;command","/machine/clocké",None,1):
        changed=json.loads(json.dumps(rtc));changed["data"]["qom-path"]=path
        reject(lambda changed=changed:stream([resume,changed]))
    changed=json.loads(json.dumps(rtc));changed["data"]["qom-path"]="/machine/peripheral/rtc"
    reject(lambda:stream([resume,rtc,changed]))
    for source in (resume,rtc):
        for field,value in (("seconds",-1),("seconds",True),("seconds",1<<63),
                            ("microseconds",-1),("microseconds",1000000),("microseconds",True)):
            changed=json.loads(json.dumps(source));changed["timestamp"][field]=value
            sequence=[changed] if source is resume else [resume,changed]
            reject(lambda sequence=sequence:stream(sequence))
        for field in source:
            changed=json.loads(json.dumps(source));del changed[field]
            sequence=[changed] if source is resume else [resume,changed]
            reject(lambda sequence=sequence:stream(sequence))
        changed=dict(source,extra=0)
        sequence=[changed] if source is resume else [resume,changed]
        reject(lambda sequence=sequence:stream(sequence))
    for data in (None,{},{"offset":0},{"qom-path":"/machine/rtc"},
                 {"offset":0,"qom-path":"/machine/rtc","extra":0}):
        reject(lambda data=data:stream([resume,dict(rtc,data=data)]))
    reject(lambda:stream([dict(resume,data={})]))

    profile=helper("vm_profile");visual=helper("visual_oracle")
    value="abcdefghijklmnop"*2
    for index in (1,2):
        assert argv(profile.argv(index,7,8,9,index==2),index,profile)
        reject(lambda index=index:argv(profile.argv(index,7,8,9,index==2)+["-device","host"],index,profile))
        prefix=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
        capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
        suffix=[capture]
        for at,key in enumerate(visual.plan(value)[index-1]):
            suffix.append({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":key}],"hold-time":50}})
            if index==1 and at==0: suffix.append(capture)
        suffix.append(capture)
        rows=[dict(row,id=i) for i,row in enumerate(prefix+suffix,1)]
        assert commands(rows,index,value,profile)==len(rows)
        reject(lambda rows=rows,index=index:commands(rows[:-1],index,value,profile))
        changed=json.loads(json.dumps(rows));changed[-1]["arguments"]["filename"]="/outside"
        reject(lambda changed=changed,index=index:commands(changed,index,value,profile))
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("pure retained-evidence self-test only")
    print("Modern retained evidence:",self_test(),"framing refusals; no VM or acceptance")
