"""Fixed Foundation node build/result evidence. No target execution in tests."""
import base64
import hashlib
import re
def unpack(raw,inspect):
    if type(raw) is not bytes or not 1<=len(raw)<=400000:raise ValueError("bounded node build")
    lines=raw.splitlines()
    if len(lines)!=3 or lines[0]!=b"RAR-NODE-FILE:node.efi" or lines[2]!=b"RAR-NODE-BUILD:END":raise ValueError("node framing")
    data=base64.b64decode(lines[1],validate=True)
    if not 512<=len(data)<=262144:raise ValueError("node executable budget")
    inspect(data,False);return data
def validate(raw):
    if type(raw) is not bytes or not 1<=len(raw)<=65536:raise ValueError("bounded actual serial")
    if any(x in raw for x in (b"RAR-PANIC",b"UNEXPECTED-USER-FAULT",b"INVALID-USER-RETURN")):raise ValueError("node failed")
    for marker in (b"RAR-BOOT:UEFI",b"RAR-KERNEL:ENTRY",b"RAR-MEMORY:READY",b"RAR-ALLOCATOR:READY",
                   b"RAR-INTERRUPTS:READY",b"RAR-TIMER:READY",b"RAR-FOUNDATION-READY",b"RAR-NODE:COMPLETE",b"RAR-NODE:CLOUD-STOPPED=124"):
        if raw.count(marker+b"\n")!=1:raise ValueError("one complete Foundation/node lifecycle")
    values=[]
    for key in ("SAMPLE","RESULT","STEPS","STATE-BYTES","ARENA-BYTES"):
        matches=re.findall(rb"(?:^|\n)RAR-NODE:"+key.encode()+rb"=([0-9a-f]{8})\n",raw)
        if len(matches)!=1:raise ValueError("actual node numeric marker")
        values.append(int(matches[0],16))
    sample,result,steps,state,arena=values
    if not 3<=sample<=2**32-8 or result!=sample+7 or steps!=5 or not 1<=state<=128 or arena!=4*1024*1024:raise ValueError("portable script result/resource bounds")
    return dict(serial_sha256=hashlib.sha256(raw).hexdigest(),script_steps=steps,state_bytes=state,
                foundation_arena_bytes=arena,firmware_vm_ram_bytes=256*1024*1024,
                target="x86_64-cloud-node",mcu_port_claimed=False,milestone_complete=False)
def observe(load,build,execute,compiler,launcher,source,controller,work,evidence,report,save):
    signed=load("signed_runtime_controller");outputs=[]
    for _ in (1,2):
        raw=execute(compiler,["/bin/sh","-c",
            '/bin/sh /opt/rar-build-node.sh 2>/tmp/build.log; result=$?; if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi'],
            [(source,"/source")],300,400000)
        outputs.append(unpack(raw,build.binary["inspect"]))
    if outputs[0]!=outputs[1]:raise ValueError("node rebuild differs")
    inputs=work/"node-inputs";inputs.mkdir(mode=0o755,exist_ok=False)
    signed.public_file(inputs/"modern.efi",outputs[0]);signed.public_file(evidence/"node.efi",outputs[0])
    raw=execute(compiler,["/bin/sh","/pack.sh"],
        [(build.HERE/"pack.sh","/pack.sh"),(controller/"nucleus/foundation/image.rs","/packager.rs"),(inputs,"/artifact")],120,24*1024*1024)
    boot=load("boot_image").unpack(raw,outputs[0])
    signed.public_file(inputs/"boot.img",boot);signed.public_file(evidence/"node-boot.img",boot)
    raw=execute(launcher,["/bin/sh","/opt/rar-node-launch.sh"],[(inputs,"/artifact")],35,65536)
    signed.public_file(evidence/"node-serial.txt",raw)
    report["node"]=validate(raw);report["node"]["reproducible"]=True
    report["node"]["executable_bytes"]=len(outputs[0]);report["node"]["boot_sha256"]=build.digest(boot);save()
def self_test():
    raw=b"\n".join([b"RAR-BOOT:UEFI",b"RAR-KERNEL:ENTRY",b"RAR-MEMORY:READY",b"RAR-ALLOCATOR:READY",
        b"RAR-INTERRUPTS:READY",b"RAR-TIMER:READY",b"RAR-FOUNDATION-READY",b"RAR-NODE:SAMPLE=00000003",
        b"RAR-NODE:RESULT=0000000a",b"RAR-NODE:STEPS=00000005",b"RAR-NODE:STATE-BYTES=00000040",
        b"RAR-NODE:ARENA-BYTES=00400000",b"RAR-NODE:COMPLETE",b"RAR-NODE:CLOUD-STOPPED=124"])+b"\n"
    assert validate(raw)["state_bytes"]==64
    rejected=0
    for bad in (raw+b"RAR-PANIC\n",raw.replace(b"0000000a",b"0000000b"),
                raw.replace(b"00400000",b"00000004"),raw.replace(b"RAR-NODE:COMPLETE",b"missing"),
                raw+raw,raw[:-1]):
        try:validate(bad)
        except ValueError:rejected+=1
        else:raise AssertionError("invalid node result accepted")
    return rejected
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure tests only")
    print("Node evidence:",self_test(),"refusals; no target execution")
