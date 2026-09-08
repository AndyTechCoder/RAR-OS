"""Read-only frozen DataVault oracle, independent of RAR target codec/crypto.
Public laboratory fixtures ONLY. Python integer arithmetic is not constant-time.
No paths, subprocesses, target imports, image writes, formatting, or runtime role.
"""
import hashlib
import hmac
import re
import struct

def _sha(data):
    return hashlib.sha256(data).digest()

def _block(key, nonce, counter):
    if len(key)!=32 or len(nonce)!=12 or type(counter) is not int or not 0<=counter<2**32:
        raise ValueError("invalid ChaCha block input")
    state=list(struct.unpack("<16I",b"expand 32-byte k"+key+struct.pack("<I",counter)+nonce))
    work=state.copy()
    def rot(value,n):
        return ((value<<n)|(value>>(32-n)))&0xffffffff
    def quarter(a,b,c,d):
        work[a]=(work[a]+work[b])&0xffffffff;work[d]=rot(work[d]^work[a],16)
        work[c]=(work[c]+work[d])&0xffffffff;work[b]=rot(work[b]^work[c],12)
        work[a]=(work[a]+work[b])&0xffffffff;work[d]=rot(work[d]^work[a],8)
        work[c]=(work[c]+work[d])&0xffffffff;work[b]=rot(work[b]^work[c],7)
    for _ in range(10):
        for indices in ((0,4,8,12),(1,5,9,13),(2,6,10,14),(3,7,11,15),
                        (0,5,10,15),(1,6,11,12),(2,7,8,13),(3,4,9,14)):
            quarter(*indices)
    return struct.pack("<16I",*((a+b)&0xffffffff for a,b in zip(state,work)))

def _tag(key,nonce,aad,ciphertext):
    one_time=_block(key,nonce,0)[:32]
    r=int.from_bytes(one_time[:16],"little")&0x0ffffffc0ffffffc0ffffffc0fffffff
    s=int.from_bytes(one_time[16:],"little")
    message=aad+bytes((-len(aad))%16)+ciphertext+bytes((-len(ciphertext))%16)
    message+=struct.pack("<QQ",len(aad),len(ciphertext))
    accumulator=0
    for at in range(0,len(message),16):
        accumulator=((accumulator+int.from_bytes(message[at:at+16]+b"\x01","little"))*r)%((1<<130)-5)
    return ((accumulator+s)%2**128).to_bytes(16,"little")

