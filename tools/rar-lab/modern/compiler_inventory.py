"""Bounded compiler Docker-save inspection; never extracts or starts an image.
Image/graph equality is evidence, not authorization or proof of runtime usability.
"""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import posixpath
import re
import tarfile

SYSROOT = "/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu"
RUSTC = SYSROOT + "/bin/rustc"
LLD = SYSROOT + "/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"
MUSL = SYSROOT + "/lib/rustlib/x86_64-unknown-linux-musl/lib/"
# Positive scratch search domains for the pinned x86-64 GNU bootstrap.
# All same-named ELF copies must be identical, so search precedence cannot
# select different bytes. Inherited-only paths and cache-only resolution fail.
DEFAULT_DIRS = ("/lib/x86_64-linux-gnu", "/usr/lib/x86_64-linux-gnu",
                "/lib64", "/usr/lib64", "/lib", "/usr/lib")
OMITTED_MUSL = {"source": MUSL + "libstd-286e4795762d614b.so", "size": 5369608,
                "sha256": "5a1f8cfcc59c4cafc031df4f648b20fb1674cc190c8b40b8d391c33ad3e391d1",
                "reason": "static-musl-only"}
EPOCH = 1785715200
LIMIT = 2 * 1024 * 1024 * 1024

class Invalid(ValueError):
    pass


# Independently pinned distribution notices, not trusted from the exporter.
ARCHIVE_NOTICE_ROOT = "/build/rust-std-1.95.0-x86_64-unknown-linux-musl"
ARCHIVE_NOTICES = {
    "LICENSE-APACHE": (9723, "62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a"),
    "LICENSE-MIT": (1068, "b71bd43a069ca0641a9ecfe585ca7b3c53b5cc1608f8b68321168698e28b5ea1"),
    "COPYRIGHT": (1571, "172020dbfd5b53a226dfde77616190a48dcff519b0bc0e6deb91a8450782c4af"),
}
INSTALLED_NOTICES = ("COPYRIGHT.html", "COPYRIGHT-library.html",
                     "licenses/Apache-2.0.txt", "licenses/MIT.txt",
                     "licenses/LLVM-exception.txt")


# Generated aggregate Rust copyright reports are large inert HTML documents.
# Only exact installed sources AND exact destinations receive the larger budget.
NOTICE_TOTAL_LIMIT = 48 * 1024 * 1024
def notice_limit(source, relative):
    for name in ("COPYRIGHT.html", "COPYRIGHT-library.html"):
        if (str(source) == str(SYSROOT) + "/share/doc/rust/" + name and
            relative == "rust/" + name):
            return 16 * 1024 * 1024
    return 1024 * 1024

def required_notices(notices):
    for name, (size, sha256) in ARCHIVE_NOTICES.items():
        entry = notices.get("rust/" + name)
        if (type(entry) is not dict or entry.get("source") != ARCHIVE_NOTICE_ROOT + "/" + name or
            type(entry.get("size")) is not int or entry["size"] != size or
            entry.get("sha256") != sha256 or entry.get("mode") != 0o444):
            raise Invalid("required pinned archive notice")
    for name in INSTALLED_NOTICES:
        entry = notices.get("rust/" + name)
        if (type(entry) is not dict or
            entry.get("source") != SYSROOT + "/share/doc/rust/" + name):
            raise Invalid("required installed Rust notice")

def unique_json(raw):
    def pairs(items):
        out = {}
        for key, value in items:
            if key in out: raise Invalid("duplicate JSON key")
            out[key] = value
        return out
    try:
        return json.loads(raw, object_pairs_hook=pairs)
    except (ValueError, UnicodeError, RecursionError) as exc:
        raise Invalid("JSON") from exc

def digest(value):
    return type(value) is str and re.fullmatch(r"[0-9a-f]{64}", value) is not None

def runtime_path(value):
    if (type(value) is not str or len(value) > 512 or
        re.fullmatch(r"/[A-Za-z0-9_./+-]+", value) is None or
        any(x in ("", ".", "..") for x in value.split("/")[1:]) or
        not value.startswith((SYSROOT + "/", "/lib/", "/lib64/", "/usr/lib/", "/usr/lib64/")) or
        any(x in value.lower() for x in ("libcrypto", "libssl", "libsodium", "openssl", "reference-"))):
        raise Invalid("compiler runtime path")
    return value

def archive_name(item):
    name = item.name
    if name.startswith("./"): name = name[2:]
    if item.isdir(): name = name.rstrip("/")
    if (not name or len(name) > 768 or name.startswith("/") or
        re.fullmatch(r"[A-Za-z0-9_./+-]+", name) is None or
        any(x in ("", ".", "..") for x in name.split("/")) or
        item.sparse is not None or not (item.isfile() or item.isdir()) or
        item.size < 0 or (item.isdir() and item.size)):
        raise Invalid("archive member")
    return name

