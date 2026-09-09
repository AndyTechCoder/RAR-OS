"""Controller-owned interrupted Data publication scenarios; not activated.
No CLI. One case owns one fresh disposable cloud container and its two VMs.
All keys/data are public synthetic fixtures. No caller supplies paths or commands.
"""
import base64
import json
import os
from pathlib import Path
import time

def cases():
    result=[]
    for operation in ("write","flush"):
        for ordinal in range(1,7):
            for effect in ("before-cut","after-cut","error","torn-cut","short-error"):
                result.append(dict(operation=operation,ordinal=ordinal,effect=effect,
                    prefix=255 if effect in ("torn-cut","short-error") else 0,
                    reverse_flush=False))
    # The guest flushes each publication sector individually. Reverse ordering
    # is explicitly exercised, but is not a claim of multi-sector guest batching.
    for ordinal in range(1,7):
        result.append(dict(operation="flush",ordinal=ordinal,effect="after-cut",
                           prefix=0,reverse_flush=True))
    return result

def selected(index):
    if type(index) is not int or not 0<=index<len(cases()):
        raise ValueError("fixed reviewed fault case index")
    result=cases()[index];reverse=result.pop("reverse_flush")
    return result,reverse

def expected_revision(plan):
    """Three ordered sector publications per CREATE/WRITE, each with a flush.
    A nonempty durable commit marker selects the complete authenticated payload.
    No failed/before operation or merely volatile WRITE can publish new state.
    """
    old,stage=divmod(plan["ordinal"]-1,3)
    published=stage==2 and (
        plan["effect"] in ("torn-cut","short-error") or
        (plan["operation"]=="flush" and plan["effect"]=="after-cut"))
    return old+int(published)

def recovered_scene(vm,oracle,state,value):
    deadline=min(vm.deadline,time.monotonic()+12)
    for _ in range(24):
        vm.service();frame=vm.frame()
        try:digest=oracle.recovered_validate(frame,state,value if state=="written" else None)
        except ValueError:
            if time.monotonic()>=deadline:raise
            vm.delay(0.25);continue
        return dict(scene="recovered-"+state,sha256=digest,
                    actual_ppm=base64.b64encode(frame).decode("ascii"))
    raise TimeoutError("recovered guest Files did not match authenticated disk")

def run(session,index):
    session.cloud_guard()
    plan,reverse=selected(index)
    persistence=session.load("persistence")
    oracle=session.load("visual_oracle")
    disk_oracle=session.load("data_oracle")
    provision=session.load("data_provision").Provisioner()
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    empty=provision.fresh(os.urandom(68))
    initial=disk_oracle.inspect(empty)
    if initial["revision"]!=0 or initial["files"]!={} or any(empty[1024:]):
        raise ValueError("one virgin uniquely keyed Data fixture required")
    boot_digest=persistence.sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))
    data=system=live=None;vms=[];frames=[];proofs=[]
    try:
        data=persistence.Fixture(root,"data",empty)
        system=persistence.Fixture(root,"system",bytes(8388608))
        live=session.VM(1,data.fd,system.fd,data_fault=plan,reverse_flush=reverse)
        vms.append(live);live.start();persistence.ready(live)
        frames.append(persistence.scene(live,oracle,0))
        value=persistence.challenge(os.urandom(16))
        first,second=oracle.plan(value)
        if second!=["f1"]:raise ValueError("fresh boot receives only Files activation")
        live.key(first[0]);frames.append(persistence.scene(live,oracle,1))
        # No deliberate fault is valid before the submitted save reaches Data.
        try:
            for key in first[1:]:live.key(key)
            while True:live.service()  # Existing absolute VM deadline bounds this.
        except session.PlannedDataFault as error:
            if type(error) is not session.PlannedDataFault:raise
            proof=persistence.joined_fault(live,error)
            if proof["fault"]["plan"]!=plan:raise ValueError("observed fault differs from fixed case")
            proofs.append(proof);live=None
        if persistence.sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))!=boot_digest:
            raise ValueError("immutable boot changed")
        frozen=data.freeze(vms)
        if frozen[:1024]!=empty[:1024]:raise ValueError("immutable Data header changed")
        recovered=disk_oracle.inspect(frozen)
        revision=expected_revision(plan)
        expected=({}, {b"note":b""}, {b"note":value.encode("ascii")})[revision]
        if (recovered["revision"]!=revision or recovered["files"]!=expected or recovered["readonly"]):
            raise ValueError("fault boundary did not recover its exact complete old/new state")
        if persistence.sha(system.freeze(vms))!=system.initial:
            raise ValueError("System changed in Data-only fault scenario")
        live=session.VM(2,data.observer,system.fd,readonly_data=True)
        vms.append(live);live.start();persistence.ready(live)
        frames.append(persistence.scene(live,oracle,0))
        live.key("f1")
        frames.append(recovered_scene(live,oracle,("absent","empty","written")[revision],value))
        proofs.append(persistence.joined(live));live=None
        if (data.freeze(vms)!=frozen or persistence.sha(system.freeze(vms))!=system.initial or
            persistence.sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))!=boot_digest):
            raise ValueError("fresh read-only boot changed retained inputs")
        result=dict(schema="rar-modern-data-fault-candidate-v0",case=index,plan=plan,
            reverse_flush=reverse,challenge=value,frames=frames,vm_proofs=proofs,
            initial_data_sha256=persistence.sha(empty),frozen_data_sha256=persistence.sha(frozen),
            initial_data_base64=base64.b64encode(empty).decode("ascii"),
            frozen_data_base64=base64.b64encode(frozen).decode("ascii"),
            expected_revision=revision,system_sha256=system.initial,boot_sha256=boot_digest,
            status="observed-not-independently-accepted",milestone_complete=False)
        if len(json.dumps(result,separators=(",",":")).encode())>64*1024*1024:
            raise ValueError("bounded retained fault evidence")
        return result
    finally:
        errors=[]
        if live is not None and not live.closed:
            try:live.destroy()
            except BaseException as error:errors.append(type(error).__name__)
        for fixture in (system,data):
            if fixture is not None:
                try:fixture.close()
                except BaseException as error:errors.append(type(error).__name__)
        if errors:raise RuntimeError("fault scenario cleanup failed: "+",".join(errors))
