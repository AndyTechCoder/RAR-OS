"""Bounded cloud-only invocation of one already-provisioned Modern adapter.
No image acquisition/build/signing or OS/VM launch. Caller must validate the
reviewed image inventory before invoking this helper; digest alone is not trust.
"""
import os
import json
import re
import selectors
import subprocess
import time
import uuid

class RunFailure(RuntimeError):
    pass

def command(image: str, implementation: int, name: str) -> list[str]:
    if type(image) is not str or re.fullmatch(r"sha256:[0-9a-f]{64}", image) is None:
        raise RunFailure("immutable image identity required")
    if type(implementation) is not int or implementation not in (1, 2, 3):
        raise RunFailure("adapter identity")
    if type(name) is not str or re.fullmatch(r"rar-modern-ref-[0-9a-f]{32}", name) is None:
        raise RunFailure("owned container name")
    entry = {1: "/reference-sodium", 2: "/reference-openssl", 3: "/target-reference"}[implementation]
    return ["/usr/bin/docker", "--host=unix:///var/run/docker.sock", "create",
            "--name", name, "--label", "rar.modern.owner=" + name,
            "--pull=never", "--interactive", "--ipc=none",
            "--network=none", "--read-only", "--user=65532:65532",
            "--cap-drop=ALL", "--security-opt=no-new-privileges",
            "--cpus=1", "--memory=128m", "--memory-swap=128m",
            "--pids-limit=16", "--ulimit=core=0:0", "--ulimit=nofile=64:64",
            "--log-driver=none", "--restart=no", "--stop-timeout=1",
            "--entrypoint", entry, image]

def cloud_guard() -> None:
    if (os.uname().sysname != "Linux" or os.uname().machine != "x86_64" or
        os.environ.get("GITHUB_ACTIONS") != "true" or
        os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_REPOSITORY") != "AndyTechCoder/RAR-OS" or
        os.environ.get("GITHUB_EVENT_NAME") != "workflow_dispatch" or
        os.environ.get("GITHUB_REF") != "refs/heads/main"):
        raise RunFailure("trusted-main cloud invocation only")

def exchange(argv, request, timeout, stdout_limit, stderr_limit):
    """Bounded attached CLI transport. Killing the CLI is NOT container cleanup."""
    env = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C",
           "DOCKER_CONFIG": "/nonexistent"}
    proc = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env, close_fds=True)
    try:
        output = {1: bytearray(), 2: bytearray()}
        position = 0
        deadline = time.monotonic() + timeout
        with selectors.DefaultSelector() as selector:
            for index, stream in enumerate((proc.stdin, proc.stdout, proc.stderr)):
                if index == 0 and not request:
                    stream.close()
                    continue
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_WRITE if index == 0 else selectors.EVENT_READ, index)
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise RunFailure("CLI deadline")
                for key, _ in selector.select(min(remaining, 0.1)):
                    if key.data == 0:
                        try:
                            count = os.write(key.fd, request[position:])
                        except BrokenPipeError as exc:
                            raise RunFailure("adapter closed input") from exc
                        except BlockingIOError:
                            continue
                        if count <= 0:
                            raise RunFailure("adapter input stalled")
                        position += count
                        if position == len(request):
                            selector.unregister(key.fileobj)
                            key.fileobj.close()
                    else:
                        try:
                            part = os.read(key.fd, 4096)
                        except BlockingIOError:
                            continue
                        if not part:
                            selector.unregister(key.fileobj)
                            key.fileobj.close()
                        else:
                            output[key.data].extend(part)
                            if len(output[key.data]) > (stdout_limit if key.data == 1 else stderr_limit):
                                raise RunFailure("CLI output limit")
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise RunFailure("CLI deadline")
            return proc.wait(timeout=remaining), bytes(output[1]), bytes(output[2])
    finally:
        if proc.poll() is None:
            proc.kill()
        proc.wait(timeout=2)
        for stream in (proc.stdin, proc.stdout, proc.stderr):
            if stream is not None and not stream.closed:
                stream.close()

def control(arguments, limit=65536):
    code, output, error = exchange(
        ["/usr/bin/docker", "--host=unix:///var/run/docker.sock"] + arguments,
        b"", 5, limit, 1024)
    if code != 0 or error:
        raise RunFailure("Docker control failed")
    return output

def owned_container(raw, cid, name, image):
    try:
        objects = json.loads(raw)
    except (ValueError, UnicodeError, RecursionError) as exc:
        raise RunFailure("container inspection JSON") from exc
    if type(objects) is not list or len(objects) != 1 or type(objects[0]) is not dict:
        raise RunFailure("container inspection shape")
    item = objects[0]
    config = item.get("Config")
    if (item.get("Id") != cid or item.get("Image") != image or
        item.get("Name") != "/" + name or type(config) is not dict or
        config.get("Labels") != {"rar.modern.owner": name}):
        raise RunFailure("container ownership identity")
    return item

