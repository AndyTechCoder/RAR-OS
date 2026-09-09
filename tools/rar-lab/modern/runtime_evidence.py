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

def argv(rows,index,profile):
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
    expected=profile.argv(index,sockets["rar-data-nbd"],sockets["rar-system-nbd"],sockets["rar-boot-nbd"],index==2)
    if rows!=expected: raise ValueError("actual QEMU arguments differ from the fixed reviewed profile")
    return True

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

def checked_event_stream(events,receipts,rows,drained):
    """One lifecycle RESUME plus bounded advisory RTC changes, never ignored events.
    Receipt positions are stream observations, not event occurrence timestamps.
    QOM paths are validated object identifiers, never host filesystem paths.
    """
    phases=receipt_phases(receipts,events,rows,drained)
    if not events:
        raise ValueError("missing actual RESUME")
    rtc_path=None
    for index,event in enumerate(events):
        if (type(event) is not dict or
            phases[index]=="preflight-reply"):
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
            if (type(path) is not str or not 1<=len(path)<=255 or
                re.fullmatch(r"/machine(?:/[A-Za-z0-9_.-]+(?:\[[0-9]+\])?){1,8}",path) is None or
                any(part in (".","..") or len(part)>64 for part in path.split("/")[1:])):
                raise ValueError("bounded canonical RTC object identifier")
            if rtc_path is not None and rtc_path!=path:
                raise ValueError("multiple RTC object identities")
            rtc_path=path
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
            cut["vm_returncode"]!=-9 or type(cut["backends"]) is not list or len(cut["backends"])!=3):
            raise ValueError("whole QEMU killed before three backend joins")
        pids.append(cut["vm_pid"])
        if type(proof["serial"]) is not str or not proof["serial"].isascii() or len(proof["serial"])>65536 or "RAR-MODERN:GUI-READY" not in proof["serial"]:
            raise ValueError("bounded actual Modern readiness transcript")
        if any(marker in proof["serial"] for marker in ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN")):
            raise ValueError("guest failure in retained transcript")
        commands(proof["commands"],index,value,profile)
        checked_event_stream(proof["events"],proof["event_receipts"],
                             proof["commands"],proof["qmp_drained"])
        argv(proof["argv"],index,profile)
        # Source-specific preflight geometry is already recorded/checked in VM;
        # here recheck against independently tool-image-bound firmware sizes.
        preflight=proof["preflight"]
        if type(preflight) is not dict or set(preflight)!={"raw","verified"}:
            raise ValueError("actual paused preflight results")
        if profile.validate_preflight(preflight["raw"],index==2,index,firmware_sizes)!=preflight["verified"]:
            raise ValueError("retained topology does not revalidate")
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
        if summaries!=proof["audit"] or len(set(current))!=3:
            raise ValueError("independent role audit or disk separation mismatch")
        bindings.append(current)
    if pids[0]==pids[1] or bindings[0]!=bindings[1]:
        raise ValueError("fresh QEMU must reopen the same independent retained disk inodes")
    return {"frames":5,"fresh_vms":2,"data_sha256":sha(data),"boot_sha256":expected_boot_digest,
            "content_validated":True,"provenance_validated":False,"milestone_complete":False}

def self_test():
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
        return checked_event_stream(sequence,rs,rows,drained)
    assert stream([resume])==0
    assert stream([resume,rtc,rtc])==2
    assert stream([resume]+[rtc]*31)==31
    assert stream([resume,rtc],(3,None))==1
    assert stream([resume],(None,))==0
    for offset in (-(1<<63),0,(1<<63)-1):
        changed=json.loads(json.dumps(rtc));changed["data"]["offset"]=offset
        assert stream([resume,changed])==1
    for sequence in ([],[rtc],[rtc,resume],[resume,resume],[resume]+[rtc]*32):
        reject(lambda sequence=sequence:stream(sequence))
    for name in ("RESET","STOP","SHUTDOWN","SUSPEND","SUSPEND_DISK","WAKEUP",
                 "WATCHDOG","GUEST_PANICKED","BLOCK_IO_ERROR","DEVICE_DELETED","UNKNOWN"):
        changed=dict(rtc,event=name)
        reject(lambda changed=changed:stream([resume,changed]))
    reject(lambda:stream([resume,rtc],(1,4)))
    reject(lambda:stream([resume,rtc],(3,2)))
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
