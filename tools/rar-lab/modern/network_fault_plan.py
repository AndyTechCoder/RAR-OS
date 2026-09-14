"""Fixed closed-peer malformed/drop/flood/expiry keyboard plans."""
import importlib.util
from pathlib import Path
def load(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def plan(case,challenges):
    if case not in ("faults","expiry","peer-stop") or type(challenges) is not list or len(challenges)!=2:raise ValueError("fixed network case")
    for c in challenges:load("visual_oracle").value_check(c)
    if challenges[0]==challenges[1]:raise ValueError("distinct challenges")
    left,right=challenges;keys=load("expansion_visual").keys
    out=[("a",[],"home",None),("b",[],"home",None),("a",["f3"],"terminal",None),("b",["f3"],"terminal",None)]
    if case=="faults":
        # Each sender ACK means queued only. A receives nothing from the first
        # three malformed frames or the fourth deliberately dropped datagram.
        for _ in range(4):
            out.append(("b",keys("net send "+left),"queued",None))
            out.append(("a",keys("net recv"),"empty",None))
        out.append(("b",keys("net send "+left),"queued",None))
        out.append(("a",keys("net recv"),"received",left))
        # Six frames overload a four-datagram application queue; no receive
        # command is sent to A during this bounded burst.
        for _ in range(6):out.append(("b",keys("net send "+right),"queued",None))
        for _ in range(4):out.append(("a",keys("net recv"),"received",right))
        out.append(("a",keys("net recv"),"empty",None))
        out.append(("a",keys("net close"),"closed",None))
        out.append(("a",keys("net send "+left),"retired",None))
    elif case=="expiry":
        # Scenario delays only after the initial Terminal capture; the actual
        # service uses a fixed 1000-tick grant rather than a forged clock.
        out.append(("a",keys("net recv"),"retired",None))
        out.append(("b",keys("net recv"),"retired",None))
    else:
        out.append(("b",keys("crash"),"peer-stopped",None))
        out.append(("a",keys("net recv"),"empty",None))
    out.append(("a",["esc","f1"],"files",None))
    return out

def expected(stage,value=None):
    if stage=="empty":
        if value is not None:raise ValueError("empty has no challenge")
        return load("alpha_visual").navigation(load("visual_oracle")._render_scene((False,(6,),6,{6:[
            "RAR NETWORK - PUBLIC LAB PEER ONLY","NO PEER DATAGRAM","","> ","","PUBLIC LAB DATA - NOT PRIVATE"]},False)))
    if stage=="peer-stopped":raise ValueError("serial event, not a rendered screenshot")
    return load("alpha_visual").expected(stage,value)

def validate(frame,stage,value=None):
    import hashlib
    if frame!=expected(stage,value):raise ValueError("network negative actual pixels: "+stage)
    return hashlib.sha256(frame).hexdigest()

def commands(rows,index,peer,mode,challenges,profile):
    if type(rows) is not list or not 1<=len(rows)<=1024:raise ValueError("bounded Alpha QMP")
    plain=[]
    for n,row in enumerate(rows,1):
        if type(row) is not dict or type(row.get("id")) is not int or row["id"]!=n:raise ValueError("contiguous QMP")
        plain.append({k:v for k,v in row.items() if k!="id"})
    prefix=[{"execute":"qmp_capabilities"}]+[v for _,v in profile.preflight_requests()]+[{"execute":"cont"}]
    if plain[:len(prefix)]!=prefix:raise ValueError("paused preflight then sole continue")
    keys=[];scenes=set()
    for owner,part,stage,_ in plan(mode,challenges):
        if owner==peer:
            keys+=part
            if stage!="peer-stopped":scenes.add(len(keys))
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
