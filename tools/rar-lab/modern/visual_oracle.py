"""RAR-authored provisional visual contract and independent scene oracle.
No target source/imports are used to decide expected pixels.
"""
import re
WIDTH, HEIGHT = 640, 480
HEADER = b"P6\n640 480\n255\n"
FONT = {"0":[14,17,19,21,25,17,14],"1":[4,12,4,4,4,4,14],"2":[14,17,1,2,4,8,31],"3":[30,1,1,14,1,1,30],"4":[2,6,10,18,31,2,2],"5":[31,16,16,30,1,1,30],"6":[14,16,16,30,17,17,14],"7":[31,1,2,4,8,8,8],"8":[14,17,17,14,17,17,14],"9":[14,17,17,15,1,1,14],"A":[14,17,17,31,17,17,17],"B":[30,17,17,30,17,17,30],"C":[15,16,16,16,16,16,15],"D":[30,17,17,17,17,17,30],"E":[31,16,16,30,16,16,31],"F":[31,16,16,30,16,16,16],"G":[15,16,16,23,17,17,15],"H":[17,17,17,31,17,17,17],"I":[31,4,4,4,4,4,31],"J":[7,2,2,2,18,18,12],"K":[17,18,20,24,20,18,17],"L":[16,16,16,16,16,16,31],"M":[17,27,21,21,17,17,17],"N":[17,25,21,19,17,17,17],"O":[14,17,17,17,17,17,14],"P":[30,17,17,30,16,16,16],"Q":[14,17,17,17,21,18,13],"R":[30,17,17,30,20,18,17],"S":[15,16,16,14,1,1,30],"T":[31,4,4,4,4,4,4],"U":[17,17,17,17,17,17,14],"V":[17,17,17,17,17,10,4],"W":[17,17,17,21,21,21,10],"X":[17,17,10,4,10,17,17],"Y":[17,17,10,4,4,4,4],"Z":[31,1,2,4,8,16,31]," ":[0,0,0,0,0,0,0],"-":[0,0,0,31,0,0,0],".":[0,0,0,0,0,6,6],":":[0,6,6,0,6,6,0],"/":[1,2,2,4,8,8,16],">":[16,8,4,2,4,8,16],"?":[14,17,1,2,4,0,4]}

SCENES = ("home","terminal","saved","fresh-files")

def value_check(value):
    if type(value) is not str or re.fullmatch("[a-p]{32}",value) is None:
        raise ValueError("exact 128-bit public challenge encoding")
    return value

def plan(value):
    value_check(value)
    # The second VM receives only F1, never the challenge or a WRITE command.
    first = ["f3"]+["spc" if ch==" " else ch for ch in "write note "+value]+["ret"]
    return (first,["f1"])

def scene(index,nonce):
    if type(index) is not int or not 0 <= index < 4:
        raise ValueError("fixed persistence scene")
    if index in (2,3):
        value_check(nonce)
    elif nonce is not None:
        raise ValueError("no challenge is available before initial GUI proof")
    if index == 0:
        return False,(),0,{},False
    if index == 1:
        lines = {6:["RAR TERMINAL","HELP LIST READ WRITE CRASH",
            "CREATE + WRITE ARE SEPARATE COMMITS","> ","",""]}
        return False,(6,),6,lines,False
    if index == 2:
        lines = {6:["RAR TERMINAL","SAVED NOTE",nonce,"> ","",
                   "PUBLIC LAB DATA - NOT PRIVATE"]}
        return False,(6,),6,lines,False
    lines = {4:["DATA VAULT - UP/DOWN SELECT, F1 REFRESH","NOTE",
                "SELECTED: NOTE",nonce,"","PUBLIC LAB DATA - NOT PRIVATE"]}
    return False,(4,),4,lines,False

def expected(index,nonce):
    return _render_scene(scene(index,nonce))

def recovered_expected(state,value=None):
    """Independent post-cut Files layout; no target source imported."""
    if state=="written":return expected(3,value_check(value))
    if state not in ("absent","empty") or value is not None:
        raise ValueError("fixed recovered Files state")
    lines={4:["DATA VAULT - UP/DOWN SELECT, F1 REFRESH",
        "" if state=="absent" else "NOTE",
        "NO FILES" if state=="absent" else "SELECTED: NOTE",
        "","","PUBLIC LAB DATA - NOT PRIVATE"]}
    return _render_scene((False,(4,),4,lines,False))

def recovered_validate(frame,state,value=None):
    import hashlib
    if type(frame) is not bytes or frame!=recovered_expected(state,value):
        raise ValueError("recovered Files pixels differ from independent disk state")
    return hashlib.sha256(frame).hexdigest()

