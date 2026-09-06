"""Pure bounded ELF64 dependency inspection of private compiler image bytes.
Not a loader, execution authority, or complete ELF validity proof.
"""
import re
import struct

class Invalid(ValueError):
    pass

def inspect(raw):
    if (type(raw) is not bytes or not 64 <= len(raw) <= 256 * 1024 * 1024 or
        raw[:7] != b"\x7fELF\x02\x01\x01"):
        raise Invalid("ELF size/class/encoding")
    kind, machine, version = struct.unpack_from("<HHI", raw, 16)
    phoff = struct.unpack_from("<Q", raw, 32)[0]
    ehsize, phsize, phnum = struct.unpack_from("<HHH", raw, 52)
    if (kind not in (2, 3) or machine != 62 or version != 1 or
        ehsize != 64 or phsize != 56 or not 1 <= phnum <= 128 or
        phoff < 64 or phoff + phnum * phsize > len(raw)):
        raise Invalid("ELF executable/header table")
    loads = []
    dynamic = None
    interpreter = None
    stacks = 0
    for n in range(phnum):
        typ, flags, offset, address, _, filesz, memsz, _ = struct.unpack_from(
            "<IIQQQQQQ", raw, phoff + n * phsize)
        if offset + filesz > len(raw) or flags & 3 == 3:
            raise Invalid("ELF extent or writable executable segment/stack")
        if typ == 1:
            if filesz > memsz or address + memsz > 2**64:
                raise Invalid("ELF load range")
            loads.append((offset, address, filesz, flags))
        elif typ == 2:
            if dynamic is not None or filesz == 0 or filesz > memsz or filesz % 16 or filesz > 16384:
                raise Invalid("ELF dynamic table")
            dynamic = (offset, filesz, address)
        elif typ == 0x6474e551:
            stacks += 1
            if flags != 6:
                raise Invalid("stack must be explicitly read-write non-executable")
        elif typ == 3:
            if interpreter is not None or not 2 <= filesz <= 512:
                raise Invalid("ELF interpreter extent")
            value = raw[offset:offset + filesz]
            if value[-1:] != b"\0" or b"\0" in value[:-1]:
                raise Invalid("ELF interpreter framing")
            try:
                interpreter = value[:-1].decode("ascii")
            except UnicodeError as exc:
                raise Invalid("ELF interpreter encoding") from exc
            if (not interpreter.startswith("/") or
                any(x in ("", ".", "..") for x in interpreter.split("/")[1:]) or
                re.fullmatch(r"/[A-Za-z0-9_./+-]+", interpreter) is None):
                raise Invalid("ELF interpreter path")
    if not loads or stacks != 1:
        raise Invalid("ELF requires loads and exactly one explicit NX stack")
    tags = {}
    needed_offsets = []
    if dynamic is not None:
        offset, size, address = dynamic
        mapped = [(off, addr) for off, addr, length, flags in loads
                  if flags & 4 and off <= offset and offset + size <= off + length and address == addr + offset - off]
        if len(mapped) != 1:
            raise Invalid("dynamic table outside unique load mapping")
        terminated = False
        for pos in range(offset, offset + size, 16):
            tag, value = struct.unpack_from("<QQ", raw, pos)
            if tag == 0:
                terminated = True
                break
            # ELF ABI: CONFIG, DEPAUDIT, AUDIT, AUXILIARY, FILTER.
            if tag in (0x6ffffefa, 0x6ffffefb, 0x6ffffefc, 0x7ffffffd, 0x7fffffff):
                raise Invalid("embedded dynamic loading authority")
            if tag == 22 or (tag == 30 and value & 4):
                raise Invalid("text relocations")
            if tag == 1:
                needed_offsets.append(value)
                if len(needed_offsets) > 128:
                    raise Invalid("dependency count")
            elif tag in (5, 10, 14, 15, 29):
                if tag in tags:
                    raise Invalid("duplicate dynamic metadata")
                tags[tag] = value
        if not terminated:
            raise Invalid("unterminated dynamic table")
    if 15 in tags and 29 in tags:
        raise Invalid("mixed RPATH and RUNPATH")
    needed = []
    search = None
    soname = None
    if needed_offsets or any(tag in tags for tag in (14, 15, 29)):
        if 5 not in tags or 10 not in tags or not 1 <= tags[10] <= 64 * 1024 * 1024:
            raise Invalid("dynamic string table missing or oversized")
        table_address, table_size = tags[5], tags[10]
        matches = [(off + table_address - addr) for off, addr, size, flags in loads
                   if flags & 4 and addr <= table_address and table_address + table_size <= addr + size]
        if len(matches) != 1:
            raise Invalid("ambiguous or unmapped dynamic string table")
        start = matches[0]
        if start + table_size > len(raw):
            raise Invalid("dynamic string table extent")
        def string(index):
            if index >= table_size:
                raise Invalid("dynamic string index")
            end = raw.find(b"\0", start + index, start + table_size)
            if end < 0 or end - (start + index) > 4096:
                raise Invalid("dynamic string termination/size")
            try:
                return raw[start + index:end].decode("ascii")
            except UnicodeError as exc:
                raise Invalid("dynamic string encoding") from exc
        for index in needed_offsets:
            name = string(index)
            if (re.fullmatch(r"[A-Za-z0-9_.+-]+", name) is None or
                name in needed or any(x in name.lower() for x in ("crypto", "ssl", "sodium"))):
                raise Invalid("unapproved dependency name")
            needed.append(name)
        if 14 in tags:
            soname = string(tags[14])
            if (re.fullmatch(r"[A-Za-z0-9_.+-]+", soname) is None or
                any(x in soname.lower() for x in ("crypto", "ssl", "sodium"))):
                raise Invalid("SONAME")
        for tag in (15, 29):
            if tag in tags:
                entries = string(tags[tag]).split(":")
                if (not 1 <= len(entries) <= 8 or len(set(entries)) != len(entries) or
                    any(entry not in ("$ORIGIN", "$ORIGIN/../lib", "$ORIGIN/../../..") for entry in entries)):
                    raise Invalid("unapproved dynamic search path")
                search = {"kind": "RPATH" if tag == 15 else "RUNPATH", "entries": entries}
    return {"kind": kind, "interpreter": interpreter, "needed": needed,
            "soname": soname, "search": search}

