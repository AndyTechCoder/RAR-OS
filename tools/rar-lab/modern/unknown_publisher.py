"""Fixed non-enrolled public laboratory publisher, host-only and variable-time.
The all-zero seed is deliberately public test data. No caller key, secret,
enrollment, production signing, file write or target execution API exists.
"""
from hashlib import sha256,sha512
from pathlib import Path
import importlib.util
def arithmetic():
    path=Path(__file__).resolve().with_name("lab_signer.py")
    spec=importlib.util.spec_from_file_location("public_arithmetic",path)
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def sign(digest):
    if type(digest) is not bytes or len(digest)!=32 or digest==bytes(32):
        raise ValueError("one exact public manifest digest")
    a=arithmetic();expanded=sha512(bytes(32)).digest()
    bits=bytearray(expanded[:32]);bits[0]&=248;bits[31]=(bits[31]&63)|64
    scalar=int.from_bytes(bits,"little");public=a._encode(a._multiply(scalar))
    if public==a.PUBLIC_KEY:raise ValueError("negative key must not be enrolled")
    message=b"RAR-LAYER-ALPHA-V0\0"+digest
    nonce=int.from_bytes(sha512(expanded[32:]+message).digest(),"little")%a._ORDER
    encoded=a._encode(a._multiply(nonce))
    challenge=int.from_bytes(sha512(encoded+public+message).digest(),"little")%a._ORDER
    response=(nonce+challenge*scalar)%a._ORDER
    return public,encoded+response.to_bytes(32,"little")
def package(template):
    if type(template) is not bytes or not 896<=len(template)<=2097536:
        raise ValueError("bounded already-built public template")
    if (template[:8]!=b"RARMODL0" or template[288:320]!=sha256(template[:288]).digest() or
        template[112:144]!=sha256(template[384:]).digest() or
        int.from_bytes(template[56:60],"little")!=len(template)-384):
        raise ValueError("canonical template digest/length")
    public,_=sign(template[288:320])
    pre=bytearray(template[:288]);pre[144:176]=sha256(public).digest();pre=bytes(pre)
    digest=sha256(pre).digest();key,signature=sign(digest)
    if key!=public:raise ValueError("fixed negative publisher")
    return public,pre+digest+signature+template[384:]
def fixture():
    # Synthetic PE data only. The separate RAR verifier checks the generated
    # signature with this non-enrolled key before proving Publisher rejection.
    a=arithmetic();template=a.codec_test_fixture()[2*1408:3*1408]
    public,value=package(template)
    return public+value
def self_test():
    value=fixture();public,body=value[:32],value[32:]
    assert len(value)==1440 and len(public)==32 and public!=arithmetic().PUBLIC_KEY
    assert body[144:176]==sha256(public).digest() and body[288:320]==sha256(body[:288]).digest()
    assert value==fixture()
    rejected=0
    for bad in (None,b"",bytes(32),bytes(31),bytearray(range(32))):
        try:sign(bad)
        except ValueError:rejected+=1
        else:raise AssertionError("bad digest accepted")
    for bad in (b"",body[:-1],bytearray(body),b"x"*896):
        try:package(bad)
        except ValueError:rejected+=1
        else:raise AssertionError("bad template accepted")
    return rejected
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated fixed public negative-fixture self-test only")
    print("Unknown publisher:",self_test(),"negative fixtures; not VM evidence")
