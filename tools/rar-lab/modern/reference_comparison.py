"""Causal Modern crypto comparison controller component.
The trusted cloud caller supplies already-confined, inventory-approved adapter
execution and evidence retention. This module grants no image/load/VM authority.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import time

MAX_EVIDENCE=16*1024*1024
DEADLINE_SECONDS=1800

class Invalid(ValueError):pass

def _load(name):
    if sys.flags.isolated!=1 or not sys.dont_write_bytecode:
        raise Invalid("isolated no-bytecode controller required")
    path=Path(__file__).resolve().with_name(name+".py")
    if path.is_symlink() or not path.is_file() or path.stat().st_size>128*1024:
        raise Invalid("fixed bounded trusted helper")
    spec=importlib.util.spec_from_file_location("modern_comparison_"+name,path)
    module=importlib.util.module_from_spec(spec)
    # Dataclass helpers need their own fixed module name registered.
    sys.modules[spec.name]=module
    spec.loader.exec_module(module)
    return module

protocol=_load("reference_protocol")
corpus=_load("reference_corpus")

def canonical(value):
    raw=json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True,
                   allow_nan=False).encode("ascii")+b"\n"
    if len(raw)>MAX_EVIDENCE:raise Invalid("comparison evidence budget")
    return raw

def _tuple(value,implementation):
    if (type(value) is not tuple or len(value)!=4 or
        type(value[0]) is not int or value[0]!=implementation or
        type(value[1]) is not int or type(value[2]) is not bytes or
        type(value[3]) is not bytes):
        raise Invalid("exact adapter result tuple")
    return value

def _wire(run):
    return {"implementation":run[0],"exit_code":run[1],
            "stdout":run[2].hex(),"stderr":run[3].hex()}

def compare_all(execute,retain):
    """Run one fixed corpus with no retries; freeze all RAR results first.
    execute(implementation_id, immutable_request_bytes) returns the exact
    (id,exit,stdout,stderr) tuple from the reviewed bounded adapter runner.
    retain(fixed_leaf_name, immutable_bytes) durably retains evidence and
    returns its SHA256 only after successful storage, or raises. The caller
    must not acknowledge data it has not retained. Neither callback is
    selected from proposal/guest inputs.
    """
    if not callable(execute) or not callable(retain):
        raise Invalid("trusted controller callbacks")
    if (os.environ.get("GITHUB_ACTIONS")!="true" or
        os.environ.get("GITHUB_REPOSITORY")!="AndyTechCoder/RAR-OS" or
        os.environ.get("GITHUB_EVENT_NAME")!="workflow_dispatch" or
        os.environ.get("GITHUB_REF")!="refs/heads/main" or sys.platform!="linux"):
        raise Invalid("trusted-main cloud comparison only")
    return _compare(execute,retain)

def _compare(execute,retain):
    # This internal function is exercised with synthetic callbacks in cloud
    # source tests. Production entry is compare_all with the cloud guard.
    deadline=time.monotonic()+DEADLINE_SECONDS
    def budget():
        if time.monotonic()>deadline:raise TimeoutError("comparison deadline")
    def invoke(ident,request):
        budget()
        result=_tuple(execute(ident,request),ident)
        budget()
        parsed=protocol.response(request,*result)
        return result,parsed
    def save(name,value):
        budget()
        raw=canonical(value)
        digest=hashlib.sha256(raw).hexdigest()
        ack=retain(name,raw)
        if type(ack) is not str or ack!=digest:
            raise Invalid("evidence retention acknowledgement")
        budget()
        return digest
    base=corpus.cases()
    if type(base) is not tuple or len(base)!=146 or len({c.name for c in base})!=146:
        raise Invalid("exact base corpus")
    rows=[]
    for case in base:
        request=protocol.request(case.operation,case.payload)
        result,parsed=invoke(3,request)
        corpus.check_expected(case,parsed.status,parsed.value)
        rows.append((case,request,result,parsed))
    # Inputs depend only on frozen RAR seal output and original fixture data,
    # never on reference outputs. They are not accepted until all seals and
    # derived cases independently agree at the final comparison.
    derived=[]
    for case,request,result,parsed in rows:
        if case.operation==4:
            derived.extend(corpus.open_cases(case,parsed.value))
    if len(derived)!=142:raise Invalid("exact derived corpus")
    for case in derived:
        request=protocol.request(case.operation,case.payload)
        result,parsed=invoke(3,request)
        corpus.check_expected(case,parsed.status,parsed.value)
        rows.append((case,request,result,parsed))
    if len(rows)!=288 or len({row[0].name for row in rows})!=288:
        raise Invalid("complete unique comparison corpus")
    frozen={"schema":"rar-modern-frozen-comparison-v0","cases":[
        {"name":case.name,"request":request.hex(),"expected_status":case.status,
         "expected_value":None if case.value is None else case.value.hex(),
         "rar":_wire(result)}
        for case,request,result,parsed in rows]}
    frozen_hash=save("frozen-rar-results.json",frozen)
    # No reference callback occurs above this point. Retention failure aborts.
    compared=[]
    for case,request,result,parsed in rows:
        one,_=invoke(1,request)
        two,_=invoke(2,request)
        agreed=protocol.compare(request,(result,one,two))
        corpus.check_expected(case,agreed.status,agreed.value)
        compared.append({"name":case.name,"request_sha256":hashlib.sha256(request).hexdigest(),
                         "runs":[_wire(run) for run in (result,one,two)]})
    results_hash=save("three-way-results.json",
        {"schema":"rar-modern-three-way-comparison-v0","frozen_sha256":frozen_hash,
         "cases":compared})
    return {"schema":"rar-modern-comparison-result-v0","base_cases":146,
            "derived_cases":142,"compared":288,"adapter_invocations":864,
            "frozen_sha256":frozen_hash,"results_sha256":results_hash,
            "three_way_agreement":True,"milestone_complete":False}

def self_test():
    import struct
    import unittest
    def output(request,ident,status,value):
        op,_=protocol.parse_request(request)
        return (b"RARMCO00"+bytes((op,status,ident))+bytes(5)+
            struct.pack("<I",len(value))+bytes(4)+hashlib.sha256(request).digest()+
            bytes(8)+value)
    def fixture():
        # Synthetic wire data exercises orchestration, never crypto correctness.
        answers={}
        for case in corpus.cases():
            value=case.value
            if value is None:
                _,dn=struct.unpack_from("<HH",case.payload,44)
                value=bytes(dn+16)
            request=protocol.request(case.operation,case.payload)
            answers[request]=(case.status,value)
            if case.operation==4:
                for derived in corpus.open_cases(case,value):
                    answers[protocol.request(derived.operation,derived.payload)]=(derived.status,derived.value)
        events=[];saved={}
        def execute(ident,request):
            events.append(ident)
            status,value=answers[request]
            return (ident,0,output(request,ident,status,value),b"")
        def retain(name,raw):
            events.append(name)
            saved[name]=raw
            return hashlib.sha256(raw).hexdigest()
        return execute,retain,events,saved
    class Tests(unittest.TestCase):
        def test_all_rar_frozen_before_any_oracle_and_reproducible(self):
            execute,retain,events,saved=fixture()
            result=_compare(execute,retain)
            self.assertEqual(events[:288],[3]*288)
            self.assertEqual(events[288],"frozen-rar-results.json")
            self.assertEqual(events[289:-1],[1,2]*288)
            self.assertEqual(events[-1],"three-way-results.json")
            self.assertEqual(result["compared"],288)
            self.assertFalse(result["milestone_complete"])
            frozen=json.loads(saved["frozen-rar-results.json"])
            self.assertEqual(len({c["name"] for c in frozen["cases"]}),288)
            e2,r2,_,s2=fixture()
            self.assertEqual(result,_compare(e2,r2))
            self.assertEqual(saved,s2)
        def test_retention_failure_prevents_all_oracles(self):
            for bad in (None,True,"0"*64):
                execute,retain,events,saved=fixture()
                with self.assertRaises(Invalid):
                    _compare(execute,lambda name,raw:bad)
                self.assertEqual(events,[3]*288)
        def test_wrong_result_identity_process_failure_and_mismatch_stop(self):
            for kind in ("identity","exit","stderr","mismatch"):
                execute,retain,events,saved=fixture()
                def bad(ident,request):
                    value=execute(ident,request)
                    if ident!=1:return value
                    if kind=="identity":return (2,*value[1:])
                    if kind=="exit":return (1,1,value[2],b"")
                    if kind=="stderr":return (1,0,value[2],b"warning")
                    raw=bytearray(value[2]);raw[-1]^=1
                    return (1,0,bytes(raw),b"")
                with self.assertRaises(ValueError):_compare(bad,retain)
                self.assertIn("frozen-rar-results.json",saved)
                self.assertNotIn("three-way-results.json",saved)
                self.assertLessEqual(len([v for v in events if type(v) is int]),290)
        def test_target_failure_never_reaches_reference_or_retention(self):
            execute,retain,events,saved=fixture()
            def bad(ident,request):
                value=execute(ident,request)
                return (ident,1,value[2],b"")
            with self.assertRaises(ValueError):_compare(bad,retain)
            self.assertEqual(events,[3]);self.assertEqual(saved,{})
        def test_timeout_stops_without_retry(self):
            execute,retain,events,saved=fixture()
            original=time.monotonic
            ticks=iter((0,DEADLINE_SECONDS+1))
            try:
                time.monotonic=lambda:next(ticks)
                with self.assertRaises(TimeoutError):_compare(execute,retain)
            finally:time.monotonic=original
            self.assertEqual(events,[])
        def test_cloud_guard_denies_source_test_context(self):
            # Source CI is pull_request_target/push, not an activation workflow.
            original=os.environ.get("GITHUB_EVENT_NAME")
            try:
                os.environ["GITHUB_EVENT_NAME"]="pull_request_target"
                with self.assertRaises(Invalid):compare_all(lambda *_:None,lambda *_:None)
            finally:
                if original is None:os.environ.pop("GITHUB_EVENT_NAME",None)
                else:os.environ["GITHUB_EVENT_NAME"]=original
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():raise SystemExit(1)

if __name__=="__main__":
    if (sys.argv!=[sys.argv[0],"--self-test"] or os.environ.get("CI")!="true" or
        os.environ.get("GITHUB_ACTIONS")!="true" or sys.platform!="linux"):
        raise SystemExit("cloud isolated source-test entrypoint only")
    self_test()
