"""Causal two-VM persistence scenario for the reviewed Modern cloud tool.
Candidate API only: no production CLI or workflow activation. All fixture files
are newly created inside disposable cloud /tmp; no owner or raw disk is exposed.
"""
import base64
import fcntl
import hashlib
import json
import os
from pathlib import Path
import stat
import time

def sha(data):
    return hashlib.sha256(data).hexdigest()

def challenge(entropy):
    if type(entropy) is not bytes or len(entropy)!=16:
        raise ValueError("exact public 128-bit random sample")
    return "".join(chr(97+(b>>4))+chr(97+(b&15)) for b in entropy)

def identity(fd,size,readonly):
    info=os.fstat(fd)
    flags=fcntl.fcntl(fd,fcntl.F_GETFL)
    if (not stat.S_ISREG(info.st_mode) or info.st_nlink!=1 or
        info.st_size!=size or info.st_uid!=65532 or flags&os.O_APPEND or
        flags&os.O_ACCMODE!=(os.O_RDONLY if readonly else os.O_RDWR)):
        raise ValueError("exclusive fixed-size owned cloud fixture descriptor")
    return (info.st_dev,info.st_ino,info.st_size)

class Fixture:
    """One exclusive regular file; no mutation API after empty provisioning."""
    def __init__(self,root,role,data):
        if role not in ("data","system") or type(data) is not bytes:
            raise ValueError("fixed fixture role")
        size={"data":99328,"system":8388608}[role]
        if len(data)!=size:
            raise ValueError("fixed fixture geometry")
        self.fd=self.observer=None
        self.role,self.size=role,size
        path=root/(role+".img")
        try:
            self.fd=os.open(path,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
            if os.write(self.fd,data)!=size:
                raise OSError("short empty fixture creation; no retry")
            os.fsync(self.fd)
            self.bound=identity(self.fd,size,False)
            self.observer=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
            if identity(self.observer,size,True)!=self.bound:
                raise ValueError("observer must reference the same retained inode")
            self.initial=sha(data)
        except BaseException:
            self.close()
            raise

    def freeze(self,vms):
        if not vms:
            raise ValueError("actual terminated VM ownership required")
        for vm in vms:
            if (not vm.closed or vm.cleanup_succeeded is not True or vm.qmp_drained is not True or vm.child is None or vm.child.poll() is None or
                len(vm.backends)!=3 or any(not b.closed or b.process.poll() is None for b in vm.backends)):
                raise ValueError("no frozen-image authority before every VM/backend reaped")
        if identity(self.fd,self.size,False)!=self.bound or identity(self.observer,self.size,True)!=self.bound:
            raise ValueError("retained fixture identity changed")
        data=os.pread(self.observer,self.size+1,0)
        if len(data)!=self.size or identity(self.observer,self.size,True)!=self.bound:
            raise ValueError("short or changed frozen image")
        return data

    def close(self):
        errors=[]
        for name in ("observer","fd"):
            fd=getattr(self,name,None)
            if fd is not None:
                try: os.close(fd)
                except OSError: errors.append(name)
                setattr(self,name,None)
        if errors: raise OSError("fixture descriptor cleanup failed")

def audit(records,role,ready,readonly_data=False):
    """Successful baseline only, not fault-injection/recovery acceptance.
    A deliberate whole-VM cut may interrupt one final READ; writes/flushes must
    have complete events. Raw audit records are retained alongside this summary.
    """
    if type(records) is not list or not 1<=len(records)<=16390 or records[0]!=ready:
        raise ValueError("exact backend readiness is required")
    if type(readonly_data) is not bool:
        raise ValueError("explicit Data authority mode")
    counts={"read":0,"write":0,"flush":0}
    dirty=False
    pending=None
    terminal=False
    size={"data":99328,"system":8388608,"boot":16777216}[role]
    physical_readonly=role=="boot" or (role=="data" and readonly_data)
    if (type(ready) is not dict or set(ready)!={"type","kind","readonly","export_readonly","capacity","device","inode"} or
        ready["type"]!="ready" or ready["kind"]!=role or ready["readonly"] is not physical_readonly or
        ready["export_readonly"] is not False or type(ready["capacity"]) is not int or ready["capacity"]!=size or
        type(ready["device"]) is not int or ready["device"]<0 or
        type(ready["inode"]) is not int or ready["inode"]<=0):
        raise ValueError("typed role and physical descriptor authority")
    for row in records[1:]:
        if type(row) is not dict or terminal:
            raise ValueError("record after terminal or non-object")
        kind=row.get("type")
        if kind=="request":
            if set(row)!={"type","operation","offset","length"} or pending is not None:
                raise ValueError("one complete request at a time")
            op,offset,length=row["operation"],row["offset"],row["length"]
            if op not in counts or type(offset) is not int or type(length) is not int:
                raise ValueError("canonical operation geometry")
            if (op=="flush" and (offset or length)) or (op!="flush" and
                (offset<0 or offset%512 or not 512<=length<=65536 or length%512 or offset+length>size)):
                raise ValueError("operation outside fixed device")
            if op=="write" and (role!="data" or readonly_data):
                raise ValueError("baseline may not attempt System/boot or read-only Data writes")
            pending=row
        elif kind=="event":
            if set(row)!={"type","event"} or pending is None or type(row["event"]) is not dict:
                raise ValueError("event without exactly one request")
            event=row["event"];op=pending["operation"]
            expected={"operation","ordinal","offset","length","status"}|({"payload_sha256"} if op=="write" else set())
            counts[op]+=1
            if (set(event)!=expected or event["operation"]!=op or
                type(event["ordinal"]) is not int or event["ordinal"]!=counts[op] or
                type(event["offset"]) is not int or event["offset"]!=pending["offset"] or
                type(event["length"]) is not int or event["length"]!=pending["length"] or
                event["status"]!="completed"):
                raise ValueError("failed, missing or mismatched baseline operation")
            if op=="write":
                digest=event["payload_sha256"]
                if type(digest) is not str or len(digest)!=64 or any(c not in "0123456789abcdef" for c in digest):
                    raise ValueError("canonical write payload digest")
            if op=="write": dirty=True
            elif op=="flush": dirty=False
            pending=None
        elif kind=="terminal":
            # EOF after deliberate QEMU SIGKILL is an ordinary transport failure,
            # not a device persistence failure; fault flags must remain false.
            if (set(row)!={"type","outcome","fault_hit","failed"} or
                row["outcome"]!="failed" or row["fault_hit"] is not False or row["failed"] is not False):
                raise ValueError("unexpected baseline terminal")
            terminal=True
        else:
            raise ValueError("unknown or repeated readiness record")
    if pending is not None and pending["operation"]!="read":
        raise ValueError("uncompleted mutation at persistence cut")
    if dirty:
        raise ValueError("completed volatile write lacks a completed durability flush")
    return dict(counts=counts,dirty=False,trailing_read=pending is not None,transport_closed=terminal)

def joined(vm):
    vm.service()
    stopped=vm.destroy()
    if (stopped.get("joined") is not True or vm.cleanup_succeeded is not True or
        vm.qmp_drained is not True or len(stopped.get("backends",[]))!=3):
        raise ValueError("whole VM and three backend joins required")
    summaries=[]
    for backend,report,role in zip(vm.backends,stopped["backends"],("data","system","boot")):
        if report["joined"] is not True or report["records"]!=backend.records:
            raise ValueError("actual joined backend records required")
        summaries.append(audit(report["records"],role,backend.records[0],vm.readonly_data))
    return dict(cut=stopped,audit=summaries,argv=vm.argv,preflight=vm.preflight,
                commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,
                qmp_drained=vm.qmp_drained,serial=bytes(vm.serial).decode("ascii"))

def joined_fault(vm,observation):
    """Stop after an exact planned signal; no generic exception can authorize it.
    This receipt alone is NOT crash-consistency acceptance. The caller must
    freeze after all joins and independently validate Data/Boot/System and UI.
    """
    receipt=vm.fault_receipt(observation)
    expected=receipt["plan"]
    cut=expected["effect"] in ("before-cut","after-cut","torn-cut")
    delivery=receipt["delivery"]
    if cut:
        if delivery!={"code":20,"problem":"cut","eof":True}:
            raise ValueError("complete cut evidence at signal delivery")
    else:
        if delivery!={"code":None,"problem":None,"eof":False}:
            raise ValueError("live error transport at signal delivery")
        # Error effects can leave the guest alive; check for any subsequent
        # unexpected event before the deliberate whole-VM stop.
        vm.service()
    stopped=vm.destroy()
    if (stopped.get("joined") is not True or vm.cleanup_succeeded is not True or
        vm.qmp_drained is not True or len(stopped.get("backends",[]))!=3):
        raise ValueError("whole VM and all backend joins required")
    summaries=[]
    for index,(backend,report,role) in enumerate(zip(vm.backends,stopped["backends"],
                                                   ("data","system","boot"))):
        if report.get("joined") is not True or report.get("records")!=backend.records:
            raise ValueError("actual joined backend records required")
        if index:
            summaries.append(audit(report["records"],role,backend.records[0],False))
            continue
        matched=vm.fault_audit.scan(report["records"],expected,backend.records[0])
        if matched is None or any(matched[key]!=receipt[key] for key in
            ("request_index","event_index","offset","length")):
            raise ValueError("joined records differ from delivered planned fault")
        if cut:
            if (matched["terminal"] is not True or type(report.get("returncode")) is not int or
                report["returncode"]!=20 or report.get("problem")!="cut"):
                raise ValueError("cut child exact final termination")
        elif (type(report.get("returncode")) is not int or report["returncode"] not in (-9,21) or
              report.get("problem")!="backend-failed"):
            raise ValueError("error child must end only with owned stop or transport EOF")
        summaries.append(matched)
    return dict(cut=stopped,audit=summaries,fault=receipt,argv=vm.argv,preflight=vm.preflight,
        commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,
        qmp_drained=vm.qmp_drained,serial=bytes(vm.serial).decode("ascii"))

def scene(vm,oracle,index,value=None):
    until=min(vm.deadline,time.monotonic()+12)
    # Capture polling is bounded and has no input/retry/reseal side effect.
    for _ in range(24):
        vm.service()
        frame=vm.frame()
        try:
            digest=oracle.validate(frame,index,value)
        except ValueError:
            if time.monotonic()>=until: raise
            vm.delay(0.25)
            continue
        return dict(scene=oracle.SCENES[index],sha256=digest,
                    actual_ppm=base64.b64encode(frame).decode("ascii"))
    raise TimeoutError("actual expected GUI scene did not become visible")

def ready(vm):
    until=min(vm.deadline,time.monotonic()+80)
    while b"RAR-MODERN:GUI-READY" not in vm.serial:
        if time.monotonic()>=until: raise TimeoutError("Modern GUI readiness deadline")
        vm.service()

def run(session):
    """Called only by the reviewed immutable cloud tool, not by source candidates.
    The outer trusted-main controller must verify tool/source/container identity
    and independently recheck retained evidence. This API grants no activation.
    """
    session.cloud_guard()
    oracle=session.load("visual_oracle")
    disk_oracle=session.load("data_oracle")
    provision=session.load("data_provision").Provisioner()
    root=Path("/tmp/rar-modern")
    root.mkdir(mode=0o700,exist_ok=False)
    # Exactly one empty image allocation; never seed challenge/file contents.
    empty=provision.fresh(os.urandom(68))
    initial=disk_oracle.inspect(empty)
    if initial["revision"]!=0 or initial["files"]!={} or any(empty[1024:]):
        raise ValueError("virgin Data fixture required")
    boot_digest=sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))
    data=system=None
    live=None
    vms=[]
    frames=[]
    proofs=[]
    try:
        data=Fixture(root,"data",empty)
        system=Fixture(root,"system",bytes(8388608))
        live=session.VM(1,data.fd,system.fd)
        vms.append(live)
        live.start();ready(live)
        frames.append(scene(live,oracle,0))
        # The entropy and command did not exist when VM1 or its disk was built.
        value=challenge(os.urandom(16))
        first,second=oracle.plan(value)
        if second!=["f1"]: raise ValueError("fresh VM must never receive challenge input")
        live.key(first[0])
        frames.append(scene(live,oracle,1))
        for key in first[1:]: live.key(key)
        frames.append(scene(live,oracle,2,value))
        proofs.append(joined(live));live=None
        if sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))!=boot_digest:
            raise ValueError("immutable boot bytes changed")
        frozen=data.freeze(vms)
        if frozen[:1024]!=empty[:1024]:
            raise ValueError("immutable Data headers changed")
        result=disk_oracle.inspect(frozen)
        if (result["files"]!={b"note":value.encode("ascii")} or result["revision"]!=2 or
            result["committed_slots"]!=[0,1] or result["burned_slots"]!=[] or
            result["next_slot"]!=2 or result["readonly"]):
            raise ValueError("actual durable create/write state differs from Terminal")
        if sha(system.freeze(vms))!=system.initial:
            raise ValueError("System changed during Data-only proof")
        live=session.VM(2,data.observer,system.fd,readonly_data=True)
        vms.append(live)
        live.start();ready(live)
        frames.append(scene(live,oracle,0))
        for key in second: live.key(key)
        frames.append(scene(live,oracle,3,value))
        proofs.append(joined(live));live=None
        if sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))!=boot_digest:
            raise ValueError("immutable boot bytes changed")
        if data.freeze(vms)!=frozen or sha(system.freeze(vms))!=system.initial:
            raise ValueError("fresh read-only workload changed retained images")
        output=dict(schema="rar-modern-persistence-candidate-v1",status="observed",
            challenge=value,frames=frames,vm_proofs=proofs,
            initial_data_sha256=sha(empty),frozen_data_sha256=sha(frozen),
            frozen_data_base64=base64.b64encode(frozen).decode("ascii"),
            system_sha256=system.initial,boot_sha256=boot_digest,crypto_interoperability_accepted=False,
            milestone_complete=False)
        if len(json.dumps(output,separators=(",",":")).encode())>64*1024*1024:
            raise ValueError("bounded retained persistence evidence")
        return output
    finally:
        errors=[]
        if live is not None and not live.closed:
            try: live.destroy()
            except BaseException as error: errors.append(type(error).__name__)
        for fixture in (system,data):
            if fixture is not None:
                try: fixture.close()
                except BaseException as error: errors.append(type(error).__name__)
        if errors: raise RuntimeError("persistence cleanup failed: "+",".join(errors))

