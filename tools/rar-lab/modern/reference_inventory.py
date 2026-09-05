"""Pure bounded inspection of a Docker-save reference image; never extracts.
All bytes are controller-owned cloud evidence, never target executable authority.
"""
import hashlib
import io
import json
import re
import struct
import tarfile

LIMIT = 96 * 1024 * 1024
FILES = frozenset(("reference-sodium", "reference-openssl", "licenses/openssl.txt", "licenses/libsodium.txt"))
class Invalid(ValueError):
    pass

def unique_json(raw):
    def pairs(items):
        out = {}
        for key, value in items:
            if key in out:
                raise Invalid("duplicate JSON key")
            out[key] = value
        return out
    try:
        return json.loads(raw, object_pairs_hook=pairs)
    except (UnicodeError, ValueError, RecursionError) as exc:
        raise Invalid("JSON") from exc

def members(raw: bytes, maximum: int) -> dict:
    if type(raw) is not bytes or len(raw) > maximum:
        raise Invalid("archive size")
    found = {}
    total = 0
    try:
        with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as archive:
            for index, item in enumerate(archive):
                if index >= 128:
                    raise Invalid("archive entry count")
                name = item.name
                if name.startswith("./"):
                    name = name[2:]
                if item.isdir() and name.endswith("/"):
                    name = name[:-1]
                if (not name or name.startswith("/") or
                    any(x in ("", ".", "..") for x in name.split("/")) or
                    len(name) > 256 or name in found or item.sparse or
                    not (item.isfile() or item.isdir())):
                    raise Invalid("archive entry")
                total += item.size
                if item.size < 0 or total > maximum or (item.isdir() and item.size):
                    raise Invalid("expanded size")
                content = b""
                if item.isfile():
                    stream = archive.extractfile(item)
                    if stream is None:
                        raise Invalid("missing content")
                    content = stream.read(item.size + 1)
                    if len(content) != item.size:
                        raise Invalid("content length")
                found[name] = (item, content)
    except (tarfile.TarError, OSError, OverflowError) as exc:
        raise Invalid("archive") from exc
    return found

def static_elf(raw: bytes) -> None:
    if not 64 <= len(raw) <= 32 * 1024 * 1024 or raw[:7] != b"\x7fELF\x02\x01\x01":
        raise Invalid("ELF class/encoding/version")
    kind, machine, version = struct.unpack_from("<HHI", raw, 16)
    offset = struct.unpack_from("<Q", raw, 32)[0]
    ehsize, size, count = struct.unpack_from("<HHH", raw, 52)
    if (kind != 2 or machine != 62 or version != 1 or ehsize != 64 or
        size != 56 or not 1 <= count <= 128 or offset < 64 or
        offset + size * count > len(raw)):
        raise Invalid("ELF executable/header table")
    loads = 0
    for index in range(count):
        at = offset + index * size
        typ, flags, off, _, _, filesz, memsz, _ = struct.unpack_from("<IIQQQQQQ", raw, at)
        if typ in (2, 3):
            raise Invalid("ELF dynamic/interpreter")
        if off + filesz > len(raw):
            raise Invalid("ELF segment extent")
        if typ == 1:
            loads += 1
            if filesz > memsz or flags & 3 == 3:
                raise Invalid("ELF load extent or W+X")
    if loads == 0:
        raise Invalid("ELF missing load")

def inspect(raw: bytes, image: str) -> dict:
    if type(image) is not str or re.fullmatch(r"sha256:[0-9a-f]{64}", image) is None:
        raise Invalid("image identity")
    outer = members(raw, LIMIT)
    def file(name, maximum):
        if type(name) is not str or name not in outer:
            raise Invalid("missing archive reference")
        member, value = outer[name]
        if not member.isfile() or len(value) > maximum:
            raise Invalid("archive reference size/type")
        return value
    manifest = unique_json(file("manifest.json", 65536))
    if type(manifest) is not list or len(manifest) != 1 or type(manifest[0]) is not dict:
        raise Invalid("single image required")
    entry = manifest[0]
    config_raw = file(entry.get("Config"), 65536)
    if "sha256:" + hashlib.sha256(config_raw).hexdigest() != image:
        raise Invalid("config digest")
    config = unique_json(config_raw)
    if type(config) is not dict or config.get("architecture") != "amd64" or config.get("os") != "linux":
        raise Invalid("image platform")
    process = config.get("config")
    if (type(process) is not dict or process.get("User") != "65532:65532" or
        process.get("WorkingDir") != "/" or process.get("Entrypoint") != ["/reference-sodium"] or
        any(process.get(key) not in (None, [], {}) for key in
            ("Cmd", "Env", "Volumes", "ExposedPorts", "OnBuild", "Healthcheck"))):
        raise Invalid("image process config")
    layers = entry.get("Layers")
    rootfs = config.get("rootfs")
    if (type(layers) is not list or not 1 <= len(layers) <= 8 or
        any(type(x) is not str for x in layers) or len(set(layers)) != len(layers) or
        type(rootfs) is not dict or rootfs.get("type") != "layers" or
        type(rootfs.get("diff_ids")) is not list or len(rootfs["diff_ids"]) != len(layers)):
        raise Invalid("layer identities")
    final = {}
    directories = {}
    for path, identity in zip(layers, rootfs["diff_ids"]):
        layer = file(path, 80 * 1024 * 1024)
        if identity != "sha256:" + hashlib.sha256(layer).hexdigest():
            raise Invalid("layer digest")
        for name, (member, content) in members(layer, 68 * 1024 * 1024).items():
            metadata = (member.mode, member.uid, member.gid, member.mtime)
            if member.uid != 0 or member.gid != 0 or member.mode & 0o7022:
                raise Invalid("ownership or unsafe permissions")
            if member.isdir():
                if name != "licenses" or (name in directories and directories[name] != metadata):
                    raise Invalid("unexpected directory")
                directories[name] = metadata
            else:
                if name not in FILES or name in final:
                    raise Invalid("unexpected or replaced file")
                if name.startswith("reference-"):
                    if member.mode & 0o555 != 0o555:
                        raise Invalid("adapter executable mode")
                    static_elf(content)
                elif not 1 <= len(content) <= 65536 or member.mode & 0o111:
                    raise Invalid("license size/mode")
                final[name] = {"sha256": hashlib.sha256(content).hexdigest(),
                               "size": len(content), "metadata": metadata}
    if set(final) != FILES or set(directories) != {"licenses"}:
        raise Invalid("incomplete role")
    return {"image": image, "files": final, "directories": directories,
            "diff_ids": rootfs["diff_ids"]}

