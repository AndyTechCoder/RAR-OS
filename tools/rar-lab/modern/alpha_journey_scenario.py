"""Fixed cloud Alpha journey; no arbitrary command, path, image or network."""
import base64
import os
from pathlib import Path
import time

def run(session,mode):
    session.cloud_guard()
    if mode not in ("install","reject","fallback","repair","final"):raise ValueError("fixed Alpha phase")
    base=session.load("persistence");visual=session.load("alpha_journey_plan");oracle=session.load("data_oracle")
    plan=session.load("alpha_journey_plan")
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    system_bytes=[session.read_regular("/artifact/system-"+p+".img",8388608,exact=8388608) for p in ("a","b")]
    boots={p:base.sha(session.read_regular("/artifact/peer-"+p+"/boot.img",16777216,exact=16777216)) for p in ("a","b")}
    paths={p:session.load("expansion_profile").capture_path(p) for p in ("a","b")}
    if any(os.path.lexists(p) for p in paths.values()):raise ValueError("fresh absent captures")
    fixtures=[];initial=[];frames=[];transition_events=[];pair=None;started=time.monotonic()
    current={"phase":"setup"};last_difference=None
    def capture(vm,peer,stage,value,compact):
        nonlocal current,last_difference
        current={"phase":"capture","peer":peer,"stage":stage,"compact":compact}
        until=min(vm.deadline,time.monotonic()+12)
        for _ in range(24):
            vm.service();frame=vm.frame()
            try:sha=visual.validate(frame,stage,value,compact)
            except ValueError:
                last_difference={"actual_sha256":base.sha(frame),"stage":stage}
                if time.monotonic()>=until:raise
                vm.delay(.25);continue
            frames.append(dict(peer=peer,stage=stage,compact=compact,sha256=sha,actual_ppm=base64.b64encode(frame).decode("ascii")))
            return
        raise TimeoutError("Alpha scene deadline; no command retry")
    try:
        for index,peer in enumerate(("a","b")):
            directory=root/("fixture-"+peer);directory.mkdir(mode=0o700,exist_ok=False)
            data=session.read_regular("/artifact/frozen-"+peer+".img",99328,exact=99328)
            state=oracle.inspect(data)
            if state["revision"] not in (0,4):raise ValueError("validated retained Data")
            fixtures.append(base.Fixture(directory,"data",data));fixtures.append(base.Fixture(directory,"system",system_bytes[index]))
            initial.append(data)
        pair=session.load("expansion_session").Pair(tuple((fixtures[i].fd,fixtures[i+1].fd) for i in (0,2)))
        pair.start()
        for peer,vm in zip(("a","b"),pair.vms):
            base.ready(vm);capture(vm,peer,"home",None,False)
        raw=session.read_regular("/artifact/challenges.txt",132,exact=132)
        challenges=raw.decode("ascii").splitlines()
        steps=plan.plan(mode,challenges)
        for peer,keys,stage,value,compact in steps[2:]:
            vm=pair.vms[0 if peer=="a" else 1]
            wanted=plan.marker(stage)
            if wanted:
                current={"phase":"event","peer":peer,"stage":stage}
                if not keys:raise ValueError("explicit event trigger")
                for key in keys[:-1]:vm.key(key)
                vm.service();before=bytes(vm.serial)
                if plan.marker_lines(before,wanted):raise ValueError("pre-existing lifecycle event")
                vm.key(keys[-1]);trigger=vm.commands[-1]["id"]
                until=min(vm.deadline,time.monotonic()+25)
                while True:
                    vm.service();after=bytes(vm.serial)
                    matches=plan.marker_lines(after,wanted)
                    if len(matches)==1 and matches[0]>=len(before):break
                    if matches:raise ValueError("noncausal lifecycle event")
                    if time.monotonic()>=until:raise TimeoutError("integrated event "+stage)
                transition_events.append(dict(peer=peer,stage=stage,before_bytes=len(before),
                    before_sha256=base.sha(before),trigger_command_id=trigger,after_bytes=len(after)))
            else:
                for key in keys:vm.key(key)
                capture(vm,peer,stage,value,compact)
        pair.service();stopped=pair.destroy()
        frozen=[fixtures[i].freeze(pair.vms) for i in (0,2)]
        frozen_system=[fixtures[i].freeze(pair.vms) for i in (1,3)]
        if frozen!=initial:raise ValueError("integrated System journey changed Data")
        if frozen_system[1]!=system_bytes[1]:raise ValueError("idle peer System changed")
        wire={p:session.read_regular(paths[p],8192) for p in ("a","b")}
        proofs=[]
        for peer,vm,cut in zip(("a","b"),pair.vms,stopped["guests"]):
            if base.sha(session.read_regular("/artifact/peer-"+peer+"/boot.img",16777216,exact=16777216))!=boots[peer]:raise ValueError("boot changed")
            audits=[]
            for backend,receipt,role in zip(vm.backends,cut["backends"],("data","system","boot")):
                if receipt.get("joined") is not True or receipt.get("records")!=backend.records:raise ValueError("joined block receipt")
                audits.append(base.audit(receipt["records"],role,backend.records[0],system_updates=(peer=="a" and role=="system")))
            proofs.append(dict(peer=peer,cut=cut,audit=audits,argv=vm.argv,preflight=vm.preflight,
                commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,
                qmp_drained=vm.qmp_drained,serial=bytes(vm.serial).decode("ascii")))
        elapsed=int((time.monotonic()-started)*1000)
        if not 0<elapsed<180000:raise ValueError("bounded Alpha duration")
        return dict(schema="rar-alpha-system-journey-v1",mode=mode,status="observed",milestone_complete=False,
            challenges=challenges,frames=frames,transition_events=transition_events,vm_proofs=proofs,pair_cleanup=stopped,
            initial_data=[base64.b64encode(x).decode("ascii") for x in initial],
            frozen_data=[base64.b64encode(x).decode("ascii") for x in frozen],
            captured_wire={p:base64.b64encode(wire[p]).decode("ascii") for p in ("a","b")},
            boot_sha256=boots,system_sha256=[base.sha(b) for b in system_bytes],
            frozen_system=[base64.b64encode(b).decode("ascii") for b in frozen_system],elapsed_milliseconds=elapsed)
    except Exception as error:
        # Public synthetic fixture diagnostics only. This deliberately different
        # schema can NEVER validate as accepted runtime evidence. Teardown below
        # must still finish; the outer controller retains this before refusing.
        return dict(schema="rar-alpha-system-journey-failure-v1",status="failed",mode=mode,
            phase=current,reason=type(error).__name__+":"+str(error)[:512],
            difference=last_difference,completed_frames=len(frames),
            serial=[] if pair is None else [bytes(vm.serial)[-8192:].decode("ascii",errors="replace") for vm in pair.vms],
            milestone_complete=False)
    finally:
        failures=[]
        if pair is not None and not pair.closed:
            try:pair.destroy()
            except BaseException:failures.append("pair")
        for fixture in reversed(fixtures):
            try:fixture.close()
            except BaseException:failures.append("fixture")
        if failures:raise RuntimeError("Alpha cleanup failed: "+",".join(failures))
