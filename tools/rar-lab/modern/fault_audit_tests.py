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
persistence=load("persistence")

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

    def test_cut_join_binds_entry_and_final_cut_receipts(self):
        from types import SimpleNamespace as NS
        with patch.object(session.time,"monotonic",return_value=0):
            vm=self.fault_vm();data=vm.backends[0]
            p=dict(operation="write",ordinal=1,effect="before-cut",prefix=0)
            vm.fault_plan=tuple(p[k] for k in ("operation","ordinal","effect","prefix"))
            data.records[2]["event"].update(injection=p,status="cut-no-reply")
            data.records.append(dict(type="terminal",outcome="cut",fault_hit=True,failed=False))
            data.poll=lambda:20;data.problem="cut";data.eof=True
            try:vm.service()
            except session.PlannedDataFault as error:signal=error
            else:self.fail("missing cut signal")
            receipt=vm.fault_receipt(signal)
            for failure in (None,"entry","exit","terminal","qemu"):
                import copy
                rows=[copy.deepcopy(data.records)]
                for role,size,inode in (("system",8388608,3),("boot",16777216,4)):
                    rows.append([dict(type="ready",kind=role,readonly=role=="boot",
                        export_readonly=False,capacity=size,device=1,inode=inode)])
                if failure=="terminal":rows[0].pop()
                fake=NS(fault_receipt=lambda error:receipt,fault_audit=audit,
                    backends=[NS(records=r) for r in rows],argv=[],preflight={},
                    commands=[],events=[],event_receipts=[],serial=b"",
                    cleanup_succeeded=True,qmp_drained=True)
                stopped=dict(joined=True,vm_returncode=0 if failure=="qemu" else -9,
                    entry=dict(vm_code=None,backend_codes=[None if failure=="entry" else 20,None,None],
                               backend_problems=["cut",None,None]),
                    backends=[dict(joined=True,records=rows[0],returncode=21 if failure=="exit" else 20,
                        problem="cut")]+[dict(joined=True,records=r,returncode=-9,
                        problem="backend-failed") for r in rows[1:]])
                fake.destroy=lambda:stopped
                if failure is None:
                    self.assertEqual(persistence.joined_fault(fake,signal)["fault"],receipt)
                else:
                    with self.assertRaises(ValueError):persistence.joined_fault(fake,signal)

    def test_entry_snapshot_failure_still_cleans_every_owned_child(self):
        import io
        from types import SimpleNamespace as NS
        class Pipe(io.BytesIO):
            def fileno(self):return 99
        for failure in ("poll","problem"):
            trace=[]
            class Child:
                pid=1
                def __init__(self):
                    self.returncode=None;self.stdout=Pipe(b"");self.stderr=Pipe(b"")
                def poll(self):return self.returncode
                def kill(self):trace.append("kill");self.returncode=-9
                def wait(self,timeout):trace.append("wait");return self.returncode
            class Backend:
                def __init__(self,index):
                    self.index=index;self.process=NS(poll=self.poll)
                def poll(self):
                    if self.index==1 and failure=="poll":raise OSError("entry poll failed")
                    return None
                @property
                def problem(self):
                    if self.index==1 and failure=="problem":raise OSError("entry problem failed")
                    return None
                def stop(self):trace.append(self.index);return dict(joined=True)
            vm=object.__new__(session.VM)
            vm.closed=False;vm.started=False;vm.fault_plan=("write",1,"error",0)
            vm.child=Child();vm.backends=[Backend(i) for i in range(3)]
            vm.serial=bytearray();vm.profile=NS(SERIAL_LIMIT=65536);vm.boot_fd=None
            vm.connection=None;vm.sockets=[];vm.selector=NS(close=lambda:trace.append("selector"))
            with patch.object(session.os,"set_blocking",return_value=None):
                with self.assertRaises(RuntimeError):vm.destroy()
            self.assertEqual(trace,["kill","wait",0,1,2,"selector"])
            self.assertFalse(vm.cleanup_succeeded)
            self.assertTrue(vm.child.stdout.closed and vm.child.stderr.closed)

    def test_fault_stop_requires_exact_signal_then_all_joins(self):
        from types import SimpleNamespace as NS
        with patch.object(session.time,"monotonic",return_value=0):
            vm=self.fault_vm()
            try:vm.service()
            except session.PlannedDataFault as error:signal=error
            else:self.fail("missing planned signal")
            receipt=vm.fault_receipt(signal)
            self.assertEqual(receipt["delivery"],dict(code=None,problem=None,eof=False))
            with self.assertRaises(ValueError):vm.fault_receipt(RuntimeError("not planned"))
            wrong=session.PlannedDataFault(("wrong",))
            with self.assertRaises(ValueError):vm.fault_receipt(wrong)
            for failure in (None,"join","qmp","vm0","vm1","vm-bool","peer0","peer20","peer22","peer-problem","data21","entry-peer","all21"):
                trace=[];data=vm.backends[0].records
                def ready(role):
                    return dict(type="ready",kind=role,readonly=role=="boot",
                        export_readonly=False,capacity=8388608 if role=="system" else 16777216,
                        device=1,inode=3 if role=="system" else 4)
                rows=[data,[ready("system")],[ready("boot")]]
                fake=NS(fault_receipt=lambda error:receipt,
                    service=lambda:trace.append("service"),fault_audit=audit,
                    backends=[NS(records=r) for r in rows],argv=[],preflight={},
                    commands=[],events=[],event_receipts=[],serial=b"",
                    cleanup_succeeded=False,qmp_drained=False)
                def destroy():
                    trace.append("destroy")
                    fake.cleanup_succeeded=failure!="join";fake.qmp_drained=failure!="qmp"
                    result=dict(joined=failure!="join",vm_returncode=-9,
                        entry=dict(vm_code=None,backend_codes=[None,None,None],
                                   backend_problems=[None,None,None]),backends=[
                        dict(joined=True,records=r,returncode=-9,problem="backend-failed") for r in rows])
                    if failure in ("vm0","vm1","vm-bool"):
                        result["vm_returncode"]={"vm0":0,"vm1":1,"vm-bool":True}[failure]
                    if failure in ("peer0","peer20","peer22"):
                        result["backends"][1]["returncode"]={"peer0":0,"peer20":20,"peer22":22}[failure]
                    if failure=="peer-problem":result["backends"][2]["problem"]="unexpected-stderr"
                    if failure=="data21":result["backends"][0]["returncode"]=21
                    if failure=="entry-peer":result["entry"]["backend_codes"][1]=21
                    if failure=="all21":
                        for index,report in enumerate(result["backends"]):
                            report["returncode"]=21
                            rows[index].append(dict(type="terminal",outcome="failed",
                                fault_hit=index==0,failed=index==0))
                    return result
                fake.destroy=destroy
                if failure not in (None,"all21"):
                    with self.assertRaises(ValueError):persistence.joined_fault(fake,signal)
                else:self.assertEqual(persistence.joined_fault(fake,signal)["fault"],receipt)
                self.assertEqual(trace,["service","destroy"])

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
        terminated=vm.backends[0].records+[dict(type="terminal",outcome="failed",fault_hit=True,failed=True)]
        with self.assertRaises(audit.Invalid):audit.observe(terminated,p,None,None,False)
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
                    with self.assertRaises(session.PlannedDataFault):vm.request({"execute":"send-key"})
                    self.assertIsNone(vm.qmp_pending)
                    self.assertTrue(vm.fault_delivered)
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
