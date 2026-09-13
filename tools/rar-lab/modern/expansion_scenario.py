"""Fixed actual two-guest network journey; activated only by reviewed cloud controller."""
import base64
import json
import os
from pathlib import Path
import time

def run(session):
    session.cloud_guard()
    base=session.load("persistence");visual=session.load("expansion_visual")
    data_oracle=session.load("data_oracle")
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    system_bytes=session.read_regular("/artifact/modern-system.img",8388608,exact=8388608)
    boot_hashes={p:base.sha(session.read_regular("/artifact/peer-"+p+"/boot.img",16777216,exact=16777216)) for p in ("a","b")}
    fixtures=[];initial=[];pair=None;frames=[];steps=[]
    def capture(vm,peer,stage,value=None):
        until=min(vm.deadline,time.monotonic()+12)
        for _ in range(24):
            vm.service();frame=vm.frame()
            try:digest=visual.validate(frame,stage,value)
            except ValueError:
                if time.monotonic()>=until:raise
                vm.delay(0.25);continue
            frames.append(dict(peer=peer,stage=stage,sha256=digest,
                actual_ppm=base64.b64encode(frame).decode("ascii")))
            return
        raise TimeoutError("network scene deadline; no command retry")
    def command(vm,peer,text,stage,value=None):
        for key in visual.keys(text):vm.key(key)
        capture(vm,peer,stage,value);steps.append(peer+":"+stage)
    started=time.monotonic()
    try:
        for peer in ("a","b"):
            directory=root/("fixture-"+peer);directory.mkdir(mode=0o700,exist_ok=False)
            empty=session.load("data_provision").Provisioner().fresh(os.urandom(68))
            state=data_oracle.inspect(empty)
            if state["revision"]!=0 or state["files"]!={} or any(empty[1024:]):
                raise ValueError("independent virgin Data fixtures")
            data=base.Fixture(directory,"data",empty);fixtures.append(data)
            system=base.Fixture(directory,"system",system_bytes);fixtures.append(system)
            initial.append(empty)
        pair=session.load("expansion_session").Pair(tuple((fixtures[i].fd,fixtures[i+1].fd) for i in (0,2)))
        pair.start();a,b=pair.vms
        for peer,vm in (("a",a),("b",b)):
            base.ready(vm);capture(vm,peer,"home")
            vm.key("f3");capture(vm,peer,"terminal")
        # Entropy exists only after both guests boot; neither disk contains it.
        left=base.challenge(os.urandom(16));right=base.challenge(os.urandom(16))
        if left==right:raise ValueError("challenge collision; no retry")
        command(a,"a","net send "+left,"queued")
        b.delay(0.5)
        command(b,"b","net recv","received",left)
        command(b,"b","net send "+right,"queued")
        a.delay(0.5)
        command(a,"a","net recv","received",right)
        command(a,"a","net close","closed")
        command(a,"a","net send "+left,"retired")
        a.key("esc");a.key("f1");capture(a,"a","files")
        pair.service()
        stopped=pair.destroy()
        if stopped.get("joined") is not True or stopped.get("network_closed") is not True or len(stopped.get("guests",[]))!=2:
            raise ValueError("joined whole pair required")
        proofs=[]
        for index,(peer,vm,cut) in enumerate(zip(("a","b"),pair.vms,stopped["guests"])):
            audits=[]
            for backend,receipt,role in zip(vm.backends,cut["backends"],("data","system","boot")):
                if receipt.get("joined") is not True or receipt.get("records")!=backend.records:
                    raise ValueError("actual reaped block records")
                audits.append(base.audit(receipt["records"],role,backend.records[0]))
            if fixtures[index*2].freeze(pair.vms)!=initial[index] or fixtures[index*2+1].freeze(pair.vms)!=system_bytes:
                raise ValueError("network journey changed Data or System")
            if base.sha(session.read_regular("/artifact/peer-"+peer+"/boot.img",16777216,exact=16777216))!=boot_hashes[peer]:
                raise ValueError("immutable boot changed")
            proofs.append(dict(peer=peer,cut=cut,audit=audits,argv=vm.argv,preflight=vm.preflight,
                commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,
                qmp_drained=vm.qmp_drained,serial=bytes(vm.serial).decode("ascii")))
        elapsed=time.monotonic()-started
        if not 0<elapsed<180:raise ValueError("bounded whole scenario duration")
        return dict(schema="rar-expansion-pair-v0",status="observed",challenges=[left,right],
            frames=frames,steps=steps,vm_proofs=proofs,pair_cleanup=stopped,
            initial_data=[base64.b64encode(x).decode("ascii") for x in initial],
            frozen_data=[base64.b64encode(fixtures[i].freeze(pair.vms)).decode("ascii") for i in (0,2)],
            data_unchanged=True,system_sha256=base.sha(system_bytes),boot_sha256=boot_hashes,
            elapsed_milliseconds=int(elapsed*1000),milestone_complete=False)
    finally:
        errors=[]
        if pair is not None and not pair.closed:
            try:pair.destroy()
            except BaseException:errors.append("pair teardown")
        for fixture in reversed(fixtures):
            try:fixture.close()
            except BaseException:errors.append("fixture close")
        if errors:raise RuntimeError("network scenario cleanup failed: "+",".join(errors))
