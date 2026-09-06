"""Pure immutable source-layer construction; no extraction or image activation.
Only the five exact adapter inputs are admitted. The trusted parent binds their
Git blobs to TargetGitSha and binds this layer into a read-only compiler image.
"""
import hashlib
import io
import re
import tarfile

EPOCH = 1785715200
FILES = frozenset((
    "tools/rar-lab/modern/target_reference.rs",
    "core/crypto/sha256.rs", "core/crypto/sha512.rs",
    "core/crypto/ed25519.rs", "core/crypto/chacha20poly1305.rs",
))
class Invalid(ValueError):
    pass

def build(source, revision):
    if (type(revision) is not str or re.fullmatch(r"[0-9a-f]{40}", revision) is None or
        type(source) is not dict or set(source) != FILES):
        raise Invalid("exact source revision/files")
    if (any(type(value) is not bytes or not 1 <= len(value) <= 256 * 1024 for value in source.values()) or
        sum(map(len, source.values())) > 512 * 1024):
        raise Invalid("source byte budget")
    directories = set()
    for path in FILES:
        parts = ("source/" + path).split("/")
        # /source already exists in the independently accepted compiler base.
        directories.update("/".join(parts[:n]) for n in range(2, len(parts)))
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w", format=tarfile.USTAR_FORMAT) as archive:
        for name in sorted(directories):
            item = tarfile.TarInfo(name)
            item.type = tarfile.DIRTYPE; item.mode = 0o555
            item.uid = item.gid = 0; item.mtime = EPOCH
            archive.addfile(item)
        for path, value in sorted(source.items()):
            item = tarfile.TarInfo("source/" + path)
            item.mode = 0o444; item.uid = item.gid = 0; item.mtime = EPOCH; item.size = len(value)
            archive.addfile(item, io.BytesIO(value))
    raw = output.getvalue()
    if len(raw) > 1024 * 1024:
        raise Invalid("source layer budget")
    report = {"schema": "rar-compiler-source-layer-v0", "target_git_sha": revision,
              "state": "source-layer-only-not-activated",
              "diff_id": "sha256:" + hashlib.sha256(raw).hexdigest(),
              "files": {path: {"size": len(value), "sha256": hashlib.sha256(value).hexdigest()}
                        for path, value in sorted(source.items())},
              "directories": sorted(directories)}
    return raw, report

def self_test():
    import unittest
    class Tests(unittest.TestCase):
        def test_exact_reproducible_readonly_layer(self):
            source = {path: ("// " + path + "\n").encode() for path in FILES}
            raw, report = build(source, "a" * 40)
            self.assertEqual((raw, report), build(dict(reversed(list(source.items()))), "a" * 40))
            with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as archive:
                found = {}
                for item in archive:
                    self.assertEqual((item.uid, item.gid, item.mtime), (0, 0, EPOCH))
                    self.assertFalse(item.pax_headers)
                    self.assertIsNone(item.sparse)
                    if item.isdir():
                        self.assertIn(item.name, report["directories"])
                        self.assertEqual(item.mode, 0o555)
                    else:
                        self.assertTrue(item.isfile())
                        self.assertEqual(item.mode, 0o444)
                        path = item.name.removeprefix("source/")
                        found[path] = archive.extractfile(item).read()
                self.assertEqual(found, source)
            self.assertEqual(report["target_git_sha"], "a" * 40)
            self.assertEqual(report["diff_id"], "sha256:" + hashlib.sha256(raw).hexdigest())
        def test_source_paths_and_revision_cannot_add_authority(self):
            source = {path: b"//fixture\n" for path in FILES}
            for revision in ("", "a" * 39, "g" * 40, "main", None):
                with self.assertRaises(Invalid): build(source, revision)
            for name in ("../escape", "/absolute", "core/crypto/extra.rs", "source/target.rs"):
                bad = dict(source); bad[name] = b"extra"
                with self.assertRaises(Invalid): build(bad, "a" * 40)
            bad = dict(source); bad.pop(next(iter(FILES)))
            with self.assertRaises(Invalid): build(bad, "a" * 40)
        def test_per_file_and_aggregate_budgets(self):
            source = {path: b"x" for path in FILES}
            for value in (b"", "text", bytearray(b"x"), b"x" * (256 * 1024 + 1)):
                bad = dict(source); bad[next(iter(FILES))] = value
                with self.assertRaises(Invalid): build(bad, "a" * 40)
            with self.assertRaises(Invalid): build({path: b"x" * (128 * 1024) for path in FILES}, "a" * 40)
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test only")
    self_test()