def self_test():
    import unittest
    def tar(entries):
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w") as archive:
            for name, value, mode, typ in entries:
                item = tarfile.TarInfo(name)
                item.mode = mode
                item.type = typ
                item.size = len(value)
                archive.addfile(item, io.BytesIO(value))
        return output.getvalue()
    def executable():
        raw = bytearray(120)
        raw[:7] = bytes((127, 69, 76, 70, 2, 1, 1))
        struct.pack_into("<HHI", raw, 16, 2, 62, 1)
        struct.pack_into("<Q", raw, 32, 64)
        struct.pack_into("<HHH", raw, 52, 64, 56, 1)
        struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0, 0, 120, 120, 4096)
        return bytes(raw)
    def fixture(change=None):
        content = executable()
        entries = [(name, content, 0o755, tarfile.REGTYPE)
                   for name in ("reference-sodium", "reference-openssl")]
        entries += [("licenses", b"", 0o755, tarfile.DIRTYPE),
                    ("licenses/openssl.txt", b"license", 0o644, tarfile.REGTYPE),
                    ("licenses/libsodium.txt", b"license", 0o644, tarfile.REGTYPE)]
        if change is not None:
            entries = change(entries)
        layer = tar(entries)
        config = {"architecture": "amd64", "os": "linux",
                  "config": {"User": "65532:65532", "WorkingDir": "/",
                             "Entrypoint": ["/reference-sodium"]},
                  "rootfs": {"type": "layers",
                             "diff_ids": ["sha256:" + hashlib.sha256(layer).hexdigest()]}}
        config_raw = json.dumps(config, sort_keys=True).encode()
        identity = "sha256:" + hashlib.sha256(config_raw).hexdigest()
        manifest = json.dumps([{"Config": "config.json", "Layers": ["layer.tar"]}]).encode()
        outer = tar([("config.json", config_raw, 0o644, tarfile.REGTYPE),
                     ("manifest.json", manifest, 0o644, tarfile.REGTYPE),
                     ("layer.tar", layer, 0o644, tarfile.REGTYPE)])
        return outer, identity
    class Tests(unittest.TestCase):
        def test_complete_static_reference_image(self):
            raw, identity = fixture()
            result = inspect(raw, identity)
            self.assertEqual(set(result["files"]), FILES)
            with self.assertRaises(Invalid): inspect(raw, "sha256:" + "f" * 64)
        def test_unexpected_replaced_missing_or_writable_files(self):
            changes = [
                lambda e: e + [("bin/sh", b"x", 0o755, tarfile.REGTYPE)],
                lambda e: e + [e[0]],
                lambda e: e[1:],
                lambda e: [(e[0][0], e[0][1], 0o777, e[0][3])] + e[1:],
                lambda e: [("../escape", b"x", 0o644, tarfile.REGTYPE)] + e,
                lambda e: [("link", b"", 0o644, tarfile.SYMTYPE)] + e,
                lambda e: [(e[0][0], b"not ELF", e[0][2], e[0][3])] + e[1:],
            ]
            for change in changes:
                raw, identity = fixture(change)
                with self.assertRaises(Invalid): inspect(raw, identity)
        def test_dynamic_wrong_machine_and_writable_executable(self):
            good = executable()
            for offset, fmt, value in ((16, "<H", 3), (18, "<H", 183),
                                       (64, "<I", 2), (64, "<I", 3), (68, "<I", 7),
                                       (32, "<Q", 2**63), (56, "<H", 55), (58, "<H", 129)):
                bad = bytearray(good)
                struct.pack_into(fmt, bad, offset, value)
                with self.assertRaises(Invalid): static_elf(bytes(bad))
            for n in range(len(good)):
                with self.assertRaises(Invalid): static_elf(good[:n])
        def test_duplicate_json_and_archive_limits(self):
            with self.assertRaises(Invalid): unique_json(b'{"x":1,"x":2}')
            with self.assertRaises(Invalid): members(bytes(65), 64)
            with self.assertRaises(Invalid):
                members(tar([(str(n), b"", 0o644, tarfile.REGTYPE) for n in range(129)]), LIMIT)
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():
        raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test entrypoint only")
    self_test()
