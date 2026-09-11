"""Trusted-main signed composition and fixed three-VM scenario campaign.
Reuses the inspected existing cloud confinement; no direct VM process API.
"""
import os
CASES=("update","bad-health","bad-signature","bad-abi","selector-error")
def public_file(path,data):
    with path.open("xb") as out:
        out.write(data);out.flush();os.fchmod(out.fileno(),0o444)
def prepare(build,execute,compiler,source,work,evidence,report,save,preliminary,unknown=False):
    bank,system,record=build.composition["compose"](preliminary,report["source"],
        report["controller"],compiler[0],build.binary,build.packages,build.signer["sign_manifest_digest"])
    if type(unknown) is not bool:raise ValueError("fixed alternate fixture selection")
    if unknown:
        from pathlib import Path
        import importlib.util
        path=Path(__file__).resolve().with_name("unknown_publisher.py")
        spec=importlib.util.spec_from_file_location("unknown_publisher",path)
        alternate=importlib.util.module_from_spec(spec);spec.loader.exec_module(alternate)
        public,value=alternate.package(bank["modern-settings-update.layer"])
        name="modern-settings-bad-signature.layer"
        bank[name]=value
        record["packages"][name]={"sha256":build.digest(value),"bytes":len(value),
            "padded_bytes":(len(value)+4095)//4096*4096}
        record["unknown_publisher"]={"public_key":public.hex(),"slot":name,
            "enrolled":False,"signature_conformance_required":True}
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
            bank["modern-settings-factory.layer"],bank["modern-settings-"+("update" if case=="selector-error" else case)+".layer"])
        report["signed_runtime_checks"]=results;save()
    report["status"]="observed";save()
    print("Signed runtime: five fixed actual scenarios independently checked; alternate publisher and milestone review remain.",flush=True)

def observe_unknown(load,build,execute,compiler,launcher,source,controller,work,evidence,report,save,preliminary,sizes):
    # A second literal five-window composition, never a sixth native mapping.
    child_work=work/"unknown-publisher";child_work.mkdir(mode=0o700,exist_ok=False)
    child_evidence=evidence/"unknown-publisher";child_evidence.mkdir(mode=0o755,exist_ok=False)
    child=dict(source=report["source"],controller=report["controller"],status="started")
    report["unknown_publisher"]=child;save()
    selected,bank,system=prepare(build,execute,compiler,source,child_work,child_evidence,child,save,preliminary,True)
    inputs=child_work/"inputs";inputs.mkdir(mode=0o755,exist_ok=False)
    public_file(inputs/"modern.efi",selected["modern.efi"])
    for name,value in bank.items():public_file(inputs/name,value)
    public_file(inputs/"modern-system.img",system)
    raw=execute(compiler,["/bin/sh","/pack.sh"],
        [(build.HERE/"pack.sh","/pack.sh"),(controller/"nucleus/foundation/image.rs","/packager.rs"),(inputs,"/artifact")],
        120,24*1024*1024)
    boot=load("boot_image").unpack(raw,selected["modern.efi"])
    public_file(inputs/"boot.img",boot);public_file(child_evidence/"boot.img",boot)
    child["boot_sha256"]=build.digest(boot);save()
    print("Signed runtime fixed alternate unknown-publisher scenario",flush=True)
    raw=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/signed_runtime_launch.py","bad-signature"],
        [(inputs,"/artifact")],600,64*1024*1024)
    public_file(child_evidence/"unknown-publisher.json",raw)
    child["check"]=load("signed_runtime_validate").validate(raw,build.digest(boot),sizes,"bad-signature",
        bank["modern-settings-factory.layer"],bank["modern-settings-bad-signature.layer"])
    child["status"]="observed";save()
    print("Signed runtime: correctly signed non-enrolled publisher rejected in the actual guest.",flush=True)
