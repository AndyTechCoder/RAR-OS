"""Host-only bounded process boundary for the private synthetic block backend.
No VM launch, image opening, listener, network, deletion or target execution.
Only a reviewed cloud caller may supply owned fixture/socket descriptors.
"""
import importlib.util
import json
import math
import os
from pathlib import Path
import selectors
import socket
import subprocess
import sys
import time

MAX_OUTPUT = 8 * 1024 * 1024
MAX_LINE = 2048
MAX_RECORDS = 16390

def cloud_guard():
    if (sys.platform != "linux" or not sys.flags.isolated or
        not sys.dont_write_bytecode or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or
        os.environ.get("RAR_CI_RUNNER_OS") != "Linux"):
        raise ValueError("isolated reviewed cloud process only")
    # Environment markers are defense in depth, NOT execution authorization.
    # The trusted outer container/controller remains the security boundary.

def configuration(value):
    if (type(value) is not dict or set(value) !=
        {"disk_fd","socket_fd","kind","readonly","fault","reverse_flush","seconds"} or
        type(value["disk_fd"]) is not int or value["disk_fd"] < 3 or
        type(value["socket_fd"]) is not int or value["socket_fd"] < 3 or
        value["disk_fd"] == value["socket_fd"] or
        value["kind"] not in ("data","system") or
        type(value["readonly"]) is not bool or
        type(value["reverse_flush"]) is not bool or
        type(value["seconds"]) not in (int,float) or
        not math.isfinite(value["seconds"]) or not 0 < value["seconds"] <= 180):
        raise ValueError("fixed bounded backend configuration")
    fault = value["fault"]
    if fault is not None:
        if (type(fault) is not dict or set(fault) != {"operation","ordinal","effect","prefix"} or
            fault["operation"] not in ("write","flush") or
            type(fault["ordinal"]) is not int or not 1 <= fault["ordinal"] <= 8192 or
            fault["effect"] not in ("error","before-cut","after-cut","torn-cut","short-error") or
            type(fault["prefix"]) is not int or not 0 <= fault["prefix"] <= 65536 or
            (fault["effect"] not in ("torn-cut","short-error") and fault["prefix"] != 0) or
            value["readonly"]):
            raise ValueError("fixed bounded backend fault")
    return value