def validate(config, report, files, directories):
    if type(config) is not dict or type(files) is not dict or type(directories) is not dict:
        raise Invalid("inventory shape")
    for entry in files.values():
        if (type(entry) is not dict or
            any(key not in entry for key in ("size", "sha256", "mode", "uid", "gid", "mtime", "elf"))):
            raise Invalid("file inventory shape")
    process = config.get("config")
    if (config.get("architecture") != "amd64" or config.get("os") != "linux" or
        type(process) is not dict or process.get("User") != "65532:65532" or
        process.get("WorkingDir") != "/source" or process.get("Entrypoint") != [RUSTC] or
        process.get("Env") != ["PATH=/nonexistent", "LD_LIBRARY_PATH=" + SYSROOT + "/lib"]):
        raise Invalid("compiler process configuration")
    for key in ("Cmd", "OnBuild", "Shell"):
        if process.get(key) not in (None, []): raise Invalid("compiler command/config")
    for key in ("Volumes", "ExposedPorts", "Healthcheck", "Labels"):
        if process.get(key) not in (None, {}): raise Invalid("compiler extra config")
    if (type(report) is not dict or report.get("state") != "private-closure-export-only" or
        report.get("target_execution") is not False or report.get("accepted_compiler_image") is not False):
        raise Invalid("construction report state")
    if report.get("omitted_musl_dynamic") != [OMITTED_MUSL]:
        raise Invalid("pinned static-only sysroot omission evidence")
    declared = report.get("files")
    graph = report.get("graph")
    license_report = report.get("licenses")
    if (type(declared) is not dict or not 1 <= len(declared) <= 4096 or
        type(graph) is not dict or not 2 <= len(graph) <= 128 or
        RUSTC not in graph or LLD not in graph or
        type(license_report) is not dict or license_report.get("state") != "captured-not-legally-certified"):
        raise Invalid("construction report shape")
    if (OMITTED_MUSL["source"] in declared or OMITTED_MUSL["source"] in graph or
        OMITTED_MUSL["source"][1:] in files):
        raise Invalid("omitted musl dynamic library present")
    notices = license_report.get("files")
    if type(notices) is not dict or not 1 <= len(notices) <= 512:
        raise Invalid("notice inventory")
    required_notices(notices)
    expected = {"evidence/compiler-closure.json"}
    total = 0
    for path, entry in declared.items():
        runtime_path(path)
        if (type(entry) is not dict or type(entry.get("size")) is not int or
            not 1 <= entry["size"] <= 256 * 1024 * 1024 or not digest(entry.get("sha256")) or
            entry.get("mode") not in (0o444, 0o555)):
            raise Invalid("declared runtime file")
        runtime_path(entry.get("source"))
        name = path[1:]; expected.add(name)
        actual = files.get(name)
        if (actual is None or any(actual[k] != entry[k] for k in ("size", "sha256", "mode")) or
            actual["uid"] != 0 or actual["gid"] != 0 or actual["mtime"] != EPOCH):
            raise Invalid("runtime bytes/metadata mismatch")
        total += entry["size"]
    if total > 1610612736 or type(report.get("total_bytes")) is not int or report["total_bytes"] != total:
        raise Invalid("runtime byte total")
    aliases = set()
    byte_edges = {}
    inspected_search_directories = set()
    for path, item in graph.items():
        runtime_path(path)
        if path not in declared or type(item) is not dict:
            raise Invalid("graph node")
        actual = files[path[1:]]
        byte_edges[path] = set()
        elf = actual["elf"]
        if elf is None or actual["mode"] != 0o555:
            raise Invalid("graph file is not an inspected executable")
        canonical = declared[path]["source"]
        aliases.add(canonical)
        if (canonical not in declared or files[canonical[1:]]["sha256"] != actual["sha256"] or
            files[canonical[1:]]["size"] != actual["size"]):
            raise Invalid("exported alias differs")
        if item.get("needed") != elf["needed"] or item.get("interpreter") != elf["interpreter"]:
            raise Invalid("ELF metadata differs from construction graph")
        resolved = item.get("resolved")
        if (type(resolved) is not list or len(resolved) > 128 or
            any(type(p) is not str for p in resolved) or len(set(resolved)) != len(resolved)):
            raise Invalid("resolved dependency set")
        for dep in resolved:
            runtime_path(dep)
            if dep not in graph or dep not in declared or files[dep[1:]]["elf"] is None:
                raise Invalid("missing resolved dependency")
        if elf["interpreter"] is not None:
            runtime_path(elf["interpreter"])
            if elf["interpreter"] not in resolved:
                raise Invalid("interpreter missing from dependency set")
            byte_edges[path].add(elf["interpreter"])
        search = set()
        if elf["search"] is not None:
            for origin in (posixpath.dirname(path), posixpath.dirname(canonical)):
                for entry in elf["search"]["entries"]:
                    directory = runtime_path(posixpath.normpath(origin + entry[len("$ORIGIN"):]))
                    if directory[1:] not in directories:
                        raise Invalid("search directory not exported")
                    search.add(directory)
        inspected_search_directories.update(search)
        if item.get("search_paths") != sorted(search):
            raise Invalid("ELF search paths differ")
        for name in elf["needed"]:
            # SONAME is not a pathname. The exact NEEDED filename must exist
            # in a direct search directory; a claimed ldd path alone is not proof.
            reachable = [directory + "/" + name
                         for directory in [SYSROOT + "/lib", *sorted(search), *DEFAULT_DIRS]
                         if (directory + "/" + name)[1:] in files]
            matches = [files[p[1:]] for p in reachable]
            same_named = [entry for filename, entry in files.items()
                          if posixpath.basename(filename) == name]
            if (not matches or any(entry["elf"] is None for entry in matches) or
                any(entry["elf"]["soname"] not in (None, name) for entry in matches) or
                len({entry["sha256"] for entry in same_named}) != 1):
                raise Invalid("dependency filename missing or shadowed")
            selected = matches[0]["sha256"]
            if not any(files[p[1:]]["sha256"] == selected and
                       posixpath.basename(p) == name for p in resolved):
                raise Invalid("loader-visible dependency differs from trace")
            # Only actual ELF NEEDED/interpreter edges confer reachability.
            # A forged extra ldd resolved edge must not admit a disconnected ELF.
            byte_edges[path].update(p for p in resolved
                if files[p[1:]]["sha256"] == selected and posixpath.basename(p) == name)

    for path in set(declared) - set(graph) - aliases:
        if (not path.startswith(MUSL) or not path.endswith((".rlib", ".rmeta", ".a", ".o")) or
            declared[path]["mode"] != 0o444):
            raise Invalid("unexpected compiler payload outside runtime graph")
    if not any(path.startswith(MUSL) and path.endswith(".rlib") for path in declared):
        raise Invalid("musl sysroot missing")
    backend = report.get("codegen_backend")
    backend_dir = SYSROOT + "/lib/rustlib/x86_64-unknown-linux-gnu/codegen-backends/"
    backend_nodes = {path for path in set(declared) | set(graph) if path.startswith(backend_dir)}
    expected_backends = set() if backend == "builtin-in-driver" else ({backend} if type(backend) is str else set())
    if backend_nodes != expected_backends:
        raise Invalid("backend selection differs from exported files")
    if backend != "builtin-in-driver":
        if (backend not in (backend_dir + "librustc_codegen_llvm.so",
                            backend_dir + "librustc_codegen_llvm-1.95.0.so") or backend not in graph):
            raise Invalid("codegen backend evidence")
    pending = [RUSTC, LLD] + ([] if backend == "builtin-in-driver" else [backend])
    reached = set()
    while pending:
        node = pending.pop()
        if node in reached:
            continue
        if node not in byte_edges:
            raise Invalid("ELF dependency absent from graph")
        reached.add(node)
        pending.extend(byte_edges[node] - reached)
    if reached != set(graph):
        raise Invalid("disconnected executable graph node")
    probe = report.get("backend_probe")
    if (type(probe) is not dict or
        set(probe) != {"llvm_version", "version_sha256", "target_cpus_sha256"} or
        type(probe["llvm_version"]) is not str or
        re.fullmatch(r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}", probe["llvm_version"]) is None or
        not digest(probe["version_sha256"]) or not digest(probe["target_cpus_sha256"])):
        raise Invalid("positive backend probe evidence")
    packages = license_report.get("runtime_packages")
    if type(packages) is not dict or len(packages) > 128:
        raise Invalid("runtime package inventory")
    external = {entry["source"] for entry in declared.values()
                if not entry["source"].startswith(SYSROOT + "/")}
    covered = set()
    for owner, entry in packages.items():
        if (type(owner) is not str or len(owner) > 128 or
            re.fullmatch(r"[a-z0-9][a-z0-9+.-]*(?::[a-z0-9][a-z0-9-]*)?", owner) is None or
            type(entry) is not dict or type(entry.get("identity")) is not str or
            len(entry["identity"]) > 512 or
            re.fullmatch(re.escape(owner) + r"\t[^\s]+", entry["identity"]) is None or
            type(entry.get("files")) is not list or not 1 <= len(entry["files"]) <= 128 or
            any(type(path) is not str for path in entry["files"]) or
            len(set(entry["files"])) != len(entry["files"]) or
            any(path not in external or path in covered for path in entry["files"]) or
            "packages/" + owner.replace(":", "-") + ".copyright" not in notices):
            raise Invalid("runtime package provenance")
        covered.update(entry["files"])
    if covered != external:
        raise Invalid("runtime package coverage")
    notice_total = 0
    for relative, entry in notices.items():
        if (type(relative) is not str or len(relative) > 256 or
            re.fullmatch(r"[A-Za-z0-9_.+/-]+", relative) is None or
            any(x in ("", ".", "..") for x in relative.split("/")) or
            type(entry) is not dict or type(entry.get("size")) is not int or
            not 1 <= entry["size"] <= notice_limit(entry.get("source"), relative) or entry.get("mode") != 0o444 or
            not digest(entry.get("sha256"))):
            raise Invalid("notice declaration")
        name = "licenses/" + relative; expected.add(name)
        actual = files.get(name)
        if (actual is None or any(actual[k] != entry[k] for k in ("size", "sha256", "mode")) or
            actual["uid"] != 0 or actual["gid"] != 0 or actual["mtime"] != EPOCH or actual["elf"] is not None):
            raise Invalid("notice bytes/metadata")
        notice_total += entry["size"]
    if (notice_total > NOTICE_TOTAL_LIMIT or type(license_report.get("total_bytes")) is not int or
        license_report["total_bytes"] != notice_total):
        raise Invalid("notice byte total")
    evidence = files.get("evidence/compiler-closure.json")
    if (evidence is None or evidence["mode"] != 0o444 or evidence["uid"] != 0 or
        evidence["gid"] != 0 or evidence["mtime"] != EPOCH or evidence["elf"] is not None):
        raise Invalid("report metadata")
    if set(files) != expected:
        raise Invalid("extra or missing image file")
    expected_dirs = {"source", "build"}
    for directory in inspected_search_directories:
        parts = directory[1:].split("/")
        expected_dirs.update("/".join(parts[:n]) for n in range(1, len(parts)+1))
    for name in expected:
        parts = name.split("/")
        expected_dirs.update("/".join(parts[:n]) for n in range(1, len(parts)))
    if set(directories) != expected_dirs:
        raise Invalid("extra or missing image directory")
    for name, metadata in directories.items():
        wanted = (0o700, 65532, 65532, EPOCH) if name == "build" else (0o555, 0, 0, EPOCH)
        if tuple(metadata) != wanted:
            raise Invalid("compiler directory metadata")
    return {"files": files, "directories": directories, "runtime_bytes": total,
            "notice_bytes": notice_total, "state": "inspected-not-activated"}

