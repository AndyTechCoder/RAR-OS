"""Pure aggregate fault-evidence checks; no VM or filesystem operations."""
import importlib.util
import json
from pathlib import Path
import time

def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed trusted helper")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def validate(captures,boot_sha256,firmware_sizes):
    # Aggregate content only: exact source/tool/container provenance is the
    # outer trusted controller's separate responsibility.
    if type(captures) is not list or len(captures)!=60:
        raise ValueError("one retained result for every fixed case")
    evidence=helper("fault_evidence");base=helper("runtime_evidence")
    total=0;seen_images=set();seen_ids=set();seen_keys=set();seen_values=set();reports=[]
    deadline=time.monotonic()+600
    for index,raw in enumerate(captures):
        if time.monotonic()>deadline:raise TimeoutError("bounded aggregate validation")
        if type(raw) is not bytes or not 1<=len(raw)<=8*1024**2:
            raise ValueError("bounded individual capture")
        total+=len(raw)
        if total>384*1024**2:raise ValueError("bounded aggregate retained content")
        checked=evidence.validate(raw,index,boot_sha256,firmware_sizes)
        # Parsing follows strict canonical/duplicate checks, never precedes them.
        doc=json.loads(raw)
        initial=base.decoded(doc["initial_data_base64"],99328)
        identity=initial[32:64];key=initial[64:96];image=doc["initial_data_sha256"]
        value=doc["challenge"]
        if (image in seen_images or identity in seen_ids or key in seen_keys or
            value in seen_values):
            raise ValueError("each writable case requires a fresh image identity/key/challenge")
        seen_images.add(image);seen_ids.add(identity);seen_keys.add(key);seen_values.add(value)
        reports.append(dict(case=index,input_sha256=base.sha(raw),content=checked))
    if time.monotonic()>deadline:raise TimeoutError("bounded aggregate validation")
    return dict(schema="rar-modern-data-fault-campaign-v0",cases=reports,count=60,
        fresh_image_ids=60,fresh_keys=60,fresh_challenges=60,
        content_validated=True,provenance_validated=False,milestone_complete=False)
