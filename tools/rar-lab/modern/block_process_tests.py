"""Cloud-only process lifecycle tests using disposable synthetic files.
No RAR target, VM, reference adapter, network listener or host disk.
"""
import importlib.util
import os
from pathlib import Path
import socket
import signal
import struct
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

def load(name):
    path = Path(__file__).with_name(name+".py")
    spec = importlib.util.spec_from_file_location(name,path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def main():
    if (sys.argv != [sys.argv[0],"--self-test"] or sys.platform != "linux" or
        not sys.flags.isolated or not sys.dont_write_bytecode or
        os.environ.get("CI") != "true" or os.environ.get("GITHUB_ACTIONS") != "true" or
        os.environ.get("RAR_CI_RUNNER_OS") != "Linux" or
        os.environ.get("RAR_CI_BOOTSTRAP_IMAGE") !=
        "sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"):
        raise SystemExit("existing isolated cloud Specifications boundary only")
    process,wire = load("block_process"),load("block_wire")
    root = Path(tempfile.mkdtemp(prefix="rar-modern-process-tests-",dir="/tmp"))
    descriptors,backends,clients = [],[],[]
    def image(readonly=False):
        path = root/("disk-"+str(len(descriptors)))
        fd = os.open(path,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
        descriptors.append(fd)
        assert os.write(fd,bytes(194*512)) == 194*512
        os.fsync(fd)
        if readonly:
            fd = os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
            descriptors.append(fd)
        return fd
    def start(fd,**kwargs):
        server,client = socket.socketpair()
        client.settimeout(3)
        clients.append(client)
        try:
            backend = process.Backend(fd,server,"data",**kwargs)
        except BaseException:
            server.close()
            raise
        backends.append(backend)
        return backend,client
    def read(client,count):
        data = b""
        while len(data) < count:
            part = client.recv(count-len(data))
            if not part:
                raise EOFError("test socket")
            data += part
        return data
    def handshake(client):
        assert read(client,18) == wire.HELLO
        client.sendall(struct.pack(">I",3))
        body = struct.pack(">IHH",0,1,3)
        header = struct.pack(">QII",wire.OPTION_MAGIC,7,len(body))
        client.sendall(header+body)
        n = wire.Negotiation(194*512,False)
        n.client_flags(struct.pack(">I",3))
        expected = n.option(header,body)
        raw = read(client,len(expected))
        # Read-only exports differ only in advertised transmission flags.
        assert len(raw) == len(expected)
        assert raw[:20] == expected[:20]
    def request(client,kind,cookie=1,offset=0,length=0,data=b"",error=0):
        client.sendall(struct.pack(">IHHQQI",wire.REQUEST_MAGIC,0,kind,cookie,offset,length)+data)
        header = read(client,16)
        assert header == wire.simple(cookie,error)
        return read(client,length) if kind == 0 and error == 0 else b""
    def finished(backend):
        until = time.monotonic()+4
        while time.monotonic() < until:
            code = backend.poll()
            if code is not None and backend.eof:
                return code
            time.sleep(0.005)
        raise AssertionError("test backend did not terminate")

    class Tests(unittest.TestCase):
        def test_flushed_bytes_survive_whole_backend_process_replacement(self):
            fd = image()
            one,c = start(fd)
            handshake(c)
            request(c,1,offset=1024,length=512,data=b"A"*512)
            request(c,3,cookie=2)
            pid = one.process.pid
            result = one.stop()
            self.assertTrue(result["joined"])
            self.assertIsNotNone(one.process.returncode)
            self.assertEqual(os.pread(fd,512,1024),b"A"*512)
            two,c2 = start(fd)
            self.assertNotEqual(pid,two.process.pid)
            handshake(c2)
            self.assertEqual(request(c2,0,offset=1024,length=512),b"A"*512)
            result2 = two.stop()
            self.assertTrue(result2["joined"])
            events = [r["event"] for r in result["records"] if r["type"] == "event"]
            self.assertEqual([e["operation"] for e in events],["write","flush"])
            self.assertEqual([e["status"] for e in events],["completed","completed"])

        def test_unflushed_volatile_bytes_do_not_survive_kill(self):
            fd = image()
            one,c = start(fd)
            handshake(c)
            request(c,1,offset=1024,length=512,data=b"B"*512)
            self.assertEqual(request(c,0,offset=1024,length=512),b"B"*512)
            one.stop()
            self.assertEqual(os.pread(fd,512,1024),bytes(512))
            two,c2 = start(fd)
            handshake(c2)
            self.assertEqual(request(c2,0,offset=1024,length=512),bytes(512))
            two.stop()

        def test_torn_cut_has_no_reply_and_records_actual_prefix(self):
            fd = image()
            b,c = start(fd,fault=dict(operation="write",ordinal=1,effect="torn-cut",prefix=255))
            handshake(c)
            c.sendall(struct.pack(">IHHQQI",wire.REQUEST_MAGIC,0,1,7,1024,512)+b"C"*512)
            self.assertEqual(c.recv(16),b"")
            self.assertEqual(finished(b),20)
            result = b.stop()
            self.assertEqual(result["problem"],"cut")
            self.assertEqual(os.pread(fd,512,1024),b"C"*255+bytes(257))
            terminal = result["records"][-1]
            self.assertEqual(terminal["outcome"],"cut")
            self.assertTrue(terminal["fault_hit"])
            self.assertEqual(result["records"][-2]["event"]["status"],"cut-no-reply")

        def test_readonly_process_rejects_write_preserving_file(self):
            fd = image(readonly=True)
            before = os.pread(fd,194*512,0)
            b,c = start(fd,readonly=True)
            handshake(c)
            request(c,1,offset=1024,length=512,data=b"D"*512,error=1)
            c.sendall(struct.pack(">IHHQQI",wire.REQUEST_MAGIC,0,2,0,0,0))
            self.assertEqual(finished(b),0)
            result = b.stop()
            self.assertEqual(os.pread(fd,194*512,0),before)
            self.assertEqual(result["records"][-1]["outcome"],"closed")
            self.assertIsNone(result["problem"])

        def test_outer_deadline_kills_unresponsive_backend_and_joins(self):
            fd = image()
            b,c = start(fd,seconds=0.1)
            # Stop this exact owned child so its internal socket timer cannot run.
            # This tests the external watchdog rather than a scheduling race.
            b.process.send_signal(signal.SIGSTOP)
            time.sleep(0.15)
            b.poll()
            result = b.stop()
            self.assertTrue(result["joined"])
            self.assertEqual(result["problem"],"deadline")
            self.assertIsNotNone(b.process.returncode)

        def test_post_spawn_setup_failure_reaps_exact_child(self):
            actual_spawn = process.subprocess.Popen
            actual_selector = process.selectors.DefaultSelector
            actual_blocking = process.os.set_blocking
            for stage in ("blocking-1","blocking-2","register-1","register-2"):
                fd = image()
                server,client = socket.socketpair()
                spawned = []
                count = {"blocking":0,"register":0}
                inner = actual_selector()
                class Selector:
                    def register(self,*args):
                        count["register"] += 1
                        if stage == "register-"+str(count["register"]):
                            raise OSError("injected registration failure")
                        return inner.register(*args)
                    def close(self):
                        inner.close()
                def spawn(*args,**kwargs):
                    result = actual_spawn(*args,**kwargs)
                    spawned.append(result)
                    return result
                def blocking(*args):
                    count["blocking"] += 1
                    if stage == "blocking-"+str(count["blocking"]):
                        raise OSError("injected blocking-mode failure")
                    return actual_blocking(*args)
                try:
                    with patch.object(process.subprocess,"Popen",spawn), \
                         patch.object(process.selectors,"DefaultSelector",Selector), \
                         patch.object(process.os,"set_blocking",blocking):
                        with self.assertRaises(OSError):
                            process.Backend(fd,server,"data")
                    self.assertEqual(len(spawned),1)
                    self.assertIsNotNone(spawned[0].poll())
                    self.assertEqual(spawned[0].wait(timeout=0),spawned[0].returncode)
                    self.assertTrue(spawned[0].stdout.closed)
                    self.assertTrue(spawned[0].stderr.closed)
                    self.assertEqual(server.fileno(),-1)
                    self.assertEqual(os.pread(fd,194*512,0),bytes(194*512))
                finally:
                    for child in spawned:
                        if child.poll() is None:
                            child.kill()
                            child.wait(timeout=2)
                    server.close()
                    client.close()
                    inner.close()

        def test_invalid_process_configuration_never_spawns(self):
            fd = image()
            server,client = socket.socketpair()
            try:
                with patch.object(process.subprocess,"Popen") as spawn:
                    for kwargs in (dict(seconds=0),dict(seconds=float("nan")),
                                   dict(seconds=181),dict(readonly=1),
                                   dict(fault={"operation":"write"})):
                        with self.assertRaises(ValueError):
                            process.Backend(fd,server,"data",**kwargs)
                    with self.assertRaises(ValueError):
                        process.Backend(True,server,"data")
                    spawn.assert_not_called()
            finally:
                server.close()
                client.close()

    try:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(Tests)
        assert suite.countTestCases() == 7
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        if not result.wasSuccessful():
            raise SystemExit(1)
        print("Modern backend process: 7 tests; real child kill/join, retained bytes, lost volatile state, torn cut, readonly and deadline; no VM/target execution")
    finally:
        for backend in backends:
            if not backend.closed:
                backend.stop()
        for client in clients:
            client.close()
        for fd in descriptors:
            os.close(fd)
        # No deletion commands. Exclusively created fixtures remain until the
        # already authorized disposable cloud container is torn down.

if __name__ == "__main__":
    main()