def self_test():
    # Pure byte/record tests only; no scenario/fixture/VM execution.
    assert challenge(bytes(range(16)))=="aaabacadaeafagahaiajakalamanaoap"
    rejected=0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected+=1
        else: raise AssertionError("invalid persistence proof accepted")
    for value in (None,b"",bytes(15),bytes(17),bytearray(16)):
        reject(lambda value=value:challenge(value))
    ready_record={"type":"ready","kind":"data","readonly":False,"export_readonly":False,
                  "capacity":99328,"device":1,"inode":2}
    req={"type":"request","operation":"write","offset":1024,"length":512}
    event={"type":"event","event":{"operation":"write","ordinal":1,"offset":1024,
        "length":512,"status":"completed","payload_sha256":"a"*64}}
    flush={"type":"request","operation":"flush","offset":0,"length":0}
    flushed={"type":"event","event":{"operation":"flush","ordinal":1,"offset":0,"length":0,"status":"completed"}}
    good=[ready_record,req,event,flush,flushed]
    assert audit(good,"data",ready_record)["counts"]["write"]==1
    for rows in ([],[ready_record,req],[ready_record,event],[ready_record,req,req],
                 good+[{"type":"unknown"}],good+[dict(ready_record)]):
        reject(lambda rows=rows:audit(rows,"data",ready_record))
    for field,value in (("ordinal",2),("status","failed-no-success"),("payload_sha256","bad"),
                        ("offset",0),("length",1024)):
        changed=json.loads(json.dumps(good));changed[2]["event"][field]=value
        reject(lambda changed=changed:audit(changed,"data",ready_record))
    for role in ("system","boot"):
        role_ready=dict(ready_record,kind=role,readonly=role=="boot",
                        capacity={"system":8388608,"boot":16777216}[role])
        reject(lambda role=role,role_ready=role_ready:audit([role_ready]+good[1:],role,role_ready))
    trailing=[ready_record,{"type":"request","operation":"read","offset":0,"length":512}]
    assert audit(trailing,"data",ready_record)["trailing_read"]
    reject(lambda:audit([ready_record,req,event],"data",ready_record))
    second_event=json.loads(json.dumps(event));second_event["event"]["ordinal"]=2
    reject(lambda:audit(good+[req,second_event],"data",ready_record))
    ro_ready=dict(ready_record,readonly=True)
    assert audit([ro_ready],"data",ro_ready,True)["dirty"] is False
    reject(lambda:audit([ro_ready,req,event,flush,flushed],"data",ro_ready,True))
    reject(lambda:audit([ready_record],"data",ready_record,True))
    # Failed cleanup/drain never authorizes a frozen read, even after closure.
    from types import SimpleNamespace
    fixture=object.__new__(Fixture)
    for cleanup,drained in ((False,True),(True,False),(1,True),(True,1)):
        vm=SimpleNamespace(closed=True,cleanup_succeeded=cleanup,qmp_drained=drained,child=None)
        reject(lambda vm=vm:fixture.freeze([vm]))
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("pure self-test only; cloud scenario has no activated CLI")
    print("Modern persistence scenario:",self_test(),"negative fixtures; no VM execution")
