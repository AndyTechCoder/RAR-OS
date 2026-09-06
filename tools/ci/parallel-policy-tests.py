"""Two bounded cloud policy-test lanes. No tests are omitted or weakened."""
import os
from pathlib import Path
import re
import selectors
import shlex
import signal
import subprocess
import sys
import time

SUITES = (
    "test-accepted-evidence-v0-policy.sh",
    "test-alpha-crypto-reference-policy.sh",
    "test-alpha-dependency-policy.sh",
    "test-alpha-preimplementation-contract-policy.sh",
    "test-alpha-boot-platform-contract-policy.sh",
    "test-controller-helper-evidence-v0-policy.sh",
    "test-controller-helper-evidence-v1-policy.sh",
    "test-controller-helper-inventory-v0-policy.sh",
    "test-controller-helper-closure-observer-run-evidence-policy.sh",
    "test-controller-helper-closure-observer-policy.sh",
    "test-controller-helper-closure-verifier-evidence-policy.sh",
    "test-development-controller-v2-policy.sh",
    "test-development-image-policy.sh",
    "test-development-lab-profile-policy.sh",
    "test-development-lab-profile-v2-policy.sh",
    "test-frozen-artifact-policy.sh",
    "test-host-policy.sh",
    "test-launch-evidence-policy.sh",
    "test-launch-handshake-policy.sh",
    "test-local-sprint-preflight-policy.sh",
    "test-pinned-file-policy.sh",
    "test-portable-stat-policy.sh",
    "test-qmp-client-source-policy.sh",
    "test-reference-evidence-v0-policy.sh",
    "test-reference-verdict-v0-policy.sh",
    "test-specifications-authority-policy.sh",
    "test-sprint-alpha-gate-report-v2-policy.sh",
    "test-trusted-launcher-policy.sh"
)
class Failure(RuntimeError):
    def __init__(self, reason, logs, codes):
        super().__init__(reason); self.logs = logs; self.codes = codes

def supervise(commands, seconds=1200, maximum=8 * 1024 * 1024):
    if len(commands) != 2 or not 0 < seconds <= 1200 or not 1 <= maximum <= 8 * 1024 * 1024:
        raise ValueError("two bounded workers required")
    processes = []
    logs = [bytearray(), bytearray()]
    reason = None
    stopped = [False]
    old_handlers = {}
    selector = selectors.DefaultSelector()
    def stop(_signum, _frame):
        stopped[0] = True
    try:
        for sig in (signal.SIGHUP, signal.SIGINT, signal.SIGTERM):
            old_handlers[sig] = signal.signal(sig, stop)
        for index, command in enumerate(commands):
            process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                       stderr=subprocess.STDOUT, start_new_session=True)
            processes.append(process)
            os.set_blocking(process.stdout.fileno(), False)
            selector.register(process.stdout, selectors.EVENT_READ, index)
        deadline = time.monotonic() + seconds
        while selector.get_map() or any(process.poll() is None for process in processes):
            if stopped[0] or time.monotonic() >= deadline:
                reason = "interrupted" if stopped[0] else "worker deadline"
                break
            for key, _ in selector.select(0.1):
                value = os.read(key.fileobj.fileno(), 16384)
                if not value:
                    selector.unregister(key.fileobj); key.fileobj.close()
                    continue
                if len(logs[key.data]) + len(value) > maximum:
                    reason = "worker output budget"
                    break
                logs[key.data].extend(value)
            if reason is not None:
                break
    except Exception as exc:
        reason = type(exc).__name__
    finally:
        # Signal only owned live session leaders. Dead-leader/orphan ambiguity
        # cannot become success: open pipes time out, then the caller fails and
        # the containing disposable CI container is torn down.
        for process in processes:
            if process.poll() is None:
                try: os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError: pass
                except OSError: reason = "worker signal"
        for process in processes:
            try: process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                try: os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError: pass
                except OSError: reason = "worker signal"
                try: process.wait(timeout=2)
                except subprocess.TimeoutExpired: reason = "worker cleanup"
            if process.stdout is not None: process.stdout.close()
        selector.close()
        for sig, handler in old_handlers.items(): signal.signal(sig, handler)
    codes = [process.returncode for process in processes]
    if reason is not None or len(codes) != 2 or codes != [0, 0]:
        raise Failure(reason or "worker failed", [bytes(value) for value in logs], codes)
    return [bytes(value) for value in logs]

