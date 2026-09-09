"""Strict pure audit matching for controller-owned Modern Data fault plans.
No device/VM execution or file/network access. A matching record is an
observation only: lifecycle joins and independent frozen bytes remain required.
"""
import json

class Invalid(ValueError):
    pass

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True,allow_nan=False).encode("ascii")

def record(raw):
    """Accept exactly the child emitter's canonical JSON, without its newline."""
    def pairs(items):
        result={}
        for key,value in items:
            if key in result:raise Invalid("duplicate audit key")
            result[key]=value
        return result
    def constant(value):raise Invalid("nonfinite audit value")
    if type(raw) is not bytes or not 1<=len(raw)<=2047:raise Invalid("audit line bound")
    try:
        value=json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)
        if type(value) is not dict or canonical(value)!=raw:raise Invalid("canonical audit object")
    except (ValueError,UnicodeError,RecursionError,OverflowError) as error:
        raise Invalid("invalid canonical audit record") from error
    return value

def plan(value):
    if (type(value) is not dict or set(value)!={"operation","ordinal","effect","prefix"} or
        value["operation"] not in ("write","flush") or type(value["ordinal"]) is not int or
        not 1<=value["ordinal"]<=8192 or value["effect"] not in
        ("error","short-error","before-cut","after-cut","torn-cut") or
        type(value["prefix"]) is not int or not 0<=value["prefix"]<=65536 or
        (value["effect"] not in ("short-error","torn-cut") and value["prefix"]!=0)):
        raise Invalid("fixed bounded Data fault plan")
    return dict(value)

def scan(records,expected,ready):
    """Validate a live or final Data audit prefix and return one matched hit.
    None means no hit YET, never success or permission to ignore a stopped child.
    Once a sticky error is observed, further requests may have no events because
    the device rejects them before starting operations. No later event is valid.
    """
    expected=plan(expected)
    if type(records) is not list or not 1<=len(records)<=16390:
        raise Invalid("bounded audit prefix")
    wanted={"type","kind","readonly","export_readonly","capacity","device","inode"}
    if (type(ready) is not dict or set(ready)!=wanted or ready["type"]!="ready" or
        ready["kind"]!="data" or ready["readonly"] is not False or
        ready["export_readonly"] is not False or type(ready["capacity"]) is not int or
        ready["capacity"]!=99328 or type(ready["device"]) is not int or ready["device"]<0 or
        type(ready["inode"]) is not int or ready["inode"]<=0 or
        canonical(records[0])!=canonical(ready)):
        raise Invalid("exact Data descriptor readiness")
    counts={"read":0,"write":0,"flush":0}
    dirty=set()
    pending=None;pending_index=None;hit=None;terminal=False
    cut=expected["effect"] in ("before-cut","after-cut","torn-cut")
    for index,row in enumerate(records[1:],1):
        if type(row) is not dict or terminal:raise Invalid("record after terminal/nonobject")
        kind=row.get("type")
        if kind=="request":
            if set(row)!={"type","operation","offset","length"}:
                raise Invalid("request fields")
            op,offset,length=row["operation"],row["offset"],row["length"]
            if (op not in counts or type(offset) is not int or type(length) is not int or
                (op=="flush" and (offset!=0 or length!=0)) or
                (op!="flush" and (offset<0 or offset%512 or not 512<=length<=65536 or
                    length%512 or offset+length>99328))):
                raise Invalid("request geometry")
            if hit is not None:
                if cut:raise Invalid("request after cut")
                # A permanently failed device emits only request records.
                continue
            if pending is not None:raise Invalid("overlapping request")
            pending=row;pending_index=index
        elif kind=="event":
            if hit is not None or set(row)!={"type","event"} or pending is None:
                raise Invalid("event without one live request")
            event=row["event"];op=pending["operation"]
            fields={"operation","ordinal","offset","length","status"}
            if op=="write":fields.add("payload_sha256")
            selected=op==expected["operation"] and counts[op]+1==expected["ordinal"]
            if selected:
                fields.add("injection")
                if expected["effect"] in ("short-error","torn-cut"):fields.add("persisted_prefix_bytes")
            if (type(event) is not dict or set(event)!=fields or event["operation"]!=op or
                type(event["ordinal"]) is not int or event["ordinal"]!=counts[op]+1 or
                type(event["offset"]) is not int or event["offset"]!=pending["offset"] or
                type(event["length"]) is not int or event["length"]!=pending["length"] or
                index!=pending_index+1):
                raise Invalid("event identity/order")
            if op=="write":
                digest=event["payload_sha256"]
                if type(digest) is not str or len(digest)!=64 or any(c not in "0123456789abcdef" for c in digest):
                    raise Invalid("write digest")
            counts[op]+=1
            if selected:
                if op=="write" and expected["prefix"]>pending["length"]:
                    raise Invalid("fault prefix exceeds observed write")
                if op=="flush" and expected["prefix"]>len(dirty)*512:
                    raise Invalid("fault prefix exceeds pending unique sectors")
                if expected["effect"] in ("short-error","torn-cut"):
                    if (type(event["persisted_prefix_bytes"]) is not int or
                        event["persisted_prefix_bytes"]!=expected["prefix"]):
                        raise Invalid("durable prefix completion required")
                if canonical(event["injection"])!=canonical(expected):
                    raise Invalid("exact fault injection")
                if event["status"]!=("cut-no-reply" if cut else "failed-no-success"):
                    raise Invalid("fault effect status")
                hit={"request_index":pending_index,"event_index":index,
                    "plan":dict(expected),"offset":pending["offset"],"length":pending["length"]}
            elif event["status"]!="completed":raise Invalid("unplanned device failure")
            elif op=="write":dirty.update(range(pending["offset"],pending["offset"]+pending["length"],512))
            elif op=="flush":dirty.clear()
            pending=None;pending_index=None
        elif kind=="terminal":
            if (hit is None or set(row)!={"type","outcome","fault_hit","failed"} or
                row["outcome"]!=("cut" if cut else "failed") or
                row["fault_hit"] is not True or row["failed"] is not (not cut)):
                raise Invalid("unexpected terminal")
            terminal=True
        else:raise Invalid("unknown audit record")
    if hit is None:return None
    return dict(hit,counts=dict(counts),terminal=terminal)

