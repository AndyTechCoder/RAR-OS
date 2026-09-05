"""Bounded trusted-cloud Desktop-v0 input and multi-scene evidence protocol."""
import base64
import hashlib
import json
from pathlib import Path
import re
oracle={}
exec(compile((Path(__file__).resolve().parent/"oracle.py").read_text(),"desktop-oracle.py","exec"),oracle)
WIDTH,HEIGHT=640,480
FRAME_HEADER=oracle["HEADER"]
FRAME_BYTES=len(FRAME_HEADER)+WIDTH*HEIGHT*3
SERIAL_LIMIT=65536
RESULT_LIMIT=25165824
FRAME_PATH="/tmp/rar-desktop/frame.ppm"
SOCKET_PATH="/tmp/rar-desktop/qmp.sock"
RECORDS=("RAR-BOOT:UEFI","RAR-KERNEL:ENTRY","RAR-MEMORY:READY","RAR-ALLOCATOR:READY",
"RAR-INTERRUPTS:READY","RAR-TIMER:READY","RAR-FOUNDATION-READY",
"RAR-DESKTOP:PROCESSES-READY","RAR-DESKTOP-READY","RAR-DESKTOP:APP-FAULT=6")
def unique_pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            raise ValueError("duplicate JSON field")
        value[key] = item
    return value

def serial_records(serial):
    if not isinstance(serial, bytes) or len(serial) > SERIAL_LIMIT:
        raise ValueError("serial output outside bound")
    found = []
    for raw in serial.replace(b"\r\n", b"\n").split(b"\n"):
        if b"RAR-" not in raw:
            continue
        if not raw.startswith(b"RAR-") or len(raw) > 96:
            raise ValueError("embedded or oversized RAR marker")
        try:
            value = raw.decode("ascii")
        except UnicodeDecodeError as error:
            raise ValueError("non-ASCII RAR marker") from error
        if not all(33 <= ord(c) <= 126 for c in value):
            raise ValueError("invalid RAR record grammar")
        found.append(value)
    return found

def validate_serial(serial):
    records = serial_records(serial)
    if records != list(RECORDS):
        raise ValueError("missing, duplicated, reordered or unexpected proof")
    return records

def decode_canonical(value, bound):
    if not isinstance(value, str) or len(value) > ((bound + 2) // 3) * 4:
        raise ValueError("encoded value outside bound")
    try:
        decoded = base64.b64decode(value, validate=True)
    except (ValueError, TypeError) as error:
        raise ValueError("invalid base64 evidence") from error
    if len(decoded) > bound or base64.b64encode(decoded).decode("ascii") != value:
        raise ValueError("noncanonical evidence encoding")
    return decoded


def qmp_request(operation, identity, nonce, scene=0, key=0):
    if type(identity) is not int or not 1<=identity<=512:
        raise ValueError("bounded QMP identity")
    plan=oracle["plan"](nonce)
    if type(scene) is not int or not 0<=scene<len(plan):
        raise ValueError("invalid scene")
    if operation=="capabilities":
        request={"execute":"qmp_capabilities"}
    elif operation=="key":
        if type(key) is not int or not 0<=key<len(plan[scene]):
            raise ValueError("invalid fixed key position")
        request={"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":plan[scene][key]}],"hold-time":50}}
    elif operation=="capture":
        request={"execute":"screendump","arguments":{"filename":FRAME_PATH}}
    elif operation=="quit":
        request={"execute":"quit"}
    else:
        raise ValueError("unapproved QMP operation")
    request["id"]=identity
    return request

def validate_frame(frame,index,nonce):
    if not isinstance(frame,bytes) or len(frame)!=FRAME_BYTES or not frame.startswith(FRAME_HEADER):
        raise ValueError("capture must be exact bounded 640x480 P6")
    if frame!=oracle["expected"](index,nonce):
        raise ValueError("guest scene pixels/text differ from independent expectation")
    return hashlib.sha256(frame).hexdigest()