def partition():
    if len(SUITES) != 28 or len(set(SUITES)) != 28:
        raise ValueError("suite coverage")
    if any(re.fullmatch(r"test-[a-z0-9-]+-policy\.sh", name) is None for name in SUITES):
        raise ValueError("fixed suite name")
    lanes = (SUITES[::2], SUITES[1::2])
    if set(lanes[0]) & set(lanes[1]) or set(lanes[0]) | set(lanes[1]) != set(SUITES):
        raise ValueError("disjoint complete partition")
    return lanes

def main():
    if (sys.argv[1:] != ["--run"] or sys.platform != "linux" or
        os.environ.get("CI") != "true" or os.environ.get("GITHUB_ACTIONS") != "true" or
        os.environ.get("RAR_CI_RUNNER_OS") != "Linux" or
        os.environ.get("RAR_CI_BOOTSTRAP_IMAGE") !=
            "sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"):
        raise SystemExit("isolated cloud policy job only")
    root = Path(__file__).resolve().parents[2]
    gate = subprocess.run(["/bin/sh", str(root / "tools/ci/require-ephemeral-policy-test-root.sh")],
                          stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, timeout=10, check=True)
    if gate.stdout.strip() != b"/tmp": raise SystemExit("ephemeral root mismatch")
    commands = []
    for lane in partition():
        lines = ["set -eu"]
        for name in lane:
            lines.extend(["printf '%s\n' " + shlex.quote("Policy suite: " + name),
                          "/bin/sh " + shlex.quote(str(root / "tools/ci" / name))])
        commands.append(["/bin/sh", "-c", "\n".join(lines)])
    print("Starting two bounded policy lanes; Rust/shared-path suites remain serial.", flush=True)
    try:
        logs = supervise(commands)
    except Failure as exc:
        for index, log in enumerate(exc.logs):
            print("Policy lane " + str(index) + ":", flush=True)
            sys.stdout.buffer.write(log); sys.stdout.buffer.flush()
        raise SystemExit("parallel policy failure: " + str(exc) + " codes=" + repr(exc.codes))
    for index, log in enumerate(logs):
        print("Policy lane " + str(index) + ":", flush=True)
        sys.stdout.buffer.write(log); sys.stdout.buffer.flush()

