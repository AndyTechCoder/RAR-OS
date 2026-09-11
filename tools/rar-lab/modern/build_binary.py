"""Pure, bounded inspection of Modern build artifacts; never executes PE code."""
import base64
import struct

SETTINGS = ("modern-settings-factory.efi", "modern-settings-update.efi",
            "modern-settings-bad-health.efi")
NAMES = ("modern.efi", "modern-service.efi") + SETTINGS
SETTINGS_CFG = {
    SETTINGS[0]: ("rar_settings_only",),
    SETTINGS[1]: ("rar_settings_only", "rar_settings_v2"),
    SETTINGS[2]: ("rar_settings_only", "rar_settings_v2", "rar_settings_fail_health"),
}
LIMIT = 2 * 1024 * 1024

def inspect(data, service):
    if not isinstance(data, bytes) or not 512 <= len(data) <= LIMIT or data[:2] != b"MZ":
        raise ValueError("invalid bounded PE file")
    def u16(at):
        return struct.unpack_from("<H", data, at)[0]
    def u32(at):
        return struct.unpack_from("<I", data, at)[0]
    def u64(at):
        return struct.unpack_from("<Q", data, at)[0]
    pe = u32(60)
    if pe > 4096 or pe + 24 > len(data) or data[pe:pe+4] != b"PE\0\0":
        raise ValueError("invalid PE header")
    count, optional, o = u16(pe+6), u16(pe+20), pe+24
    if u16(pe+4) != 0x8664 or u32(pe+8) != 0 or not 1 <= count <= 16 or not 240 <= optional <= 512:
        raise ValueError("noncanonical AMD64 PE metadata")
    if o + optional + count * 40 > len(data) or u16(o) != 0x20b:
        raise ValueError("invalid PE32+ bounds")
    base, entry, image, header = u64(o+24), u32(o+16), u32(o+56), u32(o+60)
    if u16(o+68) != 10 or u32(o+32) != 4096 or u32(o+108) != 16:
        raise ValueError("unexpected UEFI layout")
    limit=128*1024 if service else 64*1024*1024
    if not 0 < image <= limit or image % 4096:
        raise ValueError(f"image exceeds runtime budget: service={service}, mapped={image}, limit={limit}")
    if not o+optional+40*count <= header <= min(4096, len(data)):
        raise ValueError("invalid header span")
    if base % 4096 or base+image >= 1 << 47 or service and base != 0x400000:
        raise ValueError("invalid virtual base")
    # No imports, TLS, delayed imports, or debug payload. Kernel relocation is
    # allowed for UEFI; the fixed ring3 service must have no dynamic relocations.
    for index in (1, 6, 9, 13) + ((5,) if service else ()):
        if u64(o+112+index*8):
            raise ValueError("forbidden PE directory")
    virtual, raw_spans, sections, entry_ok = [], [], [], False
    for index in range(count):
        s = o+optional+index*40
        size, va, raw_size, raw_at, flags = (u32(s+x) for x in (8, 12, 16, 20, 36))
        extent = max(size, raw_size)
        end = va + ((extent+4095)//4096)*4096
        writable, executable = bool(flags & 0x80000000), bool(flags & 0x20000000)
        if not extent or va < 4096 or va % 4096 or end > image or not flags & 0x40000000 or writable and executable:
            raise ValueError("invalid W^X section layout")
        if raw_at+raw_size > len(data) or raw_size and raw_at < header:
            raise ValueError("invalid section file span")
        if any(va < b and a < end for a,b in virtual):
            raise ValueError("overlapping virtual sections")
        virtual.append((va,end))
        if raw_size:
            if any(raw_at < b and a < raw_at+raw_size for a,b in raw_spans):
                raise ValueError("overlapping raw sections")
            raw_spans.append((raw_at,raw_at+raw_size))
        entry_ok |= executable and va <= entry < va+size
        sections.append(dict(offset=va, memory_bytes=extent, file_bytes=raw_size,
                             writable=writable, executable=executable))
    if not entry_ok:
        raise ValueError("entry is not in executable memory")
    return dict(base=base, entry=base+entry, image_bytes=image, file_bytes=len(data),
                sections=sections, stack_reserve_header=u64(o+72),
                stack_commit_header=u64(o+80),
                stack_usage_proven=False)

def unpack(data):
    if not isinstance(data, bytes) or len(data) > 6*1024*1024:
        raise ValueError("build transfer outside bound")
    lines = data.split(b"\n")
    if len(lines) != 2*len(NAMES)+2 or lines[-2:] != [b"RAR-BUILD:END", b""]:
        raise ValueError("malformed build transfer")
    result = {}
    for index, name in enumerate(NAMES):
        if lines[index*2] != ("RAR-FILE:"+name).encode():
            raise ValueError("unexpected artifact name/order")
        raw = base64.b64decode(lines[index*2+1], validate=True)
        if base64.b64encode(raw) != lines[index*2+1]:
            raise ValueError("noncanonical base64")
        inspect(raw, name != "modern.efi")
        result[name] = raw
    return result

def settings_identities(built):
    """Bind inspected actual payload bytes to fixed trusted-recipe cfg names.

    Different bytes alone do not prove behavior. Runtime health/UI evidence is
    still required; cfg provenance comes from the reviewed build script.
    """
    from hashlib import sha256
    if type(built) is not dict or set(built)!=set(NAMES):
        raise ValueError("exact build inventory required")
    result={}
    for name in SETTINGS:
        inspect(built[name],True)
        result[name]={"sha256":sha256(built[name]).hexdigest(),
                      "cfg":list(SETTINGS_CFG[name])}
    if len({v["sha256"] for v in result.values()})!=len(SETTINGS):
        raise ValueError("Settings code variants must have different bytes")
    return result

def self_test():
    data = bytearray(1024)
    def p16(at, value): struct.pack_into("<H", data, at, value)
    def p32(at, value): struct.pack_into("<I", data, at, value)
    data[:2] = b"MZ"; p32(60,64); data[64:68] = b"PE\0\0"
    p16(68,0x8664); p16(70,1); p16(84,240)
    o,s = 88,328
    p16(o,0x20b); p32(o+16,4096); struct.pack_into("<Q",data,o+24,0x400000)
    p32(o+32,4096); p32(o+56,8192); p32(o+60,512); p16(o+68,10); p32(o+108,16)
    p32(s+8,16); p32(s+12,4096); p32(s+16,512); p32(s+20,512); p32(s+36,0x60000020)
    valid=bytes(data)
    assert inspect(valid,True)["entry"] == 0x401000
    assert inspect(valid,False)["stack_usage_proven"] is False
    rejected=0
    mutations = [(60,0xffffffff),(72,1),(88+16,0),(88+24,0),(88+32,512),
                 (88+56,0),(88+56,132*1024),(88+60,1025),(88+108,15),
                 (328+8,0),(328+12,0),(328+16,1024),(328+20,0xffffffff),
                 (328+36,0xe0000020)]
    mutations += [(o+112+8*i,1) for i in (1,5,6,9,13)]
    mutations += [(70,17),(s+36,0x20000020)]
    for at,value in mutations:
        bad=bytearray(valid); struct.pack_into("<I",bad,at,value)
        try: inspect(bytes(bad),True)
        except (ValueError,struct.error): rejected+=1
        else: raise AssertionError(("PE mutation accepted",at,value))
    for bad in (b"",valid[:511],b"xx"+valid[2:],valid[:700]):
        try: inspect(bad,True)
        except (ValueError,struct.error): rejected+=1
        else: raise AssertionError("PE truncation accepted")
    # Adjacent pages and raw sectors are valid, but sharing either is refused.
    two=bytearray(valid+bytes(512))
    struct.pack_into("<H",two,70,2)
    struct.pack_into("<I",two,o+56,12288)
    s2=s+40
    for offset,value in ((8,16),(12,8192),(16,512),(20,1024),(36,0x40000040)):
        struct.pack_into("<I",two,s2+offset,value)
    assert len(inspect(bytes(two),True)["sections"])==2
    assert len(inspect(bytes(two),False)["sections"])==2
    for offset,value in ((12,4096),(20,512)):
        bad=bytearray(two);struct.pack_into("<I",bad,s2+offset,value)
        try: inspect(bytes(bad),True)
        except ValueError: rejected+=1
        else: raise AssertionError("overlapping section accepted")
    packet=b"".join(("RAR-FILE:"+name+"\n").encode()+base64.b64encode(valid)+b"\n" for name in NAMES)+b"RAR-BUILD:END\n"
    assert unpack(packet)==dict.fromkeys(NAMES,valid)
    for bad in (b"",packet[:-1],packet+b"x",packet.replace(b"modern.efi",b"../escape",1),
                packet.replace(b"RAR-BUILD:END",b"OTHER"),b"x"*(6*1024*1024+1)):
        try: unpack(bad)
        except ValueError: rejected+=1
        else: raise AssertionError("transfer mutation accepted")
    variants=dict.fromkeys(NAMES,valid)
    for index,name in enumerate(SETTINGS):
        value=bytearray(valid);value[512]=index+1
        variants[name]=bytes(value)
    identities=settings_identities(variants)
    assert len({v["sha256"] for v in identities.values()})==3
    assert identities[SETTINGS[2]]["cfg"]==["rar_settings_only","rar_settings_v2","rar_settings_fail_health"]
    for bad in (dict.fromkeys(NAMES,valid),{k:v for k,v in variants.items() if k!=SETTINGS[0]},
                dict(variants,unexpected=valid),
                dict(variants,**{SETTINGS[1]:variants[SETTINGS[0]]}),
                dict(variants,**{SETTINGS[2]:b"invalid"})):
        try: settings_identities(bad)
        except (ValueError,struct.error): rejected+=1
        else: raise AssertionError("invalid Settings artifact identity accepted")
    assert rejected==38, "Modern PE negative coverage changed"
    return rejected

if __name__ == "__main__":
    import sys
    if sys.argv != [sys.argv[0],"--self-test"]:
        raise SystemExit("only --self-test is supported")
    print("Modern binary inspection:",self_test(),"negative tests; no target execution")