def confined_container(item):
    """Verify effective daemon configuration, not just requested CLI flags."""
    host = item.get("HostConfig")
    if type(host) is not dict:
        raise RunFailure("missing effective confinement")
    exact = {"NetworkMode": "none", "IpcMode": "none", "ReadonlyRootfs": True,
             "Privileged": False, "NanoCpus": 1000000000, "Memory": 134217728,
             "MemorySwap": 134217728, "PidsLimit": 16, "PublishAllPorts": False,
             "AutoRemove": False}
    for key, value in exact.items():
        if host.get(key) != value or type(host.get(key)) is not type(value):
            raise RunFailure("effective confinement: " + key)
    for key in ("Binds", "Mounts", "VolumesFrom", "Devices", "DeviceRequests",
                "Links", "ExtraHosts", "PortBindings", "CapAdd"):
        if host.get(key) not in (None, [], {}):
            raise RunFailure("unexpected host resource: " + key)
    if item.get("Mounts") not in (None, []):
        raise RunFailure("unexpected effective mount")
    if (host.get("CapDrop") != ["ALL"] or
        host.get("SecurityOpt") != ["no-new-privileges"] or
        host.get("PidMode") not in (None, "") or
        host.get("UTSMode") not in (None, "") or
        host.get("UsernsMode") not in (None, "") or
        host.get("LogConfig") != {"Type": "none", "Config": {}} or
        host.get("RestartPolicy") != {"Name": "no", "MaximumRetryCount": 0}):
        raise RunFailure("effective isolation/log/restart policy")
    limits = host.get("Ulimits")
    if type(limits) is not list or len(limits) != 2:
        raise RunFailure("effective resource limits")
    required = [{"Name": "core", "Soft": 0, "Hard": 0},
                {"Name": "nofile", "Soft": 64, "Hard": 64}]
    if any(type(x) is not dict for x in limits) or any(x not in limits for x in required):
        raise RunFailure("effective core/descriptor limits")

def execute(image: str, implementation: int, request: bytes) -> tuple[int, int, bytes, bytes]:
    """Create, verify ownership, then start by ID. Any failure is job-fatal.
    Caller must terminate the disposable job on error, not catch/retry/continue.
    Ambiguous create is never started or removed by name: daemon completion can
    race client disconnect. Only full job teardown closes that unresolved case.
    """
    cloud_guard()
    if type(request) is not bytes or not 16 <= len(request) <= 4432:
        raise RunFailure("request size")
    name = "rar-modern-ref-" + uuid.uuid4().hex
    argv = command(image, implementation, name)
    cid = None
    owned = False
    failure = None
    result = None
    try:
        code, raw, error = exchange(argv, b"", 10, 65, 1024)
        if code != 0 or error or re.fullmatch(b"[0-9a-f]{64}" + bytes([10]), raw) is None:
            raise RunFailure("ambiguous create; terminate disposable job")
        cid = raw[:-1].decode("ascii")
        item = owned_container(control(["container", "inspect", cid]), cid, name, image)
        owned = True
        confined_container(item)
        config = item["Config"]
        state = item.get("State", {})
        expected_entry = {1: "/reference-sodium", 2: "/reference-openssl", 3: "/target-reference"}[implementation]
        if (type(state) is not dict or config.get("User") != "65532:65532" or
            config.get("Entrypoint") != [expected_entry] or
            config.get("Cmd") not in (None, []) or
            config.get("Env") not in (None, []) or
            state.get("Status") != "created" or state.get("Running") is not False):
            raise RunFailure("unexpected pre-start process state")
        code, output, error = exchange(
            ["/usr/bin/docker", "--host=unix:///var/run/docker.sock",
             "start", "--attach", "--interactive", cid], request, 10, 4176, 1024)
        result = (implementation, code, output, error)
    except (OSError, subprocess.SubprocessError, RunFailure) as exc:
        failure = RunFailure(str(exc) + "; terminate disposable job, no retry")
    finally:
        if owned:
            try:
                # Full verified daemon-returned ID, never a caller/name lookup.
                control(["container", "rm", "--force", cid], 128)
                remaining = control(["container", "ls", "--all", "--no-trunc",
                                     "--filter", "id=" + cid, "--format", "{{.ID}}"], 65)
                if remaining != b"":
                    raise RunFailure("owned container remains")
            except (OSError, subprocess.SubprocessError, RunFailure):
                failure = RunFailure("cleanup unconfirmed; terminate disposable job, no retry")
    if failure is not None:
        raise failure
    if result is None:
        raise RunFailure("missing result; terminate disposable job")
    return result

