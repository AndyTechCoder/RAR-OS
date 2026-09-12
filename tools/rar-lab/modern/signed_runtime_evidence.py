"""Independent exact-byte expectations for bounded signed-update cloud cases.
No VM/process/file mutation and no target imports. These are expected bytes,
never captured evidence and never authentication or runtime success by themselves.
"""
from hashlib import sha256
import struct
SYSTEM_BYTES=8388608
SLOT_BYTES=4097*512
CASES=("update","bad-health","bad-signature","bad-abi")

def package(value):
    if type(value) is not bytes or not 896<=len(value)<=2097536:
        raise ValueError("bounded laboratory package")
    if value[:8]!=b"RARMODL0" or int.from_bytes(value[56:60],"little")+384!=len(value):
        raise ValueError("exact package framing")
    if value[288:320]!=sha256(value[:288]).digest():
        raise ValueError("manifest identity")
    generation=int.from_bytes(value[72:80],"little")
    if generation==0:raise ValueError("nonzero generation")
    return generation,value[288:320]

def record(kind,sequence,highest,active_slot,active,previous,parent):
    out=bytearray(512);out[:8]=b"RARSYS00";out[10:12]=(512).to_bytes(2,"little")
    out[12]=kind;out[13]=active_slot;out[14]=255 if previous is None else 0
    for offset,value in ((16,sequence),(24,highest),(32,1),(40,active[0]),
                         (48,0 if previous is None else previous[0]),
                         (56,0 if parent is None else sequence-1)):
        struct.pack_into("<Q",out,offset,value)
    out[64:96]=active[1]
    if previous is not None:out[96:128]=previous[1]
    if parent is not None:out[128:160]=sha256(parent).digest()
    out[480:]=sha256(out[:480]).digest();return bytes(out)

def expected_system(factory,candidate,outcome):
    """One install attempt on freshly provisioned factory media, no retries."""
    if outcome not in ("rejected","installed","fallback"):
        raise ValueError("fixed final outcome")
    old,new=package(factory),package(candidate)
    if old[0]!=1 or new[0]<=old[0]:raise ValueError("fixed increasing lab generations")
    first=record(0,1,1,0,old,None,None)
    out=bytearray(SYSTEM_BYTES);out[:512]=first
    out[1024:1024+len(factory)]=factory
    b=1024+SLOT_BYTES;out[b:b+len(candidate)]=candidate
    if outcome!="rejected":
        second=record(1,2,new[0],1,new,old,first);out[512:1024]=second
        if outcome=="fallback":out[:512]=record(2,3,new[0],0,old,None,second)
    return bytes(out)

def validate_system(observed,factory,candidate,outcome):
    expected=expected_system(factory,candidate,outcome)
    if type(observed) is not bytes or observed!=expected:
        raise ValueError("actual System differs from exact slot/selector/reserved bytes")
    generation,_=package(candidate)
    return dict(sha256=sha256(observed).hexdigest(),outcome=outcome,
        sequence={"rejected":1,"installed":2,"fallback":3}[outcome],
        active_generation=generation if outcome=="installed" else 1,
        high_water=1 if outcome=="rejected" else generation)


def repair_images(factory,candidate):
    """Fixed installed gen2 -> two invalid headers -> exact gen1 Repair.
    Pure expectations only; never media mutation or runtime evidence.
    """
    old,new=package(factory),package(candidate)
    if old[0]!=1 or new[0]!=2:raise ValueError("fixed repair generations1/2")
    installed=expected_system(factory,candidate,"installed")
    damaged=bytearray(installed)
    for offset in (1024,1024+SLOT_BYTES):damaged[offset]^=1
    repaired=bytearray(damaged)
    rounded=(len(factory)+511)//512*512
    repaired[1024:1024+rounded]=factory+bytes(rounded-len(factory))
    repaired[:512]=record(3,3,2,0,old,None,installed[512:1024])
    return installed,bytes(damaged),bytes(repaired)

