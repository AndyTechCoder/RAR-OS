"""Independent retained paired-guest proof checks; pure bytes only."""
import json
import re
import importlib.util
from pathlib import Path

def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed regular trusted helper")
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m);return m

def commands(rows,index,peer,a,b,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:raise ValueError("bounded QMP transcript")
    plain=[]
    for n,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=n:
            raise ValueError("contiguous command identities")
        plain.append({k:v for k,v in row.items() if k!="id"})
    prefix=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(prefix)]!=prefix:raise ValueError("exact paused preflight then sole continue")
    keys,scenes=helper("expansion_visual").plan(peer,a,b)
    capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
    groups={at:0 for at in scenes};at=0
    for row in plain[len(prefix):]:
        if row==capture:
            if at not in groups:raise ValueError("unplanned capture boundary")
            groups[at]+=1
            if groups[at]>24:raise ValueError("capture budget")
        else:
            if at in groups and groups[at]==0:raise ValueError("missing prior causal frame")
            if at>=len(keys) or row!={"execute":"send-key","arguments":{
                "keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}}:
                raise ValueError("unplanned input; challenge cannot be injected into receiver")
            at+=1
    if at!=len(keys) or any(n==0 for n in groups.values()):raise ValueError("incomplete keyboard proof")

def parse(raw):
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024:raise ValueError("bounded paired evidence")
    def pairs(items):
        out={}
        for k,v in items:
            if k in out:raise ValueError("duplicate evidence key")
            out[k]=v
        return out
    value=json.loads(raw,object_pairs_hook=pairs)
    if helper("runtime_evidence").canonical(value)!=raw:raise ValueError("canonical paired evidence")
    return value

def validate(raw,boots,system,firmware_sizes):
    base=helper("runtime_evidence");visual=helper("expansion_visual");persist=helper("persistence")
    value=parse(raw)
    fields={"schema","status","challenges","frames","steps","vm_proofs","pair_cleanup",
            "initial_data","frozen_data","data_unchanged","system_sha256","boot_sha256",
            "elapsed_milliseconds","milestone_complete","captured_wire","wire_proof"}
    if (type(value) is not dict or set(value)!=fields or value["schema"]!="rar-expansion-pair-v1" or
        value["status"]!="observed" or value["milestone_complete"] is not False or
        value["data_unchanged"] is not True):
        raise ValueError("exact nonaccepting pair envelope")
    if type(boots) is not dict or set(boots)!={"a","b"} or any(base.digest(h)!=h for h in boots.values()):
        raise ValueError("trusted two boot hashes")
    if value["boot_sha256"]!=boots or value["system_sha256"]!=base.digest(system):
        raise ValueError("immutable build/System binding")
    if type(value["elapsed_milliseconds"]) is not int or not 0<value["elapsed_milliseconds"]<180000:
        raise ValueError("bounded scenario runtime")
    challenges=value["challenges"]
    if type(challenges) is not list or len(challenges)!=2:raise ValueError("two fresh challenges")
    a,b=challenges;helper("visual_oracle").value_check(a);helper("visual_oracle").value_check(b)
    if a==b:raise ValueError("distinct bidirectional challenges")
    captured=value["captured_wire"]
    if type(captured) is not dict or set(captured)!={"a","b"}:raise ValueError("two captured wires")
    wire={p:base.decoded(captured[p],204) for p in ("a","b")}
    checked_wire=helper("expansion_wire").validate(wire,a,b)
    if base.canonical(checked_wire)!=base.canonical(value["wire_proof"]):raise ValueError("independent wire proof mismatch")
    for name in ("initial_data","frozen_data"):
        if type(value[name]) is not list or len(value[name])!=2:raise ValueError("two Data images")
    initial=[base.decoded(x,99328) for x in value["initial_data"]]
    frozen=[base.decoded(x,99328) for x in value["frozen_data"]]
    if initial!=frozen or initial[0]==initial[1]:raise ValueError("distinct unchanged Data")
    for image in initial:
        state=helper("data_oracle").inspect(image)
        if state["revision"]!=0 or state["files"]!={} or any(image[1024:]):raise ValueError("virgin Data without seeded challenge")
    expected_steps=["a:queued","b:received","b:queued","a:received","a:closed","a:retired"]
    if value["steps"]!=expected_steps:raise ValueError("fixed cross-guest causal order")
    wanted=[("a","home",None),("a","terminal",None),("b","home",None),("b","terminal",None),
            ("a","queued",None),("b","received",a),("b","queued",None),("a","received",b),
            ("a","closed",None),("a","retired",None),("a","files",None)]
    frames=value["frames"]
    if type(frames) is not list or len(frames)!=len(wanted):raise ValueError("eleven actual scenes")
    for frame,(peer,stage,nonce) in zip(frames,wanted):
        if type(frame) is not dict or set(frame)!={"peer","stage","sha256","actual_ppm"} or frame["peer"]!=peer or frame["stage"]!=stage:
            raise ValueError("frame identity/order")
        pixels=base.decoded(frame["actual_ppm"],len(helper("visual_oracle").HEADER)+640*480*3)
        if frame["sha256"]!=visual.validate(pixels,stage,nonce):raise ValueError("actual pixel hash")
    proofs=value["vm_proofs"];cleanup=value["pair_cleanup"]
    if type(proofs) is not list or len(proofs)!=2:raise ValueError("exact two VM proofs")
    if type(cleanup) is not dict or set(cleanup)!={"joined","guests","network_closed"} or cleanup["joined"] is not True or cleanup["network_closed"] is not True:
        raise ValueError("complete aggregate teardown")
    if base.canonical(cleanup["guests"])!=base.canonical([p.get("cut") for p in proofs]):
        raise ValueError("bound pair/member receipts")
    identities=[];pids=[]
    for index,(peer,proof) in enumerate(zip(("a","b"),proofs),1):
        if type(proof) is not dict or set(proof)!={"peer","cut","audit","argv","preflight","commands","events","event_receipts","qmp_drained","serial"} or proof["peer"]!=peer:
            raise ValueError("exact peer proof")
        args=proof["argv"]
        if type(args) is not list or len(args)>128 or any(type(x) is not str for x in args) or args.count("-netdev")!=1:
            raise ValueError("exact private socket network argument")
        at=args.index("-netdev")
        if at+1>=len(args):raise ValueError("missing backend")
        match=re.fullmatch(r"socket,id=rar-net,fd=([1-9][0-9]{0,3})",args[at+1])
        if match is None:raise ValueError("inherited private backend only")
        profile=helper("expansion_profile").Profile(helper("vm_profile"),peer,int(match[1]))
        base.argv(args,index,profile,False)
        preflight=proof["preflight"]
        if type(preflight) is not dict or set(preflight)!={"raw","verified"}:raise ValueError("paused preflight receipt")
        verified=profile.validate_preflight(preflight["raw"],False,index,firmware_sizes)
        if base.canonical(verified)!=base.canonical(preflight["verified"]):raise ValueError("effective topology differs")
        commands(proof["commands"],index,peer,a,b,profile)
        base.checked_event_stream(proof["events"],proof["event_receipts"],proof["commands"],proof["qmp_drained"],verified["rtc_path"])
        serial=proof["serial"]
        if (type(serial) is not str or not serial.isascii() or len(serial)>65536 or serial.count("RAR-MODERN:GUI-READY")!=1 or
            any(x in serial for x in ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN","APP-FAULT="))):
            raise ValueError("guest readiness/fault transcript")
        cut=proof["cut"]
        if (type(cut) is not dict or set(cut)!={"vm_pid","vm_returncode","backends","joined"} or cut["joined"] is not True or
            type(cut["vm_pid"]) is not int or cut["vm_pid"]<=0 or not base.qemu_killed(cut["vm_returncode"]) or
            type(cut["backends"]) is not list or len(cut["backends"])!=3):
            raise ValueError("whole VM reap")
        pids.append(cut["vm_pid"]);audits=[]
        for role,backend in zip(("data","system","boot"),cut["backends"]):
            if type(backend) is not dict or set(backend)!={"returncode","problem","records","joined"} or backend["joined"] is not True:
                raise ValueError("three actual joined backends")
            base.terminated(backend["returncode"],backend["problem"])
            records=backend["records"]
            if type(records) is not list or not records:raise ValueError("backend records")
            checked=persist.audit(records,role,records[0])
            if checked["counts"]["write"] or checked["counts"]["flush"]:raise ValueError("no network-triggered storage mutations")
            audits.append(checked);identities.append((records[0]["device"],records[0]["inode"]))
        if base.canonical(audits)!=base.canonical(proof["audit"]):raise ValueError("storage audit mismatch")
    if len(set(pids))!=2 or len(set(identities))!=6:raise ValueError("independent guests and six storage domains")
    return dict(content_validated=True,guest_network_roundtrips=2,frames=11,data_unchanged=True,
                network_closed=True,captured_wire=checked_wire,milestone_complete=False)

def actual_refusals(raw,boots,system,sizes):
    """Mutate retained evidence only after the actual positive proof passes."""
    validate(raw,boots,system,sizes);original=parse(raw);count=0
    changes=[("status","failed"),("milestone_complete",True),("data_unchanged",False),
             ("elapsed_milliseconds",0),("elapsed_milliseconds",True),("steps",[]),
             ("challenges",["a"*32,"a"*32]),("system_sha256","0"*64),("boot_sha256",{}),
             ("captured_wire",{}),("wire_proof",{}),("frames",[]),("vm_proofs",[]),("pair_cleanup",{}),("frozen_data",[])]
    for key,replacement in changes:
        changed=dict(original);changed[key]=replacement
        try:validate(helper("runtime_evidence").canonical(changed),boots,system,sizes)
        except (ValueError,KeyError,TypeError):count+=1
        else:raise AssertionError("corrupt actual pair proof accepted: "+key)
    return count

def self_test():
    base=helper("vm_profile");profile=helper("expansion_profile").Profile(base,"a",19)
    a,b="a"*32,"b"*32;keys,scenes=helper("expansion_visual").plan("a",a,b)
    prefix=[{"execute":"qmp_capabilities"}]+[c for _,c in profile.preflight_requests()]+[{"execute":"cont"}]
    capture={"execute":"screendump","arguments":{"filename":profile.directory(1)+"/frame.ppm"}}
    rows=prefix+[capture]
    for at,key in enumerate(keys,1):
        rows.append({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":key}],"hold-time":50}})
        if at in scenes:rows.append(capture)
    rows=[dict(r,id=i) for i,r in enumerate(rows,1)]
    commands(rows,1,"a",a,b,profile)
    count=0
    def reject(fn):
        nonlocal count
        try:fn()
        except (ValueError,KeyError,TypeError):count+=1
        else:raise AssertionError("bad paired proof accepted")
    reject(lambda:commands(rows[:-1],1,"a",a,b,profile))
    reject(lambda:commands(rows,1,"a",b,a,profile))
    for i,row in enumerate(rows):
        changed=json.loads(json.dumps(rows));changed[i]["id"]=0
        reject(lambda changed=changed:commands(changed,1,"a",a,b,profile))
    for raw in (b"",b"{}",b"[]\n",b'{"x":1,"x":2}\n'):
        reject(lambda raw=raw:validate(raw,{"a":"a"*64,"b":"b"*64},"c"*64,(1966080,131072)))
    return count

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated retained-evidence self-test only")
    print("Expansion paired evidence:",self_test(),"negative fixtures; no activation")
