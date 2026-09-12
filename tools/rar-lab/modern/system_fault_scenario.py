"""Inactive fixed System-fault scenario helpers; reviewed cloud wiring required.
No CLI, paths from callers, guest networking or owner-machine execution.
"""
import base64,json,os,time
from pathlib import Path

def joined_fault(vm,observation,base):
    receipt=vm.fault_receipt(observation)
    if getattr(vm,"fault_role",None)!="system":raise ValueError("System fault owner required")
    plan=receipt["plan"];cut=plan["effect"] in ("before-cut","after-cut","torn-cut")
    allowed=({"code":20,"problem":"cut","eof":True},) if cut else (
        {"code":None,"problem":None,"eof":False},
        {"code":21,"problem":"backend-failed","eof":True})
    if receipt["delivery"] not in allowed:raise ValueError("exact System fault delivery")
    stopped=vm.destroy()
    if (stopped.get("joined") is not True or vm.cleanup_succeeded is not True or
        vm.qmp_drained is not True or len(stopped.get("backends",[]))!=3 or
        type(stopped.get("vm_returncode")) is not int or stopped["vm_returncode"]!=-9):
        raise ValueError("whole live QEMU and all backend joins required")
    entry=stopped.get("entry")
    if (type(entry) is not dict or set(entry)!={"vm_code","backend_codes","backend_problems","event_count"} or
        entry["vm_code"] is not None or type(entry["event_count"]) is not int or
        not 1<=entry["event_count"]<=len(vm.events) or
        type(entry["backend_codes"]) is not list or len(entry["backend_codes"])!=3 or
        type(entry["backend_problems"]) is not list or len(entry["backend_problems"])!=3):
        raise ValueError("exact live teardown entry")
    for index in (0,2):
        if entry["backend_codes"][index] is not None or entry["backend_problems"][index] is not None:
            raise ValueError("unplanned peer failure before kill")
    code,problem=entry["backend_codes"][1],entry["backend_problems"][1]
    if cut:
        if type(code) is not int or code!=20 or problem!="cut":raise ValueError("cut entry mismatch")
    elif ((code is not None and (type(code) is not int or code!=21)) or
          problem not in (None,"backend-failed")):
        raise ValueError("System error entry mismatch")
    summaries=[]
    for index,(backend,report,role) in enumerate(zip(vm.backends,stopped["backends"],("data","system","boot"))):
        if report.get("joined") is not True or report.get("records")!=backend.records:
            raise ValueError("actual joined backend records")
        if index!=1:
            summary=base.audit(report["records"],role,backend.records[0],True)
            if (type(report.get("returncode")) is not int or report["returncode"] not in (-9,21) or
                report.get("problem")!="backend-failed" or
                (report["returncode"]==21 and summary["transport_closed"] is not True)):
                raise ValueError("read-only peer must end by kill or complete post-cut EOF")
            summaries.append(summary);continue
        matched=vm.fault_audit.scan(report["records"],plan,backend.records[0],role="system")
        if matched is None or any(matched[k]!=receipt[k] for k in
            ("request_index","event_index","offset","length")):
            raise ValueError("joined System audit differs from delivered fault")
        if cut:
            if (matched["terminal"] is not True or type(report.get("returncode")) is not int or
                report["returncode"]!=20 or report.get("problem")!="cut"):
                raise ValueError("exact System cut termination")
        elif (type(report.get("returncode")) is not int or report["returncode"] not in (-9,21) or
              report.get("problem")!="backend-failed" or
              (report["returncode"]==21 and matched["terminal"] is not True)):
            raise ValueError("exact System error transport termination")
        summaries.append(matched)
    return dict(cut=stopped,audit=summaries,fault=receipt,argv=vm.argv,preflight=vm.preflight,
        commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,
        qmp_drained=vm.qmp_drained,serial=bytes(vm.serial).decode("ascii"))