def validate_repair_system(observed,factory,candidate):
    expected=repair_images(factory,candidate)[2]
    if type(observed) is not bytes or observed!=expected:
        raise ValueError("repair must preserve every non-target System byte")
    return dict(sha256=sha256(observed).hexdigest(),outcome="repair",
        sequence=3,active_generation=1,high_water=2,active_slot=0,previous=None)

def system_fault_cases(factory,candidate,mode):
    """Every physical write and flush; fixed effects, no caller disk offsets."""
    if mode not in ("install","repair"):raise ValueError("fixed System transaction")
    package(factory);package(candidate)
    payload=factory if mode=="repair" else candidate
    sectors=(len(payload)+511)//512
    return [dict(operation=operation,ordinal=ordinal,effect=effect,
                 prefix=255 if effect in ("torn-cut","short-error") else 0)
        for operation,count in (("write",sectors+1),("flush",2))
        for ordinal in range(1,count+1)
        for effect in ("before-cut","after-cut","error","torn-cut","short-error")]

def system_fault_operations(factory,candidate,mode):
    """Ordered permitted System mutations, including unflushed writes."""
    if mode not in ("install","repair"):raise ValueError("fixed System transaction")
    installed,_,repaired=repair_images(factory,candidate)
    payload=factory if mode=="repair" else candidate
    start=1024 if mode=="repair" else 1024+SLOT_BYTES
    rounded=(len(payload)+511)//512*512;padded=payload+bytes(rounded-len(payload))
    writes=[dict(operation="write",offset=start+i,length=512,
        payload_sha256=sha256(padded[i:i+512]).hexdigest()) for i in range(0,rounded,512)]
    selector=repaired[:512] if mode=="repair" else installed[512:1024]
    return writes+[dict(operation="flush",offset=0,length=0),
        dict(operation="write",offset=0 if mode=="repair" else 512,length=512,
             payload_sha256=sha256(selector).hexdigest()),
        dict(operation="flush",offset=0,length=0)]

def system_fault_image(factory,candidate,mode,case):
    """Replay only the fixed native sector order against stable/volatile bytes.
    This is an independent expectation, not proof an injected operation occurred.
    Actual backend audit, complete joins and fresh-VM behavior remain required.
    """
    cases=system_fault_cases(factory,candidate,mode)
    if type(case) is not int or not 0<=case<len(cases):raise ValueError("fixed fault index")
    selected=cases[case]
    installed,damaged,repaired=repair_images(factory,candidate)
    if mode=="repair":
        initial=damaged;payload=factory;start=1024;selector=repaired[:512];selector_offset=0
    else:
        initial=bytearray(installed);initial[512:1024]=bytes(512)
        initial[1024+SLOT_BYTES:]=bytes(SYSTEM_BYTES-1024-SLOT_BYTES)
        initial=bytes(initial);payload=candidate;start=1024+SLOT_BYTES
        selector=installed[512:1024];selector_offset=512
    rounded=(len(payload)+511)//512*512
    padded=payload+bytes(rounded-len(payload))
    operations=[("write",start+i,padded[i:i+512]) for i in range(0,rounded,512)]
    operations+=[("flush",0,b""),("write",selector_offset,selector),("flush",0,b"")]
    stable=bytearray(initial);volatile=bytearray(initial);dirty=set();counts={"write":0,"flush":0}
    for operation,offset,value in operations:
        counts[operation]+=1
        hit=operation==selected["operation"] and counts[operation]==selected["ordinal"]
        if hit and selected["effect"] in ("before-cut","error"):break
        if hit and selected["effect"] in ("torn-cut","short-error"):
            parts=[(offset,value)] if operation=="write" else [
                (i,bytes(volatile[i:i+512])) for i in sorted(dirty)]
            left=selected["prefix"]
            if left>sum(len(part) for _,part in parts):raise ValueError("prefix exceeds pending bytes")
            for position,part in parts:
                n=min(left,len(part));stable[position:position+n]=part[:n];left-=n
            break
        if operation=="write":
            volatile[offset:offset+512]=value;dirty.add(offset)
        else:
            for position in sorted(dirty):stable[position:position+512]=volatile[position:position+512]
            dirty.clear()
        if hit:break
    else:raise AssertionError("fixed transaction did not hit its fault")
    return bytes(stable)