def _render_scene(layout):
    light,order,focus,lines,stopped=layout
    bg=(224,233,240) if light else (12,18,30)
    panel=(250,252,255) if light else (19,29,45)
    ink=(24,36,52) if light else (230,240,250)
    content=(255,255,255) if light else (24,36,52)
    accent=(44,110,160)
    pixels=bytearray(bytes(bg)*(WIDTH*HEIGHT))
    def rect(x,y,w,h,color):
        if min(x,y,w,h)<0 or x+w>WIDTH or y+h>HEIGHT: raise ValueError("oracle clipping")
        row=bytes(color)*w
        for yy in range(y,y+h): pixels[(yy*WIDTH+x)*3:(yy*WIDTH+x+w)*3]=row
    def text(x,y,value,color,scale=1):
        for i,c in enumerate(value.upper()):
            glyph=FONT.get(c,FONT["?"])
            for yy,row in enumerate(glyph):
                for xx in range(5):
                    if row&(1<<(4-xx)): rect(x+(i*6+xx)*scale,y+yy*scale,scale,scale,color)
    rect(0,0,640,40,panel); text(16,12,"RAR OS",ink,2)
    text(454,16,"MODERN ALPHA",ink)
    text(28,60,"YOUR RAR WORKSPACE",ink,2)
    text(28,86,"F1 FILES   F2 SETTINGS   F3 TERMINAL",ink)
    text(28,104,"KEYBOARD FIRST - CLOUD DEVELOPMENT ALPHA",ink)
    text(28,126,"PUBLIC LAB DATA - NOT PRIVATE",ink)
    for role in order:
        x,y={4:(24,152),5:(44,170),6:(64,188)}[role]
        # All windows are 548x232; z-order is independently composed.
        rect(x+4,y+4,548,232,(8,12,20))
        rect(x,y,548,232,content)
        rect(x,y,548,30,accent if role==focus else panel)
        text(x+12,y+8,{4:"FILES",5:"SETTINGS",6:"TERMINAL"}[role],(255,255,255) if role==focus else ink,2)
        text(x+426,y+11,"ESC CLOSE",(255,255,255) if role==focus else ink)
        for row,value in enumerate(lines[role]): text(x+14,y+48+row*28,value,ink)
    rect(0,440,640,40,panel)
    for x,label,role in [(16,"F1 FILES",4),(224,"F2 SETTINGS",5),(432,"F3 TERMINAL",6)]:
        rect(x,448,192,24,accent if role==focus else content)
        text(x+12,456,label,(255,255,255) if role==focus else ink)
    if stopped:
        rect(364,46,260,18,(160,54,54));text(374,52,"TERMINAL STOPPED - FILES SAFE",(255,255,255))
    return HEADER+pixels

def validate(frame,index,value=None):
    import hashlib
    if (type(frame) is not bytes or len(frame) != len(HEADER)+WIDTH*HEIGHT*3 or
        not frame.startswith(HEADER) or frame != expected(index,value)):
        raise ValueError("actual Modern scene differs from independent pixel expectation")
    return hashlib.sha256(frame).hexdigest()


def unavailable_expected(stage,command=None,first=False):
    """Fixed corrupt-mount scenes. Pending text makes repeated errors causal."""
    if type(first) is not bool:raise ValueError("typed pending state")
    initial=["RAR TERMINAL","HELP LIST READ WRITE CRASH",
        "CREATE + WRITE ARE SEPARATE COMMITS","> ","",""]
    failed=["STORAGE UNAVAILABLE","STORAGE UNAVAILABLE","","> ","","STORAGE UNAVAILABLE"]
    if stage=="pending":
        if command not in ("list","write note denied"):raise ValueError("fixed unavailable command")
        lines=list(initial if first else failed);lines[3]="> "+command
        return _render_scene((False,(6,),6,{6:lines},False))
    if command is not None or first:raise ValueError("no pending state for completed scene")
    if stage=="home":return expected(0,None)
    if stage=="terminal":return expected(1,None)
    if stage=="unavailable":return _render_scene((False,(6,),6,{6:failed},False))
    if stage=="files":
        lines=["STORAGE UNAVAILABLE","STORAGE UNAVAILABLE","","","","STORAGE UNAVAILABLE"]
        return _render_scene((False,(4,),4,{4:lines},False))
    raise ValueError("fixed unavailable scene")

def unavailable_validate(frame,stage,command=None,first=False):
    import hashlib
    if type(frame) is not bytes or frame!=unavailable_expected(stage,command,first):
        raise ValueError("actual unavailable scene differs")
    return hashlib.sha256(frame).hexdigest()

def self_test():
    value = "abcdefghijklmnop"*2
    frames = [expected(i,value if i>=2 else None) for i in range(4)]
    for i,frame in enumerate(frames):
        assert len(validate(frame,i,value if i>=2 else None)) == 64
    rejected = 0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected += 1
        else: raise AssertionError("invalid persistence scene accepted")
    for bad in (None,"","a"*31,"a"*33,"A"*32,"z"*32,"../"+"a"*29):
        reject(lambda bad=bad:plan(bad))
    for i,frame in enumerate(frames):
        nonce = value if i>=2 else None
        for changed in (b"",frame[:-1],frame+b"x",b"X"+frame[1:],
                        frame[:-1]+bytes([frame[-1]^1])):
            reject(lambda changed=changed,i=i,nonce=nonce:validate(changed,i,nonce))
    reject(lambda:validate(frames[2],3,value))
    reject(lambda:validate(frames[3],3,"p"*32))
    reject(lambda:expected(0,value))
    reject(lambda:expected(4,value))
    first,second = plan(value)
    assert first[0] == "f3" and first[-1] == "ret" and second == ["f1"]
    assert "".join(" " if key=="spc" else key for key in first[1:-1]) == "write note "+value
    assert len(first) == 45 and len("write note "+value) <= 64
    absent=recovered_expected("absent");empty=recovered_expected("empty")
    assert absent!=empty and empty!=frames[3] and absent!=frames[3]
    assert len(recovered_validate(absent,"absent"))==64
    assert len(recovered_validate(empty,"empty"))==64
    assert recovered_validate(frames[3],"written",value)==validate(frames[3],3,value)
    reject(lambda:recovered_validate(absent,"empty"))
    reject(lambda:recovered_validate(empty,"written",value))
    reject(lambda:recovered_expected("absent",value))
    reject(lambda:recovered_expected("unknown"))
    return rejected

if __name__ == "__main__":
    import sys
    if sys.argv != [sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure visual self-test only; no screenshot output entrypoint")
    print("Modern persistence visual oracle:",self_test(),"negative fixtures; no guest proof")
