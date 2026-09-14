"""Bounded public-lab native app packaging, never execution or enrollment."""
import base64
import hashlib
import struct
NAMES=("rar-notes.efi","rar-counter.efi")
IDENTITIES=(b"rar.notes.alpha0",b"rar.counter.v000")
LIMIT=128*1024
def unpack(raw,inspect):
    if type(raw) is not bytes or len(raw)>400000:raise ValueError("bounded native build")
    lines=raw.splitlines()
    if len(lines)!=5 or lines[-1]!=b"RAR-APP-BUILD:END":raise ValueError("exact native build framing")
    out={}
    for index,name in enumerate(NAMES):
        if lines[index*2]!=b"RAR-APP-FILE:"+name.encode():raise ValueError("fixed independent app order")
        payload=base64.b64decode(lines[index*2+1],validate=True)
        if not 512<=len(payload)<=LIMIT:raise ValueError("native payload budget")
        inspect(payload,True)
        out[name]=payload
    return out
def package(payload,index,signer):
    if type(payload) is not bytes or not 512<=len(payload)<=LIMIT or type(index) is not int or index not in (0,1):
        raise ValueError("fixed native package input")
    if payload[:2]!=b"MZ":raise ValueError("PE")
    pe=struct.unpack_from("<I",payload,60)[0]
    if pe>4096 or pe+88>len(payload) or payload[pe:pe+4]!=b"PE\0\0":raise ValueError("PE span")
    image=struct.unpack_from("<I",payload,pe+24+56)[0]
    if not 4096<=image<=LIMIT or image%4096:raise ValueError("mapped image budget")
    h=hashlib.sha256
    m=bytearray(512);m[:8]=b"RARAPKG0";m[16:32]=IDENTITIES[index]
    m[32:64]=h(signer["PUBLIC_KEY"]).digest();m[64:96]=h(payload).digest()
    for at,size,value in ((12,4,512),(96,8,1),(104,4,len(payload)),(108,4,image),
                          (112,4,65536),(116,4,3 if index==0 else 1),(124,4,0x8664)):
        m[at:at+size]=value.to_bytes(size,"little")
    digest=h(m[:416]).digest();m[416:448]=digest;m[448:]=signer["sign_app_manifest_digest"](digest)
    return bytes(m)+payload