def inspect(raw, image):
    if type(raw) is not bytes or not 1 <= len(raw) <= LIMIT or not isinstance(image, str) or not digest(image.removeprefix("sha256:")) or not image.startswith("sha256:"):
        raise Invalid("image bytes/identity")
    module_path = Path(__file__).resolve().with_name("compiler_elf.py")
    if module_path.is_symlink() or not module_path.is_file():
        raise Invalid("fixed ELF inspector path")
    spec = importlib.util.spec_from_file_location("modern_compiler_elf", module_path)
    elf_reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(elf_reader)
    try:
        with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as outer:
            indexed = {}
            for index, item in enumerate(outer):
                name = archive_name(item)
                if index >= 64 or name in indexed or item.size > LIMIT:
                    raise Invalid("outer archive bounds/duplicates")
                indexed[name] = item
            def outer_file(name, maximum):
                if type(name) is not str or name not in indexed or not indexed[name].isfile() or indexed[name].size > maximum:
                    raise Invalid("outer file reference")
                return outer.extractfile(indexed[name])
            with outer_file("manifest.json", 65536) as stream:
                manifest = unique_json(stream.read(65537))
            if type(manifest) is not list or len(manifest) != 1 or type(manifest[0]) is not dict:
                raise Invalid("single image manifest")
            with outer_file(manifest[0].get("Config"), 65536) as stream:
                config_bytes = stream.read(65537)
            if "sha256:" + hashlib.sha256(config_bytes).hexdigest() != image:
                raise Invalid("config identity")
            config = unique_json(config_bytes)
            if type(config) is not dict: raise Invalid("image config")
            layers = manifest[0].get("Layers")
            rootfs = config.get("rootfs")
            if (type(layers) is not list or not 1 <= len(layers) <= 8 or
                any(type(p) is not str for p in layers) or len(set(layers)) != len(layers) or
                type(rootfs) is not dict or rootfs.get("type") != "layers" or
                type(rootfs.get("diff_ids")) is not list or len(rootfs["diff_ids"]) != len(layers)):
                raise Invalid("image layer identities")
            files = {}; directories = {}; report = None; expanded = 0; count = 0
            for name, identity in zip(layers, rootfs["diff_ids"]):
                with outer_file(name, LIMIT) as stream:
                    hashed = hashlib.sha256()
                    while True:
                        part = stream.read(65536)
                        if not part: break
                        hashed.update(part)
                    if "sha256:" + hashed.hexdigest() != identity:
                        raise Invalid("image layer digest")
                    stream.seek(0)
                    with tarfile.open(fileobj=stream, mode="r|") as layer:
                        for item in layer:
                            name = archive_name(item); count += 1; expanded += item.size
                            if count > 16384 or expanded > LIMIT or item.size > 256 * 1024 * 1024:
                                raise Invalid("image expansion bounds")
                            if name in files or name in directories:
                                raise Invalid("duplicate/replaced image member")
                            metadata = (item.mode, item.uid, item.gid, item.mtime)
                            if item.isdir():
                                directories[name] = metadata
                                continue
                            stream_file = layer.extractfile(item)
                            if stream_file is None: raise Invalid("image payload")
                            if name == "evidence/compiler-closure.json" and item.size > 8 * 1024 * 1024:
                                raise Invalid("report size")
                            value = stream_file.read(item.size + 1)
                            if len(value) != item.size: raise Invalid("image payload length")
                            elf = None
                            if value.startswith(b"\x7fELF"):
                                try: elf = elf_reader.inspect(value)
                                except ValueError as exc: raise Invalid("compiler ELF rejected") from exc
                            files[name] = {"size": item.size, "sha256": hashlib.sha256(value).hexdigest(),
                                           "mode": item.mode, "uid": item.uid, "gid": item.gid,
                                           "mtime": item.mtime, "elf": elf}
                            if name == "evidence/compiler-closure.json":
                                if len(value) > 8 * 1024 * 1024: raise Invalid("report size")
                                report = unique_json(value)
            result = validate(config, report, files, directories)
            return {"image": image, "diff_ids": rootfs["diff_ids"], **result}
    except (tarfile.TarError, OSError, EOFError, OverflowError) as exc:
        raise Invalid("compiler image archive") from exc

