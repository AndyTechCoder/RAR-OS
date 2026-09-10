"""One fixed cloud corrupt-header scenario; no alternate device/profile options.
Runs only inside the existing disconnected disposable Modern launcher image.
"""
import base64
import json
import os
from pathlib import Path
import time

COMMANDS=("list","write note denied","list")

def capture(vm,oracle,stage,command=None,first=False):
    deadline=min(vm.deadline,time.monotonic()+12)
    for _ in range(24):
        vm.service();frame=vm.frame()
        try:digest=oracle.unavailable_validate(frame,stage,command,first)
        except ValueError:
            if time.monotonic()>=deadline:raise
            vm.delay(0.25);continue
        return dict(stage=stage,command=command,first=first,sha256=digest,
            actual_ppm=base64.b64encode(frame).decode("ascii"))
    raise TimeoutError("actual unavailable scene missing")

def run(session):
    session.cloud_guard()
    p=session.load("persistence");oracle=session.load("visual_oracle")
    disk=session.load("data_oracle")
    original=session.load("data_provision").Provisioner().fresh(os.urandom(68))
    initial=disk.inspect(original)
    if initial["revision"]!=0 or initial["files"]!={} or any(original[1024:]):
        raise ValueError("fresh empty public Data fixture")
    bad=bytearray(original);bad[0]^=1;bad[512]^=1;bad=bytes(bad)
    try:disk.inspect(bad)
    except ValueError:pass
    else:raise ValueError("corrupt headers unexpectedly accepted")
    root=Path("/tmp/rar-modern");root.mkdir(mode=0o700,exist_ok=False)
    boot=p.sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))
    data=system=vm=None;vms=[];frames=[]
    try:
        data=p.Fixture(root,"data",bad);system=p.Fixture(root,"system",bytes(8388608))
        vm=session.VM(1,data.fd,system.fd);vms.append(vm);vm.start();p.ready(vm)
        frames.append(capture(vm,oracle,"home"))
        vm.key("f3");frames.append(capture(vm,oracle,"terminal"))
        for index,command in enumerate(COMMANDS):
            for key in command:vm.key("spc" if key==" " else key)
            # The command-visible frame must precede Enter. Identical successive
            # error frames alone could match stale pixels before input handling.
            frames.append(capture(vm,oracle,"pending",command,index==0))
            vm.key("ret");frames.append(capture(vm,oracle,"unavailable"))
        vm.key("esc");vm.key("f1")
        frames.append(capture(vm,oracle,"files"))
        proof=p.joined(vm);vm=None
        frozen=data.freeze(vms)
        if (frozen!=bad or p.sha(system.freeze(vms))!=system.initial or
            p.sha(session.read_regular("/artifact/boot.img",16777216,exact=16777216))!=boot):
            raise ValueError("negative mount changed Data/System/boot")
        result=dict(schema="rar-modern-unavailable-candidate-v1",status="observed",
            original_data_base64=base64.b64encode(original).decode("ascii"),
            initial_data_base64=base64.b64encode(bad).decode("ascii"),
            frozen_data_base64=base64.b64encode(frozen).decode("ascii"),
            data_sha256=p.sha(frozen),system_sha256=system.initial,boot_sha256=boot,
            frames=frames,vm_proof=proof,milestone_complete=False)
        if len(json.dumps(result,separators=(",",":")).encode())>64*1024*1024:
            raise ValueError("bounded unavailable capture")
        return result
    finally:
        errors=[]
        if vm is not None and not vm.closed:
            try:vm.destroy()
            except BaseException as error:errors.append(type(error).__name__)
        for fixture in (system,data):
            if fixture is not None:
                try:fixture.close()
                except BaseException as error:errors.append(type(error).__name__)
        if errors:raise RuntimeError("unavailable cleanup failed: "+",".join(errors))
