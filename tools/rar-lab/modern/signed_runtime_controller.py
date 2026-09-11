"""Trusted-main signed composition and fixed three-VM scenario campaign.
Reuses the inspected existing cloud confinement; no direct VM process API.
"""
import os
CASES=("update","bad-health","bad-signature","bad-abi")
def public_file(path,data):
    with path.open("xb") as out:
        out.write(data);out.flush();os.fchmod(out.fileno(),0o444)
def prepare(build,execute,compiler,source,work,evidence,report,save,preliminary):
    bank,system,record=build.composition["compose"](preliminary,report["source"],
        report["controller"],compiler[0],build.binary,build.packages,build.signer["sign_manifest_digest"])
    inputs=work/"signed-inputs";inputs.mkdir(mode=0o700,exist_ok=False)
    for name,data in bank.items():public_file(inputs/name,data)
    inputs.chmod(0o555)
    report["signed_packages"]=record;save();builds=[]
    for index in (1,2):
        print("Signed runtime independent build",index,flush=True)
        raw=execute(compiler,["/bin/sh","-c",
            '/bin/sh /opt/rar-build-signed.sh 2>/tmp/build.log; result=$?; if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi'],
            [(source,"/source"),(inputs,"/inputs")],300,6*1024*1024)
        built=build.composition["unpack_signed"](raw,build.binary["inspect"])
        placement=build.composition["inspect_bank"](built["modern.efi"],bank,build.binary["inspect"])
        builds.append(built)
        report["signed_build_"+str(index)]={n:build.digest(b) for n,b in built.items()}
        report["signed_bank_"+str(index)]=placement;save()
    if builds[0]!=builds[1] or report["signed_bank_1"]!=report["signed_bank_2"]:
        raise ValueError("actual signed build/bank reproducibility")
    for name,data in builds[0].items():public_file(evidence/("signed-"+name),data)
    public_file(evidence/"modern-system.img",system)
    report["signed_composition_reproducible"]=True;save()
    return builds[0],bank,system
def observe(load,execute,launcher,inputs,boot,sizes,evidence,report,save,bank):
    validator=load("signed_runtime_validate")
    results={}
    for case in CASES:
        print("Signed runtime fixed scenario",case,flush=True)
        raw=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/signed_runtime_launch.py",case],
            [(inputs,"/artifact")],600,64*1024*1024)
        public_file(evidence/("signed-"+case+".json"),raw)
        results[case]=validator.validate(raw,boot,sizes,case,
            bank["modern-settings-factory.layer"],bank["modern-settings-"+case+".layer"])
        report["signed_runtime_checks"]=results;save()
    report["status"]="observed";save()
    print("Signed runtime: four fixed actual scenarios independently checked; milestone acceptance requires review.",flush=True)
