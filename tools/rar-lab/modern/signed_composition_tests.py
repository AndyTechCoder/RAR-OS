"""Cloud-only signed composition tests; synthetic PE bytes are never executed."""
import sys
assert sys.flags.isolated and sys.dont_write_bytecode
from pathlib import Path
import runpy
import struct
import base64
from hashlib import sha256
here=Path(__file__).resolve().parent
binary=runpy.run_path(str(here/"build_binary.py"))
signer=runpy.run_path(str(here/"lab_signer.py"))
packages=runpy.run_path(str(here/"settings_packages.py"))
composition=runpy.run_path(str(here/"signed_composition.py"))
raw=bytearray(signer["codec_test_fixture"]()[384:1408])
struct.pack_into("<H",raw,88+68,10)
valid=bytes(raw)
built=dict.fromkeys(binary["NAMES"],valid)
for i,name in enumerate(binary["SETTINGS"]):
    value=bytearray(valid);value[512]=i+1;built[name]=bytes(value)
args=(built,"a"*40,"b"*40,"sha256:"+"c"*64,binary,packages,signer["sign_manifest_digest"])
bank,system,record=composition["compose"](*args)
assert tuple(bank)==composition["NAMES"]
assert len(system)==8388608
assert system[1024:1024+len(bank[composition["NAMES"][4]])]==bank[composition["NAMES"][4]]
assert record["system_sha256"]==sha256(system).hexdigest()
assert composition["compose"](*args)==(bank,system,record)
for variant,name in zip(composition["ORDER"],composition["NAMES"]):
    source="update" if variant in ("bad-signature","bad-abi") else variant
    assert bank[name][384:]==built["modern-settings-"+source+".efi"]
    assert bank[name][176:196]==bytes.fromhex("a"*40)
    assert bank[name][196:228]==bytes.fromhex(record["build_digest"])
changed=composition["compose"](built,"d"*40,*args[2:])
assert changed[0]!=bank and changed[1]!=system
# A two-section PE with page-aligned constant objects; data only, not executable.
fakebank={name:bytes([65+i])*896 for i,name in enumerate(composition["NAMES"])}
kernel=bytearray(1024+5*4096)
kernel[:1024]=valid
struct.pack_into("<H",kernel,70,2)
struct.pack_into("<I",kernel,88+56,7*4096)
section=328+40
for offset,value in ((8,5*4096),(12,8192),(16,5*4096),(20,1024),(36,0x40000040)):
    struct.pack_into("<I",kernel,section+offset,value)
for i,value in enumerate(fakebank.values()):
    kernel[1024+i*4096:1024+i*4096+896]=value
inspect=composition["inspect_bank"]
placement=inspect(bytes(kernel),fakebank,binary["inspect"])
assert [p["rva"] for p in placement.values()]==[8192+i*4096 for i in range(5)]
rejected=0
def reject(fn):
    global rejected
    try: fn()
    except (ValueError,struct.error,KeyError,TypeError): rejected+=1
    else: raise AssertionError("invalid signed composition accepted")
for flag in (0xc0000040,0x60000020,0xe0000020):
    bad=bytearray(kernel);struct.pack_into("<I",bad,section+36,flag)
    reject(lambda bad=bytes(bad):inspect(bad,fakebank,binary["inspect"]))
for offset in (1024,1024+896,1024+4*4096+4095):
    bad=bytearray(kernel);bad[offset]^=1
    reject(lambda bad=bytes(bad):inspect(bad,fakebank,binary["inspect"]))
reject(lambda:inspect(bytes(kernel),dict(reversed(list(fakebank.items()))),binary["inspect"]))
reject(lambda:inspect(bytes(kernel),dict(fakebank,extra=b"x"*896),binary["inspect"]))
for replacement in (b"",b"x"*895,b"x"*2097537,bytearray(b"x"*896),list(fakebank.values())[0]):
    bad=dict(fakebank);bad[composition["NAMES"][1]]=replacement
    reject(lambda bad=bad:inspect(bytes(kernel),bad,binary["inspect"]))
for source in ("main","0"*40,"a"*39):
    reject(lambda source=source:composition["compose"](built,source,*args[2:]))
for ctrl,image in (("main",args[3]),("0"*40,args[3]),(args[2],"latest"),(args[2],"sha256:"+"c"*63)):
    reject(lambda ctrl=ctrl,image=image:composition["compose"](built,args[1],ctrl,image,*args[4:]))
reject(lambda:composition["compose"](dict.fromkeys(binary["NAMES"],valid),*args[1:]))
packet=b"".join(("RAR-SIGNED-FILE:"+name+"\n").encode()+base64.b64encode(value)+b"\n"
    for name,value in (("modern.efi",valid),("modern-service.efi",valid)))+b"RAR-SIGNED-BUILD:END\n"
assert composition["unpack_signed"](packet,binary["inspect"])=={"modern.efi":valid,"modern-service.efi":valid}
for bad in (b"",packet[:-1],packet+b"x",packet.replace(b"modern.efi",b"../escape",1),
            packet.replace(b"RAR-SIGNED-BUILD:END",b"RAR-BUILD:END"),b"x"*(6*1024*1024+1)):
    reject(lambda bad=bad:composition["unpack_signed"](bad,binary["inspect"]))
print("Modern signed composition:",rejected,"negative cases; real package provenance and PE bank geometry; no PE execution")