def self_test():
    import unittest
    command = lambda text: [sys.executable, "-I", "-B", "-c", text]
    class Tests(unittest.TestCase):
        def test_complete_disjoint_partition(self):
            left, right = partition()
            self.assertEqual(len(left), 14); self.assertEqual(len(right), 14)
            self.assertEqual(set(left) | set(right), set(SUITES))
            self.assertFalse(set(left) & set(right))
            self.assertNotIn("test-release-0-reference-harness-policy.sh", SUITES)
            expected = """test-accepted-evidence-v0-policy.sh
test-alpha-crypto-reference-policy.sh
test-alpha-dependency-policy.sh
test-alpha-preimplementation-contract-policy.sh
test-alpha-boot-platform-contract-policy.sh
test-controller-helper-evidence-v0-policy.sh
test-controller-helper-evidence-v1-policy.sh
test-controller-helper-inventory-v0-policy.sh
test-controller-helper-closure-observer-run-evidence-policy.sh
test-controller-helper-closure-observer-policy.sh
test-controller-helper-closure-verifier-evidence-policy.sh
test-development-controller-v2-policy.sh
test-development-image-policy.sh
test-development-lab-profile-policy.sh
test-development-lab-profile-v2-policy.sh
test-frozen-artifact-policy.sh
test-host-policy.sh
test-launch-evidence-policy.sh
test-launch-handshake-policy.sh
test-local-sprint-preflight-policy.sh
test-pinned-file-policy.sh
test-portable-stat-policy.sh
test-qmp-client-source-policy.sh
test-reference-evidence-v0-policy.sh
test-reference-verdict-v0-policy.sh
test-specifications-authority-policy.sh
test-sprint-alpha-gate-report-v2-policy.sh
test-trusted-launcher-policy.sh""".splitlines()
            self.assertEqual(tuple(SUITES), tuple(expected))
        def test_both_success_logs_are_retained_in_lane_order(self):
            self.assertEqual(supervise([command("print('left')"), command("print('right')")], 5),
                             [b"left\n", b"right\n"])
        def test_one_failure_does_not_lose_other_result(self):
            for commands, wanted in (
                ([command("raise SystemExit(3)"), command("print('right')")], [3, 0]),
                ([command("print('left')"), command("raise SystemExit(4)")], [0, 4]),
                ([command("raise SystemExit(3)"), command("raise SystemExit(4)")], [3, 4])):
                with self.assertRaises(Failure) as failure: supervise(commands, 5)
                self.assertEqual(failure.exception.codes, wanted)
        def test_deadline_and_output_are_bounded(self):
            with self.assertRaises(Failure) as failure:
                supervise([command("import time; time.sleep(30)"), command("import time; time.sleep(30)")], 0.1)
            self.assertTrue(all(code is not None for code in failure.exception.codes))
            with self.assertRaises(Failure) as failure:
                supervise([command("print('x' * 4096)"), command("print('right')")], 5, 1024)
            self.assertTrue(all(len(log) <= 1024 for log in failure.exception.logs))
            self.assertTrue(all(code is not None for code in failure.exception.codes))
        def test_signal_is_failure_and_workers_are_reaped(self):
            with self.assertRaises(Failure) as failure:
                supervise([command("import os,signal; os.kill(os.getppid(), signal.SIGTERM)"),
                           command("import time; time.sleep(30)")], 5)
            self.assertTrue(all(code is not None for code in failure.exception.codes))
        def test_partial_start_failure_cleans_the_live_worker(self):
            with self.assertRaises(Failure) as failure:
                supervise([command("import time; time.sleep(30)"), ["/nonexistent/rar-test-worker"]], 5)
            self.assertEqual(len(failure.exception.codes), 1)
            self.assertIsNotNone(failure.exception.codes[0])
        def test_group_signal_reaps_a_grandchild(self):
            import tempfile
            directory = Path(tempfile.mkdtemp(prefix="rar-policy-group-fixture-", dir="/tmp"))
            ready = directory / "ready"
            result = directory / "result"
            leader = "\n".join([
                "import subprocess,sys,signal,time",
                "child=subprocess.Popen([sys.executable,'-I','-B','-c','import time; time.sleep(30)'])",
                "def terminate(signum,frame):",
                "    status='fallback'",
                "    try: status=str(child.wait(timeout=0.8))",
                "    except subprocess.TimeoutExpired:",
                "        child.kill(); child.wait(timeout=0.8)",
                "    with open(" + repr(str(result)) + ",'x') as out: out.write(status)",
                "    raise SystemExit(0)",
                "signal.signal(signal.SIGTERM,terminate)",
                "with open(" + repr(str(ready)) + ",'x') as out: out.write('ready')",
                "time.sleep(30)",
            ])
            trigger = "\n".join([
                "import os,signal,time",
                "from pathlib import Path",
                "deadline=time.monotonic()+2",
                "ready=Path(" + repr(str(ready)) + ")",
                "while not ready.exists() and time.monotonic()<deadline: time.sleep(0.01)",
                "if not ready.exists(): raise SystemExit(2)",
                "os.kill(os.getppid(),signal.SIGTERM)",
            ])
            with self.assertRaises(Failure) as failure:
                supervise([command(leader), command(trigger)], 5)
            self.assertTrue(all(code is not None for code in failure.exception.codes))
            self.assertEqual(result.read_text(), str(-signal.SIGTERM))
            # These two tiny fixture files remain only in the disposable tmpfs.
            # No shared directory or owner file is removed.

    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        if sys.platform != "linux" or os.environ.get("CI") != "true" or os.environ.get("GITHUB_ACTIONS") != "true":
            raise SystemExit("cloud self-test only")
        self_test()
    else:
        main()