def self_test():
    import copy
    import struct
    import unittest

    # A synthetic, nonfunctional ELF is data only: no compiler/image is started.
    def elf_fixture():
        raw = bytearray(256)
        raw[:7] = b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HHI", raw, 16, 3, 62, 1)
        struct.pack_into("<Q", raw, 32, 64)
        struct.pack_into("<HHH", raw, 52, 64, 56, 2)
        struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0x400000, 0, 256, 256, 4096)
        struct.pack_into("<IIQQQQQQ", raw, 120, 0x6474e551, 6, 0, 0, 0, 0, 0, 16)
        return bytes(raw)

    def fixture():
        payloads = {RUSTC[1:]: elf_fixture(), LLD[1:]: elf_fixture(),
                    (MUSL + "libstd-fixture.rlib")[1:]: b"!<arch>\nfixture",
                    }
        notice_values = unique_json(Path(__file__).with_name("compiler-notices.json").read_bytes())
        notices = {}
        for name, (size, sha256) in ARCHIVE_NOTICES.items():
            value = notice_values[name].encode()
            if len(value) != size or hashlib.sha256(value).hexdigest() != sha256:
                raise Invalid("pinned test notice bytes")
            payloads["licenses/rust/" + name] = value
            notices["rust/" + name] = {"source": ARCHIVE_NOTICE_ROOT + "/" + name,
                "size": size, "sha256": sha256, "mode": 0o444}
        for name in INSTALLED_NOTICES:
            value = ("synthetic installed notice: " + name).encode()
            payloads["licenses/rust/" + name] = value
            notices["rust/" + name] = {"source": SYSROOT + "/share/doc/rust/" + name,
                "size": len(value), "sha256": hashlib.sha256(value).hexdigest(), "mode": 0o444}
        declared = {}
        graph = {}
        for path in (RUSTC, LLD, MUSL + "libstd-fixture.rlib"):
            value = payloads[path[1:]]
            declared[path] = {"size": len(value), "sha256": hashlib.sha256(value).hexdigest(),
                              "mode": 0o555 if path in (RUSTC, LLD) else 0o444, "source": path}
        for path in (RUSTC, LLD):
            graph[path] = {"needed": [], "interpreter": None, "resolved": [], "search_paths": []}
        report = {"state": "private-closure-export-only", "target_execution": False,
                  "accepted_compiler_image": False, "files": declared, "graph": graph,
                  "omitted_musl_dynamic": [dict(OMITTED_MUSL)],
                  "codegen_backend": "builtin-in-driver",
                  "backend_probe": {"llvm_version": "22.1.0", "version_sha256": "1" * 64,
                                    "target_cpus_sha256": "2" * 64},
                  "total_bytes": sum(x["size"] for x in declared.values()),
                  "licenses": {"state": "captured-not-legally-certified",
                    "files": notices,
                    "total_bytes": sum(x["size"] for x in notices.values()), "runtime_packages": {}}}
        config = {"architecture": "amd64", "os": "linux",
                  "config": {"User": "65532:65532", "WorkingDir": "/source",
                             "Entrypoint": [RUSTC],
                             "Env": ["PATH=/nonexistent", "LD_LIBRARY_PATH=" + SYSROOT + "/lib"]}}
        return config, report, payloads


    def dynamic_fixture(logical="/lib/x86_64-linux-gnu/libfixture.so"):
        config, report, payloads = fixture()
        def dynamic_elf(tag):
            raw = bytearray(512); raw[:7] = b"\x7fELF\x02\x01\x01"
            struct.pack_into("<HHI", raw, 16, 3, 62, 1)
            struct.pack_into("<Q", raw, 32, 64)
            struct.pack_into("<HHH", raw, 52, 64, 56, 3)
            struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0x400000, 0, 512, 512, 4096)
            struct.pack_into("<IIQQQQQQ", raw, 120, 2, 4, 256, 0x400100, 0, 64, 64, 8)
            struct.pack_into("<IIQQQQQQ", raw, 176, 0x6474e551, 6, 0, 0, 0, 0, 0, 16)
            strings = b"\0libfixture.so\0"
            for index, (kind, value) in enumerate(((tag, 1), (5, 0x400180), (10, len(strings)), (0, 0))):
                struct.pack_into("<QQ", raw, 256 + index * 16, kind, value)
            raw[384:384 + len(strings)] = strings
            return bytes(raw)
        canonical = "/usr/lib/x86_64-linux-gnu/libfixture-real.so"
        def add(path, value, source):
            payloads[path[1:]] = value
            report["files"][path] = {"source": source, "size": len(value),
                "sha256": hashlib.sha256(value).hexdigest(), "mode": 0o555}
        add(RUSTC, dynamic_elf(1), RUSTC)
        add(logical, dynamic_elf(14), canonical)
        add(canonical, dynamic_elf(14), canonical)
        report["graph"][RUSTC]["needed"] = ["libfixture.so"]
        report["graph"][RUSTC]["resolved"] = [logical]
        report["graph"][logical] = {"needed": [], "interpreter": None, "resolved": [], "search_paths": []}
        report["total_bytes"] = sum(x["size"] for x in report["files"].values())
        value = b"synthetic runtime package notice"
        payloads["licenses/packages/fixture-amd64.copyright"] = value
        report["licenses"]["files"]["packages/fixture-amd64.copyright"] = {
            "size": len(value), "mode": 0o444, "sha256": hashlib.sha256(value).hexdigest()}
        report["licenses"]["total_bytes"] += len(value)
        report["licenses"]["runtime_packages"] = {"fixture:amd64": {
            "identity": "fixture:amd64\t1.0", "files": [canonical]}}
        return config, report, payloads

    def tar_bytes(entries):
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w", format=tarfile.USTAR_FORMAT) as archive:
            for name, value, mode, uid, gid, kind in entries:
                item = tarfile.TarInfo(name)
                item.mode = mode; item.uid = uid; item.gid = gid; item.mtime = EPOCH; item.type = kind
                item.size = len(value)
                if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE): item.linkname = RUSTC
                archive.addfile(item, io.BytesIO(value) if value else None)
        return output.getvalue()

    def image_bytes(config, report, payloads, extra=(), mutate=None, duplicate_json=False):
        payloads = dict(payloads)
        payloads["evidence/compiler-closure.json"] = json.dumps(report, sort_keys=True).encode()
        dirs = {"source", "build"}
        for name in payloads:
            parts = name.split("/")
            dirs.update("/".join(parts[:n]) for n in range(1, len(parts)))
        entries = [(name, b"", 0o700 if name == "build" else 0o555,
                    65532 if name == "build" else 0, 65532 if name == "build" else 0, tarfile.DIRTYPE)
                   for name in sorted(dirs)]
        for name, value in sorted(payloads.items()):
            mode = report["files"].get("/" + name, {}).get("mode", 0o444)
            entries.append((name, value, mode, 0, 0, tarfile.REGTYPE))
        entries.extend(extra)
        if mutate is not None: entries = mutate(entries)
        layer = tar_bytes(entries)
        config = copy.deepcopy(config)
        config["rootfs"] = {"type": "layers", "diff_ids": ["sha256:" + hashlib.sha256(layer).hexdigest()]}
        cb = json.dumps(config, sort_keys=True).encode()
        if duplicate_json: cb = b'{"os":"invalid",' + cb[1:]
        image = "sha256:" + hashlib.sha256(cb).hexdigest()
        manifest = json.dumps([{"Config": "config.json", "Layers": ["layer.tar"], "RepoTags": []}]).encode()
        return tar_bytes([(name, value, 0o644, 0, 0, tarfile.REGTYPE)
                          for name, value in (("manifest.json", manifest), ("config.json", cb), ("layer.tar", layer))]), image

    class Tests(unittest.TestCase):
        def test_full_positive_archive_is_inspected_not_activated(self):
            raw, identity = image_bytes(*fixture())
            result = inspect(raw, identity)
            self.assertEqual(result["state"], "inspected-not-activated")
            self.assertEqual(len(result["files"]), 12)
            self.assertEqual(result["directories"]["build"], (0o700, 65532, 65532, EPOCH))

        def test_required_notices_cannot_be_omitted_even_with_consistent_report(self):
            for name in [*ARCHIVE_NOTICES, *INSTALLED_NOTICES]:
                config, report, payloads = fixture()
                entry = report["licenses"]["files"].pop("rust/" + name)
                del payloads["licenses/rust/" + name]
                report["licenses"]["total_bytes"] -= entry["size"]
                with self.subTest(name=name), self.assertRaisesRegex(Invalid, "required"):
                    inspect(*image_bytes(config, report, payloads))
        def test_required_notice_sources_are_not_self_asserted(self):
            for name in [*ARCHIVE_NOTICES, *INSTALLED_NOTICES]:
                for source in (None, "/build/unrelated", SYSROOT + "/share/doc/rust/../COPYRIGHT"):
                    config, report, payloads = fixture()
                    report["licenses"]["files"]["rust/" + name]["source"] = source
                    with self.subTest(name=name, source=source), self.assertRaisesRegex(Invalid, "required"):
                        inspect(*image_bytes(config, report, payloads))
        def test_archive_notice_mutation_cannot_be_report_rehashed(self):
            for name in ARCHIVE_NOTICES:
                for change_size in (False, True):
                    config, report, payloads = fixture()
                    key = "licenses/rust/" + name
                    old = payloads[key]
                    value = old + b"x" if change_size else b"x" + old[1:]
                    payloads[key] = value
                    report["licenses"]["files"]["rust/" + name].update(
                        size=len(value), sha256=hashlib.sha256(value).hexdigest())
                    report["licenses"]["total_bytes"] += len(value) - len(old)
                    with self.subTest(name=name, change_size=change_size), self.assertRaisesRegex(Invalid, "required"):
                        inspect(*image_bytes(config, report, payloads))
        def test_required_notice_mode_and_actual_bytes_are_bound(self):
            for name in [*ARCHIVE_NOTICES, *INSTALLED_NOTICES]:
                config, report, payloads = fixture()
                report["licenses"]["files"]["rust/" + name]["mode"] = 0o555
                with self.assertRaises(Invalid):
                    inspect(*image_bytes(config, report, payloads))
                config, report, payloads = fixture()
                payloads["licenses/rust/" + name] += b"x"
                with self.assertRaises(Invalid):
                    inspect(*image_bytes(config, report, payloads))


        def test_only_fixed_generated_notices_have_large_file_budget(self):
            for name in ("COPYRIGHT.html", "COPYRIGHT-library.html"):
                source = SYSROOT + "/share/doc/rust/" + name
                self.assertEqual(notice_limit(source, "rust/" + name), 16 * 1024 * 1024)
                for wrong_source, destination in (
                    (source + ".extra", "rust/" + name),
                    ("/build/" + name, "rust/" + name),
                    (source, "rust/licenses/" + name)):
                    self.assertEqual(notice_limit(wrong_source, destination), 1024 * 1024)
                config, report, payloads = fixture()
                key = "rust/" + name
                value = b"x" * (1024 * 1024 + 1)
                old = report["licenses"]["files"][key]["size"]
                payloads["licenses/" + key] = value
                report["licenses"]["files"][key].update(
                    size=len(value), sha256=hashlib.sha256(value).hexdigest())
                report["licenses"]["total_bytes"] += len(value) - old
                self.assertEqual(inspect(*image_bytes(config, report, payloads))["state"],
                                 "inspected-not-activated")
                value = b"x" * (16 * 1024 * 1024 + 1)
                old = report["licenses"]["files"][key]["size"]
                payloads["licenses/" + key] = value
                report["licenses"]["files"][key].update(
                    size=len(value), sha256=hashlib.sha256(value).hexdigest())
                report["licenses"]["total_bytes"] += len(value) - old
                with self.assertRaisesRegex(Invalid, "notice declaration"):
                    inspect(*image_bytes(config, report, payloads))
            config, report, payloads = fixture()
            key = "rust/licenses/ordinary.txt"
            value = b"x" * (1024 * 1024 + 1)
            payloads["licenses/" + key] = value
            report["licenses"]["files"][key] = {"source": SYSROOT + "/share/doc/rust/licenses/ordinary.txt",
                "size": len(value), "sha256": hashlib.sha256(value).hexdigest(), "mode": 0o444}
            report["licenses"]["total_bytes"] += len(value)
            with self.assertRaisesRegex(Invalid, "notice declaration"):
                inspect(*image_bytes(config, report, payloads))
            self.assertEqual(NOTICE_TOTAL_LIMIT, 48 * 1024 * 1024)


        def test_notice_aggregate_budget_from_inspected_metadata(self):
            # Pure metadata gate test; full-image tests separately bind actual bytes.
            for count, accepted in ((47, True), (48, False)):
                config, report, payloads = fixture()
                inspected = inspect(*image_bytes(config, report, payloads))
                inventory = copy.deepcopy(inspected["files"])
                directories = copy.deepcopy(inspected["directories"])
                directories["licenses/extra"] = (0o555, 0, 0, EPOCH)
                for index in range(count):
                    key = "extra/notice-" + str(index)
                    entry = {"source": "/usr/share/doc/fixture/copyright",
                             "size": 1024 * 1024, "sha256": "1" * 64, "mode": 0o444}
                    report["licenses"]["files"][key] = entry
                    report["licenses"]["total_bytes"] += entry["size"]
                    inventory["licenses/" + key] = {
                        **{k:entry[k] for k in ("size", "sha256", "mode")},
                        "uid": 0, "gid": 0, "mtime": EPOCH, "elf": None}
                if accepted:
                    self.assertEqual(validate(config, report, inventory, directories)["state"],
                                     "inspected-not-activated")
                else:
                    with self.assertRaisesRegex(Invalid, "notice byte total"):
                        validate(config, report, inventory, directories)

        def test_exact_empty_elf_search_directory_is_readonly(self):
            config, report, payloads = fixture()
            raw = bytearray(512); raw[:7] = b"\x7fELF\x02\x01\x01"
            struct.pack_into("<HHI", raw, 16, 3, 62, 1)
            struct.pack_into("<Q", raw, 32, 64)
            struct.pack_into("<HHH", raw, 52, 64, 56, 3)
            struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0x400000, 0, 512, 512, 4096)
            struct.pack_into("<IIQQQQQQ", raw, 120, 2, 4, 256, 0x400100, 0, 64, 64, 8)
            struct.pack_into("<IIQQQQQQ", raw, 176, 0x6474e551, 6, 0, 0, 0, 0, 0, 16)
            strings = b"\0$ORIGIN/../lib\0"
            for index, (kind, value) in enumerate(((29, 1), (5, 0x400180), (10, len(strings)), (0, 0))):
                struct.pack_into("<QQ", raw, 256 + index * 16, kind, value)
            raw[384:384 + len(strings)] = strings
            path = SYSROOT + "/lib/rustlib/x86_64-unknown-linux-gnu/lib"
            payloads[LLD[1:]] = bytes(raw)
            report["total_bytes"] += len(raw) - report["files"][LLD]["size"]
            report["files"][LLD].update(size=len(raw), sha256=hashlib.sha256(raw).hexdigest())
            report["graph"][LLD]["search_paths"] = [path]
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            entry = (path[1:], b"", 0o555, 0, 0, tarfile.DIRTYPE)
            self.assertEqual(inspect(*image_bytes(config, report, payloads, extra=[entry]))["state"],
                             "inspected-not-activated")
            for mode, uid in ((0o777, 0), (0o555, 65532)):
                bad = (path[1:], b"", mode, uid, 0, tarfile.DIRTYPE)
                with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads, extra=[bad]))
            report["graph"][LLD]["search_paths"] = [path + "/unrelated"]
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads, extra=[entry]))
        def test_config_and_report_authority(self):
            for key, value in (("User", "0"), ("WorkingDir", "/build"),
                               ("Env", ["PATH=/usr/bin"]), ("Entrypoint", ["/bin/sh"]),
                               ("Cmd", ["--version"]), ("Volumes", {"/build": {}}),
                               ("OnBuild", ["RUN true"]), ("Healthcheck", {"Test": ["CMD", "x"]})):
                config, report, payloads = fixture(); config["config"][key] = value
                with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            for key, value in (("state", "accepted"), ("target_execution", True),
                               ("accepted_compiler_image", True), ("total_bytes", True)):
                config, report, payloads = fixture(); report[key] = value
                with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            with self.assertRaises(Invalid): validate([], {}, {}, {})
        def test_positive_inventory_is_exact(self):
            for name, kind in (("bin/sh", tarfile.REGTYPE), ("extra", tarfile.DIRTYPE),
                               ("lib/alias", tarfile.SYMTYPE), ("lib/hard", tarfile.LNKTYPE),
                               ("../escape", tarfile.REGTYPE), ("/absolute", tarfile.REGTYPE),
                               (RUSTC[1:], tarfile.REGTYPE)):
                extra = [(name, b"", 0o555, 0, 0, kind)]
                with self.assertRaises(Invalid): inspect(*image_bytes(*fixture(), extra=extra))
        def test_metadata_and_content_bound(self):
            for position, value in ((2, 0o777), (3, 1), (4, 1)):
                def mutate(entries):
                    result = []
                    for entry in entries:
                        if entry[0] == RUSTC[1:]:
                            entry = list(entry); entry[position] = value; entry = tuple(entry)
                        result.append(entry)
                    return result
                with self.assertRaises(Invalid): inspect(*image_bytes(*fixture(), mutate=mutate))
            config, report, payloads = fixture(); payloads[RUSTC[1:]] += b"x"
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            config, report, payloads = fixture(); report["files"][RUSTC]["source"] = "/build/rustc"
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
        def test_graph_and_notices_must_match(self):
            for field, value in (("needed", ["libc.so.6"]), ("interpreter", "/lib64/ld.so"),
                                 ("resolved", ["/lib/missing.so"]), ("search_paths", ["/build"])):
                config, report, payloads = fixture(); report["graph"][RUSTC][field] = value
                with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            config, report, payloads = fixture()
            report["licenses"]["files"]["rust/LICENSE-MIT"]["sha256"] = "0" * 64
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            config, report, payloads = fixture()
            report["licenses"]["files"]["../../escape"] = report["licenses"]["files"].pop("rust/LICENSE-MIT")
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))

        def test_dynamic_filename_alias_and_package_coverage(self):
            result = inspect(*image_bytes(*dynamic_fixture()))
            self.assertEqual(result["state"], "inspected-not-activated")
            for path in ("/lib/x86_64-linux-gnu/libfixture-renamed.so",
                         SYSROOT + "/lib/hidden/libfixture.so"):
                with self.assertRaises(Invalid): inspect(*image_bytes(*dynamic_fixture(path)))
            config, report, payloads = dynamic_fixture()
            report["licenses"]["runtime_packages"] = {}
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            config, report, payloads = dynamic_fixture()
            report["licenses"]["runtime_packages"]["fixture:amd64"]["files"].append("/lib/unrelated.so")
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
        def test_different_loader_shadow_bytes_fail(self):
            config, report, payloads = dynamic_fixture()
            path = SYSROOT + "/lib/libfixture.so"
            value = bytearray(payloads["lib/x86_64-linux-gnu/libfixture.so"]); value[-1] = 1
            payloads[path[1:]] = bytes(value)
            report["files"][path] = {"source": path, "size": len(value), "mode": 0o555,
                                    "sha256": hashlib.sha256(value).hexdigest()}
            report["graph"][path] = {"needed": [], "interpreter": None, "resolved": [], "search_paths": []}
            report["total_bytes"] += len(value)
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))

        def test_backend_selection_matches_actual_payload(self):
            config, report, payloads = fixture()
            directory = SYSROOT + "/lib/rustlib/x86_64-unknown-linux-gnu/codegen-backends/"
            def add(name):
                path = directory + name; value = elf_fixture()
                payloads[path[1:]] = value
                report["files"][path] = {"source": path, "size": len(value), "mode": 0o555,
                                        "sha256": hashlib.sha256(value).hexdigest()}
                report["graph"][path] = {"needed": [], "interpreter": None, "resolved": [], "search_paths": []}
                report["total_bytes"] += len(value)
                return path
            selected = add("librustc_codegen_llvm.so")
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            report["codegen_backend"] = selected
            self.assertEqual(inspect(*image_bytes(config, report, payloads))["state"], "inspected-not-activated")
            add("librustc_codegen_llvm-1.95.0.so")
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
        def test_exact_omission_evidence_required(self):
            for omitted in ([], [dict(OMITTED_MUSL, size=1)],
                            [dict(OMITTED_MUSL), dict(OMITTED_MUSL)]):
                config, report, payloads = fixture()
                report["omitted_musl_dynamic"] = omitted
                with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
        def test_disconnected_elf_and_forged_trace_are_rejected(self):
            for path in (OMITTED_MUSL["source"], MUSL + "libunrelated.so"):
                for forged_edge in (False, True):
                    config, report, payloads = fixture()
                    value = elf_fixture()
                    payloads[path[1:]] = value
                    report["files"][path] = {"source": path, "size": len(value),
                        "mode": 0o555, "sha256": hashlib.sha256(value).hexdigest()}
                    report["graph"][path] = {"needed": [], "interpreter": None,
                                             "resolved": [], "search_paths": []}
                    report["total_bytes"] += len(value)
                    if forged_edge:
                        report["graph"][RUSTC]["resolved"].append(path)
                    with self.assertRaises(Invalid):
                        inspect(*image_bytes(config, report, payloads))
        def test_provenance_required(self):
            for field in ("backend_probe", "codegen_backend"):
                config, report, payloads = fixture(); del report[field]
                with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            config, report, payloads = fixture(); del report["licenses"]["runtime_packages"]
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
            config, report, payloads = fixture(); report["backend_probe"]["llvm_version"] = "unknown"
            with self.assertRaises(Invalid): inspect(*image_bytes(config, report, payloads))
        def test_image_identity_and_json(self):
            raw, identity = image_bytes(*fixture())
            with self.assertRaises(Invalid): inspect(raw, "sha256:" + "0" * 64)
            with self.assertRaises(Invalid): inspect(*image_bytes(*fixture(), duplicate_json=True))
            for value in (b"", b"not tar", raw[:1024]):
                with self.assertRaises(Invalid): inspect(value, identity)
            with self.assertRaises(Invalid): unique_json(b'{"a":1,"a":2}')
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test only")
    self_test()
