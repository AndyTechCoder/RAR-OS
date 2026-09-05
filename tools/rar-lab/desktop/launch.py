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

protocol = {"__file__": str(Path(__file__).resolve().parent / "protocol.py")}
exec(compile((Path(__file__).resolve().parent / "protocol.py").read_text(),
             "desktop-protocol.py", "exec"), protocol)

def remaining(deadline):
    value = deadline - time.monotonic()
    if value <= 0:
        raise TimeoutError("Desktop boot exceeded 90-second proof deadline")
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

    def request(self, operation, index, nonce, scene=0, key=0):
        request = protocol["qmp_request"](operation, index, nonce, scene, key)
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
        if len(frame) != protocol["FRAME_BYTES"] or not frame.startswith(protocol["FRAME_HEADER"]):
            raise ValueError("capture header or actual read length mismatch")
        return frame
    finally:
        os.close(fd)

def main():
    if len(sys.argv) != 1 or os.getuid() != 65532 or sys.desktop != "linux":
        raise SystemExit("fixed cloud container entrypoint only")
    if not Path("/opt/rar-desktop-container").is_file():
        raise SystemExit("trusted tool-image identity absent")
    work = Path("/tmp/rar-desktop")
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

    deadline=time.monotonic()+90
    connection=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
    poll=selectors.DefaultSelector()
    serial=bytearray()
    frames=[]
    nonce="".join(chr(97+(b&15)) for b in os.urandom(8))
    identity=0
    captures=0
    current_scene=0
    def drain(duration):
        until=min(deadline,time.monotonic()+duration)
        while time.monotonic()<until:
            remaining(deadline)
            for key,_ in poll.select(min(0.02,max(0,until-time.monotonic()))):
                block=os.read(key.fd,65536)
                if not block:
                    poll.unregister(key.fileobj)
                    continue
                serial.extend(block)
                if len(serial)>protocol["SERIAL_LIMIT"]:
                    raise ValueError("serial budget")
                last=serial.rfind(b"\n")
                records=protocol["serial_records"](bytes(serial[:last+1]))
                if records!=list(protocol["RECORDS"][:len(records)]):
                    raise ValueError("unexpected live record or kernel failure")
                if len(records)==len(protocol["RECORDS"]) and current_scene<9:
                    raise ValueError("application fault occurred before crash input")
    try:
        while True:
            remaining(deadline)
            if child.poll() is not None: raise ValueError("VM exited before QMP")
            try:
                connection.connect(protocol["SOCKET_PATH"])
                break
            except (FileNotFoundError,ConnectionRefusedError): time.sleep(0.01)
        qmp=Qmp(connection,deadline)
        greeting=qmp.receive()
        if set(greeting)!={"QMP"} or not isinstance(greeting["QMP"],dict):
            raise ValueError("QMP greeting")
        def request(operation,scene=0,key=0):
            nonlocal identity,captures
            identity+=1
            if operation=="capture":
                captures+=1
                if captures>128: raise ValueError("capture budget")
            qmp.request(operation,identity,nonce,scene,key)
        request("capabilities")
        poll.register(child.stdout,selectors.EVENT_READ)
        while "RAR-DESKTOP-READY" not in protocol["serial_records"](bytes(serial)):
            if child.poll() is not None: raise ValueError("VM exited before desktop")
            drain(0.05)
        plan=protocol["oracle"]["plan"](nonce)
        for scene,keys in enumerate(plan):
            current_scene=scene
            for key in range(len(keys)):
                request("key",scene,key)
                drain(0.09)
            scene_deadline=min(deadline,time.monotonic()+5)
            while True:
                if child.poll() is not None: raise ValueError("VM exited during scene")
                request("capture",scene)
                frame=read_frame()
                try:
                    protocol["validate_frame"](frame,scene,nonce)
                    break
                except ValueError:
                    if time.monotonic()>=scene_deadline:
                        raise ValueError("scene did not converge: "+protocol["oracle"]["SCENES"][scene])
                    drain(0.1)
            frames.append(frame)
        drain(0.1)
        protocol["validate_serial"](bytes(serial))
        request("quit")
        while child.poll() is None:
            drain(0.05)
        drain(0.05)
        code=child.wait(timeout=min(2,remaining(deadline)))
        result=dict(serial_b64=base64.b64encode(serial).decode(),
            frames_b64=[base64.b64encode(f).decode() for f in frames],qemu_exit=code,
            nonce=nonce,injected_keys=[k for stage in plan for k in stage],
            frame_sha256=[protocol["validate_frame"](f,i,nonce) for i,f in enumerate(frames)])
        raw=json.dumps(result,separators=(",",":")).encode()
        protocol["validate_result"](raw)
        sys.stdout.buffer.write(raw+b"\n")
    except BaseException as error:
        diagnostic=dict(status="failed",error=str(error)[:2048],scene=current_scene,
            serial_b64=base64.b64encode(serial[:protocol["SERIAL_LIMIT"]]).decode())
        sys.stderr.write(json.dumps(diagnostic,separators=(",",":"))+"\n")
        raise
    finally:
        connection.close();poll.close()
        if child.poll() is None:
            child.terminate()
            try: child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                child.kill();child.wait(timeout=2)


