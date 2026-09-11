"""One fixed System selector-write error; pure live/final evidence checks.
No role/path selector, arbitrary fault operation, VM, file or process API.
"""
from hashlib import sha256
RECONCILE=b"RAR-PANIC:CODE=UPDATE-RECONCILE"
PANIC=b"RAR-PANIC:BEGIN\n"+RECONCILE+b"\nRAR-PANIC:HALT\n"
B_START=4099*512
def plan(candidate):
    if type(candidate) is not bytes or not 896<=len(candidate)<=2097536:
        raise ValueError("bounded trusted candidate")
    count=(len(candidate)+511)//512
    return dict(operation="write",ordinal=count+1,effect="error",prefix=0)
def scan(records,candidate,selector):
    wanted=plan(candidate);count=wanted["ordinal"]-1
    if type(selector) is not bytes or len(selector)!=512:raise ValueError("exact trusted selector bytes")
    if type(records) is not list or not 1<=len(records)<=16390:raise ValueError("bounded System records")
    ready=records[0]
    if (type(ready) is not dict or set(ready)!={"type","kind","readonly","export_readonly","capacity","device","inode"} or
        ready["type"]!="ready" or ready["kind"]!="system" or ready["readonly"] is not False or
        ready["export_readonly"] is not False or type(ready["capacity"]) is not int or ready["capacity"]!=8388608 or
        type(ready["device"]) is not int or ready["device"]<0 or type(ready["inode"]) is not int or ready["inode"]<=0):
        raise ValueError("exact writable synthetic System descriptor")
    counts=dict(read=0,write=0,flush=0);pending=None;hit=None;terminal=False
    for index,row in enumerate(records[1:],1):
        if type(row) is not dict or terminal:raise ValueError("System record after terminal")
        if row.get("type")=="request":
            if hit is not None or pending is not None or set(row)!={"type","operation","offset","length"}:
                raise ValueError("one operation; no System retry after publication error")
            op,offset,length=row["operation"],row["offset"],row["length"]
            if (op not in counts or type(offset) is not int or type(length) is not int or
                (op=="flush" and (offset!=0 or length!=0)) or
                (op!="flush" and (offset<0 or offset%512 or not 512<=length<=65536 or length%512 or offset+length>8388608))):
                raise ValueError("fixed System geometry")
            if op=="write":
                n=counts["write"]
                if n>count or length!=512 or offset!=(B_START+n*512 if n<count else 512):
                    raise ValueError("only exact inactive payload then selector1")
                if n==count and counts["flush"]!=1:raise ValueError("candidate durability before publication")
            if op=="flush" and (counts["write"]!=count or counts["flush"]!=0):
                raise ValueError("one candidate flush before selector error")
            pending=(index,row)
        elif row.get("type")=="event":
            if hit is not None or pending is None or set(row)!={"type","event"}:raise ValueError("one pending System event")
            request_index,request=pending;op=request["operation"];event=row["event"]
            selected=op=="write" and counts["write"]==count
            fields={"operation","ordinal","offset","length","status"}|({"payload_sha256"} if op=="write" else set())|({"injection"} if selected else set())
            if (type(event) is not dict or set(event)!=fields or index!=request_index+1 or
                event["operation"]!=op or type(event["ordinal"]) is not int or event["ordinal"]!=counts[op]+1 or
                type(event["offset"]) is not int or event["offset"]!=request["offset"] or
                type(event["length"]) is not int or event["length"]!=request["length"]):
                raise ValueError("System event identity")
            if op=="write":
                digest=event["payload_sha256"]
                if type(digest) is not str or len(digest)!=64 or any(c not in "0123456789abcdef" for c in digest):
                    raise ValueError("System write hash")
                if selected and digest!=sha256(selector).hexdigest():raise ValueError("exact candidate selector publication bytes")
                if not selected:
                    part=candidate[counts["write"]*512:(counts["write"]+1)*512]
                    if digest!=sha256(part+bytes(512-len(part))).hexdigest():raise ValueError("actual candidate write bytes")
            if selected:
                if (type(event["injection"]) is not dict or event["injection"]!=wanted or
                    any(type(event["injection"].get(k)) is not type(v) for k,v in wanted.items()) or event["status"]!="failed-no-success"):
                    raise ValueError("exact selector write error, not success or cut")
                hit=dict(plan=wanted,request_index=request_index,event_index=index,offset=512,length=512)
            elif event["status"]!="completed":raise ValueError("unexpected System error")
            counts[op]+=1;pending=None
        elif row.get("type")=="terminal":
            if (hit is None or set(row)!={"type","outcome","fault_hit","failed"} or
                row["outcome"]!="failed" or row["fault_hit"] is not True or row["failed"] is not True):
                raise ValueError("exact failed transport termination")
            terminal=True
        else:raise ValueError("unknown System audit record")
    return None if hit is None else dict(hit,counts=counts,terminal=terminal)
