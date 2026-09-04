"""Trusted GitHub-only Foundation controller. Never run on a developer machine."""
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import time

PROFILES = ("normal", "panic", "exception")
READY = ["RAR-BOOT:UEFI", "RAR-KERNEL:ENTRY", "RAR-MEMORY:READY",
         "RAR-ALLOCATOR:READY", "RAR-INTERRUPTS:READY", "RAR-TIMER:READY",
         "RAR-FOUNDATION-READY"]
EXPECTED = {
    "normal": READY,
    "panic": READY[:4] + ["RAR-PANIC:BEGIN", "RAR-PANIC:CODE=SELFTEST", "RAR-PANIC:HALT"],
    "exception": READY[:5] + ["RAR-EXCEPTION:VECTOR=6", "RAR-PANIC:BEGIN",
                             "RAR-PANIC:CODE=EXCEPTION-06", "RAR-PANIC:HALT"],
}
POLICY = dict(network="none", readonly=True, user="65532:65532",
              cpus="2", memory="1024m", pids="64", capabilities="ALL",
              privilege="no-new-privileges", guest_network=False,
              passthrough=False, credentials=False, raw_disk=False)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def check_policy(value):
    if value != POLICY:
        raise ValueError("sandbox policy is not the certified Foundation profile")

def sandbox(policy):
    check_policy(policy)
    return ["--read-only", "--network", policy["network"], "--user", policy["user"],
            "--cpus", policy["cpus"], "--memory", policy["memory"],
            "--memory-swap", policy["memory"], "--pids-limit", policy["pids"],
            "--cap-drop", policy["capabilities"], "--security-opt", policy["privilege"],
            "--tmpfs", "/tmp:rw,exec,nosuid,nodev,size=256m,uid=65532,gid=65532,mode=700",
            "--ulimit", "fsize=67108864:67108864"]

def transcript(profile, data):
    if len(data) > 262144:
        raise ValueError("serial output exceeds budget")
    # Firmware output is allowed before the first RAR record. Every RAR-looking
    # line thereafter is parsed; extra/duplicated/reordered records are errors.
    lines = data.replace(b"\r\n", b"\n").split(b"\n")
    records = []
    for line in lines:
        if b"RAR-" in line:
            records.append(line.decode("ascii", errors="strict"))
    if records != EXPECTED[profile]:
        raise ValueError("unexpected serial transcript: " + repr(records))
    return records

def unpack(data):
    names = [p + "." + ext for p in PROFILES for ext in ("efi", "img")] + ["model-tests.log"]
    lines = data.split(b"\n")
    if len(lines) != 16 or lines[-2:] != [b"RAR-BUILD:END", b""]:
        raise ValueError("malformed or truncated build transfer")
    result = {}
    for i, name in enumerate(names):
        if lines[2*i] != ("RAR-FILE:" + name).encode():
            raise ValueError("unexpected transfer name or ordering")
        raw = base64.b64decode(lines[2*i+1], validate=True)
        if base64.b64encode(raw) != lines[2*i+1]:
            raise ValueError("noncanonical transfer")
        if name.endswith(".img"):
            if len(raw) != 16777216 or raw[510:512] != b"\x55\xaa":
                raise ValueError("invalid FAT image size/signature")
        elif name == "model-tests.log":
            if len(raw) > 8192 or b"test result: ok." not in raw:
                raise ValueError("focused test evidence missing")
        elif not (1024 <= len(raw) <= 2097152 and raw[:2] == b"MZ"):
            raise ValueError("invalid UEFI artifact")
        result[name] = raw
    return result