def load(name):
    path = Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():
        raise ValueError("fixed read-only sibling source required")
    spec = importlib.util.spec_from_file_location(name,path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def emit(record):
    raw = json.dumps(record,separators=(",",":"),sort_keys=True).encode("ascii")+b"\n"
    if len(raw) > MAX_LINE:
        raise ValueError("bounded backend record")
    # Parent must drain continuously. A blocked audit write is bounded by the
    # external process watchdog, never worked around by dropping evidence.
    at = 0
    while at < len(raw):
        count = os.write(1,raw[at:])
        if count <= 0:
            raise OSError("short audit write")
        at += count

def child(config):
    import resource
    cloud_guard()
    configuration(config)
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    resource.setrlimit(resource.RLIMIT_AS,(128*1024*1024,128*1024*1024))
    cpu = max(1,math.ceil(config["seconds"]))
    resource.setrlimit(resource.RLIMIT_CPU,(cpu,cpu))
    block,wire = load("block_disk"),load("block_wire")
    class Observed(block.Disk):
        def execute(self,operation,offset=0,length=0,data=b""):
            before = len(self.events)
            emit(dict(type="request",operation=operation,offset=offset,length=length))
            try:
                return super().execute(operation,offset,length,data)
            finally:
                if len(self.events) != before:
                    emit(dict(type="event",event=self.events[-1]))
    disk = Observed(config["disk_fd"],config["kind"],config["readonly"],
                    config["fault"],config["reverse_flush"])
    channel = socket.socket(fileno=config["socket_fd"])
    started = time.monotonic()
    emit(dict(type="ready",kind=disk.kind,readonly=disk.readonly,
              capacity=disk.size,device=disk.identity[0],inode=disk.identity[1]))
    try:
        result = wire.serve(channel,disk,started+config["seconds"])
        emit(dict(type="terminal",outcome="closed",result=result,
                  fault_hit=disk.fault_hit,failed=disk.failed))
        return 0
    except block.Cut:
        emit(dict(type="terminal",outcome="cut",fault_hit=disk.fault_hit,failed=disk.failed))
        return 20
    except (OSError,ValueError,EOFError,TimeoutError):
        emit(dict(type="terminal",outcome="failed",fault_hit=disk.fault_hit,failed=disk.failed))
        return 21
    finally:
        channel.close()
        os.close(config["disk_fd"])  # Never flush volatile state on shutdown.

class Backend:
    """Own exactly one child, not a VM. Caller must service poll continuously.
    On any failed/cut/expired child the outer controller must also destroy and
    join its whole VM before freezing retained images. This class cannot prove
    that external VM lifecycle, and never labels a snapshot as acceptance.
    Ownership of server_socket transfers only after successful spawn.
    """
    def __init__(self,disk_fd,server_socket,kind,readonly=False,fault=None,
                 reverse_flush=False,seconds=30):
        cloud_guard()
        config = configuration(dict(disk_fd=disk_fd,socket_fd=server_socket.fileno(),
            kind=kind,readonly=readonly,fault=fault,reverse_flush=reverse_flush,seconds=seconds))
        if (server_socket.family != socket.AF_UNIX or
            server_socket.getsockopt(socket.SOL_SOCKET,socket.SO_TYPE) != socket.SOCK_STREAM or
            server_socket.getsockopt(socket.SOL_SOCKET,socket.SO_ACCEPTCONN) != 0):
            raise ValueError("private connected socketpair required")
        server_socket.getpeername()
        path = Path(__file__).absolute()
        if path.is_symlink() or not path.is_file() or not os.path.isabs(sys.executable):
            raise ValueError("fixed backend source/interpreter required")
        encoded = json.dumps(config,separators=(",",":"),sort_keys=True)
        if len(encoded) > 1024:
            raise ValueError("configuration budget")
        self.records, self.buffer = [],bytearray()
        self.total = 0
        self.problem = None
        self.eof = False
        self.closed = False
        self.deadline = time.monotonic()+seconds
        self.process = subprocess.Popen(
            [sys.executable,"-I","-B",str(path),"--child",encoded],
            stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            close_fds=True,pass_fds=(disk_fd,server_socket.fileno()),
            cwd="/tmp",start_new_session=True,
            env={"CI":"true","GITHUB_ACTIONS":"true","RAR_CI_RUNNER_OS":"Linux"})
        server_socket.close()
        self.selector = selectors.DefaultSelector()
        for stream in (self.process.stdout,self.process.stderr):
            os.set_blocking(stream.fileno(),False)
            self.selector.register(stream,selectors.EVENT_READ)

    def _fail(self,reason):
        if self.problem is None:
            self.problem = reason
        if self.process.poll() is None:
            self.process.kill()  # Exact owned child only; no process-name lookup.

    def poll(self):
        if self.closed:
            raise ValueError("closed backend lifecycle")
        if time.monotonic() >= self.deadline and self.process.poll() is None:
            self._fail("deadline")
        # Bound work per call so a noisy child cannot monopolize VM monitoring.
        budget = 65536
        for key,_ in self.selector.select(0):
            try:
                raw = os.read(key.fileobj.fileno(),min(32768,budget))
            except BlockingIOError:
                continue
            if not raw:
                self.selector.unregister(key.fileobj)
                continue
            budget -= len(raw)
            self.total += len(raw)
            if self.total > MAX_OUTPUT:
                self._fail("output-budget")
                continue
            if key.fileobj is self.process.stderr:
                self._fail("unexpected-stderr")
                continue
            self.buffer.extend(raw)
            while b"\n" in self.buffer:
                at = self.buffer.index(10)
                line = bytes(self.buffer[:at])
                del self.buffer[:at+1]
                if len(line)+1 > MAX_LINE or len(self.records) >= MAX_RECORDS:
                    self._fail("record-budget")
                    continue
                try:
                    record = json.loads(line)
                    if type(record) is not dict or record.get("type") not in ("ready","request","event","terminal"):
                        raise ValueError("record")
                    self.records.append(record)
                except (ValueError,UnicodeError):
                    self._fail("malformed-record")
            if len(self.buffer) > MAX_LINE:
                self._fail("line-budget")
                self.buffer.clear()
            if not budget:
                break
        self.eof = not self.selector.get_map()
        code = self.process.poll()
        if self.eof and self.buffer:
            self._fail("truncated-record")
        if code is not None and code != 0 and self.problem is None:
            self.problem = "cut" if code == 20 else "backend-failed"
        return code

    def stop(self):
        """Kill if alive, drain and join within two seconds; never flush.
        A failure to join is an error, NOT permission to freeze or continue.
        """
        if self.closed:
            raise ValueError("backend already closed")
        if self.process.poll() is None:
            self.process.kill()
        until = time.monotonic()+2
        while (self.process.poll() is None or not self.eof) and time.monotonic() < until:
            self.poll()
            time.sleep(0.005)
        if self.process.poll() is None or not self.eof:
            raise TimeoutError("backend kill/join did not finish; no snapshot authority")
        self.process.wait(timeout=0)
        self.selector.close()
        self.process.stdout.close()
        self.process.stderr.close()
        self.closed = True
        return dict(returncode=self.process.returncode,problem=self.problem,
                    records=list(self.records),joined=True)

if __name__ == "__main__":
    try:
        cloud_guard()
        if len(sys.argv) != 3 or sys.argv[1] != "--child" or len(sys.argv[2]) > 1024:
            raise ValueError("fixed child invocation")
        result = child(configuration(json.loads(sys.argv[2])))
    except (OSError,ValueError,EOFError,TimeoutError):
        result = 22
    raise SystemExit(result)
