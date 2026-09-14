"""Fixed integrated native-app/System lifecycle. No caller-selected commands."""
import importlib.util
from pathlib import Path
MODES=("install","reject","fallback","repair","final")
def load(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def plan(mode,challenges):
    if mode not in MODES:raise ValueError("fixed System lifecycle")
    # Reuse exact four distinct post-GUI challenge constraints.
    load("alpha_plan").plan("fresh",challenges)
    doc,shared,_,_=challenges;keys=load("expansion_visual").keys
    out=[("a",[],"home",None,False),("b",[],"home",None,False),
         ("a",["f4"],"notes-ready",doc,False),("a",["f6"],"home",None,False)]
    if mode in ("install","reject"):
        out.append(("a",["f3"],"terminal",None,False))
        command="update" if mode=="install" else "update badsig"
        out.append(("a",keys(command),"installed-event" if mode=="install" else "rejected-event",None,False))
        out.append(("a",["esc","f2"],"settings-new" if mode=="install" else "settings-old",None,False))
        if mode=="install":out.append(("a",["d"],"settings-new",None,True))
    elif mode=="fallback":
        out.append(("a",["f2"],"settings-new",None,False))
        out.append(("a",["x"],"fallback-event",None,False))
        out.append(("a",["f2"],"settings-old",None,False))
    else:
        out.append(("a",["f2"],"settings-old",None,False))
    out.extend([("a",["esc","f4"],"notes-ready",doc,False),
                ("a",["f7"],"notes-ready",doc,True),("a",["f7"],"notes-ready",doc,False),
                ("a",["f6","f1"],"files-shared",shared,False)])
    if mode=="final":
        out.extend([("a",["esc","f3"],"terminal",None,False),
                    ("a",keys("update"),"rejected-event",None,False),
                    ("a",["esc","f1"],"files-shared",shared,False)])
    return out
def marker(stage):
    return {"installed-event":"RAR-MODERN:UPDATE-INSTALLED",
            "rejected-event":"RAR-MODERN:UPDATE-REJECTED",
            "fallback-event":"RAR-MODERN:UPDATE-FALLBACK"}.get(stage)
def expected(stage,value=None,compact=False):
    if stage in ("settings-new","settings-old"):
        if value is not None:raise ValueError("fixed Settings scene")
        pixels=load("signed_runtime_evidence").settings_expected(load("visual_oracle"),stage=="settings-new",compact)
        return load("alpha_visual").navigation(pixels)
    if marker(stage):raise ValueError("serial event, not pixels")
    return load("alpha_visual").expected(stage,value,compact)
def validate(frame,stage,value=None,compact=False):
    import hashlib
    if frame!=expected(stage,value,compact):raise ValueError("actual integrated full pixels: "+stage)
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
    for owner,part,stage,_,_compact in plan(mode,challenges):
        if owner==peer:
            keys+=part
            if not marker(stage):scenes.add(len(keys))
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

def marker_lines(raw,marker):
    import re
    if type(raw) is not bytes or type(marker) is not str:raise ValueError("typed serial")
    return [m.start() for m in re.finditer(rb"(?m)^"+re.escape(marker.encode("ascii"))+rb"\n",raw)]
def event_receipt(receipt,serial,stage,peer,trigger):
    import hashlib
    if (type(receipt) is not dict or set(receipt)!={"peer","stage","before_bytes","before_sha256","trigger_command_id","after_bytes"} or
        receipt["peer"]!=peer or receipt["stage"]!=stage or type(receipt["trigger_command_id"]) is not int or receipt["trigger_command_id"]!=trigger):
        raise ValueError("exact event trigger receipt")
    before,after=receipt["before_bytes"],receipt["after_bytes"];wanted=marker(stage)
    if wanted is None or type(before) is not int or type(after) is not int or not 0<=before<after<=len(serial):raise ValueError("event serial bounds")
    if receipt["before_sha256"]!=hashlib.sha256(serial[:before]).hexdigest():raise ValueError("pre-trigger transcript binding")
    matches=marker_lines(serial,wanted)
    if len(matches)!=1 or not before<=matches[0] or matches[0]+len(wanted)+1>after or marker_lines(serial[:before],wanted):
        raise ValueError("one exact complete event strictly after trigger")