def system_fault_restart(factory,candidate,mode,case):
    """Exact old/new/repair/fallback selection after whole-VM destruction."""
    frozen=system_fault_image(factory,candidate,mode,case)
    installed,damaged,repaired=repair_images(factory,candidate)
    if mode=="install":
        # Exact or invalid selector1; selector0 remains intact and authenticated.
        committed=frozen[512:1024]==installed[512:1024]
        return frozen,committed,"installed" if committed else "factory"
    if frozen[:512]==repaired[:512]:return frozen,False,"repair"
    rounded=(len(factory)+511)//512*512
    intact=frozen[1024:1024+rounded]==factory+bytes(rounded-len(factory))
    if intact:
        result=bytearray(frozen)
        result[:512]=record(2,3,2,0,package(factory),None,installed[512:1024])
        return bytes(result),False,"fallback"
    # With both damaged headers and intact selector2, bootstrap must repair anew.
    result=bytearray(frozen)
    result[1024:1024+rounded]=factory+bytes(rounded-len(factory))
    result[:512]=repaired[:512]
    return bytes(result),False,"repair"

def settings_expected(visual,updated,compact=False):
    if type(updated) is not bool or type(compact) is not bool or compact and not updated:
        raise ValueError("fixed Settings code state")
    lines=["APPEARANCE","DARK","SPACE TO CHANGE THEME"]
    lines+=(["D SPACING / X LAB FAULT","COMPACT" if compact else "COMFORTABLE","UPDATED SETTINGS"]
        if updated else ["SESSION ONLY","",""])
    return visual._render_scene((False,(5,),5,{5:lines},False))

def settings_validate(frame,visual,updated,compact=False):
    if type(frame) is not bytes or frame!=settings_expected(visual,updated,compact):
        raise ValueError("actual Settings pixels differ from independent code-state expectation")
    return sha256(frame).hexdigest()