def run(session,mode,case):
    session.cloud_guard()
    base=session.load("persistence");visual=session.load("visual_oracle")
    expected=session.load("signed_runtime_evidence");oracle=session.load("data_oracle")
    scenario=session.load("signed_runtime_scenario")
    read=session.read_regular
    factory=read("/artifact/modern-settings-factory.layer",2097536)
    candidate=read("/artifact/modern-settings-update.layer",2097536)
    plans=expected.system_fault_cases(factory,candidate,mode)
    if type(case) is not int or not 0<=case<len(plans)<=256:
        raise ValueError("bounded fixed System campaign")
    plan=plans[case]
    system_bytes=read("/artifact/modern-system.img",8388608,exact=8388608)
    initial=bytearray(expected.expected_system(factory,candidate,"rejected"))
    initial[1024+expected.SLOT_BYTES:]=bytes(8388608-1024-expected.SLOT_BYTES)
    if bytes(initial)!=system_bytes:raise ValueError("exact matched factory System")
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    empty=session.load("data_provision").Provisioner().fresh(os.urandom(68))
    if oracle.inspect(empty)["revision"]!=0 or any(empty[1024:]):raise ValueError("virgin Data")
    boot=base.sha(read("/artifact/boot.img",16777216,exact=16777216))
    data=system=live=None;vms=[];frames=[];proofs=[];damage=None
    try:
        data=base.Fixture(root,"data",empty);system=base.Fixture(root,"system",system_bytes)
        live=session.VM(1,data.fd,system.fd);vms.append(live);live.start();base.ready(live)
        frames.append(scenario.capture(live,"home-1",lambda f:visual.validate(f,0)))
        value=base.challenge(os.urandom(16));keys,_=visual.plan(value)
        live.key("f3");frames.append(scenario.capture(live,"terminal-1",lambda f:visual.validate(f,1)))
        for key in keys[1:]:live.key(key)
        frames.append(scenario.capture(live,"saved",lambda f:visual.validate(f,2,value)))
        if mode=="repair":
            for ch in "update":live.key(ch)
            live.key("ret");scenario.marker(live,"RAR-MODERN:UPDATE-INSTALLED")
            live.key("esc");live.key("f2")
            frames.append(scenario.capture(live,"installed-1",lambda f:expected.settings_validate(f,visual,True)))
        proofs.append(base.joined(live,system_updates=mode=="repair"));live=None
        frozen_data=data.freeze(vms)
        state=oracle.inspect(frozen_data)
        if (frozen_data[:1024]!=empty[:1024] or state["files"]!={b"note":value.encode()} or
            state["revision"]!=2 or state["committed_slots"]!=[0,1] or state["burned_slots"]!=[]):
            raise ValueError("guest-created committed note required before System fault")
        if mode=="repair":
            installed,damaged,_=expected.repair_images(factory,candidate)
            if system.freeze(vms)!=installed:raise ValueError("guest installed image required")
            damage=system.damage_system_headers(vms,installed,damaged)
        elif system.freeze(vms)!=system_bytes:raise ValueError("factory System changed")
        if data.freeze(vms)!=frozen_data:raise ValueError("System preparation changed Data")
        before_fault=system.freeze(vms)
        live=session.VM(2,data.observer,system.fd,readonly_data=True,
            system_transaction_fault=(mode,case,factory,candidate))
        vms.append(live)
        try:
            live.start()
            if mode=="install":
                base.ready(live)
                frames.append(scenario.capture(live,"home-2",lambda f:visual.validate(f,0)))
                live.key("f3")
                frames.append(scenario.capture(live,"terminal-2",lambda f:visual.validate(f,1)))
                for ch in "update":live.key(ch)
                live.key("ret")
            while True:live.service()
        except session.PlannedSystemFault as error:
            if type(error) is not session.PlannedSystemFault:raise
            proof=joined_fault(live,error,base)
            if proof["fault"]["plan"]!=plan:raise ValueError("wrong observed System fault")
            proofs.append(proof);live=None
        frozen_system=system.freeze(vms)
        if frozen_system!=expected.system_fault_image(factory,candidate,mode,case):
            raise ValueError("actual fault disk differs from exact stable-byte oracle")
        if data.freeze(vms)!=frozen_data:raise ValueError("fault changed Data")
        restarted,updated,choice=expected.system_fault_restart(factory,candidate,mode,case)
        live=session.VM(3,data.observer,system.fd,readonly_data=True)
        vms.append(live);live.start();base.ready(live)
        frames.append(scenario.capture(live,"home-3",lambda f:visual.validate(f,0)))
        live.key("f2")
        frames.append(scenario.capture(live,"selected-3",lambda f:expected.settings_validate(f,visual,updated)))
        live.key("esc");live.key("f1")
        frames.append(scenario.capture(live,"files-3",lambda f:visual.validate(f,3,value)))
        proofs.append(base.joined(live,system_updates=mode=="repair"));live=None
        if system.freeze(vms)!=restarted or data.freeze(vms)!=frozen_data:
            raise ValueError("fresh boot did not preserve exact expected System/Data")
        if base.sha(read("/artifact/boot.img",16777216,exact=16777216))!=boot:
            raise ValueError("immutable boot changed")
        result=dict(schema="rar-system-fault-candidate-v0",mode=mode,case=case,plan=plan,
            challenge=value,frames=frames,vm_proofs=proofs,boot_sha256=boot,
            initial_data_sha256=base.sha(empty),frozen_data_sha256=base.sha(frozen_data),
            frozen_data_base64=base64.b64encode(frozen_data).decode(),
            before_system_base64=base64.b64encode(before_fault).decode(),
            frozen_system_base64=base64.b64encode(frozen_system).decode(),
            restarted_system_base64=base64.b64encode(restarted).decode(),
            system_corruption=damage,choice=choice,status="observed",milestone_complete=False)
        if len(json.dumps(result,separators=(",",":")).encode())>64*1024*1024:
            raise ValueError("bounded System fault evidence")
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
        if errors:raise RuntimeError("System fault cleanup failed: "+",".join(errors))
