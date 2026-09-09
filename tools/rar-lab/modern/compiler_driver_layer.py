"""Pure compiler-driver layer construction and inspection; never activates images.
The trusted cloud parent must bind driver bytes to two reproducible builds of the
reviewed driver source. Hash labels alone are not build provenance or authority.
"""
import hashlib
import importlib.util
import io
from pathlib import Path
import re
import struct
import sys
import tarfile

EPOCH = 1785715200
BASE_IMAGE = "sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914"
DRIVER = "rar-compile-driver"
MAX_DRIVER = 2 * 1024 * 1024
MAX_LAYER = MAX_DRIVER + 32768

class Invalid(ValueError):
    pass

def identity(value, width):
    return (type(value) is str and re.fullmatch("[0-9a-f]{" + str(width) + "}", value)
            is not None and value != "0" * width)

def _tool_sources():
    if sys.flags.isolated != 1 or not sys.dont_write_bytecode:
        raise Invalid("isolated no-bytecode controller required")
    result = {}
    for name in ("compiler_driver_layer.py", "compiler_elf.py"):
        path = Path(__file__).with_name(name)
        if path.is_symlink() or not path.is_file():
            raise Invalid("trusted tool location")
        with path.open("rb") as stream:
            raw = stream.read(131073)
        if not 1 <= len(raw) <= 131072:
            raise Invalid("trusted tool source budget")
        result[name] = {"sha256": hashlib.sha256(raw).hexdigest(),
                        "git_blob": hashlib.sha1(b"blob " + str(len(raw)).encode("ascii") +
                                                b"\0" + raw).hexdigest(),
                        "size": len(raw)}
    return result

def static_driver(raw):
    _tool_sources()
    if type(raw) is not bytes or not 176 <= len(raw) <= MAX_DRIVER:
        raise Invalid("driver size/type")
    # Reuse the independent byte parser, not a compiler/exporter's metadata.
    path = Path(__file__).with_name("compiler_elf.py")
    if path.is_symlink() or not path.is_file():
        raise Invalid("trusted parser location")
    spec = importlib.util.spec_from_file_location("rar_driver_elf", path)
    parser = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parser)
    try:
        metadata = parser.inspect(raw)
    except parser.Invalid as exc:
        raise Invalid("driver ELF: " + str(exc)) from exc
    if metadata != {"kind": 2, "interpreter": None, "needed": [],
                    "soname": None, "search": None}:
        raise Invalid("driver must be static ET_EXEC")
    entry, phoff = struct.unpack_from("<QQ", raw, 24)
    phsize, phnum = struct.unpack_from("<HH", raw, 54)
    entry_mappings = 0
    mapped_bytes = 0
    for n in range(phnum):
        typ, flags, offset, address, _, filesz, memsz, align = struct.unpack_from(
            "<IIQQQQQQ", raw, phoff + n * phsize)
        if typ in (2, 3):
            raise Invalid("driver dynamic/interpreter segment")
        if typ == 1:
            mapped_bytes += memsz
            if mapped_bytes > 64 * 1024 * 1024:
                raise Invalid("driver aggregate mapping budget")
            if (flags & ~7 or memsz > 64 * 1024 * 1024 or
                align not in (0, 1) and (align & (align - 1) or align > 2 * 1024 * 1024)):
                raise Invalid("driver load budget/alignment")
            if align > 1 and offset % align != address % align:
                raise Invalid("driver load congruence")
            if flags & 1 and address <= entry < address + filesz:
                entry_mappings += 1
    if entry_mappings != 1:
        raise Invalid("driver entrypoint mapping")
    return hashlib.sha256(raw).hexdigest()

def provenance(source_revision, source_sha256, recipe_sha256):
    if (not identity(source_revision, 40) or not identity(source_sha256, 64) or
        not identity(recipe_sha256, 64)):
        raise Invalid("driver provenance labels")
    return {"source_revision": source_revision, "source_sha256": source_sha256,
            "recipe_sha256": recipe_sha256}

