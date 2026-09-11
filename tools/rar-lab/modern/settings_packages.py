"""Pure public-laboratory Settings package/System fixture construction.
Construction APIs perform no I/O; self-tests read fixed sibling source helpers.
No CLI signing, production key, installation, VM launch or target execution.
The trusted cloud controller supplies already-built bytes and exact provenance.
"""
from hashlib import sha256
import re

SIZE=384
SYSTEM_BYTES=8_388_608
SLOT_BYTES=4097*512
VARIANTS={"factory":(1,1),"update":(2,1),"bad-health":(3,1),
          "bad-signature":(2,1),"bad-abi":(2,0)}
PUBLIC_KEY=bytes.fromhex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")

def _put(out,offset,size,value):
    out[offset:offset+size]=value.to_bytes(size,"little")

def build(payload,source_sha,build_digest,variant,inspect,sign):
    if (type(payload) is not bytes or not 512<=len(payload)<=2_097_152 or
        type(source_sha) is not str or re.fullmatch("[0-9a-f]{40}",source_sha) is None or
        source_sha=="0"*40 or type(build_digest) is not bytes or
        len(build_digest)!=32 or build_digest==bytes(32) or variant not in VARIANTS):
        raise ValueError("bounded exact public-laboratory build inputs")
    layout=inspect(payload,True)
    image=layout["image_bytes"]
    if type(image) is not int or not 4096<=image<=128*1024 or image%4096:
        raise ValueError("exact service mapping budget")
    generation,abi=VARIANTS[variant]
    pre=bytearray(288)
    pre[:8]=b"RARMODL0"
    pre[16:40]=b"rar.alpha.ed25519.v0\0\0\0\0"
    for offset,size,value in ((10,2,384),(12,4,1),(40,4,5),(44,2,1),(48,4,1),
        (56,4,len(payload)),(60,4,image),(64,8,7),(72,8,generation),
        (228,4,abi),(232,4,50),(240,4,16384)):
        _put(pre,offset,size,value)
    pre[80:112]=sha256(b"RAR-MODERN-SETTINGS-HEALTH-V0\0").digest()
    pre[112:144]=sha256(payload).digest()
    pre[144:176]=sha256(PUBLIC_KEY).digest()
    pre[176:196]=bytes.fromhex(source_sha)
    pre[196:228]=build_digest
    pre=bytes(pre)
    digest=sha256(pre).digest()
    signature=sign(digest)
    if type(signature) is not bytes or len(signature)!=64:
        raise ValueError("exact laboratory signature")
    if variant=="bad-signature":
        signature=signature[:-1]+bytes([signature[-1]^1])
    return pre+digest+signature+payload

def factory_system(factory,source_sha,build_digest,inspect,sign):
    """Return newly provisioned bytes, never mutate an existing device/image.

    Equality to a freshly constructed canonical factory package is framing/
    fixture consistency, not independent signature or malicious-publisher proof.
    Native manager verification and the separate immutable boot copy remain
    mandatory. The function has no Data input or output.
    """
    if type(factory) is not bytes or not 896<=len(factory)<=2_097_536:
        raise ValueError("exact factory package")
    expected=build(factory[SIZE:],source_sha,build_digest,"factory",inspect,sign)
    if factory!=expected:
        raise ValueError("canonical factory and provenance must match")
    record=bytearray(512);record[:8]=b"RARSYS00"
    _put(record,10,2,512);record[14]=255
    for offset in (16,24,32,40):
        _put(record,offset,8,1)
    record[64:96]=factory[288:320]
    record[480:]=sha256(record[:480]).digest()
    result=bytearray(SYSTEM_BYTES)
    result[:512]=record
    result[1024:1024+len(factory)]=factory
    # Selector1, all unused A/B slot bytes and reserved tail remain explicit zero.
    return bytes(result)