def self_test():
    ready=dict(type="ready",kind="data",readonly=False,export_readonly=False,
               capacity=99328,device=1,inode=2)
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except (Invalid,ValueError,TypeError):rejected+=1
        else:raise AssertionError("invalid fault receipt accepted")
    assert record(canonical(ready))==ready
    for raw in (b'{"a":1,"a":2}',b'{"a":NaN}',b'{"a": 1}',b'[]',b'{}\n'):
        reject(lambda raw=raw:record(raw))
    for op in ("write","flush"):
        for effect in ("error","short-error","before-cut","after-cut","torn-cut"):
            p=dict(operation=op,ordinal=1,effect=effect,prefix=0)
            req=dict(type="request",operation=op,offset=1024 if op=="write" else 0,
                     length=512 if op=="write" else 0)
            cut=effect.endswith("cut")
            e=dict(operation=op,ordinal=1,offset=req["offset"],length=req["length"],
                   status="cut-no-reply" if cut else "failed-no-success",injection=dict(p))
            if op=="write":e["payload_sha256"]="a"*64
            if effect in ("short-error","torn-cut"):e["persisted_prefix_bytes"]=0
            rows=[ready,req,dict(type="event",event=e)]
            assert scan([ready],p,ready) is None
            assert scan([ready,req],p,ready) is None
            got=scan(rows,p,ready)
            assert got["request_index"]==1 and got["event_index"]==2 and not got["terminal"]
            term=dict(type="terminal",outcome="cut" if cut else "failed",fault_hit=True,failed=not cut)
            assert scan(rows+[term],p,ready)["terminal"]
            for field,value in (("ordinal",True),("offset",1),("length",1),("status","completed"),
                                ("injection",dict(p,ordinal=2))):
                changed=dict(e);changed[field]=value
                reject(lambda changed=changed:scan([ready,req,dict(type="event",event=changed)],p,ready))
            reject(lambda:scan(rows+[dict(type="event",event=e)],p,ready))
            reject(lambda:scan(rows+[term,req],p,ready))
            if cut:reject(lambda:scan(rows+[req],p,ready))
            else:assert scan(rows+[req,req],p,ready)["event_index"]==2
            for field,value in (("fault_hit",1),("failed",int(not cut)),("outcome","closed")):
                changed=dict(term);changed[field]=value
                reject(lambda changed=changed:scan(rows+[changed],p,ready))
    # Overlapping successful writes dirty one physical sector, not two.
    p=dict(operation="flush",ordinal=1,effect="short-error",prefix=512)
    req=dict(type="request",operation="write",offset=1024,length=512)
    prefix=[ready]
    for ordinal in (1,2):
        prefix.extend([req,dict(type="event",event=dict(operation="write",ordinal=ordinal,
            offset=1024,length=512,status="completed",payload_sha256="a"*64))])
    flush=dict(type="request",operation="flush",offset=0,length=0)
    e=dict(operation="flush",ordinal=1,offset=0,length=0,status="failed-no-success",
           injection=p,persisted_prefix_bytes=512)
    assert scan(prefix+[flush,dict(type="event",event=e)],p,ready)["counts"]["write"]==2
    too_large=dict(p,prefix=513)
    reject(lambda:scan(prefix+[flush,dict(type="event",event=dict(e,injection=too_large,
        persisted_prefix_bytes=513))],too_large,ready))
    # An underlying _persist failure never receives the completion field.
    missing=dict(e);del missing["persisted_prefix_bytes"]
    reject(lambda:scan(prefix+[flush,dict(type="event",event=missing)],p,ready))
    for value in (True,511,513):
        reject(lambda value=value:scan(prefix+[flush,dict(type="event",
            event=dict(e,persisted_prefix_bytes=value))],p,ready))
    return rejected

if __name__=="__main__":
    import os
    import sys
    if (sys.argv!=[sys.argv[0],"--self-test"] or sys.platform!="linux" or
        os.environ.get("CI")!="true" or os.environ.get("GITHUB_ACTIONS")!="true" or
        not sys.flags.isolated or not sys.dont_write_bytecode):
        raise SystemExit("isolated cloud pure self-tests only")
    print("Modern Data fault audit:",self_test(),"negative fixtures; no VM execution")
