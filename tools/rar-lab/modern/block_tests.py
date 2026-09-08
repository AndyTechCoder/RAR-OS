"""Cloud-only real regular-file and private-socket tests. Never a VM launch."""
import importlib.util
import os
from pathlib import Path
import socket
import struct
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

def load(name):
    path = Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():
        raise ValueError("trusted sibling test source required")
    spec = importlib.util.spec_from_file_location(name,path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def main():
    if (sys.argv != [sys.argv[0],"--self-test"] or not sys.flags.isolated or
        not sys.dont_write_bytecode or sys.platform != "linux" or
        os.environ.get("CI") != "true" or os.environ.get("GITHUB_ACTIONS") != "true" or
        os.environ.get("RAR_CI_RUNNER_OS") != "Linux" or
        os.environ.get("RAR_CI_BOOTSTRAP_IMAGE") !=
        "sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"):
        raise SystemExit("existing isolated cloud Specifications boundary only")
    block,wire = load("block_disk"),load("block_wire")
    root = Path(tempfile.mkdtemp(prefix="rar-modern-block-tests-",dir="/tmp"))
    descriptors = []
    serial = 0
    def image(kind="data",readonly=False,**kwargs):
        nonlocal serial
        serial += 1
        path = root/("image-"+str(serial))
        fd = os.open(path,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
        descriptors.append(fd)
        data = bytes(block.CAPACITY[kind])
        assert os.write(fd,data) == len(data)
        os.fsync(fd)
        if readonly:
            ro = os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
            descriptors.append(ro)
            fd = ro
        return block.Disk(fd,kind,readonly=readonly,**kwargs)
    def plan(operation,effect,prefix=0,ordinal=1):
        return dict(operation=operation,ordinal=ordinal,effect=effect,prefix=prefix)
    def option(kind,data=b""):
        return struct.pack(">QII",wire.OPTION_MAGIC,kind,len(data)),data
    def negotiation(readonly=False):
        n = wire.Negotiation(194*512,readonly)
        n.client_flags(struct.pack(">I",3))
        return n
    go = struct.pack(">IHH",0,1,3)
    def packet(kind,cookie=0,offset=0,length=0,flags=0,magic=None):
        return struct.pack(">IHHQQI",wire.REQUEST_MAGIC if magic is None else magic,
                           flags,kind,cookie,offset,length)

    class Tests(unittest.TestCase):
        def test_write_is_volatile_until_real_flush_and_fresh_reader(self):
            d = image()
            d.execute("write",1024,512,b"A"*512)
            self.assertEqual(d.execute("read",1024,512),b"A"*512)
            self.assertEqual(d.frozen()[0][1024:1536],bytes(512))
            self.assertEqual(block.Disk(d.fd,"data").execute("read",1024,512),bytes(512))
            d.execute("flush")
            fresh = block.Disk(d.fd,"data")
            self.assertEqual(fresh.execute("read",1024,512),b"A"*512)
            self.assertEqual(d.frozen(),fresh.frozen())
            self.assertEqual(d.events[-1]["status"],"completed")

        def test_reordered_flush_commits_every_dirty_sector(self):
            d = image(reverse_flush=True)
            d.execute("write",1024,512,b"A"*512)
            d.execute("write",1536,512,b"B"*512)
            d.execute("write",1024,512,b"C"*512)
            actual = os.pwrite
            writes = []
            def observed(fd,data,offset):
                self.assertEqual(fd,d.fd)
                writes.append(offset)
                return actual(fd,data,offset)
            with patch.object(block.os,"pwrite",observed):
                d.execute("flush")
            self.assertEqual(writes,[1536,1024])
            self.assertEqual(d.frozen()[0][1024:2048],b"C"*512+b"B"*512)
            self.assertEqual(d.dirty,set())

        def test_write_prefix_cuts_are_actual_retained_bytes(self):
            for prefix in (0,1,255,511,512):
                d = image(fault=plan("write","torn-cut",prefix))
                with self.assertRaises(block.Cut):
                    d.execute("write",1024,512,b"A"*512)
                self.assertEqual(d.frozen()[0][1024:1536],b"A"*prefix+bytes(512-prefix))
                self.assertTrue(d.fault_hit)
                self.assertEqual(d.events[-1]["status"],"cut-no-reply")
                with self.assertRaises(block.DeviceError):
                    d.execute("flush")

        def test_flush_prefix_and_before_after_cut(self):
            d = image(reverse_flush=True,fault=plan("flush","torn-cut",513))
            d.execute("write",1024,512,b"A"*512)
            d.execute("write",1536,512,b"B"*512)
            with self.assertRaises(block.Cut):
                d.execute("flush")
            self.assertEqual(d.frozen()[0][1024:2048],b"A"+bytes(511)+b"B"*512)
            for effect,expected in (("before-cut",bytes(512)),("after-cut",b"X"*512)):
                d = image(fault=plan("flush",effect))
                d.execute("write",1024,512,b"X"*512)
                with self.assertRaises(block.Cut):
                    d.execute("flush")
                self.assertEqual(d.frozen()[0][1024:1536],expected)

        def test_short_error_and_real_io_failure_are_sticky(self):
            d = image(fault=plan("write","short-error",257))
            with self.assertRaises(block.DeviceError):
                d.execute("write",1024,512,b"A"*512)
            self.assertEqual(d.frozen()[0][1024:1536],b"A"*257+bytes(255))
            with self.assertRaises(block.DeviceError):
                d.execute("read",1024,512)
            for operation in ("pwrite","fsync"):
                d = image()
                d.execute("write",1024,512,b"B"*512)
                with patch.object(block.os,operation,side_effect=OSError("owned fixture failure")):
                    with self.assertRaises(OSError):
                        d.execute("flush")
                self.assertTrue(d.failed)
                self.assertEqual(d.events[-1]["status"],"failed-no-success")
                with self.assertRaises(block.DeviceError):
                    d.execute("write",1536,512,b"C"*512)

        def test_error_ordinal_is_observed_operation_not_guest_marker(self):
            d = image(fault=plan("write","error",ordinal=2))
            d.execute("read",0,512)
            d.execute("write",1024,512,b"A"*512)
            d.execute("flush")
            with self.assertRaises(block.DeviceError):
                d.execute("write",1536,512,b"B"*512)
            self.assertEqual(d.events[-1]["ordinal"],2)
            self.assertEqual(d.events[-1]["injection"],plan("write","error",ordinal=2))
            self.assertEqual(d.frozen()[0][1024:2048],b"A"*512+bytes(512))

        def test_readonly_and_geometry_identity_fail_without_repair(self):
            d = image(readonly=True)
            before = d.frozen()
            with self.assertRaises(PermissionError):
                d.execute("write",0,512,b"X"*512)
            d.execute("flush")
            self.assertEqual(d.frozen(),before)
            for args in (("read",1,512),("read",0,0),("read",0,513),
                         ("read",0,66048),("read",d.size,512),
                         ("flush",1,0),("write",0,512,b""),("trim",0,512)):
                with self.assertRaises(ValueError):
                    d.execute(*args)
            d = image()
            os.ftruncate(d.fd,d.size-512)  # This exact cloud fixture only.
            with self.assertRaises(block.DeviceError):
                d.execute("read",0,512)
            self.assertTrue(d.failed)
            with self.assertRaises(ValueError):
                block.Disk(d.fd,"data")

        def test_plan_shape_bounds_and_never_silently_truncate_prefix(self):
            d = image()
            for fault in ({},plan("read","error"),plan("write","error",1),
                          plan("write","error",ordinal=0),plan("write","error",ordinal=True),
                          plan("write","unknown"),plan("write","torn-cut",65537)):
                with self.assertRaises(ValueError):
                    block.Disk(d.fd,"data",fault=fault)
            d = image(fault=plan("write","torn-cut",513))
            before = d.frozen()
            with self.assertRaises(block.DeviceError):
                d.execute("write",1024,512,b"A"*512)
            self.assertEqual(d.frozen(),before)

        def test_block_event_and_traffic_budgets_use_real_operations(self):
            d = image()
            for _ in range(block.MAX_EVENTS):
                d.execute("read",0,512)
            with self.assertRaises(block.DeviceError):
                d.execute("read",0,512)
            self.assertEqual(len(d.events),block.MAX_EVENTS)
            d = image(kind="system")
            for _ in range(block.MAX_TRAFFIC//65536):
                d.execute("read",0,65536)
            with self.assertRaises(block.DeviceError):
                d.execute("read",0,512)
            self.assertEqual(d.traffic,block.MAX_TRAFFIC)

        def test_exact_negotiation_flags_bounds_and_no_export_selector(self):
            n = negotiation()
            self.assertEqual(n.option(*option(8)),n.reply(8,0x80000001))
            raw = n.option(*option(7,go))
            self.assertTrue(n.ready)
            self.assertEqual(raw,n.reply(7,3,struct.pack(">HQH",0,194*512,5))+
                             n.reply(7,3,struct.pack(">HIII",3,512,512,65536))+n.reply(7,1))
            with self.assertRaises(ValueError):
                n.option(*option(7,go))
            n = negotiation(True)
            self.assertEqual(n.flags,7)
            raw = n.option(*option(7,struct.pack(">IH",0,0)))
            self.assertFalse(n.ready)
            self.assertTrue(raw.endswith(n.reply(7,0x80000008)))
            raw = n.option(*option(7,struct.pack(">I",4)+b"data"+struct.pack(">HH",1,3)))
            self.assertEqual(raw,n.reply(7,0x80000006))
            with self.assertRaises(ValueError):
                n.option(*option(1))
            self.assertFalse(n.ready)

        def test_negotiation_malformed_fields_and_budget(self):
            for flags in (0,2,4,7,0xffffffff):
                with self.assertRaises(ValueError):
                    wire.Negotiation(194*512,False).client_flags(struct.pack(">I",flags))
            n = negotiation()
            with self.assertRaises(ValueError):
                n.client_flags(struct.pack(">I",3))
            for header in (b"",bytes(16),struct.pack(">QII",wire.OPTION_MAGIC,7,4097)):
                with self.assertRaises(ValueError):
                    n.option_size(header)
            for data in (b"",bytes(5),struct.pack(">I",20)+bytes(2),
                         struct.pack(">IHHH",0,2,3,3)):
                self.assertEqual(n.option(*option(7,data)),n.reply(7,0x80000003))
            n = negotiation()
            for _ in range(wire.MAX_OPTIONS):
                n.option(*option(8))
            with self.assertRaises(ValueError):
                n.option(*option(8))
            n = negotiation()
            self.assertEqual(n.option(*option(2)),n.reply(2,1))
            self.assertTrue(n.closed)

        def test_request_and_response_full_cookie_and_boundaries(self):
            for cookie in (0,2**32,2**64-1):
                raw = packet(0,cookie,193*512,512)
                self.assertEqual(wire.request(raw,194*512),(0,cookie,193*512,512))
                self.assertEqual(wire.simple(cookie),struct.pack(">IIQ",wire.SIMPLE_MAGIC,0,cookie))
            for raw in (b"",bytes(28),packet(0,length=512,flags=1),
                        packet(4,length=512),packet(0,length=0),packet(0,length=513),
                        packet(0,length=66048),packet(0,offset=1,length=512),
                        packet(0,offset=2**64-1,length=512),
                        packet(3,length=512),packet(2,offset=512)):
                with self.assertRaises(ValueError):
                    wire.request(raw,194*512)
            for args in ((True,),(-1,),(2**64,),(0,2),(0,1,b"x"),(0,0,bytes(65537))):
                with self.assertRaises(ValueError):
                    wire.simple(*args)

        def test_real_private_socket_fragmented_write_flush_read(self):
            d = image()
            server,client = socket.socketpair(socket.AF_UNIX,socket.SOCK_STREAM)
            client.settimeout(3)
            outcome = []
            def worker():
                try:
                    outcome.append(wire.serve(server,d,time.monotonic()+15))
                except BaseException as exc:
                    outcome.append(exc)
                finally:
                    server.close()
            thread = threading.Thread(target=worker,daemon=True)
            thread.start()
            def read(size):
                data = b""
                while len(data) < size:
                    part = client.recv(size-len(data))
                    if not part:
                        raise EOFError("test peer")
                    data += part
                return data
            try:
                self.assertEqual(read(18),wire.HELLO)
                client.sendall(struct.pack(">I",3))
                header,payload = option(7,go)
                client.sendall(header+payload)
                expected = negotiation().option(header,payload)
                self.assertEqual(read(len(expected)),expected)
                raw = packet(1,2**64-1,1024,512)+b"Z"*512
                for at in range(0,len(raw),7):
                    client.sendall(raw[at:at+7])
                self.assertEqual(read(16),wire.simple(2**64-1))
                with self.assertRaises(block.DeviceError):
                    d.frozen()
                self.assertEqual(os.pread(d.fd,512,1024),bytes(512))
                client.sendall(packet(3,7))
                self.assertEqual(read(16),wire.simple(7))
                self.assertEqual(os.pread(d.fd,512,1024),b"Z"*512)
                client.sendall(packet(0,9,1024,512))
                self.assertEqual(read(528),wire.simple(9,0,b"Z"*512))
                client.sendall(packet(2))
                thread.join(3)
                self.assertFalse(thread.is_alive())
                self.assertEqual(outcome[0]["termination"],"client-disconnect")
                self.assertEqual(outcome[0]["requests"],3)
                self.assertEqual(d.frozen()[0][1024:1536],b"Z"*512)
                with self.assertRaises(block.DeviceError):
                    d.attach()
                with self.assertRaises(block.DeviceError):
                    d.execute("flush")
            finally:
                try:
                    client.shutdown(socket.SHUT_RDWR)
                except OSError:
                    pass
                client.close()
                thread.join(3)
                self.assertFalse(thread.is_alive())

    try:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(Tests)
        assert suite.countTestCases() == 13
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        if not result.wasSuccessful():
            raise SystemExit(1)
        print("Modern block backend: 13 focused tests; real disposable-file durability and private socketpair; no VM or target execution")
    finally:
        for fd in descriptors:
            os.close(fd)
        # No fixture deletion commands. Only this cloud container's normal
        # disposable tmpfs teardown retires these exclusively created files.

if __name__ == "__main__":
    main()
