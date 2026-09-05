"""Fixed cloud-container-only QMP input/capture controller. No CLI options."""
import base64
import json
import os
from pathlib import Path
import selectors
import socket
import stat
import subprocess
import sys
import time

protocol = {}
exec(compile((Path(__file__).resolve().parent / "protocol.py").read_text(),
             "platform-protocol.py", "exec"), protocol)

def remaining(deadline):
    value = deadline - time.monotonic()
    if value <= 0:
        raise TimeoutError("Platform boot exceeded 25-second proof deadline")
    return value

class Qmp:
    def __init__(self, connection, deadline):
        self.connection, self.deadline = connection, deadline
        self.pending = bytearray()
        self.events = 0

    def receive(self):
        while b"\n" not in self.pending:
            self.connection.settimeout(min(2, remaining(self.deadline)))
            block = self.connection.recv(4096)
            if not block:
                raise ValueError("QMP closed before response")
            self.pending.extend(block)
            if len(self.pending) > 16384:
                raise ValueError("QMP response exceeds bound")
        line, _, rest = self.pending.partition(b"\n")
        self.pending = bytearray(rest)
        item = json.loads(line, object_pairs_hook=protocol["unique_pairs"])
        if not isinstance(item, dict):
            raise ValueError("QMP response is not an object")
        return item

    def request(self, index):
        request = protocol["qmp_request"](index)
        self.connection.settimeout(min(2, remaining(self.deadline)))
        self.connection.sendall(json.dumps(request).encode() + b"\n")
        while True:
            item = self.receive()
            if "event" in item and "id" not in item and "error" not in item:
                self.events += 1
                if self.events > 32:
                    raise ValueError("QMP event budget exhausted")
                continue
            if type(item.get("id")) is not int or item["id"] != index:
                raise ValueError("QMP response identity mismatch")
            if set(item) != {"return", "id"} or item["return"] != {}:
                raise ValueError("QMP command failed or unexpected return")
            return

def read_frame():
    fd = os.open(protocol["FRAME_PATH"], os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_size != protocol["FRAME_BYTES"]:
            raise ValueError("capture is not the fixed bounded regular file")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            frame = stream.read(protocol["FRAME_BYTES"] + 1)
        protocol["validate_frame"](frame)
        return frame
    finally:
        os.close(fd)

def main():
    if len(sys.argv) != 1 or os.getuid() != 65532 or sys.platform != "linux":
        raise SystemExit("fixed cloud container entrypoint only")
    if not Path("/opt/rar-platform-container").is_file():
        raise SystemExit("trusted tool-image identity absent")
    work = Path("/tmp/rar-platform")
    work.mkdir(mode=0o700, exist_ok=False)
    # QEMU rewrites TMPDIR=/tmp to /var/tmp; use the private nested tmpfs path.
    environment = {"PATH": "/usr/bin:/bin", "LC_ALL": "C", "TMPDIR": str(work)}
    firmware = work / "OVMF_VARS.fd"
    firmware.write_bytes(Path("/usr/share/OVMF/OVMF_VARS.fd").read_bytes())
    argv = [
        "/usr/bin/qemu-system-x86_64", "-machine", "q35,accel=tcg",
        "-cpu", "qemu64", "-smp", "1", "-m", "256M",
        "-nodefaults", "-no-user-config", "-display", "none", "-monitor", "none",
        "-serial", "stdio", "-nic", "none", "-no-reboot", "-no-shutdown",
        "-device", "VGA",
        "-qmp", "unix:" + protocol["SOCKET_PATH"] + ",server=on,wait=off",
        "-sandbox", "on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny",
        "-drive", "if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd",
        "-drive", "if=pflash,format=raw,file=" + str(firmware),
        "-drive", "if=ide,format=raw,snapshot=on,file=/artifact/boot.img",
    ]
    child = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT, env=environment)
    deadline = time.monotonic() + 25
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    poll = selectors.DefaultSelector()
    serial, frame = bytearray(), None
    injected, captured, finished = False, False, False
    try:
        while True:
            remaining(deadline)
            if child.poll() is not None:
                raise ValueError("VM exited before QMP connected")
            try:
                connection.connect(protocol["SOCKET_PATH"])
                break
            except (FileNotFoundError, ConnectionRefusedError):
                time.sleep(0.01)
        qmp = Qmp(connection, deadline)
        greeting = qmp.receive()
        if set(greeting) != {"QMP"} or not isinstance(greeting["QMP"], dict):
            raise ValueError("QMP greeting mismatch")
        qmp.request(1)
        poll.register(child.stdout, selectors.EVENT_READ)
        while poll.get_map():
            remaining(deadline)
            for key, _ in poll.select(min(0.1, remaining(deadline))):
                block = os.read(key.fd, 65536)
                if not block:
                    poll.unregister(key.fileobj)
                    continue
                serial.extend(block)
                if len(serial) > protocol["SERIAL_LIMIT"]:
                    raise ValueError("serial budget exhausted")
                # Only complete lines can authorize the next fixed operation.
                last_newline = serial.rfind(b"\n")
                complete = bytes(serial[:last_newline + 1])
                records = protocol["serial_records"](complete)
                if records != list(protocol["RECORDS"][:len(records)]):
                    raise ValueError("invalid or out-of-order live proof")
                if "RAR-PLATFORM:INPUT-WAIT" in records and not injected:
                    # Already seeing input proof before injection is invalid.
                    if "RAR-PLATFORM:INPUT-PASS" in records:
                        raise ValueError("input proof preceded synthetic input")
                    qmp.request(2)
                    injected = True
                if "RAR-PLATFORM:CAPTURE" in records and not captured:
                    if not injected:
                        raise ValueError("capture preceded input")
                    qmp.request(3)
                    frame = read_frame()
                    captured = True
                if "RAR-PLATFORM-READY" in records and not finished:
                    if not captured:
                        raise ValueError("completion preceded actual capture")
                    qmp.request(4)
                    finished = True
        code = child.wait(timeout=min(2, remaining(deadline)))
        if not (injected and captured and finished and code == 0):
            raise ValueError("Platform proof sequence did not complete")
        result = dict(serial_b64=base64.b64encode(serial).decode(),
                      frame_b64=base64.b64encode(frame).decode(), qemu_exit=code,
                      injected_keys=["a"], frame_sha256=protocol["validate_frame"](frame))
        raw = json.dumps(result, separators=(",", ":")).encode()
        protocol["validate_result"](raw)
        sys.stdout.buffer.write(raw + b"\n")
    except BaseException as error:
        # Retain bounded guest diagnostics without treating them as trusted proof.
        diagnostic = dict(status="failed", error=str(error)[:2048],
                          serial_b64=base64.b64encode(serial[:protocol["SERIAL_LIMIT"]]).decode())
        sys.stderr.write(json.dumps(diagnostic, separators=(",", ":")) + "\n")
        raise
    finally:
        connection.close()
        poll.close()
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=2)

if __name__ == "__main__":
    main()
