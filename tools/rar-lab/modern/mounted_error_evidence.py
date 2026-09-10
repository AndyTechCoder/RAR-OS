"""Pure retained mounted-error proof, independent of the scenario producer."""
import json
from pathlib import Path
import importlib.util
def helper(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed immutable sibling")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode("ascii")+b"\n"
def input_plan():
    keys=["f3"];captures=[(0,"home",None,False),(1,"terminal",None,False)]
    for index,command in enumerate(("write note denied","list","write note denied","list")):
        keys.extend("spc" if ch==" " else ch for ch in command)
        captures.append((len(keys),"pending",command,index==0))
        keys.append("ret");captures.append((len(keys),"unavailable",None,False))
    keys.extend(("esc","f1"));captures.append((len(keys),"files",None,False))
    return keys,captures

def commands(rows,value,profile):
    if type(rows) is not list or not 1<=len(rows)<=512:raise ValueError("bounded negative QMP")
    plain=[]
    for index,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=index:
            raise ValueError("negative QMP identity")
        plain.append({key:value for key,value in row.items() if key!="id"})
    start=[{"execute":"qmp_capabilities"}]+[command for _,command in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(start)]!=start:raise ValueError("negative paused preflight first")
    keys,captures=input_plan();positions={item[0] for item in captures}
    groups={position:0 for position in positions};at=0
    frame={"execute":"screendump","arguments":{"filename":profile.directory(1)+"/frame.ppm"}}
    for row in plain[len(start):]:
        if row==frame:
            if at not in groups:raise ValueError("negative capture at wrong input boundary")
            groups[at]+=1
            if groups[at]>24:raise ValueError("negative capture polling budget")
        else:
            if at>=len(keys):raise ValueError("extra negative input")
            wanted={"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}}
            if row!=wanted:raise ValueError("unplanned negative command")
            # Observe the previous scene before advancing past its boundary.
            if at in groups and groups[at]==0:raise ValueError("missing causal prior frame")
            at+=1
    if at!=len(keys) or any(count==0 for count in groups.values()):
        raise ValueError("incomplete negative input/captures")


def validate(raw,boot_digest,firmware_sizes):
    base=helper("runtime_evidence");fault=helper("fault_evidence")
    visual=helper("visual_oracle");disk=helper("data_oracle")
    if type(raw) is not bytes or not 1<=len(raw)<=64*1024*1024:raise ValueError("bounded mounted-error evidence")
    def pairs(rows):
        result={}
        for k,v in rows:
            if k in result:raise ValueError("duplicate evidence key")
            result[k]=v
        return result
    try:doc=json.loads(raw,object_pairs_hook=pairs)
    except (ValueError,UnicodeError,RecursionError) as error:raise ValueError("invalid JSON") from error
    if canonical(doc)!=raw:raise ValueError("canonical evidence")
    fault.fields(doc,"schema status original_data_base64 initial_data_base64 frozen_data_base64 data_sha256 system_sha256 boot_sha256 frames vm_proof milestone_complete")
    fault.exact(doc["schema"],"rar-modern-mounted-error-candidate-v1")
    fault.exact(doc["status"],"observed");fault.exact(doc["milestone_complete"],False)
    fault.exact(doc["boot_sha256"],base.digest(boot_digest))
    fault.exact(doc["system_sha256"],base.sha(bytes(8388608)))
    original=base.decoded(doc["original_data_base64"],99328)
    initial=base.decoded(doc["initial_data_base64"],99328)
    frozen=base.decoded(doc["frozen_data_base64"],99328)
    state=disk.inspect(original)
    if original!=initial or initial!=frozen or any(original[1024:]) or state["files"]!={} or state["revision"]!=0:
        raise ValueError("mounted error must preserve valid empty fixture exactly")
    fault.exact(doc["data_sha256"],base.sha(frozen))
    _,plan=input_plan();frames=doc["frames"]
    if type(frames) is not list or len(frames)!=len(plan):raise ValueError("eleven causal frames")
    for frame,(_,stage,command,first) in zip(frames,plan):
        fault.fields(frame,"stage command first sha256 actual_ppm")
        fault.exact([frame["stage"],frame["command"],frame["first"]],[stage,command,first])
        pixels=base.decoded(frame["actual_ppm"],len(visual.HEADER)+640*480*3)
        fault.exact(frame["sha256"],visual.unavailable_validate(pixels,stage,command,first))
    selected={"operation":"write","ordinal":1,"effect":"error","prefix":0}
    proof=doc["vm_proof"]
    fault.vm_proof(proof,1,None,selected,firmware_sizes,command_checker=commands)
    # The common proof checks real QMP topology, lifecycle, event streams,
    # distinct role inodes, exact fault receipt and one failed publication write.
    rows=proof["cut"]["backends"][0]["records"]
    event=proof["fault"]["event_index"]
    if any(r["type"]!="terminal" for r in rows[event+1:]):raise ValueError("I/O after sticky failure")
    requests=[r for r in rows if r["type"]=="request"]
    reads=[r for r in requests if r["operation"]=="read"]
    # Mount scans every sector, then publication verifies three empty slot sectors.
    # There may also be one initial firmware sector-zero probe.
    if len(reads) not in (197,198) or [r["offset"] for r in reads[-197:]]!=list(range(0,99328,512))+[1024,1536,2048]:
        raise ValueError("one complete mounted Data scan")
    if any(r["length"]!=512 for r in reads) or (len(reads)==198 and reads[0]["offset"]!=0):
        raise ValueError("fixed mount read geometry")
    if len(requests)!=len(reads)+1:raise ValueError("one failed write and no retry/flush")
    for report in proof["cut"]["backends"][1:]:
        if any(r.get("type")=="request" and r.get("operation")!="read" for r in report["records"]):
            raise ValueError("System/boot mutation attempt")
    return dict(schema="rar-modern-mounted-error-result-v1",frames=11,fresh_vms=1,
        unchanged_data_sha256=base.sha(frozen),mount_reads=len(reads),failed_writes=1,
        post_error_requests=0,content_validated=True,provenance_validated=False,milestone_complete=False)
