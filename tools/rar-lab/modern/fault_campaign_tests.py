"""Inert aggregate tests: no VM, actual crypto capture or file mutation."""
import copy
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
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud tests only")

spec=importlib.util.spec_from_file_location("fault_campaign",Path(__file__).with_name("fault_campaign.py"))
campaign=importlib.util.module_from_spec(spec);spec.loader.exec_module(campaign)

def documents():
    return [dict(case=i,initial_data_base64=str(i),initial_data_sha256=str(i),
                 challenge=str(i)) for i in range(60)]

def encode(doc):return json.dumps(doc,sort_keys=True,separators=(",",":")).encode()+b"\n"

class Tests(unittest.TestCase):
    def check(self,docs,repeat=None,clock=None):
        seen=[]
        def decoded(value,size):
            self.assertEqual(size,99328);index=int(value)
            identity=(0 if repeat=="id" and index==59 else index).to_bytes(32,"little")
            key=(0 if repeat=="key" and index==59 else index).to_bytes(32,"little")
            return bytes(32)+identity+key+bytes(99328-96)
        def validate(raw,index,*args):
            doc=json.loads(raw);seen.append(index)
            if type(doc["case"]) is not int or doc["case"]!=index:
                raise ValueError("fixed independently checked case")
            return dict(case=index,content_validated=True,provenance_validated=False)
        base=NS(decoded=decoded,sha=lambda raw:hashlib.sha256(raw).hexdigest())
        modules={"runtime_evidence":base,"fault_evidence":NS(validate=validate)}
        with patch.object(campaign,"helper",side_effect=modules.__getitem__), \
             patch.object(campaign.time,"monotonic",side_effect=clock if clock is not None else lambda:0):
            return campaign.validate([encode(doc) for doc in docs],"a"*64,(1966080,131072)),seen

    def test_complete_unique_inert_campaign(self):
        result,seen=self.check(documents())
        self.assertEqual(seen,list(range(60)))
        self.assertEqual(result["count"],60)
        self.assertEqual(result["fresh_image_ids"],60)
        self.assertEqual(result["fresh_keys"],60)
        self.assertFalse(result["provenance_validated"])
        self.assertFalse(result["milestone_complete"])

    def test_missing_duplicate_reordered_and_reused_authority_are_rejected(self):
        original=documents()
        for docs in (original[:-1],original+[original[-1]],original[1:]+original[:1],
                     original[:-1]+[original[0]]):
            with self.assertRaises(ValueError):self.check(docs)
        for key in ("initial_data_sha256","challenge"):
            docs=copy.deepcopy(original);docs[-1][key]=docs[0][key]
            with self.assertRaises(ValueError):self.check(docs)
        for repeat in ("id","key"):
            with self.assertRaises(ValueError):self.check(original,repeat)
        with self.assertRaises(TimeoutError):self.check(original,clock=iter((0,601)).__next__)

    def test_failure_does_not_skip_a_case_or_mark_completion(self):
        docs=documents();docs[10]["case"]=True
        with self.assertRaises(ValueError):self.check(docs)

if __name__=="__main__":unittest.main(verbosity=2)
