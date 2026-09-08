"""Concrete cloud-only Modern VM lifecycle; candidate, not activated.
QEMU stays -S until fixed paused-machine preflight passes. No local entrypoint.
Requires the reviewed immutable tool image and outer trusted-main controller.
"""
import importlib.util
import json
import os
from pathlib import Path
import selectors
import socket
import stat
import subprocess
import sys
import time

def load(name):
    path = Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():
        raise ValueError("fixed tool sibling required")
    spec = importlib.util.spec_from_file_location(name,path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def cloud_guard():
    if (sys.platform != "linux" or os.getuid() != 65532 or not sys.flags.isolated or
        not sys.dont_write_bytecode or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or
        os.environ.get("RAR_CI_RUNNER_OS") != "Linux"):
        raise ValueError("isolated nonroot reviewed Modern cloud container only")
    marker = Path("/opt/rar-modern-container")
    if marker.is_symlink() or marker.read_bytes() != b"trusted-modern-container-v0\n":
        raise ValueError("Modern tool image marker absent")
    # Marker/environment are not certification: outer controller pins the image,
    # checks effective confinement and source identity, and grants execution.

def read_regular(path,limit,exact=None):
    fd = os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
    try:
        before = os.fstat(fd)
        if not stat.S_ISREG(before.st_mode) or not 1 <= before.st_size <= limit:
            raise ValueError("bounded regular tool/evidence file required")
        if exact is not None and before.st_size != exact:
            raise ValueError("fixed file size")
        data = os.pread(fd,before.st_size+1,0)
        after = os.fstat(fd)
        if (len(data) != before.st_size or
            (before.st_dev,before.st_ino,before.st_size) !=
            (after.st_dev,after.st_ino,after.st_size)):
            raise ValueError("file changed during bounded read")
        return data
    finally:
        os.close(fd)

def line_json(raw):
    if type(raw) is not bytes or not 1 <= len(raw) <= 262144:
        raise ValueError("bounded QMP record")
    def pairs(items):
        result = {}
        for key,value in items:
            if key in result:
                raise ValueError("duplicate QMP field")
            result[key] = value
        return result
    try:
        item = json.loads(raw,object_pairs_hook=pairs)
    except (UnicodeError,ValueError,RecursionError) as error:
        raise ValueError("malformed QMP record") from error
    if type(item) is not dict:
        raise ValueError("QMP object required")
    return item

class VM:
    """One fresh firmware instance and one non-reconnectable backend per role.
    All public operations are synchronous but continuously service the backend
    watchdogs and drain bounded serial/QMP output. No guest-supplied command,
    path, disk selector or shell enters this API.
    """
    def __init__(self,index,data_fd,system_fd,readonly_data=False,data_fault=None,
                 reverse_flush=False):
        cloud_guard()
        self.profile,self.backend = load("vm_profile"),load("block_process")
        self.index = index
        self.work = Path(self.profile.directory(index))
        self.readonly_data = readonly_data
        self.boot_fd = None
        self.child = None
        self.connection = None
        self.backends = []
        self.sockets = []
        self.serial,self.qmp_bytes = bytearray(),bytearray()
        self.qmp_total = 0
        self.events = []
        self.commands = []
        self.identity = 0
        self.certified = False
        self.started = False
        self.closed = False
        self.preflight = None
        self.deadline = time.monotonic()+120
        self.selector = selectors.DefaultSelector()
        try:
            root = Path(self.profile.WORK)
            info = root.lstat()
            if (not stat.S_ISDIR(info.st_mode) or info.st_uid != 65532 or
                stat.S_IMODE(info.st_mode) != 0o700):
                raise ValueError("exclusive private Modern session directory")
            self.work.mkdir(mode=0o700,exist_ok=False)
            firmware = read_regular("/usr/share/OVMF/OVMF_VARS.fd",2*1024*1024)
            fd = os.open(self.work/"OVMF_VARS.fd",
                os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
            try:
                if os.write(fd,firmware) != len(firmware):
                    raise OSError("short fresh firmware copy")
                os.fsync(fd)
            finally:
                os.close(fd)
            self.boot_fd = os.open("/artifact/boot.img",os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
            clients = []
            supplied = (("data",data_fd),("system",system_fd),("boot",self.boot_fd))
            for role,fd in supplied:
                server,client = socket.socketpair(socket.AF_UNIX,socket.SOCK_STREAM)
                self.sockets.extend((server,client))
                backend = self.backend.Backend(fd,server,role,
                    readonly=role=="boot" or (role=="data" and readonly_data),
                    write_refusing=role=="boot" or (role=="data" and readonly_data),
                    fault=data_fault if role=="data" else None,
                    reverse_flush=reverse_flush if role=="data" else False,seconds=130)
                self.backends.append(backend)
                clients.append(client)
            # Require each actual child to validate its descriptor/geometry
            # before starting even a paused QEMU machine.
            until = time.monotonic()+5
            while True:
                self.service()
                if all(b.records and b.records[0].get("type")=="ready" for b in self.backends):
                    break
                if time.monotonic() >= until:
                    raise TimeoutError("backend startup proof deadline")
            for backend,device in zip(self.backends,self.profile.DEVICES+(self.profile.BOOT,)):
                ready = backend.records[0]
                info = os.fstat(dict(supplied)[device[0]])
                expected = dict(type="ready",kind=device[0],
                    readonly=device[0]=="boot" or (device[0]=="data" and readonly_data),capacity=device[4],
                    export_readonly=False,
                    device=info.st_dev,inode=info.st_ino)
                if ready != expected:
                    raise ValueError("actual backend descriptor binding")
            self.argv = self.profile.argv(index,clients[0].fileno(),clients[1].fileno(),clients[2].fileno(),readonly_data)
            self.child = subprocess.Popen(self.argv,stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,stderr=subprocess.PIPE,close_fds=True,
                pass_fds=tuple(c.fileno() for c in clients),start_new_session=True,
                env={"PATH":"/usr/bin:/bin","LC_ALL":"C","TMPDIR":str(self.work)})
            for client in clients:
                client.close()
            for stream,label in ((self.child.stdout,"serial"),(self.child.stderr,"stderr")):
                os.set_blocking(stream.fileno(),False)
                self.selector.register(stream,selectors.EVENT_READ,label)
            self.connection = socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
            self.connection.settimeout(0.1)
            while True:
                self.service()
                try:
                    socket_info = (self.work/"qmp.sock").lstat()
                    if not stat.S_ISSOCK(socket_info.st_mode) or socket_info.st_uid != 65532:
                        raise ValueError("private QMP socket identity")
                    self.connection.connect(str(self.work/"qmp.sock"))
                    break
                except (FileNotFoundError,ConnectionRefusedError):
                    continue
            self.connection.setblocking(False)
            self.selector.register(self.connection,selectors.EVENT_READ,"qmp")
            greeting = self.receive()
            if set(greeting) != {"QMP"} or type(greeting["QMP"]) is not dict:
                raise ValueError("QMP greeting")
            self.request({"execute":"qmp_capabilities"})
            evidence = {}
            for key,command in self.profile.preflight_requests():
                evidence[key] = self.request(command)
            self.preflight = dict(raw=evidence,
                verified=self.profile.validate_preflight(evidence,readonly_data))
            self.certified = True
        except BaseException:
            self.destroy()
            raise

    def service(self):
        if self.closed:
            raise ValueError("closed VM")
        if time.monotonic() >= self.deadline:
            raise TimeoutError("whole VM proof deadline")
        for backend in self.backends:
            code = backend.poll()
            if code is not None or backend.problem is not None:
                raise ValueError("block backend stopped before deliberate whole-VM cut")
        if self.child is not None and self.child.poll() is not None:
            raise ValueError("VM exited before deliberate cut")
        for key,_ in self.selector.select(0.01):
            try:
                raw = os.read(key.fileobj.fileno(),65536)
            except BlockingIOError:
                continue
            if not raw:
                raise EOFError("VM/QMP output closed")
            if key.data == "stderr":
                raise ValueError("unexpected QEMU stderr")
            if key.data == "serial":
                self.serial.extend(raw)
                if len(self.serial) > self.profile.SERIAL_LIMIT:
                    raise ValueError("serial budget")
                if b"RAR-PANIC" in self.serial or b"UNEXPECTED-USER-FAULT" in self.serial or b"INVALID-USER-RETURN" in self.serial:
                    raise ValueError("guest panic/isolation failure")
            elif key.data == "qmp":
                self.qmp_total += len(raw)
                self.qmp_bytes.extend(raw)
                if self.qmp_total > 2*1024*1024 or len(self.qmp_bytes) > 262144:
                    raise ValueError("QMP output budget")
            else:
                raise ValueError("unknown monitored descriptor")

    def receive(self):
        while b"\n" not in self.qmp_bytes:
            self.service()
        at = self.qmp_bytes.index(10)
        raw = bytes(self.qmp_bytes[:at]).rstrip(b"\r")
        del self.qmp_bytes[:at+1]
        return line_json(raw)

    def allowed(self,command):
        if command == {"execute":"qmp_capabilities"}:
            return self.identity == 0
        if command in [value for _,value in self.profile.preflight_requests()]:
            return not self.started
        if command == {"execute":"cont"}:
            return self.certified and not self.started
        if command == {"execute":"screendump","arguments":{"filename":str(self.work/"frame.ppm")}}:
            return self.started
        if (type(command) is dict and set(command)=={"execute","arguments"} and
            command["execute"]=="send-key" and self.started):
            args = command["arguments"]
            if type(args) is not dict or set(args)!={"keys","hold-time"} or args["hold-time"] != 50:
                return False
            keys = args["keys"]
            return (type(keys) is list and len(keys)==1 and type(keys[0]) is dict and
                set(keys[0])=={"type","data"} and keys[0]["type"]=="qcode" and
                keys[0]["data"] in tuple("abcdefghijklmnopqrstuvwxyz0123456789")+
                ("spc","ret","backspace","esc","f1","f2","f3","up","down"))
        return False

    def request(self,command):
        if not self.allowed(command) or self.identity >= 512:
            raise ValueError("unapproved QMP command/state/budget")
        self.identity += 1
        message = dict(command,id=self.identity)
        raw = json.dumps(message,separators=(",",":")).encode()+b"\n"
        if len(raw)>2048:
            raise ValueError("QMP request bound")
        # Bounded control write; no replay after partial send.
        self.connection.settimeout(0.1)
        try:
            self.connection.sendall(raw)
        finally:
            self.connection.setblocking(False)
        self.commands.append(message)
        while True:
            answer = self.receive()
            if "event" in answer and "id" not in answer and "error" not in answer:
                if len(self.events)>=32 or len(json.dumps(answer))>2048:
                    raise ValueError("QMP event budget")
                self.events.append(answer)
                continue
            if (set(answer)!={"return","id"} or type(answer["id"]) is not int or
                answer["id"] != self.identity):
                raise ValueError("QMP error/mismatched reply")
            return answer["return"]

    def start(self):
        if self.request({"execute":"cont"}) != {}:
            raise ValueError("QMP continue did not succeed")
        self.started = True

    def delay(self,seconds):
        if type(seconds) not in (int,float) or not 0 <= seconds <= 5:
            raise ValueError("bounded monitored delay")
        until = time.monotonic()+seconds
        while time.monotonic() < until:
            self.service()

    def key(self,key):
        result = self.request({"execute":"send-key","arguments":{
            "keys":[{"type":"qcode","data":key}],"hold-time":50}})
        if result != {}:
            raise ValueError("key injection failed")
        self.delay(0.09)

    def frame(self):
        result = self.request({"execute":"screendump","arguments":{
            "filename":str(self.work/"frame.ppm")}})
        if result != {}:
            raise ValueError("capture failed")
        size = len(b"P6\n640 480\n255\n")+640*480*3
        frame = read_regular(self.work/"frame.ppm",size,exact=size)
        if not frame.startswith(b"P6\n640 480\n255\n"):
            raise ValueError("fixed RGB capture")
        return frame

    def destroy(self):
        """Deliberately kill the WHOLE QEMU first, then all three backends.
        No reset, savevm, writable overlay, graceful disk flush, retry or reuse.
        Every child must be reaped before this can return joined=True.
        """
        if self.closed:
            raise ValueError("VM already destroyed")
        result = {"vm_pid":None,"vm_returncode":None,"backends":[],"joined":False}
        failures = []
        if self.child is not None:
            result["vm_pid"] = self.child.pid
            try:
                if self.child.poll() is None:
                    self.child.kill()
                self.child.wait(timeout=2)
                result["vm_returncode"] = self.child.returncode
                # After reap, discard no unbounded stream: bounded final drain.
                for stream,label in ((self.child.stdout,"serial"),(self.child.stderr,"stderr")):
                    os.set_blocking(stream.fileno(),False)
                    raw = stream.read(self.profile.SERIAL_LIMIT+1)
                    if raw:
                        if label=="stderr" or len(self.serial)+len(raw)>self.profile.SERIAL_LIMIT:
                            failures.append("post-cut output failure")
                        else:
                            self.serial.extend(raw)
            except BaseException as error:
                failures.append(type(error).__name__+": VM reap/drain failed")
        for backend in self.backends:
            try:
                result["backends"].append(backend.stop())
            except BaseException as error:
                failures.append(type(error).__name__+": backend reap failed")
        resources = [self.selector,self.connection]+self.sockets
        if self.child is not None:
            resources += [self.child.stdout,self.child.stderr]
        for resource in resources:
            if resource is not None:
                try:
                    resource.close()
                except BaseException:
                    failures.append("descriptor cleanup failed")
        if self.boot_fd is not None:
            try:
                os.close(self.boot_fd)
                self.boot_fd = None
            except OSError:
                failures.append("boot descriptor close failed")
        if failures:
            raise RuntimeError("; ".join(failures)+"; no frozen-image authority")
        self.closed = True
        result["joined"] = True
        return result

def self_test():
    # Pure parser/authority/lifecycle mocks only; never invokes VM constructor.
    import io
    from unittest.mock import patch
    from types import SimpleNamespace
    profile = load("vm_profile")
    rejected = 0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected += 1
        else: raise AssertionError("invalid VM session input accepted")
    for raw in (b"",b"[]",b'{"id":1,"id":2}',b"["*2000,b"x"*262145):
        reject(lambda raw=raw:line_json(raw))
    assert line_json(b'{"return":{},"id":1}') == {"return":{},"id":1}
    vm = object.__new__(VM)
    vm.profile=profile;vm.work=Path("/tmp/rar-modern/vm-1")
    vm.identity=0;vm.certified=False;vm.started=False
    assert vm.allowed({"execute":"qmp_capabilities"})
    assert not vm.allowed({"execute":"cont"})
    vm.certified=True
    assert vm.allowed({"execute":"cont"})
    vm.started=True
    assert not vm.allowed({"execute":"cont"})
    for command in ({"execute":"system_reset"},{"execute":"quit"},
                    {"execute":"human-monitor-command","arguments":{"command-line":"quit"}},
                    {"execute":"screendump","arguments":{"filename":"/outside"}},
                    {"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":"ctrl"}],"hold-time":50}}):
        assert not vm.allowed(command);rejected += 1
    for key in ("f1","f3","a","spc","ret"):
        assert vm.allowed({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":key}],"hold-time":50}})
    with patch.object(sys,"platform","darwin"):
        reject(cloud_guard)
    order = []
    class Pipe(io.BytesIO):
        def fileno(self): return 99
    class Child:
        pid=71
        returncode=None
        stdout=Pipe(b"")
        stderr=Pipe(b"")
        def poll(self): return self.returncode
        def kill(self): order.append("vm-kill");self.returncode=-9
        def wait(self,timeout): order.append("vm-reap");return self.returncode
    class Backend:
        def stop(self): order.append("backend-reap");return {"joined":True}
    vm.closed=False;vm.boot_fd=None;vm.child=Child();vm.backends=[Backend(),Backend()]
    vm.connection=None;vm.sockets=[];vm.serial=bytearray()
    vm.selector=SimpleNamespace(close=lambda:None)
    with patch.object(os,"set_blocking",lambda *args:None):
        assert vm.destroy()["joined"]
    assert order == ["vm-kill","vm-reap","backend-reap","backend-reap"]
    reject(vm.destroy)
    return rejected

if __name__ == "__main__":
    if sys.argv != [sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure self-test only; VM entry is not activated")
    print("Modern VM session:",self_test(),"pure refusal fixtures; actual VM lifecycle not yet proven")
