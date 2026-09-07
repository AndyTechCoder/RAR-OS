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


def _git_hash(kind, raw):
    return hashlib.sha1(kind + b" " + str(len(raw)).encode("ascii") + b"\0" + raw).hexdigest()

def _tree_entries(raw):
    if type(raw) is not bytes or not 1 <= len(raw) <= 131072:
        raise Invalid("Git tree budget")
    entries = {}
    at = 0
    while at < len(raw):
        space = raw.find(b" ", at)
        end = raw.find(b"\0", space + 1)
        if space < at or end < space or end + 21 > len(raw):
            raise Invalid("Git tree framing")
        mode, name = raw[at:space], raw[space + 1:end]
        if (mode not in (b"40000", b"100644", b"100755", b"120000", b"160000") or
            not 1 <= len(name) <= 255 or b"/" in name or name in (b".", b"..") or name in entries):
            raise Invalid("Git tree entry")
        entries[name] = (mode, raw[end + 1:end + 21].hex())
        at = end + 21
        if len(entries) > 4096:
            raise Invalid("Git tree entry count")
    return entries

def build_from_objects(revision, commit, trees, blobs):
    """Bind exact proposal blobs as data; caller separately selects the revision.
    Obtain raw Git objects with replacement objects disabled, without checkout,
    hooks, filters, attributes, LFS resolution or submodules. This function does
    no acquisition and trusts neither filenames nor object-ID labels alone.
    """
    if (type(revision) is not str or re.fullmatch(r"[0-9a-f]{40}", revision) is None or
        type(commit) is not bytes or not 1 <= len(commit) <= 65536 or
        _git_hash(b"commit", commit) != revision or type(trees) is not dict or
        type(blobs) is not dict or not 1 <= len(trees) <= 32 or not 1 <= len(blobs) <= 5):
        raise Invalid("Git commit/object envelope")
    header = commit.split(b"\n\n", 1)[0].split(b"\n")
    if (not header or re.fullmatch(b"tree [0-9a-f]{40}", header[0]) is None or
        sum(line.startswith(b"tree ") for line in header) != 1):
        raise Invalid("Git commit tree")
    for objects, kind, maximum, total in ((trees, b"tree", 131072, 2097152),
                                         (blobs, b"blob", 262144, 524288)):
        if (any(type(key) is not str or re.fullmatch(r"[0-9a-f]{40}", key) is None or
                type(raw) is not bytes or not 1 <= len(raw) <= maximum or
                _git_hash(kind, raw) != key for key, raw in objects.items()) or
            sum(map(len, objects.values())) > total):
            raise Invalid("Git object identity/budget")
    root = header[0][5:].decode("ascii")
    used_trees = set()
    used_blobs = set()
    source = {}
    identities = {}
    for path in sorted(FILES):
        tree = root
        parts = path.encode("ascii").split(b"/")
        for number, component in enumerate(parts):
            if tree not in trees:
                raise Invalid("missing Git tree")
            used_trees.add(tree)
            entry = _tree_entries(trees[tree]).get(component)
            if entry is None:
                raise Invalid("missing fixed Git path")
            mode, oid = entry
            if number + 1 < len(parts):
                if mode != b"40000":
                    raise Invalid("Git source directory mode")
                tree = oid
            else:
                if mode != b"100644" or oid not in blobs:
                    raise Invalid("Git source regular blob")
                raw = blobs[oid]
                if raw.startswith(b"version https://git-lfs.github.com/spec/v1\n"):
                    raise Invalid("LFS source pointer")
                used_blobs.add(oid)
                source[path] = raw
                identities[path] = oid
    if used_trees != set(trees) or used_blobs != set(blobs):
        raise Invalid("unneeded Git objects")
    layer, report = build(source, revision)
    report["source_tree"] = root
    report["source_commit_sha256"] = hashlib.sha256(commit).hexdigest()
    report["git_blobs"] = identities
    report["source_commit_size"] = len(commit)
    report["git_trees"] = {oid: {"sha256": hashlib.sha256(trees[oid]).hexdigest(),
                                  "size": len(trees[oid])} for oid in sorted(used_trees)}
    return layer, report