def self_test():
    import unittest
    def fixture(entries=None, strings=b"\0libc.so.6\0$ORIGIN\0"):
        raw = bytearray(512)
        raw[:7] = b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HHI", raw, 16, 3, 62, 1)
        struct.pack_into("<Q", raw, 32, 64)
        struct.pack_into("<HHH", raw, 52, 64, 56, 3)
        struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0x400000, 0, 512, 512, 4096)
        if entries is None:
            entries = [(1, 1), (5, 0x400180), (10, len(strings)), (29, 11), (0, 0)]
        struct.pack_into("<IIQQQQQQ", raw, 120, 2, 4, 256, 0x400100, 0, len(entries) * 16, len(entries) * 16, 8)
        struct.pack_into("<IIQQQQQQ", raw, 176, 0x6474e551, 6, 0, 0, 0, 0, 0, 16)
        for index, (tag, value) in enumerate(entries):
            struct.pack_into("<QQ", raw, 256 + index * 16, tag, value)
        raw[384:384 + len(strings)] = strings
        return bytes(raw)
    class Tests(unittest.TestCase):
        def test_dependency_and_search_are_parsed_from_bytes(self):
            result = inspect(fixture())
            self.assertEqual(result["needed"], ["libc.so.6"])
            self.assertEqual(result["search"], {"kind": "RUNPATH", "entries": ["$ORIGIN"]})
            self.assertIsNone(result["interpreter"])
        def test_loading_authority_and_text_relocations_rejected(self):
            for tag in (0x6ffffefa, 0x6ffffefb, 0x6ffffefc, 0x7ffffffd, 0x7fffffff, 22):
                with self.assertRaises(Invalid): inspect(fixture([(tag, 1), (0, 0)]))
            with self.assertRaises(Invalid): inspect(fixture([(30, 4), (0, 0)]))
        def test_no_malformed_dynamic_metadata(self):
            for entries in (
                [(1, 1), (0, 0)],
                [(1, 1), (5, 0x900000), (10, 19), (0, 0)],
                [(1, 100), (5, 0x400180), (10, 19), (0, 0)],
                [(1, 1), (1, 1), (5, 0x400180), (10, 19), (0, 0)],
                [(5, 0x400180), (5, 0x400180), (0, 0)],
                [(15, 11), (29, 11), (5, 0x400180), (10, 19), (0, 0)],
                [(1, 1), (5, 0x400180), (10, 19)]):
                with self.assertRaises(Invalid): inspect(fixture(entries))
        def test_no_writable_or_relative_search(self):
            for search in (b"", b"/build", b"/source", b".", b"$ORIGIN:", b"$ORIGIN:$ORIGIN", b"$LIB"):
                with self.assertRaises(Invalid): inspect(fixture(strings=b"\0libc.so.6\0" + search + b"\0"))
            with self.assertRaises(Invalid):
                inspect(fixture(strings=b"\0libssl.so\0$ORIGIN\0"))
        def test_forbidden_soname(self):
            entries = [(14, 1), (5, 0x400180), (10, 19), (0, 0)]
            with self.assertRaises(Invalid):
                inspect(fixture(entries, strings=b"\0libssl.so\0$ORIGIN\0"))
        def test_headers_bounds_and_wx(self):
            raw = fixture()
            for offset, fmt, value in ((16, "<H", 1), (18, "<H", 183), (32, "<Q", 2**63),
                                       (54, "<H", 55), (56, "<H", 129), (68, "<I", 7),
                                       (124, "<I", 7), (128, "<Q", 2**63),
                                       (56, "<H", 2), (68, "<I", 1), (68, "<I", 0),
                                       (160, "<Q", 0)):
                bad = bytearray(raw); struct.pack_into(fmt, bad, offset, value)
                with self.assertRaises(Invalid): inspect(bytes(bad))
            stack = bytearray(raw)
            struct.pack_into("<I", stack, 180, 5)
            with self.assertRaises(Invalid): inspect(bytes(stack))
            struct.pack_into("<I", stack, 180, 6)
            self.assertEqual(inspect(bytes(stack))["needed"], ["libc.so.6"])
            struct.pack_into("<II", stack, 120, 0x6474e551, 6)
            with self.assertRaises(Invalid): inspect(bytes(stack))
            outside = bytearray(raw)
            struct.pack_into("<Q", outside, 136, 0x800000)
            with self.assertRaises(Invalid): inspect(bytes(outside))
            for n in range(len(raw)):
                with self.assertRaises(Invalid): inspect(raw[:n])
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test only")
    self_test()
