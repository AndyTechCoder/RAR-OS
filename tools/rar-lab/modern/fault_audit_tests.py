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
session=load("vm_session")

class Tests(unittest.TestCase):
    def test_pure_parser_and_matcher(self):
        self.assertGreater(audit.self_test(),90)

    def fault_vm(self):
        from types import SimpleNamespace as NS
        vm=object.__new__(session.VM)
        p=dict(operation="write",ordinal=1,effect="error",prefix=0)
        ready=dict(type="ready",kind="data",readonly=False,export_readonly=False,
                   capacity=99328,device=1,inode=2)
        req=dict(type="request",operation="write",offset=1024,length=512)
        event=dict(type="event",event=dict(operation="write",ordinal=1,offset=1024,
            length=512,status="failed-no-success",payload_sha256="a"*64,injection=p))
        data=NS(records=[ready,req,event],problem=None,eof=False,poll=lambda:None)
        vm.backends=[data,NS(problem=None,poll=lambda:None),NS(problem=None,poll=lambda:None)]
        vm.closed=False;vm.deadline=100;vm.child=NS(poll=lambda:None)
        vm.fault_audit=audit;vm.fault_plan=tuple(p[k] for k in ("operation","ordinal","effect","prefix"))
        vm.fault_notice=None;vm.fault_delivered=False;vm.fault_deadline=None
        vm.fault_qmp_deadline=None;vm.qmp_pending=None
        vm.selector=NS(select=lambda timeout:[])
        vm.serial=bytearray();vm.qmp_bytes=bytearray();vm.qmp_total=0
        vm.profile=NS(SERIAL_LIMIT=65536)
        vm.events=[];vm.event_receipts=[];vm.identity=0;vm.commands=[]
        vm.allowed=lambda command:True
        vm.connection=NS(settimeout=lambda value:None,setblocking=lambda value:None,
                         sendall=lambda value:None)
        return vm

    def test_cut_requires_exact_records_exit_and_complete_pipe_drain(self):
        vm=self.fault_vm()
        records=vm.backends[0].records
        p=dict(operation="write",ordinal=1,effect="before-cut",prefix=0)
        records[2]["event"].update(injection=p,status="cut-no-reply")
        term=dict(type="terminal",outcome="cut",fault_hit=True,failed=False)
        self.assertEqual(audit.observe(records,p,20,"cut",False),"waiting")
        self.assertEqual(audit.observe(records,p,None,None,False),"waiting")
        hit=audit.observe(records+[term],p,20,"cut",True)
        self.assertEqual(hit["event_index"],2)
        for rows,code,problem,eof in ((records,20,"cut",True),
                ([records[0]],20,"cut",True),(records+[term],21,"backend-failed",True),
                (records+[term],20,None,True),(records+[term],20,"malformed-record",True)):
            with self.assertRaises(audit.Invalid):audit.observe(rows,p,code,problem,eof)
        vm=self.fault_vm();p=dict(zip(("operation","ordinal","effect","prefix"),vm.fault_plan))
        for code,problem,eof in ((21,"backend-failed",True),(None,None,True),(-9,None,False)):
            with self.assertRaises(audit.Invalid):
                audit.observe(vm.backends[0].records,p,code,problem,eof)

    def test_planned_signal_is_one_shot_and_unexpected_failures_win(self):
        from types import SimpleNamespace as NS
        with patch.object(session.time,"monotonic",return_value=0):
            vm=self.fault_vm()
            with self.assertRaises(session.PlannedDataFault):vm.service()
            self.assertTrue(vm.fault_delivered)
            vm.service()  # The error backend stays live; signal is not repeated.
            for failure in ("peer","qemu","stderr","panic","qmp"):
                vm=self.fault_vm()
                if failure=="peer":vm.backends[1].poll=lambda:21
                if failure=="qemu":vm.child.poll=lambda:1
                if failure in ("stderr","panic"):
                    key=NS(fileobj=NS(fileno=lambda:99),
                           data="stderr" if failure=="stderr" else "serial")
                    vm.selector=NS(select=lambda timeout:[(key,None)])
                if failure=="qmp":vm.qmp_bytes=bytearray(b'{"error":{},"id":1}\n')
                with patch.object(session.os,"read",return_value=b"RAR-PANIC:fixture"):
                    with self.assertRaises(ValueError):vm.service()
                self.assertFalse(vm.fault_delivered)

    def test_pending_qmp_reply_is_consumed_before_fault_delivery(self):
        with patch.object(session.time,"monotonic",return_value=0):
            for bad_reply in (False,True):
                vm=self.fault_vm();data=vm.backends[0];tail=data.records[1:];data.records=data.records[:1]
                def receive():
                    if len(data.records)==1:data.records.extend(tail)
                    vm.service()  # Fault arrives while the exact command is pending.
                    return {"return":{},"id":True if bad_reply else vm.identity}
                vm.receive=receive
                if bad_reply:
                    with self.assertRaises(ValueError):vm.request({"execute":"send-key"})
                    self.assertFalse(vm.fault_delivered)
                else:
                    self.assertEqual(vm.request({"execute":"send-key"}),{})
                    self.assertIsNone(vm.qmp_pending)
                    with self.assertRaises(session.PlannedDataFault):vm.service()
                    self.assertEqual(vm.request({"execute":"screendump"}),{})
                    self.assertEqual([m["id"] for m in vm.commands],[1,2])

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