def inspect(raw, expected_sha256):
    if (type(raw) is not bytes or not 10240 <= len(raw) <= MAX_LAYER or
        len(raw) % 512 or not identity(expected_sha256, 64)):
        raise Invalid("driver layer bounds/identity")
    seen = set()
    value = None
    try:
        with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as archive:
            for member in archive:
                if (member.name != DRIVER or member.name in seen or
                    member.uid != 0 or member.gid != 0 or member.mode != 0o555 or
                    member.mtime != EPOCH or member.uname or member.gname or
                    member.pax_headers or member.sparse is not None or member.linkname):
                    raise Invalid("driver layer member metadata")
                seen.add(member.name)
                if not member.isfile() or not 176 <= member.size <= MAX_DRIVER:
                    raise Invalid("driver layer executable")
                stream = archive.extractfile(member)
                if stream is None:
                    raise Invalid("driver layer bytes absent")
                value = stream.read(MAX_DRIVER + 1)
                if len(value) != member.size:
                    raise Invalid("driver layer bytes length")
    except (tarfile.TarError, OSError, OverflowError) as exc:
        raise Invalid("driver layer framing") from exc
    if seen != {DRIVER} or value is None:
        raise Invalid("driver layer incomplete")
    actual = static_driver(value)
    if actual != expected_sha256:
        raise Invalid("driver byte identity")
    # Require the canonical USTAR representation, including zero tail. This
    # rejects hidden concatenated entries, extra trailing bytes and alternate
    # encodings that a Docker extractor might interpret differently.
    if raw != _encode(value):
        raise Invalid("noncanonical driver layer")
    return {"base_image": BASE_IMAGE, "driver": DRIVER, "driver_sha256": actual,
            "driver_size": len(value), "diff_id": "sha256:" + hashlib.sha256(raw).hexdigest(),
            "state": "driver-layer-inspected-not-activated"}

def _encode(driver):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w", format=tarfile.USTAR_FORMAT) as archive:
        member = tarfile.TarInfo(DRIVER)
        member.mode = 0o555; member.mtime = EPOCH; member.size = len(driver)
        archive.addfile(member, io.BytesIO(driver))
    return output.getvalue()

def build(driver, source_revision, source_sha256, recipe_sha256):
    labels = provenance(source_revision, source_sha256, recipe_sha256)
    sha = static_driver(driver)
    raw = _encode(driver)
    report = inspect(raw, sha)
    report["provenance_labels"] = labels
    report["tool_sources"] = _tool_sources()
    return raw, report

