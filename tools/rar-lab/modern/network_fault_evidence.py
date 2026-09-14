"""Independent actual closed-peer negative-campaign proof. Pure bytes only."""
import base64
import re
import importlib.util
from pathlib import Path
def helper(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def expected_wire(mode,challenges):
    wire=helper("expansion_wire");left,right=challenges
    if mode!="faults":return []
    original=[wire.packet("b",left.encode(),i) for i in range(5)]
    malformed=bytearray(original[0]);malformed[14]=0x65
    checksum=bytearray(original[2]);checksum[40]^=1
    return [bytes(malformed),original[1][:60],bytes(checksum),original[4]]+[
        wire.packet("b",right.encode(),i) for i in range(5,11)]
def validate(raw,boots,system,firmware_sizes,mode):
    base=helper("runtime_evidence");persist=helper("persistence")
    value=helper("expansion_evidence").parse(raw)
    if type(value) is dict and value.get("schema")=="rar-network-campaign-failure-v1" and value.get("status")=="failed":
        # Public synthetic cloud diagnostics only; still an unconditional refusal.
        # Escape and cap before exposing guest-derived serial in the job log.
        detail=base.canonical(value)[:24576].decode("ascii",errors="replace")
        raise ValueError("actual cloud scenario failed: "+detail)
    fields={"schema","mode","status","milestone_complete","challenges","frames","vm_proofs","pair_cleanup",
            "initial_data","frozen_data","captured_wire","boot_sha256","system_sha256","elapsed_milliseconds"}
    if (type(value) is not dict or set(value)!=fields or value["schema"]!="rar-network-campaign-v1" or
        value["mode"]!=mode or value["status"]!="observed" or value["milestone_complete"] is not False):raise ValueError("nonaccepting campaign envelope")
    challenges=value["challenges"];steps=helper("network_fault_plan").plan(mode,challenges)
    if type(boots) is not dict or set(boots)!={"a","b"} or any(base.digest(h)!=h for h in boots.values()):raise ValueError("two boot identities")
    if value["boot_sha256"]!=boots or value["system_sha256"]!=base.digest(system):raise ValueError("build binding")
    if type(value["elapsed_milliseconds"]) is not int or not 0<value["elapsed_milliseconds"]<180000:raise ValueError("bounded duration")
    for key in ("initial_data","frozen_data"):
        if type(value[key]) is not list or len(value[key])!=2:raise ValueError("two Data domains")
    initial=[base.decoded(v,99328) for v in value["initial_data"]]
    frozen=[base.decoded(v,99328) for v in value["frozen_data"]]
    if initial!=frozen or initial[0]==initial[1]:raise ValueError("independent unchanged Data")
    for data in initial:
        state=helper("data_oracle").inspect(data)
        if state["revision"]!=0 or state["files"]!={} or any(data[1024:]):raise ValueError("virgin Data")
    captured=value["captured_wire"]
    if type(captured) is not dict or set(captured)!={"a","b"}:raise ValueError("two captures")
    wanted=expected_wire(mode,challenges);length=24+sum(16+len(p) for p in wanted)
    for peer in ("a","b"):
        wire=base.decoded(captured[peer],length)
        if helper("expansion_wire").parse(wire)!=wanted:raise ValueError("actual malformed/drop/flood wire differs")
    scenes=[s for s in steps if s[2]!="peer-stopped"];frames=value["frames"]
    if type(frames) is not list or len(frames)!=len(scenes):raise ValueError("complete scene sequence")
    for frame,(peer,keys,stage,nonce) in zip(frames,scenes):
        if type(frame) is not dict or set(frame)!={"peer","stage","compact","sha256","actual_ppm"} or frame["peer"]!=peer or frame["stage"]!=stage or frame["compact"] is not False:raise ValueError("scene identity")
        pixels=base.decoded(frame["actual_ppm"],len(helper("visual_oracle").HEADER)+640*480*3)
        if frame["sha256"]!=helper("network_fault_plan").validate(pixels,stage,nonce):raise ValueError("actual full pixels")
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
        helper("network_fault_plan").commands(proof["commands"],index,peer,mode,challenges,profile)
        base.checked_event_stream(proof["events"],proof["event_receipts"],proof["commands"],proof["qmp_drained"],verified["rtc_path"])
        serial=proof["serial"]
        if (type(serial) is not str or not serial.isascii() or len(serial)>65536 or serial.count("RAR-MODERN:GUI-READY")!=1 or
            any(x in serial for x in ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN"))):
            raise ValueError("guest readiness/fault transcript")
        if serial.count("RAR-MODERN:APP-FAULT=6")!=(1 if mode=="peer-stop" and peer=="b" else 0):raise ValueError("exact peer-stop lifecycle")
        if "RAR-EXPANSION:APP-" in serial:raise ValueError("no independent app in network-only campaign")
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

    return dict(content_validated=True,case=mode,frames=len(frames),wire_packets_per_peer=len(wanted),
                data_unchanged=True,scope="closed synthetic peers",milestone_complete=False)
def actual_refusals(raw,boots,system,sizes,mode):
    import copy
    validate(raw,boots,system,sizes,mode);base=helper("runtime_evidence")
    original=helper("expansion_evidence").parse(raw);count=0
    mutations=[(["boot_sha256"],{}),(["system_sha256"],"0"*64),(["frames"],[]),
        (["captured_wire"],{}),(["initial_data"],[]),(["frozen_data"],[]),
        (["vm_proofs",0,"argv"],[]),(["vm_proofs",0,"preflight","raw"],{}),
        (["vm_proofs",0,"commands"],[]),(["vm_proofs",0,"serial"],""),
        (["vm_proofs",0,"audit"],[]),(["vm_proofs",0,"qmp_drained"],False),
        (["pair_cleanup","joined"],False),(["elapsed_milliseconds"],True)]
    for path,replacement in mutations:
        changed=copy.deepcopy(original);item=changed
        for key in path[:-1]:item=item[key]
        item[path[-1]]=replacement
        try:validate(base.canonical(changed),boots,system,sizes,mode)
        except (ValueError,KeyError,TypeError):count+=1
        else:raise AssertionError("changed campaign evidence accepted")
    return count
