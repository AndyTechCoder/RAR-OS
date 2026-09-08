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
        evidence["schema"]!="rar-modern-persistence-candidate-v0" or evidence["status"]!="observed" or
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
        if type(proof) is not dict or set(proof)!={"cut","audit","argv","preflight","commands","events","serial"}:
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
        if type(proof["events"]) is not list or len(proof["events"])!=1:
            raise ValueError("sole actual RESUME event")
        event=proof["events"][0]
        if type(event) is not dict or set(event)!={"event","timestamp"} or event["event"]!="RESUME":
            raise ValueError("unexpected VM event")
        stamp=event["timestamp"]
        if (type(stamp) is not dict or set(stamp)!={"seconds","microseconds"} or
            type(stamp["seconds"]) is not int or stamp["seconds"]<0 or
            type(stamp["microseconds"]) is not int or not 0<=stamp["microseconds"]<1000000):
            raise ValueError("canonical event timestamp")
        argv(proof["argv"],index,profile)
        # Source-specific preflight geometry is already recorded/checked in VM;
        # here recheck against independently tool-image-bound firmware sizes.
        preflight=proof["preflight"]
        if type(preflight) is not dict or set(preflight)!={"raw","verified"}:
            raise ValueError("actual paused preflight results")
        if profile.validate_preflight(preflight["raw"],index==2,index,firmware_sizes)!=preflight["verified"]:
            raise ValueError("retained topology does not revalidate")
        commands(proof["commands"],index,value,profile)
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
