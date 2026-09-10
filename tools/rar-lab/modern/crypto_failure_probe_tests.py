"""Mock-only cloud source checks for real failure-probe controller paths."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import unittest
if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")

def load(name):
    spec=importlib.util.spec_from_file_location("probe_test_"+name,Path(__file__).with_name(name+".py"))
    value=importlib.util.module_from_spec(spec);sys.modules[spec.name]=value
    spec.loader.exec_module(value);return value
gate=load("crypto_failure_probes");real=load("reference_runner")

class Fake:
    RunFailure=real.RunFailure
    owned_container=staticmethod(real.owned_container)
    confined_container=staticmethod(real.confined_container)
    def __init__(self,mode="timeout",fault=None):
        self.mode=mode;self.fault=fault;self.calls=[];self.cid="c"*64
        self.started=False;self.killed=False;self.removed=False;self.state_override=None
    def cloud_guard(self):self.calls.append(("guard",))
    def command(self,image,ident,name):
        self.image=image;self.ident=ident;self.name=name
        return real.command(image,ident,name)
    def item(self):
        host=dict(NetworkMode="none",IpcMode="none",ReadonlyRootfs=True,Privileged=False,
            NanoCpus=1000000000,Memory=134217728,MemorySwap=134217728,PidsLimit=16,
            PublishAllPorts=False,AutoRemove=False,CapDrop=["ALL"],SecurityOpt=["no-new-privileges"],
            LogConfig=dict(Type="none",Config={}),RestartPolicy=dict(Name="no",MaximumRetryCount=0),
            Ulimits=[dict(Name="core",Soft=0,Hard=0),dict(Name="nofile",Soft=64,Hard=64)])
        if self.fault=="confinement":host["Privileged"]=True
        state=dict(Status="exited" if self.killed else "running" if self.started else "created",
            Running=self.started and not self.killed,Paused=False,Pid=0 if self.killed or not self.started else 42,
            OOMKilled=False,Restarting=False,Dead=False,Error="",ExitCode=137 if self.killed else 0)
        if self.state_override is not None:
            when,key,value=self.state_override
            if (when=="running" and self.started and not self.killed) or (when=="crash" and self.killed):
                state[key]=value
        return dict(Id=self.cid,Image=self.image,Name="/"+self.name,HostConfig=host,Mounts=[],
            Config=dict(Labels={"rar.modern.owner":self.name},User="65532:65532",
                Entrypoint=[gate.ENTRIES[self.ident]],Cmd=None,Env=["PATH=/nonexistent"],
                WorkingDir="/",OpenStdin=True),State=state)
    def exchange(self,argv,request,timeout,out_limit,err_limit):
        self.calls.append(("exchange",argv,request,timeout,out_limit))
        if "create" in argv:
            if self.fault=="ambiguous":return 1,b"",b"failure"
            return 0,(self.cid+"\n").encode(),b""
        if self.fault=="unexpected-success":return 0,b"",b""
        if "wait" in argv:
            if self.fault=="cleanup-transport":raise self.RunFailure("CLI reap/pipe cleanup unconfirmed; terminate disposable job")
            error=self.RunFailure("CLI deadline")
            error.partial_stdout=b"x" if self.fault=="timeout-stdout" else b""
            error.partial_stderr=b"x" if self.fault=="timeout-stderr" else b""
            error.stream_truncated=self.fault=="timeout-truncated"
            raise error
        self.started=True
        error=self.RunFailure("CLI output limit");error.partial_stdout=b"R"
        error.partial_stderr=b"x" if self.fault=="output-stderr" else b""
        error.stream_truncated=True
        raise error
    def control(self,args,limit=65536):
        self.calls.append(("control",args))
        if args[:2]==["container","inspect"]:return gate.canonical([self.item()])
        if args[0]=="start":self.started=True;return (self.cid+"\n").encode()
        if args[0]=="kill":self.killed=True;return (self.cid+"\n").encode()
        if args[0]=="wait":return b"137\n"
        if args[:2]==["container","rm"]:self.removed=True;return (self.cid+"\n").encode()
        if args[:2]==["container","ls"]:return (self.cid+"\n").encode() if self.fault=="cleanup" else b""
        raise AssertionError(args)

class Tests(unittest.TestCase):
    def test_each_actual_transport_path_has_scoped_cleanup(self):
        for ident in (1,2,3):
            for mode in gate.MODES:
                fake=Fake(mode);saved={}
                def retain(name,raw):saved[name]=raw
                result=gate._probe(fake,"sha256:"+"a"*64,ident,mode,retain)
                self.assertTrue(result["cleanup_confirmed"]);self.assertFalse(result["successful_crypto_result"])
                self.assertTrue(fake.removed)
                self.assertIn("before.json",saved);self.assertIn("after.json",saved)
                self.assertIn("cleanup.json",saved);self.assertIn("result.json",saved)
                rm=[call for call in fake.calls if call[0]=="control" and call[1][:2]==["container","rm"]]
                self.assertEqual(rm,[("control",["container","rm","--force",fake.cid])])
                if mode=="timeout":
                    wait=[call for call in fake.calls if call[0]=="exchange" and "wait" in call[1]][0]
                    self.assertEqual(wait[3],0.25);self.assertEqual(wait[4],128)
                if mode=="output-limit":
                    attached=[call for call in fake.calls if call[0]=="exchange" and "--attach" in call[1]][0]
                    self.assertEqual(attached[2],b"RARMCR00"+bytes((1,))+bytes(7))
                    self.assertEqual(attached[4],1)
    def test_unexpected_results_and_cleanup_fail_closed(self):
        for fault in ("unexpected-success","cleanup-transport","cleanup","confinement","ambiguous"):
            fake=Fake(fault=fault)
            with self.assertRaises((gate.Invalid,real.RunFailure)):
                gate._probe(fake,"sha256:"+"a"*64,3,"timeout",lambda *_:None)
            if fault=="ambiguous":self.assertFalse(fake.removed);self.assertFalse(fake.started)
            else:self.assertTrue(fake.removed)
            if fault=="confinement":self.assertFalse(fake.started)
    def test_stream_anomalies_are_not_expected_failures(self):
        for fault in ("timeout-stdout","timeout-stderr","timeout-truncated","output-stderr"):
            fake=Fake(fault=fault);mode="output-limit" if fault=="output-stderr" else "timeout"
            with self.assertRaises(gate.Invalid):gate._probe(fake,"sha256:"+"a"*64,3,mode,lambda *_:None)
            self.assertTrue(fake.removed)
    def test_all_running_and_crash_state_anomalies_rejected(self):
        for when,mode in (("running","timeout"),("crash","crash")):
            changes=[("Paused",True),("Restarting",True),("OOMKilled",True),
                     ("Dead",True),("Error","daemon anomaly"),("Pid",True)]
            changes+=([("Running",1),("Status","exited")] if when=="running" else
                      [("Running",0),("Status","running"),("ExitCode",True),("Pid",False)])
            for key,value in changes:
                fake=Fake(mode);fake.state_override=(when,key,value)
                with self.subTest(when=when,key=key),self.assertRaises(gate.Invalid):
                    gate._probe(fake,"sha256:"+"a"*64,3,mode,lambda *_:None)
                self.assertTrue(fake.removed)

    def test_retention_failure_after_ownership_still_cleans(self):
        fake=Fake()
        def fail(name,raw):
            if name=="before.json":raise ValueError("retention failure")
        with self.assertRaises(ValueError):gate._probe(fake,"sha256:"+"a"*64,3,"timeout",fail)
        self.assertTrue(fake.removed);self.assertFalse(fake.started)
    def test_run_exact_nine_guard_and_acknowledgement(self):
        from unittest import mock
        fake=Fake();calls=[]
        def probe(transport,image,ident,mode,retain):
            calls.append((ident,mode,image))
            retain("result.json",b"{}\n")
            return dict(cleanup_confirmed=True)
        def retain(name,raw):return hashlib.sha256(raw).hexdigest()
        with mock.patch.object(gate,"_probe",side_effect=probe):
            result=gate.run(fake,"sha256:"+"a"*64,"sha256:"+"b"*64,retain)
            self.assertEqual(result["expected_failures"],9)
            self.assertEqual([(i,m) for i,m,_ in calls],[(i,m) for i in (1,2,3) for m in gate.MODES])
            self.assertEqual(fake.calls[0],("guard",))
            with self.assertRaises(gate.Invalid):
                gate.run(fake,"sha256:"+"a"*64,"sha256:"+"b"*64,lambda *_:"bad")
        with mock.patch.object(fake,"cloud_guard",side_effect=real.RunFailure("host refused")):
            with self.assertRaises(real.RunFailure):
                gate.run(fake,"sha256:"+"a"*64,"sha256:"+"b"*64,retain)

if __name__=="__main__":unittest.main(argv=[sys.argv[0]],verbosity=2)