def _xor(key,nonce,data):
    stream=b"".join(_block(key,nonce,1+at//64) for at in range(0,len(data),64))
    return bytes(a^b for a,b in zip(data,stream))

def open_public_lab(key,nonce,aad,ciphertext,tag):
    # The tiny bounds cover only this format and its RFC8439 test vector.
    if any(type(v) is not bytes for v in (key,nonce,aad,ciphertext,tag)) or len(key)!=32 or len(nonce)!=12 or len(tag)!=16 or len(aad)>112 or len(ciphertext)>256:
        raise ValueError("oracle AEAD bounds")
    if not hmac.compare_digest(_tag(key,nonce,aad,ciphertext),tag):
        raise ValueError("oracle authentication failed")
    return _xor(key,nonce,ciphertext)

def snapshot(raw):
    if type(raw) is not bytes or len(raw)!=256 or raw[:8]!=b"RARDAT00" or raw[8]>4 or any(raw[9:16]):
        raise ValueError("noncanonical snapshot header")
    files={};at=16;previous=b"";total=0
    for _ in range(raw[8]):
        if at+2>256: raise ValueError("truncated record")
        names,values=raw[at],raw[at+1];at+=2
        if not 1<=names<=12 or values>64 or at+names+values>256:
            raise ValueError("record bounds")
        name=raw[at:at+names];at+=names
        if re.fullmatch(rb"[A-Za-z0-9._-]{1,12}",name) is None or name<=previous:
            raise ValueError("invalid or unordered name")
        previous=name
        files[name]=raw[at:at+values];at+=values;total+=values
    if total>128 or any(raw[at:]):
        raise ValueError("aggregate size or trailing padding")
    return files

def inspect(image):
    """Authenticate every committed record before returning the final snapshot.
    Caller must independently bind these exact bytes to the frozen cloud disk.
    No guest claim, screenshot, expected file value, or reconstruction is input.
    """
    if type(image) is not bytes or not 5*512<=len(image)<=194*512 or len(image)%512:
        raise ValueError("bounded Data image required")
    h=image[:512];slots=int.from_bytes(h[20:24],"little")
    if h!=image[512:1024] or h[:8]!=b"RARVLT00" or h[8:12]!=bytes((0,0,0,2)) or h[12:16]!=(1).to_bytes(4,"little") or h[16:20]!=(512).to_bytes(4,"little") or h[24:32]!=(2).to_bytes(8,"little"):
        raise ValueError("invalid header copies/framing")
    if not 1<=slots<=64 or len(image)!=(2+3*slots)*512 or not any(h[32:64]) or not any(h[64:96]) or any(h[100:480]) or _sha(h[:480])!=h[480:]:
        raise ValueError("invalid header bounds/identity/checksum")
    header_hash=_sha(h)
    revision=0;parent=bytes(32);files={};free=False;next_slot=slots
    burned=[];committed=[]
    for slot in range(slots):
        at=(2+3*slot)*512
        reserve,payload,commit=(image[at+n*512:at+(n+1)*512] for n in range(3))
        if not any(reserve+payload+commit):
            if not free: next_slot=slot;free=True
            continue
        if free: raise ValueError("nonvirgin slot after free suffix")
        if reserve!=bytes((0xa5,))*512:
            if any(reserve) and all(v in (0,0xa5) for v in reserve) and not any(payload+commit):
                burned.append(slot);continue
            raise ValueError("conflicting reservation")
        if not any(commit):
            burned.append(slot);continue
        if not all(v in (0,0xc3) for v in commit):
            raise ValueError("corrupt commit marker")
        if payload[:8]!=b"RARVPY00" or int.from_bytes(payload[8:16],"little")!=slot or int.from_bytes(payload[16:24],"little")!=revision+1 or payload[24:56]!=parent or any(payload[56:64]+payload[336:]):
            raise ValueError("invalid authenticated record chain")
        nonce=h[96:100]+slot.to_bytes(8,"little")
        aad=b"RAR-VAULT-AAD-V0"+header_hash+payload[:64]
        plain=open_public_lab(h[64:96],nonce,aad,payload[64:320],payload[320:336])
        files=snapshot(plain);revision+=1;parent=_sha(payload);committed.append(slot)
    return dict(image_sha256=_sha(image).hex(),header_sha256=header_hash.hex(),
                revision=revision,files=files,next_slot=next_slot,
                readonly=next_slot==slots,burned_slots=burned,committed_slots=committed)

def self_test():
    # RFC8439 section2.8.2; standalone oracle, not the target implementation.
    key=bytes(range(0x80,0xa0));nonce=bytes.fromhex("070000004041424344454647")
    aad=bytes.fromhex("50515253c0c1c2c3c4c5c6c7")
    cipher=bytes.fromhex("d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d63dbe"
                        "a45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b3692ddbd7f"
                        "2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc3ff4def08e4b7"
                        "a9de576d26586cec64b6116")
    tag=bytes.fromhex("1ae10b594f09e26a7e902ecbd0600691")
    plain=b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
    assert open_public_lab(key,nonce,aad,cipher,tag)==plain
    assert _block(bytes(range(32)),bytes.fromhex("000000090000004a00000000"),1).hex()==(
        "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4e"
        "d2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e")
    rejected=0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected+=1
        else: raise AssertionError("invalid oracle fixture accepted")
    for index in range(16):
        bad=bytearray(tag);bad[index]^=1
        reject(lambda bad=bytes(bad):open_public_lab(key,nonce,aad,cipher,bad))
    for values in ((key[:-1],nonce,aad,cipher,tag),(key,nonce[:-1],aad,cipher,tag),
                   (key,nonce,aad,cipher,tag[:-1]),(key,nonce,bytes(113),cipher,tag),
                   (key,nonce,aad,bytes(257),tag),(bytearray(key),nonce,aad,cipher,tag)):
        reject(lambda values=values:open_public_lab(*values))
    for index,original in enumerate((key,nonce,aad,cipher)):
        bad=bytearray(original);bad[0]^=1
        values=[key,nonce,aad,cipher,tag];values[index]=bytes(bad)
        reject(lambda values=values:open_public_lab(*values))
    # In-memory synthetic fixtures only. Never export these repeated test keys
    # into independently writable images. No production/provisioning seal API.
    header=bytearray(512);header[:8]=b"RARVLT00";header[8:12]=bytes((0,0,0,2))
    for at,value in ((12,1),(16,512),(20,4)):
        header[at:at+4]=value.to_bytes(4,"little")
    header[24:32]=(2).to_bytes(8,"little");header[32:64]=bytes((7,))*32
    header[64:96]=bytes((9,))*32;header[96:100]=bytes((1,2,3,4))
    header[480:]=_sha(header[:480]);header=bytes(header)
    empty=header*2+bytes(12*512)
    assert inspect(empty)["files"]=={} and inspect(empty)["next_slot"]==0
    def encoded(entries):
        raw=b"RARDAT00"+bytes((len(entries),))+bytes(7)
        for name,value in entries: raw+=bytes((len(name),len(value)))+name+value
        return raw+bytes(256-len(raw))
    def payload(slot,revision,parent,raw):
        prefix=b"RARVPY00"+slot.to_bytes(8,"little")+revision.to_bytes(8,"little")+parent+bytes(8)
        n=header[96:100]+slot.to_bytes(8,"little");a=b"RAR-VAULT-AAD-V0"+_sha(header)+prefix
        c=_xor(header[64:96],n,raw)
        return prefix+c+_tag(header[64:96],n,a,c)+bytes(176)
    first=payload(0,1,bytes(32),encoded([(b"note",b"challenge-one")]))
    disk=header*2+bytes((0xa5,))*512+first+bytes((0xc3,))*512+bytes(9*512)
    before=bytes(disk);result=inspect(disk)
    assert disk==before and result["files"]=={b"note":b"challenge-one"} and result["revision"]==1
    # Every partial reservation and commit prefix, including zero/full, follows
    # the independent format classification without resealing a burned slot.
    for length in range(513):
        d=bytearray(empty);d[1024:1024+length]=bytes((0xa5,))*length
        got=inspect(bytes(d))
        assert got["revision"]==0 and got["next_slot"]==(0 if length==0 else 1)
        d=bytearray(disk);d[4*512:5*512]=bytes((0xc3,))*length+bytes(512-length)
        got=inspect(bytes(d))
        assert got["revision"]==(0 if length==0 else 1)
    # A burned slot can precede a committed successor with consecutive logical
    # revision and unchanged parent; physical slot is nevertheless nonce-bound.
    burned=bytes((0xa5,))*512+bytes(1024)
    second=payload(2,2,_sha(first),encoded([(b"note",b"challenge-two")]))
    chain=header*2+bytes((0xa5,))*512+first+bytes((0xc3,))*512+burned+bytes((0xa5,))*512+second+bytes((0xc3,))*512+bytes(1536)
    got=inspect(chain)
    assert got["revision"]==2 and got["burned_slots"]==[1] and got["committed_slots"]==[0,2] and got["next_slot"]==3 and got["files"][b"note"]==b"challenge-two"
    for offset in (0,8,12,16,20,24,32,64,96,100,480,512,1024,1536,1544,1552,1560,1592,1600,1856,1872,2048):
        bad=bytearray(disk);bad[offset]^=0x40
        reject(lambda bad=bytes(bad):inspect(bad))
    # Mutate both identical headers and recompute checksum, so semantic guards
    # are reached independently of the copy/checksum mismatch guards above.
    header_mutations=((8,b"\x01"),(10,b"\x01"),(12,(2).to_bytes(4,"little")),
                      (16,(1024).to_bytes(4,"little")),(20,bytes(4)),
                      (20,(65).to_bytes(4,"little")),(20,(3).to_bytes(4,"little")),
                      (24,(3).to_bytes(8,"little")),(32,bytes(32)),(64,bytes(32)),(100,b"\x01"))
    for at,value in header_mutations:
        h=bytearray(header);h[at:at+len(value)]=value;h[480:]=_sha(h[:480])
        bad=bytes(h)*2+disk[1024:]
        reject(lambda bad=bad:inspect(bad))
    full=header*2+(bytes((0xa5,))*512+bytes(1024))*4
    assert inspect(full)["readonly"] and inspect(full)["revision"]==0
    assert len(snapshot(encoded([(b"a",bytes(64)),(b"b",bytes(64)),(b"c",b""),(b"d",b"")])))==4
    for bad in (b"",disk[:-1],disk+bytes(512),bytes(195*512),bytearray(disk)):
        reject(lambda bad=bad:inspect(bad))
    hole=bytearray(empty);hole[5*512]=0xa5
    reject(lambda:inspect(bytes(hole)))
    conflict=bytearray(empty);conflict[2*512]=0xa5;conflict[3*512]=1
    reject(lambda:inspect(bytes(conflict)))
    fork=header*2+bytes((0xa5,))*512+first+bytes((0xc3,))*512+bytes((0xa5,))*512+first+bytes((0xc3,))*512+bytes(6*512)
    reject(lambda:inspect(fork))
    for entries in ([(b"b",b"2"),(b"a",b"1")],[(b"a",b"1"),(b"a",b"2")],
                    [(b"../x",b"bad")],[(b"a",bytes(65))],
                    [(b"a",bytes(64)),(b"b",bytes(64)),(b"c",b"x")]):
        reject(lambda entries=entries:snapshot(encoded(entries)))
    bad=bytearray(encoded([]));bad[-1]=1
    reject(lambda:snapshot(bytes(bad)))
    assert rejected==73
    return dict(negative_tests=rejected,marker_prefix_cases=1026,rfc_vectors=2)

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"]:
        raise SystemExit("only --self-test is supported; runtime requires trusted controller byte binding")
    print("Frozen Data oracle:",self_test(),"; no disk I/O or guest evidence")
