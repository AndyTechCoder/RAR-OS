"""Independent bounded retained Alpha proof validation; pure bytes only."""
import re
import importlib.util
from pathlib import Path
def helper(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def validate(raw,boots,system,firmware_sizes,owner,mode,prior=None):
    base=helper("runtime_evidence");persist=helper("persistence")
    value=helper("expansion_evidence").parse(raw)
    fields={"schema","mode","status","milestone_complete","challenges","frames","vm_proofs","pair_cleanup",
            "initial_data","frozen_data","captured_wire","boot_sha256","system_sha256","elapsed_milliseconds"}
    if (type(value) is not dict or set(value)!=fields or value["schema"]!="rar-native-alpha-v1" or
        mode not in ("first","fresh") or value["mode"]!=mode or value["status"]!="observed" or value["milestone_complete"] is not False):
        raise ValueError("exact nonaccepting Alpha envelope")
    if type(boots) is not dict or set(boots)!={"a","b"} or any(base.digest(h)!=h for h in boots.values()):raise ValueError("two trusted boot hashes")
    if value["boot_sha256"]!=boots or value["system_sha256"]!=base.digest(system):raise ValueError("build binding")
    if type(owner) is not bytes or len(owner)!=32 or not any(owner):raise ValueError("trusted signed Notes owner")
    if type(value["elapsed_milliseconds"]) is not int or not 0<value["elapsed_milliseconds"]<180000:raise ValueError("duration")
    challenges=value["challenges"];steps=helper("alpha_plan").plan(mode,challenges)
    doc,shared,left,right=challenges
    for name in ("initial_data","frozen_data"):
        if type(value[name]) is not list or len(value[name])!=2:raise ValueError("two Data images")
    initial=[base.decoded(x,99328) for x in value["initial_data"]]
    frozen=[base.decoded(x,99328) for x in value["frozen_data"]]
    if initial[0]==initial[1] or frozen[1]!=initial[1]:raise ValueError("separate preserved idle peer")
    oracle=helper("data_oracle")
    if mode=="first":
        if prior is not None:raise ValueError("no prior at initial boot")
        for data in initial:
            state=oracle.inspect(data)
            if state["revision"]!=0 or state["files"]!={} or any(data[1024:]):raise ValueError("virgin Data before challenges")
    else:
        if type(prior) is not bytes:raise ValueError("exact prior retained proof")
        validate(prior,boots,system,firmware_sizes,owner,"first")
        previous=helper("expansion_evidence").parse(prior)
        if value["initial_data"]!=previous["frozen_data"] or challenges!=previous["challenges"] or initial!=frozen:raise ValueError("fresh causal byte continuity")
    state=oracle.inspect(frozen[0])
    if state["revision"]!=4 or state["files"]!={b"_rar.app0":owner,b"_rar.doc0":doc.encode("ascii"),b"note":shared.encode("ascii")}:
        raise ValueError("authenticated private and shared document content")
    if state["burned_slots"] or state["committed_slots"]!=[0,1,2,3]:raise ValueError("exact four acknowledged commits")
    frames=value["frames"]
    if type(frames) is not list or len(frames)!=len(steps):raise ValueError("complete actual scene sequence")
    for frame,(peer,keys,stage,nonce,compact) in zip(frames,steps):
        if type(frame) is not dict or set(frame)!={"peer","stage","compact","sha256","actual_ppm"} or frame["peer"]!=peer or frame["stage"]!=stage or type(frame["compact"]) is not bool or frame["compact"]!=compact:raise ValueError("frame identity")
        pixels=base.decoded(frame["actual_ppm"],len(helper("visual_oracle").HEADER)+640*480*3)
        if frame["sha256"]!=helper("alpha_visual").validate(pixels,stage,nonce,compact):raise ValueError("full actual pixels")
    captured=value["captured_wire"]
    if type(captured) is not dict or set(captured)!={"a","b"}:raise ValueError("two actual wire captures")
    if mode=="first":
        helper("expansion_wire").validate({p:base.decoded(captured[p],204) for p in ("a","b")},left,right)
    else:
        # A fresh read-only journey emits no Ethernet packet. Require two
        # matching valid empty classic-PCAP headers, not a guest assertion.
        import struct
        empty=struct.pack("<IHHIIII",0xa1b2c3d4,2,4,0,0,554,1)
        if any(base.decoded(captured[p],24)!=empty for p in ("a","b")):raise ValueError("unexpected fresh-boot traffic")
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
        helper("alpha_plan").commands(proof["commands"],index,peer,mode,challenges,profile)
        base.checked_event_stream(proof["events"],proof["event_receipts"],proof["commands"],proof["qmp_drained"],verified["rtc_path"])
        serial=proof["serial"]
        if (type(serial) is not str or not serial.isascii() or len(serial)>65536 or serial.count("RAR-MODERN:GUI-READY")!=1 or
            any(x in serial for x in ("RAR-PANIC","UNEXPECTED-USER-FAULT","INVALID-USER-RETURN","APP-FAULT="))):
            raise ValueError("guest readiness/fault transcript")
        expected=4 if mode=="first" else 1
        if peer=="b":expected=0
        for marker in ("RAR-EXPANSION:APP-HELD","RAR-EXPANSION:APP-STARTED"):
            if serial.count(marker)!=expected:raise ValueError("actual app lifecycle count")
        if serial.count("RAR-EXPANSION:APP-FAULT")!=(1 if mode=="first" and peer=="a" else 0):raise ValueError("single contained C app fault")
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
            if mode=="fresh" and (checked["counts"]["write"] or checked["counts"]["flush"]):raise ValueError("fresh read journey mutated disk")
            if peer=="b" and (checked["counts"]["write"] or checked["counts"]["flush"]):raise ValueError("idle peer mutated disk")
            audits.append(checked);identities.append((records[0]["device"],records[0]["inode"]))
        if base.canonical(audits)!=base.canonical(proof["audit"]):raise ValueError("storage audit mismatch")
    if len(set(pids))!=2 or len(set(identities))!=6:raise ValueError("independent guests and six storage domains")

    return dict(content_validated=True,mode=mode,frames=len(frames),independent_apps=2 if mode=="first" else 1,
                private_document_bytes=32,shared_document_bytes=32,fresh_boot=mode=="fresh",
                profiles=["wide-548x260","compact-320x260"],milestone_complete=False)

def actual_refusals(raw,boots,system,sizes,owner,mode,prior=None):
    """Mutate copies of the actual successful proof, never fixtures or targets."""
    validate(raw,boots,system,sizes,owner,mode,prior)
    import copy
    import base64
    base=helper("runtime_evidence");original=helper("expansion_evidence").parse(raw);count=0
    def rejected(changed,changed_prior=prior):
        nonlocal count
        try:validate(base.canonical(changed),boots,system,sizes,owner,mode,changed_prior)
        except (ValueError,KeyError,TypeError):count+=1
        else:raise AssertionError("altered actual Alpha proof accepted")
    for field,replacement in (("mode","bad"),("status","failed"),("milestone_complete",True),
                             ("frames",[]),("vm_proofs",[]),("pair_cleanup",{}),("frozen_data",[]),
                             ("captured_wire",{}),("elapsed_milliseconds",True),("boot_sha256",{}),
                             ("system_sha256","0"*64),("challenges",["a"*32]*4)):
        changed=copy.deepcopy(original);changed[field]=replacement;rejected(changed)
    def edit(path,replacement):
        changed=copy.deepcopy(original);item=changed
        for key in path[:-1]:item=item[key]
        item[path[-1]]=replacement;rejected(changed)
    edit(["initial_data",0],original["initial_data"][1])
    edit(["frozen_data",0],original["frozen_data"][1])
    edit(["frames",0,"sha256"],"0"*64)
    pixels=bytearray(base.decoded(original["frames"][0]["actual_ppm"],len(helper("visual_oracle").HEADER)+640*480*3))
    pixels[-1]^=1;edit(["frames",0,"actual_ppm"],base64.b64encode(pixels).decode("ascii"))
    wire=bytearray(base.decoded(original["captured_wire"]["a"],204 if mode=="first" else 24))
    wire[-1]^=1;edit(["captured_wire","a"],base64.b64encode(wire).decode("ascii"))
    edit(["vm_proofs",0,"argv",0],"/unapproved/emulator")
    edit(["vm_proofs",0,"preflight","raw"],{})
    edit(["vm_proofs",0,"preflight","verified"],{})
    edit(["vm_proofs",0,"commands",0,"id"],0)
    edit(["vm_proofs",0,"commands"],original["vm_proofs"][0]["commands"][:-1])
    edit(["vm_proofs",0,"qmp_drained"],False)
    edit(["vm_proofs",0,"event_receipts"],[])
    serial=original["vm_proofs"][0]["serial"]
    edit(["vm_proofs",0,"serial"],serial.replace("RAR-EXPANSION:APP-HELD","MISSING",1))
    edit(["vm_proofs",0,"serial"],serial+"RAR-EXPANSION:APP-FAULT\n")
    edit(["pair_cleanup","network_closed"],False)
    edit(["vm_proofs",0,"cut","joined"],False)
    edit(["vm_proofs",0,"audit"],[])
    # Keep aggregate and member copies in sync so the deeper reap/audit checks,
    # rather than only the duplicate-receipt equality check, must reject.
    for path,replacement in ((["vm_returncode"],0),(["backends",0,"joined"],False),
                             (["backends",0,"records"],[]),
                             (["backends",0,"records",0,"inode"],0)):
        changed=copy.deepcopy(original)
        for root in (changed["vm_proofs"][0]["cut"],changed["pair_cleanup"]["guests"][0]):
            item=root
            for key in path[:-1]:item=item[key]
            item[path[-1]]=replacement
        rejected(changed)
    changed=copy.deepcopy(original);pid=changed["vm_proofs"][0]["cut"]["vm_pid"]
    changed["vm_proofs"][1]["cut"]["vm_pid"]=pid;changed["pair_cleanup"]["guests"][1]["vm_pid"]=pid
    rejected(changed)
    changed=copy.deepcopy(original)
    same=changed["vm_proofs"][0]["cut"]["backends"][0]["records"][0]
    for root in (changed["vm_proofs"][1]["cut"],changed["pair_cleanup"]["guests"][1]):
        ready=root["backends"][0]["records"][0];ready["device"]=same["device"];ready["inode"]=same["inode"]
    rejected(changed)
    if mode=="fresh":
        changed_prior=copy.deepcopy(helper("expansion_evidence").parse(prior))
        changed_prior["frozen_data"][0]=changed_prior["frozen_data"][1]
        rejected(copy.deepcopy(original),base.canonical(changed_prior))
    return count
