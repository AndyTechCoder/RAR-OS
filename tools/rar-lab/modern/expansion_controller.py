"""Concrete Expansion build, package and paired evidence controller.
Only the trusted-main Modern outer controller may supply the fixed sandbox API.
No direct Docker, QEMU, socket, network or host filesystem access is added.
"""
import re

def run(load,build,execute,image,compiler,source,controller,work,evidence,report,save,preliminary):
    signed=load("signed_runtime_controller")
    bank,system,record=build.composition["compose"](preliminary,report["source"],
        report["controller"],compiler[0],build.binary,build.packages,build.signer["sign_manifest_digest"])
    inputs=work/"expansion-inputs";inputs.mkdir(mode=0o755,exist_ok=False)
    for name,data in bank.items():signed.public_file(inputs/name,data)
    signed.public_file(inputs/"modern-system.img",system)
    report["signed_packages"]=record;report["expansion_builds"]={};save()
    boots={}
    for peer in ("a","b"):
        builds=[]
        for index in (1,2):
            print("Expansion independent UEFI build",peer,index,flush=True)
            # peer is an internal literal, never a workflow-supplied shell argument.
            raw=execute(compiler,["/bin/sh","-c",
                '/bin/sh /opt/rar-build-expansion.sh '+peer+' 2>/tmp/build.log; result=$?; if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi'],
                [(source,"/source"),(inputs,"/inputs")],300,6*1024*1024)
            built=build.composition["unpack_signed"](raw,build.binary["inspect"])
            placement=build.composition["inspect_bank"](built["modern.efi"],bank,build.binary["inspect"])
            builds.append(built)
            report["expansion_builds"][peer+str(index)]={"hashes":{n:build.digest(b) for n,b in built.items()},"bank":placement};save()
        if builds[0]!=builds[1]:raise ValueError("actual peer UEFI rebuild differs")
        directory=inputs/("peer-"+peer);directory.mkdir(mode=0o755,exist_ok=False)
        target=evidence/("peer-"+peer);target.mkdir(mode=0o755,exist_ok=False)
        for name,data in builds[0].items():signed.public_file(target/name,data)
        signed.public_file(directory/"modern.efi",builds[0]["modern.efi"])
        raw=execute(compiler,["/bin/sh","/pack.sh"],
            [(build.HERE/"pack.sh","/pack.sh"),(controller/"nucleus/foundation/image.rs","/packager.rs"),(directory,"/artifact")],
            120,24*1024*1024)
        boot=load("boot_image").unpack(raw,builds[0]["modern.efi"])
        signed.public_file(directory/"boot.img",boot);signed.public_file(target/"boot.img",boot)
        boots[peer]=build.digest(boot)
    if boots["a"]==boots["b"]:raise ValueError("different compiled peer identities required")
    report["boot_sha256"]=boots;report["system_sha256"]=build.digest(system)
    report["expansion_reproducible"]=True;save()
    launcher=image("expansion-launch.Containerfile","expansion-runtime")
    identities=execute(launcher,["/bin/sh","-c",
        "sha256sum /usr/bin/python3.11 /usr/bin/qemu-system-x86_64 /usr/share/OVMF/OVMF_CODE.fd /usr/share/OVMF/OVMF_VARS.fd; stat -c %s /usr/share/OVMF/OVMF_CODE.fd /usr/share/OVMF/OVMF_VARS.fd"],
        [],20,8192)
    lines=identities.decode("ascii").splitlines()
    paths=("/usr/bin/python3.11","/usr/bin/qemu-system-x86_64","/usr/share/OVMF/OVMF_CODE.fd","/usr/share/OVMF/OVMF_VARS.fd")
    if len(lines)!=6 or any(re.fullmatch("[0-9a-f]{64}  "+re.escape(p),s) is None for p,s in zip(paths,lines[:4])) or any(re.fullmatch("[0-9]+",s) is None for s in lines[4:]):
        raise ValueError("fixed tool identities and firmware geometry")
    sizes=tuple(int(s) for s in lines[4:])
    signed.public_file(evidence/"tool-identities.txt",identities)
    raw=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/expansion_launch.py"],
        [(inputs,"/artifact")],240,64*1024*1024)
    signed.public_file(evidence/"paired-network.json",raw)
    validator=load("expansion_evidence")
    report["paired_network"]=validator.validate(raw,boots,build.digest(system),sizes)
    report["actual_refusals"]=validator.actual_refusals(raw,boots,build.digest(system),sizes)
    report["evidence_sha256"]=build.digest(raw);report["status"]="observed";save()
    print("Expansion: actual bidirectional GUI network proof retained; wider M5 acceptance remains separate.",flush=True)