def negative_tests():
    rejected = 0
    for key, replacement in {
        "network": "host", "readonly": False, "user": "0:0", "cpus": "0",
        "memory": "unlimited", "pids": "-1", "capabilities": "",
        "privilege": "", "guest_network": True, "passthrough": True,
        "credentials": True, "raw_disk": True, "extra": "--device=/dev/sda",
    }.items():
        candidate = dict(POLICY)
        candidate[key] = replacement
        try:
            check_policy(candidate)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError(key)
    for bad in [READY[-1:], READY + [READY[-1]], READY[::-1],
                READY[:-1], ["prefix " + READY[0]] + READY[1:],
                READY + ["RAR-PANIC:HALT"]]:
        try:
            transcript("normal", ("\n".join(bad) + "\n").encode())
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("spoofed transcript accepted")
    for bad in [b"", b"RAR-BUILD:END\n", b"RAR-FILE:../../escape\nAA==\n"]:
        try:
            unpack(bad)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("invalid transfer accepted")
    transcript("normal", ("\n".join(READY) + "\n").encode())
    return rejected

def run(argv, timeout, limit, allowed=(0,), log_path=None):
    # Bounded pipe capture; no shell, source-selected commands or environment.
    child = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    output = bytearray()
    poll = selectors.DefaultSelector()
    poll.register(child.stdout, selectors.EVENT_READ)
    deadline = time.monotonic() + timeout
    try:
        while poll.get_map():
            if time.monotonic() > deadline:
                raise TimeoutError("command exceeded wall-clock budget")
            for key, _ in poll.select(0.2):
                block = os.read(key.fd, 65536)
                if not block:
                    poll.unregister(key.fileobj)
                    continue
                output.extend(block)
                if len(output) > limit:
                    raise ValueError("command exceeded output budget")
        code = child.wait(timeout=3)
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        poll.close()
    if log_path is not None:
        log_path.write_bytes(output)
    if code not in allowed:
        print(bytes(output[-12000:]).decode("utf-8", errors="replace"), flush=True)
        raise RuntimeError("command failed: " + str(argv[:3]) + " exit=" + str(code))
    return code, bytes(output)

