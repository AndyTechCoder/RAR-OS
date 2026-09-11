"""Trusted public-lab package composition and boot-PE bank inspection.
Pure construction/inspection APIs; no disk, process, VM or network operations.
"""
from hashlib import sha256
import json
import struct

ORDER=("update","bad-health","bad-signature","bad-abi","factory")
NAMES=tuple("modern-settings-"+v+".layer" for v in ORDER)

def compose(built,source,controller,image,binary,packages,sign):
    identities=binary["settings_identities"](built)
    import re
    if (type(controller) is not str or re.fullmatch("[0-9a-f]{40}",controller) is None or
        controller=="0"*40 or type(image) is not str or
        re.fullmatch("sha256:[0-9a-f]{64}",image) is None):
        raise ValueError("exact trusted build provenance")
    provenance={"source":source,"controller":controller,"image":image,
                "artifacts":{name:sha256(data).hexdigest() for name,data in built.items()},
                "settings":identities}
    encoded=json.dumps(provenance,sort_keys=True,separators=(",",":")).encode()
    build_digest=sha256(encoded).digest()
    result={}
    for variant,name in zip(ORDER,NAMES):
        executable="update" if variant in ("bad-signature","bad-abi") else variant
        payload=built["modern-settings-"+executable+".efi"]
        result[name]=packages["build"](payload,source,build_digest,variant,binary["inspect"],sign)
    system=packages["factory_system"](result[NAMES[4]],source,build_digest,binary["inspect"],sign)
    return result,system,{"build_digest":build_digest.hex(),"provenance":provenance,
        "packages":{name:{"sha256":sha256(data).hexdigest(),"bytes":len(data),
                         "padded_bytes":(len(data)+4095)//4096*4096}
                    for name,data in result.items()},
        "system_sha256":sha256(system).hexdigest()}

def inspect_bank(kernel,bank,inspect):
    inspect(kernel,False)
    if type(bank) is not dict or tuple(bank)!=NAMES:
        raise ValueError("exact ordered immutable input bank")
    pe=struct.unpack_from("<I",kernel,60)[0]
    count,optional=struct.unpack_from("<H",kernel,pe+6)[0],struct.unpack_from("<H",kernel,pe+20)[0]
    table=pe+24+optional
    extents=[];result={}
    for name,package in bank.items():
        if type(package) is not bytes or not 896<=len(package)<=2097536:
            raise ValueError("bounded immutable package")
        padded=(len(package)+4095)//4096*4096
        value=package+bytes(padded-len(package))
        at=kernel.find(value)
        if at<0 or kernel.find(value,at+1)>=0:
            raise ValueError("missing or repeated immutable package")
        matches=[]
        for i in range(count):
            off=table+i*40
            size,va,raw_size,raw_at=struct.unpack_from("<IIII",kernel,off+8)
            flags=struct.unpack_from("<I",kernel,off+36)[0]
            if raw_at<=at and at+padded<=raw_at+raw_size:
                rva=va+at-raw_at
                if flags&0xa0000000 or not flags&0x40000000 or rva%4096:
                    raise ValueError("bank is not page-aligned read-only non-executable")
                if rva+padded>va+max(size,raw_size):
                    raise ValueError("bank outside mapped section")
                matches.append(rva)
        if len(matches)!=1:
            raise ValueError("bank has no unique contained section")
        rva=matches[0]
        if any(rva<b and a<rva+padded for a,b in extents):
            raise ValueError("bank virtual page alias")
        extents.append((rva,rva+padded))
        result[name]={"rva":rva,"file_offset":at,"bytes":len(package),
                      "padded_bytes":padded,"sha256":sha256(package).hexdigest()}
    return result

def unpack_signed(data,inspect):
    import base64
    if type(data) is not bytes or len(data)>6*1024*1024:
        raise ValueError("bounded signed build transfer")
    lines=data.split(b"\n")
    if len(lines)!=6 or lines[-2:]!=[b"RAR-SIGNED-BUILD:END",b""]:
        raise ValueError("signed build transfer framing")
    result={}
    for i,name in enumerate(("modern.efi","modern-service.efi")):
        if lines[i*2]!=("RAR-SIGNED-FILE:"+name).encode():
            raise ValueError("signed artifact order")
        value=base64.b64decode(lines[i*2+1],validate=True)
        if base64.b64encode(value)!=lines[i*2+1]:
            raise ValueError("canonical signed artifact")
        inspect(value,name!="modern.efi")
        result[name]=value
    return result