def self_test():
    import unittest
    def executable():
        raw = bytearray(256)
        raw[:7] = b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HHI", raw, 16, 2, 62, 1)
        struct.pack_into("<QQ", raw, 24, 0x4000b0, 64)
        struct.pack_into("<HHH", raw, 52, 64, 56, 2)
        struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0x400000, 0, 256, 256, 4096)
        struct.pack_into("<IIQQQQQQ", raw, 120, 0x6474e551, 6, 0, 0, 0, 0, 0, 16)
        return bytes(raw)
    class Tests(unittest.TestCase):
        def test_reproducible_exact_single_entry_layer(self):
            driver = executable()
            raw, report = build(driver, "a" * 40, "b" * 64, "c" * 64)
            self.assertEqual((raw, report), build(driver, "a" * 40, "b" * 64, "c" * 64))
            self.assertEqual(report["base_image"], BASE_IMAGE)
            self.assertEqual(inspect(raw, hashlib.sha256(driver).hexdigest())["driver_size"], 256)
            with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as archive:
                self.assertEqual(archive.getnames(), [DRIVER])
        def test_dynamic_wrong_arch_entrypoint_and_stack_refused(self):
            for offset, fmt, value in ((16,"<H",3),(18,"<H",183),(24,"<Q",0),
                    (24,"<Q",0x400100),(64,"<I",2),(64,"<I",3),(68,"<I",7),
                    (68,"<I",13),(104,"<Q",65*1024*1024),(112,"<Q",3),
                    (124,"<I",7),(120,"<I",0)):
                bad = bytearray(executable()); struct.pack_into(fmt,bad,offset,value)
                with self.assertRaises(Invalid): static_driver(bytes(bad))
            for size in range(256):
                with self.assertRaises(Invalid): static_driver(executable()[:size])
        def test_load_congruence_aggregate_and_ambiguous_entry(self):
            bad=bytearray(executable())
            struct.pack_into("<Q",bad,80,0x400001)
            with self.assertRaises(Invalid): static_driver(bytes(bad))
            def second_load(flags, address, memsz):
                raw=bytearray(executable()+bytes(64))
                struct.pack_into("<H",raw,56,3)
                struct.pack_into("<IIQQQQQQ",raw,176,1,flags,0,address,0,256,memsz,4096)
                return raw
            raw=second_load(4,0x800000,64*1024*1024)
            with self.assertRaises(Invalid): static_driver(bytes(raw))
            raw=second_load(5,0x400000,256)
            with self.assertRaises(Invalid): static_driver(bytes(raw))
            # A distinct readonly load within the aggregate budget is valid.
            raw=second_load(4,0x800000,256)
            static_driver(bytes(raw))
        def test_tool_binding_and_no_bytecode_guard(self):
            from unittest.mock import patch
            sources=_tool_sources()
            self.assertEqual(set(sources),{"compiler_driver_layer.py","compiler_elf.py"})
            self.assertTrue(all(identity(x["sha256"],64) and identity(x["git_blob"],40)
                                for x in sources.values()))
            with patch.object(sys,"dont_write_bytecode",False):
                with self.assertRaises(Invalid): static_driver(executable())
        def test_exact_maximum_driver_size_is_admitted(self):
            raw=executable()+bytes(MAX_DRIVER-256)
            self.assertEqual(static_driver(raw),hashlib.sha256(raw).hexdigest())
        def test_identity_and_budgets(self):
            driver = executable(); raw, _ = build(driver,"a"*40,"b"*64,"c"*64)
            for value in (None,"", "0"*64,"g"*64,"a"*63):
                with self.assertRaises(Invalid): inspect(raw,value)
            with self.assertRaises(Invalid): inspect(raw,"d"*64)
            for labels in (("0"*40,"b"*64,"c"*64),("a"*40,"0"*64,"c"*64),
                           ("a"*40,"b"*64,"c"*63)):
                with self.assertRaises(Invalid): build(driver,*labels)
            for value in (b"",bytearray(driver),b"x"*(MAX_DRIVER+1)):
                with self.assertRaises(Invalid): static_driver(value)
        def test_extra_tail_duplicate_link_mode_and_payload_refused(self):
            driver=executable(); raw,_=build(driver,"a"*40,"b"*64,"c"*64)
            sha=hashlib.sha256(driver).hexdigest()
            for bad in (raw+b"\0"*512,raw+raw,raw[:-512],raw[:-1]):
                with self.assertRaises(Invalid): inspect(bad,sha)
            for name,mode,typ in ((DRIVER,0o755,tarfile.REGTYPE),
                    (DRIVER,0o555,tarfile.SYMTYPE),("escape",0o555,tarfile.REGTYPE)):
                out=io.BytesIO()
                with tarfile.open(fileobj=out,mode="w",format=tarfile.USTAR_FORMAT) as archive:
                    f=tarfile.TarInfo(name);f.mode=mode;f.type=typ;f.mtime=EPOCH
                    f.size=len(driver) if typ==tarfile.REGTYPE else 0
                    archive.addfile(f,io.BytesIO(driver))
                with self.assertRaises(Invalid): inspect(out.getvalue(),sha)
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test only")
    self_test()
