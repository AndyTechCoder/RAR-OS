"""Trusted-main cloud candidate provisioning only; no target source or OS boot.
Two no-cache provisions retain image bytes and inventories. Passing this script
does NOT activate the compiler role or establish RAR algorithm comparisons.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import resource
import ssl
import sys
import time
import urllib.parse
import urllib.request

BASE = "rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"
EPOCH = 1785715200
MAX_IMAGE_ARCHIVE = 1024 * 1024 * 1024
SOURCES = {
    "rust-musl.tar.xz": (
        "https://static.rust-lang.org/dist/rust-std-1.95.0-x86_64-unknown-linux-musl.tar.xz",
        "aee540abf132920f791ef781489851a078d69dff493fb628d49c1d573f92bb3a"),
}
CONTEXT = ("compiler_closure.py", "compiler.Containerfile")
HOSTS = frozenset(("static.rust-lang.org",))

class Invalid(RuntimeError):
    pass

def url_allowed(url):
    try:
        value = urllib.parse.urlsplit(url)
        port = value.port
    except (TypeError, ValueError) as exc:
        raise Invalid("acquisition URL") from exc
    if (value.scheme != "https" or value.hostname not in HOSTS or
        value.username is not None or value.password is not None or
        port not in (None, 443) or value.fragment or value.query or
        value.path != "/dist/rust-std-1.95.0-x86_64-unknown-linux-musl.tar.xz"):
        raise Invalid("acquisition URL")
    return url

class Redirect(urllib.request.HTTPRedirectHandler):
    max_redirections = 4
    max_repeats = 1
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return super().redirect_request(req, fp, code, msg, headers, url_allowed(newurl))

def acquire(name):
    if name not in SOURCES:
        raise Invalid("source identity")
    url, expected = SOURCES[name]
    opener = urllib.request.build_opener(
        urllib.request.ProxyHandler({}), Redirect(),
        urllib.request.HTTPSHandler(context=ssl.create_default_context()))
    request = urllib.request.Request(url_allowed(url), headers={"User-Agent": "RAR-Modern-Compiler-Provision-v0"})
    result = bytearray()
    deadline = time.monotonic() + 90
    with opener.open(request, timeout=10) as response:
        url_allowed(response.geturl())
        if response.status != 200:
            raise Invalid("acquisition HTTP status")
        while True:
            if time.monotonic() >= deadline:
                raise Invalid("acquisition deadline")
            part = response.read(32768)
            if not part:
                break
            result.extend(part)
            if len(result) > 64 * 1024 * 1024:
                raise Invalid("acquisition size")
    raw = bytes(result)
    if hashlib.sha256(raw).hexdigest() != expected:
        raise Invalid("source archive digest")
    return raw

def load_module(path, name):
    if path.is_symlink() or not path.is_file():
        raise Invalid("controller module path")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def guard():
    required = {"GITHUB_ACTIONS": "true", "CI": "true",
                "GITHUB_REPOSITORY": "AndyTechCoder/RAR-OS",
                "GITHUB_EVENT_NAME": "workflow_dispatch", "GITHUB_REF": "refs/heads/main",
                "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64", "ImageOS": "ubuntu24"}
    if (os.uname().sysname != "Linux" or os.uname().machine != "x86_64" or
        any(os.environ.get(k) != v for k, v in required.items())):
        raise Invalid("trusted-main disposable cloud job only")
    sha = os.environ.get("GITHUB_SHA", "")
    run_id = os.environ.get("GITHUB_RUN_ID", "")
    attempt = os.environ.get("GITHUB_RUN_ATTEMPT", "")
    if (re.fullmatch("[0-9a-f]{40}", sha) is None or
        re.fullmatch("[0-9]+", run_id) is None or re.fullmatch("[0-9]+", attempt) is None):
        raise Invalid("workflow identity")
    if re.fullmatch("[A-Za-z0-9._-]{1,128}", os.environ.get("ImageVersion", "")) is None:
        raise Invalid("hosted runner image identity")
    workspace = Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)
    source = workspace / "controller"
    if Path(__file__).resolve() != source / "tools/rar-lab/modern/provision_compiler.py":
        raise Invalid("controller checkout location")
    return workspace, source, sha, run_id, attempt

def file_identity(path, maximum):
    if path.is_symlink() or not path.is_file() or path.stat().st_size > maximum:
        raise Invalid("evidence/tool file")
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        while True:
            part = stream.read(65536)
            if not part:
                break
            size += len(part)
            if size > maximum:
                raise Invalid("evidence/tool growth")
            digest.update(part)
    return {"sha256": digest.hexdigest(), "size": size}

def child_environment(workspace):
    if not workspace.is_absolute() or ".." in workspace.parts:
        raise Invalid("cloud workspace path")
    return {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
            "DOCKER_CONFIG": str(workspace / "modern-compiler-docker-config"),
            "DOCKER_BUILDKIT": "1", "BUILDX_BUILDER": "default",
            "SOURCE_DATE_EPOCH": str(EPOCH), "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "/nonexistent", "GIT_ATTR_NOSYSTEM": "1"}

def main():
    workspace, source, sha, run_id, attempt = guard()
    runner_image = os.environ["ImageVersion"]
    space = os.statvfs(workspace)
    if space.f_bavail * space.f_frsize < 16 * 1024 * 1024 * 1024:
        raise Invalid("cloud workspace reserve")
    # Child tools receive no user Docker/Git/proxy/plugin/TLS configuration.
    # This changes only this cloud process, never workflow-wide settings.
    environment = child_environment(workspace)
    config = Path(environment["DOCKER_CONFIG"])
    config.mkdir(mode=0o700, exist_ok=False)
    if config.is_symlink() or config.stat().st_mode & 0o777 != 0o700 or any(config.iterdir()):
        raise Invalid("private empty Docker client state")
    os.environ.clear()
    os.environ.update(environment)
    # All dynamic code is the same trusted-main checkout, never a PR checkout.
    foundation = load_module(source / "tools/rar-lab/foundation/controller.py", "foundation")
    run = foundation.run
    _, actual = run(["/usr/bin/git", "-C", str(source), "rev-parse", "HEAD"], 10, 128)
    _, dirty = run(["/usr/bin/git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=/nonexistent",
                    "-C", str(source), "status", "--porcelain",
                    "--untracked-files=all"], 10, 65536)
    if actual.strip().decode("ascii") != sha or dirty:
        raise Invalid("checkout identity/cleanliness")
    inventory = load_module(source / "tools/rar-lab/modern/compiler_inventory.py", "inventory")
    archive_gate = load_module(source / "tools/rar-lab/modern/compiler_archive.py", "archive_gate")
    if SOURCES["rust-musl.tar.xz"] != (archive_gate.URL, archive_gate.SHA256):
        raise Invalid("archive gate identity differs")
    evidence = workspace / "modern-compiler-evidence"
    evidence.mkdir(mode=0o700, exist_ok=False)
    summary = {"source": sha, "run": run_id, "attempt": attempt, "status": "started",
               "target_execution": False, "compiler_activation": False, "adapter_execution": False,
               "runner_image": runner_image,
               "construction_base": BASE, "source_date_epoch": EPOCH, "builds": []}
    def save():
        summary["controller_maxrss_kib"] = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        summary["evidence_files"] = {path.name: file_identity(path, MAX_IMAGE_ARCHIVE)
                                     for path in sorted(evidence.iterdir()) if path.name != "manifest.json"}
        (evidence / "manifest.json").write_text(json.dumps(summary, sort_keys=True, indent=2) + chr(10))
    summary["controller_tools"] = {
        str(path.resolve(strict=True)): file_identity(path.resolve(strict=True), 128 * 1024 * 1024)
        for path in (Path(sys.executable), Path("/usr/bin/git"), Path("/usr/bin/docker"))}
    save()
    docker = ["/usr/bin/docker", "--host=unix:///var/run/docker.sock"]
    try:
        # Network is used only for fixed, hash-pinned acquisition. No source
        # checkout/credentials are mounted into construction or final images.
        summary["stage"] = "docker-identity"
        save()
        _, version = run(docker + ["version", "--format", "{{json .}}"], 15, 65536)
        (evidence / "docker-version.json").write_bytes(version)
        _, info = run(docker + ["info", "--format",
                    "{{.ServerVersion}}|{{.Architecture}}|{{.OSType}}|{{.NCPU}}|{{.MemTotal}}|{{json .SecurityOptions}}"], 15, 65536)
        (evidence / "docker-platform.txt").write_bytes(info)
        summary["stage"] = "base-acquisition"
        save()
        run(docker + ["pull", "--platform=linux/amd64", BASE], 180, 1048576,
            log_path=evidence / "base-acquisition.log")
        summary["stage"] = "base-and-builder-identity"
        save()
        _, pulled = run(docker + ["image", "inspect", "--format",
            '{"Id":{{json .Id}},"Architecture":{{json .Architecture}},"Os":{{json .Os}},"RepoDigests":{{json .RepoDigests}}}', BASE], 15, 65536)
        base_identity = json.loads(pulled)
        if (base_identity.get("Architecture") != "amd64" or base_identity.get("Os") != "linux" or
            re.fullmatch("sha256:[0-9a-f]{64}", base_identity.get("Id", "")) is None or
            type(base_identity.get("RepoDigests")) is not list or
            not any(type(x) is str and x.endswith("@" + BASE.split("@")[1]) for x in base_identity["RepoDigests"])):
            raise Invalid("pulled base identity/platform")
        summary["pulled_base"] = base_identity
        _, builder = run(docker + ["buildx", "inspect", "default"], 15, 65536)
        if [b"Driver:", b"docker"] not in [line.split() for line in builder.splitlines()] or b"BuildKit version:" not in builder:
            raise Invalid("expected default Docker BuildKit")
        (evidence / "buildkit-identity.txt").write_bytes(builder)
        _, buildx = run(docker + ["buildx", "version"], 15, 65536)
        (evidence / "buildx-version.txt").write_bytes(buildx)
        archives = {}
        for name in SOURCES:
            summary["stage"] = "acquire-" + name
            save()
            raw = acquire(name)
            summary["stage"] = "inventory-" + name
            save()
            index = archive_gate.verify(raw)
            archives[name] = raw
            (evidence / name).write_bytes(raw)
            (evidence / (name + ".inventory.json")).write_text(json.dumps(index, sort_keys=True) + chr(10))
        source_hashes = {}
        context_bytes = {}
        for name in CONTEXT:
            path = source / "tools/rar-lab/modern" / name
            if path.is_symlink() or not path.is_file() or path.stat().st_size > 131072:
                raise Invalid("construction source path/size")
            raw = path.read_bytes()
            context_bytes[name] = raw
            source_hashes[name] = hashlib.sha256(raw).hexdigest()
        summary["construction_sources"] = source_hashes
        for number in (1, 2):
            context = workspace / ("modern-compiler-context-" + str(number))
            context.mkdir(mode=0o700, exist_ok=False)
            for name, raw in {**context_bytes, **archives}.items():
                path = context / name
                with path.open("xb") as stream:
                    stream.write(raw)
                path.chmod(0o644)
                os.utime(path, (EPOCH, EPOCH))
            os.utime(context, (EPOCH, EPOCH))
            tag = "rar-modern-compiler-" + run_id + "-" + attempt + "-" + str(number)
            command = docker + ["build", "--platform=linux/amd64", "--network=none",
                                "--no-cache", "--pull=false", "--progress=plain",
                                "--build-arg", "SOURCE_DATE_EPOCH=" + str(EPOCH),
                                "--tag", tag, "--file", str(context / "compiler.Containerfile"),
                                str(context)]
            summary["stage"] = "build-" + str(number)
            save()
            run(command, 900, 8 * 1024 * 1024, log_path=evidence / ("build-" + str(number) + ".log"))
            _, identity = run(docker + ["image", "inspect", "--format",
                '{"Id":{{json .Id}},"Size":{{json .Size}}}', tag], 15, 1024)
            constructed = json.loads(identity)
            image = constructed.get("Id")
            if (type(image) is not str or re.fullmatch("sha256:[0-9a-f]{64}", image) is None or
                type(constructed.get("Size")) is not int or
                not 1 <= constructed["Size"] <= 768 * 1024 * 1024):
                raise Invalid("constructed image identity/size")
            _, raw = run(docker + ["image", "save", image], 180, MAX_IMAGE_ARCHIVE)
            # Retain bounded, unaccepted bytes before validation for diagnosis.
            # Only a successful inventory is appended to accepted build evidence.
            (evidence / ("compiler-" + str(number) + ".tar")).write_bytes(raw)
            summary["stage"] = "inventory-" + str(number)
            save()
            inspected = inventory.inspect(raw, image)
            (evidence / ("inventory-" + str(number) + ".json")).write_text(
                json.dumps(inspected, sort_keys=True, indent=2) + chr(10))
            summary["builds"].append(inspected)
            save()
        if summary["builds"][0] != summary["builds"][1]:
            raise Invalid("independent provisions differ")
        summary["stage"] = "complete"
        summary["status"] = "candidate-reproduced-not-activated"
        save()
        print("Compiler candidate reproduced; final candidate not started; no target or adapter execution.", flush=True)
    except BaseException as exc:
        summary["status"] = "failed"
        summary["error_type"] = type(exc).__name__
        save()
        # No retry, broad deletion, cache pruning or fallback identity. The
        # ephemeral hosted job/daemon is torn down by the workflow platform.
        raise

def self_test():
    import unittest
    from unittest.mock import patch
    class Tests(unittest.TestCase):
        def test_fixed_acquisition_url(self):
            url = SOURCES["rust-musl.tar.xz"][0]
            self.assertEqual(url_allowed(url), url)
            for bad in ("http://static.rust-lang.org" + urllib.parse.urlsplit(url).path,
                        url + "?x=1", url + "#fragment", url.replace("1.95.0", "1.94.0"),
                        url.replace("static.rust-lang.org", "evil.example"),
                        url.replace("static.rust-lang.org", "user:pass@static.rust-lang.org"),
                        url.replace("static.rust-lang.org", "static.rust-lang.org:8443"),
                        "file:///etc/passwd"):
                with self.assertRaises(Invalid): url_allowed(bad)
            with self.assertRaises(Invalid): acquire("unknown")
        def test_context_contains_no_proposal_or_reference_source(self):
            self.assertEqual(CONTEXT, ("compiler_closure.py", "compiler.Containerfile"))
            self.assertEqual(set(SOURCES), {"rust-musl.tar.xz"})
        def test_private_child_environment(self):
            with patch.dict(os.environ, {"DOCKER_CONFIG": "/owner", "DOCKER_HOST": "tcp://untrusted",
                                         "HTTPS_PROXY": "https://private", "GIT_CONFIG_COUNT": "1"}, clear=True):
                value = child_environment(Path("/cloud/job"))
                self.assertEqual(value["DOCKER_CONFIG"], "/cloud/job/modern-compiler-docker-config")
                for key in ("DOCKER_HOST", "HTTPS_PROXY", "GIT_CONFIG_COUNT", "HOME"):
                    self.assertNotIn(key, value)
            for path in (Path("relative"), Path("/cloud/../owner")):
                with self.assertRaises(Invalid): child_environment(path)
        def test_default_entry_is_denied_before_any_work(self):
            with patch.dict(os.environ, {}, clear=True):
                with self.assertRaises(Invalid): guard()
                with self.assertRaises(Invalid): main()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("CI") != "true" or sys.platform != "linux":
            raise SystemExit("cloud self-test only")
        self_test()
    elif sys.argv[1:]:
        raise SystemExit("no provisioner arguments allowed")
    else:
        main()