def self_test() -> None:
    import unittest
    from unittest.mock import patch
    class Tests(unittest.TestCase):
        def test_exact_policy(self):
            image = "sha256:" + "a" * 64
            name = "rar-modern-ref-" + "b" * 32
            for ident, entry in ((1, "/reference-sodium"), (2, "/reference-openssl"), (3, "/target-reference")):
                argv = command(image, ident, name)
                self.assertEqual(argv[-3:], ["--entrypoint", entry, image])
                self.assertEqual(argv[:3], ["/usr/bin/docker", "--host=unix:///var/run/docker.sock", "create"])
                for flag in ("--pull=never", "--network=none", "--read-only",
                             "--cap-drop=ALL", "--security-opt=no-new-privileges",
                             "--ulimit=core=0:0", "--user=65532:65532"):
                    self.assertIn(flag, argv)
                for forbidden in ("--privileged", "--volume", "--mount", "--device", "--env", "--pid=host"):
                    self.assertFalse(any(x == forbidden or x.startswith(forbidden + "=") for x in argv))
        def test_no_arbitrary_identity_entrypoint_or_name(self):
            image = "sha256:" + "a" * 64
            name = "rar-modern-ref-" + "b" * 32
            for bad in ("latest", image + "x", image.upper(), "--privileged", "", None):
                with self.assertRaises(RunFailure): command(bad, 1, name)
            for bad in (0, 4, True, "1", None):
                with self.assertRaises(RunFailure): command(image, bad, name)
            for bad in ("existing-container", name + "x", "--all", "", None):
                with self.assertRaises(RunFailure): command(image, 1, bad)
        def test_verified_id_lifecycle_and_ambiguous_create(self):
            image = "sha256:" + "a" * 64
            cid = "c" * 64
            name = "rar-modern-ref-" + "b" * 32
            item = {"Id": cid, "Image": image, "Name": "/" + name,
                    "Config": {"Labels": {"rar.modern.owner": name},
                               "User": "65532:65532", "Entrypoint": ["/reference-sodium"],
                               "Cmd": None, "Env": None},
                    "State": {"Status": "created", "Running": False}}
            module = __name__
            with patch(module + ".cloud_guard"), patch(module + ".uuid.uuid4") as uid:
                uid.return_value.hex = "b" * 32
                with patch(module + ".confined_container"), patch(module + ".exchange", side_effect=[(0, (cid + chr(10)).encode(), b""), (0, b"result", b"")]) as ex:
                    with patch(module + ".control", side_effect=[json.dumps([item]).encode(), b"", b""]) as ctl:
                        self.assertEqual(execute(image, 1, bytes(16)), (1, 0, b"result", b""))
                        self.assertEqual(ex.call_args_list[1].args[0][-4:], ["start", "--attach", "--interactive", cid])
                        self.assertEqual(ctl.call_args_list[1].args[0], ["container", "rm", "--force", cid])
                for effect in (RunFailure("timeout"), (1, b"", b"collision")):
                    with patch(module + ".exchange") as ex, patch(module + ".control") as ctl:
                        if isinstance(effect, Exception): ex.side_effect = effect
                        else: ex.return_value = effect
                        with self.assertRaises(RunFailure): execute(image, 1, bytes(16))
                        ctl.assert_not_called()
                item["Config"]["Labels"] = {"rar.modern.owner": "someone-else"}
                with patch(module + ".exchange", return_value=(0, (cid + chr(10)).encode(), b"")) as ex:
                    with patch(module + ".control", return_value=json.dumps([item]).encode()) as ctl:
                        with self.assertRaises(RunFailure): execute(image, 1, bytes(16))
                        self.assertEqual(ex.call_count, 1)
                        self.assertEqual(ctl.call_count, 1)
        def test_effective_confinement_field_mutations(self):
            import copy
            host = {"NetworkMode": "none", "IpcMode": "none", "ReadonlyRootfs": True,
                    "Privileged": False, "NanoCpus": 1000000000, "Memory": 134217728,
                    "MemorySwap": 134217728, "PidsLimit": 16, "PublishAllPorts": False,
                    "AutoRemove": False, "CapDrop": ["ALL"],
                    "SecurityOpt": ["no-new-privileges"],
                    "LogConfig": {"Type": "none", "Config": {}},
                    "RestartPolicy": {"Name": "no", "MaximumRetryCount": 0},
                    "Ulimits": [{"Name": "core", "Soft": 0, "Hard": 0},
                                {"Name": "nofile", "Soft": 64, "Hard": 64}]}
            confined_container({"HostConfig": host, "Mounts": []})
            for key in host:
                changed = copy.deepcopy(host)
                changed[key] = None
                with self.assertRaises(RunFailure): confined_container({"HostConfig": changed})
            for key in ("Binds", "Mounts", "VolumesFrom", "Devices", "DeviceRequests",
                        "Links", "ExtraHosts", "PortBindings", "CapAdd"):
                changed = copy.deepcopy(host)
                changed[key] = ["unexpected"]
                with self.assertRaises(RunFailure): confined_container({"HostConfig": changed})
            with self.assertRaises(RunFailure):
                confined_container({"HostConfig": host, "Mounts": [{"Source": "/host"}]})
        def test_missing_confinement_fails(self):
            for item in ({}, {"HostConfig": None}, {"HostConfig": {}},
                         {"HostConfig": {"NetworkMode": "host"}}):
                with self.assertRaises(RunFailure): confined_container(item)
        def test_default_environment_cannot_launch(self):
            with patch.dict(os.environ, {}, clear=True):
                with self.assertRaises(RunFailure): cloud_guard()
                with patch.object(subprocess, "Popen") as popen:
                    with self.assertRaises(RunFailure):
                        execute("sha256:" + "a" * 64, 1, bytes(16))
                    popen.assert_not_called()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():
        raise SystemExit(1)

if __name__ == "__main__":
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test entrypoint only")
    self_test()
