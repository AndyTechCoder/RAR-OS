"""Fixed public Alpha input/scene plan, independent of guest implementation."""
import importlib.util
from pathlib import Path
def sibling(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    obj=importlib.util.module_from_spec(spec);spec.loader.exec_module(obj);return obj
def plan(mode,challenges):
    if mode not in ("first","fresh") or type(challenges) is not list or len(challenges)!=4:raise ValueError("fixed Alpha plan")
    for value in challenges:sibling("visual_oracle").value_check(value)
    if len(set(challenges))!=4:raise ValueError("independent challenges")
    doc,shared,left,right=challenges
    out=[("a",[],"home",None,False),("b",[],"home",None,False)]
    def add(peer,keys,stage,value=None,compact=False):out.append((peer,keys,stage,value,compact))
    if mode=="fresh":
        add("a",["f4"],"notes-ready",doc)
        add("a",["f7"],"notes-ready",doc,True)
        add("a",["f7"],"notes-ready",doc)
        add("a",["f6"],"home")
        add("a",["f1"],"files-shared",shared)
        return out
    add("a",["f4"],"notes-ready","")
    add("a",list(doc)+["ret"],"notes-saved",doc)
    add("a",["f7"],"notes-saved",doc,True)
    add("a",["f7"],"notes-saved",doc)
    add("a",["f6"],"home")
    add("a",["f4"],"notes-ready",doc)
    add("a",["f6"],"home")
    add("a",["f5"],"counter")
    add("a",["g"],"agent")
    add("a",["q"],"home")
    add("a",["f4"],"notes-ready",doc)
    add("a",["f6"],"home")
    add("a",["f3"],"terminal")
    add("b",["f3"],"terminal")
    keys=sibling("expansion_visual").keys
    add("a",keys("net send "+left),"queued")
    add("b",keys("net recv"),"received",left)
    add("b",keys("net send "+right),"queued")
    add("a",keys("net recv"),"received",right)
    add("a",keys("net close"),"closed")
    add("a",keys("net send "+left),"retired")
    add("a",keys("write note "+shared),"saved-shared",shared)
    add("a",["esc","f1"],"files-shared",shared)
    return out

def commands(rows,index,peer,mode,challenges,profile):
    if type(rows) is not list or not 1<=len(rows)<=1024:raise ValueError("bounded Alpha QMP")
    plain=[]
    for n,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=n:raise ValueError("contiguous QMP")
        plain.append({k:v for k,v in row.items() if k!="id"})
    prefix=[{"execute":"qmp_capabilities"}]+[v for _,v in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(prefix)]!=prefix:raise ValueError("paused preflight then sole continue")
    keys=[];scenes=set()
    for owner,part,*_ in plan(mode,challenges):
        if owner==peer:keys+=part;scenes.add(len(keys))
    capture={"execute":"screendump","arguments":{"filename":profile.directory(index)+"/frame.ppm"}}
    seen={at:0 for at in scenes};at=0
    for row in plain[len(prefix):]:
        if row==capture:
            if at not in seen or seen[at]>=24:raise ValueError("fixed capture boundary/budget")
            seen[at]+=1
        else:
            if at in seen and seen[at]==0:raise ValueError("missing causal capture")
            if at>=len(keys) or row!={"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":keys[at]}],"hold-time":50}}:raise ValueError("unexpected input")
            at+=1
    if at!=len(keys) or any(n==0 for n in seen.values()):raise ValueError("incomplete exact Alpha journey")
