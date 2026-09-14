"""Fixed cloud Alpha journey; no arbitrary command, path, image or network."""
import base64
import os
from pathlib import Path
import time

def run(session,mode):
    session.cloud_guard()
    if mode not in ("first","fresh"):raise ValueError("fixed Alpha phase")
    base=session.load("persistence");visual=session.load("alpha_visual");oracle=session.load("data_oracle")
    plan=session.load("alpha_plan")
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    system_bytes=session.read_regular("/artifact/modern-system.img",8388608,exact=8388608)
    boots={p:base.sha(session.read_regular("/artifact/peer-"+p+"/boot.img",16777216,exact=16777216)) for p in ("a","b")}
    paths={p:session.load("expansion_profile").capture_path(p) for p in ("a","b")}
    if any(os.path.lexists(p) for p in paths.values()):raise ValueError("fresh absent captures")
    fixtures=[];initial=[];frames=[];pair=None;started=time.monotonic()
    def capture(vm,peer,stage,value,compact):
        until=min(vm.deadline,time.monotonic()+12)
        for _ in range(24):
            vm.service();frame=vm.frame()
            try:sha=visual.validate(frame,stage,value,compact)
            except ValueError:
                if time.monotonic()>=until:raise
                vm.delay(.25);continue
            frames.append(dict(peer=peer,stage=stage,compact=compact,sha256=sha,actual_ppm=base64.b64encode(frame).decode("ascii")))
            return
        raise TimeoutError("Alpha scene deadline; no command retry")
    try:
        for peer in ("a","b"):
            directory=root/("fixture-"+peer);directory.mkdir(mode=0o700,exist_ok=False)
            data=(session.load("data_provision").Provisioner().fresh(os.urandom(68)) if mode=="first" else
                  session.read_regular("/artifact/frozen-"+peer+".img",99328,exact=99328))
            state=oracle.inspect(data)
            if mode=="first" and (state["revision"]!=0 or state["files"]!={} or any(data[1024:])):raise ValueError("virgin Data")
            fixtures.append(base.Fixture(directory,"data",data));fixtures.append(base.Fixture(directory,"system",system_bytes))
            initial.append(data)
        pair=session.load("expansion_session").Pair(tuple((fixtures[i].fd,fixtures[i+1].fd) for i in (0,2)))
        pair.start()
        for peer,vm in zip(("a","b"),pair.vms):
            base.ready(vm);capture(vm,peer,"home",None,False)
        if mode=="first":
            # Entropy is sampled only AFTER both actual GUI captures. No input
            # image, app binary or expected framebuffer contains these values.
            challenges=[base.challenge(os.urandom(16)) for _ in range(4)]
        else:
            raw=session.read_regular("/artifact/challenges.txt",132,exact=132)
            challenges=raw.decode("ascii").splitlines()
        steps=plan.plan(mode,challenges)
        for peer,keys,stage,value,compact in steps[2:]:
            vm=pair.vms[0 if peer=="a" else 1]
            for key in keys:vm.key(key)
            capture(vm,peer,stage,value,compact)
        pair.service();stopped=pair.destroy()
        frozen=[fixtures[i].freeze(pair.vms) for i in (0,2)]
        if any(fixtures[i].freeze(pair.vms)!=system_bytes for i in (1,3)):raise ValueError("native apps changed System")
        wire={p:session.read_regular(paths[p],8192) for p in ("a","b")}
        proofs=[]
        for peer,vm,cut in zip(("a","b"),pair.vms,stopped["guests"]):
            if base.sha(session.read_regular("/artifact/peer-"+peer+"/boot.img",16777216,exact=16777216))!=boots[peer]:raise ValueError("boot changed")
            audits=[]
            for backend,receipt,role in zip(vm.backends,cut["backends"],("data","system","boot")):
                if receipt.get("joined") is not True or receipt.get("records")!=backend.records:raise ValueError("joined block receipt")
                audits.append(base.audit(receipt["records"],role,backend.records[0]))
            proofs.append(dict(peer=peer,cut=cut,audit=audits,argv=vm.argv,preflight=vm.preflight,
                commands=vm.commands,events=vm.events,event_receipts=vm.event_receipts,
                qmp_drained=vm.qmp_drained,serial=bytes(vm.serial).decode("ascii")))
        elapsed=int((time.monotonic()-started)*1000)
        if not 0<elapsed<180000:raise ValueError("bounded Alpha duration")
        return dict(schema="rar-native-alpha-v1",mode=mode,status="observed",milestone_complete=False,
            challenges=challenges,frames=frames,vm_proofs=proofs,pair_cleanup=stopped,
            initial_data=[base64.b64encode(x).decode("ascii") for x in initial],
            frozen_data=[base64.b64encode(x).decode("ascii") for x in frozen],
            captured_wire={p:base64.b64encode(wire[p]).decode("ascii") for p in ("a","b")},
            boot_sha256=boots,system_sha256=base.sha(system_bytes),elapsed_milliseconds=elapsed)
    finally:
        failures=[]
        if pair is not None and not pair.closed:
            try:pair.destroy()
            except BaseException:failures.append("pair")
        for fixture in reversed(fixtures):
            try:fixture.close()
            except BaseException:failures.append("fixture")
        if failures:raise RuntimeError("Alpha cleanup failed: "+",".join(failures))