def main():
    if not (os.environ.get("GITHUB_ACTIONS") == "true" and
            os.environ.get("GITHUB_REPOSITORY") == "AndyTechCoder/RAR-OS" and
            os.environ.get("RUNNER_OS") == "Linux" and
            os.environ.get("RUNNER_ARCH") == "X64" and
            os.environ.get("ImageOS") == "ubuntu24"):
        raise SystemExit("Foundation controller requires the certified cloud runner")
    workspace = Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)
    controller = workspace / "controller"
    source = workspace / "source"
    if Path(__file__).resolve() != controller / "tools/rar-lab/foundation/controller.py":
        raise SystemExit("controller path mismatch")
    for root, key in [(controller, "RAR_CONTROLLER_SHA"), (source, "RAR_SOURCE_SHA")]:
        sha = os.environ[key]
        if not re.fullmatch("[0-9a-f]{40}", sha):
            raise SystemExit("invalid revision")
        _, actual = run(["git", "-C", str(root), "rev-parse", "HEAD"], 10, 1024)
        if actual.decode().strip() != sha:
            raise SystemExit("checkout identity mismatch")
    run_id = os.environ["GITHUB_RUN_ID"]
    attempt = os.environ["GITHUB_RUN_ATTEMPT"]
    if not re.fullmatch("[0-9]+", run_id) or not re.fullmatch("[0-9]+", attempt):
        raise SystemExit("invalid run identity")
    evidence = workspace / "foundation-evidence"
    evidence.mkdir(exist_ok=False)
    summary = dict(source=os.environ["RAR_SOURCE_SHA"], controller=os.environ["RAR_CONTROLLER_SHA"],
                   run=run_id, attempt=attempt, job=os.environ["GITHUB_JOB"],
                   runner_image=os.environ["ImageVersion"], status="started",
                   policy=POLICY, negative_tests=negative_tests())
    def save():
        (evidence / "manifest.json").write_text(json.dumps(summary, indent=2) + "\n")
    save()
    context = controller / "tools/rar-lab/foundation"
    containers = []
    try:
        for name in ("build", "launch"):
            tag = "rar-foundation-" + name + ":" + run_id + "-" + attempt
            print("Provisioning pinned " + name + " tool image", flush=True)
            _, log = run(["docker", "build", "--pull=false", "--tag", tag,
                          "--file", str(context / (name + ".Containerfile")), str(context)],
                         900, 2097152, log_path=evidence / (name + "-image.log"))
            (evidence / (name + "-image.log")).write_bytes(log)
            _, identity = run(["docker", "image", "inspect", "--format", "{{.Id}}", tag], 10, 1024)
            image = identity.decode().strip()
            if not re.fullmatch("sha256:[0-9a-f]{64}", image):
                raise ValueError("unbound tool image")
            summary[name + "_image"] = image
        # Record exact executable identities from the pinned images.
        for name, command in [
            ("build", "sha256sum /opt/rar-toolchain/bin/rustc /opt/rar-toolchain/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"),
            ("launch", "cat /opt/identities.sha256")]:
            _, identities = run(["docker", "run", "--rm"] + sandbox(POLICY) +
                               ["--entrypoint", "/bin/sh", summary[name + "_image"], "-c", command],
                               20, 8192)
            (evidence / (name + "-identities.txt")).write_bytes(identities)
        builds = []
        for index in (1, 2):
            cname = "rar-foundation-build-" + run_id + "-" + attempt + "-" + str(index)
            containers.append(cname)
            # Stderr is separated inside the container; stdout carries the
            # strict artifact transfer only. Both channels remain bounded.
            argv = ["docker", "run", "--name", cname] + sandbox(POLICY) + [
                "--mount", "type=bind,src=" + str(source) + ",dst=/source,readonly",
                "--entrypoint", "/bin/sh", summary["build_image"], "-c",
                "/bin/sh /opt/rar-build.sh 2>/tmp/build.log; result=$?; "
                "if [ \"$result\" -ne 0 ]; then tail -c 12000 /tmp/build.log; exit \"$result\"; fi"]
            print("Independent build " + str(index), flush=True)
            _, encoded = run(argv, 240, 75497472)
            # Full successful transfer becomes bounded decoded artifacts below.
            build = unpack(encoded)
            builds.append(build)
            summary["build_" + str(index)] = {n: digest(b) for n, b in build.items() if n != "model-tests.log"}
            (evidence / ("model-tests-" + str(index) + ".log")).write_bytes(build["model-tests.log"])
            save()
        if summary["build_1"] != summary["build_2"]:
            raise ValueError("clean builds differ")
        summary["reproducible"] = True
        summary["boots"] = {}
        for profile in PROFILES:
            directory = evidence / profile
            directory.mkdir()
            (directory / "boot.img").write_bytes(builds[0][profile + ".img"])
            (directory / "boot.efi").write_bytes(builds[0][profile + ".efi"])
            cname = "rar-foundation-boot-" + run_id + "-" + attempt + "-" + profile
            containers.append(cname)
            argv = ["docker", "run", "--name", cname] + sandbox(POLICY) + [
                "--mount", "type=bind,src=" + str(directory) + ",dst=/artifact,readonly",
                summary["launch_image"]]
            print("Booting " + profile + " profile in isolated RAR Lab", flush=True)
            code, serial = run(argv, 35, 262144, (124,), directory / "serial.log")
            (directory / "serial.log").write_bytes(serial)
            records = transcript(profile, serial)
            summary["boots"][profile] = dict(exit=code, records=records, serial_sha256=digest(serial),
                                             image_sha256=digest(builds[0][profile + ".img"]))
            save()
        summary["status"] = "passed"
        save()
        print("RAR Foundation: two reproducible builds; normal, panic and exception boots passed.", flush=True)
    except BaseException as error:
        summary["status"] = "failed"
        summary["failure"] = str(error)
        save()
        raise
    finally:
        # Only named disposable cloud containers from this run; never host files.
        for name in containers:
            run(["docker", "rm", "--force", name], 15, 4096, (0, 1))

if __name__ == "__main__":
    main()
