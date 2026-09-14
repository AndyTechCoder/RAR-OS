"""Fixed negative campaigns using the existing confined pair and immutable inputs."""
def observe(load,build,execute,compiler,launcher,source,controller,work,evidence,report,save,bank,app_packages,system,normal_inputs,normal_boots,sizes):
    signed=load("signed_runtime_controller");validator=load("network_fault_evidence")
    report["network_campaigns"]={}
    for mode in ("faults","expiry","peer-stop"):
        inputs=work/("network-"+mode);inputs.mkdir(mode=0o755,exist_ok=False)
        target=evidence/("network-"+mode);target.mkdir(mode=0o755,exist_ok=False)
        for name,data in bank.items():signed.public_file(inputs/name,data)
        for name,data in app_packages.items():signed.public_file(inputs/name,data)
        signed.public_file(inputs/"modern-system.img",system)
        boots={};build_record={}
        for peer in ("a","b"):
            directory=inputs/("peer-"+peer);directory.mkdir(mode=0o755,exist_ok=False)
            if mode=="peer-stop":
                boot=(normal_inputs/("peer-"+peer)/"boot.img").read_bytes()
                if len(boot)!=16777216 or build.digest(boot)!=normal_boots[peer]:raise ValueError("validated normal boot copy")
            else:
                variants=[]
                for repetition in (1,2):
                    print("Closed-peer campaign build",mode,peer,repetition,flush=True)
                    raw=execute(compiler,["/bin/sh","-c",
                        '/bin/sh /opt/rar-build-network-case.sh '+peer+' '+mode+' 2>/tmp/build.log; result=$?; if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi'],
                        [(source,"/source"),(inputs,"/inputs")],300,6*1024*1024)
                    candidate=build.composition["unpack_signed"](raw,build.binary["inspect"])
                    build.composition["inspect_bank"](candidate["modern.efi"],bank,build.binary["inspect"])
                    if any(candidate["modern.efi"].count(pack)!=1 for pack in app_packages.values()):raise ValueError("exact immutable apps")
                    variants.append(candidate)
                if variants[0]!=variants[1]:raise ValueError("network case reproducibility")
                build_record[peer]={n:build.digest(v) for n,v in variants[0].items()}
                signed.public_file(directory/"modern.efi",variants[0]["modern.efi"])
                raw=execute(compiler,["/bin/sh","/pack.sh"],
                    [(build.HERE/"pack.sh","/pack.sh"),(controller/"nucleus/foundation/image.rs","/packager.rs"),(directory,"/artifact")],120,24*1024*1024)
                boot=load("boot_image").unpack(raw,variants[0]["modern.efi"])
            signed.public_file(directory/"boot.img",boot);boots[peer]=build.digest(boot)
        entry={"faults":"network_faults_launch.py","expiry":"network_expiry_launch.py","peer-stop":"network_peer_stop_launch.py"}[mode]
        raw=execute(launcher,["/usr/bin/python3","-I","-B","/opt/rar-modern/"+entry],[(inputs,"/artifact")],240,64*1024*1024)
        signed.public_file(target/"actual.json",raw)
        result=validator.validate(raw,boots,build.digest(system),sizes,mode)
        result["refusals"]=validator.actual_refusals(raw,boots,build.digest(system),sizes,mode)
        result["builds"]=build_record;result["evidence_sha256"]=build.digest(raw)
        report["network_campaigns"][mode]=result;save()
