"""Inert controller callback tests. No Docker, VM, files or network."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
from types import SimpleNamespace as NS
import unittest
from unittest.mock import patch

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud source tests only")
spec=importlib.util.spec_from_file_location("campaign_controller",Path(__file__).with_name("fault_campaign_controller.py"))
controller=importlib.util.module_from_spec(spec);spec.loader.exec_module(controller)

class Tests(unittest.TestCase):
    def test_fixed_launcher_arguments_grant_no_path_or_plan_authority(self):
        spec=importlib.util.spec_from_file_location("fault_launch_test",Path(__file__).with_name("fault_launch.py"))
        launch=importlib.util.module_from_spec(spec);spec.loader.exec_module(launch)
        for index in range(60):
            self.assertEqual(launch.case_argument(["fixed",str(index)]),index)
        for value in ("-1","60","00","01","+1","1.0"," 1","1 ","--help","/tmp/anything","1\\n",True,1):
            with self.assertRaises(ValueError):launch.case_argument(["fixed",value])
        for argv in (None,(),{},["fixed"],["fixed","0","extra"]):
            with self.assertRaises(ValueError):launch.case_argument(argv)

    def scenario(self,fault=None):
        events=[];report={"status":"started","milestone_complete":False}
        retained={};checked=[];calls=[]
        def execute(tool,entry,mounts,seconds,limit):
            index=len(calls);calls.append(index)
            self.assertEqual(tool,"immutable-launcher")
            self.assertEqual(entry,["/usr/bin/python3","-I","-B","/opt/rar-modern/fault_launch.py",str(index)])
            self.assertEqual(mounts,[("inputs","/artifact")])
            self.assertTrue(1<=seconds<=300);self.assertEqual(limit,8*1024**2)
            events.append(("execute",index))
            if fault=="execute" and index==3:raise RuntimeError("inert execution failure")
            doc=dict(case=index,initial_data_base64=str(index),initial_data_sha256=str(index),challenge=str(index))
            if fault=="duplicate" and index==3:doc["challenge"]="0"
            if fault=="bytes" and index==3:return "not bytes"
            return json.dumps(doc).encode()
        def validate(raw,index,boot,sizes):
            events.append(("validate",index));self.assertEqual(boot,"a"*64)
            self.assertEqual(sizes,(1966080,131072))
            self.assertIn("fault-"+str(index).zfill(2)+".json",retained)
            if fault=="content" and index==3:raise ValueError("invalid actual envelope")
            return {"case":index,"content_validated":True}
        def decoded(value,size):
            index=int(value);self.assertEqual(size,99328)
            return bytes(32)+index.to_bytes(32,"little")+(index+1).to_bytes(32,"little")+bytes(99328-96)
        def aggregate(captures,boot,sizes):
            checked.extend(captures)
            self.assertEqual(len(captures),60);self.assertEqual(len(report["fault_cases"]),60)
            if fault=="aggregate":raise ValueError("aggregate failure")
            return {"count":60,"content_validated":True,"provenance_validated":False,"milestone_complete":False}
        modules={"fault_evidence":NS(validate=validate),"fault_campaign":NS(validate=aggregate),
                 "runtime_evidence":NS(decoded=decoded,sha=lambda raw:hashlib.sha256(raw).hexdigest())}
        def retain(name,raw):
            self.assertNotIn(name,retained);retained[name]=raw
        clock=iter((0,2401)).__next__ if fault=="deadline" else lambda:0
        with patch.object(controller,"helper",side_effect=modules.__getitem__), \
             patch.object(controller.time,"monotonic",side_effect=clock),patch("builtins.print") as printed:
            if fault:
                with self.assertRaises((ValueError,RuntimeError,TimeoutError)):
                    controller.observe(execute,"immutable-launcher","inputs","a"*64,
                        (1966080,131072),retain,lambda:None,report)
                printed.assert_not_called()
                self.assertEqual(report["status"],"started")
                self.assertNotIn("fault_campaign",report)
                self.assertFalse(report["milestone_complete"])
                if fault=="deadline":self.assertEqual(calls,[])
                elif fault!="aggregate":self.assertEqual(calls,[0,1,2,3])
                if fault=="content":self.assertIn("fault-03.json",retained)
            else:
                controller.observe(execute,"immutable-launcher","inputs","a"*64,
                    (1966080,131072),retain,lambda:None,report)
                self.assertEqual(calls,list(range(60)))
                self.assertEqual(len(retained),60);self.assertEqual(len(checked),60)
                self.assertEqual(report["status"],"fault-campaign-observed")
                self.assertFalse(report["milestone_complete"])
                self.assertFalse(report["fault_campaign"]["provenance_validated"])
                self.assertFalse(report["crypto_interoperability_accepted"])
                printed.assert_called_once()
                self.assertEqual(events,[(kind,i) for i in range(60) for kind in ("execute","validate")])

    def test_all_sixty_cases_in_separate_fixed_execute_calls(self):self.scenario()
    def test_failures_stop_without_retry_or_false_completion(self):
        for fault in ("execute","content","bytes","duplicate","aggregate","deadline"):
            with self.subTest(fault=fault):self.scenario(fault)

if __name__=="__main__":unittest.main(verbosity=2)
