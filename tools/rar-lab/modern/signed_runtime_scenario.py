"""Fixed signed-update scenario. Only called in the reviewed disposable cloud.
No owner paths, raw disks, arbitrary command, reset, network or retry API.
"""
import base64,json,os,time
from pathlib import Path
CASES={"update":"update","bad-health":"update badhealth",
       "bad-signature":"update badsig","bad-abi":"update badabi","selector-error":"update"}
def marker(vm,text):
    until=min(vm.deadline,time.monotonic()+25)
    while text.encode() not in vm.serial:
        vm.service()
        if time.monotonic()>=until:raise TimeoutError("signed transaction marker deadline")
        vm.delay(0.05)
def capture(vm,name,check):
    until=min(vm.deadline,time.monotonic()+12)
    for _ in range(24):
        vm.service();frame=vm.frame()
        try:digest=check(frame)
        except ValueError:
            if time.monotonic()>=until:raise
            vm.delay(0.25);continue
        return dict(scene=name,sha256=digest,actual_ppm=base64.b64encode(frame).decode("ascii"))
    raise TimeoutError("signed expected scene deadline")
def run(session,case):
    session.cloud_guard()
    if case not in CASES:raise ValueError("fixed signed test case")
    base=session.load("persistence");visual=session.load("visual_oracle")
    expected=session.load("signed_runtime_evidence");oracle=session.load("data_oracle")
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    read=session.read_regular
    system_bytes=read("/artifact/modern-system.img",8388608,exact=8388608)
    factory=read("/artifact/modern-settings-factory.layer",2097536)
    candidate=read("/artifact/modern-settings-"+("update" if case=="selector-error" else case)+".layer",2097536)
    # Initial System is supplied by the trusted matched-build controller, never
    # formatted from an existing image. Reconstruct its exact factory-only form.
    initial=bytearray(expected.expected_system(factory,candidate,"rejected"))
    b=1024+expected.SLOT_BYTES;initial[b:b+len(candidate)]=bytes(len(candidate))
    if bytes(initial)!=system_bytes:raise ValueError("exact matched factory System")
    empty=session.load("data_provision").Provisioner().fresh(os.urandom(68))
    if oracle.inspect(empty)["revision"]!=0 or any(empty[1024:]):raise ValueError("virgin Data")
    boot=base.sha(read("/artifact/boot.img",16777216,exact=16777216))
    data=system=live=None;vms=[];frames=[];proofs=[]
    try:
        data=base.Fixture(root,"data",empty);system=base.Fixture(root,"system",system_bytes)
        live=session.VM(1,data.fd,system.fd);vms.append(live);live.start();base.ready(live)
        frames.append(capture(live,"home-1",lambda f:visual.validate(f,0)))
        value=base.challenge(os.urandom(16));keys,_=visual.plan(value)
        live.key("f3")
        frames.append(capture(live,"terminal-1",lambda f:visual.validate(f,1)))
        for key in keys[1:]:live.key(key)
        frames.append(capture(live,"saved",lambda f:visual.validate(f,2,value)))
        proofs.append(base.joined(live));live=None
        frozen=data.freeze(vms)
        if frozen[:1024]!=empty[:1024]:raise ValueError("Data headers changed")
        state=oracle.inspect(frozen)
        if (state["files"]!={b"note":value.encode()} or state["revision"]!=2 or
            state["committed_slots"]!=[0,1] or state["burned_slots"]!=[]):
            raise ValueError("actual pre-update Data differs")
        if system.freeze(vms)!=system_bytes:raise ValueError("signed boot changed factory System")
        live=session.VM(2,data.observer,system.fd,readonly_data=True,
            system_selector_fault=candidate if case=="selector-error" else None)
        vms.append(live);live.start();base.ready(live)
        frames.append(capture(live,"home-2",lambda f:visual.validate(f,0)))
        live.key("f3")
        frames.append(capture(live,"terminal-2",lambda f:visual.validate(f,1)))
        for ch in CASES[case]:live.key("spc" if ch==" " else ch)
        live.key("ret")
        if case=="selector-error":
            fault=session.load("system_selector_fault")
            until=min(live.deadline,time.monotonic()+25)
            while fault.serial_status(bytes(live.serial),live.system_fault_hit)!="reconciled":
                live.service()
                if time.monotonic()>=until:raise TimeoutError("exact selector fault reconcile deadline")
            proofs.append(fault.joined(live,candidate,base));live=None
        else:
            marker(live,"RAR-MODERN:UPDATE-INSTALLED" if case=="update" else "RAR-MODERN:UPDATE-REJECTED")
            live.key("esc");live.key("f2")
            frames.append(capture(live,"candidate",lambda f:expected.settings_validate(f,visual,case=="update")))
            if case=="update":
                live.key("d")
                frames.append(capture(live,"compact",lambda f:expected.settings_validate(f,visual,True,True)))
                live.key("x");marker(live,"RAR-MODERN:UPDATE-FALLBACK")
                live.key("f2")
                frames.append(capture(live,"fallback",lambda f:expected.settings_validate(f,visual,False)))
            live.key("esc");live.key("f1")
            frames.append(capture(live,"files-2",lambda f:visual.validate(f,3,value)))
            proofs.append(base.joined(live,system_updates=True));live=None
        observed_system=system.freeze(vms)
        outcome="fallback" if case=="update" else "rejected"
        system_check=expected.validate_system(observed_system,factory,candidate,outcome)
        if data.freeze(vms)!=frozen:raise ValueError("update changed retained Data")
        live=session.VM(3,data.observer,system.fd,readonly_data=True)
        vms.append(live);live.start();base.ready(live)
        frames.append(capture(live,"home-3",lambda f:visual.validate(f,0)))
        live.key("f2")
        frames.append(capture(live,"selected-3",lambda f:expected.settings_validate(f,visual,False)))
        live.key("esc");live.key("f1")
        frames.append(capture(live,"files-3",lambda f:visual.validate(f,3,value)))
        if case=="update":
            live.key("esc");live.key("f3")
            frames.append(capture(live,"terminal-3",lambda f:visual.validate(f,1)))
            for ch in "update":live.key(ch)
            live.key("ret");marker(live,"RAR-MODERN:UPDATE-REJECTED")
            live.key("esc");live.key("f2")
            frames.append(capture(live,"stale-rejected",lambda f:expected.settings_validate(f,visual,False)))
        proofs.append(base.joined(live));live=None
        if data.freeze(vms)!=frozen or system.freeze(vms)!=observed_system:
            raise ValueError("fresh selected boot changed retained images")
        if base.sha(read("/artifact/boot.img",16777216,exact=16777216))!=boot:
            raise ValueError("immutable boot changed")
        result=dict(schema="rar-signed-runtime-candidate-v1",case=case,status="observed",
            challenge=value,frames=frames,vm_proofs=proofs,boot_sha256=boot,
            initial_data_sha256=base.sha(empty),frozen_data_sha256=base.sha(frozen),
            frozen_data_base64=base64.b64encode(frozen).decode(),
            system_base64=base64.b64encode(observed_system).decode(),system_check=system_check,
            milestone_complete=False)
        if len(json.dumps(result,separators=(",",":")).encode())>64*1024*1024:
            raise ValueError("bounded signed evidence")
        return result
    finally:
        errors=[]
        if live is not None and not live.closed:
            try:live.destroy()
            except BaseException as e:errors.append(type(e).__name__)
        for fixture in (system,data):
            if fixture is not None:
                try:fixture.close()
                except BaseException as e:errors.append(type(e).__name__)
        if errors:raise RuntimeError("signed scenario cleanup failed: "+",".join(errors))