def serial_status(serial,hit,final=False):
    if type(serial) is not bytes or len(serial)>65536 or not serial.isascii():raise ValueError("bounded guest serial")
    if any(s in serial for s in (b"UNEXPECTED-USER-FAULT",b"INVALID-USER-RETURN")):raise ValueError("guest isolation fault")
    at=serial.find(b"RAR-PANIC")
    if at<0:
        if final:raise ValueError("missing exact reconcile panic frame")
        return "running"
    tail=serial[at:]
    if (at!=0 and serial[at-1]!=10) or not PANIC.startswith(tail):
        # Bounded inert hex aids diagnosis without emitting guest control bytes.
        raise ValueError("unplanned guest panic frame: "+tail[:256].hex())
    if tail!=PANIC:
        if final:raise ValueError("incomplete reconcile panic frame")
        return "waiting"
    if hit is None:
        if final:raise ValueError("reconcile without exact System receipt")
        return "waiting"
    return "reconciled"
def joined(vm,candidate,selector,base):
    vm.service()
    if serial_status(bytes(vm.serial),vm.system_fault_hit,True)!="reconciled":raise ValueError("exact fault stop")
    stopped=vm.destroy()
    if (stopped.get("joined") is not True or vm.cleanup_succeeded is not True or vm.qmp_drained is not True or
        len(stopped.get("backends",[]))!=3):raise ValueError("whole VM/backends joined")
    summaries=[]
    for index,(backend,report,role) in enumerate(zip(vm.backends,stopped["backends"],("data","system","boot"))):
        if report.get("joined") is not True or report["records"]!=backend.records:raise ValueError("actual joined records")
        if index==1:
            summary=scan(report["records"],candidate,selector)
            if summary is None or any(summary[k]!=vm.system_fault_hit[k] for k in
                ("plan","request_index","event_index","offset","length")):raise ValueError("retained fault changed")
        else:summary=base.audit(report["records"],role,report["records"][0],True)
        if (type(report.get("returncode")) is not int or report["returncode"] not in (-9,21) or
            report.get("problem")!="backend-failed" or
            report["returncode"]==21 and not summary.get("terminal",summary.get("transport_closed",False))):
            raise ValueError("owned stop or complete transport EOF")
        summaries.append(summary)
    serial_status(bytes(vm.serial),summaries[1],True)
    return dict(cut=stopped,audit=summaries,system_fault=summaries[1],argv=vm.argv,preflight=vm.preflight,
        commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,qmp_drained=vm.qmp_drained,
        serial=bytes(vm.serial).decode("ascii"))
def self_test():
    candidate=b"x"*896;selector=b"s"*512;p=plan(candidate)
    ready=dict(type="ready",kind="system",readonly=False,export_readonly=False,capacity=8388608,device=1,inode=2)
    rows=[ready];counts=dict(read=0,write=0,flush=0)
    def emit(op,offset,length,status="completed",**extra):
        counts[op]+=1;rows.append(dict(type="request",operation=op,offset=offset,length=length))
        rows.append(dict(type="event",event=dict(operation=op,ordinal=counts[op],offset=offset,length=length,status=status,**extra)))
    for n in range(2):
        part=candidate[n*512:(n+1)*512];emit("write",B_START+n*512,512,payload_sha256=sha256(part+bytes(512-len(part))).hexdigest())
    emit("flush",0,0);emit("read",B_START,512)
    assert scan(rows,candidate,selector) is None
    emit("write",512,512,"failed-no-success",payload_sha256=sha256(selector).hexdigest(),injection=p)
    hit=scan(rows,candidate,selector);assert hit["offset"]==512 and hit["counts"]["write"]==3
    assert serial_status(PANIC,hit,True)=="reconciled"
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except ValueError:rejected+=1
        else:raise AssertionError("invalid fixed selector fault accepted")
    import copy
    for target,field,value in ((0,"kind","data"),(0,"readonly",True),(2,"status","failed-no-success"),
        (2,"offset",512),(2,"payload_sha256","b"*64),(len(rows)-1,"status","completed"),
        (len(rows)-1,"payload_sha256","a"*64),(len(rows)-1,"injection",dict(p,ordinal=1))):
        bad=copy.deepcopy(rows);entry=bad[target] if target==0 else bad[target]["event"];entry[field]=value
        reject(lambda bad=bad:scan(bad,candidate,selector))
    reject(lambda:scan(rows+[dict(type="request",operation="read",offset=0,length=512)],candidate,selector))
    for serial in (RECONCILE+b"\n",PANIC+b"extra",b"prefix"+PANIC,PANIC+PANIC,
        PANIC.replace(b"UPDATE-RECONCILE",b"OTHER"),PANIC.replace(b"BEGIN",b"WRONG"),
        PANIC.replace(b"HALT",b"WRONG"),b"UNEXPECTED-USER-FAULT",b""):
        reject(lambda serial=serial:serial_status(serial,hit,True))
    reject(lambda:serial_status(PANIC,None,True))
    assert serial_status(PANIC,None)=="waiting"
    assert serial_status(b"RAR-MODERN:GUI-READY\n"+PANIC,hit,True)=="reconciled"
    for n in range(len(b"RAR-PANIC"),len(PANIC)):
        assert serial_status(PANIC[:n],hit)=="waiting"
        reject(lambda n=n:serial_status(PANIC[:n],hit,True))
    return rejected
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated fixed fault self-test only")
    print("System selector fault:",self_test(),"negative fixtures; no VM evidence")
