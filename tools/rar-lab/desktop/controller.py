"""Trusted-main Desktop controller. Cloud-only; never execute on developer hosts."""
import base64
import json
import os
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
helpers = {"__name__": "trusted_foundation_helpers"}
exec(compile((HERE.parent / "foundation/controller.py").read_text(),
             "trusted-foundation-controller.py", "exec"), helpers)
protocol = {"__name__": "trusted_desktop_protocol", "__file__": str(HERE / "protocol.py")}
exec(compile((HERE / "protocol.py").read_text(), "trusted-desktop-protocol.py", "exec"), protocol)
run, digest, sandbox = (helpers[name] for name in ("run", "digest", "sandbox"))
POLICY = helpers["POLICY"]

def kernel_state(tree):
    prefixes = ("nucleus/desktop/", "core/desktop/", "services/desktop/", "apps/desktop/")
    present = any(p.startswith(prefixes) or p in tuple(x[:-1] for x in prefixes) for p in tree)
    if not present:
        return "absent"
    required = ("nucleus/desktop/main.rs", "nucleus/desktop/model.rs",
                "core/desktop/main.rs", "services/desktop/model.rs", "apps/desktop/model.rs")
    return "complete" if all(tree.get(p, ("", ""))[0] in ("100644", "100755")
                             for p in required) else "partial"

def source_kind(base, source, changed):
    if "partial" in (base, source):
        raise ValueError("partial Desktop source")
    if base == "complete" and source == "absent":
        raise ValueError("proposal removes Desktop")
    if source == "complete":
        return "target"
    if base != "absent" or source != "absent":
        raise ValueError("invalid Desktop source state")
    exact = {"AGENTS.md", "README.md", "SPRINT_STATUS.md",
             "tools/ci/check-sprint-static.sh", ".github/workflows/desktop.yml"}
    if any(p not in exact and not p.startswith(("tools/rar-lab/desktop/", "docs/"))
           for p in changed):
        raise ValueError("kernel-absent proposal exceeds controller/contract scope")
    return "controller-only"

def unpack(data):
    names = ("desktop.efi", "desktop.img", "desktop-service.efi", "model-tests.log")
    if not isinstance(data, bytes) or len(data) > 33554432:
        raise ValueError("build transfer outside bound")
    lines = data.split(b"\n")
    if len(lines) != 10 or lines[-2:] != [b"RAR-BUILD:END", b""]:
        raise ValueError("truncated or malformed transfer")
    result = {}
    for index, name in enumerate(names):
        if lines[2 * index] != ("RAR-FILE:" + name).encode():
            raise ValueError("artifact name/order mismatch")
        raw = base64.b64decode(lines[2 * index + 1], validate=True)
        if base64.b64encode(raw) != lines[2 * index + 1]:
            raise ValueError("noncanonical artifact encoding")
        if name.endswith(".img"):
            if len(raw) != 16777216 or raw[510:512] != b"\x55\xaa":
                raise ValueError("invalid bounded FAT boot artifact")
        elif name.endswith(".efi"):
            if not (1024 <= len(raw) <= 2097152 and raw[:2] == b"MZ"):
                raise ValueError("invalid bounded executable artifact")
        elif len(raw) > 16384 or raw.count(b"test result: ok.") != 2 or b"FAILED" in raw:
            raise ValueError("both focused kernel/service test suites must pass")
        result[name] = raw
    return result

def self_test():
    import runpy
    launcher=runpy.run_path(str(HERE/"launch.py"),run_name="desktop_launcher_test")
    rejected = protocol["self_test"]() + helpers["negative_tests"]() + launcher["self_test"]()
    assert source_kind("absent", "absent", ["docs/tasks/a.md"]) == "controller-only"
    assert source_kind("absent", "complete", []) == "target"
    assert source_kind("complete", "complete", []) == "target"
    assert kernel_state({}) == "absent"
    assert kernel_state({"core/desktop/main.rs": ("120000", "x")}) == "partial"
    for base, source, paths in (
        ("complete", "absent", []), ("partial", "complete", []),
        ("absent", "partial", []), ("absent", "absent", ["nucleus/foundation/main.rs"]),
        ("unknown", "absent", []),
    ):
        try:
            source_kind(base, source, paths)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("invalid source classification accepted")
    for raw in (b"", b"RAR-BUILD:END\n", b"RAR-FILE:../../escape\nAA==\n",
                b"x" * 33554433):
        try:
            unpack(raw)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("invalid build transfer accepted")
    return rejected