def self_test():
    """In-memory parser/file-adapter faults; never starts a VM or writes a file."""
    import io
    from types import SimpleNamespace
    from unittest.mock import patch
    rejected=0
    class Fake:
        def __init__(self,chunks): self.chunks=list(chunks);self.sent=[]
        def settimeout(self,value): assert 0<value<=2
        def recv(self,size):
            if not self.chunks: return b""
            value=self.chunks.pop(0)
            if isinstance(value,Exception): raise value
            return value
        def sendall(self,value): self.sent.append(value)
    def reject(fn):
        nonlocal rejected
        try: fn()
        except (ValueError,TimeoutError,OSError): rejected+=1
        else: raise AssertionError("malformed QMP/capture accepted")
    for chunks in ([b'{"x":1,"x":2}\n'],[b'[]\n'],[b'!\n'],
                   [b"x"*16385],[b""],[TimeoutError("fixture")]):
        reject(lambda chunks=chunks: Qmp(Fake(chunks),time.monotonic()+1).receive())
    nonce="abcdefgh"
    for chunks in ([b'{"return":{},"id":2}\n'],[b'{"return":{},"id":true}\n'],
                   [b'{"error":{},"id":1}\n'],[b'{"return":{},"id":1,"extra":1}\n'],
                   [b'{"return":{"x":1},"id":1}\n'],
                   [b'{"event":"TEST"}\n']*33):
        reject(lambda chunks=chunks: Qmp(Fake(chunks),time.monotonic()+1).request("capture",1,nonce))
    reject(lambda: Qmp(Fake([]),time.monotonic()-1).request("capture",1,nonce))
    good=Fake([b'{"ret',b'urn":{},',b'"id":1}\n'])
    Qmp(good,time.monotonic()+1).request("capture",1,nonce)
    assert json.loads(good.sent[0])["arguments"]=={"filename":protocol["FRAME_PATH"]}
    frame=protocol["oracle"]["expected"](0,nonce)
    def capture(mode,size,data,opening=None):
        def opened(path,flags):
            assert path==protocol["FRAME_PATH"] and flags & os.O_NOFOLLOW
            if opening: raise opening
            return 71
        with patch.object(os,"open",opened),patch.object(os,"close",lambda fd:None), \
             patch.object(os,"fstat",lambda fd:SimpleNamespace(st_mode=mode,st_size=size)), \
             patch.object(os,"fdopen",lambda *args,**kwargs:io.BytesIO(data)):
            return read_frame()
    regular=stat.S_IFREG|0o600
    for error in (FileNotFoundError("missing"),OSError(40,"symlink refused")):
        reject(lambda error=error:capture(regular,len(frame),frame,error))
    reject(lambda:capture(stat.S_IFDIR, len(frame),frame))
    reject(lambda:capture(regular,len(frame)-1,frame))
    reject(lambda:capture(regular,len(frame),b"X"+frame[1:]))
    reject(lambda:capture(regular,len(frame),frame[:-1]))
    reject(lambda:capture(regular,len(frame),frame+b"x"))
    assert capture(regular,len(frame),frame)==frame
    assert rejected==20
    return rejected

if __name__=="__main__": main()