def self_test():
    import unittest

    def object_fixture(change_path=None, extra_blob=False, change_mode=b"100755", lfs=False, values=None):
        source={path: ("// "+path+"\n").encode() for path in FILES}
        if values is not None: source=dict(zip(sorted(FILES),values))
        if lfs: source[sorted(FILES)[0]]=b"version https://git-lfs.github.com/spec/v1\noid sha256:fixture\n"
        trees={};blobs={};root={}
        for path,value in source.items():
            cursor=root
            parts=path.split("/")
            for name in parts[:-1]:
                cursor=cursor.setdefault(name,{})
            cursor[parts[-1]]=value
        def encode(node,prefix=""):
            raw=bytearray()
            for name,value in sorted(node.items(),key=lambda item:
                    (item[0]+("/" if isinstance(item[1],dict) else "")).encode()):
                path=prefix+name
                if isinstance(value,dict):
                    mode=b"40000";oid=encode(value,path+"/")
                else:
                    mode=b"100644"
                    oid=_git_hash(b"blob",value);blobs[oid]=value
                if path==change_path: mode=change_mode
                raw.extend(mode+b" "+name.encode()+b"\0"+bytes.fromhex(oid))
            raw=bytes(raw);oid=_git_hash(b"tree",raw);trees[oid]=raw;return oid
        tree=encode(root)
        commit=(b"tree "+tree.encode()+b"\nauthor RAR <lab@example.invalid> 1 +0000\n"
                b"committer RAR <lab@example.invalid> 1 +0000\n\nfixture\n")
        if extra_blob:
            blobs[_git_hash(b"blob",b"extra")]=b"extra"
        return _git_hash(b"commit",commit),commit,trees,blobs,source

    class Tests(unittest.TestCase):

        def test_exact_git_object_source_binding(self):
            revision,commit,trees,blobs,source=object_fixture()
            layer,report=build_from_objects(revision,commit,trees,blobs)
            self.assertEqual(layer,build(source,revision)[0])
            self.assertEqual(report["source_tree"],commit.splitlines()[0][5:].decode())
            self.assertEqual(report["git_blobs"],{p:_git_hash(b"blob",v) for p,v in source.items()})
            self.assertEqual(report["source_commit_sha256"],hashlib.sha256(commit).hexdigest())
            self.assertEqual(report["source_commit_size"],len(commit))
            self.assertEqual(list(report["git_trees"]),sorted(trees))
            self.assertEqual(report["git_trees"],{k:{"sha256":hashlib.sha256(v).hexdigest(),
                                                    "size":len(v)} for k,v in trees.items()})
        def test_git_labels_corruption_missing_and_extra_objects_refused(self):
            revision,commit,trees,blobs,_=object_fixture()
            with self.assertRaises(Invalid): build_from_objects("f"*40,commit,trees,blobs)
            with self.assertRaises(Invalid): build_from_objects(revision,commit+b"x",trees,blobs)
            for objects in (trees,blobs):
                key=next(iter(objects))
                for replacement in (b"x",None):
                    bad=dict(objects)
                    if replacement is None: bad.pop(key)
                    else: bad[key]=replacement
                    with self.assertRaises(Invalid):
                        build_from_objects(revision,commit,bad if objects is trees else trees,
                                           bad if objects is blobs else blobs)
            r,c,t,b,_=object_fixture(extra_blob=True)
            with self.assertRaises(Invalid): build_from_objects(r,c,t,b)
            for path in FILES:
                r,c,t,b,_=object_fixture(change_path=path)
                with self.assertRaises(Invalid): build_from_objects(r,c,t,b)


        def test_unused_objects_and_resource_boundaries(self):
            r,c,t,b,_=object_fixture(values=[b"// shared"]*5,extra_blob=True)
            with self.assertRaisesRegex(Invalid,"unneeded Git objects"):
                build_from_objects(r,c,t,b)
            r,c,t,b,_=object_fixture()
            extra=b"100644 unused\0"+bytes.fromhex(next(iter(b)))
            additional=dict(t);additional[_git_hash(b"tree",extra)]=extra
            with self.assertRaisesRegex(Invalid,"unneeded Git objects"):
                build_from_objects(r,c,additional,b)
            additional=dict(t)
            for n in range(33):
                raw=b"100644 unused"+str(n).encode()+b"\0"+bytes(20)
                additional[_git_hash(b"tree",raw)]=raw
            with self.assertRaisesRegex(Invalid,"Git commit/object envelope"):
                build_from_objects(r,c,additional,b)
            additional=dict(t)
            for n in range(17):
                raw=bytes([n])*131072
                additional[_git_hash(b"tree",raw)]=raw
            with self.assertRaisesRegex(Invalid,"Git object identity/budget"):
                build_from_objects(r,c,additional,b)
            values=[b"a"*262144]+[bytes([n])*65536 for n in range(1,5)]
            r,c,t,b,_=object_fixture(values=values)
            build_from_objects(r,c,t,b)  # Exact per-blob and aggregate limits.
            for index in [0,1]:
                bad=list(values);bad[index]+=b"x"
                r,c,t,b,_=object_fixture(values=bad)
                with self.assertRaisesRegex(Invalid,"Git object identity/budget"):
                    build_from_objects(r,c,t,b)
        def test_no_link_submodule_lfs_or_executable_source(self):
            for path in (sorted(FILES)[0],"core","tools/rar-lab"):
                for mode in (b"100755",b"120000",b"160000"):
                    r,c,t,b,_=object_fixture(change_path=path,change_mode=mode)
                    with self.assertRaises(Invalid): build_from_objects(r,c,t,b)
            r,c,t,b,_=object_fixture(lfs=True)
            with self.assertRaises(Invalid): build_from_objects(r,c,t,b)
            r,c,t,b,_=object_fixture()
            duplicate=c.split(b"\n",1)[0]+b"\n"+c
            with self.assertRaises(Invalid):
                build_from_objects(_git_hash(b"commit",duplicate),duplicate,t,b)
        def test_git_tree_framing_and_duplicate_paths(self):
            entry=b"100644 file\0"+bytes.fromhex("a"*40)
            self.assertEqual(_tree_entries(entry),{b"file":(b"100644","a"*40)})
            for raw in (entry+entry,entry[:-1],b"100644 ../file\0"+bytes(20),
                        b"000000 file\0"+bytes(20),b"40000\0"+bytes(20)):
                with self.assertRaises(Invalid): _tree_entries(raw)
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
