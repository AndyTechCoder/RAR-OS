"""Pinned musl archive inspection. Pure bytes only; never extracts or installs."""
import hashlib
import io
import lzma
import re
import tarfile

URL = "https://static.rust-lang.org/dist/rust-std-1.95.0-x86_64-unknown-linux-musl.tar.xz"
SHA256 = "aee540abf132920f791ef781489851a078d69dff493fb628d49c1d573f92bb3a"
ROOT = "rust-std-1.95.0-x86_64-unknown-linux-musl"
LIB = ROOT + "/rust-std-x86_64-unknown-linux-musl/lib/rustlib/x86_64-unknown-linux-musl/lib/"
MAX_COMPRESSED = 64 * 1024 * 1024
MAX_INFLATED = 512 * 1024 * 1024
MAX_ENTRIES = 8192
class Invalid(ValueError):
    pass

def inflate(raw):
    if type(raw) is not bytes or not 1 <= len(raw) <= MAX_COMPRESSED:
        raise Invalid("compressed archive bounds")
    decoder = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=256 * 1024 * 1024)
    result = bytearray()
    position = 0
    try:
        while not decoder.eof:
            if decoder.needs_input:
                if position == len(raw):
                    raise Invalid("truncated XZ stream")
                chunk = raw[position:position + 65536]
                position += len(chunk)
            else:
                chunk = b""
            part = decoder.decompress(chunk, max_length=min(65536, MAX_INFLATED - len(result) + 1))
            result.extend(part)
            if len(result) > MAX_INFLATED:
                raise Invalid("archive inflation bound")
        if decoder.unused_data or position != len(raw):
            raise Invalid("trailing or concatenated XZ stream")
    except lzma.LZMAError as exc:
        raise Invalid("XZ archive") from exc
    return bytes(result)

def inventory(raw):
    expanded = inflate(raw)
    found = {}
    total = 0
    try:
        with tarfile.open(fileobj=io.BytesIO(expanded), mode="r:") as archive:
            for index, item in enumerate(archive):
                if index >= MAX_ENTRIES:
                    raise Invalid("archive entry count")
                name = item.name.rstrip("/") if item.isdir() else item.name
                if (not name or len(name) > 512 or name.startswith("/") or
                    re.fullmatch(r"[A-Za-z0-9_./+-]+", name) is None or
                    any(x in ("", ".", "..") for x in name.split("/")) or
                    name.split("/")[0] != ROOT or name in found or item.sparse or
                    not (item.isfile() or item.isdir()) or item.mode & 0o7022 or
                    not 0 <= item.size <= 256 * 1024 * 1024 or
                    (item.isdir() and item.size != 0) or (item.isfile() and "/" not in name)):
                    raise Invalid("archive member")
                total += item.size
                if total > MAX_INFLATED:
                    raise Invalid("declared payload budget")
                record = {"size": item.size, "mode": item.mode,
                          "kind": "directory" if item.isdir() else "file"}
                if item.isfile():
                    stream = archive.extractfile(item)
                    if stream is None:
                        raise Invalid("missing archive payload")
                    digest = hashlib.sha256()
                    remaining = item.size
                    while remaining:
                        part = stream.read(min(65536, remaining))
                        if not part:
                            raise Invalid("truncated archive payload")
                        remaining -= len(part)
                        digest.update(part)
                    record["sha256"] = digest.hexdigest()
                found[name] = record
    except (tarfile.TarError, OSError, EOFError, OverflowError) as exc:
        raise Invalid("tar archive") from exc
    installer = found.get(ROOT + "/install.sh", {})
    if (installer.get("kind") != "file" or not installer.get("size") or
        installer.get("mode", 0) & 0o555 != 0o555):
        raise Invalid("pinned installer missing or not executable")
    if (not any(name.startswith(LIB) and name.endswith(".rlib") and
                info["kind"] == "file" and info["size"] > 0 for name, info in found.items()) or
        found.get(LIB + "self-contained/libc.a", {}).get("kind") != "file"):
        raise Invalid("musl target payload incomplete")
    return {"entries": found, "expanded_bytes": len(expanded), "payload_bytes": total}

def verify(raw):
    if type(raw) is not bytes or not 1 <= len(raw) <= MAX_COMPRESSED:
        raise Invalid("compressed archive bounds")
    if hashlib.sha256(raw).hexdigest() != SHA256:
        raise Invalid("pinned musl archive digest")
    return {"source": URL, "sha256": SHA256, **inventory(raw)}

def self_test():
    import unittest
    from unittest.mock import patch
    def fixture(change=None):
        entries = [(ROOT, b"", 0o755, tarfile.DIRTYPE),
                   (ROOT + "/install.sh", b"fixture, never executed", 0o755, tarfile.REGTYPE),
                   (LIB + "libstd.rlib", b"fixture library", 0o644, tarfile.REGTYPE),
                   (LIB + "self-contained/libc.a", b"fixture libc", 0o644, tarfile.REGTYPE)]
        if change is not None:
            entries = change(entries)
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w") as archive:
            for name, data, mode, kind in entries:
                member = tarfile.TarInfo(name)
                member.size = len(data); member.mode = mode; member.type = kind
                archive.addfile(member, io.BytesIO(data))
        return lzma.compress(output.getvalue(), format=lzma.FORMAT_XZ)
    class Tests(unittest.TestCase):
        def test_inventory_and_pin_are_separate(self):
            raw = fixture()
            result = inventory(raw)
            self.assertEqual(len(result["entries"]), 4)
            self.assertEqual(result["entries"][LIB + "libstd.rlib"]["sha256"],
                             hashlib.sha256(b"fixture library").hexdigest())
            with self.assertRaises(Invalid): verify(raw)
        def test_no_links_devices_escapes_duplicates_or_writable_members(self):
            bad = [
                lambda e: e + [e[1]], lambda e: e[:1] + e[2:], lambda e: e[:-1],
                lambda e: e + [(ROOT + "/../escape", b"x", 0o644, tarfile.REGTYPE)],
                lambda e: e + [("/absolute", b"x", 0o644, tarfile.REGTYPE)],
                lambda e: e + [(ROOT + "/link", b"", 0o644, tarfile.SYMTYPE)],
                lambda e: e + [(ROOT + "/hard", b"", 0o644, tarfile.LNKTYPE)],
                lambda e: e + [(ROOT + "/device", b"", 0o644, tarfile.CHRTYPE)],
                lambda e: e + [(ROOT + "/writable", b"x", 0o666, tarfile.REGTYPE)]]
            for change in bad:
                with self.assertRaises(Invalid): inventory(fixture(change))
            inventory(fixture(lambda e: e[1:]))
        def test_xz_truncation_trailing_concatenation_and_budgets(self):
            raw = fixture()
            for bad in (b"", b"not xz", raw[:-1], raw + b"x", raw + raw):
                with self.assertRaises(Invalid): inventory(bad)
            with patch(__name__ + ".MAX_INFLATED", 1024):
                with self.assertRaises(Invalid): inventory(raw)
            with patch(__name__ + ".MAX_COMPRESSED", 1):
                with self.assertRaises(Invalid): inventory(raw)
            with patch(__name__ + ".MAX_ENTRIES", 2):
                with self.assertRaises(Invalid): inventory(raw)
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test only")
    self_test()
