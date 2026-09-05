"""Cloud construction helper: positively export a pinned Rust compiler closure.
Not a compiler runtime launcher, OS entrypoint, or accepted image identity.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import time

SYSROOT = Path("/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu")
RUSTC = SYSROOT / "bin/rustc"
LLD = SYSROOT / "lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"
MUSL = SYSROOT / "lib/rustlib/x86_64-unknown-linux-musl/lib"
EPOCH = 1785715200

class Invalid(RuntimeError):
    pass

def run(argv):
    env = {"PATH": "/usr/bin:/bin", "LC_ALL": "C", "LANG": "C",
           "LD_LIBRARY_PATH": str(SYSROOT / "lib")}
    child = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT, env=env, close_fds=True)
    output = bytearray()
    deadline = time.monotonic() + 10
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(child.stdout, selectors.EVENT_READ)
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise Invalid("inspection deadline")
                for key, _ in selector.select(min(0.1, remaining)):
                    part = os.read(key.fd, 16384)
                    if not part:
                        selector.unregister(key.fileobj)
                    else:
                        output.extend(part)
                        if len(output) > 1048576:
                            raise Invalid("inspection output")
        code = child.wait(timeout=max(0.01, deadline - time.monotonic()))
        if code != 0:
            raise Invalid("inspection command failed")
        return bytes(output).decode("ascii")
    finally:
        if child.poll() is None:
            child.kill()
        child.wait(timeout=2)
        child.stdout.close()

def safe_path(value):
    if type(value) is not str or not value.startswith("/") or len(value) > 512:
        raise Invalid("closure path")
    parts = value.split("/")[1:]
    if len(parts) > 32:
        raise Invalid("closure path depth")
    if any(not x or x in (".", "..") or re.fullmatch("[A-Za-z0-9_.+-]+", x) is None for x in parts):
        raise Invalid("closure path component")
    low = value.lower()
    if any(x in low for x in ("libcrypto", "libssl", "libsodium", "openssl", "reference-")):
        raise Invalid("reference material in compiler closure")
    path = Path(value)
    if not (value.startswith(str(SYSROOT) + "/") or
            value.startswith(("/lib/", "/lib64/", "/usr/lib/", "/usr/lib64/"))):
        raise Invalid("closure path domain")
    return path

def trace_paths(text):
    paths = set()
    resolved = {}
    for line in text.splitlines():
        words = line.split()
        if not words:
            continue
        if len(words) == 2 and words[0] == "linux-vdso.so.1" and re.fullmatch(r"\(0x[0-9a-f]+\)", words[1]):
            continue
        if (len(words) == 4 and words[1] == "=>" and
            re.fullmatch("[A-Za-z0-9_.+-]+", words[0]) and
            re.fullmatch(r"\(0x[0-9a-f]+\)", words[3])):
            path = safe_path(os.path.normpath(words[2]))
            if words[0] in resolved and resolved[words[0]] != path:
                raise Invalid("ambiguous dependency")
            resolved[words[0]] = path
            paths.add(path)
        elif len(words) == 2 and words[0].startswith("/") and re.fullmatch(r"\(0x[0-9a-f]+\)", words[1]):
            path = safe_path(os.path.normpath(words[0]))
            if path.name in resolved and resolved[path.name] != path:
                raise Invalid("ambiguous interpreter dependency")
            resolved[path.name] = path
            paths.add(path)
        else:
            raise Invalid("unrecognized or unresolved loader trace")
    return paths, resolved

def dynamic_metadata(program, dynamic):
    interpreters = []
    for line in program.splitlines():
        if "Requesting program interpreter" not in line:
            continue
        match = re.fullmatch(r"\[Requesting program interpreter: ([^\]]+)\]", line.strip())
        if match is None:
            raise Invalid("malformed interpreter metadata")
        interpreters.append(match.group(1))
    if len(interpreters) > 1:
        raise Invalid("multiple interpreters")
    if any(line.split()[:1] == ["INTERP"] for line in program.splitlines()) and not interpreters:
        raise Invalid("missing interpreter metadata")
    interpreter = safe_path(interpreters[0]) if interpreters else None
    needed = []
    for line in dynamic.splitlines():
        if "FILTER" in line or "AUXILIARY" in line:
            raise Invalid("unsupported dynamic dependency tag")
        if "NEEDED" not in line:
            continue
        match = re.fullmatch(r"0x[0-9a-fA-F]+[ \t]+\(NEEDED\)[ \t]+Shared library: \[([^\]]+)\]", line.strip())
        if match is None:
            raise Invalid("malformed dynamic dependency metadata")
        needed.append(match.group(1))
    if len(needed) > 128 or len(set(needed)) != len(needed):
        raise Invalid("dependency count/duplicates")
    for name in needed:
        if re.fullmatch("[A-Za-z0-9_.+-]+", name) is None or any(
            x in name.lower() for x in ("crypto", "ssl", "sodium")):
            raise Invalid("unexpected dynamic dependency")
    return interpreter, needed

def select_backend(entries):
    if type(entries) is not list or len(entries) != 1:
        raise Invalid("exactly one LLVM backend required")
    name, regular, symlink = entries[0]
    if (type(name) is not str or len(name) > 128 or
        re.fullmatch(r"librustc_codegen_llvm(?:-[0-9a-f]+)?\.so", name) is None or
        regular is not True or symlink is not False):
        raise Invalid("LLVM backend type/name")
    return name

def main():
    if (os.uname().sysname != "Linux" or os.uname().machine != "x86_64" or
        os.geteuid() != 0 or Path(__file__).resolve() != Path("/build/compiler_closure.py") or
        os.environ.get("RAR_COMPILER_PROVISION") != "pinned-rust-1.95.0"):
        raise Invalid("private cloud construction stage only")
    if not run([str(RUSTC), "--version"]).startswith("rustc 1.95.0 ("):
        raise Invalid("compiler version")
    root = Path("/compiler-root")
    root.mkdir(mode=0o700, exist_ok=False)
    files = {}
    graph = {}
    total = 0
    def export(path, executable):
        nonlocal total
        path = safe_path(str(path))
        actual = safe_path(str(path.resolve(strict=True)))
        if not actual.is_file():
            raise Invalid("closure file type")
        size = actual.stat().st_size
        if not 1 <= size <= 268435456:
            raise Invalid("closure file size")
        if str(path) in files:
            return
        total += size
        if total > 1610612736 or len(files) >= 4096:
            raise Invalid("compiler closure budget")
        destination = root / str(path).lstrip("/")
        destination.parent.mkdir(parents=True, exist_ok=True)
        digest = hashlib.sha256()
        copied = 0
        with actual.open("rb") as source, destination.open("xb") as output:
            while True:
                part = source.read(65536)
                if not part:
                    break
                copied += len(part)
                if copied > size:
                    raise Invalid("construction source changed")
                digest.update(part)
                output.write(part)
        if copied != size:
            raise Invalid("construction source truncated")
        destination.chmod(0o555 if executable else 0o444)
        os.utime(destination, (EPOCH, EPOCH))
        files[str(path)] = {"source": str(actual), "sha256": digest.hexdigest(),
                            "size": size, "mode": 0o555 if executable else 0o444}
    backend_root = SYSROOT / "lib/rustlib/x86_64-unknown-linux-gnu/codegen-backends"
    if backend_root.is_symlink() or not backend_root.is_dir():
        raise Invalid("LLVM backend directory")
    entries = []
    for path in backend_root.iterdir():
        entries.append((path.name, path.is_file(), path.is_symlink()))
        if len(entries) > 1:
            raise Invalid("ambiguous codegen backend directory")
    backend = backend_root / select_backend(entries)
    pending = [RUSTC, LLD, backend]
    seen = set()
    while pending:
        path = safe_path(str(pending.pop()))
        if path in seen:
            continue
        if len(seen) >= 128:
            raise Invalid("runtime dependency closure")
        seen.add(path)
        actual = safe_path(str(path.resolve(strict=True)))
        with actual.open("rb") as stream:
            header = stream.read(20)
        if len(header) != 20 or header[:7] != bytes((127,69,76,70,2,1,1)) or header[18:20] != bytes((62,0)):
            raise Invalid("compiler runtime ELF platform")
        program = run(["/usr/bin/readelf", "-lW", str(actual)])
        dynamic = run(["/usr/bin/readelf", "-dW", str(actual)])
        interpreter, needed = dynamic_metadata(program, dynamic)
        dependencies = set()
        if needed or interpreter:
            trace, resolved = trace_paths(run(["/usr/bin/ldd", str(actual)]))
            if any(name not in resolved for name in needed):
                raise Invalid("unresolved DT_NEEDED")
            dependencies.update(trace)
        if interpreter is not None:
            dependencies.add(interpreter)
        export(path, True)
        # Preserve canonical source locations too, without exporting symlinks.
        export(actual, True)
        graph[str(path)] = {"needed": needed, "interpreter": str(interpreter) if interpreter else None,
                            "resolved": sorted(str(x) for x in dependencies)}
        pending.extend(dependencies - seen)
    if not MUSL.is_dir() or MUSL.is_symlink():
        raise Invalid("musl sysroot missing")
    libraries = []
    for path in MUSL.rglob("*"):
        libraries.append(path)
        if len(libraries) > 4096:
            raise Invalid("musl sysroot count")
    if not libraries:
        raise Invalid("musl sysroot count")
    for path in sorted(libraries):
        if path.is_symlink():
            raise Invalid("unexpected musl symlink")
        if path.is_dir():
            continue
        if path.suffix not in (".rlib", ".rmeta", ".a", ".o"):
            raise Invalid("unexpected musl sysroot file")
        export(path, False)
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_dir():
            path.chmod(0o555)
            os.utime(path, (EPOCH, EPOCH))
    root.chmod(0o555)
    os.utime(root, (EPOCH, EPOCH))
    report = {"state": "private-closure-export-only", "files": files, "graph": graph,
              "total_bytes": total, "codegen_backend": str(backend),
              "licenses": "must be completed and reviewed before image publication",
              "target_execution": False, "accepted_compiler_image": False}
    Path("/build/compiler-closure.json").write_text(json.dumps(report, sort_keys=True, indent=2) + chr(10))

def self_test():
    import unittest
    from unittest.mock import patch
    class Tests(unittest.TestCase):
        def test_forbidden_domains_and_reference_material(self):
            for path in ("/etc/passwd", "/bin/sh", "/usr/bin/openssl",
                         "/usr/lib/libcrypto.so.3", "/usr/lib/libsodium.so",
                         "/lib/libssl.so.3", "/usr/lib/../etc/passwd",
                         "/usr/lib//double", "/usr/lib/sp ace"):
                with self.assertRaises(Invalid): safe_path(path)
            self.assertEqual(safe_path("/lib64/ld-linux-x86-64.so.2"),
                             Path("/lib64/ld-linux-x86-64.so.2"))
        def test_trace_resolution_and_unknown_lines(self):
            raw = ("linux-vdso.so.1 (0x1234)" + chr(10) +
                   "libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x2345)" + chr(10) +
                   "/lib64/ld-linux-x86-64.so.2 (0x3456)" + chr(10))
            paths, resolved = trace_paths(raw)
            self.assertEqual(len(paths), 2)
            self.assertEqual(str(resolved["libc.so.6"]), "/lib/x86_64-linux-gnu/libc.so.6")
            for raw in ("libc.so.6 => not found", "arbitrary output",
                        "libcrypto.so.3 => /usr/lib/libcrypto.so.3 (0x1234)",
                        "a.so => /usr/lib/a.so (0x1)" + chr(10) + "a.so => /lib/a.so (0x2)"):
                with self.assertRaises(Invalid): trace_paths(raw)
            paths, _ = trace_paths("a.so => /usr/lib/sub/../a.so (0x1)")
            self.assertEqual(paths, {Path("/usr/lib/a.so")})
        def test_dynamic_metadata(self):
            program = "[Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]"
            dynamic = "0x1 (NEEDED) Shared library: [libc.so.6]"
            interpreter, needed = dynamic_metadata(program, dynamic)
            self.assertEqual(str(interpreter), "/lib64/ld-linux-x86-64.so.2")
            self.assertEqual(needed, ["libc.so.6"])
            for bad in (dynamic + chr(10) + dynamic,
                        "0x1 (NEEDED) Shared library: [libcrypto.so.3]",
                        "0x1 (NEEDED) Shared library: [../lib.so]"):
                with self.assertRaises(Invalid): dynamic_metadata(program, bad)
            with self.assertRaises(Invalid): dynamic_metadata(program + program, dynamic)
        def test_codegen_backend_selection(self):
            good = ("librustc_codegen_llvm-abcdef123456.so", True, False)
            self.assertEqual(select_backend([good]), good[0])
            for values in ([], [good, good], [(good[0], True, True)],
                           [(good[0], False, False)], [("other.so", True, False)],
                           [("librustc_codegen_llvm-abc.so.extra", True, False)]):
                with self.assertRaises(Invalid): select_backend(values)
        def test_malformed_metadata_is_not_empty_closure(self):
            for dynamic in ("NEEDED unknown", "0x1 (NEEDED) truncated",
                            "0x1 (FILTER) Shared library: [a.so]",
                            "0x1 (AUXILIARY) Shared library: [a.so]"):
                with self.assertRaises(Invalid): dynamic_metadata("", dynamic)
            for program in ("prefix [Requesting program interpreter: /lib/ld.so]",
                            "[Requesting program interpreter: /lib/ld.so",
                            "INTERP 0x0 0x0"):
                with self.assertRaises(Invalid): dynamic_metadata(program, "")
        def test_default_guard_never_inspects_or_exports(self):
            with patch.dict(os.environ, {}, clear=True), patch(__name__ + ".run") as runner:
                with self.assertRaises(Invalid): main()
                runner.assert_not_called()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():
        raise SystemExit(1)

if __name__ == "__main__":
    import sys
    if sys.argv[1:] == ["--self-test"]:
        if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("CI") != "true" or sys.platform != "linux":
            raise SystemExit("cloud self-test only")
        self_test()
    elif sys.argv[1:]:
        raise SystemExit("no construction arguments")
    else:
        main()
