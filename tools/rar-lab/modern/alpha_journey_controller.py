"""Same exact Alpha source and same observed Data through fixed System phases."""
def observe(load,build,execute,launcher,normal_inputs,boots,bank,system,sizes,owner,first,work,evidence,report,save):
    signed=load("signed_runtime_controller");validator=load("alpha_journey_evidence")
    base=load("runtime_evidence");previous=load("expansion_evidence").parse(first)
    factory,candidate,badsig=[bank["modern-settings-"+n+".layer"] for n in ("factory","update","bad-signature")]
    observed={};report["integrated_journey"]={}
    for mode in ("install","reject","fallback","repair","final"):
        expected_initial,_=validator.systems(factory,candidate,badsig,mode)
        # For subsequent phases use bytes from fully validated actual prior
        # receipts. Never substitute expected repair/install bytes for proof.
        current=[system,system]
        if mode in ("fallback","repair"):
            installed=load("expansion_evidence").parse(observed["install"])
            current=[base.decoded(x,8388608) for x in installed["frozen_system"]]
            if mode=="repair":
                damaged=bytearray(current[0])
                for offset in (1024,1024+load("signed_runtime_evidence").SLOT_BYTES):damaged[offset]^=1
                current[0]=bytes(damaged)
                report["system_fault"]={"scope":"owned synthetic System only","offsets":[1024,1024+load("signed_runtime_evidence").SLOT_BYTES],
                    "installed_sha256":base.sha(base.decoded(installed["frozen_system"][0],8388608)),"damaged_sha256":base.sha(current[0])}
        elif mode=="final":
            repaired=load("expansion_evidence").parse(observed["repair"])
            current=[base.decoded(x,8388608) for x in repaired["frozen_system"]]
        if current!=expected_initial:raise ValueError("actual System handoff differs")
        inputs=work/("journey-"+mode);inputs.mkdir(mode=0o755,exist_ok=False)
        for peer,index in (("a",0),("b",1)):
            directory=inputs/("peer-"+peer);directory.mkdir(mode=0o755,exist_ok=False)
            boot=(normal_inputs/("peer-"+peer)/"boot.img").read_bytes()
            if len(boot)!=16777216 or base.sha(boot)!=boots[peer]:raise ValueError("same exact reviewed Alpha boot")
            signed.public_file(directory/"boot.img",boot)
            signed.public_file(inputs/("system-"+peer+".img"),current[index])
            signed.public_file(inputs/("frozen-"+peer+".img"),base.decoded(previous["frozen_data"][index],99328))
        signed.public_file(inputs/"challenges.txt",("\n".join(previous["challenges"])+"\n").encode("ascii"))
        raw=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/alpha_journey_"+mode+"_launch.py"],
                    [(inputs,"/artifact")],240,64*1024*1024)
        signed.public_file(evidence/("journey-"+mode+".json"),raw)
        result=validator.validate(raw,boots,sizes,owner,bank,mode,first,observed.get("install"),observed.get("repair"))
        result["refusals"]=validator.actual_refusals(raw,boots,sizes,owner,bank,mode,first,observed.get("install"),observed.get("repair"))
        result["evidence_sha256"]=base.sha(raw);report["integrated_journey"][mode]=result
        observed[mode]=raw;save()
