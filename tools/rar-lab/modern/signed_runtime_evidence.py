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
