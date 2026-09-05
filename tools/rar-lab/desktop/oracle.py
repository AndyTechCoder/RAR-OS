"""RAR-authored provisional visual contract and independent scene oracle.
No target source/imports are used to decide expected pixels.
"""
import re
WIDTH, HEIGHT = 640, 480
HEADER = b"P6\\n640 480\\n255\\n".replace(b"\\n", b"\n")
FONT = {"0":[14,17,19,21,25,17,14],"1":[4,12,4,4,4,4,14],"2":[14,17,1,2,4,8,31],"3":[30,1,1,14,1,1,30],"4":[2,6,10,18,31,2,2],"5":[31,16,16,30,1,1,30],"6":[14,16,16,30,17,17,14],"7":[31,1,2,4,8,8,8],"8":[14,17,17,14,17,17,14],"9":[14,17,17,15,1,1,14],"A":[14,17,17,31,17,17,17],"B":[30,17,17,30,17,17,30],"C":[15,16,16,16,16,16,15],"D":[30,17,17,17,17,17,30],"E":[31,16,16,30,16,16,31],"F":[31,16,16,30,16,16,16],"G":[15,16,16,23,17,17,15],"H":[17,17,17,31,17,17,17],"I":[31,4,4,4,4,4,31],"J":[7,2,2,2,18,18,12],"K":[17,18,20,24,20,18,17],"L":[16,16,16,16,16,16,31],"M":[17,27,21,21,17,17,17],"N":[17,25,21,19,17,17,17],"O":[14,17,17,17,17,17,14],"P":[30,17,17,30,16,16,16],"Q":[14,17,17,17,21,18,13],"R":[30,17,17,30,20,18,17],"S":[15,16,16,14,1,1,30],"T":[31,4,4,4,4,4,4],"U":[17,17,17,17,17,17,14],"V":[17,17,17,17,17,10,4],"W":[17,17,17,21,21,21,10],"X":[17,17,10,4,10,17,17],"Y":[17,17,10,4,4,4,4],"Z":[31,1,2,4,8,16,31]," ":[0,0,0,0,0,0,0],"-":[0,0,0,31,0,0,0],".":[0,0,0,0,0,6,6],":":[0,6,6,0,6,6,0],"/":[1,2,2,4,8,8,16],">":[16,8,4,2,4,8,16],"?":[14,17,1,2,4,0,4]}
SCENES = ("home","files","settings","light","hidden","terminal","written","readback","file-readback","terminal-stopped","files-survive","settings-survive")

def plan(nonce):
    if not isinstance(nonce,str) or re.fullmatch("[a-p]{8}",nonce) is None:
        raise ValueError("invalid bounded synthetic text")
    def typed(s):
        return ["spc" if c==" " else "ret" if c=="\n" else c for c in s]
    return ([],["f1"],["f2"],["spc"],["esc"],["f3"],
            typed("write note "+nonce+"x")+["backspace","ret"],typed("read note\n"),
            ["f1","down"],typed("crash\n")+["f3"],["f1"],["f2","spc"])

def scene(index,nonce):
    plan(nonce)
    if type(index) is not int or not 0<=index<len(SCENES): raise ValueError("bad scene")
    light=3<=index<=10
    order=() if index==0 else (4,) if index in (1,4) else (4,5) if index in (2,3) else (4,6) if 5<=index<=7 else (6,4) if index==8 else (4,) if index in (9,10) else (4,5)
    focus=order[-1] if order else 0
    lines={
      4:["TEMPORARY WORKSPACE","WELCOME","RAR OS ALPHA","UP/DOWN SELECT  F1 REFRESH","RAM ONLY - LOST ON STOP"],
      5:["APPEARANCE","LIGHT" if light else "DARK","SPACE TO CHANGE THEME","SESSION ONLY"],
      6:["RAR TERMINAL","HELP LIST READ WRITE CRASH","","> "],
    }
    if index>=6: lines[6]=["RAR TERMINAL","SAVED NOTE" if index==6 else "NOTE",nonce,"> "]
    if index>=8: lines[4]=["TEMPORARY WORKSPACE","NOTE",nonce,"UP/DOWN SELECT  F1 REFRESH","RAM ONLY - LOST ON STOP"]
    return light,order,focus,lines,index>=9

def expected(index,nonce):
    light,order,focus,lines,stopped=scene(index,nonce)
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
    text(454,16,"USABLE ALPHA",ink)
    text(28,60,"YOUR RAR WORKSPACE",ink,2)
    text(28,86,"F1 FILES   F2 SETTINGS   F3 TERMINAL",ink)
    text(28,104,"KEYBOARD FIRST - CLOUD DEVELOPMENT ALPHA",ink)
    text(28,126,"RAM FILES ARE TEMPORARY",ink)
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