def self_test():
    def fake(generation,byte):
        data=bytearray(896);data[:8]=b"RARMODL0"
        data[56:60]=(512).to_bytes(4,"little");data[72:80]=generation.to_bytes(8,"little")
        data[384:]=bytes([byte])*512
        data[288:320]=sha256(data[:288]).digest();return bytes(data)
    factory,candidate=fake(1,1),fake(2,2)
    outputs=[expected_system(factory,candidate,outcome) for outcome in ("rejected","installed","fallback")]
    for outcome,image in zip(("rejected","installed","fallback"),outputs):
        assert validate_system(image,factory,candidate,outcome)["outcome"]==outcome
        for at in (0,12,13,14,16,24,40,48,64,128,480,512,992,1024,
                   1024+len(factory),1024+SLOT_BYTES,8196*512,SYSTEM_BYTES-1):
            changed=bytearray(image);changed[at]^=1
            try:validate_system(bytes(changed),factory,candidate,outcome)
            except ValueError:pass
            else:raise AssertionError("changed System accepted")
    assert len(set(map(lambda b:sha256(b).digest(),outputs)))==3
    for image in outputs[1:]:
        try:validate_system(image,factory,candidate,"rejected")
        except ValueError:pass
        else:raise AssertionError("publication accepted as rejection")
    installed,damaged,repaired=repair_images(factory,candidate)
    assert installed==outputs[1]
    assert [i for i,(a,b) in enumerate(zip(installed,damaged)) if a!=b]==[1024,1024+SLOT_BYTES]
    assert validate_repair_system(repaired,factory,candidate)["high_water"]==2
    assert repaired[512:1024]==installed[512:1024]
    assert repaired[1024+SLOT_BYTES:]==damaged[1024+SLOT_BYTES:]
    for image in (installed,damaged,outputs[0],outputs[2]):
        try:validate_repair_system(image,factory,candidate)
        except ValueError:pass
        else:raise AssertionError("non-repair System accepted")
    for at in (12,13,14,16,24,40,48,56,64,128,480,512,992,1024,
               1024+len(factory),1024+SLOT_BYTES,8196*512,SYSTEM_BYTES-1):
        changed=bytearray(repaired);changed[at]^=1
        try:validate_repair_system(bytes(changed),factory,candidate)
        except ValueError:pass
        else:raise AssertionError("changed repair System accepted")
    # Independent stable-byte crash expectations at every native System boundary.
    for mode in ("install","repair"):
        plans=system_fault_cases(factory,candidate,mode)
        assert len(plans)==25
        assert {(p["operation"],p["ordinal"]) for p in plans}=={
            ("write",1),("write",2),("write",3),("flush",1),("flush",2)}
        images=[];outcomes=set()
        for case,plan in enumerate(plans):
            frozen=system_fault_image(factory,candidate,mode,case)
            after,updated,chosen=system_fault_restart(factory,candidate,mode,case)
            images.append(frozen);outcomes.add(chosen)
            assert type(frozen) is bytes and len(frozen)==SYSTEM_BYTES
            assert type(after) is bytes and len(after)==SYSTEM_BYTES
            if mode=="install":
                assert frozen[:512]==outputs[0][:512]
                assert after==frozen
                assert updated==(chosen=="installed")
                assert frozen[1024:1024+SLOT_BYTES]==outputs[0][1024:1024+SLOT_BYTES]
            else:
                assert frozen[512:1024]==installed[512:1024]
                assert frozen[1024+SLOT_BYTES:]==damaged[1024+SLOT_BYTES:]
                assert after[512:1024]==installed[512:1024]
                assert after[1024:1024+len(factory)]==factory
                assert after[1024+SLOT_BYTES:]==damaged[1024+SLOT_BYTES:]
                assert after[:512] in (
                    repaired[:512],record(2,3,2,0,package(factory),None,installed[512:1024]))
                assert not updated and chosen in ("repair","fallback")
            if plan["operation"]=="flush" and plan["ordinal"]==2 and plan["effect"]=="after-cut":
                assert frozen==(repaired if mode=="repair" else installed)
        assert len(set(map(lambda b:sha256(b).digest(),images)))>=3
        assert outcomes==({"repair","fallback"} if mode=="repair" else {"installed","factory"})
        for invalid in (-1,True,1.0,len(plans),None):
            try:system_fault_image(factory,candidate,mode,invalid)
            except ValueError:pass
            else:raise AssertionError("invalid fixed System fault index accepted")
    # A volatile first write never changes the physical image. A torn header can
    # restore the prior unit, in which case fresh boot must use normal fallback.
    assert system_fault_image(factory,candidate,"repair",0)==damaged
    assert system_fault_image(factory,candidate,"repair",1)==damaged
    assert system_fault_restart(factory,candidate,"repair",3)[2]=="fallback"
    from pathlib import Path
    import runpy
    visual=type("Visual",(),runpy.run_path(str(Path(__file__).resolve().with_name("visual_oracle.py"))))
    frames=[settings_expected(visual,False),settings_expected(visual,True),
            settings_expected(visual,True,True)]
    assert len(set(frames))==3
    for frame,(updated,compact) in zip(frames,((False,False),(True,False),(True,True))):
        assert len(settings_validate(frame,visual,updated,compact))==64
        try:settings_validate(frame[:-1],visual,updated,compact)
        except ValueError:pass
        else:raise AssertionError("truncated Settings frame accepted")
    try:settings_expected(visual,False,True)
    except ValueError:pass
    else:raise AssertionError("factory cannot have updated spacing")
    return True

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure self-test only")
    self_test();print("Signed runtime evidence byte/visual tests passed; no captured VM evidence")
