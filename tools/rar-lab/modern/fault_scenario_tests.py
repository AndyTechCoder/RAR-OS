"""Inert campaign orchestration tests; no files, target or VM execution."""
import copy
import importlib.util
import os
from pathlib import Path
import sys
from types import SimpleNamespace as NS
import unittest
from unittest.mock import patch

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):
    raise SystemExit("isolated cloud tests only")

def load(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module
scenario=load("fault_scenarios")
visual=load("visual_oracle")

class Tests(unittest.TestCase):
    def test_fixed_case_geometry(self):
        self.assertEqual(len(scenario.cases()),60)
        self.assertEqual(len({tuple(sorted(x.items())) for x in scenario.cases()}),60)
        for bad in (-1,60,True,False,None,"1",1.0):
            with self.assertRaises(ValueError):scenario.selected(bad)
        for index in range(60):
            p,reverse=scenario.selected(index)
            self.assertIs(type(reverse),bool)
            self.assertEqual(set(p),{"operation","ordinal","effect","prefix"})
            self.assertIn(p["ordinal"],range(1,7))
            self.assertEqual(p["prefix"],255 if p["effect"] in ("torn-cut","short-error") else 0)
            old=0 if p["ordinal"]<=3 else 1
            publishes=p["ordinal"] in (3,6) and (
                p["effect"] in ("torn-cut","short-error") or
                p["operation"]=="flush" and p["effect"]=="after-cut")
            self.assertEqual(scenario.expected_revision(p),old+int(publishes))
        p,_=scenario.selected(0);p["ordinal"]=999
        self.assertEqual(scenario.selected(0)[0]["ordinal"],1)

    def test_exact_slot_classification(self):
        for index in range(60):
            p,_=scenario.selected(index)
            result=scenario.expected_state(p)
            slot=(p["ordinal"]-1)//3
            new=scenario.expected_revision(p)>slot
            reservation=p["ordinal"] in (1,4)
            virgin=reservation and (p["effect"] in ("before-cut","error") or
                p["operation"]=="write" and p["effect"]=="after-cut")
            self.assertEqual(result,dict(revision=slot+int(new),
                committed_slots=list(range(slot+int(new))),
                burned_slots=[] if new or virgin else [slot],
                next_slot=slot+int(new or not virgin),readonly=False))

    def test_actual_visual_contract(self):
        self.assertGreater(visual.self_test(),30)

    def exercise(self,index=0,failure=None):
        trace=[];vms=[];fixtures=[]
        initial=bytes(99328);frozen=initial[:1024]+b"x"+initial[1025:]
        plan,_=scenario.selected(index);revision=scenario.expected_revision(plan)
        value="a"*32
        class Signal(RuntimeError):pass
        class VM:
            def __init__(self,number,*args,**kwargs):
                self.number=number;self.closed=False;self.deadline=100
                trace.append(("vm",number,args,kwargs));vms.append(self)
            def start(self):trace.append(("start",self.number))
            def key(self,key):
                trace.append(("key",self.number,key))
                if self.number==1 and key=="ret":
                    if failure=="unexpected":raise RuntimeError("unplanned failure")
                    raise Signal("planned")
            def service(self):raise AssertionError("test signal was not consumed")
            def destroy(self):
                trace.append(("destroy",self.number));self.closed=True
        class Fixture:
            def __init__(self,root,role,data):
                self.role=role;self.fd=10 if role=="data" else 11
                self.observer=20 if role=="data" else 21
                self.initial="system" if role=="system" else "initial"
                fixtures.append(self);trace.append(("fixture",role))
            def freeze(self,owned):
                if not owned or not all(vm.closed for vm in owned):
                    raise AssertionError("freeze before whole VM join")
                trace.append(("freeze",self.role,len(owned)))
                if self.role=="system":return b"system"
                return frozen
            def close(self):trace.append(("close",self.role))
        def joined_fault(vm,error):
            self.assertIs(type(error),Signal);trace.append(("fault",vm.number))
            vm.destroy()
            return {"fault":{"plan":dict(plan)}}
        def joined(vm):
            trace.append(("join",vm.number));vm.destroy();return {"joined":True}
        def inspect(raw):
            trace.append(("inspect","initial" if raw is initial else "frozen"))
            if raw is initial:return {"revision":0,"files":{}}
            files=({}, {b"note":b""}, {b"note":value.encode()})[revision]
            result=dict(scenario.expected_state(plan),files=files)
            if failure=="disk":result["revision"]+=1
            if failure=="slots":result["burned_slots"]=[63]
            return result
        persistence=NS(canonical_entry=lambda value:__import__("json").dumps(value,sort_keys=True),
            Fixture=Fixture,sha=lambda raw:"system" if raw==b"system" else "digest",
            ready=lambda vm:trace.append(("ready",vm.number)),
            scene=lambda vm,*args:{"scene":"inert"},
            challenge=lambda entropy:value,joined_fault=joined_fault,joined=joined)
        oracle=NS(plan=lambda value:(["f3","ret"],["f1"]))
        modules={"persistence":persistence,"visual_oracle":oracle,
            "data_oracle":NS(inspect=inspect),
            "data_provision":NS(Provisioner=lambda:NS(fresh=lambda entropy:initial))}
        session=NS(cloud_guard=lambda:trace.append(("guard",)),load=modules.__getitem__,
            read_regular=lambda *args,**kwargs:b"boot",VM=VM,PlannedDataFault=Signal)
        def recovered(vm,oracle,state,value):
            self.assertEqual(vm.number,2)
            self.assertEqual(state,("absent","empty","written")[revision])
            trace.append(("recovered",state));return {"scene":state}
        with patch.object(scenario.Path,"mkdir",lambda *args,**kwargs:trace.append(("mkdir",))), \
             patch.object(scenario.os,"urandom",lambda size:bytes(size)), \
             patch.object(scenario,"recovered_scene",side_effect=recovered):
            if failure:
                with self.assertRaises((RuntimeError,ValueError)):scenario.run(session,index)
                result=None
            else:result=scenario.run(session,index)
        self.assertTrue(all(vm.closed for vm in vms))
        self.assertEqual([x for x in trace if x[0]=="close"],[("close","system"),("close","data")])
        return trace,result

    def test_all_fixed_flows_are_two_joined_vms_and_readonly_reboot(self):
        for index in range(60):
            trace,result=self.exercise(index)
            self.assertEqual(result["status"],"observed-not-independently-accepted")
            self.assertIs(result["milestone_complete"],False)
            creates=[x for x in trace if x[0]=="vm"]
            self.assertEqual([x[1] for x in creates],[1,2])
            self.assertEqual(creates[1][2],(20,11))
            self.assertEqual(creates[1][3],{"readonly_data":True})
            self.assertEqual([x[2] for x in trace if x[:2]==("key",2)],["f1"])
            self.assertLess(trace.index(("destroy",1)),trace.index(("freeze","data",1)))
            self.assertLess(trace.index(("inspect","frozen")),trace.index(creates[1]))

    def test_unplanned_failure_or_bad_disk_never_starts_reboot(self):
        for failure in ("unexpected","disk","slots"):
            trace,_=self.exercise(failure=failure)
            self.assertEqual([x[1] for x in trace if x[0]=="vm"],[1])
            self.assertNotIn(("join",1),trace)
            if failure=="unexpected":
                self.assertFalse(any(x[0]=="freeze" for x in trace))

if __name__=="__main__":
    unittest.main(verbosity=2)
