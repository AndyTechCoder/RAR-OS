"""Inert end-to-end outer-controller orchestration tests: no files or processes."""
import importlib.util
from pathlib import Path
from types import SimpleNamespace

def module(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def scenario(fault):
    files={};directories=set();calls=[];build_count=0;proof_count=0
    class P:
        def __init__(self,s):self.s=s
        def __truediv__(self,other):return P(self.s+"/"+other)
        def __str__(self):return self.s
        def mkdir(self,**kw):
            assert kw.get("exist_ok") is False and self.s not in directories
            directories.add(self.s)
    def public(path,data):
        assert str(path) not in files;files[str(path)]=data
    digest=lambda b:__import__("hashlib").sha256(b).hexdigest()
    bank={"modern-settings-factory.layer":b"factory"}
    def compose(*args):return bank,b"system",{"fixture":True}
    def unpack(raw,inspect):
        peer=raw.decode()
        if fault=="different-rebuild" and build_count==2:peer+="changed"
        return {"modern.efi":peer.encode(),"modern-service.efi":b"service"+peer.encode()}
    def pack(raw,pe):return b"boot"+pe
    def execute(tool,entry,mounts,seconds,limit):
        nonlocal build_count,proof_count
        calls.append((entry,[(str(p),d) for p,d in mounts],seconds,limit))
        assert seconds<=300 and limit<=64*1024*1024
        if fault=="call-"+str(len(calls)):raise RuntimeError("fixed injected execution failure")
        if entry[:2]==["/bin/sh","-c"] and "rar-build-expansion" in entry[2]:
            build_count+=1
            peer="a" if "rar-build-expansion.sh a " in entry[2] else "b"
            assert peer in ("a","b")
            assert [(str(p),d) for p,d in mounts]==[("/source","/source"),("/work/expansion-inputs","/inputs")]
            return peer.encode()
        if entry==["/bin/sh","/pack.sh"]:
            assert len(mounts)==3 and str(mounts[1][0])=="/controller/nucleus/foundation/image.rs"
            return b"packed"
        if entry[:2]==["/bin/sh","-c"] and entry[2].startswith("sha256sum"):
            paths=("/usr/bin/python3.11","/usr/bin/qemu-system-x86_64","/usr/share/OVMF/OVMF_CODE.fd","/usr/share/OVMF/OVMF_VARS.fd")
            return ("".join("a"*64+"  "+p+"\n" for p in paths)+"1966080\n131072\n").encode()
        assert entry==["/usr/bin/python3","-I","-B","/opt/rar-modern/expansion_launch.py"]
        assert [(str(p),d) for p,d in mounts]==[("/work/expansion-inputs","/artifact")]
        proof_count+=1
        return b"evidence"
    def validate(raw,boots,system,sizes):
        if fault=="evidence":raise ValueError("invalid observed proof")
        assert raw==b"evidence" and boots=={p:digest(b"boot"+p.encode()) for p in ("a","b")}
        assert system==digest(b"system") and sizes==(1966080,131072)
        return {"content_validated":True}
    def load(name):
        return {"signed_runtime_controller":SimpleNamespace(public_file=public),
            "boot_image":SimpleNamespace(unpack=pack),
            "expansion_evidence":SimpleNamespace(validate=validate,actual_refusals=lambda *a:13)}[name]
    def image(recipe,role):
        assert (recipe,role)==("expansion-launch.Containerfile","expansion-runtime")
        return ("runtime",[],{})
    build=SimpleNamespace(HERE=P("/controller/tools/rar-lab/modern"),digest=digest,
        composition={"compose":compose,"unpack_signed":unpack,"inspect_bank":lambda *a:{"checked":True}},
        binary={"inspect":lambda *a:None},packages={},signer={"sign_manifest_digest":lambda *a:None})
    report={"source":"a"*40,"controller":"b"*40,"status":"started"}
    failed=False
    try:module("expansion_controller").run(load,build,execute,image,("compiler",[],{}),
        P("/source"),P("/controller"),P("/work"),P("/evidence"),report,lambda:None,{})
    except (RuntimeError,ValueError):failed=True
    if fault:
        assert failed and report["status"]!="observed"
    else:
        assert not failed and report["status"]=="observed" and build_count==4 and proof_count==1
        assert len(calls)==8 and len(files)==14
        assert report["expansion_reproducible"] and report["actual_refusals"]==13
    if fault.startswith("call-"):assert len(calls)==int(fault[5:])
    return bool(fault)

def self_test():
    assert not scenario("")
    count=0
    for fault in ["call-"+str(i) for i in range(1,9)]+["different-rebuild","evidence"]:
        count+=scenario(fault)
    return count

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("inert controller self-test only")
    print("Expansion controller:",self_test(),"failure fixtures; no execution")
