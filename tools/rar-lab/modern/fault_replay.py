"""Independent in-memory expected bytes/requests for the fixed public fault lab.
Never writes an image, mounts it, feeds expected bytes to a guest, or imports
RAR target code. Request geometry is confirmed by the retained baseline inspection34350130709:
one header-probe read precedes the complete194-sector mount scan.
"""
import hashlib
import importlib.util
from pathlib import Path

def oracle():
    path=Path(__file__).with_name("data_oracle.py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed independent oracle")
    spec=importlib.util.spec_from_file_location("fault_public_crypto_oracle",path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def sectors(initial,value):
    op=oracle()
    if type(initial) is not bytes or len(initial)!=99328 or any(initial[1024:]):
        raise ValueError("one exact virgin in-memory public fixture")
    if type(value) is not str or len(value)!=32 or any(c not in "abcdefghijklmnop" for c in value):
        raise ValueError("exact public challenge")
    state=op.inspect(initial)
    if state["revision"]!=0 or state["files"]!={} or state["next_slot"]!=0 or state["readonly"]:
        raise ValueError("authenticated empty Data state")
    header=initial[:512];parent=bytes(32);result=[]
    for slot,content in enumerate((b"",value.encode("ascii"))):
        raw=b"RARDAT00"+bytes((1,))+bytes(7)+bytes((4,len(content)))+b"note"+content
        raw+=bytes(256-len(raw))
        prefix=b"RARVPY00"+slot.to_bytes(8,"little")+(slot+1).to_bytes(8,"little")+parent+bytes(8)
        nonce=header[96:100]+slot.to_bytes(8,"little")
        aad=b"RAR-VAULT-AAD-V0"+hashlib.sha256(header).digest()+prefix
        cipher=op._xor(header[64:96],nonce,raw)
        payload=prefix+cipher+op._tag(header[64:96],nonce,aad,cipher)+bytes(176)
        if len(payload)!=512:raise ValueError("fixed payload geometry")
        parent=hashlib.sha256(payload).digest()
        result.extend((bytes((0xa5,))*512,payload,bytes((0xc3,))*512))
    return result

def expected(initial,value,plan):
    """Return expected inert bytes and complete request prefix up to one hit.
    The expected value never provisions or repairs the actual retained fixture.
    """
    if (type(plan) is not dict or set(plan)!={"operation","ordinal","effect","prefix"} or
        plan["operation"] not in ("write","flush") or type(plan["ordinal"]) is not int or
        not 1<=plan["ordinal"]<=6 or plan["effect"] not in
        ("before-cut","after-cut","error","torn-cut","short-error") or
        type(plan["prefix"]) is not int or plan["prefix"]!=
        (255 if plan["effect"] in ("torn-cut","short-error") else 0)):
        raise ValueError("fixed reviewed publication plan")
    payloads=sectors(initial,value);durable=bytearray(initial);requests=[];writes=[]
    def request(operation,sector=0):
        row=dict(type="request",operation=operation,offset=sector*512 if operation!="flush" else 0,
            length=512 if operation!="flush" else 0)
        requests.append(row)
    request("read",0)  # Observed header probe before the independent full mount scan.
    for sector in range(194):request("read",sector)
    for slot in range(2):
        for sector in range(2+3*slot,5+3*slot):request("read",sector)
        for stage in range(3):
            index=slot*3+stage;sector=2+index;payload=payloads[index]
            for operation in ("write","flush"):
                request(operation,sector)
                if operation=="write":writes.append(hashlib.sha256(payload).hexdigest())
                hit=operation==plan["operation"] and index+1==plan["ordinal"]
                if hit:
                    prefix=(255 if plan["effect"] in ("torn-cut","short-error") else
                            512 if operation=="flush" and plan["effect"]=="after-cut" else 0)
                    durable[sector*512:sector*512+prefix]=payload[:prefix]
                    return bytes(durable),requests,writes
                if operation=="flush":durable[sector*512:(sector+1)*512]=payload
            request("read",sector)
    raise AssertionError("fixed fault must be hit")

def validate(records,initial,frozen,value,plan):
    predicted,requests,writes=expected(initial,value,plan)
    actual=[row for row in records if row.get("type")=="request"]
    if actual!=requests:raise ValueError("complete mount/preread/publication/readback prefix differs")
    actual_hashes=[row["event"]["payload_sha256"] for row in records
        if row.get("type")=="event" and row["event"].get("operation")=="write"]
    if actual_hashes!=writes:raise ValueError("actual guest write hashes differ from independent public sectors")
    if type(frozen) is not bytes or frozen!=predicted:
        raise ValueError("frozen bytes differ from exact injected durable prefix")
    return dict(requests=len(requests),writes=len(writes),
                frozen_sha256=hashlib.sha256(frozen).hexdigest())

def readonly(records):
    requests=[row for row in records if row.get("type")=="request"]
    expected=[dict(type="request",operation="read",offset=sector*512,length=512)
              for sector in [0]+list(range(194))]
    if requests!=expected:raise ValueError("fresh readonly VM complete sequential mount")
    events=[row["event"] for row in records if row.get("type")=="event"]
    if len(events)!=195 or any(event.get("operation")!="read" or
        event.get("status")!="completed" for event in events):
        raise ValueError("fresh readonly mount read completion")
    return True