def validate_result(raw):
    if not isinstance(raw,bytes) or len(raw)>RESULT_LIMIT:
        raise ValueError("launch result outside bound")
    try:
        result=json.loads(raw,object_pairs_hook=unique_pairs)
    except (UnicodeDecodeError,json.JSONDecodeError,RecursionError) as error:
        raise ValueError("invalid launch result") from error
    if not isinstance(result,dict) or set(result)!={"serial_b64","frames_b64","qemu_exit","nonce","injected_keys","frame_sha256"}:
        raise ValueError("unexpected result fields")
    nonce=result["nonce"]
    plan=oracle["plan"](nonce)
    if result["injected_keys"]!=[k for stage in plan for k in stage]:
        raise ValueError("input sequence mismatch")
    if type(result["qemu_exit"]) is not int or result["qemu_exit"]!=0:
        raise ValueError("VM did not complete trusted QMP quit")
    encoded=result["frames_b64"]
    if not isinstance(encoded,list) or len(encoded)!=len(oracle["SCENES"]):
        raise ValueError("missing/extra scene")
    frames=[decode_canonical(f,FRAME_BYTES) for f in encoded]
    hashes=[validate_frame(f,i,nonce) for i,f in enumerate(frames)]
    if result["frame_sha256"]!=hashes:
        raise ValueError("scene hash mismatch")
    serial=decode_canonical(result["serial_b64"],SERIAL_LIMIT)
    return serial,frames,validate_serial(serial),nonce

def self_test():
    nonce="abcdefghijkl"[:8]
    frames=[oracle["expected"](i,nonce) for i in range(len(oracle["SCENES"]))]
    serial=("\n".join(RECORDS)+"\n").encode()
    value=dict(serial_b64=base64.b64encode(serial).decode(),
        frames_b64=[base64.b64encode(f).decode() for f in frames],qemu_exit=0,nonce=nonce,
        injected_keys=[k for stage in oracle["plan"](nonce) for k in stage],
        frame_sha256=[hashlib.sha256(f).hexdigest() for f in frames])
    assert validate_result(json.dumps(value).encode())[2]==list(RECORDS)
    rejected=0
    def reject(fn,*args):
        nonlocal rejected
        try: fn(*args)
        except (ValueError,TypeError,KeyError): rejected+=1
        else: raise AssertionError("invalid evidence accepted")
    for item in ("","a"*9,"../../xx","12345678",None):
        reject(oracle["plan"],item)
    for item in (b"",serial[:-len(RECORDS[-1])-1],serial+serial,
                 b"spoof"+serial,serial.replace(b"RAR-DESKTOP-READY",b"RAR-PANIC:BEGIN")):
        reject(validate_serial,item)
    for item in (b"",frames[0][:-1],frames[0]+b"x",frames[1]):
        reject(validate_frame,item,0,nonce)
    changed=bytearray(frames[0]);changed[-1]^=1
    reject(validate_frame,bytes(changed),0,nonce)
    for args in (("system_reset",1,nonce),("key",1,nonce,0,0),
                 ("capture",0,nonce),("capture",513,nonce),("capture",True,nonce),
                 ("key",1,nonce,1,99),("key",1,nonce,-1,0)):
        reject(qmp_request,*args)
    for operation in ("capabilities","capture","quit"):
        assert qmp_request(operation,1,nonce)["execute"] in ("qmp_capabilities","screendump","quit")
    assert qmp_request("capture",2,nonce)["arguments"]=={"filename":FRAME_PATH}
    assert qmp_request("key",3,nonce,1,0)["arguments"]["keys"]==[{"type":"qcode","data":"f1"}]
    for key,item in (("qemu_exit",124),("qemu_exit",True),("nonce","12345678"),
                     ("frames_b64",value["frames_b64"][:-1]),("injected_keys",[]),
                     ("frame_sha256",["0"*64]*len(frames))):
        bad=dict(value);bad[key]=item
        reject(validate_result,json.dumps(bad).encode())
    reject(validate_result,b'{"qemu_exit":0,"qemu_exit":124}')
    reject(validate_result,b"["*2000)
    reject(validate_result,b"x"*(RESULT_LIMIT+1))
    focus=0
    typed=""
    crashed=False
    for index,keys in enumerate(oracle["plan"](nonce)):
        for key in keys:
            if key in ("f1","f2","f3"):
                focus={"f1":4,"f2":5,"f3":6}[key]
                if focus==6 and crashed: focus=4
            elif key=="esc": focus=4
            elif key=="down": assert focus==4
            elif focus==5: assert key=="spc"
            else:
                assert focus==6, "text must reach Terminal, never another focused app"
                if key=="backspace": typed=typed[:-1]
                elif key=="ret":
                    assert typed=={6:"write note "+nonce,7:"read note",9:"crash"}[index]
                    if typed=="crash": crashed=True
                    typed=""
                else: typed+=" " if key=="spc" else key
        assert focus==oracle["scene"](index,nonce)[2]
    assert crashed and oracle["plan"](nonce)[9][-1]=="f3"
    assert len(oracle["plan"](nonce))<=16
    assert sum(map(len,oracle["plan"](nonce)))<=256
    return rejected
