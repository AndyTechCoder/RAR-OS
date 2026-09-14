"""Exact native-app/Data continuity through observed signed System transitions."""
import re
import importlib.util
from pathlib import Path
def helper(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def systems(factory,candidate,badsig,mode):
    e=helper("signed_runtime_evidence")
    installed,damaged,repaired=e.repair_images(factory,candidate)
    start=bytearray(e.expected_system(factory,candidate,"rejected"))
    at=1024+e.SLOT_BYTES;start[at:at+len(candidate)]=bytes(len(candidate));start=bytes(start)
    if mode=="install":return [start,start],[installed,start]
    if mode=="reject":return [start,start],[e.expected_system(factory,badsig,"rejected"),start]
    if mode=="fallback":return [installed,start],[e.expected_system(factory,candidate,"fallback"),start]
    if mode=="repair":return [damaged,start],[repaired,start]
    if mode=="final":return [repaired,start],[repaired,start]
    raise ValueError("fixed integrated transition")
def validate(raw,boots,firmware_sizes,owner,bank,mode,first,installed=None,repaired=None):
    base=helper("runtime_evidence");persist=helper("persistence");plan=helper("alpha_journey_plan")
    factory,candidate,badsig=[bank["modern-settings-"+n+".layer"] for n in ("factory","update","bad-signature")]
    initial_system,expected_system=systems(factory,candidate,badsig,mode)
    factory_system=systems(factory,candidate,badsig,"install")[0][0]
    helper("alpha_evidence").validate(first,boots,base.sha(factory_system),firmware_sizes,owner,"first")
    previous=helper("expansion_evidence").parse(first)
    if mode in ("fallback","repair","final"):
        if type(installed) is not bytes:raise ValueError("actual prior guest install required")
        validate(installed,boots,firmware_sizes,owner,bank,"install",first)
        observed=helper("expansion_evidence").parse(installed)
        actual=base.decoded(observed["frozen_system"][0],8388608)
        if mode=="repair":
            damaged=bytearray(actual)
            for offset in (1024,1024+helper("signed_runtime_evidence").SLOT_BYTES):damaged[offset]^=1
            if bytes(damaged)!=initial_system[0]:raise ValueError("only two fixed System header damage bytes")
        elif mode=="fallback" and actual!=initial_system[0]:raise ValueError("actual installed System continuity")
    if mode=="final":
        if type(repaired) is not bytes:raise ValueError("actual prior automatic repair required")
        validate(repaired,boots,firmware_sizes,owner,bank,"repair",first,installed)
        actual=helper("expansion_evidence").parse(repaired)["frozen_system"]
        if [base.decoded(x,8388608) for x in actual]!=initial_system:raise ValueError("actual repair continuity")
    value=helper("expansion_evidence").parse(raw)
    fields={"schema","mode","status","milestone_complete","challenges","frames","vm_proofs","pair_cleanup",
            "initial_data","frozen_data","captured_wire","boot_sha256","system_sha256","frozen_system","elapsed_milliseconds"}
    if (type(value) is not dict or set(value)!=fields or value["schema"]!="rar-alpha-system-journey-v1" or
        value["mode"]!=mode or value["status"]!="observed" or value["milestone_complete"] is not False):raise ValueError("exact nonaccepting journey envelope")
    if value["boot_sha256"]!=boots or value["system_sha256"]!=[base.sha(x) for x in initial_system]:raise ValueError("trusted source/System binding")
    if type(value["elapsed_milliseconds"]) is not int or not 0<value["elapsed_milliseconds"]<180000:raise ValueError("bounded journey")
    if value["challenges"]!=previous["challenges"]:raise ValueError("same post-GUI challenges")
    challenges=value["challenges"];steps=plan.plan(mode,challenges)
    if value["initial_data"]!=previous["frozen_data"] or value["frozen_data"]!=value["initial_data"]:raise ValueError("same exact authenticated Data, never reseeded")
    if type(value["frozen_system"]) is not list or len(value["frozen_system"])!=2:raise ValueError("two System domains")
    if [base.decoded(x,8388608) for x in value["frozen_system"]]!=expected_system:raise ValueError("exact System transitions and all other bytes preserved")
    scenes=[s for s in steps if not plan.marker(s[2])];frames=value["frames"]
    if type(frames) is not list or len(frames)!=len(scenes):raise ValueError("complete observed scenes")
    for frame,(peer,keys,stage,nonce,compact) in zip(frames,scenes):
        if type(frame) is not dict or set(frame)!={"peer","stage","compact","sha256","actual_ppm"} or frame["peer"]!=peer or frame["stage"]!=stage or type(frame["compact"]) is not bool or frame["compact"]!=compact:raise ValueError("exact scene identity")
        pixels=base.decoded(frame["actual_ppm"],len(helper("visual_oracle").HEADER)+640*480*3)
        if frame["sha256"]!=plan.validate(pixels,stage,nonce,compact):raise ValueError("full actual framebuffer")
    import struct
    captured=value["captured_wire"]
    if type(captured) is not dict or set(captured)!={"a","b"}:raise ValueError("two wire observations")
    empty=struct.pack("<IHHIIII",0xa1b2c3d4,2,4,0,0,554,1)
    if any(base.decoded(captured[p],24)!=empty for p in ("a","b")):raise ValueError("no unexpected network during System journey")
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
        helper("alpha_journey_plan").commands(proof["commands"],index,peer,mode,challenges,profile)
        base.checked_event_stream(proof["events"],proof["event_receipts"],proof["commands"],proof["qmp_drained"],verified["rtc_path"])
        serial=proof["serial"]
        if (type(serial) is not str or not serial.isascii() or len(serial)>65536 or serial.count("RAR-MODERN:GUI-READY")!=1 or
            any(x in serial for x in ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN"))):
            raise ValueError("guest readiness/fault transcript")
        expected=2 if peer=="a" else 0
        for marker in ("RAR-EXPANSION:APP-HELD","RAR-EXPANSION:APP-STARTED"):
            if serial.count(marker)!=expected:raise ValueError("actual independent Notes lifecycle")
        if "RAR-EXPANSION:APP-FAULT" in serial or "RAR-MODERN:APP-FAULT=" in serial:raise ValueError("no unexpected app fault")
        events={"RAR-MODERN:UPDATE-INSTALLED":mode=="install",
                "RAR-MODERN:UPDATE-REJECTED":mode in ("reject","final"),
                "RAR-MODERN:UPDATE-FALLBACK":mode=="fallback",
                "RAR-MODERN:SETTINGS-ACTIVE-FAULT":mode=="fallback"}
        for marker,needed in events.items():
            if serial.count(marker)!=(1 if needed and peer=="a" else 0):raise ValueError("exact signed lifecycle "+marker)
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
            checked=persist.audit(records,role,records[0],system_updates=(peer=="a" and role=="system"))
            if (role!="system" or peer=="b" or mode=="final") and (checked["counts"]["write"] or checked["counts"]["flush"]):raise ValueError("only selected System transaction may mutate")
            if peer=="b" and (checked["counts"]["write"] or checked["counts"]["flush"]):raise ValueError("idle peer mutated disk")
            audits.append(checked);identities.append((records[0]["device"],records[0]["inode"]))
        if base.canonical(audits)!=base.canonical(proof["audit"]):raise ValueError("storage audit mismatch")
    if len(set(pids))!=2 or len(set(identities))!=6:raise ValueError("independent guests and six storage domains")

    return dict(content_validated=True,case=mode,frames=len(frames),private_and_shared_data_unchanged=True,
                exact_system_bytes=True,whole_pair_reaped=True,milestone_complete=False)
def actual_refusals(raw,boots,sizes,owner,bank,mode,first,installed=None,repaired=None):
    import copy,base64
    validate(raw,boots,sizes,owner,bank,mode,first,installed,repaired)
    base=helper("runtime_evidence");original=helper("expansion_evidence").parse(raw);count=0
    paths=[(["initial_data"],[]),(["frozen_data"],[]),(["frozen_system"],[]),(["system_sha256"],[]),
           (["frames"],[]),(["captured_wire"],{}),(["vm_proofs",0,"argv"],[]),
           (["vm_proofs",0,"commands"],[]),(["vm_proofs",0,"preflight","raw"],{}),
           (["vm_proofs",0,"serial"],""),(["vm_proofs",0,"audit"],[]),
           (["vm_proofs",0,"qmp_drained"],False),(["pair_cleanup","joined"],False)]
    damaged=bytearray(base.decoded(original["frozen_system"][0],8388608));damaged[-1]^=1
    paths.append((["frozen_system",0],base64.b64encode(damaged).decode("ascii")))
    for path,replacement in paths:
        changed=copy.deepcopy(original);item=changed
        for key in path[:-1]:item=item[key]
        item[path[-1]]=replacement
        try:validate(base.canonical(changed),boots,sizes,owner,bank,mode,first,installed,repaired)
        except (ValueError,KeyError,TypeError):count+=1
        else:raise AssertionError("changed actual journey accepted")
    return count
