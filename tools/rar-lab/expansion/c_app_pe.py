"""RAR-owned bounded static-ELF to private-PE app packager. No execution.
Format reference: https://gabi.xinuos.com/elf/02-eheader.html and https://gabi.xinuos.com/elf/07-pheader.html.
Only convert our fixed freestanding linked C app; this is not an ELF loader.
"""
import struct
import sys

LIMIT = 128 * 1024
INPUT_LIMIT = 1024 * 1024
BASE = 0x400000

def need(ok, reason):
    if not ok:
        raise ValueError(reason)

def span(data, offset, size):
    need(isinstance(offset, int) and isinstance(size, int) and
         0 <= offset <= len(data) and 0 <= size <= len(data) - offset, "span")
    return data[offset:offset + size]

def unpack(fmt, data, offset):
    return struct.unpack(fmt, span(data, offset, struct.calcsize(fmt)))

def rounded(value, alignment):
    return (value + alignment - 1) // alignment * alignment

def convert(data):
    need(isinstance(data, bytes) and 64 <= len(data) <= INPUT_LIMIT, "ELF size")
    ident, kind, machine, version, entry, phoff, shoff, flags, ehsize, phsize, phnum, shsize, shnum, shstr = unpack("<16sHHIQQQIHHHHHH", data, 0)
    need(ident == b"\x7fELF\x02\x01\x01" + bytes(9) and (kind, machine, version, flags, ehsize) == (2, 62, 1, 0, 64), "ELF identity")
    need(64 <= phoff <= 4096 and phsize == 56 and 1 <= phnum <= 8, "program headers")
    need(shsize == 64 and 2 <= shnum <= 128 and 0 < shstr < shnum and shoff >= 64, "section headers")
    span(data, phoff, phsize * phnum)
    span(data, shoff, shsize * shnum)
    segments = []
    segment_offsets = []
    for i in range(phnum):
        tag, access, offset, virtual, physical, files, memory, align = unpack("<IIQQQQQQ", data, phoff + i * phsize)
        need(tag == 1, "non-load segment")
        # GNU ld can emit an empty explicit data PHDR when this app has no
        # globals. It is not mapped and must carry no bytes or authority.
        if memory == 0:
            need(files == 0 and access in (4, 5, 6), "empty segment")
            continue
        need(access in (4, 5, 6) and align == 4096 and virtual == physical and virtual % 4096 == 0 and
             BASE + 4096 <= virtual < BASE + LIMIT and files <= memory and memory <= LIMIT and
             offset % 4096 == 0 and offset >= phoff + phsize * phnum, "segment layout")
        raw = span(data, offset, files)
        end = virtual + rounded(max(memory, rounded(files, 512)), 4096)
        need(end <= BASE + LIMIT, "image budget")
        need(not segments or segments[-1][0] + rounded(max(segments[-1][2], rounded(len(segments[-1][3]), 512)), 4096) <= virtual, "segment overlap/order")
        segments.append((virtual, access, memory, raw))
        segment_offsets.append(offset)
    need(segments and any(access == 5 and virtual <= entry < virtual + min(memory, len(raw))
                         for virtual, access, memory, raw in segments), "executable file-backed entry")
    # Inspect the actual linked symbol table: unresolved imports or retained
    # relocations/dynamic/TLS state are not silently dropped by PE conversion.
    sections = [unpack("<IIQQQQIIQQ", data, shoff + i * shsize) for i in range(shnum)]
    need(sections[0] == (0,) * 10, "null section")
    need(sections[shstr][1] == 3, "section names")
    tables = []
    for section in sections[1:]:
        name, tag, access, address, offset, size, link, info, align, entsize = section
        need(not access & 0x400 and tag not in (4, 6, 9, 11), "relocation/dynamic/TLS section")
        if tag != 8:
            span(data, offset, size)
        if access & 2 and size:
            matches = [(segment, file_offset) for segment, file_offset in zip(segments, segment_offsets)
                       if segment[0] <= address and address + size <= segment[0] + segment[2]]
            need(len(matches) == 1 and tag in (1, 8), "allocated section outside load/type")
            (virtual, permissions, memory, raw), file_offset = matches[0]
            need(not access & 1 or permissions == 6, "writable section permissions")
            need(not access & 4 or permissions == 5, "executable section permissions")
            if tag == 1:
                need(offset == file_offset + address - virtual and
                     address - virtual + size <= len(raw), "section/segment bytes disagree")
            else:
                need(address >= virtual + len(raw), "zero-fill overlaps file bytes")
        if tag == 2:
            need(entsize == 24 and size % 24 == 0 and 1 <= size // 24 <= 4096 and 0 < link < shnum and sections[link][1] == 3, "symbol table")
            tables.append(section)
    need(len(tables) == 1, "one static symbol table required")
    table = tables[0]
    strings = sections[table[6]]
    names = span(data, strings[4], strings[5])
    found_entry = 0
    for i in range(table[5] // 24):
        name, info, other, section, value, size = unpack("<IBBHQQ", data, table[4] + i * 24)
        if i == 0:
            need((name, info, other, section, value, size) == (0,) * 6, "null symbol")
            continue
        need(section != 0, "undefined symbol")
        need(name < len(names), "symbol name")
        end = names.find(b"\0", name)
        need(end >= name and end - name <= 256, "symbol string")
        if names[name:end] == b"efi_main":
            need(info == 0x12 and other == 0 and 0 < section < shnum and value == entry and size > 0, "entry symbol")
            found_entry += 1
    need(found_entry == 1, "unique native entry")
    headers = rounded(88 + 240 + len(segments) * 40, 512)
    out = bytearray(headers)
    out[:2] = b"MZ"
    struct.pack_into("<I", out, 60, 64)
    out[64:68] = b"PE\0\0"
    struct.pack_into("<HHIIIHH", out, 68, 0x8664, len(segments), 0, 0, 0, 240, 0x23)
    optional = 88
    struct.pack_into("<H", out, optional, 0x20b)
    struct.pack_into("<I", out, optional + 16, entry - BASE)
    struct.pack_into("<Q", out, optional + 24, BASE)
    struct.pack_into("<II", out, optional + 32, 4096, 512)
    struct.pack_into("<I", out, optional + 60, headers)
    struct.pack_into("<H", out, optional + 68, 10)
    struct.pack_into("<QQQQ", out, optional + 72, 65536, 16384, 0, 0)
    struct.pack_into("<I", out, optional + 108, 16)
    image_size = 4096
    for i, (virtual, access, memory, raw) in enumerate(segments):
        at = optional + 240 + i * 40
        out[at:at + 8] = (".rar" + str(i)).encode().ljust(8, b"\0")
        payload = raw + bytes(rounded(len(raw), 512) - len(raw))
        need(len(out) + len(payload) <= LIMIT, "file budget")
        struct.pack_into("<IIII", out, at + 8, memory, virtual - BASE, len(payload), len(out) if payload else 0)
        attributes = 0x40000000 | (0x20000020 if access == 5 else 0x40) | (0x80000000 if access == 6 else 0)
        struct.pack_into("<I", out, at + 36, attributes)
        out.extend(payload)
        image_size = max(image_size, virtual - BASE + rounded(max(memory, len(payload)), 4096))
    struct.pack_into("<I", out, optional + 56, image_size)
    return bytes(out)

def fixture():
    # Synthetic machine bytes are inert test data, never executed.
    data = bytearray(0x1300)
    struct.pack_into("<16sHHIQQQIHHHHHH", data, 0,
        b"\x7fELF\x02\x01\x01" + bytes(9), 2, 62, 1, BASE + 4096, 64, 0x1100, 0, 64, 56, 1, 64, 4, 3)
    struct.pack_into("<IIQQQQQQ", data, 64, 1, 5, 0x1000, BASE + 4096, BASE + 4096, 1, 1, 4096)
    data[0x1000] = 0xc3
    struct.pack_into("<IIQQQQIIQQ", data, 0x1140, 0, 1, 6, BASE + 4096, 0x1000, 1, 0, 0, 1, 0)
    struct.pack_into("<IIQQQQIIQQ", data, 0x1180, 0, 2, 0, 0, 0x1200, 48, 3, 1, 8, 24)
    struct.pack_into("<IIQQQQIIQQ", data, 0x11c0, 0, 3, 0, 0, 0x1240, 10, 0, 0, 1, 0)
    data[0x1240:0x124a] = b"\0efi_main\0"
    struct.pack_into("<IBBHQQ", data, 0x1218, 1, 0x12, 0, 1, BASE + 4096, 1)
    return bytes(data)

def self_test():
    good = fixture()
    pe = convert(good)
    need(pe[:2] == b"MZ" and len(pe) == 1024 and pe[512] == 0xc3, "fixture conversion")
    for length in (0, 63, 120, 0x1000, 0x1240):
        try:
            convert(good[:length])
        except ValueError:
            pass
        else:
            raise AssertionError("truncation accepted")
    for offset, fmt, value in (
        (16, "<H", 3), (18, "<H", 3), (24, "<Q", BASE),
        (64, "<I", 2), (68, "<I", 7), (80, "<Q", BASE + 4097),
        (96, "<Q", 2), (112, "<Q", 1), (0x1184, "<I", 11),
        (0x121e, "<H", 0), (0x1220, "<Q", BASE + 4097),
        (0x1140 + 24, "<Q", 0x1001), (0x1140 + 8, "<Q", 7)):
        bad = bytearray(good)
        struct.pack_into(fmt, bad, offset, value)
        try:
            convert(bytes(bad))
        except ValueError:
            pass
        else:
            raise AssertionError(f"mutation accepted at {offset}")
    print("C app packaging: bounded inert ELF/PE conversion refusals passed")

if __name__ == "__main__":
    need(sys.flags.isolated and sys.dont_write_bytecode, "isolated host Python only")
    if sys.argv[1:] == ["--self-test"]:
        self_test()
    elif sys.argv[1:] == ["--convert"]:
        sys.stdout.buffer.write(convert(sys.stdin.buffer.read(INPUT_LIMIT + 1)))
    elif sys.argv[1:] == ["--fixture"]:
        sys.stdout.buffer.write(convert(fixture()))
    else:
        raise SystemExit("fixed --self-test/--convert/--fixture only")
