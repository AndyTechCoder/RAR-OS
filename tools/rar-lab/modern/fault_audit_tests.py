"""Cloud-only pure receipt/emitter tests. No files, disk I/O or VM activation."""
import importlib.util
import os
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or not sys.dont_write_bytecode):
    raise SystemExit("isolated cloud tests only")

def load(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module
audit=load("fault_audit")
block=load("block_disk")
process=load("block_process")

class Tests(unittest.TestCase):
    def test_pure_parser_and_matcher(self):
        self.assertGreater(audit.self_test(),90)

    def test_backend_poll_requires_canonical_duplicate_free_records(self):
        from types import SimpleNamespace as NS
        good=audit.canonical(dict(type="ready",kind="data"))+b"\n"
        for raw,accepted in ((good,True),(b'{"type":"ready","type":"event"}\n',False),
                             (b'{"type": "ready"}\n',False),
                             (b'{"type":"ready","bad":NaN}\n',False)):
            with self.subTest(raw=raw):
                backend=object.__new__(process.Backend)
                stream=NS(fileno=lambda:99);code=[None]
                backend.closed=False;backend.deadline=10;backend.problem=None
                backend.audit=audit;backend.records=[];backend.buffer=bytearray();backend.total=0
                backend.process=NS(stderr=object(),poll=lambda:code[0],
                                   kill=lambda:code.__setitem__(0,-9))
                backend.selector=NS(select=lambda seconds:[(NS(fileobj=stream),None)],
                                    get_map=lambda:{1:stream})
                with patch.object(process.time,"monotonic",return_value=0),\
                     patch.object(process.os,"read",return_value=raw):
                    backend.poll()
                if accepted:
                    self.assertEqual(backend.records,[dict(type="ready",kind="data")])
                    self.assertIsNone(backend.problem)
                else:
                    self.assertEqual(backend.records,[])
                    self.assertEqual(backend.problem,"malformed-record")
                    self.assertEqual(code[0],-9)

    def test_real_emitter_distinguishes_completed_prefix_from_storage_failure(self):
        ready=dict(type="ready",kind="data",readonly=False,export_readonly=False,
                   capacity=99328,device=1,inode=2)
        for operation in ("write","flush"):
            for effect in ("short-error","torn-cut"):
                for broken in (False,True):
                    with self.subTest(operation=operation,effect=effect,broken=broken):
                        disk=object.__new__(block.Disk)
                        disk.failed=False;disk.cut=False;disk.served=False;disk.attached=False
                        disk.readonly=False;disk.events=[];disk.traffic=0
                        disk.counts={"read":0,"write":0,"flush":0}
                        disk.size=99328;disk.volatile=bytearray(99328);disk.dirty=set()
                        disk.reverse=False;disk.fault_hit=False
                        disk.fault=dict(operation=operation,ordinal=1,effect=effect,prefix=256)
                        rows=[ready]
                        def execute(op,offset=0,length=0,data=b""):
                            rows.append(dict(type="request",operation=op,offset=offset,length=length))
                            try:return disk.execute(op,offset,length,data)
                            finally:
                                rows.append(dict(type="event",event=dict(disk.events[-1])))
                        with patch.object(disk,"_identity",return_value=None),patch.object(disk,
                                "_persist",side_effect=OSError("underlying persistence failed") if broken else None):
                            if operation=="flush":
                                execute("write",1024,512,b"x"*512)
                            with self.assertRaises((block.Cut,OSError)):
                                if operation=="flush":execute("flush")
                                else:execute("write",1024,512,b"x"*512)
                        event=rows[-1]["event"]
                        if broken:
                            self.assertNotIn("persisted_prefix_bytes",event)
                            with self.assertRaises(audit.Invalid):audit.scan(rows,disk.fault,ready)
                        else:
                            self.assertEqual(event["persisted_prefix_bytes"],256)
                            self.assertIsNotNone(audit.scan(rows,disk.fault,ready))
                            self.assertEqual(event["status"],
                                "cut-no-reply" if effect=="torn-cut" else "failed-no-success")

if __name__=="__main__":
    unittest.main(argv=[sys.argv[0]],verbosity=2)
