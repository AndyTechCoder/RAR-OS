"""Independent restricted-profile wire/PCAP oracle. Pure bytes; never a NIC or target."""
import hashlib
import struct

HEADER=struct.pack("<IHHIIII",0xa1b2c3d4,2,4,0,0,554,1)
def checksum(data):
    data=data+(b"\0" if len(data)%2 else b"")
    total=sum(struct.unpack("!"+"H"*(len(data)//2),data))
    while total>>16:total=(total&65535)+(total>>16)
    return total^65535

def packet(peer,payload,identifier=0):
    if peer not in ("a","b") or type(payload) is not bytes or not 1<=len(payload)<=512:
        raise ValueError("fixed peer and bounded bytes")
    if type(identifier) is not int or not 0<=identifier<=65535:raise ValueError("IPv4 identifier")
    source,dest=(1,2) if peer=="a" else (2,1)
    src=bytes((10,42,0,source));dst=bytes((10,42,0,dest))
    udp=struct.pack("!HHHH",3999+source,3999+dest,8+len(payload),0)+payload
    check=checksum(src+dst+struct.pack("!BBH",0,17,len(udp))+udp) or 65535
    udp=udp[:6]+struct.pack("!H",check)+payload
    ip=struct.pack("!BBHHHBBH4s4s",0x45,0,20+len(udp),identifier,0x4000,64,17,0,src,dst)
    ip=ip[:10]+struct.pack("!H",checksum(ip))+ip[12:]
    return (bytes((2,0,0,0,0,dest,2,0,0,0,0,source,8,0))+ip+udp).ljust(60,b"\0")

def parse(raw):
    if type(raw) is not bytes or not 24<=len(raw)<=8192 or raw[:24]!=HEADER:
        raise ValueError("exact bounded Ethernet PCAP 2.4 little-endian profile")
    at=24;frames=[];previous=None
    while at<len(raw):
        if len(frames)>=16 or len(raw)-at<16:raise ValueError("capture count/framing")
        sec,usec,captured,original=struct.unpack_from("<IIII",raw,at);at+=16
        stamp=sec*1000000+usec
        if usec>=1000000 or (previous is not None and stamp<previous):
            raise ValueError("monotonic capture timestamps")
        if captured!=original or not 1<=captured<=554 or len(raw)-at<captured:
            raise ValueError("complete bounded packet, never a clipped capture")
        previous=stamp;frames.append(raw[at:at+captured]);at+=captured
    return frames

def validate(captures,left,right):
    if type(captures) is not dict or set(captures)!={"a","b"}:
        raise ValueError("two independent captures")
    for text in (left,right):
        if type(text) is not str or len(text)!=32 or any(c not in "abcdefghijklmnop" for c in text):
            raise ValueError("fresh fixed challenge shape")
    if left==right:raise ValueError("distinct challenges")
    wanted=[packet("a",left.encode("ascii")),packet("b",right.encode("ascii"))]
    for peer in ("a","b"):
        if parse(captures[peer])!=wanted:
            raise ValueError("actual full wire bytes/order/count differ from independent oracle")
    return dict(packets_per_peer=2,wire_bytes_per_peer=sum(map(len,wanted)),
                capture_sha256={p:hashlib.sha256(captures[p]).hexdigest() for p in ("a","b")},
                post_close_transmissions=0)

def self_test():
    assert checksum(bytes.fromhex("0001f203f4f5f6f7"))==0x220d
    a,b="a"*32,"b"*32
    frames=[packet("a",a.encode()),packet("b",b.encode())]
    def encode(values):
        return HEADER+b"".join(struct.pack("<IIII",1,n,len(x),len(x))+x for n,x in enumerate(values))
    raw=encode(frames);assert len(raw)==204
    proof=validate({"a":raw,"b":raw},a,b);assert proof["wire_bytes_per_peer"]==148
    count=0
    def reject(fn):
        nonlocal count
        try:fn()
        except ValueError:count+=1
        else:raise AssertionError("bad wire evidence accepted")
    for i in range(len(raw)):
        changed=bytearray(raw);changed[i]^=128
        # Capture timestamps may change validly; packet/header/length must not.
        if 24<=i<32 or 114<=i<122:continue
        reject(lambda changed=changed:validate({"a":bytes(changed),"b":raw},a,b))
    for size in range(len(raw)):
        reject(lambda size=size:validate({"a":raw[:size],"b":raw},a,b))
    for changed in (encode([]),encode(frames[::-1]),encode(frames*2),
        encode(frames+[frames[0]]),raw+b"\0",HEADER+struct.pack("<IIII",1,1000000,74,74)+frames[0]):
        reject(lambda changed=changed:validate({"a":changed,"b":raw},a,b))
    reject(lambda:validate({"a":raw,"b":raw},b,a))
    reject(lambda:validate({"a":raw,"b":raw},a,a))
    for peer in ("a","b"):
        for size in (1,17,511,512):
            frame=packet(peer,bytes((i%256 for i in range(size))))
            assert 60<=len(frame)<=554 and checksum(frame[14:34])==0
    return count

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure wire tests only")
    print("Expansion captured wire:",self_test(),"negative byte fixtures")