def conformance_fixture():
    """Fixed synthetic codec bytes for the cloud Rust verifier, never an app."""
    from pathlib import Path
    import runpy
    here=Path(__file__).resolve().parent
    signer=runpy.run_path(str(here/"lab_signer.py"))
    binary=runpy.run_path(str(here/"build_binary.py"))
    raw=bytearray(signer["codec_test_fixture"]()[384:1408])
    _put(raw,88+68,2,10)
    payload=bytes(raw);source="a"*40;provenance=bytes([3])*32
    inspect,sign=binary["inspect"],signer["sign_manifest_digest"]
    variants=("factory","update","bad-health","bad-signature","bad-abi")
    packages=[build(payload,source,provenance,v,inspect,sign) for v in variants]
    image=factory_system(packages[0],source,provenance,inspect,sign)
    return b"".join(packages)+image

def self_test():
    # Isolated cloud source tests only. This imports reviewed sibling helpers,
    # never source-selected code paths or executable artifacts.
    from pathlib import Path
    here=Path(__file__).resolve().parent
    signer={"__name__":"fixture_signer"}
    binary={"__name__":"fixture_binary"}
    exec(compile((here/"lab_signer.py").read_text(),"lab_signer.py","exec"),signer)
    exec(compile((here/"build_binary.py").read_text(),"build_binary.py","exec"),binary)
    raw=bytearray(signer["codec_test_fixture"]()[384:1408])
    _put(raw,88+68,2,10)  # Synthetic UEFI header, never executed.
    payload=bytes(raw);source="a"*40;provenance=bytes([3])*32
    inspect,sign=binary["inspect"],signer["sign_manifest_digest"]
    packages={v:build(payload,source,provenance,v,inspect,sign)for v in VARIANTS}
    assert len(set(packages.values()))==5
    for v,(generation,abi)in VARIANTS.items():
        p=packages[v];assert p[384:]==payload
        assert int.from_bytes(p[72:80],"little")==generation
        assert int.from_bytes(p[228:232],"little")==abi
        assert p[176:196]==bytes.fromhex(source) and p[196:228]==provenance
        assert p[288:320]==sha256(p[:288]).digest()
        assert (p[320:384]==sign(p[288:320]))==(v!="bad-signature")
    image=factory_system(packages["factory"],source,provenance,inspect,sign)
    assert len(image)==SYSTEM_BYTES
    assert image[:8]==b"RARSYS00" and image[480:512]==sha256(image[:480]).digest()
    assert image[512:1024]==bytes(512)
    assert image[1024:1024+len(packages["factory"])]==packages["factory"]
    assert not any(image[1024+len(packages["factory"]):])
    assert image==factory_system(packages["factory"],source,provenance,inspect,sign)
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except (ValueError,KeyError,TypeError):rejected+=1
        else:raise AssertionError("invalid fixture input accepted")
    for source_bad in ("main","0"*40,"A"*40,"a"*39,"a"*41,"a"*40+"\n",None):
        reject(lambda source_bad=source_bad:build(payload,source_bad,provenance,"factory",inspect,sign))
    for bad in (b"",bytes(31),bytes(32),bytes(33),bytearray(provenance),None):
        reject(lambda bad=bad:build(payload,source,bad,"factory",inspect,sign))
    for variant in ("unknown","",None):
        reject(lambda variant=variant:build(payload,source,provenance,variant,inspect,sign))
    for value in (b"",payload[:511],bytearray(payload),b"x"*1024):
        reject(lambda value=value:build(value,source,provenance,"factory",inspect,sign))
    for variant in ("update","bad-health","bad-signature","bad-abi"):
        reject(lambda variant=variant:factory_system(packages[variant],source,provenance,inspect,sign))
    for offset in (0,72,112,176,196,288,320,383,384,len(packages["factory"])-1):
        bad=bytearray(packages["factory"]);bad[offset]^=1
        reject(lambda bad=bytes(bad):factory_system(bad,source,provenance,inspect,sign))
    reject(lambda:factory_system(packages["factory"],"b"*40,provenance,inspect,sign))
    reject(lambda:factory_system(packages["factory"],source,bytes([4])*32,inspect,sign))
    return rejected

if __name__=="__main__":
    import sys
    if not(sys.flags.isolated and sys.dont_write_bytecode and sys.argv==[sys.argv[0],"--self-test"]):
        raise SystemExit("cloud pure self-test only; no signing or provisioning CLI")
    print("Modern signed package provisioning:",self_test(),"negative fixtures; no target execution")