def main():
    if not (os.environ.get("GITHUB_ACTIONS") == "true" and
            os.environ.get("GITHUB_REPOSITORY") == "AndyTechCoder/RAR-OS" and
            os.environ.get("RUNNER_OS") == "Linux" and
            os.environ.get("RUNNER_ARCH") == "X64" and
            os.environ.get("ImageOS") == "ubuntu24"):
        raise SystemExit("Desktop controller requires the certified cloud runner")
    workspace = Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)
    controller, source = workspace / "controller", workspace / "source"
    if HERE != controller / "tools/rar-lab/desktop":
        raise SystemExit("trusted controller path mismatch")
    for root, key in ((controller, "RAR_CONTROLLER_SHA"), (source, "RAR_SOURCE_SHA")):
        if not re.fullmatch("[0-9a-f]{40}", os.environ[key]):
            raise SystemExit("invalid revision")
        _, actual = run(["git", "-C", str(root), "rev-parse", "HEAD"], 10, 1024)
        if actual.decode().strip() != os.environ[key]:
            raise SystemExit("checkout identity mismatch")
    run_id, attempt = os.environ["GITHUB_RUN_ID"], os.environ["GITHUB_RUN_ATTEMPT"]
    if not re.fullmatch("[0-9]+", run_id) or not re.fullmatch("[0-9]+", attempt):
        raise SystemExit("invalid run identity")
    evidence = workspace / "desktop-evidence"
    evidence.mkdir(exist_ok=False)
    summary = dict(source=os.environ["RAR_SOURCE_SHA"], controller=os.environ["RAR_CONTROLLER_SHA"],
                   run=run_id, attempt=attempt, job=os.environ["GITHUB_JOB"],
                   runner_image=os.environ["ImageVersion"], policy=POLICY,
                   negative_tests=self_test(), status="started",
                   profile="q35-tcg-desktop-keyboard-multiscene-v0")
    def save():
        (evidence / "manifest.json").write_text(json.dumps(summary, indent=2) + "\n")
    save()
    containers = []
    try:
        base_tree, source_tree = helpers["git_tree"](controller), helpers["git_tree"](source)
        changed = [p for p in base_tree.keys() | source_tree.keys()
                   if base_tree.get(p) != source_tree.get(p)]
        kind = source_kind(kernel_state(base_tree), kernel_state(source_tree), changed)
        if kind == "controller-only":
            summary.update(status=kind, target_execution="none: Desktop implementation absent")
            save()
            print("Controller/contract validation only; no Desktop implementation or boot claim.", flush=True)
            return
        for name in ("build", "launch"):
            tag = "rar-desktop-" + name + ":" + run_id + "-" + attempt
            print("Provisioning pinned " + name + " image", flush=True)
            run(["docker", "build", "--pull=false", "--tag", tag,
                 "--file", str(HERE / (name + ".Containerfile")), str(HERE)],
                900, 2097152, log_path=evidence / (name + "-image.log"))
            _, identity = run(["docker", "image", "inspect", "--format", "{{.Id}}", tag], 10, 1024)
            image = identity.decode().strip()
            if not re.fullmatch("sha256:[0-9a-f]{64}", image):
                raise ValueError("tool image identity missing")
            summary[name + "_image"] = image
        for name, command in (
            ("build", "sha256sum /opt/rar-toolchain/bin/rustc /opt/rar-toolchain/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"),
            ("launch", "cat /opt/identities.sha256"),
        ):
            _, data = run(["docker", "run", "--rm"] + sandbox(POLICY) +
                          ["--entrypoint", "/bin/sh", summary[name + "_image"], "-c", command], 20, 8192)
            (evidence / (name + "-identities.txt")).write_bytes(data)
        builds = []
        for index in (1, 2):
            name = "rar-desktop-build-" + run_id + "-" + attempt + "-" + str(index)
            containers.append(name)
            argv = ["docker", "run", "--name", name] + sandbox(POLICY) + [
                "--mount", "type=bind,src=" + str(source) + ",dst=/source,readonly",
                "--entrypoint", "/bin/sh", summary["build_image"], "-c",
                '/bin/sh /opt/rar-build.sh 2>/tmp/build.log; result=$?; '
                'if [ "$result" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit "$result"; fi']
            print("Independent Desktop build " + str(index), flush=True)
            _, encoded = run(argv, 240, 33554432)
            build = unpack(encoded)
            builds.append(build)
            summary["build_" + str(index)] = {n: digest(b) for n, b in build.items()
                                              if n != "model-tests.log"}
            (evidence / ("model-tests-" + str(index) + ".log")).write_bytes(build["model-tests.log"])
            save()
        if summary["build_1"] != summary["build_2"]:
            raise ValueError("Desktop clean builds differ")
        summary["reproducible"] = True
        directory = evidence / "boot"
        directory.mkdir()
        for name, raw in builds[0].items():
            if name != "model-tests.log":
                (directory / ("boot.img" if name == "desktop.img" else name)).write_bytes(raw)
        name = "rar-desktop-boot-" + run_id + "-" + attempt
        containers.append(name)
        print("Booting isolated Desktop with fixed synthetic input/capture", flush=True)
        _, result = run(["docker", "run", "--name", name] + sandbox(POLICY) + [
            "--mount", "type=bind,src=" + str(directory) + ",dst=/artifact,readonly",
            summary["launch_image"]], 100, protocol["RESULT_LIMIT"], log_path=directory / "launch-result.log")
        serial, frames, records, nonce = protocol["validate_result"](result)
        (directory / "serial.log").write_bytes(serial)
        for index, frame in enumerate(frames):
            (directory / ("scene-" + protocol["oracle"]["SCENES"][index] + ".ppm")).write_bytes(frame)
        summary.update(status="passed", records=records,
                       serial_sha256=digest(serial), frame_sha256=[digest(frame) for frame in frames],
                       scenes=list(protocol["oracle"]["SCENES"]), nonce=nonce,
                       qemu_exit=0, injected_keys=[k for stage in protocol["oracle"]["plan"](nonce) for k in stage])
        save()
        print("Desktop: two reproducible builds, isolated apps, real keyboard actions and twelve independent scenes passed.", flush=True)
    except BaseException as error:
        summary.update(status="failed", failure=str(error))
        save()
        raise
    finally:
        for name in containers:
            run(["docker", "rm", "--force", name], 15, 4096, (0, 1))

if __name__ == "__main__":
    main()
