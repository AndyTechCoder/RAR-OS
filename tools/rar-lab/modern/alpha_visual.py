"""Independent Alpha app/profile pixels. No target sources or target execution."""
import hashlib
import importlib.util
from pathlib import Path

def sibling(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+".py"))
    obj=importlib.util.module_from_spec(spec);spec.loader.exec_module(obj);return obj

def navigation(frame):
    base=sibling("visual_oracle")
    if type(frame) is not bytes or not frame.startswith(base.HEADER) or len(frame)!=len(base.HEADER)+640*480*3:
        raise ValueError("fixed full framebuffer")
    pixels=bytearray(frame[len(base.HEADER):])
    rect,text=canvas(pixels,base.FONT)
    rect(28,104,420,7,(12,18,30))
    text(28,104,"F4 NOTES F5 COUNTER F6 CLOSE F7 COMPACT/WIDE",(230,240,250))
    return base.HEADER+pixels

def canvas(pixels,font):
    def rect(x,y,w,h,c):
        if min(x,y,w,h)<0 or x+w>640 or y+h>480:raise ValueError("bounded oracle geometry")
        for yy in range(y,y+h):pixels[(yy*640+x)*3:(yy*640+x+w)*3]=bytes(c)*w
    def text(x,y,value,c,scale=1):
        for i,char in enumerate(value[:48].upper()):
            for yy,row in enumerate(font.get(char,font["?"])):
                for xx in range(5):
                    if row&(1<<(4-xx)):rect(x+(i*6+xx)*scale,y+yy*scale,scale,scale,c)
    return rect,text

def expected(stage,value=None,compact=False):
    if type(compact) is not bool:raise ValueError("typed profile")
    base=sibling("visual_oracle")
    if stage in ("notes-ready","notes-saved","counter","agent"):
        if stage.startswith("notes"):
            if value!="":base.value_check(value)
            rows=["RAR NOTES / INDEPENDENT RUST APP",
                  "SAVED" if stage=="notes-saved" else "READY",value[:48],value[48:],
                  "ENTER: SAVE   ESC: LOAD   BACKSPACE: EDIT",
                  "PRIVATE DOCUMENT / NO DISK OR NETWORK ACCESS"]
            title="NOTES / RUST"
        else:
            if value is not None:raise ValueError("fixed counter scenario")
            rows=["RAR COUNTER / INDEPENDENT C APP",
                  "0001" if stage=="agent" else "0000",
                  "PLUS/MINUS: CHANGE   0:RESET   Q:FAULT",
                  "UI ONLY / DEVICE AND STORAGE ACCESS DENIED",
                  "AGENT: 1 ALLOWED / 2 DENIED" if stage=="agent" else "G: RUN SCOPED AGENT DEMONSTRATION",
                  "DETERMINISTIC TEST PROVIDER / NOT PAL"]
            title="COUNTER / C"
        result=navigation(base.expected(0,None));pixels=bytearray(result[len(base.HEADER):])
        rect,text=canvas(pixels,base.FONT);x,width=(160,320) if compact else (44,548)
        rect(x+4,166,width,260,(8,12,20));rect(x,162,width,260,(24,36,52))
        rect(x,162,width,30,(44,110,160));text(x+12,170,title,(255,255,255),1 if compact else 2)
        text(x+width-70,173,"F6 CLOSE",(255,255,255))
        for row,line in enumerate(rows):text(x+14,210+row*30,line,(230,240,250))
        return base.HEADER+pixels
    if compact:raise ValueError("profile only on native app scenes")
    if stage=="saved-shared":return navigation(base.expected(2,value))
    if stage=="files-shared":return navigation(base.expected(3,value))
    return navigation(sibling("expansion_visual").expected(stage,value))

def validate(frame,stage,value=None,compact=False):
    if type(frame) is not bytes or frame!=expected(stage,value,compact):
        raise ValueError("actual Alpha full pixels differ: "+stage)
    return hashlib.sha256(frame).hexdigest()

def self_test():
    rejected=0
    for stage,value in (("home",None),("notes-ready",""),("notes-ready","a"*32),
                         ("notes-saved","b"*32),("counter",None),("agent",None),
                         ("received","c"*32),("files-shared","d"*32)):
        modes=(False,True) if stage in ("notes-ready","notes-saved","counter","agent") else (False,)
        for compact in modes:
            frame=expected(stage,value,compact);validate(frame,stage,value,compact)
            for bad in (frame[:-1],frame+b"x",frame[:-1]+bytes([frame[-1]^1])):
                try:validate(bad,stage,value,compact)
                except ValueError:rejected+=1
                else:raise AssertionError("changed frame accepted")
    assert expected("notes-ready","a"*32)!=expected("notes-ready","a"*32,True)
    assert expected("counter")!=expected("agent")
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure self-test only")
    print("Alpha visual:",self_test(),"negative fixtures; no target execution")
