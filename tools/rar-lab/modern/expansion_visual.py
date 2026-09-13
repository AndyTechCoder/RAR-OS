"""Independent fixed network GUI expectations. No target source is imported."""
import hashlib
import importlib.util
from pathlib import Path

def visual():
    spec=importlib.util.spec_from_file_location("visual_oracle",Path(__file__).with_name("visual_oracle.py"))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def expected(stage,value=None):
    base=visual()
    if stage in ("home","terminal"):
        if value is not None:raise ValueError("no challenge at initial scene")
        return base.expected(0 if stage=="home" else 1,None)
    if stage=="files":
        if value is not None:raise ValueError("no document created")
        return base.recovered_expected("absent")
    lines=["RAR NETWORK - PUBLIC LAB PEER ONLY","","","> ","","PUBLIC LAB DATA - NOT PRIVATE"]
    if stage=="received":
        base.value_check(value)
        lines[0]="PEER DATAGRAM - UNAUTHENTICATED";lines[1]=value
    elif stage in ("queued","closed","retired"):
        if value is not None:raise ValueError("no payload on status")
        lines[1]=({"queued":"QUEUED ONCE - DELIVERY NOT CONFIRMED","closed":"NETWORK CHANNEL CLOSED",
                   "retired":"NETWORK UNAVAILABLE"})[stage]
        if stage=="retired":lines[2]="EFFECT MAY HAVE OCCURRED; CLIENT RETIRED"
    else:raise ValueError("fixed network scene")
    return base._render_scene((False,(6,),6,{6:lines},False))

def validate(frame,stage,value=None):
    if type(frame) is not bytes or frame!=expected(stage,value):
        raise ValueError("actual network pixels differ from independent expectation")
    return hashlib.sha256(frame).hexdigest()

def keys(command):
    if type(command) is not str or not 1<=len(command)<=64 or any(c not in "abcdefghijklmnopqrstuvwxyz " for c in command):
        raise ValueError("bounded fixed lowercase keyboard command")
    return ["spc" if c==" " else c for c in command]+["ret"]

def plan(peer,a,b):
    base=visual();base.value_check(a);base.value_check(b)
    if a==b:raise ValueError("independent distinct challenges")
    if peer not in ("a","b"):raise ValueError("fixed peer")
    out=["f3"];scenes={0:("home",None),1:("terminal",None)}
    commands=([("net send "+a,"queued",None),("net recv","received",b),
               ("net close","closed",None),("net send "+a,"retired",None)]
              if peer=="a" else [("net recv","received",a),("net send "+b,"queued",None)])
    for command,stage,value in commands:
        out+=keys(command);scenes[len(out)]=(stage,value)
    if peer=="a":
        out+=["esc","f1"];scenes[len(out)]=("files",None)
    return out,scenes

def self_test():
    a,b="a"*32,"b"*32
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except ValueError:rejected+=1
        else:raise AssertionError("invalid network scene accepted")
    for stage,value in (("home",None),("terminal",None),("files",None),
                        ("queued",None),("closed",None),("retired",None),("received",a)):
        frame=expected(stage,value);assert len(validate(frame,stage,value))==64
        for bad in (frame[:-1],frame+b"x",b"X"+frame[1:]):
            reject(lambda bad=bad:validate(bad,stage,value))
    reject(lambda:validate(expected("received",a),"received",b))
    reject(lambda:expected("received",None))
    reject(lambda:expected("queued",a))
    reject(lambda:plan("a",a,a))
    reject(lambda:plan("c",a,b))
    for bad in ("","A","../x","a"*65,"net\nrecv"):reject(lambda bad=bad:keys(bad))
    for peer in ("a","b"):
        commands,scenes=plan(peer,a,b)
        assert len(commands)<128 and len(scenes)==(7 if peer=="a" else 4)
        joined="".join(commands)
        assert (b if peer=="a" else a) not in joined
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure visual self-test only")
    print("Expansion network visual:",self_test(),"negative fixtures; no guest execution")
